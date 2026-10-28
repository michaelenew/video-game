//! What every major wants of the rest of the body, and the two steps that
//! make a crossed genome cohere: **settle**, which moves the sliders toward
//! what the plan's parts want, and **repair**, which moves the majors to the
//! nearest plan that stands up. Then the **assessment**: can it fly, can it
//! charge, does it hold together, and how plausible an animal is it.
//!
//! The torso's say is the strongest (a body makes no sense without it), a
//! limb pair's is medium (it is how the animal fights), the head's and the
//! minors' are small. A Ridgeback's torso given a Mantis's arms pulls them
//! toward heavy and walking; the arms pull the torso toward upright; the
//! settled result is between, nearer the torso.

use super::{
    Back, Build, Crown, Genome, HeadKind, Hide, Jaw, LimbPair, Role, STATIONS, Slider, Stance,
    Station, Tail, TailTip, fx, unit,
};
use crate::fixed::Fx;

/// One thing a major wants: a slider at a target, this strongly.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Want {
    pub slider: Slider,
    pub target: Fx,
    pub weight: Fx,
}

/// What a major wants of a limb pair: its length and girth.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LimbWant {
    pub length: Fx,
    pub girth: Fx,
    pub weight: Fx,
}

/// The weight of each kind of part's say.
pub const TORSO_SAY: Fx = fx(1, 1);
pub const LIMB_SAY: Fx = fx(1, 2);
pub const HEAD_SAY: Fx = fx(3, 10);
pub const MINOR_SAY: Fx = fx(1, 5);

/// How far a settle moves a slider toward the mean want: half the way,
/// times the slider's inertia.
pub const SETTLE_RATE: Fx = fx(1, 2);
/// A limb's proportions move with the inertia of bone: a sixth of the way
/// a generation, so a limb rebuilt as a wing takes generations to reach a
/// wing's length.
pub const LIMB_SETTLE_RATE: Fx = fx(1, 6);

/// The most wants a genome can raise at once.
pub const MAX_WANTS: usize = 48;

/// A fixed list of wants.
pub struct Wants {
    pub items: [Option<Want>; MAX_WANTS],
    pub count: usize,
}

impl Wants {
    pub fn new() -> Wants {
        Wants {
            items: [None; MAX_WANTS],
            count: 0,
        }
    }

    fn push(&mut self, slider: Slider, target: Fx, weight: Fx) {
        if self.count < MAX_WANTS {
            self.items[self.count] = Some(Want {
                slider,
                target,
                weight,
            });
            self.count += 1;
        }
    }

    /// The weighted mean target for a slider, if anything wants it.
    pub fn mean(&self, slider: Slider) -> Option<Fx> {
        let mut sum = Fx::ZERO;
        let mut total = Fx::ZERO;
        for w in self.items[..self.count].iter().flatten() {
            if w.slider == slider {
                sum = sum.add(w.target.mul(w.weight));
                total = total.add(w.weight);
            }
        }
        if total == Fx::ZERO {
            None
        } else {
            Some(sum.div(total))
        }
    }
}

impl Default for Wants {
    fn default() -> Self {
        Wants::new()
    }
}

// ---------------------------------------------------------------------------
// What each major wants
// ---------------------------------------------------------------------------

impl Stance {
    /// The torso's say on the body's sliders.
    fn wants(self, out: &mut Wants) {
        let w = TORSO_SAY;
        match self {
            Stance::Level => {
                out.push(Slider::Pitch, fx(0, 1), w);
                // Cats and birds are level and light, the Ridgeback level
                // and stone: a level torso has only a weak say on mass.
                out.push(Slider::Mass, fx(1, 2), w.mul(fx(1, 2)));
            }
            Stance::Upright => {
                out.push(Slider::Pitch, fx(4, 5), w);
                out.push(Slider::Mass, fx(3, 10), w);
                out.push(Slider::Tempo, fx(7, 10), w);
            }
            Stance::Lowslung => {
                out.push(Slider::Pitch, fx(0, 1), w);
                out.push(Slider::Mass, fx(7, 10), w);
                out.push(Slider::Speed, fx(3, 10), w);
            }
            Stance::Limbless => {
                out.push(Slider::Pitch, fx(0, 1), w);
                out.push(Slider::Mass, fx(1, 2), w);
            }
        }
    }

    /// The torso's say on a limb pair at a station.
    pub fn limb_want(self, station: Station, role: Role, build: Build) -> Option<LimbWant> {
        let w = TORSO_SAY;
        // A wing's length is the wing's own business: the torso says
        // nothing about it, so a flyer's wings stay as long as flight needs.
        if role == Role::Wing {
            return None;
        }
        match self {
            // Heavy walking pillars, a third of the body long.
            Stance::Level => Some(LimbWant {
                length: fx(7, 20),
                girth: fx(3, 5),
                weight: w,
            }),
            // Fore limbs free and long; the hind pairs carry.
            Stance::Upright => Some(if station == Station::Fore && role != Role::Walk {
                LimbWant {
                    length: fx(1, 2),
                    girth: fx(1, 5),
                    weight: w,
                }
            } else {
                LimbWant {
                    length: fx(3, 10),
                    girth: fx(3, 10),
                    weight: w,
                }
            }),
            // Short and splayed -- unless they are spindles, which reach
            // out and come back down, so the same low belly takes a long
            // leg: a spider's.
            Stance::Lowslung => Some(LimbWant {
                length: if build == Build::Spindle {
                    fx(3, 4)
                } else {
                    fx(1, 5)
                },
                girth: if build == Build::Spindle {
                    fx(1, 10)
                } else {
                    fx(1, 2)
                },
                weight: w,
            }),
            Stance::Limbless => None,
        }
    }

    /// How many grounded pairs it wants at least.
    pub const fn grounded_pairs_wanted(self) -> usize {
        match self {
            Stance::Level | Stance::Lowslung => 2,
            Stance::Upright => 1,
            Stance::Limbless => 0,
        }
    }
}

impl Build {
    /// A limb's say on the body it is on.
    fn wants(self, role: Role, out: &mut Wants) {
        let w = LIMB_SAY;
        match self {
            Build::Column => {
                out.push(Slider::Mass, fx(4, 5), w);
                out.push(Slider::Pitch, fx(0, 1), w);
                out.push(Slider::Tempo, fx(3, 10), w);
            }
            Build::Sprung => {
                out.push(Slider::Mass, fx(2, 5), w);
                out.push(Slider::Speed, fx(4, 5), w);
                out.push(Slider::Tempo, fx(7, 10), w);
            }
            Build::Raptorial => {
                out.push(Slider::Pitch, fx(4, 5), w);
                out.push(Slider::Mass, fx(1, 4), w);
                out.push(Slider::Tempo, fx(4, 5), w);
                out.push(Slider::Guard, fx(7, 10), w);
            }
            Build::Spindle => {
                out.push(Slider::Mass, fx(1, 2), w);
                out.push(Slider::Pitch, fx(0, 1), w);
            }
            Build::Stub => {
                out.push(Slider::Mass, fx(7, 10), w);
                out.push(Slider::Speed, fx(1, 4), w);
                out.push(Slider::Pitch, fx(0, 1), w);
            }
            Build::Wing => {
                out.push(Slider::Mass, fx(1, 5), w);
                out.push(Slider::Speed, fx(3, 5), w);
            }
            // A digger is small and low and solid: a big body on spades
            // strains, and its burrowing goes (`Caps::burrows`).
            Build::Spade => {
                out.push(Slider::Size, fx(3, 10), w);
                out.push(Slider::Mass, fx(3, 5), w);
                out.push(Slider::Pitch, fx(0, 1), w);
            }
        }
        if role == Role::Strike {
            out.push(Slider::Aggression, fx(7, 10), w);
        }
    }

    /// A limb's say on its own proportions.
    pub fn own_want(self) -> LimbWant {
        let w = LIMB_SAY;
        match self {
            Build::Column => LimbWant {
                length: fx(7, 20),
                girth: fx(7, 10),
                weight: w,
            },
            Build::Sprung => LimbWant {
                length: fx(9, 20),
                girth: fx(2, 5),
                weight: w,
            },
            Build::Raptorial => LimbWant {
                length: fx(3, 5),
                girth: fx(3, 20),
                weight: w,
            },
            Build::Spindle => LimbWant {
                length: fx(7, 10),
                girth: fx(1, 10),
                weight: w,
            },
            Build::Stub => LimbWant {
                length: fx(3, 20),
                girth: fx(1, 2),
                weight: w,
            },
            Build::Wing => LimbWant {
                length: fx(1, 1),
                girth: fx(1, 4),
                weight: w,
            },
            Build::Spade => LimbWant {
                length: fx(1, 5),
                girth: fx(3, 5),
                weight: w,
            },
        }
    }

    /// What role the build is for, when it has to carry one.
    pub const fn natural_role(self) -> Role {
        match self {
            Build::Column | Build::Stub | Build::Spindle => Role::Walk,
            Build::Sprung | Build::Spade => Role::Both,
            Build::Raptorial => Role::Strike,
            Build::Wing => Role::Wing,
        }
    }
}

impl HeadKind {
    fn wants(self, out: &mut Wants) {
        let w = HEAD_SAY;
        match self {
            HeadKind::Forward => out.push(Slider::Pitch, fx(0, 1), w),
            HeadKind::Raised => out.push(Slider::Pitch, fx(1, 10), w),
            HeadKind::Low => {
                out.push(Slider::Pitch, fx(0, 1), w);
                out.push(Slider::Mass, fx(7, 10), w);
            }
            HeadKind::Merged => out.push(Slider::Reaction, fx(2, 5), w),
        }
    }
}

impl Hide {
    fn wants(self, out: &mut Wants) {
        let w = MINOR_SAY;
        match self {
            Hide::Feathers => out.push(Slider::Mass, fx(1, 5), w),
            Hide::Plates | Hide::Shell => {
                out.push(Slider::Mass, fx(9, 10), w);
                out.push(Slider::Speed, fx(3, 10), w);
            }
            Hide::Fur => out.push(Slider::Speed, fx(7, 10), w),
            Hide::Slick => out.push(Slider::Speed, fx(1, 5), w),
            Hide::Skin | Hide::Chitin => {}
        }
    }
}

/// Everything the plan wants of the body's sliders.
pub fn wants(g: &Genome) -> Wants {
    let mut out = Wants::new();
    g.stance.wants(&mut out);
    g.head.wants(&mut out);
    g.minors.hide.wants(&mut out);
    for (_, p) in g.pairs() {
        p.build.wants(p.role, &mut out);
    }
    if g.minors.crown == Crown::Horns {
        out.push(Slider::Aggression, fx(3, 5), MINOR_SAY);
    }
    if g.minors.jaw == Jaw::Grinder {
        out.push(Slider::Aggression, fx(1, 4), MINOR_SAY);
    }
    out
}

// ---------------------------------------------------------------------------
// Leg strength
// ---------------------------------------------------------------------------

/// How much leg a body has for its weight, nought up: one is the Ridgeback
/// (thirteen metres of stone on pillars half a metre across), under a half
/// is a body its legs cannot lift.
///
/// Bone carries load by its cross-section and weight grows with volume, so
/// the rule is **area against mass**: the grounded pairs' cross-sections
/// summed, over the body's length cubed times its density, against the
/// Ridgeback's ratio. A creature that grows faster than its legs thicken
/// loses it, which is what the gnawer fed a spider's sacs did.
pub fn leg_strength(g: &Genome) -> Fx {
    let p = super::build::proportions(g);
    let weight = p
        .len
        .mul(p.len)
        .mul(p.len)
        .mul(fx(1, 2).add(g.sliders.mass));
    let mut area = Fx::ZERO;
    for (st, pair) in g.pairs() {
        if pair.role.grounded() {
            let gi = p.girth[st.index()].mul(Fx::from_int(2));
            area = area.add(gi.mul(gi).mul(Fx::from_int(2)));
        }
    }
    if g.stance == Stance::Limbless || area == Fx::ZERO {
        return Fx::ONE;
    }
    // The Ridgeback: area 2.16 over weight 3200.
    area.div(weight.max(fx(1, 100)))
        .mul(fx(3200, 1))
        .div(fx(216, 100))
}

/// Legs this weak cannot lift the body: it rests on its belly.
pub const WEAK_LEGS: Fx = fx(1, 2);

// ---------------------------------------------------------------------------
// Settle
// ---------------------------------------------------------------------------

/// Move every slider part of the way toward what the plan wants of it. The
/// heavy sliders move less (their inertia); a slider nothing wants stays.
pub fn settle(g: &mut Genome) {
    let wants = wants(g);
    for s in Slider::ALL {
        if let Some(target) = wants.mean(s) {
            let v = g.sliders.get(s);
            let step = target.sub(v).mul(SETTLE_RATE).mul(s.inertia());
            g.sliders.set(s, v.add(step));
        }
    }
    let stance = g.stance;
    // Legs want to be as thick as the body is heavy: a weak body's legs
    // thicken over the generations, a light body's slim. The say is the
    // torso's -- the weight is the torso's.
    let strength = leg_strength(g);
    for station in Station::ALL {
        if let Some(p) = g.limbs[station.index()].as_mut() {
            let own = p.build.own_want();
            let (mut len, mut girth, mut total) = (
                own.length.mul(own.weight),
                own.girth.mul(own.weight),
                own.weight,
            );
            if let Some(t) = stance.limb_want(station, p.role, p.build) {
                len = len.add(t.length.mul(t.weight));
                girth = girth.add(t.girth.mul(t.weight));
                total = total.add(t.weight);
            }
            if p.role.grounded() && strength < Fx::ONE {
                // Thicker by the root of what is missing, at the torso's say.
                let want = unit(p.girth.div(strength.max(fx(1, 10)).sqrt()));
                girth = girth.add(want.mul(TORSO_SAY));
                total = total.add(TORSO_SAY);
            }
            let (len, girth) = (len.div(total), girth.div(total));
            p.length = unit(p.length.add(len.sub(p.length).mul(LIMB_SETTLE_RATE)));
            p.girth = unit(p.girth.add(girth.sub(p.girth).mul(LIMB_SETTLE_RATE)));
        }
    }
}

// ---------------------------------------------------------------------------
// Repair
// ---------------------------------------------------------------------------

/// Why a repair changed something. Kept as a list so a test, and later the
/// game, can say what the egg could not be.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fix {
    /// Limbs on a limbless plan: it is low-slung now.
    LimbsOnLimbless,
    /// No limbs on a plan that needs them: it is limbless now.
    NoLimbs,
    /// Nothing to stand on: the hindmost pair walks now.
    NothingToStandOn,
    /// A wing behind the shoulders: it walks now.
    WingAstern,
    /// A wing's build and role must agree.
    WingMismatch,
    /// A raptorial limb does not carry weight alone.
    RaptorialWalks,
    /// An upright body with nothing under its hips.
    UprightUnsupported,
    /// A level body on too few legs.
    LevelUnsupported,
    /// A tail tip with no tail.
    TipWithoutTail,
    /// A maw on a body too small to swallow anything.
    MawTooSmall,
    /// A middle walking pair longer than both its neighbours: no longer
    /// than the longer of them.
    MiddleOutgrown,
    /// A merged head on a body that walks upright.
    MergedUpright,
}

/// The most fixes one repair records.
pub const MAX_FIXES: usize = 8;

/// What a repair did.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Fixes {
    pub items: [Option<Fix>; MAX_FIXES],
    pub count: usize,
}

impl Fixes {
    fn push(&mut self, f: Fix) {
        if self.count < MAX_FIXES {
            self.items[self.count] = Some(f);
            self.count += 1;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Fix> {
        self.items[..self.count].iter().flatten()
    }

    pub fn has(&self, f: Fix) -> bool {
        self.iter().any(|x| *x == f)
    }
}

/// Move the plan to the nearest one that stands up. Every rule changes the
/// least it can, and the rules run until nothing changes (at most a few
/// passes, since each one only narrows).
pub fn repair(g: &mut Genome) -> Fixes {
    let mut fixes = Fixes::default();
    for _ in 0..4 {
        let before = *g;
        pass(g, &mut fixes);
        if *g == before {
            break;
        }
    }
    fixes
}

fn pass(g: &mut Genome, fixes: &mut Fixes) {
    // Limbs and limblessness.
    if g.stance == Stance::Limbless && g.pair_count() > 0 {
        g.stance = Stance::Lowslung;
        fixes.push(Fix::LimbsOnLimbless);
    }
    if g.stance != Stance::Limbless && g.pair_count() == 0 {
        g.stance = Stance::Limbless;
        fixes.push(Fix::NoLimbs);
    }

    // A wing's role and build agree, and a wing sits at the shoulders.
    for station in Station::ALL {
        if let Some(p) = g.limbs[station.index()].as_mut() {
            if p.role == Role::Wing && !matches!(station, Station::Fore | Station::MidFore) {
                p.role = Role::Walk;
                fixes.push(Fix::WingAstern);
            }
            if p.role == Role::Wing && p.build != Build::Wing {
                // A limb that has just become a wing is a stub of one.
                p.build = Build::Wing;
                p.tip = super::Tip::Feather;
                p.length = p.length.mul(fx(1, 2));
                fixes.push(Fix::WingMismatch);
            }
            if p.build == Build::Wing && p.role != Role::Wing {
                p.build = Build::Stub;
                fixes.push(Fix::WingMismatch);
            }
            if p.build == Build::Raptorial && p.role == Role::Walk {
                p.role = Role::Both;
                fixes.push(Fix::RaptorialWalks);
            }
        }
    }

    // Something to stand on.
    if g.stance != Stance::Limbless && g.grounded_pairs() == 0 {
        if let Some(p) = hindmost(g) {
            p.role = match p.role {
                Role::Strike => Role::Both,
                _ => Role::Walk,
            };
            if p.build == Build::Wing {
                p.build = Build::Stub;
            }
            fixes.push(Fix::NothingToStandOn);
        }
    }

    // Enough of it for the stance.
    match g.stance {
        Stance::Upright => {
            let hind_grounded = g.limbs[Station::Hind.index()]
                .or(g.limbs[Station::MidHind.index()])
                .is_some_and(|p| p.role.grounded());
            if !hind_grounded {
                g.stance = if g.grounded_pairs() >= 2 {
                    Stance::Level
                } else {
                    Stance::Lowslung
                };
                fixes.push(Fix::UprightUnsupported);
            }
        }
        Stance::Level => {
            let hind_only = g.grounded_pairs() == 1
                && g.limbs[Station::Hind.index()].is_some_and(|p| p.role.grounded());
            if g.grounded_pairs() < 2 && !hind_only {
                g.stance = Stance::Lowslung;
                fixes.push(Fix::LevelUnsupported);
            }
        }
        Stance::Lowslung | Stance::Limbless => {}
    }

    // **A middle walking pair no longer than the longer of its
    // neighbours** (exploration 0034). The pairs either side stand the
    // body at their height, so a longer one between them crouched deep and
    // stepped its knee out round the next -- a body a walking animal does
    // not have. A spider's legs fan out instead, and are let be.
    let walks = |l: &super::LimbPair| {
        matches!(l.role, Role::Walk | Role::Both) && l.build != Build::Spindle
    };
    let pairs: [Option<super::LimbPair>; STATIONS] = g.limbs;
    for j in 1..STATIONS - 1 {
        let Some(mid) = pairs[j].filter(walks) else {
            continue;
        };
        let before = (0..j).rev().find_map(|k| pairs[k]).filter(walks);
        let after = (j + 1..STATIONS).find_map(|k| pairs[k]).filter(walks);
        if let (Some(a), Some(b)) = (before, after) {
            let most = a.length.max(b.length);
            if mid.length > most {
                if let Some(l) = g.limbs[j].as_mut() {
                    l.length = most;
                }
                fixes.push(Fix::MiddleOutgrown);
            }
        }
    }

    // The minors fit the plan.
    if g.segments.tail == Tail::None && g.minors.tail_tip != TailTip::Plain {
        g.minors.tail_tip = TailTip::Plain;
        fixes.push(Fix::TipWithoutTail);
    }
    if g.minors.jaw == Jaw::Maw && g.sliders.size < fx(1, 2) {
        g.minors.jaw = Jaw::Teeth;
        fixes.push(Fix::MawTooSmall);
    }
    if g.head == HeadKind::Merged && g.stance == Stance::Upright {
        g.head = HeadKind::Forward;
        fixes.push(Fix::MergedUpright);
    }
}

fn hindmost(g: &mut Genome) -> Option<&mut LimbPair> {
    let mut found = None;
    for i in (0..STATIONS).rev() {
        if g.limbs[i].is_some() {
            found = Some(i);
            break;
        }
    }
    found.and_then(move |i| g.limbs[i].as_mut())
}

// ---------------------------------------------------------------------------
// Assessment
// ---------------------------------------------------------------------------

/// What a body can do, from its plan.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Caps {
    pub flies: bool,
    /// Wings it has that cannot lift it.
    pub vestigial_wings: bool,
    pub charges: bool,
    pub pounces: bool,
    pub guards: bool,
    pub burrows: bool,
    /// Something a rider can stand on: a big enough level back.
    pub mountable: bool,
    pub swallows: bool,
    /// Its legs cannot lift it ([`leg_strength`] under a half): it rests
    /// on its belly and nothing it does is quick.
    pub weak_legs: bool,
}

/// The verdict on a genome.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Assessment {
    pub caps: Caps,
    /// Nought to one: how far the sliders are from what the plan wants, and
    /// how many of the plan's parts strain against each other. One is an
    /// animal at ease in its body.
    pub plausibility: Fx,
    /// Its danger, one to five, the way the bestiary tiers it.
    pub tier: u8,
}

/// A legged body that burrows: it stands on a pair of spades at its front
/// half, it is small enough to go under (size to three fifths), it is not
/// upright, and its legs can lift it. Too big a body on spades keeps them
/// and loses the burrowing -- the plan's push back.
pub fn digs(g: &Genome, weak: bool) -> bool {
    g.stance != Stance::Upright
        && g.stance != Stance::Limbless
        && !weak
        && g.sliders.size <= fx(3, 5)
        && g.pairs().any(|(st, p)| {
            p.build == Build::Spade
                && p.role.grounded()
                && matches!(st, super::Station::Fore | super::Station::MidFore)
        })
}

/// Lift against weight, as a margin: above zero it flies. The wing's
/// length (as a share of the body) is lift; mass and size are weight.
pub fn flight_margin(g: &Genome) -> Fx {
    let Some((_, wing)) = g.pairs().find(|(_, p)| p.role == Role::Wing) else {
        return Fx::from_int(-1);
    };
    // Armour is weight a wing has to lift: a plated beast that has just
    // grown wings does not take off on them (exploration 0033).
    let armour = match (g.minors.hide, g.minors.back) {
        (Hide::Plates | Hide::Shell, _) | (_, Back::Burden) => fx(2, 5),
        _ => Fx::ZERO,
    };
    wing.length
        .mul(fx(3, 2))
        .add(Fx::ONE.sub(g.sliders.mass).mul(fx(4, 5)))
        .sub(g.sliders.size.mul(fx(3, 5)))
        .sub(fx(9, 10))
        .sub(armour)
}

pub fn assess(g: &Genome) -> Assessment {
    let s = &g.sliders;
    let margin = flight_margin(g);
    let strength = leg_strength(g);
    let weak = strength < WEAK_LEGS;
    let caps = Caps {
        flies: g.has_wings() && margin > Fx::ZERO,
        vestigial_wings: g.has_wings() && margin <= Fx::ZERO,
        charges: g.grounded_pairs() >= 2
            && g.stance == Stance::Level
            && s.speed >= fx(2, 5)
            && s.size >= fx(1, 4)
            && !weak,
        pounces: g
            .pairs()
            .any(|(_, p)| p.build == Build::Sprung && p.role.grounded())
            && s.mass < fx(7, 10)
            && !weak,
        weak_legs: weak,
        guards: s.guard >= fx(1, 2)
            || g.pairs()
                .any(|(_, p)| p.build == Build::Raptorial && p.role != Role::Walk),
        burrows: g.stance == Stance::Limbless || digs(g, weak),
        mountable: s.size >= fx(2, 5) && g.stance != Stance::Upright,
        swallows: g.minors.jaw == Jaw::Maw && s.size >= fx(1, 2),
    };

    // Plausibility: one, less a penalty for every slider off what the plan
    // wants, and for the strains the plan carries.
    let wants = wants(g);
    let mut p = Fx::ONE;
    for sl in Slider::ALL {
        if let Some(t) = wants.mean(sl) {
            p = p.sub(t.sub(s.get(sl)).abs().mul(fx(1, 4)));
        }
    }
    for (station, pair) in g.pairs() {
        let own = pair.build.own_want();
        let mut want_len = own.length;
        if let Some(t) = g.stance.limb_want(station, pair.role, pair.build) {
            want_len = own
                .length
                .mul(own.weight)
                .add(t.length.mul(t.weight))
                .div(own.weight.add(t.weight));
        }
        p = p.sub(want_len.sub(pair.length).abs().mul(fx(1, 5)));
    }
    if caps.vestigial_wings {
        p = p.sub(fx(3, 20));
    }
    if strength < Fx::ONE {
        p = p.sub(Fx::ONE.sub(strength).mul(fx(3, 10)));
    }
    if g.stance == Stance::Upright && s.mass > fx(7, 10) {
        p = p.sub(fx(1, 5));
    }
    if g.grounded_pairs() < g.stance.grounded_pairs_wanted() {
        p = p.sub(fx(1, 5));
    }
    if g.minors.back == Back::Burden && s.size < fx(3, 5) {
        p = p.sub(fx(1, 10));
    }
    let plausibility = unit(p);

    // Tier: size mostly, growing with its square; then how quickly it
    // reads you and how well it guards, which is what makes a duellist of
    // a four-metre insect; a step for flight and for a maw.
    let mut t = s
        .size
        .mul(s.size)
        .mul(fx(9, 2))
        .add(s.reaction)
        .add(s.guard.mul(fx(3, 2)))
        .add(s.tempo.mul(fx(3, 10)))
        .add(s.aggression.mul(fx(2, 5)))
        .add(fx(1, 2));
    if caps.flies {
        t = t.add(fx(1, 2));
    }
    if caps.swallows {
        t = t.add(fx(1, 2));
    }
    let tier = t.to_int().clamp(1, 5) as u8;

    Assessment {
        caps,
        plausibility,
        tier,
    }
}
