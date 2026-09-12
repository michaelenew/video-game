//! Stones: the Elementalist's structures, as things that take up space.
//!
//! A structure used to be a marker — a position, an age that drove a cosmetic
//! rise, and nothing else. It is a **solid** now, and the only moving one in
//! the game. That asks three questions, and this module is the three answers.
//!
//! **What happens when a stone arrives where another already is.** Whatever is
//! in the way gets moved, along whichever axis is the shorter way out: straight
//! up if the new stone comes up underneath, sideways if it comes up beside. A
//! stone thrown into another hands over the speed it was carrying and both come
//! out of it slower, so a stone cannot bowl through a row of them.
//!
//! **What happens when a stone arrives where a fighter is.** The same rule, so
//! you can stand on one. That is the point: the class builds the terrain, and
//! terrain you cannot get on top of is a wall rather than a floor. A stone
//! still climbing carries whoever is on it, which is what makes raising one
//! under your own feet a way up rather than a way to be shoved aside.
//!
//! **What it costs to be standing on the spot when one comes up.** A stone
//! spends the first half of its rise barely out of the floor, and that half
//! **slows** whoever is over it — the ground is churning, and you feel it
//! before you see it. The second half is the eruption, and that **hits**: a
//! little damage and a stagger, once per fighter, because a stone erupts once.
//!
//! The phases come off the rise *curve* rather than a frame count, so reshaping
//! the rise moves the telegraph with it. A warning that can drift out of step
//! with the thing it is warning about is worse than no warning.

use crate::DT;
use crate::arena;
use crate::class::{MAX_STRUCTURES, Mechanic, Structure};
use crate::fixed::Fx;
use crate::math::V3;
use crate::state::{Action, MAX_PLAYERS, Player};
use crate::tuning as t;

/// Every stone the arena can hold: the per-fighter cap, once per fighter.
pub const MAX_STONES: usize = MAX_PLAYERS * MAX_STRUCTURES;

/// Every stone on the field, flattened out of the mechanics that own them.
///
/// Indexed by `owner * MAX_STRUCTURES + slot`, so the owner of a stone is
/// arithmetic and writing the field back is never a search. A fixed order is
/// also what makes resolving every pair deterministic.
pub type Field = [Option<Structure>; MAX_STONES];

/// What a stone is doing, which is what decides whether it is a warning or a
/// hit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Still under the floor, churning the ground it is coming through.
    Churning,
    /// Coming out, fast. The part that hurts.
    Erupting,
    /// Out, and just a solid from here on.
    Standing,
}

impl Structure {
    /// A stone as it is raised: buried, still, and having caught nobody.
    pub fn raised(at: V3) -> Structure {
        Structure {
            at,
            vel: V3::ZERO,
            age: 0,
            struck: 0,
            launched: false,
            launch_from: V3::ZERO,
            knock_struck: 0,
        }
    }

    /// How far out of the ground it is, from none of it to all of it.
    pub fn risen(&self) -> Fx {
        risen_at(self.age)
    }

    /// The surface you can stand on.
    pub fn top(&self) -> Fx {
        self.at.y.add(t::structure_height().mul(self.risen()))
    }

    /// How tall the part above its own base is. Zero while fully buried, which
    /// is what keeps a stone that has not arrived yet from blocking anything.
    pub fn standing_height(&self) -> Fx {
        t::structure_height().mul(self.risen())
    }

    pub fn phase(&self) -> Phase {
        if self.age >= t::structure_rise() {
            Phase::Standing
        } else if self.risen().raw() >= t::stone_erupt().raw() {
            Phase::Erupting
        } else {
            Phase::Churning
        }
    }

    /// How fast the top of this stone is moving.
    ///
    /// Its own speed plus how fast it is still coming out of the ground. This
    /// is the number that makes an eruption a launch rather than a lift: the
    /// burst climbs far faster than anything walks, and whatever is riding the
    /// top keeps that speed when the burst ends.
    pub fn surface_speed(&self) -> Fx {
        let grew = risen_at(self.age).sub(risen_at(self.age.saturating_sub(1)));
        self.vel.y.add(t::structure_height().mul(grew).div(DT))
    }

    /// Has the eruption already caught this fighter?
    fn already_struck(&self, player: usize) -> bool {
        self.struck & (1 << player) != 0
    }
}

/// The rise at a given age, along the Oven's curve.
fn risen_at(age: u16) -> Fx {
    let frames = t::structure_rise().max(1);
    t::structure_rise_curve().at(Fx::ratio(age as i32, frames as i32))
}

// ---------------------------------------------------------------------------
// Gathering
// ---------------------------------------------------------------------------

/// Every stone in the world, in one flat array.
///
/// Stones belong to a fighter's mechanic, which is right — the cap of three is
/// the Elementalist's resource and nothing else should be able to spend it. But
/// they act on *each other* and on *both* fighters, and neither of those can be
/// written from inside one player's borrow. Gathering, resolving and writing
/// back is the whole trick.
pub fn gather(players: &[Player; MAX_PLAYERS]) -> Field {
    let mut field = [None; MAX_STONES];
    for (owner, p) in players.iter().enumerate() {
        let Mechanic::Structures(slots) = p.mechanic else {
            continue;
        };
        for (slot, stone) in slots.iter().enumerate() {
            field[owner * MAX_STRUCTURES + slot] = *stone;
        }
    }
    field
}

fn scatter(players: &mut [Player; MAX_PLAYERS], field: &Field) {
    for (owner, p) in players.iter_mut().enumerate() {
        let Mechanic::Structures(mut slots) = p.mechanic else {
            continue;
        };
        for (slot, stone) in slots.iter_mut().enumerate() {
            *stone = field[owner * MAX_STRUCTURES + slot];
        }
        p.mechanic = Mechanic::Structures(slots);
    }
}

/// Who owns the stone at this index.
fn owner_of(index: usize) -> u8 {
    (index / MAX_STRUCTURES) as u8
}

// ---------------------------------------------------------------------------
// The stones' own frame
// ---------------------------------------------------------------------------

/// Age the stones, move them, and settle them against each other.
///
/// Runs **before** the fighters step, so what a fighter collides with this
/// frame is where the stone actually is rather than where it was.
pub fn step(players: &mut [Player; MAX_PLAYERS]) {
    let mut field = gather(players);

    for stone in field.iter_mut().flatten() {
        // Saturating, because this is not a lifetime: once a stone is out of
        // the ground the number stops mattering and the stone stays.
        stone.age = stone.age.saturating_add(1);
        // Gravity always. A stone at rest has its fall zeroed by the floor
        // every frame, which costs nothing and means resting needs no flag.
        stone.vel.y = stone.vel.y.add(t::gravity().mul(DT));
        stone.at = stone.at.add(stone.vel.scale(DT));
    }

    // Every pair once, in index order, so two stones settle the same way on
    // every machine.
    for i in 0..MAX_STONES {
        for j in (i + 1)..MAX_STONES {
            let (Some(a), Some(b)) = (field[i], field[j]) else {
                continue;
            };
            let (a, b) = meet(a, b);
            field[i] = Some(a);
            field[j] = Some(b);
        }
    }

    for stone in field.iter_mut().flatten() {
        let r = arena::resolve_sized(
            stone.at,
            stone.vel,
            true,
            t::structure_radius(),
            stone.standing_height(),
        );
        stone.at = r.pos;
        stone.vel = r.vel;
        if r.wall {
            // Hitting a wall is a collision, not a glance: the resolver above
            // only zeroes the component that was driving the stone into the
            // wall, which on its own leaves a diagonally-kicked stone free to
            // keep sliding along the wall at whatever speed it had sideways --
            // `launch_decel` has no idea a wall was ever there and would
            // otherwise keep paying out from the kick's own distance budget
            // regardless. Cutting the remaining speed the same way two
            // stones knocking into each other do, and ending the launch,
            // hands the rest of it to ordinary friction below -- a real stop
            // rather than a slow creep along the wall.
            stone.vel.x = stone.vel.x.mul(t::stone_knock_damp());
            stone.vel.z = stone.vel.z.mul(t::stone_knock_damp());
            stone.launched = false;
        }
        if stone.launched {
            // A kicked stone dies off on its own schedule, not the ambient
            // friction every other stone rubs to a halt with -- see
            // `launch_decel`.
            launch_decel(stone);
        } else if stone.vel.y.raw() == 0 {
            // A stone that is not falling is resting on something, and a
            // resting stone rubs to a halt. Without this a knocked stone
            // slides until it finds a wall, which makes the knock a delivery
            // rather than a shove.
            stone.vel.x = stone.vel.x.mul(t::stone_friction());
            stone.vel.z = stone.vel.z.mul(t::stone_friction());
        }
    }

    // Whoever a fast stone swept past this frame, caught against positions
    // from *before* anyone moves. `resolve_body` keeps a body from ever
    // penetrating a solid, which is right for a wall and wrong for a check
    // that reads "did this connect" -- it would otherwise push a standing
    // fighter out to exactly the boundary every single frame, so a kicked
    // stone could close the whole arena on someone and never once measure as
    // closer than `reach`.
    knock_touch(&mut field, players);

    scatter(players, &field);
}

/// A launched stone catching a fighter on its way past. See `step`'s comment
/// on why this runs here rather than alongside the churn/eruption checks in
/// `touch`.
fn knock_touch(field: &mut Field, players: &mut [Player; MAX_PLAYERS]) {
    let reach = t::body_radius().add(t::structure_radius());
    for (index, slot) in field.iter_mut().enumerate() {
        let Some(stone) = slot else { continue };
        if !stone.launched {
            continue;
        }
        let owner = owner_of(index);
        for (i, p) in players.iter_mut().enumerate() {
            if i as u8 == owner
                || p.health <= 0
                || p.action.invulnerable()
                || stone.knock_struck & (1 << i) != 0
            {
                continue;
            }
            let apart = V3::new(p.pos.x.sub(stone.at.x), Fx::ZERO, p.pos.z.sub(stone.at.z));
            if apart.flat_len().raw() >= reach.raw() {
                continue;
            }

            // Speed *relative to the target* -- the same quantity `knock`
            // hands between two stones -- so a stone barely still moving does
            // not read as a hit just because it once was fast.
            let dir = apart.normalized();
            let closing = stone.vel.sub(p.vel).dot(dir);
            if closing.raw() <= t::bolt_knock_min_speed().raw() {
                continue;
            }
            stone.knock_struck |= 1 << i;
            let dmg = closing
                .mul(Fx::from_int(t::bolt_knock_damage_per_speed()))
                .to_int()
                .max(0);
            p.health = (p.health - dmg).max(0);
            p.action = Action::Stagger {
                left: t::bolt_knock_stagger(),
            };
            let push = dir.scale(closing.mul(t::bolt_knock_push()));
            p.vel.x = push.x;
            p.vel.z = push.z;
        }
    }
}

/// Two stones sharing space. The shorter way out wins.
fn meet(mut a: Structure, mut b: Structure) -> (Structure, Structure) {
    let span = t::structure_radius().add(t::structure_radius());
    let apart = V3::new(b.at.x.sub(a.at.x), Fx::ZERO, b.at.z.sub(a.at.z));
    let flat = apart.flat_len();
    if flat.raw() >= span.raw() {
        return (a, b);
    }
    let (a_top, b_top) = (a.top(), b.top());
    if a_top.raw() <= b.at.y.raw() || b_top.raw() <= a.at.y.raw() {
        return (a, b);
    }

    let side = span.sub(flat);
    // How far each would have to climb to stand clear on top of the other.
    let b_climbs = a_top.sub(b.at.y);
    let a_climbs = b_top.sub(a.at.y);

    // Two stones sharing a centre exactly have no sideways to separate along,
    // so the way out is up -- the same reason `resolve_body` sends a fighter
    // upward out of one.
    if flat.raw() == 0 || b_climbs.min(a_climbs).raw() <= side.raw() {
        // Up. Always the one already nearer the other's top, and never the
        // other way about -- pushing the lower one down would drive it into the
        // floor it is standing on.
        if b_climbs.raw() <= a_climbs.raw() {
            ride(&mut b, &a, apart);
        } else {
            ride(&mut a, &b, V3::ZERO.sub(apart));
        }
    } else {
        knock(&mut a, &mut b, apart.normalized(), side);
    }
    (a, b)
}

/// Put `rider` on top of `under`, moving at whatever speed that top is moving.
///
/// A still surface only stops the fall — the rider has landed. A climbing one
/// **throws** it: straight up if it sits square on the middle, and increasingly
/// outward the further off centre it sits, because a boulder coming up under
/// the edge of another flips it clear rather than balancing it there. That is
/// the only thing on the field that gives a stone horizontal speed today, and
/// it is what makes one stone knocking into another something you can actually
/// make happen.
fn ride(rider: &mut Structure, under: &Structure, offset: V3) {
    rider.at.y = under.top();
    let climb = under.surface_speed().mul(t::stone_lift());
    if climb.raw() <= 0 {
        if rider.vel.y.raw() < 0 {
            rider.vel.y = Fx::ZERO;
        }
        return;
    }
    rider.vel.y = climb.max(rider.vel.y);

    let span = t::structure_radius().add(t::structure_radius());
    let out = climb.mul(offset.flat_len().div(span));
    let away = offset.normalized();
    // Set rather than added: the rider is standing on a moving surface for as
    // long as the burst lasts, not being hit by it once a frame.
    rider.vel.x = away.x.mul(out);
    rider.vel.z = away.z.mul(out);
}

/// Two stones meeting sideways.
///
/// They separate evenly, the closing speed is handed across, and both keep less
/// than they brought. The damping is what makes a knock cost something: without
/// it one stone could be relayed the length of the arena through the others.
fn knock(a: &mut Structure, b: &mut Structure, away: V3, overlap: Fx) {
    let push = away.scale(overlap.mul(Fx::ratio(1, 2)));
    a.at = a.at.sub(push);
    b.at = b.at.add(push);

    let closing = a.vel.sub(b.vel).dot(away);
    if closing.raw() <= 0 {
        return;
    }
    let handed = away.scale(closing.mul(t::stone_knock_handed()));
    a.vel = a.vel.sub(handed);
    b.vel = b.vel.add(handed);
    for stone in [a, b] {
        stone.vel.x = stone.vel.x.mul(t::stone_knock_damp());
        stone.vel.z = stone.vel.z.mul(t::stone_knock_damp());
    }
}

// ---------------------------------------------------------------------------
// The Elementalist's auto, aimed through a stone
// ---------------------------------------------------------------------------
//
// See `docs/design/kits/elementalist.md`. Bolt reads what it is aimed
// through: a structure in the way is not a fighter to poke, it is terrain to
// kick -- fast at first, dying off over the back quarter of its travel, and
// hurting whoever it is still moving fast enough to catch.

/// Ease a kicked stone's speed down over the back of its travel, and let it
/// go once that travel is spent.
///
/// A stone friction alone would not do: friction only fires once a stone is
/// resting, and a kicked stone is briefly airless-fast and grounded at once.
/// This is the shot's own schedule, keyed to *distance travelled* rather than
/// frames, so retuning the range moves the whole shape with it the same way
/// the structure rise curve does for the telegraph.
fn launch_decel(stone: &mut Structure) {
    let travelled = V3::new(
        stone.at.x.sub(stone.launch_from.x),
        Fx::ZERO,
        stone.at.z.sub(stone.launch_from.z),
    )
    .flat_len();
    let range = t::bolt_knock_range().max(Fx::ratio(1, 10));
    let progress = travelled.div(range);
    if progress.raw() >= Fx::ONE.raw() {
        // Spent. An ordinary stone from here on, subject to ordinary friction.
        stone.launched = false;
        return;
    }

    let start = t::bolt_knock_decel_start();
    if progress.raw() <= start.raw() {
        return; // full speed until the last stretch
    }
    let span = Fx::ONE.sub(start).max(Fx::ratio(1, 100));
    let eased = crate::math::smoothstep(progress.sub(start).div(span));
    let retain = Fx::ONE.sub(eased);

    // A ceiling, not a floor: collisions with other stones already cost this
    // one speed, and the decel should never hand any of that back.
    let allowed = t::bolt_knock_speed().mul(retain);
    let speed = V3::new(stone.vel.x, Fx::ZERO, stone.vel.z).flat_len();
    if speed.raw() > allowed.raw() && speed.raw() > 0 {
        let scale = allowed.div(speed);
        stone.vel.x = stone.vel.x.mul(scale);
        stone.vel.z = stone.vel.z.mul(scale);
    }
}

/// The nearest stone a shot meets, and how far along the shot it sits -- so
/// the caller can tell it apart from whatever else the same shot might be
/// aimed through.
///
/// A real ray against the stone's own cylinder, caps included, so a shot
/// aimed over the top of a stone passes over it and one aimed up at a stone
/// standing on another finds it. `swell` is the shot's own radius, added to
/// the stone's. A stone still buried has no standing height and so nothing to
/// hit.
pub fn first_along_shot(
    field: &Field,
    from: V3,
    dir: V3,
    limit: Fx,
    swell: Fx,
) -> Option<(usize, Fx)> {
    let mut best: Option<(usize, Fx)> = None;
    for (i, slot) in field.iter().enumerate() {
        let Some(stone) = slot else { continue };
        let Some(dist) = crate::math::ray_hits_cylinder(
            from,
            dir,
            stone.at,
            t::structure_radius().add(swell),
            stone.standing_height(),
        ) else {
            continue;
        };
        if dist.raw() > limit.raw() {
            continue;
        }
        if best.is_none_or(|(_, d)| dist.raw() < d.raw()) {
            best = Some((i, dist));
        }
    }
    best
}

/// Kick the stone at `index` (as returned by `first_along_shot`) forward
/// along `dir` at the shot's launch speed.
///
/// `dir` is the shot's real, three-dimensional aim -- pitch included, not
/// flattened -- so a structure struck while aiming above the horizon takes
/// some of that speed upward instead of only forward. Aimed level or below,
/// there is nothing to add: a kick cannot drive a stone through the floor, so
/// only the case that means something changes anything.
///
/// Resets its own strike record rather than touching `struck`: a stone that
/// already erupted once is still fair game to hurt someone when it is kicked,
/// because the kick is a different event.
pub fn kick(players: &mut [Player; MAX_PLAYERS], index: usize, dir: V3) {
    let mut field = gather(players);
    if let Some(stone) = field[index].as_mut() {
        stone.launched = true;
        stone.launch_from = stone.at;
        stone.knock_struck = 0;
        stone.vel.x = dir.x.mul(t::bolt_knock_speed());
        stone.vel.z = dir.z.mul(t::bolt_knock_speed());
        if dir.y.raw() > 0 {
            stone.vel.y = dir.y.mul(t::bolt_knock_speed());
        }
    }
    scatter(players, &field);
}

// ---------------------------------------------------------------------------
// Stones against fighters
// ---------------------------------------------------------------------------

/// Push a fighter out of the stones, and tell them what they are standing on.
///
/// The arena's least-penetration rule against a cylinder instead of a box. A
/// stone is a wall the Elementalist made, so it stops you like a wall and holds
/// you up like a platform, and a climbing one takes you with it.
pub fn resolve_body(
    field: &Field,
    mut pos: V3,
    mut vel: V3,
    mut grounded: bool,
    was_grounded: bool,
) -> arena::Resolved {
    let reach = t::body_radius().add(t::structure_radius());
    let height = t::body_height();

    for stone in field.iter().flatten() {
        let top = stone.top();
        let feet = pos.y;
        let head = pos.y.add(height);
        if head.raw() <= stone.at.y.raw() || feet.raw() >= top.raw() {
            continue;
        }
        let apart = V3::new(pos.x.sub(stone.at.x), Fx::ZERO, pos.z.sub(stone.at.z));
        let flat = apart.flat_len();
        if flat.raw() >= reach.raw() {
            continue;
        }

        let side = reach.sub(flat);
        let step_up = top.sub(feet);
        let duck_under = head.sub(stone.at.y);

        // Dead centre there is no sideways to be pushed, and a body with
        // nowhere to go is a body stuck inside a stone. The vertical way out
        // always exists, and it is the one a fighter wants anyway: a stone that
        // lands on you should put you on top of it.
        if flat.raw() > 0 && step_up.min(duck_under).raw() > side.raw() {
            // Out the side. Only the speed *into* the stone is spent, so a
            // fighter slides around one instead of sticking to it.
            let away = apart.normalized();
            pos.x = pos.x.add(away.x.mul(side));
            pos.z = pos.z.add(away.z.mul(side));
            let into = vel.x.mul(away.x).add(vel.z.mul(away.z));
            if into.raw() < 0 {
                vel.x = vel.x.sub(away.x.mul(into));
                vel.z = vel.z.sub(away.z.mul(into));
            }
        } else if step_up.raw() <= duck_under.raw() {
            pos.y = top;
            let climb = stone.surface_speed().mul(t::stone_lift());
            if climb.raw() > vel.y.raw() {
                // Carried. A stone erupting under your feet is the class's way
                // up, and it has to hand over real speed or it is a lift in a
                // game that is about jumping.
                vel.y = climb;
            } else if vel.y.raw() < 0 {
                vel.y = Fx::ZERO;
            }
            grounded = true;
        } else {
            pos.y = stone.at.y.sub(height);
            if vel.y.raw() > 0 {
                vel.y = Fx::ZERO;
            }
        }
    }

    // Resolved exactly on to a surface, there is nothing left to collide with
    // next frame, so without a skin a fighter standing on a stone reads as
    // airborne every other frame. Same tolerance the arena uses for its own
    // platforms.
    if !grounded && was_grounded && vel.y.raw() <= 0 && standing_on(field, pos) {
        grounded = true;
    }

    // A fighter sliding off the side of a stone is not a wall contact in the
    // sense `stones::step` cares about -- that flag is for a *stone* meeting
    // the arena, not a body meeting a stone.
    arena::Resolved {
        pos,
        vel,
        grounded,
        wall: false,
    }
}

/// Is there a stone directly beneath the feet?
fn standing_on(field: &Field, pos: V3) -> bool {
    let reach = t::body_radius().add(t::structure_radius());
    field.iter().flatten().any(|stone| {
        let apart = V3::new(pos.x.sub(stone.at.x), Fx::ZERO, pos.z.sub(stone.at.z));
        apart.flat_len().raw() < reach.raw()
            && pos.y.sub(stone.top()).abs().raw() <= arena::SKIN.raw()
    })
}

/// What it costs to be standing where a stone is coming up.
///
/// Runs after both fighters have moved, like the persistent effects do, so
/// being knocked on to a stone costs you the same as walking on to it.
pub fn touch(players: &mut [Player; MAX_PLAYERS]) {
    let mut field = gather(players);
    let reach = t::body_radius().add(t::structure_radius());

    for (index, slot) in field.iter_mut().enumerate() {
        let Some(stone) = slot else { continue };
        let owner = owner_of(index);
        for (i, p) in players.iter_mut().enumerate() {
            // A stone never touches the fighter who raised it. She raises them
            // under her own feet on purpose, and a class that staggers itself
            // doing the thing it is for is not a class.
            if i as u8 == owner || p.health <= 0 {
                continue;
            }
            let apart = V3::new(p.pos.x.sub(stone.at.x), Fx::ZERO, p.pos.z.sub(stone.at.z));
            if apart.flat_len().raw() >= reach.raw() {
                continue;
            }
            match stone.phase() {
                // The churn is felt through the floor, so jumping clears it.
                Phase::Churning => {
                    if p.grounded {
                        p.slow(t::slow_frames(), t::stone_churn_slow());
                    }
                }
                Phase::Erupting => {
                    // Feet at the top counts, and has to: the eruption pushes
                    // whoever was standing over it up on to itself, so a strict
                    // test would mean the stone only ever caught people it had
                    // already missed.
                    let caught = p.pos.y.raw() <= stone.top().raw()
                        && p.pos.y.add(t::body_height()).raw() > stone.at.y.raw();
                    if !caught || stone.already_struck(i) || p.action.invulnerable() {
                        continue;
                    }
                    stone.struck |= 1 << i;
                    p.health = (p.health - t::stone_erupt_damage()).max(0);
                    p.action = Action::Stagger {
                        left: t::stone_erupt_stagger(),
                    };
                }
                Phase::Standing => {}
            }
        }
    }

    scatter(players, &field);
}
