//! **Distance loading** (`docs/design/atlas.md`): in the valley, what is
//! drawn is what is near the camera, and nothing else.
//!
//! The valley is one map (`sim::atlas`), and drawing all of it at once is
//! what a world of a million boxes cannot do. So the map is drawn in two
//! kinds of piece, each loaded when the camera comes within reach of it and
//! dropped when it goes further than that again:
//!
//! - **A place**: its floor, the patches on it, its props and what is
//!   scattered on the floor, its seams and vines -- everything
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

use bevy::prelude::*;
use sim::arena::ArenaId;
use sim::atlas::{self, Atlas, EXTRA};

use crate::arenas::{PlaceLook, Under, draw_place, draw_solid};

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

/// What is loaded, and where it hangs.
#[derive(Resource, Default)]
pub struct Stream {
    root: Option<Entity>,
    /// The map origin the root was last put at.
    origin: Option<Vec3>,
    /// Places by their index on the map.
    places: HashMap<usize, Entity>,
    /// Boxes by their index on the map.
    solids: HashMap<u32, Entity>,
    /// Each place's look, worked out once.
    looks: HashMap<ArenaId, PlaceLook>,
    white: Option<Handle<StandardMaterial>>,
    /// Where on the map the camera was when the set was last worked out.
    at: Option<Vec3>,
    /// The waystones' doors: one slab per gated doorway, shown while shut.
    gates: Vec<(usize, Entity)>,
}

impl Stream {
    /// How many places and boxes are drawn: what the dev overlay shows.
    pub fn loaded(&self) -> (usize, usize) {
        (self.places.len(), self.solids.len())
    }
}

/// RENDER-ONLY. Fixed point to metres.
fn fx(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
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
    mut st: ResMut<Stream>,
    camera: Query<&Transform, With<crate::MainCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gates: Query<&mut Visibility>,
) {
    let w = &sim.cur;
    if !w.valley.on {
        if let Some(root) = st.root.take() {
            commands.entity(root).despawn();
        }
        st.places.clear();
        st.solids.clear();
        st.gates.clear();
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
        .get_or_insert_with(|| materials.add(crate::shapes::plain()))
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

    // Boxes: through the tiles.
    let want = boxes_near(atlas, at, reach);
    let keep = boxes_near(atlas, at, reach + SLACK);
    let gone: Vec<u32> = st
        .solids
        .keys()
        .copied()
        .filter(|i| !keep.contains(i))
        .collect();
    for i in gone {
        if let Some(e) = st.solids.remove(&i) {
            commands.entity(e).despawn();
        }
    }
    let mut new: Vec<u32> = want
        .into_iter()
        .filter(|i| !st.solids.contains_key(i))
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
    for i in new {
        let s = &atlas.solids[i as usize];
        let src = atlas.sources[i as usize];
        // A box is coloured as its own place colours it, and a box of no
        // place -- a cliff round a room, a door's sill -- as the place it
        // stands in.
        let place = if src.place == EXTRA {
            let mid = s.min.add(s.max);
            atlas
                .place_at(
                    sim::Fx::from_raw(mid.x.raw() / 2),
                    sim::Fx::from_raw(mid.z.raw() / 2),
                )
                .or(atlas.places.first())
        } else {
            atlas.places.get(src.place as usize)
        };
        let Some(place) = place else {
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
            s,
            v3(place.at),
            &look,
            &white,
            Under::Parent(root),
        );
        st.solids.insert(i, e);
    }
    // More to build: work the set out again next frame rather than waiting
    // for the camera to move.
    if behind {
        st.at = None;
    }
}

/// One slab per waystone's doorway, dark and a little translucent, hidden
/// until the simulation shuts it.
fn spawn_gates(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
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
                MeshMaterial3d(materials.add(StandardMaterial {
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
