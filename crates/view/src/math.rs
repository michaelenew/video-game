//! Just enough vector and quaternion arithmetic to pose a skeleton.
//!
//! Deliberately not a dependency. `view` has none, on purpose -- it is the
//! layer that must stay unit-testable without a window and reusable if the
//! engine is ever swapped -- and forward kinematics needs exactly two
//! operations: rotate a vector, and compose two rotations. That is a hundred
//! lines, not a crate.
//!
//! Rotations are quaternions rather than Euler triples. Euler angles are what
//! a *person* should edit, and the pose channels stay Euler-shaped for that
//! reason; but composing a chain of them is where gimbal lock bites, and a
//! shoulder that flips when the arm passes overhead is exactly the artefact a
//! fighting game cannot afford.

/// A point or direction in character-local space: +Y up, +Z along facing.
pub type V3 = [f32; 3];

pub const ZERO: V3 = [0.0, 0.0, 0.0];

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(a: V3, k: f32) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn length(a: V3) -> f32 {
    dot(a, a).sqrt()
}

/// Unit vector, or `fallback` when the input is too short to have a direction.
pub fn normalize_or(a: V3, fallback: V3) -> V3 {
    let len = length(a);
    if len < 1e-6 {
        fallback
    } else {
        scale(a, 1.0 / len)
    }
}

/// A rotation, stored `x, y, z, w` -- the order Bevy's `Quat::from_xyzw` wants,
/// so the renderer hands this straight over with no shuffling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat(pub [f32; 4]);

impl Quat {
    pub const IDENTITY: Quat = Quat([0.0, 0.0, 0.0, 1.0]);

    pub fn axis_angle(axis: V3, angle: f32) -> Quat {
        let a = normalize_or(axis, [0.0, 1.0, 0.0]);
        let (s, c) = (angle * 0.5).sin_cos();
        Quat([a[0] * s, a[1] * s, a[2] * s, c])
    }

    pub fn from_x(angle: f32) -> Quat {
        let (s, c) = (angle * 0.5).sin_cos();
        Quat([s, 0.0, 0.0, c])
    }

    pub fn from_y(angle: f32) -> Quat {
        let (s, c) = (angle * 0.5).sin_cos();
        Quat([0.0, s, 0.0, c])
    }

    pub fn from_z(angle: f32) -> Quat {
        let (s, c) = (angle * 0.5).sin_cos();
        Quat([0.0, 0.0, s, c])
    }

    /// `self` then... no: `self * other` applies `other` first, as matrices do.
    // Named `mul` rather than given the `Mul` trait on purpose: composition
    // order is the one thing about this rig that everything has to agree on,
    // and `a.mul(b)` reads left to right at every call site where `a * b`
    // invites a reader to guess.
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, other: Quat) -> Quat {
        let [ax, ay, az, aw] = self.0;
        let [bx, by, bz, bw] = other.0;
        Quat([
            aw * bx + ax * bw + ay * bz - az * by,
            aw * by - ax * bz + ay * bw + az * bx,
            aw * bz + ax * by - ay * bx + az * bw,
            aw * bw - ax * bx - ay * by - az * bz,
        ])
    }

    pub fn rotate(self, v: V3) -> V3 {
        // v + 2w(q x v) + 2(q x (q x v)) -- the usual expansion, which avoids
        // building a matrix for a single vector.
        let q = [self.0[0], self.0[1], self.0[2]];
        let w = self.0[3];
        let t = scale(cross(q, v), 2.0);
        add(add(v, scale(t, w)), cross(q, t))
    }

    pub fn conjugate(self) -> Quat {
        Quat([-self.0[0], -self.0[1], -self.0[2], self.0[3]])
    }

    /// Renormalise. Long chains of multiplications drift, and a drifting
    /// quaternion scales the mesh it is applied to.
    pub fn normalized(self) -> Quat {
        let [x, y, z, w] = self.0;
        let len = (x * x + y * y + z * z + w * w).sqrt();
        if len < 1e-6 {
            Quat::IDENTITY
        } else {
            Quat([x / len, y / len, z / len, w / len])
        }
    }
}

/// Shortest-arc rotation taking `from` to `to`, both unit vectors.
pub fn rotation_between(from: V3, to: V3) -> Quat {
    let d = dot(from, to);
    if d > 0.9999 {
        return Quat::IDENTITY;
    }
    if d < -0.9999 {
        // Opposed: any perpendicular axis is a valid half turn. Pick one that
        // is definitely not parallel to `from`.
        let seed = if from[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        return Quat::axis_angle(cross(from, seed), std::f32::consts::PI);
    }
    let axis = cross(from, to);
    Quat([axis[0], axis[1], axis[2], 1.0 + d]).normalized()
}

pub fn deg(v: f32) -> f32 {
    v * (std::f32::consts::PI / 180.0)
}

pub fn to_deg(v: f32) -> f32 {
    v * (180.0 / std::f32::consts::PI)
}
