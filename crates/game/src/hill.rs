//! **The capture points, drawn** (`sim::versus`): a disc on the floor of a
//! King of the Hill map -- or each of a five-point map's five -- in the
//! colour of the side that holds it, pale while nobody does, a bright ring
//! round its edge, and a second disc growing out from its middle in the
//! colour of the side taking it, as far as the capture has got. Hidden on
//! every arena without one.
//!
//! What it is and where comes from the simulation; this only paints it.
//! The disc lies at the point's own height, which is the plinth's or the
//! dome's top rather than the floor's.

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use sim::versus::NOBODY;

/// Which part of a point a mesh is.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Disc,
    Rim,
    Fill,
}

/// A mesh of point `k`'s: the hill's is point 0.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    part: Part,
    k: usize,
}

/// One point as it is drawn this frame: where, who holds it, who is taking
/// it and how far.
struct Shown {
    hill: sim::versus::Hill,
    owner: u8,
    capper: u8,
    percent: u32,
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
    for k in 0..sim::versus::POINTS {
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
                Piece { part, k },
            ));
        }
    }
    commands.insert_resource(inks);
}

pub fn place(
    sim: Res<crate::Sim>,
    inks: Res<Inks>,
    mut parts: Query<(
        &Piece,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let w = &sim.cur;
    let stage = sim::versus::of(w.arena);
    // What is drawn: the hill's one point, or the line's five.
    let mut shown: [Option<Shown>; sim::versus::POINTS] = Default::default();
    if let Some(hill) = stage.and_then(|s| s.hill).filter(|_| w.hill.on) {
        let k = &w.hill;
        shown[0] = Some(Shown {
            hill,
            owner: k.owner,
            capper: k.capper,
            percent: if k.progress > 0 { k.percent() } else { 0 },
        });
    }
    if let Some(line) = stage.and_then(|s| s.front).filter(|_| w.front.on) {
        let f = &w.front;
        for (k, slot) in shown.iter_mut().enumerate() {
            *slot = Some(Shown {
                hill: line.points[k],
                owner: f.owner[k],
                capper: f.capper[k],
                percent: if f.progress[k] > 0 { f.percent(k) } else { 0 },
            });
        }
    }
    let flat = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    for (piece, mut transform, mut visible, mut material) in parts.iter_mut() {
        let Some(point) = &shown[piece.k] else {
            *visible = Visibility::Hidden;
            continue;
        };
        let hill = point.hill;
        let at = Vec3::new(
            hill.at.x.to_f32_for_render(),
            hill.at.y.to_f32_for_render(),
            hill.at.z.to_f32_for_render(),
        );
        let r = hill.radius.to_f32_for_render();
        let owner = if point.owner == NOBODY {
            0
        } else {
            1 + point.owner as usize % 2
        };
        let (wanted, scale, lift) = match piece.part {
            Part::Disc => (&inks.disc[owner], r, LIFT),
            Part::Rim => (&inks.rim[owner], r, LIFT),
            Part::Fill => {
                if point.capper == NOBODY || point.percent == 0 {
                    *visible = Visibility::Hidden;
                    continue;
                }
                let share = point.percent as f32 / 100.0;
                (&inks.fill[point.capper as usize % 2], r * share, FILL_LIFT)
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
