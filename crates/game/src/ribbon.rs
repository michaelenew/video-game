//! Drawing the trail a weapon leaves.
//!
//! The geometry comes from [`view::trail`], which recomputes where the edge
//! was rather than remembering it -- see that module for why that matters
//! under rollback. This is only the mesh.
//!
//! **One mesh per fighter, rebuilt in place.** A ribbon is at most sixteen
//! vertices and it changes every frame, so the cost is rewriting two small
//! buffers; spawning and despawning an entity per swing would put allocation
//! on exactly the frames that are already the busiest.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use sim::state::Action;
use view::pose::PoseInput;
use view::trail::{self, Edge};

use crate::{MAX_PLAYERS, Sim};

/// Which fighter's trail this is.
#[derive(Component)]
pub struct Ribbon(pub usize);

/// Below this, the edge is not really travelling and a ribbon would be a smear
/// rather than an arc. In metres per frame.
const MOVING: f32 = 0.06;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for owner in 0..MAX_PLAYERS {
        // The trail wears its fighter's identity colour, which is a free piece
        // of information in a crowded fight: whose swing that was, read off the
        // arc rather than off the fighter it came from.
        // Shaded down, not up. A trail at lightness 0.92 is white whatever its
        // hue -- that is what high lightness *means* -- so the identity colour
        // it was supposed to carry went missing and every fighter's swing
        // looked the same. Taking the lightness down and the emissive
        // multiplier up gives the same brightness with the hue intact.
        let rgb = art::palette::player_shade(owner, 0.60);
        commands.spawn((
            Mesh3d(meshes.add(empty_ribbon())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                // Emissive and additive, the same reasoning as the fire: a
                // trail is light left hanging in the air, so it brightens what
                // is behind it and never darkens it. That also means the taper
                // needs no alpha -- where the ribbon has faded to black it
                // simply stops contributing.
                emissive: LinearRgba::rgb(rgb[0] * 22.0, rgb[1] * 22.0, rgb[2] * 22.0),
                alpha_mode: AlphaMode::Add,
                cull_mode: None,
                ..default()
            })),
            Transform::default(),
            Visibility::Hidden,
            Ribbon(owner),
        ));
    }
}

/// Sixteen vertices of nothing, allocated once and overwritten every frame.
fn empty_ribbon() -> Mesh {
    let n = trail::MAX_SAMPLES;
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; n * 2]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; n * 2]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32; 2]; n * 2]);

    // A strip of quads between consecutive samples, written as triangles once
    // and never rebuilt -- only the positions move.
    let mut indices = Vec::new();
    for i in 0..n.saturating_sub(1) {
        let (a, b, c, d) = (i * 2, i * 2 + 1, i * 2 + 2, i * 2 + 3);
        indices.extend_from_slice(&[a as u32, b as u32, c as u32]);
        indices.extend_from_slice(&[b as u32, d as u32, c as u32]);
    }
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn update(
    sim: Res<Sim>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ribbons: Query<(&Ribbon, &Mesh3d, &mut Transform, &mut Visibility)>,
) {
    let frame = view::interp::interpolate(&sim.prev, &sim.cur, sim.clock.alpha());

    for (tag, handle, mut tf, mut vis) in ribbons.iter_mut() {
        let p = frame.players[tag.0];
        let class = sim.cur.players[tag.0].class;

        // Only while a move is actually out. A trail during recovery is the
        // weapon being drawn back, which is not the motion anyone is reading.
        let swinging = matches!(p.action, Action::Startup { .. } | Action::Active { .. });
        let (into, total) = crate::phase_frames(&p, class);
        let input = PoseInput {
            clip: crate::clip_for(&p, class),
            action: p.action,
            frames_into: into,
            frames_total: total,
            speed: p.speed,
            grounded: p.grounded,
            crouching: p.crouching,
            sim_frame: frame.sim_frame,
        };

        let sweep = if swinging {
            trail::sweep(input, trail::MAX_SAMPLES)
        } else {
            Vec::new()
        };

        // The speed gate is what keeps this off every jab and off every move
        // that still falls back to a procedural pose -- those are one pose per
        // phase, so their sweep is the same edge eight times and a ribbon
        // would be a stationary smear. See the test in `view`.
        if sweep.len() < 2 || trail::speed(&sweep) < MOVING {
            *vis = Visibility::Hidden;
            continue;
        }

        // The ribbon is built in character-local space and the entity carries
        // the fighter's own transform, which is the same trick the body parts
        // use -- and it means a trail cannot drift away from the fighter that
        // threw it.
        tf.translation = Vec3::new(p.pos[0], p.pos[1], p.pos[2]);
        tf.rotation = Quat::from_rotation_y(p.facing[0].atan2(p.facing[2]));
        *vis = Visibility::Inherited;

        if let Some(mesh) = meshes.get_mut(&handle.0) {
            write_ribbon(mesh, &sweep);
        }
    }
}

fn write_ribbon(mesh: &mut Mesh, sweep: &[Edge]) {
    let n = trail::MAX_SAMPLES;
    let mut pos = vec![[0.0f32; 3]; n * 2];
    let mut uv = vec![[0.0f32; 2]; n * 2];

    for i in 0..n {
        // Samples past the end of a short sweep collapse onto the last real
        // one, which makes their triangles degenerate and therefore invisible.
        // Cheaper and less error-prone than rewriting the index buffer for
        // every length a sweep can be.
        let e = sweep
            .get(i)
            .or_else(|| sweep.last())
            .copied()
            .unwrap_or(Edge {
                near: [0.0; 3],
                far: [0.0; 3],
                age: 1.0,
            });
        pos[i * 2] = e.near;
        pos[i * 2 + 1] = e.far;
        // The V coordinate carries age, so the material can taper the trail
        // toward its tail without needing a vertex colour attribute.
        uv[i * 2] = [0.0, e.age];
        uv[i * 2 + 1] = [1.0, e.age];
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
}
