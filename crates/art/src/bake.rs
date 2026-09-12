//! Turning a [`Surface`] into textures a renderer can already eat.
//!
//! ## Why the maths runs on the processor and not in a shader
//!
//! The instinct with procedural materials is to evaluate them per pixel in a
//! fragment shader: infinite resolution, no memory, no seams. It is the right
//! answer for a large open world and it is the wrong one here, for four
//! reasons that are all about this project specifically.
//!
//! - **One implementation.** Shader code cannot be unit tested, cannot be
//!   stepped through, and cannot be shared with a tool. Written in Rust it is
//!   ordinary code with ordinary tests, and the same function serves the game,
//!   the preview sheet and the checks.
//! - **No render pipeline.** A baked texture is a `StandardMaterial` field.
//!   A live one is a custom material, a custom pipeline, and a WGSL file that
//!   drifts from the Rust beside it. That is a large amount of engine surface
//!   bought for a small arena.
//! - **It runs anywhere.** Baked textures work on an integrated laptop chip
//!   and in a browser. Six octaves of noise triplanar-projected per pixel do
//!   not, reliably.
//! - **It is still live enough.** Measured on four cores: the full set of ten
//!   materials at 256 pixels takes about 280 ms, and one ordinary material on
//!   its own about 20 ms. Re-baking a single material when a slider is
//!   released is comfortable; re-baking on every frame of a drag is not, and
//!   neither is re-baking everything. That is a real limit and it is the one
//!   thing a shader would still buy.
//!
//! None of that is permanent. The parameters are the asset; moving one
//! material into a shader later means writing that shader, not redoing the
//! work. The bet is on the parameter space, not on where it is evaluated.
//!
//! ## What the limit actually is
//!
//! Tiling. A baked texture repeated across a forty-metre floor shows its
//! period, and no amount of octaves fixes that — it is a property of repeating
//! a finite thing. The moment the floor reads as wallpaper is the moment one
//! triplanar shader earns its place, and it will be the environment that needs
//! it and never the characters. Said plainly so that it is a scheduled cost
//! rather than a surprise.

use crate::surface::Surface;

/// An RGBA8 image.
#[derive(Clone, Debug)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    /// Four bytes per pixel, row-major from the top left.
    pub pixels: Vec<u8>,
}

impl Texture {
    fn new(size: u32) -> Texture {
        Texture {
            width: size,
            height: size,
            pixels: vec![0; (size * size * 4) as usize],
        }
    }

    fn put(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        let i = ((y * self.width + x) * 4) as usize;
        self.pixels[i..i + 4].copy_from_slice(&rgba);
    }
}

/// The set of maps one material produces.
///
/// Split the way a physically-based renderer wants them, because that is what
/// the engine already knows how to consume — there is nothing to be gained by
/// inventing a layout.
#[derive(Clone, Debug)]
pub struct Maps {
    /// Base colour, sRGB-encoded. Alpha is 255 throughout.
    pub albedo: Texture,
    /// Tangent-space normals, linear. The gradient of the height field.
    pub normal: Texture,
    /// glTF's packed convention: green is roughness, blue is metallic. Red is
    /// left for ambient occlusion, which nothing generates yet.
    pub metallic_roughness: Texture,
    /// Emitted light, sRGB-encoded, with the overall strength factored out
    /// into `emissive_strength` so that values above one survive the eight
    /// bits. Fire is many times brighter than white.
    pub emissive: Texture,
    /// What to multiply the emissive texture by. One when nothing glows.
    pub emissive_strength: f32,
    /// Whether any texel is less than fully opaque.
    ///
    /// Reported rather than inferred from the surface, because a material that
    /// *can* fade may still come out solid at a given tile -- and switching a
    /// material to alpha blending it does not need costs correct depth sorting
    /// for nothing.
    pub has_alpha: bool,
}

/// How to bake.
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    /// Pixels along each side.
    pub size: u32,
    /// How many **metres** the texture covers. This is what makes the
    /// frequencies in [`crate::materials`] mean something: a field at two
    /// cycles per metre baked across a two-metre tile has four cycles across
    /// the image, on any machine, at any resolution.
    pub extent: f32,
    /// Whether the result has to repeat without a visible join.
    ///
    /// Costs four field evaluations per texel instead of one, and flattens the
    /// contrast slightly, because periodicity is bought by blending the tile
    /// against its own wrapped copies. Worth it for a floor that repeats forty
    /// times and pointless for a fighter's arm, which is why it is a choice
    /// rather than always on.
    pub tileable: bool,
    /// Where in the field to sample. Two materials with the same parameters
    /// and different origins are the same material somewhere else — which is
    /// how one stone gives every wall in the arena a different face.
    pub origin: [f32; 3],
    /// Which direction in the field the texture's horizontal runs along, and
    /// which its vertical does. See [`Plan::wall`] for why this is not a
    /// detail.
    pub u_axis: [f32; 3],
    pub v_axis: [f32; 3],
}

impl Plan {
    /// Half a metre of detail at whatever resolution, not tiling, sliced
    /// across a fighter's front. For anything worn or worn out.
    pub const fn character(size: u32) -> Plan {
        Plan {
            size,
            extent: 1.0,
            tileable: false,
            origin: [0.0; 3],
            u_axis: [1.0, 0.0, 0.0],
            v_axis: [0.0, 1.0, 0.0],
        }
    }

    /// A wall: the texture's vertical runs along the world's vertical.
    ///
    /// Which plane of the field a texture slices is not a detail. Stone's
    /// strata are a high frequency in **Y**, so a wall sliced across X and Z
    /// never crosses a single one and the rock comes out as featureless mush —
    /// the material would look broken and the parameters would be blameless.
    pub const fn wall(size: u32, extent: f32) -> Plan {
        Plan {
            size,
            extent,
            tileable: true,
            origin: [0.0; 3],
            u_axis: [1.0, 0.0, 0.0],
            v_axis: [0.0, 1.0, 0.0],
        }
    }

    /// A floor, sliced flat across X and Z.
    pub const fn floor(size: u32, extent: f32) -> Plan {
        Plan {
            size,
            extent,
            tileable: true,
            origin: [0.0; 3],
            u_axis: [1.0, 0.0, 0.0],
            v_axis: [0.0, 0.0, 1.0],
        }
    }

    pub const fn at(mut self, origin: [f32; 3]) -> Plan {
        self.origin = origin;
        self
    }

    pub const fn tiling(mut self, on: bool) -> Plan {
        self.tileable = on;
        self
    }

    /// The point in the field that texel `(u, v)` reads from.
    fn point(&self, u: f32, v: f32) -> [f32; 3] {
        let (du, dv) = (u * self.extent, v * self.extent);
        [
            self.origin[0] + self.u_axis[0] * du + self.v_axis[0] * dv,
            self.origin[1] + self.u_axis[1] * du + self.v_axis[1] * dv,
            self.origin[2] + self.u_axis[2] * du + self.v_axis[2] * dv,
        ]
    }
}

/// The field at a texel, made periodic if asked.
///
/// Periodicity is the four-corner blend: the tile is mixed with the three
/// copies of itself shifted by one period, weighted so the weights at `u = 0`
/// and `u = 1` come out identical. That makes the edges *equal* rather than
/// merely similar, which is what a seam is.
///
/// The cost is contrast: averaging four samples pulls everything toward the
/// mean. Cheap to state, hard to avoid, and the reason this is opt-in.
fn scalar_at(s: &Surface, plan: &Plan, u: f32, v: f32) -> f32 {
    if !plan.tileable {
        return s.scalar(plan.point(u, v));
    }
    let (iu, iv) = (1.0 - u, 1.0 - v);
    s.scalar(plan.point(u, v)) * iu * iv
        + s.scalar(plan.point(u - 1.0, v)) * u * iv
        + s.scalar(plan.point(u, v - 1.0)) * iu * v
        + s.scalar(plan.point(u - 1.0, v - 1.0)) * u * v
}

/// Bake one material.
///
/// Every texel is independent, so the rows are split across the machine's
/// cores. `std::thread::scope` does that without a dependency, which keeps the
/// crate's zero-dependency guarantee intact -- worth the twenty lines, since a
/// full set of materials went from 950 ms to 280 ms on a four-core machine and
/// that is the difference between a startup pause you notice and one you do
/// not.
pub fn bake(surface: &Surface, plan: Plan) -> Maps {
    let n = plan.size.max(2);
    let count = (n * n) as usize;

    // Both kept whole: the normals are central differences over the height, and
    // the emission has to be normalised against its own peak before it can be
    // packed into eight bits. One pass, three buffers, no field evaluated twice.
    let mut height = vec![0.0f32; count];
    let mut emitted = vec![[0.0f32; 3]; count];
    let mut albedo = Texture::new(n);
    let mut orm = Texture::new(n);

    let threads = std::thread::available_parallelism()
        .map_or(1, |p| p.get())
        .min(n as usize);
    let rows_per = n.div_ceil(threads as u32).max(1) as usize;
    let row_bytes = (n * 4) as usize;

    std::thread::scope(|scope| {
        let bands = height
            .chunks_mut(rows_per * n as usize)
            .zip(emitted.chunks_mut(rows_per * n as usize))
            .zip(albedo.pixels.chunks_mut(rows_per * row_bytes))
            .zip(orm.pixels.chunks_mut(rows_per * row_bytes));
        for (band, (((h, e), a), o)) in bands.enumerate() {
            let plan = &plan;
            scope.spawn(move || {
                let y0 = (band * rows_per) as u32;
                for i in 0..h.len() {
                    let (x, y) = (i as u32 % n, y0 + i as u32 / n);
                    let t = scalar_at(surface, plan, x as f32 / n as f32, y as f32 / n as f32);
                    h[i] = t;

                    let s = surface.shade(t);
                    // Alpha rides in the albedo map's fourth channel, which is
                    // where every renderer already looks for it. Linear, not
                    // sRGB: coverage is a fraction of a surface, not a colour,
                    // and putting it through a transfer function meant for
                    // light makes every soft edge the wrong softness.
                    a[i * 4..i * 4 + 4].copy_from_slice(&[
                        to_byte_srgb(s.albedo[0]),
                        to_byte_srgb(s.albedo[1]),
                        to_byte_srgb(s.albedo[2]),
                        to_byte_linear(s.alpha),
                    ]);
                    // glTF's packing, which Bevy reads directly: red is free
                    // for ambient occlusion, green is roughness, blue is
                    // metallic.
                    o[i * 4..i * 4 + 4].copy_from_slice(&[
                        255,
                        to_byte_linear(s.roughness),
                        to_byte_linear(s.metallic),
                        255,
                    ]);
                    e[i] = s.emissive;
                }
            });
        }
    });

    // Emission runs hot -- fire's brightest point is several times white -- so
    // the map is normalised and the factor handed back separately. Packing it
    // into eight bits without this would clip every flame to a flat orange slab
    // and throw away the core, which is the only part anyone looks at.
    let mut peak = 1.0f32;
    for e in &emitted {
        peak = peak.max(e[0]).max(e[1]).max(e[2]);
    }
    let mut emissive = Texture::new(n);
    for (i, e) in emitted.iter().enumerate() {
        emissive.pixels[i * 4..i * 4 + 4].copy_from_slice(&[
            to_byte_srgb(e[0] / peak),
            to_byte_srgb(e[1] / peak),
            to_byte_srgb(e[2] / peak),
            255,
        ]);
    }

    // `relief` is in metres and the texel spacing is `extent / n` metres, so
    // the slope is the height difference scaled by their ratio -- which is what
    // makes relief mean the same thing at any resolution. Get that wrong and a
    // material baked at 1024 looks flatter than the same one at 512, which is a
    // genuinely confusing bug to go looking for.
    let mut normal = Texture::new(n);
    let texel = plan.extent / n as f32;
    let slope = if texel > 0.0 {
        surface.relief / texel
    } else {
        0.0
    };
    for y in 0..n {
        for x in 0..n {
            let at = |xx: u32, yy: u32| height[((yy % n) * n + (xx % n)) as usize];
            let dx = (at((x + 1) % n, y) - at((x + n - 1) % n, y)) * 0.5 * slope;
            let dy = (at(x, (y + 1) % n) - at(x, (y + n - 1) % n)) * 0.5 * slope;
            let len = (dx * dx + dy * dy + 1.0).sqrt();
            normal.put(
                x,
                y,
                [
                    to_byte_linear(-dx / len * 0.5 + 0.5),
                    to_byte_linear(-dy / len * 0.5 + 0.5),
                    to_byte_linear(1.0 / len * 0.5 + 0.5),
                    255,
                ],
            );
        }
    }

    let has_alpha = albedo.pixels.chunks(4).any(|p| p[3] < 250);
    Maps {
        albedo,
        normal,
        metallic_roughness: orm,
        emissive,
        emissive_strength: peak,
        has_alpha,
    }
}

fn to_byte_srgb(linear: f32) -> u8 {
    (crate::color::linear_to_srgb(linear.clamp(0.0, 1.0)) * 255.0 + 0.5) as u8
}

fn to_byte_linear(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}
