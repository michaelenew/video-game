//! **Grown trees** (2026-10-15, the owner: "make a procedural generator that
//! produces the trees using some nature mimicking algorithm ... branches
//! arranged like branches in nature ... roots that wrap around the landscape
//! ... a few versions for different kinds of trees ... the algorithm to run
//! in a specific place in the world where the tree grows so it can respond
//! to the landscape").
//!
//! A tree is **grown where it stands**, from a [`Site`]: the ground under and
//! round it, and its neighbours. Three kinds, three of the ways botanists
//! describe trees growing:
//!
//! - **The Sentinel** -- a giant conifer, Massart's model in Hallé and
//!   Oldeman's catalogue of tree architectures: one leading shoot that grows
//!   straight up, and **tiers of branches** off it, each tier turned from the
//!   last by the golden angle (137.5 degrees, the angle leaves and branches
//!   are set at round a stem, so that none shades the one below). Its lower
//!   branches droop and its upper ones rise, as a fir's do, and its crown is
//!   a cone. The tiers are close where you start climbing.
//! - **The Elder** -- a broad oak: a short trunk that breaks up into a crown,
//!   grown by **space colonisation** (Runions, Lane and Prusinkiewicz,
//!   "Modeling Trees with a Space Colonization Algorithm", 2007): points
//!   scattered through the space the crown may fill, each pulling the nearest
//!   bud toward it, and taken once a branch reaches it. Branches compete for
//!   space, so they spread and fork as an oak's do.
//! - **The Strangler** -- a banyan, the strangler fig: a thick trunk under a
//!   wide flat crown grown the same way but held down toward the level, and
//!   **prop roots** dropped from its great limbs to the ground, which thicken
//!   into pillars.
//!
//! Then, for all three:
//!
//! - **The pipe model** (da Vinci's rule, and Shinozaki's): a branch is as
//!   thick as all the twigs it carries put together -- its cross-section the
//!   sum of its children's -- so trunks and limbs taper the way real ones do,
//!   and the limbs low down, which carry most, are the ones you can stand on.
//! - **It answers its place.** Its crown is pushed away from its neighbours'
//!   and toward the open side (the space a neighbour's crown already fills is
//!   shade, and fewer of the colonising points survive there); its leader
//!   leans a little toward the light; a branch that would grow into a
//!   hillside bends up off it.
//! - **Roots that wrap the landscape**: buttresses flaring off the trunk's
//!   foot, then roots running outward along the ground -- downhill rather
//!   than up, round a neighbour's foot rather than into it -- that follow the
//!   land's height step by step, half buried, so they drape over every bump.
//!
//! Everything is integers in centimetres (the simulation has no floating
//! point), so both machines grow the same tree. The skeleton is what is
//! drawn (`game::grove`) and what is stood on: [`solids`] turns its thick
//! parts into the shapes a body collides with.

use crate::arena::{Material, Shape, Solid};
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::Zone;

/// **What kind of tree.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Sentinel,
    Elder,
    Strangler,
}

/// What a node of a tree is part of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Trunk,
    Limb,
    Twig,
    Root,
    /// A strangler's prop root, from a limb to the ground.
    Prop,
}

/// No parent: the trunk's foot.
pub const NONE: u32 = u32::MAX;

/// **A node of a tree**: where it is on the map (centimetres), the node it
/// grew from, how thick the tree is here (a radius, centimetres), and what it
/// is part of. A segment runs from each node's parent to it.
#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub at: [i32; 3],
    pub parent: u32,
    pub radius: i32,
    pub part: Part,
}

/// **A grown tree.**
#[derive(Clone, Debug)]
pub struct Tree {
    pub kind: Kind,
    /// The foot of its trunk on the map.
    pub base: [i32; 3],
    pub nodes: Vec<Node>,
    /// Its foliage: a clump at each twig's end -- where (x, y, z) and how big
    /// (a radius), centimetres.
    pub leaves: Vec<[i32; 4]>,
    /// Tops a cairn stands on: the highest of its thick limbs, and some on
    /// the way.
    pub cairns: Vec<[i32; 3]>,
}

/// A neighbour, as the tree growing sees it: where its trunk stands, how
/// thick it is and how far its crown reaches (centimetres).
#[derive(Clone, Copy, Debug)]
pub struct Neighbour {
    pub at: (i32, i32),
    pub girth: i32,
    pub crown: i32,
}

/// **Where a tree grows**: its foot on the map, the ground (a height in
/// centimetres at a map point in centimetres), its neighbours, how tall and
/// how thick it is to be, and a seed.
pub struct Site<'a> {
    pub at: (i32, i32),
    pub ground: &'a dyn Fn(i32, i32) -> i32,
    pub neighbours: &'a [Neighbour],
    pub height: i32,
    pub girth: i32,
    pub seed: u32,
}

// ---------------------------------------------------------------------------
// Integers
// ---------------------------------------------------------------------------

type V = [i64; 3];

/// A unit vector is a thousand long.
const UNIT: i64 = 1000;

fn add(a: V, b: V) -> V {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: V, num: i64, den: i64) -> V {
    [a[0] * num / den, a[1] * num / den, a[2] * num / den]
}

fn isqrt(n: i64) -> i64 {
    crate::math::isqrt_wide(n.max(0) as u128) as i64
}

fn len(a: V) -> i64 {
    isqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2])
}

fn flat_len(a: V) -> i64 {
    isqrt(a[0] * a[0] + a[2] * a[2])
}

/// `a` a thousand long, or nothing.
fn unit(a: V) -> V {
    let l = len(a);
    if l == 0 { [0, 0, 0] } else { scale(a, UNIT, l) }
}

fn d2(a: V, b: V) -> i64 {
    let d = sub(a, b);
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

fn v(at: [i32; 3]) -> V {
    [at[0] as i64, at[1] as i64, at[2] as i64]
}

fn at(p: V) -> [i32; 3] {
    [p[0] as i32, p[1] as i32, p[2] as i32]
}

/// A deterministic stream of numbers.
struct Dice(u32);

impl Dice {
    fn next(&mut self) -> u32 {
        // xorshift32, then a mix.
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x.wrapping_mul(0x9E37_79B1) >> 8
    }

    /// A number in `lo..=hi`.
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next() as i64) % (hi - lo + 1)
    }
}

/// A unit direction in the floor plane at an angle in thousandths of a turn.
fn heading(milli: i64) -> V {
    let d = crate::math::V3::from_turns(Fx::ratio(milli.rem_euclid(1000) as i32, 1000));
    [
        (d.x.raw() as i64 * UNIT) >> 16,
        0,
        (d.z.raw() as i64 * UNIT) >> 16,
    ]
}

/// The golden angle, in thousandths of a turn: 137.5 degrees.
const GOLDEN: i64 = 382;

// ---------------------------------------------------------------------------
// The site, read
// ---------------------------------------------------------------------------

/// What a tree reads off its place before it grows.
struct Reading<'a> {
    site: &'a Site<'a>,
    /// The foot of the trunk: the lowest ground under it.
    base: V,
    /// **Where the light is**, flat, a thousand long at most: away from the
    /// neighbours, weighted by how near and how big, and down the slope.
    light: V,
}

fn read<'a>(site: &'a Site<'a>) -> Reading<'a> {
    let g = site.ground;
    let (x, z) = site.at;
    let r = site.girth * 8 / 10;
    let mut low = g(x, z);
    for (dx, dz) in [(r, 0), (-r, 0), (0, r), (0, -r)] {
        low = low.min(g(x + dx, z + dz));
    }
    let base = [x as i64, low as i64, z as i64];
    let mut light = [0i64, 0, 0];
    for n in site.neighbours {
        let d = [(x - n.at.0) as i64, 0, (z - n.at.1) as i64];
        let l = flat_len(d).max(1);
        let reach = (n.crown as i64 * 3).max(1);
        if l > reach {
            continue;
        }
        let w = UNIT * (reach - l) / reach;
        light = add(light, scale(unit(d), w, UNIT));
    }
    // Downhill is open: the ground falls away and the sky with it.
    let s = 800;
    let dx = (g(x + s, z) - g(x - s, z)) as i64;
    let dz = (g(x, z + s) - g(x, z - s)) as i64;
    let downhill = [-dx, 0, -dz];
    if flat_len(downhill) > 0 {
        light = add(light, scale(unit(downhill), 400, UNIT));
    }
    let l = flat_len(light);
    if l > UNIT {
        light = scale(light, UNIT, l);
    }
    Reading { site, base, light }
}

impl Reading<'_> {
    fn ground(&self, p: V) -> i64 {
        (self.site.ground)(p[0] as i32, p[2] as i32) as i64
    }

    /// **Is a point in a neighbour's shade**: inside its crown's reach and
    /// nearer its trunk than this one's.
    fn shaded(&self, p: V) -> bool {
        let own = flat_len(sub(p, self.base));
        self.site.neighbours.iter().any(|n| {
            let d = flat_len(sub(p, [n.at.0 as i64, 0, n.at.1 as i64]));
            d < n.crown as i64 * 9 / 10 && d < own
        })
    }

    /// **Is a point against a neighbour's bark**: within `keep` of its trunk.
    fn against(&self, p: V, keep: i64) -> bool {
        self.site
            .neighbours
            .iter()
            .any(|n| flat_len(sub(p, [n.at.0 as i64, 0, n.at.1 as i64])) < n.girth as i64 + keep)
    }

    /// **How far a branch may run** from `from` along the flat direction
    /// `dir` before it comes within a few metres of a neighbour's bark.
    fn room(&self, from: V, dir: V, most: i64) -> i64 {
        let mut best = most;
        for n in self.site.neighbours {
            let o = sub([n.at.0 as i64, 0, n.at.1 as i64], [from[0], 0, from[2]]);
            let along = (o[0] * dir[0] + o[2] * dir[2]) / UNIT;
            if along <= 0 {
                continue;
            }
            let p = sub(o, scale(dir, along, UNIT));
            let off = flat_len(p);
            let keep = n.girth as i64 + 600;
            if off < keep {
                best = best.min(along - isqrt(keep * keep - off * off));
            }
        }
        best.max(0)
    }
}

// ---------------------------------------------------------------------------
// Growing
// ---------------------------------------------------------------------------

/// **Grow a tree** of `kind` at `site`.
pub fn grow(kind: Kind, site: &Site) -> Tree {
    let r = read(site);
    let mut dice = Dice(site.seed.wrapping_mul(2_654_435_761) | 1);
    let mut nodes: Vec<Node> = Vec::new();
    let mut leaves: Vec<[i32; 4]> = Vec::new();
    match kind {
        Kind::Sentinel => sentinel(&r, &mut dice, &mut nodes, &mut leaves),
        Kind::Elder | Kind::Strangler => broadleaf(kind, &r, &mut dice, &mut nodes, &mut leaves),
    }
    pipe(&mut nodes, site.girth);
    if kind == Kind::Strangler {
        props(&r, &mut dice, &mut nodes);
    }
    roots(kind, &r, &mut dice, &mut nodes);
    let cairns = cairns(&nodes, kind);
    Tree {
        kind,
        base: at(r.base),
        nodes,
        leaves,
        cairns,
    }
}

/// **The leader**: a trunk from the foot up to `top`, a node every `step`,
/// leaning toward the light by `lean` thousandths of its height at the top
/// (bending more the higher it goes, as a stem bends toward light).
fn leader(
    r: &Reading,
    dice: &mut Dice,
    top: i64,
    step: i64,
    lean: i64,
    nodes: &mut Vec<Node>,
) -> Vec<u32> {
    let mut ids = Vec::new();
    let mut parent = NONE;
    let mut y = 0;
    while y <= top {
        let t = y * UNIT / top.max(1);
        let push = lean * top / UNIT * t / UNIT * t / UNIT;
        let wob = [dice.range(-30, 30), 0, dice.range(-30, 30)];
        let p = add(add(r.base, [0, y, 0]), add(scale(r.light, push, UNIT), wob));
        nodes.push(Node {
            at: at(p),
            parent,
            radius: 0,
            part: Part::Trunk,
        });
        parent = nodes.len() as u32 - 1;
        ids.push(parent);
        y += step;
    }
    ids
}

/// The leader's node nearest under a height over the foot, and where along
/// the trunk that height is.
fn on_leader(nodes: &[Node], leader: &[u32], base: V, h: i64) -> (u32, V) {
    let mut best = leader[0];
    for &k in leader {
        if (nodes[k as usize].at[1] as i64 - base[1]) <= h {
            best = k;
        }
    }
    let p = v(nodes[best as usize].at);
    (best, [p[0], base[1] + h, p[2]])
}

/// **The Sentinel**: Massart's model. A leader, and tiers of branches off it
/// at the golden angle, close where you start climbing and further apart
/// above, the lower drooping and the upper rising, each with side shoots and
/// foliage.
fn sentinel(r: &Reading, dice: &mut Dice, nodes: &mut Vec<Node>, leaves: &mut Vec<[i32; 4]>) {
    let h = r.site.height as i64;
    let girth = r.site.girth as i64;
    let lead = leader(r, dice, h, 250, 40, nodes);
    let mut phase = dice.range(0, 999);
    let mut y = 200;
    while y < h - 600 {
        // The tiers: two metres apart for the first forty -- a plain
        // jump each -- six above, four and a half near the top.
        let gap = if y < 4000 {
            200
        } else if y < h - 2500 {
            600
        } else {
            450
        };
        let (from, p) = on_leader(nodes, &lead, r.base, y);
        // How thick the trunk is here, before the pipe model says: enough to
        // start the branch at its bark.
        let bark = girth * (UNIT - 700 * y / h) / UNIT;
        // Three to a tier: the golden angle then sets each tier's nearest
        // branch a short step round from the one below, never over it.
        let limbs = 3;
        for k in 0..limbs {
            // Held close to the golden angle low down, where it makes the stair.
            let wander = if y < 4000 { 6 } else { 25 };
            let turn = phase + k * (1000 / limbs) + dice.range(-wander, wander);
            let dir = heading(turn);
            // The crown a cone: long low, short high; longer toward the light
            // and shorter into a neighbour's shade.
            let cone = ((h - y) * 33 / 100).clamp(800, 3200);
            let lit = UNIT + (dir[0] * r.light[0] + dir[2] * r.light[2]) / UNIT * 25 / 100;
            let mut long = cone * lit / UNIT;
            if y < 4000 {
                long = long.min(2600);
            }
            let start = add(p, scale(dir, bark * 8 / 10, UNIT));
            long = long.min(r.room(start, dir, long));
            if long < 500 {
                continue;
            }
            // Low limbs droop a little and high ones rise.
            let rise = -60 + 300 * y / h;
            limb(r, dice, nodes, leaves, from, start, dir, long, rise);
        }
        phase += GOLDEN;
        y += gap;
    }
    // The leader's tip: a last clump of needles.
    let tip = nodes[*lead.last().unwrap() as usize].at;
    leaves.push([tip[0], tip[1] + 200, tip[2], 500]);
}

/// **A Sentinel's limb**: out from `start` along `dir` for `long`, rising
/// `rise` in a thousand and drooping toward its tip, a side shoot each way
/// at every other node, foliage at every end. It bends up off ground it would
/// grow into.
#[allow(clippy::too_many_arguments)]
fn limb(
    r: &Reading,
    dice: &mut Dice,
    nodes: &mut Vec<Node>,
    leaves: &mut Vec<[i32; 4]>,
    from: u32,
    start: V,
    dir: V,
    long: i64,
    rise: i64,
) {
    let step = 400;
    // Its first node at the bark: the trunk holds it to there.
    nodes.push(Node {
        at: at(start),
        parent: from,
        radius: 0,
        part: Part::Limb,
    });
    let mut parent = nodes.len() as u32 - 1;
    let mut s = 0;
    let mut k = 0;
    while s < long {
        s = (s + step).min(long);
        let t = s * UNIT / long;
        let droop = long * 7 / 100 * t / UNIT * t / UNIT;
        let mut p = add(start, scale(dir, s, UNIT));
        p[1] += rise * s / UNIT - droop + dice.range(-15, 15);
        // A branch does not grow into the ground: it lifts off it.
        let floor = r.ground(p) + 250;
        let lifted = p[1] < floor;
        if lifted {
            p[1] = floor;
        }
        nodes.push(Node {
            at: at(p),
            parent,
            radius: 0,
            part: Part::Limb,
        });
        parent = nodes.len() as u32 - 1;
        k += 1;
        if k % 2 == 0 && s < long {
            for side in [-1i64, 1] {
                let turn = side * 190 + dice.range(-40, 40);
                let d = rotate(dir, turn);
                let shoot = ((long - s) * 35 / 100 + 200).min(1400);
                let tip = add(p, add(scale(d, shoot, UNIT), [0, shoot / 4, 0]));
                let mid = add(p, add(scale(d, shoot / 2, UNIT), [0, shoot / 10, 0]));
                twig(nodes, leaves, parent, &[mid, tip], 260 + shoot / 6);
            }
        }
        if lifted {
            break;
        }
    }
    let end = nodes[parent as usize].at;
    leaves.push([end[0], end[1] + 100, end[2], 450]);
}

/// A flat direction turned by thousandths of a turn.
fn rotate(dir: V, milli: i64) -> V {
    let h = heading(milli);
    // (c, s) applied to (x, z).
    let (c, s) = (h[0], h[2]);
    unit([
        (dir[0] * c - dir[2] * s) / UNIT,
        0,
        (dir[0] * s + dir[2] * c) / UNIT,
    ])
}

/// Twig nodes from a parent through `points`, foliage at the end.
fn twig(nodes: &mut Vec<Node>, leaves: &mut Vec<[i32; 4]>, from: u32, points: &[V], leaf: i64) {
    let mut parent = from;
    for &p in points {
        nodes.push(Node {
            at: at(p),
            parent,
            radius: 0,
            part: Part::Twig,
        });
        parent = nodes.len() as u32 - 1;
    }
    let end = nodes[parent as usize].at;
    leaves.push([end[0], end[1], end[2], leaf as i32]);
}

/// **The Elder and the Strangler**: a trunk, and a crown grown by space
/// colonisation into the room the site leaves it.
fn broadleaf(
    kind: Kind,
    r: &Reading,
    dice: &mut Dice,
    nodes: &mut Vec<Node>,
    leaves: &mut Vec<[i32; 4]>,
) {
    let h = r.site.height as i64;
    // The trunk's height, the crown's reach out and its depth, and how the
    // crown leans: an oak's crown is a dome, a banyan's a wide low roof.
    let (bole, reach, depth, up, flat) = match kind {
        Kind::Elder => (h * 30 / 100, h * 45 / 100, h * 70 / 100, 220, 1000),
        _ => (h * 38 / 100, h * 85 / 100, h * 40 / 100, 60, 550),
    };
    let lead = leader(r, dice, bole, 300, 30, nodes);
    let top = v(nodes[*lead.last().unwrap() as usize].at);
    // The crown's middle, pushed toward the light.
    let mid = add(
        add(top, [0, depth / 2, 0]),
        scale(r.light, reach * 22 / 100, UNIT),
    );
    let mut points: Vec<V> = Vec::new();
    let mut tries = 0;
    let want = 900;
    while points.len() < want && tries < want * 6 {
        tries += 1;
        let p = [
            dice.range(-reach, reach),
            dice.range(-depth / 2, depth / 2),
            dice.range(-reach, reach),
        ];
        // Inside the crown's ellipsoid.
        let e = p[0] * p[0] * UNIT / (reach * reach)
            + p[1] * p[1] * UNIT / (depth * depth / 4)
            + p[2] * p[2] * UNIT / (reach * reach);
        if e > UNIT {
            continue;
        }
        let p = add(mid, p);
        // Not in the ground, and seldom in a neighbour's shade.
        if p[1] < r.ground(p) + 600 {
            continue;
        }
        if r.shaded(p) && dice.range(0, 99) < 75 || r.against(p, 600) {
            continue;
        }
        points.push(p);
    }
    let first = lead[lead.len() / 3] as usize;
    colonise(r, nodes, &mut points, first, (2400, 650, 380), (up, flat));
    // Foliage at every branch's end.
    let mut children = vec![0u32; nodes.len()];
    for n in nodes.iter() {
        if n.parent != NONE {
            children[n.parent as usize] += 1;
        }
    }
    for (i, n) in nodes.iter().enumerate() {
        if n.part == Part::Limb && children[i] == 0 {
            let size = 420 + dice.range(0, 220) as i32;
            leaves.push([n.at[0], n.at[1], n.at[2], size]);
        }
    }
}

/// **Space colonisation** (Runions et al. 2007). Every point pulls the bud
/// nearest it, within `influence`; every bud pulled grows a step toward the
/// pull's mean, lifted by `up` (a thousandth of a step) and held toward the
/// level by `flat` (a thousand leaves it be); a point a branch comes within
/// `kill` of is taken. Buds are the nodes from `first` on.
fn colonise(
    r: &Reading,
    nodes: &mut Vec<Node>,
    points: &mut Vec<V>,
    first: usize,
    sizes: (i64, i64, i64),
    lean: (i64, i64),
) {
    let (influence, kill, step) = sizes;
    let (up, flat) = lean;
    let (inf2, kill2) = (influence * influence, kill * kill);
    // Each point's nearest bud, kept as buds are added.
    let mut nearest: Vec<(u32, i64)> = points
        .iter()
        .map(|&p| {
            let mut best = (NONE, i64::MAX);
            for (i, n) in nodes.iter().enumerate().skip(first) {
                let d = d2(p, v(n.at));
                if d < best.1 {
                    best = (i as u32, d);
                }
            }
            best
        })
        .collect();
    let mut alive = vec![true; points.len()];
    for _ in 0..90 {
        let mut pull: Vec<[i64; 4]> = vec![[0; 4]; nodes.len()];
        for (k, &p) in points.iter().enumerate() {
            let (n, d) = nearest[k];
            if !alive[k] || n == NONE || d > inf2 {
                continue;
            }
            let u = unit(sub(p, v(nodes[n as usize].at)));
            let a = &mut pull[n as usize];
            a[0] += u[0];
            a[1] += u[1];
            a[2] += u[2];
            a[3] += 1;
        }
        let before = nodes.len();
        for (i, a) in pull.iter().enumerate() {
            if a[3] == 0 {
                continue;
            }
            let mut d = unit([a[0], a[1], a[2]]);
            if len(d) == 0 {
                continue;
            }
            d[1] = (d[1] + up) * flat / UNIT;
            let d = unit(d);
            let mut p = add(v(nodes[i].at), scale(d, step, UNIT));
            let floor = r.ground(p) + 250;
            if p[1] < floor {
                p[1] = floor;
            }
            // A branch stops short of a neighbour's trunk.
            if r.against(p, 250) {
                continue;
            }
            nodes.push(Node {
                at: at(p),
                parent: i as u32,
                radius: 0,
                part: Part::Limb,
            });
        }
        if nodes.len() == before {
            break;
        }
        for (k, &p) in points.iter().enumerate() {
            if !alive[k] {
                continue;
            }
            for (i, n) in nodes.iter().enumerate().skip(before) {
                let d = d2(p, v(n.at));
                if d < kill2 {
                    alive[k] = false;
                }
                if d < nearest[k].1 {
                    nearest[k] = (i as u32, d);
                }
            }
        }
        if nodes.len() > 4000 {
            break;
        }
    }
}

/// **The pipe model**: a node is as thick as all the ends it carries, a
/// branch's cross-section the sum of its children's, scaled so the trunk's
/// foot is `girth` across (a radius). Roots and props are not in it here.
fn pipe(nodes: &mut [Node], girth: i32) {
    let mut ends = vec![0i64; nodes.len()];
    let mut children = vec![0u32; nodes.len()];
    for n in nodes.iter() {
        if n.parent != NONE {
            children[n.parent as usize] += 1;
        }
    }
    for i in (0..nodes.len()).rev() {
        if children[i] == 0 {
            ends[i] += 1;
        }
        let p = nodes[i].parent;
        if p != NONE {
            ends[p as usize] += ends[i];
        }
    }
    let foot = isqrt(ends[0].max(1) * 10_000).max(1);
    for (i, n) in nodes.iter_mut().enumerate() {
        let r = isqrt(ends[i] * 10_000) * girth as i64 / foot;
        n.radius = r.max(5) as i32;
    }
}

/// **A strangler's prop roots**: from great limbs well out from the trunk, a
/// root straight down to the ground, thickening into a pillar.
fn props(r: &Reading, dice: &mut Dice, nodes: &mut Vec<Node>) {
    let mut placed: Vec<V> = Vec::new();
    let count = nodes.len();
    for i in 0..count {
        let n = nodes[i];
        if n.part != Part::Limb || n.radius < 60 {
            continue;
        }
        let p = v(n.at);
        let out = flat_len(sub(p, r.base));
        if out < 1500 || p[1] - r.ground(p) < 800 {
            continue;
        }
        if placed.len() >= 10
            || placed.iter().any(|q| flat_len(sub(*q, p)) < 2000)
            || dice.range(0, 99) < 35
        {
            continue;
        }
        placed.push(p);
        let ground = r.ground(p);
        let thick = (n.radius * 7 / 10).clamp(45, 140);
        let mut parent = i as u32;
        let mut y = p[1] - 600;
        while y > ground - 60 {
            nodes.push(Node {
                at: [p[0] as i32, y.max(ground - 60) as i32, p[2] as i32],
                parent,
                radius: thick,
                part: Part::Prop,
            });
            parent = nodes.len() as u32 - 1;
            y -= 600;
        }
        nodes.push(Node {
            at: [p[0] as i32, (ground - 60) as i32, p[2] as i32],
            parent,
            radius: thick * 13 / 10,
            part: Part::Prop,
        });
    }
}

/// **The roots**: buttresses off the trunk's foot, running out over the
/// ground and following it, downhill rather than up, round a neighbour's
/// foot rather than into it, forking on the way.
fn roots(kind: Kind, r: &Reading, dice: &mut Dice, nodes: &mut Vec<Node>) {
    let girth = r.site.girth as i64;
    let (count, spread) = match kind {
        Kind::Sentinel => (7, 400),
        Kind::Elder => (9, 480),
        Kind::Strangler => (10, 560),
    };
    let first = nodes.len();
    let mut phase = dice.range(0, 999);
    // Down the slope is where a root runs furthest.
    let g = r.site.ground;
    let (x, z) = (r.base[0] as i32, r.base[2] as i32);
    let downhill = unit([
        -(g(x + 800, z) - g(x - 800, z)) as i64,
        0,
        -(g(x, z + 800) - g(x, z - 800)) as i64,
    ]);
    for _ in 0..count {
        let dir = heading(phase + dice.range(-40, 40));
        phase += GOLDEN;
        let along = (dir[0] * downhill[0] + dir[2] * downhill[2]) / UNIT;
        let long = girth * spread / 100 * (UNIT + along * 35 / 100) / UNIT;
        root(r, dice, nodes, 0, dir, long, downhill, true);
    }
    // Their thickness: the pipe model again, the roots' cross-sections
    // together most of the trunk's.
    let mut ends = vec![0i64; nodes.len()];
    let mut children = vec![0u32; nodes.len()];
    for n in nodes.iter().skip(first) {
        if (n.parent as usize) >= first {
            children[n.parent as usize] += 1;
        }
    }
    for i in (first..nodes.len()).rev() {
        if children[i] == 0 {
            ends[i] += 1;
        }
        let p = nodes[i].parent as usize;
        if p >= first {
            ends[p] += ends[i];
        }
    }
    let total: i64 = (first..nodes.len())
        .filter(|&i| (nodes[i].parent as usize) < first)
        .map(|i| ends[i])
        .sum::<i64>()
        .max(1);
    for i in first..nodes.len() {
        let share = isqrt(ends[i] * 1_000_000 / total);
        nodes[i].radius = ((girth * 85 / 100) * share / 1000).max(8) as i32;
    }
    // Half buried: each root node a third of its thickness into the ground,
    // and the buttress at its foot flaring up the trunk.
    for i in first..nodes.len() {
        let n = nodes[i];
        let p = v(n.at);
        let gy = r.ground(p);
        let out = flat_len(sub(p, r.base));
        let flare = if out < girth * 2 {
            (girth * 2 - out) * 45 / 100
        } else {
            0
        };
        nodes[i].at[1] = (gy - n.radius as i64 / 3 + flare) as i32;
    }
}

/// One root: out from `from` along `dir` for `long`, a node every metre and
/// a half on the ground, bending downhill and round other trees' feet,
/// forking once or twice if `forks`.
#[allow(clippy::too_many_arguments)]
fn root(
    r: &Reading,
    dice: &mut Dice,
    nodes: &mut Vec<Node>,
    from: u32,
    dir: V,
    long: i64,
    downhill: V,
    forks: bool,
) {
    let girth = r.site.girth as i64;
    let start = if from == 0 {
        add(r.base, scale(dir, girth * 7 / 10, UNIT))
    } else {
        v(nodes[from as usize].at)
    };
    let mut p = start;
    let mut d = dir;
    let mut parent = from;
    let mut s = 0;
    let step = 160;
    let mut k = 0;
    while s < long {
        s += step;
        k += 1;
        let wob = [dice.range(-120, 120), 0, dice.range(-120, 120)];
        let mut nd = add(add(d, scale(downhill, 120, UNIT)), wob);
        for n in r.site.neighbours {
            let o = sub([p[0], 0, p[2]], [n.at.0 as i64, 0, n.at.1 as i64]);
            let off = flat_len(o);
            if off < n.girth as i64 + 500 {
                nd = add(nd, scale(unit(o), 600, UNIT));
            }
        }
        d = unit([nd[0], 0, nd[2]]);
        p = add(p, scale(d, step, UNIT));
        p[1] = r.ground(p);
        nodes.push(Node {
            at: at(p),
            parent,
            radius: 0,
            part: Part::Root,
        });
        parent = nodes.len() as u32 - 1;
        if forks && k > 3 && k % 6 == 0 && dice.range(0, 99) < 60 {
            let side = if dice.range(0, 1) == 0 { -120 } else { 120 };
            let bd = rotate(d, side);
            root(
                r,
                dice,
                nodes,
                parent,
                bd,
                (long - s) * 45 / 100,
                downhill,
                false,
            );
        }
    }
}

/// **Where cairns go**: on the highest thick, level limbs, and two on the
/// way up, each well apart.
fn cairns(nodes: &[Node], kind: Kind) -> Vec<[i32; 3]> {
    let base = nodes[0].at[1];
    let thick = match kind {
        Kind::Sentinel => 70,
        _ => 80,
    };
    let mut level: Vec<(i32, [i32; 3])> = nodes
        .iter()
        .filter(|n| n.part == Part::Limb && n.radius >= thick && n.parent != NONE)
        .filter(|n| {
            let p = nodes[n.parent as usize].at;
            let rise = (n.at[1] - p[1]).abs() as i64;
            let run = flat_len(sub(v(n.at), v(p)));
            rise * 4 < run
        })
        .map(|n| (n.at[1], [n.at[0], n.at[1] + n.radius, n.at[2]]))
        .collect();
    level.sort_by_key(|(y, _)| -*y);
    let mut out: Vec<[i32; 3]> = Vec::new();
    for (y, p) in level {
        if out.len() >= 3 {
            break;
        }
        if out.iter().all(|q| (q[1] - p[1]).abs() > 3000) && y - base > 2500 {
            out.push(p);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Stood on
// ---------------------------------------------------------------------------

/// How thick a part must be to be stood on and run into: thinner, it is
/// drawn and nothing more -- you fall through a twig.
pub const STOOD_ON: i32 = 45;

fn cm(v: i64) -> Fx {
    Fx::from_raw((v * 65536 / 100) as i32)
}

/// **A tree's solids**: every segment thick enough to stand on, as what a
/// body can collide with. A steep one is a stack of round slices up its
/// axis; a gentle one a run of turned boards along it, each its own top at
/// the branch's top and no more than a step above the last, so a limb is a
/// slope you walk up. Roots the same, hugging the ground.
pub fn solids(tree: &Tree, out: &mut Vec<Solid>) {
    // A broadleaf's limbs are thick a long way out: only the great ones are
    // stood on. A root only near the trunk, where it is a buttress.
    let thick = match tree.kind {
        Kind::Sentinel => STOOD_ON as i64,
        Kind::Elder => STOOD_ON as i64 * 2,
        Kind::Strangler => STOOD_ON as i64 * 5 / 2,
    };
    let nodes = &tree.nodes;
    let girth = nodes[0].radius as i64;
    let mut children = vec![0u32; nodes.len()];
    for n in nodes.iter() {
        if n.parent != NONE {
            children[n.parent as usize] += 1;
        }
    }
    let stood = |i: usize| {
        let n = nodes[i];
        if n.parent == NONE || n.part == Part::Twig {
            return false;
        }
        let p = nodes[n.parent as usize];
        // Where a branch leaves the trunk it is inside it.
        if p.part == Part::Trunk && n.part != Part::Trunk {
            return false;
        }
        if (p.radius.max(n.radius) as i64) < thick {
            return false;
        }
        !(n.part == Part::Root
            && ((n.radius as i64) < thick * 4 / 3
                || flat_len(sub(v(n.at), v(tree.base))) > girth * 3))
    };
    let steep = |i: usize| {
        let n = nodes[i];
        let d = sub(v(n.at), v(nodes[n.parent as usize].at));
        d[1].abs() * 2 > flat_len(d)
    };
    // The one child of a node that carries on its run, if it has exactly one.
    let only = |i: usize| -> Option<usize> {
        if children[i] != 1 {
            return None;
        }
        (i + 1..nodes.len()).find(|&c| nodes[c].parent == i as u32)
    };
    let mut done = vec![false; nodes.len()];
    for i in 0..nodes.len() {
        if done[i] || !stood(i) {
            continue;
        }
        done[i] = true;
        // A run: this segment and the ones carrying straight on from it, the
        // same part and the same lie, until it is long enough to be one piece.
        let first = nodes[i].parent as usize;
        let mut last = i;
        let up = steep(i);
        let most = if up { 1200 } else { 900 };
        while let Some(c) = only(last) {
            let span = len(sub(v(nodes[c].at), v(nodes[first].at)));
            if done[c]
                || !stood(c)
                || steep(c) != up
                || nodes[c].part != nodes[i].part
                || span > most
            {
                break;
            }
            done[c] = true;
            last = c;
        }
        let (a, b) = (v(nodes[first].at), v(nodes[last].at));
        let (r0, r1) = (nodes[first].radius as i64, nodes[last].radius as i64);
        let d = sub(b, a);
        let run = flat_len(d);
        let rise = d[1].abs();
        if up {
            // Steep: round slices up its axis.
            let r = r0.min(r1).max(1);
            let pieces = (rise / (r.max(200) * 4)).clamp(1, 4);
            for k in 0..pieces {
                let (t0, t1) = (k * UNIT / pieces, (k + 1) * UNIT / pieces);
                let c = add(a, scale(d, (t0 + t1) / 2, UNIT));
                let rk = r0 + (r1 - r0) * (t0 + t1) / 2 / UNIT;
                let (y0, y1) = (a[1] + d[1] * t0 / UNIT, a[1] + d[1] * t1 / UNIT);
                let (lo, hi) = (y0.min(y1), y0.max(y1));
                out.push(Solid::round(
                    cm(c[0]),
                    cm(c[2]),
                    cm(lo - 20),
                    cm(hi + 20),
                    cm(rk),
                    cm(rk),
                    Material::Wood,
                ));
            }
        } else {
            // Gentle: boards, each a step up from the last at most.
            let pieces = (rise / 40).max(run / 900).clamp(1, 8);
            let yaw =
                crate::math::atan2_turns(Fx::from_int(d[2] as i32), Fx::from_int(d[0] as i32));
            for k in 0..pieces {
                let (t0, t1) = (k * UNIT / pieces, (k + 1) * UNIT / pieces);
                let c = add(a, scale(d, (t0 + t1) / 2, UNIT));
                let rk = (r0 + (r1 - r0) * (t0 + t1) / 2 / UNIT).max(STOOD_ON as i64);
                let half = run / pieces / 2 + 10;
                out.push(Solid::turned(
                    cm(c[0]),
                    cm(c[2]),
                    cm(c[1] - rk),
                    cm(c[1] + rk * 9 / 10),
                    cm(half),
                    cm(rk * 8 / 10),
                    yaw,
                    Material::Wood,
                ));
            }
        }
    }
}

/// A tree's cairns: a slab of snow on each of its [`Tree::cairns`].
pub fn cairns_of(tree: &Tree, out: &mut Vec<Solid>) {
    for c in &tree.cairns {
        let (x, y, z) = (c[0] as i64, c[1] as i64, c[2] as i64);
        out.push(Solid {
            min: V3::new(cm(x - 100), cm(y - 30), cm(z - 100)),
            max: V3::new(cm(x + 100), cm(y + 40), cm(z + 100)),
            material: Material::Snow,
            shape: Shape::Box,
        });
    }
}

/// **A vine up a trunk's north side**, from under its foot to sixty metres:
/// the slow way past a Sentinel's first tiers, for anybody.
pub fn vine(tree: &Tree, girth: i32) -> Zone {
    let [x, y, z] = tree.base;
    let out = girth * 14 / 10 + 80;
    Zone::cm([x - 120, y - 2000, z], [x + 120, y + 6000, z + out])
}
