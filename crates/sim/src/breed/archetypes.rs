//! The bestiary as genomes: every creature a person can hunt, written as
//! the choices that make it, and the trophies each drops.
//!
//! These are the litmus test of the builder. `tests/breed.rs` builds each
//! one and lays it beside the hand-written species table: the same number
//! of legs, hips within a fifth of the table's, every one of its moves'
//! kinds in the derived set, the flyer the only one that flies. They are
//! also what a creature spawned in the world is **varied** from
//! ([`vary`]): the same plan with the sliders nudged, so no two Ridgebacks
//! are the same size and the biggest is worth breeding.

use super::build::MoveKind;
use super::cross::TrophyKind;
use super::{
    Back, Build, Crown, Genome, HeadKind, Hide, Jaw, LimbPair, Minors, Rng, Role, STATIONS,
    Segments, Sliders, Stance, Tail, TailTip, Tip, fx,
};
use crate::species::SpeciesId;

const fn pair(
    role: Role,
    build: Build,
    tip: Tip,
    length: (i32, i32),
    girth: (i32, i32),
) -> Option<LimbPair> {
    Some(LimbPair {
        role,
        build,
        tip,
        length: fx(length.0, length.1),
        girth: fx(girth.0, girth.1),
    })
}

#[allow(clippy::too_many_arguments)]
const fn sliders(
    size: (i32, i32),
    mass: (i32, i32),
    pitch: (i32, i32),
    speed: (i32, i32),
    reaction: (i32, i32),
    aggression: (i32, i32),
    tempo: (i32, i32),
    guard: (i32, i32),
) -> Sliders {
    Sliders {
        size: fx(size.0, size.1),
        mass: fx(mass.0, mass.1),
        pitch: fx(pitch.0, pitch.1),
        speed: fx(speed.0, speed.1),
        reaction: fx(reaction.0, reaction.1),
        aggression: fx(aggression.0, aggression.1),
        tempo: fx(tempo.0, tempo.1),
        guard: fx(guard.0, guard.1),
    }
}

const NO_LIMBS: [Option<LimbPair>; STATIONS] = [None; STATIONS];

/// Thirteen metres of armoured animal on four pillars, a level tail with a
/// club of a tip, a ridge of spines along its back, and two soft places on
/// top.
pub const RIDGEBACK: Genome = Genome {
    stance: Stance::Level,
    head: HeadKind::Raised,
    segments: Segments {
        thorax: false,
        abdomen: false,
        tail: Tail::Level,
    },
    limbs: [
        pair(Role::Both, Build::Column, Tip::Hoof, (19, 50), (7, 10)),
        None,
        None,
        pair(Role::Both, Build::Column, Tip::Hoof, (19, 50), (3, 4)),
    ],
    minors: Minors {
        jaw: Jaw::Teeth,
        crown: Crown::None,
        tail_tip: TailTip::Club,
        hide: Hide::Plates,
        back: Back::Ridge,
    },
    sliders: sliders(
        (39, 50),
        (4, 5),
        (0, 1),
        (1, 2),
        (1, 2),
        (3, 5),
        (7, 20),
        (1, 10),
    ),
};

/// A duellist: an upright thorax on a trailing abdomen, two folded blades
/// at the fore, four sprung legs under it, chitin, mandibles.
pub const MANTIS: Genome = Genome {
    stance: Stance::Upright,
    head: HeadKind::Forward,
    segments: Segments {
        thorax: true,
        abdomen: true,
        tail: Tail::None,
    },
    limbs: [
        pair(Role::Strike, Build::Raptorial, Tip::Blade, (3, 5), (3, 25)),
        pair(Role::Walk, Build::Sprung, Tip::Claw, (6, 25), (3, 25)),
        None,
        pair(Role::Walk, Build::Sprung, Tip::Claw, (6, 25), (3, 25)),
    ],
    minors: Minors {
        jaw: Jaw::Mandibles,
        crown: Crown::None,
        tail_tip: TailTip::Plain,
        hide: Hide::Chitin,
        back: Back::Plain,
    },
    sliders: sliders(
        (27, 50),
        (1, 4),
        (17, 20),
        (11, 20),
        (19, 20),
        (11, 20),
        (4, 5),
        (4, 5),
    ),
};

/// A raptor: level on two taloned legs, wings as long as the body at the
/// fore, a beak, a crest, feathers, light bones.
pub const GALEWING: Genome = Genome {
    stance: Stance::Level,
    head: HeadKind::Raised,
    segments: Segments {
        thorax: false,
        abdomen: false,
        tail: Tail::Level,
    },
    limbs: [
        pair(Role::Wing, Build::Wing, Tip::Feather, (1, 1), (1, 4)),
        None,
        None,
        pair(Role::Both, Build::Sprung, Tip::Talon, (23, 50), (7, 20)),
    ],
    minors: Minors {
        jaw: Jaw::Beak,
        crown: Crown::Crest,
        tail_tip: TailTip::Plain,
        hide: Hide::Feathers,
        back: Back::Plain,
    },
    sliders: sliders(
        (117, 200),
        (1, 5),
        (1, 20),
        (7, 10),
        (3, 5),
        (3, 5),
        (3, 5),
        (1, 10),
    ),
};

/// A spider: low on eight spindle legs, the fore pair stabbing, a huge
/// abdomen behind the hips carrying her sacs, fangs.
pub const BROODMOTHER: Genome = Genome {
    stance: Stance::Lowslung,
    head: HeadKind::Forward,
    segments: Segments {
        thorax: false,
        abdomen: true,
        tail: Tail::None,
    },
    limbs: [
        pair(Role::Both, Build::Spindle, Tip::Claw, (4, 5), (1, 10)),
        pair(Role::Walk, Build::Spindle, Tip::Claw, (4, 5), (1, 10)),
        pair(Role::Walk, Build::Spindle, Tip::Claw, (4, 5), (1, 10)),
        pair(Role::Walk, Build::Spindle, Tip::Claw, (4, 5), (1, 10)),
    ],
    minors: Minors {
        jaw: Jaw::Fangs,
        crown: Crown::None,
        tail_tip: TailTip::Plain,
        hide: Hide::Chitin,
        back: Back::Sacs,
    },
    sliders: sliders(
        (133, 200),
        (1, 2),
        (0, 1),
        (7, 20),
        (1, 2),
        (1, 2),
        (2, 5),
        (1, 5),
    ),
};

/// A walking hill: six pillars splayed out from under a shell, a load on
/// its back, a grinding mouth that ploughs, slow as stone.
pub const SIEGESHELL: Genome = Genome {
    stance: Stance::Lowslung,
    head: HeadKind::Forward,
    segments: Segments {
        thorax: false,
        abdomen: false,
        tail: Tail::None,
    },
    limbs: [
        pair(Role::Both, Build::Column, Tip::Hoof, (1, 2), (9, 10)),
        pair(Role::Walk, Build::Column, Tip::Hoof, (1, 2), (9, 10)),
        None,
        pair(Role::Walk, Build::Column, Tip::Hoof, (1, 2), (9, 10)),
    ],
    minors: Minors {
        jaw: Jaw::Grinder,
        crown: Crown::None,
        tail_tip: TailTip::Plain,
        hide: Hide::Shell,
        back: Back::Burden,
    },
    sliders: sliders(
        (47, 50),
        (1, 1),
        (0, 1),
        (1, 10),
        (1, 10),
        (3, 10),
        (1, 10),
        (3, 10),
    ),
};

/// One of the two cats: level, sprung, clawed at the fore, fanged, furred,
/// quick, and a tail for tripping.
pub const PAIR: Genome = Genome {
    stance: Stance::Level,
    head: HeadKind::Forward,
    segments: Segments {
        thorax: false,
        abdomen: false,
        tail: Tail::Level,
    },
    limbs: [
        pair(Role::Both, Build::Sprung, Tip::Claw, (3, 5), (7, 20)),
        None,
        None,
        pair(Role::Walk, Build::Sprung, Tip::Pad, (3, 5), (2, 5)),
    ],
    minors: Minors {
        jaw: Jaw::Fangs,
        crown: Crown::None,
        tail_tip: TailTip::Plain,
        hide: Hide::Fur,
        back: Back::Plain,
    },
    sliders: sliders(
        (19, 50),
        (7, 20),
        (0, 1),
        (17, 20),
        (7, 10),
        (7, 10),
        (17, 20),
        (1, 10),
    ),
};

/// The animal you never see: a bigger cat with a raised tail that ends in
/// a spear, quills along its back.
pub const VEILSTALKER: Genome = Genome {
    stance: Stance::Level,
    head: HeadKind::Forward,
    segments: Segments {
        thorax: false,
        abdomen: false,
        tail: Tail::Raised,
    },
    limbs: [
        pair(Role::Both, Build::Sprung, Tip::Claw, (1, 2), (7, 20)),
        None,
        None,
        pair(Role::Walk, Build::Sprung, Tip::Pad, (1, 2), (2, 5)),
    ],
    minors: Minors {
        jaw: Jaw::Fangs,
        crown: Crown::None,
        tail_tip: TailTip::Blade,
        hide: Hide::Fur,
        back: Back::Ridge,
    },
    sliders: sliders(
        (19, 40),
        (2, 5),
        (0, 1),
        (4, 5),
        (3, 5),
        (1, 2),
        (3, 4),
        (1, 5),
    ),
};

/// A toad: low and wide on four stubs, no neck, a maw with a tongue in it,
/// a slick hide and a throat sac.
pub const MIREBACK: Genome = Genome {
    stance: Stance::Lowslung,
    head: HeadKind::Merged,
    segments: Segments {
        thorax: false,
        abdomen: false,
        tail: Tail::None,
    },
    limbs: [
        pair(Role::Walk, Build::Stub, Tip::Pad, (11, 50), (1, 2)),
        None,
        None,
        pair(Role::Walk, Build::Stub, Tip::Pad, (11, 50), (3, 5)),
    ],
    minors: Minors {
        jaw: Jaw::Maw,
        crown: Crown::None,
        tail_tip: TailTip::Plain,
        hide: Hide::Slick,
        back: Back::Sacs,
    },
    sliders: sliders(
        (147, 200),
        (3, 4),
        (0, 1),
        (3, 20),
        (3, 10),
        (9, 20),
        (1, 5),
        (3, 10),
    ),
};

/// A worm: no limbs, a column of segments with a maw at the front and a
/// tail behind, hunting by ear.
pub const SANDMAW: Genome = Genome {
    stance: Stance::Limbless,
    head: HeadKind::Merged,
    segments: Segments {
        thorax: false,
        abdomen: false,
        tail: Tail::Level,
    },
    limbs: NO_LIMBS,
    minors: Minors {
        jaw: Jaw::Maw,
        crown: Crown::Feelers,
        tail_tip: TailTip::Plain,
        hide: Hide::Skin,
        back: Back::Plain,
    },
    sliders: sliders(
        (4, 5),
        (1, 2),
        (0, 1),
        (11, 20),
        (1, 2),
        (7, 10),
        (1, 2),
        (0, 1),
    ),
};

/// The herd's bull: a level grazer with a low head and horns, hooves,
/// heavy, and a kick behind.
pub const HORNBACK: Genome = Genome {
    stance: Stance::Level,
    head: HeadKind::Low,
    segments: Segments {
        thorax: false,
        abdomen: false,
        tail: Tail::Drooping,
    },
    limbs: [
        pair(Role::Walk, Build::Column, Tip::Hoof, (2, 5), (11, 20)),
        None,
        None,
        pair(Role::Both, Build::Column, Tip::Hoof, (2, 5), (3, 5)),
    ],
    minors: Minors {
        jaw: Jaw::Grinder,
        crown: Crown::Horns,
        tail_tip: TailTip::Plain,
        hide: Hide::Fur,
        back: Back::Plain,
    },
    sliders: sliders(
        (1, 2),
        (7, 10),
        (0, 1),
        (13, 20),
        (3, 10),
        (1, 2),
        (1, 2),
        (2, 5),
    ),
};

/// One gnawer: knee-high, sprung, toothed, quick and aggressive.
pub const GNAWER: Genome = Genome {
    stance: Stance::Level,
    head: HeadKind::Forward,
    segments: Segments {
        thorax: false,
        abdomen: false,
        tail: Tail::Level,
    },
    limbs: [
        pair(Role::Both, Build::Sprung, Tip::Claw, (2, 5), (3, 10)),
        None,
        None,
        pair(Role::Walk, Build::Sprung, Tip::Pad, (2, 5), (7, 20)),
    ],
    minors: Minors {
        jaw: Jaw::Teeth,
        crown: Crown::None,
        tail_tip: TailTip::Plain,
        hide: Hide::Fur,
        back: Back::Plain,
    },
    sliders: sliders(
        (6, 25),
        (2, 5),
        (0, 1),
        (7, 10),
        (3, 5),
        (4, 5),
        (9, 10),
        (0, 1),
    ),
};

/// The genome of a species the bestiary has, if it is one of the eleven.
pub const fn of(id: SpeciesId) -> Option<&'static Genome> {
    match id {
        SpeciesId::RIDGEBACK => Some(&RIDGEBACK),
        SpeciesId::GNAWERS => Some(&GNAWER),
        SpeciesId::HORNBACK => Some(&HORNBACK),
        SpeciesId::MIREBACK => Some(&MIREBACK),
        SpeciesId::SANDMAW => Some(&SANDMAW),
        SpeciesId::PAIR => Some(&PAIR),
        SpeciesId::BROODMOTHER => Some(&BROODMOTHER),
        SpeciesId::VEILSTALKER => Some(&VEILSTALKER),
        SpeciesId::MANTIS => Some(&MANTIS),
        SpeciesId::GALEWING => Some(&GALEWING),
        SpeciesId::SIEGESHELL => Some(&SIEGESHELL),
        _ => None,
    }
}

/// Every creature with a genome, with its name and the kind of moves its
/// species table has, in the terms of [`MoveKind`]. The litmus test reads
/// this: every kind here must come out of the builder.
pub const CAST: [(SpeciesId, &str, &Genome, &[MoveKind]); 11] = [
    (
        SpeciesId::RIDGEBACK,
        "Ridgeback",
        &RIDGEBACK,
        &[
            MoveKind::Bite,
            MoveKind::Stomp,
            MoveKind::TailSweep,
            MoveKind::Charge,
            MoveKind::RearSlam,
            MoveKind::Shake,
            MoveKind::BackKick,
            MoveKind::SpikeSpray,
        ],
    ),
    (
        SpeciesId::MANTIS,
        "Mantis",
        &MANTIS,
        &[
            MoveKind::Scythe,
            MoveKind::Lunge,
            MoveKind::Guard,
            MoveKind::Parry,
            MoveKind::Leap,
            MoveKind::WingFlare,
        ],
    ),
    (
        SpeciesId::GALEWING,
        "Galewing",
        &GALEWING,
        &[
            MoveKind::Stoop,
            MoveKind::TalonPass,
            MoveKind::Downwash,
            MoveKind::FeatherVolley,
            MoveKind::Screech,
            MoveKind::Buffet,
            MoveKind::Leap,
        ],
    ),
    (
        SpeciesId::BROODMOTHER,
        "Broodmother",
        &BROODMOTHER,
        &[
            MoveKind::LegStab,
            MoveKind::Bite,
            MoveKind::SacSlam,
            MoveKind::Screech,
            MoveKind::WebShot,
        ],
    ),
    (
        SpeciesId::SIEGESHELL,
        "Siegeshell",
        &SIEGESHELL,
        &[
            MoveKind::Stomp,
            MoveKind::Shed,
            MoveKind::Plough,
            MoveKind::Shrug,
        ],
    ),
    (
        SpeciesId::PAIR,
        "The Pair",
        &PAIR,
        &[
            MoveKind::Pounce,
            MoveKind::Rake,
            MoveKind::Ambush,
            MoveKind::TailTrip,
            MoveKind::Leap,
        ],
    ),
    (
        SpeciesId::VEILSTALKER,
        "Veilstalker",
        &VEILSTALKER,
        &[
            MoveKind::Ambush,
            MoveKind::TailSpear,
            MoveKind::Rake,
            MoveKind::Pounce,
            MoveKind::QuillFling,
        ],
    ),
    (
        SpeciesId::MIREBACK,
        "Mireback",
        &MIREBACK,
        &[
            MoveKind::Spew,
            MoveKind::BellyFlop,
            MoveKind::Tongue,
            MoveKind::Swallow,
            MoveKind::Inflate,
            MoveKind::Wallow,
        ],
    ),
    (
        SpeciesId::SANDMAW,
        "Sandmaw",
        &SANDMAW,
        &[
            MoveKind::RiseBite,
            MoveKind::BreachDive,
            MoveKind::Spit,
            MoveKind::Lash,
            MoveKind::Swallow,
        ],
    ),
    (
        SpeciesId::HORNBACK,
        "Hornback",
        &HORNBACK,
        &[
            MoveKind::Charge,
            MoveKind::Gore,
            MoveKind::RearSlam,
            MoveKind::Guard,
            MoveKind::BackKick,
        ],
    ),
    (
        SpeciesId::GNAWERS,
        "Gnawers",
        &GNAWER,
        &[MoveKind::Bite, MoveKind::Pounce],
    ),
];

/// A species' genome, by its id: what its egg is and its trophies come
/// from. None for one with no genome (a dev pack).
pub fn of_species(id: SpeciesId) -> Option<&'static Genome> {
    CAST.iter()
        .find(|(s, _, _, _)| *s == id)
        .map(|(_, _, g, _)| *g)
}

/// What a creature drops, by its genome: the trophies its parts are.
pub fn drops(g: &Genome) -> [Option<TrophyKind>; 6] {
    let mut out = [None; 6];
    let mut n = 0;
    let mut push = |k: TrophyKind| {
        if n < out.len() {
            out[n] = Some(k);
            n += 1;
        }
    };
    match g.minors.jaw {
        Jaw::Fangs => push(TrophyKind::Fang),
        Jaw::Beak => push(TrophyKind::Beak),
        Jaw::Teeth | Jaw::Maw | Jaw::Mandibles => push(TrophyKind::Tooth),
        Jaw::Grinder => {}
    }
    if g.minors.crown == Crown::Horns {
        push(TrophyKind::Horn);
    }
    match g.minors.hide {
        Hide::Feathers => push(TrophyKind::Feather),
        Hide::Plates => push(TrophyKind::Plate),
        Hide::Shell => push(TrophyKind::Shell),
        Hide::Fur => push(TrophyKind::Pelt),
        Hide::Chitin => push(TrophyKind::Chitin),
        Hide::Skin | Hide::Slick => {}
    }
    if g.minors.back == Back::Ridge {
        push(TrophyKind::Quill);
    }
    if g.minors.back == Back::Sacs {
        push(TrophyKind::Sac);
    }
    if g.minors.tail_tip != TailTip::Plain {
        push(TrophyKind::TailTip);
    }
    let mut tips = g.limbs.iter().flatten().map(|p| p.tip);
    if let Some(t) = tips.find(|t| matches!(t, Tip::Claw | Tip::Hoof | Tip::Talon)) {
        push(match t {
            Tip::Claw => TrophyKind::Claw,
            Tip::Hoof => TrophyKind::Hoof,
            _ => TrophyKind::Talon,
        });
    }
    out
}

/// A creature of the species as it spawns in the world: the archetype with
/// its sliders nudged by a seed, so a herd has a biggest bull and a
/// quickest cow. Majors never move here; the plan is the species.
pub fn vary(archetype: &Genome, seed: u64) -> Genome {
    let mut rng = Rng::new(seed);
    let mut g = *archetype;
    for s in super::Slider::ALL {
        let v = g.sliders.get(s);
        g.sliders
            .set(s, v.add(rng.normal(VARIETY.mul(s.inertia()))));
    }
    for p in g.limbs.iter_mut().flatten() {
        p.length = super::unit(p.length.add(rng.normal(VARIETY.mul(fx(1, 2)))));
        p.girth = super::unit(p.girth.add(rng.normal(VARIETY.mul(fx(1, 2)))));
    }
    g
}

/// The spread of a wild population's sliders: a standard deviation of a
/// twentieth, times each slider's inertia.
pub const VARIETY: crate::fixed::Fx = fx(1, 20);
