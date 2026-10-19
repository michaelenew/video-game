//! The morph: a companion that becomes what it is fed.
//!
//! The world's first use of breeding (`docs/design/exploration/0022`
//! §12), built for short sessions: the reward has to land at once. Feed it
//! an **egg** and it starts over as that creature's young -- the egg's
//! genome, small. Feed it a **trophy** and that is a **generation**: the
//! trophy grafted on at the morph's rates ([`super::cross::feed`]), always
//! with a change you can see, and the morph grows a step toward full size.
//! What changed is returned ([`Feeding`]), and [`changes`] finds it again
//! from two genomes, so the game can say it.
//!
//! A morph is a value small enough for the snapshot: its genome, how grown
//! it is, its generation, its dice. Its body is built from those wherever
//! it is drawn.

use super::cross::{self, Trophy, TrophyKind};
use super::plan::assess;
use super::{
    Build, Crown, Genome, HeadKind, Hide, Jaw, Rng, Role, STATIONS, Stance, Tail, TailTip, Tip, fx,
};
use crate::fixed::Fx;

/// One thing a feeding changed, for the game to put into words.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Line {
    Stance(Stance),
    Head(HeadKind),
    /// Its tail is now this: grown, lost, raised, drooping.
    Tail(Tail),
    /// Its body took on a thorax or an abdomen, or lost one.
    Segments,
    /// A pair at a station became this build, in this role.
    Limb(usize, Build, Role),
    /// A pair budded where there was none.
    NewLimb(usize, Build),
    /// A pair at a station is gone.
    LostLimb(usize),
    /// A pair at a station ends in this now: a hoof, a claw, a blade.
    Tip(usize, Tip),
    Jaw(Jaw),
    Crown(Crown),
    TailTip(TailTip),
    Hide(Hide),
    Back,
    Flies(bool),
    Burrows(bool),
    /// Its adult size moved, by this much (a share, signed).
    Size(Fx),
}

/// What one feeding did.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Feeding {
    /// The trophy, or none for an egg.
    pub trophy: Option<TrophyKind>,
    pub lines: [Option<Line>; 6],
}

impl Feeding {
    fn push(&mut self, line: Line) {
        if let Some(slot) = self.lines.iter_mut().find(|l| l.is_none()) {
            *slot = Some(line);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.iter().all(|l| l.is_none())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Line> {
        self.lines.iter().flatten()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Morph {
    /// What it is, grown.
    pub genome: Genome,
    /// How grown, nought to one: it is drawn at this share of its size.
    pub growth: Fx,
    /// Trophies eaten since its egg.
    pub generation: u16,
    /// The creature its egg was, by species id; [`Morph::UNFORMED`] for
    /// one that has eaten no egg yet.
    pub of: u8,
    /// Whose it is: its dice are its seat's, its genome's and its
    /// generation's, so two morphs fed alike still differ.
    pub seat: u8,
}

impl Morph {
    /// What a morph that has eaten no egg is of.
    pub const UNFORMED: u8 = u8::MAX;

    /// An egg, hatched: the creature's young.
    pub fn hatch(egg: &Genome, seat: u8) -> Morph {
        Morph {
            genome: *egg,
            growth: crate::tuning::morph_born(),
            generation: 0,
            of: Morph::UNFORMED,
            seat,
        }
    }

    /// Fed an egg of species `of`: it starts over as that creature, young.
    pub fn eat_egg(&mut self, egg: &Genome, of: u8) {
        *self = Morph {
            of,
            ..Morph::hatch(egg, self.seat)
        };
    }

    /// This feeding's dice.
    fn rng(&self) -> Rng {
        Rng::new(
            self.genome.fingerprint()
                ^ (self.generation as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                ^ (self.seat as u64) << 56,
        )
    }

    /// Fed a trophy of `kind` from `donor`: a generation.
    pub fn eat(&mut self, kind: TrophyKind, donor: &Genome) -> Feeding {
        let before = self.genome;
        let trophy = Trophy {
            kind,
            donor: *donor,
        };
        let mut rng = self.rng();
        let out = cross::feed(&before, &trophy, &mut rng);
        self.genome = out.genome;
        self.generation = self.generation.saturating_add(1);
        self.growth = self.growth.add(crate::tuning::morph_grows()).min(Fx::ONE);
        changes(&before, &self.genome, Some(kind))
    }

    /// The genome it is drawn and moves as: its size scaled by how grown
    /// it is -- a young one is small.
    pub fn shown(&self) -> Genome {
        let mut g = self.genome;
        g.sliders.size = g.sliders.size.mul(self.growth);
        g
    }
}

/// Everything a player would see differ between `a` and `b`: the one
/// definition of a visible change, which [`cross::visible`] asks too.
pub fn changes(a: &Genome, b: &Genome, trophy: Option<TrophyKind>) -> Feeding {
    let mut f = Feeding {
        trophy,
        ..Feeding::default()
    };
    if a.stance != b.stance {
        f.push(Line::Stance(b.stance));
    }
    if a.head != b.head {
        f.push(Line::Head(b.head));
    }
    if a.segments.tail != b.segments.tail {
        f.push(Line::Tail(b.segments.tail));
    }
    if a.segments.thorax != b.segments.thorax || a.segments.abdomen != b.segments.abdomen {
        f.push(Line::Segments);
    }
    for i in 0..STATIONS {
        match (a.limbs[i], b.limbs[i]) {
            (Some(x), Some(y)) if x.build != y.build || x.role != y.role => {
                f.push(Line::Limb(i, y.build, y.role));
            }
            (Some(x), Some(y)) if x.tip != y.tip => f.push(Line::Tip(i, y.tip)),
            (None, Some(y)) => f.push(Line::NewLimb(i, y.build)),
            (Some(_), None) => f.push(Line::LostLimb(i)),
            _ => {}
        }
    }
    if a.minors.jaw != b.minors.jaw {
        f.push(Line::Jaw(b.minors.jaw));
    }
    if a.minors.crown != b.minors.crown {
        f.push(Line::Crown(b.minors.crown));
    }
    if a.minors.tail_tip != b.minors.tail_tip {
        f.push(Line::TailTip(b.minors.tail_tip));
    }
    if a.minors.hide != b.minors.hide {
        f.push(Line::Hide(b.minors.hide));
    }
    if a.minors.back != b.minors.back {
        f.push(Line::Back);
    }
    let (ca, cb) = (assess(a).caps, assess(b).caps);
    if ca.flies != cb.flies {
        f.push(Line::Flies(cb.flies));
    }
    if ca.burrows != cb.burrows {
        f.push(Line::Burrows(cb.burrows));
    }
    let d = b.sliders.size.sub(a.sliders.size);
    if d.abs() >= fx(1, 10) {
        f.push(Line::Size(d));
    }
    f
}
