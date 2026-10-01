//! The Veilstalker's hunter: the plan a decent player follows, with a camera,
//! and the report lines the fight is measured by (`veilstalker.md` §9).
//!
//! **This hunter knows only what is drawn.** It never reads where the
//! creature is. It reads, a reaction late and only inside its own camera's
//! view (`sim::aim::in_view`): the parts the simulation shows
//! (`World::shown` -- a decloak, a shimmer, a mottled region, an outline),
//! the paint on its hide, every decloak being drawn (`fight::apparitions`,
//! the real one and a mimic's ghost alike, with the silhouette each shows),
//! the floor markers, the prints in the snow, the smoke and the braziers.
//! "Visible" means exactly what the renderer draws. A hunter that read the
//! creature's true position would make the fight unmeasured.
//!
//! **The plan** (§9), what a person learns in the first ten minutes:
//!
//! 1. Stand in the open, on snow, and watch the floor. Sweep the camera
//!    slowly; face any print younger than a second within twelve metres.
//! 2. Answer the rear, not the shimmer. On a decloak: if there are no fresh
//!    prints under it, do nothing (mimic). Otherwise read the silhouette at
//!    frame eight -- lunge: dodge at its arrival; spear: walk sideways; rake:
//!    jump; pounce circle: walk out; quills: step behind the nearest solid,
//!    or dodge if none is within three metres.
//! 3. Punish the recovery, and unload into the second hit on the paint.
//! 4. Follow the trail after a retreat at a walk, facing the newest print.
//! 5. Leave smoke at right angles to the way in.
//! 6. Tip a brazier across the trail when the newest prints are heading
//!    toward one and within six metres of it.

use sim::aim::{Scene, in_view};
use sim::fixed::Fx;
use sim::math::{atan2_turns, flat_segment_gap, wide_flat_dist, wrap_turns};
use sim::monster::{Doing, Monster};
use sim::species::veilstalker::fight::{self, Apparition, Print};
use sim::species::veilstalker::{self as vs};
use sim::state::{Action, Phase, Player};
use sim::{Input, V3, World};

use crate::report::{HALF_VIEW, Tally};
use crate::{Hunter, Intent, Plan, REACTION, heavy, steer, turns_to_aim};

/// Watching the floor.
pub const WATCH: Intent = Intent("Watch");
/// Facing a fresh print.
pub const READ: Intent = Intent("Read");
/// Holding still through a decloak with nothing under it.
pub const HOLD: Intent = Intent("Hold");
/// The dodge at a lunge's arrival.
pub const DODGE: Intent = Intent("Dodge");
/// Out of a lane or a circle.
pub const EVADE: Intent = Intent("Evade");
/// Over the rake.
pub const JUMP: Intent = Intent("Jump");
/// Behind a trunk from the quills.
pub const COVER: Intent = Intent("Cover");
/// Hitting it in its recovery, or on its paint.
pub const PUNISH: Intent = Intent("Punish");
/// Unloading into the panic.
pub const UNLOAD: Intent = Intent("Unload");
/// After the retreat, along the trail.
pub const FOLLOW: Intent = Intent("Follow");
/// Out of the smoke.
pub const LEAVE: Intent = Intent("Leave");
/// A brazier tipped across the trail.
pub const TIP: Intent = Intent("Tip");
/// Back out into the open.
pub const OPEN: Intent = Intent("Open");

/// How fast it turns its camera: a person's flick, three quarters of a turn
/// a second -- the Pair's hunter's.
pub const TURN: Fx = Fx::ratio(3, 4);
/// How fast it sweeps while it watches, a sixth of that.
const SWEEP: Fx = Fx::ratio(1, 8);
/// How far either side of where it is watching it sweeps, in turns.
const SWEEP_WIDE: Fx = Fx::ratio(1, 10);
/// Its camera's pitch while it watches: a little down, so the floor round it
/// and out to the edge of the fight is on the screen.
const WATCH_PITCH: i16 = -2400;
/// How many frames early or late its timed presses can be.
const SLOP_EARLY: i32 = 3;
/// A judged arrival is off by up to one frame in this many of the wait.
const JUDGE: i32 = 10;
const SLOP_LATE: i32 = 3;
/// What the renderer shows faintly enough that a person still sees it: a
/// shimmer is, a part fading out of its last frames is not.
pub const SEE: Fx = Fx::ratio(1, 5);
/// The silhouette tells the move by this frame of a decloak (§2).
const SILHOUETTE: i32 = 8;
/// A print this young is "fresh": a second.
const FRESH: u32 = 60;
/// Prints are read within this far.
const READ_FAR: Fx = Fx::from_int(12);
/// A decloak's feet set within this far of it.
const FEET: Fx = Fx::from_int(2);
/// Frames of jump held: a full hop.
const LEAP_HOLD: u16 = 24;
/// How far past a marker's edge counts as out of it.
const MARGIN: Fx = Fx::ratio(5, 10);
/// A trunk within this of it is cover from the quills.
const COVER_NEAR: Fx = Fx::from_int(3);
/// Standing this near a trunk is standing under a perch: it moves off.
const OPEN_FROM: Fx = Fx::from_int(5);
/// Frames between swings.
const SWING_GAP: u16 = 16;
/// Frames of an opening kept back to get out in.
const EXIT: i32 = 6;
/// Frames after a dodge before another.
const DODGE_REST: u16 = 2;
/// A brazier this near the head of the trail is worth tipping.
const TIP_NEAR: Fx = Fx::from_int(6);
/// Prints are a trail heading somewhere if the newest two are this far
/// apart.
const STEP: Fx = Fx::ratio(1, 2);
/// A sighting this old is forgotten.
const STALE: u32 = 240;

/// What it knows of a decloak it saw begin.
#[derive(Clone, Copy, Debug)]
struct Decloak {
    /// The frame it began, in the present's clock.
    began: u32,
    at: V3,
    yaw: Fx,
    rear: u8,
    /// No feet set under it: a mimic.
    empty: bool,
    /// Already answered.
    done: bool,
    /// How far off its judgement of the arrival is, in frames: drawn once.
    judged: i32,
}

pub struct Veilstalker {
    who: usize,
    memory: Vec<World>,
    at: usize,
    filled: usize,
    intent: Intent,
    /// Where its camera points, in world turns, and which way it is sweeping.
    look: Fx,
    looked: bool,
    sweep: Fx,
    /// Where it last saw the creature, and when (present clock).
    known: Option<(V3, u32)>,
    decloak: Option<Decloak>,
    /// Where it came into the smoke from.
    smoke_in: Option<V3>,
    cooldown: u16,
    dodge_left: u16,
    leap_left: u16,
    /// Frames to keep following the trail after a retreat.
    follow_left: u16,
    slop: i32,
    rng: u32,
    hop: Fx,
}

impl Veilstalker {
    pub fn new(who: usize, seed: u32, hop: Fx) -> Veilstalker {
        let mut p = Veilstalker {
            who,
            memory: Vec::with_capacity(REACTION + 1),
            at: 0,
            filled: 0,
            intent: WATCH,
            look: Fx::ZERO,
            looked: false,
            sweep: SWEEP,
            known: None,
            decloak: None,
            smoke_in: None,
            cooldown: 0,
            dodge_left: 0,
            leap_left: 0,
            follow_left: 0,
            slop: 0,
            rng: (0x9E37_79B9
                ^ (who as u32).wrapping_mul(0x85EB_CA6B)
                ^ seed.wrapping_mul(0x27D4_EB2F))
                | 1,
            hop,
        };
        p.roll_slop();
        p
    }

    fn recall(&self) -> &World {
        let len = self.memory.len();
        let back = REACTION.min(self.filled.saturating_sub(1));
        let idx = (self.at + len - 1 - back) % len;
        &self.memory[idx]
    }

    /// **A timed press is off by a share of the wait**: the further ahead a
    /// person has to judge an arrival, the further off the judgement -- a
    /// tenth of the frames being judged, either way, on top of the fixed
    /// three. Without it the hunter dodges a lunge on the same frame of its
    /// flight from any distance, which nobody does.
    fn judged(&mut self, wait: i32) -> i32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        let spread = (wait.max(0) / JUDGE) as u32;
        let off = (self.rng % (2 * spread + 1)) as i32 - spread as i32;
        self.slop + off
    }

    fn roll_slop(&mut self) {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        let span = (SLOP_LATE + SLOP_EARLY + 1) as u32;
        self.slop = (self.rng % span) as i32 - SLOP_EARLY;
    }
}

fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

fn unit(v: V3, fallback: V3) -> V3 {
    let v = flat(v);
    if v.flat_len().raw() > 0 {
        v.normalized()
    } else {
        fallback
    }
}

fn yaw_of(v: V3) -> Fx {
    atan2_turns(v.z, v.x)
}

fn scene_of<'a>(
    w: &'a World,
    stones: &'a sim::stones::Field,
    ground: &'a sim::arena::Terrain,
) -> Scene<'a> {
    Scene {
        stones,
        players: &w.players,
        effects: &w.effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: ground,
    }
}

/// The creature's slot, if there is one alive.
fn slot_of(w: &World) -> Option<usize> {
    (0..w.monsters.len())
        .find(|s| w.monsters[*s].is_some_and(|m| m.species == vs::SPECIES.id && m.alive()))
}

/// The middle of one part, where it is drawn.
fn part_at(m: &Monster, part: usize) -> V3 {
    let sh = vs::SPECIES.shape(part);
    m.world_of(part, sh.min.add(sh.max).scale(crate::HALF))
}

/// **What of the body is drawn on this screen**: the middle of the body if
/// any part is shown enough to see and in view, by the part it saw.
pub fn body_seen(w: &World, who: usize, view: Input, scene: &Scene) -> Option<V3> {
    let slot = slot_of(w)?;
    let m = w.monsters[slot]?;
    (0..vs::PART_COUNT)
        .filter(|p| w.shown(slot, *p).raw() >= SEE.raw())
        .map(|p| part_at(&m, p))
        .find(|at| in_view(who, view, *at, HALF_VIEW, scene))
}

/// Every lit paint mark on this screen, in the world.
fn paint_seen(w: &World, who: usize, view: Input, scene: &Scene) -> Option<V3> {
    let slot = slot_of(w)?;
    let m = w.monsters[slot]?;
    fight::paints(w)
        .filter(|p| fight::paint_strength(p).raw() > 0)
        .map(|p| fight::paint_at(&m, &p))
        .find(|at| in_view(who, view, *at, HALF_VIEW, scene))
}

/// A decloak this screen shows: the body's or a ghost's, alike.
fn decloak_seen(w: &World, who: usize, view: Input, scene: &Scene) -> Option<Apparition> {
    let mid = |a: &Apparition| a.at.add(V3::new(Fx::ZERO, Fx::ratio(15, 10), Fx::ZERO));
    fight::apparitions(w)
        .into_iter()
        .flatten()
        .find(|a| in_view(who, view, mid(a), HALF_VIEW, scene))
}

/// The prints on this screen, newest first.
fn prints_seen(w: &World, who: usize, view: Input, scene: &Scene) -> Vec<Print> {
    let mut out: Vec<Print> = fight::prints(w)
        .filter(|p| p.age <= fight::print_life(w, p))
        .filter(|p| in_view(who, view, p.at, HALF_VIEW, scene))
        .collect();
    out.sort_by_key(|p| p.age);
    out
}

/// The floor markers drawn this frame: the body's own and a ghost's.
fn markers(w: &World) -> Vec<sim::monster::Telegraph> {
    let mut out = Vec::new();
    if let Some(m) = slot_of(w).and_then(|s| w.monsters[s]) {
        if let Some(t) = m.telegraph().filter(|t| !(t.live && m.hit_used)) {
            out.push(t);
        }
    }
    if let Some(t) = fight::ghost(w).and_then(|g| g.telegraph()) {
        out.push(t);
    }
    out
}

/// Does a marker cover a body standing here?
fn covers(t: &sim::monster::Telegraph, at: V3, margin: Fx) -> bool {
    let end = t.anchor.add(t.along.scale(t.sweep));
    let gap = flat_segment_gap(at, t.anchor, end);
    gap.raw() <= t.radius.add(sim::tuning::body_radius()).add(margin).raw()
}

/// Is any point of a marker on this screen?
fn marker_in_view(t: &sim::monster::Telegraph, who: usize, view: Input, scene: &Scene) -> bool {
    let end = t.anchor.add(t.along.scale(t.sweep));
    [t.anchor, end, t.anchor.add(end).scale(crate::HALF)]
        .iter()
        .any(|p| in_view(who, view, *p, HALF_VIEW, scene))
}

/// A look as the wire carries it: a little down while it watches the floor,
/// the crosshair on a point when it throws something at it.
fn wire(me: &Player, yaw: Fx, at: V3, bits: u16) -> Input {
    let aim = turns_to_aim(yaw.sub(me.carry_yaw));
    let throwing = Input::LEFT | Input::RIGHT | Input::MIDDLE | Input::MECHANIC | Input::SPECIAL;
    let pitch = if bits & throwing != 0 {
        sim::aim::look_onto(me.pos, aim, me.aloft, at)
    } else {
        WATCH_PITCH
    };
    Input::looking_at(bits, aim, pitch)
}

/// Keep a direction inside the walls: along them rather than into them.
fn keep_in(w: &World, at: V3, dir: V3) -> V3 {
    let b = w.arena().bounds;
    let room = Fx::from_int(4);
    let ahead = at.add(dir.scale(Fx::from_int(2)));
    let out = ahead.x.raw() < b.lo_x.add(room).raw()
        || ahead.x.raw() > b.hi_x.sub(room).raw()
        || ahead.z.raw() < b.lo_z.add(room).raw()
        || ahead.z.raw() > b.hi_z.sub(room).raw();
    if !out {
        return dir;
    }
    let across = V3::new(dir.z.neg(), Fx::ZERO, dir.x);
    let middle = flat(V3::ZERO.sub(at));
    let side = if across.dot(middle).raw() >= 0 {
        across
    } else {
        across.scale(Fx::ONE.neg())
    };
    unit(side.add(unit(middle, side).scale(crate::HALF)), side)
}

/// Is there floor to go to that way: nothing standing in the next three
/// metres, and inside the walls?
fn clear(w: &World, ground: &sim::arena::Terrain, at: V3, dir: V3) -> bool {
    let b = w.arena().bounds;
    let room = Fx::from_int(2);
    (1..=3).all(|m| {
        let p = at.add(dir.scale(Fx::from_int(m)));
        let inside = p.x.raw() > b.lo_x.add(room).raw()
            && p.x.raw() < b.hi_x.sub(room).raw()
            && p.z.raw() > b.lo_z.add(room).raw()
            && p.z.raw() < b.hi_z.sub(room).raw();
        inside && ground.ground_under(p).raw() <= at.y.add(Fx::ratio(3, 10)).raw()
    })
}

/// The first of these ways that is clear, or the first if none is.
fn first_clear(w: &World, at: V3, ways: &[V3]) -> V3 {
    let ground = w.terrain();
    ways.iter()
        .copied()
        .find(|d| d.flat_len().raw() > 0 && clear(w, &ground, at, *d))
        .unwrap_or(ways[0])
}

/// The trunks in the arena: every standing solid narrow and tall.
fn trunks(w: &World) -> Vec<(V3, Fx)> {
    w.arena()
        .solids
        .iter()
        .filter(|s| {
            let wx = s.max.x.sub(s.min.x);
            let wz = s.max.z.sub(s.min.z);
            !s.hangs() && wx.raw() < Fx::from_int(3).raw() && wz.raw() < Fx::from_int(3).raw()
        })
        .map(|s| {
            let mid = s.min.add(s.max).scale(crate::HALF);
            (
                V3::new(mid.x, Fx::ZERO, mid.z),
                s.max.x.sub(s.min.x).max(s.max.z.sub(s.min.z)),
            )
        })
        .collect()
}

impl Plan for Veilstalker {
    fn watch(&mut self, w: &World) {
        if self.memory.len() < REACTION + 1 {
            self.memory.push(w.clone());
            self.at = self.memory.len() % (REACTION + 1);
        } else {
            self.memory[self.at] = w.clone();
            self.at = (self.at + 1) % self.memory.len();
        }
        self.filled = (self.filled + 1).min(REACTION + 1);
    }

    fn act(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        self.follow_left = self.follow_left.saturating_sub(1);
        let me = w.players[self.who];
        if !self.looked {
            self.look = yaw_of(me.facing);
            self.looked = true;
        }
        if !me.grounded || me.action.actionable() {
            self.leap_left = self.leap_left.saturating_sub(1);
        }
        if me.health <= 0 || !matches!(w.phase, Phase::Fighting) || self.memory.is_empty() {
            return Input::default();
        }
        let seen = self.recall().clone();
        let now = w.frame;
        let late = now.saturating_sub(seen.frame);
        let stones = sim::stones::gather(&seen.players);
        let ground = seen.terrain();
        let scene = scene_of(&seen, &stones, &ground);
        let view = wire(&me, self.look, me.pos, 0);

        // **What it can see**, a reaction ago.
        let body = body_seen(&seen, self.who, view, &scene);
        let paint = paint_seen(&seen, self.who, view, &scene);
        if let Some(at) = body.or(paint) {
            self.known = Some((flat(at), now));
        }
        if self
            .known
            .is_some_and(|(_, f)| now.saturating_sub(f) > STALE)
        {
            self.known = None;
        }
        let prints = prints_seen(&seen, self.who, view, &scene);

        // A decloak begun on its screen.
        if let Some(a) = decloak_seen(&seen, self.who, view, &scene) {
            let began = seen.frame.saturating_sub(a.frame as u32);
            let fresh = self
                .decloak
                .is_none_or(|d| d.began.abs_diff(began) > 3 || d.done && d.began != began);
            if fresh && self.decloak.is_none_or(|d| d.began != began) {
                // **Feet set under it**: a real decloak stamps four prints on
                // its first frame. Looked for in what it sees, at the frame
                // it sees the decloak begin.
                let feet = fight::prints(&seen).any(|p| {
                    p.age <= a.frame as u32 + 2 && wide_flat_dist(p.at, a.at).raw() <= FEET.raw()
                });
                self.decloak = Some(Decloak {
                    began,
                    at: a.at,
                    yaw: a.yaw,
                    rear: a.rear,
                    empty: !feet,
                    done: false,
                    judged: i32::MIN,
                });
                self.roll_slop();
            }
            if let Some(d) = self.decloak.as_mut() {
                d.at = a.at;
                d.yaw = a.yaw;
                if a.frame >= SILHOUETTE {
                    d.rear = a.rear;
                }
            }
            self.known = Some((flat(a.at), now));
        }
        // A decloak long over is forgotten.
        if self
            .decloak
            .is_some_and(|d| now.saturating_sub(d.began) > 150)
        {
            self.decloak = None;
        }

        if std::env::var_os("VEIL_DEBUG").is_some() {
            let m = slot_of(w).and_then(|s| w.monsters[s]);
            if let Some(m) = m {
                eprintln!(
                    "   veil: stalk {} plan {:.1} view {:b} edge {:.2} strikes {} d {:.1} think {} hits {}",
                    fight::stalk_left(&w.lore),
                    fight::plan_range(&w.lore).to_f32_for_render(),
                    fight::view_of(&w.lore, 0),
                    fight::edge_of(&w.lore, 0).to_f32_for_render(),
                    fight::strikes_this_engagement(&w.lore),
                    wide_flat_dist(m.pos, me.pos).to_f32_for_render(),
                    m.brain.think_left,
                    fight::hits(&m),
                );
            }
            eprintln!(
                "{} me ({:.1},{:.1}) hp {} {:?} look {:.2} | {:?} ({:.1},{:.1}) | known {:?} decloak {:?} prints {} intent {:?}",
                now,
                me.pos.x.to_f32_for_render(),
                me.pos.z.to_f32_for_render(),
                me.health,
                me.action,
                self.look.to_f32_for_render(),
                m.map(|m| m.doing),
                m.map_or(0.0, |m| m.pos.x.to_f32_for_render()),
                m.map_or(0.0, |m| m.pos.z.to_f32_for_render()),
                self.known.map(|(a, f)| (
                    a.x.to_f32_for_render(),
                    a.z.to_f32_for_render(),
                    now - f
                )),
                self.decloak.map(|d| (d.rear, d.empty, now - d.began)),
                prints.len(),
                self.intent,
            );
        }
        let _ = late;

        if self.leap_left > 0 {
            return self.turn_and(&me, None, V3::ZERO, Input::SPACE);
        }
        // 2. The answer to a decloak.
        if let Some(input) = self.answer(w, &seen, &me, &scene, view, now) {
            return input;
        }
        // 5. Out of the smoke.
        if let Some(input) = self.smoke(w, &seen, &me) {
            return input;
        }
        // 3. Punish what it can see that cannot answer.
        if let Some(input) = self.punish(w, &seen, &me, &scene, view) {
            return input;
        }
        // 3, the second half: the second hit, aimed at the lamp.
        if let Some(input) = self.on_the_paint(&seen, &me, &scene, view, paint) {
            return input;
        }
        // 6. A brazier across the trail.
        if let Some(input) = self.tip(w, &seen, &me, &prints) {
            return input;
        }
        // 1 and 4. Watch, read, follow.
        self.watch_floor(w, &seen, &me, &prints)
    }

    fn intent(&self) -> Intent {
        self.intent
    }
}

impl Veilstalker {
    /// Turn the camera toward `toward` at a person's rate, walk along `dir`,
    /// press `bits`.
    fn turn_and(&mut self, me: &Player, toward: Option<V3>, dir: V3, bits: u16) -> Input {
        if let Some(at) = toward {
            let want = yaw_of(flat(at.sub(me.pos)));
            let error = wrap_turns(want.sub(self.look));
            let step = TURN.mul(sim::DT);
            self.look = self.look.add(error.clamp(step.neg(), step));
        }
        let at = toward.unwrap_or(me.pos.add(V3::from_turns(self.look).scale(Fx::from_int(8))));
        let mut input = wire(me, self.look, at, bits);
        if dir.flat_len().raw() > 0 {
            input.bits |= steer(self.look, dir);
        }
        input
    }

    /// Is the camera on this point now?
    fn facing(&self, me: &Player, at: V3, within: Fx) -> bool {
        let want = yaw_of(flat(at.sub(me.pos)));
        wrap_turns(want.sub(self.look)).abs().raw() <= within.raw()
    }

    /// **The answer to a decloak** it saw begin: nothing for a mimic; for the
    /// rest, the silhouette's own answer, from frame eight.
    fn answer(
        &mut self,
        w: &World,
        seen: &World,
        me: &Player,
        scene: &Scene,
        view: Input,
        now: u32,
    ) -> Option<Input> {
        let d = self.decloak?;
        if d.done {
            return None;
        }
        let since = now.saturating_sub(d.began) as i32;
        let toward = Some(d.at.add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO)));
        // **A marker under it that is answered by where it stands** -- the
        // pounce's circle, the spear's lane -- is walked out of the moment it
        // is seen: the floor says it before the silhouette does.
        if let Some(input) = self.walk_out(w, seen, me, scene, view, toward) {
            return Some(input);
        }
        // **A decloak beside it is the rake**: nothing else is thrown from
        // that close, and the rake does not wait for its silhouette to be
        // read -- over it, now.
        let rake = vs::SPECIES.attack(vs::RAKE);
        let beside = wide_flat_dist(d.at, me.pos).raw()
            <= rake
                .hit_x
                .add(rake.hit_radius)
                .add(sim::tuning::body_radius())
                .add(MARGIN)
                .raw();
        if beside && !d.empty {
            self.intent = JUMP;
            if me.grounded && self.leap_left == 0 {
                self.leap_left = LEAP_HOLD;
                self.mark_done();
                return Some(self.turn_and(me, toward, V3::ZERO, Input::SPACE));
            }
            return Some(self.turn_and(me, toward, V3::ZERO, 0));
        }
        // Nothing under it: hold, and watch it.
        if d.empty && matches!(d.rear, vs::LUNGE | vs::SPEAR) {
            self.intent = HOLD;
            return Some(self.turn_and(me, toward, V3::ZERO, 0));
        }
        // Before the silhouette has said anything, it watches.
        if since < SILHOUETTE + REACTION as i32 {
            self.intent = HOLD;
            return Some(self.turn_and(me, toward, V3::ZERO, 0));
        }
        // The judgement of when it arrives is made once, as the silhouette
        // is read, and kept.
        if self.decloak.is_some_and(|d| d.judged == i32::MIN) {
            let a = vs::SPECIES.attack(d.rear);
            let wait = a.startup as i32 - since;
            let j = self.judged(wait.max(0) + REACTION as i32);
            if let Some(d) = self.decloak.as_mut() {
                d.judged = j;
            }
        }
        let judged = self.decloak.map_or(self.slop, |d| d.judged);
        let a = vs::SPECIES.attack(d.rear);
        let marks = markers(seen);
        let mine = marks
            .iter()
            .find(|t| t.kind == d.rear && marker_in_view(t, self.who, view, scene))
            .copied();
        let body = sim::tuning::body_radius();
        match d.rear {
            // **The lunge**: dodge at its arrival, across its line.
            vs::LUNGE => {
                let dist = wide_flat_dist(d.at, me.pos);
                let reach = a.hit_x.add(a.hit_radius).add(body);
                let speed = a.advance.mul(sim::DT).max(Fx::ratio(1, 100));
                let fly = dist.sub(reach).max(Fx::ZERO).div(speed).to_int();
                let arrive = a.startup as i32 + fly.min(a.active as i32 - 1);
                let go = arrive - 3 + judged;
                if since >= go && self.dodge_left == 0 && me.action.actionable() {
                    self.intent = DODGE;
                    self.dodge_left = sim::tuning::dodge_frames() + DODGE_REST;
                    let along = unit(me.pos.sub(d.at), V3::from_turns(self.look));
                    let side = V3::new(along.z.neg(), Fx::ZERO, along.x);
                    let side = first_clear(w, me.pos, &[side, side.scale(Fx::ONE.neg())]);
                    self.mark_done();
                    return Some(self.turn_and(me, toward, side, Input::SHIFT));
                }
                self.intent = DODGE;
                Some(self.turn_and(me, toward, V3::ZERO, 0))
            }
            // **The spear**: out of the lane, sideways, at a walk.
            vs::SPEAR => {
                let Some(t) = mine else {
                    self.intent = EVADE;
                    return Some(self.turn_and(me, toward, V3::ZERO, 0));
                };
                if !covers(&t, me.pos, MARGIN) {
                    if since > a.startup as i32 + a.active as i32 {
                        self.mark_done();
                    }
                    return None;
                }
                let side = V3::new(t.along.z.neg(), Fx::ZERO, t.along.x);
                let rel = flat(me.pos.sub(t.anchor));
                let out = if side.dot(rel).raw() >= 0 {
                    side
                } else {
                    side.scale(Fx::ONE.neg())
                };
                let out = first_clear(w, me.pos, &[out, out.scale(Fx::ONE.neg())]);
                self.intent = EVADE;
                Some(self.turn_and(me, toward, out, 0))
            }
            // **The rake**: over it.
            vs::RAKE => {
                let near = wide_flat_dist(d.at, me.pos).raw()
                    <= a.hit_x.add(a.hit_radius).add(body).add(MARGIN).raw();
                if !near {
                    self.mark_done();
                    return None;
                }
                self.intent = JUMP;
                if me.grounded && self.leap_left == 0 {
                    self.leap_left = LEAP_HOLD;
                    self.mark_done();
                    return Some(self.turn_and(me, toward, V3::ZERO, Input::SPACE));
                }
                Some(self.turn_and(me, toward, V3::ZERO, 0))
            }
            // **The pounce**: out of its circle, which does not follow.
            vs::POUNCE => {
                let t = marks.iter().find(|t| t.kind == vs::POUNCE).copied()?;
                if !covers(&t, me.pos, MARGIN) {
                    if since > a.startup as i32 + a.active as i32 {
                        self.mark_done();
                    }
                    return None;
                }
                let away = unit(me.pos.sub(t.anchor), V3::from_turns(self.look));
                let side = V3::new(away.z.neg(), Fx::ZERO, away.x);
                let out = first_clear(w, me.pos, &[away, side, side.scale(Fx::ONE.neg())]);
                self.intent = EVADE;
                Some(self.turn_and(me, toward, out, 0))
            }
            // **The quills**: behind the nearest trunk, or a dodge as they
            // arrive.
            vs::QUILLS => {
                let near = trunks(w)
                    .into_iter()
                    .filter(|(c, wd)| {
                        wide_flat_dist(*c, me.pos).raw()
                            <= COVER_NEAR.add(crate::HALF.mul(*wd)).raw()
                    })
                    .min_by_key(|(c, _)| wide_flat_dist(*c, me.pos).raw());
                let flight = wide_flat_dist(d.at, me.pos)
                    .div(fight_quill_speed().mul(sim::DT).max(Fx::ratio(1, 100)))
                    .to_int();
                let arrive = a.startup as i32 + flight;
                if since > arrive + 10 {
                    self.mark_done();
                    return None;
                }
                if let Some((c, wd)) = near {
                    // The far side of the trunk from it.
                    let behind =
                        c.add(unit(c.sub(d.at), V3::ZERO).scale(crate::HALF.mul(wd).add(Fx::ONE)));
                    self.intent = COVER;
                    let dir = unit(behind.sub(me.pos), V3::ZERO);
                    let there = wide_flat_dist(behind, me.pos).raw() < Fx::ratio(4, 10).raw();
                    return Some(self.turn_and(me, toward, if there { V3::ZERO } else { dir }, 0));
                }
                let go = arrive - 3 + judged;
                if since >= go && self.dodge_left == 0 && me.action.actionable() {
                    self.intent = DODGE;
                    self.dodge_left = sim::tuning::dodge_frames() + DODGE_REST;
                    let along = unit(me.pos.sub(d.at), V3::from_turns(self.look));
                    let side = V3::new(along.z.neg(), Fx::ZERO, along.x);
                    let side = first_clear(w, me.pos, &[side, side.scale(Fx::ONE.neg())]);
                    self.mark_done();
                    return Some(self.turn_and(me, toward, side, Input::SHIFT));
                }
                self.intent = DODGE;
                Some(self.turn_and(me, toward, V3::ZERO, 0))
            }
            _ => None,
        }
    }

    /// **Out of a circle or a lane** it can see under it: away from the
    /// circle's middle, square off the lane.
    fn walk_out(
        &mut self,
        w: &World,
        seen: &World,
        me: &Player,
        scene: &Scene,
        view: Input,
        toward: Option<V3>,
    ) -> Option<Input> {
        let t = markers(seen).into_iter().find(|t| {
            matches!(t.kind, vs::POUNCE | vs::SPEAR)
                && covers(t, me.pos, MARGIN)
                && marker_in_view(t, self.who, view, scene)
        })?;
        let out = if t.kind == vs::POUNCE {
            let away = unit(me.pos.sub(t.anchor), V3::from_turns(self.look));
            let side = V3::new(away.z.neg(), Fx::ZERO, away.x);
            first_clear(w, me.pos, &[away, side, side.scale(Fx::ONE.neg())])
        } else {
            let side = V3::new(t.along.z.neg(), Fx::ZERO, t.along.x);
            let rel = flat(me.pos.sub(t.anchor));
            let out = if side.dot(rel).raw() >= 0 {
                side
            } else {
                side.scale(Fx::ONE.neg())
            };
            first_clear(w, me.pos, &[out, out.scale(Fx::ONE.neg())])
        };
        self.intent = EVADE;
        Some(self.turn_and(me, toward, out, 0))
    }

    fn mark_done(&mut self) {
        if let Some(d) = self.decloak.as_mut() {
            d.done = true;
        }
    }

    /// **Out of the smoke**, at right angles to the way it came in.
    fn smoke(&mut self, w: &World, seen: &World, me: &Player) -> Option<Input> {
        let inside = sim::hazard::all(&seen.lore).find(|(_, h)| {
            h.index() == fight::CLOUD
                && wide_flat_dist(h.centre(), me.pos).raw() <= h.radius().raw()
        });
        let Some((_, h)) = inside else {
            self.smoke_in = Some(me.pos);
            return None;
        };
        let from = self.smoke_in.unwrap_or(h.centre());
        let came = unit(me.pos.sub(from), V3::from_turns(self.look));
        let side = V3::new(came.z.neg(), Fx::ZERO, came.x);
        let rel = flat(me.pos.sub(h.centre()));
        let out = if side.dot(rel).raw() >= 0 {
            side
        } else {
            side.scale(Fx::ONE.neg())
        };
        let out = keep_in(
            w,
            me.pos,
            first_clear(w, me.pos, &[out, out.scale(Fx::ONE.neg())]),
        );
        self.intent = LEAVE;
        Some(self.turn_and(me, None, out, 0))
    }

    /// **Hit what it can see that cannot answer**: a recovery, a flinch, a
    /// stagger, a panic -- and the paint, while a decloak's lunge or spear has
    /// not begun.
    fn punish(
        &mut self,
        w: &World,
        seen: &World,
        me: &Player,
        scene: &Scene,
        view: Input,
    ) -> Option<Input> {
        let slot = slot_of(seen)?;
        let m = seen.monsters[slot]?;
        let open = matches!(
            m.doing,
            Doing::Recovery { .. }
                | Doing::Flinch { .. }
                | Doing::Stumble { .. }
                | Doing::Toppled { .. }
        ) && !matches!(
            m.doing.attacking(),
            Some(vs::RETREAT | vs::CLIMB | vs::SMOKE | vs::MIMIC)
        );
        if !open {
            return None;
        }
        // It has to be seen to be punished.
        let at = body_seen(seen, self.who, view, scene)?;
        let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
        let poke_busy = (poke.startup + poke.active + poke.recovery) as i32;
        let heavy_busy = crate::heavy_commitment(me.class) as i32;
        let reach = poke.reach.add(Fx::ONE);
        let target = fight::middle_of(&m);
        let window = m.frames_until_free() as i32 - REACTION as i32;
        if window <= poke_busy + EXIT {
            return None;
        }
        let d = wide_flat_dist(target, me.pos);
        let walk_frames = d
            .sub(reach)
            .max(Fx::ZERO)
            .div(sim::tuning::move_speed().mul(sim::DT))
            .to_int();
        if walk_frames + poke_busy + EXIT > window {
            return None;
        }
        self.intent = if matches!(m.doing, Doing::Toppled { .. }) {
            UNLOAD
        } else {
            PUNISH
        };
        let _ = (at, w);
        let dir = if d.raw() > reach.raw() {
            unit(target.sub(me.pos), V3::ZERO)
        } else {
            V3::ZERO
        };
        let ready = self.cooldown == 0
            && me.action.actionable()
            && d.raw() <= reach.raw()
            && self.facing(me, target, Fx::ratio(3, 100));
        let swing = if ready {
            self.cooldown = SWING_GAP;
            if window - walk_frames > heavy_busy + EXIT {
                heavy(me.class)
            } else {
                Input::LEFT
            }
        } else {
            0
        };
        Some(self.turn_and(me, Some(target), dir, swing))
    }

    /// **Unload into the second hit on the paint** (§9.3): a lit mark it can
    /// see, near enough to reach before the creature could start anything it
    /// has not already been seen to start -- no decloak under way -- is hit
    /// where it hangs.
    fn on_the_paint(
        &mut self,
        seen: &World,
        me: &Player,
        scene: &Scene,
        view: Input,
        paint: Option<V3>,
    ) -> Option<Input> {
        let at = paint.or_else(|| body_seen(seen, self.who, view, scene))?;
        if fight::apparitions(seen).iter().flatten().next().is_some() {
            return None;
        }
        let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
        let reach = poke.reach.add(Fx::ONE);
        let d = wide_flat_dist(at, me.pos);
        // Not across the arena: a few strides at most.
        if d.raw() > reach.add(Fx::from_int(4)).raw() {
            return None;
        }
        self.intent = PUNISH;
        let dir = if d.raw() > reach.raw() {
            unit(at.sub(me.pos), V3::ZERO)
        } else {
            V3::ZERO
        };
        let ready = self.cooldown == 0
            && me.action.actionable()
            && d.raw() <= reach.raw()
            && self.facing(me, at, Fx::ratio(3, 100));
        let swing = if ready {
            self.cooldown = SWING_GAP;
            Input::LEFT
        } else {
            0
        };
        Some(self.turn_and(me, Some(at), dir, swing))
    }

    /// **A brazier across the trail**: when the newest prints head toward a
    /// standing brazier within six metres of their head, go and tip it.
    fn tip(&mut self, w: &World, seen: &World, me: &Player, prints: &[Print]) -> Option<Input> {
        let (head, dir) = trail_head(prints)?;
        let brazier = fight::braziers_of(seen)
            .filter(|(i, _)| fight::brazier_standing(&seen.lore, *i))
            .map(|(_, at)| at)
            .filter(|at| {
                let ahead = flat(at.sub(head));
                wide_flat_dist(*at, head).raw() <= TIP_NEAR.raw() && ahead.dot(dir).raw() > 0
            })
            .min_by_key(|at| wide_flat_dist(*at, me.pos).raw())?;
        // Only worth it if it can get there first: nearer than the trail.
        if wide_flat_dist(brazier, me.pos).raw() > Fx::from_int(8).raw() {
            return None;
        }
        self.intent = TIP;
        let reach = sim::moves::get(me.class, sim::state::SLOT_POKE).reach;
        let d = wide_flat_dist(brazier, me.pos);
        let dir = if d.raw() > reach.raw() {
            unit(brazier.sub(me.pos), V3::ZERO)
        } else {
            V3::ZERO
        };
        let target = brazier.add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO));
        let swing = if self.cooldown == 0
            && me.action.actionable()
            && d.raw() <= reach.add(Fx::ratio(5, 10)).raw()
            && self.facing(me, target, Fx::ratio(3, 100))
        {
            self.cooldown = SWING_GAP;
            Input::LEFT
        } else {
            0
        };
        let _ = w;
        Some(self.turn_and(me, Some(target), dir, swing))
    }

    /// **Watch the floor**: face any fresh print within twelve metres; follow
    /// the trail after a retreat; otherwise stand in the open and sweep.
    fn watch_floor(&mut self, w: &World, seen: &World, me: &Player, prints: &[Print]) -> Input {
        let slot = slot_of(seen);
        // A retreat seen: follow, for a while.
        if let Some(m) = slot.and_then(|s| seen.monsters[s]) {
            if m.doing.attacking() == Some(vs::RETREAT)
                && body_seen(
                    seen,
                    self.who,
                    wire(me, self.look, me.pos, 0),
                    &scene_of(seen, &sim::stones::gather(&seen.players), &seen.terrain()),
                )
                .is_some()
            {
                self.follow_left = 300;
            }
        }
        let fresh = prints
            .iter()
            .find(|p| p.age <= FRESH && wide_flat_dist(p.at, me.pos).raw() <= READ_FAR.raw());
        if self.follow_left > 0 {
            if let Some((head, _)) = trail_head(prints) {
                self.intent = FOLLOW;
                let d = wide_flat_dist(head, me.pos);
                let dir = if d.raw() > Fx::from_int(6).raw() {
                    keep_in(w, me.pos, unit(head.sub(me.pos), V3::ZERO))
                } else {
                    V3::ZERO
                };
                return self.turn_and(me, Some(head), dir, 0);
            }
        }
        if let Some(p) = fresh {
            self.intent = READ;
            return self.turn_and(me, Some(p.at), V3::ZERO, 0);
        }
        // Out from under a perch: into the open.
        let near_trunk = trunks(w)
            .into_iter()
            .filter(|(c, _)| wide_flat_dist(*c, me.pos).raw() < OPEN_FROM.raw())
            .min_by_key(|(c, _)| wide_flat_dist(*c, me.pos).raw());
        let mut dir = V3::ZERO;
        if let Some((c, _)) = near_trunk {
            self.intent = OPEN;
            dir = keep_in(w, me.pos, unit(me.pos.sub(c), V3::from_turns(self.look)));
        } else {
            self.intent = WATCH;
            // Nothing too near a wall either: back toward the middle.
            let b = w.arena().bounds;
            let room = Fx::from_int(7);
            let out = me.pos.x.raw() < b.lo_x.add(room).raw()
                || me.pos.x.raw() > b.hi_x.sub(room).raw()
                || me.pos.z.raw() < b.lo_z.add(room).raw()
                || me.pos.z.raw() > b.hi_z.sub(room).raw();
            if out {
                dir = unit(V3::ZERO.sub(me.pos), V3::ZERO);
            }
        }
        // The sweep: round where it last knew the creature was, or round
        // the middle of the arena.
        let centre = self
            .known
            .map(|(at, _)| at)
            .unwrap_or_else(|| me.pos.add(V3::from_turns(self.look).scale(Fx::from_int(8))));
        let base = yaw_of(flat(centre.sub(me.pos)));
        let off = wrap_turns(self.look.sub(base));
        if off.raw() > SWEEP_WIDE.raw() {
            self.sweep = SWEEP.neg();
        } else if off.raw() < SWEEP_WIDE.neg().raw() {
            self.sweep = SWEEP;
        }
        self.look = self.look.add(self.sweep.mul(sim::DT));
        self.turn_and(me, None, dir, 0)
    }
}

/// The head of a trail: the newest print, and the way the newest few head.
fn trail_head(prints: &[Print]) -> Option<(V3, V3)> {
    let newest = prints.first()?;
    if newest.age > 240 {
        return None;
    }
    let older = prints
        .iter()
        .find(|p| wide_flat_dist(p.at, newest.at).raw() >= STEP.raw() && p.age > newest.age)?;
    let dir = unit(newest.at.sub(older.at), V3::ZERO);
    Some((newest.at, dir))
}

/// The quills' speed, live.
fn fight_quill_speed() -> Fx {
    vs::Knob::QuillSpeed.fx()
}

/// The jump height, for whoever wants it.
pub fn hop_of(p: &Veilstalker) -> Fx {
    p.hop
}

// ---------------------------------------------------------------------------
// The report's lines
// ---------------------------------------------------------------------------

/// What the report counts about the Veilstalker (`veilstalker.md` §9).
#[derive(Default)]
pub struct VeilTally {
    /// The strike in progress: whom it is at, the look they had as it began,
    /// how many of its decloak's frames it was on that screen, its kind, and
    /// whether that hunter was in its cloud.
    commit: Option<(usize, Input, u32, u8)>,
    /// Hits landed; those whose decloak was on screen under fifteen frames.
    hits: u32,
    blind: u32,
    /// Hits taken inside its smoke.
    smoke_hits: u32,
    /// Mimics played, dodged at, held.
    mimics: u32,
    mimic_dodged: u32,
    mimic_open: Option<u32>,
    mimic_was_dodged: bool,
    /// From a retreat to the next hit on it or its next decloak.
    retreat_at: Option<u32>,
    finds: Vec<u32>,
    /// From the first print within ten metres of the hunter to the decloak.
    first_print: Option<u32>,
    leads: Vec<u32>,
    /// Frames with a lit paint mark; frames fought.
    paint_frames: u32,
    fought: u32,
    /// Engagements, and those in which a second hit landed.
    engagements: u32,
    follow_ups: u32,
    engaged: bool,
    /// The frame each region mottled.
    mottled_at: [Option<u32>; vs::REGIONS],
    /// Panics seen begin.
    panics: u32,
    /// Strikes thrown, by kind; decloaks seen.
    decloaks: u32,
    retreats: u32,
    kept: u32,
    frame: u32,
    /// The lore at the end, for its own counts.
    panics_by: [u32; 5],
    thirds: [u32; 3],
    burnt: u32,
}

impl Tally for VeilTally {
    fn observe(&mut self, before: &World, after: &World) {
        self.observe_with(before, after, &[]);
    }

    fn observe_with(&mut self, before: &World, after: &World, bots: &[Hunter]) {
        self.frame = after.frame;
        let Some(slot) = slot_of(after).or_else(|| slot_of(before)) else {
            return;
        };
        let (Some(was), Some(now)) = (before.monsters[slot], after.monsters[slot]) else {
            return;
        };
        let stones = sim::stones::gather(&after.players);
        let ground = after.terrain();
        let scene = scene_of(after, &stones, &ground);
        let look = |who: usize| bots.iter().find(|h| h.who == who).map(|h| h.last);
        if now.brain.grace == 0 {
            self.fought += 1;
        }
        if fight::paints(after).next().is_some() {
            self.paint_frames += 1;
        }
        let who = (now.brain.target as usize).min(after.players.len() - 1);
        let began = |k: u8| matches!(now.doing, Doing::Startup { kind, left } if kind == k && left == vs::SPECIES.attack(k).startup);
        // A strike beginning: its decloak's frames on its target's screen.
        if let Doing::Startup { kind, left } = now.doing {
            if fight::strikes(kind) && kind != vs::MIMIC && left == vs::SPECIES.attack(kind).startup
            {
                self.decloaks += 1;
                if let Some(v) = look(who) {
                    self.commit = Some((who, v, 0, kind));
                } else {
                    self.commit = Some((who, Input::default(), u32::MAX / 2, kind));
                }
                if let Some(at) = self.retreat_at.take() {
                    self.finds.push(after.frame.saturating_sub(at));
                }
                if let Some(at) = self.first_print.take() {
                    self.leads.push(after.frame.saturating_sub(at));
                }
            }
        }
        if let (Some((target, v, seen, kind)), Some((k, e))) = (self.commit, fight::elapsed(&now)) {
            if k == kind && e <= fight::decloak(kind) && seen < u32::MAX / 2 {
                // On the screen: its middle, its head or its hips.
                let rig = now.rig();
                let points = [
                    fight::middle_of(&now),
                    rig.bone[vs::bones::HEAD].at,
                    rig.bone[vs::bones::ROOT].at,
                ];
                if points
                    .iter()
                    .any(|p| in_view(target, v, *p, HALF_VIEW, &scene))
                {
                    self.commit = Some((target, v, seen + 1, kind));
                }
            }
        }
        // The mimic: dodged at, or held.
        if began(vs::MIMIC) {
            self.mimics += 1;
            self.mimic_open = Some(after.frame);
            self.mimic_was_dodged = false;
        }
        if let Some(at) = self.mimic_open {
            let dodged = matches!(after.players[who].action, Action::Dodge { .. })
                && !matches!(before.players[who].action, Action::Dodge { .. });
            if dodged && !self.mimic_was_dodged {
                self.mimic_was_dodged = true;
                self.mimic_dodged += 1;
            }
            let span = vs::SPECIES.attack(vs::MIMIC).startup as u32 + REACTION as u32;
            if after.frame.saturating_sub(at) > span {
                self.mimic_open = None;
            }
        }
        // The retreat, and the burst that keeps it.
        if began(vs::RETREAT) {
            self.retreats += 1;
            self.retreat_at = Some(after.frame);
        }
        if matches!(
            was.doing,
            Doing::Startup {
                kind: vs::RETREAT,
                ..
            }
        ) && matches!(now.doing, Doing::Flinch { .. })
        {
            self.kept += 1;
        }
        if std::env::var_os("VEIL_EVENTS").is_some() {
            let pl = &after.players[0];
            if now.health < was.health {
                eprintln!(
                    "{:6} HIT IT {:4} doing {:?} hits {} shown {:.2} d {:.1}",
                    after.frame,
                    was.health - now.health,
                    was.doing,
                    fight::hits(&now),
                    after.shown(slot, vs::BARREL_L).to_f32_for_render(),
                    wide_flat_dist(now.pos, pl.pos).to_f32_for_render()
                );
            }
            if pl.health < before.players[0].health {
                eprintln!(
                    "{:6} HUNTER HIT {:4} by {:?} hunter {:?}",
                    after.frame,
                    before.players[0].health - pl.health,
                    now.doing,
                    before.players[0].action
                );
            }
            if let Doing::Startup { kind, left } = now.doing {
                if left == vs::SPECIES.attack(kind).startup && was.doing.attacking() != Some(kind) {
                    eprintln!(
                        "{:6} START {} d {:.1} stalk {}",
                        after.frame,
                        vs::MOVES[kind as usize].name,
                        wide_flat_dist(now.pos, pl.pos).to_f32_for_render(),
                        fight::stalk_left(&after.lore)
                    );
                }
            }
        }
        // A hit on it: the end of a hunt for it, and a second hit is a
        // follow-up.
        if now.health < was.health {
            if let Some(at) = self.retreat_at.take() {
                self.finds.push(after.frame.saturating_sub(at));
            }
        }
        let engaged = fight::hits(&now) > 0;
        if engaged && !self.engaged {
            self.engagements += 1;
        }
        if fight::hits(&now) == 2 && fight::hits(&was) == 1 {
            self.follow_ups += 1;
        }
        if began(vs::RETREAT) && !self.engaged {
            // Hits counted reset as it leaves: the engagement's second hit
            // was the one that made it go.
        }
        self.engaged = engaged || matches!(now.doing.attacking(), Some(vs::RETREAT));
        // The first print near the hunter since the last decloak.
        if self.first_print.is_none() {
            let me = after.players[0].pos;
            let near = fight::prints(after)
                .any(|p| p.age == 0 && wide_flat_dist(p.at, me).raw() <= Fx::from_int(10).raw());
            if near {
                self.first_print = Some(after.frame);
            }
        }
        // Mottling.
        for r in 0..vs::REGIONS {
            if self.mottled_at[r].is_none() && fight::mottled(&now, r) {
                self.mottled_at[r] = Some(after.frame);
            }
        }
        if matches!(now.doing, Doing::Toppled { .. }) && !matches!(was.doing, Doing::Toppled { .. })
        {
            self.panics += 1;
        }
        // A hit landing on a hunter.
        let landed = !was.hit_used && now.hit_used;
        if landed {
            let kind = now.doing.attacking().unwrap_or(0);
            let hurt: Vec<usize> = (0..after.players.len())
                .filter(|i| after.players[*i].health < before.players[*i].health)
                .collect();
            if !hurt.is_empty() && vs::SPECIES.attack(kind).damage > 0 {
                self.hits += 1;
                let in_smoke = fight::cloud(after).is_some_and(|(c, r, _)| {
                    hurt.iter()
                        .any(|i| wide_flat_dist(c, after.players[*i].pos).raw() <= r.raw())
                });
                if in_smoke {
                    self.smoke_hits += 1;
                } else if let Some((_, _, seen, k)) = self.commit {
                    if k == kind && seen < REACTION as u32 {
                        self.blind += 1;
                        if std::env::var_os("VEIL_DEBUG").is_some() {
                            eprintln!("BLIND HIT {} kind {} seen {}", after.frame, kind, seen);
                        }
                    }
                }
            }
        }
        self.panics_by = fight::panics(&after.lore);
        self.thirds = fight::thirds(&after.lore);
        self.burnt = fight::smokes_burnt(&after.lore);
    }

    fn lines(&self) -> Vec<(String, String, String)> {
        let pct = |a: u32, b: u32| {
            if b == 0 {
                "--".to_string()
            } else {
                format!("{:.0}%", 100.0 * a as f32 / b as f32)
            }
        };
        let mean = |v: &[u32]| {
            if v.is_empty() {
                "--".to_string()
            } else {
                format!(
                    "{:.1} s",
                    v.iter().sum::<u32>() as f32 / v.len() as f32 / 60.0
                )
            }
        };
        let mottled: Vec<String> = (0..vs::REGIONS)
            .filter_map(|r| {
                self.mottled_at[r]
                    .map(|f| format!("{} at {:.0} s", vs::REGION_NAMES[r], f as f32 / 60.0))
            })
            .collect();
        let causes: Vec<String> = fight::FireFrom::ALL
            .iter()
            .zip(self.panics_by.iter())
            .filter(|(_, n)| **n > 0)
            .map(|(c, n)| format!("{} {}", n, c.name()))
            .collect();
        vec![
            (
                "blind hits".into(),
                format!("{} of {}", self.blind, self.hits),
                "decloak on screen under 15f at commit: must be zero".into(),
            ),
            (
                "smoke hits".into(),
                self.smoke_hits.to_string(),
                "taken inside its cloud: answered by leaving it".into(),
            ),
            (
                "mimics: dodged at / held".into(),
                format!(
                    "{} / {}",
                    self.mimic_dodged,
                    self.mimics.saturating_sub(self.mimic_dodged)
                ),
                "a dodge at one means a mimic stamped a print".into(),
            ),
            (
                "time to find".into(),
                mean(&self.finds),
                "a retreat to the next hit on it or its next decloak".into(),
            ),
            (
                "print lead".into(),
                mean(&self.leads),
                "first print within 10 m to the decloak".into(),
            ),
            (
                "paint uptime".into(),
                pct(self.paint_frames, self.fought),
                "frames with a lit mark".into(),
            ),
            (
                "follow-up rate".into(),
                format!("{} of {}", self.follow_ups, self.engagements),
                "engagements where the second hit landed".into(),
            ),
            (
                "retreats, kept".into(),
                format!("{}, {}", self.retreats, self.kept),
                "and bursts in the recoil that kept it in the fight".into(),
            ),
            (
                "mottled".into(),
                if mottled.is_empty() {
                    "none".into()
                } else {
                    mottled.join(", ")
                },
                "regions that never cloak again, and when".into(),
            ),
            (
                "panics".into(),
                if causes.is_empty() {
                    self.panics.to_string()
                } else {
                    format!("{} ({})", self.panics, causes.join(", "))
                },
                "the big window, and what fire caused it".into(),
            ),
            (
                "decloaks by view".into(),
                format!(
                    "{} centre, {} middle, {} edge",
                    self.thirds[0], self.thirds[1], self.thirds[2]
                ),
                "all edge means `EdgeBias` is too high".into(),
            ),
            (
                "smoke burnt off".into(),
                self.burnt.to_string(),
                "clouds fire took".into(),
            ),
        ]
    }

    fn unanswerable(&self) -> u32 {
        self.blind
    }

    /// **The soonest it can hit** (§9): a stalk is threatening only once it
    /// is in range and in view; until then it has its stalk, then a decloak,
    /// before it can touch anybody. And a retreat is not an attack: its
    /// recoil, its bound and its re-cloak are all time it cannot hit in.
    fn until_free(&self, w: &World, slot: usize, free: u16) -> u16 {
        let Some(m) = w.monsters[slot] else {
            return free;
        };
        let floor = vs::Knob::DecloakFloor.raw().max(0) as u16;
        let stalk = fight::stalk_left(&w.lore) as u16;
        let target = (m.brain.target as usize).min(1);
        let ready = fight::view_of(&w.lore, target) & fight::view::IN_VIEW != 0
            && wide_flat_dist(m.pos, m.brain.seen).raw()
                <= vs::SPECIES
                    .attack(vs::QUILLS)
                    .ideal_range
                    .add(vs::SPECIES.attack(vs::QUILLS).range_span)
                    .raw();
        match m.doing {
            Doing::Prowl if fight::perched(&m) => free.saturating_add(floor),
            Doing::Prowl => {
                let wait = if stalk > 0 || !ready {
                    stalk.max(free).saturating_add(STALE as u16)
                } else {
                    free
                };
                wait.saturating_add(floor)
            }
            Doing::Startup { kind, left }
            | Doing::Active { kind, left }
            | Doing::Recovery { kind, left }
                if matches!(
                    kind,
                    vs::RETREAT | vs::GETUP | vs::CLIMB | vs::SMOKE | vs::MIMIC
                ) =>
            {
                let a = vs::SPECIES.attack(kind);
                let rest = match m.doing {
                    Doing::Startup { .. } => left + a.active + a.recovery,
                    Doing::Active { .. } => left + a.recovery,
                    _ => left,
                };
                let after = if kind == vs::RETREAT || kind == vs::GETUP {
                    vs::Knob::StalkMin.raw().max(0) as u16
                } else {
                    0
                };
                rest.saturating_add(after).saturating_add(floor)
            }
            _ => free,
        }
    }
}

impl VeilTally {
    pub fn blind(&self) -> u32 {
        self.blind
    }
}

fn make_tally() -> Box<dyn Tally> {
    Box::new(VeilTally::default())
}

fn make(who: usize, seed: u32, hop: Fx) -> Box<dyn Plan + Send + Sync> {
    Box::new(Veilstalker::new(who, seed, hop))
}

/// Nobody rides it.
fn bucks(_kind: u8) -> bool {
    false
}

pub static CARD: crate::plans::Card = crate::plans::Card {
    species: sim::species::SpeciesId::VEILSTALKER,
    plan: make,
    bucks,
    words: crate::plans::Words {
        weak_hits: "weak-point hits (none: its hide is four regions, none weak)",
        broken: "parts broken (none)",
        into_breakables: "damage into breakable parts",
        into_breakables_why: "none: its lasting consequence is the mottle",
        worst: "worst part",
        ride_for: "rides (nobody rides it)",
        toppled_pool: "while panicked",
    },
    tally: Some(make_tally),
    gamble: None,
};
