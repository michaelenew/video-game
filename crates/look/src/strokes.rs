//! **The brush strokes, baked.**
//!
//! A painted surface is covered in broad flat-brush strokes, each with its own
//! place, angle, length, width, curve, tapered ends, shade and warmth, the
//! topmost covering a point being the one seen (`docs/design/painting.md`).
//! The first version worked all of that out in the surface shader, for every
//! pixel of every frame: thirty-six candidate strokes a pixel, a dozen hashes
//! each. It was the single heaviest thing on the screen, and it made the
//! shader big enough that compiling it stalled the game.
//!
//! None of it depends on the frame. A stroke field is a function of where on
//! the surface you are, how long the strokes are, and how rigidly they are
//! laid -- and it is **self-similar**: strokes `len` metres long are the
//! strokes one metre long, scaled. So one tiling sheet per rigidity, drawn
//! once here, serves every surface at every distance: the shader scales and
//! turns its lookup and reads two texels. Four sheets, stacked, are the
//! texture `game/src/paint.wgsl` reads (`game/build.rs` bakes them into the
//! binary at compile time).
//!
//! Each texel holds the stroke's shade, its warmth and a third colour axis,
//! -1 to 1 each, as bytes, and in the fourth byte the stroke's **phase**: a
//! random number of its own, the same at every texel it covers.
//!
//! **Why six angles a rigidity rather than one sheet turned.** The strokes
//! follow a direction field (along a road, uphill), and the obvious way to
//! turn a sheet is to turn the lookup. But a lookup turned about the origin
//! moves by the distance from the origin times the change in angle: at seven
//! hundred metres from the map's origin, the few degrees the field wanders
//! between two neighbouring pixels shifts the sample by metres, and every
//! stroke on the ground smeared into a streak along the field. So the sheets
//! are drawn at six angles with the **same** strokes -- same places, sizes,
//! colours, order -- turned about their own middles, and a pixel reads the
//! two nearest. It does not blend them: half the texels change stroke across
//! a turn (strokes overlap, and a turn changes which is on top), and blending
//! those is the soft, unfinished look the owner rejected. It picks one, by
//! the stroke's phase: the earlier sheet while the field's place between the
//! two is short of the phase of the stroke it shows there, the later after.
//! Every stroke switches at its own moment, so there is no line where they
//! all jump at once, only strokes re-angling one by one across the band.

/// The rigidities the sheets are drawn at: a meadow, bare earth and rock, a
/// road, built work. A surface's rigidity picks the nearest.
pub const RIGIDITIES: [f32; 4] = [0.0, 0.25, 0.5, 0.85];

/// How many angles a sheet is drawn at, evenly over a half turn: a stroke
/// turned half a turn is the same stroke. The game blends the two nearest.
pub const ANGLES: usize = 6;

/// How many stroke lengths a sheet is across; it tiles at that.
pub const PERIOD: f32 = 8.0;

/// A sheet's edge in texels. A stroke a length long is `SIZE / PERIOD`
/// texels, 64, and is seen magnified about twice.
pub const SIZE: usize = 512;

fn hash(a: u32, b: u32, c: u32) -> f32 {
    let mut h =
        a.wrapping_mul(0x8da6_b343) ^ b.wrapping_mul(0xd816_3841) ^ c.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 16_777_216.0
}

fn hash2f(x: f32, y: f32) -> f32 {
    let mut q = [
        (x * 0.1031).fract(),
        (y * 0.1031).fract(),
        (x * 0.1031).fract(),
    ];
    let d = q[0] * (q[1] + 33.33) + q[1] * (q[2] + 33.33) + q[2] * (q[0] + 33.33);
    q = [q[0] + d, q[1] + d, q[2] + d];
    ((q[0] + q[1]) * q[2]).fract().abs()
}

/// Smooth value noise, 0 to 1, as `paint.wgsl`'s `noise2`.
fn noise2(x: f32, y: f32) -> f32 {
    let (i, j) = (x.floor(), y.floor());
    let (fx, fy) = (x - i, y - j);
    let (u, v) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = hash2f(i, j);
    let b = hash2f(i + 1.0, j);
    let c = hash2f(i, j + 1.0);
    let d = hash2f(i + 1.0, j + 1.0);
    let top = a + (b - a) * u;
    let bottom = c + (d - c) * u;
    top + (bottom - top) * v
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The stroke field at a point `q` (in stroke lengths) on a sheet of this
/// rigidity: shade, warmth and a third axis, -1 to 1, or zero where no stroke
/// covers it.
///
/// A few strokes in every cell of a grid half a stroke across, each with its
/// own place, angle, length, width, curve, tapered dry ends and colours.
/// `rigid` is how much they may vary: at 1 they sit on the grid, parallel and
/// alike, a mason's strokes along a course; at 0 they are scattered, turned,
/// sized and coloured as freely as a meadow's. The topmost covering stroke is
/// the one seen. Periodic every `PERIOD`, by hashing the cell modulo it.
pub fn field(q: [f32; 2], rigid: f32, turn: f32) -> [f32; 4] {
    let loose = 1.0 - rigid;
    let cell = 0.5;
    let cells = (PERIOD / cell).round() as i32;
    let home = [(q[0] / cell).floor() as i32, (q[1] / cell).floor() as i32];
    let mut best = -1.0;
    // No stroke: a phase of one half, so bare ground switches sheets midway.
    let mut out = [0.0, 0.0, 0.0, 0.5];
    // Two cells either way: the longest stroke reaches nearly two cells from
    // its middle, and a search of one cut it off at the cell's edge.
    for j in -2..=2 {
        for i in -2..=2 {
            let c = [home[0] + i, home[1] + j];
            let wrapped = [c[0].rem_euclid(cells) as u32, c[1].rem_euclid(cells) as u32];
            for k in 0..2u32 {
                let h = |n: u32| hash(wrapped[0], wrapped[1], k * 16 + n);
                let (h1, h2, h3, h4, h5, h6) = (h(1), h(2), h(3), h(4), h(5), h(6));
                // Where it sits: on the grid when ruled, anywhere in its cell
                // when loose.
                let scatter = mix(0.2, 1.0, loose);
                let centre = [
                    (c[0] as f32 + 0.5 + (h1 - 0.5) * scatter) * cell,
                    (c[1] as f32 + 0.5 + (h2 - 0.5) * scatter) * cell,
                ];
                let angle = turn + (h3 - 0.5) * loose * 1.6;
                let size = 1.0 + (h4 - 0.5) * loose * 1.4;
                let long = size * mix(0.5, 0.55, h5);
                let wide = size * mix(0.15, 0.13 + 0.12 * h6, loose);
                let curve = (h5 - 0.5) * loose * 0.8;
                let (sn, cs) = angle.sin_cos();
                let rel = [q[0] - centre[0], q[1] - centre[1]];
                let local = [cs * rel[0] + sn * rel[1], -sn * rel[0] + cs * rel[1]];
                let u = local[0] / long;
                if u.abs() > 1.2 {
                    continue;
                }
                let v = (local[1] - curve * wide * u * u) / wide;
                // Ends: square and ruled, or tapered and dry, by how loose.
                let taper = mix(1.0, (1.0 - u * u).max(0.0).sqrt(), 0.25 + 0.6 * loose);
                let bristle = noise2(u * 4.0 + h1 * 9.0, v * 5.0 + h2 * 9.0) - 0.5;
                let inside = v.abs() < taper * (1.0 + bristle * 0.35 * loose)
                    && u.abs() < 1.0 + bristle * 0.5;
                let order = h(8);
                if inside && order > best {
                    best = order;
                    let k = 0.85 + 0.15 * bristle * 2.0;
                    out = [
                        (h(9) * 2.0 - 1.0) * k,
                        (h(10) * 2.0 - 1.0) * k,
                        (h(11) * 2.0 - 1.0) * k,
                        h(12),
                    ];
                }
            }
        }
    }
    out
}

/// One sheet, `SIZE` square, row by row: the field at each texel's middle,
/// as bytes (0 is -1, 255 is 1; the phase 0 to 1), its strokes turned by
/// `turn` radians.
pub fn sheet(rigid: f32, turn: f32) -> Vec<[u8; 4]> {
    let mut px = Vec::with_capacity(SIZE * SIZE);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let q = [
                (x as f32 + 0.5) / SIZE as f32 * PERIOD,
                (y as f32 + 0.5) / SIZE as f32 * PERIOD,
            ];
            let f = field(q, rigid, turn);
            px.push([
                byte(f[0]),
                byte(f[1]),
                byte(f[2]),
                (f[3] * 255.0 + 0.5) as u8,
            ]);
        }
    }
    px
}

fn byte(v: f32) -> u8 {
    ((v.clamp(-1.0, 1.0) * 0.5 + 0.5) * 255.0 + 0.5) as u8
}

/// Every sheet, stacked top to bottom: each rigidity in `RIGIDITIES`' order,
/// and within it each angle, so sheet `r * ANGLES + a` is rigidity `r` turned
/// `a / ANGLES` of a half turn. `SIZE` wide, `SIZE * LAYERS` tall, which is
/// what the game reads as a texture array.
pub fn stack() -> Vec<[u8; 4]> {
    RIGIDITIES
        .iter()
        .flat_map(|&r| (0..ANGLES).flat_map(move |a| sheet(r, turn_of(a))))
        .collect()
}

/// How many sheets `stack` holds.
pub const LAYERS: usize = RIGIDITIES.len() * ANGLES;

/// The angle sheet `a` of a rigidity is turned by, in radians.
pub fn turn_of(a: usize) -> f32 {
    a as f32 * std::f32::consts::PI / ANGLES as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sheet_tiles() {
        // The field at a point and the same point a period over are the same
        // stroke, so the sheet's edges meet.
        for (q, r) in [([0.3, 0.7], 0.0), ([5.1, 2.2], 0.5), ([7.9, 7.95], 0.85)] {
            let a = field(q, r, 0.5);
            let b = field([q[0] + PERIOD, q[1]], r, 0.5);
            let c = field([q[0], q[1] + PERIOD], r, 0.5);
            // The same stroke, to within the rounding of a period added.
            for k in 0..4 {
                assert!((a[k] - b[k]).abs() < 1e-4, "{q:?} at {r}: {a:?} {b:?}");
                assert!((a[k] - c[k]).abs() < 1e-4, "{q:?} at {r}: {a:?} {c:?}");
            }
        }
    }

    #[test]
    fn a_ruled_sheet_is_covered_and_a_loose_one_varies() {
        // Ruled strokes sit on the grid and cover it; loose ones leave gaps
        // and take every shade.
        let count = |r: f32| {
            let mut covered = 0;
            let mut shades = std::collections::BTreeSet::new();
            for y in 0..64 {
                for x in 0..64 {
                    let f = field([x as f32 / 8.0, y as f32 / 8.0], r, 0.0);
                    if f[..3] != [0.0; 3] {
                        covered += 1;
                        shades.insert((f[0] * 4.0).round() as i32);
                    }
                }
            }
            (covered, shades.len())
        };
        let (ruled, _) = count(0.85);
        let (loose, loose_shades) = count(0.0);
        // Ruled rows leave a thin gap between courses, as a mason's do.
        assert!(ruled > 64 * 64 * 7 / 10, "ruled covers {ruled} of 4096");
        assert!(loose > 64 * 64 / 2, "loose covers {loose} of 4096");
        assert!(loose_shades >= 7, "loose shades: {loose_shades}");
    }

    #[test]
    fn every_angle_holds_the_same_strokes() {
        // Turned, a stroke keeps its place and its colours: at a stroke's
        // own middle every angle sheet shows that stroke.
        let mut same = 0;
        let mut seen = 0;
        for y in 0..32 {
            for x in 0..32 {
                let q = [x as f32 / 4.0 + 0.1, y as f32 / 4.0 + 0.1];
                let a = field(q, 0.0, 0.0);
                if a[..3] == [0.0; 3] {
                    continue;
                }
                seen += 1;
                let b = field(q, 0.0, turn_of(1));
                if (a[0] - b[0]).abs() < 0.2 && (a[1] - b[1]).abs() < 0.2 {
                    same += 1;
                }
            }
        }
        // About half do (measured: 527 of 1012), which is why the sheets are
        // picked between by phase rather than blended.
        assert!(
            same * 10 > seen * 3,
            "{same} of {seen} points keep their stroke a turn on"
        );
    }
}
