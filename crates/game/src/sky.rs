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

/// How far either side of a doorway the two places' skies blend, in metres.
const BLEND: f32 = 24.0;

/// What the air was last set to in the valley: the place the camera was in,
/// the one across the nearest doorway, how far toward it in fiftieths, and
/// the dome it was put on.
#[derive(Default)]
pub struct Weather {
    shown: Option<(sim::arena::ArenaId, sim::arena::ArenaId, u8, Entity)>,
}

/// **The air across a doorway** (`docs/design/atlas.md`): in the valley each
/// place keeps its own sky, and walking through a doorway goes from one to
/// the other over [`BLEND`] metres either side of it rather than at the
/// moment the world moves into the next place. Dome, fog, clear colour, the
/// sky's light and the sun's bearing all follow. The blend is
/// `look::Sky::toward`; this only says how far.
#[allow(clippy::too_many_arguments)] // A Bevy system: one argument per resource it reads.
pub fn weather(
    sim: Res<crate::Sim>,
    mut state: Local<Weather>,
    camera: Query<(Entity, &Transform), With<crate::MainCamera>>,
    domes: Query<(Entity, &Mesh3d), With<Dome>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<AmbientLight>,
    mut fill: Query<&mut DirectionalLight, With<crate::arenas::Skylight>>,
    mut suns: Query<&mut Transform, (With<crate::arenas::Sun>, Without<crate::MainCamera>)>,
) {
    let w = &sim.cur;
    if !w.valley.on {
        state.shown = None;
        return;
    }
    let (Ok((eye, at)), Ok((dome, mesh))) = (camera.single(), domes.single()) else {
        return;
    };
    let atlas = sim::atlas::valley();
    let o = w.map_origin();
    let to_fx = |v: f32| sim::Fx::from_raw((v * 65536.0) as i32);
    let f = |v: sim::Fx| v.to_f32_for_render();
    let (x, z) = (at.translation.x + f(o.x), at.translation.z + f(o.z));
    let home = atlas
        .place_at(to_fx(x), to_fx(z))
        .map_or(w.arena, |p| p.arena);
    // The nearest doorway out of where the camera is, and how far.
    let near = atlas
        .doors
        .iter()
        .filter_map(|d| {
            let other = if d.a.0 == home {
                d.b.0
            } else if d.b.0 == home {
                d.a.0
            } else {
                return None;
            };
            let dx = (f(d.cut.min.x) - x).max(x - f(d.cut.max.x)).max(0.0);
            let dz = (f(d.cut.min.z) - z).max(z - f(d.cut.max.z)).max(0.0);
            Some((other, (dx * dx + dz * dz).sqrt()))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1));
    // Half-way at the doorway itself, from either side, so the two places
    // meet in the middle and nothing jumps as the camera crosses.
    let (other, t) = match near {
        Some((other, d)) if d < BLEND => {
            let u = 1.0 - d / BLEND;
            (other, 0.5 * u * u * (3.0 - 2.0 * u))
        }
        _ => (home, 0.0),
    };
    let step = (t * 50.0).round() as u8;
    let key = (home, other, step, dome);
    if state.shown == Some(key) {
        return;
    }
    state.shown = Some(key);
    let t = step as f32 / 50.0;
    let sky = look::skies::of(home).toward(look::skies::of(other), t);
    let sun = crate::arenas::sun(home).lerp(crate::arenas::sun(other), t);
    meshes.insert(&mesh.0, self::dome(&sky, sun));
    commands.entity(eye).insert(fog(&sky));
    let h = sky.horizon;
    clear.0 = Color::srgb(h[0], h[1], h[2]);
    let shade = look::palette::Palette::under(&sky).shade;
    ambient.color = Color::srgb(shade[0], shade[1], shade[2]);
    for mut light in &mut fill {
        light.color = Color::srgb(shade[0], shade[1], shade[2]);
    }
    for mut light in &mut suns {
        *light = Transform::from_translation(sun).looking_at(Vec3::ZERO, Vec3::Y);
    }
}
