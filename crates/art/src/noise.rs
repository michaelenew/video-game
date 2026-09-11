//! The three noise shapes everything else is built from.
//!
//! Not a noise library. Three primitives, chosen because between them they
//! cover every material the game needs and no fourth one has yet earned its
//! place:
//!
//! - **Gradient noise** — smooth, band-limited hills. Perlin's 1985 shape.
//!   The base of anything organic: skin mottle, cloth slub, the body of a
//!   flame.
//! - **Worley** — distance to the nearest of a set of scattered points
//!   (Worley 1996). `F1` gives cells, `F2 - F1` gives the *walls between*
//!   cells, which is the only cheap way to get a line that closes on itself.
//!   Cracks, mortar, scales, the seams between armour plates.
//! - **Stripe** — a sine along one axis. Trivial, and the thing that makes
//!   cloth read as woven rather than as marble.
//!
//! Two operators combine them: **fBm**, which sums octaves at rising frequency
//! and falling amplitude, and **ridging**, which folds the signal at zero so
//! the peaks become creases. Ridged fBm is what turns hills into rock.
//!
//! ## Why these are not enough on their own
//!
//! Stacking octaves gives you more *detail*, never more *structure*, and
//! structure is what separates stone from marble from skin from plastic. Real
//! stone has strata; cloth has a weave direction; armour has plates. All three
//! are anisotropic — they look different along different axes — so every field
//! here takes a **frequency per axis** rather than one scalar. Strata are just
//! a high frequency in Y against a low one in X and Z.
//!
//! That single parameter does more work than four extra octaves ever will, and
//! it is the answer to why naive procedural materials all look like the same
//! grey marble.

/// Integer hash. Deterministic, seeded, and the same on every machine, which
/// matters for the golden-image tests rather than for the network -- nothing
/// in this crate reaches the simulation.
///
/// This is the finalizer from MurmurHash3, which is a standard choice for
/// exactly this job: cheap, and it avalanches well enough that neighbouring
/// lattice points give uncorrelated gradients.
#[inline]
fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 13;
    x = x.wrapping_mul(0xc2b2_ae35);
    x ^= x >> 16;
    x
}

#[inline]
fn hash3(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let h = (x as u32).wrapping_mul(0x27d4_eb2d)
        ^ (y as u32).wrapping_mul(0x1656_67b1)
        ^ (z as u32).wrapping_mul(0x4f6c_dd1d)
        ^ seed.wrapping_mul(0x9e37_79b9);
    hash(h)
}

/// A unit vector from a hash, picked from the twelve edge-midpoints of a cube.
///
/// Perlin's own improvement (2002) over choosing a random direction: the twelve
/// are well distributed, and the dot product against them is a couple of adds
/// rather than three multiplies. The visible payoff is fewer directional
/// artefacts than the original gradient table produced.
#[inline]
fn gradient(h: u32, dx: f32, dy: f32, dz: f32) -> f32 {
    match h % 12 {
        0 => dx + dy,
        1 => -dx + dy,
        2 => dx - dy,
        3 => -dx - dy,
        4 => dx + dz,
        5 => -dx + dz,
        6 => dx - dz,
        7 => -dx - dz,
        8 => dy + dz,
        9 => -dy + dz,
        10 => dy - dz,
        _ => -dy - dz,
    }
}

/// Perlin's quintic fade, `6t^5 - 15t^4 + 10t^3`.
///
/// Its first *and* second derivatives are zero at both ends, which is why it
/// replaced the original cubic: a discontinuous second derivative shows up as
/// visible creases along the lattice once the field drives a normal map, and
/// normal maps are most of what this crate produces.
#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Gradient noise at a point. Range is roughly -1 to 1, zero at every lattice
/// point.
pub fn gradient_noise(p: [f32; 3], seed: u32) -> f32 {
    let xi = p[0].floor();
    let yi = p[1].floor();
    let zi = p[2].floor();
    let (x0, y0, z0) = (xi as i32, yi as i32, zi as i32);
    let (fx, fy, fz) = (p[0] - xi, p[1] - yi, p[2] - zi);
    let (u, v, w) = (fade(fx), fade(fy), fade(fz));

    let mut corner = [0.0f32; 8];
    for (i, slot) in corner.iter_mut().enumerate() {
        let (cx, cy, cz) = (i & 1, (i >> 1) & 1, (i >> 2) & 1);
        let h = hash3(x0 + cx as i32, y0 + cy as i32, z0 + cz as i32, seed);
        *slot = gradient(h, fx - cx as f32, fy - cy as f32, fz - cz as f32);
    }

    let x00 = lerp(corner[0], corner[1], u);
    let x10 = lerp(corner[2], corner[3], u);
    let x01 = lerp(corner[4], corner[5], u);
    let x11 = lerp(corner[6], corner[7], u);
    lerp(lerp(x00, x10, v), lerp(x01, x11, v), w)
}

/// Distance to the nearest and second-nearest scattered point.
///
/// One feature point per lattice cell, jittered inside it, and the 27
/// neighbouring cells searched. Returns `(f1, f2)` in the same units as `p`.
///
/// `f2 - f1` is the useful one and it is worth knowing why: it goes to zero
/// exactly on the set of points equidistant from two features, which is the
/// boundary between cells. That is a closed curve obtained without ever
/// representing a curve -- the cheapest crack, mortar line and plate seam
/// available.
pub fn worley(p: [f32; 3], seed: u32) -> (f32, f32) {
    let base = [
        p[0].floor() as i32,
        p[1].floor() as i32,
        p[2].floor() as i32,
    ];
    let (mut f1, mut f2) = (f32::MAX, f32::MAX);

    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cell = [base[0] + dx, base[1] + dy, base[2] + dz];
                let h = hash3(cell[0], cell[1], cell[2], seed);
                // Three independent fractions out of one hash: the top, middle
                // and bottom bits are decorrelated by the avalanche above.
                let jx = (h & 0x3ff) as f32 / 1023.0;
                let jy = ((h >> 10) & 0x3ff) as f32 / 1023.0;
                let jz = ((h >> 20) & 0x3ff) as f32 / 1023.0;
                let fp = [
                    cell[0] as f32 + jx,
                    cell[1] as f32 + jy,
                    cell[2] as f32 + jz,
                ];
                let d = ((fp[0] - p[0]).powi(2) + (fp[1] - p[1]).powi(2) + (fp[2] - p[2]).powi(2))
                    .sqrt();
                if d < f1 {
                    f2 = f1;
                    f1 = d;
                } else if d < f2 {
                    f2 = d;
                }
            }
        }
    }
    (f1, f2)
}
