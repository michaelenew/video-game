//! Crossing genomes: breeding two, grafting a trophy onto an egg, and
//! mutation. Each ends with the plan settled and repaired (`plan`), so what
//! comes out is always a creature that stands up.
//!
//! **Breeding** two parents: every major from one parent or the other, a
//! limb pair present in only one inherited half the time, every slider
//! somewhere between the two, then a mutation. Two of the same species
//! breed true to within the mutation; the biggest two breed bigger.
//!
//! **Grafting** a trophy onto an egg: the trophy carries its donor's genome
//! and a *pull* per channel -- a feather pulls hard on the limbs and the
//! hide and a little on everything else, a horn on the head. Each slider
//! moves a random share of the way toward the donor, scaled by the pull
//! and its own inertia; each minor flips to the donor's with a chance
//! scaled by the pull; each major with a far smaller one. A limb pair the
//! egg already has at the donor's station is **blended by a proportion**
//! rolled on the spot, mostly small -- ten percent Mantis, ninety Ridgeback
//! -- its role and build going over only when the proportion passes a half.
//! A pair the egg does not have is added rarely; nothing a trophy does
//! removes one.
//!
//! **Mutation** is a normally distributed nudge on every slider, scaled by
//! the slider's inertia, and a rare flip of a minor; rarer still of a major.

use super::plan::{Fixes, repair, settle};
use super::{
    Back, Build, Crown, Genome, HeadKind, Hide, Jaw, LimbPair, Rng, Role, STATIONS, Slider, Stance,
    Station, Tail, TailTip, Tip, fx, unit,
};
use crate::fixed::Fx;

/// How often things move. **A tuning table**: these will be Oven knobs once
/// an egg is in the game and there is a feel to tune; until then they are
/// judged by `tests/breed.rs` running generations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rates {
    /// Chance a trophy at full pull flips a major.
    pub graft_major: Fx,
    /// Chance a trophy at full pull flips a minor.
    pub graft_minor: Fx,
    /// Chance a trophy at full pull adds a limb pair the egg lacks.
    pub graft_new_limb: Fx,
    /// Standard deviation of a slider's mutation, before inertia.
    pub mutation_sigma: Fx,
    /// Chance a minor mutates at random.
    pub mutation_minor: Fx,
    /// Chance a major mutates at random.
    pub mutation_major: Fx,
    /// How much harder a trophy pulls the sliders than the slow rate: one
    /// for a lineage, more for a morph that has to change while you watch.
    pub slider_gain: Fx,
}

impl Rates {
    pub const DEFAULT: Rates = Rates {
        graft_major: fx(1, 8),
        graft_minor: fx(2, 5),
        graft_new_limb: fx(1, 16),
        mutation_sigma: fx(3, 100),
        mutation_minor: fx(1, 200),
        mutation_major: fx(1, 400),
        slider_gain: Fx::ONE,
    };
}

impl Default for Rates {
    fn default() -> Self {
        Rates::DEFAULT
    }
}

// ---------------------------------------------------------------------------
// Trophies
// ---------------------------------------------------------------------------

/// What a creature drops, and what it carries of the creature.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TrophyKind {
    Feather,
    Horn,
    Fang,
    Beak,
    Claw,
    Hoof,
    Talon,
    Plate,
    Shell,
    Pelt,
    Chitin,
    Quill,
    TailTip,
    Sac,
    Tooth,
}

/// How hard a trophy pulls on each channel of the egg, nought to one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pull {
    /// The stance, the segments.
    pub plan: Fx,
    /// Limb pairs: their role, build, tip and sliders.
    pub limbs: Fx,
    /// The head's placement, jaw and crown.
    pub head: Fx,
    /// The tail and its tip.
    pub tail: Fx,
    /// Hide and back.
    pub hide: Fx,
    /// Size and mass.
    pub bulk: Fx,
    /// Speed, reaction, aggression, tempo, guard.
    pub temper: Fx,
}

impl TrophyKind {
    pub const ALL: [TrophyKind; 15] = [
        TrophyKind::Feather,
        TrophyKind::Horn,
        TrophyKind::Fang,
        TrophyKind::Beak,
        TrophyKind::Claw,
        TrophyKind::Hoof,
        TrophyKind::Talon,
        TrophyKind::Plate,
        TrophyKind::Shell,
        TrophyKind::Pelt,
        TrophyKind::Chitin,
        TrophyKind::Quill,
        TrophyKind::TailTip,
        TrophyKind::Sac,
        TrophyKind::Tooth,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            TrophyKind::Feather => "feather",
            TrophyKind::Horn => "horn",
            TrophyKind::Fang => "fang",
            TrophyKind::Beak => "beak",
            TrophyKind::Claw => "claw",
            TrophyKind::Hoof => "hoof",
            TrophyKind::Talon => "talon",
            TrophyKind::Plate => "plate",
            TrophyKind::Shell => "shell",
            TrophyKind::Pelt => "pelt",
            TrophyKind::Chitin => "chitin",
            TrophyKind::Quill => "quill",
            TrophyKind::TailTip => "tail tip",
            TrophyKind::Sac => "sac",
            TrophyKind::Tooth => "tooth",
        }
    }

    pub const fn pull(self) -> Pull {
        let (plan, limbs, head, tail, hide, bulk, temper) = match self {
            TrophyKind::Feather => (
                fx(3, 10),
                fx(1, 1),
                fx(1, 10),
                fx(1, 10),
                fx(1, 2),
                fx(3, 10),
                fx(1, 5),
            ),
            TrophyKind::Horn => (
                fx(1, 5),
                fx(1, 10),
                fx(1, 1),
                fx(0, 1),
                fx(1, 10),
                fx(1, 2),
                fx(1, 2),
            ),
            TrophyKind::Fang | TrophyKind::Beak | TrophyKind::Tooth => (
                fx(1, 10),
                fx(1, 10),
                fx(1, 1),
                fx(0, 1),
                fx(0, 1),
                fx(1, 5),
                fx(3, 5),
            ),
            TrophyKind::Claw | TrophyKind::Hoof | TrophyKind::Talon => (
                fx(1, 5),
                fx(4, 5),
                fx(1, 10),
                fx(0, 1),
                fx(1, 10),
                fx(1, 5),
                fx(3, 10),
            ),
            TrophyKind::Plate | TrophyKind::Shell | TrophyKind::Pelt | TrophyKind::Chitin => (
                fx(3, 10),
                fx(1, 10),
                fx(0, 1),
                fx(1, 10),
                fx(1, 1),
                fx(3, 5),
                fx(1, 5),
            ),
            TrophyKind::Quill | TrophyKind::TailTip => (
                fx(1, 5),
                fx(1, 10),
                fx(0, 1),
                fx(1, 1),
                fx(1, 2),
                fx(1, 5),
                fx(1, 5),
            ),
            TrophyKind::Sac => (
                fx(1, 2),
                fx(1, 5),
                fx(1, 10),
                fx(0, 1),
                fx(3, 5),
                fx(2, 5),
                fx(1, 5),
            ),
        };
        Pull {
            plan,
            limbs,
            head,
            tail,
            hide,
            bulk,
            temper,
        }
    }
}

/// A trophy: what it is, and the creature it was won from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Trophy {
    pub kind: TrophyKind,
    pub donor: Genome,
}

// ---------------------------------------------------------------------------
// The outcome of a crossing
// ---------------------------------------------------------------------------

/// What came out, and what the repair had to change to make it stand.
#[derive(Clone, Copy, Debug)]
pub struct Outcome {
    pub genome: Genome,
    pub fixes: Fixes,
    /// Which majors moved in this crossing.
    pub moved: Moved,
}

/// Which majors a crossing moved.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Moved {
    pub stance: bool,
    pub head: bool,
    pub segments: bool,
    pub limbs: [bool; STATIONS],
}

impl Moved {
    pub fn any(&self) -> bool {
        self.stance || self.head || self.segments || self.limbs.iter().any(|b| *b)
    }
}

/// Settle, repair, done.
fn finish(mut g: Genome, moved: Moved) -> Outcome {
    settle(&mut g);
    let fixes = repair(&mut g);
    // A repaired plan wants different things of its sliders: one more
    // settle toward them, and only then.
    if fixes.count > 0 {
        settle(&mut g);
    }
    Outcome {
        genome: g,
        fixes,
        moved,
    }
}

// ---------------------------------------------------------------------------
// Breeding
// ---------------------------------------------------------------------------

/// Two parents, a child.
pub fn breed(a: &Genome, b: &Genome, rates: &Rates, rng: &mut Rng) -> Outcome {
    let mut g = *a;
    let mut moved = Moved::default();
    if rng.chance(fx(1, 2)) && b.stance != a.stance {
        g.stance = b.stance;
        moved.stance = true;
    }
    if rng.chance(fx(1, 2)) && b.head != a.head {
        g.head = b.head;
        moved.head = true;
    }
    if rng.chance(fx(1, 2)) && b.segments != a.segments {
        g.segments = b.segments;
        moved.segments = true;
    }
    for i in 0..STATIONS {
        g.limbs[i] = match (a.limbs[i], b.limbs[i]) {
            (Some(x), Some(y)) => {
                let share = rng.unit();
                let blended = blend_pair(&x, &y, share, share > fx(1, 2));
                moved.limbs[i] = blended.role != x.role || blended.build != x.build;
                Some(blended)
            }
            (Some(x), None) => {
                if rng.chance(fx(1, 2)) {
                    Some(x)
                } else {
                    moved.limbs[i] = true;
                    None
                }
            }
            (None, Some(y)) => {
                if rng.chance(fx(1, 2)) {
                    moved.limbs[i] = true;
                    Some(y)
                } else {
                    None
                }
            }
            (None, None) => None,
        };
    }
    g.minors.jaw = if rng.chance(fx(1, 2)) {
        a.minors.jaw
    } else {
        b.minors.jaw
    };
    g.minors.crown = if rng.chance(fx(1, 2)) {
        a.minors.crown
    } else {
        b.minors.crown
    };
    g.minors.tail_tip = if rng.chance(fx(1, 2)) {
        a.minors.tail_tip
    } else {
        b.minors.tail_tip
    };
    g.minors.hide = if rng.chance(fx(1, 2)) {
        a.minors.hide
    } else {
        b.minors.hide
    };
    g.minors.back = if rng.chance(fx(1, 2)) {
        a.minors.back
    } else {
        b.minors.back
    };
    for s in Slider::ALL {
        let t = rng.unit();
        let v = a
            .sliders
            .get(s)
            .add(b.sliders.get(s).sub(a.sliders.get(s)).mul(t));
        g.sliders.set(s, v);
    }
    mutate_in_place(&mut g, rates, rng, &mut moved);
    finish(g, moved)
}

/// A pair that is `share` of `b` and the rest `a`: sliders between, the
/// discrete parts from `b` if `from_b`.
fn blend_pair(a: &LimbPair, b: &LimbPair, share: Fx, from_b: bool) -> LimbPair {
    let mut length = unit(a.length.add(b.length.sub(a.length).mul(share)));
    // A limb that becomes a wing comes in short -- a wing that was a leg
    // is a stub of a wing first, and has to grow before it lifts. A leg
    // rebuilt as another kind of leg keeps its length: it still has to
    // carry the body.
    if from_b && b.role == Role::Wing && a.role != Role::Wing {
        length = length.mul(fx(1, 2));
    }
    LimbPair {
        role: if from_b { b.role } else { a.role },
        build: if from_b { b.build } else { a.build },
        tip: if from_b { b.tip } else { a.tip },
        length,
        girth: unit(a.girth.add(b.girth.sub(a.girth).mul(share))),
    }
}

// ---------------------------------------------------------------------------
// Grafting
// ---------------------------------------------------------------------------

/// A trophy given to an egg: the egg, pulled toward the donor.
pub fn graft(egg: &Genome, trophy: &Trophy, rates: &Rates, rng: &mut Rng) -> Outcome {
    let d = &trophy.donor;
    let pull = trophy.kind.pull();
    let mut g = *egg;
    let mut moved = Moved::default();

    // Majors: rarely.
    if d.stance != g.stance && rng.chance(rates.graft_major.mul(pull.plan)) {
        g.stance = d.stance;
        moved.stance = true;
    }
    // Burrowing, from a burrower to a body that keeps its legs: its front
    // walking pair becomes spades, as rarely as a major moves, pulled by
    // the trophy's say on the limbs and on the plan together. (A body can
    // still become limbless -- the worm's way -- by its stance, above.)
    if g.stance != Stance::Limbless
        && super::plan::assess(d).caps.burrows
        && !super::plan::assess(&g).caps.burrows
        && rng.chance(rates.graft_major.mul(pull.limbs.add(pull.plan)))
        && let Some(i) = (0..STATIONS).find(|&i| g.limbs[i].is_some_and(|p| p.role.grounded()))
        && let Some(p) = g.limbs[i].as_mut()
    {
        p.build = Build::Spade;
        p.tip = Tip::Claw;
        moved.limbs[i] = true;
    }
    if d.head != g.head && rng.chance(rates.graft_major.mul(pull.head)) {
        g.head = d.head;
        moved.head = true;
    }
    if d.segments.thorax != g.segments.thorax && rng.chance(rates.graft_major.mul(pull.plan)) {
        g.segments.thorax = d.segments.thorax;
        moved.segments = true;
    }
    if d.segments.abdomen != g.segments.abdomen && rng.chance(rates.graft_major.mul(pull.plan)) {
        g.segments.abdomen = d.segments.abdomen;
        moved.segments = true;
    }
    if d.segments.tail != g.segments.tail && rng.chance(rates.graft_major.mul(pull.tail)) {
        g.segments.tail = d.segments.tail;
        moved.segments = true;
    }

    // Limbs: a matched pair blends by a rolled proportion; an unmatched one
    // is added rarely, and never removed.
    for i in 0..STATIONS {
        match (g.limbs[i], d.limbs[i]) {
            (Some(mine), Some(theirs)) => {
                // The proportion moves the limb's sliders; what the limb
                // *is* goes over as rarely as any other major.
                // ... and a limb of another kind takes a quarter of the
                // pull: a feather does little to a leg's proportions until
                // the leg is a wing.
                // A limb's proportions are bone, and move with the heavy
                // sliders' inertia.
                let mut share = rng.share().mul(pull.limbs).mul(Slider::Size.inertia());
                if mine.build != theirs.build {
                    share = share.mul(fx(1, 4));
                }
                let flip = rng.chance(rates.graft_major.mul(pull.limbs));
                let blended = blend_pair(&mine, &theirs, share, flip);
                moved.limbs[i] = blended.role != mine.role || blended.build != mine.build;
                g.limbs[i] = Some(blended);
            }
            (None, Some(theirs)) if rng.chance(rates.graft_new_limb.mul(pull.limbs)) => {
                // It arrives small, a new limb budding.
                g.limbs[i] = Some(LimbPair {
                    length: theirs.length.mul(fx(1, 2)),
                    girth: theirs.girth.mul(fx(1, 2)),
                    ..theirs
                });
                moved.limbs[i] = true;
            }
            _ => {}
        }
    }

    // Minors: more readily.
    if d.minors.jaw != g.minors.jaw && rng.chance(rates.graft_minor.mul(pull.head)) {
        g.minors.jaw = d.minors.jaw;
    }
    if d.minors.crown != g.minors.crown && rng.chance(rates.graft_minor.mul(pull.head)) {
        g.minors.crown = d.minors.crown;
    }
    if d.minors.tail_tip != g.minors.tail_tip && rng.chance(rates.graft_minor.mul(pull.tail)) {
        g.minors.tail_tip = d.minors.tail_tip;
    }
    if d.minors.hide != g.minors.hide && rng.chance(rates.graft_minor.mul(pull.hide)) {
        g.minors.hide = d.minors.hide;
    }
    if d.minors.back != g.minors.back && rng.chance(rates.graft_minor.mul(pull.hide)) {
        g.minors.back = d.minors.back;
    }
    for i in 0..STATIONS {
        if let (Some(mine), Some(theirs)) = (g.limbs[i].as_mut(), d.limbs[i].as_ref()) {
            if mine.tip != theirs.tip && rng.chance(rates.graft_minor.mul(pull.limbs)) {
                mine.tip = theirs.tip;
            }
        }
    }

    // Sliders: a random share of the way, by the channel's pull and the
    // slider's inertia.
    for s in Slider::ALL {
        let channel = match s {
            Slider::Size | Slider::Mass => pull.bulk,
            Slider::Pitch => pull.plan,
            _ => pull.temper,
        };
        let share = rng
            .share()
            .mul(channel)
            .mul(s.inertia())
            .mul(rates.slider_gain)
            .min(Fx::ONE);
        let v = g.sliders.get(s);
        g.sliders.set(s, v.add(d.sliders.get(s).sub(v).mul(share)));
    }

    mutate_in_place(&mut g, rates, rng, &mut moved);
    finish(g, moved)
}

// ---------------------------------------------------------------------------
// Feeding a morph
// ---------------------------------------------------------------------------

/// A trophy fed to a morph: **one step, the same step every time.**
///
/// A lineage's graft (above) rolls dice on every part, which is right for
/// generations nobody watches and wrong for a morph somebody is feeding:
/// fed the same thing twice it grew and then shrank, and its body plan
/// wandered. So a feeding is a walk toward the donor in a fixed order:
///
/// - **From where it stands.** `body` is the morph's body now, after
///   every feeding before this one; nothing else is remembered, and there
///   is no shape it is pulled back toward (`stand`).
/// - **Its proportions** move a fixed share of the way -- the trophy's
///   pull on the channel, the slider's inertia, the morph's gain -- never a
///   rolled one, so they only ever close on the donor's.
/// - **One part** of the donor goes over: of the parts they differ in,
///   the one the trophy pulls hardest on (a feather the limbs, a horn the
///   head, a plate the hide), front to back, ties to the plan first. The
///   next feeding of the same takes the next. A part that the repair would
///   undo is passed over for the next.
/// - **No mutation**: a morph changes by what it eats, not by chance.
///
/// So a handful of one creature's trophies walk it, a visible step at a
/// time, toward that creature, in an order a player can learn.
pub fn feed(body: &Genome, trophy: &Trophy, _rng: &mut Rng) -> Outcome {
    let d = &trophy.donor;
    let pull = trophy.kind.pull();
    let gain = crate::tuning::morph_slider_gain();
    let mut g = *body;
    for s in Slider::ALL {
        let channel = match s {
            Slider::Size | Slider::Mass => pull.bulk,
            Slider::Pitch => pull.plan,
            _ => pull.temper,
        };
        let share = channel.mul(s.inertia()).mul(gain).min(Fx::ONE);
        let v = g.sliders.get(s);
        g.sliders.set(s, v.add(d.sliders.get(s).sub(v).mul(share)));
    }
    for i in 0..STATIONS {
        if let (Some(mine), Some(theirs)) = (g.limbs[i].as_mut(), d.limbs[i])
            && mine.build == theirs.build
        {
            let share = pull.limbs.mul(fx(1, 2));
            mine.length = unit(mine.length.add(theirs.length.sub(mine.length).mul(share)));
            mine.girth = unit(mine.girth.add(theirs.girth.sub(mine.girth).mul(share)));
        }
    }
    let moved = Moved::default();
    let mut tried = 0u32;
    for _ in 0..16 {
        let mut next = g;
        let mut m = moved;
        let Some(which) = step_toward(&mut next, d, &pull, &mut m, tried) else {
            break;
        };
        tried |= 1 << which;
        let out = stand(next, m);
        if visible(body, &out.genome) {
            return out;
        }
    }
    stand(g, moved)
}

/// A fed body made to stand, and nothing else. A lineage's crossing also
/// settles its sliders toward what its plan wants (`plan::settle`), and a
/// morph's must not: those wants are a table of what each stance and
/// build "should" be, a fixed point every feeding would be pulled back
/// toward. A morph has no such point. Every body is as natural as any
/// other, and a trophy moves the one it is fed to from where it stands
/// -- so only the repair, which makes it a body at all.
fn stand(mut g: Genome, moved: Moved) -> Outcome {
    let fixes = repair(&mut g);
    Outcome {
        genome: g,
        fixes,
        moved,
    }
}

/// Whether a player would see that `a` became `b`: anything the morph's
/// account of a change ([`super::morph::changes`]) would say.
pub fn visible(a: &Genome, b: &Genome) -> bool {
    !super::morph::changes(a, b, None).is_empty()
}

/// One part of the donor, onto the body: of the parts they differ in, the
/// one the trophy pulls hardest on -- the first offered, on a tie --
/// skipping the ones in `skip` (a bit each). Which it moved, or none if
/// nothing differs.
fn step_toward(
    g: &mut Genome,
    d: &Genome,
    pull: &Pull,
    moved: &mut Moved,
    skip: u32,
) -> Option<u8> {
    // (weight, which) for every difference.
    let mut options = [(Fx::ZERO, 0u8); 16];
    let mut n = 0;
    // Every difference is on offer, the trophy's own channels first: a
    // trophy carries a little of everything its donor is.
    let mut offer = |w: Fx, which: u8| {
        let w = w.max(fx(1, 20));
        if n < options.len() && skip & (1 << which) == 0 {
            options[n] = (w, which);
            n += 1;
        }
    };
    if g.stance != d.stance {
        offer(pull.plan, 0);
    }
    if g.head != d.head {
        offer(pull.head, 1);
    }
    if g.segments.tail != d.segments.tail {
        offer(pull.tail, 2);
    }
    if g.segments.thorax != d.segments.thorax || g.segments.abdomen != d.segments.abdomen {
        offer(pull.plan, 3);
    }
    if g.minors.jaw != d.minors.jaw || g.minors.crown != d.minors.crown {
        offer(pull.head, 4);
    }
    if g.minors.tail_tip != d.minors.tail_tip {
        offer(pull.tail, 5);
    }
    if g.minors.hide != d.minors.hide || g.minors.back != d.minors.back {
        offer(pull.hide, 6);
    }
    for i in 0..STATIONS {
        let differs = match (g.limbs[i], d.limbs[i]) {
            (Some(x), Some(y)) => x.role != y.role || x.build != y.build || x.tip != y.tip,
            (None, Some(_)) => true,
            // A pair the donor has nothing at goes, if it is not the last
            // the body stands on.
            (Some(_), None) => g.pair_count() > 1,
            _ => false,
        };
        if differs {
            offer(pull.limbs, 7 + i as u8);
        }
    }
    // Burrowing, from a burrower to a body that keeps its legs: its front
    // walking pair turns to spades (a body can still go limbless -- the
    // worm's way -- by its stance).
    if g.stance != Stance::Limbless
        && super::plan::assess(d).caps.burrows
        && !super::plan::assess(g).caps.burrows
        && g.limbs.iter().flatten().any(|p| p.role.grounded())
    {
        offer(pull.limbs.add(pull.plan), 12);
    }
    // Size from a twentieth off: the feeding's own pull has already closed
    // some of the gap, and what is left may still be a tenth from the egg.
    if g.sliders.size.sub(d.sliders.size).abs() >= fx(1, 20) {
        offer(pull.bulk, 11);
    }
    let mut which = None;
    let mut best = Fx::ZERO;
    for (w, k) in &options[..n] {
        if *w > best {
            best = *w;
            which = Some(*k);
        }
    }
    let which = which?;
    match which {
        0 => {
            g.stance = d.stance;
            moved.stance = true;
        }
        1 => {
            g.head = d.head;
            moved.head = true;
        }
        2 => {
            g.segments.tail = d.segments.tail;
            moved.segments = true;
        }
        3 => {
            g.segments.thorax = d.segments.thorax;
            g.segments.abdomen = d.segments.abdomen;
            moved.segments = true;
        }
        4 => {
            g.minors.jaw = d.minors.jaw;
            g.minors.crown = d.minors.crown;
        }
        5 => g.minors.tail_tip = d.minors.tail_tip,
        6 => {
            g.minors.hide = d.minors.hide;
            g.minors.back = d.minors.back;
        }
        // Toward the donor's size: half the way, all of it when close, so
        // a feeding of the same never overshoots and comes back.
        11 => {
            let gap = d.sliders.size.sub(g.sliders.size);
            g.sliders.size = if gap.abs() < fx(1, 5) {
                d.sliders.size
            } else {
                g.sliders.size.add(gap.mul(fx(1, 2)))
            };
        }
        12 => {
            if let Some(p) = g.limbs.iter_mut().flatten().find(|p| p.role.grounded()) {
                p.build = Build::Spade;
                p.tip = Tip::Claw;
            }
        }
        k => {
            let i = (k - 7) as usize;
            match (g.limbs[i].as_mut(), d.limbs[i]) {
                (Some(mine), Some(theirs)) => {
                    mine.role = theirs.role;
                    mine.build = theirs.build;
                    mine.tip = theirs.tip;
                }
                (None, Some(theirs)) => {
                    g.limbs[i] = Some(LimbPair {
                        length: theirs.length.mul(fx(1, 2)),
                        girth: theirs.girth.mul(fx(1, 2)),
                        ..theirs
                    });
                }
                (Some(_), None) => g.limbs[i] = None,
                _ => {}
            }
            moved.limbs[i] = true;
        }
    }
    Some(which)
}

// ---------------------------------------------------------------------------
// Mutation
// ---------------------------------------------------------------------------

/// A generation's drift with nobody pushing: the mutation alone, settled
/// and repaired.
pub fn mutate(g: &Genome, rates: &Rates, rng: &mut Rng) -> Outcome {
    let mut out = *g;
    let mut moved = Moved::default();
    mutate_in_place(&mut out, rates, rng, &mut moved);
    finish(out, moved)
}

fn mutate_in_place(g: &mut Genome, rates: &Rates, rng: &mut Rng, moved: &mut Moved) {
    for s in Slider::ALL {
        let v = g.sliders.get(s);
        g.sliders
            .set(s, v.add(rng.normal(rates.mutation_sigma.mul(s.inertia()))));
    }
    for p in g.limbs.iter_mut().flatten() {
        p.length = unit(p.length.add(rng.normal(rates.mutation_sigma)));
        p.girth = unit(p.girth.add(rng.normal(rates.mutation_sigma)));
    }
    // Minors.
    if rng.chance(rates.mutation_minor) {
        g.minors.jaw = Jaw::ALL[rng.below(Jaw::ALL.len())];
    }
    if rng.chance(rates.mutation_minor) {
        g.minors.crown = Crown::ALL[rng.below(Crown::ALL.len())];
    }
    if rng.chance(rates.mutation_minor) {
        g.minors.tail_tip = TailTip::ALL[rng.below(TailTip::ALL.len())];
    }
    if rng.chance(rates.mutation_minor) {
        g.minors.hide = Hide::ALL[rng.below(Hide::ALL.len())];
    }
    if rng.chance(rates.mutation_minor) {
        g.minors.back = Back::ALL[rng.below(Back::ALL.len())];
    }
    for p in g.limbs.iter_mut().flatten() {
        if rng.chance(rates.mutation_minor) {
            p.tip = Tip::ALL[rng.below(Tip::ALL.len())];
        }
    }
    // Majors.
    if rng.chance(rates.mutation_major) {
        g.stance = Stance::ALL[rng.below(Stance::ALL.len())];
        moved.stance = true;
    }
    if rng.chance(rates.mutation_major) {
        g.head = HeadKind::ALL[rng.below(HeadKind::ALL.len())];
        moved.head = true;
    }
    if rng.chance(rates.mutation_major) {
        g.segments.tail = Tail::ALL[rng.below(Tail::ALL.len())];
        moved.segments = true;
    }
    if rng.chance(rates.mutation_major) {
        g.segments.thorax = !g.segments.thorax;
        moved.segments = true;
    }
    if rng.chance(rates.mutation_major) {
        g.segments.abdomen = !g.segments.abdomen;
        moved.segments = true;
    }
    for i in 0..STATIONS {
        if rng.chance(rates.mutation_major) {
            moved.limbs[i] = true;
            match g.limbs[i].as_mut() {
                Some(p) => {
                    if rng.chance(fx(1, 2)) {
                        p.role = Role::ALL[rng.below(Role::ALL.len())];
                    } else {
                        p.build = Build::ALL[rng.below(Build::ALL.len())];
                    }
                }
                None => {
                    // A limb buds where there was none: a nub, a stub that
                    // walks, half as long as a neighbour. What it becomes
                    // is later generations' business.
                    let model = g.limbs.iter().flatten().next().copied();
                    g.limbs[i] = Some(LimbPair {
                        role: Role::Walk,
                        build: Build::Stub,
                        tip: Tip::Pad,
                        length: model.map_or(fx(1, 5), |m| m.length.mul(fx(1, 2))),
                        girth: model.map_or(fx(1, 2), |m| m.girth),
                    });
                }
            }
        }
    }
    // A pair lost: rarer than anything, and only one of several.
    if g.pair_count() > 1 && rng.chance(rates.mutation_major.mul(fx(1, 2))) {
        let which = rng.below(STATIONS);
        for k in 0..STATIONS {
            let i = (which + k) % STATIONS;
            if g.limbs[i].is_some() {
                g.limbs[i] = None;
                moved.limbs[i] = true;
                break;
            }
        }
    }
    let _ = Station::ALL;
}
