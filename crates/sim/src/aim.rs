//! Where an ability goes.
//!
//! One rule, for everything a fighter places or throws:
//!
//! > **Follow the line the player is looking along, out from the point
//! > abilities come from, and stop at the first of two things: the terrain, or
//! > the edge of the ability's range.**
//!
//! That is the whole module. Everything else here is the arithmetic for
//! "first", done in fixed point so two machines agree on it.
//!
//! ## Why the ray starts at the eye
//!
//! Because the crosshair is the aim. The player is not pointing a gun held at
//! their chest; they are pointing at a *place on the screen*, and the middle of
//! the screen is a ray out of the eye. Trace that ray, take what it meets
//! first, and that is what they meant. Then draw the line from the ability's
//! own origin to it, and send the ability along that.
//!
//! Tracing from the chest instead -- which this did, and which is the obvious
//! thing to do -- makes the crosshair a liar wherever the eye is not on the
//! chest. The two rays are parallel, so they never converge: aimed down at the
//! ground in the middle of the neutral zone, the reticle sat on a spot about
//! four metres beyond where the ability actually landed.
//!
//! The cost is that this has to know where the eye is, so the camera's geometry
//! is simulation state and its numbers are in the desync checksum. See
//! `crate::camera`, which says what that bought and what it cost.
//!
//! ## Why the range is a sphere and not a clamp on the ground
//!
//! Trace to terrain alone and the target lurches: aim a hair over the lip of a
//! platform and the hit jumps from two metres away to the far wall, so a
//! fraction of a degree of mouse movement swings the ability across the arena.
//! Stopping the trace at the range sphere bounds that jump to the ability's own
//! reach, and gives the player both halves of what they want at once — aim at
//! the ground to pick a *direction*, or aim at a spot inside your reach to pick
//! a *place*, and in both cases it went where the crosshair was.
//!
//! ## What counts as terrain
//!
//! The floor, the arena's solids, and **stones** — the Elementalist's
//! structures are objects you can aim at and land things on top of, which is
//! most of the point of them being solid.
//!
//! Fighters are deliberately not traced against. An aim that snapped to a body
//! walking through the line would move the target without the player moving the
//! mouse.
//!
//! Instead, **a ray that lands on the floor means the person standing there**.
//! Anything not placed on the ground targets the middle of a fighter who would
//! be standing at that spot rather than the spot itself, so putting the reticle
//! at someone's feet throws the bolt through their chest. On anything that is
//! not the floor -- a platform, a stone -- the point is taken exactly, because
//! there the player is pointing at a surface and not through it.

use crate::arena::{self, Solid};
use crate::class::Structure;
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::V3;
use crate::stones::Field;
use crate::tuning as t;

/// Where a fighter standing at `pos` casts from.
pub fn origin(pos: V3) -> V3 {
    V3::new(pos.x, pos.y.add(t::cast_height()), pos.z)
}

/// What the crosshair is on, and therefore what the ability is aimed at.
///
/// The one entry point. Everything a fighter places or throws comes through
/// here, so that "where the crosshair is" means one thing across the whole game
/// rather than one thing per ability.
///
/// `grounded` is a property of the thing being placed rather than of the move
/// that places it: a pillar of flame comes out of the floor whatever you were
/// doing when you cast it. It decides which of the two readings of the same ray
/// applies -- a place on the ground, or a person standing in it.
pub fn intent(pos: V3, look: Input, reach: Fx, grounded: bool, stones: &Field) -> V3 {
    let eye = crate::camera::eye(pos, look);
    let dir = look.look_dir();
    let cast = origin(pos);

    // Whichever comes first: the terrain, or the edge of what this ability can
    // reach. The reach is a sphere about the *cast origin* rather than a length
    // along the ray, because it is the ability's range and the ability starts at
    // the fighter. The ray only decides the direction.
    let stop = reach_hit(eye, dir, cast, reach);
    let limit = stop.unwrap_or(Fx::MAX);
    let (distance, on_floor) = match (first_hit(eye, dir, limit, stones), stop) {
        (Some(hit), _) => hit,
        (None, Some(edge)) => (edge, false),
        // The ray leaves the arena without meeting anything at all, which wants
        // the reach laid flat rather than a point in the sky: an ability with
        // nowhere to land still has to land somewhere ahead of you.
        (None, None) => {
            let flat = V3::new(dir.x, Fx::ZERO, dir.z).normalized();
            return settle(cast.add(flat.scale(reach)), stones);
        }
    };

    let at = eye.add(dir.scale(distance));
    if grounded {
        settle(at, stones)
    } else if on_floor {
        // A spot on the floor stands for the person standing in it.
        V3::new(at.x, at.y.add(t::body_height().div(Fx::from_int(2))), at.z)
    } else {
        at
    }
}

/// Distance to the first terrain along the ray, if anything is within `limit`.
pub fn trace(from: V3, dir: V3, limit: Fx, stones: &Field) -> Option<Fx> {
    first_hit(from, dir, limit, stones).map(|(d, _)| d)
}

/// The same, and whether what it found was the floor rather than an object.
///
/// The difference matters for anything not placed on the ground: the floor
/// stands for a fighter standing on it, and an object stands for itself.
fn first_hit(from: V3, dir: V3, limit: Fx, stones: &Field) -> Option<(Fx, bool)> {
    let mut best: Option<(Fx, bool)> = None;
    let mut keep = |hit: Option<Fx>, floor: bool| {
        if let Some(d) = hit {
            if d.raw() >= 0 && d.raw() <= limit.raw() && best.is_none_or(|(b, _)| d.raw() < b.raw())
            {
                best = Some((d, floor));
            }
        }
    };

    keep(floor_hit(from, dir), true);
    for solid in arena::SOLIDS.iter() {
        keep(box_hit(from, dir, solid), false);
    }
    for stone in stones.iter().flatten() {
        keep(stone_hit(from, dir, stone), false);
    }
    best
}

/// How far along the ray the ability's own range runs out.
///
/// A sphere about the cast origin, met from outside it or from within, and it
/// is the *far* crossing that counts: the near one is the ray on its way in
/// past the fighter, which is behind anything being aimed at.
fn reach_hit(from: V3, dir: V3, centre: V3, radius: Fx) -> Option<Fx> {
    let m = from.sub(centre);
    let b = m.dot(dir);
    let under = b.mul(b).sub(m.len_sq().sub(radius.mul(radius)));
    if under.raw() < 0 {
        return None;
    }
    let far = under.sqrt().sub(b);
    (far.raw() > 0).then_some(far)
}

/// Drop a point onto whatever it would stand on.
pub fn settle(at: V3, stones: &Field) -> V3 {
    let mut floor = arena::ground_under(at);
    for stone in stones.iter().flatten() {
        let apart = V3::new(at.x.sub(stone.at.x), Fx::ZERO, at.z.sub(stone.at.z)).flat_len();
        if apart.raw() < t::structure_radius().raw() && stone.top().raw() > floor.raw() {
            floor = stone.top();
        }
    }
    V3::new(at.x, floor, at.z)
}

// ---------------------------------------------------------------------------
// The shapes
// ---------------------------------------------------------------------------

/// The arena floor. A plane rather than a box, because that is what the
/// simulation collides against -- `arena::resolve` treats `y <= 0` as the
/// ground and never consults `SOLIDS` for it.
fn floor_hit(from: V3, dir: V3) -> Option<Fx> {
    if dir.y.raw() >= 0 || from.y.raw() < 0 {
        return None;
    }
    Some(from.y.div(dir.y.neg()))
}

/// Slab method: the ray is inside the box over the intersection of the three
/// per-axis intervals it is inside each slab for.
fn box_hit(from: V3, dir: V3, solid: &Solid) -> Option<Fx> {
    let mut near = Fx::ZERO;
    let mut far = Fx::MAX;
    for axis in 0..3 {
        let o = component(from, axis);
        let d = component(dir, axis);
        let lo = component(solid.min, axis);
        let hi = component(solid.max, axis);
        if d.raw() == 0 {
            // Parallel to this pair of faces: either always between them or
            // never. Checked by hand because dividing by zero saturates, which
            // would read as "always".
            if o.raw() < lo.raw() || o.raw() > hi.raw() {
                return None;
            }
            continue;
        }
        let a = lo.sub(o).div(d);
        let b = hi.sub(o).div(d);
        let (enter, leave) = if a.raw() <= b.raw() { (a, b) } else { (b, a) };
        near = near.max(enter);
        far = far.min(leave);
        if near.raw() > far.raw() {
            return None;
        }
    }
    Some(near)
}

/// A stone: an upright cylinder standing on its base, with both end caps.
///
/// The top cap is not a detail. Aiming at the top of a stone is how you put the
/// next thing on top of it, and a cylinder without caps is a tube the ray goes
/// straight down.
fn stone_hit(from: V3, dir: V3, stone: &Structure) -> Option<Fx> {
    let (base, top) = (stone.at.y, stone.top());
    if top.raw() <= base.raw() {
        return None; // still buried, so there is nothing there to hit
    }
    let radius = t::structure_radius();
    let ox = from.x.sub(stone.at.x);
    let oz = from.z.sub(stone.at.z);
    let mut best: Option<Fx> = None;
    let mut keep = |d: Fx| {
        if d.raw() >= 0 && best.is_none_or(|b| d.raw() < b.raw()) {
            best = Some(d);
        }
    };

    // The curved side. `a` is zero looking straight up or down, where there is
    // no side to hit and the caps are the whole answer.
    let a = dir.x.mul(dir.x).add(dir.z.mul(dir.z));
    if a.raw() > 0 {
        let half_b = ox.mul(dir.x).add(oz.mul(dir.z));
        let c = ox.mul(ox).add(oz.mul(oz)).sub(radius.mul(radius));
        let disc = half_b.mul(half_b).sub(a.mul(c));
        if disc.raw() >= 0 {
            let root = disc.sqrt();
            for d in [half_b.neg().sub(root).div(a), half_b.neg().add(root).div(a)] {
                let y = from.y.add(dir.y.mul(d));
                if y.raw() >= base.raw() && y.raw() <= top.raw() {
                    keep(d);
                }
            }
        }
    }

    // The caps.
    if dir.y.raw() != 0 {
        for face in [base, top] {
            let d = face.sub(from.y).div(dir.y);
            if d.raw() < 0 {
                continue;
            }
            let x = ox.add(dir.x.mul(d));
            let z = oz.add(dir.z.mul(d));
            if x.mul(x).add(z.mul(z)).raw() <= radius.mul(radius).raw() {
                keep(d);
            }
        }
    }
    best
}

fn component(v: V3, axis: usize) -> Fx {
    match axis {
        0 => v.x,
        1 => v.y,
        _ => v.z,
    }
}
