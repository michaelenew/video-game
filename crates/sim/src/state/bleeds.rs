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

/// **What a spike press raises** (2026-11-06): the pool of hers under the
/// crosshair -- or at her own feet, for the Rising spike -- and, on bare
/// ground, **her own blood**: the price is the row's cost, and a pool pays
/// what it holds toward it, so she pays only what the pool lacks (0038 §"The
/// heal stays a refund"). The press always raises something.
///
/// `Some(None)` is a move that is not a spike; `Some(Some((frames,
/// volume)))` is a spike: how long it takes to rise -- its startup, in
/// place of its row's -- and how much blood the pool already holds toward
/// the price. Bare ground rises as a fighter's pool does.
pub(super) fn spike_rise(
    p: &Player,
    who: usize,
    kind: u8,
    input: Input,
    scene: &Scene,
) -> Option<Option<(u16, i32)>> {
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
    };
    Some(Some(match slot {
        Some(slot) => {
            let volume = scene.effects[slot].map_or(0, |e| e.pool_volume());
            (rise_of(volume), volume)
        }
        None => (t::spike_rise_body().max(1), 0),
    }))
}

/// **A pool pays toward the spike**: what she paid on the press, less what
/// the pool already held, is given back -- she pays only what it lacks.
pub(super) fn the_pool_pays(p: &mut Player, volume: i32) {
    let back = volume.min(p.paid as i32).max(0);
    if back == 0 {
        return;
    }
    p.health += back;
    p.grey = (p.grey - back).max(0);
    p.paid -= back as u16;
}

/// **The chain in her hand** (2026-11-07, the owner's answers): while her
/// scythe's head is out -- or on its way out of her hand -- **left click is
/// the chain** and right click calls it home.
///
/// - **Left tapped** (let go inside `tuning::hold_from`): the zip, to whatever
///   the head bit -- kept and started the frame it bites if it is still
///   flying (`Garden::zip_queued`).
/// - **Left held to `tuning::reel_full`**: with the head in a fighter, she
///   reels them in (`World::tend_the_zips`); with it in the world, the zip.
/// - **Left let go in between, with the head in a fighter**: it rips out of
///   them -- a hit, a bleed, a stagger -- and comes home.
/// - **Right pressed**: the head is called home now.
///
/// Returns whether the press was the chain's, so it throws nothing else.
pub(super) fn chain_in_hand(
    p: &mut Player,
    who: usize,
    pressed_right: bool,
    input: Input,
    scene: &Scene,
) -> bool {
    if p.class != Class::BloodMage || p.health <= 0 {
        return false;
    }
    let head = scene
        .effects
        .iter()
        .flatten()
        .find(|e| e.owner == who as u8 && e.kind == EffectKind::ScytheHead)
        .copied();
    let leaving = p.action.attack_kind() == Some(moves::blood::CAST)
        && !matches!(p.action, Action::Recovery { .. });
    let out = head.is_some_and(|h| h.head_state() != Head::Home) || (head.is_none() && leaving);
    let Some(g) = garden_of(p) else {
        return false;
    };
    if !out || g.zipping {
        if g.chain_held != 0 && !input.has(moves::blood::keys::HAND) {
            tend(p, |g| g.chain_held = 0);
        }
        return false;
    }
    if pressed_right {
        tend(p, |g| {
            g.recall = true;
            g.chain_held = 0;
            g.zip_queued = false;
        });
        return true;
    }
    let in_a_fighter = head.is_some_and(|h| matches!(h.head_state(), Head::Fighter(_)));
    if input.has(moves::blood::keys::HAND) {
        if g.chain_held == u16::MAX {
            return true;
        }
        let held = g.chain_held.saturating_add(1);
        tend(p, |g| g.chain_held = held);
        if held >= t::reel_full() {
            tend(p, |g| g.chain_held = u16::MAX);
            match head.map(|h| h.head_state()) {
                Some(Head::Fighter(v)) => tend(p, |g| g.reeling = v as u8 + 1),
                _ => zip_now_or_later(p, head),
            }
        }
        return true;
    }
    if g.chain_held == 0 {
        return false;
    }
    let held = g.chain_held;
    tend(p, |g| g.chain_held = 0);
    if held == u16::MAX {
        return true;
    }
    if held >= t::hold_from() && in_a_fighter {
        tend(p, |g| g.rip = true);
    } else {
        zip_now_or_later(p, head);
    }
    true
}

/// The zip, now if the head holds something, or kept for the frame it bites.
fn zip_now_or_later(p: &mut Player, head: Option<Effect>) {
    match head {
        Some(h) if h.is_a_head_holding() && free_to_zip(p) => start_the_zip(p, &h),
        Some(h) if h.head_state() != Head::Flying && !h.is_a_head_holding() => {}
        _ if t::zip_queue() => tend(p, |g| g.zip_queued = true),
        _ => {}
    }
}

/// Can she start a zip now: free, or anywhere in the Cast itself -- the
/// press that queues it usually lands while the head is still leaving.
fn free_to_zip(p: &Player) -> bool {
    p.action.actionable() || p.action.attack_kind() == Some(moves::blood::CAST)
}

/// **The zip begins** (2026-11-07: a pull, not a glide): the chain gives
/// her a tug toward the head (`tuning::zip_start`) and from then on pulls her
/// along it every frame (`World::tend_the_zips`), her own momentum and her
/// weight going with her, in the Cast's recovery while it does.
pub(super) fn start_the_zip(p: &mut Player, head: &Effect) {
    let to = zip_target(p.pos, head.home, head.head_state());
    let along = to.sub(p.pos).normalized();
    p.haul = 0;
    p.haul_to = to;
    p.vel = p.vel.add(along.scale(t::zip_start()));
    p.grounded = false;
    p.swim = SWIM_NONE;
    p.action = Action::Recovery {
        kind: moves::blood::CAST,
        left: ZIP_LIMIT,
    };
    let body = matches!(head.head_state(), Head::Fighter(_) | Head::Creature(_));
    tend(p, |g| {
        g.zipping = true;
        g.zip_jump = false;
        g.zip_queued = false;
        g.zip_to_body = body;
        g.zip_for = 0;
    });
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
/// is within `tuning::zip_jump_buffer` frames of the head at the speed she is
/// going, and thrown the frame she gets there -- the Reaver's dash jump.
/// Earlier than that it is dropped.
pub(super) fn bank_the_zip_jump(p: &mut Player) {
    if !garden_of(p).is_some_and(|g| g.zipping) {
        return;
    }
    let reach = p
        .vel
        .len()
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

    /// **The Cast comes out** (2026-11-04; a weight on a chain since
    /// 2026-11-07): the head leaves her hand **thrown**, at the speed that
    /// carries it through the point the crosshair is on (`aim::skillshot_path`
    /// solved it) under `tuning::cast_gravity` -- an arc that meets what she
    /// aimed at, so grabbing someone stays a matter of putting the crosshair on
    /// them -- and from there it is a falling thing: it carries on past and
    /// comes down. Its velocity is kept in `dir`; its chain's length, the row's
    /// reach, in `reach`. One head at a time.
    pub(super) fn throw_the_head(&mut self, i: usize, mut born: Effect) {
        let p = self.players[i];
        let m = moves::get(p.class, moves::blood::CAST);
        let aimed = p.aim_path.to;
        let line = aimed.sub(born.pos);
        let far = line.len();
        let speed = t::cast_speed().max(Fx::ONE);
        // Time to the crosshair at the throw's speed, and the lift that
        // gravity takes back over that time.
        let time = far.div(speed).max(DT.mul(Fx::from_int(4)));
        let half = Fx::ratio(1, 2);
        let lift = t::cast_gravity().mul(time).mul(half);
        born.dir = line
            .scale(Fx::ONE.div(time))
            .add(V3::new(Fx::ZERO, lift, Fx::ZERO));
        born.reach = m.reach;
        born.home = born.pos;
        born.life = u16::MAX;
        born.set_head(Head::Flying);
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
                let reeling =
                    garden_of(&self.players[owner]).is_some_and(|g| g.reeling == v as u8 + 1);
                if victim.health <= 0 {
                    effect.set_head(Head::Home);
                } else if !reeling && effect.head_for() >= t::cast_holds() {
                    self.wrench_free(effect, v);
                }
            }
            Head::World | Head::Creature(_) => {
                effect.hold_the_head();
                if effect.head_for() >= t::cast_holds_world() {
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

    /// **The head in flight** (2026-11-07): a weight thrown on a chain. It
    /// falls at `tuning::cast_gravity`; at the chain's length (`reach`) the
    /// chain goes taut and holds it there, so it swings round her rather than
    /// flying on; and it bites the first thing it meets on its way -- a body,
    /// a creature, a critter, the world (`aim::first_along`), or the floor it
    /// comes down on. A head that has flown `HEAD_FLIGHT` frames without
    /// biting anything comes home.
    fn fly_the_head(&mut self, effect: &mut Effect) {
        let owner = effect.owner as usize;
        effect.hold_the_head();
        let hand = effect.pos;
        let was = effect.home;
        effect.dir.y = effect.dir.y.sub(t::cast_gravity().mul(DT));
        let mut at = was.add(effect.dir.scale(DT));
        // The chain: never longer than it is.
        let off = at.sub(hand);
        let far = off.len();
        if far.raw() > effect.reach.raw() && far.raw() > 0 {
            let out = off.scale(Fx::ONE.div(far));
            at = hand.add(out.scale(effect.reach));
            let away = effect.dir.dot(out);
            if away.raw() > 0 {
                effect.dir = effect.dir.sub(out.scale(away));
            }
        }
        let radius = effect.field_radius();
        // A fighter on the way.
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
                effect.dir = V3::ZERO;
                effect.set_head(Head::Fighter(v));
            } else {
                // Guarded: it glances off, and comes home.
                effect.home = at;
                effect.dir = V3::ZERO;
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
            effect.home = at;
            effect.dir = V3::ZERO;
            effect.reach = flat_apart(self.players[owner].pos, at).max(t::cast_least_leash());
            effect.set_head(Head::Creature(slot));
            return;
        }
        // The world on the way: a wall, a stone, a critter, the floor.
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
        let path = aim::Path { from: was, to: at };
        let wall = if path.length().raw() > 0 {
            aim::first_along(
                path,
                radius,
                owner as u8,
                &scene,
                Targets::none().terrain().stones(),
            )
            .map(|c| was.add(path.dir().scale(c.dist())))
        } else {
            None
        };
        let floor = terrain.ground_under(V3::new(at.x, at.y.add(radius), at.z));
        if let Some(hit) = wall {
            let floor = terrain.ground_under(V3::new(hit.x, hit.y.add(radius), hit.z));
            self.bite_the_world(effect, hit, floor);
            return;
        }
        if self.gore_critters(effect, at, radius, Fx::ONE) > 0 || at.y.raw() <= floor.raw() {
            self.bite_the_world(effect, at, floor);
            return;
        }
        effect.home = at;
        if effect.head_for() >= HEAD_FLIGHT {
            effect.dir = V3::ZERO;
            effect.set_head(Head::Home);
        }
    }

    /// **The head bites the world** at `at`: a wall, a stone, the floor. It
    /// holds there, down on the floor when it was within a body's height of
    /// it, and **the world bleeds** -- `tuning::cast_bleeds` of blood on the
    /// floor under the bite.
    fn bite_the_world(&mut self, effect: &mut Effect, at: V3, floor: Fx) {
        let owner = effect.owner as usize;
        let near_the_floor = at.y.sub(floor).raw() <= effect.field_radius().raw();
        let bite = if near_the_floor {
            V3::new(at.x, floor, at.z)
        } else {
            at
        };
        effect.home = bite;
        effect.dir = V3::ZERO;
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

    /// **The head comes home** along the chain, straight back to her hand at
    /// `tuning::cast_drag` -- in the air as on the floor, from wherever it
    /// was (2026-11-06: it used to drop to the floor first and drag along it,
    /// which read as a teleport) -- **reaping everything on its line**: every
    /// stain of hers it passes over is drunk (the refund, never past her
    /// grey), and every plant of hers it passes through bursts into thorns
    /// the way it was going. Gone when it reaches her.
    fn drag_home(&mut self, effect: &mut Effect) {
        let owner = effect.owner as usize;
        let her = self.players[owner];
        let hand = moves::get(her.class, effect.slot).hand;
        let home = aim::hand_origin(her.pos, her.facing, hand);
        let was = effect.home;
        let gap = home.sub(was).len();
        // **Reeled**, not glided (2026-11-07): it starts from rest and picks up
        // speed, to `tuning::cast_drag`. Its speed is kept in `dir`.
        let speed = effect
            .dir
            .len()
            .add(t::reel_accel().mul(DT))
            .min(t::cast_drag());
        let step = speed.mul(DT);
        let next = if gap.raw() <= step.raw() {
            home
        } else {
            was.add(home.sub(was).scale(step.div(gap)))
        };
        effect.dir = home.sub(was).normalized().scale(speed);
        effect.home = next;
        let along = flat_toward(was, next);
        let m = moves::get(Class::BloodMage, moves::blood::CAST);
        let reach = effect.field_radius().add(t::body_radius());
        // Her stains under the line: within a body's height of it.
        for slot in 0..MAX_EFFECTS {
            let crossed = self.effects[slot].is_some_and(|e| {
                e.is_a_pool()
                    && !e.is_world_blood()
                    && e.owner == owner as u8
                    && e.age > t::stain_settles()
                    && e.life == UNLIT
                    && was.y.min(next.y).sub(e.pos.y).raw() <= t::body_height().raw()
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
        // Her plants the line passes through.
        if let Some(g) = garden_of(&self.players[owner]) {
            for (slot, plant) in g.plants.iter().enumerate() {
                let Some(plant) = plant else { continue };
                let top = V3::new(plant.at.x, plant.top(), plant.at.z);
                let gap = crate::math::segment_gap(was, next, plant.at, top);
                if gap.raw() <= reach.add(plant.radius()).raw() {
                    self.reap_the_plant(owner, slot, along);
                }
            }
        }
        if home.sub(next).len().raw() <= t::body_radius().raw() {
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
        self.tend_the_chains();
        // A zip pressed while the head flew starts the frame it bites, and is
        // forgotten when it comes home without biting.
        for i in 0..MAX_PLAYERS {
            if !garden_of(&self.players[i]).is_some_and(|g| g.zip_queued && !g.zipping) {
                continue;
            }
            let head = self
                .effects
                .iter()
                .flatten()
                .find(|e| e.kind == EffectKind::ScytheHead && e.owner == i as u8)
                .copied();
            match head {
                Some(head) if head.is_a_head_holding() => {
                    if free_to_zip(&self.players[i]) {
                        start_the_zip(&mut self.players[i], &head);
                    }
                }
                Some(head) if head.head_state() == Head::Flying => {}
                None if matches!(self.players[i].action, Action::Startup { kind, .. } | Action::Active { kind, .. } if kind == moves::blood::CAST) =>
                    {}
                _ => tend(&mut self.players[i], |g| g.zip_queued = false),
            }
        }
        let seen = &{
            let mut now = *seen;
            for (i, p) in now.iter_mut().enumerate() {
                if garden_of(&self.players[i]).is_some_and(|g| g.zipping)
                    && !garden_of(p).is_some_and(|g| g.zipping)
                {
                    *p = self.players[i];
                }
            }
            now
        };
        let _ = seen;
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
                // Which body it is in, for the pull to follow: kept in the
                // hold count, which a zipping head does not use.
                let follow = match state {
                    Head::Fighter(v) => v as i32 + 1,
                    _ => 0,
                };
                e.banked |= follow << 8;
            }
            let follow = e.head_for() as usize;
            if follow > 0 && follow <= MAX_PLAYERS {
                e.home = aim::origin(self.players[follow - 1].pos);
            }
            let p = self.players[i];
            let to = zip_target(
                p.pos,
                e.home,
                if g.zip_to_body {
                    Head::Fighter(0)
                } else {
                    Head::World
                },
            );
            let gap = to.sub(p.pos);
            let far = gap.len();
            let speed = p.vel.len();
            let pulled =
                matches!(p.action, Action::Recovery { kind, .. } if kind == moves::blood::CAST);
            let arrived = far.raw() <= t::body_radius().add(speed.mul(DT)).raw();
            let spent = g.zip_for >= ZIP_LIMIT;
            if arrived || spent || !pulled {
                tend(&mut self.players[i], |g| {
                    g.zipping = false;
                    g.zip_jump = false;
                });
                if pulled {
                    let q = &mut self.players[i];
                    // Onto a body she stops beside it, to fight; onto the
                    // world she carries on with what the pull gave her.
                    if g.zip_to_body {
                        q.vel = V3::new(
                            Fx::ZERO,
                            q.vel.y.max(Fx::ZERO).min(Fx::from_int(3)),
                            Fx::ZERO,
                        );
                    } else {
                        q.vel = q.vel.scale(t::zip_keeps());
                    }
                    if g.zip_jump {
                        let jump = t::jump_speed().mul(q.class.mobility().jump);
                        q.vel.y = q.vel.y.max(Fx::ZERO).add(jump);
                        q.jump_hold = t::jump_hold_frames();
                    }
                    q.grounded = false;
                    q.action = Action::Free;
                    self.effects[head] = None;
                } else {
                    // A hit broke it: the head comes home.
                    e.dir = V3::ZERO;
                    e.set_head(Head::Home);
                    self.effects[head] = Some(e);
                }
                continue;
            }
            // **The pull**: the chain hauls her toward the head, her weight
            // and her momentum still hers -- so a pull up to a high bite arcs.
            let along = gap.scale(Fx::ONE.div(far));
            let q = &mut self.players[i];
            q.vel = q.vel.add(along.scale(t::zip_pull().mul(DT)));
            let now = q.vel.len();
            if now.raw() > t::zip_speed().raw() {
                q.vel = q.vel.scale(t::zip_speed().div(now));
            }
            q.haul_to = to;
            q.grounded = false;
            q.action = Action::Recovery {
                kind: moves::blood::CAST,
                left: ZIP_LIMIT,
            };
            tend(q, |g| g.zip_for = g.zip_for.saturating_add(1));
            self.effects[head] = Some(e);
        }
    }

    /// **What her left hand and right click asked of the chain**, this
    /// frame (`chain_in_hand`): the head called home, ripped out of who it
    /// holds, or them reeled in to her.
    fn tend_the_chains(&mut self) {
        for i in 0..MAX_PLAYERS {
            let Some(g) = garden_of(&self.players[i]) else {
                continue;
            };
            let slot = self.effects.iter().position(|e| {
                e.is_some_and(|e| e.kind == EffectKind::ScytheHead && e.owner == i as u8)
            });
            let head = slot.and_then(|s| self.effects[s]);
            if g.recall {
                tend(&mut self.players[i], |g| {
                    g.recall = false;
                    g.reeling = 0;
                });
                if let (Some(s), Some(mut e)) = (slot, head)
                    && e.head_state() != Head::Home
                {
                    e.dir = V3::ZERO;
                    e.set_head(Head::Home);
                    self.effects[s] = Some(e);
                }
                continue;
            }
            if g.rip {
                tend(&mut self.players[i], |g| g.rip = false);
                if let (Some(s), Some(mut e)) = (slot, head)
                    && let Head::Fighter(v) = e.head_state()
                {
                    self.rip_it_out(i, v);
                    e.dir = V3::ZERO;
                    e.set_head(Head::Home);
                    self.effects[s] = Some(e);
                }
                continue;
            }
            if g.reeling > 0 {
                let v = g.reeling as usize - 1;
                let holds = head.is_some_and(|e| e.head_state() == Head::Fighter(v));
                let q = self.players[v];
                let near = flat_apart(q.pos, self.players[i].pos).raw()
                    <= t::body_radius().mul(Fx::from_int(4)).raw();
                if !holds || q.health <= 0 || near {
                    tend(&mut self.players[i], |g| g.reeling = 0);
                    if holds
                        && near
                        && matches!(q.action, Action::Held { .. })
                        && q.held_by == i as u8
                    {
                        // Reeled in: staggered at her feet, for the hand.
                        let q = &mut self.players[v];
                        q.action = Action::Stagger {
                            left: t::cast_stagger(),
                        };
                        q.stun_total = t::cast_stagger();
                        q.held_by = NOBODY;
                    }
                    if holds && let Some(s) = slot {
                        self.effects[s] = None;
                    }
                    continue;
                }
                // Hand over hand: held on her chain, and dragged in to her
                // arm at `tuning::chain_reel` (`drag_the_held`).
                if !q.untouchable() && !matches!(q.action, Action::Held { .. }) {
                    self.players[v].seized(i as u8, ZIP_LIMIT, 0);
                }
                if matches!(self.players[v].action, Action::Held { .. }) {
                    let q = &mut self.players[v];
                    q.action = Action::Held { left: ZIP_LIMIT };
                }
            }
        }
    }

    /// **Ripped out** (2026-11-07): left let go part-way through its hold,
    /// with the head in a fighter. The rip row's blow, a fresh bleed, and a
    /// stagger.
    fn rip_it_out(&mut self, owner: usize, v: usize) {
        let her = self.players[owner];
        let q = self.players[v];
        if q.untouchable() || q.health <= 0 {
            return;
        }
        let m = moves::get(Class::BloodMage, moves::blood::RIP);
        let damage = Fx::from_int(m.damage)
            .mul(worth_of(&her, Some(moves::blood::RIP)))
            .to_int();
        let dealt = damage.min(q.health).max(0);
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
                dir: flat_toward(q.pos, her.pos),
                blocked: false,
                parried: false,
                interrupts: true,
            },
        );
        {
            let q = &mut self.players[v];
            q.bleed(owner as u8, t::bleed_lasts());
            q.action = Action::Stagger {
                left: t::cast_stagger(),
            };
            q.stun_total = t::cast_stagger();
        }
        if dealt > 0 {
            let q = self.players[v];
            self.credit(owner as u8, dealt);
            self.spill_under(owner as u8, Class::BloodMage, moves::blood::RIP, &q, dealt);
        }
        let hold = impact_freeze(m.hitstop, false);
        freeze(&mut self.players[owner], hold);
        freeze(&mut self.players[v], hold);
    }

    /// **The Red claw lands** (2026-11-07): where her hold put it -- along
    /// the floor toward the crosshair, `reach_after` the frames she held --
    /// it rakes everything inside it at a body's middle, slowing what it hits
    /// to `tuning::claw_slow`, and it is bigger and harder the further out it
    /// went: at the row's whole reach `tuning::claw_grows` times as wide and
    /// half again as hard. Held too long, it lands past them. A plant of hers
    /// inside it is fed. Its marks stay a moment (`EffectKind::Claw`).
    pub(super) fn claw_lands(&mut self, i: usize, mut born: Effect) {
        let p = self.players[i];
        let m = moves::get(p.class, moves::blood::CLAW);
        let at = born.pos;
        let near = m.channel_from.min(m.reach);
        let far = flat_apart(p.pos, at);
        let share = if m.reach.raw() > near.raw() {
            Fx::ratio(far.sub(near).raw().max(0), m.reach.sub(near).raw()).min(Fx::ONE)
        } else {
            Fx::ONE
        };
        let size = Fx::ONE.add(t::claw_grows().sub(Fx::ONE).mul(share));
        let radius = m.radius.mul(size);
        let half = Fx::ratio(1, 2);
        let mid = V3::new(at.x, at.y.add(t::body_height().mul(half)), at.z);
        let hard = Fx::ONE.add(share.mul(half));
        let damage = Fx::from_int(m.damage)
            .mul(worth_of(&p, Some(moves::blood::CLAW)))
            .mul(hard)
            .to_int();
        let along = V3::new(p.facing.x, Fx::ZERO, p.facing.z).normalized();
        for v in 0..MAX_PLAYERS {
            let q = self.players[v];
            if v == i
                || !self.effects_reach(v, i as u8)
                || q.untouchable()
                || !self.inside(v, mid, radius)
            {
                continue;
            }
            let (guarding, parried) = guard_against(&q, p.pos, m.unblockable);
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
                    by: i as u8,
                    dir: along,
                    blocked: guarding,
                    parried,
                    interrupts: true,
                },
            );
            if dealt > 0 {
                self.players[v].slow(t::slow_frames(), t::claw_slow());
                self.credit(i as u8, dealt);
                self.spill_under(i as u8, Class::BloodMage, moves::blood::CLAW, &q, dealt);
            }
            let hold = impact_freeze(m.hitstop, guarding && !parried);
            freeze(&mut self.players[i], hold);
            freeze(&mut self.players[v], hold);
        }
        self.gore_each(&mut born, 0, mid, radius, hard);
        self.gore_critters(&mut born, mid, radius, hard);
        born.forget_hits();
        // Her plants inside it are fed.
        if let Some(g) = garden_of(&self.players[i]) {
            for (slot, plant) in g.plants.iter().enumerate() {
                if plant.is_some_and(|s| flat_apart(s.at, at).raw() <= radius.add(s.radius()).raw())
                {
                    self.feed_the_plant(i, slot);
                }
            }
        }
        born.dir = along;
        born.reach = radius;
        born.home = at;
        born.life = t::reaping_life();
        born.power = size;
        spawn_effect(&mut self.effects, born);
    }

    /// **A plant fed by her claw** (2026-11-07, the owner's answer): it
    /// grows by `tuning::feed_grows` of a stone, to at most `feed_most` --
    /// and its lash and its thorns grow with it -- and it starts its life
    /// over, so a tended plant stands.
    pub(super) fn feed_the_plant(&mut self, owner: usize, slot: usize) {
        tend(&mut self.players[owner], |g| {
            if let Some(stone) = g.plants[slot].as_mut() {
                stone.scale = stone.scale.add(t::feed_grows()).min(t::feed_most());
                stone.age = 1;
            }
        });
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
        // **Bare ground: her own blood** (2026-11-06). What she paid lands as
        // a pool on the floor there, and that pool rises.
        if self.pool_touching(i as u8, at, reach).is_none() {
            let paid = self.players[i].paid as i32;
            let lift = V3::new(at.x, at.y.add(t::body_height()), at.z);
            let floor = self.terrain().ground_under(lift);
            if paid > 0 && at.y.sub(floor).raw() <= t::body_height().raw() {
                let class = self.players[i].class;
                self.pool_blood(i as u8, class, kind, V3::new(at.x, floor, at.z), paid);
            }
        }
        let Some(slot) = self.pool_touching(i as u8, at, reach) else {
            return;
        };
        let Some(pool) = self.effects[slot] else {
            return;
        };
        self.erupt_from(i, kind, slot, false);
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
        let worth = pool
            .pool_volume()
            .max(paid as i32)
            .clamp(0, u16::MAX as i32) as u16;
        plant(&mut self.players[i], stone, worth, paid);
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
        self.the_shrike();
    }

    /// **The shrike** (2026-11-07): a body reeled in on her chain, or
    /// knocked into one of her plants at `tuning::shrike_speed` or more, is
    /// impaled on it -- pinned on its thorns for `tuning::impale_for`,
    /// bleeding, for the impaling's row and a share of what the plant holds --
    /// and the plant drinks what it dealt. One at a time: her plants rest
    /// a moment after.
    fn the_shrike(&mut self) {
        for i in 0..MAX_PLAYERS {
            let Some(g) = garden_of(&self.players[i]) else {
                continue;
            };
            if g.shrike_rest > 0 {
                tend(&mut self.players[i], |g| g.shrike_rest -= 1);
                continue;
            }
            'plants: for (slot, plant) in g.plants.iter().enumerate() {
                let Some(plant) = plant else { continue };
                for v in 0..MAX_PLAYERS {
                    let q = self.players[v];
                    if v == i || !self.effects_reach(v, i as u8) || q.untouchable() || q.health <= 0
                    {
                        continue;
                    }
                    let touching = flat_apart(q.pos, plant.at).raw()
                        <= plant
                            .radius()
                            .add(t::body_radius())
                            .add(Fx::ratio(1, 5))
                            .raw()
                        && q.pos.y.raw() <= plant.top().raw()
                        && q.pos.y.add(t::body_height()).raw() >= plant.at.y.raw();
                    let reeled = matches!(q.action, Action::Held { .. }) && q.held_by == i as u8;
                    let flung = matches!(q.action, Action::HitStun { .. } | Action::Stagger { .. })
                        && V3::new(q.vel.x, Fx::ZERO, q.vel.z).flat_len().raw()
                            >= t::shrike_speed().raw();
                    if touching && (reeled || flung) {
                        self.impale(i, slot, v);
                        break 'plants;
                    }
                }
            }
        }
    }

    /// One body impaled on one of her plants (`the_shrike`).
    fn impale(&mut self, owner: usize, slot: usize, v: usize) {
        let Some(g) = garden_of(&self.players[owner]) else {
            return;
        };
        let Some(plant) = g.plants[slot] else {
            return;
        };
        let m = moves::get(Class::BloodMage, moves::blood::IMPALE);
        let q = self.players[v];
        let damage = m.damage + g.blood[slot] as i32 / 4;
        let dealt = damage.min(q.health).max(0);
        apply_hit(
            &mut self.players[v],
            Hit {
                damage,
                hitstun: m.hitstun,
                blockstun: m.blockstun,
                knockback: Fx::ZERO,
                launch: Fx::ZERO,
                grabs: 0,
                by: owner as u8,
                dir: flat_toward(plant.at, q.pos),
                blocked: false,
                parried: false,
                interrupts: true,
            },
        );
        // Pinned on its thorns, at its side, half way up.
        let out = flat_toward(plant.at, q.pos);
        let half = Fx::ratio(1, 2);
        let at = V3::new(
            plant
                .at
                .x
                .add(out.x.mul(plant.radius().add(t::body_radius()))),
            plant
                .at
                .y
                .add(plant.height().mul(half))
                .sub(t::body_height().mul(half)),
            plant
                .at
                .z
                .add(out.z.mul(plant.radius().add(t::body_radius()))),
        );
        let pin = t::impale_for();
        {
            let q = &mut self.players[v];
            q.pos = at;
            q.vel = V3::ZERO;
            q.seized(NAILED, pin, pin);
            q.bleed(owner as u8, t::bleed_lasts());
        }
        let drank = dealt.clamp(0, u16::MAX as i32) as u16;
        tend(&mut self.players[owner], |g| {
            g.blood[slot] = g.blood[slot].saturating_add(drank);
            g.shrike_rest = pin.saturating_add(20);
            if g.reeling == v as u8 + 1 {
                g.reeling = 0;
            }
        });
        // The chain lets go of them.
        for e in self.effects.iter_mut().flatten() {
            if e.kind == EffectKind::ScytheHead
                && e.owner == owner as u8
                && e.head_state() == Head::Fighter(v)
            {
                e.dir = V3::ZERO;
                e.set_head(Head::Home);
            }
        }
        if dealt > 0 {
            self.credit(owner as u8, dealt);
        }
        let hold = impact_freeze(m.hitstop, false);
        freeze(&mut self.players[owner], hold);
        freeze(&mut self.players[v], hold);
    }

    /// One plant's lash: every enemy body within `tuning::plant_lash_reach`
    /// of its edge, at its height.
    fn lash(&mut self, owner: usize, stone: Structure) {
        let m = moves::get(Class::BloodMage, moves::blood::LASH);
        // A fed plant (`feed_the_plant`) lashes further and harder, by how
        // much bigger than a fresh one it has grown.
        let grown = stone.scale.div(t::plant_scale().max(Fx::ratio(1, 10)));
        let reach = stone.radius().add(t::plant_lash_reach().mul(grown));
        for v in 0..MAX_PLAYERS {
            let q = self.players[v];
            // Not one her own hand is holding: the plant would knock them out
            // of her Seize before she brought them down on it.
            let held_by_her = matches!(q.action, Action::Held { .. }) && q.held_by == owner as u8;
            if !self.effects_reach(v, owner as u8)
                || q.untouchable()
                || held_by_her
                || flat_apart(q.pos, stone.at).raw() > reach.add(t::body_radius()).raw()
                || q.pos.y.raw() > stone.top().add(t::body_height()).raw()
                || q.pos.y.add(t::body_height()).raw() < stone.at.y.raw()
            {
                continue;
            }
            let (guarding, parried) = guard_against(&q, stone.at, m.unblockable);
            let damage = Fx::from_int(m.damage).mul(grown).to_int();
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
        let mid = V3::new(
            stone.at.x,
            stone.at.y.add(stone.height().mul(half)),
            stone.at.z,
        );
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
                self.spill_under(
                    owner as u8,
                    Class::BloodMage,
                    moves::blood::THORNS,
                    &q,
                    dealt,
                );
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
        let mid = stone.at.add(dir.scale(reach.mul(half))).add(V3::new(
            Fx::ZERO,
            t::body_height().mul(half),
            Fx::ZERO,
        ));
        self.gore_each(&mut thorns, 0, mid, reach.mul(half), share);
        self.gore_critters(&mut thorns, mid, reach.mul(half), share);
        thorns.forget_hits();
        spawn_effect(&mut self.effects, thorns);
        if paid > 0 {
            self.players[owner].absorb(paid);
        }
    }

    /// **The Red Hand through a plant feeds it** (2026-11-07): Rend or the
    /// Rising rend whose volume passes through one of her plants on its first
    /// active frame (`feed_the_plant`).
    pub(super) fn feed_with_the_hand(&mut self, who: usize) {
        let p = self.players[who];
        let Action::Active { kind, left } = p.action else {
            return;
        };
        if p.class != Class::BloodMage
            || !matches!(kind, moves::blood::REND | moves::blood::RISING_REND)
            || left != moves::get(p.class, kind).active
        {
            return;
        }
        let (Some(hb), Some(g)) = (hitbox(&p), garden_of(&p)) else {
            return;
        };
        for (slot, plant) in g.plants.iter().enumerate() {
            let Some(plant) = plant else { continue };
            let top = V3::new(plant.at.x, plant.top(), plant.at.z);
            let gap = crate::math::segment_gap(hb.from, hb.to, plant.at, top);
            if gap.raw() <= hb.radius.add(plant.radius()).raw() {
                self.feed_the_plant(who, slot);
            }
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
            // **Not the one under her feet** (2026-11-06, from play: "jumping
            // off a tree ... causes it to explode"): a Rising reap or a Sweep
            // thrown standing on a plant cuts down through it, and that is a
            // foothold, not a harvest.
            let on_it = flat_apart(p.pos, plant.at).raw()
                <= plant.radius().add(t::body_radius()).raw()
                && p.pos.y.sub(plant.top()).abs().raw() <= t::body_radius().raw();
            if on_it {
                continue;
            }
            let top = V3::new(plant.at.x, plant.top(), plant.at.z);
            let gap = crate::math::segment_gap(hb.from, hb.to, plant.at, top);
            if gap.raw() <= hb.radius.add(plant.radius()).raw() {
                self.reap_the_plant(who, slot, flat_toward(p.pos, plant.at));
            }
        }
    }
}

/// How long the head flies without biting anything before it is reeled
/// home: a second and a half, by when a throw at nothing is on the floor.
const HEAD_FLIGHT: u16 = 90;

/// How long a zip may pull before it gives up: two seconds.
const ZIP_LIMIT: u16 = 120;
