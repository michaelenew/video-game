//! **The Quarry**, a King of the Hill map (`super`, `crate::versus`,
//! `docs/design/versus.md` §3, `docs/design/fight-spaces.md`).
//!
//! Rebuilt to the fight-space rules (2026-11-02): a terraced pit 128 by 92
//! metres, the point on its floor, the quarry's two faces stepping up north
//! and south of it in two benches, six and a half and fourteen and a half
//! metres up -- three levels, each more than a jump and a half over the
//! last -- with the lower bench pushed out toward the point as the contest's
//! high ground and the upper one standing just out of a reach behind it.

#![allow(unused_imports)]

use super::super::relief::Bump;
use super::super::rim::Rim;
use super::super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns, cm};
use super::{round, slab, turned, whole};
use crate::drift::Drift;
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::Vent;
use crate::versus::{Hill, Kind, Stage, Way};

use Material::{Ash, Grass, Ground, Peat, Rock, Sand, Snow, Stone, Water, Wood};

/// **The Quarry**: a worked-out stone quarry, 128 by 92 metres. The point
/// is on a cut plinth on **the pit floor**. North and south the quarry's
/// faces climb in two benches the length of the pit: **the lower bench**,
/// six and a half metres up, and **the upper bench**, fourteen and a half,
/// against the rim. In the middle each face **juts out** toward the point --
/// the lower bench fourteen metres from it, the contest's high ground, and
/// the upper bench standing twenty-six metres back, just out of a reach,
/// over everything. Through each jut at its foot runs **the cutting**, a
/// tunnel forty metres long, and halfway along it **the slot** opens onto
/// the point: the way in nobody sees, and a hole to drop through from the
/// bench. Each jut has **a hoist**, a lift from the lower bench to the upper,
/// and **a crane jib** reaching out from the upper bench over the slot: a
/// plank fourteen metres up, in reach of the point, to be knocked off.
/// **Two gantries** cross the pit from jut to jut, sixteen metres west and
/// east of the point, over the open floor. Each side climbs the face on its
/// left by **the steps** at its own end, and the face on its right by **a
/// spoil ramp** to the lower bench and an updraft up the upper bench's face;
/// each jut's lower bench is also climbed from the pit by an updraft and by a
/// ledge on its face. Out past a reach the floor has a cut block to climb,
/// a tor, spoil heaps and a low wall by each start.
pub static QUARRY: Arena = Arena {
    id: ArenaId::QUARRY,
    name: "Quarry",
    creature: None,
    bounds: Bounds::cm((-6400, 6400), (-4600, 4600)),
    floor: Rock,
    regions: &QUARRY_REGIONS,
    solids: &QUARRY_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-5800, 0, (0, 0)), Mark::cm(5800, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    // The bank starts at the upper benches' height on the north and south,
    // whose backs are against it: walking off a bench's back is walking onto
    // the bank, not into a crease under it. It climbs twenty-five metres, so
    // nobody leaving the top bench clears it.
    rim: Some(Rim::new(
        (-6400, 6400),
        (-4600, 4600),
        [1450, 1450, 0, 0],
        2500,
    )),
};

pub static QUARRY_STAGE: Stage = Stage {
    arena: ArenaId::QUARRY,
    kind: Kind::Hill,
    theme: "A terraced stone quarry: the point on the pit floor, two benches up each face",
    pit: None,
    hill: Some(Hill::cm(0, 45, 0, 400, 50, 1000)),
    front: None,
    vents: &QUARRY_VENTS,
    drifts: &QUARRY_DRIFTS,
    ways: &QUARRY_WAYS,
};

const QUARRY_REGIONS: [Region; 9] = [
    // Sand blown into the hollow round the point.
    Region {
        area: Area::disc_cm(0, 0, 900),
        material: Sand,
    },
    // The spoil ramps, tipped against the faces.
    Region {
        area: Area::disc_cm(4410, 3000, 1300),
        material: Ground,
    },
    Region {
        area: Area::disc_cm(-4410, -3000, 1300),
        material: Ground,
    },
    // Spoil heaps on the open floor.
    Region {
        area: Area::disc_cm(-2800, -1800, 700),
        material: Ground,
    },
    Region {
        area: Area::disc_cm(2800, 1800, 700),
        material: Ground,
    },
    // Sand drifted against the tors.
    Region {
        area: Area::disc_cm(-4800, -1200, 400),
        material: Sand,
    },
    Region {
        area: Area::disc_cm(4800, 1200, 400),
        material: Sand,
    },
    // The cuttings' floors, trodden.
    Region {
        area: Area::rect_cm((-2000, 2000), (1800, 2200)),
        material: Ground,
    },
    Region {
        area: Area::rect_cm((-2000, 2000), (-2200, -1800)),
        material: Ground,
    },
];

const QUARRY_SOLIDS: [Solid; 49] = whole(
    [
        // The plinth the point is on: a step up, walked.
        Solid::cm([-500, 0, -500], [500, 45, 500], Stone),
    ],
    [
        // ---- The north face, the length of the pit (its turn is the south).
        // The upper bench along the rim, fourteen and a half metres up, and
        // its jut toward the point in the middle.
        Solid::cm([-5600, 0, 3800], [5600, 1450, 4600], Rock),
        Solid::cm([-2000, 0, 2600], [2000, 1450, 3800], Rock),
        // The lower bench in front of it, six and a half metres up.
        Solid::cm([-5600, 0, 3000], [5600, 650, 3800], Rock),
        // Its jut toward the point, fourteen metres from it: the face in two
        // either side of the slot, the cutting's roof behind, the back
        // behind that. The cutting runs the jut's length at its foot, four
        // metres wide and four and a half high.
        Solid::cm([-2000, 0, 1400], [-200, 650, 1800], Rock),
        Solid::cm([200, 0, 1400], [2000, 650, 1800], Rock),
        Solid::cm([-2000, 450, 1800], [2000, 650, 2200], Rock),
        Solid::cm([-2000, 0, 2200], [2000, 650, 2600], Rock),
        // The two shoulders joining the jut to the bench, either side of the
        // upper bench's jut; the cutting's mouths open beside them.
        Solid::cm([-2400, 0, 2200], [-2000, 650, 3000], Rock),
        Solid::cm([2000, 0, 2200], [2400, 650, 3000], Rock),
        // The steps at the west end: two cut blocks up to the lower bench,
        // two more on it up to the upper.
        Solid::cm([-5600, 0, 2600], [-5200, 220, 3000], Stone),
        Solid::cm([-5200, 0, 2600], [-4800, 440, 3000], Stone),
        Solid::cm([-4800, 0, 3400], [-4400, 920, 3800], Stone),
        Solid::cm([-5200, 0, 3400], [-4800, 1190, 3800], Stone),
        // A ledge on the jut's face, east of the slot: up from the pit.
        Solid::cm([900, 0, 1100], [1500, 220, 1400], Stone),
        // The crane jib, out from the upper bench over the slot.
        Solid::cm([-150, 1400, 1400], [150, 1450, 2650], Wood),
        // ---- The pit.
        // The west gantry, jut to jut across the pit sixteen metres west of
        // the point, and its pier in the middle.
        Solid::cm([-1800, 590, -1450], [-1500, 650, 1450], Wood),
        Solid::cm([-1800, 0, -150], [-1500, 590, 150], Stone),
        // A block left on the plinth: cover on the point.
        Solid::cm([-375, 45, -475], [-125, 185, -225], Stone),
        // Cut blocks on the floor round the point, and a slab fallen.
        Solid::cm([-1350, 0, -1100], [-1050, 200, -700], Stone),
        Solid::cm([-1550, 0, 700], [-1250, 160, 1000], Stone),
        slab(-1900, -1000, (0, 150), (300, 80), (2, 1), Rock),
        // A great cut block out on the floor, past a reach: cover across the
        // way in, and a perch caught from the floor.
        Solid::cm([-3400, 0, -400], [-3000, 450, 400], Stone),
        // The tor by the west start, and a low wall across it: cover coming
        // out.
        round(-4800, -1200, (0, 500), (250, 150), Rock),
        Solid::cm([-5100, 0, -600], [-4950, 150, 600], Rock),
    ],
);

/// **The hoist** on each jut: a cage from the lower bench to the upper,
/// two and a half seconds waiting at each, three seconds on the way. Both
/// on one clock, so neither side's is ever the one waiting.
const fn hoist(solid: Solid) -> Drift {
    Drift {
        solid,
        travel: V3::new(Fx::ZERO, cm(800), Fx::ZERO),
        wait: 150,
        go: 180,
        stay: 150,
        back: 180,
        phase: 0,
    }
}

/// The north jut's cage at its foot, its floor flush with the lower bench,
/// against the upper bench's face. It is a timber stack the whole height of
/// its rise, sunk in the bench at its foot: a thin floor would let a body
/// falling onto it as it rises pass through it, and there is never a gap
/// under it to be caught in.
const CAGE: Solid = Solid::cm([750, -200, 2250], [1150, 650, 2600], Wood);

const QUARRY_DRIFTS: [Drift; 2] = [hoist(CAGE), hoist(turned(CAGE))];

/// A column of air rising from a bench at `from` (centimetres).
const fn draft(x: i32, z: i32, radius: i32, from: i32, top: i32) -> Vent {
    Vent {
        at: (cm(x), cm(z)),
        radius: cm(radius),
        top: cm(top),
        from: cm(from),
    }
}

const QUARRY_VENTS: [Vent; 4] = [
    // Up each jut's face from the pit, west of the slot (east of it on the
    // south jut): out of the point onto the lower bench.
    Vent::cm(-1000, 1250, 150, 800),
    Vent::cm(1000, -1250, 150, 800),
    // Up the upper bench's face from the lower, above each spoil ramp.
    draft(3600, 3700, 150, 650, 1600),
    draft(-3600, -3700, 150, 650, 1600),
];

/// Every way is the west side's; the east side's are their turn.
const QUARRY_WAYS: [Way; 12] = [
    Way::new(
        "the hoist, from the lower bench to the upper",
        (950, 650, 1900),
        &[(950, 1450, 2420), (950, 1450, 3000)],
    ),
    Way::new(
        "the open floor, from the start onto the point",
        (-5800, 0, 0),
        &[(-5000, 0, -900), (-2400, 0, -700), (0, 45, 0)],
    ),
    Way::new(
        "the high road: the steps, the lower bench, the jut, down onto the point",
        (-5400, 0, 2000),
        &[
            (-5400, 220, 2800),
            (-5000, 440, 2800),
            (-5000, 650, 3150),
            (-2200, 650, 3300),
            (-2200, 650, 2500),
            (-1000, 650, 2400),
            (-600, 650, 1600),
            (0, 45, 0),
        ],
    ),
    Way::new(
        "the cutting and the slot, onto the point",
        (-3000, 0, 2000),
        &[(-2200, 0, 2000), (0, 0, 2000), (0, 0, 1550), (0, 45, 0)],
    ),
    Way::new(
        "the steps from the lower bench to the upper",
        (-4200, 650, 3200),
        &[(-4600, 920, 3600), (-5000, 1190, 3600), (-5000, 1450, 4200)],
    ),
    Way::new(
        "the updraft up the jut's face from the pit",
        (-1000, 0, 800),
        &[(-1000, 650, 1700)],
    ),
    Way::new(
        "the ledge on the jut's face from the pit",
        (1200, 0, 600),
        &[(1200, 220, 1250), (1200, 650, 1700)],
    ),
    Way::new(
        "out along the crane jib",
        (0, 1450, 3200),
        &[(0, 1450, 1600)],
    ),
    Way::new(
        "the west gantry, jut to jut over the pit",
        (-1650, 650, 1700),
        &[(-1650, 650, -1700)],
    ),
    Way::new(
        "the spoil ramp up the south face",
        (-4410, 0, -1600),
        &[(-4410, 650, -3300)],
    ),
    Way::new(
        "the updraft up the south face's upper bench",
        (-3600, 650, -3200),
        &[(-3600, 1450, -4200)],
    ),
    Way::new(
        "out of the point by the cutting, past a reach",
        (0, 45, 0),
        &[
            (0, 0, 1600),
            (0, 0, 2000),
            (-2200, 0, 2000),
            (-3500, 0, 2000),
        ],
    ),
];

/// The floor's relief (`arena::relief`): smooth hills and hollows, no boxes.
/// Kept clear of the point, the starts and the rim. Each with its turn.
pub const BUMPS: [Bump; 4] = [
    // The spoil ramps: tipped against the lower bench's face, their crowns
    // at its lip, so the slope walks up onto it. Off the metre grid's
    // middles by 10 cm, so no four square metres of slope share one height
    // and read as a level (`sim::fightspace`).
    Bump::cm(4410, 3000, 1300, 650),
    Bump::cm(-4410, -3000, 1300, 650),
    // Spoil heaps on the open floor, past a reach.
    Bump::cm(-2800, -1800, 700, 250),
    Bump::cm(2800, 1800, 700, 250),
];

// The tables a stretched map is made from (`super::sizes`).
pub(super) const SOLIDS: [Solid; 49] = QUARRY_SOLIDS;
pub(super) const REGIONS: [Region; 9] = QUARRY_REGIONS;
pub(super) const VENTS: [crate::valley::Vent; 4] = QUARRY_VENTS;
pub(super) const DRIFTS: [crate::drift::Drift; 2] = QUARRY_DRIFTS;
