//! Crossbreeding: a creature as a genome, and the builder that turns one into
//! a body, a move set and a verdict on whether it is a plausible animal.
//!
//! Every creature in the bestiary is a hand-written table
//! ([`crate::species`]). This module is the other way round: a creature is a
//! **genome** -- a handful of discrete choices about the body plan and a row
//! of sliders -- and what it looks like, how it moves and what it can do are
//! *derived* from that, deterministically. Two genomes can be crossed, a
//! genome can be pushed toward another by a trophy, and a lineage drifts over
//! generations. The litmus test of the builder is that it can rebuild the
//! existing cast closely ([`archetypes`], `tests/breed.rs`).
//!
//! ## Majors, minors and sliders
//!
//! Three kinds of gene, moving at three speeds:
//!
//! - **Majors** are the body plan: the stance the torso sits in, where the
//!   head is, whether there is a thorax, an abdomen, a tail; how many limb
//!   pairs there are, where each attaches, what each is *for* (walking,
//!   striking, both, flying) and what it is built like. Majors are discrete,
//!   and they change rarely -- a trophy flips one with a small chance, a
//!   mutation with a far smaller one.
//! - **Minors** graft onto any plan: what is at the end of a limb, what the
//!   jaw is, what grows out of the crown, what the tail ends in, what the
//!   hide is. Horns fit on any head. They move more easily.
//! - **Sliders** are the magnitudes: size, mass, how upright the torso is,
//!   every limb's length and girth, speed, reaction, aggression, tempo, guard.
//!   They move every generation, by a random share toward a donor and by a
//!   normally distributed mutation. Some have more inertia than others.
//!
//! ## Cohesion: every major wants things of the rest
//!
//! A major is not just a shape; it is a **preference** about the body it is
//! part of ([`plan::Wants`]). A horizontal torso wants heavy walking limbs,
//! at least two pairs on the ground, a level head. A raptorial fore pair
//! wants an upright torso, long thin limbs, a quick tempo. After every
//! crossing the sliders are **settled** toward the weighted mean of what the
//! plan's parts want, with the torso's say the strongest, and the plan is
//! **repaired** to the nearest body that stands up (at least one walking
//! pair on the ground unless it has no limbs at all; wings only at the
//! fore; a tail tip only on a tail). That is what keeps a cross between two
//! creatures a creature rather than a pile of parts: the Ridgeback given a
//! Mantis's arms comes out with mid-long limbs, walking but used to strike,
//! on a torso tilted a little up to carry them.
//!
//! ## What is here
//!
//! | File | What |
//! | --- | --- |
//! | [`mod.rs`] | [`Genome`] and its genes, the sliders, the deterministic [`Rng`] |
//! | [`plan`] | what every major wants, the settle, the repair, the plausibility verdict |
//! | [`build`] | the genome as a body (bones, parts, legs in metres) and a move set |
//! | [`cross`] | breeding two genomes, grafting a trophy, mutation |
//! | [`rear`] | an egg incubated, a hatchling fed and grown, the rebellion, the taming |
//! | [`archetypes`] | the eleven creatures of the bestiary as genomes, and the trophies they drop |
//!
//! Nothing here touches the `World` yet. It is the builder and the evaluator
//! the rest will be made from, measured by `tests/breed.rs` and printed by
//! `cargo run -p sim --bin breedcheck`. The design is
//! `docs/design/exploration/0021_crossbreeding.md`.

pub mod archetypes;
pub mod build;
pub mod cross;
pub mod plan;
pub mod rear;

use crate::fixed::Fx;

/// A fixed-point value from a ratio, for the tables in this module.
pub(crate) const fn fx(num: i32, den: i32) -> Fx {
    Fx::ratio(num, den)
}

// ---------------------------------------------------------------------------
// Majors
// ---------------------------------------------------------------------------

/// How the torso sits: the first and heaviest decision in a body plan.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Stance {
    /// Horizontal on walking limbs: the Ridgeback, the Pair, the Hornback;
    /// the Galewing too, level on two legs with its wings at the fore.
    Level,
    /// Torso raised toward the vertical on the hind limbs, the fore limbs
    /// free: the Mantis.
    Upright,
    /// Belly close to the floor, limbs splayed to the sides: the Mireback,
    /// the Broodmother.
    Lowslung,
    /// No limbs at all; the body is the mover: the Sandmaw.
    Limbless,
}

impl Stance {
    pub const ALL: [Stance; 4] = [
        Stance::Level,
        Stance::Upright,
        Stance::Lowslung,
        Stance::Limbless,
    ];
}

/// Where the head is carried.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum HeadKind {
    /// Out in front on a neck, level with the back.
    Forward,
    /// Up on a raised neck, above the back: the Ridgeback, the Galewing.
    Raised,
    /// Hung low, under the line of the back: the Hornback bull's, a grazer.
    Low,
    /// No neck; the front of the body is the head: the Sandmaw.
    Merged,
}

impl HeadKind {
    pub const ALL: [HeadKind; 4] = [
        HeadKind::Forward,
        HeadKind::Raised,
        HeadKind::Low,
        HeadKind::Merged,
    ];
}

/// Whether there is a tail, and how it is carried.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Tail {
    None,
    /// Carried level off the hips: the Ridgeback's.
    Level,
    /// Dropping to the floor behind.
    Drooping,
    /// Held up over the back: a scorpion's, a cat's.
    Raised,
}

impl Tail {
    pub const ALL: [Tail; 4] = [Tail::None, Tail::Level, Tail::Drooping, Tail::Raised];
}

/// The body's segments past the torso. Each is a yes or a no.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Segments {
    /// A second body segment ahead of the one the hind limbs hang from, the
    /// fore limbs on it: the Mantis's, the Broodmother's cephalothorax.
    pub thorax: bool,
    /// A heavy segment behind the hips: the Broodmother's.
    pub abdomen: bool,
    pub tail: Tail,
}

/// Where a limb pair attaches along the body. Four stations, fore to hind;
/// a body has a pair at any of them or none.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Station {
    Fore = 0,
    MidFore = 1,
    MidHind = 2,
    Hind = 3,
}

impl Station {
    pub const ALL: [Station; STATIONS] = [
        Station::Fore,
        Station::MidFore,
        Station::MidHind,
        Station::Hind,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }
}

/// How many limb pairs a body can have.
pub const STATIONS: usize = 4;

/// What a limb pair is *for*. A major: changing it changes what the animal is.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Role {
    /// It carries the body and nothing else.
    Walk,
    /// It is a weapon and does not touch the floor.
    Strike,
    /// It walks, and is swung too: a cat's forelegs, a stamping foot.
    Both,
    /// It flies.
    Wing,
}

impl Role {
    pub const ALL: [Role; 4] = [Role::Walk, Role::Strike, Role::Both, Role::Wing];

    /// Does it carry weight on the floor?
    pub const fn grounded(self) -> bool {
        matches!(self, Role::Walk | Role::Both)
    }
}

/// What a limb pair is built like: the **basis** a crossed limb keeps,
/// which decides its proportions, what it prefers of the body it is on, and
/// what it can be swung as.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Build {
    /// Straight and heavy, a pillar: the Ridgeback's, the Siegeshell's.
    Column,
    /// Bent and sprung, a jumper's: the Pair's, the Veilstalker's.
    Sprung,
    /// Folded blades, held up: the Mantis's forelimbs.
    Raptorial,
    /// Thin and many-jointed, splayed: the Broodmother's.
    Spindle,
    /// Short and squat, splayed out from the side: the Mireback's.
    Stub,
    /// A wing: the Galewing's.
    Wing,
}

impl Build {
    pub const ALL: [Build; 6] = [
        Build::Column,
        Build::Sprung,
        Build::Raptorial,
        Build::Spindle,
        Build::Stub,
        Build::Wing,
    ];
}

/// What is at the end of a limb. A minor.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Tip {
    Pad,
    Hoof,
    Claw,
    Talon,
    /// A blade's edge: the Mantis's.
    Blade,
    /// Feathers, for a wing.
    Feather,
}

impl Tip {
    pub const ALL: [Tip; 6] = [
        Tip::Pad,
        Tip::Hoof,
        Tip::Claw,
        Tip::Talon,
        Tip::Blade,
        Tip::Feather,
    ];
}

/// One pair of limbs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LimbPair {
    pub role: Role,
    pub build: Build,
    pub tip: Tip,
    /// Length, as a share of the body's length: `0` is a stump, `1` is as
    /// long as the body. Slider.
    pub length: Fx,
    /// Thickness, nought to one. Slider.
    pub girth: Fx,
}

// ---------------------------------------------------------------------------
// Minors
// ---------------------------------------------------------------------------

/// What the jaw is.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Jaw {
    /// A grinding mouth: a grazer's.
    Grinder,
    Teeth,
    Fangs,
    Beak,
    Mandibles,
    /// A mouth big enough to swallow a body: the Sandmaw's.
    Maw,
}

impl Jaw {
    pub const ALL: [Jaw; 6] = [
        Jaw::Grinder,
        Jaw::Teeth,
        Jaw::Fangs,
        Jaw::Beak,
        Jaw::Mandibles,
        Jaw::Maw,
    ];

    /// What it eats. A grinder eats grass; teeth, fangs and a beak eat meat;
    /// mandibles and a maw eat anything.
    pub const fn diet(self) -> Diet {
        match self {
            Jaw::Grinder => Diet::Grazer,
            Jaw::Teeth | Jaw::Fangs | Jaw::Beak => Diet::Carnivore,
            Jaw::Mandibles | Jaw::Maw => Diet::Omnivore,
        }
    }
}

/// What a creature eats, from its jaw.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Diet {
    Grazer,
    Carnivore,
    Omnivore,
}

/// What grows out of the top of the head.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Crown {
    None,
    Horns,
    Crest,
    /// Eyes on stalks, or feelers: a creature that senses more than it sees.
    Feelers,
}

impl Crown {
    pub const ALL: [Crown; 4] = [Crown::None, Crown::Horns, Crown::Crest, Crown::Feelers];
}

/// What the tail ends in.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TailTip {
    Plain,
    Club,
    Blade,
    Sting,
}

impl TailTip {
    pub const ALL: [TailTip; 4] = [
        TailTip::Plain,
        TailTip::Club,
        TailTip::Blade,
        TailTip::Sting,
    ];
}

/// What the skin is.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Hide {
    Skin,
    Fur,
    /// Armour plates, with soft places between: the Ridgeback's.
    Plates,
    /// A shell over the whole back: the Siegeshell's.
    Shell,
    Chitin,
    Feathers,
    /// A wet, soft hide: the Mireback's.
    Slick,
}

impl Hide {
    pub const ALL: [Hide; 7] = [
        Hide::Skin,
        Hide::Fur,
        Hide::Plates,
        Hide::Shell,
        Hide::Chitin,
        Hide::Feathers,
        Hide::Slick,
    ];
}

/// What is carried on the back.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Back {
    Plain,
    /// A ridge of spines along the spine.
    Ridge,
    /// Egg sacs, or a throat sac: something soft that can be torn.
    Sacs,
    /// A load carried: the Siegeshell's siege works.
    Burden,
}

impl Back {
    pub const ALL: [Back; 4] = [Back::Plain, Back::Ridge, Back::Sacs, Back::Burden];
}

/// The minors together.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Minors {
    pub jaw: Jaw,
    pub crown: Crown,
    pub tail_tip: TailTip,
    pub hide: Hide,
    pub back: Back,
}

// ---------------------------------------------------------------------------
// Sliders
// ---------------------------------------------------------------------------

/// The body's magnitudes, each nought to one. What each means in metres
/// and frames is the builder's business ([`build`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Sliders {
    /// Overall size: `0` is a gnawer, `1` the Siegeshell. Heavy inertia.
    pub size: Fx,
    /// Bone density and bulk at a given size: `0` is a bird, `1` is stone.
    /// Heavy inertia. Flight needs it low.
    pub mass: Fx,
    /// How upright the torso is: `0` level, `1` vertical. Heavy inertia.
    pub pitch: Fx,
    /// Top speed for its size.
    pub speed: Fx,
    /// How quickly it reacts: `1` is instant.
    pub reaction: Fx,
    /// How readily it attacks rather than waits.
    pub aggression: Fx,
    /// Its preference for quick moves over slow heavy ones.
    pub tempo: Fx,
    /// Its tendency to guard, parry and wait for an opening.
    pub guard: Fx,
}

/// Which slider, for anything that walks them.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Slider {
    Size,
    Mass,
    Pitch,
    Speed,
    Reaction,
    Aggression,
    Tempo,
    Guard,
}

impl Slider {
    pub const ALL: [Slider; 8] = [
        Slider::Size,
        Slider::Mass,
        Slider::Pitch,
        Slider::Speed,
        Slider::Reaction,
        Slider::Aggression,
        Slider::Tempo,
        Slider::Guard,
    ];

    /// How much of a push reaches it, nought to one: the heavy sliders are
    /// the ones a body cannot change without becoming another body.
    pub const fn inertia(self) -> Fx {
        match self {
            Slider::Size | Slider::Mass | Slider::Pitch => fx(1, 3),
            Slider::Speed | Slider::Reaction => fx(2, 3),
            Slider::Aggression | Slider::Tempo | Slider::Guard => fx(1, 1),
        }
    }
}

impl Sliders {
    pub fn get(&self, s: Slider) -> Fx {
        match s {
            Slider::Size => self.size,
            Slider::Mass => self.mass,
            Slider::Pitch => self.pitch,
            Slider::Speed => self.speed,
            Slider::Reaction => self.reaction,
            Slider::Aggression => self.aggression,
            Slider::Tempo => self.tempo,
            Slider::Guard => self.guard,
        }
    }

    pub fn set(&mut self, s: Slider, v: Fx) {
        let v = unit(v);
        match s {
            Slider::Size => self.size = v,
            Slider::Mass => self.mass = v,
            Slider::Pitch => self.pitch = v,
            Slider::Speed => self.speed = v,
            Slider::Reaction => self.reaction = v,
            Slider::Aggression => self.aggression = v,
            Slider::Tempo => self.tempo = v,
            Slider::Guard => self.guard = v,
        }
    }
}

/// Clamp to nought..one.
pub fn unit(v: Fx) -> Fx {
    v.clamp(Fx::ZERO, Fx::ONE)
}

// ---------------------------------------------------------------------------
// The genome
// ---------------------------------------------------------------------------

/// One creature, as the choices that make it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Genome {
    pub stance: Stance,
    pub head: HeadKind,
    pub segments: Segments,
    pub limbs: [Option<LimbPair>; STATIONS],
    pub minors: Minors,
    pub sliders: Sliders,
}

impl Genome {
    pub fn pair(&self, s: Station) -> Option<&LimbPair> {
        self.limbs[s.index()].as_ref()
    }

    pub fn pairs(&self) -> impl Iterator<Item = (Station, &LimbPair)> {
        Station::ALL
            .iter()
            .filter_map(move |s| self.limbs[s.index()].as_ref().map(|p| (*s, p)))
    }

    pub fn pair_count(&self) -> usize {
        self.limbs.iter().flatten().count()
    }

    pub fn grounded_pairs(&self) -> usize {
        self.limbs
            .iter()
            .flatten()
            .filter(|p| p.role.grounded())
            .count()
    }

    pub fn has_wings(&self) -> bool {
        self.limbs.iter().flatten().any(|p| p.role == Role::Wing)
    }

    pub fn diet(&self) -> Diet {
        self.minors.jaw.diet()
    }

    /// A number that changes with any gene: for a quick "is it the same
    /// creature" and for seeding anything derived from it.
    pub fn fingerprint(&self) -> u64 {
        let mut h = Hasher::new();
        h.word(self.stance as u32);
        h.word(self.head as u32);
        h.word(self.segments.thorax as u32);
        h.word(self.segments.abdomen as u32);
        h.word(self.segments.tail as u32);
        for pair in self.limbs.iter() {
            match pair {
                None => h.word(u32::MAX),
                Some(p) => {
                    h.word(p.role as u32);
                    h.word(p.build as u32);
                    h.word(p.tip as u32);
                    h.word(p.length.raw() as u32);
                    h.word(p.girth.raw() as u32);
                }
            }
        }
        h.word(self.minors.jaw as u32);
        h.word(self.minors.crown as u32);
        h.word(self.minors.tail_tip as u32);
        h.word(self.minors.hide as u32);
        h.word(self.minors.back as u32);
        for s in Slider::ALL {
            h.word(self.sliders.get(s).raw() as u32);
        }
        h.finish()
    }
}

/// FNV-1a over words, the way the snapshot hashes.
struct Hasher(u64);

impl Hasher {
    fn new() -> Hasher {
        Hasher(0xcbf2_9ce4_8422_2325)
    }
    fn word(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100_0000_01b3);
        }
    }
    fn finish(self) -> u64 {
        self.0
    }
}

// ---------------------------------------------------------------------------
// The deterministic die
// ---------------------------------------------------------------------------

/// Splitmix64: a seed in, the same numbers out on every machine. Breeding
/// has to be reproducible -- an egg's outcome is decided by the seed the
/// world gives it and nothing else, so two peers agree on the hatchling.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Rng(pub u64);

impl Rng {
    pub const fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform in nought..one.
    pub fn unit(&mut self) -> Fx {
        Fx::from_raw((self.next_u64() >> 48) as i32)
    }

    /// `true` with this probability.
    pub fn chance(&mut self, p: Fx) -> bool {
        self.unit() < p
    }

    /// One of `n`.
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }

    /// Normally distributed about zero with this standard deviation: the
    /// sum of four uniforms, which is near enough a bell and needs no
    /// transcendental function. The sum of four has a standard deviation
    /// of `sqrt(1/3)`, so it is scaled by `sqrt(3)`.
    pub fn normal(&mut self, sigma: Fx) -> Fx {
        let sum = self
            .unit()
            .add(self.unit())
            .add(self.unit())
            .add(self.unit());
        sum.sub(Fx::from_int(2)).mul(fx(1732, 1000)).mul(sigma)
    }

    /// A share: mostly near zero, sometimes most of the way. The square of
    /// a uniform -- the proportion a crossed limb keeps of the donor.
    pub fn share(&mut self) -> Fx {
        let u = self.unit();
        u.mul(u)
    }
}
