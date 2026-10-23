//! **Looking at a character up close** (F6, 2026-10-22).
//!
//! The fight's camera is built for fighting: it rides a sphere behind the
//! shoulder, is pulled in by walls, and takes your own body away as the eye
//! gets close to it (`view::camera`, `InsideOwnHead`), because a body filling
//! the screen is a body you cannot aim past. All of that is right in a fight
//! and all of it is in the way of looking at the model. This puts the camera
//! on an orbit round one fighter instead -- the mouse swings round them, the
//! wheel comes in and goes out, Alt and the wheel slide what it looks at up
//! and down the body -- with nobody faded and no crosshair.
//!
//! Only the drawn camera moves. The aim is the simulation's, worked out from
//! `Look` and the fighter's own eye, and `Look` is left exactly where it was,
//! so the fighter keeps playing -- walk, swing, and watch it from the side --
//! and P, `]` and `[` pause and step it, which is how a pose is walked round.
//! `SHOT_PORTRAIT` is the same idea fixed at launch, for screenshots.

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

/// Who the camera is looking at, if anyone, and from where.
#[derive(Resource, Default)]
pub struct Inspect {
    /// The fighter looked at. `None` is the fight's own camera.
    pub seat: Option<usize>,
    /// Round them about the vertical, radians; nought is from `+Z`.
    yaw: f32,
    /// Up from level, radians.
    pitch: f32,
    /// How far the eye is from the point it looks at, metres.
    distance: f32,
    /// How far above their feet that point is, metres.
    height: f32,
}

/// Where a fresh look starts: a little to one side of straight in front, a
/// little above, the whole body in frame. Presentation, not a rule.
const START_ROUND: f32 = 35.0;
const START_UP: f32 = 10.0;
const START_DISTANCE: f32 = 3.0;
const START_HEIGHT: f32 = 1.0;
/// As close as it comes: a face fills the screen, inside the camera's near
/// plane by a margin.
const NEAREST: f32 = 0.25;
const FURTHEST: f32 = 15.0;
/// How much one notch of the wheel brings it in.
const ZOOM_A_NOTCH: f32 = 0.88;
/// How far one notch with Alt slides the point looked at.
const RISE_A_NOTCH: f32 = 0.08;
const HIGHEST: f32 = 2.5;
/// Pixels of a trackpad's scroll to one notch of a wheel.
const PIXELS_A_NOTCH: f32 = 40.0;

/// F6 to the next fighter, and the mouse and wheel moving round them.
#[allow(clippy::too_many_arguments)] // a Bevy system: its arguments are its resources
pub fn steer(
    sim: Res<crate::Sim>,
    look: Res<crate::Look>,
    settings: Res<crate::settings::Settings>,
    keys: Res<ButtonInput<KeyCode>>,
    mut motion: EventReader<MouseMotion>,
    mut wheel: EventReader<MouseWheel>,
    mut inspect: ResMut<Inspect>,
) {
    if keys.just_pressed(KeyCode::F6) {
        let next = next_seat(inspect.seat, sim.local_player(), |i| sim.cur.is_present(i));
        if next != inspect.seat {
            if let Some(i) = next {
                // Turned to face them, whichever way they are facing.
                let f = sim.cur.players[i].facing;
                let facing = f.x.to_f32_for_render().atan2(f.z.to_f32_for_render());
                *inspect = Inspect {
                    seat: Some(i),
                    yaw: facing + START_ROUND.to_radians(),
                    pitch: START_UP.to_radians(),
                    distance: START_DISTANCE,
                    height: START_HEIGHT,
                };
            } else {
                inspect.seat = None;
            }
        }
    }
    if inspect.seat.is_none() || !look.grabbed {
        motion.clear();
        wheel.clear();
        return;
    }
    // The fight's own sensitivity, so the orbit turns as far for a sweep of
    // the mouse as the look does.
    let per_pixel = settings.radians_per_pixel();
    for ev in motion.read() {
        inspect.yaw -= ev.delta.x * per_pixel;
        // The mouse up looks up at them, which is the eye going down.
        inspect.pitch =
            (inspect.pitch + ev.delta.y * per_pixel).clamp(-85f32.to_radians(), 85f32.to_radians());
    }
    let alt = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
    for ev in wheel.read() {
        let notches = match ev.unit {
            MouseScrollUnit::Line => ev.y,
            MouseScrollUnit::Pixel => ev.y / PIXELS_A_NOTCH,
        };
        if alt {
            inspect.height = (inspect.height + notches * RISE_A_NOTCH).clamp(0.0, HIGHEST);
        } else {
            inspect.distance =
                (inspect.distance * ZOOM_A_NOTCH.powf(notches)).clamp(NEAREST, FURTHEST);
        }
    }
}

/// After `current`: you first, then everyone else present in seat order, then
/// back to the fight's camera.
fn next_seat(current: Option<usize>, me: usize, present: impl Fn(usize) -> bool) -> Option<usize> {
    let order =
        std::iter::once(me).chain((0..crate::MAX_PLAYERS).filter(|&i| i != me && present(i)));
    match current {
        None => Some(me),
        Some(now) => order.skip_while(|&i| i != now).nth(1),
    }
}

impl Inspect {
    /// Where the camera goes, and what it looks at, round a fighter standing
    /// with their feet at `feet`. `None` while it is the fight's camera.
    pub fn frame(&self, feet: [f32; 3]) -> Option<(Vec3, Vec3)> {
        self.seat?;
        let at = Vec3::new(feet[0], feet[1] + self.height, feet[2]);
        let out = Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        );
        let mut eye = at + out * self.distance;
        // Never under the floor they stand on: looking up at them from it is
        // as low as there is anything to see.
        eye.y = eye.y.max(feet[1] + 0.05);
        Some((eye, at))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f6_goes_round_everyone_present_and_back_to_the_fight() {
        let everyone = |_| true;
        let mut seen = vec![];
        let mut at = None;
        for _ in 0..crate::MAX_PLAYERS + 1 {
            at = next_seat(at, 1, everyone);
            seen.push(at);
        }
        assert_eq!(seen[0], Some(1), "you first");
        assert_eq!(seen[1], Some(0));
        assert_eq!(*seen.last().unwrap(), None, "and back to the fight");
        // An empty seat is skipped.
        assert_eq!(next_seat(Some(1), 1, |i| i == 1), None);
    }

    #[test]
    fn the_eye_is_as_far_as_the_wheel_put_it_and_never_under_the_floor() {
        let look = Inspect {
            seat: Some(0),
            yaw: 0.3,
            pitch: 0.2,
            distance: 2.0,
            height: 1.0,
        };
        let (eye, at) = look.frame([4.0, 1.0, -2.0]).unwrap();
        assert!((eye.distance(at) - 2.0).abs() < 1e-4);
        assert_eq!(at, Vec3::new(4.0, 2.0, -2.0));
        let below = Inspect {
            pitch: -1.4,
            ..look
        };
        assert!(below.frame([0.0, 1.0, 0.0]).unwrap().0.y >= 1.05);
        assert!(Inspect::default().frame([0.0; 3]).is_none());
    }
}
