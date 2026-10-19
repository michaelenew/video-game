//! Rearing: an egg incubated, a hatchling fed and grown, the rebellion at
//! full size, and the taming.
//!
//! The hatchling follows you and helps fight; it can die, so a careful
//! keeper keeps it back. It grows by what it eats and by what it fights:
//! food that matches its diet and its tier grows it, a fight against
//! something harder than itself grows it faster and raises the size it
//! will reach. At full size it **rebels** and must be subdued -- beaten --
//! once; then it is tame. Everything here is a pure function of the ward
//! and what was done to it, with no world in it yet.

use super::plan::assess;
use super::{Diet, Genome, fx, unit};
use crate::fixed::Fx;

/// Where a ward is in its life.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    /// Warmth gathered, nought to one; it hatches at one.
    Egg {
        warmth: Fx,
    },
    /// Grown, nought to one; at one it rebels.
    Hatchling {
        grown: Fx,
    },
    /// Full size, and no longer yours until it is beaten.
    Rebel,
    Tame,
}

/// A creature in your care.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ward {
    /// What it will be, grown.
    pub genome: Genome,
    pub stage: Stage,
    /// How much bigger than its genome it will reach, from the fights it
    /// has won: nought to one, a tenth of the size slider at most.
    pub vigor: Fx,
}

/// What was given to eat.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Food {
    Grass,
    /// Meat of a creature of this tier.
    Meat {
        tier: u8,
    },
    /// Greens from somewhere hard to reach, of this tier: the treetops,
    /// the Needle.
    Greens {
        tier: u8,
    },
}

impl Food {
    pub const fn tier(self) -> u8 {
        match self {
            Food::Grass => 1,
            Food::Meat { tier } | Food::Greens { tier } => tier,
        }
    }
}

/// What a feeding did.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fed {
    /// It grew this much.
    Grew(Fx),
    /// It will not eat that.
    Refused,
    /// Not a hatchling: eggs and grown creatures do not eat for growth.
    NotHungry,
}

/// Warmth an egg gathers per frame kept warm: a thousand frames to hatch.
pub const WARMTH_PER_FRAME: Fx = fx(1, 1000);
/// Growth from one meal of exactly its tier.
pub const GROWTH_PER_MEAL: Fx = fx(1, 12);
/// Growth from a fight won against something of its own tier.
pub const GROWTH_PER_FIGHT: Fx = fx(1, 20);
/// Vigor from a fight won against something a tier harder.
pub const VIGOR_PER_HARD_FIGHT: Fx = fx(1, 10);
/// The size slider at hatching, as a share of the adult's.
pub const HATCHLING_SIZE: Fx = fx(2, 5);

impl Ward {
    pub fn egg(genome: Genome) -> Ward {
        Ward {
            genome,
            stage: Stage::Egg { warmth: Fx::ZERO },
            vigor: Fx::ZERO,
        }
    }

    /// The tier it will be at full size, with its vigor.
    pub fn adult_tier(&self) -> u8 {
        assess(&self.adult()).tier
    }

    /// Its genome grown, with the vigor folded in.
    pub fn adult(&self) -> Genome {
        let mut g = self.genome;
        g.sliders.size = unit(g.sliders.size.add(self.vigor.mul(fx(1, 10))));
        g
    }

    /// The body it has now: an egg has none, a hatchling is a smaller
    /// version of the adult, a rebel or a tame one is the adult.
    pub fn current(&self) -> Option<Genome> {
        match self.stage {
            Stage::Egg { .. } => None,
            Stage::Hatchling { grown } => {
                let adult = self.adult();
                let mut g = adult;
                let share = HATCHLING_SIZE.add(Fx::ONE.sub(HATCHLING_SIZE).mul(grown));
                g.sliders.size = adult.sliders.size.mul(share);
                Some(g)
            }
            Stage::Rebel | Stage::Tame => Some(self.adult()),
        }
    }

    /// Keep the egg warm for this many frames.
    pub fn incubate(&mut self, frames: u32) {
        if let Stage::Egg { warmth } = self.stage {
            let w = warmth.add(WARMTH_PER_FRAME.mul(Fx::from_int(frames.min(1 << 20) as i32)));
            self.stage = if w >= Fx::ONE {
                Stage::Hatchling { grown: Fx::ZERO }
            } else {
                Stage::Egg { warmth: w }
            };
        }
    }

    /// Feed it. A grazer eats grass and greens, a carnivore meat, an
    /// omnivore anything; food under its tier grows it less, so a
    /// dangerous creature needs dangerous food.
    pub fn feed(&mut self, food: Food) -> Fed {
        let Stage::Hatchling { grown } = self.stage else {
            return Fed::NotHungry;
        };
        let eats = matches!(
            (self.genome.diet(), food),
            (Diet::Grazer, Food::Grass | Food::Greens { .. })
                | (Diet::Carnivore, Food::Meat { .. })
                | (Diet::Omnivore, _)
        );
        if !eats {
            return Fed::Refused;
        }
        let need = self.adult_tier().max(1) as i32;
        let got = (food.tier().max(1) as i32).min(need);
        let amount = GROWTH_PER_MEAL.mul(Fx::ratio(got, need));
        self.grow(grown, amount);
        Fed::Grew(amount)
    }

    /// It helped win a fight against something of this tier.
    pub fn fought(&mut self, foe_tier: u8) {
        let Stage::Hatchling { grown } = self.stage else {
            return;
        };
        let own = self.adult_tier();
        let amount = if foe_tier > own {
            self.vigor = unit(self.vigor.add(VIGOR_PER_HARD_FIGHT));
            GROWTH_PER_FIGHT.mul(Fx::from_int(2))
        } else if foe_tier == own {
            GROWTH_PER_FIGHT
        } else {
            GROWTH_PER_FIGHT.mul(fx(1, 2))
        };
        self.grow(grown, amount);
    }

    fn grow(&mut self, grown: Fx, amount: Fx) {
        let g = grown.add(amount);
        self.stage = if g >= Fx::ONE {
            Stage::Rebel
        } else {
            Stage::Hatchling { grown: g }
        };
    }

    /// It was beaten in its rebellion: tame now. False if it was not
    /// rebelling.
    pub fn subdue(&mut self) -> bool {
        if self.stage == Stage::Rebel {
            self.stage = Stage::Tame;
            true
        } else {
            false
        }
    }

    pub fn is_tame(&self) -> bool {
        self.stage == Stage::Tame
    }
}
