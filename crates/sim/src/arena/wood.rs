//! **The Great Wood**: a grove of giant trees south of Hearth, the valley's
//! playground (`docs/design/exploration/0019_spire_wood_and_the_deep.md`
//! §2b, built 2026-10-15). Land, like every reach: a road out of the town's
//! east gate, over the river by a footbridge, south into a clearing four
//! hundred metres across, and in it fourteen trees **13 to 20 metres
//! across and 150 to 200 tall**.
//!
//! Sized for what the classes do, not for the plain jump: the owner,
//! 2026-10-15, "however big you're thinking of, probably triple that size --
//! all the developed classes are easily clearing about half of most jump
//! maps in a single go". Every tree is climbed in **three bands**:
//!
//! | Band | How high | Between branches | Who |
//! | --- | --- | --- | --- |
//! | **Walking** | the first 40 m | 2.2 m up, a metre across at the trunk | anybody, with the plain jump: the crags' rule |
//! | **Mechanic** | 40 m to 25 m under the crown | 6.5 m up, four metres across | one class mechanic a hop |
//! | **Expert** | the last 25 m, the crown, the bridges | 11 m up, a quarter turn; gaps of 25--40 m between trees | mechanics chained |
//!
//! A **branch** is a turned board (`Shape::Turned`) four to five metres wide
//! running out from the trunk, the boards spiralling up round it; a **trunk**
//! is round (`Shape::Round`), flared at its foot and tapering to its crown; a
//! **crown** is a disc of leaves twice and a half the trunk across, with a
//! cairn on it. **Bridges** run between neighbouring trees at sixty to a
//! hundred and forty metres: some meet, some stop short with a gap only the
//! expert band crosses. Cairns on the mechanic band every five branches and
//! on every crown: falls cost what the fall rule says (VALE-2, "fall damage
//! applies"), and a fall that ends you stands you on the last one you
//! touched.
//!
//! The boxes stand on the land, so they are made where the land is
//! ([`build`], from `valley::layout`); what is here is where each tree is.

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::land::{Point, at};
use crate::valley::{Gate, Kind, Place, Seam, Zone};

pub static ARENA: Arena = Arena {
    id: ArenaId::WOOD,
    name: "Great Wood",
    creature: None,
    bounds: Bounds::cm((-23000, 23000), (-57000, -14000)),
    floor: Material::Grass,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        // At the clearing's edge, facing in.
        versus: [
            Mark::cm(1200, -15500, (1200, -25000)),
            Mark::cm(1600, -15500, (1600, -25000)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

const SOLIDS: [Solid; 0] = [];

/// **The road**, centimetres: x, z, floor, half its width. It leaves the
/// Mouth's road just outside the east gate, crosses the river on a
/// footbridge ([`BRIDGE`]), passes the Spire's plaza under the town's south
/// wall (`super::spire`), and runs south into the clearing.
pub const WAY: [Point; 6] = [
    at(6000, 300, 0, 300),
    at(7000, -1200, 0, 300),
    at(7200, -4500, 0, 350),
    at(3000, -5800, 0, 400),
    at(1000, -9000, 0, 400),
    at(1500, -13500, 0, 600),
];

/// **The clearing**: a way short and wide, as a lake is, its floor level
/// for two hundred metres either side of its line. The trees stand in it.
pub const CLEARING: [Point; 3] = [
    at(1000, -16000, 0, 9000),
    at(0, -33000, 0, 21000),
    at(0, -40000, 0, 21000),
];

/// The footbridge over the river, where the road crosses it.
pub const BRIDGE: Solid = Solid::cm([6500, -40, -2900], [7700, 20, -600], Material::Wood);

/// **A giant**: where it stands on the map (centimetres), its trunk's radius
/// above the flare and its height to the crown's top.
#[derive(Clone, Copy, Debug)]
pub struct Giant {
    pub x: i32,
    pub z: i32,
    pub radius: i32,
    pub height: i32,
}

const fn giant(x: i32, z: i32, radius: i32, height: i32) -> Giant {
    Giant {
        x,
        z,
        radius,
        height,
    }
}

/// **The trees**: the Great Tree in the middle, six round it at about fifty
/// metres, seven further out at about a hundred and ten.
pub const GIANTS: [Giant; 14] = [
    giant(0, -35000, 1000, 20000),
    giant(5500, -35000, 800, 17000),
    giant(2750, -30240, 700, 16000),
    giant(-2750, -30240, 750, 15500),
    giant(-5500, -35000, 800, 17500),
    giant(-2750, -39760, 650, 15000),
    giant(2750, -39760, 700, 16500),
    giant(10625, -32153, 650, 15000),
    giant(4474, -24951, 700, 16000),
    giant(-5164, -25288, 750, 17000),
    giant(-10798, -32901, 650, 15500),
    giant(-8302, -42217, 700, 16000),
    giant(384, -45993, 800, 18000),
    giant(8899, -41466, 650, 15000),
];

/// **The bridges between trees**: from one to the other, the height its
/// first half leaves at, how much higher the far half is, and the gap left
/// between the two halves' ends (centimetres). A gap of nothing is a bridge
/// you walk across; the others are the expert band.
pub const SPANS: [(u8, u8, i32, i32, i32); 12] = [
    (0, 1, 6000, 0, 0),
    (0, 3, 8000, 300, 0),
    (0, 5, 10000, 0, 2500),
    (1, 2, 9000, 400, 0),
    (2, 8, 11000, 0, 3000),
    (3, 9, 7000, 500, 0),
    (4, 10, 12000, 0, 3500),
    (4, 5, 6500, 300, 0),
    (6, 13, 9500, 0, 2800),
    (6, 12, 13000, 600, 4000),
    (1, 7, 8500, 0, 0),
    (5, 11, 14000, 0, 3000),
];

/// The trees a vine hangs down, and how far up it goes: the slow way past
/// the walking band, for anybody.
const VINED: [usize; 4] = [0, 3, 6, 10];
const VINE_TOP: i32 = 6000;

/// The flare at a trunk's foot: wider by this share of its radius, over
/// this height.
const FLARE: i32 = 140;
const FLARE_HIGH: i32 = 800;
/// What a trunk tapers to at its top, a share of its radius in hundredths.
const TAPER: i32 = 55;

/// The trunk's radius at a height above its foot, centimetres.
pub const fn radius_at(g: &Giant, y: i32) -> i32 {
    if y < FLARE_HIGH {
        let wide = g.radius * FLARE / 100;
        wide - (wide - g.radius) * y / FLARE_HIGH
    } else {
        let top = g.radius * TAPER / 100;
        g.radius - (g.radius - top) * (y - FLARE_HIGH) / (g.height - FLARE_HIGH)
    }
}

/// A vine down a trunk's north side, from below its foot to [`VINE_TOP`].
const fn vine(g: &Giant) -> Zone {
    let out = g.radius * FLARE / 100 + 80;
    Zone::cm([g.x - 120, -2000, g.z], [g.x + 120, VINE_TOP, g.z + out])
}

/// The vines, for the movement that reads them.
pub const VINES: [Zone; 4] = [
    vine(&GIANTS[VINED[0]]),
    vine(&GIANTS[VINED[1]]),
    vine(&GIANTS[VINED[2]]),
    vine(&GIANTS[VINED[3]]),
];

/// Where the floor cairns lie: at the clearing's edge.
pub const CAIRNS: [(i32, i32); 1] = [(1500, -16500)];

/// How the Wood joins the rest of the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::WOOD,
    kind: Kind::Reach,
    seams: &SEAMS,
    vines: &VINES,
    vents: &[],
};

const SEAMS: [Seam; 1] = [
    // Back up the road past the Spire, toward the town.
    Seam {
        zone: Zone::cm([800, -300, -14500], [2000, 700, -14000]),
        to: ArenaId::SPIRE,
        at: 1,
        marks: [
            Mark::cm(1300, -15000, (1300, -25000)),
            Mark::cm(1600, -15000, (1600, -25000)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Needle",
    },
];

// ---------------------------------------------------------------------------
// The trees, built on the land
// ---------------------------------------------------------------------------

fn cm(v: i32) -> Fx {
    super::cm(v)
}

/// A deterministic number in [0, 1000) from two integers and a salt.
fn dice(a: i32, b: i32, salt: u32) -> i32 {
    let mut h = (a as u32).wrapping_mul(0x9E37_79B1)
        ^ (b as u32).wrapping_mul(0x85EB_CA77)
        ^ salt.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h % 1000) as i32
}

/// Thousandths of a turn, as fixed point.
fn turns(milli: i32) -> Fx {
    Fx::ratio(milli.rem_euclid(1000), 1000)
}

/// The integer square root, for flat distances in centimetres.
fn isqrt(n: i64) -> i64 {
    crate::math::isqrt_wide(n.max(0) as u128) as i64
}

/// **How far a branch may run** from a tree along a bearing before it
/// comes within six metres of another tree's bark: the bearing's unit
/// vector in thousandths, and the most it would run.
fn room_toward(k: usize, from: (i32, i32), dir: (i64, i64), most: i32, y: i32) -> i32 {
    let mut best = most as i64;
    for (j, o) in GIANTS.iter().enumerate() {
        if j == k {
            continue;
        }
        let (ox, oz) = ((o.x - from.0) as i64, (o.z - from.1) as i64);
        // Along the bearing to the nearest approach, and how near.
        let along = (ox * dir.0 + oz * dir.1) / 1000;
        if along <= 0 {
            continue;
        }
        let px = ox - dir.0 * along / 1000;
        let pz = oz - dir.1 * along / 1000;
        let off = isqrt(px * px + pz * pz);
        let keep = radius_at(o, y.min(o.height)) as i64 + 600;
        if off < keep {
            // It would reach that bark: stop short of it.
            let back = isqrt(keep * keep - off * off);
            best = best.min(along - back);
        }
    }
    best.max(0) as i32
}

/// One board: from `inner` to `outer` centimetres out from a tree's axis
/// along a bearing (thousandths of a turn), its top at `top`, `wide` across.
fn board(g: &Giant, base: Fx, bearing: i32, inner: i32, outer: i32, top: i32, wide: i32) -> Solid {
    let d = V3::from_turns(turns(bearing));
    let mid = cm((inner + outer) / 2);
    Solid::turned(
        cm(g.x).add(d.x.mul(mid)),
        cm(g.z).add(d.z.mul(mid)),
        base.add(cm(top - 120)),
        base.add(cm(top)),
        cm((outer - inner) / 2),
        cm(wide / 2),
        turns(bearing),
        Material::Wood,
    )
}

/// A cairn: a slab of snow two metres across on a top at `y`.
fn cairn_on(x: Fx, z: Fx, y: Fx) -> Solid {
    Solid {
        min: V3::new(x.sub(Fx::ONE), y.sub(Fx::ratio(3, 10)), z.sub(Fx::ONE)),
        max: V3::new(x.add(Fx::ONE), y.add(Fx::ratio(2, 5)), z.add(Fx::ONE)),
        material: Material::Snow,
        shape: super::Shape::Box,
    }
}

/// **The bands**, as built: where each starts above the foot, the rise from
/// one branch to the next, the gap across at the bark, the board's width,
/// and how long the boards run (shortest, longest).
struct Band {
    from: i32,
    rise: i32,
    across: i32,
    wide: i32,
    long: (i32, i32),
}

const WALKING: Band = Band {
    from: 220,
    rise: 220,
    across: 120,
    wide: 440,
    long: (1800, 2600),
};
const MECHANIC: Band = Band {
    from: 4000,
    rise: 650,
    across: 400,
    wide: 500,
    long: (2400, 3600),
};
const EXPERT: Band = Band {
    from: 0,
    rise: 1100,
    across: 0,
    wide: 500,
    long: (2800, 3800),
};

/// **One tree's solids**: its trunk, its branches, its crown and cairns.
/// `base` is the land under it.
fn tree(k: usize, g: &Giant, base: Fx, out: &mut Vec<Solid>) {
    let (x, z) = (cm(g.x), cm(g.z));
    // The trunk: the flare, then four lengths tapering to the crown.
    let foot = base.sub(Fx::ONE);
    out.push(Solid::round(
        x,
        z,
        foot,
        base.add(cm(FLARE_HIGH)),
        cm(radius_at(g, 0)),
        cm(radius_at(g, FLARE_HIGH)),
        Material::Wood,
    ));
    let crown_under = g.height - 300;
    for seg in 0..4 {
        let y0 = FLARE_HIGH + (crown_under - FLARE_HIGH) * seg / 4;
        let y1 = FLARE_HIGH + (crown_under - FLARE_HIGH) * (seg + 1) / 4;
        out.push(Solid::round(
            x,
            z,
            base.add(cm(y0)),
            base.add(cm(y1)),
            cm(radius_at(g, y0)),
            cm(radius_at(g, y1)),
            Material::Wood,
        ));
    }

    // The spiral of branches.
    let expert_from = crown_under - 2500;
    let mut bearing = dice(g.x, g.z, 1);
    let mut y = WALKING.from;
    let mut count = 0;
    while y < crown_under - 400 {
        let (band, mechanic) = if y < MECHANIC.from {
            (&WALKING, false)
        } else if y < expert_from {
            (&MECHANIC, true)
        } else {
            (&EXPERT, false)
        };
        let r = radius_at(g, y);
        let roll = dice(g.x + y, g.z, 2);
        let long = band.long.0 + (band.long.1 - band.long.0) * roll / 1000;
        let d = V3::from_turns(turns(bearing));
        let dir = (
            (d.x.raw() as i64 * 1000) >> 16,
            (d.z.raw() as i64 * 1000) >> 16,
        );
        let reach = room_toward(k, (g.x, g.z), dir, r + long, y);
        if reach > r + 600 {
            out.push(board(g, base, bearing, r - 150, reach, y, band.wide));
            count += 1;
            // A cairn every fifth branch of the mechanic band, at its tip.
            if mechanic && count % 5 == 0 {
                let tip = cm(reach - 150);
                out.push(cairn_on(
                    x.add(d.x.mul(tip)),
                    z.add(d.z.mul(tip)),
                    base.add(cm(y)),
                ));
            }
        }
        // Round the trunk far enough that this board and the next do not
        // overlap at the bark, plus the gap the band leaves.
        let step = if band.across == 0 {
            250
        } else {
            // A turn's thousandths: arc (board's width + gap) over the
            // circumference at the bark.
            let arc = (band.wide + band.across) as i64 * 1000;
            let round = 2 * 314 * r as i64 / 100;
            (arc / round.max(1)) as i32 + 5
        };
        bearing += step;
        y += band.rise;
    }

    // The crown: a disc of leaves over the top, with a cairn on it.
    let crown = cm(radius_at(g, crown_under) * 5 / 2 + 400);
    out.push(Solid::round(
        x,
        z,
        base.add(cm(crown_under)),
        base.add(cm(g.height)),
        crown.mul(Fx::ratio(4, 5)),
        crown,
        Material::Grass,
    ));
    out.push(cairn_on(x, z, base.add(cm(g.height))));
}

/// **A bridge between two trees**: a board out from each toward the other,
/// meeting in the middle or stopping short of it by the span's gap.
fn span(a: &Giant, b: &Giant, base: (Fx, Fx), s: (i32, i32, i32), out: &mut Vec<Solid>) {
    let (y, rise, gap) = s;
    let (dx, dz) = ((b.x - a.x) as i64, (b.z - a.z) as i64);
    let dist = isqrt(dx * dx + dz * dz) as i32;
    let ra = radius_at(a, y);
    let rb = radius_at(b, (y + rise).min(b.height));
    let open = dist - ra - rb;
    let half = (open - gap) / 2;
    if half <= 300 {
        return;
    }
    // The bearing from a to b, in thousandths of a turn.
    let toward = |fx: i64, fz: i64| {
        let t = crate::math::atan2_turns(Fx::from_int(fz as i32), Fx::from_int(fx as i32));
        ((t.raw() as i64 * 1000) >> 16) as i32
    };
    let ab = toward(dx, dz);
    let ba = toward(-dx, -dz);
    out.push(board(a, base.0, ab, ra - 150, ra + half, y, 400));
    out.push(board(b, base.1, ba, rb - 150, rb + half, y + rise, 400));
}

/// **The Wood's solids**: every tree and every bridge, standing on the land
/// (`land` gives the ground's height at a point, in map metres).
pub fn build(height: &dyn Fn(Fx, Fx) -> Fx, out: &mut Vec<Solid>) {
    let base = |g: &Giant| height(cm(g.x), cm(g.z));
    for (k, g) in GIANTS.iter().enumerate() {
        tree(k, g, base(g), out);
    }
    for &(a, b, y, rise, gap) in &SPANS {
        let (ga, gb) = (&GIANTS[a as usize], &GIANTS[b as usize]);
        span(ga, gb, (base(ga), base(gb)), (y, rise, gap), out);
    }
    out.push(BRIDGE);
}
