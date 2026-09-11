//! Static arena geometry and collision.
//!
//! The arena is a fixed set of axis-aligned boxes. Not a general physics
//! engine, and deliberately so: capsule-versus-box and capsule-versus-capsule
//! is all an arena fighter needs, and writing it here is far less work than
//! establishing cross-platform determinism in someone else's solver.
//!
//! Geometry is a constant rather than part of `World`. It never changes during
//! a match, so it does not belong in the rollback snapshot.

use crate::fixed::Fx;
use crate::math::V3;
use crate::tuning as t;

// The body is a vertical cylinder -- capsule-ish is close enough when nothing
// rolls or ragdolls -- and its size lives in the Oven. It used to be a `const`
// here as well as a knob there, which meant tuning the body changed what
// attacks could reach but not what walls could stop.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Solid {
    pub min: V3,
    pub max: V3,
}

impl Solid {
    pub const fn new(min: V3, max: V3) -> Solid {
        Solid { min, max }
    }
}

const fn vr(x: (i32, i32), y: (i32, i32), z: (i32, i32)) -> V3 {
    V3::new(
        Fx::ratio(x.0, x.1),
        Fx::ratio(y.0, y.1),
        Fx::ratio(z.0, z.1),
    )
}

pub const ARENA_HALF: Fx = Fx::from_int(14);
pub const WALL_HEIGHT: Fx = Fx::ratio(3, 2);

/// The blockout. Four low walls and two raised platforms, so the arena has an
/// edge and more than one height. Pushback toward an edge is meaningless
/// without an edge.
///
/// The walls are deliberately low. Tall ones read as a box and put geometry
/// between the camera and the fight, and the camera's only answer is to pull in
/// toward the fighter's back -- it will not turn away, because the angle
/// belongs to the player.
pub const SOLIDS: [Solid; 6] = [
    // Walls, one unit thick, sitting just outside the play area.
    Solid::new(
        vr((-15, 1), (0, 1), (-15, 1)),
        vr((-14, 1), (3, 2), (15, 1)),
    ),
    Solid::new(vr((14, 1), (0, 1), (-15, 1)), vr((15, 1), (3, 2), (15, 1))),
    Solid::new(
        vr((-15, 1), (0, 1), (-15, 1)),
        vr((15, 1), (3, 2), (-14, 1)),
    ),
    Solid::new(vr((-15, 1), (0, 1), (14, 1)), vr((15, 1), (3, 2), (15, 1))),
    // Two low platforms. Low enough to jump onto, high enough that standing on
    // one changes the fight.
    Solid::new(vr((-9, 1), (0, 1), (-4, 1)), vr((-5, 1), (3, 2), (4, 1))),
    Solid::new(vr((5, 1), (0, 1), (-4, 1)), vr((9, 1), (3, 2), (4, 1))),
];

/// Result of resolving a body against the arena.
#[derive(Clone, Copy, Debug)]
pub struct Resolved {
    pub pos: V3,
    pub vel: V3,
    pub grounded: bool,
}

/// Push a body out of the arena geometry.
///
/// Resolves along the axis of least penetration, one solid at a time, in a
/// fixed order. Order matters for determinism, which is why `SOLIDS` is a
/// fixed-size array rather than anything with unstable iteration.
pub fn resolve(mut pos: V3, mut vel: V3, was_grounded: bool) -> Resolved {
    let mut grounded = false;

    // Ground plane first.
    if pos.y.raw() <= 0 {
        pos.y = Fx::ZERO;
        if vel.y.raw() < 0 {
            vel.y = Fx::ZERO;
        }
        grounded = true;
    }

    for solid in SOLIDS.iter() {
        // Expand the box by the body radius horizontally, so the body can be
        // treated as a point in X and Z.
        let min_x = solid.min.x.sub(t::body_radius());
        let max_x = solid.max.x.add(t::body_radius());
        let min_z = solid.min.z.sub(t::body_radius());
        let max_z = solid.max.z.add(t::body_radius());

        let feet = pos.y;
        let head = pos.y.add(t::body_height());

        let inside = pos.x.raw() > min_x.raw()
            && pos.x.raw() < max_x.raw()
            && pos.z.raw() > min_z.raw()
            && pos.z.raw() < max_z.raw()
            && head.raw() > solid.min.y.raw()
            && feet.raw() < solid.max.y.raw();
        if !inside {
            continue;
        }

        // Penetration depth on each axis; resolve the shallowest.
        let px = min_penetration(pos.x, min_x, max_x);
        let pz = min_penetration(pos.z, min_z, max_z);
        let py_up = solid.max.y.sub(feet); // push up onto the top
        let py_down = head.sub(solid.min.y); // push down out from under

        let ax = px.abs();
        let az = pz.abs();
        let ay = py_up.min(py_down).abs();

        if ay.raw() <= ax.raw() && ay.raw() <= az.raw() {
            if py_up.raw() <= py_down.raw() {
                pos.y = solid.max.y;
                if vel.y.raw() < 0 {
                    vel.y = Fx::ZERO;
                }
                grounded = true;
            } else {
                pos.y = solid.min.y.sub(t::body_height());
                if vel.y.raw() > 0 {
                    vel.y = Fx::ZERO;
                }
            }
        } else if ax.raw() <= az.raw() {
            pos.x = pos.x.add(px);
            vel.x = Fx::ZERO;
        } else {
            pos.z = pos.z.add(pz);
            vel.z = Fx::ZERO;
        }
    }

    // Standing exactly on a surface reads as grounded even when the resolver
    // did not have to move anything this tick.
    if !grounded && was_grounded && vel.y.raw() <= 0 && supported(pos) {
        grounded = true;
    }

    Resolved { pos, vel, grounded }
}

/// Signed push needed to leave the span by the nearer edge.
fn min_penetration(v: Fx, lo: Fx, hi: Fx) -> Fx {
    let out_lo = lo.sub(v); // negative: push toward lo
    let out_hi = hi.sub(v); // positive: push toward hi
    if out_hi.abs().raw() < out_lo.abs().raw() {
        out_hi
    } else {
        out_lo
    }
}

/// Is there a surface directly beneath the feet?
fn supported(pos: V3) -> bool {
    const SKIN: Fx = Fx::ratio(1, 32);
    if pos.y.abs().raw() <= SKIN.raw() {
        return true;
    }
    SOLIDS.iter().any(|s| {
        pos.x.raw() > s.min.x.sub(t::body_radius()).raw()
            && pos.x.raw() < s.max.x.add(t::body_radius()).raw()
            && pos.z.raw() > s.min.z.sub(t::body_radius()).raw()
            && pos.z.raw() < s.max.z.add(t::body_radius()).raw()
            && pos.y.sub(s.max.y).abs().raw() <= SKIN.raw()
    })
}

/// Height of the highest surface under a point, for spawning and for the
/// renderer's shadow projection.
pub fn ground_under(pos: V3) -> Fx {
    let mut best = Fx::ZERO;
    for s in SOLIDS.iter() {
        let over = pos.x.raw() > s.min.x.raw()
            && pos.x.raw() < s.max.x.raw()
            && pos.z.raw() > s.min.z.raw()
            && pos.z.raw() < s.max.z.raw();
        if over && s.max.y.raw() > best.raw() {
            best = s.max.y;
        }
    }
    best
}
