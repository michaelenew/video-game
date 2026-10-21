//! **The capture point, drawn** (`sim::versus`): a disc on the floor of a
//! King of the Hill map in the colour of the side that holds it -- pale
//! while nobody does -- a bright ring round its edge, and a second disc
//! growing out from its middle in the colour of the side taking it, as far
//! as the capture has got. Hidden on every arena without a hill.
//!
//! What it is and where comes from the simulation; this only paints it.
//! The disc lies at the point's own height, which is the plinth's or the
//! dome's top rather than the floor's.

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use sim::versus::NOBODY;

/// Which part of the point a mesh is.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Disc,
    Rim,
    Fill,
}

/// The point's colours: nobody's, and each side's -- the colours of seats
/// one and two, whose sides they are -- for the disc, the rim and the fill.
#[derive(Resource)]
pub struct Inks {
    disc: [Handle<StandardMaterial>; 3],
    rim: [Handle<StandardMaterial>; 3],
    fill: [Handle<StandardMaterial>; 2],
}

/// How far over the surface the disc lies, and the fill over it.
const LIFT: f32 = 0.04;
const FILL_LIFT: f32 = 0.06;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut ink = |c: Color, alpha: f32| {
        materials.add(StandardMaterial {
            base_color: c.with_alpha(alpha),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        })
    };
    let open = Color::srgb(0.95, 0.95, 0.90);
    let [blue, orange] = [crate::hud::SEAT_COLOURS[0], crate::hud::SEAT_COLOURS[1]];
    let inks = Inks {
        disc: [ink(open, 0.30), ink(blue, 0.40), ink(orange, 0.40)],
        rim: [ink(open, 0.95), ink(blue, 1.0), ink(orange, 1.0)],
        fill: [ink(blue, 0.45), ink(orange, 0.45)],
    };
    let disc = meshes.add(Circle::new(1.0));
    let rim = meshes.add(Annulus::new(0.9, 1.0));
    for (part, mesh, material) in [
        (Part::Disc, &disc, &inks.disc[0]),
        (Part::Rim, &rim, &inks.rim[0]),
        (Part::Fill, &disc, &inks.fill[0]),
    ] {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Visibility::Hidden,
            NotShadowCaster,
            part,
        ));
    }
    commands.insert_resource(inks);
}

pub fn place(
    sim: Res<crate::Sim>,
    inks: Res<Inks>,
    mut parts: Query<(
        &Part,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let k = sim.cur.hill;
    let hill = sim::versus::of(sim.cur.arena).and_then(|s| s.hill);
    let flat = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    for (part, mut transform, mut visible, mut material) in parts.iter_mut() {
        let Some(hill) = hill.filter(|_| k.on) else {
            *visible = Visibility::Hidden;
            continue;
        };
        let at = Vec3::new(
            hill.at.x.to_f32_for_render(),
            hill.at.y.to_f32_for_render(),
            hill.at.z.to_f32_for_render(),
        );
        let r = hill.radius.to_f32_for_render();
        let owner = if k.owner == NOBODY {
            0
        } else {
            1 + k.owner as usize % 2
        };
        let (wanted, scale, lift) = match part {
            Part::Disc => (&inks.disc[owner], r, LIFT),
            Part::Rim => (&inks.rim[owner], r, LIFT),
            Part::Fill => {
                if k.capper == NOBODY || k.progress == 0 {
                    *visible = Visibility::Hidden;
                    continue;
                }
                let share = k.percent() as f32 / 100.0;
                (&inks.fill[k.capper as usize % 2], r * share, FILL_LIFT)
            }
        };
        *transform = Transform {
            translation: at + Vec3::Y * lift,
            rotation: flat,
            scale: Vec3::new(scale.max(0.01), scale.max(0.01), 1.0),
        };
        *visible = Visibility::Inherited;
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }
}
