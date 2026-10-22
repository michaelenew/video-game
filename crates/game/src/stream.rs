//! **Distance loading** (`docs/design/atlas.md`): in the valley, what is
//! drawn is what is near the camera, and nothing else.
//!
//! The valley is one map (`sim::atlas`), and drawing all of it at once is
//! what a world of a million boxes cannot do. So the map is drawn in two
//! kinds of piece, each loaded when the camera comes within reach of it and
//! dropped when it goes further than that again:
//!
//! - **A place**: its floor, the patches on it, its props and what is
//!   scattered on the floor, its seams -- everything
//!   `arenas::draw_place` draws -- under one entity, gone together.
//! - **A box**: each of the map's boxes on its own, found through the map's
//!   tiles, so loading the boxes round the camera reads the tiles round the
//!   camera and not the map. A box is drawn once however many tiles it lies
//!   across.
//!
//! The reach is the current sky's own (`look::Sky::reach`): past it the fog is
//! the horizon's colour and a thing loading there cannot be seen arriving.
//! Dropping waits a further [`SLACK`], so the edge of the loaded world never
//! flickers as the camera rocks across it.
//!
//! **Everything hangs off one root**, at the map's origin as seen from the
//! place the world is in: everything the simulation says is in that place's
//! coordinates, and every piece here is in the map's, so the root's offset is
//! the difference. When the world moves into another place
//! (`sim::valley::open`), only the root moves.

use std::collections::{HashMap, HashSet};

use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, MeshAabb, MeshVertexAttribute, VertexAttributeValues};
use bevy::render::primitives::Aabb;
use bevy::render::render_resource::VertexFormat;
use sim::arena::ArenaId;
use sim::atlas::{self, Atlas, EXTRA};

use crate::arenas::{PlaceLook, Under, draw_place, draw_solid, solid_mesh};
use crate::paint::{Materials, Paint};

/// How much further than the reach a piece is kept before it is dropped.
const SLACK: f32 = 60.0;

/// The nearest and furthest the reach may be: a sky that sees two
/// kilometres does not load the whole valley, and one in thick fog still
/// loads what is round the corner.
const REACH: (f32, f32) = (160.0, 600.0);

/// How far the camera moves before the loaded set is worked out again.
const RESTREAM: f32 = 6.0;

/// **How much is built in one frame**: places, and boxes. Building a place's
/// floor and its scatter, or a cliff's roughened mesh, is real work on the
/// frame that does it; a hundred of them at once is a frame dropped. So what
/// is wanted is built nearest first, this much a frame, and the rest waits
/// for the next -- arriving at the far edge of the fog, where nobody sees the
/// order it came in.
const PLACES_PER_FRAME: usize = 1;
const BOXES_PER_FRAME: usize = 48;

/// **And how long**: boxes and tiles stop being built once this much of the
/// frame has gone on them, whatever the counts above allow -- at least one
/// of each a frame, so the world always arrives. A tile of land in full
/// detail is half a millisecond and a wall in full relief more; twenty-four
/// tiles at once, on the frame the camera comes within reach of them, was a
/// dropped frame every few steps.
const BUILD_BUDGET_MS: f32 = 3.0;

/// **How close a box is drawn in its full relief** (`forms::build`'s
/// `near`), flat, to the nearest edge of its footprint; past it, plus
/// [`NEAR_SLACK`] before a near box goes back, the same colours with the
/// relief flattened. Five centimetres of a block proud of its wall is half a
/// pixel here at the window's width, and a stone wall in full relief is ten
/// triangles a block, drawn into four shadow cascades.
const NEAR: f32 = 60.0;
const NEAR_SLACK: f32 = 12.0;

/// What is loaded, and where it hangs.
#[derive(Resource, Default)]
pub struct Stream {
    root: Option<Entity>,
    /// The map origin the root was last put at.
    origin: Option<Vec3>,
    /// Places by their index on the map.
    places: HashMap<usize, Entity>,
    /// Boxes by their index on the map, and -- for a box drawn finer close
    /// up than far off -- whether it is drawn near.
    solids: HashMap<u32, (Entity, Option<bool>)>,
    /// Each place's look, worked out once.
    looks: HashMap<ArenaId, PlaceLook>,
    white: Option<Handle<Paint>>,
    /// Where on the map the camera was when the set was last worked out.
    at: Option<Vec3>,
    /// The waystones' doors: one slab per gated doorway, shown while shut.
    gates: Vec<(usize, Entity)>,
    /// **The boxes that move** (`sim::drift`), by their index on the map:
    /// each drawn once as the box it is at its first stop, and every frame
    /// moved to where it is between the last two simulated frames, as
    /// everything that moves is.
    drifts: Vec<(Entity, Vec3)>,
    /// The land's tiles by (column, row): the tile -- for one of a far block
    /// drawn whole, an entity that only carries its strokes, or
    /// `Entity::PLACEHOLDER` where there are none -- how finely it is drawn,
    /// and its water.
    tiles: HashMap<(i32, i32), (Entity, usize, Option<Entity>)>,
    /// Which block of boxes each of the map's boxes is drawn in when its
    /// block is far (`merge_boxes`), by box -- `None` for one never merged:
    /// a tree, a giant's, or one wider than a block -- and the boxes of each
    /// block. Worked out once.
    box_block: Vec<Option<(i32, i32)>>,
    box_blocks: HashMap<(i32, i32), Vec<u32>>,
    /// The far blocks of boxes drawn merged: an entity a kind of mesh.
    merged: HashMap<(i32, i32), Vec<Entity>>,
    /// The blocks of far land drawn whole (`BLOCK` tiles a side), by block
    /// (column, row): the mesh, and how finely along its side.
    blocks: HashMap<(i32, i32), (Entity, usize)>,
    /// The palette for each place, for the land's colours.
    palettes: HashMap<ArenaId, look::Palette>,
    trees: crate::land::Trees,
    /// The grown giants (`Atlas::trees`) drawn, by index, and at which
    /// level (`grove::LEVELS`).
    groves: HashMap<usize, (Entity, usize)>,
    /// Each giant's meshes by (giant, level): built when it comes within
    /// reach of that level and let go with it, but for the far levels, which
    /// are small and kept while the giant is in reach. A giant up close is a
    /// hundred thousand triangles; kept for the session once seen, the
    /// Wood's fourteen were a couple of hundred megabytes nobody was looking
    /// at (`docs/design/exploration/0024_distant_detail.md`).
    grown: HashMap<(usize, usize), Grown>,
    /// How far each giant reaches from its foot, flat, and the box it fills
    /// about its foot, worked out the first time it is asked: what it is
    /// loaded and levelled by, so they are kept when its meshes go.
    spreads: HashMap<usize, (f32, Vec3, Vec3)>,
    ground: Option<Handle<Paint>>,
    water: Option<Handle<StandardMaterial>>,
    /// One tile's water surface, shared by every tile that has water.
    pond: Option<Handle<Mesh>>,
}

/// One level of a giant: its wood and its leaves, and the bounds of each,
/// which a mesh held only on the graphics card cannot be asked for later.
#[derive(Clone)]
struct Grown {
    wood: (Handle<Mesh>, Aabb),
    leaves: (Handle<Mesh>, Aabb),
}

/// How much nearer than a level's distance a giant drawn coarser comes
/// before it is drawn finer again, so a step back and forth across the line
/// does not rebuild it.
const LEVEL_SLACK: f32 = 12.0;

/// What is streamed in, for `--profile`'s census.
pub struct Census {
    pub places: usize,
    pub boxes: usize,
    /// Tiles of land drawn on their own, and blocks of them drawn whole.
    pub tiles: usize,
    pub blocks: usize,
    /// Blocks of boxes drawn merged, and how many entities draw them.
    pub merged: usize,
    pub merged_meshes: usize,
    pub giants: usize,
    /// Every mesh of every giant built.
    pub built: Vec<AssetId<Mesh>>,
}

/// **A mesh made to be drawn and nothing else**: let go of in main memory
/// once the graphics card has it (`RenderAssetUsages::RENDER_WORLD`), rather
/// than held twice. Gone from `Assets<Mesh>` at the end of the frame it is
/// made, so it is for a mesh spawned that same frame and never handed out
/// again later: Bevy works out what is on screen from bounds it reads off
/// the mesh the frame an entity gets it, and so does the warm-up
/// (`crate::warm`); a handle given to a new entity a frame later has nothing
/// left to read, so such an entity is given its `Aabb` by whoever kept the
/// handle (the giants' levels do). `--profile`'s census weighs it as it is
/// made.
pub fn drawn_only(mut mesh: Mesh) -> Mesh {
    mesh.asset_usage = bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD;
    mesh
}

impl Stream {
    /// The land's tiles by (column, row): the tile, how finely it is drawn,
    /// and its water. `splats::follow` places the brush's strokes on them.
    pub fn tiles(&self) -> &HashMap<(i32, i32), (Entity, usize, Option<Entity>)> {
        &self.tiles
    }

    /// How many places, boxes and tiles of land are drawn -- a box drawn in
    /// a merged block, and a tile drawn in a block, counted as drawn: what
    /// the dev overlay shows.
    pub fn loaded(&self) -> (usize, usize, usize) {
        let merged: usize = self
            .merged
            .keys()
            .filter_map(|b| self.box_blocks.get(b))
            .map(Vec::len)
            .sum();
        (
            self.places.len(),
            self.solids.len() + merged,
            self.tiles.len(),
        )
    }

    /// What is streamed in, counted.
    pub fn census(&self) -> Census {
        Census {
            places: self.places.len(),
            boxes: self.solids.len(),
            tiles: self
                .tiles
                .keys()
                .filter(|t| {
                    !self
                        .blocks
                        .contains_key(&(t.0.div_euclid(BLOCK), t.1.div_euclid(BLOCK)))
                })
                .count(),
            blocks: self.blocks.len(),
            merged: self.merged.len(),
            merged_meshes: self.merged.values().map(Vec::len).sum(),
            giants: self.groves.len(),
            built: self
                .grown
                .values()
                .flat_map(|g| [g.wood.0.id(), g.leaves.0.id()])
                .collect(),
        }
    }
}

/// RENDER-ONLY. Fixed point to metres.
fn fx(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}

impl Stream {
    /// The entity everything streamed hangs off, while the valley is drawn.
    pub fn root(&self) -> Option<Entity> {
        self.root
    }

    /// The map origin the root stands at the negative of: add it to a
    /// position in the picture to have it in the map's own coordinates.
    pub fn origin(&self) -> Option<Vec3> {
        self.origin
    }
}

fn v3(v: sim::V3) -> Vec3 {
    Vec3::new(fx(v.x), fx(v.y), fx(v.z))
}

/// How far a map point (x, z) is from a place's footprint, flat: zero inside.
fn distance_to(p: &atlas::Placed, at: Vec3) -> f32 {
    let dx = (fx(p.lo.0) - at.x).max(at.x - fx(p.hi.0)).max(0.0);
    let dz = (fx(p.lo.1) - at.z).max(at.z - fx(p.hi.1)).max(0.0);
    (dx * dx + dz * dz).sqrt()
}

/// The boxes in every tile within `reach` of a map point, flat.
fn boxes_near(atlas: &Atlas, at: Vec3, reach: f32) -> HashSet<u32> {
    let to_fx = |v: f32| sim::Fx::from_raw((v * 65536.0) as i32);
    let (x0, z0) = atlas.tile_at(to_fx(at.x - reach), to_fx(at.z - reach));
    let (x1, z1) = atlas.tile_at(to_fx(at.x + reach), to_fx(at.z + reach));
    let mut out = HashSet::new();
    for tz in z0..=z1 {
        for tx in x0..=x1 {
            out.extend(atlas.tile(tx, tz).iter().copied());
        }
    }
    out
}

/// **Load and drop** what is near the camera, in the valley; clear all of it
/// outside the valley.
#[allow(clippy::too_many_arguments)] // A Bevy system: one argument per resource it reads.
pub fn stream(
    mut commands: Commands,
    sim: Res<crate::Sim>,
    mut windows: ResMut<crate::window::Windows>,
    mut st: ResMut<Stream>,
    camera: Query<&Transform, With<crate::MainCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard: ResMut<Assets<StandardMaterial>>,
    mut paints: ResMut<Assets<Paint>>,
    mut gates: Query<&mut Visibility>,
    mut moving: Query<&mut Transform, Without<crate::MainCamera>>,
) {
    let mut materials = Materials {
        standard: &mut standard,
        paint: &mut paints,
    };
    let w = &sim.cur;
    if !w.valley.on {
        if let Some(root) = st.root.take() {
            commands.entity(root).despawn();
        }
        st.places.clear();
        st.solids.clear();
        st.merged.clear();
        st.gates.clear();
        st.drifts.clear();
        st.tiles.clear();
        st.blocks.clear();
        st.groves.clear();
        st.grown.clear();
        st.origin = None;
        st.at = None;
        return;
    }
    let atlas = atlas::valley();
    let origin = v3(w.map_origin());
    let root = match st.root {
        Some(root) => root,
        None => {
            let root = commands
                .spawn((
                    Transform::default(),
                    Visibility::default(),
                    Name::new("the map"),
                ))
                .id();
            st.root = Some(root);
            root
        }
    };
    if st.origin != Some(origin) {
        commands
            .entity(root)
            .insert(Transform::from_translation(-origin));
        st.origin = Some(origin);
    }
    let white = st
        .white
        .get_or_insert_with(|| materials.paint.add(Paint::white()))
        .clone();
    if st.gates.is_empty() {
        spawn_gates(
            &mut commands,
            &mut meshes,
            &mut materials,
            atlas,
            root,
            &mut st,
        );
    }
    // The doors that are shut this frame are the ones the simulation raised.
    let ground = w.terrain();
    let o = w.map_origin();
    for &(k, e) in &st.gates {
        let door = &atlas.doors[k];
        let shut = ground
            .raised()
            .iter()
            .any(|s| s.min == door.cut.min.sub(o) && s.max == door.cut.max.sub(o));
        if let Ok(mut v) = gates.get_mut(e) {
            v.set_if_neq(if shut {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }

    // What the camera passes through is drawn see-through round the line
    // to the fighter (`crate::window`): the small boxes and the giants.
    let see = windows.white(materials.paint);

    // The drifts: drawn the first time, and put where they are every frame.
    if st.drifts.is_empty() && !atlas.drifts.is_empty() {
        for d in &atlas.drifts {
            let mid = (v3(d.solid.min) + v3(d.solid.max)) * 0.5;
            let (x, z) = (
                sim::Fx::from_raw(d.solid.min.x.raw() / 2 + d.solid.max.x.raw() / 2),
                sim::Fx::from_raw(d.solid.min.z.raw() / 2 + d.solid.max.z.raw() / 2),
            );
            let Some(place) = atlas.place_at(x, z).or(atlas.places.first()) else {
                continue;
            };
            let look = st
                .looks
                .entry(place.arena)
                .or_insert_with(|| PlaceLook::of(place.get()))
                .clone();
            let e = draw_solid(
                &mut commands,
                &mut meshes,
                &d.solid,
                Vec3::ZERO,
                true,
                &look,
                (&white, &see),
                Under::Parent(root),
                true,
            );
            st.drifts.push((e, mid));
        }
    }
    let alpha = sim.clock.alpha();
    for (d, &(e, mid)) in atlas.drifts.iter().zip(&st.drifts) {
        let (a, b) = (v3(d.offset(sim.prev.frame)), v3(d.offset(w.frame)));
        if let Ok(mut t) = moving.get_mut(e) {
            t.translation = mid + a.lerp(b, alpha);
        }
    }
    let Ok(eye) = camera.single() else {
        return;
    };
    let at = eye.translation + origin;
    if st.at.is_some_and(|last| last.distance(at) < RESTREAM) {
        return;
    }
    st.at = Some(at);
    let reach = w.arena().id;
    let reach = look::skies::of(reach)
        .resolved()
        .reach
        .clamp(REACH.0, REACH.1);

    let started = Instant::now();
    // Places: their floors and everything on them. Dropped at once; built
    // nearest first, a few a frame.
    let mut wanted: Vec<(f32, usize)> = Vec::new();
    for (k, p) in atlas.places.iter().enumerate() {
        let d = distance_to(p, at);
        let loaded = st.places.contains_key(&k);
        if !loaded && d <= reach {
            wanted.push((d, k));
        } else if loaded && d > reach + SLACK {
            if let Some(e) = st.places.remove(&k) {
                commands.entity(e).despawn();
            }
        }
    }
    wanted.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut behind = wanted.len() > PLACES_PER_FRAME;
    for &(_, k) in wanted.iter().take(PLACES_PER_FRAME) {
        let p = &atlas.places[k];
        let arena = p.get();
        let look = st
            .looks
            .entry(p.arena)
            .or_insert_with(|| PlaceLook::of(arena))
            .clone();
        let place = commands
            .spawn((
                Transform::from_translation(v3(p.at)),
                Visibility::default(),
                ChildOf(root),
            ))
            .id();
        draw_place(
            &mut commands,
            &mut meshes,
            &mut materials,
            arena,
            &look,
            &white,
            Under::Parent(place),
        );
        st.places.insert(k, place);
    }

    // Boxes: through the tiles -- one by one near, a block at a time far.
    if st.box_block.is_empty() {
        index_boxes(&mut st, atlas);
    }
    let far = far_blocks(&st, at, reach);
    let want = boxes_near(atlas, at, reach);
    let keep = boxes_near(atlas, at, reach + SLACK);
    let gone: Vec<u32> = st
        .solids
        .keys()
        .copied()
        .filter(|i| !keep.contains(i))
        .collect();
    for i in gone {
        if let Some((e, _)) = st.solids.remove(&i) {
            commands.entity(e).despawn();
        }
    }
    // A box's flat distance from the camera, to the nearest edge of it.
    let edge = |i: u32| {
        let s = &atlas.solids[i as usize];
        let (lo, hi) = (v3(s.min), v3(s.max));
        let dx = (lo.x - at.x).max(at.x - hi.x).max(0.0);
        let dz = (lo.z - at.z).max(at.z - hi.z).max(0.0);
        (dx * dx + dz * dz).sqrt()
    };
    // What is wanted that is not loaded, and what is loaded at the wrong
    // fineness for where the camera now is -- but for a box of a far
    // block, which its block draws.
    let mut new: Vec<u32> = want
        .into_iter()
        // A grown tree's boxes are drawn as the tree.
        .filter(|i| !atlas.hidden[*i as usize])
        .filter(|i| !st.box_block[*i as usize].is_some_and(|b| far.contains(&b)))
        .filter(|i| match st.solids.get(i) {
            None => true,
            Some((_, None)) => false,
            Some((_, Some(true))) => edge(*i) > NEAR + NEAR_SLACK,
            Some((_, Some(false))) => edge(*i) <= NEAR,
        })
        .collect();
    // Nearest first, by the middle of the box; ties in map order.
    let middle = |i: u32| {
        let s = &atlas.solids[i as usize];
        let m = (v3(s.min) + v3(s.max)) * 0.5;
        (m.x - at.x).powi(2) + (m.z - at.z).powi(2)
    };
    new.sort_by(|a, b| middle(*a).total_cmp(&middle(*b)).then(a.cmp(b)));
    behind |= new.len() > BOXES_PER_FRAME;
    new.truncate(BOXES_PER_FRAME);
    let spent = || started.elapsed().as_secs_f32() * 1000.0;
    let mut kit = BoxKit {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        windows: &mut windows,
        atlas,
        root,
        white: &white,
        see: &see,
    };
    for (n, i) in new.into_iter().enumerate() {
        if n > 0 && spent() > BUILD_BUDGET_MS {
            behind = true;
            break;
        }
        if let Some(drawn) = kit.draw(&mut st, i, edge(i) <= NEAR) {
            if let Some((old, _)) = st.solids.insert(i, drawn) {
                kit.commands.entity(old).despawn();
            }
        }
    }
    behind |= merge_boxes(&mut kit, &mut st, &far, at, reach, &started);
    // The grown giants: each whole, by how near its crown comes.
    behind |= groves(
        &mut commands,
        &mut meshes,
        &mut st,
        atlas,
        root,
        at,
        reach,
        &see,
    );
    // The land: every tile within reach, drawn as finely as its distance
    // asks, nearest first.
    behind |= land_tiles(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut st,
        atlas,
        root,
        at,
        reach,
        &started,
    );

    // More to build: work the set out again next frame rather than waiting
    // for the camera to move.
    if behind {
        st.at = None;
    }
}

/// **Load, level and drop the grown giants** (`crate::grove`): a tree is
/// drawn while its crown is within reach, at the level its nearest part's
/// distance asks for (`grove::LEVELS`), its meshes built as it comes. One
/// built a frame; true if more wait.
#[allow(clippy::too_many_arguments)]
fn groves(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    st: &mut Stream,
    atlas: &Atlas,
    root: Entity,
    at: Vec3,
    reach: f32,
    see: &(Handle<Paint>, Handle<Paint>),
) -> bool {
    let (white, ghost) = see;
    let mut built = false;
    let mut behind = false;
    // How many level lines a distance is past.
    let level = |d: f32| crate::grove::LEVELS.iter().filter(|&&b| d > b).count();
    for (k, tree) in atlas.trees.iter().enumerate() {
        let foot = Vec3::new(
            tree.base[0] as f32 / 100.0,
            tree.base[1] as f32 / 100.0,
            tree.base[2] as f32 / 100.0,
        );
        let flat = Vec2::new(foot.x - at.x, foot.z - at.z).length();
        let (spread, lo, hi) = *st.spreads.entry(k).or_insert_with(|| {
            let (lo, hi) = crate::grove::bounds(tree);
            (crate::grove::reach(tree), lo, hi)
        });
        let d = (flat - spread).max(0.0);
        let drawn = st.groves.get(&k).copied();
        if drawn.is_some() && d > reach + SLACK {
            if let Some((e, _)) = st.groves.remove(&k) {
                commands.entity(e).despawn();
            }
            st.grown.retain(|(t, _), _| *t != k);
            continue;
        }
        if drawn.is_none() && d > reach {
            continue;
        }
        // The level: by the nearest point of the tree's box, which up a
        // giant is the giant. Finer as soon as a line is crossed inward,
        // coarser only a little past it.
        let near = ((foot + lo) - at)
            .max(at - (foot + hi))
            .max(Vec3::ZERO)
            .length();
        let want = match drawn {
            None => level(near),
            Some((_, now)) => {
                let finer = level(near);
                let coarser = level(near - LEVEL_SLACK);
                if finer < now {
                    finer
                } else if coarser > now {
                    coarser
                } else {
                    now
                }
            }
        };
        if drawn.is_some_and(|(_, now)| now == want) {
            continue;
        }
        if !st.grown.contains_key(&(k, want)) {
            if built {
                behind = true;
                continue;
            }
            built = true;
            let place = atlas
                .place_at(
                    sim::Fx::from_raw((foot.x * 65536.0) as i32),
                    sim::Fx::from_raw((foot.z * 65536.0) as i32),
                )
                .or(atlas.places.first());
            let Some(place) = place else {
                continue;
            };
            let look = st
                .looks
                .entry(place.arena)
                .or_insert_with(|| PlaceLook::of(place.get()))
                .clone();
            let detail = match want {
                0 => 0.0,
                n => {
                    crate::grove::detail_at(crate::grove::LEVELS[n - 1], crate::wash::style().brush)
                }
            };
            let (wood, leaves) = crate::grove::meshes(tree, &look.palette, detail);
            let mut keep = |m: Mesh| {
                let bounds = m.compute_aabb().unwrap_or_default();
                (meshes.add(drawn_only(m)), bounds)
            };
            st.grown.insert(
                (k, want),
                Grown {
                    wood: keep(wood),
                    leaves: keep(leaves),
                },
            );
        }
        if let Some((old, _)) = drawn {
            commands.entity(old).despawn();
        }
        // Up close is the heavy one: let it go as soon as it is not drawn.
        if want != 0 {
            st.grown.remove(&(k, 0));
        }
        let Grown {
            wood: (wood, wood_box),
            leaves: (leaves, leaves_box),
        } = st.grown[&(k, want)].clone();
        let e = commands
            .spawn((
                Mesh3d(wood.clone()),
                MeshMaterial3d(white.clone()),
                wood_box,
                Transform::from_translation(foot),
                Visibility::default(),
                ChildOf(root),
                Name::new("a grown giant"),
            ))
            .id();
        // Leaves let the light through: a crown a hundred metres across
        // casting one shadow left the grove under it lit by the ambient
        // alone, which is black (2026-10-15).
        commands.spawn((
            Mesh3d(leaves.clone()),
            MeshMaterial3d(white.clone()),
            leaves_box,
            Transform::default(),
            bevy::pbr::NotShadowCaster,
            ChildOf(e),
        ));
        // The same wood and leaves again, see-through, over the window. The
        // tree already casts its shadow; a ghost casting one too would
        // darken it twice.
        // Shown while the line from the eye to the fighter comes within
        // its reach, so a tree is drawn twice only where it could be in the
        // way.
        let g = commands
            .spawn((
                Mesh3d(wood),
                MeshMaterial3d(ghost.clone()),
                wood_box,
                Transform::default(),
                Visibility::Hidden,
                bevy::pbr::NotShadowCaster,
                crate::window::Ghost::over(foot - spread, foot + spread),
                ChildOf(e),
            ))
            .id();
        commands.spawn((
            Mesh3d(leaves),
            MeshMaterial3d(ghost.clone()),
            leaves_box,
            Transform::default(),
            bevy::pbr::NotShadowCaster,
            ChildOf(g),
        ));
        st.groves.insert(k, (e, want));
    }
    behind
}

/// One slab per waystone's doorway, dark and a little translucent, hidden
/// until the simulation shuts it.
fn spawn_gates(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Materials,
    atlas: &Atlas,
    root: Entity,
    st: &mut Stream,
) {
    for (k, door) in atlas.doors.iter().enumerate() {
        if matches!(door.gate, sim::valley::Gate::Open) {
            continue;
        }
        let palette = look::palette::of(door.near);
        let c = look::tint::linear(palette.beacon(false));
        let (lo, hi) = (v3(door.cut.min), v3(door.cut.max));
        let e = commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::from_size(hi - lo))),
                MeshMaterial3d(materials.standard.add(StandardMaterial {
                    base_color: Color::linear_rgba(c[0], c[1], c[2], 0.55),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation((lo + hi) * 0.5),
                Visibility::Hidden,
                bevy::pbr::NotShadowCaster,
                ChildOf(root),
            ))
            .id();
        st.gates.push((k, e));
    }
}

/// How many tiles of land are built, or rebuilt finer or coarser, in one
/// frame.
const TILES_PER_FRAME: usize = 24;

/// How far the land is drawn: past the sky's reach, at least far enough that
/// the mountains round a valley are never missing, and never the whole map.
const LAND_REACH: (f32, f32) = (300.0, 700.0);

/// **How many tiles a side the land far off is drawn as one**: a block of
/// them, one mesh at the same fineness the tiles would each have had. Far
/// off, a tile is nine vertices, and what it cost was being one of six
/// thousand things to cull and draw, not its triangles (`docs/design/
/// exploration/0024_distant_detail.md`, step 3).
const BLOCK: i32 = 4;

/// How far a block's nearest point is before it is drawn as one mesh -- past
/// the last distance a tile is drawn finer than a vertex every four metres
/// (`land::cells_at`), so a block is never coarser than its tiles were.
const BLOCK_FROM: f32 = 200.0;

/// How much nearer than [`BLOCK_FROM`] a block drawn whole comes before it is
/// drawn as tiles again, so a step back and forth does not rebuild it.
const BLOCK_SLACK: f32 = 12.0;

/// What the land wants built, nearest first.
enum Job {
    /// A tile on its own, so finely.
    Tile((i32, i32), usize),
    /// A block whole, so finely along its side, in place of its tiles.
    Block((i32, i32), usize),
    /// A block drawn whole that is near enough now to be its tiles: all of
    /// them in the one frame, and the block gone, so there is no hole.
    Split((i32, i32)),
}

/// **The land's tiles**: load the ones within reach at the fineness their
/// distance asks -- near ones on their own, far ones a block at a time --
/// rebuild the ones whose fineness has changed, drop the ones past reach.
/// True if there is more to do than one frame's worth.
#[allow(clippy::too_many_arguments)]
fn land_tiles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Materials,
    st: &mut Stream,
    atlas: &Atlas,
    root: Entity,
    at: Vec3,
    reach: f32,
    started: &Instant,
) -> bool {
    if atlas.land.is_none() {
        return false;
    }
    let reach = (reach * 1.2).clamp(LAND_REACH.0, LAND_REACH.1);
    let size = sim::atlas::TILE_M as f32;
    let centre =
        |t: (i32, i32)| Vec3::new((t.0 as f32 + 0.5) * size, 0.0, (t.1 as f32 + 0.5) * size);
    let flat = |t: (i32, i32)| {
        let c = centre(t);
        ((c.x - at.x).powi(2) + (c.z - at.z).powi(2)).sqrt()
    };
    let side = size * BLOCK as f32;
    let block_of = |t: (i32, i32)| (t.0.div_euclid(BLOCK), t.1.div_euclid(BLOCK));
    let tiles_of = |b: (i32, i32)| {
        (0..BLOCK * BLOCK).map(move |k| (b.0 * BLOCK + k % BLOCK, b.1 * BLOCK + k / BLOCK))
    };
    // A block's distance, flat, to the nearest of it.
    let near = |b: (i32, i32)| {
        let (x0, z0) = (b.0 as f32 * side, b.1 as f32 * side);
        let dx = (x0 - at.x).max(at.x - x0 - side).max(0.0);
        let dz = (z0 - at.z).max(at.z - z0 - side).max(0.0);
        (dx * dx + dz * dz).sqrt()
    };
    // Drop what is past reach: a tile on its own by its middle, a block by
    // its nearest, with its tiles.
    let gone: Vec<(i32, i32)> = st
        .tiles
        .keys()
        .copied()
        .filter(|t| !st.blocks.contains_key(&block_of(*t)) && flat(*t) > reach + SLACK)
        .collect();
    for t in gone {
        drop_tile(commands, st, t);
    }
    let gone: Vec<(i32, i32)> = st
        .blocks
        .keys()
        .copied()
        .filter(|b| near(*b) > reach + SLACK)
        .collect();
    for b in gone {
        if let Some((e, _)) = st.blocks.remove(&b) {
            commands.entity(e).despawn();
        }
        for t in tiles_of(b) {
            drop_tile(commands, st, t);
        }
    }
    // What is wanted, and how finely; nearest first.
    let to_fx = |v: f32| sim::Fx::from_raw((v * 65536.0) as i32);
    let (x0, z0) = block_of(atlas.tile_at(to_fx(at.x - reach), to_fx(at.z - reach)));
    let (x1, z1) = block_of(atlas.tile_at(to_fx(at.x + reach), to_fx(at.z + reach)));
    let mut work: Vec<(f32, Job)> = Vec::new();
    for bz in z0..=z1 {
        for bx in x0..=x1 {
            let b = (bx, bz);
            let d = near(b);
            if d > reach {
                continue;
            }
            let drawn = st.blocks.get(&b).map(|&(_, cells)| cells);
            let whole = match drawn {
                Some(_) => d >= BLOCK_FROM - BLOCK_SLACK,
                None => d >= BLOCK_FROM,
            };
            if whole {
                let cells = crate::land::cells_at(d) * BLOCK as usize;
                if drawn != Some(cells) {
                    work.push((d, Job::Block(b, cells)));
                }
            } else if drawn.is_some() {
                work.push((d, Job::Split(b)));
            } else {
                for t in tiles_of(b) {
                    let d = flat(t);
                    if d > reach {
                        continue;
                    }
                    let cells = crate::land::cells_at(d);
                    match st.tiles.get(&t) {
                        Some(&(_, now, _)) if now == cells => {}
                        _ => work.push((d, Job::Tile(t, cells))),
                    }
                }
            }
        }
    }
    work.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut behind = work.len() > TILES_PER_FRAME;
    let ground = st
        .ground
        .get_or_insert_with(|| {
            // Seen from below only by its skirts, which face out either
            // way round.
            materials.paint.add(Paint {
                two_sided: true,
                ..Paint::white()
            })
        })
        .clone();
    for (n, (_, job)) in work.into_iter().take(TILES_PER_FRAME).enumerate() {
        if n > 0 && started.elapsed().as_secs_f32() * 1000.0 > BUILD_BUDGET_MS {
            behind = true;
            break;
        }
        match job {
            Job::Tile(t, cells) => {
                let (mesh, water) =
                    crate::land::tile_mesh(atlas, t.0, t.1, cells, &mut st.palettes);
                let corner = Vec3::new(t.0 as f32 * size, 0.0, t.1 as f32 * size);
                let e = commands
                    .spawn((
                        Mesh3d(meshes.add(drawn_only(mesh))),
                        MeshMaterial3d(ground.clone()),
                        Transform::from_translation(corner),
                        bevy::pbr::NotShadowCaster,
                        ChildOf(root),
                    ))
                    .id();
                let w =
                    water.map(|surface| pond(commands, meshes, materials, st, root, t, surface));
                drop_tile(commands, st, t);
                st.tiles.insert(t, (e, cells, w));
            }
            Job::Block(b, cells) => {
                let corner = (b.0 * BLOCK, b.1 * BLOCK);
                let (mesh, water) =
                    crate::land::area_mesh(atlas, corner, BLOCK, cells, &mut st.palettes);
                let e = commands
                    .spawn((
                        Mesh3d(meshes.add(drawn_only(mesh))),
                        MeshMaterial3d(ground.clone()),
                        Transform::from_translation(Vec3::new(
                            corner.0 as f32 * size,
                            0.0,
                            corner.1 as f32 * size,
                        )),
                        bevy::pbr::NotShadowCaster,
                        ChildOf(root),
                        Name::new("a block of far land"),
                    ))
                    .id();
                if let Some((old, _)) = st.blocks.insert(b, (e, cells)) {
                    commands.entity(old).despawn();
                }
                // Each of its tiles is still a tile, to the brush and the
                // water: where strokes are laid as splats, an entity to
                // carry its strokes (`crate::splats`), drawing nothing itself;
                // elsewhere nothing at all.
                for t in tiles_of(b) {
                    let w = water
                        .get(&t)
                        .map(|&surface| pond(commands, meshes, materials, st, root, t, surface));
                    drop_tile(commands, st, t);
                    let stub = if crate::splats::on() {
                        commands.spawn(Name::new("a tile of a far block")).id()
                    } else {
                        Entity::PLACEHOLDER
                    };
                    st.tiles.insert(t, (stub, cells / BLOCK as usize, w));
                }
            }
            Job::Split(b) => {
                for t in tiles_of(b) {
                    let cells = crate::land::cells_at(flat(t));
                    let (mesh, water) =
                        crate::land::tile_mesh(atlas, t.0, t.1, cells, &mut st.palettes);
                    let corner = Vec3::new(t.0 as f32 * size, 0.0, t.1 as f32 * size);
                    let e = commands
                        .spawn((
                            Mesh3d(meshes.add(drawn_only(mesh))),
                            MeshMaterial3d(ground.clone()),
                            Transform::from_translation(corner),
                            bevy::pbr::NotShadowCaster,
                            ChildOf(root),
                        ))
                        .id();
                    let w = water
                        .map(|surface| pond(commands, meshes, materials, st, root, t, surface));
                    drop_tile(commands, st, t);
                    st.tiles.insert(t, (e, cells, w));
                }
                if let Some((e, _)) = st.blocks.remove(&b) {
                    commands.entity(e).despawn();
                }
            }
        }
    }
    behind
}

/// A tile let go of: whatever draws it or carries its strokes, and its water.
fn drop_tile(commands: &mut Commands, st: &mut Stream, t: (i32, i32)) {
    if let Some((e, _, water)) = st.tiles.remove(&t) {
        if e != Entity::PLACEHOLDER {
            commands.entity(e).despawn();
        }
        if let Some(w) = water {
            commands.entity(w).despawn();
        }
    }
}

/// **The water over a tile**: one shared plane, at its surface.
fn pond(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Materials,
    st: &mut Stream,
    root: Entity,
    t: (i32, i32),
    surface: f32,
) -> Entity {
    let size = sim::atlas::TILE_M as f32;
    let plane = st
        .pond
        .get_or_insert_with(|| meshes.add(Plane3d::default().mesh().size(size, size)))
        .clone();
    let colour = st
        .water
        .get_or_insert_with(|| {
            let c = look::palette::of(sim::arena::ArenaId::MOUTH).of(sim::arena::Material::Water);
            materials.standard.add(StandardMaterial {
                base_color: Color::srgba(c[0], c[1], c[2], 0.78),
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.15,
                reflectance: 0.6,
                ..default()
            })
        })
        .clone();
    let corner = Vec3::new(t.0 as f32 * size, 0.0, t.1 as f32 * size);
    commands
        .spawn((
            Mesh3d(plane),
            MeshMaterial3d(colour),
            Transform::from_translation(corner + Vec3::new(size * 0.5, surface, size * 0.5)),
            bevy::pbr::NotShadowCaster,
            ChildOf(root),
        ))
        .id()
}

/// **How far boxes are drawn a block at a time**: from this far, by the
/// nearest of their block (`BLOCK` tiles a side), every box in it is one of a
/// handful of meshes rather than an entity each (`docs/design/exploration/
/// 0024_distant_detail.md`, step 4). Far enough that none of them is drawn
/// in its near relief (a box is at most half a block from its block, so
/// further than [`NEAR`] and its slack), and far enough that the window to
/// the fighter (`crate::window`), which is round the camera, never meets one.
const MERGED_FROM: f32 = 160.0;

/// How much nearer a merged block comes before its boxes are drawn one by
/// one again.
const MERGED_SLACK: f32 = 12.0;

/// What drawing a box needs, held together.
struct BoxKit<'k, 'w, 's, 'm> {
    commands: &'k mut Commands<'w, 's>,
    meshes: &'k mut Assets<Mesh>,
    materials: &'k mut Materials<'m>,
    windows: &'k mut crate::window::Windows,
    atlas: &'k Atlas,
    root: Entity,
    white: &'k Handle<Paint>,
    see: &'k (Handle<Paint>, Handle<Paint>),
}

/// A box as it is drawn: the place it is coloured by, where its coordinates'
/// origin is drawn, whether it hangs, and whether its form is drawn
/// differently near and far.
fn box_look(st: &mut Stream, atlas: &Atlas, i: u32) -> Option<(PlaceLook, Vec3, bool, bool)> {
    let s = &atlas.solids[i as usize];
    let src = atlas.sources[i as usize];
    // A box is coloured as its own place colours it, and a box of no
    // place -- a cliff round a room, a door's sill -- as the place it
    // stands in.
    let mid = s.min.add(s.max);
    let (x, z) = (
        sim::Fx::from_raw(mid.x.raw() / 2),
        sim::Fx::from_raw(mid.z.raw() / 2),
    );
    let place = if src.place == EXTRA {
        atlas.place_at(x, z).or(atlas.places.first())
    } else {
        atlas.places.get(src.place as usize)
    }?;
    let look = st
        .looks
        .entry(place.arena)
        .or_insert_with(|| PlaceLook::of(place.get()))
        .clone();
    let hangs = s.min.y.raw() > atlas.relief_at(x, z).raw();
    // Only a form that is drawn differently near and far is ever
    // rebuilt for it.
    let lod = crate::forms::form_of(look.id, s, hangs) == crate::forms::Form::Masonry;
    Some((look, v3(place.at), hangs, lod))
}

impl BoxKit<'_, '_, '_, '_> {
    /// **One box on its own**: what draws it, and -- for a form drawn
    /// differently near and far -- whether it is drawn near.
    fn draw(&mut self, st: &mut Stream, i: u32, near: bool) -> Option<(Entity, Option<bool>)> {
        let s = &self.atlas.solids[i as usize];
        let (look, offset, hangs, lod) = box_look(st, self.atlas, i)?;
        // A tree's trunk is drawn as a tree.
        if crate::land::is_tree(s) {
            let palette = look.palette;
            let e = st.trees.spawn(
                self.commands,
                self.meshes,
                self.materials,
                s,
                look.id,
                &palette,
                self.root,
                (!self.atlas.holds_the_camera(i as usize)).then_some(&mut *self.windows),
            );
            return Some((e, None));
        }
        let e = draw_solid(
            self.commands,
            self.meshes,
            s,
            offset,
            hangs,
            &look,
            (self.white, self.see),
            Under::Parent(self.root),
            near || !lod,
        );
        Some((e, lod.then_some(near)))
    }
}

/// Which block of boxes each box is merged into far off, and each block's
/// boxes.
fn index_boxes(st: &mut Stream, atlas: &Atlas) {
    let side = sim::atlas::TILE_M as f32 * BLOCK as f32;
    st.box_block = atlas
        .solids
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let (lo, hi) = (v3(s.min), v3(s.max));
            let wide = (hi.x - lo.x).max(hi.z - lo.z) > side;
            if atlas.hidden[i] || crate::land::is_tree(s) || wide {
                return None;
            }
            let m = (lo + hi) * 0.5;
            Some(((m.x / side).floor() as i32, (m.z / side).floor() as i32))
        })
        .collect();
    st.box_blocks.clear();
    for (i, b) in st.box_block.iter().enumerate() {
        if let Some(b) = b {
            st.box_blocks.entry(*b).or_default().push(i as u32);
        }
    }
}

/// A block's flat distance from `at`, to the nearest of it.
fn block_distance(b: (i32, i32), at: Vec3) -> f32 {
    let side = sim::atlas::TILE_M as f32 * BLOCK as f32;
    let (x0, z0) = (b.0 as f32 * side, b.1 as f32 * side);
    let dx = (x0 - at.x).max(at.x - x0 - side).max(0.0);
    let dz = (z0 - at.z).max(at.z - z0 - side).max(0.0);
    (dx * dx + dz * dz).sqrt()
}

/// The blocks of boxes that are far, within reach: drawn merged, or to be.
fn far_blocks(st: &Stream, at: Vec3, reach: f32) -> std::collections::HashSet<(i32, i32)> {
    st.box_blocks
        .keys()
        .copied()
        .filter(|&b| {
            let d = block_distance(b, at);
            let merged = st.merged.contains_key(&b);
            let from = if merged {
                MERGED_FROM - MERGED_SLACK
            } else {
                MERGED_FROM
            };
            d >= from && d <= reach + if merged { SLACK } else { 0.0 }
        })
        .collect()
}

/// **Merge the far blocks of boxes, and split the near ones**: a block that
/// has gone far is drawn merged and its boxes' own entities let go; one that
/// has come near again is drawn box by box, all of it in one frame, so there
/// is never a hole where it was; one past reach is dropped. Nearest first,
/// at least one a frame; true if more wait.
fn merge_boxes(
    kit: &mut BoxKit,
    st: &mut Stream,
    far: &std::collections::HashSet<(i32, i32)>,
    at: Vec3,
    reach: f32,
    started: &Instant,
) -> bool {
    let gone: Vec<(i32, i32)> = st
        .merged
        .keys()
        .copied()
        .filter(|&b| block_distance(b, at) > reach + SLACK)
        .collect();
    for b in gone {
        for e in st.merged.remove(&b).unwrap_or_default() {
            kit.commands.entity(e).despawn();
        }
    }
    let mut work: Vec<(f32, (i32, i32), bool)> = far
        .iter()
        .filter(|b| !st.merged.contains_key(b))
        .map(|&b| (block_distance(b, at), b, true))
        .chain(
            st.merged
                .keys()
                .filter(|b| !far.contains(b))
                .map(|&b| (block_distance(b, at), b, false)),
        )
        .collect();
    work.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut behind = false;
    for (n, &(_, b, merge)) in work.iter().enumerate() {
        if n > 0 && started.elapsed().as_secs_f32() * 1000.0 > BUILD_BUDGET_MS {
            behind = true;
            break;
        }
        let boxes = st.box_blocks.get(&b).cloned().unwrap_or_default();
        if merge {
            let parts = merged_parts(kit, st, &boxes, b);
            st.merged.insert(b, parts);
            for i in boxes {
                if let Some((e, _)) = st.solids.remove(&i) {
                    kit.commands.entity(e).despawn();
                }
            }
        } else {
            for &i in &boxes {
                if let Some(drawn) = kit.draw(st, i, false) {
                    if let Some((old, _)) = st.solids.insert(i, drawn) {
                        kit.commands.entity(old).despawn();
                    }
                }
            }
            for e in st.merged.remove(&b).unwrap_or_default() {
                kit.commands.entity(e).despawn();
            }
        }
    }
    behind
}

/// **One block of boxes as a few meshes**: every box's mesh, as far off it is
/// drawn, moved to where it stands and appended to the mesh of its kind --
/// its paint (windowed or not), whether it casts a shadow, and which
/// attributes its vertices carry, so nothing about how it is drawn changes.
fn merged_parts(kit: &mut BoxKit, st: &mut Stream, boxes: &[u32], b: (i32, i32)) -> Vec<Entity> {
    let side = sim::atlas::TILE_M as f32 * BLOCK as f32;
    let corner = Vec3::new(b.0 as f32 * side, 0.0, b.1 as f32 * side);
    let mut kinds: Vec<(Kind, Joined)> = Vec::new();
    for &i in boxes {
        let s = &kit.atlas.solids[i as usize];
        let Some((look, offset, hangs, lod)) = box_look(st, kit.atlas, i) else {
            continue;
        };
        let Some((mesh, relief, at)) = solid_mesh(s, offset, hangs, &look, !lod) else {
            continue;
        };
        let see = !s.holds_the_camera();
        // A crown of leaves lets the light through, as `draw_solid` has it.
        let casts = !(matches!(s.shape, sim::arena::Shape::Round { .. })
            && s.material == sim::arena::Material::Grass);
        let moved = at + offset - corner;
        for (m, casts) in [(Some(mesh), casts), (relief, false)] {
            let Some(m) = m else {
                continue;
            };
            let kind = Kind::of(&m, see, casts);
            match kinds.iter_mut().find(|(k, _)| *k == kind) {
                Some((_, j)) => j.add(&m, moved),
                None => {
                    let mut j = Joined::default();
                    j.add(&m, moved);
                    kinds.push((kind, j));
                }
            }
        }
    }
    kinds
        .into_iter()
        .map(|(kind, joined)| {
            let paint = if kind.see { &kit.see.0 } else { kit.white };
            let mut e = kit.commands.spawn((
                Mesh3d(kit.meshes.add(drawn_only(joined.build()))),
                MeshMaterial3d(paint.clone()),
                Transform::from_translation(corner),
                Visibility::default(),
                ChildOf(kit.root),
                Name::new("a block of far boxes"),
            ));
            if !kind.casts {
                e.insert(bevy::pbr::NotShadowCaster);
            }
            e.id()
        })
        .collect()
}

/// Which merged mesh a box's mesh goes into.
#[derive(PartialEq)]
struct Kind {
    see: bool,
    casts: bool,
    topology: bevy::render::mesh::PrimitiveTopology,
    /// The attributes, by id and format, in the mesh's order.
    layout: Vec<(bevy::render::mesh::MeshVertexAttributeId, VertexFormat)>,
}

impl Kind {
    fn of(m: &Mesh, see: bool, casts: bool) -> Kind {
        Kind {
            see,
            casts,
            topology: m.primitive_topology(),
            layout: m.attributes().map(|(a, _)| (a.id, a.format)).collect(),
        }
    }
}

/// Meshes of one kind, appended.
#[derive(Default)]
struct Joined {
    topology: Option<bevy::render::mesh::PrimitiveTopology>,
    attributes: Vec<(MeshVertexAttribute, VertexAttributeValues)>,
    indices: Vec<u32>,
    count: u32,
}

impl Joined {
    /// `m`, moved by `by`: its positions moved, everything else as it is.
    fn add(&mut self, m: &Mesh, by: Vec3) {
        self.topology.get_or_insert(m.primitive_topology());
        let n = m.count_vertices() as u32;
        for (attribute, values) in m.attributes() {
            let slot = match self
                .attributes
                .iter()
                .position(|(a, _)| a.id == attribute.id)
            {
                Some(k) => k,
                None => {
                    self.attributes.push((*attribute, empty_like(values)));
                    self.attributes.len() - 1
                }
            };
            let moved = attribute.id == Mesh::ATTRIBUTE_POSITION.id;
            append(&mut self.attributes[slot].1, values, moved.then_some(by));
        }
        match m.indices() {
            Some(Indices::U32(ix)) => self.indices.extend(ix.iter().map(|k| k + self.count)),
            Some(Indices::U16(ix)) => self
                .indices
                .extend(ix.iter().map(|&k| k as u32 + self.count)),
            None => self.indices.extend((0..n).map(|k| k + self.count)),
        }
        self.count += n;
    }

    fn build(self) -> Mesh {
        let mut mesh = Mesh::new(
            self.topology
                .unwrap_or(bevy::render::mesh::PrimitiveTopology::TriangleList),
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );
        for (attribute, values) in self.attributes {
            mesh.insert_attribute(attribute, values);
        }
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

/// An empty list of the same kind of value.
fn empty_like(v: &VertexAttributeValues) -> VertexAttributeValues {
    let mut v = v.clone();
    match &mut v {
        VertexAttributeValues::Float32(x) => x.clear(),
        VertexAttributeValues::Float32x2(x) => x.clear(),
        VertexAttributeValues::Float32x3(x) => x.clear(),
        VertexAttributeValues::Float32x4(x) => x.clear(),
        VertexAttributeValues::Uint32(x) => x.clear(),
        VertexAttributeValues::Uint16x4(x) => x.clear(),
        VertexAttributeValues::Unorm8x4(x) => x.clear(),
        _ => {}
    }
    v
}

/// Append `from` to `to`, the positions moved by `by`. Values of a kind no
/// box's mesh carries are left as they were, and would show as a mesh with
/// too few of them: `every_box_mesh_merges` is what says none does.
fn append(to: &mut VertexAttributeValues, from: &VertexAttributeValues, by: Option<Vec3>) {
    use VertexAttributeValues as V;
    match (to, from) {
        (V::Float32x3(a), V::Float32x3(b)) => match by {
            Some(by) => a.extend(b.iter().map(|p| (Vec3::from(*p) + by).to_array())),
            None => a.extend_from_slice(b),
        },
        (V::Float32(a), V::Float32(b)) => a.extend_from_slice(b),
        (V::Float32x2(a), V::Float32x2(b)) => a.extend_from_slice(b),
        (V::Float32x4(a), V::Float32x4(b)) => a.extend_from_slice(b),
        (V::Uint32(a), V::Uint32(b)) => a.extend_from_slice(b),
        (V::Uint16x4(a), V::Uint16x4(b)) => a.extend_from_slice(b),
        (V::Unorm8x4(a), V::Unorm8x4(b)) => a.extend_from_slice(b),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every box a far block merges keeps all it carries**: each of its
    /// mesh's attributes is a kind [`append`] appends, so a merged block is
    /// never a mesh with fewer colours or stroke kinds than vertices.
    #[test]
    fn every_box_mesh_merges() {
        use VertexAttributeValues as V;
        let atlas = atlas::valley();
        let mut st = Stream::default();
        index_boxes(&mut st, atlas);
        let mut seen = 0;
        for (i, b) in st.box_block.clone().iter().enumerate() {
            if b.is_none() {
                continue;
            }
            let s = &atlas.solids[i];
            let Some((look, offset, hangs, lod)) = box_look(&mut st, atlas, i as u32) else {
                continue;
            };
            let Some((mesh, relief, _)) = solid_mesh(s, offset, hangs, &look, !lod) else {
                continue;
            };
            for m in [Some(mesh), relief].into_iter().flatten() {
                for (attribute, values) in m.attributes() {
                    assert!(
                        matches!(
                            values,
                            V::Float32(_)
                                | V::Float32x2(_)
                                | V::Float32x3(_)
                                | V::Float32x4(_)
                                | V::Uint32(_)
                                | V::Uint16x4(_)
                                | V::Unorm8x4(_)
                        ),
                        "box {i}'s {} is a kind a merge would drop",
                        attribute.name
                    );
                }
                seen += 1;
            }
        }
        assert!(seen > 1000, "only {seen} box meshes looked at");
    }
}
