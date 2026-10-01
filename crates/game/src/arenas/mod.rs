//! How each arena is drawn: its floor, its solids and its dressing.
//!
//! **The geometry is the simulation's.** Every solid is drawn from
//! `sim::arena::Arena::solids`, the same boxes bodies collide against, and the
//! floor from its bounds and material regions -- if you can see it, you
//! collide with it, and a region you can see is the region the simulation
//! answers for. What an arena adds here is presentation only: [`Dressing`],
//! things to look at that nothing collides with (bones in the sand, reeds, a
//! banner), and the colour of its sky.
//!
//! Every planned creature's arena already has a line in [`dressing`],
//! commented out and one blank line from the next, so two branches each adding
//! an arena do not conflict (the same arrangement as `crate::species`).
//!
//! The scenery is rebuilt when the arena changes -- the picker, a reset into
//! another one -- and not otherwise. That is the one time this spends meshes,
//! and it is on the frame the fight starts over anyway.

use bevy::prelude::*;
use sim::arena::{Area, ArenaId, Material};

pub mod proving_ground;

pub mod range;

pub mod gnawers;

pub mod hornback;

pub mod mireback;

pub mod sandmaw;

// pub mod pair;

// pub mod broodmother;

// pub mod veilstalker;

// pub mod mantis;

// pub mod galewing;

// pub mod siegeshell;

/// One thing to look at that nothing collides with.
#[derive(Clone, Copy, Debug)]
pub struct Prop {
    pub shape: Shape,
    /// The middle of its base, in metres.
    pub at: [f32; 3],
    /// Width, height, depth: a box's sides, a cylinder's diameter and height,
    /// a sphere's diameter.
    pub size: [f32; 3],
    /// Turned about the vertical, in turns.
    pub yaw: f32,
    pub rgb: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Box,
    Cylinder,
    Sphere,
}

/// What an arena adds to its geometry, for the eye only.
#[derive(Clone, Copy, Debug)]
pub struct Dressing {
    /// The colour behind everything.
    pub sky: [f32; 3],
    pub props: &'static [Prop],
}

/// An arena's dressing. One with none is drawn bare under the proving
/// ground's sky, which is plain but honest: the geometry is all there.
pub fn dressing(id: ArenaId) -> &'static Dressing {
    match id {
        ArenaId::RANGE => &range::DRESSING,

        ArenaId::GNAWERS => &gnawers::DRESSING,

        ArenaId::HORNBACK => &hornback::DRESSING,

        ArenaId::HORNBACK_CROSSING => &hornback::CROSSING,

        ArenaId::MIREBACK => &mireback::DRESSING,

        ArenaId::SANDMAW => &sandmaw::DRESSING,

        // ArenaId::PAIR => &pair::DRESSING,

        // ArenaId::BROODMOTHER => &broodmother::DRESSING,

        // ArenaId::VEILSTALKER => &veilstalker::DRESSING,

        // ArenaId::MANTIS => &mantis::DRESSING,

        // ArenaId::GALEWING => &galewing::DRESSING,

        // ArenaId::SIEGESHELL => &siegeshell::DRESSING,
        _ => &proving_ground::DRESSING,
    }
}

/// What a material looks like. One colour each, for every arena: sand is the
/// same sand wherever it is, and an arena that wants its own wants a new
/// material in the simulation, because a floor the creature reads differently
/// should look different.
pub fn colour(material: Material) -> [f32; 3] {
    match material {
        // The proving ground's floor and walls, as they always were.
        Material::Ground => [0.13, 0.15, 0.18],
        Material::Stone => [0.30, 0.34, 0.40],
        Material::Grass => [0.20, 0.30, 0.16],
        Material::Rock => [0.36, 0.33, 0.30],
        Material::Sand => [0.62, 0.55, 0.40],
        Material::Snow => [0.82, 0.85, 0.90],
        Material::Ash => [0.32, 0.31, 0.31],
        Material::Peat => [0.17, 0.13, 0.09],
        Material::Water => [0.16, 0.30, 0.42],
        Material::Wood => [0.33, 0.24, 0.16],
    }
}

/// Everything drawn for the arena, so all of it can go when the arena does.
#[derive(Component)]
pub struct Scenery;

/// Which arena is drawn, if any yet.
#[derive(Resource, Default)]
pub struct Drawn(Option<ArenaId>);

/// How far the floor runs past the bounds, so the edge of the world is not the
/// edge of the walls.
const APRON: f32 = 12.0;

/// Draw the arena the simulation is in, when it is not the one already drawn.
pub fn dress(
    mut commands: Commands,
    sim: Res<crate::Sim>,
    mut drawn: ResMut<Drawn>,
    old: Query<Entity, With<Scenery>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sky: ResMut<ClearColor>,
) {
    let arena = sim.cur.arena();
    if drawn.0 == Some(arena.id) {
        return;
    }
    drawn.0 = Some(arena.id);
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let dressing = dressing(arena.id);
    sky.0 = Color::srgb(dressing.sky[0], dressing.sky[1], dressing.sky[2]);

    let mut paint = |rgb: [f32; 3]| {
        materials.add(StandardMaterial {
            base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
            perceptual_roughness: 0.92,
            ..default()
        })
    };

    // The floor: its own material under everything, then each region on top,
    // a hair higher than the one before so a later region wins on screen the
    // way it wins in `Arena::floor_at`.
    let b = arena.bounds;
    let (lo_x, hi_x) = (fx(b.lo_x), fx(b.hi_x));
    let (lo_z, hi_z) = (fx(b.lo_z), fx(b.hi_z));
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(hi_x - lo_x + APRON * 2.0, hi_z - lo_z + APRON * 2.0),
            ),
        ),
        MeshMaterial3d(paint(colour(arena.floor))),
        Transform::from_xyz((lo_x + hi_x) * 0.5, 0.0, (lo_z + hi_z) * 0.5),
        Scenery,
    ));
    for (i, region) in arena.regions.iter().enumerate() {
        let lift = 0.004 * (i + 1) as f32;
        let look = paint(colour(region.material));
        match region.area {
            Area::Rect { lo, hi } => {
                let (x0, z0, x1, z1) = (fx(lo.0), fx(lo.1), fx(hi.0), fx(hi.1));
                commands.spawn((
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(x1 - x0, z1 - z0))),
                    MeshMaterial3d(look),
                    Transform::from_xyz((x0 + x1) * 0.5, lift, (z0 + z1) * 0.5),
                    Scenery,
                ));
            }
            Area::Disc { at, radius } => {
                commands.spawn((
                    Mesh3d(meshes.add(Circle::new(fx(radius)))),
                    MeshMaterial3d(look),
                    Transform::from_xyz(fx(at.0), lift, fx(at.1))
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    Scenery,
                ));
            }
        }
    }

    // The geometry, straight from the simulation's own collision data. One
    // source of truth: if you can see it, you collide with it.
    for solid in arena.solids() {
        let min = Vec3::new(fx(solid.min.x), fx(solid.min.y), fx(solid.min.z));
        let max = Vec3::new(fx(solid.max.x), fx(solid.max.y), fx(solid.max.z));
        let size = max - min;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(paint(colour(solid.material))),
            Transform::from_translation((min + max) * 0.5),
            Scenery,
        ));
    }

    for prop in dressing.props {
        let [w, h, d] = prop.size;
        let mesh = match prop.shape {
            Shape::Box => meshes.add(Cuboid::new(w, h, d)),
            Shape::Cylinder => meshes.add(Cylinder::new(w * 0.5, h)),
            Shape::Sphere => meshes.add(Sphere::new(w * 0.5)),
        };
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(paint(prop.rgb)),
            Transform::from_xyz(prop.at[0], prop.at[1] + h * 0.5, prop.at[2])
                .with_rotation(Quat::from_rotation_y(-prop.yaw * std::f32::consts::TAU)),
            Scenery,
        ));
    }
}

/// RENDER-ONLY. Fixed point to metres.
fn fx(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}
