//! **The ledge grab, and the stick against a wall** (2026-10-16,
//! `state::catch_a_ledge`, `state::hang_on`, `state::keep_sliding`).
//!
//! What is pinned: one rule catches an edge however the body got there --
//! fully off it, not leaving fast, the top at its hands, the stick not
//! pushing away -- so drifting off one catches it, and walking, dodging or
//! falling away from it fast does not; a drop shorter than a hang is a step
//! down, and anything deeper is a ledge; an edge with no room over it is hung
//! from and gone along, never climbed; nothing pressed hangs; a stick held
//! into the wall through the catch climbs only after the hold-on, one pressed
//! in afresh at once; the climb up is slow and ends standing on the top, and
//! letting go is quick; along the wall goes hand over hand, at the edge's own
//! height; crouch lets go into a slide; a blow lets go. And the slide: only
//! pushing into a wall starts one, pushing away ends one, neither faster than
//! the flip, and along the wall steers a little.
//!
//! The walls are the range's tower: its north ledge (x 17 to 23, z 4.5 to 6)
//! is 12 m to its top, and its face at z = 6 drops clean to the floor.

use sim::arena::ArenaId;
use sim::fixed::Fx;
use sim::state::{Action, Grip};
use sim::tuning as t;
use sim::{Class, Input, V3, World};

/// Looking +z (off the north face, away from the tower), and -z (into it).
const NORTH: u16 = 16384;
const SOUTH: u16 = 49152;

fn top() -> Fx {
    Fx::from_int(12)
}

/// A Champion standing on the tower's north ledge, `z` across it, the other
/// fighter out of the way.
fn on_the_top(z: Fx) -> World {
    let mut w = World::versus_in([Class::Champion; 2], ArenaId::RANGE);
    let p = &mut w.players[0];
    p.pos = V3::new(Fx::from_int(20), top(), z);
    p.vel = V3::ZERO;
    p.grounded = true;
    w.players[1].pos = V3::new(Fx::from_int(-100), Fx::ZERO, Fx::ZERO);
    w
}

/// The same Champion in the air beside the north face at height `y`, falling.
fn beside_the_face(y: Fx) -> World {
    let mut w = on_the_top(Fx::ratio(66, 10));
    let p = &mut w.players[0];
    p.pos.y = y;
    p.vel = V3::new(Fx::ZERO, Fx::from_int(-2), Fx::ZERO);
    p.grounded = false;
    w
}

fn press(w: &mut World, bits: u16, aim: u16, frames: u32) {
    for _ in 0..frames {
        w.advance([Input::aimed(bits, aim), Input::default()]);
    }
}

fn holding(w: &World) -> bool {
    w.players[0].ledge.is_some()
}

/// Just off the north edge, its top at the shoulders, falling: caught.
fn caught() -> World {
    let mut w = beside_the_face(top().sub(Fx::ratio(16, 10)));
    w.players[0].vel = V3::ZERO;
    press(&mut w, 0, SOUTH, 1);
    assert!(holding(&w), "never caught the edge: {:?}", w.players[0].pos);
    w
}

/// Caught and settled into the hang.
fn hanging() -> World {
    let mut w = caught();
    press(&mut w, 0, SOUTH, t::ledge_catch() as u32 + 1);
    assert_eq!(w.players[0].ledge.grip, Grip::Hang);
    w
}

#[test]
fn the_hang_is_under_the_top_facing_the_wall() {
    let w = hanging();
    let p = &w.players[0];
    assert_eq!(p.pos.y, top().sub(t::ledge_depth()));
    assert!(
        p.pos.z.raw() > Fx::from_int(6).raw(),
        "not off the face: {:?}",
        p.pos
    );
    assert!(p.facing.z.raw() < 0, "facing {:?}", p.facing);
}

#[test]
fn drifting_off_an_edge_catches_it() {
    // Just gone over the rim, drifting off it at half a metre a second with
    // nothing pressed: as the top comes to the shoulders, it holds.
    let mut w = on_the_top(Fx::ratio(655, 100));
    let p = &mut w.players[0];
    p.grounded = false;
    p.vel = V3::new(Fx::ZERO, Fx::ZERO, Fx::ratio(1, 2));
    for _ in 0..60 {
        press(&mut w, 0, NORTH, 1);
        if holding(&w) {
            break;
        }
    }
    assert!(
        holding(&w),
        "drifted off into nothing: {:?}",
        w.players[0].pos
    );
}

#[test]
fn walking_off_an_edge_at_a_walk_does_not_catch_it() {
    // A walk is far faster away from the face than `Ledge, caught moving
    // away from it at up to`, and by the time the top is at the shoulders
    // the body is well out from it: a body that meant to go.
    let mut w = on_the_top(Fx::ratio(55, 10));
    for _ in 0..120 {
        press(&mut w, Input::W, NORTH, 1);
        assert!(
            !holding(&w),
            "a walk off caught the edge at {:?}",
            w.players[0].pos
        );
    }
}

#[test]
fn moving_away_fast_does_not_catch() {
    let mut w = beside_the_face(top().sub(Fx::ratio(16, 10)));
    w.players[0].vel = V3::new(Fx::ZERO, Fx::ZERO, t::ledge_away().add(Fx::ONE));
    for _ in 0..30 {
        press(&mut w, 0, NORTH, 1);
        assert!(!holding(&w), "caught leaving fast");
    }
}

#[test]
fn held_into_the_wall_climbs_only_after_the_hold_on() {
    let mut w = beside_the_face(top().sub(Fx::ratio(16, 10)));
    w.players[0].vel = V3::ZERO;
    press(&mut w, Input::W, SOUTH, 1);
    assert!(holding(&w));
    let mut since = 1;
    while w.players[0].ledge.grip != Grip::Climb {
        press(&mut w, Input::W, SOUTH, 1);
        since += 1;
        assert!(since < 200, "never climbed");
    }
    assert!(
        since >= t::ledge_hold_on() as u32,
        "climbed {since} frames after the catch"
    );
}

#[test]
fn pressed_into_the_wall_afresh_climbs_at_once() {
    let mut w = beside_the_face(top().sub(Fx::ratio(16, 10)));
    w.players[0].vel = V3::ZERO;
    press(&mut w, Input::W, SOUTH, 1);
    assert!(holding(&w));
    press(&mut w, 0, SOUTH, t::ledge_catch() as u32 + 1);
    press(&mut w, Input::W, SOUTH, 1);
    assert_eq!(
        w.players[0].ledge.grip,
        Grip::Climb,
        "a fresh press did not climb"
    );
}

#[test]
fn a_dodge_off_an_edge_does_not_catch_it() {
    let mut w = on_the_top(Fx::ratio(55, 10));
    press(&mut w, Input::W | Input::SHIFT, NORTH, 1);
    for _ in 0..90 {
        press(&mut w, 0, NORTH, 1);
        assert!(!holding(&w), "a dodge caught the edge");
    }
}

#[test]
fn a_short_drop_is_a_step_down() {
    // The bank along the range's north wall: 3 m, with a 1.5 m step in
    // front of it at z 21 to 22. Walking off the top on to the step is a
    // drop of 1.5 m, and so is walking off the step: shorter than a hang, so
    // a hanging body would stand in the floor, and neither is a ledge.
    let mut w = on_the_top(Fx::ZERO);
    w.players[0].pos = V3::new(Fx::from_int(20), Fx::from_int(3), Fx::ratio(225, 10));
    for _ in 0..120 {
        press(&mut w, Input::W, SOUTH, 1);
        assert!(!holding(&w), "caught a step at {:?}", w.players[0].pos);
    }
    assert!(w.players[0].grounded && w.players[0].pos.y == Fx::ZERO);
}

#[test]
fn falling_past_an_edge_catches_it() {
    // Feet a metre under the top: the top comes to the hands as it falls.
    let mut w = beside_the_face(Fx::from_int(11));
    for _ in 0..30 {
        press(&mut w, 0, SOUTH, 1);
        if holding(&w) {
            break;
        }
    }
    assert!(holding(&w), "fell past it: {:?}", w.players[0].pos);
    press(&mut w, 0, SOUTH, 60);
    assert_eq!(w.players[0].pos.y, top().sub(t::ledge_depth()));
}

#[test]
fn pushing_away_falls_past_an_edge() {
    let mut w = beside_the_face(Fx::from_int(11));
    for _ in 0..30 {
        press(&mut w, Input::W, NORTH, 1);
        assert!(!holding(&w), "caught it pushing away");
    }
}

#[test]
fn the_climb_up_is_slow_and_ends_standing_on_the_top() {
    let mut w = hanging();
    let mut frames = 0;
    while !w.players[0].grounded {
        // Into the wall, and a click every frame: nothing comes out of a
        // climb.
        press(&mut w, Input::W | Input::LEFT, SOUTH, 1);
        frames += 1;
        assert!(frames <= 600, "never got up: {:?}", w.players[0].pos);
        if !w.players[0].grounded {
            assert_eq!(w.players[0].action, Action::Free, "attacked out of a climb");
            assert_eq!(w.players[0].ledge.grip, Grip::Climb);
        }
    }
    assert!(
        frames >= t::ledge_climb() as u32,
        "a climb of {frames} frames"
    );
    // Slower than the jump that would have made it: a full hop is about a
    // second, and the climb comes after the catch and the hang.
    assert!(frames >= 30, "the climb is not slow: {frames} frames");
    let p = &w.players[0];
    assert_eq!(p.pos.y, top());
    assert!(
        p.pos.z.raw() < Fx::from_int(6).raw(),
        "not on the top: {:?}",
        p.pos
    );
    assert!(!holding(&w));
    // And it stands there.
    press(&mut w, 0, SOUTH, 10);
    assert!(w.players[0].grounded && w.players[0].pos.y == top());
}

#[test]
fn letting_go_is_quick() {
    let mut w = hanging();
    press(&mut w, Input::W, NORTH, 1);
    let p = &w.players[0];
    assert!(!holding(&w));
    assert!(p.action.actionable(), "not free to act: {:?}", p.action);
    assert!(p.vel.z.raw() > 0, "not off the face: {:?}", p.vel);
    // And it falls: not caught again by the ledge just let go of.
    for _ in 0..200 {
        press(&mut w, 0, NORTH, 1);
        assert!(!holding(&w), "caught again at {:?}", w.players[0].pos);
    }
    assert!(w.players[0].grounded);
}

#[test]
fn along_the_wall_is_hand_over_hand() {
    let mut w = hanging();
    let x0 = w.players[0].pos.x;
    // Looking into the wall, a strafe is along it.
    press(&mut w, Input::A, SOUTH, 30);
    let p = &w.players[0];
    assert!(holding(&w));
    assert_eq!(p.ledge.grip, Grip::Hang);
    let moved = p.pos.x.sub(x0).abs();
    assert!(moved.raw() > 0, "did not move along");
    // A shuffle, not a walk.
    let most = t::ledge_shuffle().mul(Fx::from_int(30)).mul(sim::DT);
    assert!(
        moved.raw() <= most.add(Fx::ratio(1, 100)).raw(),
        "moved {moved:?}"
    );
    assert_eq!(p.pos.y, top().sub(t::ledge_depth()));
    // For as far as the edge goes: a long way along, and still holding the
    // same top.
    press(&mut w, Input::A, SOUTH, 600);
    assert!(holding(&w), "came off the end: {:?}", w.players[0].pos);
    assert_eq!(w.players[0].ledge.edge.at.y, top());
}

#[test]
fn crouch_lets_go_into_a_slide() {
    let mut w = hanging();
    press(&mut w, Input::CROUCH, SOUTH, 1);
    assert!(!holding(&w));
    for _ in 0..20 {
        press(&mut w, 0, SOUTH, 1);
        assert!(
            w.players[0].sliding,
            "off the wall at {:?}",
            w.players[0].pos
        );
    }
    assert!(w.players[0].vel.y.raw() < 0);
}

#[test]
fn a_blow_lets_go() {
    let mut w = hanging();
    w.players[0].action = Action::HitStun { left: 20 };
    press(&mut w, 0, SOUTH, 1);
    assert!(!holding(&w));
}

/// Against the face well below the top, falling: a wall slide's ground.
fn low_beside_the_face() -> World {
    let mut w = beside_the_face(Fx::from_int(8));
    w.players[0].pos.z = Fx::ratio(651, 100);
    w
}

/// **Touching a wall slides, unless pushing away** (2026-10-20; until then
/// only pushing into it did, and nothing pressed fell past).
#[test]
fn touching_a_wall_slides_unless_pushing_away() {
    let mut w = low_beside_the_face();
    w.players[0].vel.z = Fx::from_int(-2);
    press(&mut w, 0, SOUTH, 10);
    assert!(w.players[0].sliding, "nothing pressed fell past the wall");
    let mut w = low_beside_the_face();
    press(&mut w, Input::W, SOUTH, 6);
    assert!(w.players[0].sliding, "pushing in did not slide");
    let mut w = low_beside_the_face();
    w.players[0].vel.z = Fx::from_int(-2);
    press(&mut w, Input::W, NORTH, 10);
    assert!(!w.players[0].sliding, "pushing away slid");
}

#[test]
fn a_slide_does_not_flip_faster_than_the_debounce() {
    let mut w = low_beside_the_face();
    let mut on = 0;
    for f in 0..10 {
        press(&mut w, Input::W, SOUTH, 1);
        if w.players[0].sliding {
            on = f;
            break;
        }
    }
    assert!(w.players[0].sliding);
    // Pushed away at once: held on until the flip has run.
    let flip = t::wall_slide_flip() as u32;
    let mut off = None;
    for f in 0..flip + 5 {
        press(&mut w, Input::W, NORTH, 1);
        if !w.players[0].sliding {
            off = Some(f + 1);
            break;
        }
    }
    let off = off.expect("pushing away never left the slide");
    assert!(
        off + 1 >= flip,
        "left {off} frames after starting on frame {on}: the flip is {flip}"
    );
}

#[test]
fn along_a_slide_steers_a_little() {
    let mut w = low_beside_the_face();
    press(&mut w, Input::W, SOUTH, 6);
    assert!(w.players[0].sliding);
    press(&mut w, Input::A, SOUTH, 20);
    let p = &w.players[0];
    assert!(p.sliding, "came off steering along");
    let along = p.vel.x.abs();
    assert!(along.raw() > 0, "no steering");
    assert!(
        along.raw() <= t::wall_slide_steer().add(Fx::ratio(1, 100)).raw(),
        "steered at {along:?}"
    );
}

#[test]
fn any_drop_a_body_hangs_in_is_a_ledge() {
    // A boulder on the range's grass, 2 m tall: a hang is 1.9 m, so the feet
    // clear the floor, and a body falling just off its north side catches it.
    let mut w = on_the_top(Fx::ZERO);
    let p = &mut w.players[0];
    p.pos = V3::new(Fx::from_int(90), Fx::ratio(4, 10), Fx::ratio(-82, 10));
    p.grounded = false;
    press(&mut w, 0, SOUTH, 1);
    assert!(holding(&w), "fell past the boulder: {:?}", w.players[0].pos);
}

/// Hanging from a pillar in the range's cave, which runs from the floor into
/// the vault: its top is the roof's underside, with no room over it.
fn under_the_roof() -> World {
    let mut w = on_the_top(Fx::ZERO);
    let p = &mut w.players[0];
    p.pos = V3::new(Fx::from_int(55), Fx::from_int(11), Fx::ratio(-54, 10));
    p.vel = V3::new(Fx::ZERO, Fx::from_int(-2), Fx::ZERO);
    p.grounded = false;
    for _ in 0..30 {
        press(&mut w, 0, SOUTH, 1);
        if holding(&w) {
            break;
        }
    }
    assert!(holding(&w), "fell past the pillar: {:?}", w.players[0].pos);
    press(&mut w, 0, SOUTH, t::ledge_catch() as u32 + 1);
    w
}

#[test]
fn an_edge_with_no_room_over_it_is_hung_from_and_not_climbed() {
    let mut w = under_the_roof();
    let y = w.players[0].pos.y;
    press(&mut w, Input::W, SOUTH, 120);
    assert!(holding(&w), "let go pushing into the wall");
    assert_eq!(w.players[0].ledge.grip, Grip::Hang, "climbed into the roof");
    assert_eq!(w.players[0].pos.y, y);
    // And along it, hand over hand: a shuffle along a wall with no way up.
    let x0 = w.players[0].pos.x;
    press(&mut w, Input::A, SOUTH, 30);
    assert!(holding(&w));
    assert!(w.players[0].pos.x != x0, "did not go along");
}

#[test]
fn neither_the_climb_nor_letting_go_is_drawn_as_landing_from_the_hang() {
    // A long hang counts frames in the air; the climb is its own landing, and
    // a let-go fall starts at the hang, so neither carries the hang's length
    // into the landing clip (`view::play`'s heavy landing reads it).
    let mut w = hanging();
    press(&mut w, 0, SOUTH, 120);
    while w.players[0].ledge.is_some() {
        press(&mut w, Input::W, SOUTH, 1);
    }
    assert!(w.players[0].grounded);
    assert!(
        w.players[0].air_frames <= 1,
        "climbed up with {} frames of flight",
        w.players[0].air_frames
    );
    let mut w = hanging();
    press(&mut w, 0, SOUTH, 120);
    press(&mut w, Input::W, NORTH, 1);
    assert!(!holding(&w));
    assert!(w.players[0].air_frames <= 1, "{}", w.players[0].air_frames);
}
