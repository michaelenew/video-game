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
/// Deliberately the same distance ahead that the camera aims at, so the two
/// coincide exactly when facing and aim agree -- which is what makes a reticle
/// that sits still mean "your attack goes where you are looking" and one that
/// drifts mean "it does not".
fn aim_point(pos: [f32; 3], facing: [f32; 3], cfg: &view::camera::RigConfig) -> Vec3 {
    Vec3::new(
        pos[0] + facing[0] * cfg.look_ahead,
        pos[1] + cfg.look_height,
        pos[2] + facing[2] * cfg.look_ahead,
    )
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

    let target = aim_point(p.pos, p.facing, &view::camera::RigConfig::default());

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
            let yaw = eighth as f32 / 8.0 * std::f32::consts::TAU;
            let pos = [1.5, 0.0, -2.0];
            let facing = [yaw.cos(), 0.0, yaw.sin()];

            let mut rig = CameraRig::new(cfg);
            let framing = rig.update(0.016, pos, yaw, 0.0);
            let point = aim_point(pos, facing, &cfg);

            let look = Vec3::from_array(framing.look_at);
            assert!(
                point.distance(look) < 0.01,
                "at yaw {yaw:.2}: crosshair at {point:?}, screen centre at {look:?}"
            );
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
        let committed = aim_point(pos, [1.0, 0.0, 0.0], &cfg);
        let centre = Vec3::from_array(framing.look_at);
        assert!(
            committed.distance(centre) > 1.0,
            "reticle stayed put while facing and aim disagreed"
        );
    }
}
