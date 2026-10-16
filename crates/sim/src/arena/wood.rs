//! **The Great Wood**: a grove of giant trees south of Hearth, the valley's
//! playground (`docs/design/exploration/0019_spire_wood_and_the_deep.md`
//! §2b, built 2026-10-15). Land, like every reach: a road out of the town's
//! east gate, over the river by a footbridge, past the Needle, south into a
//! clearing four hundred metres across, and in it fourteen giants.
//!
//! **The trees are grown** (`crate::valley::trees`, the owner's second ask,
//! the same day: "a procedural generator ... using some nature mimicking
//! algorithm ... roots that wrap around the landscape"), each where it
//! stands, from the land under it and the trees round it: seven
//! **Sentinels**, giant conifers with tiers of branches at the golden angle
//! -- two metres apart for the first forty metres, a plain jump each,
//! then six apart -- four **Elders**, broad oaks whose crowns are grown by
//! space colonisation, and three **Stranglers**, banyans under wide low
//! crowns with prop roots dropped to the ground. Their limbs reach into each
//! other's: there are no bridges but the trees' own.
//!
//! Sized for what the classes do, not for the plain jump: "however big
//! you're thinking of, probably triple that size". Falls cost what the fall
//! rule says (VALE-2); a cairn stands on the highest of each tree's thick
//! limbs and on two on the way up.

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::fixed::Fx;
use crate::valley::land::{Point, at};
use crate::valley::trees::{self, Kind as Grows, Neighbour, Site, Tree, Vine};
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

/// **A giant**: what kind of tree, where it stands on the map
/// (centimetres), its trunk's radius at its foot and how tall it grows.
#[derive(Clone, Copy, Debug)]
pub struct Giant {
    pub kind: Grows,
    pub x: i32,
    pub z: i32,
    pub radius: i32,
    pub height: i32,
}

const fn giant(kind: Grows, x: i32, z: i32, radius: i32, height: i32) -> Giant {
    Giant {
        kind,
        x,
        z,
        radius,
        height,
    }
}

use Grows::{Elder, Sentinel, Strangler};

/// **The trees**: the Great Tree in the middle, six round it at about fifty
/// metres, seven further out at about a hundred and ten.
pub const GIANTS: [Giant; 14] = [
    giant(Sentinel, 0, -35000, 1000, 20000),
    giant(Sentinel, 5500, -35000, 800, 17000),
    giant(Elder, 2750, -30240, 700, 14000),
    giant(Sentinel, -2750, -30240, 750, 15500),
    giant(Sentinel, -5500, -35000, 800, 17500),
    giant(Elder, -2750, -39760, 650, 13500),
    giant(Sentinel, 2750, -39760, 700, 16500),
    giant(Strangler, 10625, -32153, 750, 10000),
    giant(Elder, 4474, -24951, 700, 15000),
    giant(Strangler, -5164, -25288, 800, 11000),
    giant(Sentinel, -10798, -32901, 650, 15500),
    giant(Elder, -8302, -42217, 700, 14000),
    giant(Strangler, 384, -45993, 850, 10500),
    giant(Elder, 8899, -41466, 650, 13000),
];

/// How far a giant's crown reaches from its trunk, for its neighbours as
/// they grow (centimetres).
pub const fn crown(g: &Giant) -> i32 {
    match g.kind {
        Sentinel => {
            let c = g.height * 33 / 100;
            (if c > 3200 { 3200 } else { c }) + g.radius
        }
        Elder => g.height * 45 / 100,
        Strangler => g.height * 85 / 100,
    }
}

/// How high a giant's trunk stands before it breaks into its crown
/// (centimetres): a Sentinel's all the way.
pub const fn bole(g: &Giant) -> i32 {
    match g.kind {
        Sentinel => g.height,
        Elder => g.height * 30 / 100,
        Strangler => g.height * 38 / 100,
    }
}

/// The trees a vine hangs down -- Sentinels all -- and how far up it goes:
/// to the first shelf, forty metres up, for anybody.
const VINED: [usize; 4] = [0, 3, 6, 10];
const VINE_TOP: i32 = 4100;

/// How far out from a trunk's axis its vine hangs: past the bark at its foot
/// and the shelf at its top, so nothing of the tree is in its way.
const fn vine_out(g: &Giant) -> i32 {
    g.radius * 13 / 10
}

/// A vine hung down a trunk's north side, clear of the bark, from below its
/// foot to [`VINE_TOP`], where it comes up beside the first shelf's edge.
const fn vine(g: &Giant) -> Zone {
    let out = vine_out(g);
    Zone::cm(
        [g.x - 120, -2000, g.z + out],
        [g.x + 120, VINE_TOP, g.z + out + 240],
    )
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
// The trees, grown on the land
// ---------------------------------------------------------------------------

/// **Grow the Wood**: every giant where it stands, each seeing the others as
/// neighbours, on the land under it (`height` is the land in centimetres at a
/// map point in centimetres).
pub fn grow(height: &dyn Fn(i32, i32) -> i32) -> Vec<Tree> {
    GIANTS
        .iter()
        .enumerate()
        .map(|(k, g)| {
            let neighbours: Vec<Neighbour> = GIANTS
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != k)
                .map(|(_, o)| Neighbour {
                    at: (o.x, o.z),
                    girth: o.radius,
                    crown: crown(o),
                    bole: bole(o),
                })
                .collect();
            let site = Site {
                at: (g.x, g.z),
                ground: height,
                neighbours: &neighbours,
                height: g.height,
                girth: g.radius,
                seed: 7919 * (k as u32 + 1),
                // North is a quarter turn; the Wood is at the map's origin,
                // so the vine's top is its height on the map.
                vine: VINED.contains(&k).then(|| Vine {
                    heading: 250,
                    out: vine_out(g),
                    top: VINE_TOP,
                }),
            };
            trees::grow(g.kind, &site)
        })
        .collect()
}

/// **The Wood's solids**: the footbridge, and each grown tree's wood and
/// cairns -- the wood drawn as the tree it is, not as boxes
/// (`Plan::hidden`).
pub fn build(trees: &[Tree], wood: &mut Vec<Solid>, cairns: &mut Vec<Solid>) {
    for t in trees {
        trees::solids(t, wood);
        trees::cairns_of(t, cairns);
    }
    cairns.push(BRIDGE);
}

/// **A liana off every broadleaf's roof**: a vine hung from just outside its
/// lip to the ground, where nothing of any tree (`wood`, every tree's
/// solids) is in its way -- the first such place round the roof, from the
/// side facing the Great Tree. A climb of it comes up beside the lip and
/// steps on to it (`tests/wood.rs`). Map coordinates (`Atlas::vines`).
pub fn lianas(trees: &[Tree], wood: &[Solid], height: &dyn Fn(i32, i32) -> i32) -> Vec<Zone> {
    let mut out = Vec::new();
    for t in trees {
        let Some(rim) = t.pads.iter().find(|p| p.kind == trees::PadKind::Rim) else {
            continue;
        };
        let (mx, mz) = (rim.at[0] as i64, rim.at[2] as i64);
        let toward = crate::math::atan2_turns(
            Fx::from_int(((GIANTS[0].z as i64 - mz) / 100) as i32),
            Fx::from_int(((GIANTS[0].x as i64 - mx) / 100) as i32),
        );
        let start = (toward.raw() as i64 * 1000) >> 16;
        let out_r = rim.outer as i64 + 150;
        let found = (0..40).find_map(|k| {
            let turn = start + if k % 2 == 0 { k / 2 } else { -(k + 1) / 2 } * 25;
            let d = crate::math::V3::from_turns(Fx::ratio(turn.rem_euclid(1000) as i32, 1000));
            let cx = mx + ((d.x.raw() as i64 * out_r) >> 16);
            let cz = mz + ((d.z.raw() as i64 * out_r) >> 16);
            let floor = height(cx as i32, cz as i32);
            let zone = Zone::cm(
                [cx as i32 - 120, floor - 200, cz as i32 - 120],
                [cx as i32 + 120, rim.at[1] + 30, cz as i32 + 120],
            );
            let (x, z) = (super::cm(cx as i32), super::cm(cz as i32));
            let (lo, hi) = (super::cm(floor + 250), zone.max.y);
            let blocked = wood.iter().any(|s| {
                let (a, b) = (s.min.y.max(lo), s.max.y.min(hi));
                a.raw() < b.raw() && s.side_push(x, z, Fx::ratio(1, 2), a, b).is_some()
            });
            (!blocked).then_some(zone)
        });
        out.extend(found);
    }
    out
}
