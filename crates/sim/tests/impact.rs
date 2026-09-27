//! The impact freeze: when a blow connects, both bodies hold still together.
//!
//! What is pinned here is the mechanism rather than any length -- the lengths
//! are a column in the Oven (`Impact freeze`) and move freely. The properties
//! that must survive tuning are that the two bodies stop and start on the same
//! frames, so no frame advantage moves; that nothing about either of them
//! moves while it lasts; that the knockback is still there when it ends; and
//! that a button pressed inside it is not thrown away.

use sim::class::Class;
use sim::moves::{self, champion as c};
use sim::state::Action;
use sim::{Input, World};

const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

/// A Champion a step from a Bulwark who does nothing, so the first swing lands.
///
/// **Out on open floor**, off to the side of both platforms. The obvious
/// fixture -- walk one into the other -- pushes the victim back against the
/// side of a platform, and a body pinned to a wall slides nowhere, which reads
/// exactly like knockback that does not work.
fn engaged() -> World {
    let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
    let y = w.players[0].pos.y;
    w.players[0].pos = sim::V3::new(sim::Fx::ZERO, y, sim::Fx::from_int(8));
    w.players[1].pos = sim::V3::new(sim::Fx::ratio(6, 5), y, sim::Fx::from_int(8));
    for _ in 0..2 {
        w.advance([Input::aimed(0, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
    }
    w
}

/// Hold `button` until the hit lands, and return the world on that frame.
fn until_contact(button: u16) -> World {
    let mut w = engaged();
    for _ in 0..60 {
        w.advance([Input::aimed(button, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        if w.players[1].frozen > 0 {
            return w;
        }
    }
    panic!("the swing never connected: {:?}", w.players[0].action);
}

#[test]
fn a_hit_freezes_both_bodies_for_the_moves_own_length() {
    for (button, kind) in [
        (Input::LEFT, c::SWORD_GROUND),
        (Input::MIDDLE, c::HAMMER_GROUND),
        (Input::RIGHT, c::SPEAR_GROUND),
    ] {
        let m = moves::get(Class::Champion, kind);
        let w = until_contact(button);
        assert_eq!(
            w.players[1].frozen as u16, m.hitstop,
            "{}: the victim froze for {} frames and the move says {}",
            m.name, w.players[1].frozen, m.hitstop
        );
        assert_eq!(
            w.players[0].frozen, w.players[1].frozen,
            "{}: the attacker and the victim are not held for the same length, so \
             the freeze moved the frame advantage",
            m.name
        );
    }
}

#[test]
fn nothing_moves_during_a_freeze_and_the_knockback_is_spent_after() {
    let mut w = until_contact(Input::MIDDLE);
    let held = w.players[1].frozen;
    assert!(held > 1, "the hammer does not freeze long enough to test");
    let before = w.players;
    // Hammering the buttons the whole time: a freeze holds both bodies whatever
    // either player is doing.
    for _ in 0..held {
        w.advance([
            Input::aimed(Input::MIDDLE | Input::W, LOOK_RIGHT),
            Input::aimed(Input::LEFT | Input::S, LOOK_LEFT),
        ]);
    }
    for (i, was) in before.iter().enumerate() {
        assert_eq!(
            w.players[i].pos, was.pos,
            "body {i} moved during the freeze"
        );
        assert_eq!(
            w.players[i].action, was.action,
            "body {i}'s action ran during the freeze"
        );
    }
    assert_eq!(
        w.players[1].frozen, 0,
        "the freeze outlasted its own length"
    );
    // And the shove was waiting for it.
    let gap = |w: &World| w.players[1].pos.sub(w.players[0].pos).flat_len();
    let start = gap(&w);
    for _ in 0..20 {
        w.advance([Input::aimed(0, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
    }
    assert!(
        gap(&w).raw() > start.raw(),
        "the knockback did not survive the freeze"
    );
}

#[test]
fn a_blocked_blow_freezes_for_less_than_a_landed_one() {
    let m = moves::get(Class::Champion, c::HAMMER_GROUND);
    let mut w = engaged();
    for _ in 0..60 {
        // The Bulwark guards, facing the Champion.
        w.advance([
            Input::aimed(Input::MIDDLE, LOOK_RIGHT),
            Input::aimed(Input::RIGHT, LOOK_LEFT),
        ]);
        if w.players[1].frozen > 0 {
            break;
        }
    }
    assert!(
        matches!(w.players[1].action, Action::BlockStun { .. }),
        "the fixture did not block: {:?}",
        w.players[1].action
    );
    let blocked = w.players[1].frozen as u16;
    assert!(
        blocked > 0 && blocked < m.hitstop,
        "a blocked hammer froze for {blocked} and a landed one for {}",
        m.hitstop
    );
    assert_eq!(w.players[0].frozen, w.players[1].frozen);
}

#[test]
fn a_press_inside_the_freeze_is_the_same_press_on_the_frame_after_it() {
    // **A freeze is a pause in the fight, not a pause in the player.** A button
    // that went down and came back up inside one must do exactly what it would
    // have done pressed on the first frame the fight resumes -- no more, since
    // the fight was not running, and no less, since the player did press it.
    // Checked for every button that means something on the frame it goes down.
    for button in [
        Input::LEFT,
        Input::MIDDLE,
        Input::RIGHT,
        Input::MECHANIC,
        Input::SPACE,
        Input::SHIFT,
    ] {
        let start = until_contact(Input::MIDDLE);
        let held = start.players[0].frozen as u32;
        assert!(held > 2, "the hammer does not freeze long enough to test");
        let mut inside = start.clone();
        let mut after = start;
        for f in 0..held + 40 {
            let tap_inside = if f == 1 { button } else { 0 };
            let tap_after = if f == held { button } else { 0 };
            inside.advance([
                Input::aimed(tap_inside, LOOK_RIGHT),
                Input::aimed(0, LOOK_LEFT),
            ]);
            after.advance([
                Input::aimed(tap_after, LOOK_RIGHT),
                Input::aimed(0, LOOK_LEFT),
            ]);
        }
        assert_eq!(
            inside.checksum(),
            after.checksum(),
            "button {button:#x} pressed inside the freeze did not do what it does \
             pressed on the frame after"
        );
    }
}
