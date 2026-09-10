//! Vector math over fixed point.

use crate::fixed::Fx;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct V3 {
    pub x: Fx,
    pub y: Fx,
    pub z: Fx,
}

impl V3 {
    pub const ZERO: V3 = V3 {
        x: Fx::ZERO,
        y: Fx::ZERO,
        z: Fx::ZERO,
    };

    pub const fn new(x: Fx, y: Fx, z: Fx) -> V3 {
        V3 { x, y, z }
    }

    pub const fn add(self, o: V3) -> V3 {
        V3::new(self.x.add(o.x), self.y.add(o.y), self.z.add(o.z))
    }

    pub const fn sub(self, o: V3) -> V3 {
        V3::new(self.x.sub(o.x), self.y.sub(o.y), self.z.sub(o.z))
    }

    pub const fn scale(self, s: Fx) -> V3 {
        V3::new(self.x.mul(s), self.y.mul(s), self.z.mul(s))
    }

    pub const fn dot(self, o: V3) -> Fx {
        self.x.mul(o.x).add(self.y.mul(o.y)).add(self.z.mul(o.z))
    }

    pub const fn len_sq(self) -> Fx {
        self.dot(self)
    }

    pub const fn len(self) -> Fx {
        self.len_sq().sqrt()
    }

    /// Zero-length input returns zero rather than NaN-equivalent nonsense.
    pub const fn normalized(self) -> V3 {
        let l = self.len();
        if l.raw() == 0 {
            V3::ZERO
        } else {
            V3::new(self.x.div(l), self.y.div(l), self.z.div(l))
        }
    }

    /// Horizontal plane only. Most gameplay distance checks want this.
    pub const fn flat_len(self) -> Fx {
        self.x.mul(self.x).add(self.z.mul(self.z)).sqrt()
    }
}
