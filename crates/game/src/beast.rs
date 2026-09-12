//! Drawing the Ridgeback.
//!
//! One box per part, placed from the same `stance` and `shape` the simulation
//! collides against. Nothing here rebuilds the creature's geometry: if you can
//! see a box, that box is what stops you walking through it and what your
//! attacks are hitting.
//!
//! The ridge is a different colour from everything else on purpose. It is the
//! one place on the animal worth hitting, and a player should be able to see
//! that from across the arena without being told.

use bevy::prelude::*;
use sim::monster::{self, Doing, Monster};

/// One drawable part. A fixed pool, spawned once: the creature comes and goes
/// with the match, and spawning meshes when it does would put allocation on a
/// path the rollback re-runs.
#[derive(Component)]
pub struct Limb(pub usize);

/// Materials, made once. Which one a limb wears changes when it breaks.
#[derive(Resource)]
pub struct Hide {
    armour: Handle<StandardMaterial>,
    ridge: Handle<StandardMaterial>,
    limb: Handle<StandardMaterial>,
    broken: Handle<StandardMaterial>,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let hide = Hide {
        armour: materials.add(StandardMaterial {
            base_color: Color::srgb(0.24, 0.27, 0.23),
            perceptual_roughness: 0.92,
            ..default()
        }),
        // The weak point, and it says so.
        ridge: materials.add(StandardMaterial {
            base_color: Color::srgb(0.86, 0.32, 0.22),
            emissive: LinearRgba::rgb(0.55, 0.08, 0.04),
            perceptual_roughness: 0.7,
            ..default()
        }),
        limb: materials.add(StandardMaterial {
            base_color: Color::srgb(0.32, 0.34, 0.29),
            perceptual_roughness: 0.9,
            ..default()
        }),
        broken: materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.13, 0.13),
            perceptual_roughness: 0.98,
            ..default()
        }),
    };
    // A unit cube, scaled per part. The part boxes are axis-aligned in the
    // creature's own frame, so one mesh covers all of them.
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    for index in 0..monster::PARTS {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(hide.armour.clone()),
            Transform::default(),
            Visibility::Hidden,
            Limb(index),
        ));
    }
    commands.insert_resource(hide);
}

/// Put every part where the simulation says it is.
pub fn place(
    sim: Res<crate::Sim>,
    hide: Res<Hide>,
    mut limbs: Query<(
        &Limb,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let Some(beast) = sim.cur.monster else {
        for (_, _, mut visible, _) in limbs.iter_mut() {
            *visible = Visibility::Hidden;
        }
        return;
    };
    let stance = beast.stance();
    for (limb, mut transform, mut visible, mut material) in limbs.iter_mut() {
        let index = limb.0;
        let shape = monster::shape(index);
        let rides = monster::SHAPES[index].rides;
        let mid = shape.min.add(shape.max).scale(sim::Fx::ratio(1, 2));
        let size = shape.max.sub(shape.min);

        // The part's frame, read off the transform itself rather than
        // reconstructed from angles. The articulation is affine, so stepping
        // one unit along each body axis and subtracting gives the basis --
        // which means the tail's swing comes out for free and there is no
        // handedness convention here to get wrong.
        let at = |offset: sim::V3| fx3(stance.to_world(stance.articulate(rides, mid.add(offset))));
        let origin = at(sim::V3::ZERO);
        let x = at(sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO)) - origin;
        let y = at(sim::V3::new(sim::Fx::ZERO, sim::Fx::ONE, sim::Fx::ZERO)) - origin;

        *transform = Transform {
            translation: origin,
            rotation: orient(x, y),
            scale: fx3(size),
        };
        *visible = Visibility::Inherited;

        let wanted = if index == monster::RIDGE {
            &hide.ridge
        } else if beast.broken(index) {
            &hide.broken
        } else if monster::SHAPES[index].breakable {
            &hide.limb
        } else {
            &hide.armour
        };
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }
}

/// A rotation from two nearly-orthogonal axes.
///
/// Re-orthogonalised rather than trusted: the axes come through the fixed-point
/// sine table and are a fraction of a degree off square, which a quaternion
/// built straight from them turns into a visible shear.
fn orient(x: Vec3, y: Vec3) -> Quat {
    let x = x.normalize_or_zero();
    let y = (y - x * x.dot(y)).normalize_or_zero();
    if x == Vec3::ZERO || y == Vec3::ZERO {
        return Quat::IDENTITY;
    }
    Quat::from_mat3(&Mat3::from_cols(x, y, x.cross(y)))
}

fn fx3(v: sim::V3) -> Vec3 {
    Vec3::new(
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    )
}

// ---------------------------------------------------------------------------
// The overlay
// ---------------------------------------------------------------------------

/// The creature's live attack volume, and where you can stand on it.
///
/// Both come from the simulation's own functions. An overlay that rebuilds the
/// shape it illustrates is worse than none: it is confidently wrong at exactly
/// the moment you are using it to work out why something missed.
pub fn overlay(show: Res<crate::debug::ShowDebug>, sim: Res<crate::Sim>, mut gizmos: Gizmos) {
    if !show.0 {
        return;
    }
    let Some(beast) = sim.cur.monster else {
        return;
    };
    let stance = beast.stance();

    // Mountable tops, in the creature's frame, so it is obvious where the
    // climb goes and where it dead-ends.
    for index in 0..monster::PARTS {
        if !monster::SHAPES[index].mountable {
            continue;
        }
        let shape = monster::shape(index);
        let rides = monster::SHAPES[index].rides;
        let corner = |x: sim::Fx, z: sim::Fx| {
            fx3(stance.to_world(stance.articulate(rides, sim::V3::new(x, shape.max.y, z))))
        };
        let quad = [
            corner(shape.min.x, shape.min.z),
            corner(shape.max.x, shape.min.z),
            corner(shape.max.x, shape.max.z),
            corner(shape.min.x, shape.max.z),
        ];
        for i in 0..4 {
            gizmos.line(quad[i], quad[(i + 1) % 4], MOUNTABLE);
        }
    }

    // The attack out this frame -- a vertical cylinder in body space, which is
    // what the hit test compares against. Drawn as two rings and a spine, so
    // the height band is visible: unlike the fighters' own attacks, this one
    // genuinely cares how high you are standing.
    if let Some((anchor, radius, low, high)) = beast.hit_volume() {
        let spent = beast.hit_used;
        let colour = if spent { HIT_SPENT } else { HIT };
        let ring = |gizmos: &mut Gizmos, y: sim::Fx| {
            let centre = fx3(stance.to_world(sim::V3::new(anchor.x, y, anchor.z)));
            gizmos.circle(
                Isometry3d::new(centre, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                radius.to_f32_for_render(),
                colour,
            );
            centre
        };
        let bottom = ring(&mut gizmos, low);
        let top = ring(&mut gizmos, high);
        gizmos.line(bottom, top, colour);
    }

    // What it is doing, as a line off the nose: long for a committed move,
    // short while it is only looking.
    let nose = fx3(stance.to_world(sim::V3::new(
        monster::shape(monster::HEAD).max.x,
        monster::shape(monster::HEAD).max.y,
        sim::Fx::ZERO,
    )));
    let ahead = fx3(stance.dir_to_world(sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO)));
    let length = match beast.doing {
        Doing::Startup { .. } => 2.5,
        Doing::Active { .. } => 4.0,
        _ => 1.0,
    };
    gizmos.line(nose, nose + ahead * length, intent_colour(&beast));
}

const MOUNTABLE: Color = Color::srgb(0.38, 0.85, 0.62);
const HIT: Color = Color::srgb(1.0, 0.23, 0.31);
const HIT_SPENT: Color = Color::srgb(0.55, 0.16, 0.20);
const WINDUP: Color = Color::srgb(1.0, 0.78, 0.25);
const DOWN: Color = Color::srgb(0.45, 0.62, 1.0);
const CALM: Color = Color::srgb(0.62, 0.66, 0.72);

fn intent_colour(beast: &Monster) -> Color {
    match beast.doing {
        Doing::Startup { .. } => WINDUP,
        Doing::Active { .. } => HIT,
        Doing::Toppled { .. } | Doing::Flinch { .. } => DOWN,
        _ => CALM,
    }
}
