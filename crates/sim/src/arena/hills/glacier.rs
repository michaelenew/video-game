//! **The Glacier**, a King of the Hill map (`super`, `crate::versus`,
//! `docs/design/versus.md` §3, `docs/design/fight-spaces.md`).
//!
//! Rebuilt 2026-11-02 to the fight-space rules: 128 by 92 m, the starts 58 m
//! (2.3 reaches) from the point, three levels -- the snow, the snout's
//! shelf at 8 m, its ridge at 16 m -- and five things that move you: the
//! moulin, the geyser under the point, the sentinel's updraft, the calving
//! lift and the floe.

use super::super::relief::Bump;
use super::super::rim::Rim;
use super::super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use super::{round, slab, turned, whole};
use crate::drift::Drift;
use crate::math::V3;
use crate::valley::Vent;
use crate::versus::{Hill, Kind, Stage, Way};

use Material::{Rock, Snow};

/// **The Glacier**: a snowfield under a glacier's snout.
///
/// The point is on **the dome**, a hump of old ice four metres sixty up in
/// the middle, climbed by a stair of ice at one corner of each side's face
/// and a ledge in the middle of the north and south faces. Under it each
/// side has **a melt cave**, a grotto four metres tall running in from the
/// dome's end and ending under a hole in the dome's top beside the point,
/// where the meltwater's breath (**the geyser**) throws whoever stands in it
/// up onto the point. Each side's own high ground is **the snout**, the
/// glacier's front along the rim on its left: **a shelf** eight metres up,
/// reached by a stair from the start, a ledge to catch at its front and a
/// lift of ice at its end; behind it **the ridge** sixteen metres up,
/// reached by a stair on the shelf and by **the moulin**, a shaft through
/// the shelf breathing an updraft to the ridge's lip. From the shelf's end
/// **an ice bridge** crosses over the open floor to **the gate serac**, and
/// **a floe** slides out from it over the dome's flank and back. On the
/// other hand, round the dome, **seracs**: the gate and its twin a chimney
/// apart, the sentinel with its shoulder and its updraft, a leaning one,
/// and blocks and boulders on the long approach.
pub static GLACIER: Arena = Arena {
    id: ArenaId::GLACIER,
    name: "Glacier",
    creature: None,
    bounds: Bounds::cm((-6400, 6400), (-4600, 4600)),
    floor: Snow,
    regions: &GLACIER_REGIONS,
    solids: &GLACIER_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-5800, 0, (0, 0)), Mark::cm(5800, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    // Nothing stands against the rim: a lane of snow runs behind the snout
    // and the stairs, so the bank starts at the floor on every side. It is
    // tall, the valley's walls: the moulin throws a body well over the
    // ridge, and the crest is over all of it.
    rim: Some(Rim::new((-6400, 6400), (-4600, 4600), [0, 0, 0, 0], 2600)),
};

pub static GLACIER_STAGE: Stage = Stage {
    arena: ArenaId::GLACIER,
    kind: Kind::Hill,
    theme: "A snowfield under a glacier's snout: the point on an ice dome, caves beneath",
    pit: None,
    hill: Some(Hill::cm(0, 460, 0, 400, 50, 1000)),
    vents: &GLACIER_VENTS,
    drifts: &GLACIER_DRIFTS,
    ways: &GLACIER_WAYS,
};

const GLACIER_REGIONS: [Region; 6] = [
    // Each start's yard, trodden down to the rock.
    Region {
        area: Area::disc_cm(-5800, 0, 600),
        material: Rock,
    },
    Region {
        area: Area::disc_cm(5800, 0, 600),
        material: Rock,
    },
    // Wind-scoured rock where the snow has blown off the approach.
    Region {
        area: Area::disc_cm(-3000, -600, 500),
        material: Rock,
    },
    Region {
        area: Area::disc_cm(3000, 600, 500),
        material: Rock,
    },
    // Moraine round the boulder out past the leaning serac.
    Region {
        area: Area::disc_cm(-3400, -3000, 700),
        material: Rock,
    },
    Region {
        area: Area::disc_cm(3400, 3000, 700),
        material: Rock,
    },
];

const GLACIER_SOLIDS: [Solid; 49] = whole(
    [
        // The plug of ice between the two melt caves, under the point's
        // middle: the dome's top at 4.6 m.
        Solid::cm([-300, 0, -250], [300, 460, 250], Snow),
    ],
    [
        // -- The dome --
        // The west melt cave's roof, from the dome's end to the geyser's
        // hole (x -500..-300, open to the sky): four metres of headroom,
        // four wide.
        Solid::cm([-1100, 400, -250], [-500, 460, 250], Snow),
        // The dome's north half, 22 m long.
        Solid::cm([-1100, 0, 200], [1100, 460, 900], Snow),
        // Its stair, at the west end of the north face (a step of 2.3 m),
        // and a ledge in the middle of that face.
        Solid::cm([-1500, 0, 400], [-1100, 230, 900], Snow),
        Solid::cm([-250, 0, 900], [250, 230, 1200], Snow),
        // -- The snout, the west side's high ground, along the north --
        // The shelf, eight metres up, in two pieces either side of the
        // moulin (x -3000..-2600), which is open from the floor to the sky.
        Solid::cm([-5000, 0, 1600], [-3000, 800, 3000], Snow),
        Solid::cm([-2600, 0, 1400], [-1400, 800, 3000], Snow),
        // The ridge behind it, sixteen metres up, with a lane of snow
        // behind it against the rim.
        Solid::cm([-4600, 0, 3000], [-2400, 1600, 4000], Snow),
        // The stair from the start to the shelf: 2.7 m, 5.4 m, then the
        // shelf's 8.
        Solid::cm([-5800, 0, 1400], [-5400, 270, 2000], Rock),
        Solid::cm([-5800, 0, 2000], [-5000, 540, 2800], Rock),
        // The stair on the shelf to the ridge: 10.7 m, 13.4 m, then 16.
        Solid::cm([-4400, 0, 2400], [-4000, 1070, 3000], Snow),
        Solid::cm([-4000, 0, 2600], [-3600, 1340, 3000], Snow),
        // The cornice: a block of ice at the shelf's front, five metres up,
        // a ledge to catch from the snow and a step to the shelf.
        Solid::cm([-1800, 0, 1000], [-1400, 500, 1400], Snow),
        // -- Over the open floor --
        // The ice bridge from the shelf's front to the gate serac: over the
        // approach at 7.2 m, its deck flush with the shelf.
        Solid::cm([-2200, 720, -800], [-1800, 800, 1400], Snow),
        // The gate serac, eight metres, the bridge's far end.
        Solid::cm([-2300, 0, -1400], [-1700, 800, -800], Snow),
        // Its twin, eleven metres, a chimney of two metres apart: kick up
        // between them.
        Solid::cm([-2300, 0, -1900], [-1700, 1100, -1600], Snow),
        // A step down from the gate toward the dome, 5.6 m.
        Solid::cm([-1700, 0, -1300], [-1300, 560, -900], Snow),
        // A boulder beside the cave's mouth.
        round(-1300, -600, (0, 250), (150, 110), Rock),
        // A stub of a serac north of the dome, under the floe's path.
        Solid::cm([-1000, 0, 1500], [-600, 300, 1900], Snow),
        // -- The seracs south of the dome --
        // The sentinel, nine metres, its shoulder at five (a ledge to catch,
        // then another), and an updraft up its west face.
        Solid::cm([-1000, 0, -2200], [-600, 900, -1800], Snow),
        Solid::cm([-1000, 0, -1800], [-600, 500, -1400], Snow),
        // The leaning serac, out of reach: eleven metres of ice on a slant.
        slab(-2600, -2700, (0, 1100), (250, 150), (1, 2), Snow),
        // -- The long approach --
        // A block of ice and a boulder of the moraine, cover out of reach.
        Solid::cm([-4200, 0, -2000], [-3800, 550, -1600], Snow),
        round(-3400, -3000, (0, 350), (300, 200), Rock),
        // A wall of packed snow in front of the start.
        Solid::cm([-5000, 0, -1400], [-4800, 250, -400], Snow),
    ],
);

/// **The updrafts**: the moulin up the ridge's face, the geyser up through
/// the hole beside the point, the sentinel's up its face. Each side's, and
/// the other's turned.
const GLACIER_VENTS: [Vent; 6] = [
    Vent::cm(-2800, 2850, 150, 1700),
    Vent::cm(2800, -2850, 150, 1700),
    Vent::cm(-400, 0, 100, 600),
    Vent::cm(400, 0, 100, 600),
    Vent::cm(-1090, -2000, 120, 1000),
    Vent::cm(1090, 2000, 120, 1000),
];

const fn moving(solid: Solid, travel: (i32, i32, i32), clock: (u16, u16, u16, u16)) -> Drift {
    Drift {
        solid,
        travel: V3::new(
            super::super::cm(travel.0),
            super::super::cm(travel.1),
            super::super::cm(travel.2),
        ),
        wait: clock.0,
        go: clock.1,
        stay: clock.2,
        back: clock.3,
        phase: 0,
    }
}

/// The calving lift, a slab of ice at the shelf's east end that rises from
/// the snow to the shelf and sinks again.
const LIFT: Solid = Solid::cm([-1400, 0, 2400], [-1000, 60, 2900], Snow);
const LIFT_CLOCK: (u16, u16, u16, u16) = (120, 180, 120, 180);

/// The floe, a slab of ice docked at the shelf's east end at its height,
/// that slides out over the dome's flank and back.
const FLOE: Solid = Solid::cm([-1400, 740, 1600], [-1000, 800, 2000], Snow);
const FLOE_CLOCK: (u16, u16, u16, u16) = (180, 240, 120, 240);

const GLACIER_DRIFTS: [Drift; 4] = [
    moving(LIFT, (0, 740, 0), LIFT_CLOCK),
    moving(turned(LIFT), (0, 740, 0), LIFT_CLOCK),
    moving(FLOE, (600, 0, -1200), FLOE_CLOCK),
    moving(turned(FLOE), (-600, 0, 1200), FLOE_CLOCK),
];

/// The west side's ways (the east's are the same turned), the lift and the
/// floe among them.
const GLACIER_WAYS: [Way; 12] = [
    Way::new(
        "the calving lift to the shelf",
        (-1200, 0, 3400),
        &[(-1200, 60, 2650), (-1900, 800, 2650)],
    ),
    Way::new(
        "onto the floe at its dock",
        (-1900, 800, 1800),
        &[(-1200, 800, 1800)],
    ),
    Way::new(
        "the open floor to the dome's stair, onto the point",
        (-5200, 0, 200),
        &[(-1300, 230, 650), (-200, 460, 300)],
    ),
    Way::new(
        "the dome's north ledge, onto the point",
        (-200, 0, 2000),
        &[(0, 230, 1050), (0, 460, 300)],
    ),
    Way::new(
        "the melt cave and the geyser, onto the point (hidden)",
        (-2000, 0, 0),
        &[(-400, 0, 0), (-100, 460, 0)],
    ),
    Way::new(
        "the high road: the shelf, the bridge, the gate serac, onto the point",
        (-2000, 800, 2000),
        &[(-2000, 800, -1100), (0, 460, -300)],
    ),
    Way::new(
        "the stair from the start to the shelf",
        (-5600, 0, 900),
        &[(-5600, 270, 1700), (-5500, 540, 2400), (-4600, 800, 2300)],
    ),
    Way::new(
        "the cornice, a ledge to catch, to the shelf",
        (-1600, 0, 500),
        &[(-1600, 500, 1200), (-1800, 800, 2000)],
    ),
    Way::new(
        "the stair on the shelf to the ridge",
        (-4300, 800, 2000),
        &[
            (-4200, 1070, 2700),
            (-3800, 1340, 2800),
            (-3700, 1600, 3400),
        ],
    ),
    Way::new(
        "the moulin to the ridge",
        (-2800, 0, 1200),
        &[(-2800, 1600, 3300)],
    ),
    Way::new(
        "the sentinel's updraft",
        (-1700, 0, -2100),
        &[(-800, 900, -2000)],
    ),
    Way::new(
        "the sentinel's shoulder, two ledges to catch",
        (-800, 0, -1100),
        &[(-800, 500, -1600), (-800, 900, -2000)],
    ),
];

/// The floor's relief (`arena::relief`): drifts of snow on the approach and
/// crevasses in the corners, no boxes. Kept clear of the point, the starts,
/// the boxes and the rim.
pub const BUMPS: [Bump; 10] = [
    Bump::cm(-3800, -600, 700, 200),
    Bump::cm(3800, 600, 700, 200),
    Bump::cm(-3500, 1000, 600, 150),
    Bump::cm(3500, -1000, 600, 150),
    Bump::cm(-1000, -3200, 600, -250),
    Bump::cm(1000, 3200, 600, -250),
    Bump::cm(-300, 3600, 600, -250),
    Bump::cm(300, -3600, 600, -250),
    Bump::cm(-5000, -3200, 700, -200),
    Bump::cm(5000, 3200, 700, -200),
];

// The tables a stretched map is made from (`super::sizes`).
pub(super) const SOLIDS: [Solid; 49] = GLACIER_SOLIDS;
pub(super) const REGIONS: [Region; 6] = GLACIER_REGIONS;
pub(super) const VENTS: [crate::valley::Vent; 6] = GLACIER_VENTS;
pub(super) const DRIFTS: [crate::drift::Drift; 4] = GLACIER_DRIFTS;
