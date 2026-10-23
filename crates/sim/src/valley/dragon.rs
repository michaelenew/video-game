//! **The Sleeper**: a great beast asleep in the mountains north of the
//! Pinewood, which is one of those mountains (the owner, 2026-10-22: "the
//! dragon itself should essentially be the mountain, and its dragon like
//! features should be just discernible only from certain angles").
//!
//! It is the land, not a creature: a height added to the mountain wherever
//! its body lies, before the land's noise and crags are laid over it, so it
//! wears the same rock, snow and broken crests as the range round it and
//! what is drawn is what a foot would stand on.
//!
//! **Its shape is a creature built for it** (2026-10-23): a giant armoured
//! lizard, a skeleton with flesh, plates, horns and a clubbed tail on it
//! (`scripts/sleeper/beast.py`), asleep on its belly with its head on its
//! forearms, looked down on by `scripts/sleeper/terrain.py`, which writes
//! `sleeper.bin`: how far its body stands over the mountain, cell by cell.
//! Change the beast, its pose or where it lies there, not here. The
//! heights are level data (`tests/knobs.rs`), judged by looking at them.
//!
//! **Its horns are stone** (the owner: "I want the horn to read as a
//! natural arch in the stone the way it's resting"): the land is one height
//! a point and cannot arch, so each horn standing clear of its body is a
//! curve (`sleeper_arch.txt`, from the same script) that [`arch_rock`]
//! stands as rock -- turned boxes along it, which a body stands on and walks
//! under -- and the renderer draws as one piece of stone.

use crate::arena::{Material, Solid};
use crate::fixed::Fx;
use crate::math::V3;

/// The heightfield `scripts/sleeper/terrain.py` writes: a header of six
/// i32 and a u16 a cell.
static FIELD: &[u8] = include_bytes!("sleeper.bin");

/// The horns' curves `scripts/sleeper/terrain.py` writes: a horn a block of
/// lines, each point x and z on the map, its height over the mountain and
/// its radius, centimetres.
static ARCH: &str = include_str!("sleeper_arch.txt");

/// How long a piece of a horn's rock is, at most, as a share of the horn's
/// radius there: shorter is a smoother arch and more boxes.
const ROCK_STEP: Fx = Fx::ratio(3, 5);

/// How far a piece of rock reaches above and below the line of the horn,
/// and either side of it, as a share of its radius: less than the radius,
/// so the boxes sit inside the stone that is drawn.
const ROCK_DEEP: Fx = Fx::ratio(3, 4);
const ROCK_WIDE: Fx = Fx::ratio(4, 5);

/// **How much of the Sleeper shows over the mountain**, by how high the
/// mountain has risen off its floor there: none of it until the mountain is
/// this high, so no valley or foothill is touched, all of it by
/// [`FULL_FROM`].
const SHOWS_FROM: Fx = Fx::from_int(40);
const FULL_FROM: Fx = Fx::from_int(100);

fn header(k: usize) -> i32 {
    i32::from_le_bytes([
        FIELD[k * 4],
        FIELD[k * 4 + 1],
        FIELD[k * 4 + 2],
        FIELD[k * 4 + 3],
    ])
}

/// The field's height at a cell (east, north), in its own units; zero off
/// it.
fn cell(i: i32, j: i32, nx: i32, nz: i32) -> i64 {
    if i < 0 || j < 0 || i >= nx || j >= nz {
        return 0;
    }
    let k = 24 + 2 * (j * nx + i) as usize;
    u16::from_le_bytes([FIELD[k], FIELD[k + 1]]) as i64
}

/// **How far the Sleeper raises the land** at a map point (x, z), on a
/// mountain that has risen `rise` off its floor there: zero off its body.
pub fn lift(x: Fx, z: Fx, rise: Fx) -> Fx {
    if rise.raw() <= SHOWS_FROM.raw() {
        return Fx::ZERO;
    }
    let (west, south, side, nx, nz, scale) = (
        header(0),
        header(1),
        header(2),
        header(3),
        header(4),
        header(5),
    );
    // Where the point is in the grid, in cells, 16.16.
    let one = Fx::ONE.raw() as i64;
    let at = |v: Fx, edge: i32| (v.raw() as i64 * 100 - edge as i64 * one) / side as i64;
    let (gx, gz) = (at(x, west), at(z, south));
    if gx < 0 || gz < 0 || gx >= (nx as i64 - 1) * one || gz >= (nz as i64 - 1) * one {
        return Fx::ZERO;
    }
    let (i, j) = ((gx / one) as i32, (gz / one) as i32);
    let (fx, fz) = (gx % one, gz % one);
    let row = |j: i32| cell(i, j, nx, nz) * (one - fx) + cell(i + 1, j, nx, nz) * fx;
    let h = (row(j) * (one - fz) + row(j + 1) * fz) / one;
    let h = Fx::from_raw((h / scale as i64) as i32);
    if h.raw() <= 0 {
        return Fx::ZERO;
    }
    let shows = rise
        .sub(SHOWS_FROM)
        .div(FULL_FROM.sub(SHOWS_FROM))
        .min(Fx::ONE);
    h.mul(shows)
}

/// **The Sleeper's horns as stone**: each horn a curve of points on the map
/// and the stone's radius at each, set on the mountain under it -- `bare`,
/// the ground with no Sleeper -- taken as one height over the whole horn, so
/// the stone is one smooth piece and not the land's noise traced in rock.
pub fn arches(bare: &dyn Fn(Fx, Fx) -> Fx) -> Vec<Vec<(V3, Fx)>> {
    let mut horns: Vec<Vec<[Fx; 4]>> = vec![Vec::new()];
    for line in ARCH.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if line.is_empty() {
            if !horns.last().is_some_and(|h| h.is_empty()) {
                horns.push(Vec::new());
            }
            continue;
        }
        let mut v = line
            .split_whitespace()
            .map(|n| Fx::ratio(n.parse::<i32>().unwrap_or(0), 100));
        let mut next = || v.next().unwrap_or(Fx::ZERO);
        let point = [next(), next(), next(), next()];
        horns.last_mut().expect("a horn").push(point);
    }
    horns
        .into_iter()
        .filter(|h| h.len() > 1)
        .map(|h| {
            let mut sum = Fx::ZERO;
            for p in &h {
                sum = sum.add(bare(p[0], p[1]));
            }
            let base = sum.div(Fx::from_int(h.len() as i32));
            h.iter()
                .map(|p| (V3::new(p[0], base.add(p[2]), p[1]), p[3]))
                .collect()
        })
        .collect()
}

/// **A horn's rock**: turned boxes along its curve, each a short piece of
/// it -- as long as [`ROCK_STEP`] of its radius at most -- standing on
/// nothing but the piece before and after it, so where the horn lifts off
/// the body there is air under it: an arch.
pub fn arch_rock(arch: &[(V3, Fx)], out: &mut Vec<Solid>) {
    for pair in arch.windows(2) {
        let ((a, ra), (b, rb)) = (pair[0], pair[1]);
        let d = b.sub(a);
        let len = crate::math::wide_len(d);
        let step = ra.min(rb).max(Fx::ONE).mul(ROCK_STEP);
        let n = len.div(step).raw() / Fx::ONE.raw() + 1;
        let flat = crate::math::wide_len(V3::new(d.x, Fx::ZERO, d.z));
        // Along the piece in the floor plane; straight up, any way will do.
        let (cos, sin) = if flat.raw() > step.raw() / 8 {
            (d.x.div(flat), d.z.div(flat))
        } else {
            (Fx::ONE, Fx::ZERO)
        };
        for k in 0..n {
            let t = Fx::from_int(2 * k + 1).div(Fx::from_int(2 * n));
            let c = a.add(d.scale(t));
            let r = crate::math::lerp(ra, rb, t);
            let half = flat
                .div(Fx::from_int(2 * n))
                .add(r.mul(ROCK_STEP).div(Fx::from_int(2)));
            let half_wide = r.mul(ROCK_WIDE);
            let deep = r.mul(ROCK_DEEP);
            let ex = half.mul(cos.abs()).add(half_wide.mul(sin.abs()));
            let ez = half.mul(sin.abs()).add(half_wide.mul(cos.abs()));
            out.push(Solid {
                min: V3::new(c.x.sub(ex), c.y.sub(deep), c.z.sub(ez)),
                max: V3::new(c.x.add(ex), c.y.add(deep), c.z.add(ez)),
                material: Material::Rock,
                shape: crate::arena::Shape::Turned {
                    half,
                    half_wide,
                    cos,
                    sin,
                },
            });
        }
    }
}
