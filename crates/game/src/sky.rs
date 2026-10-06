//! Putting a sky on the screen: the dome, and the haze on the camera.
//!
//! **What** a sky is -- the gradient, the glow, how far the eye reaches, and
//! why an arena needs all three -- is [`look::sky`], which has no engine in it
//! and can be looked at with `cargo run -p look --example skies`. This file is
//! only the part that needs Bevy: a mesh, a material, and the fog component.
//!
//! Two pieces, and they have to agree. The **dome** is a sphere turned
//! inside out, coloured per vertex and unlit, riding the camera so nothing ever
//! reaches it. The **fog** fades distant geometry to the dome's horizon colour,
//! so a thing far away goes pale toward exactly what is behind it. Give them
//! different colours and the fade stops reading as air and starts reading as a
//! grey wash laid over the picture.

use bevy::pbr::{DistanceFog, FogFalloff, NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use look::tint;

pub use look::Sky;

/// The dome, so it can be moved with the camera and replaced with the arena.
#[derive(Component)]
pub struct Dome;

/// How far away the dome sits. Inside the camera's far plane with room to
/// spare, and it rides the camera so nothing ever reaches it.
const RADIUS: f32 = 480.0;

/// Build the dome for a sky. Rings of vertices, coloured by which way they
/// face, wound inward so the inside is what is drawn.
pub fn dome(sky: &Sky, sun: Vec3) -> Mesh {
    // Dense, because the colour lives in the vertices and a gradient across a
    // whole screen shows every facet it is made of. Forty rings is one band
    // every four and a half degrees, and with a shaped falloff over them the
    // sky came out as visible concentric rings -- a target, not a sky. These
    // are a few thousand triangles drawn once per arena.
    const RINGS: usize = 96;
    const SEGMENTS: usize = 72;

    let sky = sky.resolved();
    let sun = sun.normalize_or_zero();
    let mut positions = Vec::with_capacity((RINGS + 1) * (SEGMENTS + 1));
    let mut colours = Vec::with_capacity((RINGS + 1) * (SEGMENTS + 1));
    let mut normals = Vec::with_capacity((RINGS + 1) * (SEGMENTS + 1));
    let mut indices = Vec::with_capacity(RINGS * SEGMENTS * 6);

    for ring in 0..=RINGS {
        // Latitude from straight up to straight down.
        let phi = ring as f32 / RINGS as f32 * std::f32::consts::PI;
        let (sin_phi, cos_phi) = phi.sin_cos();
        for seg in 0..=SEGMENTS {
            let theta = seg as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let (sin_t, cos_t) = theta.sin_cos();
            let dir = Vec3::new(sin_phi * cos_t, cos_phi, sin_phi * sin_t);
            positions.push((dir * RADIUS).to_array());
            // Facing inward, which is the only side ever seen.
            normals.push((-dir).to_array());
            let c = sky.looking(dir.to_array(), sun.to_array());
            // The mesh carries linear colour; the gradient is authored in
            // sRGB, which is how a person reads a colour.
            let c = tint::linear(c);
            colours.push([c[0], c[1], c[2], 1.0]);
        }
    }

    let stride = SEGMENTS + 1;
    for ring in 0..RINGS {
        for seg in 0..SEGMENTS {
            let a = (ring * stride + seg) as u32;
            let (b, c, d) = (a + 1, a + stride as u32, a + stride as u32 + 1);
            // Wound so the inward faces are the front ones.
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// The material a dome wears: its own colour and nothing else's.
pub fn material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        // Unlit, because a sky is the light rather than a thing lit by it.
        unlit: true,
        // **Not fogged.** The dome is what everything else fades *into*, so
        // fogging it would flatten the whole gradient to the horizon colour and
        // take the sky away in the act of drawing it.
        fog_enabled: false,
        // Drawn from the inside, which is also why it needs no shadows in
        // either direction.
        cull_mode: None,
        ..default()
    }
}

/// Put the dome where the camera is, so its far wall is always the same
/// distance away and the eye can never reach the seam.
pub fn follow(
    camera: Query<&GlobalTransform, (With<crate::MainCamera>, Without<Dome>)>,
    mut domes: Query<&mut Transform, With<Dome>>,
) {
    let Ok(eye) = camera.single() else {
        return;
    };
    for mut dome in &mut domes {
        dome.translation = eye.translation();
    }
}

/// Spawn the dome and set the camera's fog. Called when the arena changes.
pub fn raise(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    sky: &Sky,
    sun: Vec3,
    scenery: impl Bundle,
) {
    commands.spawn((
        Mesh3d(meshes.add(dome(sky, sun))),
        MeshMaterial3d(materials.add(material())),
        Transform::default(),
        NotShadowCaster,
        NotShadowReceiver,
        Dome,
        scenery,
    ));
}

/// The fog for a sky, ready to go on the camera.
pub fn fog(sky: &Sky) -> DistanceFog {
    let sky = sky.resolved();
    // Fading into the horizon's colour rather than into a grey is the whole
    // trick: a thing that goes pale toward exactly what is behind it reads as
    // air, and a thing that goes pale toward something else reads as a bug.
    let c = sky.horizon;
    DistanceFog {
        color: Color::srgb(c[0], c[1], c[2]),
        falloff: {
            let (start, end) = sky.fog();
            FogFalloff::Linear { start, end }
        },
        ..default()
    }
}
