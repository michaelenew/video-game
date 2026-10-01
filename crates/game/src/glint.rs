//! **The windup glint** (The Pair §6): while a creature that is off your
//! screen winds up a move at you, a short bar glows at the screen's edge
//! nearest it, and goes the frame the hit is out.
//!
//! It says *something is coming from there*, never *something is there*: a
//! creature circling behind you shows nothing until it commits. It stands in
//! for the snarl the game would play if it had audio, and it is read from
//! `Monster::telegraph` -- the same thing the floor marker is drawn from -- so
//! it is on exactly while there is a windup to answer, and cannot drift from
//! the move.
//!
//! Every creature gets it, not only the cats: a windup you cannot see is the
//! same problem whoever throws it, and a creature that never leaves the screen
//! never shows one.

use bevy::prelude::*;
use sim::monster::MAX_MONSTERS;

/// One glint per creature slot.
#[derive(Component)]
pub struct Glint(pub usize);

/// The bar: long along the edge, thin across it, and a gap from the edge so
/// it reads as a light rather than a border.
const LONG: f32 = 120.0;
const THIN: f32 = 6.0;
const INSET: f32 = 10.0;
const GLOW: Color = Color::srgba(1.0, 0.82, 0.45, 0.9);

pub fn setup(mut commands: Commands) {
    for slot in 0..MAX_MONSTERS {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(LONG),
                height: Val::Px(THIN),
                ..default()
            },
            BackgroundColor(GLOW),
            BorderRadius::all(Val::Px(THIN / 2.0)),
            Visibility::Hidden,
            Glint(slot),
        ));
    }
}

/// Where on a `size` screen the bar goes for something at `toward` -- the
/// direction to it in the camera's own frame, x right, y up, z out of the
/// screen behind the viewer -- or `None` if it is on the screen.
///
/// Returns the bar's top-left corner and whether it lies along a side
/// (vertical) rather than the top or bottom.
pub fn place(size: Vec2, toward: Vec3, on_screen: bool) -> Option<(Vec2, bool)> {
    if on_screen {
        return None;
    }
    // Behind the camera, what is left of it is still left: the bar goes to
    // the edge of the side it is on, and to the bottom when it is dead behind.
    let mut dir = Vec2::new(toward.x, -toward.y);
    if toward.z > 0.0 && dir.length_squared() < 1e-6 {
        dir = Vec2::new(0.0, 1.0);
    }
    if toward.z > 0.0 {
        // Behind: push it out to the edge it is nearest, never into the middle.
        dir.y = dir.y.max(0.0);
    }
    let dir = dir.normalize_or(Vec2::new(0.0, 1.0));
    let half = size * 0.5;
    // Out along `dir` from the middle until it meets the screen's edge.
    let sx = if dir.x.abs() > 1e-6 {
        half.x / dir.x.abs()
    } else {
        f32::MAX
    };
    let sy = if dir.y.abs() > 1e-6 {
        half.y / dir.y.abs()
    } else {
        f32::MAX
    };
    let side = sx < sy;
    let edge = half + dir * sx.min(sy);
    let (w, h) = if side { (THIN, LONG) } else { (LONG, THIN) };
    let x = (edge.x - w * 0.5).clamp(INSET, size.x - w - INSET);
    let y = (edge.y - h * 0.5).clamp(INSET, size.y - h - INSET);
    Some((Vec2::new(x, y), side))
}

type CamQuery<'w, 's> =
    Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<crate::MainCamera>>;

pub fn update(
    sim: Res<crate::Sim>,
    cam: CamQuery,
    mut glints: Query<(&Glint, &mut Node, &mut Visibility)>,
) {
    let Ok((camera, eye)) = cam.single() else {
        return;
    };
    let Some(size) = camera.logical_viewport_size() else {
        return;
    };
    let me = sim.local_player();
    for (glint, mut node, mut visible) in glints.iter_mut() {
        *visible = Visibility::Hidden;
        let Some(beast) = sim.cur.monsters[glint.0] else {
            continue;
        };
        if !beast.alive() || beast.brain.target as usize != me {
            continue;
        }
        // A windup with a hit in it, and not yet out.
        let Some(t) = beast.telegraph() else {
            continue;
        };
        if t.live || beast.sp().attack(t.kind).damage <= 0 {
            continue;
        }
        let body = Vec3::new(
            beast.pos.x.to_f32_for_render(),
            beast.pos.y.to_f32_for_render() + 1.0,
            beast.pos.z.to_f32_for_render(),
        );
        let on_screen = camera
            .world_to_viewport(eye, body)
            .is_ok_and(|p| p.x >= 0.0 && p.y >= 0.0 && p.x <= size.x && p.y <= size.y);
        let toward = eye.affine().inverse().transform_point3(body);
        let Some((at, side)) = place(size, toward, on_screen) else {
            continue;
        };
        node.left = Val::Px(at.x);
        node.top = Val::Px(at.y);
        node.width = Val::Px(if side { THIN } else { LONG });
        node.height = Val::Px(if side { LONG } else { THIN });
        *visible = Visibility::Inherited;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Vec2 = Vec2::new(1280.0, 720.0);

    #[test]
    fn nothing_glints_for_a_windup_on_screen() {
        assert!(place(SCREEN, Vec3::new(0.0, 0.0, -5.0), true).is_none());
    }

    #[test]
    fn a_windup_to_the_left_glints_on_the_left_edge() {
        let (at, side) = place(SCREEN, Vec3::new(-10.0, 0.0, -1.0), false).unwrap();
        assert!(side, "a side edge");
        assert!(at.x < SCREEN.x * 0.1, "the left one: {at:?}");
    }

    #[test]
    fn a_windup_dead_behind_glints_at_the_bottom() {
        let (at, side) = place(SCREEN, Vec3::new(0.0, 0.0, 5.0), false).unwrap();
        assert!(!side);
        assert!(at.y > SCREEN.y * 0.9, "the bottom: {at:?}");
    }

    #[test]
    fn behind_and_to_the_right_glints_on_the_right_never_in_the_middle() {
        let (at, _) = place(SCREEN, Vec3::new(3.0, 1.0, 5.0), false).unwrap();
        assert!(at.x > SCREEN.x * 0.5, "the right: {at:?}");
        let middle = (at - SCREEN * 0.5).abs();
        assert!(middle.x > 200.0 || middle.y > 200.0, "at an edge: {at:?}");
    }
}
