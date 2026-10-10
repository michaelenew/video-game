//! **The land, drawn** (`sim::valley::land`, `docs/design/atlas.md`): the
//! valley's ground as meshes, one per tile of the map, and its trees.
//!
//! Nothing here decides a height or a colour. Every height is the
//! simulation's own floor (`Atlas::relief_at`), so what is drawn is what feet
//! stand on; every colour is `look::Palette::land` for what the ground is made
//! of there, how steep it is and how high up a mountain. What this decides is
//! how finely to draw it: a tile near the camera a metre a vertex, one far off
//! eight, with a skirt hanging from its edges so two tiles drawn at different
//! fineness never show a crack between them.

use std::collections::HashMap;

use crate::paint::{Materials, Paint};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use sim::Fx;
use sim::arena::{ArenaId, Material};
use sim::atlas::{Atlas, TILE_M};

/// How far a skirt hangs below a tile's edge, in metres.
const SKIRT: f32 = 3.0;

/// How finely a tile at this distance from the camera is drawn: vertices a
/// side, less one. Sixteen is a metre apart.
pub fn cells_at(distance: f32) -> usize {
    if distance < 90.0 {
        16
    } else if distance < 200.0 {
        8
    } else if distance < 380.0 {
        4
    } else {
        2
    }
}

fn fx(v: f32) -> Fx {
    Fx::from_raw((v * 65536.0) as i32)
}

fn f(v: Fx) -> f32 {
    v.to_f32_for_render()
}

/// The palette of the place a map point is in, or of the reach whose stretch
/// of the valley it is level with: the mountains beyond a reach's bounds are
/// that reach's mountains.
fn palette_at(
    atlas: &Atlas,
    x: f32,
    z: f32,
    cache: &mut HashMap<ArenaId, look::Palette>,
) -> look::Palette {
    let id = atlas
        .place_at(fx(x), fx(z))
        .map(|p| p.arena)
        .or_else(|| {
            atlas
                .places
                .iter()
                .filter(|p| !p.pad)
                .find(|p| x >= f(p.lo.0) && x <= f(p.hi.0))
                .map(|p| p.arena)
        })
        .unwrap_or(ArenaId::HEARTH);
    *cache.entry(id).or_insert_with(|| look::palette::of(id))
}

/// What the ground is made of at a map point, from its sample and how steep
/// the drawn surface is there: the simulation's own reading
/// (`Atlas::floor_at`), but with the slope this mesh already measured rather
/// than four more samples of the land.
fn ground_at(atlas: &Atlas, x: f32, z: f32, slope: f32, steepest: f32) -> (Material, f32, f32) {
    let land = atlas.land.as_ref().expect("a map with land");
    if let Some(p) = atlas.pad_at(fx(x), fx(z)) {
        let m = p.get().floor_at(fx(x).sub(p.at.x), fx(z).sub(p.at.z));
        return (m, 0.0, f32::MAX);
    }
    let s = land.sample(fx(x), fx(z));
    let rise = f(s.rise);
    if s.water.is_some() {
        return (Material::Water, rise, f32::MAX);
    }
    if slope > steepest {
        return (Material::Rock, rise, f32::MAX);
    }
    // The trodden path is blended in by distance, not switched: a road has
    // a worn edge, not a line.
    let m = atlas.place_at(fx(x), fx(z)).map_or(Material::Grass, |p| {
        p.get().floor_at(fx(x).sub(p.at.x), fx(z).sub(p.at.z))
    });
    (m, rise, f(s.path))
}

/// Smooth value noise for colour, render side: about 0..1, cells `scale`
/// metres across, keyed to the map so tiles agree.
fn patches(x: f32, z: f32, scale: f32, salt: u32) -> f32 {
    let (u, v) = (x / scale, z / scale);
    let (i, j) = (u.floor(), v.floor());
    let (fu, fv) = (u - i, v - j);
    let (su, sv) = (fu * fu * (3.0 - 2.0 * fu), fv * fv * (3.0 - 2.0 * fv));
    let at = |a: f32, b: f32| crate::shapes::hash01(a as i32 as u32, b as i32 as u32, salt);
    let top = at(i, j) + (at(i + 1.0, j) - at(i, j)) * su;
    let bottom = at(i, j + 1.0) + (at(i + 1.0, j + 1.0) - at(i, j + 1.0)) * su;
    top + (bottom - top) * sv
}

/// **One tile of land**, `cells` to a side, its vertices relative to the
/// tile's corner at (`tx`, `tz`) tiles: the mesh, and the water's surface over
/// it if any of it is under water.
pub fn tile_mesh(
    atlas: &Atlas,
    tx: i32,
    tz: i32,
    cells: usize,
    cache: &mut HashMap<ArenaId, look::Palette>,
) -> (Mesh, Option<f32>) {
    let size = TILE_M as f32;
    let (x0, z0) = (tx as f32 * size, tz as f32 * size);
    let steepest = f(sim::tuning::terrain_steepest());
    let land = atlas.land.as_ref().expect("a map with land");
    let mut water: Option<f32> = None;
    let mesh = grid_mesh(
        x0,
        z0,
        size,
        cells,
        &|x, z| f(atlas.relief_at(fx(x), fx(z))),
        &mut |x, z, slope| {
            let (m, rise, path) = ground_at(atlas, x, z, slope, steepest);
            let palette = palette_at(atlas, x, z, cache);
            if m == Material::Water {
                if let Some(surface) = land.sample(fx(x), fx(z)).water {
                    water = Some(water.map_or(f(surface), |s: f32| s.max(f(surface))));
                }
            }
            ground_colour(&palette, m, slope, steepest, rise, path, x, z)
        },
    );
    (mesh, water)
}

/// **The colour of open ground**: `Palette::land` for what it is made of,
/// how steep and how high, with the trodden path worn in by its distance
/// `path` from the middle of the way, and patches across it -- greener here,
/// drier there -- keyed to where it is so tiles agree at their edges.
#[allow(clippy::too_many_arguments)]
pub fn ground_colour(
    palette: &look::Palette,
    m: Material,
    slope: f32,
    steepest: f32,
    rise: f32,
    path: f32,
    x: f32,
    z: f32,
) -> [f32; 3] {
    let mut rgb = palette.land(m, slope, steepest, rise);
    // The road, worn in: full dirt down its middle, fading out over a metre
    // past its edge, never quite bare.
    let half = f(sim::valley::land::PATH_HALF);
    let worn = (1.0 - ((path - half + 0.8) / 1.4).clamp(0.0, 1.0)) * 0.75;
    if worn > 0.0 {
        let dirt = palette.trodden();
        rgb = look::tint::mix(rgb, dirt, worn * worn * (3.0 - 2.0 * worn));
    }
    let broad = patches(x, z, 23.0, 7) - 0.5;
    let fine = crate::shapes::hash01(x.round() as i32 as u32, z.round() as i32 as u32, 11) - 0.5;
    let k = 1.0 + 0.16 * broad + 0.07 * fine;
    let dry = (patches(x, z, 41.0, 9) - 0.55).max(0.0) * 0.5;
    let rgb = if matches!(m, Material::Grass | Material::Ground | Material::Peat) {
        look::tint::mix(rgb, palette.of(Material::Sand), dry * (1.0 - worn))
    } else {
        rgb
    };
    [rgb[0] * k, rgb[1] * k, rgb[2] * k]
}

/// **A square of ground as a mesh**: `cells` to a side, `size` metres, its
/// corner at (`x0`, `z0`), each vertex at `height` and coloured by `colour`
/// (of where it is and how steep the drawn surface is there, sRGB), with a
/// skirt hanging from its edges. Vertices are relative to the corner.
pub fn grid_mesh(
    x0: f32,
    z0: f32,
    size: f32,
    cells: usize,
    height: &dyn Fn(f32, f32) -> f32,
    colour: &mut dyn FnMut(f32, f32, f32) -> [f32; 3],
) -> Mesh {
    let d = size / cells as f32;
    let n = cells + 1;
    // Heights with a ring one cell wider, for the normals at the edges.
    let w = n + 2;
    let mut h = vec![0.0f32; w * w];
    for j in 0..w {
        for i in 0..w {
            let (x, z) = (x0 + (i as f32 - 1.0) * d, z0 + (j as f32 - 1.0) * d);
            h[j * w + i] = height(x, z);
        }
    }
    let at = |i: usize, j: usize| h[(j + 1) * w + (i + 1)];
    let mut positions = Vec::with_capacity(n * n + 4 * n);
    let mut normals = Vec::with_capacity(n * n + 4 * n);
    let mut colours = Vec::with_capacity(n * n + 4 * n);
    for j in 0..n {
        for i in 0..n {
            let (x, z) = (x0 + i as f32 * d, z0 + j as f32 * d);
            let y = at(i, j);
            // Central differences, a cell either way.
            let l = h[(j + 1) * w + i];
            let r = h[(j + 1) * w + i + 2];
            let b = h[j * w + i + 1];
            let t = h[(j + 2) * w + i + 1];
            let normal = Vec3::new(l - r, 2.0 * d, b - t).normalize_or(Vec3::Y);
            let slope = ((r - l).abs().max((t - b).abs())) / (2.0 * d);
            let rgb = colour(x, z, slope);
            let c = look::tint::linear(rgb);
            positions.push([i as f32 * d, y, j as f32 * d]);
            normals.push(normal.to_array());
            colours.push([c[0], c[1], c[2], 1.0]);
        }
    }
    let row = n as u32;
    let mut indices: Vec<u32> = Vec::with_capacity(cells * cells * 6 + 4 * cells * 6);
    for j in 0..cells as u32 {
        for i in 0..cells as u32 {
            let a = j * row + i;
            let b = a + row;
            // Up-facing, as `shapes::ground_grid` winds its floor.
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    // The skirt: each edge's vertices again, hung below, and a strip between,
    // wound to face out of the tile.
    let edges: [Vec<u32>; 4] = [
        (0..n as u32).collect(),                                   // z = 0
        (0..n as u32).map(|i| (n as u32 - 1) * row + i).collect(), // z = max
        (0..n as u32).map(|j| j * row).collect(),                  // x = 0
        (0..n as u32).map(|j| j * row + n as u32 - 1).collect(),   // x = max
    ];
    for (e, edge) in edges.iter().enumerate() {
        let start = positions.len() as u32;
        for &v in edge {
            let p = positions[v as usize];
            positions.push([p[0], p[1] - SKIRT, p[2]]);
            normals.push(normals[v as usize]);
            colours.push(colours[v as usize]);
        }
        for k in 0..edge.len() as u32 - 1 {
            let (a, b) = (edge[k as usize], edge[k as usize + 1]);
            let (c, dd) = (start + k, start + k + 1);
            // Outward on the -z and +x sides one way round, the others the
            // other; either way it is only ever seen from outside the tile.
            if e == 0 || e == 3 {
                indices.extend_from_slice(&[a, c, b, b, c, dd]);
            } else {
                indices.extend_from_slice(&[a, b, c, b, dd, c]);
            }
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(indices))
}

/// **The trees' shared shapes**: one trunk, one pine's crown, one
/// broadleaf's, each a unit to be scaled, and a material per place and part
/// -- so a wood of a thousand trees is three meshes and a few colours.
#[derive(Default)]
pub struct Trees {
    trunk: Option<Handle<Mesh>>,
    cone: Option<Handle<Mesh>>,
    crown: Option<Handle<Mesh>>,
    materials: HashMap<(ArenaId, u8), Handle<Paint>>,
}

/// Is a box of the map a tree's trunk: timber, thin and tall?
pub fn is_tree(s: &sim::arena::Solid) -> bool {
    let size = s.max.sub(s.min);
    s.material == Material::Wood && f(size.x) < 1.0 && f(size.z) < 1.0 && f(size.y) > 4.0
}

/// Is a reach's wood pines?
pub fn pines(id: ArenaId) -> bool {
    matches!(
        id,
        ArenaId::SHELVES | ArenaId::PINEWOOD | ArenaId::SADDLE | ArenaId::VEILSTALKER
    )
}

impl Trees {
    fn material(
        &mut self,
        materials: &mut Materials,
        id: ArenaId,
        part: u8,
        rgb: [f32; 3],
    ) -> Handle<Paint> {
        self.materials
            .entry((id, part))
            .or_insert_with(|| materials.paint.add(Paint::srgb(rgb)))
            .clone()
    }

    /// **A tree round a trunk box**: the trunk, and a crown -- three cones up
    /// a pine, three balls on a broadleaf -- under `parent`, in map
    /// coordinates. Returns the tree's own entity.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Materials,
        s: &sim::arena::Solid,
        id: ArenaId,
        palette: &look::Palette,
        parent: Entity,
    ) -> Entity {
        let trunk = self
            .trunk
            .get_or_insert_with(|| meshes.add(Cylinder::new(0.5, 1.0).mesh().resolution(8)))
            .clone();
        let cone = self
            .cone
            .get_or_insert_with(|| meshes.add(Cone::new(0.5, 1.0).mesh().resolution(10)))
            .clone();
        let crown = self
            .crown
            .get_or_insert_with(|| meshes.add(Sphere::new(0.5).mesh().ico(2).unwrap()))
            .clone();
        let pine = pines(id);
        let bark = self.material(materials, id, 0, palette.bark());
        let leaves = self.material(
            materials,
            id,
            if pine { 1 } else { 2 },
            palette.foliage(pine),
        );
        let foot = Vec3::new(
            (f(s.min.x) + f(s.max.x)) * 0.5,
            f(s.min.y) + 0.5,
            (f(s.min.z) + f(s.max.z)) * 0.5,
        );
        let tall = f(s.max.y) - foot.y;
        // A little turn and lean of its own, keyed to where it stands.
        let seed = (foot.x * 13.0 + foot.z * 7.0) as i32 as u32;
        let turn = crate::shapes::hash01(seed, 1, 2) * std::f32::consts::TAU;
        let tree = commands
            .spawn((
                Transform::from_translation(foot).with_rotation(Quat::from_rotation_y(turn)),
                Visibility::default(),
                ChildOf(parent),
            ))
            .id();
        // A trunk as thick as the tree is tall would have it, and a broadleaf
        // that branches low: a crown of clumps, the biggest in the middle,
        // the smaller ones lower and further out, so it is a canopy and not
        // a ball on a stick.
        let girth = if pine {
            0.3 + tall * 0.025
        } else {
            0.35 + tall * 0.045
        };
        let bole = if pine { tall * 0.32 } else { tall * 0.42 };
        commands.spawn((
            Mesh3d(trunk),
            MeshMaterial3d(bark),
            Transform::from_xyz(0.0, bole * 0.5, 0.0).with_scale(Vec3::new(girth, bole, girth)),
            ChildOf(tree),
        ));
        if pine {
            // Four tiers, each narrower and overlapping the last.
            let body = tall - bole * 0.6;
            for k in 0..4 {
                let t = k as f32 / 4.0;
                let width = (tall * 0.42 - tall * 0.3 * t)
                    * (0.85 + 0.3 * crate::shapes::hash01(seed, k, 3));
                let height = body * 0.4;
                commands.spawn((
                    Mesh3d(cone.clone()),
                    MeshMaterial3d(leaves.clone()),
                    Transform::from_xyz(0.0, bole * 0.6 + body * (0.2 + 0.26 * t), 0.0)
                        .with_scale(Vec3::new(width, height, width)),
                    ChildOf(tree),
                ));
            }
        } else {
            let spread = tall * 0.62;
            // The heart of the crown.
            commands.spawn((
                Mesh3d(crown.clone()),
                MeshMaterial3d(leaves.clone()),
                Transform::from_translation(Vec3::Y * (bole + spread * 0.42))
                    .with_scale(Vec3::new(spread, spread * 0.78, spread)),
                ChildOf(tree),
            ));
            for k in 0..4u32 {
                let a = k as f32 * 1.57 + crate::shapes::hash01(seed, k, 4);
                let off = Vec3::new(a.cos(), 0.0, a.sin()) * spread * 0.36;
                let r = spread * (0.5 + 0.18 * crate::shapes::hash01(seed, k, 5));
                let up = bole + spread * (0.22 + 0.3 * crate::shapes::hash01(seed, k, 6));
                commands.spawn((
                    Mesh3d(crown.clone()),
                    MeshMaterial3d(leaves.clone()),
                    Transform::from_translation(off + Vec3::Y * up).with_scale(Vec3::new(
                        r,
                        r * 0.8,
                        r,
                    )),
                    ChildOf(tree),
                ));
            }
        }
        tree
    }
}

/// **A fight's ground, alone** (`sim::arena::rim`): its floor and the rim
/// round it drawn as land, a tile at a time like the valley's -- the floor
/// exactly the simulation's, the bank too steep to climb, the hills and
/// mountains past it -- and what grows on the rim: trees past the crest,
/// bushes and boulders on the bank. A rim that is a drop (the Cliffs) is
/// drawn falling away past its crest to a floor far below.
///
/// Only for an arena with a rim, fought on its own; on the valley's map the
/// land is drawn by `crate::stream`.
pub fn draw_arena(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Materials,
    arena: &'static sim::arena::Arena,
    palette: &look::Palette,
    white: &Handle<Paint>,
) {
    let Some(rim) = arena.rim else {
        return;
    };
    let steepest = f(sim::tuning::terrain_steepest());
    let run = f(rim.run());
    let ((lo_x, lo_z), (hi_x, hi_z)) = rim.rect();
    let (lo_x, lo_z, hi_x, hi_z) = (f(lo_x), f(lo_z), f(hi_x), f(hi_z));
    let margin = run + 150.0;
    let size = TILE_M as f32;
    let (tx0, tz0) = (
        ((lo_x - margin) / size).floor() as i32,
        ((lo_z - margin) / size).floor() as i32,
    );
    let (tx1, tz1) = (
        ((hi_x + margin) / size).ceil() as i32,
        ((hi_z + margin) / size).ceil() as i32,
    );
    // How far a point is outside the rectangle, flat.
    let out = |x: f32, z: f32| {
        let dx = (lo_x - x).max(x - hi_x).max(0.0);
        let dz = (lo_z - z).max(z - hi_z).max(0.0);
        (dx * dx + dz * dz).sqrt()
    };
    // The drawn height: the simulation's everywhere it can be stood on; a
    // drop's fall past its crest.
    let height = |x: f32, z: f32| -> f32 {
        let (px, pz) = (fx(x), fx(z));
        match rim.out_of(px, pz) {
            Some((d, foot)) if rim.drop && f(d) > run => {
                let crest = f(foot.add(rim.rise(rim.run(), px, pz)));
                let fall = crest - (f(d) - run) * 2.6;
                let lumps = (patches(x, z, 19.0, 3) - 0.5) * 6.0;
                fall.max(-70.0 + lumps)
            }
            _ => f(arena.relief_at(px, pz)),
        }
    };
    let root = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            super::arenas::Scenery,
        ))
        .id();
    for tz in tz0..tz1 {
        for tx in tx0..tx1 {
            let (x0, z0) = (tx as f32 * size, tz as f32 * size);
            let d = out(x0 + size * 0.5, z0 + size * 0.5);
            let cells = if d < 24.0 {
                16
            } else if d < 70.0 {
                8
            } else if d < 130.0 {
                4
            } else {
                2
            };
            let mut colour = |x: f32, z: f32, slope: f32| {
                let (px, pz) = (fx(x), fx(z));
                match rim.out_of(px, pz) {
                    None => {
                        let m = arena.floor_at(px, pz);
                        ground_colour(palette, m, slope, steepest, 0.0, f32::MAX, x, z)
                    }
                    Some((_, foot)) => {
                        let y = height(x, z);
                        let rise = y - f(foot);
                        let m = match arena.floor {
                            Material::Stone | Material::Ash => Material::Ground,
                            m => m,
                        };
                        let c = ground_colour(palette, m, slope, steepest, rise, f32::MAX, x, z);
                        // A face too steep to stand on shows its strata:
                        // bands of lighter and darker rock, wandering.
                        if slope > steepest {
                            let k = 0.9 + 0.12 * (y * 1.7 + patches(x, z, 9.0, 31) * 4.0).sin();
                            [c[0] * k, c[1] * k, c[2] * k]
                        } else {
                            c
                        }
                    }
                }
            };
            let mesh = grid_mesh(x0, z0, size, cells, &height, &mut colour);
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(white.clone()),
                Transform::from_xyz(x0, 0.0, z0),
                bevy::pbr::NotShadowCaster,
                ChildOf(root),
            ));
        }
    }

    // What grows on it: a jittered grid over the rim, thinner further out.
    let mut trees = Trees::default();
    let rock = palette.of(Material::Rock);
    let bush = palette.foliage(false);
    let crown = meshes.add(Sphere::new(0.5).mesh().ico(1).unwrap());
    let leaves = materials.paint.add(Paint::srgb(bush));
    let pine = pines(arena.id);
    let step = 4.0f32;
    let (gx0, gz0) = (lo_x - margin * 0.8, lo_z - margin * 0.8);
    let (nx, nz) = (
        ((hi_x - lo_x + margin * 1.6) / step) as i32,
        ((hi_z - lo_z + margin * 1.6) / step) as i32,
    );
    let seed = arena.id.0 as u32 * 977 + 13;
    for j in 0..nz {
        for i in 0..nx {
            let r = |salt: u32| crate::shapes::hash01(seed + i as u32, j as u32, salt);
            let x = gx0 + (i as f32 + r(1)) * step;
            let z = gz0 + (j as f32 + r(2)) * step;
            let d = out(x, z);
            if d < 0.8 {
                continue;
            }
            // Fewer the further out, past what the eye resolves.
            let keep = if d < run + 40.0 { 1.0 } else { 0.35 };
            if r(3) > keep {
                continue;
            }
            let y = height(x, z);
            if rim.drop && d > run + 1.0 && y < -40.0 {
                // The floor far below the Cliffs: a wood seen from above.
                if r(4) < 0.25 {
                    let tall = 9.0 + 7.0 * r(5);
                    tree(
                        &mut trees, commands, meshes, materials, arena.id, palette, root, x, y, z,
                        tall,
                    );
                }
                continue;
            }
            if rim.drop && d > run * 0.6 {
                continue;
            }
            // Steepness here, roughly: trees do not stand on cliffs.
            let e = 1.0;
            let slope = ((height(x + e, z) - height(x - e, z))
                .abs()
                .max((height(x, z + e) - height(x, z - e)).abs()))
                / (2.0 * e);
            let pick = r(6);
            if d > run + 2.0 && slope < 1.1 && pick < 0.32 {
                let tall = if pine {
                    9.0 + 9.0 * r(7)
                } else {
                    7.0 + 7.0 * r(7)
                };
                tree(
                    &mut trees, commands, meshes, materials, arena.id, palette, root, x, y, z, tall,
                );
            } else if pick < 0.5 {
                // A bush, sat into the slope.
                let s = 1.0 + 1.6 * r(8);
                commands.spawn((
                    Mesh3d(crown.clone()),
                    MeshMaterial3d(leaves.clone()),
                    Transform::from_xyz(x, y + s * 0.08, z).with_scale(Vec3::new(
                        s * 1.3,
                        s * 0.85,
                        s * 1.1,
                    )),
                    ChildOf(root),
                ));
            } else if pick < 0.62 {
                // A boulder, half buried.
                let s = 0.8 + 2.4 * r(9) * r(9);
                let tone = 0.85 + 0.25 * r(10);
                let mesh = crate::shapes::rock(
                    Vec3::new(s * 1.3, s, s * (0.8 + 0.5 * r(11))),
                    seed ^ (i as u32 * 31 + j as u32),
                    None,
                );
                let mut kit = crate::forms::Kit::new();
                kit.mesh(
                    &mesh,
                    Transform::IDENTITY,
                    [rock[0] * tone, rock[1] * tone, rock[2] * tone],
                );
                commands.spawn((
                    Mesh3d(meshes.add(kit.build())),
                    MeshMaterial3d(white.clone()),
                    Transform::from_xyz(x, y, z)
                        .with_rotation(Quat::from_rotation_y(r(12) * std::f32::consts::TAU)),
                    ChildOf(root),
                ));
            }
        }
    }
}

/// One tree standing at (x, y, z), `tall` metres, under `root`.
#[allow(clippy::too_many_arguments)]
fn tree(
    trees: &mut Trees,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Materials,
    id: ArenaId,
    palette: &look::Palette,
    root: Entity,
    x: f32,
    y: f32,
    z: f32,
    tall: f32,
) {
    let trunk = sim::arena::Solid {
        min: sim::V3::new(fx(x - 0.3), fx(y - 0.5), fx(z - 0.3)),
        max: sim::V3::new(fx(x + 0.3), fx(y + tall), fx(z + 0.3)),
        material: Material::Wood,
    };
    trees.spawn(commands, meshes, materials, &trunk, id, palette, root);
}

/// **The land far below a jump course**: hills and woods sixty metres and
/// more under the islands, hazed by the air between, so a gap reads as a
/// long fall onto somewhere rather than into a dark floor. Nothing stands on
/// it -- a fall is caught by the course's pit long before -- so its heights
/// are drawn, not the simulation's; they come down to the floor near the
/// course, where its first spire stands on it.
pub fn draw_below(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Materials,
    arena: &'static sim::arena::Arena,
    palette: &look::Palette,
    white: &Handle<Paint>,
) {
    let b = arena.bounds;
    let (lo_x, hi_x, lo_z, hi_z) = (f(b.lo_x), f(b.hi_x), f(b.lo_z), f(b.hi_z));
    let (mx, mz) = ((lo_x + hi_x) * 0.5, (lo_z + hi_z) * 0.5);
    let margin = 700.0;
    let size = 48.0f32;
    let steepest = f(sim::tuning::terrain_steepest());
    // Rolling hills and a few big ones, flattening to the floor round the
    // course's own footprint.
    let height = |x: f32, z: f32| -> f32 {
        let dx = (lo_x - x).max(x - hi_x).max(0.0);
        let dz = (lo_z - z).max(z - hi_z).max(0.0);
        let away = ((dx * dx + dz * dz).sqrt() / 60.0).clamp(0.0, 1.0);
        let away = away * away * (3.0 - 2.0 * away);
        let hills = 16.0 * (patches(x, z, 97.0, 21) - 0.5) + 7.0 * (patches(x, z, 33.0, 22) - 0.5);
        let peaks = (patches(x, z, 260.0, 23) - 0.55).max(0.0) * 220.0;
        (hills + peaks) * away + 2.0 * (patches(x, z, 11.0, 24) - 0.5)
    };
    let root = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            super::arenas::Scenery,
        ))
        .id();
    let n = ((hi_x - lo_x + 2.0 * margin) / size).ceil() as i32;
    let m = ((hi_z - lo_z + 2.0 * margin) / size).ceil() as i32;
    for j in 0..m {
        for i in 0..n {
            let (x0, z0) = (
                lo_x - margin + i as f32 * size,
                lo_z - margin + j as f32 * size,
            );
            let d = ((x0 + size * 0.5 - mx).abs()).max((z0 + size * 0.5 - mz).abs());
            let cells = if d < 250.0 {
                12
            } else if d < 500.0 {
                6
            } else {
                3
            };
            let mut colour = |x: f32, z: f32, slope: f32| {
                let h = height(x, z);
                ground_colour(
                    palette,
                    Material::Grass,
                    slope,
                    steepest,
                    h + 20.0,
                    f32::MAX,
                    x,
                    z,
                )
            };
            let mesh = grid_mesh(x0, z0, size, cells, &height, &mut colour);
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(white.clone()),
                Transform::from_xyz(x0, 0.0, z0),
                bevy::pbr::NotShadowCaster,
                bevy::pbr::NotShadowReceiver,
                ChildOf(root),
            ));
        }
    }
    // Woods, seen from above: a tree every so often on the gentler ground.
    let mut trees = Trees::default();
    let step = 14.0f32;
    let (nx, nz) = (
        ((hi_x - lo_x + 2.0 * 400.0) / step) as i32,
        ((hi_z - lo_z + 2.0 * 400.0) / step) as i32,
    );
    for j in 0..nz {
        for i in 0..nx {
            let r = |salt: u32| crate::shapes::hash01(i as u32 + 7, j as u32, salt);
            if r(1) > 0.3 {
                continue;
            }
            let x = lo_x - 400.0 + (i as f32 + r(2)) * step;
            let z = lo_z - 400.0 + (j as f32 + r(3)) * step;
            // Not right under the course, where the spires stand.
            if x > lo_x - 10.0 && x < hi_x + 10.0 && z > lo_z - 10.0 && z < hi_z + 10.0 {
                continue;
            }
            let y = height(x, z);
            if y > 40.0 {
                continue;
            }
            let tall = 10.0 + 8.0 * r(4);
            tree(
                &mut trees, commands, meshes, materials, arena.id, palette, root, x, y, z, tall,
            );
        }
    }
}
