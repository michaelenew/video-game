//! **The land**: the valley's ground as one surface, not a floor per room.
//!
//! The valley was rooms of boxes: a flat floor, cliffs either side as walls,
//! a notch for every door. What makes an open field read as a place you could
//! walk across -- the Guild Wars kind, the Final Fantasy kind -- is the
//! ground itself: a path that winds along the bottom of a valley, slopes that
//! rise off it into hills and then mountains, a river cut into the meadow,
//! clearings where things happen. So the ground between places is a height
//! function, worked out here, from three kinds of thing:
//!
//! - **A way**: a line of points, each with a floor height and a half-width.
//!   Along it the floor is that height, flat across its width and easing from
//!   point to point; off its edge the ground rises, gently and then steeply
//!   (`rise`). The main road and every side path to a room is a way.
//! - **A pad**: a place that keeps its own floor -- the town, the Ring, a
//!   creature's room -- with a ring of level ground round it. Inside, the
//!   place's own relief is the floor (`atlas`); outside, the ground rises off
//!   its edge the way it rises off a way.
//! - **Water**: a way whose floor is a bed, with a surface over it. A river
//!   is one cut through a meadow; a lake is a short wide one.
//!
//! The ground anywhere is the **lowest** of what every way and pad near it
//! would make it: two valleys meet as a saddle, a side path is a notch in a
//! hillside, and the mountains are what is left between them. Over that lies
//! a little noise -- a hand's breadth on a floor, metres on a mountainside --
//! so nothing is a plane. Everything is fixed point and hashes integers, so
//! both machines raise the same mountains.
//!
//! A slope too steep to walk is a wall (`arena::Terrain`, `tuning`): that is
//! what the mountains are for.

use crate::fixed::Fx;
use crate::math::V3;

/// A point of a way, in centimetres on the map: where, how high its floor is,
/// and how far either side of the line that floor runs flat.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: i32,
    pub z: i32,
    pub y: i32,
    pub half: i32,
}

/// A point, from centimetres.
pub const fn at(x: i32, z: i32, y: i32, half: i32) -> Point {
    Point { x, z, y, half }
}

/// A way: a line of points, a path along it or not, and water over it or not.
#[derive(Clone, Debug)]
pub struct Way {
    pub points: Vec<Point>,
    /// Trodden: a dirt path runs down its middle.
    pub path: bool,
    /// Water this deep over its floor, in centimetres: a river, a lake.
    pub water: Option<i32>,
}

/// A place that keeps its own floor, as the land sees it: its footprint on
/// the map (the ground round it included) and the height of its floor.
#[derive(Clone, Copy, Debug)]
pub struct Pad {
    pub lo: (Fx, Fx),
    pub hi: (Fx, Fx),
    pub y: Fx,
    /// **Its rim** (`arena::rim`), and where the place's origin is on the
    /// map: the bank round a room, which the land holds up against the
    /// valley round it -- a clearing has an edge -- except where a trodden
    /// way comes through it. `None` for level ground running off into the
    /// foothills, as round the town.
    pub rim: Option<(crate::arena::rim::Rim, (Fx, Fx))>,
    /// Where on the map its door is: the one place a trodden way may cut
    /// its rim.
    pub door: (Fx, Fx),
}

/// How far from its door a way may cut a rim.
const DOORWAY: Fx = Fx::from_int(20);

/// How steeply the land a rim holds up falls away on its far side.
const FALL: Fx = Fx::from_int(3);

/// One straight piece of a way, ready to measure against.
#[derive(Clone, Copy, Debug)]
struct Seg {
    a: (i64, i64),
    b: (i64, i64),
    ya: Fx,
    yb: Fx,
    ha: Fx,
    hb: Fx,
    path: bool,
    water: Option<Fx>,
}

/// How steep a mountainside gets: its slope, far from the floor.
const STEEP: Fx = Fx::ratio(7, 5);

/// How far off a floor's edge the ground takes to turn from level to steep.
const FOOT: Fx = Fx::from_int(5);

/// How steeply an embankment falls off a raised path's edge: a drop, not a
/// slope anybody walks up.
const BANK: Fx = Fx::from_int(1);

/// How steep the foothills are: walkable, and a climb.
const HILL: Fx = Fx::ratio(8, 25);

/// How far the foothills run off a floor's edge before the mountain starts.
const HILL_RUN: Fx = Fx::from_int(16);

/// How high the ground is where nothing is near: over every mountain.
const SKY_HIGH: Fx = Fx::from_int(180);

/// How far a way or a pad shapes the ground round it, past its own edge.
/// Further than this its mountainside is above [`SKY_HIGH`] anyway.
const REACH: Fx = Fx::from_int(126);

/// The side of a cell of the land's own index, in metres.
const CELL_M: i64 = 32;

/// How wide the trodden path down a way's middle is, either side.
pub const PATH_HALF: Fx = Fx::ratio(8, 5);

/// **The land**, ready to ask.
#[derive(Clone, Debug, Default)]
pub struct Land {
    segs: Vec<Seg>,
    pads: Vec<Pad>,
    x0: i64,
    z0: i64,
    w: i64,
    h: i64,
    starts: Vec<u32>,
    entries: Vec<u16>,
}

/// What the land is at a point.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    /// The height of the ground.
    pub height: Fx,
    /// How far above the floor it rises from: zero on a floor, the height
    /// of the mountainside on one.
    pub rise: Fx,
    /// How far from the middle of the nearest trodden way, flat.
    pub path: Fx,
    /// The water's surface, if the ground here is under it.
    pub water: Option<Fx>,
}

/// The integer square root of a non-negative number, by Newton's method from
/// a power of two above it.
fn isqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let bits = 128 - n.leading_zeros();
    let mut x = 1u128 << bits.div_ceil(2);
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// How far the land has risen `s` metres off a floor's edge: **foothills**
/// first -- [`HILL`], walkable, for [`HILL_RUN`] metres, easing in from level
/// -- and then the mountainside, turning to [`STEEP`] over about [`FOOT`]
/// metres. The foothills are what make a valley a place to wander rather than
/// a corridor; the mountains are its edge.
pub fn rise(s: Fx) -> Fx {
    if s.raw() <= 0 {
        return Fx::ZERO;
    }
    let ease = Fx::from_int(4);
    let hills = HILL.mul(s.min(HILL_RUN)).mul(s.div(s.add(ease)));
    let t = s.sub(HILL_RUN);
    if t.raw() <= 0 {
        return hills;
    }
    hills.add(STEEP.mul(t).mul(t.div(t.add(FOOT))))
}

/// A hash of two integers and a seed: a number in [-1, 1].
fn lattice(i: i64, j: i64, seed: u64) -> i32 {
    let mut h = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (j as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ seed.wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 29;
    ((h >> 40) as i64 % 131_073 - 65_536) as i32
}

/// Smooth value noise at a map point, cells `cell` metres across: [-1, 1].
pub(crate) fn value(x: Fx, z: Fx, cell: i64, seed: u64) -> Fx {
    let size = cell * Fx::ONE.raw() as i64;
    let (xr, zr) = (x.raw() as i64, z.raw() as i64);
    let (i, j) = (xr.div_euclid(size), zr.div_euclid(size));
    let fx = Fx::from_raw(((xr - i * size) * Fx::ONE.raw() as i64 / size) as i32);
    let fz = Fx::from_raw(((zr - j * size) * Fx::ONE.raw() as i64 / size) as i32);
    let ease = |t: Fx| t.mul(t).mul(Fx::from_int(3).sub(t.add(t)));
    let (sx, sz) = (ease(fx), ease(fz));
    let v = |a: i64, b: i64| Fx::from_raw(lattice(a, b, seed));
    let top = crate::math::lerp(v(i, j), v(i + 1, j), sx);
    let bottom = crate::math::lerp(v(i, j + 1), v(i + 1, j + 1), sx);
    crate::math::lerp(top, bottom, sz)
}

/// Three octaves: hills, hummocks, lumps. About [-1, 1].
pub(crate) fn noise(x: Fx, z: Fx) -> Fx {
    let a = value(x, z, 48, 1);
    let b = value(x, z, 17, 2);
    let c = value(x, z, 6, 3);
    a.mul(Fx::ratio(4, 7))
        .add(b.mul(Fx::ratio(2, 7)))
        .add(c.mul(Fx::ratio(1, 7)))
}

fn cm(v: i32) -> Fx {
    Fx::ratio(v, 100)
}

impl Land {
    /// **Lay out the land** from its ways and the pads of the places that
    /// keep their own floors.
    pub fn new(ways: &[Way], pads: Vec<Pad>) -> Land {
        let mut segs = Vec::new();
        for way in ways {
            for pair in way.points.windows(2) {
                let (p, q) = (pair[0], pair[1]);
                let raw = |v: i32| cm(v).raw() as i64;
                segs.push(Seg {
                    a: (raw(p.x), raw(p.z)),
                    b: (raw(q.x), raw(q.z)),
                    ya: cm(p.y),
                    yb: cm(q.y),
                    ha: cm(p.half),
                    hb: cm(q.half),
                    path: way.path,
                    water: way.water.map(cm),
                });
            }
        }
        let mut land = Land {
            segs,
            pads,
            ..Land::default()
        };
        land.index();
        land
    }

    /// Each feature's reach, as a box of land cells.
    fn boxes(&self) -> Vec<(i64, i64, i64, i64)> {
        let reach = REACH.raw() as i64;
        let mut out = Vec::new();
        for s in &self.segs {
            let pad = reach + s.ha.max(s.hb).raw() as i64;
            out.push((
                s.a.0.min(s.b.0) - pad,
                s.a.1.min(s.b.1) - pad,
                s.a.0.max(s.b.0) + pad,
                s.a.1.max(s.b.1) + pad,
            ));
        }
        for p in &self.pads {
            out.push((
                p.lo.0.raw() as i64 - reach,
                p.lo.1.raw() as i64 - reach,
                p.hi.0.raw() as i64 + reach,
                p.hi.1.raw() as i64 + reach,
            ));
        }
        out
    }

    fn cell(v: i64) -> i64 {
        v.div_euclid(CELL_M * Fx::ONE.raw() as i64)
    }

    fn index(&mut self) {
        let boxes = self.boxes();
        if boxes.is_empty() {
            return;
        }
        let (mut x0, mut z0, mut x1, mut z1) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
        for b in &boxes {
            x0 = x0.min(Self::cell(b.0));
            z0 = z0.min(Self::cell(b.1));
            x1 = x1.max(Self::cell(b.2));
            z1 = z1.max(Self::cell(b.3));
        }
        let (w, h) = (x1 - x0 + 1, z1 - z0 + 1);
        let mut lists = vec![Vec::new(); (w * h) as usize];
        for (k, b) in boxes.iter().enumerate() {
            for cz in Self::cell(b.1)..=Self::cell(b.3) {
                for cx in Self::cell(b.0)..=Self::cell(b.2) {
                    lists[((cz - z0) * w + (cx - x0)) as usize].push(k as u16);
                }
            }
        }
        let mut starts = Vec::with_capacity(lists.len() + 1);
        let mut entries = Vec::new();
        starts.push(0);
        for l in lists {
            entries.extend(l);
            starts.push(entries.len() as u32);
        }
        (self.x0, self.z0, self.w, self.h) = (x0, z0, w, h);
        self.starts = starts;
        self.entries = entries;
    }

    /// The features near a map point, by index: ways' pieces first, then
    /// pads.
    fn near(&self, x: Fx, z: Fx) -> &[u16] {
        let (cx, cz) = (
            Self::cell(x.raw() as i64) - self.x0,
            Self::cell(z.raw() as i64) - self.z0,
        );
        if cx < 0 || cz < 0 || cx >= self.w || cz >= self.h {
            return &[];
        }
        let c = (cz * self.w + cx) as usize;
        &self.entries[self.starts[c] as usize..self.starts[c + 1] as usize]
    }

    /// Is a map point on one of the pads -- a place with its own floor?
    pub fn on_pad(&self, x: Fx, z: Fx) -> Option<&Pad> {
        self.pads.iter().find(|p| {
            x.raw() >= p.lo.0.raw()
                && x.raw() <= p.hi.0.raw()
                && z.raw() >= p.lo.1.raw()
                && z.raw() <= p.hi.1.raw()
        })
    }

    /// **What the land is at a map point** (x, z).
    pub fn sample(&self, x: Fx, z: Fx) -> Sample {
        let (px, pz) = (x.raw() as i64, z.raw() as i64);
        let mut best = SKY_HIGH;
        let mut best_rise = SKY_HIGH;
        // How close the lowest feature's floor edge is, for a pad: noise
        // fades in off a pad's edge, so the ground meets its floor exactly.
        let mut by_pad: Option<Fx> = None;
        let mut path = SKY_HIGH;
        let mut water: Option<Fx> = None;
        // The highest any trodden way or pad holds the ground up to: its
        // floor on it, falling away off its edge as an embankment. What lets
        // a path climb above the valley it leaves, to a room on a rise.
        let mut fill = Fx::from_int(-10_000);
        // The highest a rim holds the ground up to here, and the lowest a
        // trodden way would have it: the rim stands except where a path
        // comes through it.
        let mut held = Fx::from_int(-10_000);
        let mut trail = SKY_HIGH;
        for &k in self.near(x, z) {
            let k = k as usize;
            if k < self.segs.len() {
                let s = &self.segs[k];
                let (vx, vz) = (s.b.0 - s.a.0, s.b.1 - s.a.1);
                let (wx, wz) = (px - s.a.0, pz - s.a.1);
                let len2 = vx as i128 * vx as i128 + vz as i128 * vz as i128;
                let dot = wx as i128 * vx as i128 + wz as i128 * vz as i128;
                let t = if len2 == 0 {
                    0
                } else {
                    (dot.clamp(0, len2) * Fx::ONE.raw() as i128 / len2) as i32
                };
                let t = Fx::from_raw(t);
                let cx = s.a.0 as i128 + vx as i128 * t.raw() as i128 / Fx::ONE.raw() as i128;
                let cz = s.a.1 as i128 + vz as i128 * t.raw() as i128 / Fx::ONE.raw() as i128;
                let (dx, dz) = (px as i128 - cx, pz as i128 - cz);
                let d = isqrt((dx * dx + dz * dz) as u128).min(i32::MAX as u128) as i32;
                let d = Fx::from_raw(d);
                let ease = t.mul(t).mul(Fx::from_int(3).sub(t.add(t)));
                let floor = crate::math::lerp(s.ya, s.yb, ease);
                let half = crate::math::lerp(s.ha, s.hb, t);
                let r = rise(d.sub(half));
                let h = floor.add(r);
                if h.raw() < best.raw() {
                    best = h;
                    best_rise = r;
                    by_pad = None;
                }
                if s.path && s.water.is_none() && h.raw() < trail.raw() {
                    trail = h;
                }
                if s.path && s.water.is_none() && d.raw() < path.raw() {
                    path = d;
                }
                if s.path && s.water.is_none() {
                    let off = d.sub(half).max(Fx::ZERO);
                    fill = fill.max(floor.sub(off.mul(BANK)));
                }
                if let Some(depth) = s.water {
                    if d.raw() <= half.raw() {
                        let surface = floor.add(depth);
                        if water.is_none_or(|w| surface.raw() > w.raw()) {
                            water = Some(surface);
                        }
                    }
                }
            } else {
                let p = &self.pads[k - self.segs.len()];
                let dx = p.lo.0.sub(x).max(x.sub(p.hi.0)).max(Fx::ZERO);
                let dz = p.lo.1.sub(z).max(z.sub(p.hi.1)).max(Fx::ZERO);
                let d = crate::math::wide_len(V3::new(dx, Fx::ZERO, dz));
                let r = match &p.rim {
                    Some((rim, at)) => {
                        let (lx, lz) = (x.sub(at.0), z.sub(at.1));
                        let r = rim.height(lx, lz).unwrap_or(Fx::ZERO);
                        // Held up for its bank and a little past its
                        // crest, then falling away to whatever is round it.
                        let band = rim.run().add(crate::arena::rim::HELD);
                        let up = if d.raw() <= band.raw() {
                            r
                        } else {
                            let foot = rim.foot_at(lx, lz);
                            let crest = foot.add(rim.rise(band, lx, lz));
                            crest.sub(d.sub(band).mul(FALL))
                        };
                        // Only its own path, at its own door, cuts it.
                        let to_door = crate::math::wide_len(V3::new(
                            x.sub(p.door.0),
                            Fx::ZERO,
                            z.sub(p.door.1),
                        ));
                        let up = p.y.add(up);
                        let up = if to_door.raw() < DOORWAY.raw() {
                            up.min(trail)
                        } else {
                            up
                        };
                        held = held.max(up);
                        r
                    }
                    None => rise(d),
                };
                let h = p.y.add(r);
                if h.raw() < best.raw() {
                    best = h;
                    best_rise = r;
                    by_pad = Some(d);
                }
                fill = fill.max(p.y.sub(d.mul(BANK)));
            }
        }
        if fill.raw() > best.raw() {
            best = fill;
            best_rise = Fx::ZERO;
        }
        // A rim stands over the valley round it, but its path cuts it.
        let rimmed = held.raw() > best.raw();
        if rimmed {
            best = held;
            best_rise = Fx::ZERO;
        }
        // A hand's breadth on a floor, metres up a mountainside.
        let amp = Fx::ratio(3, 10).add(best_rise.min(Fx::from_int(60)).mul(Fx::ratio(3, 25)));
        let amp = match by_pad {
            // A rim carries its own noise.
            _ if rimmed => Fx::ZERO,
            Some(d) => amp.mul(d.div(Fx::from_int(6)).min(Fx::ONE)),
            None => amp,
        };
        // Up a mountainside, crests: a ridged noise -- the folds where two
        // faces meet stand out -- growing with the height of the mountain.
        let crag = if best_rise.raw() > Fx::from_int(6).raw() {
            let r = Fx::ONE.sub(value(x, z, 29, 4).abs());
            r.mul(r).mul(
                best_rise
                    .sub(Fx::from_int(6))
                    .min(Fx::from_int(80))
                    .mul(Fx::ratio(1, 5)),
            )
        } else {
            Fx::ZERO
        };
        let height = best.add(noise(x, z).mul(amp)).add(crag);
        Sample {
            height,
            rise: best_rise,
            path,
            water: water.filter(|w| height.raw() < w.raw()),
        }
    }

    /// The ground's height at a map point.
    pub fn height(&self, x: Fx, z: Fx) -> Fx {
        self.sample(x, z).height
    }

    /// **How steep the ground is** at a map point: the larger of its rises
    /// across half a metre either way, as a slope.
    pub fn steepness(&self, x: Fx, z: Fx) -> Fx {
        let e = Fx::ratio(1, 2);
        let h = |x: Fx, z: Fx| self.height(x, z);
        let gx = h(x.add(e), z).sub(h(x.sub(e), z)).abs();
        let gz = h(x, z.add(e)).sub(h(x, z.sub(e))).abs();
        gx.max(gz)
    }

    /// The ways' pieces, for a renderer or a tool: from, to, and whether
    /// water lies along it.
    pub fn pieces(&self) -> impl Iterator<Item = (V3, V3, bool)> + '_ {
        self.segs.iter().map(|s| {
            (
                V3::new(Fx::from_raw(s.a.0 as i32), s.ya, Fx::from_raw(s.a.1 as i32)),
                V3::new(Fx::from_raw(s.b.0 as i32), s.yb, Fx::from_raw(s.b.1 as i32)),
                s.water.is_some(),
            )
        })
    }
}
