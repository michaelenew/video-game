//! **What a box is drawn as** (`docs/design/forms.md`): a wall of dressed
//! stone laid in courses, a dry-stone dyke, a cordwood fence, a fallen log, a
//! dead trunk, a carved column, a standing stone, a floating island of turf
//! on rock, a stepping stone that was once a column's drum, a scaffold of
//! planks -- rather than a box.
//!
//! **Every form stays inside its box.** The box is what a body collides
//! against (`sim::arena::Solid`), so a form that poked out of it would be
//! something you walk through, and one far inside it would be air you bump
//! into. Each is built in the box's own coordinates, `-size/2..size/2` about
//! its middle, with its top **at** the box's top wherever a body stands on
//! it; what hangs below a floating island (roots) is the one exception, and
//! it is a few thin strands. `tests::every_form_stays_in_its_box` holds this.
//!
//! Which form a box gets is [`form_of`]: what it is made of, its shape, and
//! where it is -- read off the same table the simulation collides against,
//! so nothing has to be kept in step. Colours are `look::Palette`'s; nothing
//! here decides one.
//!
//! Every form is one mesh, built of parts into a [`Kit`], so a wall of two
//! hundred stones is one draw.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use look::Palette;
use sim::arena::{ArenaId, Material, Solid};

use crate::shapes::hash01;

/// **A mesh being built of parts**: flat-shaded pieces, each a colour, put
/// into one list of triangles.
#[derive(Default)]
pub struct Kit {
    p: Vec<[f32; 3]>,
    n: Vec<[f32; 3]>,
    c: Vec<[f32; 4]>,
    i: Vec<u32>,
}

/// A colour, in linear light, as a vertex colour.
fn lin(rgb: [f32; 3]) -> [f32; 4] {
    let c = look::tint::linear(rgb);
    [c[0], c[1], c[2], 1.0]
}

/// A colour a little off its own, by a number in 0..1: stones in a wall are
/// not one stone.
fn vary(rgb: [f32; 3], k: f32, by: f32) -> [f32; 3] {
    let m = 1.0 + by * (k - 0.5) * 2.0;
    [rgb[0] * m, rgb[1] * m, rgb[2] * m]
}

impl Kit {
    pub fn new() -> Kit {
        Kit::default()
    }

    /// One flat polygon (a triangle fan over `pts`), wound to face `out`.
    pub fn poly(&mut self, pts: &[Vec3], out: Vec3, rgb: [f32; 3]) {
        if pts.len() < 3 {
            return;
        }
        let raw = (pts[1] - pts[0]).cross(pts[2] - pts[0]);
        let flip = raw.dot(out) < 0.0;
        let n = if raw.length_squared() > 1e-12 {
            let n = raw.normalize();
            if flip { -n } else { n }
        } else {
            out.normalize_or(Vec3::Y)
        };
        let base = self.p.len() as u32;
        let c = lin(rgb);
        for p in pts {
            self.p.push(p.to_array());
            self.n.push(n.to_array());
            self.c.push(c);
        }
        for t in 1..pts.len() as u32 - 1 {
            if flip {
                self.i.extend_from_slice(&[base, base + t + 1, base + t]);
            } else {
                self.i.extend_from_slice(&[base, base + t, base + t + 1]);
            }
        }
    }

    /// A box between two corners, its six faces flat. `top` colours its top
    /// face, `rgb` the rest.
    pub fn cube(&mut self, lo: Vec3, hi: Vec3, rgb: [f32; 3], top: [f32; 3]) {
        self.cube_against(lo, hi, rgb, top, Vec3::ZERO);
    }

    /// [`Kit::cube`] laid **against** something on its `against` side -- a
    /// block proud of a wall's core, a flag on its top -- so that face, which
    /// nothing can ever see, is left out. A sixth of every block's triangles,
    /// in a form made of thousands of blocks, drawn six times a frame.
    pub fn cube_against(
        &mut self,
        lo: Vec3,
        hi: Vec3,
        rgb: [f32; 3],
        top: [f32; 3],
        against: Vec3,
    ) {
        let c = |x: f32, y: f32, z: f32| {
            Vec3::new(
                if x > 0.0 { hi.x } else { lo.x },
                if y > 0.0 { hi.y } else { lo.y },
                if z > 0.0 { hi.z } else { lo.z },
            )
        };
        let m = (lo + hi) * 0.5;
        for axis in 0..3 {
            for side in [-1.0f32, 1.0] {
                let (u, w) = ((axis + 1) % 3, (axis + 2) % 3);
                let pts: Vec<Vec3> = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
                    .iter()
                    .map(|&(a, b)| {
                        let mut k = [0.0f32; 3];
                        k[axis] = side;
                        k[u] = a;
                        k[w] = b;
                        c(k[0], k[1], k[2])
                    })
                    .collect();
                let mut out = Vec3::ZERO;
                out[axis] = side;
                if out.dot(against) > 0.5 {
                    continue;
                }
                let col = if axis == 1 && side > 0.0 { top } else { rgb };
                let _ = m;
                self.poly(&pts, out, col);
            }
        }
    }

    /// A box turned `yaw` radians about the vertical through `centre`.
    pub fn turned(&mut self, centre: Vec3, half: Vec3, yaw: f32, rgb: [f32; 3], top: [f32; 3]) {
        let start = self.p.len();
        self.cube(-half, half, rgb, top);
        let q = Quat::from_rotation_y(yaw);
        for k in start..self.p.len() {
            self.p[k] = (q * Vec3::from(self.p[k]) + centre).to_array();
            self.n[k] = (q * Vec3::from(self.n[k])).to_array();
        }
    }

    /// **A tube** from `a` to `b`: `sides` faces round, radius `ra` at `a`
    /// and `rb` at `b`, its ends capped or not. A log, a post, a column's
    /// shaft. `bulge` pushes every other side out by that share, for flutes
    /// or bark.
    #[allow(clippy::too_many_arguments)]
    pub fn tube(
        &mut self,
        a: Vec3,
        b: Vec3,
        ra: f32,
        rb: f32,
        sides: usize,
        rgb: [f32; 3],
        caps: Option<[f32; 3]>,
    ) {
        let axis = (b - a).normalize_or(Vec3::Y);
        let side = if axis.y.abs() < 0.9 {
            axis.cross(Vec3::Y).normalize()
        } else {
            axis.cross(Vec3::X).normalize()
        };
        let up = side.cross(axis).normalize();
        let ring = |at: Vec3, r: f32, k: usize| {
            let t = std::f32::consts::TAU * k as f32 / sides as f32;
            at + (side * t.cos() + up * t.sin()) * r
        };
        for k in 0..sides {
            let (p0, p1) = (ring(a, ra, k), ring(a, ra, k + 1));
            let (q0, q1) = (ring(b, rb, k), ring(b, rb, k + 1));
            let mid = (p0 + p1 + q0 + q1) * 0.25;
            let out = mid - (a + (b - a) * 0.5);
            let out = out - axis * out.dot(axis);
            self.poly(&[p0, p1, q1, q0], out, rgb);
        }
        if let Some(cap) = caps {
            let ends = [(a, ra, -axis), (b, rb, axis)];
            for (at, r, out) in ends {
                let pts: Vec<Vec3> = (0..sides).map(|k| ring(at, r, k)).collect();
                self.poly(&pts, out, cap);
            }
        }
    }

    /// A mesh made elsewhere, moved by `t`, every vertex `rgb`.
    pub fn mesh(&mut self, m: &Mesh, t: Transform, rgb: [f32; 3]) {
        let Some(ps) = m
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|a| a.as_float3())
        else {
            return;
        };
        let ns: Vec<[f32; 3]> = m
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .and_then(|a| a.as_float3())
            .map(|n| n.to_vec())
            .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; ps.len()]);
        let base = self.p.len() as u32;
        let c = lin(rgb);
        let mat = t.compute_matrix();
        let nm = Mat3::from_mat4(mat).inverse().transpose();
        for (p, n) in ps.iter().zip(&ns) {
            self.p.push(mat.transform_point3(Vec3::from(*p)).to_array());
            self.n
                .push((nm * Vec3::from(*n)).normalize_or(Vec3::Y).to_array());
            self.c.push(c);
        }
        match m.indices() {
            Some(Indices::U32(ix)) => self.i.extend(ix.iter().map(|k| k + base)),
            Some(Indices::U16(ix)) => self.i.extend(ix.iter().map(|&k| k as u32 + base)),
            None => self.i.extend((0..ps.len() as u32).map(|k| k + base)),
        }
    }

    /// **A small stone**: an octahedron, its six points pulled about by
    /// `seed`, squashed to `size` -- eight faces, which at a pebble's size
    /// is a pebble.
    pub fn nugget(&mut self, centre: Vec3, size: Vec3, seed: u32, rgb: [f32; 3]) {
        let j = |k: u32| 0.75 + 0.5 * hash01(seed, k, 77);
        let half = size * 0.5;
        let pts = [
            Vec3::new(half.x * j(0), 0.0, 0.0),
            Vec3::new(-half.x * j(1), 0.0, 0.0),
            Vec3::new(0.0, half.y * j(2), 0.0),
            Vec3::new(0.0, -half.y * j(3), 0.0),
            Vec3::new(0.0, 0.0, half.z * j(4)),
            Vec3::new(0.0, 0.0, -half.z * j(5)),
        ];
        let turn = Quat::from_rotation_y(hash01(seed, 6, 77) * std::f32::consts::TAU);
        let p = pts.map(|q| centre + turn * q);
        let top = [rgb[0] * 1.08, rgb[1] * 1.08, rgb[2] * 1.08];
        for (a, b) in [(0, 4), (4, 1), (1, 5), (5, 0)] {
            for (y, c) in [(2, top), (3, rgb)] {
                let out = (p[a] + p[b] + p[y]) / 3.0 - centre;
                self.poly(&[p[a], p[b], p[y]], out, c);
            }
        }
    }

    /// A mesh made elsewhere, moved by `t`, keeping the colours it has.
    pub fn keep(&mut self, m: &Mesh, t: Transform) {
        let from = self.p.len();
        self.mesh(m, t, [1.0, 1.0, 1.0]);
        if let Some(bevy::render::mesh::VertexAttributeValues::Float32x4(cs)) =
            m.attribute(Mesh::ATTRIBUTE_COLOR)
        {
            for (k, c) in cs.iter().enumerate() {
                if let Some(slot) = self.c.get_mut(from + k) {
                    *slot = *c;
                }
            }
        }
    }

    /// **Shade what is near the ground**: every vertex within `reach` of
    /// `floor` darkened, by `amount` at the floor -- the dark that collects
    /// at the foot of anything, which is most of what makes a thing stand on
    /// the ground rather than float over it.
    pub fn settle(&mut self, floor: f32, reach: f32, amount: f32) {
        for (p, c) in self.p.iter().zip(self.c.iter_mut()) {
            let t = ((p[1] - floor) / reach).clamp(0.0, 1.0);
            let k = 1.0 - amount * (1.0 - t) * (1.0 - t);
            c[0] *= k;
            c[1] *= k;
            c[2] *= k;
        }
    }

    /// Colour every vertex of what has been added since `from` by `f` of
    /// where it is and which way it faces.
    pub fn recolour(&mut self, from: usize, f: impl Fn(Vec3, Vec3, [f32; 4]) -> [f32; 4]) {
        for k in from..self.p.len() {
            self.c[k] = f(Vec3::from(self.p[k]), Vec3::from(self.n[k]), self.c[k]);
        }
    }

    /// How many vertices so far.
    pub fn len(&self) -> usize {
        self.p.len()
    }

    /// Is it empty?
    pub fn is_empty(&self) -> bool {
        self.p.is_empty()
    }

    pub fn build(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.p)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.n)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.c)
        .with_inserted_indices(Indices::U32(self.i))
    }
}

// ---------------------------------------------------------------------------
// Which form
// ---------------------------------------------------------------------------

/// What a box is drawn as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// Dressed stone, laid in courses, flagged on top.
    Masonry,
    /// Field stones stacked without mortar: a low wall of rock.
    DryStone,
    /// Logs stacked lengthways between posts: a fence of cordwood.
    Cordwood,
    /// A fallen trunk.
    Log,
    /// A dead tree, broken off.
    Trunk,
    /// A carved column: plinth, shaft, capital.
    Column,
    /// A standing stone.
    Menhir,
    /// A pillar of rock.
    Spire,
    /// A floating island: turf on a lump of rock.
    Island,
    /// A stepping stone: a column's drum, fallen.
    Drum,
    /// A scaffold: a deck of planks on beams and posts.
    Deck,
    /// Whatever `arenas::draw_solid` drew before there were forms: a rock,
    /// a cliff, a soft mound.
    Plain,
}

/// Is this arena a jump course?
fn course(id: ArenaId) -> bool {
    (ArenaId::CLIMB_STAIR.0..=ArenaId::CLIMB_REACH.0).contains(&id.0)
}

/// **Which form a box is drawn as**, from what it is made of, its size, and
/// whether it floats: one rule for every arena, so a new arena's walls come
/// out walls without anybody choosing.
pub fn form_of(arena: ArenaId, s: &Solid, hangs: bool) -> Form {
    let f = |v: sim::Fx| v.to_f32_for_render();
    let size = Vec3::new(
        f(s.max.x) - f(s.min.x),
        f(s.max.y) - f(s.min.y),
        f(s.max.z) - f(s.min.z),
    );
    let (long, thin) = (size.x.max(size.z), size.x.min(size.z));
    let tall = size.y >= 2.5 && size.y >= 1.4 * long;
    if course(arena) {
        return match s.material {
            Material::Grass | Material::Snow | Material::Wood if hangs => Form::Island,
            // Anything standing on the floor far below is a pillar of rock,
            // its top whatever it is made of.
            m if !hangs && tall && m != Material::Wood => Form::Spire,
            Material::Sand => Form::Drum,
            Material::Wood => Form::Deck,
            Material::Rock if tall => Form::Spire,
            Material::Stone => Form::Masonry,
            _ => Form::Plain,
        };
    }
    match s.material {
        Material::Stone if tall && size.y >= 6.0 && long <= 2.5 => Form::Column,
        Material::Stone if tall && long <= 2.5 => Form::Menhir,
        Material::Stone => Form::Masonry,
        Material::Rock if tall && !hangs => Form::Spire,
        // A long low run of rock: a dyke of field stones.
        Material::Rock if !hangs && size.y <= 3.5 && thin <= 2.0 && long >= 3.0 => Form::DryStone,
        Material::Wood if tall && long <= 2.5 => Form::Trunk,
        Material::Wood if size.y >= 2.0 && long >= 3.0 * thin => Form::Cordwood,
        Material::Wood if long >= 2.5 * thin && size.y <= 2.0 => Form::Log,
        Material::Wood if hangs => Form::Deck,
        _ => Form::Plain,
    }
}

/// A seed for a box: where it is, so the same box is the same every time.
pub fn seed_of(centre: Vec3, size: Vec3) -> u32 {
    (centre.x * 7.0 + centre.z * 13.0 + size.y * 3.0) as i32 as u32
}

/// **The form, built**: a mesh in the box's own coordinates, or `None` for
/// [`Form::Plain`], which `arenas::draw_solid` draws as it always has.
///
/// `near` is how closely it will be seen: a form far from the camera keeps
/// its colours, block by block, and loses relief nobody could see -- a block
/// five centimetres proud of its wall is under a pixel past fifty metres
/// (`crate::stream` decides which, by distance).
///
/// The second mesh, if there is one, is **relief that casts no shadow**:
/// the blocks of a wall, standing a few centimetres proud of a core that
/// casts the wall's shadow as well as they would. Ten triangles a block in
/// four shadow cascades was most of what the shadow passes drew in the
/// town; the core is twelve.
pub fn build_parts(
    form: Form,
    size: Vec3,
    seed: u32,
    material: Material,
    p: &Palette,
    near: bool,
) -> Option<(Mesh, Option<Mesh>)> {
    let mut kit = Kit::new();
    let mut relief = Kit::new();
    match form {
        Form::Masonry => masonry(&mut kit, &mut relief, size, seed, p, near),
        Form::DryStone => dry_stone(&mut kit, size, seed, p),
        Form::Cordwood => cordwood(&mut kit, size, seed, p),
        Form::Log => log(&mut kit, size, seed, p),
        Form::Trunk => trunk(&mut kit, size, seed, p),
        Form::Column => column(&mut kit, size, seed, p),
        Form::Menhir => menhir(&mut kit, size, seed, p),
        Form::Spire => spire(&mut kit, size, seed, material, p),
        Form::Island => island(&mut kit, size, seed, material, p),
        Form::Drum => drum(&mut kit, size, seed, material, p),
        Form::Deck => deck(&mut kit, size, seed, p),
        Form::Plain => return None,
    }
    let (floor, reach) = (-size.y * 0.5, 0.6_f32.min(size.y * 0.5));
    kit.settle(floor, reach, 0.3);
    relief.settle(floor, reach, 0.3);
    Some((kit.build(), (!relief.is_empty()).then(|| relief.build())))
}

// ---------------------------------------------------------------------------
// The forms
// ---------------------------------------------------------------------------

/// The four upright faces of a box: its outward normal, the axis it runs
/// along, and its length along it.
fn faces(size: Vec3) -> [(Vec3, Vec3, f32); 4] {
    [
        (Vec3::X, Vec3::Z, size.z),
        (Vec3::NEG_X, Vec3::Z, size.z),
        (Vec3::Z, Vec3::X, size.x),
        (Vec3::NEG_Z, Vec3::X, size.x),
    ]
}

/// **Dressed stone**: a core of mortar, every upright face laid in courses
/// of blocks a little proud of it, the top flagged. Blocks grow with the
/// face, so a wall of fifty metres is not ten thousand stones.
///
/// Seen from afar (`near` false) each block is only its face, flush with the
/// box: the same courses in the same colours, two triangles a block rather
/// than ten.
fn masonry(kit: &mut Kit, relief: &mut Kit, size: Vec3, seed: u32, p: &Palette, near: bool) {
    let half = size * 0.5;
    let skin = 0.05f32.min(size.min_element() * 0.2);
    let stone = p.of(Material::Stone);
    let mortar = p.mortar();
    let moss = p.moss();
    kit.cube(
        -half + Vec3::new(skin, 0.0, skin),
        half - Vec3::splat(skin),
        mortar,
        mortar,
    );
    let joint = 0.03;
    let course = (size.y / 28.0).max(0.34);
    for (fi, (out, along, length)) in faces(size).into_iter().enumerate() {
        let depth = (out * half).abs().max_element();
        let unit = (length / 36.0).max(0.55);
        let mut y = -half.y;
        let mut row = 0u32;
        while y < half.y - 0.02 {
            let h = (course * (0.8 + 0.45 * hash01(seed, row, 1 + fi as u32 * 7))).min(half.y - y);
            let mut a = -length * 0.5;
            let mut k = 0u32;
            // Each course starts a part-block in, so joints never line up.
            let shift = unit * hash01(seed, row, 3);
            let mut first = true;
            while a < length * 0.5 - 0.01 {
                let w = if first {
                    shift.max(0.2)
                } else {
                    unit * (0.7 + 0.7 * hash01(seed ^ row, k, 5 + fi as u32))
                };
                first = false;
                let b = (a + w).min(length * 0.5);
                let r = hash01(seed.wrapping_add(fi as u32 * 131), row * 97 + k, 9);
                let proud = skin * (0.55 + 0.45 * r);
                let lo_a = a + if a > -length * 0.5 + 0.001 {
                    joint
                } else {
                    0.0
                };
                if b - lo_a < 0.05 {
                    a = b;
                    k += 1;
                    continue;
                }
                let lo = out * (depth - proud) + along * lo_a + Vec3::Y * (y + joint * 0.5);
                let hi = out * depth + along * b + Vec3::Y * (y + h - joint * 0.5);
                let col = vary(stone, r, 0.10);
                // Old stone near the top greens over.
                let top_col = if y + h > half.y - 0.05 && r > 0.6 {
                    look::tint::mix(col, moss, 0.5)
                } else {
                    col
                };
                if near {
                    relief.cube_against(lo.min(hi), lo.max(hi), col, top_col, -out);
                } else {
                    let (y0, y1) = (lo.y, hi.y);
                    let face = out * depth;
                    let (a0, a1) = (along * lo_a, along * b);
                    relief.poly(
                        &[
                            face + a0 + Vec3::Y * y0,
                            face + a1 + Vec3::Y * y0,
                            face + a1 + Vec3::Y * y1,
                            face + a0 + Vec3::Y * y1,
                        ],
                        out,
                        col,
                    );
                }
                a = b;
                k += 1;
            }
            y += h;
            row += 1;
        }
    }
    // The top: flags, a hair under the box's top at the joints and level
    // with it on the stones.
    let unit = (size.x.max(size.z) / 24.0).clamp(0.6, 1.6);
    let nx = ((size.x / unit).ceil() as u32).max(1);
    let nz = ((size.z / unit).ceil() as u32).max(1);
    for j in 0..nz {
        for i in 0..nx {
            let x0 = -half.x + size.x * i as f32 / nx as f32;
            let x1 = -half.x + size.x * (i + 1) as f32 / nx as f32;
            let z0 = -half.z + size.z * j as f32 / nz as f32;
            let z1 = -half.z + size.z * (j + 1) as f32 / nz as f32;
            let r = hash01(seed, i * 131 + j, 21);
            let col = vary(stone, r, 0.08);
            let col = if r > 0.85 {
                look::tint::mix(col, moss, 0.35)
            } else {
                col
            };
            let (lo, hi) = (
                Vec3::new(x0 + joint * 0.5, half.y - skin, z0 + joint * 0.5),
                Vec3::new(x1 - joint * 0.5, half.y, z1 - joint * 0.5),
            );
            if near {
                relief.cube_against(lo, hi, col, col, Vec3::NEG_Y);
            } else {
                relief.poly(
                    &[
                        Vec3::new(lo.x, hi.y, lo.z),
                        Vec3::new(hi.x, hi.y, lo.z),
                        Vec3::new(hi.x, hi.y, hi.z),
                        Vec3::new(lo.x, hi.y, hi.z),
                    ],
                    Vec3::Y,
                    col,
                );
            }
        }
    }
}

/// **A dry-stone dyke**: field stones piled the length of the wall, each
/// through its thickness, the top course flat where it is stood on.
fn dry_stone(kit: &mut Kit, size: Vec3, seed: u32, p: &Palette) {
    let half = size * 0.5;
    let rock = p.of(Material::Rock);
    let moss = p.moss();
    let along_x = size.x >= size.z;
    let (length, thick) = if along_x {
        (size.x, size.z)
    } else {
        (size.z, size.x)
    };
    // A dark core, for the gaps between stones.
    let core = vary(rock, 0.0, 0.25);
    kit.cube(
        -half + Vec3::new(0.12, 0.0, 0.12).min(half * 0.5),
        half - Vec3::new(0.12, 0.05, 0.12).min(half * 0.5),
        core,
        core,
    );
    let rows = ((size.y / 0.5).round() as u32).max(1);
    let h = size.y / rows as f32;
    for row in 0..rows {
        let mut a = -length * 0.5;
        let mut k = 0u32;
        while a < length * 0.5 - 0.05 {
            let w = (0.55 + 0.6 * hash01(seed, row * 211 + k, 2)).min(length * 0.5 - a);
            let r = hash01(seed, row * 211 + k, 4);
            let top = row + 1 == rows;
            let sz = if along_x {
                Vec3::new(w, h * (1.0 + 0.2 * r), thick)
            } else {
                Vec3::new(thick, h * (1.0 + 0.2 * r), w)
            };
            let sz = sz.min(size);
            let mut y = -half.y + h * row as f32 + sz.y * 0.5;
            if top {
                y = half.y - sz.y * 0.5;
            }
            let c = a + w * 0.5;
            let at = if along_x {
                Vec3::new(c, y, 0.0)
            } else {
                Vec3::new(0.0, y, c)
            };
            let stone = crate::shapes::rock(sz, seed ^ (row * 7919 + k), None);
            let col = vary(rock, r, 0.14);
            let col = if top && r > 0.45 {
                look::tint::mix(col, moss, 0.45)
            } else {
                col
            };
            kit.mesh(&stone, Transform::from_translation(at), col);
            a += w;
            k += 1;
        }
    }
}

/// **A cordwood fence**: logs laid lengthways in rows, staggered, between
/// posts; the top row level with the box's top, where it is a ledge.
fn cordwood(kit: &mut Kit, size: Vec3, seed: u32, p: &Palette) {
    let half = size * 0.5;
    let wood = p.of(Material::Wood);
    let bark = p.bark();
    let along_x = size.x >= size.z;
    let (length, thick) = if along_x {
        (size.x, size.z)
    } else {
        (size.z, size.x)
    };
    let dir = if along_x { Vec3::X } else { Vec3::Z };
    let across = if along_x { Vec3::Z } else { Vec3::X };
    // The dark between the logs.
    kit.cube(
        -half + Vec3::new(0.15, 0.0, 0.15).min(half * 0.5),
        half - Vec3::new(0.15, 0.1, 0.15).min(half * 0.5),
        vary(bark, 0.0, 0.3),
        bark,
    );
    let r = 0.22f32.min(thick * 0.25).min(size.y * 0.25);
    let rows = ((size.y / (2.0 * r)).floor() as u32).max(1);
    let cols = ((thick / (2.0 * r)).floor() as u32).max(1);
    let span = 3.2f32;
    for row in 0..rows {
        let y = half.y - r - row as f32 * (size.y - 2.0 * r) / (rows.max(2) - 1) as f32;
        for col in 0..cols {
            let off = -thick * 0.5 + r + col as f32 * (thick - 2.0 * r) / (cols.max(2) - 1) as f32;
            let off = if cols == 1 { 0.0 } else { off };
            let mut a = -length * 0.5 + span * hash01(seed, row * 31 + col, 1) * 0.6;
            let mut first = true;
            let mut k = 0;
            while a < length * 0.5 - 0.05 {
                let start = if first { -length * 0.5 } else { a };
                first = false;
                let end = (a + span * (0.8 + 0.4 * hash01(seed, row * 31 + col + k * 7, 2)))
                    .min(length * 0.5);
                let rr = r * (0.85 + 0.15 * hash01(seed, row + k * 13, col + 3));
                let shade = vary(bark, hash01(seed, row * 5 + k, col + 9), 0.18);
                kit.tube(
                    dir * start + across * off + Vec3::Y * y,
                    dir * end + across * off + Vec3::Y * y,
                    rr,
                    rr,
                    7,
                    shade,
                    Some(vary(wood, hash01(seed, k, row), 0.12)),
                );
                a = end;
                k += 1;
            }
        }
    }
    // Posts, both faces, every few metres.
    let posts = ((length / 3.5).round() as u32).max(1);
    for k in 0..=posts {
        let a = -length * 0.5 + length * k as f32 / posts as f32;
        let a = a.clamp(-length * 0.5 + 0.16, length * 0.5 - 0.16);
        for side in [-1.0f32, 1.0] {
            let at = dir * a + across * side * (thick * 0.5 - 0.15);
            kit.tube(
                at - Vec3::Y * half.y,
                at + Vec3::Y * (half.y - 0.02),
                0.15,
                0.13,
                6,
                vary(bark, 0.2, 0.1),
                Some(wood),
            );
        }
    }
}

/// **A fallen trunk**: a log along the box's length -- as round as the box
/// lets it be, squashed to fit -- with its sawn or broken ends, a root plate
/// at one of them, and a few branch stubs.
fn log(kit: &mut Kit, size: Vec3, seed: u32, p: &Palette) {
    let half = size * 0.5;
    let bark = p.bark();
    let wood = p.of(Material::Wood);
    let along_x = size.x >= size.z;
    let (length, thick) = if along_x {
        (size.x, size.z)
    } else {
        (size.z, size.x)
    };
    let dir = if along_x { Vec3::X } else { Vec3::Z };
    let across = if along_x { Vec3::Z } else { Vec3::X };
    let (rw, rh) = (thick * 0.5, half.y);
    let sides = 10;
    let rings = ((length / 0.8).ceil() as usize).max(2);
    // The trunk, ring by ring, its section an ellipse that fits the box,
    // narrowing a little from the root end.
    let at = |t: f32, k: usize| {
        let a = std::f32::consts::TAU * k as f32 / sides as f32;
        let taper = 1.0 - 0.12 * t;
        let lump = 1.0 - 0.06 * hash01(seed, (t * 50.0) as u32, k as u32);
        dir * (-length * 0.5 + length * t)
            + across * a.cos() * rw * taper * lump
            + Vec3::Y * a.sin() * rh * taper * lump
    };
    for j in 0..rings {
        let (t0, t1) = (j as f32 / rings as f32, (j + 1) as f32 / rings as f32);
        for k in 0..sides {
            let quad = [at(t0, k), at(t0, k + 1), at(t1, k + 1), at(t1, k)];
            let mid = (quad[0] + quad[1] + quad[2] + quad[3]) * 0.25;
            let out = mid - dir * mid.dot(dir);
            let col = vary(bark, hash01(seed, j as u32, k as u32), 0.16);
            // The top of a log lying out greens over.
            let col = if out.y > 0.5 * rh && hash01(seed, j as u32, 77) > 0.5 {
                look::tint::mix(col, p.moss(), 0.4)
            } else {
                col
            };
            kit.poly(&quad, out, col);
        }
    }
    for (t, out) in [(0.0f32, -dir), (1.0, dir)] {
        let pts: Vec<Vec3> = (0..sides).map(|k| at(t, k)).collect();
        kit.poly(&pts, out, wood);
    }
    // A root plate at the near end: stubs of root reaching out, inside the
    // box.
    for k in 0..6u32 {
        let a = std::f32::consts::TAU * k as f32 / 6.0 + hash01(seed, k, 3);
        let from = dir * (-length * 0.5 + 0.2);
        let reach = Vec3::new(0.0, a.sin() * rh * 0.92, 0.0) + across * a.cos() * rw * 0.92;
        kit.tube(from, from + reach - dir * 0.15, 0.09, 0.04, 5, bark, None);
    }
    // Stubs of branches, up and out, short enough to stay in.
    for k in 0..3u32 {
        let t = 0.3 + 0.5 * hash01(seed, k, 11);
        let from = dir * (-length * 0.5 + length * t) + Vec3::Y * rh * 0.6;
        let side = if k % 2 == 0 { 1.0 } else { -1.0 };
        let to = from + across * side * rw * 0.3 + Vec3::Y * rh * 0.35;
        kit.tube(from, to, 0.08, 0.05, 5, bark, Some(wood));
    }
}

/// **A dead trunk**, broken off: tapering, its foot flared into roots, its
/// top a jagged break round a flat heart -- since its top is somewhere to
/// stand.
fn trunk(kit: &mut Kit, size: Vec3, seed: u32, p: &Palette) {
    let half = size * 0.5;
    let bark = p.bark();
    let wood = p.of(Material::Wood);
    let r0 = half.x.min(half.z);
    let sides = 11;
    let rings = 8;
    let at = |t: f32, k: usize| -> Vec3 {
        let a = std::f32::consts::TAU * k as f32 / sides as f32;
        // Flared at the foot, tapering, ridged.
        let flare = 1.0 + 0.0 * t;
        let r = r0 * (0.68 + 0.32 * (1.0 - t).powi(6)) * flare * (1.0 - 0.18 * t);
        let ridge = 1.0 - 0.08 * ((k % 2) as f32);
        let jag = if t >= 1.0 {
            0.15 + 0.55 * hash01(seed, k as u32, 1)
        } else {
            0.0
        };
        Vec3::new(
            a.cos() * r * ridge,
            -half.y + size.y * t - jag,
            a.sin() * r * ridge,
        )
    };
    for j in 0..rings {
        let (t0, t1) = (j as f32 / rings as f32, (j + 1) as f32 / rings as f32);
        for k in 0..sides {
            let quad = [at(t0, k), at(t0, k + 1), at(t1, k + 1), at(t1, k)];
            let mid = (quad[0] + quad[1] + quad[2] + quad[3]) * 0.25;
            let col = vary(bark, hash01(seed, j, k as u32), 0.14);
            kit.poly(&quad, Vec3::new(mid.x, 0.0, mid.z), col);
        }
    }
    // The break: the jagged rim down to a flat heart at the box's top.
    let heart = r0 * 0.45;
    for k in 0..sides {
        let (a0, a1) = (at(1.0, k), at(1.0, k + 1));
        let ang = |k: usize| std::f32::consts::TAU * k as f32 / sides as f32;
        let h0 = Vec3::new(ang(k).cos() * heart, half.y, ang(k).sin() * heart);
        let h1 = Vec3::new(ang(k + 1).cos() * heart, half.y, ang(k + 1).sin() * heart);
        kit.poly(
            &[a0, a1, h1, h0],
            Vec3::Y,
            vary(wood, hash01(seed, k as u32, 5), 0.1),
        );
    }
    let hearts: Vec<Vec3> = (0..sides)
        .map(|k| {
            let a = std::f32::consts::TAU * k as f32 / sides as f32;
            Vec3::new(a.cos() * heart, half.y, a.sin() * heart)
        })
        .collect();
    kit.poly(&hearts, Vec3::Y, wood);
}

/// **A carved column**: a square plinth, a fluted shaft, a capital under a
/// square abacus at the box's top. Weathered: the stones of the shaft vary,
/// and moss climbs its foot.
fn column(kit: &mut Kit, size: Vec3, seed: u32, p: &Palette) {
    let half = size * 0.5;
    let stone = p.of(Material::Stone);
    let w = half.x.min(half.z);
    let plinth = 0.45f32.min(size.y * 0.1);
    kit.cube(
        Vec3::new(-half.x, -half.y, -half.z),
        Vec3::new(half.x, -half.y + plinth, half.z),
        vary(stone, 0.4, 0.1),
        stone,
    );
    kit.cube(
        Vec3::new(-half.x * 0.86, -half.y + plinth, -half.z * 0.86),
        Vec3::new(half.x * 0.86, -half.y + plinth + 0.18, half.z * 0.86),
        vary(stone, 0.6, 0.1),
        stone,
    );
    let abacus = 0.3f32.min(size.y * 0.06);
    let top = half.y - abacus;
    let neck = top - 0.35;
    let foot = -half.y + plinth + 0.18;
    // The shaft, in drums.
    let drums = ((neck - foot) / 1.1).round().max(1.0) as u32;
    for d in 0..drums {
        let y0 = foot + (neck - foot) * d as f32 / drums as f32;
        let y1 = foot + (neck - foot) * (d + 1) as f32 / drums as f32;
        let r = w * 0.66;
        let col = vary(stone, hash01(seed, d, 3), 0.09);
        let col = if d == 0 {
            look::tint::mix(col, p.moss(), 0.35)
        } else {
            col
        };
        // Fluted: sixteen faces, every other one set back.
        let sides = 16;
        for k in 0..sides {
            let a0 = std::f32::consts::TAU * k as f32 / sides as f32;
            let a1 = std::f32::consts::TAU * (k + 1) as f32 / sides as f32;
            let rr = if k % 2 == 0 { r } else { r * 0.93 };
            let pt = |a: f32, y: f32, taper: f32| {
                Vec3::new(a.cos() * rr * taper, y, a.sin() * rr * taper)
            };
            let t0 = 1.0 - 0.06 * (y0 - foot) / (neck - foot);
            let t1 = 1.0 - 0.06 * (y1 - foot) / (neck - foot);
            let quad = [
                pt(a0, y0, t0),
                pt(a1, y0, t0),
                pt(a1, y1 - 0.02, t1),
                pt(a0, y1 - 0.02, t1),
            ];
            let mid = (quad[0] + quad[2]) * 0.5;
            kit.poly(&quad, Vec3::new(mid.x, 0.0, mid.z), col);
        }
    }
    // The capital: a flaring echinus, then the abacus flush with the top.
    kit.tube(
        Vec3::Y * neck,
        Vec3::Y * top,
        w * 0.62,
        w * 0.92,
        16,
        vary(stone, 0.7, 0.08),
        Some(stone),
    );
    kit.cube(
        Vec3::new(-half.x, top, -half.z),
        Vec3::new(half.x, half.y, half.z),
        vary(stone, 0.55, 0.08),
        vary(stone, 0.5, 0.05),
    );
}

/// **A standing stone**: a pillar of grey stone, rough, with a carved band
/// round it two-thirds up -- somebody set it there.
fn menhir(kit: &mut Kit, size: Vec3, seed: u32, p: &Palette) {
    let stone = vary(p.of(Material::Stone), 0.35, 0.1);
    let m = unit_column(seed, size);
    let from = kit.len();
    kit.mesh(&m, Transform::IDENTITY, stone);
    let band = size.y * 0.16;
    let mortar = p.mortar();
    let moss = p.moss();
    kit.recolour(from, |pos, n, c| {
        let t = pos.y / size.y;
        if (t - band / size.y).abs() < 0.035 && n.y.abs() < 0.6 {
            lin(mortar)
        } else if t < -0.35 && n.y.abs() < 0.8 {
            let m = lin(moss);
            [
                c[0] * 0.5 + m[0] * 0.5,
                c[1] * 0.5 + m[1] * 0.5,
                c[2] * 0.5 + m[2] * 0.5,
                1.0,
            ]
        } else {
            c
        }
    });
}

/// `shapes::rock_column`, stretched to a box.
fn unit_column(seed: u32, size: Vec3) -> Mesh {
    let mut m = crate::shapes::rock_column(seed);
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(ps)) =
        m.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for q in ps.iter_mut() {
            *q = [q[0] * size.x, q[1] * size.y, q[2] * size.z];
        }
    }
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(ns)) =
        m.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
    {
        for n in ns.iter_mut() {
            let v = Vec3::new(n[0] / size.x, n[1] / size.y, n[2] / size.z).normalize_or(Vec3::Y);
            *n = v.to_array();
        }
    }
    m
}

/// **A pillar of rock**: a crag, banded -- grey rock, a little lighter in
/// layers, grass or snow on its top if that is what its top is.
fn spire(kit: &mut Kit, size: Vec3, seed: u32, material: Material, p: &Palette) {
    let rock = p.crag();
    let m = unit_column(seed, size);
    let from = kit.len();
    kit.mesh(&m, Transform::IDENTITY, rock);
    let top = p.of(material);
    let moss = p.moss();
    let half = size.y * 0.5;
    kit.recolour(from, |pos, n, c| {
        if n.y > 0.7 && pos.y > half - 0.05 {
            lin(top)
        } else {
            // Strata: lighter and darker layers up the face.
            let layer = ((pos.y * 0.7 + seed as f32 * 0.01).sin() * 0.5 + 0.5) * 0.14 + 0.93;
            let k = if n.y > 0.35 { 0.0 } else { 1.0 };
            let c = [c[0] * layer, c[1] * layer, c[2] * layer, 1.0];
            if k == 0.0 {
                let m = lin(moss);
                [
                    c[0] * 0.4 + m[0] * 0.6,
                    c[1] * 0.4 + m[1] * 0.6,
                    c[2] * 0.4 + m[2] * 0.6,
                    1.0,
                ]
            } else {
                c
            }
        }
    });
}

/// **A floating island**: a slab of turf (or snow) flat at the box's top
/// with a lip of soil, over a lump of rock tapering away beneath it, and a
/// few roots hanging out of the bottom.
fn island(kit: &mut Kit, size: Vec3, seed: u32, material: Material, p: &Palette) {
    let half = size * 0.5;
    let top = p.of(material);
    let soil = p.soil();
    let rock = p.of(Material::Rock);
    let spokes = 20usize;
    let rings = 6usize;
    let s = seed as f32 * 0.013;
    // The footprint at angle `a`: a rounded square, pulled in a little by
    // noise, so the island's edge is not a box's.
    let rim = |a: f32, k: f32| -> Vec3 {
        let (sa, ca) = a.sin_cos();
        let e = 0.45;
        let sx = ca.signum() * ca.abs().powf(e);
        let sz = sa.signum() * sa.abs().powf(e);
        let wob = 1.0 - 0.07 * (0.5 + 0.5 * ((a * 3.0 + s).sin() * (a * 5.0 - s).cos()));
        Vec3::new(sx * half.x * wob * k, 0.0, sz * half.z * wob * k)
    };
    // Profile, top down: the turf (0..turf), then the rock tapering to a
    // rounded point a little above the box's bottom.
    let turf = 0.35f32.min(size.y * 0.25);
    let profile = |j: usize| -> (f32, f32) {
        if j == 0 {
            (half.y, 1.0)
        } else if j == 1 {
            (half.y - turf, 0.98)
        } else {
            let t = (j - 1) as f32 / (rings - 1) as f32;
            let y = half.y - turf - (size.y - turf) * t;
            let k = (1.0 - t * t).max(0.0).sqrt() * 0.92 + 0.06;
            (y, k)
        }
    };
    let point = |j: usize, i: usize| -> Vec3 {
        let a = std::f32::consts::TAU * i as f32 / spokes as f32;
        let (y, k) = profile(j);
        let lump = if j >= 2 {
            1.0 - 0.18 * hash01(seed, (j * 31 + i) as u32, 3)
        } else {
            1.0
        };
        rim(a, k * lump) + Vec3::Y * y
    };
    for j in 0..rings {
        for i in 0..spokes {
            let quad = [
                point(j, i),
                point(j, i + 1),
                point(j + 1, i + 1),
                point(j + 1, i),
            ];
            let mid = (quad[0] + quad[2]) * 0.5;
            let col = if j == 0 {
                soil
            } else {
                vary(rock, hash01(seed, (j * 31 + i) as u32, 7), 0.12)
            };
            kit.poly(&quad, Vec3::new(mid.x, -0.3, mid.z), col);
        }
    }
    // The top, flat: turf or snow -- or, for the nest, a deck of planks laid
    // over the rock, its edge the island's.
    let pts: Vec<Vec3> = (0..spokes).map(|i| point(0, i)).collect();
    if material == Material::Wood {
        kit.poly(&pts, Vec3::Y, p.soil());
        let n = ((size.z / 0.45).floor() as u32).max(2);
        for k in 0..n {
            let z0 = -half.z * 0.86 + size.z * 0.86 * k as f32 / n as f32;
            let z1 = z0 + size.z * 0.86 / n as f32 - 0.04;
            let c = vary(top, hash01(seed, k, 12), 0.12);
            kit.cube(
                Vec3::new(-half.x * 0.86, half.y - 0.1, z0),
                Vec3::new(half.x * 0.86, half.y, z1),
                vary(c, 0.3, 0.1),
                c,
            );
        }
    } else {
        kit.poly(&pts, Vec3::Y, top);
    }
    // A few roots out of the underside: short, and allowed out.
    for k in 0..3u32 {
        let a = std::f32::consts::TAU * hash01(seed, k, 9);
        let at = rim(a, 0.45) + Vec3::Y * (half.y - turf - size.y * 0.35);
        let len = 0.5 + 1.0 * hash01(seed, k, 10);
        kit.tube(
            at,
            at + Vec3::new(0.12, -len, 0.08),
            0.09,
            0.03,
            5,
            vary(soil, 0.3, 0.1),
            None,
        );
    }
}

/// **A stepping stone**: a column's drum, fallen and set upright -- round,
/// fluted, its top flat and worn.
fn drum(kit: &mut Kit, size: Vec3, seed: u32, material: Material, p: &Palette) {
    let half = size * 0.5;
    let stone = p.of(material);
    let r = half.x.min(half.z);
    let sides = 18;
    let lip = 0.12f32.min(size.y * 0.2);
    for k in 0..sides {
        let a0 = std::f32::consts::TAU * k as f32 / sides as f32;
        let a1 = std::f32::consts::TAU * (k + 1) as f32 / sides as f32;
        let rr = if k % 2 == 0 { r } else { r * 0.94 };
        let chip = 1.0 - 0.1 * hash01(seed, k, 2);
        let pt = |a: f32, y: f32, f: f32| Vec3::new(a.cos() * rr * f, y, a.sin() * rr * f);
        let col = vary(stone, hash01(seed, k, 4), 0.08);
        kit.poly(
            &[
                pt(a0, -half.y, 0.9),
                pt(a1, -half.y, 0.9),
                pt(a1, half.y - lip, chip),
                pt(a0, half.y - lip, chip),
            ],
            Vec3::new(a0.cos() + a1.cos(), 0.0, a0.sin() + a1.sin()),
            col,
        );
        kit.poly(
            &[
                pt(a0, half.y - lip, chip),
                pt(a1, half.y - lip, chip),
                pt(a1, half.y, 0.9),
                pt(a0, half.y, 0.9),
            ],
            Vec3::new(a0.cos() + a1.cos(), 1.0, a0.sin() + a1.sin()),
            vary(stone, 0.7, 0.05),
        );
    }
    let pts: Vec<Vec3> = (0..sides)
        .map(|k| {
            let a = std::f32::consts::TAU * k as f32 / sides as f32;
            let rr = if k % 2 == 0 { r } else { r * 0.94 };
            Vec3::new(a.cos() * rr * 0.9, half.y, a.sin() * rr * 0.9)
        })
        .collect();
    kit.poly(&pts, Vec3::Y, vary(stone, 0.6, 0.04));
}

/// **A scaffold**: a deck of planks at the box's top, on beams, on posts
/// braced corner to corner.
fn deck(kit: &mut Kit, size: Vec3, seed: u32, p: &Palette) {
    let half = size * 0.5;
    let wood = p.of(Material::Wood);
    let dark = p.framing();
    let plank = 0.12f32.min(size.y * 0.2);
    let along_x = size.x >= size.z;
    let n = (((if along_x { size.z } else { size.x }) / 0.4).floor() as u32).max(1);
    for k in 0..n {
        let t0 = k as f32 / n as f32;
        let t1 = (k + 1) as f32 / n as f32;
        let col = vary(wood, hash01(seed, k, 1), 0.12);
        let (lo, hi) = if along_x {
            (
                Vec3::new(-half.x, half.y - plank, -half.z + size.z * t0 + 0.02),
                Vec3::new(half.x, half.y, -half.z + size.z * t1 - 0.02),
            )
        } else {
            (
                Vec3::new(-half.x + size.x * t0 + 0.02, half.y - plank, -half.z),
                Vec3::new(-half.x + size.x * t1 - 0.02, half.y, half.z),
            )
        };
        kit.cube(lo, hi, vary(col, 0.3, 0.1), col);
    }
    // Beams under the planks, across them.
    let beam = 0.25f32.min(size.y * 0.25);
    let beams = (((if along_x { size.x } else { size.z }) / 1.6).ceil() as u32).max(2);
    for k in 0..=beams {
        let t = -0.5 + k as f32 / beams as f32;
        let (lo, hi) = if along_x {
            let x = (t * size.x).clamp(-half.x + 0.12, half.x - 0.12);
            (
                Vec3::new(x - 0.12, half.y - plank - beam, -half.z),
                Vec3::new(x + 0.12, half.y - plank, half.z),
            )
        } else {
            let z = (t * size.z).clamp(-half.z + 0.12, half.z - 0.12);
            (
                Vec3::new(-half.x, half.y - plank - beam, z - 0.12),
                Vec3::new(half.x, half.y - plank, z + 0.12),
            )
        };
        kit.cube(lo, hi, dark, dark);
    }
    // Posts at the corners, down to the box's bottom, and braces.
    let post = 0.16;
    let corners = [
        Vec3::new(-half.x + post, 0.0, -half.z + post),
        Vec3::new(half.x - post, 0.0, -half.z + post),
        Vec3::new(half.x - post, 0.0, half.z - post),
        Vec3::new(-half.x + post, 0.0, half.z - post),
    ];
    let floor = -half.y;
    let under = half.y - plank - beam;
    for c in corners {
        kit.tube(
            Vec3::new(c.x, floor, c.z),
            Vec3::new(c.x, under, c.z),
            post,
            post,
            6,
            dark,
            Some(dark),
        );
    }
    if under - floor > 1.0 {
        for k in 0..4 {
            let (a, b) = (corners[k], corners[(k + 1) % 4]);
            kit.tube(
                Vec3::new(a.x, floor + 0.3, a.z),
                Vec3::new(b.x, under - 0.2, b.z),
                0.07,
                0.07,
                5,
                dark,
                None,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every form's vertices lie in its box -- but an island's roots, which
    /// hang out of its bottom.
    #[test]
    fn every_form_stays_in_its_box() {
        let p = look::palette::of(ArenaId::GNAWERS);
        let forms = [
            Form::Masonry,
            Form::DryStone,
            Form::Cordwood,
            Form::Log,
            Form::Trunk,
            Form::Column,
            Form::Menhir,
            Form::Spire,
            Form::Island,
            Form::Drum,
            Form::Deck,
        ];
        let sizes = [
            Vec3::new(30.0, 1.5, 1.0),
            Vec3::new(1.2, 8.0, 1.2),
            Vec3::new(8.0, 1.5, 1.0),
            Vec3::new(5.0, 3.0, 5.0),
            Vec3::new(36.0, 3.0, 1.5),
        ];
        for form in forms {
            for (size, near) in sizes.into_iter().flat_map(|s| [(s, true), (s, false)]) {
                let (body, relief) =
                    build_parts(form, size, 7, Material::Grass, &p, near).expect("a form");
                let ps: Vec<[f32; 3]> = [Some(body), relief]
                    .into_iter()
                    .flatten()
                    .flat_map(|mesh| {
                        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                            .and_then(|a| a.as_float3())
                            .expect("positions")
                            .to_vec()
                    })
                    .collect();
                let half = size * 0.5 + Vec3::splat(1e-3);
                for q in ps {
                    let v = Vec3::from(q);
                    let roots = form == Form::Island && v.y < 0.0;
                    if roots {
                        continue;
                    }
                    assert!(
                        v.x.abs() <= half.x && v.y.abs() <= half.y && v.z.abs() <= half.z,
                        "{form:?} at {size} (near: {near}): a vertex at {v} is outside its box"
                    );
                }
            }
        }
    }

    #[test]
    fn walls_and_columns_are_what_they_look_like() {
        let s = |min: [i32; 3], max: [i32; 3], m: Material| Solid::cm(min, max, m);
        let g = ArenaId::GNAWERS;
        assert_eq!(
            form_of(g, &s([0, 0, 0], [3000, 150, 100], Material::Stone), false),
            Form::Masonry
        );
        assert_eq!(
            form_of(g, &s([0, 0, 0], [3000, 150, 100], Material::Rock), false),
            Form::DryStone
        );
        assert_eq!(
            form_of(g, &s([0, 0, 0], [120, 800, 120], Material::Stone), false),
            Form::Column
        );
        assert_eq!(
            form_of(g, &s([0, 0, 0], [120, 500, 120], Material::Wood), false),
            Form::Trunk
        );
        assert_eq!(
            form_of(g, &s([0, 0, 0], [800, 150, 100], Material::Wood), false),
            Form::Log
        );
        assert_eq!(
            form_of(g, &s([0, 0, 0], [3600, 300, 150], Material::Wood), false),
            Form::Cordwood
        );
        assert_eq!(
            form_of(
                ArenaId::CLIMB_FALLS,
                &s([0, 6000, 0], [500, 6300, 500], Material::Grass),
                true
            ),
            Form::Island
        );
    }
}
