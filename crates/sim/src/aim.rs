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
//! ## Why the ray starts at the fighter and not at the eye
//!
//! It would be easier to trace from the camera, and every third-person shooter
//! does. It also means the ability travels a *different* line from the one the
//! player drew — the two converge on the target but diverge in between, which
//! is why a shot you lined up past a corner in those games clips the corner.
//!
//! Starting at the fighter makes the aimed line and the travelled line the same
//! line, so aiming below the horizon gives the direction to a point on the
//! ground for free: point at the floor between you and someone, and the ray
//! from your chest to that point is the ray that hits them on the way.
//!
//! It also keeps the eye out of the simulation, which it has to be. The camera
//! carries a per-player distance setting and a smoothed follow position;
//! solving the aim from there would mean two peers at different zoom levels
//! placing a fire pillar in different spots and neither of them being wrong.
//!
//! The camera is then arranged around this rather than the other way about: it
//! orbits the cast origin, and it is **pointed at whatever this module says the
//! player is aiming at**, which is what pins the crosshair to the exact centre
//! of the screen. See `view::camera`.
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
//! mouse, and "aim at the floor under them" already hits someone of their
//! height, which is the shot that wants to be available.

use crate::arena::{self, Solid};
use crate::class::Structure;
use crate::fixed::Fx;
use crate::math::V3;
use crate::stones::Field;
use crate::tuning as t;

/// Where a fighter standing at `pos` casts from.
pub fn origin(pos: V3) -> V3 {
    V3::new(pos.x, pos.y.add(t::cast_height()), pos.z)
}

/// Where an ability aimed along `dir` from `from` with this much reach lands.
///
/// `grounded` is a property of the thing being placed rather than of the move
/// that places it: a pillar of flame comes out of the floor whatever you were
/// doing when you cast it. It means two things — the target is settled onto
/// whatever surface is under it, and an aim that leaves the ground entirely
/// falls back to the reach laid flat ahead rather than to a point in the sky.
pub fn target(from: V3, dir: V3, reach: Fx, grounded: bool, stones: &Field) -> V3 {
    match trace(from, dir, reach, stones) {
        Some(hit) => {
            let at = from.add(dir.scale(hit));
            if grounded { settle(at, stones) } else { at }
        }
        // Nothing within reach. Out to the edge of the sphere -- flattened
        // first if it has to come down somewhere, so looking at the sky still
        // puts the ability the full distance ahead instead of at your feet.
        None if grounded => {
            let flat = V3::new(dir.x, Fx::ZERO, dir.z).normalized();
            settle(from.add(flat.scale(reach)), stones)
        }
        None => from.add(dir.scale(reach)),
    }
}

/// Distance to the first terrain along the ray, if anything is within `limit`.
pub fn trace(from: V3, dir: V3, limit: Fx, stones: &Field) -> Option<Fx> {
    let mut best: Option<Fx> = None;
    let mut keep = |hit: Option<Fx>| {
        if let Some(d) = hit {
            if d.raw() >= 0 && d.raw() <= limit.raw() && best.is_none_or(|b| d.raw() < b.raw()) {
                best = Some(d);
            }
        }
    };

    keep(floor_hit(from, dir));
    for solid in arena::SOLIDS.iter() {
        keep(box_hit(from, dir, solid));
    }
    for stone in stones.iter().flatten() {
        keep(stone_hit(from, dir, stone));
    }
    best
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
