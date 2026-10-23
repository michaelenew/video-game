//! **Ranges**: creatures out on the land (exploration 0019 §2e, built
//! 2026-10-15). The owner: "monsters just roaming -- easy ones close, harder
//! ones the further you go into the world", and "I don't want every creature
//! encounter now to be a fight necessarily: there can be a herd of peaceful
//! Ridgebacks".
//!
//! **A range is a room without walls.** It is a place of the map like a room
//! -- an arena of its own, so the creature fights in its own coordinates and
//! is kept inside its own bounds until it is after somebody who has left them
//! ([`super::roam`]) -- but its floor is the land, not a pad, and
//! nothing rings it: a pocket of meadow beside the road, a clearing in the
//! Wood, the top of the Needle's dome. Walking into it wakes it, exactly as
//! walking into a room does (`valley::open`), and walking well away puts it
//! to sleep again. Each range has:
//!
//! - **a temperament** ([`Temperament`]): whether it comes for you, waits for
//!   you to come close, or wants nothing to do with you unless you start it --
//!   and if you do, whether it fights back or runs;
//! - **a temper** (`crate::temper`): how clever it is, rising with how far it
//!   is from Hearth -- the same Gnawers are sharper in the Pinewood than by
//!   the Mouth's river;
//! - **a rest**: beaten, it is back after `tuning::range_rest` (VALE-3,
//!   "after a while"), not the next time you walk past.
//!
//! The snapshot keeps one live hunt, as it always has; ranges are spaced so
//! that is all a pair meets.

use crate::arena::{Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Spawns};
use crate::objective::Site;
use crate::species::SpeciesId;
use crate::valley::{Kind, Place};

/// **How a range's creatures take to you.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Temperament {
    /// It comes for you the moment you walk in.
    Hunting,
    /// It watches until somebody comes within `ring` centimetres of the
    /// range's middle, or hurts it; then it fights.
    Territorial { ring: i32 },
    /// It grazes and watches and never starts anything; hurt, it fights back.
    Peaceful,
    /// It wants nothing to do with you; hurt, the herd runs for good, and a
    /// creature with a body turns its back.
    Skittish,
}

/// **One range.**
#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub arena: ArenaId,
    pub name: &'static str,
    pub species: SpeciesId,
    /// How many of it, for a creature with a body: two Ridgebacks are a herd.
    pub bodies: u8,
    /// Its middle on the map, centimetres (x, z), and the floor's height
    /// there.
    pub at: (i32, i32),
    pub floor: i32,
    /// Half its side, centimetres.
    pub half: i32,
    pub temperament: Temperament,
    /// How clever (`crate::temper`), 0 to 3.
    pub temper: u8,
    /// **Stacked over another place**: only a range between its floor and
    /// this far above it -- the roost, over the Needle's plaza.
    pub stacked: Option<i32>,
    /// Where the creature's own fight looks for its features, in the range's
    /// coordinates: the Galewing's circle and perch.
    pub sites: &'static [Site],
}

/// The first range's id. Ranges are numbered from here...
pub const FIRST: u8 = 40;

/// ...for [`BLOCK`] of them, and on from here after that: forty-four to
/// sixty-three are other arenas' (the stages from fifty-four).
pub const MORE: u8 = 64;

/// How many ranges are numbered from [`FIRST`].
pub const BLOCK: u8 = 14;

/// The id of range `k`.
pub const fn id_of(k: u8) -> ArenaId {
    if k < BLOCK {
        ArenaId(FIRST + k)
    } else {
        ArenaId(MORE + k - BLOCK)
    }
}

#[allow(clippy::too_many_arguments)]
const fn range(
    k: u8,
    name: &'static str,
    species: SpeciesId,
    bodies: u8,
    at: (i32, i32, i32),
    half: i32,
    temperament: Temperament,
    temper: u8,
) -> Range {
    Range {
        arena: id_of(k),
        name,
        species,
        bodies,
        at: (at.0, at.1),
        floor: at.2,
        half,
        temperament,
        temper,
        stacked: None,
        sites: &[],
    }
}

/// The Needle's dome: where the roost's middle is, and its top's height
/// over the plaza (`arena::spire`).
const NEEDLE: (i32, i32) = crate::arena::spire::AT;

/// The roost's features, in its own coordinates (its middle the Needle's
/// axis, its floor the dome's top): the Galewing circles twenty metres out,
/// and perches on the tallest stump of the ruins.
const ROOST_SITES: [Site; 2] = [
    Site {
        name: "circle",
        route: &[(0, 0)],
        size: (2000, 2000, 0),
        material: Material::Stone,
    },
    Site {
        name: "perch",
        route: &[(-1112, -3424)],
        size: (75, 75, 500),
        material: Material::Stone,
    },
];

/// **The ranges**, nearest the town first. Easy by the town and harder the
/// further in -- by species, the tiers the waystones count, and by temper.
pub const RANGES: [Range; 17] = [
    // The Hornback herd, grazing the Great Wood's edge: they run if you
    // start anything.
    range(
        0,
        "Wood's Edge",
        SpeciesId::HORNBACK,
        1,
        (-6000, -20000, 0),
        2500,
        Temperament::Skittish,
        0,
    ),
    // Gnawers rooting along the Mouth's south bank, across the river from
    // the road: they come for you.
    range(
        1,
        "South Bank",
        SpeciesId::GNAWERS,
        1,
        (12500, -5400, 50),
        1500,
        Temperament::Hunting,
        0,
    ),
    // Two Ridgebacks on the Bank's upper meadow: a herd that wants nothing
    // but the grass, and fights back if hurt.
    range(
        2,
        "Upper Meadow",
        SpeciesId::RIDGEBACK,
        2,
        (29000, -5600, 2100),
        2000,
        Temperament::Peaceful,
        0,
    ),
    // The Pair in a glade north of the Pinewood's road: left alone at the
    // glade's edge, come for you in its middle.
    range(
        3,
        "Glade",
        SpeciesId::PAIR,
        2,
        (66500, 5600, 5650),
        2000,
        Temperament::Territorial { ring: 1400 },
        2,
    ),
    // A Ridgeback on the Saddle's snowfield, cleverer than any by the town,
    // and hunting.
    range(
        4,
        "Snowfield",
        SpeciesId::RIDGEBACK,
        1,
        (88500, -4600, 8400),
        2000,
        Temperament::Hunting,
        3,
    ),
    // **The roost**: a Galewing on the Needle's dome, three hundred and
    // fifty metres up -- watching from its perch until somebody sets foot
    // on the ruins (VALE-5, "fight").
    Range {
        stacked: Some(4000),
        sites: &ROOST_SITES,
        ..range(
            5,
            "Roost",
            SpeciesId::GALEWING,
            1,
            (NEEDLE.0, NEEDLE.1, crate::arena::spire::TOP),
            crate::arena::spire::RIM,
            Temperament::Territorial { ring: 3200 },
            1,
        )
    },
    // **The Deep** (`arena::deep`): a range on each warren's floor and two
    // in the Caverns, stacked over each other -- each only at its own level.
    deep_range(
        6,
        "Upper Warren",
        SpeciesId::GNAWERS,
        1,
        (5, 4),
        0,
        Temperament::Hunting,
        1,
    ),
    deep_range(
        7,
        "Middle Warren",
        SpeciesId::PAIR,
        2,
        (4, 3),
        1,
        Temperament::Territorial { ring: 1000 },
        2,
    ),
    deep_range(
        8,
        "Lower Warren",
        SpeciesId::GNAWERS,
        1,
        (3, 4),
        2,
        Temperament::Hunting,
        2,
    ),
    Range {
        half: 2600,
        stacked: Some(3000),
        ..range(
            9,
            "Crystal Shore",
            SpeciesId::HORNBACK,
            1,
            (
                crate::arena::deep::AT[0] - 3000,
                crate::arena::deep::AT[2],
                crate::arena::deep::AT[1],
            ),
            2600,
            Temperament::Skittish,
            0,
        )
    },
    Range {
        stacked: Some(3000),
        ..range(
            10,
            "Caverns",
            SpeciesId::RIDGEBACK,
            2,
            (
                crate::arena::deep::AT[0] + 4500,
                crate::arena::deep::AT[2] + 3500,
                crate::arena::deep::AT[1],
            ),
            2400,
            Temperament::Hunting,
            3,
        )
    },
    // **The High Plane** (`arena::heights`): its own creatures, 300 m up.
    range(
        11,
        "Sky Meadow",
        SpeciesId::RIDGEBACK,
        2,
        plane(-6000, 4000),
        2500,
        Temperament::Peaceful,
        1,
    ),
    range(
        12,
        "Eyrie",
        SpeciesId::GALEWING,
        1,
        plane(8000, -2000),
        3000,
        Temperament::Hunting,
        2,
    ),
    range(
        13,
        "Sky Shrine",
        SpeciesId::MANTIS,
        1,
        plane(2000, 8000),
        2200,
        Temperament::Territorial { ring: 1500 },
        2,
    ),
    // **By the road** (2026-10-22): the creatures whose rooms were clearings
    // at the end of side paths -- the herd's low meadow, the Gnawers' den, the
    // Highlands' Ridgeback -- out on the land beside the road instead, where
    // they are seen from it. The owner: "remove the very obvious little areas
    // off the valley".
    //
    // The herd in the Mouth's meadow, north of the road: it grazes, and
    // fights back if hurt.
    range(
        14,
        "Riverside",
        SpeciesId::HORNBACK,
        1,
        (11500, 6600, 50),
        2200,
        Temperament::Peaceful,
        0,
    ),
    // The Gnawers on the Bank's upper meadow, north of the road: they let
    // you pass on the road and come for you nearer.
    range(
        15,
        "Hillside",
        SpeciesId::GNAWERS,
        1,
        (30500, 6100, 2100),
        2000,
        Temperament::Territorial { ring: 1400 },
        0,
    ),
    // A Ridgeback on the Pinewood's moor, in a bay north of the road.
    range(
        16,
        "Moor",
        SpeciesId::RIDGEBACK,
        1,
        (73500, 6200, 5750),
        2000,
        Temperament::Territorial { ring: 1600 },
        2,
    ),
];

/// A point of the High Plane on the map, from its middle (centimetres), at
/// its floor.
const fn plane(x: i32, z: i32) -> (i32, i32, i32) {
    let p = crate::arena::heights::PLANE_AT;
    (p[0] + x, p[2] + z, p[1])
}

/// **A range on a warren's floor**: centred on a cell of its maze, three
/// cells across, only at its level.
#[allow(clippy::too_many_arguments)]
const fn deep_range(
    k: u8,
    name: &'static str,
    species: SpeciesId,
    bodies: u8,
    cell: (i32, i32),
    level: usize,
    temperament: Temperament,
    temper: u8,
) -> Range {
    use crate::arena::deep;
    let (x, z) = deep::cell_middle(cell);
    Range {
        stacked: Some(2500),
        ..range(
            k,
            name,
            species,
            bodies,
            (
                deep::AT[0] + x,
                deep::AT[2] + z,
                deep::AT[1] + deep::LEVELS[level],
            ),
            1800,
            temperament,
            temper,
        )
    }
}

/// How many ranges there are.
pub const COUNT: usize = RANGES.len();

/// **The place each range stands in**, by number: the reach whose land is
/// its floor -- or the Wood, the Needle, the Deep or the High Plane. What
/// it looks like is that place's (`look::skies`): a range is a pocket of
/// its reach, not a place with an air of its own. `tests/ranges.rs` holds
/// it to the map.
pub const IN: [ArenaId; COUNT] = [
    ArenaId::WOOD,
    ArenaId::MOUTH,
    ArenaId::BANK,
    ArenaId::PINEWOOD,
    ArenaId::SADDLE,
    ArenaId::SPIRE,
    ArenaId::DEEP,
    ArenaId::DEEP,
    ArenaId::DEEP,
    ArenaId::DEEP,
    ArenaId::DEEP,
    ArenaId::HIGH_PLANE,
    ArenaId::HIGH_PLANE,
    ArenaId::HIGH_PLANE,
    ArenaId::MOUTH,
    ArenaId::BANK,
    ArenaId::PINEWOOD,
];

/// The place a range stands in, if `id` is a range.
pub const fn in_place(id: ArenaId) -> Option<ArenaId> {
    match index(id) {
        Some(k) => Some(IN[k]),
        None => None,
    }
}

/// The range an arena is, if it is one.
pub const fn of(id: ArenaId) -> Option<&'static Range> {
    match index(id) {
        Some(k) => Some(&RANGES[k]),
        None => None,
    }
}

/// Which range, by number.
pub const fn index(id: ArenaId) -> Option<usize> {
    let k = if id.0 >= FIRST && id.0 < FIRST + BLOCK {
        id.0 - FIRST
    } else if id.0 >= MORE {
        id.0 - MORE + BLOCK
    } else {
        return None;
    };
    if (k as usize) < COUNT {
        Some(k as usize)
    } else {
        None
    }
}

const fn arena_of(r: &Range) -> Arena {
    let h = r.half;
    Arena {
        id: r.arena,
        name: r.name,
        creature: None,
        bounds: Bounds::cm((-h, h), (-h, h)),
        floor: Material::Grass,
        regions: &[],
        solids: &[],
        spawns: Spawns {
            // Where a fighter stands after a hunt here is lost and there is
            // no cairn: the range's near edge, facing in.
            versus: [
                Mark::cm(-150, -h + 300, (-150, 0)),
                Mark::cm(150, -h + 300, (150, 0)),
            ],
            hunt: Some(HuntMarks {
                hunters: [
                    Mark::cm(-150, -h + 300, (-150, 0)),
                    Mark::cm(150, -h + 300, (150, 0)),
                ],
                creatures: [
                    Mark::cm(-h / 4, h / 3, (-h / 4, -h)),
                    Mark::cm(h / 4, h / 3, (h / 4, -h)),
                ],
            }),
        },
        sites: r.sites,
        rim: None,
    }
}

/// Every range's arena, for `arena::lookup`.
pub static ARENAS: [Arena; COUNT] = [
    arena_of(&RANGES[0]),
    arena_of(&RANGES[1]),
    arena_of(&RANGES[2]),
    arena_of(&RANGES[3]),
    arena_of(&RANGES[4]),
    arena_of(&RANGES[5]),
    arena_of(&RANGES[6]),
    arena_of(&RANGES[7]),
    arena_of(&RANGES[8]),
    arena_of(&RANGES[9]),
    arena_of(&RANGES[10]),
    arena_of(&RANGES[11]),
    arena_of(&RANGES[12]),
    arena_of(&RANGES[13]),
    arena_of(&RANGES[14]),
    arena_of(&RANGES[15]),
    arena_of(&RANGES[16]),
];

const fn place_of(r: &Range) -> Place {
    Place {
        arena: r.arena,
        kind: Kind::Room(r.species),
        seams: &[],
        vents: &[],
    }
}

/// Every range's place, for `valley::place`.
pub static PLACES: [Place; COUNT] = [
    place_of(&RANGES[0]),
    place_of(&RANGES[1]),
    place_of(&RANGES[2]),
    place_of(&RANGES[3]),
    place_of(&RANGES[4]),
    place_of(&RANGES[5]),
    place_of(&RANGES[6]),
    place_of(&RANGES[7]),
    place_of(&RANGES[8]),
    place_of(&RANGES[9]),
    place_of(&RANGES[10]),
    place_of(&RANGES[11]),
    place_of(&RANGES[12]),
    place_of(&RANGES[13]),
    place_of(&RANGES[14]),
    place_of(&RANGES[15]),
    place_of(&RANGES[16]),
];
