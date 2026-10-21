//! **King of the Hill maps** (2026-10-21, `crate::versus`,
//! `docs/design/versus.md`): one capture point in the middle, a side's
//! start at each end, and three or four ways from one to the other.
//!
//! Every map is **turned half a turn about its middle** -- (x, z) to
//! (-x, -z) -- so each side has exactly what the other has: a table here is
//! the middle's own solids and one half's, and [`whole`] makes the other
//! half by turning it. A ledge west of the point for the west side is a
//! ledge east of it for the east side.
//!
//! What each map is built to have, from Team Fortress 2's King of the Hill
//! maps and nothing copied from any one of them:
//!
//! - **High ground over the point**, close enough to fight down from and
//!   reached by steps a jump each, so the point is contested from above.
//! - **A way in nobody sees**: a tunnel or a covered passage, the one kind
//!   of confined space on the map, for getting onto the point from where
//!   the defenders are not looking.
//! - **Open air everywhere else**: from either start to the point and to
//!   every high place there is a way that never goes under a roof, through
//!   at most one or two narrow places.
//! - **No buildings**: the cover is rocks, blocks, stones and banks.
//!
//! Every climb on a route is at most a jump the shortest-jumping class makes
//! with room to spare (2.7 m at the top of a held jump), so nobody is shut
//! out of high ground in a team game -- a different thing from a movement
//! section, which these are not. The edge is a rim, as every fight's is.

use super::rim::Rim;
use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use crate::fixed::Fx;
use crate::math::V3;
use crate::versus::{Hill, Kind, Stage};

use Material::{Ash, Grass, Ground, Peat, Rock, Sand, Snow, Stone, Water, Wood};

/// A solid turned half a turn about the middle: (x, z) to (-x, -z). A round
/// solid is round still, and a turned box lies along the same line.
pub(super) const fn turned(s: Solid) -> Solid {
    Solid {
        min: V3::new(s.max.x.neg(), s.min.y, s.max.z.neg()),
        max: V3::new(s.min.x.neg(), s.max.y, s.min.z.neg()),
        material: s.material,
        shape: s.shape,
    }
}

/// **A whole map from its middle and one half**: the middle's solids, the
/// half, and the half turned. `N` must be `C + 2 H`.
pub(super) const fn whole<const C: usize, const H: usize, const N: usize>(
    centre: [Solid; C],
    half: [Solid; H],
) -> [Solid; N] {
    assert!(N == C + 2 * H, "a map is its middle and two halves");
    let mut out = [half[0]; N];
    let mut i = 0;
    while i < C {
        out[i] = centre[i];
        i += 1;
    }
    let mut k = 0;
    while k < H {
        out[C + k] = half[k];
        out[C + H + k] = turned(half[k]);
        k += 1;
    }
    out
}

/// A round solid from centimetres: centred on (x, z), from `y0` to `y1`,
/// `bottom` and `top` across (radii).
const fn round(x: i32, z: i32, y: (i32, i32), r: (i32, i32), material: Material) -> Solid {
    let (bottom, top) = r;
    let big = if bottom > top { bottom } else { top };
    Solid {
        min: V3::new(super::cm(x - big), super::cm(y.0), super::cm(z - big)),
        max: V3::new(super::cm(x + big), super::cm(y.1), super::cm(z + big)),
        material,
        shape: super::Shape::Round {
            bottom: super::cm(bottom),
            top: super::cm(top),
            hollow: Fx::ZERO,
        },
    }
}

/// **A slab lying along a direction**: a box `half.0` long either way along
/// (dx, dz) and `half.1` either way across it, centred on (x, z), from `y.0`
/// to `y.1` (centimetres) -- a fallen block, a broken arch.
pub(super) const fn slab(
    x: i32,
    z: i32,
    y: (i32, i32),
    half: (i32, i32),
    along: (i32, i32),
    material: Material,
) -> Solid {
    let d = V3::new(Fx::from_int(along.0), Fx::ZERO, Fx::from_int(along.1)).normalized();
    let (c, s) = (d.x, d.z);
    let (long, wide) = (super::cm(half.0), super::cm(half.1));
    let ex = long.mul(c.abs()).add(wide.mul(s.abs()));
    let ez = long.mul(s.abs()).add(wide.mul(c.abs()));
    let (cx, cz) = (super::cm(x), super::cm(z));
    Solid {
        min: V3::new(cx.sub(ex), super::cm(y.0), cz.sub(ez)),
        max: V3::new(cx.add(ex), super::cm(y.1), cz.add(ez)),
        material,
        shape: super::Shape::Turned {
            half: long,
            half_wide: wide,
            cos: c,
            sin: s,
        },
    }
}

// ---------------------------------------------------------------------------
// The Quarry
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// The Glacier
// ---------------------------------------------------------------------------

/// **The Glacier**: a snowfield under a glacier's snout. The point is on
/// **the dome**, a hump of old ice three metres up in the middle, climbed by
/// a stair of ice at one end of each side's face and a ledge in the middle
/// of the north and south faces. Under the dome runs **the melt cave**, east
/// to west, a sneak from one side's half of the field to the other's
/// beneath the defenders' feet. Each side's own high ground is **an ice
/// shelf** five and a half metres up along the rim on its left, climbed from
/// its start, whose edge looks down on the dome across a gap you can drop
/// from -- and under each shelf another cave runs from the start out to the
/// dome's foot. Seracs, boulders and blocks of ice stand about the open
/// field for cover.
pub static GLACIER: Arena = Arena {
    id: ArenaId::GLACIER,
    name: "Glacier",
    creature: None,
    bounds: Bounds::cm((-3800, 3800), (-2800, 2800)),
    floor: Snow,
    regions: &GLACIER_REGIONS,
    solids: &GLACIER_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-3300, -400, (0, 0)), Mark::cm(3300, 400, (0, 0))],
        hunt: None,
    },
    sites: &[],
    // The bank starts at the shelves' height on the north and south, whose
    // backs are against it: walking off a shelf's back is walking onto the
    // bank, not into a crease under it.
    rim: Some(Rim::new(
        (-3800, 3800),
        (-2800, 2800),
        [550, 550, 0, 0],
        1100,
    )),
};

pub static GLACIER_STAGE: Stage = Stage {
    arena: ArenaId::GLACIER,
    kind: Kind::Hill,
    theme: "A snowfield under a glacier: the point on an ice dome, caves beneath",
    pit: None,
    hill: Some(Hill::cm(0, 300, 0, 400, 50, 1000)),
};

const GLACIER_REGIONS: [Region; 4] = [
    // Wind-scoured rock where the snow has blown off.
    Region {
        area: Area::disc_cm(-2000, 800, 500),
        material: Rock,
    },
    Region {
        area: Area::disc_cm(2000, -800, 500),
        material: Rock,
    },
    // Each start's yard, trodden down to the rock.
    Region {
        area: Area::disc_cm(-3300, -400, 500),
        material: Rock,
    },
    Region {
        area: Area::disc_cm(3300, 400, 500),
        material: Rock,
    },
];

const GLACIER_SOLIDS: [Solid; 27] = whole(
    [
        // The melt cave's roof under the point: two metres twenty of
        // headroom, three wide, fourteen long.
        Solid::cm([-700, 220, -150], [700, 300, 150], Snow),
    ],
    [
        // The dome's north half, three metres up.
        Solid::cm([-700, 0, 150], [700, 300, 900], Snow),
        // Its stair at the west end of the north face, and a ledge in the
        // middle of that face.
        Solid::cm([-1100, 0, 450], [-700, 150, 900], Snow),
        Solid::cm([-250, 0, 900], [250, 150, 1150], Snow),
        // The west side's ice shelf along the north rim, five and a half
        // metres up, with a cave the length of it at its foot: the shelf's
        // front, the cave's roof, the shelf's back.
        Solid::cm([-3000, 0, 1500], [-1200, 550, 1800], Snow),
        Solid::cm([-3000, 300, 1800], [-1200, 550, 2100], Snow),
        Solid::cm([-3000, 0, 2100], [-1200, 550, 2800], Snow),
        // Its stair up from the west start.
        Solid::cm([-3500, 0, 1200], [-3000, 180, 1500], Rock),
        Solid::cm([-3300, 0, 1200], [-3000, 360, 1500], Rock),
        // A serac in the west half of the field: tall, thin, cover.
        round(-1800, -400, (0, 600), (100, 70), Snow),
        // Blocks of ice and a boulder.
        Solid::cm([-2400, 0, -1400], [-2100, 140, -900], Snow),
        Solid::cm([-1500, 0, 200], [-1300, 180, 600], Snow),
        round(-900, -1800, (0, 200), (130, 90), Rock),
        // A low wall of packed snow across the west start.
        Solid::cm([-2900, 0, -900], [-2750, 130, 100], Snow),
    ],
);

// ---------------------------------------------------------------------------
// The Delta
// ---------------------------------------------------------------------------

/// **The Delta**: a river's mouth among reeds, the point on a low island
/// ringed by four standing stones. Each side's half is crossed by **a
/// dyke**, an old earth bank three metres high from rim to rim but for a
/// lane at each end and **the breach** in its middle -- the map's one
/// choke, straight at the point. Its south half hides **the sluice**, a
/// covered channel inside it from the south lane to the breach: the way
/// onto the point nobody sees. Over the north lane runs a broken
/// **aqueduct**, the side's high road, five and a half metres up from its
/// start to a broken end over the point's north flank. Steps climb the dyke
/// from either face; reeds and a fallen arch are cover.
pub static DELTA: Arena = Arena {
    id: ArenaId::DELTA,
    name: "Delta",
    creature: None,
    bounds: Bounds::cm((-4000, 4000), (-2800, 2800)),
    floor: Grass,
    regions: &DELTA_REGIONS,
    solids: &DELTA_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-3600, 0, (0, 0)), Mark::cm(3600, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    rim: Some(Rim::new((-4000, 4000), (-2800, 2800), [0, 0, 0, 0], 900)),
};

pub static DELTA_STAGE: Stage = Stage {
    arena: ArenaId::DELTA,
    kind: Kind::Hill,
    theme: "A river mouth among reeds: dykes, a hidden sluice, a broken aqueduct",
    pit: None,
    hill: Some(Hill::cm(0, 45, 0, 400, 50, 1000)),
};

const DELTA_REGIONS: [Region; 7] = [
    // The river's channels, south of the west dyke and north of the east.
    Region {
        area: Area::rect_cm((-1300, 1300), (-2800, -2450)),
        material: Water,
    },
    Region {
        area: Area::rect_cm((-1300, 1300), (2450, 2800)),
        material: Water,
    },
    // Pools among the reeds.
    Region {
        area: Area::disc_cm(-2600, 700, 300),
        material: Water,
    },
    Region {
        area: Area::disc_cm(2600, -700, 300),
        material: Water,
    },
    // Peat round the point's island and the starts.
    Region {
        area: Area::disc_cm(0, 0, 650),
        material: Peat,
    },
    Region {
        area: Area::disc_cm(-3600, 0, 450),
        material: Ash,
    },
    Region {
        area: Area::disc_cm(3600, 0, 450),
        material: Ash,
    },
];

const DELTA_SOLIDS: [Solid; 33] = whole(
    [
        // The point's island: a low mound, walked up.
        round(0, 0, (0, 45), (500, 450), Grass),
    ],
    [
        // Two of the four standing stones round the point.
        round(600, 450, (0, 280), (50, 40), Stone),
        round(-600, 450, (0, 280), (50, 40), Stone),
        // The west dyke, three metres high: its north length, and its south
        // length in three -- two banks and the roof of the sluice between
        // them, two metres wide and two twenty high -- with the breach
        // between, and a lane at each end before the rim.
        Solid::cm([-1900, 0, 350], [-1300, 300, 2400], Ground),
        Solid::cm([-1900, 0, -2400], [-1700, 300, -350], Ground),
        Solid::cm([-1500, 0, -2400], [-1300, 300, -350], Ground),
        Solid::cm([-1700, 220, -2400], [-1500, 300, -350], Ground),
        // Steps up the dyke: from the start's side on the north length, and
        // from the point's side on the south.
        Solid::cm([-2200, 0, 800], [-1900, 150, 1200], Ground),
        Solid::cm([-1300, 0, -1300], [-1000, 150, -900], Ground),
        // The aqueduct's deck, five metres up to its underside, from the
        // west start to a broken end over the point's north flank, and the
        // two piers it stands on.
        Solid::cm([-3000, 500, 1300], [500, 570, 1700], Stone),
        round(-2400, 1500, (0, 500), (60, 60), Stone),
        round(-600, 1500, (0, 500), (60, 60), Stone),
        // Its stair at the start.
        Solid::cm([-3400, 0, 1300], [-3000, 180, 1700], Stone),
        Solid::cm([-3200, 0, 1300], [-3000, 360, 1700], Stone),
        // A hummock of reeds; a low wall across the start; a fallen arch.
        round(-2600, -1200, (0, 120), (150, 110), Peat),
        Solid::cm([-3300, 0, -600], [-3150, 130, 600], Rock),
        slab(-800, -1500, (0, 250), (250, 40), (10, 1), Stone),
    ],
);

// ---------------------------------------------------------------------------
// The Gorge
// ---------------------------------------------------------------------------

/// **The Gorge**: a river gorge twenty metres wide between cliffs seven
/// metres high, the two sides' starts at its two ends. The point is on a
/// gravel bar in the middle of the river, and **a plank bridge** crosses the
/// gorge straight over it, four body heights up, from the north cliff-top to
/// the south: a walkway to fight down onto the point from, and to be knocked
/// off. Each cliff-top is a long terrace of open high ground, climbed out of
/// the gorge by two ledges at each end of it. Inside the north cliff runs
/// **the cave**, in from a bay by the west side's start and out through a
/// mouth in the cliff beside the point: the way onto the point nobody sees
/// (the south cliff's, turned, is the east side's). The gorge floor is open,
/// with boulders for cover; the river runs down it.
pub static GORGE: Arena = Arena {
    id: ArenaId::GORGE,
    name: "Gorge",
    creature: None,
    bounds: Bounds::cm((-3800, 3800), (-2600, 2600)),
    floor: Rock,
    regions: &GORGE_REGIONS,
    solids: &GORGE_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-3300, 0, (0, 0)), Mark::cm(3300, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    // The bank starts at the cliff-tops' height all round: at the gorge's
    // two ends that is a wall seven metres up from the river, and walking
    // off a cliff-top's back or end is walking onto the bank.
    rim: Some(Rim::new(
        (-3800, 3800),
        (-2600, 2600),
        [700, 700, 700, 700],
        1000,
    )),
};

pub static GORGE_STAGE: Stage = Stage {
    arena: ArenaId::GORGE,
    kind: Kind::Hill,
    theme: "A river gorge: the point on a gravel bar, a plank bridge high over it",
    pit: None,
    // Room up to just under the bridge's planks, 6.55 m over the bar:
    // anybody in the air under them counts, nobody standing on them.
    hill: Some(Hill::cm(0, 45, 0, 400, 50, 640)),
};

const GORGE_REGIONS: [Region; 3] = [
    // The river, either side of the bar.
    Region {
        area: Area::rect_cm((-3800, -600), (-150, 150)),
        material: Water,
    },
    Region {
        area: Area::rect_cm((600, 3800), (-150, 150)),
        material: Water,
    },
    // The gravel bar.
    Region {
        area: Area::disc_cm(0, 0, 650),
        material: Sand,
    },
];

const GORGE_SOLIDS: [Solid; 30] = whole(
    [
        // The bar the point is on, a step up, walked.
        round(0, 0, (0, 45), (500, 450), Sand),
        // The bridge, straight over the point: planks four metres wide,
        // their underside 6.3 m over the bar, flush with both cliff-tops.
        Solid::cm([-200, 630, -1000], [200, 700, 1000], Wood),
    ],
    [
        // The north cliff, seven metres: its face west of the cave's mouth,
        // the mouth's lintel, the cave's roof, the cliff east of the cave,
        // and its back to the rim. West of the face, a bay by the west start
        // that the cave runs in from.
        Solid::cm([-3000, 0, 1000], [-300, 700, 1300], Rock),
        Solid::cm([-300, 300, 1000], [0, 700, 1300], Rock),
        Solid::cm([-3000, 300, 1300], [0, 700, 1600], Rock),
        Solid::cm([0, 0, 1000], [3800, 700, 1600], Rock),
        Solid::cm([-3800, 0, 1600], [3800, 700, 2600], Rock),
        // Two ledges up the north cliff's face at its west end, and two at
        // its east end.
        Solid::cm([-2600, 0, 700], [-2000, 230, 1000], Rock),
        Solid::cm([-2600, 0, 850], [-2000, 470, 1000], Rock),
        Solid::cm([1200, 0, 700], [1800, 230, 1000], Rock),
        Solid::cm([1200, 0, 850], [1800, 470, 1000], Rock),
        // Boulders in the gorge.
        round(-1800, -450, (0, 160), (120, 90), Rock),
        round(-2800, 550, (0, 120), (100, 80), Rock),
        Solid::cm([-1200, 0, 300], [-900, 160, 700], Rock),
        // Boulders on the north cliff-top.
        round(-1500, 1900, (700, 900), (120, 90), Rock),
        round(800, 2100, (700, 880), (100, 80), Rock),
    ],
);

// ---------------------------------------------------------------------------
// The Henge
// ---------------------------------------------------------------------------

/// **The Henge**: a giant's stone circle on a grass down. Its great stones
/// and lintels are rough rock, not dressed masonry: drawn as masonry, block by
/// block, seven-metre stones were most of the map's cost to draw. Four standing
/// stones seven metres tall stand nine metres out from the point, north,
/// south, east and west, and **two lintels** cross between them over it in
/// a cross, four body heights up: a walkway over the point from every side,
/// to drop onto it from or to be knocked off. Each stone is climbed by a
/// stair of three blocks on its outer face; the west stone is the west
/// side's, the east the east side's, the north and south nobody's. Out by
/// each start is **a long barrow**, an earth mound with a passage through it
/// that comes out beside the south (or north) stone: the way in nobody sees.
/// A ring of smaller stones stands about the circle for cover.
pub static HENGE: Arena = Arena {
    id: ArenaId::HENGE,
    name: "Henge",
    creature: None,
    bounds: Bounds::cm((-3800, 3800), (-2800, 2800)),
    floor: Grass,
    regions: &HENGE_REGIONS,
    solids: &HENGE_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-3400, 0, (0, 0)), Mark::cm(3400, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    rim: Some(Rim::new((-3800, 3800), (-2800, 2800), [0, 0, 0, 0], 900)),
};

pub static HENGE_STAGE: Stage = Stage {
    arena: ArenaId::HENGE,
    kind: Kind::Hill,
    theme: "A giant's stone circle: lintels crossing high over the point",
    pit: None,
    // Room up to just under the lintels' tops, 6.55 m over the altar:
    // anybody in the air under them counts, nobody standing on them.
    hill: Some(Hill::cm(0, 45, 0, 400, 50, 640)),
};

const HENGE_REGIONS: [Region; 3] = [
    // Chalk worn bare inside the circle.
    Region {
        area: Area::disc_cm(0, 0, 800),
        material: Sand,
    },
    // Each start, trodden.
    Region {
        area: Area::disc_cm(-3400, 0, 450),
        material: Rock,
    },
    Region {
        area: Area::disc_cm(3400, 0, 450),
        material: Rock,
    },
];

const HENGE_SOLIDS: [Solid; 32] = whole(
    [
        // The altar stone the point is on, a step up, walked.
        round(0, 0, (0, 45), (500, 450), Stone),
        // The north-south lintel, over the point: its underside 6.3 m up,
        // flush with the north and south stones' tops.
        Solid::cm([-200, 630, -900], [200, 700, 900], Rock),
    ],
    [
        // The west stone, seven metres, and its stair.
        Solid::cm([-1300, 0, -200], [-900, 700, 200], Rock),
        Solid::cm([-1700, 0, -200], [-1300, 230, 200], Rock),
        Solid::cm([-1500, 0, -200], [-1300, 470, 200], Rock),
        // The west half of the east-west lintel, from the west stone to the
        // north-south one.
        Solid::cm([-900, 630, -200], [-200, 700, 200], Rock),
        // The north stone and its stair.
        Solid::cm([-200, 0, 900], [200, 700, 1300], Rock),
        Solid::cm([-200, 0, 1300], [200, 230, 1700], Rock),
        Solid::cm([-200, 0, 1300], [200, 470, 1500], Rock),
        // The west side's long barrow: two banks and the roof of the
        // passage between them, three metres wide, twelve long.
        Solid::cm([-3000, 0, -1600], [-1800, 300, -1450], Ground),
        Solid::cm([-3000, 0, -1150], [-1800, 300, -1000], Ground),
        Solid::cm([-3000, 220, -1450], [-1800, 300, -1150], Ground),
        // The ring of smaller stones.
        round(-1600, 600, (0, 300), (60, 50), Stone),
        round(-1600, -600, (0, 300), (60, 50), Stone),
        round(-600, 1600, (0, 300), (60, 50), Stone),
        round(600, 1600, (0, 300), (60, 50), Stone),
        // A low wall across the west start.
        Solid::cm([-3000, 0, -600], [-2850, 130, 600], Rock),
    ],
);
