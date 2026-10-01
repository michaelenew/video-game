//! **A fight's own floor signs** (`sim::sign`): what a species draws besides
//! its bodies' telegraphs -- the Hornback's stampede lane and the lees cut out
//! of it, the ring on the rock a charge will stop at, the guard's *no*, a
//! cow's rear wedge under whoever stands in it.
//!
//! Drawn from `World::signs`, the list a test holds the herd to, in the same
//! drawn-over-everything material the telegraphs use (`beast::MarkMaterial`):
//! a lane on the floor is projected onto whatever you stand on. A fixed pool,
//! spawned once.

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use sim::sign::{MAX_SIGNS, Says, Shape};

use crate::beast::{MarkMaterial, OnTop};

/// Which sign a mesh draws, and as which shape.
#[derive(Component, Clone, Copy)]
pub struct Drawn(pub usize, pub Shape);

/// A sign's fill, drawn over it while it fills.
#[derive(Component, Clone, Copy)]
pub struct Fill(pub usize);

/// One material per thing a sign can say.
#[derive(Resource)]
pub struct Inks {
    coming: Handle<MarkMaterial>,
    fill: Handle<MarkMaterial>,
    live: Handle<MarkMaterial>,
    clear: Handle<MarkMaterial>,
    no: Handle<MarkMaterial>,
    stops: Handle<MarkMaterial>,
    faint: Handle<MarkMaterial>,
}

impl Inks {
    fn of(&self, says: Says) -> &Handle<MarkMaterial> {
        match says {
            Says::Coming => &self.coming,
            Says::Live => &self.live,
            Says::Clear => &self.clear,
            Says::No => &self.no,
            Says::Stops => &self.stops,
            Says::Faint => &self.faint,
        }
    }
}

/// How far above the floor a sign lies; a clear one a little above the lane
/// it is cut out of, so it reads as a hole in it.
const FLOOR: f32 = 0.03;
const ABOVE: f32 = 0.05;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut marks: ResMut<Assets<MarkMaterial>>,
) {
    let mut tint = |c: Color| {
        marks.add(MarkMaterial {
            base: StandardMaterial {
                base_color: c,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            },
            extension: OnTop {},
        })
    };
    let inks = Inks {
        coming: tint(Color::srgba(1.0, 0.72, 0.22, 0.16)),
        fill: tint(Color::srgba(1.0, 0.55, 0.12, 0.30)),
        live: tint(Color::srgba(1.0, 0.16, 0.18, 0.40)),
        // A lee is safe ground in a lane: drawn in the opposite of the hit's
        // colours, cool and plain.
        clear: tint(Color::srgba(0.35, 0.9, 0.75, 0.45)),
        // A guard says *no*, not *hit*: blue-grey, the shield's.
        no: tint(Color::srgba(0.55, 0.7, 1.0, 0.45)),
        stops: tint(Color::srgba(1.0, 0.85, 0.3, 0.7)),
        faint: tint(Color::srgba(1.0, 0.6, 0.3, 0.12)),
    };
    let strip = meshes.add(Rectangle::new(1.0, 1.0));
    let disc = meshes.add(Circle::new(0.5));
    let ring = meshes.add(Annulus::new(0.42, 0.5));
    for i in 0..MAX_SIGNS {
        for (shape, mesh) in [
            (Shape::Strip, &strip),
            (Shape::Disc, &disc),
            (Shape::Ring, &ring),
        ] {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(inks.coming.clone()),
                Transform::default(),
                Visibility::Hidden,
                NotShadowCaster,
                Drawn(i, shape),
            ));
        }
        commands.spawn((
            Mesh3d(strip.clone()),
            MeshMaterial3d(inks.fill.clone()),
            Transform::default(),
            Visibility::Hidden,
            NotShadowCaster,
            Fill(i),
        ));
    }
    commands.insert_resource(inks);
}

pub fn place(
    sim: Res<crate::Sim>,
    inks: Res<Inks>,
    mut drawn: Query<
        (
            &Drawn,
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<MarkMaterial>,
        ),
        Without<Fill>,
    >,
    mut fills: Query<(&Fill, &mut Transform, &mut Visibility), Without<Drawn>>,
) {
    let signs = sim.cur.signs();
    let list: Vec<&sim::sign::Sign> = signs.iter().collect();
    let flat = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    let v3 = |v: sim::V3| {
        Vec3::new(
            v.x.to_f32_for_render(),
            v.y.to_f32_for_render(),
            v.z.to_f32_for_render(),
        )
    };
    for (d, mut transform, mut visible, mut material) in drawn.iter_mut() {
        let Some(sign) = list.get(d.0).filter(|s| s.shape == d.1) else {
            *visible = Visibility::Hidden;
            continue;
        };
        let at = v3(sign.at);
        let along = v3(sign.along).with_y(0.0).normalize_or_zero();
        let length = sign.length.to_f32_for_render();
        let width = sign.width.to_f32_for_render();
        let lift = if sign.says == Says::Clear {
            ABOVE
        } else {
            FLOOR
        };
        *transform = match sign.shape {
            Shape::Strip => Transform {
                translation: Vec3::new(at.x, lift, at.z) + along * (length * 0.5),
                rotation: Quat::from_rotation_y(-along.z.atan2(along.x)) * flat,
                scale: Vec3::new(length.max(0.01), width.max(0.01), 1.0),
            },
            Shape::Disc | Shape::Ring => Transform {
                translation: Vec3::new(at.x, lift, at.z),
                rotation: flat,
                scale: Vec3::new(width, width, 1.0),
            },
        };
        *visible = Visibility::Inherited;
        let wanted = inks.of(sign.says);
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }
    // A coming strip fills along its length as what it warns of nears.
    for (f, mut transform, mut visible) in fills.iter_mut() {
        let Some(sign) = list
            .get(f.0)
            .filter(|s| s.shape == Shape::Strip && s.says == Says::Coming)
        else {
            *visible = Visibility::Hidden;
            continue;
        };
        let at = v3(sign.at);
        let along = v3(sign.along).with_y(0.0).normalize_or_zero();
        let length =
            sign.length.to_f32_for_render() * sign.progress.to_f32_for_render().clamp(0.0, 1.0);
        let width = sign.width.to_f32_for_render();
        *transform = Transform {
            translation: Vec3::new(at.x, FLOOR + 0.01, at.z) + along * (length * 0.5),
            rotation: Quat::from_rotation_y(-along.z.atan2(along.x)) * flat,
            scale: Vec3::new(length.max(0.01), width.max(0.01), 1.0),
        };
        *visible = Visibility::Inherited;
    }
}
