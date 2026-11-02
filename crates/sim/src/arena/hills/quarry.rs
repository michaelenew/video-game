//! **The Quarry**, a King of the Hill map (`super`, `crate::versus`,
//! `docs/design/versus.md` §3, `docs/design/fight-spaces.md`).

#![allow(unused_imports)]

use super::super::relief::Bump;
use super::super::rim::Rim;
use super::super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use super::{round, slab, whole};
use crate::versus::{Hill, Kind, Stage, Way};

use Material::{Ash, Grass, Ground, Peat, Rock, Sand, Snow, Stone, Water, Wood};

/// **The Quarry**: a worked-out stone quarry, the point on a cut plinth in
/// the pit's floor. North and south of it two **benches** -- terraces the
/// quarrymen left, four metres up and twelve from the point -- are the high
/// ground, each climbed by a stair at its own side's end and a ledge on its
/// face toward the point. Through each bench at its foot runs **the old
/// cutting**, a tunnel the length of it, and halfway along it opens onto the
/// point through **the slot**, a gap down through the bench: the way onto
/// the point nobody sees, and a hole to drop through from the top. Behind
/// each bench a lane runs in the open between it and the quarry's wall; the
/// floor between the benches is open, strewn with cut blocks. A tor by each
/// start is a perch.
pub static QUARRY: Arena = Arena {
    id: ArenaId::QUARRY,
    name: "Quarry",
    creature: None,
    bounds: Bounds::cm((-3800, 3800), (-2900, 2900)),
    floor: Rock,
    regions: &QUARRY_REGIONS,
    solids: &QUARRY_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-3300, 0, (0, 0)), Mark::cm(3300, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    rim: Some(Rim::new((-3800, 3800), (-2900, 2900), [0, 0, 0, 0], 1000)),
};

pub static QUARRY_STAGE: Stage = Stage {
    arena: ArenaId::QUARRY,
    kind: Kind::Hill,
    theme: "A worked-out stone quarry: benches over the point, a cutting under them",
    pit: None,
    hill: Some(Hill::cm(0, 45, 0, 400, 50, 1000)),
    vents: &[],
    drifts: &[],
    ways: &QUARRY_WAYS,
};

const QUARRY_REGIONS: [Region; 5] = [
    // Spoil where the benches were cut.
    Region {
        area: Area::rect_cm((-1200, 1200), (2400, 2900)),
        material: Ground,
    },
    Region {
        area: Area::rect_cm((-1200, 1200), (-2900, -2400)),
        material: Ground,
    },
    // Sand blown into the hollow round the point.
    Region {
        area: Area::disc_cm(0, 0, 900),
        material: Sand,
    },
    // Sand drifted against the tors.
    Region {
        area: Area::disc_cm(-2300, -1500, 400),
        material: Sand,
    },
    Region {
        area: Area::disc_cm(2300, 1500, 400),
        material: Sand,
    },
];

const QUARRY_SOLIDS: [Solid; 31] = whole(
    [
        // The plinth the point is on: a step up, walked.
        Solid::cm([-500, 0, -500], [500, 45, 500], Stone),
    ],
    [
        // The north bench, four metres up: its face toward the point in two,
        // either side of the slot, the cutting's roof behind, and the bench's
        // back behind that. The cutting runs its length at the foot, three
        // metres wide and three high.
        Solid::cm([-1200, 0, 1200], [-150, 400, 1500], Rock),
        Solid::cm([150, 0, 1200], [1200, 400, 1500], Rock),
        Solid::cm([-1200, 300, 1500], [1200, 400, 1800], Rock),
        Solid::cm([-1200, 0, 1800], [1200, 400, 2400], Rock),
        // Its stair, at the west side's end: two cut steps.
        Solid::cm([-1750, 0, 1200], [-1200, 130, 1500], Stone),
        Solid::cm([-1475, 0, 1200], [-1200, 270, 1500], Stone),
        // A ledge on its face toward the point, at the east end: up from
        // the point's side.
        Solid::cm([600, 0, 950], [1000, 200, 1200], Stone),
        // A low wall across the west start: cover coming out.
        Solid::cm([-2800, 0, -600], [-2650, 120, 600], Rock),
        // The tor by the west start, and the step up onto it: a perch over
        // the open floor.
        round(-2300, -1500, (0, 450), (160, 110), Rock),
        Solid::cm([-2200, 0, -1700], [-1800, 220, -1300], Rock),
        // Cut blocks on the floor between the benches.
        Solid::cm([-1900, 0, 100], [-1700, 160, 500], Stone),
        Solid::cm([-1100, 0, -900], [-850, 120, -500], Stone),
        Solid::cm([-750, 0, 650], [-650, 100, 750], Stone),
        // A block left on the plinth: cover on the point.
        Solid::cm([-400, 45, 250], [-250, 165, 400], Stone),
        // A slab fallen across the lane behind the north bench.
        slab(-2600, 1700, (0, 150), (250, 70), (3, 1), Rock),
    ],
);

const QUARRY_WAYS: [Way; 6] = [
    Way::new("way 0", (-2500, 0, 0), &[(0, 45, 0)]),
    Way::new(
        "way 1",
        (-2300, 0, 1350),
        &[(-1600, 130, 1350), (-1330, 270, 1350), (-700, 400, 1350)],
    ),
    Way::new(
        "way 2",
        (800, 0, 500),
        &[(800, 200, 1075), (800, 400, 1650)],
    ),
    Way::new(
        "way 3",
        (-1500, 0, -1500),
        &[(-2000, 220, -1500), (-2300, 450, -1500)],
    ),
    Way::new("way 4", (-1700, 0, 1650), &[(1700, 0, 1650)]),
    Way::new(
        "way 5",
        (-1700, 0, 1650),
        &[(0, 0, 1650), (0, 0, 1350), (0, 45, 300)],
    ),
];

/// The floor's relief (`arena::relief`): smooth hills and hollows, no boxes.
/// Kept clear of the point, the starts and the rim.
pub const BUMPS: [Bump; 0] = [];
