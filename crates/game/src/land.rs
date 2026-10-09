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
    let d = size / cells as f32;
    let (x0, z0) = (tx as f32 * size, tz as f32 * size);
    let n = cells + 1;
    // Heights with a ring one cell wider, for the normals at the edges.
    let w = n + 2;
    let mut h = vec![0.0f32; w * w];
    for j in 0..w {
        for i in 0..w {
            let (x, z) = (x0 + (i as f32 - 1.0) * d, z0 + (j as f32 - 1.0) * d);
            h[j * w + i] = f(atlas.relief_at(fx(x), fx(z)));
        }
    }
    let at = |i: usize, j: usize| h[(j + 1) * w + (i + 1)];
    let steepest = f(sim::tuning::terrain_steepest());
    let mut positions = Vec::with_capacity(n * n + 4 * n);
    let mut normals = Vec::with_capacity(n * n + 4 * n);
    let mut colours = Vec::with_capacity(n * n + 4 * n);
    let mut water: Option<f32> = None;
    let land = atlas.land.as_ref().expect("a map with land");
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
            let (m, rise, path) = ground_at(atlas, x, z, slope, steepest);
            let palette = palette_at(atlas, x, z, cache);
            let mut rgb = palette.land(m, slope, steepest, rise);
            // The road, worn in: full dirt down its middle, fading out over a
            // metre past its edge, never quite bare.
            let half = f(sim::valley::land::PATH_HALF);
            let worn = (1.0 - ((path - half + 0.8) / 1.4).clamp(0.0, 1.0)) * 0.75;
            if worn > 0.0 {
                let dirt = palette.trodden();
                rgb = look::tint::mix(rgb, dirt, worn * worn * (3.0 - 2.0 * worn));
            }
            // Patches across a meadow -- greener here, drier there -- and a
            // little wander per metre, keyed to the map so tiles agree at
            // their edges.
            let broad = patches(x, z, 23.0, 7) - 0.5;
            let fine =
                crate::shapes::hash01(x.round() as i32 as u32, z.round() as i32 as u32, 11) - 0.5;
            let k = 1.0 + 0.16 * broad + 0.07 * fine;
            let dry = (patches(x, z, 41.0, 9) - 0.55).max(0.0) * 0.5;
            let rgb = look::tint::mix(rgb, palette.of(Material::Sand), dry * (1.0 - worn));
            let c = look::tint::linear([rgb[0] * k, rgb[1] * k, rgb[2] * k]);
            positions.push([i as f32 * d, y, j as f32 * d]);
            normals.push(normal.to_array());
            colours.push([c[0], c[1], c[2], 1.0]);
            if m == Material::Water {
                if let Some(surface) = land.sample(fx(x), fx(z)).water {
                    water = Some(water.map_or(f(surface), |s: f32| s.max(f(surface))));
                }
            }
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
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(indices));
    (mesh, water)
}

/// **The trees' shared shapes**: one trunk, one pine's crown, one
/// broadleaf's, each a unit to be scaled, and a material per place and part
/// -- so a wood of a thousand trees is three meshes and a few colours.
#[derive(Default)]
pub struct Trees {
    trunk: Option<Handle<Mesh>>,
    cone: Option<Handle<Mesh>>,
    crown: Option<Handle<Mesh>>,
    materials: HashMap<(ArenaId, u8), Handle<StandardMaterial>>,
}

/// Is a box of the map a tree's trunk: timber, thin and tall?
pub fn is_tree(s: &sim::arena::Solid) -> bool {
    let size = s.max.sub(s.min);
    s.material == Material::Wood && f(size.x) < 1.0 && f(size.z) < 1.0 && f(size.y) > 4.0
}

/// Is a reach's wood pines?
pub fn pines(id: ArenaId) -> bool {
    matches!(id, ArenaId::SHELVES | ArenaId::PINEWOOD | ArenaId::SADDLE)
}

impl Trees {
    fn material(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        id: ArenaId,
        part: u8,
        rgb: [f32; 3],
    ) -> Handle<StandardMaterial> {
        self.materials
            .entry((id, part))
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
                    perceptual_roughness: 0.95,
                    ..default()
                })
            })
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
        materials: &mut Assets<StandardMaterial>,
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
        let girth = if pine { 0.42 } else { 0.5 };
        let bole = if pine { tall * 0.35 } else { tall * 0.5 };
        commands.spawn((
            Mesh3d(trunk),
            MeshMaterial3d(bark),
            Transform::from_xyz(0.0, bole * 0.5, 0.0).with_scale(Vec3::new(girth, bole, girth)),
            ChildOf(tree),
        ));
        if pine {
            // Three tiers, each narrower and overlapping the last.
            let body = tall - bole * 0.6;
            for k in 0..3 {
                let t = k as f32 / 3.0;
                let width = (3.6 - 2.0 * t) * (0.85 + 0.3 * crate::shapes::hash01(seed, k, 3));
                let height = body * 0.48;
                commands.spawn((
                    Mesh3d(cone.clone()),
                    MeshMaterial3d(leaves.clone()),
                    Transform::from_xyz(0.0, bole * 0.6 + body * (0.24 + 0.3 * t), 0.0)
                        .with_scale(Vec3::new(width, height, width)),
                    ChildOf(tree),
                ));
            }
        } else {
            let spread = tall * 0.55;
            for k in 0..3u32 {
                let a = k as f32 * 2.1 + crate::shapes::hash01(seed, k, 4);
                let off = Vec3::new(a.cos(), 0.0, a.sin()) * spread * 0.22;
                let r = spread * (0.75 + 0.25 * crate::shapes::hash01(seed, k, 5));
                commands.spawn((
                    Mesh3d(crown.clone()),
                    MeshMaterial3d(leaves.clone()),
                    Transform::from_translation(off + Vec3::Y * (bole + r * 0.35 + k as f32 * 0.4))
                        .with_scale(Vec3::new(r, r * 0.85, r)),
                    ChildOf(tree),
                ));
            }
        }
        tree
    }
}
