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

use crate::valley::land::{Point, at};
use crate::valley::trees::{self, Kind as Grows, Neighbour, Site, Tree};
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

/// Where the floor cairns lie: at the clearing's edge.
pub const CAIRNS: [(i32, i32); 1] = [(1500, -16500)];

/// How the Wood joins the rest of the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::WOOD,
    kind: Kind::Reach,
    seams: &SEAMS,
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
