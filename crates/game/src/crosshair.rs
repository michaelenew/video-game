//! The crosshair.
//!
//! It marks **where your attack will go**, which is not always the middle of
//! the screen.
//!
//! Facing locks the instant a move starts, and turns at a limited rate while
//! guarding (see `controls.md`). During those frames the camera still goes
//! wherever the mouse goes, but the fighter does not -- so a reticle painted at
//! screen centre would be telling you a lie exactly when the answer matters.
//!
//! So it is placed by projecting the point the fighter is actually pointed at
//! back onto the screen. When facing tracks aim, which is most of the time,
//! that lands dead centre and the crosshair sits still. When facing is locked
//! or lagging, it slides off to the side the attack is really going, and you
//! can see your commitment.
//!
//! Colour carries the other half: bright when you can act, dim when you are
//! committed to something and the button will not answer.

use bevy::prelude::*;
use view::interpolate;

const LIVE: Color = Color::srgba(0.98, 0.99, 1.0, 0.95);
const COMMITTED: Color = Color::srgba(0.60, 0.66, 0.76, 0.65);
/// Drawn behind the mark so it stays visible against a pale wall as well as a
/// dark one.
const SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);

#[derive(Component)]
pub struct Crosshair;

#[derive(Component)]
pub struct CrosshairInk;

/// Container size. Everything below is placed relative to its centre.
const SIZE: f32 = 30.0;
const MID: f32 = SIZE / 2.0;
/// Tick geometry: how far the gap runs from centre, and how long and thick the
/// marks are. A gap in the middle matters -- a solid cross hides the one pixel
/// you are trying to aim at.
const GAP: f32 = 5.0;
const LEN: f32 = 7.0;
const THICK: f32 = 2.0;
const DOT: f32 = 3.0;

/// One mark, drawn twice: a dark copy a pixel larger underneath, then the
/// bright one on top. Without the dark copy the reticle disappears against a
/// pale wall, and this arena has both pale walls and near-black ones.
fn mark(c: &mut ChildSpawnerCommands, left: f32, top: f32, w: f32, h: f32) {
    c.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(left - 1.0),
            top: Val::Px(top - 1.0),
            width: Val::Px(w + 2.0),
            height: Val::Px(h + 2.0),
            ..default()
        },
        BackgroundColor(SHADOW),
        BorderRadius::all(Val::Px(1.0)),
    ));
    c.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(left),
            top: Val::Px(top),
            width: Val::Px(w),
            height: Val::Px(h),
            ..default()
        },
        BackgroundColor(LIVE),
        BorderRadius::all(Val::Px(1.0)),
        CrosshairInk,
    ));
}

pub fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(SIZE),
                height: Val::Px(SIZE),
                ..default()
            },
            // Off-screen until the first frame places it, so it never flashes
            // in the corner on startup.
            Visibility::Hidden,
            Crosshair,
        ))
        .with_children(|c| {
            // Four ticks and a centre dot. Ticks rather than a ring: at this
            // size a ring reads as a smudge, and four marks give the eye
            // something to centre between.
            mark(c, MID - THICK / 2.0, MID - GAP - LEN, THICK, LEN); // up
            mark(c, MID - THICK / 2.0, MID + GAP, THICK, LEN); // down
            mark(c, MID - GAP - LEN, MID - THICK / 2.0, LEN, THICK); // left
            mark(c, MID + GAP, MID - THICK / 2.0, LEN, THICK); // right
            mark(c, MID - DOT / 2.0, MID - DOT / 2.0, DOT, DOT);
        });
}

/// The world point the fighter is actually pointed at.
///
/// **The camera's own centre ray, turned by however far the fighter's facing
/// lags the camera's.** Built that way rather than from a distance ahead of the
/// fighter, because screen centre now follows the full look direction: an aim
/// point constructed independently would have to re-derive the pitch, the eye
/// lift and the shoulder offset, and the first time one of those changed the
/// reticle would start lying. Turning the ray the camera is already using
/// cannot drift from it.
///
/// Facing is yaw only -- pitch is renderer-local and attacks are flat -- so the
/// difference between the two is a rotation about Y and nothing else.
fn aim_point(eye: Vec3, forward: Vec3, facing: [f32; 3]) -> Vec3 {
    const FAR: f32 = 64.0;
    let camera_yaw = forward.z.atan2(forward.x);
    let facing_yaw = facing[2].atan2(facing[0]);
    let turn = facing_yaw - camera_yaw;
    let (sin, cos) = turn.sin_cos();
    let turned = Vec3::new(
        forward.x * cos - forward.z * sin,
        forward.y,
        forward.x * sin + forward.z * cos,
    );
    eye + turned * FAR
}

type CamQuery<'w, 's> =
    Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<crate::MainCamera>>;

pub fn update(
    sim: Res<crate::Sim>,
    cam: CamQuery,
    mut cross: Query<(&mut Node, &mut Visibility), With<Crosshair>>,
    mut ink: Query<&mut BackgroundColor, With<CrosshairInk>>,
) {
    let Ok((camera, cam_tf)) = cam.single() else {
        return;
    };
    let Ok((mut node, mut visible)) = cross.single_mut() else {
        return;
    };

    let frame = interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    let me = sim.local_player();
    let p = &frame.players[me];

    let target = aim_point(cam_tf.translation(), cam_tf.forward().as_vec3(), p.facing);

    match camera.world_to_viewport(cam_tf, target) {
        Ok(screen) => {
            node.left = Val::Px(screen.x - SIZE * 0.5);
            node.top = Val::Px(screen.y - SIZE * 0.5);
            *visible = Visibility::Inherited;
        }
        // Behind the camera or otherwise unprojectable. Hiding beats drawing a
        // mark in a place that means nothing.
        Err(_) => *visible = Visibility::Hidden,
    }

    let want = if p.action.actionable() {
        LIVE
    } else {
        COMMITTED
    };
    for mut colour in ink.iter_mut() {
        colour.0 = want;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use view::CameraRig;
    use view::camera::RigConfig;

    #[test]
    fn the_reticle_is_centred_exactly_when_facing_matches_aim() {
        // The contract, checked against the camera itself rather than against a
        // restatement of it. If the rig's aim point and the crosshair's ever
        // drift apart, a still crosshair stops meaning anything.
        let cfg = RigConfig::default();
        for eighth in 0..8 {
            for pitch in [-0.9f32, -0.3, 0.0, 0.5, 1.2] {
                let yaw = eighth as f32 / 8.0 * std::f32::consts::TAU;
                let pos = [1.5, 0.0, -2.0];
                let facing = [yaw.cos(), 0.0, yaw.sin()];

                let mut rig = CameraRig::new(cfg);
                let framing = rig.update(0.016, pos, yaw, pitch);
                let eye = Vec3::from_array(framing.eye);
                let forward = (Vec3::from_array(framing.look_at) - eye).normalize();
                let point = aim_point(eye, forward, facing);

                let centre = eye + forward * 64.0;
                assert!(
                    point.distance(centre) < 0.05,
                    "at yaw {yaw:.2} pitch {pitch:.2}: crosshair {point:?}, centre {centre:?}"
                );
            }
        }
    }

    #[test]
    fn a_locked_facing_moves_the_reticle_off_centre() {
        // The case the whole thing exists for: mid-move the camera keeps
        // turning and the fighter does not, so the reticle has to leave the
        // middle of the screen rather than keep promising a hit it cannot land.
        let cfg = RigConfig::default();
        let pos = [0.0, 0.0, 0.0];
        let mut rig = CameraRig::new(cfg);
        // Looking a quarter turn away from where the fighter is committed.
        let framing = rig.update(0.016, pos, std::f32::consts::FRAC_PI_2, 0.0);
        let eye = Vec3::from_array(framing.eye);
        let forward = (Vec3::from_array(framing.look_at) - eye).normalize();
        let committed = aim_point(eye, forward, [1.0, 0.0, 0.0]);
        let centre = eye + forward * 64.0;
        assert!(
            committed.distance(centre) > 1.0,
            "reticle stayed put while facing and aim disagreed"
        );
    }
}
