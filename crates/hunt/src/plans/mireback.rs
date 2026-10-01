//! The Mireback's hunter: the plan a decent player follows, and the report
//! lines the fight is measured by.
//!
//! **Stay on clean floor at its flank; walk out of the spew's marker; dodge
//! the crash; kick a brazier when its tar is joined to the toad; leave the tar
//! before a belch and hit the sac; light a wallow or break it; climb a mound
//! when one is inside your hop to the rim, walk to the crown and burst a wart;
//! never be swallowed.** `docs/design/creatures/mireback.md` §9, in order.
//! A second plan, [`greedy`], lets the tongue land once the toad is under a
//! quarter, so the report can say whether the gamble pays.
//!
//! Like every plan it sees the world `REACTION` frames late and never reads
//! the creature's mind: it reads the body, the floor (the hazard list, which
//! is what is drawn) and the markers the creature's moves draw -- the spew's
//! landing circle, the flop's crash and ring, the tongue's line -- which a
//! person sees too.

use sim::fixed::Fx;
use sim::hazard::{Floor, Placed};
use sim::math::{atan2_turns, wide_flat_dist, wrap_turns};
use sim::monster::{self, Doing, Monster};
use sim::species::mireback::{self, Knob, fight};
use sim::state::{Action, Phase};
use sim::{Input, V3, World};

use crate::report::{Report, Tally};
use crate::{HALF, Intent, Plan, QUARTER, REACTION, heavy, steer, turns_to_aim};

/// Hold at the flank, on clean floor.
pub const FLANK: Intent = Intent("Flank");
/// Close and hit it.
pub const PUNISH: Intent = Intent("Punish");
/// Get out of the way.
pub const EVADE: Intent = Intent("Evade");
/// Take it on the shield.
pub const GUARD: Intent = Intent("Guard");
/// Go and kick a brazier into its tar.
pub const KINDLE: Intent = Intent("Kindle");
/// Up a mound and onto its back.
pub const CLIMB: Intent = Intent("Climb");
/// Aboard: to a wart.
pub const TO_WART: Intent = Intent("ToWart");
/// Aboard: bursting it.
pub const WORK: Intent = Intent("Work");
/// Aboard: holding on through the swell.
pub const BRACE: Intent = Intent("Brace");
/// Aboard: off before the roll or the crash.
pub const LEAVE: Intent = Intent("Leave");
/// Inside it: hit the stomach.
pub const GUT: Intent = Intent("Gut");

/// How many frames early or late its dodges and jumps can be.
const SLOP_EARLY: i32 = 4;
const SLOP_LATE: i32 = 3;
/// The dodge goes this many frames before the hit.
const DODGE_LEAD: i32 = 3;
/// Frames of jump held: a full hop.
const LEAP_HOLD: u16 = 32;
/// The jump goes this many frames before a low sheet arrives.
const HOP_LEAD: i32 = 6;
/// Frames of jump held over a low sheet.
const HOP_HOLD: u16 = 14;
/// How far past a marker's edge counts as out of it.
const MARGIN: Fx = Fx::ratio(8, 10);
/// How far from its middle it waits: at the flank, outside its swell and
/// its tongue's cone, inside the spew's least range.
const STANDOFF: Fx = Fx::ratio(80, 10);
/// How far inside its own hop a surface has to be to count as a route.
const ROUTE_MARGIN: Fx = Fx::ratio(20, 100);
/// How far a mound's edge may be from the rim's for a jump between them.
const MOUND_GAP: Fx = Fx::ratio(25, 10);
/// Frames between swings.
const SWING_GAP: u16 = 24;
/// Swings in a string before it stops and backs off: the swell punishes
/// the third.
const STRING: u8 = 2;
/// Frames between the Elementalist's pillars: one burns for a while, and a
/// second while it does is a press that does nothing.
const FIRE_GAP: u16 = 90;
/// Frames of an opening kept back to get out in.
const EXIT: i32 = 10;
/// Do not bother correcting for less than this.
const SETTLED: Fx = Fx::ratio(6, 10);
/// Once committed to the brazier or a climb, keep at it this long.
const COMMIT: u16 = 600;

/// What the hunter can see, one frame of it.
#[derive(Clone, Copy)]
struct Seen {
    beast: Option<Monster>,
    floor: Floor,
    braziers: [(V3, bool); 4],
    count: usize,
}

impl Default for Seen {
    fn default() -> Seen {
        Seen {
            beast: None,
            floor: Floor::NONE,
            braziers: [(V3::ZERO, false); 4],
            count: 0,
        }
    }
}

pub struct Mireback {
    who: usize,
    memory: Vec<Seen>,
    at: usize,
    filled: usize,
    intent: Intent,
    cooldown: u16,
    dodge_left: u16,
    leap_left: u16,
    commit_left: u16,
    /// The brazier it is going to, if any.
    brazier: Option<usize>,
    slop: i32,
    rng: u32,
    hop: Fx,
    stirred: bool,
    /// Swings thrown in this opening.
    string: u8,
    was_open: bool,
    /// Lets the tongue land when the toad is low: [`greedy`].
    greed: bool,
    /// Frames before it will set off for a brazier again, after giving up.
    rest_left: u16,
    /// Frames before the Elementalist plants another pillar.
    fire_left: u16,
}

impl Mireback {
    pub fn new(who: usize, seed: u32, hop: Fx) -> Mireback {
        Mireback {
            who,
            memory: vec![Seen::default(); REACTION + 1],
            at: 0,
            filled: 0,
            intent: FLANK,
            cooldown: 0,
            dodge_left: 0,
            leap_left: 0,
            commit_left: 0,
            brazier: None,
            slop: 0,
            rng: (0x9E37_79B9
                ^ (who as u32).wrapping_mul(0x85EB_CA6B)
                ^ seed.wrapping_mul(0x27D4_EB2F))
                | 1,
            hop,
            stirred: false,
            string: 0,
            was_open: false,
            greed: false,
            rest_left: 0,
            fire_left: 0,
        }
    }

    fn recall(&self) -> Seen {
        let back = REACTION.min(self.filled.saturating_sub(1));
        let idx = (self.at + self.memory.len() - 1 - back) % self.memory.len();
        self.memory[idx]
    }

    fn roll_slop(&mut self) {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        let span = (SLOP_LATE + SLOP_EARLY + 1) as u32;
        self.slop = (self.rng % span) as i32 - SLOP_LATE;
    }
}

/// The greedy plan: as the decent one, but it lets the tongue land once the
/// toad is under a quarter, and guts it from inside.
pub fn greedy(who: usize, seed: u32, hop: Fx) -> Box<dyn Plan + Send + Sync> {
    let mut m = Mireback::new(who, seed, hop);
    m.greed = true;
    Box::new(m)
}

// ---------------------------------------------------------------------------
// Reading the floor
// ---------------------------------------------------------------------------

fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

fn on_kind(floor: &Floor, at: V3, kinds: &[u8]) -> bool {
    floor
        .iter()
        .any(|h| kinds.contains(&h.kind) && h.covers_flat(at))
}

fn touching(a: &Placed, b: &Placed) -> bool {
    a.flat_gap(b.nearest_to(a.a)).raw() <= a.radius.add(b.radius).raw()
}

/// **Which fuses are ready**: braziers whose lane meets tar that is joined,
/// pool to touching pool, to tar under the toad. A bit per brazier.
fn fuses(seen: &Seen, beast: &Monster) -> u32 {
    let foot = fight::foot_radius(beast);
    let tar: Vec<&Placed> = seen.floor.iter().filter(|h| h.kind == fight::TAR).collect();
    let under: Vec<bool> = tar
        .iter()
        .map(|h| wide_flat_dist(h.a, beast.pos).raw() < h.radius.add(foot).raw())
        .collect();
    if !under.iter().any(|u| *u) {
        return 0;
    }
    // Every pool joined to one under it.
    let mut joined = under.clone();
    for _ in 0..tar.len() {
        for i in 0..tar.len() {
            if joined[i] {
                continue;
            }
            if (0..tar.len()).any(|j| joined[j] && touching(tar[i], tar[j])) {
                joined[i] = true;
            }
        }
    }
    let length = Knob::CoalLength.fx();
    let width = Knob::CoalWidth.fx();
    let mut ready = 0;
    for (b, (at, lit)) in seen.braziers.iter().enumerate().take(seen.count) {
        if !lit {
            continue;
        }
        let a = flat(*at);
        let end = a.add(inward_of(a).scale(length));
        if tar.iter().enumerate().any(|(i, h)| {
            joined[i] && sim::math::flat_segment_gap(h.a, a, end).raw() <= h.radius.add(width).raw()
        }) {
            ready |= 1 << b;
        }
    }
    ready
}

/// Which way is into the arena from a point on its edge.
fn inward_of(at: V3) -> V3 {
    let to = V3::new(at.x.neg(), Fx::ZERO, at.z.neg());
    if to.flat_len().raw() > 0 {
        to.normalized()
    } else {
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
    }
}

/// The lowest point of its rim, and where on it -- the middle of the top
/// face of whichever rim part is lowest, nearest the hunter.
fn rim(beast: &Monster, from: V3) -> (V3, Fx) {
    let rig = beast.rig();
    let mut best: Option<(V3, Fx, Fx)> = None;
    for part in [
        mireback::FLANK_L,
        mireback::FLANK_R,
        mireback::RUMP,
        mireback::BROW,
    ] {
        if rig.of(part).rot.r[1].y.raw() <= 0 {
            continue;
        }
        let sh = mireback::SPECIES.shape(part);
        let mid = sh.min.add(sh.max).scale(HALF);
        // The point of its top face nearest the hunter, clamped onto the face.
        let local = rig.world_to_part(part, from);
        let x = local.x.clamp(sh.min.x, sh.max.x);
        let z = local.z.clamp(sh.min.z, sh.max.z);
        let edge = rig.part_to_world(part, V3::new(x, sh.max.y, z));
        let _ = mid;
        let d = wide_flat_dist(edge, from);
        if best.is_none_or(|(_, _, seen)| d.raw() < seen.raw()) {
            best = Some((edge, edge.y, d));
        }
    }
    best.map_or((beast.pos, Fx::MAX), |(at, top, _)| (at, top))
}

/// The nearest whole wart, on its back, in the world.
fn wart(beast: &Monster, from: V3) -> Option<V3> {
    let rig = beast.rig();
    mireback::WARTS
        .iter()
        .filter(|w| !beast.broken(**w))
        .map(|w| {
            let sh = mireback::SPECIES.shape(*w);
            let mid = sh.min.add(sh.max).scale(HALF);
            rig.part_to_world(*w, V3::new(mid.x, mid.y, mid.z))
        })
        .min_by_key(|at| wide_flat_dist(*at, from).raw())
}

/// A mound beside it that is a step to the rim for a hop this high: the top
/// to stand on, and the rim point to jump for.
fn step_up(seen: &Seen, beast: &Monster, me: V3, hop: Fx) -> Option<(V3, Fx)> {
    let sp = &mireback::SPECIES;
    let layer = sim::hazard::stat_fx(sp, fight::SLAG, sim::hazard::HazardField::Solid);
    let foot = fight::foot_radius(beast);
    seen.floor
        .iter()
        .filter(|h| h.kind == fight::SLAG && h.state > 0)
        .filter_map(|h| {
            let top = layer.mul(Fx::from_int(h.state as i32));
            let (edge, rim_top) = rim(beast, h.a);
            let reach = top.add(hop).sub(ROUTE_MARGIN);
            let gap = wide_flat_dist(h.a, beast.pos).sub(h.radius).sub(foot);
            // Somewhere it can get onto, too: its own hop clears the top.
            let onto = hop.sub(ROUTE_MARGIN).raw() >= top.raw();
            (reach.raw() >= rim_top.raw() && gap.raw() <= MOUND_GAP.raw() && onto)
                .then_some((V3::new(h.a.x, top, h.a.z), wide_flat_dist(edge, me)))
        })
        .min_by_key(|(_, d)| d.raw())
}

// ---------------------------------------------------------------------------
// One frame
// ---------------------------------------------------------------------------

/// How far from its middle a move's hit reaches, and where from: the marker
/// it draws.
fn danger(beast: &Monster, me: V3) -> Option<(u8, bool)> {
    let t = beast.telegraph()?;
    let m = beast.sp().attack(t.kind);
    let r = t.radius.add(sim::tuning::body_radius()).add(MARGIN);
    let inside = if t.sweep.raw() > 0 {
        // A lane: the line from the anchor along it.
        let end = t.anchor.add(t.along.scale(t.sweep));
        sim::math::flat_segment_gap(me, t.anchor, end).raw() <= r.raw()
    } else {
        wide_flat_dist(me, t.anchor).raw() <= r.raw()
    };
    let _ = m;
    Some((t.kind, inside))
}

/// Frames until a move's hit lands on somebody at `range` from where its
/// volume starts, in the present.
fn to_contact(beast: &Monster, kind: u8, me: V3) -> i32 {
    let m = beast.sp().attack(kind);
    let to_active = match beast.doing {
        Doing::Startup { left, .. } => left as i32 + 1,
        Doing::Active { left, .. } => -((m.active as i32) - left as i32),
        _ => 0,
    } - REACTION as i32;
    let fly = if m.travel.raw() > 0 {
        let from = beast.telegraph().map_or(beast.pos, |t| t.anchor);
        wide_flat_dist(me, from).div(m.travel.mul(sim::DT)).to_int()
    } else {
        0
    };
    to_active + fly
}

impl Mireback {
    fn watch(&mut self, w: &World) {
        let mut braziers = [(V3::ZERO, false); 4];
        let mut count = 0;
        for (i, at) in fight::braziers(w) {
            braziers[i] = (at, fight::brazier_lit(&w.lore, i));
            count = count.max(i + 1);
        }
        let seen = Seen {
            beast: w.monster().copied(),
            floor: w.terrain().floor,
            braziers,
            count,
        };
        self.memory[self.at] = seen;
        self.at = (self.at + 1) % self.memory.len();
        self.filled = (self.filled + 1).min(self.memory.len());
    }

    fn act(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        self.commit_left = self.commit_left.saturating_sub(1);
        self.rest_left = self.rest_left.saturating_sub(1);
        self.fire_left = self.fire_left.saturating_sub(1);
        let me = w.players[self.who];
        if !me.grounded || me.aboard() || me.action.actionable() {
            self.leap_left = self.leap_left.saturating_sub(1);
        }
        let seen = self.recall();
        let (Some(beast), true) = (seen.beast, me.health > 0) else {
            return Input::default();
        };
        if !beast.alive() || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        // Inside it: there is one thing to do.
        if sim::state::inside(&me, &w.monsters) {
            self.intent = GUT;
            let swing = if self.cooldown == 0 && me.action.actionable() {
                self.cooldown = 8;
                Input::LEFT
            } else {
                0
            };
            return Input::aimed(swing, 0);
        }
        self.stirred |= matches!(beast.doing, Doing::Startup { .. });
        let to = flat(beast.pos.sub(me.pos));
        let aim = atan2_turns(to.z, to.x);
        let wire = turns_to_aim(aim.sub(me.carry_yaw));
        if !self.stirred && !me.aboard() {
            self.intent = FLANK;
            return Input::aimed(0, wire);
        }
        if me.aboard() {
            return self.ride(&me, &beast);
        }
        self.ground(w, &me, &beast, &seen)
    }

    fn ground(
        &mut self,
        w: &World,
        me: &sim::state::Player,
        beast: &Monster,
        seen: &Seen,
    ) -> Input {
        // **Strayed out over a bank**: back in, over it.
        let bounds = w.arena().bounds;
        let out = me.pos.x.raw() < bounds.lo_x.raw()
            || me.pos.x.raw() > bounds.hi_x.raw()
            || me.pos.z.raw() < bounds.lo_z.raw()
            || me.pos.z.raw() > bounds.hi_z.raw();
        if out {
            self.intent = FLANK;
            let home = flat(beast.pos.sub(me.pos));
            let at = atan2_turns(home.z, home.x);
            if me.grounded && self.leap_left == 0 {
                self.leap_left = LEAP_HOLD;
            }
            let jump = if self.leap_left > 0 { Input::SPACE } else { 0 };
            return Input::aimed(steer(at, home) | jump, turns_to_aim(at.sub(me.carry_yaw)));
        }
        let to_beast = flat(beast.pos.sub(me.pos));
        let range = to_beast.flat_len();
        let toward = to_beast.normalized();
        let aim = atan2_turns(toward.z, toward.x);
        let wire = turns_to_aim(aim.sub(me.carry_yaw));
        let side = {
            let bearing = wrap_turns(atan2_turns(toward.z.neg(), toward.x.neg()).sub(beast.yaw));
            if bearing.raw() >= 0 {
                QUARTER
            } else {
                QUARTER.neg()
            }
        };
        let hop = if self.leap_left > 0 { Input::SPACE } else { 0 };

        // 0. What is coming, and whether it is coming here.
        let coming = match beast.doing {
            Doing::Startup { kind, left } | Doing::Active { kind, left } => {
                let a = beast.sp().attack(kind);
                if matches!(beast.doing, Doing::Startup { .. }) && left == a.startup {
                    self.roll_slop();
                }
                Some(kind)
            }
            _ => None,
        };
        if let Some(kind) = coming {
            let contact = to_contact(beast, kind, me.pos) - self.slop;
            let inside = danger(beast, me.pos).is_some_and(|(_, inside)| inside);
            let greedy =
                self.greed && kind == mireback::TONGUE && beast.health * 4 < beast.sp().health();
            if inside && !greedy {
                let a = beast.sp().attack(kind);
                // The Bulwark takes a blockable blow on the shield -- not the
                // tongue, which would take the shield.
                let guards = me.shield().is_some_and(|s| s.in_hand())
                    && a.damage > 0
                    && !a.unblockable
                    && kind != mireback::TONGUE
                    && contact > -(a.active as i32);
                if guards {
                    self.intent = GUARD;
                    let face = turns_to_aim(
                        atan2_turns(beast.pos.z.sub(me.pos.z), beast.pos.x.sub(me.pos.x))
                            .sub(me.carry_yaw),
                    );
                    return Input::aimed(Input::RIGHT, face);
                }
                // A low sheet behind it: jump it.
                if kind == mireback::BACKWASH {
                    if contact <= HOP_LEAD && self.leap_left == 0 && me.grounded {
                        self.intent = EVADE;
                        self.leap_left = HOP_HOLD;
                        return Input::aimed(Input::SPACE, wire);
                    }
                    if self.leap_left > 0 {
                        return Input::aimed(Input::SPACE, wire);
                    }
                }
                // Out of the marker, if there is time to walk it; the dodge
                // on the hit if there is not.
                let out = self.way_out(beast, kind, me.pos);
                let speed = sim::tuning::move_speed().mul(me.slow_mul_now());
                let room = self.room_out(beast, kind, me.pos);
                let walkable =
                    Fx::from_int(contact.max(0)).mul(speed).mul(sim::DT).raw() > room.raw();
                if walkable {
                    self.intent = EVADE;
                    return Input::aimed(steer(aim, out), wire);
                }
                if contact <= DODGE_LEAD && self.dodge_left == 0 && me.action.actionable() {
                    self.intent = EVADE;
                    self.dodge_left = sim::tuning::dodge_frames();
                    return Input::aimed(steer(aim, out) | Input::SHIFT, wire);
                }
                // Not yet: walk out anyway, it is never wrong.
                self.intent = EVADE;
                return Input::aimed(steer(aim, out), wire);
            }
            // The belch: off the tar near its mouth, and the sac if it is in
            // reach.
            if kind == mireback::BELCH && on_kind(&seen.floor, me.pos, &[fight::TAR]) {
                let mouth = fight::mouth(beast);
                if wide_flat_dist(mouth, me.pos).raw()
                    <= Knob::BelchReach.fx().add(Fx::from_int(4)).raw()
                {
                    self.intent = EVADE;
                    let away = flat(me.pos.sub(mouth)).normalized();
                    return Input::aimed(steer(aim, self.clean_way(seen, me.pos, away)), wire);
                }
            }
        }

        // Standing in fire: out of it, first. Twelve every ten frames is the
        // fastest thing in the fight to die to, and it is a thing you walk out
        // of.
        if on_kind(&seen.floor, me.pos, &[fight::BURNING, fight::COALS]) && me.grounded {
            self.intent = EVADE;
            let away = flat(me.pos.sub(beast.pos)).normalized();
            return Input::aimed(steer(aim, self.clean_way(seen, me.pos, away)), wire);
        }

        // 0b. **Her own fire.** The Elementalist does not need a brazier: a
        //     pillar planted in tar under the toad, or tar joined to it,
        //     from wherever she stands -- the doc's "she decides when the
        //     floor is cleared".
        if me.class == sim::Class::Elementalist && self.fire_left == 0 && me.action.actionable() {
            let reach = sim::moves::get(me.class, sim::state::SLOT_SPECIAL).reach;
            let foot = fight::foot_radius(beast);
            let pool = seen
                .floor
                .iter()
                .filter(|h| h.kind == fight::TAR)
                .filter(|h| wide_flat_dist(h.a, beast.pos).raw() < h.radius.add(foot).raw())
                .filter(|h| !h.covers_flat(me.pos))
                .map(|h| {
                    // The nearest point of the pool to her, a little inside it.
                    let toward = flat(me.pos.sub(h.a));
                    let inside = h.radius.sub(Fx::ratio(1, 2)).max(Fx::ZERO);
                    let spot = if toward.flat_len().raw() > 0 {
                        h.a.add(toward.normalized().scale(inside))
                    } else {
                        h.a
                    };
                    (spot, wide_flat_dist(spot, me.pos))
                })
                .filter(|(_, d)| d.raw() <= reach.raw())
                .min_by_key(|(_, d)| d.raw());
            if let Some((spot, _)) = pool {
                self.fire_left = FIRE_GAP;
                self.intent = KINDLE;
                return looking(me, flat(spot), Input::SPECIAL);
            }
        }

        // 1. Kindle: a brazier whose tar is joined to the toad. Kept to the
        //    one it chose while that one is still a fuse, and given up if it
        //    is taking too long to get there.
        let ready = fuses(seen, beast);
        if self.brazier.is_some_and(|b| ready & 1 << b == 0) || self.commit_left == 0 {
            self.brazier = None;
        }
        if self.brazier.is_none() && ready != 0 && self.rest_left == 0 {
            let nearest = (0..seen.count)
                .filter(|b| ready & 1 << b != 0)
                .min_by_key(|b| wide_flat_dist(seen.braziers[*b].0, me.pos).raw());
            self.brazier = nearest;
            self.commit_left = COMMIT;
        }
        // An opening right in front of it is worth more than a walk.
        let close_opening = beast.doing.open()
            && range.raw() < STANDOFF.add(Fx::ONE).raw()
            && beast.doing.frames_left() as i32 > REACTION as i32 + 20;
        if let Some(b) = self.brazier.filter(|_| !close_opening) {
            let out = self.kindle(me, seen.braziers[b].0);
            if self.commit_left == 1 {
                // Gave up: do not pick it straight back up.
                self.rest_left = COMMIT;
            }
            return out;
        }
        let fuse_ready = (ready != 0).then_some(());

        // 2. Climb, if a mound or a sag puts the rim inside its hop.
        let (rim_at, rim_top) = rim(beast, me.pos);
        let from_floor = rim_top.add(ROUTE_MARGIN).raw() < self.hop.raw();
        let mound = step_up(seen, beast, me.pos, self.hop);
        let open_enough = !matches!(
            beast.doing,
            Doing::Startup {
                kind: mireback::INFLATE | mireback::WALLOW | mireback::FLOP,
                ..
            }
        ) && beast.doing.attacking() != Some(mireback::WALLOW);
        if open_enough && (from_floor || mound.is_some()) && range.raw() < Fx::from_int(14).raw() {
            self.intent = CLIMB;
            // On the mound already, or the rim within a hop: jump for it.
            let standing_on = me.pos.y.raw() > Fx::ONE.raw();
            if from_floor || standing_on {
                let to_rim = flat(rim_at.sub(me.pos));
                if to_rim.flat_len().raw() < Fx::from_int(3).raw()
                    && me.grounded
                    && self.leap_left == 0
                {
                    self.leap_left = LEAP_HOLD;
                }
                let jump = if self.leap_left > 0 { Input::SPACE } else { 0 };
                return Input::aimed(steer(aim, to_rim) | jump, wire);
            }
            if let Some((top, _)) = mound {
                let to_top = flat(top.sub(me.pos));
                if to_top.flat_len().raw() < Fx::from_int(2).raw()
                    && me.grounded
                    && self.leap_left == 0
                {
                    self.leap_left = LEAP_HOLD;
                }
                let jump = if self.leap_left > 0 { Input::SPACE } else { 0 };
                return Input::aimed(steer(aim, to_top) | jump, wire);
            }
        }

        // 3. Hit it in an opening; otherwise hold the flank on clean floor.
        let opening = match beast.doing {
            Doing::Recovery { kind, left } => {
                // Not the swell's: the puffed hold is the sac's window, and
                // the sac is under its chin.
                kind != mireback::SWALLOW && left as i32 > REACTION as i32 + 10
            }
            Doing::Flinch { left } | Doing::Stumble { left, .. } | Doing::Toppled { left } => {
                left as i32 > REACTION as i32
            }
            // A wallow with no fire ready is broken by damage.
            Doing::Active {
                kind: mireback::WALLOW,
                ..
            } => fuse_ready.is_none(),
            Doing::Active {
                kind: mireback::SWALLOW,
                ..
            } => true,
            _ => false,
        };
        if opening && !self.was_open {
            self.string = 0;
        }
        self.was_open = opening;
        // The sac, when it is out and near.
        let sac = if fight::sac_shown(beast) {
            let sh = mireback::SPECIES.shape(mireback::SAC);
            let mid = sh.min.add(sh.max).scale(HALF);
            Some(beast.world_of(mireback::SAC, mid))
        } else {
            None
        };
        let target = match sac {
            Some(s) if wide_flat_dist(s, me.pos).raw() < Fx::from_int(6).raw() => flat(s),
            _ => flat(beast.pos).add(
                flat(me.pos.sub(beast.pos))
                    .normalized()
                    .scale(fight::foot_radius(beast)),
            ),
        };
        // Where on it to put the crosshair: the sac where it hangs, or its
        // flank at the height of a fighter's chest.
        let target_point = match sac {
            Some(s) if wide_flat_dist(s, me.pos).raw() < Fx::from_int(6).raw() => s,
            _ => V3::new(target.x, Fx::ratio(15, 10), target.z),
        };
        let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
        let strike = poke.reach.add(Fx::ratio(1, 2));
        let to_target = flat(target.sub(me.pos));
        let at_target = atan2_turns(to_target.z, to_target.x);
        let target_wire = turns_to_aim(at_target.sub(me.carry_yaw));
        // A short opening it can answer is where the swell punishes the
        // third hit; a long one -- winded, gutted, staggered -- is not.
        let long_window = matches!(
            beast.doing,
            Doing::Toppled { .. } | Doing::Stumble { .. } | Doing::Flinch { .. }
        ) || matches!(
            beast.doing,
            Doing::Active {
                kind: mireback::SWALLOW | mireback::WALLOW,
                ..
            }
        );
        let going_in = (opening
            || sac.is_some_and(|_| beast.doing.attacking() != Some(mireback::INFLATE)))
            && (self.string < STRING || long_window);
        if going_in {
            self.intent = PUNISH;
            let walk = if to_target.flat_len().raw() > strike.raw() {
                steer(aim, to_target)
            } else {
                0
            };
            // **Only a swing that is over before it can act again**: the
            // window it can see, less its own reaction, against the swing.
            let window = beast.frames_until_free() as i32 - REACTION as i32;
            let poke_busy = (poke.startup + poke.active + poke.recovery) as i32;
            let heavy_busy = crate::heavy_commitment(me.class) as i32;
            let swing = if self.cooldown == 0
                && me.action.actionable()
                && to_target.flat_len().raw() <= strike.add(Fx::ONE).raw()
                && window > poke_busy + EXIT
            {
                self.cooldown = SWING_GAP;
                self.string = self.string.saturating_add(1);
                if window > heavy_busy + EXIT {
                    heavy(me.class)
                } else {
                    Input::LEFT
                }
            } else {
                0
            };
            let _ = target_wire;
            return looking(me, target_point, walk | swing | hop);
        }

        // Hold the flank, on clean floor.
        self.intent = FLANK;
        let station = self.station(seen, beast, me.pos, side);
        let to_station = flat(station.sub(me.pos));
        let walk = if to_station.flat_len().raw() > SETTLED.raw() {
            steer(aim, to_station)
        } else {
            0
        };
        let _ = w;
        Input::aimed(walk | hop, wire)
    }

    /// Where to wait: at its flank on the side it is on, the clean spot
    /// nearest, round the circle at the standoff.
    fn station(&self, seen: &Seen, beast: &Monster, me: V3, side: Fx) -> V3 {
        let bounds = sim::arena::ArenaId::MIREBACK.get().bounds;
        let inside = |p: V3| {
            p.x.raw() > bounds.lo_x.add(Fx::ONE).raw()
                && p.x.raw() < bounds.hi_x.sub(Fx::ONE).raw()
                && p.z.raw() > bounds.lo_z.add(Fx::ONE).raw()
                && p.z.raw() < bounds.hi_z.sub(Fx::ONE).raw()
        };
        let mut best: Option<(V3, Fx)> = None;
        for step in 0..12 {
            let off = Fx::from_int(step - 6).div(Fx::from_int(24));
            let turn = beast.yaw.add(side).add(off);
            let at = flat(beast.pos).add(V3::from_turns(turn).scale(STANDOFF));
            if !inside(at) || on_kind(&seen.floor, at, &[fight::TAR, fight::BURNING, fight::COALS])
            {
                continue;
            }
            let d = wide_flat_dist(at, me).add(off.abs().mul(Fx::from_int(8)));
            if best.is_none_or(|(_, b)| d.raw() < b.raw()) {
                best = Some((at, d));
            }
        }
        best.map_or_else(
            || flat(beast.pos).add(V3::from_turns(beast.yaw.add(side)).scale(STANDOFF)),
            |(at, _)| at,
        )
    }

    /// A direction from `from` toward floor that is not tar, near `want`.
    fn clean_way(&self, seen: &Seen, from: V3, want: V3) -> V3 {
        for step in 0..8 {
            let turn = atan2_turns(want.z, want.x).add(
                Fx::from_int(if step % 2 == 0 {
                    step / 2
                } else {
                    -(step / 2) - 1
                })
                .div(Fx::from_int(12)),
            );
            let dir = V3::from_turns(turn);
            if !on_kind(
                &seen.floor,
                from.add(dir.scale(Fx::from_int(3))),
                &[fight::TAR, fight::BURNING],
            ) {
                return dir;
            }
        }
        want
    }

    /// Which way out of a move's marker.
    fn way_out(&self, beast: &Monster, kind: u8, me: V3) -> V3 {
        let Some(t) = beast.telegraph() else {
            return flat(me.sub(beast.pos)).normalized();
        };
        if t.sweep.raw() > 0 || kind == mireback::TONGUE {
            // Across the lane, never along it.
            let across = V3::from_turns(atan2_turns(t.along.z, t.along.x).add(QUARTER));
            let rel = flat(me.sub(t.anchor));
            if rel.dot(across).raw() >= 0 {
                across
            } else {
                across.scale(Fx::ONE.neg())
            }
        } else {
            let away = flat(me.sub(t.anchor));
            if away.flat_len().raw() > 0 {
                away.normalized()
            } else {
                V3::from_turns(beast.yaw.add(QUARTER))
            }
        }
    }

    /// How far it has to go to be out of a move's marker.
    fn room_out(&self, beast: &Monster, kind: u8, me: V3) -> Fx {
        let Some(t) = beast.telegraph() else {
            return Fx::ZERO;
        };
        let r = t.radius.add(sim::tuning::body_radius()).add(MARGIN);
        let d = if t.sweep.raw() > 0 || kind == mireback::TONGUE {
            let end = t.anchor.add(t.along.scale(t.sweep));
            sim::math::flat_segment_gap(me, t.anchor, end)
        } else {
            wide_flat_dist(me, t.anchor)
        };
        r.sub(d).max(Fx::ZERO)
    }

    /// Go to a brazier and kick it into the arena: to the floor in front of
    /// its plinth, a jump up onto the plinth past the brazier, and a swing
    /// back toward the middle from behind it.
    fn kindle(&mut self, me: &sim::state::Player, brazier: V3) -> Input {
        self.intent = KINDLE;
        let inward = inward_of(flat(brazier));
        let behind = flat(brazier).sub(inward.scale(Fx::ratio(9, 10)));
        let front = flat(brazier).add(inward.scale(Fx::ratio(30, 10)));
        let look = atan2_turns(inward.z, inward.x);
        let face = turns_to_aim(look.sub(me.carry_yaw));
        let on_plinth = me.grounded && me.pos.y.raw() >= brazier.y.sub(Fx::ratio(1, 4)).raw();
        let go = |to: V3| {
            let d = flat(to.sub(me.pos));
            let at = atan2_turns(d.z, d.x);
            (
                steer(at, d),
                turns_to_aim(at.sub(me.carry_yaw)),
                d.flat_len(),
            )
        };
        if on_plinth {
            let (walk, wire, far) = go(behind);
            if far.raw() > Fx::ratio(3, 10).raw() {
                return Input::aimed(walk, wire);
            }
            let swing = if self.cooldown == 0 && me.action.actionable() {
                self.cooldown = SWING_GAP;
                Input::LEFT
            } else {
                0
            };
            return Input::aimed(swing, face);
        }
        // In the air on the way up: keep going for the spot behind it.
        if self.leap_left > 0 || !me.grounded {
            let (walk, wire, _) = go(behind);
            let jump = if self.leap_left > 0 { Input::SPACE } else { 0 };
            return Input::aimed(walk | jump, wire);
        }
        // On the floor: to the front of the plinth, then up and over.
        let (walk, wire, far) = go(front);
        if far.raw() < Fx::ratio(8, 10).raw() {
            self.leap_left = LEAP_HOLD;
            let (walk, wire, _) = go(behind);
            return Input::aimed(walk | Input::SPACE, wire);
        }
        Input::aimed(walk, wire)
    }

    fn ride(&mut self, me: &sim::state::Player, beast: &Monster) -> Input {
        let along = V3::from_turns(beast.yaw);
        let aim = atan2_turns(along.z, along.x);
        let wire = turns_to_aim(aim.sub(me.carry_yaw));
        if self.leap_left > 0 {
            return Input::aimed(Input::SPACE, wire);
        }
        match beast.doing {
            Doing::Startup {
                kind: mireback::WALLOW | mireback::FLOP,
                ..
            } => {
                self.intent = LEAVE;
                self.leap_left = LEAP_HOLD;
                return Input::aimed(Input::SPACE, wire);
            }
            Doing::Startup {
                kind: mireback::INFLATE,
                ..
            }
            | Doing::Active {
                kind: mireback::INFLATE,
                ..
            } => {
                self.intent = BRACE;
                return Input::aimed(Input::CROUCH, wire);
            }
            _ => {}
        }
        let Some(target) = wart(beast, me.pos) else {
            // Nothing left to burst up here: off, and back to the floor game.
            self.intent = LEAVE;
            self.leap_left = LEAP_HOLD;
            return Input::aimed(Input::SPACE, wire);
        };
        let to = flat(target.sub(me.pos));
        let at = atan2_turns(to.z, to.x);
        let face = turns_to_aim(at.sub(me.carry_yaw));
        if to.flat_len().raw() > Fx::ratio(12, 10).raw() {
            self.intent = TO_WART;
            return Input::aimed(steer(at, to), face);
        }
        self.intent = WORK;
        let swing = if self.cooldown == 0 && matches!(me.action, Action::Free) {
            self.cooldown = SWING_GAP;
            Input::LEFT
        } else {
            0
        };
        let _ = face;
        looking(me, target, swing)
    }
}

/// Buttons, with the crosshair put on a point: the yaw toward it and the
/// pitch the camera needs (`aim::look_onto`). What a person does with the
/// mouse, and what a shot or a swing at something not at chest height needs.
fn looking(me: &sim::state::Player, at: V3, bits: u16) -> Input {
    let d = flat(at.sub(me.pos));
    let yaw = atan2_turns(d.z, d.x);
    let wire = turns_to_aim(yaw.sub(me.carry_yaw));
    let pitch = sim::aim::look_onto(me.pos, wire, me.aloft, at);
    Input::looking_at(bits, wire, pitch)
}

trait SlowNow {
    fn slow_mul_now(&self) -> Fx;
}

impl SlowNow for sim::state::Player {
    /// How fast it walks now, as a share: the slow it is under, if any.
    fn slow_mul_now(&self) -> Fx {
        if self.slowed > 0 {
            self.slow_mul
        } else {
            Fx::ONE
        }
    }
}

impl Plan for Mireback {
    fn watch(&mut self, w: &World) {
        Mireback::watch(self, w);
    }

    fn act(&mut self, w: &World) -> Input {
        Mireback::act(self, w)
    }

    fn intent(&self) -> Intent {
        self.intent
    }
}

/// Moves that move its back hard enough to throw a rider.
pub fn bucks(kind: u8) -> bool {
    matches!(kind, mireback::INFLATE | mireback::WALLOW | mireback::FLOP)
}

/// The Mireback's entry in the harness.
pub static CARD: crate::plans::Card = crate::plans::Card {
    species: sim::species::SpeciesId::MIREBACK,
    plan: |who, seed, hop| Box::new(Mireback::new(who, seed, hop)),
    bucks,
    words: crate::plans::Words {
        weak_hits: "weak-point hits",
        broken: "warts burst",
        into_breakables: "damage into warts",
        into_breakables_why: "the climb, played",
        worst: "worst wart",
        ride_for: "long enough to reach a wart?",
        toppled_pool: "off the pool under a gutted Mireback",
    },
    tally: Some(|| Box::new(MireTally::default())),
};

// ---------------------------------------------------------------------------
// The report's Mireback lines (§9)
// ---------------------------------------------------------------------------

/// What the Mireback's report counts.
#[derive(Default)]
pub struct MireTally {
    /// Samples of how much of the floor is tar, in thousandths.
    coverage_sum: u64,
    coverage_samples: u32,
    pub coverage_peak: u32,
    /// Frames a hunter stood in tar, and hits it took there.
    pub tar_frames: u32,
    pub hits_in_tar: u32,
    pub hits: u32,
    /// Hunters' mounts that started from a slag mound.
    pub climbed_from_slag: u32,
    on_slag: [u32; sim::state::MAX_PLAYERS],
    /// The frame each wart went.
    pub warts: Vec<u32>,
    /// Flops that landed on somebody who was not in tar when it committed
    /// but was when it crashed: a combo that was not on the floor when they
    /// had to decide.
    pub unanswerable: u32,
    in_tar_at_commit: [bool; sim::state::MAX_PLAYERS],
    /// The lore's counters, as the hunt ended.
    lore: [u32; 32],
    frames: u32,
}

impl Tally for MireTally {
    fn observe(&mut self, before: &World, after: &World) {
        let Some(now) = after.monster() else { return };
        self.frames = after.frame;
        let ground = after.terrain();
        // Coverage, every half second: a metre grid over the arena.
        if after.frame % 30 == 0 {
            let b = after.arena().bounds;
            let (x0, x1, z0, z1) = (
                b.lo_x.to_int(),
                b.hi_x.to_int(),
                b.lo_z.to_int(),
                b.hi_z.to_int(),
            );
            let mut covered = 0u32;
            let mut total = 0u32;
            for x in x0..x1 {
                for z in z0..z1 {
                    total += 1;
                    let p = V3::new(
                        Fx::from_int(x).add(HALF),
                        Fx::ZERO,
                        Fx::from_int(z).add(HALF),
                    );
                    if ground
                        .floor
                        .iter()
                        .any(|h| h.kind == fight::TAR && h.covers_flat(p))
                    {
                        covered += 1;
                    }
                }
            }
            let share = covered * 1000 / total.max(1);
            self.coverage_sum += share as u64;
            self.coverage_samples += 1;
            self.coverage_peak = self.coverage_peak.max(share);
        }
        let in_tar = |p: &sim::state::Player| {
            p.footed()
                && !p.aboard()
                && ground
                    .floor
                    .iter()
                    .any(|h| h.kind == fight::TAR && h.holds(p.pos))
        };
        // Where each hunter stood when a flop committed.
        if let Doing::Startup {
            kind: mireback::FLOP,
            left,
        } = now.doing
        {
            if left == now.sp().attack(mireback::FLOP).startup {
                for (i, p) in after.players.iter().enumerate() {
                    self.in_tar_at_commit[i] = in_tar(p);
                }
            }
        }
        for (i, (was, p)) in before.players.iter().zip(after.players.iter()).enumerate() {
            if p.health <= 0 {
                continue;
            }
            let tarred = in_tar(was);
            if tarred {
                self.tar_frames += 1;
            }
            if p.health < was.health {
                self.hits += 1;
                if tarred {
                    self.hits_in_tar += 1;
                }
                if let Doing::Active {
                    kind: mireback::FLOP,
                    ..
                } = now.doing
                {
                    if !self.in_tar_at_commit[i] && tarred {
                        self.unanswerable += 1;
                    }
                }
            }
            // Standing on a mound, and then aboard.
            let on_mound = ground.floor.iter().any(|h| {
                h.kind == fight::SLAG && h.covers_flat(p.pos) && p.pos.y.raw() > Fx::ONE.raw()
            });
            if on_mound {
                self.on_slag[i] = after.frame;
            }
            if p.aboard()
                && !was.aboard()
                && !sim::state::inside(p, &after.monsters)
                && after.frame.saturating_sub(self.on_slag[i]) < 90
            {
                self.climbed_from_slag += 1;
            }
        }
        if let Some(was) = before.monster() {
            for w in mireback::WARTS {
                if !was.broken(w) && now.broken(w) {
                    self.warts.push(after.frame);
                }
            }
        }
        for (i, slot) in self.lore.iter_mut().enumerate() {
            *slot = after.lore.word(i);
        }
    }

    fn unanswerable(&self) -> u32 {
        self.unanswerable
    }

    fn render(&self, report: &Report) -> String {
        let mut out = String::from("\nTHE MIRE\n");
        let line = |out: &mut String, name: &str, value: String, why: &str| {
            out.push_str(&format!("  {name:<26} {value:>9}   {why}\n"));
        };
        let w = |i: usize| self.lore[i];
        let mean = if self.coverage_samples == 0 {
            0
        } else {
            (self.coverage_sum / self.coverage_samples as u64) as u32
        };
        line(
            &mut out,
            "tar coverage, mean / peak",
            format!("{}% / {}%", mean / 10, self.coverage_peak / 10),
            "the clock",
        );
        line(
            &mut out,
            "time in tar",
            format!("{:.0}s", self.tar_frames as f32 / 60.0),
            "a hunter's feet in it",
        );
        line(
            &mut out,
            "hits taken in tar",
            format!("{} of {}", self.hits_in_tar, self.hits),
            "is the clock what kills?",
        );
        let lit = w(fight::word::LIT);
        line(
            &mut out,
            "fires: brazier / belch",
            format!("{} / {}", lit & 0xFF, (lit >> 8) & 0xFF),
            "pools lit, by what",
        );
        line(
            &mut out,
            "fires: backfire / class",
            format!("{} / {}", (lit >> 16) & 0xFF, (lit >> 24) & 0xFF),
            "",
        );
        let burn = w(fight::word::SELF_BURN) as i32;
        line(
            &mut out,
            "self-burn",
            format!(
                "{} ({:.0}%)",
                burn,
                if report.dealt > 0 {
                    burn as f32 * 100.0 / report.dealt as f32
                } else {
                    0.0
                }
            ),
            "of all damage dealt: 30-45% is the target",
        );
        line(
            &mut out,
            "coat uptime",
            format!(
                "{:.0}%",
                w(fight::word::COAT_FRAMES) as f32 * 100.0 / self.frames.max(1) as f32
            ),
            "more than half a coat",
        );
        let slag = w(fight::word::SLAG_BUILT);
        line(
            &mut out,
            "slag built / layered",
            format!("{} / {}", slag & 0xFFFF, slag >> 16),
            "mounds, and second layers",
        );
        line(
            &mut out,
            "climbed from slag",
            format!("{}", self.climbed_from_slag),
            "rides that started on a mound",
        );
        line(
            &mut out,
            "warts burst",
            if self.warts.is_empty() {
                "0".to_string()
            } else {
                self.warts
                    .iter()
                    .map(|f| format!("{:.1}m", *f as f32 / 3600.0))
                    .collect::<Vec<_>>()
                    .join(" ")
            },
            "and the minute each went",
        );
        let sw = w(fight::word::SWALLOWS);
        line(
            &mut out,
            "swallowed / hot",
            format!("{} / {}", sw & 0xFFFF, sw >> 16),
            "times inside it",
        );
        line(
            &mut out,
            "stomach dealt / acid",
            format!(
                "{} / {}",
                w(fight::word::STOMACH_DEALT),
                w(fight::word::ACID)
            ),
            "the swallow's trade",
        );
        let g = w(fight::word::GUTTED);
        line(
            &mut out,
            "gutted / wallows broken",
            format!("{} / {}", g & 0xFFFF, g >> 16),
            "the big window, and the consolation",
        );
        let wd = w(fight::word::WINDED);
        line(
            &mut out,
            "pools laid",
            format!("{}", w(fight::word::POOLS)),
            "",
        );
        line(
            &mut out,
            "backfires",
            format!("{}", wd >> 16),
            "the belch broken through the sac",
        );
        line(
            &mut out,
            "flops from fresh tar",
            format!("{}", self.unanswerable),
            "tar under you after it committed (unanswerable)",
        );
        let _ = monster::NO_PART;
        out
    }
}
