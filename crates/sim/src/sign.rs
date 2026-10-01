//! **Floor signs a species draws** beyond its bodies' own telegraphs.
//!
//! Every windup already marks the floor with the hit test's own answer
//! (`Monster::telegraph`, `Critter::telegraph`). Some fights have more on the
//! floor than one body's next hit: the Hornback's stampede is a lane eight
//! metres wide with the lee of every solid in it cut out, a charge's lane
//! ends at a solid that is ringed, a guard is an arc that says *no*. Those are
//! the species' own state -- a lane in the pack's memo, a solid the rules
//! found -- so the species says what to draw, through
//! `FightDecl::signs`, as a short fixed list of plain shapes. The renderer
//! draws the list and nothing else; a test can ask the same list whether
//! what is drawn is where the animals can be. The overlay's rule, applied to
//! the floor: a picture of a hit that is not the hit is worse than none.
//!
//! No allocation: a fixed array, filled each time it is asked for.

use crate::fixed::Fx;
use crate::math::V3;

/// The most signs a fight draws at once.
pub const MAX_SIGNS: usize = 24;

/// What a sign looks like on the floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// A rectangle on the floor: from `at`, `length` along `along`, `width`
    /// across it, centred on the line.
    Strip,
    /// A disc about `at`, `width` across.
    Disc,
    /// A ring about `at`, `width` across: something singled out.
    Ring,
}

/// What it says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Says {
    /// Something is coming here: a windup's colour, filling by `progress`.
    Coming,
    /// It is happening here now: a hit's colour.
    Live,
    /// Safe here, inside something that is not: a lee in a stampede's lane.
    Clear,
    /// Hits here are turned: a guard. Not the hit colour.
    No,
    /// This will stop it: the solid a charge's lane ends at.
    Stops,
    /// A warning, faint: you are standing somewhere a move would reach.
    Faint,
}

/// One sign.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sign {
    pub shape: Shape,
    pub says: Says,
    pub at: V3,
    /// A unit vector in the floor plane: the strip's long side.
    pub along: V3,
    pub length: Fx,
    pub width: Fx,
    /// How far through what it warns of, nought to one: a windup's fill.
    pub progress: Fx,
}

impl Sign {
    pub const fn strip(says: Says, at: V3, along: V3, length: Fx, width: Fx) -> Sign {
        Sign {
            shape: Shape::Strip,
            says,
            at,
            along,
            length,
            width,
            progress: Fx::ONE,
        }
    }

    pub const fn disc(says: Says, at: V3, width: Fx) -> Sign {
        Sign {
            shape: Shape::Disc,
            says,
            at,
            along: V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
            length: Fx::ZERO,
            width,
            progress: Fx::ONE,
        }
    }

    pub const fn ring(says: Says, at: V3, width: Fx) -> Sign {
        Sign {
            shape: Shape::Ring,
            ..Sign::disc(says, at, width)
        }
    }

    pub const fn filled(mut self, progress: Fx) -> Sign {
        self.progress = progress;
        self
    }

    /// Does a strip cover the floor point `p`? Edges count. What a test asks
    /// to hold the drawing to the rule.
    pub fn covers(&self, p: V3) -> bool {
        let d = V3::new(p.x.sub(self.at.x), Fx::ZERO, p.z.sub(self.at.z));
        match self.shape {
            Shape::Strip => {
                let side = V3::new(self.along.z.neg(), Fx::ZERO, self.along.x);
                let a = d.dot(self.along);
                let c = d.dot(side);
                a.raw() >= 0
                    && a.raw() <= self.length.raw()
                    && c.abs().raw() <= self.width.mul(Fx::ratio(1, 2)).raw()
            }
            Shape::Disc | Shape::Ring => {
                crate::math::big_len(d).raw() <= self.width.mul(Fx::ratio(1, 2)).raw()
            }
        }
    }
}

/// A fight's signs this frame: a fixed list.
#[derive(Clone, Copy, Debug)]
pub struct Signs {
    pub all: [Option<Sign>; MAX_SIGNS],
}

impl Default for Signs {
    fn default() -> Signs {
        Signs::NONE
    }
}

impl Signs {
    pub const NONE: Signs = Signs {
        all: [None; MAX_SIGNS],
    };

    /// Add one, if there is room. A full list drops the rest: what is drawn
    /// first is what a species thinks matters most.
    pub fn push(&mut self, s: Sign) {
        if let Some(slot) = self.all.iter_mut().find(|s| s.is_none()) {
            *slot = Some(s);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Sign> {
        self.all.iter().flatten()
    }
}
