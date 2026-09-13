//! The crosshair.
//!
//! It marks **where the next ability would land**, and it does that by never
//! moving: it sits at the exact centre of the screen, always, and the *camera*
//! is what turns to keep the aim point under it.
//!
//! That is the whole design, and the ordering matters. The alternative --
//! leaving the camera pointed along the raw look axis and sliding the reticle
//! to wherever the aim really lands -- would be equally honest and would feel
//! terrible. A reticle that wanders reads as the aim slipping out of your
//! hands, and the reticle is the one thing on screen a player is deliberately
//! holding still. So the parallax from the camera sitting above the fighter's
//! head goes into the view instead, where it is a few degrees of pitch nobody
//! has to fight.
//!
//! It follows that there is nothing to compute here. `sim::aim` solves where
//! the ability goes, `main::aim_point` hands that to the rig, and the rig
//! points the camera at it -- so the middle of the screen is the answer by
//! construction rather than by agreement between two pieces of code that could
//! drift apart.
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

/// Top-left corner of the reticle box, so that its middle is the middle of a
/// viewport this size.
fn centred_in(width: f32, height: f32) -> (f32, f32) {
    (width * 0.5 - SIZE * 0.5, height * 0.5 - SIZE * 0.5)
}

type CamQuery<'w, 's> =
    Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<crate::MainCamera>>;

pub fn update(
    sim: Res<crate::Sim>,
    cam: CamQuery,
    mut cross: Query<(&mut Node, &mut Visibility), With<Crosshair>>,
    mut ink: Query<&mut BackgroundColor, With<CrosshairInk>>,
) {
    let Ok((camera, _)) = cam.single() else {
        return;
    };
    let Ok((mut node, mut visible)) = cross.single_mut() else {
        return;
    };

    // Dead centre of the viewport, whatever its size. The camera has already
    // been pointed at the aim point, so this *is* the aim point.
    let Some(viewport) = camera.logical_viewport_size() else {
        *visible = Visibility::Hidden;
        return;
    };
    let (left, top) = centred_in(viewport.x, viewport.y);
    node.left = Val::Px(left);
    node.top = Val::Px(top);
    *visible = Visibility::Inherited;

    let frame = interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    let want = if frame.players[sim.local_player()].action.actionable() {
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

    #[test]
    fn the_reticle_is_centred_on_the_viewport() {
        // The contract, and the only thing this module decides any more: the
        // mark's own middle is the middle of the screen, at every size. Where
        // that middle *is* in the world is the camera's problem, and the camera
        // is pointed at `sim::aim`'s answer.
        for (w, h) in [(1280.0, 760.0), (800.0, 600.0), (3440.0, 1440.0)] {
            let (left, top) = centred_in(w, h);
            let middle = (left + SIZE * 0.5, top + SIZE * 0.5);
            assert!(
                (middle.0 - w * 0.5).abs() < 0.001 && (middle.1 - h * 0.5).abs() < 0.001,
                "at {w}x{h} the reticle's middle is {middle:?}, not ({}, {})",
                w * 0.5,
                h * 0.5
            );
        }
    }
}
