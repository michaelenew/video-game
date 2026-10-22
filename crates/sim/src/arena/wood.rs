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
//!
//! **And quicker ways up** ([`ways_up`], 2026-10-22, the owner: "the forest
//! needs some more entertaining and artificially shortened ways up"): a
//! broadleaf's hanging root leads to a landing 36 m up, and from there an
//! Elder's updraft or a Strangler's spring; most Sentinels have a hoist from
//! their first shelf to their highest.

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::drift::Drift;
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::Vent;

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

/// How far apart a hanging root's shelves are, one over the next: a catch.
pub const LADDER_RISE: i32 = 400;
/// How far either side of its root a shelf stands, how big it is (a
/// radius) and how deep.
const LADDER_SIDE: i32 = 260;
const LADDER_SHELF: i32 = 130;
const LADDER_DEEP: i32 = 150;
/// **How high a hanging root's shelves go** (2026-10-22): to a landing this
/// far over the ground, a cairn on it, and from there something else takes
/// you up -- where until then the shelves went on, a catch apart, all the
/// way to the lip, thirty of them up an Elder.
pub const LANDING_UP: i32 = 3600;
/// The landing's radius.
pub const LANDING_R: i32 = 300;

/// **An Elder's updraft**: a column of rising air over the landing, this
/// wide (a radius) and letting go this far over the lip, so a body carried
/// to its top is high enough to drift in over the lip.
pub const DRAFT_R: i32 = 170;
pub const DRAFT_OVER: i32 = 400;

/// **A Strangler's spring**: a stone beside the landing, out from the roof,
/// that shoots up to stop this far under the lip -- past every plain jump's
/// catch from where it stops -- so the way on to the roof is a jump off it
/// while it is still going up, its speed added to the jump's. Its deck's
/// half-side, and its clock: the wait at the landing, frames a metre on the
/// way up (about eight and a half metres a second, thirteen at its fastest),
/// the stay at the top, the way down.
pub const SPRING_SHORT: i32 = 900;
pub const SPRING_HALF: i32 = 150;
const SPRING_CLOCK: (u16, u16, u16) = (150, 40, 120);
const SPRING_FRAMES_A_METRE: i32 = 5;

/// **A Sentinel's hoist**: two cars side by side outside the crown's trunk,
/// from a fungus landing at its first shelf to one at its highest. How far
/// out it may stand from the trunk (centimetres), a car's half-side, how far
/// apart the two cars are, how far in from them each landing's middle is
/// and its radius; its clock: the wait at either end, and frames a metre up
/// (ten metres a second, fifteen at its fastest) and down.
const HOIST_OUT: (i32, i32) = (1300, 2400);
const HOIST_HALF: i32 = 150;
const HOIST_APART: i32 = 320;
const HOIST_IN: i32 = 470;
pub const HOIST_PAD: i32 = 300;
const HOIST_WAIT: u16 = 150;
const HOIST_FRAMES_A_METRE: (i32, i32) = (6, 4);

/// **What takes you up a broadleaf from its landing** (2026-10-22): an
/// Elder's updraft or a Strangler's spring.
#[derive(Clone, Copy, Debug)]
pub struct Climb {
    /// Which tree (an index into `GIANTS`), and what kind.
    pub tree: usize,
    pub kind: Grows,
    /// The landing the shelves lead to.
    pub landing: trees::Pad,
    /// The lip's top, the roof's middle and its outer radius (centimetres).
    pub lip: i32,
    pub roof: (i32, i32),
    pub roof_outer: i32,
}

/// **A Sentinel's hoist** (2026-10-22): its tree, the fungus landings at its
/// foot and its head, and its two cars.
#[derive(Clone, Copy, Debug)]
pub struct Hoist {
    pub tree: usize,
    pub foot: trees::Pad,
    pub head: trees::Pad,
    pub cars: [Drift; 2],
}

/// **The ways up the Wood that are not its own limbs** (2026-10-22): what
/// [`ways_up`] puts in it.
#[derive(Clone, Debug, Default)]
pub struct WaysUp {
    /// The shelves and landings, as the trees' wood: drawn as the trees are.
    pub boxes: Vec<Solid>,
    /// The cairns on the landings.
    pub cairns: Vec<Solid>,
    pub climbs: Vec<Climb>,
    pub hoists: Vec<Hoist>,
    /// The springs and the hoists' cars.
    pub drifts: Vec<Drift>,
    /// The Elders' updrafts.
    pub vents: Vec<Vent>,
}

fn cm(v: i32) -> Fx {
    super::cm(v)
}

/// Is nothing of `wood` in an upright cylinder at (x, z), `r` wide, between
/// heights `lo` and `hi` -- but what `skip` says may be?
fn clear(
    wood: &[Solid],
    x: i32,
    z: i32,
    r: i32,
    lo: i32,
    hi: i32,
    skip: &dyn Fn(&Solid) -> bool,
) -> bool {
    let (lo, hi) = (cm(lo), cm(hi));
    !wood.iter().any(|s| {
        let (a, b) = (s.min.y.max(lo), s.max.y.min(hi));
        a.raw() < b.raw() && !skip(s) && s.side_push(cm(x), cm(z), cm(r), a, b).is_some()
    })
}

/// `v` centimetres along the unit `d`, flat.
fn along(d: V3, v: i32) -> (i32, i32) {
    (
        ((d.x.raw() as i64 * v as i64) >> 16) as i32,
        ((d.z.raw() as i64 * v as i64) >> 16) as i32,
    )
}

/// A fungus landing: a shelf `r` wide at `at`.
fn landing(at: [i32; 3], r: i32) -> trees::Pad {
    trees::Pad {
        at,
        inner: 0,
        outer: r,
        thick: LADDER_DEEP,
        kind: trees::PadKind::Shelf,
    }
}

fn pad_solid(p: &trees::Pad) -> Solid {
    Solid::ring(
        cm(p.at[0]),
        cm(p.at[2]),
        cm(p.at[1] - p.thick),
        cm(p.at[1]),
        (cm(p.outer), cm(p.outer)),
        Fx::ZERO,
        Material::Wood,
    )
}

fn cairn_at(at: [i32; 3]) -> Solid {
    Solid::cm(
        [at[0] - 100, at[1] - 30, at[2] - 100],
        [at[0] + 100, at[1] + 40, at[2] + 100],
        Material::Snow,
    )
}

/// **The ways up the Wood's broadleaves and Sentinels** (`ways_up`'s two
/// halves: [`ladders`] and [`hoists`]).
pub fn ways_up(trees: &mut [Tree], wood: &[Solid], height: &dyn Fn(i32, i32) -> i32) -> WaysUp {
    let mut out = WaysUp::default();
    ladders(trees, wood, height, &mut out);
    // A hoist keeps out of the ladders' way too, and of the springs' and
    // the updrafts' columns.
    let mut all = wood.to_vec();
    all.extend(out.boxes.iter().copied());
    all.extend(out.drifts.iter().map(|d| d.swept()));
    all.extend(out.vents.iter().map(|v| Solid {
        min: V3::new(v.at.0.sub(v.radius), v.from, v.at.1.sub(v.radius)),
        max: V3::new(v.at.0.add(v.radius), v.top, v.at.1.add(v.radius)),
        material: Material::Rock,
        shape: crate::arena::Shape::Box,
    }));
    hoists(trees, &all, &mut out);
    out
}

/// **A hanging root up every broadleaf** (2026-10-16, where a liana hung
/// until then; cut short 2026-10-22): a root dropped from just outside the
/// roof's lip to the ground, where nothing of any tree (`wood`) is in its
/// way -- nor in the way of what takes you on up from it -- the first such
/// place round the roof, from the side facing the Great Tree; with bracket
/// fungus up it, either side by turns, a catch apart, to **a landing**
/// [`LANDING_UP`] over the ground with a cairn. From the landing:
///
/// - **an Elder's updraft** -- a column of rising air over the landing to
///   over the lip: step off the landing into it, ride it up, drift in;
/// - **a Strangler's spring** -- a stone beside the landing that shoots up
///   to [`SPRING_SHORT`] under the lip: ride it, and jump off it while it is
///   still going up.
///
/// The shelves are put on the tree (`Tree::pads`, drawn as its shelves are)
/// and the root among its nodes.
fn ladders(trees: &mut [Tree], wood: &[Solid], height: &dyn Fn(i32, i32) -> i32, out: &mut WaysUp) {
    for (k, t) in trees.iter_mut().enumerate() {
        let Some(rim) = t
            .pads
            .iter()
            .find(|p| p.kind == trees::PadKind::Rim)
            .copied()
        else {
            continue;
        };
        let (mx, mz) = (rim.at[0] as i64, rim.at[2] as i64);
        let toward = crate::math::atan2_turns(
            Fx::from_int(((GIANTS[0].z as i64 - mz) / 100) as i32),
            Fx::from_int(((GIANTS[0].x as i64 - mx) / 100) as i32),
        );
        let start = (toward.raw() as i64 * 1000) >> 16;
        let out_r = rim.outer as i64 + LADDER_SHELF as i64 + 60;
        let lip = rim.at[1];
        let kind = t.kind;
        let found = (0..40).find_map(|n| {
            let turn = start + if n % 2 == 0 { n / 2 } else { -(n + 1) / 2 } * 25;
            let d = crate::math::V3::from_turns(Fx::ratio(turn.rem_euclid(1000) as i32, 1000));
            let cx = mx + ((d.x.raw() as i64 * out_r) >> 16);
            let cz = mz + ((d.z.raw() as i64 * out_r) >> 16);
            let floor = height(cx as i32, cz as i32);
            let (pads, top) = shelves(cx as i32, cz as i32, d, floor);
            let none = |_: &Solid| false;
            // The whole ladder's width, from the ground to over the
            // landing: a jump between two shelves rises past the one above.
            let ladder_clear = clear(
                wood,
                cx as i32,
                cz as i32,
                LADDER_SIDE + LADDER_SHELF + 60,
                floor + 100,
                top.at[1] + 250,
                &none,
            );
            let rungs_clear = ladder_clear
                && pads.iter().all(|p| {
                    clear(
                        wood,
                        p.at[0],
                        p.at[2],
                        p.outer + 60,
                        p.at[1] - p.thick - 200,
                        p.at[1] + 250,
                        &none,
                    )
                });
            // What takes you on up, clear of every tree from the landing to
            // over the lip -- but its own roof, which it rises beside (a
            // terrace is a ring round the roof's middle).
            let roof = |s: &Solid| {
                let (x, z) = s.middle();
                s.material == Material::Grass
                    && x.sub(cm(rim.at[0])).abs().raw() < Fx::ONE.raw()
                    && z.sub(cm(rim.at[2])).abs().raw() < Fx::ONE.raw()
            };
            let device_clear = match kind {
                Grows::Elder => clear(
                    wood,
                    top.at[0],
                    top.at[2],
                    DRAFT_R + 30,
                    top.at[1] - 100,
                    lip + DRAFT_OVER + 300,
                    &roof,
                ),
                _ => {
                    let (sx, sz) = spring_at(&top, d);
                    clear(
                        wood,
                        sx,
                        sz,
                        SPRING_HALF + 80,
                        top.at[1] - 100,
                        lip - SPRING_SHORT + 300,
                        &roof,
                    )
                }
            };
            (rungs_clear && device_clear).then_some((cx as i32, cz as i32, floor, pads, top, d))
        });
        let Some((cx, cz, floor, pads, top, d)) = found else {
            continue;
        };
        // The root: from the lip's height down to the ground, hung from the
        // nearest of the crown's nodes.
        let root_top = [cx, lip, cz];
        let parent = t
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.part != trees::Part::Root)
            .min_by_key(|(_, n)| {
                let d = [
                    n.at[0] - root_top[0],
                    n.at[1] - root_top[1],
                    n.at[2] - root_top[2],
                ];
                d.iter().map(|v| (*v as i64).pow(2)).sum::<i64>()
            })
            .map_or(0, |(i, _)| i as u32);
        let mut last = parent;
        let mut y = lip;
        loop {
            t.nodes.push(trees::Node {
                at: [cx, y, cz],
                parent: last,
                radius: 40,
                part: trees::Part::Prop,
            });
            last = t.nodes.len() as u32 - 1;
            if y <= floor {
                break;
            }
            y = (y - 600).max(floor - 50);
        }
        for p in pads.iter().chain(core::iter::once(&top)) {
            out.boxes.push(pad_solid(p));
        }
        t.pads.extend(pads);
        t.pads.push(top);
        t.cairns.push(top.at);
        out.cairns.push(cairn_at(top.at));
        let climb = Climb {
            tree: k,
            kind,
            landing: top,
            lip,
            roof: (rim.at[0], rim.at[2]),
            roof_outer: rim.outer,
        };
        match kind {
            Grows::Elder => out.vents.push(draft(&climb)),
            _ => out.drifts.push(spring(&climb, d)),
        }
        out.climbs.push(climb);
    }
}

/// A hanging root's shelves, at `(x, z)` hung `d` out from its roof: from
/// [`LADDER_RISE`] over the ground `floor` to a catch under the landing,
/// evenly and no more than three and a half metres apart, either side of the
/// root by turns -- and the landing, on the side the next would have been.
fn shelves(x: i32, z: i32, d: crate::math::V3, floor: i32) -> (Vec<trees::Pad>, trees::Pad) {
    let (from, to) = (floor + LADDER_RISE, floor + LANDING_UP - LADDER_RISE);
    // A little closer than a rise apart: off a shelf a body has hardly a
    // step of run-up, and four metres up is the edge of the Champion's catch
    // from one.
    let n = ((to - from) / (LADDER_RISE - 50) + 1).max(1);
    // Across the way out from the roof: (-z, x) of `d`, in centimetres.
    let (ax, az) = (
        -((d.z.raw() as i64 * LADDER_SIDE as i64) >> 16) as i32,
        ((d.x.raw() as i64 * LADDER_SIDE as i64) >> 16) as i32,
    );
    let side = |k: i32| if k % 2 == 0 { 1 } else { -1 };
    let pads = (0..=n)
        .map(|k| trees::Pad {
            at: [
                x + side(k) * ax,
                from + (to - from) * k / n,
                z + side(k) * az,
            ],
            inner: 0,
            outer: LADDER_SHELF,
            thick: LADDER_DEEP,
            kind: trees::PadKind::Shelf,
        })
        .collect();
    // The landing on the side the next shelf would have been, as far out
    // from the root as a shelf's inner edge is: wider than a shelf, it would
    // otherwise hang over the last one.
    let s = side(n + 1);
    let far = LADDER_SIDE + LANDING_R - LADDER_SHELF;
    let (lx, lz) = (
        ax as i64 * far as i64 / LADDER_SIDE as i64,
        az as i64 * far as i64 / LADDER_SIDE as i64,
    );
    let top = landing(
        [x + s * lx as i32, floor + LANDING_UP, z + s * lz as i32],
        LANDING_R,
    );
    (pads, top)
}

/// **An Elder's updraft**, over its landing.
fn draft(c: &Climb) -> Vent {
    Vent {
        at: (cm(c.landing.at[0]), cm(c.landing.at[2])),
        radius: cm(DRAFT_R),
        top: cm(c.lip + DRAFT_OVER),
        from: cm(c.landing.at[1] + 10),
    }
}

/// Where a Strangler's spring stands: beside its landing, out from the roof.
pub fn spring_at(landing: &trees::Pad, d: V3) -> (i32, i32) {
    let (ox, oz) = along(d, landing.outer + 30 + SPRING_HALF);
    (landing.at[0] + ox, landing.at[2] + oz)
}

/// **A Strangler's spring**: its deck level with the landing, out from the
/// roof (`d`), and up to [`SPRING_SHORT`] under the lip.
fn spring(c: &Climb, d: V3) -> Drift {
    let (sx, sz) = spring_at(&c.landing, d);
    let top = c.landing.at[1];
    let h = SPRING_HALF;
    let rise = c.lip - SPRING_SHORT - top;
    let (wait, stay, back) = SPRING_CLOCK;
    Drift {
        solid: Solid::cm(
            [sx - h, top - 40, sz - h],
            [sx + h, top, sz + h],
            Material::Sand,
        ),
        travel: V3::new(Fx::ZERO, cm(rise), Fx::ZERO),
        wait,
        go: (rise / 100 * SPRING_FRAMES_A_METRE) as u16,
        stay,
        back,
        phase: 0,
    }
}

/// **A hoist up every Sentinel** (2026-10-22): two cars side by side
/// outside its trunk, from a fungus landing at its first shelf -- overlapping
/// one of that shelf's lobes -- to one level with its highest, a cairn on it, where
/// nothing of any tree is in the way of either car or either landing. The
/// first such place, searched out from the trunk and round it; a Sentinel
/// with none has no hoist.
fn hoists(trees: &mut [Tree], wood: &[Solid], out: &mut WaysUp) {
    for (k, t) in trees.iter_mut().enumerate() {
        if t.kind != Grows::Sentinel {
            continue;
        }
        let shelves: Vec<trees::Pad> = t
            .pads
            .iter()
            .filter(|p| p.kind == trees::PadKind::Shelf)
            .copied()
            .collect();
        let (Some(foot_y), Some(head_y)) = (
            shelves.iter().map(|p| p.at[1]).min(),
            shelves.iter().map(|p| p.at[1]).max(),
        ) else {
            continue;
        };
        if head_y - foot_y < 3000 {
            continue;
        }
        let axis = (GIANTS[k].x, GIANTS[k].z);
        // A lobe at a level, and whether a landing at `at` reaches it.
        let lobe = |s: &Solid, y: i32| s.max.y == cm(y) && s.material == Material::Wood;
        let reaches = |at: (i32, i32), y: i32| {
            shelves.iter().filter(|p| p.at[1] == y).any(|p| {
                let (dx, dz) = ((p.at[0] - at.0) as i64, (p.at[2] - at.1) as i64);
                let d = crate::math::isqrt_wide((dx * dx + dz * dz) as u128) as i32;
                d <= p.outer + HOIST_PAD + 30
            })
        };
        let found = (HOIST_OUT.0..=HOIST_OUT.1).step_by(100).find_map(|r| {
            (0..72).find_map(|b| {
                let d = V3::from_turns(Fx::ratio(b, 72));
                let across = V3::new(d.z.neg(), Fx::ZERO, d.x);
                let (ox, oz) = along(d, r);
                let (tx, tz) = along(across, HOIST_APART / 2);
                let cars = [
                    (axis.0 + ox - tx, axis.1 + oz - tz),
                    (axis.0 + ox + tx, axis.1 + oz + tz),
                ];
                let (ix, iz) = along(d, r - HOIST_IN);
                let pad = (axis.0 + ix, axis.1 + iz);
                let cars_clear = cars.iter().all(|c| {
                    clear(
                        wood,
                        c.0,
                        c.1,
                        HOIST_HALF + 50,
                        foot_y - 60,
                        head_y + 300,
                        &|_| false,
                    )
                });
                // The foot's landing reaches a lobe of the first shelf, the
                // stair's top; the head's is a shelf of its own.
                let pads_clear = reaches(pad, foot_y)
                    && [foot_y, head_y].iter().all(|&y| {
                        clear(
                            wood,
                            pad.0,
                            pad.1,
                            HOIST_PAD,
                            y - LADDER_DEEP - 80,
                            y + 250,
                            &|s| lobe(s, y),
                        )
                    });
                (cars_clear && pads_clear).then_some((cars, pad))
            })
        });
        let Some((cars, pad)) = found else {
            continue;
        };
        let rise = head_y - foot_y;
        let up = (rise / 100 * HOIST_FRAMES_A_METRE.0) as u16;
        let down = (rise / 100 * HOIST_FRAMES_A_METRE.1) as u16;
        let period = 2 * HOIST_WAIT as u32 + up as u32 + down as u32;
        let h = HOIST_HALF;
        let car = |n: usize| {
            let (cx, cz) = cars[n];
            Drift {
                solid: Solid::cm(
                    [cx - h, foot_y - 40, cz - h],
                    [cx + h, foot_y, cz + h],
                    Material::Wood,
                ),
                travel: V3::new(Fx::ZERO, cm(rise), Fx::ZERO),
                wait: HOIST_WAIT,
                go: up,
                stay: HOIST_WAIT,
                back: down,
                phase: (n as u32 * period / 2) as u16,
            }
        };
        let foot = landing([pad.0, foot_y, pad.1], HOIST_PAD);
        let head = landing([pad.0, head_y, pad.1], HOIST_PAD);
        for p in [&foot, &head] {
            out.boxes.push(pad_solid(p));
            t.pads.push(*p);
        }
        t.cairns.push(head.at);
        out.cairns.push(cairn_at(head.at));
        let cars = [car(0), car(1)];
        out.drifts.extend(cars);
        out.hoists.push(Hoist {
            tree: k,
            foot,
            head,
            cars,
        });
    }
}
