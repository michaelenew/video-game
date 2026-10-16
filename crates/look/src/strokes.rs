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
    let cells = (PERIOD / 0.5).round() as i32;
    field_in(q, rigid, turn, Some(cells))
}

/// **The field unwrapped**: the strokes of the whole plane, never repeating,
/// which is what a near tile is baked from (`game::near`) -- a sheet's period
/// is a few metres up close, and the eye finds a repeat at once. Shade,
/// warmth and the third axis, -1 to 1; `q` in stroke lengths.
pub fn stroke(q: [f32; 2], rigid: f32, turn: f32) -> [f32; 3] {
    let f = field_in(q, rigid, turn, None);
    [f[0], f[1], f[2]]
}

/// **The brush at a distance**, as `paint.wgsl`'s `broad_strokes` lays it:
/// two layers of strokes, their lengths the powers of two either side of
/// `broad_size * dist * size` metres, blended by where between the two it
/// falls, and the blend kept at full strength. `q` in metres on the plane.
/// The near bake fixes `dist` (`game::near::BAKE_DIST`), so its strokes are
/// a fixed size in the world rather than on the screen.
pub fn broad(
    q: [f32; 2],
    turn: f32,
    rigid: f32,
    size: f32,
    dist: f32,
    broad_size: f32,
) -> [f32; 3] {
    let l = (broad_size * dist * size).max(0.05).log2();
    let lo = l.floor();
    let t = l - lo;
    let (len1, len2) = (lo.exp2(), (lo + 1.0).exp2());
    let one = stroke([q[0] / len1, q[1] / len1], rigid, turn);
    let two = stroke([(q[0] + 17.3) / len2, (q[1] + 9.7) / len2], rigid, turn);
    let k = 1.0 / (t * t + (1.0 - t) * (1.0 - t)).sqrt();
    [
        mix(one[0], two[0], t) * k,
        mix(one[1], two[1], t) * k,
        mix(one[2], two[2], t) * k,
    ]
}

/// **Which way the strokes run on the ground**, as `paint.wgsl`'s
/// `stroke_way` decides it: along `flow` where the mesh gave one (a road),
/// else uphill, wandering slowly on the flat; built work (`rigid` high) runs
/// level along its courses. `p` is the point and `n` the ground's normal.
pub fn way_on_ground(p: [f32; 3], n: [f32; 3], flow: [f32; 2], rigid: f32) -> [f32; 3] {
    let up = [0.0, 1.0, 0.0];
    let mut level = cross(up, n);
    if len3(level) < 1e-3 {
        level = [1.0, 0.0, 0.0];
    }
    let level = norm3(level);
    if (flow[0] * flow[0] + flow[1] * flow[1]).sqrt() > 0.1 {
        let f = [flow[0], 0.0, flow[1]];
        let on = sub3(f, scale3(n, dot3(f, n)));
        if len3(on) > 1e-3 {
            return norm3(on);
        }
    }
    let lean = (1.0 - n[1] * n[1]).max(0.0).sqrt();
    let mut uphill = level;
    if lean > 1e-3 {
        uphill = norm3(sub3(up, scale3(n, n[1])));
    }
    // 6.28 rather than tau: the shader's number, matched to the digit.
    #[allow(clippy::approx_constant)]
    let a = noise2(p[0] / 38.0, p[2] / 38.0) * 6.28;
    let mut wander = [a.cos(), 0.0, a.sin()];
    wander = sub3(wander, scale3(n, dot3(wander, n)));
    if dot3(wander, uphill) < 0.0 {
        wander = scale3(wander, -1.0);
    }
    let on_slope = smoothstep(0.03, 0.15, lean);
    let w = norm3(add3(wander, scale3(n, 1e-4)));
    let natural = norm3(add3(lerp3(w, uphill, on_slope), scale3(n, 1e-5)));
    let built = smoothstep(0.55, 0.8, rigid) * (1.0 - smoothstep(0.5, 0.8, n[1].abs()));
    norm3(lerp3(natural, level, built))
}

/// **The brush with its strokes worked out once**: what `broad` says, for
/// many points over one patch of plane -- a tile's bake. `broad` works every
/// candidate stroke's place, size, angle and colours out again at every
/// point, which is a hundred strokes a texel and most of the cost; here each
/// stroke is worked out once for its cell and only the cheap part -- is this
/// point inside it -- is asked per point. The answer is `broad`'s to within
/// rounding (`the_brush_remembers_what_broad_says`).
pub struct Brush {
    lo: [f32; 2],
    hi: [f32; 2],
    /// One plane of marks per stroke length (a power of two), as they are
    /// needed.
    planes: Vec<(i32, Plane)>,
}

impl Brush {
    /// A brush for points with `lo <= q <= hi`, in metres.
    pub fn over(lo: [f32; 2], hi: [f32; 2]) -> Brush {
        Brush {
            lo,
            hi,
            planes: Vec::new(),
        }
    }

    /// `broad`, remembered.
    pub fn broad(
        &mut self,
        q: [f32; 2],
        turn: f32,
        rigid: f32,
        size: f32,
        dist: f32,
        broad_size: f32,
    ) -> [f32; 3] {
        let l = (broad_size * dist * size).max(0.05).log2();
        let lo = l.floor();
        let t = l - lo;
        let (len1, len2) = (lo.exp2(), (lo + 1.0).exp2());
        let (sn, cs) = turn.sin_cos();
        let one =
            self.plane(lo as i32, len1, [0.0, 0.0])
                .at([q[0] / len1, q[1] / len1], rigid, sn, cs);
        let two = self.plane(lo as i32 + 1, len2, [17.3, 9.7]).at(
            [(q[0] + 17.3) / len2, (q[1] + 9.7) / len2],
            rigid,
            sn,
            cs,
        );
        let k = 1.0 / (t * t + (1.0 - t) * (1.0 - t)).sqrt();
        [
            mix(one[0], two[0], t) * k,
            mix(one[1], two[1], t) * k,
            mix(one[2], two[2], t) * k,
        ]
    }

    fn plane(&mut self, level: i32, len: f32, shift: [f32; 2]) -> &mut Plane {
        let at = match self.planes.iter().position(|(l, _)| *l == level) {
            Some(at) => at,
            None => {
                let lo = [(self.lo[0] + shift[0]) / len, (self.lo[1] + shift[1]) / len];
                let hi = [(self.hi[0] + shift[0]) / len, (self.hi[1] + shift[1]) / len];
                self.planes.push((level, Plane::over(lo, hi)));
                self.planes.len() - 1
            }
        };
        &mut self.planes[at].1
    }
}

/// One stroke, worked out for its cell: everything about it that does not
/// depend on the point asking.
#[derive(Clone, Copy)]
struct Mark {
    centre: [f32; 2],
    /// Its own turn off the field's, as a sine and cosine.
    sn: f32,
    cs: f32,
    long: f32,
    wide: f32,
    curve: f32,
    order: f32,
    colour: [f32; 3],
    h1: f32,
    h2: f32,
}

/// How many index cells a stroke's cell is across. The marks can be indexed
/// on a finer grid than they are seeded on, so a point's list holds fewer
/// strokes that cannot reach it; at two it held 31 where one holds 36, and
/// the bake was slower for the lists it built, so one.
const FINER: i32 = 1;

/// The marks of a patch of plane, in stroke lengths: for each index cell,
/// every stroke that can reach into it, worked out for the rigidity last
/// asked there (a rigidity is a kind of ground, so it changes only at a
/// border).
struct Plane {
    x0: i32,
    y0: i32,
    w: usize,
    h: usize,
    cells: Vec<Option<(f32, Vec<Mark>)>>,
}

impl Plane {
    fn over(lo: [f32; 2], hi: [f32; 2]) -> Plane {
        let cell = 0.5 / FINER as f32;
        let x0 = (lo[0] / cell).floor() as i32 - 1;
        let y0 = (lo[1] / cell).floor() as i32 - 1;
        let w = ((hi[0] / cell).floor() as i32 + 2 - x0).max(1) as usize;
        let h = ((hi[1] / cell).floor() as i32 + 2 - y0).max(1) as usize;
        Plane {
            x0,
            y0,
            w,
            h,
            cells: vec![None; w * h],
        }
    }

    fn mark(c: [i32; 2], k: u32, rigid: f32) -> Mark {
        let loose = 1.0 - rigid;
        let cell = 0.5;
        let h = |n: u32| hash(c[0] as u32, c[1] as u32, k * 16 + n);
        let (h1, h2, h3, h4, h5, h6) = (h(1), h(2), h(3), h(4), h(5), h(6));
        let scatter = mix(0.2, 1.0, loose);
        let centre = [
            (c[0] as f32 + 0.5 + (h1 - 0.5) * scatter) * cell,
            (c[1] as f32 + 0.5 + (h2 - 0.5) * scatter) * cell,
        ];
        let (sn, cs) = ((h3 - 0.5) * loose * 1.6).sin_cos();
        let size = 1.0 + (h4 - 0.5) * loose * 1.4;
        Mark {
            centre,
            sn,
            cs,
            long: size * mix(0.5, 0.55, h5),
            wide: size * mix(0.15, 0.13 + 0.12 * h6, loose),
            curve: (h5 - 0.5) * loose * 0.8,
            order: h(8),
            colour: [h(9) * 2.0 - 1.0, h(10) * 2.0 - 1.0, h(11) * 2.0 - 1.0],
            h1,
            h2,
        }
    }

    /// Every stroke of the two cells either way that can reach index cell
    /// `c`, at this rigidity.
    fn reaching(c: [i32; 2], rigid: f32) -> Vec<Mark> {
        let cell = 0.5 / FINER as f32;
        let (lo, hi) = (
            [c[0] as f32 * cell, c[1] as f32 * cell],
            [(c[0] + 1) as f32 * cell, (c[1] + 1) as f32 * cell],
        );
        let home = [c[0].div_euclid(FINER), c[1].div_euclid(FINER)];
        let mut out = Vec::with_capacity(12);
        for j in -2..=2 {
            for i in -2..=2 {
                for k in 0..2u32 {
                    let m = Plane::mark([home[0] + i, home[1] + j], k, rigid);
                    // How far the stroke reaches from its middle, any way
                    // round: its half length past the prune, and its width
                    // with its curve and bristles.
                    let reach = m.long * 1.2 + m.wide * 2.0;
                    let dx = (lo[0] - m.centre[0]).max(m.centre[0] - hi[0]).max(0.0);
                    let dy = (lo[1] - m.centre[1]).max(m.centre[1] - hi[1]).max(0.0);
                    if dx * dx + dy * dy <= reach * reach {
                        out.push(m);
                    }
                }
            }
        }
        // Topmost first: the first stroke a point is inside is the one it
        // shows, and the rest need not be asked.
        out.sort_by(|a, b| b.order.total_cmp(&a.order));
        out
    }

    /// `stroke(q, rigid, turn)`, with the field's turn as its sine and
    /// cosine.
    fn at(&mut self, q: [f32; 2], rigid: f32, sn_t: f32, cs_t: f32) -> [f32; 3] {
        let loose = 1.0 - rigid;
        let cell = 0.5 / FINER as f32;
        let home = [(q[0] / cell).floor() as i32, (q[1] / cell).floor() as i32];
        let (cx, cy) = (home[0] - self.x0, home[1] - self.y0);
        if cx < 0 || cy < 0 || cx as usize >= self.w || cy as usize >= self.h {
            return stroke(q, rigid, sn_t.atan2(cs_t));
        }
        let slot = &mut self.cells[cy as usize * self.w + cx as usize];
        let marks = match slot {
            Some((r, marks)) if *r == rigid => marks,
            _ => {
                *slot = Some((rigid, Plane::reaching(home, rigid)));
                &slot.as_ref().unwrap().1
            }
        };
        for m in marks {
            // The stroke's turn is the field's and its own: the sine and
            // cosine of the sum.
            let sn = sn_t * m.cs + cs_t * m.sn;
            let cs = cs_t * m.cs - sn_t * m.sn;
            let rel = [q[0] - m.centre[0], q[1] - m.centre[1]];
            let local = [cs * rel[0] + sn * rel[1], -sn * rel[0] + cs * rel[1]];
            let u = local[0] / m.long;
            if u.abs() > 1.2 {
                continue;
            }
            let v = (local[1] - m.curve * m.wide * u * u) / m.wide;
            let taper = mix(1.0, (1.0 - u * u).max(0.0).sqrt(), 0.25 + 0.6 * loose);
            // The bristles push an edge out by a sixth at most: a point
            // further out than that is outside before the noise is asked.
            if v.abs() >= taper * (1.0 + 0.175 * loose) {
                continue;
            }
            let bristle = noise2(u * 4.0 + m.h1 * 9.0, v * 5.0 + m.h2 * 9.0) - 0.5;
            let inside =
                v.abs() < taper * (1.0 + bristle * 0.35 * loose) && u.abs() < 1.0 + bristle * 0.5;
            if inside {
                let k = 0.85 + 0.15 * bristle * 2.0;
                return [m.colour[0] * k, m.colour[1] * k, m.colour[2] * k];
            }
        }
        [0.0; 3]
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}
fn norm3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a).max(1e-9);
    [a[0] / l, a[1] / l, a[2] / l]
}
fn scale3(a: [f32; 3], k: f32) -> [f32; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [mix(a[0], b[0], t), mix(a[1], b[1], t), mix(a[2], b[2], t)]
}
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The field, wrapped at `wrap` cells (a sheet) or not at all (the plane).
fn field_in(q: [f32; 2], rigid: f32, turn: f32, wrap: Option<i32>) -> [f32; 4] {
    let loose = 1.0 - rigid;
    let cell = 0.5;
    let home = [(q[0] / cell).floor() as i32, (q[1] / cell).floor() as i32];
    let mut best = -1.0;
    // No stroke: a phase of one half, so bare ground switches sheets midway.
    let mut out = [0.0, 0.0, 0.0, 0.5];
    // Two cells either way: the longest stroke reaches nearly two cells from
    // its middle, and a search of one cut it off at the cell's edge.
    for j in -2..=2 {
        for i in -2..=2 {
            let c = [home[0] + i, home[1] + j];
            let wrapped = match wrap {
                Some(cells) => [c[0].rem_euclid(cells) as u32, c[1].rem_euclid(cells) as u32],
                None => [c[0] as u32, c[1] as u32],
            };
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

    #[test]
    fn the_plane_does_not_repeat_but_the_sheet_does() {
        let q = [1.3, 2.1];
        let shifted = [q[0] + PERIOD, q[1]];
        let a = field(q, 0.0, 0.0);
        let b = field(shifted, 0.0, 0.0);
        for k in 0..4 {
            assert!(
                (a[k] - b[k]).abs() < 0.05,
                "a sheet tiles at its period: {a:?} {b:?}"
            );
        }
        // Over a run of points, the plane differs from its shift somewhere.
        let differs = (0..64).any(|k| {
            let q = [k as f32 * 0.37, k as f32 * 0.23];
            stroke(q, 0.0, 0.0) != stroke([q[0] + PERIOD, q[1]], 0.0, 0.0)
        });
        assert!(differs, "the plane repeated at the sheet's period");
    }

    #[test]
    fn the_way_on_the_ground_follows_the_road_and_runs_uphill() {
        let flat = [0.0, 1.0, 0.0];
        let along = way_on_ground([3.0, 0.0, 4.0], flat, [0.0, 1.0], 0.2);
        assert!(along[2] > 0.99, "{along:?} is not along the road");
        // A slope facing +x: uphill is -x on the surface.
        let n = [-0.6, 0.8, 0.0];
        let up = way_on_ground([3.0, 0.0, 4.0], n, [0.0, 0.0], 0.2);
        assert!(up[1] > 0.5 && up[0] > 0.5, "{up:?} does not run uphill");
        // Built work on a wall runs level.
        let wall = [1.0, 0.0, 0.0];
        let course = way_on_ground([0.0, 0.0, 0.0], wall, [0.0, 0.0], 0.9);
        assert!(course[1].abs() < 0.05, "{course:?} is not level");
    }

    #[test]
    fn the_brush_blends_two_sizes_and_keeps_its_strength() {
        let mut strong = 0.0f32;
        for k in 0..400 {
            let q = [k as f32 * 0.113, k as f32 * 0.071];
            let b = broad(q, 0.3, 0.0, 1.0, 12.0, 0.04);
            strong = strong.max(b[0].abs());
        }
        assert!(strong > 0.6, "the brush is faint: {strong}");
    }

    #[test]
    fn the_brush_remembers_what_broad_says() {
        let mut brush = Brush::over([0.0, 0.0], [16.0, 16.0]);
        let mut off = 0;
        let n = 2000;
        for k in 0..n {
            let q = [
                (hash(k, 1, 1) * 16.0).clamp(0.0, 16.0),
                (hash(k, 2, 2) * 16.0).clamp(0.0, 16.0),
            ];
            let turn = hash(k, 3, 3) * std::f32::consts::TAU;
            let rigid = [0.0, 0.25, 0.45, 0.85][k as usize % 4];
            let size = 0.6 + hash(k, 4, 4);
            let a = broad(q, turn, rigid, size, 12.0, 0.04);
            let b = brush.broad(q, turn, rigid, size, 12.0, 0.04);
            if (0..3).any(|i| (a[i] - b[i]).abs() > 1e-3) {
                off += 1;
            }
        }
        // A point on a stroke's very edge can fall either side by rounding.
        assert!(off * 100 < n, "{off} of {n} points differ");
    }
}

#[cfg(test)]
mod bench {
    use super::*;

    /// How long a tile's worth of points takes, and how many marks a cell
    /// holds: `cargo test -p look brush_speed -- --ignored --nocapture`.
    /// 0.4 s for a million points on 2026-10-15, down from 3.5 s before the
    /// width prune and the topmost-first order.
    #[test]
    #[ignore]
    fn brush_speed() {
        let res = 1024;
        let mut brush = Brush::over([0.0, 0.0], [16.0, 16.0]);
        let t = std::time::Instant::now();
        let mut acc = 0.0f32;
        for j in 0..res {
            for i in 0..res {
                let q = [
                    (i as f32 + 0.5) / res as f32 * 16.0,
                    (j as f32 + 0.5) / res as f32 * 16.0,
                ];
                let b = brush.broad(q, 0.3, 0.0, 2.4, 12.0, 0.04);
                acc += b[0];
            }
        }
        let took = t.elapsed();
        let (mut cells, mut marks) = (0usize, 0usize);
        for (_, plane) in &brush.planes {
            for c in plane.cells.iter().flatten() {
                cells += 1;
                marks += c.1.len();
            }
        }
        eprintln!(
            "1M points in {took:?} ({acc}); {marks} marks over {cells} cells = {:.1} a cell; planes {:?}",
            marks as f32 / cells.max(1) as f32,
            brush
                .planes
                .iter()
                .map(|(l, p)| (*l, p.w, p.h))
                .collect::<Vec<_>>()
        );
    }
}
