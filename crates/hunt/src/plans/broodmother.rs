//! The Broodmother's hunter: the plan a decent player follows, the two plans
//! that each ignore half the fight, and the report lines the fight is
//! measured by.
//!
//! `docs/design/creatures/broodmother.md` §9, in order:
//!
//! 1. **Stand between her third and fourth leg**, never at a foot, never under
//!    her.
//! 2. **Kill a broodling that comes to you; never walk to one.**
//! 3. **Watch the sacs; know which is reddest.**
//! 4. **On the screech, go to the abdomen's edge** on the reddest sac's side,
//!    wait out the crash, step in, pop the lowest-health sac first, step out
//!    before the lift.
//! 5. **After a web shot's windup, out of its landing; caught, attack to cut
//!    free.**
//! 6. **Strands: crouch.**
//! 7. **Enraged: punish the slam's recovery at the pedicel**, and the rest of
//!    her openings at whatever is near.
//!
//! The balanced plan also breaks legs when one is near in an opening (§1
//! step 5 of the loop) and, playing the Elementalist, shoots the reddest sac
//! from range whenever nothing is on her -- the fight's one free pop (§7).
//! **The ablations** (§9, "the dilemma is measured"): [`brood_only`] never
//! touches a sac or her while a broodling stands, and [`mother_only`] never
//! swings at a broodling or a sac. Both dodge what she throws exactly as the
//! balanced plan does, so the difference between them is the target, and
//! nothing else.
//!
//! Like every plan it sees the world `REACTION` frames late and never reads
//! the creature's mind: it reads her body, the floor and the signs her moves
//! draw -- the discs, the lanes, the rings under the red sacs, the slam's
//! footprint through the screech -- which a person sees too.

use sim::critter::{MAX_CRITTERS, flag, is};
use sim::fixed::Fx;
use sim::math::{atan2_turns, flat_segment_gap, wide_flat_dist};
use sim::monster::{Doing, Monster};
use sim::species::broodmother::{self as bm, Knob, fight};
use sim::state::{MAX_PLAYERS, Phase, Player};
use sim::{Input, V3, World};

use crate::report::Tally;
use crate::{Hands, Intent, Plan, QUARTER, REACTION, steer, turns_to_aim};

/// Hold the post between her legs.
pub const POST: Intent = Intent("Post");
/// Swinging at a broodling.
pub const BROOD: Intent = Intent("Brood");
/// At a sac.
pub const SAC: Intent = Intent("Sac");
/// At her.
pub const MOTHER: Intent = Intent("Mother");
/// Out of what is coming.
pub const EVADE: Intent = Intent("Evade");
/// To the abdomen's edge, for the slam.
pub const EDGE: Intent = Intent("Edge");
/// Cutting free of a glob.
pub const CUT: Intent = Intent("Cut");

/// Which half of the fight a plan hits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Aims {
    /// The balanced plan: brood that come, sacs in the window, legs and her
    /// in her openings.
    Both,
    /// Brood only, until there are none: then as the balanced plan.
    Brood,
    /// Her only: never a broodling, never a sac.
    Mother,
}

/// How many frames early or late its dodges can be.
const SLOP_EARLY: i32 = 4;
const SLOP_LATE: i32 = 3;
/// The dodge goes this many frames before the hit.
const DODGE_LEAD: i32 = 3;
/// How far past a marker's edge counts as out of it.
const MARGIN: Fx = Fx::ratio(8, 10);
/// Frames between swings: a person does not mash.
const SWING_GAP: u16 = 10;
/// Close enough to its post to stop walking to it.
const SETTLED: Fx = Fx::ratio(7, 10);
/// The post, in her own frame: between the third and fourth legs, beside
/// the abdomen, outside every foot's disc -- the Ridgeback's "beside a hind
/// leg" (§3).
const POST_ALONG: Fx = Fx::ratio(-24, 10);
const POST_OUT: Fx = Fx::ratio(36, 10);
/// How far outside the slam's footprint it waits for the crash.
const EDGE_OUT: Fx = Fx::ratio(5, 10);
/// Frames of the slam's recovery it keeps back to get out before the lift.
const LIFT_EXIT: i32 = 20;
/// A broodling this near with its tail up or crouched is the one to hit.
const BROOD_NEAR: Fx = Fx::ratio(45, 10);
/// Frames a walk may fail to move it before it sidesteps, and how long the
/// sidestep lasts.
const STUCK: u16 = 6;
const DETOUR: u16 = 24;
/// One frame of what it saw: the whole world, as it was on screen.
#[derive(Clone)]
struct Seen {
    world: World,
}

pub struct Broodmother {
    who: usize,
    aims: Aims,
    memory: Vec<Option<Seen>>,
    at: usize,
    filled: usize,
    intent: Intent,
    cooldown: u16,
    dodge_left: u16,
    slop: i32,
    rng: u32,
    /// Which side of her it holds, in her frame: `-1` her left, `+1` her
    /// right. Chosen from where it is, and kept until she turns it round.
    side: i32,
    /// Where it stood last frame, how many frames running a walk has not
    /// moved it, and the sidestep it is taking round whatever stopped it.
    last: V3,
    stuck: u16,
    detour_left: u16,
    /// Its class (`crate::class`), and what this frame's choice was, for
    /// it: the point a swing was aimed at and the window it was thrown in,
    /// the way a dodge went, or where it waits.
    hands: Hands,
    aimed: Option<(V3, Option<i32>)>,
    out: Option<V3>,
    waiting: Option<(V3, V3, i32)>,
}

impl Broodmother {
    pub fn new(who: usize, seed: u32, aims: Aims) -> Broodmother {
        Broodmother {
            who,
            aims,
            memory: vec![None; REACTION + 1],
            at: 0,
            filled: 0,
            intent: POST,
            cooldown: 0,
            dodge_left: 0,
            slop: 0,
            rng: (0x9E37_79B9
                ^ (who as u32).wrapping_mul(0x85EB_CA6B)
                ^ seed.wrapping_mul(0x27D4_EB2F))
                | 1,
            side: 1,
            last: V3::ZERO,
            stuck: 0,
            detour_left: 0,
            hands: Hands::new(who, seed),
            aimed: None,
            out: None,
            waiting: None,
        }
    }

    /// **Round what stops it**: a walk that has not moved it for a few
    /// frames -- a shelf, a pillar, a leg -- bears an eighth to its left for
    /// a moment. What a person does without
    /// thinking, and without it the plan stands against a shelf all window.
    fn unstick(&mut self, me: &Player, input: Input) -> Input {
        const WALK: u16 = Input::W | Input::A | Input::S | Input::D;
        let walking = input.bits & WALK != 0;
        let moved = wide_flat_dist(me.pos, self.last).raw() > Fx::ratio(1, 100).raw();
        self.last = me.pos;
        self.stuck = if walking && !moved && me.action.actionable() {
            self.stuck + 1
        } else {
            0
        };
        if self.stuck >= STUCK {
            self.stuck = 0;
            self.detour_left = DETOUR;
        }
        if self.detour_left == 0 || !walking {
            return input;
        }
        self.detour_left -= 1;
        let b = input.bits;
        let turned = (if b & Input::W != 0 { Input::A } else { 0 })
            | (if b & Input::A != 0 { Input::S } else { 0 })
            | (if b & Input::S != 0 { Input::D } else { 0 })
            | (if b & Input::D != 0 { Input::W } else { 0 });
        Input {
            bits: (b & !WALK) | turned | (b & Input::W),
            ..input
        }
    }

    fn recall(&self) -> Option<World> {
        let back = REACTION.min(self.filled.saturating_sub(1));
        let idx = (self.at + self.memory.len() - 1 - back) % self.memory.len();
        self.memory[idx].as_ref().map(|s| s.world.clone())
    }

    fn roll(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }

    fn roll_slop(&mut self) {
        let span = (SLOP_LATE + SLOP_EARLY + 1) as u32;
        self.slop = (self.roll() % span) as i32 - SLOP_LATE;
    }
}

/// The balanced plan.
pub fn balanced(who: usize, seed: u32, _hop: Fx) -> Box<dyn Plan + Send + Sync> {
    Box::new(Broodmother::new(who, seed, Aims::Both))
}

/// The first ablation: the brood, and nothing else while any stand.
pub fn brood_only(who: usize, seed: u32, _hop: Fx) -> Box<dyn Plan + Send + Sync> {
    Box::new(Broodmother::new(who, seed, Aims::Brood))
}

/// The second ablation: her, and never a broodling or a sac.
pub fn mother_only(who: usize, seed: u32, _hop: Fx) -> Box<dyn Plan + Send + Sync> {
    Box::new(Broodmother::new(who, seed, Aims::Mother))
}

// ---------------------------------------------------------------------------
// Reading her
// ---------------------------------------------------------------------------

fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

fn unit(v: V3) -> V3 {
    let v = flat(v);
    if v.flat_len().raw() > 0 {
        v.normalized()
    } else {
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
    }
}

/// A point in her own frame -- `along` toward her head, `out` to her right --
/// on the floor.
fn body_point(m: &Monster, along: Fx, out: Fx) -> V3 {
    let f = V3::from_turns(m.yaw);
    let r = V3::from_turns(m.yaw.add(QUARTER));
    flat(m.pos).add(f.scale(along)).add(r.scale(out))
}

/// Which side of her a point is on: `-1` her left, `+1` her right.
fn side_of(m: &Monster, at: V3) -> i32 {
    let r = V3::from_turns(m.yaw.add(QUARTER));
    if flat(at.sub(m.pos)).dot(r).raw() >= 0 {
        1
    } else {
        -1
    }
}

/// Does a move's floor marker (her telegraph) cover a point, with a margin?
fn covers(m: &Monster, at: V3) -> bool {
    let Some(t) = m.telegraph() else {
        return false;
    };
    let r = t.radius.add(sim::tuning::body_radius()).add(MARGIN);
    if t.sweep.raw() > 0 {
        let end = t.anchor.add(t.along.scale(t.sweep));
        flat_segment_gap(at, t.anchor, end).raw() <= r.raw()
    } else {
        wide_flat_dist(at, t.anchor).raw() <= r.raw()
    }
}

/// Does a floor sign that warns of something cover a point?
fn signed(w: &World, at: V3) -> Option<sim::sign::Sign> {
    w.signs()
        .iter()
        .find(|s| matches!(s.says, sim::sign::Says::Coming | sim::sign::Says::Live) && s.covers(at))
        .copied()
}

/// Frames until her move under way lands, in the present.
fn to_contact(m: &Monster) -> i32 {
    let Some(kind) = m.doing.attacking() else {
        return i32::MAX;
    };
    let a = m.sp().attack(kind);
    (match m.doing {
        Doing::Startup { left, .. } => left as i32 + 1,
        Doing::Active { left, .. } => -((a.active as i32) - left as i32),
        _ => i32::MAX / 2,
    }) - REACTION as i32
}

/// Which way out of her marker.
fn way_out(m: &Monster, me: V3) -> V3 {
    let Some(t) = m.telegraph() else {
        return unit(me.sub(m.pos));
    };
    if t.sweep.raw() > 0 {
        let across = V3::from_turns(atan2_turns(t.along.z, t.along.x).add(QUARTER));
        if flat(me.sub(t.anchor)).dot(across).raw() >= 0 {
            across
        } else {
            across.scale(Fx::ONE.neg())
        }
    } else {
        let away = flat(me.sub(t.anchor));
        if away.flat_len().raw() > 0 {
            away.normalized()
        } else {
            unit(me.sub(m.pos))
        }
    }
}

/// How far to walk to be out of her marker.
fn room_out(m: &Monster, me: V3) -> Fx {
    let Some(t) = m.telegraph() else {
        return Fx::ZERO;
    };
    let r = t.radius.add(sim::tuning::body_radius()).add(MARGIN);
    let d = if t.sweep.raw() > 0 {
        let end = t.anchor.add(t.along.scale(t.sweep));
        flat_segment_gap(me, t.anchor, end)
    } else {
        wide_flat_dist(me, t.anchor)
    };
    r.sub(d).max(Fx::ZERO)
}

/// Where the slam will come down, as the screech's footprint shows it: the
/// hit test's own answer.
fn crash_point(m: &Monster) -> Option<(V3, Fx)> {
    let mut crash = *m;
    crash.doing = Doing::Active {
        kind: bm::SLAM,
        left: m.sp().attack(bm::SLAM).active,
    };
    crash.hit_volume().map(|(at, r, _, _)| (flat(at), r))
}

/// The living sacs, with their world middles and health.
fn sacs(m: &Monster) -> impl Iterator<Item = (usize, V3, i32)> + '_ {
    (0..bm::SAC_COUNT).filter_map(|i| {
        let part = bm::sac_part(i);
        let rig = m.rig();
        if !rig.there(part) {
            return None;
        }
        let slot = m.sp().break_slot(part)?;
        Some((i, fight::sac_middle(m, i), m.breaks[slot]))
    })
}

/// Which sac to go for: the nearest walk, a metre added for every hundred of
/// health it still has -- a sac already cut is worth a longer walk.
fn sac_cost(m: &Monster, me: V3, sac: &(usize, V3, i32)) -> i32 {
    let walk = wide_flat_dist(beside_sac(m, sac.1, me), me);
    walk.add(Fx::from_int(sac.2).div(Fx::from_int(100))).raw()
}

/// Where to stand to swing at a sac lying on the floor with the abdomen: the
/// point beside the abdomen nearest it, on the hunter's side, a body off its
/// edge.
fn beside_sac(m: &Monster, sac: V3, me: V3) -> V3 {
    let f = V3::from_turns(m.yaw);
    let r = V3::from_turns(m.yaw.add(QUARTER));
    let rel = flat(sac.sub(m.pos));
    let along = rel.dot(f);
    let side = if flat(me.sub(m.pos)).dot(r).raw() >= 0 {
        Fx::ONE
    } else {
        Fx::ONE.neg()
    };
    let half = bm::SPECIES.shape(bm::ABDOMEN_PART).max.z;
    let out = half
        .add(sim::tuning::body_radius())
        .add(Fx::ratio(1, 10))
        .mul(side);
    flat(m.pos).add(f.scale(along)).add(r.scale(out))
}

/// The way from `from` to `to` that keeps outside the circle round `at`:
/// straight there if the line clears it, along the circle's tangent the
/// short way round if not, and straight out if already in it.
fn around(from: V3, at: V3, radius: Fx, to: V3) -> V3 {
    let rel = flat(from.sub(at));
    let d = rel.flat_len();
    let want = flat(to.sub(from));
    if want.flat_len().raw() <= SETTLED.raw() {
        return V3::ZERO;
    }
    if d.raw() < radius.sub(Fx::ratio(1, 10)).raw() {
        return unit(rel);
    }
    // Does the straight line pass inside the circle?
    let dir = unit(want);
    let along = rel.scale(Fx::ONE.neg()).dot(dir);
    let len = want.flat_len();
    let closest = if along.raw() <= 0 {
        from
    } else if along.raw() >= len.raw() {
        to
    } else {
        from.add(dir.scale(along))
    };
    if wide_flat_dist(closest, at).raw() >= radius.raw() {
        return dir;
    }
    // Along the tangent from here to the circle, on the side `to` is: the
    // line to the centre turned by the angle whose sine is radius over
    // distance.
    let inward = unit(rel).scale(Fx::ONE.neg());
    let left = V3::new(inward.z.neg(), Fx::ZERO, inward.x);
    let side = if left.dot(want).raw() >= 0 {
        left
    } else {
        left.scale(Fx::ONE.neg())
    };
    let sin = radius.div(d).min(Fx::ONE);
    let cos = Fx::ONE.sub(sin.mul(sin)).max(Fx::ZERO).sqrt();
    unit(inward.scale(cos).add(side.scale(sin)))
}

/// Straight out from her flank from `beside` until clear of a circle round
/// `at` of radius `clear`: where to wait for the crash so the walk in to the
/// sac is the shortest one the footprint allows.
fn off_the_flank(m: &Monster, beside: V3, at: V3, clear: Fx) -> V3 {
    let r = V3::from_turns(m.yaw.add(QUARTER));
    let side = if flat(beside.sub(m.pos)).dot(r).raw() >= 0 {
        r
    } else {
        r.scale(Fx::ONE.neg())
    };
    // |beside + side t - at| = clear: t = -b + sqrt(b^2 - c).
    let rel = flat(beside.sub(at));
    let b = rel.dot(side);
    let c = rel.dot(rel).sub(clear.mul(clear));
    if c.raw() <= 0 || b.raw() < 0 {
        let disc = b.mul(b).sub(c).max(Fx::ZERO);
        let t = b.neg().add(disc.sqrt()).max(Fx::ZERO);
        return flat(beside).add(side.scale(t));
    }
    flat(beside)
}

/// The point on a part's box nearest a world point, at chest height or the
/// box's own, for a crosshair: the middle of the part's face toward it.
fn part_point(m: &Monster, part: usize, from: V3) -> V3 {
    let rig = m.rig();
    let sh = m.sp().shape(part);
    let local = rig.world_to_part(part, from);
    let clamp = V3::new(
        local.x.clamp(sh.min.x, sh.max.x),
        local.y.clamp(sh.min.y, sh.max.y),
        local.z.clamp(sh.min.z, sh.max.z),
    );
    rig.part_to_world(part, clamp)
}

/// Look at a point, with the crosshair on it, pressing `bits`.
fn looking(me: &Player, at: V3, bits: u16) -> Input {
    let d = flat(at.sub(me.pos));
    let yaw = atan2_turns(d.z, d.x);
    let wire = turns_to_aim(yaw.sub(me.carry_yaw));
    let pitch = sim::aim::look_onto_closely(me.pos, wire, me.aloft, at);
    Input::looking_at(bits, wire, pitch)
}

/// The auto's button: the Dual mage throws the hand whose bar is lower (the
/// Gnawers' hunter's rule, and why).
fn auto_button(me: &Player) -> u16 {
    match sim::dual::bars(me) {
        Some((dark, light)) if dark.raw() < light.raw() => Input::LEFT,
        Some(_) => Input::RIGHT,
        None => Input::LEFT,
    }
}

/// The auto's move, for its reach: the Blood mage's is her sweep.
fn auto_slot(class: sim::Class) -> u8 {
    match class {
        sim::Class::BloodMage => sim::moves::blood::SWEEP,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// One frame
// ---------------------------------------------------------------------------

impl Plan for Broodmother {
    fn watch(&mut self, w: &World) {
        self.memory[self.at] = Some(Seen { world: w.clone() });
        self.at = (self.at + 1) % self.memory.len();
        self.filled = (self.filled + 1).min(self.memory.len());
    }

    fn act(&mut self, w: &World) -> Input {
        self.aimed = None;
        self.out = None;
        self.waiting = None;
        let input = self.choose(w);
        let me = w.players[self.who];
        let input = self.unstick(&me, input);
        // The class's turn: what the choice was for, in its own hands.
        const ATTACKS: u16 =
            Input::LEFT | Input::RIGHT | Input::MIDDLE | Input::SPECIAL | Input::MECHANIC;
        if input.bits & ATTACKS != 0
            && let Some((at, window)) = self.aimed
        {
            return self.hands.hit(w, &me, at, input, window);
        }
        if input.bits & Input::SHIFT != 0
            && let Some(out) = self.out
        {
            return self.hands.leave(w, &me, out, input);
        }
        // Walking in on a window: the class's own way in, if it has one.
        if let Some((at, Some(window))) = self.aimed
            && wide_flat_dist(at, me.pos).raw() > self.hands.reach(&me).add(Fx::ONE).raw()
            && let Some(go) = self.hands.close_in(w, &me, at, window)
        {
            return go;
        }
        if let Some((beast, at, safe)) = self.waiting
            && let Some(own) = self.hands.idle(w, &me, beast, at, safe)
        {
            return own;
        }
        input
    }

    fn intent(&self) -> Intent {
        self.intent
    }

    fn hands(&mut self) -> Option<&mut Hands> {
        Some(&mut self.hands)
    }

    fn hands_ref(&self) -> Option<&Hands> {
        Some(&self.hands)
    }
}

impl Broodmother {
    fn choose(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        let me = w.players[self.who];
        let Some(seen) = self.recall() else {
            return Input::default();
        };
        let Some(m) = seen.monsters[0] else {
            return Input::default();
        };
        if me.health <= 0 || !m.alive() || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        let free = me.action.actionable();
        let poke = sim::moves::get(me.class, auto_slot(me.class));
        let reach = poke.reach.max(Fx::ONE);
        let brood_alive = (0..MAX_CRITTERS)
            .filter(|i| seen.critters[*i].alive())
            .count();
        // What it is after this frame, by plan.
        let aims = match self.aims {
            Aims::Brood if brood_alive == 0 => Aims::Both,
            a => a,
        };
        let hits_brood = aims != Aims::Mother;
        let hits_sacs = aims == Aims::Both;
        let hits_her = aims != Aims::Brood;

        // A new windup: a new slop on its timing.
        if let Doing::Startup { kind, left } = m.doing {
            if left == m.sp().attack(kind).startup {
                self.roll_slop();
            }
        }

        // 5b. **Caught: cut free.** A glob under the feet is felt at once;
        // every attack cuts it shorter, so swing -- at a broodling if one is
        // near, at her if not.
        let caught = sim::hazard::all(&w.lore)
            .any(|(_, h)| h.index() == fight::GLOB && h.state == self.who as u8 + 1);
        if caught {
            self.intent = CUT;
            let near = nearest_brood(&seen, me.pos, BROOD_NEAR);
            let at = near.map_or(part_point(&m, bm::THORAX, me.pos), |(_, mid)| mid);
            let bits = if free && self.cooldown == 0 {
                self.cooldown = SWING_GAP / 2;
                auto_button(&me)
            } else {
                0
            };
            return looking(&me, at, bits);
        }

        // Which side it holds: the one it is on, kept while she stands still.
        let my_side = side_of(&m, me.pos);
        if m.doing.free() {
            self.side = my_side;
        }

        // 4. **The screech, and the slam it leads to.** To the edge of the
        // footprint drawn through the scream, on the reddest sac's side; out
        // before the crash; in, as it lies there; out before the lift.
        let slamming = matches!(
            m.doing,
            Doing::Startup {
                kind: bm::SCREECH | bm::SLAM,
                ..
            } | Doing::Active {
                kind: bm::SCREECH | bm::SLAM,
                ..
            } | Doing::Recovery {
                kind: bm::SCREECH,
                ..
            }
        );
        // The crash, timed off the tell: once it has come down -- by the
        // count from what it saw -- in to the sac at once, as it will lie.
        let landed = matches!(
            m.doing,
            Doing::Startup { kind: bm::SLAM, .. } | Doing::Active { kind: bm::SLAM, .. }
        ) && to_contact(&m) - self.slop < 0;
        if landed && hits_sacs && !fight::enraged(&m) {
            let mut down = m;
            down.doing = Doing::Recovery {
                kind: bm::SLAM,
                left: m.sp().attack(bm::SLAM).recovery,
            };
            if let Some((i, mid, _)) = sacs(&down).min_by_key(|s| sac_cost(&down, me.pos, s)) {
                let face = part_point(&down, bm::sac_part(i), me.pos);
                let stand = beside_sac(&down, mid, me.pos);
                return self.pop(&down, &me, stand, mid, face, reach);
            }
        }
        if slamming && !landed {
            if let Some((at, r)) = crash_point(&m) {
                // Where the sacs will lie: her, posed half a second into the
                // crash -- what the footprint drawn through the scream shows.
                let mut down = m;
                down.doing = Doing::Recovery {
                    kind: bm::SLAM,
                    left: m.sp().attack(bm::SLAM).recovery.saturating_sub(30),
                };
                let target = (0..bm::SAC_COUNT)
                    .filter(|i| m.own[fight::body::SACS] & (1 << i) == 0)
                    .max_by_key(|i| fight::ripeness(&seen, *i).raw())
                    .map(|i| fight::sac_middle(&down, i));
                let beside = target.map_or(me.pos, |s| beside_sac(&down, s, me.pos));
                let edge = off_the_flank(
                    &down,
                    beside,
                    at,
                    r.add(EDGE_OUT).add(sim::tuning::body_radius()),
                );
                let inside =
                    wide_flat_dist(me.pos, at).raw() <= r.add(sim::tuning::body_radius()).raw();
                let contact = to_contact(&m) - self.slop;
                // The bare slam's tell is short: a dodge out if it is close.
                if inside
                    && matches!(m.doing, Doing::Startup { kind: bm::SLAM, .. })
                    && contact <= DODGE_LEAD
                    && self.dodge_left == 0
                    && free
                {
                    self.intent = EVADE;
                    self.dodge_left = sim::tuning::dodge_frames();
                    let away = unit(me.pos.sub(at));
                    self.out = Some(away);
                    return face_walk(&me, &m, away, Input::SHIFT);
                }
                self.intent = EDGE;
                let clear = r.add(sim::tuning::body_radius());
                if inside {
                    // Out, leaning toward the edge it means to wait at.
                    let out = unit(me.pos.sub(at)).scale(Fx::from_int(2));
                    let way = unit(out.add(unit(edge.sub(me.pos))));
                    return face_walk(&me, &m, way, 0);
                }
                // At the edge, brood that come are still hit.
                if let Some(input) = self.swing_brood(&seen, &me, reach, hits_brood, true) {
                    return input;
                }
                return face_walk(&me, &m, around(me.pos, at, clear.add(EDGE_OUT), edge), 0);
            }
        }

        // 1. **What is coming, and whether it is coming here.**
        if let Some(kind) = m.doing.attacking() {
            let active_or_winding = matches!(m.doing, Doing::Startup { .. } | Doing::Active { .. });
            if active_or_winding && covers(&m, me.pos) && kind != bm::SCREECH {
                let contact = to_contact(&m) - self.slop;
                let out = way_out(&m, me.pos);
                let speed = sim::tuning::move_speed().mul(if me.slowed > 0 {
                    me.slow_mul
                } else {
                    Fx::ONE
                });
                let walkable = Fx::from_int(contact.max(0)).mul(speed).mul(sim::DT).raw()
                    > room_out(&m, me.pos).raw();
                self.intent = EVADE;
                // The Bulwark takes a blockable blow on the shield.
                let a = m.sp().attack(kind);
                let shield = me.shield().is_some_and(|s| s.in_hand())
                    && a.damage > 0
                    && !a.unblockable
                    && !walkable
                    && contact > -(a.active as i32);
                if shield {
                    return looking(&me, part_point(&m, bm::THORAX, me.pos), Input::RIGHT);
                }
                if !walkable && contact <= DODGE_LEAD && self.dodge_left == 0 && free {
                    self.dodge_left = sim::tuning::dodge_frames();
                    self.out = Some(out);
                    return face_walk(&me, &m, out, Input::SHIFT);
                }
                return face_walk(&me, &m, out, 0);
            }
        }
        // A red sac's ring, or another warning drawn under it: step out.
        if let Some(sign) = signed(&seen, me.pos) {
            if sign.shape == sim::sign::Shape::Ring {
                self.intent = EVADE;
                return face_walk(&me, &m, unit(me.pos.sub(sign.at)), 0);
            }
        }

        // A broodling's lane drawn under it: off the line, sideways -- the
        // answer to a bite whose crouch is off the screen.
        for i in 0..MAX_CRITTERS {
            if !seen.critters[i].alive() {
                continue;
            }
            let Some(t) = sim::pack::telegraph(seen.pack.as_ref(), &seen.critters, i) else {
                continue;
            };
            let a = flat(t.anchor);
            let b = a.add(t.along.scale(t.sweep));
            let wide = t.radius.add(sim::tuning::body_radius());
            if sim::math::flat_segment_gap(me.pos, a, b).raw() <= wide.raw() {
                self.intent = EVADE;
                let across = V3::new(t.along.z.neg(), Fx::ZERO, t.along.x);
                let side = if flat(me.pos.sub(a)).dot(across).raw() >= 0 {
                    across
                } else {
                    across.scale(Fx::ONE.neg())
                };
                return face_walk(&me, &m, unit(side), 0);
            }
        }

        // 2. **Kill a broodling that comes to you** -- in the window, only
        // one already crouched at you: the sacs come first.
        let window = matches!(m.doing, Doing::Recovery { kind: bm::SLAM, left }
            if left as i32 - REACTION as i32 > LIFT_EXIT);
        if let Some(input) = self.swing_brood(&seen, &me, reach, hits_brood, !window) {
            return input;
        }

        // The slam's recovery: the window. In to the lowest-health sac, and
        // out before the lift.
        if let Doing::Recovery {
            kind: bm::SLAM,
            left,
        } = m.doing
        {
            let time = left as i32 - REACTION as i32;
            if time > LIFT_EXIT {
                if hits_sacs && !fight::enraged(&m) {
                    if let Some((i, mid, _)) = sacs(&m).min_by_key(|s| sac_cost(&m, me.pos, s)) {
                        let face = part_point(&m, bm::sac_part(i), me.pos);
                        let stand = beside_sac(&m, mid, me.pos);
                        return self.pop(&m, &me, stand, mid, face, reach);
                    }
                }
                if hits_her {
                    // Enraged, the waist; otherwise whatever of her is near.
                    let part = if fight::enraged(&m) {
                        bm::PEDICEL_PART
                    } else {
                        nearest_part(&m, me.pos)
                    };
                    let at = part_point(&m, part, me.pos);
                    return self.strike(&me, &m, flat(at), at, reach, MOTHER);
                }
            } else if covers_abdomen(&m, me.pos) {
                // Out before the lift.
                self.intent = EVADE;
                return face_walk(&me, &m, unit(me.pos.sub(m.pos)), 0);
            }
        }

        // 7, and the loop's step 5. **Her openings**: a recovery, a flinch, a
        // stumble, the collapse. A middle shin in reach first (two broken on
        // a side list her), the waist when she is down, else the nearest of
        // her.
        let opening = match m.doing {
            Doing::Recovery { left, .. }
            | Doing::Flinch { left }
            | Doing::Stumble { left, .. }
            | Doing::Toppled { left } => left as i32 > REACTION as i32 + 6,
            _ => false,
        };
        if opening && hits_her {
            let down = matches!(m.doing, Doing::Stumble { .. } | Doing::Toppled { .. });
            let shin = (0..bm::LEG_COUNT)
                .filter(|leg| bm::middle_leg(*leg) && !m.broken(bm::shin_part(*leg)))
                .map(|leg| (leg, part_point(&m, bm::shin_part(leg), me.pos)))
                .filter(|(_, at)| wide_flat_dist(*at, me.pos).raw() <= reach.add(Fx::ONE).raw())
                .min_by_key(|(_, at)| wide_flat_dist(*at, me.pos).raw());
            let (at, aimed) = match (down, shin, aims) {
                (true, _, _) => {
                    let p = part_point(&m, bm::PEDICEL_PART, me.pos);
                    (flat(p), p)
                }
                (_, Some((_, s)), Aims::Both) => (flat(s), V3::new(s.x, Fx::ONE, s.z)),
                _ => {
                    let part = nearest_part(&m, me.pos);
                    let p = part_point(&m, part, me.pos);
                    (flat(p), p)
                }
            };
            return self.strike(&me, &m, at, aimed, reach, MOTHER);
        }

        // 6. **Strands: crouch** while one is under the way to the post.
        // 1. **The post**, facing her.
        self.intent = POST;
        let post = body_point(&m, POST_ALONG, POST_OUT.mul(Fx::from_int(self.side)));
        let to = flat(post.sub(me.pos));
        let crouch = if strand_ahead(&seen, me.pos, post) {
            Input::CROUCH
        } else {
            0
        };
        if to.flat_len().raw() > SETTLED.raw() {
            return face_walk(&me, &m, unit(to), crouch);
        }
        let thorax = part_point(&m, bm::THORAX, me.pos);
        let safe = match m.doing {
            Doing::Startup { .. } | Doing::Active { .. } => to_contact(&m) - self.slop,
            _ => i32::MAX,
        };
        self.waiting = Some((m.pos, thorax, safe));
        looking(&me, thorax, 0)
    }

    /// **Swing at a broodling coming in**: one with its tail up or crouched,
    /// near; stepped toward only as far as its reach, never walked to.
    fn swing_brood(
        &mut self,
        seen: &World,
        me: &Player,
        reach: Fx,
        allowed: bool,
        tails: bool,
    ) -> Option<Input> {
        if !allowed {
            return None;
        }
        let sp = seen.critters.sp();
        let target = (0..MAX_CRITTERS)
            .map(|i| seen.critters[i])
            .filter(|c| c.alive())
            .filter(|c| (tails && c.has(flag::TOKEN)) || c.state == is::STARTUP)
            .filter(|c| wide_flat_dist(c.pos, me.pos).raw() <= BROOD_NEAR.raw())
            .min_by_key(|c| wide_flat_dist(c.pos, me.pos).raw())
            .or_else(|| {
                if !tails {
                    return None;
                }
                // Any broodling already in reach.
                (0..MAX_CRITTERS)
                    .map(|i| seen.critters[i])
                    .filter(|c| c.alive())
                    .filter(|c| {
                        wide_flat_dist(c.pos, me.pos).raw() <= reach.add(Fx::ratio(1, 2)).raw()
                    })
                    .min_by_key(|c| wide_flat_dist(c.pos, me.pos).raw())
            })?;
        self.intent = BROOD;
        let middle = target.body(sp).middle();
        let gap = wide_flat_dist(target.pos, me.pos);
        let walk = if gap.raw() > reach.add(Fx::ratio(1, 2)).raw() {
            let d = flat(target.pos.sub(me.pos));
            steer(atan2_turns(d.z, d.x), d)
        } else {
            0
        };
        let swing = if self.cooldown == 0 && me.action.actionable() && walk == 0 {
            self.cooldown = SWING_GAP + (self.roll() % 4) as u16;
            auto_button(me)
        } else {
            0
        };
        self.aimed = Some((middle, None));
        Some(looking(me, middle, walk | swing))
    }

    /// **A sac lying in the window**: walk to the abdomen's edge level with
    /// it, the crosshair on it the whole way, and swing only once its near
    /// face is in reach -- a swing from further meets the abdomen's flank
    /// instead.
    fn pop(&mut self, m: &Monster, me: &Player, stand: V3, sac: V3, face: V3, reach: Fx) -> Input {
        self.intent = SAC;
        // In her frame: if it is not yet out past the flank line, out to it
        // first, straight sideways -- the way along the abdomen's end or a
        // leg is the way it sticks.
        let f = V3::from_turns(m.yaw);
        let r = V3::from_turns(m.yaw.add(QUARTER));
        let mine = flat(me.pos.sub(m.pos));
        let theirs = flat(stand.sub(m.pos));
        let lat = theirs.dot(r);
        let my_lat = mine.dot(r);
        let sign = if lat.raw() >= 0 {
            Fx::ONE
        } else {
            Fx::ONE.neg()
        };
        let short = lat.abs().sub(Fx::ratio(15, 100));
        let abdomen = m.sp().shape(bm::ABDOMEN_PART);
        let tip = m.world_of(bm::ABDOMEN_PART, V3::new(abdomen.min.x, Fx::ZERO, Fx::ZERO));
        let tip = flat(tip.sub(m.pos)).dot(f);
        let behind =
            mine.dot(f).raw() < theirs.dot(f).raw() && mine.dot(f).raw() > tip.sub(Fx::ONE).raw();
        let stand = if my_lat.mul(sign).raw() < short.raw()
            && behind
            && mine.dot(f).sub(theirs.dot(f)).abs().raw() > Fx::ratio(5, 10).raw()
        {
            flat(m.pos)
                .add(f.scale(mine.dot(f)))
                .add(r.scale(lat.add(sign.mul(Fx::ratio(5, 10)))))
        } else {
            stand
        };
        let to = flat(stand.sub(me.pos));
        let far = flat(face.sub(me.pos)).flat_len();
        let look = flat(sac.sub(me.pos));
        let walk = if to.flat_len().raw() > Fx::ratio(15, 100).raw() {
            steer(atan2_turns(look.z, look.x), to)
        } else {
            0
        };
        // At the flank, as near as the body lets it: swing -- §1 rule 4 says
        // every class reaches every sac from there.
        let arrived = walk == 0 || wide_flat_dist(me.pos, self.last).raw() == 0;
        let swing = if self.cooldown == 0
            && me.action.actionable()
            && (far.raw() <= reach.add(Fx::ratio(1, 2)).raw() || arrived)
        {
            self.cooldown = SWING_GAP;
            auto_button(me)
        } else {
            0
        };
        let window = m.frames_until_free() as i32 - REACTION as i32;
        self.aimed = Some((sac, Some(window)));
        looking(me, sac, walk | swing)
    }

    /// Walk to `stand`, and swing at `aimed` once within reach of it.
    fn strike(
        &mut self,
        me: &Player,
        m: &Monster,
        stand: V3,
        aimed: V3,
        reach: Fx,
        intent: Intent,
    ) -> Input {
        self.intent = intent;
        let to = flat(stand.sub(me.pos));
        let look = flat(aimed.sub(me.pos));
        let far = look.flat_len();
        let walk = if to.flat_len().raw() > SETTLED.raw() && far.raw() > reach.raw() {
            steer(atan2_turns(look.z, look.x), to)
        } else {
            0
        };
        let swing = if self.cooldown == 0
            && me.action.actionable()
            && far.raw() <= reach.add(Fx::ratio(15, 10)).raw()
        {
            self.cooldown = SWING_GAP;
            auto_button(me)
        } else {
            0
        };
        let window = m.frames_until_free() as i32 - REACTION as i32;
        self.aimed = Some((aimed, Some(window)));
        looking(me, aimed, walk | swing)
    }
}

/// Walk along `dir`, facing her.
fn face_walk(me: &Player, m: &Monster, dir: V3, extra: u16) -> Input {
    let to = flat(m.pos.sub(me.pos));
    let yaw = atan2_turns(to.z, to.x);
    let wire = turns_to_aim(yaw.sub(me.carry_yaw));
    Input::aimed(steer(yaw, dir) | extra, wire)
}

/// The nearest broodling within `near`, and its middle.
fn nearest_brood(seen: &World, at: V3, near: Fx) -> Option<(usize, V3)> {
    let sp = seen.critters.sp();
    (0..MAX_CRITTERS)
        .filter(|i| seen.critters[*i].alive())
        .filter(|i| wide_flat_dist(seen.critters[*i].pos, at).raw() <= near.raw())
        .min_by_key(|i| wide_flat_dist(seen.critters[*i].pos, at).raw())
        .map(|i| (i, seen.critters[i].body(sp).middle()))
}

/// Her part nearest a point, of the ones a fighter on the floor swings at.
fn nearest_part(m: &Monster, at: V3) -> usize {
    let mut parts = vec![bm::THORAX, bm::HEAD_PART, bm::ABDOMEN_PART, bm::UNDERSIDE];
    parts.extend((0..bm::LEG_COUNT).map(bm::shin_part));
    parts
        .into_iter()
        .min_by_key(|p| {
            let q = part_point(m, *p, at);
            sim::math::big_len(q.sub(V3::new(at.x, Fx::ONE, at.z))).raw()
        })
        .unwrap_or(bm::THORAX)
}

/// Is a point on the floor under her abdomen?
fn covers_abdomen(m: &Monster, at: V3) -> bool {
    bm::mind::under_abdomen(m, at)
}

/// Does a strand lie across the straight way from `from` to `to`?
fn strand_ahead(seen: &World, from: V3, to: V3) -> bool {
    sim::hazard::all(&seen.lore)
        .filter(|(_, h)| h.index() == fight::STRAND)
        .any(|(_, h)| {
            let a = flat(h.centre());
            let b = V3::new(
                sim::lore::from_cm(h.to[0]),
                Fx::ZERO,
                sim::lore::from_cm(h.to[1]),
            );
            let step = unit(to.sub(from)).scale(Fx::from_int(2));
            fight::crosses(from, from.add(step), a, b, h.radius())
        })
}

// ---------------------------------------------------------------------------
// The report's own lines (§9)
// ---------------------------------------------------------------------------

/// **The Broodmother's report lines**: what the hunter's attacks were on,
/// and what the time converted to; the brood's count; the clock's side of
/// the fight; the window; the guard; the root; the strands; the arc.
#[derive(Default)]
pub struct Lines {
    pub frames: u32,
    /// Frames an attack was out, by what it was nearest: brood, sacs, her,
    /// nothing.
    pub on: [u32; 4],
    /// Damage dealt, by the same: brood health, sac health, her health.
    pub dealt: [i64; 3],
    pub brood_sum: u64,
    pub brood_peak: usize,
    pub at_cap: u32,
    pub popped: u32,
    pub burst: u32,
    pub held_frames: u32,
    pub windows: u32,
    pub pops_in_windows: u32,
    pub in_window: bool,
    pub window_pops: u32,
    /// Frames since a sac was last struck by a hunter.
    pub since_sac: u32,
    pub guarded: u32,
    pub bites: u32,
    pub rooted_frames: u32,
    pub rooted_hits: u32,
    pub lunges_on_rooted: u32,
    pub trips: u32,
    pub trips_late: u32,
    pub enraged_at: Option<u32>,
    pub clutches: u32,
    pub glob_hits: u32,
}

/// What a hunter's swing this frame is nearest: 0 brood, 1 a sac, 2 her, 3
/// nothing in reach.
fn aimed_at(w: &World, p: &Player) -> usize {
    let Some(hb) = sim::state::hitbox(p) else {
        return 3;
    };
    let sp = w.critters.sp();
    if w.critters
        .iter()
        .any(|c| c.alive() && c.body(sp).touched_by(&hb))
    {
        return 0;
    }
    let Some(m) = w.monsters[0] else { return 3 };
    let mid = hb.centre();
    let reach = hb.radius.add(Fx::ratio(1, 2));
    let rig = m.rig();
    let near = |part: usize| {
        rig.there(part)
            && sim::math::big_len(part_point(&m, part, mid).sub(mid)).raw() <= reach.raw()
    };
    if (0..bm::SAC_COUNT).any(|i| near(bm::sac_part(i))) {
        return 1;
    }
    if (0..bm::PART_COUNT).any(near) {
        return 2;
    }
    3
}

impl Tally for Lines {
    fn observe(&mut self, before: &World, after: &World) {
        let (Some(b), Some(a)) = (before.monsters[0], after.monsters[0]) else {
            return;
        };
        self.frames += 1;
        // What the attacks were on.
        for p in after.players.iter().filter(|p| p.health > 0) {
            if sim::state::hitbox(p).is_some() {
                self.on[aimed_at(after, p)] += 1;
            }
        }
        // What the time converted to.
        for (x, y) in before.critters.iter().zip(after.critters.iter()) {
            if x.alive() {
                self.dealt[0] += (x.health as i64 - y.health.max(0) as i64).max(0);
            }
        }
        for i in 0..bm::SAC_COUNT {
            if let Some(slot) = bm::SPECIES.break_slot(bm::sac_part(i)) {
                let lost = (b.breaks[slot] - a.breaks[slot]).max(0);
                if lost > 0 && b.own[fight::body::SACS] & (1 << i) == 0 {
                    self.dealt[1] += lost as i64;
                    self.since_sac = 0;
                }
            }
        }
        // Her own health, less what a pop took (that is the sacs').
        let pops = (0..bm::SAC_COUNT)
            .filter(|i| {
                fight::site(after, *i).0 == fight::sac::SCARRED
                    && fight::site(before, *i).0 != fight::sac::SCARRED
            })
            .count() as u32;
        let pop_damage = pops as i32 * Knob::PopDamage.raw();
        self.dealt[2] += ((b.health - a.health) - pop_damage).max(0) as i64;
        self.popped += pops;
        // The brood.
        let alive = fight::brood(after);
        self.brood_sum += alive as u64;
        self.brood_peak = self.brood_peak.max(alive);
        if alive >= Knob::BroodCap.raw() as usize {
            self.at_cap += 1;
        }
        // The clock.
        for i in 0..bm::SAC_COUNT {
            let (was, _) = fight::site(before, i);
            let (now, _) = fight::site(after, i);
            if now == fight::sac::EMPTY && was != fight::sac::EMPTY {
                self.burst += 1;
            }
            if now == fight::sac::HELD {
                self.held_frames += 1;
            }
        }
        // The window: the slam's recovery.
        let open = matches!(a.doing, Doing::Recovery { kind: bm::SLAM, .. });
        if open && !self.in_window {
            self.windows += 1;
            self.window_pops = 0;
        }
        if open || self.in_window {
            self.window_pops += pops;
        }
        if !open && self.in_window {
            self.pops_in_windows += self.window_pops;
        }
        self.in_window = open;
        // Bites, and which were guarded or on a rooted fighter.
        self.since_sac = self.since_sac.saturating_add(1);
        let rooted = |w: &World, who: usize| {
            sim::hazard::all(&w.lore)
                .any(|(_, h)| h.index() == fight::GLOB && h.state == who as u8 + 1)
        };
        for (x, y) in before.critters.iter().zip(after.critters.iter()) {
            let landed = y.state == is::ACTIVE
                && y.has(flag::HIT_USED)
                && !(x.state == is::ACTIVE && x.has(flag::HIT_USED));
            if landed {
                self.bites += 1;
                let who = (y.target as usize).min(MAX_PLAYERS - 1);
                if self.since_sac <= Knob::GuardFrames.raw().max(0) as u32 {
                    self.guarded += 1;
                }
                if rooted(before, who) {
                    self.rooted_hits += 1;
                }
            }
        }
        for who in 0..MAX_PLAYERS {
            if after.players[who].health > 0 && rooted(after, who) {
                self.rooted_frames += 1;
            }
            // Her hits on a rooted fighter: the lunge pressing the set-up.
            let hurt = after.players[who].health < before.players[who].health;
            if hurt && rooted(before, who) && b.doing.attacking() == Some(bm::LUNGE) {
                self.lunges_on_rooted += 1;
            }
            if hurt && a.doing.attacking() == Some(bm::WEB_SHOT) {
                self.glob_hits += 1;
            }
            // A trip: staggered on a strand without her move doing it.
            let tripped = matches!(
                after.players[who].action,
                sim::state::Action::HitStun { .. }
            ) && !matches!(
                before.players[who].action,
                sim::state::Action::HitStun { .. }
            ) && hurt
                && before.players[who].health - after.players[who].health == Knob::TripDamage.raw()
                && sim::hazard::all(&before.lore).any(|(_, h)| h.index() == fight::STRAND);
            if tripped {
                self.trips += 1;
                if after.frame > 3600 {
                    self.trips_late += 1;
                }
            }
        }
        // The arc.
        if fight::enraged(&a) && !fight::enraged(&b) {
            self.enraged_at = Some(after.frame);
        }
        let clutch = |w: &World| w.lore.word(fight::word::FLAGS) & fight::flag::CLUTCH != 0;
        if clutch(after) && !clutch(before) {
            self.clutches += 1;
        }
    }

    fn lines(&self) -> Vec<(String, String, String)> {
        let row = |name: &str, value: String, why: &str| (name.to_string(), value, why.to_string());
        let out: u32 = self.on.iter().sum::<u32>().max(1);
        let pct = |n: u32| n * 100 / out;
        let dealt: i64 = self.dealt.iter().sum::<i64>().max(1);
        let dpct = |n: i64| n * 100 / dealt;
        let frames = self.frames.max(1);
        vec![
            row(
                "time on target",
                format!(
                    "brood {}%  sacs {}%  mother {}%  nothing {}%",
                    pct(self.on[0]),
                    pct(self.on[1]),
                    pct(self.on[2]),
                    pct(self.on[3])
                ),
                "what its swings were nearest, frame by frame",
            ),
            row(
                "damage on target",
                format!(
                    "brood {}%  sacs {}%  mother {}%",
                    dpct(self.dealt[0]),
                    dpct(self.dealt[1]),
                    dpct(self.dealt[2])
                ),
                "whether the time converted",
            ),
            row(
                "brood alive",
                format!(
                    "mean {:.1}  peak {}  at cap {}%",
                    self.brood_sum as f32 / frames as f32,
                    self.brood_peak,
                    self.at_cap * 100 / frames
                ),
                "whether the count was held",
            ),
            row(
                "sacs",
                format!(
                    "popped {}  burst {}  held {}s",
                    self.popped,
                    self.burst,
                    self.held_frames / 60
                ),
                "the clock's side of the fight",
            ),
            row(
                "pops per slam window",
                format!(
                    "{} in {} windows",
                    self.pops_in_windows + if self.in_window { self.window_pops } else { 0 },
                    self.windows
                ),
                "whether the window is usable",
            ),
            row(
                "guarded hits",
                format!("{} of {} bites", self.guarded, self.bites),
                "bites within the guard of a sac struck",
            ),
            row(
                "rooted, then",
                format!(
                    "{}s rooted, {} bites, {} lunges, {} globs",
                    self.rooted_frames / 60,
                    self.rooted_hits,
                    self.lunges_on_rooted,
                    self.glob_hits
                ),
                "whether the set-up works",
            ),
            row(
                "strand trips",
                format!(
                    "{} ({} after the first minute)",
                    self.trips, self.trips_late
                ),
                "whether the strands are seen",
            ),
            row(
                "enraged at",
                self.enraged_at
                    .map_or("--".to_string(), |f| format!("{:.0}s", f as f32 / 60.0)),
                "the arc",
            ),
            row(
                "clutches",
                format!("{}", self.clutches),
                "the arc's punishment",
            ),
        ]
    }
}

pub const CARD_PLAIN: crate::plans::Card = crate::plans::Card {
    species: sim::species::SpeciesId::BROODMOTHER,
    plan: balanced,
    bucks: |_| false,
    words: crate::plans::Words {
        weak_hits: "weak-point hits",
        broken: "parts broken (sacs and shins)",
        into_breakables: "damage into sacs and shins",
        into_breakables_why: "what the fight is for: the clock and the list",
        worst: "worst part",
        ride_for: "nothing to ride",
        toppled_pool: "off a pool under her collapse",
    },
    tally: Some(|| Box::new(Lines::default())),
    gamble: None,
};

pub static CARD: crate::plans::Card = CARD_PLAIN;

/// The first ablation's card: the same report, the brood-only plan.
pub static BROOD_ONLY: crate::plans::Card = crate::plans::Card {
    plan: brood_only,
    ..CARD_PLAIN
};

/// The second ablation's card: the same report, the mother-only plan.
pub static MOTHER_ONLY: crate::plans::Card = crate::plans::Card {
    plan: mother_only,
    ..CARD_PLAIN
};
