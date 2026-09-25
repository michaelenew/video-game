//! Drawing the Ridgeback.
//!
//! One box per part, placed from the same rig and the same boxes the
//! simulation collides against. Nothing here rebuilds the creature's geometry:
//! if you can see a box, that box is what stops you walking through it and what
//! your attacks are hitting.
//!
//! The parts are boxes and the creature is a skeleton, so consecutive segments
//! of a limb open a wedge between them whenever the joint bends. A **sphere at
//! each joint** closes it -- the classic capsule silhouette, built out of the
//! shapes that were already there. It is the one thing drawn that the
//! simulation does not collide against, and it is deliberately *inside* the
//! parts it joins, so it can never make the animal look bigger than it is.
//!
//! The two weak points are a different colour from everything else on purpose.
//! They are the only places on the animal worth hitting, and a player should be
//! able to see that from across the arena without being told.

use bevy::prelude::*;
use sim::beast;
use sim::monster::{self, Doing, Monster};

/// One drawable part. A fixed pool, spawned once: the creature comes and goes
/// with the match, and spawning meshes when it does would put allocation on a
/// path the rollback re-runs.
#[derive(Component)]
pub struct Limb(pub usize);

/// One drawable joint filler.
#[derive(Component)]
pub struct Knuckle(pub usize);

/// Materials, made once. Which one a part wears changes when it breaks.
#[derive(Resource)]
pub struct Hide {
    armour: Handle<StandardMaterial>,
    weak: Handle<StandardMaterial>,
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
        // The weak points, and they say so.
        weak: materials.add(StandardMaterial {
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
    // A unit cube, scaled per part. The part boxes are axis-aligned in their
    // own bone's frame, so one mesh covers all of them.
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
    let ball = meshes.add(Sphere::new(0.5).mesh().ico(2).unwrap());
    for bone in 0..beast::BONES {
        commands.spawn((
            Mesh3d(ball.clone()),
            MeshMaterial3d(hide.armour.clone()),
            Transform::default(),
            Visibility::Hidden,
            Knuckle(bone),
        ));
    }
    commands.insert_resource(hide);
}

/// Put every part where the simulation says it is.
pub fn place(
    sim: Res<crate::Sim>,
    hide: Res<Hide>,
    mut limbs: Query<
        (
            &Limb,
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        Without<Knuckle>,
    >,
    mut knuckles: Query<
        (
            &Knuckle,
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        Without<Limb>,
    >,
) {
    let Some(beast) = sim.cur.monster else {
        for (_, _, mut visible, _) in limbs.iter_mut() {
            *visible = Visibility::Hidden;
        }
        for (_, _, mut visible, _) in knuckles.iter_mut() {
            *visible = Visibility::Hidden;
        }
        return;
    };
    let rig = beast.rig();

    for (limb, mut transform, mut visible, mut material) in limbs.iter_mut() {
        let index = limb.0;
        let shape = monster::shape(index);
        let mid = shape.min.add(shape.max).scale(sim::Fx::ratio(1, 2));
        let size = shape.max.sub(shape.min);
        let frame = rig.of(index);

        // The bone's frame, read off the transform itself rather than
        // reconstructed from angles: step one unit along each axis and
        // subtract. There is no handedness convention here to get wrong.
        let at = |offset: sim::V3| fx3(frame.local_to_world(mid.add(offset)));
        let origin = at(sim::V3::ZERO);
        let x = at(sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO)) - origin;
        let y = at(sim::V3::new(sim::Fx::ZERO, sim::Fx::ONE, sim::Fx::ZERO)) - origin;

        *transform = Transform {
            translation: origin,
            rotation: orient(x, y),
            scale: fx3(size),
        };
        *visible = Visibility::Inherited;

        let wanted = skin(&hide, &beast, index);
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }

    // The joints. Each is sized to the thinner of the parts meeting there, so
    // it disappears inside them and only shows in the wedge a bend opens up.
    for (knuckle, mut transform, mut visible, mut material) in knuckles.iter_mut() {
        let bone = knuckle.0;
        let Some(width) = joint_width(bone) else {
            *visible = Visibility::Hidden;
            continue;
        };
        *transform = Transform {
            translation: fx3(rig.bone[bone].at),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(width),
        };
        *visible = Visibility::Inherited;
        let wanted = &hide.armour;
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }
}

/// How wide a filler at this bone should be, or `None` where nothing meets.
///
/// The narrower of the two parts hanging off the joint: a sphere the size of
/// the *wider* one would bulge out of the slimmer segment and read as a bead on
/// a string rather than as an elbow.
fn joint_width(bone: usize) -> Option<f32> {
    let mut narrowest: Option<sim::Fx> = None;
    for index in 0..monster::PARTS {
        let shape = monster::shape(index);
        if beast::SHAPES[index].bone != bone || !beast::SHAPES[index].solid {
            continue;
        }
        let size = shape.max.sub(shape.min);
        let thin = size.y.min(size.z);
        if narrowest.is_none_or(|seen| thin.raw() < seen.raw()) {
            narrowest = Some(thin);
        }
    }
    // The root and the chest carry the barrel; a ball there would sit inside a
    // box two and a half metres wide and cost a draw call for nothing.
    if matches!(bone, beast::ROOT | beast::SPINE | beast::CHEST) {
        return None;
    }
    narrowest.map(|w| w.to_f32_for_render() * 0.95)
}

fn skin<'a>(hide: &'a Hide, beast: &Monster, index: usize) -> &'a Handle<StandardMaterial> {
    if monster::is_weak_point(index) {
        &hide.weak
    } else if beast.broken(index) {
        &hide.broken
    } else if beast::SHAPES[index].breakable {
        &hide.limb
    } else {
        &hide.armour
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
    let rig = beast.rig();

    // Mountable tops, so it is obvious where the climb goes and where it dead
    // ends. Straight off the rig's own top-face corners.
    for index in 0..monster::PARTS {
        if !sim::beast::SHAPES[index].mountable {
            continue;
        }
        let quad = rig.top_face(index).map(fx3);
        for i in 0..4 {
            gizmos.line(quad[i], quad[(i + 1) % 4], MOUNTABLE);
        }
    }

    // The attack out this frame -- a vertical cylinder, which is what the hit
    // test compares against. Drawn as two rings and a spine, so the height band
    // is visible: unlike the fighters' own attacks, this one genuinely cares
    // how high you are standing.
    if let Some((anchor, radius, low, high)) = beast.hit_volume() {
        let spent = beast.hit_used;
        let colour = if spent { HIT_SPENT } else { HIT };
        let ring = |gizmos: &mut Gizmos, y: sim::Fx| {
            let centre = fx3(sim::V3::new(anchor.x, y, anchor.z));
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
    let head = monster::shape(monster::HEAD);
    let nose = fx3(rig.part_to_world(
        monster::HEAD,
        sim::V3::new(head.max.x, head.max.y, sim::Fx::ZERO),
    ));
    let ahead = fx3(rig.dir_to_world(sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO)));
    let length = match beast.doing {
        Doing::Startup { .. } => 2.5,
        Doing::Active { .. } => 4.0,
        _ => 1.0,
    };
    gizmos.line(nose, nose + ahead * length, intent_colour(&beast));
}

/// **The spikes in flight**, always drawn, not only with the overlay on.
///
/// The spray is the one move whose hit leaves the animal, so the animal's pose
/// cannot show where it is: without this the only thing a fighter at range
/// sees is a tail flicking, and then being rooted. Drawn from the simulation's
/// own hit volume, for the rule the overlay above follows -- a picture of a
/// hit that is not the hit is worse than none. A spread of streaks across the
/// volume's width at the height it catches you, each pointing the way the
/// volume is going; once it has caught somebody it stops being drawn, because
/// the simulation has stopped testing it.
pub fn spikes(sim: Res<crate::Sim>, mut gizmos: Gizmos) {
    let Some(beast) = sim.cur.monster else {
        return;
    };
    let Some(kind) = beast.doing.attacking() else {
        return;
    };
    if monster::attack(kind).travel.raw() <= 0 || beast.hit_used {
        return;
    }
    let Some((anchor, radius, low, high)) = beast.hit_volume() else {
        return;
    };
    let rig = beast.rig();
    let ahead = fx3(rig.dir_to_world(sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO)));
    let across = Vec3::new(-ahead.z, 0.0, ahead.x);
    let centre = fx3(anchor);
    let r = radius.to_f32_for_render();
    let (lo, hi) = (low.to_f32_for_render(), high.to_f32_for_render());
    for i in 0..SPIKES {
        // Spread across the width and up the height band, staggered along
        // the flight so they read as a volley rather than a wall.
        let u = i as f32 / (SPIKES - 1) as f32;
        let side = (u * 2.0 - 1.0) * r;
        let up = lo + (hi - lo) * (0.25 + 0.5 * ((i * 7 % SPIKES) as f32 / SPIKES as f32));
        let back = ((i * 3 % 5) as f32) * 0.25;
        let tip = Vec3::new(centre.x, up, centre.z) + across * side - ahead * back;
        gizmos.line(tip - ahead * SPIKE_LENGTH, tip, SPIKE);
    }
}

const SPIKES: usize = 9;
const SPIKE_LENGTH: f32 = 0.9;
const SPIKE: Color = Color::srgb(0.93, 0.86, 0.70);

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
        Doing::Toppled { .. } | Doing::Flinch { .. } | Doing::Stumble { .. } => DOWN,
        _ => CALM,
    }
}
