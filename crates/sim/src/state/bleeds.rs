//! **The Blood mage: the world bleeds** (2026-11-04,
//! `docs/design/exploration/0038_blood_mage_the_world_bleeds.md`, slice 1).
//!
//! Whatever she cuts bleeds -- a fighter, a creature, the floor -- and where
//! blood falls something grows. Five pieces, each a few functions here:
//!
//! - **The Cast** (right held): the scythe's head on its chain
//!   (`EffectKind::ScytheHead`), flying the crosshair's line until it embeds
//!   in what it meets. A fighter it bites is bled, slowed and **leashed**; the
//!   world it bites bleeds a pool. Left alone it wrenches free and drags home
//!   along the floor, drinking her stains and reaping her plants on its line.
//! - **The zip** (right pressed again while it holds): she is hauled to the
//!   head, keeps a share of the speed, and a jump pressed near the end fires
//!   on arrival.
//! - **The spike from a pool, and the plant it leaves** (`class::Garden`):
//!   only a pool rises; what it raises launches what is over it -- her too --
//!   and stands, solid, lashing an enemy beside it.
//! - **Reap**: the scythe through a plant breaks it into a cone of thorns.
//! - **Seize**: the Red Hand's grab, which feeds them to a plant near them or
//!   slams them into the floor.
//!
//! A child of `state` so it reaches the world's own private machinery -- the
//! cut, the spill, the drink -- rather than growing a second copy of it.

use super::*;
use crate::class::{Garden, MAX_STRUCTURES, Structure};
use crate::effects::Head;

/// What a Seize does with the body it caught.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Seized {
    /// Thrown on to the plant in this slot of her garden.
    Impaled(usize),
    /// Slammed into the floor.
    Slammed,
}

/// Her garden, if she has one.
pub(super) fn garden_of(p: &Player) -> Option<Garden> {
    match p.mechanic {
        Mechanic::Blood(g) => Some(g),
        _ => None,
    }
}

/// Change her garden, if she has one.
fn tend(p: &mut Player, f: impl FnOnce(&mut Garden)) {
    if let Mechanic::Blood(g) = &mut p.mechanic {
        f(g);
    }
}

/// The flat unit vector from `from` to `to`, or nothing when they are one
/// above the other.
fn flat_toward(from: V3, to: V3) -> V3 {
    V3::new(to.x.sub(from.x), Fx::ZERO, to.z.sub(from.z)).normalized()
}

/// The flat distance between two points.
fn flat_apart(a: V3, b: V3) -> Fx {
    V3::new(a.x.sub(b.x), Fx::ZERO, a.z.sub(b.z)).flat_len()
}

/// **The nearest unlit stain of `owner`'s that a disc of `reach` at `at`
/// touches**, level with it -- what a spike raises. The question
/// `World::pool_touching` asks, answered off any effects table, so that the
/// press can ask it of the frame's scene before anything is thrown.
pub(super) fn pool_touching_in(
    effects: &[Option<Effect>; MAX_EFFECTS],
    owner: u8,
    at: V3,
    reach: Fx,
) -> Option<usize> {
    let mut best: Option<(usize, i32)> = None;
    for (slot, e) in effects.iter().enumerate() {
        let Some(e) = e else { continue };
        if !e.is_a_pool() || e.owner != owner || e.life != UNLIT {
            continue;
        }
        let far = flat_apart(at, e.pos);
        let level = at.y.sub(e.pos.y).abs().raw() <= e.pool_height().raw();
        if level
            && far.raw() <= e.pool_radius().add(reach).raw()
            && best.is_none_or(|(_, b)| far.raw() < b)
        {
            best = Some((slot, far.raw()));
        }
    }
    best.map(|(slot, _)| slot)
}

/// **How long a pool of `volume` takes to rise** (2026-11-04): a small one in
/// `tuning::spike_rise_small`, a fighter's (`tuning::pool_full`) in
/// `spike_rise_body`, a big one in `spike_rise_big`, and in between along a
/// straight line. Small grows fast, big grows slow.
pub(super) fn rise_of(volume: i32) -> u16 {
    let small = t::spike_small_pool().max(1);
    let body = t::pool_full().max(small + 1);
    let big = t::spike_big_pool().max(body + 1);
    let (from, to, a, b) = if volume <= small {
        return t::spike_rise_small().max(1);
    } else if volume <= body {
        (small, body, t::spike_rise_small(), t::spike_rise_body())
    } else if volume <= big {
        (body, big, t::spike_rise_body(), t::spike_rise_big())
    } else {
        return t::spike_rise_big().max(1);
    };
    let share = Fx::ratio(volume - from, to - from);
    let frames = Fx::from_int(a as i32).add(Fx::from_int(b as i32 - a as i32).mul(share));
    frames.to_int().clamp(1, u16::MAX as i32) as u16
}

/// **Does this press raise anything?** The Black spike and the Rising spike
/// raise a pool of hers -- under the crosshair, or at her own feet -- and on
/// bare ground nothing comes out: the press does nothing and costs nothing.
///
/// `None` is "nothing comes out"; `Some(None)` is a move that is not a spike;
/// `Some(Some(frames))` is a spike, and how long its pool takes to rise --
/// its startup, in place of its row's.
pub(super) fn spike_rise(
    p: &Player,
    who: usize,
    kind: u8,
    input: Input,
    scene: &Scene,
) -> Option<Option<u16>> {
    if p.class != Class::BloodMage || !moves::blood::raises_a_pool(kind) {
        return Some(None);
    }
    let m = moves::get(p.class, kind);
    let slot = if kind == moves::blood::RISING_SPIKE {
        pool_touching_in(scene.effects, who as u8, p.pos, t::body_radius())
    } else {
        let mut aimed = *p;
        lock_aim(&mut aimed, who, kind, input, scene);
        pool_touching_in(scene.effects, who as u8, aimed.aim_at(), m.radius)
    }?;
    let volume = scene.effects[slot].map_or(0, |e| e.pool_volume());
    Some(Some(rise_of(volume)))
}

/// **The zip** (2026-11-04): right click pressed again while her scythe's
/// head holds something hauls her to it -- to a fighter, a creature, a wall
/// or the floor, in the air as on the ground -- and never the other way. The
/// haul is `begin_haul`'s, at `tuning::zip_speed`; for its length she is in
/// the Cast's recovery, and arriving frees her (`World::tend_the_zips`).
/// Returns whether it began, so the press throws nothing else.
pub(super) fn zip_to_the_head(p: &mut Player, who: usize, pressed: bool, scene: &Scene) -> bool {
    if p.class != Class::BloodMage || !pressed || p.health <= 0 {
        return false;
    }
    let free = p.action.actionable()
        || matches!(p.action, Action::Recovery { kind, .. } if kind == moves::blood::CAST);
    if !free || garden_of(p).is_some_and(|g| g.zipping) {
        return false;
    }
    let Some(head) = scene
        .effects
        .iter()
        .flatten()
        .find(|e| e.owner == who as u8 && e.is_a_head_holding())
    else {
        return false;
    };
    let to = zip_target(p.pos, head.home, head.head_state());
    let far = to.sub(p.pos).len();
    let speed = t::zip_speed().max(Fx::ONE);
    let frames = far.div(speed).div(DT).to_int().clamp(0, 600) as u16;
    begin_haul(p, to, speed, frames.saturating_add(2));
    p.grounded = false;
    p.swim = SWIM_NONE;
    p.action = Action::Recovery {
        kind: moves::blood::CAST,
        left: frames.saturating_add(2),
    };
    tend(p, |g| {
        g.zipping = true;
        g.zip_jump = false;
    });
    true
}

/// Where the zip puts her: the head, short of it by a body when it is in a
/// fighter or a creature or a wall, so she arrives beside what it bit rather
/// than inside it.
fn zip_target(from: V3, head: V3, state: Head) -> V3 {
    let back = from.sub(head);
    let far = back.len();
    let short = match state {
        Head::Fighter(_) | Head::Creature(_) => t::body_radius().add(t::body_radius()),
        _ => t::body_radius(),
    };
    if far.raw() <= short.raw() {
        return from;
    }
    head.add(back.scale(short.div(far)))
}

/// A jump pressed while the zip is still running: kept for the arrival if she
/// is within `tuning::zip_jump_buffer` frames of the head, and thrown the
/// frame she gets there -- the Reaver's dash jump. Earlier than that it is
/// dropped.
pub(super) fn bank_the_zip_jump(p: &mut Player) {
    if p.haul == 0 || !garden_of(p).is_some_and(|g| g.zipping) {
        return;
    }
    let reach = p
        .haul_speed
        .mul(DT)
        .mul(Fx::from_int(t::zip_jump_buffer() as i32));
    if p.haul_to.sub(p.pos).len().raw() <= reach.raw() {
        tend(p, |g| g.zip_jump = true);
    }
}

/// Put a plant into her garden, the oldest making way when it is full -- the
/// Elementalist's cap, which is `stones::raise`'s rule.
fn plant(p: &mut Player, stone: Structure, blood: u16, paid: u16) {
    tend(p, |g| {
        let slot = match g.plants.iter().position(|s| s.is_none()) {
            Some(free) => free,
            None => {
                g.plants.rotate_left(1);
                g.blood.rotate_left(1);
                g.paid.rotate_left(1);
                MAX_STRUCTURES - 1
            }
        };
        g.plants[slot] = Some(stone);
        g.blood[slot] = blood;
        g.paid[slot] = paid;
    });
}

impl World {
    // -- The Cast ----------------------------------------------------------

    /// **The Cast comes out** (2026-11-04): the head leaves her hand along
    /// the line `aim::skillshot_path` solved, for the point the crosshair is
    /// on. When that point is the floor's -- raised to a body's middle, as a
    /// skillshot's floor is -- the head goes down into the floor there, which
    /// is where the player pointed. Otherwise it flies on, out to the row's
    /// reach, headed for the first solid thing on the line
    /// (`aim::first_along`: the world and its stones) -- or for nothing, past
    /// which it comes home. Bodies are met on the way, each frame
    /// (`step_the_head`). One head at a time.
    pub(super) fn throw_the_head(&mut self, i: usize, mut born: Effect) {
        let p = self.players[i];
        let m = moves::get(p.class, moves::blood::CAST);
        let stones = stones::gather(&self.players);
        let seen = self.players;
        let effects = self.effects;
        let terrain = self.terrain();
        let scene = aim::Scene {
            stones: &stones,
            players: &seen,
            effects: &effects,
            quarry: &self.monsters,
            critters: &self.critters,
            arena: &terrain,
        };
        let aimed = p.aim_path.to;
        let floor = terrain.ground_under(aimed);
        let short = p.aim_path.length().raw() < m.reach.sub(t::body_radius()).raw();
        let on_the_floor = short && aimed.y.sub(floor).raw() <= t::body_height().raw();
        let (dir, reach, solid) = if on_the_floor {
            let to = V3::new(aimed.x, floor, aimed.z);
            let line = to.sub(born.pos);
            (line.normalized(), line.len(), true)
        } else {
            let path = aim::Path {
                from: born.pos,
                to: born.pos.add(born.dir.scale(m.reach)),
            };
            let hit = aim::first_along(
                path,
                m.radius,
                i as u8,
                &scene,
                Targets::none().terrain().stones(),
            );
            (born.dir, hit.map_or(m.reach, |c| c.dist()), hit.is_some())
        };
        born.dir = dir;
        born.reach = reach;
        born.home = born.pos;
        born.life = u16::MAX;
        born.set_head(Head::Flying);
        // Flying at something solid: the low bit of the frame count, which a
        // flying head does not otherwise use.
        if solid {
            born.hold_the_head();
        }
        for slot in self.effects.iter_mut() {
            if slot.is_some_and(|e| e.kind == EffectKind::ScytheHead && e.owner == i as u8) {
                *slot = None;
            }
        }
        tend(&mut self.players[i], |g| g.zipping = false);
        spawn_effect(&mut self.effects, born);
    }

    /// **The head, one frame.** Flying, it meets a fighter, a creature, a
    /// critter or the world. Holding, it counts to `tuning::cast_holds` and
    /// wrenches free. Coming home, it drags along the floor. See the module.
    pub(super) fn step_the_head(&mut self, effect: &mut Effect) {
        let owner = effect.owner as usize;
        if owner >= MAX_PLAYERS || self.players[owner].health <= 0 {
            effect.life = effect.age;
            return;
        }
        // The chain runs from her hand, live: `pos` is it this frame.
        let her = self.players[owner];
        let hand = moves::get(her.class, effect.slot).hand;
        effect.pos = aim::hand_origin(her.pos, her.facing, hand);
        match effect.head_state() {
            Head::Flying => self.fly_the_head(effect),
            Head::Fighter(v) => {
                effect.hold_the_head();
                let victim = self.players[v];
                effect.home = aim::origin(victim.pos);
                if victim.health <= 0 {
                    effect.set_head(Head::Home);
                } else if effect.head_for() >= t::cast_holds() {
                    self.wrench_free(effect, v);
                }
            }
            Head::World | Head::Creature(_) => {
                effect.hold_the_head();
                if effect.head_for() >= t::cast_holds() {
                    effect.set_head(Head::Home);
                }
            }
            // She is on her way to it; `tend_the_zips` ends it when she gets
            // there, or sends it home when she does not.
            Head::Zipping => {
                if !garden_of(&self.players[owner]).is_some_and(|g| g.zipping) {
                    effect.set_head(Head::Home);
                }
            }
            Head::Home => self.drag_home(effect),
        }
    }

    /// The head in flight: out along its line at `tuning::cast_speed`, and
    /// bitten into the first body it reaches, or the world at the end. Its
    /// `reach` is how much of the flight is left.
    fn fly_the_head(&mut self, effect: &mut Effect) {
        let owner = effect.owner as usize;
        let solid = effect.head_for() > 0;
        let step = t::cast_speed().mul(DT).min(effect.reach);
        effect.reach = effect.reach.sub(step);
        let at = effect.home.add(effect.dir.scale(step));
        effect.home = at;
        let spent = effect.reach.raw() <= 0;
        let radius = effect.field_radius();
        // A fighter on the line.
        for v in 0..MAX_PLAYERS {
            if !self.effects_reach(v, effect.owner)
                || effect.already_hit(0, v)
                || !self.inside(v, at, radius)
            {
                continue;
            }
            effect.take_hit(0, v);
            let dealt = self.cut(v, effect, at, effect.source().damage);
            if dealt > 0 {
                self.players[v].bleed(effect.owner, t::bleed_lasts());
                let apart = flat_apart(self.players[owner].pos, self.players[v].pos);
                effect.reach = apart.max(t::cast_least_leash());
                effect.home = aim::origin(self.players[v].pos);
                effect.set_head(Head::Fighter(v));
            } else {
                // Guarded: it glances off, and comes home.
                effect.set_head(Head::Home);
            }
            return;
        }
        // A creature: it bleeds as the Haemorrhage's did, and holds there.
        let cut = self.gore_each(effect, 0, at, radius, Fx::ONE);
        if let Some(slot) = cut.iter().position(|d| *d > 0) {
            if let Some(beast) = self.monsters[slot].as_mut() {
                beast.bleeding = beast.bleeding.max(t::bleed_lasts());
                beast.bled_by = effect.owner;
            }
            effect.reach = flat_apart(self.players[owner].pos, at).max(t::cast_least_leash());
            effect.set_head(Head::Creature(slot));
            return;
        }
        // A critter, or the floor under the head: the world, here.
        let floor = self.terrain().ground_under(at);
        let into_the_floor = at.y.raw() <= floor.raw();
        if self.gore_critters(effect, at, radius, Fx::ONE) > 0 || into_the_floor || (solid && spent)
        {
            self.bite_the_world(effect, at, floor);
            return;
        }
        if spent {
            effect.set_head(Head::Home);
        }
    }

    /// **The head bites the world** at `at`: a wall, a stone, the floor. It
    /// holds there, down on the floor when it was within a body's height of
    /// it, and **the world bleeds** -- `tuning::cast_bleeds` of blood on the
    /// floor under the bite.
    fn bite_the_world(&mut self, effect: &mut Effect, at: V3, floor: Fx) {
        let owner = effect.owner as usize;
        let near_the_floor = at.y.sub(floor).raw() <= t::body_height().raw();
        let bite = if near_the_floor {
            V3::new(at.x, floor, at.z)
        } else {
            at
        };
        effect.home = bite;
        effect.reach = flat_apart(self.players[owner].pos, bite).max(t::cast_least_leash());
        effect.set_head(Head::World);
        self.bleed_the_world(
            owner,
            effect.slot,
            V3::new(at.x, floor, at.z),
            t::cast_bleeds(),
        );
    }

    /// **Wrenched free**: the fighter it held is staggered for
    /// `tuning::cast_stagger` and shoved toward her at `cast_shove` -- not
    /// all the way -- and the head starts home.
    fn wrench_free(&mut self, effect: &mut Effect, v: usize) {
        let owner = effect.owner as usize;
        let her = self.players[owner].pos;
        let q = &mut self.players[v];
        if !q.action.invulnerable() {
            let toward = flat_toward(q.pos, her);
            q.action = Action::Stagger {
                left: t::cast_stagger(),
            };
            q.stun_total = t::cast_stagger();
            q.vel.x = toward.x.mul(t::cast_shove());
            q.vel.z = toward.z.mul(t::cast_shove());
        }
        effect.set_head(Head::Home);
    }

    /// **The head drags home** along the floor under the chain, at
    /// `tuning::cast_drag`, **reaping everything on its line**: every stain of
    /// hers it crosses is drunk (the refund, never past her grey), and every
    /// plant of hers it crosses bursts into thorns the way it was going. Gone
    /// when it reaches her.
    fn drag_home(&mut self, effect: &mut Effect) {
        let owner = effect.owner as usize;
        let her = self.players[owner].pos;
        let was = effect.home;
        let gap = flat_apart(her, was);
        let step = t::cast_drag().mul(DT);
        let terrain = self.terrain();
        let next = if gap.raw() <= step.raw() {
            her
        } else {
            let ahead = flat_toward(was, her).scale(step);
            let moved = was.add(ahead);
            let lift = V3::new(moved.x, moved.y.add(t::body_height()), moved.z);
            V3::new(moved.x, terrain.ground_under(lift), moved.z)
        };
        effect.home = next;
        let along = flat_toward(was, next);
        let m = moves::get(Class::BloodMage, moves::blood::CAST);
        let reach = effect.field_radius().add(t::body_radius());
        // Her stains on the line.
        for slot in 0..MAX_EFFECTS {
            let crossed = self.effects[slot].is_some_and(|e| {
                e.is_a_pool()
                    && !e.is_world_blood()
                    && e.owner == owner as u8
                    && e.age > t::stain_settles()
                    && e.life == UNLIT
                    && crate::math::segment_gap(
                        V3::new(was.x, e.pos.y, was.z),
                        V3::new(next.x, e.pos.y, next.z),
                        e.pos,
                        e.pos,
                    )
                    .raw()
                        <= reach.add(e.pool_radius()).raw()
            });
            if crossed {
                self.drink_from(slot, owner, &m);
            }
        }
        // Her plants on the line.
        if let Some(g) = garden_of(&self.players[owner]) {
            for (slot, plant) in g.plants.iter().enumerate() {
                let Some(plant) = plant else { continue };
                let gap = crate::math::segment_gap(
                    V3::new(was.x, plant.at.y, was.z),
                    V3::new(next.x, plant.at.y, next.z),
                    plant.at,
                    plant.at,
                );
                if gap.raw() <= reach.add(plant.radius()).raw() {
                    self.reap_the_plant(owner, slot, along);
                }
            }
        }
        if flat_apart(her, next).raw() <= t::body_radius().add(t::body_radius()).raw() {
            effect.life = effect.age;
        }
    }

    /// **The leash** (2026-11-04): while her head holds a fighter, neither
    /// body may get further from the other than the chain's length -- fixed
    /// at the bite -- or than they already were. Each is held against where
    /// the other **was** at the top of the frame, so a body standing still is
    /// never dragged; only the one walking away is stopped, and the speed it
    /// was putting into the chain is spent. Flat: the chain does not hold
    /// anybody up. The held one is slowed while it holds.
    pub(super) fn hold_the_leashes(&mut self, seen: &[Player; MAX_PLAYERS]) {
        for slot in 0..MAX_EFFECTS {
            let Some(e) = self.effects[slot] else {
                continue;
            };
            if e.kind != EffectKind::ScytheHead {
                continue;
            }
            let Head::Fighter(v) = e.head_state() else {
                continue;
            };
            let o = e.owner as usize;
            if o >= MAX_PLAYERS || v >= MAX_PLAYERS {
                continue;
            }
            self.players[v].slow(t::slow_frames(), t::cast_slow());
            for (x, y) in [(o, v), (v, o)] {
                let anchor = seen[y].pos;
                let before = flat_apart(seen[x].pos, anchor);
                let limit = e.reach.max(before);
                let now = self.players[x].pos;
                let far = flat_apart(now, anchor);
                if far.raw() <= limit.raw() || far.raw() == 0 {
                    continue;
                }
                let out = flat_toward(anchor, now);
                let q = &mut self.players[x];
                q.pos.x = anchor.x.add(out.x.mul(limit));
                q.pos.z = anchor.z.add(out.z.mul(limit));
                let away = q.vel.x.mul(out.x).add(q.vel.z.mul(out.z));
                if away.raw() > 0 {
                    q.vel.x = q.vel.x.sub(out.x.mul(away));
                    q.vel.z = q.vel.z.sub(out.z.mul(away));
                }
            }
        }
    }

    /// **The zips, after the step**: a zip that has begun takes its head
    /// (which stops counting toward the wrench), a head in a body keeps her
    /// haul pointed at it, and one that has arrived lets her go -- with
    /// `tuning::zip_keeps` of the speed, and the jump banked near the end
    /// (`bank_the_zip_jump`) thrown. A zip a hit broke sends its head home.
    pub(super) fn tend_the_zips(&mut self, seen: &[Player; MAX_PLAYERS]) {
        for i in 0..MAX_PLAYERS {
            let Some(g) = garden_of(&self.players[i]) else {
                continue;
            };
            if !g.zipping {
                continue;
            }
            let head = self.effects.iter().position(|e| {
                e.is_some_and(|e| e.kind == EffectKind::ScytheHead && e.owner == i as u8)
            });
            let Some(head) = head else {
                tend(&mut self.players[i], |g| g.zipping = false);
                continue;
            };
            let Some(mut e) = self.effects[head] else {
                continue;
            };
            let state = e.head_state();
            if state.holds() {
                e.set_head(Head::Zipping);
                // Which body it is in, for the haul to follow: kept in the
                // hold count, which a zipping head does not use.
                let follow = match state {
                    Head::Fighter(v) => v as i32 + 1,
                    _ => 0,
                };
                e.banked |= follow << 8;
            }
            let p = self.players[i];
            if p.haul > 0 {
                // Follow a fighter it is in.
                let follow = e.head_for() as usize;
                if follow > 0 && follow <= MAX_PLAYERS {
                    let body = self.players[follow - 1].pos;
                    e.home = aim::origin(body);
                    let to = zip_target(p.pos, e.home, Head::Fighter(follow - 1));
                    self.players[i].haul_to = to;
                }
                self.effects[head] = Some(e);
                continue;
            }
            // The haul is over: arrived, or broken.
            let was = seen[i];
            let arrived =
                was.haul > 0 && p.pos.sub(was.haul_to).len().raw() <= t::body_radius().raw();
            tend(&mut self.players[i], |g| {
                g.zipping = false;
                g.zip_jump = false;
            });
            if arrived {
                let run = was.haul_to.sub(was.pos).normalized();
                let q = &mut self.players[i];
                q.vel = run.scale(was.haul_speed.mul(t::zip_keeps()));
                if g.zip_jump || garden_of(&was).is_some_and(|g| g.zip_jump) {
                    let jump = t::jump_speed().mul(q.class.mobility().jump);
                    q.vel.y = q.vel.y.max(Fx::ZERO).add(jump);
                    q.jump_hold = t::jump_hold_frames();
                }
                q.grounded = false;
                if matches!(q.action, Action::Recovery { kind, .. } if kind == moves::blood::CAST)
                {
                    q.action = Action::Free;
                }
                self.effects[head] = None;
            } else {
                e.set_head(Head::Home);
                self.effects[head] = Some(e);
            }
        }
    }

    // -- The world bleeds ---------------------------------------------------

    /// **The world bleeds** (2026-11-04): `volume` of blood on the floor at
    /// `at`, a pool of hers for a spike to raise -- but not hers to drink.
    /// The heal stays a refund: what she paid, never what the floor gave.
    pub(super) fn bleed_the_world(&mut self, owner: usize, slot: u8, at: V3, volume: i32) {
        if owner >= MAX_PLAYERS {
            return;
        }
        let class = self.players[owner].class;
        self.pool_blood_as(owner as u8, class, slot, at, volume, true);
    }

    /// **The floor a grounded Sweep cuts bleeds** (2026-11-04): on its first
    /// active frame, `tuning::sweep_bleeds` at the middle of its arc, on the
    /// floor -- so the scythe into the floor and then the spike works with
    /// nobody there. The Rising reap cuts the floor under her as she leaves
    /// it, `tuning::rising_reap_bleeds`.
    pub(super) fn the_floor_bleeds(&mut self, i: usize, kind: u8) {
        let p = self.players[i];
        let terrain = self.terrain();
        let (at, volume) = if kind == moves::blood::SWEEP {
            if !p.grounded {
                return;
            }
            let m = moves::get(p.class, kind);
            let half = Fx::ratio(1, 2);
            let ahead = V3::new(p.facing.x, Fx::ZERO, p.facing.z).normalized();
            (
                p.pos.add(ahead.scale(live_reach(&p, &m).mul(half))),
                t::sweep_bleeds(),
            )
        } else {
            (p.pos, t::rising_reap_bleeds())
        };
        let lift = V3::new(at.x, at.y.add(t::body_height()), at.z);
        let floor = terrain.ground_under(lift);
        if at.y.sub(floor).raw() > t::body_height().raw() {
            return;
        }
        self.bleed_the_world(i, kind, V3::new(at.x, floor, at.z), volume);
    }

    // -- The spike and the garden --------------------------------------------

    /// **A spike rises out of a pool** (2026-11-04): the pool of hers that a
    /// disc of `reach` at `at` touches erupts (`erupt_from`, worth its blood
    /// as before), and **stands as a plant** where it was, holding what the
    /// spike cost her. It launches whoever is over it -- the eruption's own
    /// blow -- **her too**, at `tuning::spike_launches_her`: her way up. With
    /// no pool there (it drained while it rose), nothing.
    pub(super) fn raise_from_the_pool(&mut self, i: usize, kind: u8, at: V3, reach: Fx) {
        let Some(slot) = self.pool_touching(i as u8, at, reach) else {
            return;
        };
        let Some(pool) = self.effects[slot] else {
            return;
        };
        self.erupt_from(i, kind, slot);
        let paid = self.players[i].paid;
        self.players[i].paid = 0;
        let stone = Structure::planted(pool.pos);
        // Her, over it.
        let p = self.players[i];
        let over = flat_apart(p.pos, pool.pos).raw()
            <= stone
                .radius()
                .max(pool.pool_radius())
                .add(t::body_radius())
                .raw()
            && p.pos.y.sub(pool.pos.y).abs().raw() <= t::body_height().raw();
        plant(&mut self.players[i], stone, paid, paid);
        if over {
            let q = &mut self.players[i];
            q.pos.y = q.pos.y.max(stone.top());
            q.vel.y = q.vel.y.max(t::spike_launches_her());
            q.grounded = false;
            q.jump_hold = 0;
        }
    }

    /// **Her garden, one frame** (2026-11-04): every plant ages; one that has
    /// stood `tuning::plant_stands` withers, and what it held with it. Every
    /// `plant_lash_every` frames each one **lashes** every enemy within
    /// `plant_lash_reach` of its edge -- the plant's lash's row, and a slow --
    /// and the creature and the critters as well.
    pub(super) fn step_the_gardens(&mut self) {
        for i in 0..MAX_PLAYERS {
            let Some(g) = garden_of(&self.players[i]) else {
                continue;
            };
            let mut kept = g;
            for slot in 0..MAX_STRUCTURES {
                let Some(mut stone) = g.plants[slot] else {
                    continue;
                };
                stone.age = stone.age.saturating_add(1);
                if stone.age >= t::plant_stands() || self.players[i].health <= 0 {
                    kept.plants[slot] = None;
                    kept.blood[slot] = 0;
                    kept.paid[slot] = 0;
                    continue;
                }
                kept.plants[slot] = Some(stone);
                if stone.age % t::plant_lash_every().max(1) == 0 {
                    self.lash(i, stone);
                }
            }
            tend(&mut self.players[i], |g| *g = kept);
        }
    }

    /// One plant's lash: every enemy body within `tuning::plant_lash_reach`
    /// of its edge, at its height.
    fn lash(&mut self, owner: usize, stone: Structure) {
        let m = moves::get(Class::BloodMage, moves::blood::LASH);
        let reach = stone.radius().add(t::plant_lash_reach());
        for v in 0..MAX_PLAYERS {
            let q = self.players[v];
            if !self.effects_reach(v, owner as u8)
                || q.untouchable()
                || flat_apart(q.pos, stone.at).raw() > reach.add(t::body_radius()).raw()
                || q.pos.y.raw() > stone.top().add(t::body_height()).raw()
                || q.pos.y.add(t::body_height()).raw() < stone.at.y.raw()
            {
                continue;
            }
            let (guarding, parried) = guard_against(&q, stone.at, m.unblockable);
            let dealt = if guarding || parried {
                0
            } else {
                m.damage.min(q.health)
            };
            apply_hit(
                &mut self.players[v],
                Hit {
                    damage: m.damage,
                    hitstun: m.hitstun,
                    blockstun: m.blockstun,
                    knockback: m.knockback,
                    launch: m.launch,
                    grabs: 0,
                    by: owner as u8,
                    dir: flat_toward(stone.at, q.pos),
                    blocked: guarding,
                    parried,
                    interrupts: true,
                },
            );
            if dealt > 0 {
                self.players[v].slow(t::slow_frames(), t::plant_lash_slow());
                self.spill_under(owner as u8, Class::BloodMage, moves::blood::LASH, &q, dealt);
            }
        }
        let mut touch = Effect::cast(
            EffectKind::Nova,
            owner as u8,
            Class::BloodMage,
            moves::blood::LASH,
            stone.at,
            V3::ZERO,
            reach,
        );
        let half = Fx::ratio(1, 2);
        let mid = V3::new(stone.at.x, stone.at.y.add(stone.height().mul(half)), stone.at.z);
        self.gore_each(&mut touch, 0, mid, reach, Fx::ONE);
        self.gore_critters(&mut touch, mid, reach, Fx::ONE);
    }

    /// **Reap** (2026-11-04): the plant in `slot` of her garden breaks, and
    /// its thorns fly `along` in a cone (`tuning::thorns_reach`,
    /// `thorns_cone`), each enemy in it struck once for the thorns' row and
    /// all the blood the plant held. What she paid into it she drinks back,
    /// never past her grey.
    pub(super) fn reap_the_plant(&mut self, owner: usize, slot: usize, along: V3) {
        let Some(g) = garden_of(&self.players[owner]) else {
            return;
        };
        let Some(stone) = g.plants[slot] else {
            return;
        };
        let (blood, paid) = (g.blood[slot] as i32, g.paid[slot] as i32);
        tend(&mut self.players[owner], |g| {
            g.plants[slot] = None;
            g.blood[slot] = 0;
            g.paid[slot] = 0;
        });
        let dir = if along.len_sq().raw() == 0 {
            flat_toward(self.players[owner].pos, stone.at)
        } else {
            V3::new(along.x, Fx::ZERO, along.z).normalized()
        };
        let m = moves::get(Class::BloodMage, moves::blood::THORNS);
        let damage = m.damage + blood;
        let reach = t::thorns_reach();
        let cone = t::thorns_cone();
        for v in 0..MAX_PLAYERS {
            let q = self.players[v];
            if !self.effects_reach(v, owner as u8) || q.untouchable() {
                continue;
            }
            let off = V3::new(q.pos.x.sub(stone.at.x), Fx::ZERO, q.pos.z.sub(stone.at.z));
            let far = off.flat_len();
            let in_cone = far.raw() <= reach.add(t::body_radius()).raw()
                && (far.raw() <= stone.radius().add(t::body_radius()).raw()
                    || off.dot(dir).raw() >= far.mul(cone).raw());
            let level = q.pos.y.sub(stone.at.y).abs().raw() <= reach.raw();
            if !in_cone || !level {
                continue;
            }
            let (guarding, parried) = guard_against(&q, stone.at, m.unblockable);
            let dealt = if guarding || parried {
                0
            } else {
                damage.min(q.health)
            };
            apply_hit(
                &mut self.players[v],
                Hit {
                    damage,
                    hitstun: m.hitstun,
                    blockstun: m.blockstun,
                    knockback: m.knockback,
                    launch: m.launch,
                    grabs: 0,
                    by: owner as u8,
                    dir,
                    blocked: guarding,
                    parried,
                    interrupts: true,
                },
            );
            if dealt > 0 {
                self.spill_under(owner as u8, Class::BloodMage, moves::blood::THORNS, &q, dealt);
            }
        }
        // The creature and the critters along the cone's middle.
        let mut thorns = Effect::cast(
            EffectKind::Cascade,
            owner as u8,
            Class::BloodMage,
            moves::blood::THORNS,
            stone.at,
            dir,
            reach,
        );
        thorns.life = t::spell_flash();
        let share = Fx::ratio(damage, m.damage.max(1));
        let half = Fx::ratio(1, 2);
        let mid = stone
            .at
            .add(dir.scale(reach.mul(half)))
            .add(V3::new(Fx::ZERO, t::body_height().mul(half), Fx::ZERO));
        self.gore_each(&mut thorns, 0, mid, reach.mul(half), share);
        self.gore_critters(&mut thorns, mid, reach.mul(half), share);
        thorns.forget_hits();
        spawn_effect(&mut self.effects, thorns);
        if paid > 0 {
            self.players[owner].absorb(paid);
        }
    }

    /// **The scythe through a plant reaps it**: a Sweep or a Rising reap
    /// whose volume passes through one of her plants on an active frame
    /// breaks it, its thorns flying out from her through it.
    pub(super) fn reap_with_the_scythe(&mut self, who: usize) {
        let p = self.players[who];
        let Some(kind) = p.action.attack_kind() else {
            return;
        };
        if p.class != Class::BloodMage || !moves::blood::scythe(kind) {
            return;
        }
        if !matches!(p.action, Action::Active { .. }) {
            return;
        }
        let Some(hb) = hitbox(&p) else {
            return;
        };
        let Some(g) = garden_of(&p) else {
            return;
        };
        for (slot, plant) in g.plants.iter().enumerate() {
            let Some(plant) = plant else { continue };
            let top = V3::new(plant.at.x, plant.top(), plant.at.z);
            let gap = crate::math::segment_gap(hb.from, hb.to, plant.at, top);
            if gap.raw() <= hb.radius.add(plant.radius()).raw() {
                self.reap_the_plant(who, slot, flat_toward(p.pos, plant.at));
            }
        }
    }

    // -- Seize -------------------------------------------------------------

    /// **What her Seize does with who it caught** (2026-11-04): thrown on to
    /// the nearest plant of hers within `tuning::seize_plant_reach` of them,
    /// or slammed into the floor. `None` for every other blow. Sets the blow
    /// to the ending's row: the impaling worth its row and the blood the
    /// plant holds, and a launch; the slam worth its row, driven down from
    /// the air (`tuning::seize_drive`).
    pub(super) fn seize_how(&self, attacker: usize, defender: usize, hit: &mut Hit) -> Option<Seized> {
        let p = self.players[attacker];
        if p.class != Class::BloodMage
            || p.action.attack_kind() != Some(moves::blood::SEIZE)
            || hit.blocked
            || hit.parried
        {
            return None;
        }
        let q = self.players[defender];
        let g = garden_of(&p)?;
        let near = g
            .plants
            .iter()
            .enumerate()
            .filter_map(|(slot, s)| s.map(|s| (slot, flat_apart(s.at, q.pos))))
            .filter(|(_, far)| far.raw() <= t::seize_plant_reach().raw())
            .min_by_key(|(_, far)| far.raw());
        let (how, row) = match near {
            Some((slot, _)) => (Seized::Impaled(slot), moves::blood::IMPALE),
            None => (Seized::Slammed, moves::blood::SLAM),
        };
        let m = moves::get(p.class, row);
        hit.damage = m.damage;
        hit.hitstun = m.hitstun;
        hit.knockback = m.knockback;
        hit.launch = m.launch;
        hit.grabs = 0;
        hit.interrupts = true;
        match how {
            Seized::Impaled(slot) => {
                hit.damage += g.blood[slot] as i32;
            }
            Seized::Slammed if !q.grounded => {
                hit.launch = Fx::ZERO.sub(t::seize_drive());
            }
            Seized::Slammed => {}
        }
        Some(how)
    }

    /// **The Seize lands**: impaled, they are put on the plant's top and it
    /// drinks what the blow dealt; slammed, their blood lands in a ring round
    /// them rather than under them.
    pub(super) fn seize_lands(&mut self, attacker: usize, defender: usize, how: Seized, dealt: i32) {
        match how {
            Seized::Impaled(slot) => {
                let Some(g) = garden_of(&self.players[attacker]) else {
                    return;
                };
                let Some(stone) = g.plants[slot] else {
                    return;
                };
                let q = &mut self.players[defender];
                q.pos = V3::new(stone.at.x, stone.top(), stone.at.z);
                let drank = dealt.clamp(0, u16::MAX as i32) as u16;
                tend(&mut self.players[attacker], |g| {
                    g.blood[slot] = g.blood[slot].saturating_add(drank);
                });
                self.credit(attacker as u8, dealt);
            }
            Seized::Slammed => {
                if dealt <= 0 {
                    return;
                }
                self.credit(attacker as u8, dealt);
                let q = self.players[defender];
                let terrain = self.terrain();
                let floor = terrain.ground_under(q.pos);
                let ring = t::seize_ring();
                let each = (dealt / RING_POOLS as i32).max(1);
                for k in 0..RING_POOLS {
                    let turn = Fx::ratio(k as i32, RING_POOLS as i32);
                    let round = V3::from_turns(crate::math::wrap_turns(turn));
                    let at = V3::new(
                        q.pos.x.add(round.x.mul(ring)),
                        floor,
                        q.pos.z.add(round.z.mul(ring)),
                    );
                    let lift = V3::new(at.x, at.y.add(t::body_height()), at.z);
                    let at = V3::new(at.x, terrain.ground_under(lift), at.z);
                    self.pool_blood(
                        attacker as u8,
                        Class::BloodMage,
                        moves::blood::SLAM,
                        at,
                        each,
                    );
                }
            }
        }
    }
}

/// How many pools a slam's ring of blood is.
const RING_POOLS: usize = 4;
