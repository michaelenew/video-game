//! The Champion's rework (exploration 0015): the Rush, fast and braced, whose
//! run into a body is the string's first hit (C1); the Beat (C2).

use sim::class::{Class, Mechanic};
use sim::moves::champion as c;
use sim::state::Action;
use sim::{Input, World};

const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn step(w: &mut World, a: u16) {
    w.advance([Input::aimed(a, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
}

/// A Champion and a Bulwark, the Champion walked in to a Rush's length.
fn in_reach_of_a_rush() -> World {
    let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
    for _ in 0..20 {
        step(&mut w, Input::W);
    }
    w
}

#[test]
fn the_rush_is_faster_than_a_dodge() {
    let rush = sim::tuning::rush_speed().to_f32_for_render();
    let dodge = sim::tuning::dodge_speed().to_f32_for_render();
    assert!(
        rush > dodge * 1.6,
        "a Rush at {rush} m/s is a dodge with a hitbox beside a {dodge} m/s dodge"
    );
}

#[test]
fn a_rush_into_a_body_is_the_strings_first_hit() {
    let mut w = in_reach_of_a_rush();
    let before = w.players[1].health;
    step(&mut w, Input::W | Input::MECHANIC);
    let mut struck = false;
    for _ in 0..30 {
        step(&mut w, Input::W);
        if w.players[1].health < before {
            struck = true;
            break;
        }
    }
    assert!(struck, "the shoulder never landed");
    assert_eq!(before - w.players[1].health, sim::tuning::shoulder_damage());
    let Mechanic::Forms {
        chain,
        chain_hit,
        rush,
        ..
    } = w.players[0].mechanic
    else {
        panic!("not a Champion");
    };
    assert_eq!(chain, 1, "the shoulder is hit one");
    assert!(chain_hit);
    assert_eq!(rush, 0, "and the run stops where it struck");
    // Out of the freeze, the next click is the second link.
    for _ in 0..12 {
        step(&mut w, 0);
    }
    step(&mut w, Input::LEFT);
    let thrown = w.players[0].action.attack_kind();
    assert_eq!(thrown, Some(c::BACKCUT), "{:?}", w.players[0].action);
    assert!(
        matches!(w.players[1].action, Action::HitStun { .. }),
        "and they are still in the shoulder's stun when it starts: {:?}",
        w.players[1].action
    );
}

/// Walked in to sword range, both facing.
fn engaged() -> World {
    let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
    for _ in 0..90 {
        step(&mut w, Input::W);
    }
    w
}

/// Open with the sword, then press the sword again either inside the
/// opener's freeze or after it. What the second hit dealt.
fn second_hit(on_the_beat: bool) -> (i32, bool) {
    let mut w = engaged();
    step(&mut w, Input::LEFT);
    let mut f = 0;
    while w.players[0].frozen == 0 {
        step(&mut w, 0);
        f += 1;
        assert!(f < 30, "the opener never landed");
    }
    // Late, the button is held down the way a person holds it, until the
    // chain takes it. On the beat it is a single frame's press: the beat is
    // held for him (`hold_the_beat`).
    let held = if on_the_beat {
        1
    } else {
        while w.players[0].frozen > 0 {
            step(&mut w, 0);
        }
        step(&mut w, 0);
        15
    };
    let before = w.players[1].health;
    let mut true_link = false;
    for f in 0..40 {
        step(&mut w, if f < held { Input::LEFT } else { 0 });
        if let Mechanic::Forms { fury, .. } = w.players[0].mechanic {
            true_link |= fury.true_link;
        }
    }
    (before - w.players[1].health, true_link)
}

#[test]
fn a_link_pressed_in_the_freeze_is_a_true_link() {
    let (late, late_true) = second_hit(false);
    let (beat, beat_true) = second_hit(true);
    assert!(
        !late_true,
        "a link pressed after the freeze is an ordinary link"
    );
    assert!(beat_true, "a link pressed inside the freeze is a true link");
    let backcut = sim::moves::get(Class::Champion, c::BACKCUT).damage;
    assert_eq!(late, backcut, "the ordinary backcut");
    assert_eq!(
        beat,
        backcut * (100 + sim::tuning::true_link_damage() as i32) / 100,
        "the true one hits harder"
    );
}

/// Press a weapon fresh and keep it down for `frames`, then let go and let
/// the swing play out. What it dealt, and whether it was ever held.
fn held_swing(button: u16, frames: u32) -> (i32, bool) {
    let mut w = engaged();
    let before = w.players[1].health;
    let mut held = false;
    for f in 0..(frames + 60) {
        step(&mut w, if f < frames { button } else { 0 });
        held |= matches!(w.players[0].action, Action::Channel { .. });
    }
    (before - w.players[1].health, held)
}

#[test]
fn a_link_pressed_for_and_held_waits_at_the_top_of_its_wind_up() {
    let mut w = engaged();
    for _ in 0..40 {
        step(&mut w, Input::MIDDLE);
    }
    assert!(
        matches!(
            w.players[0].action,
            Action::Channel {
                kind: c::HAMMER_GROUND,
                ..
            }
        ),
        "the hammer held is held: {:?}",
        w.players[0].action
    );
    assert_eq!(
        w.players[1].health,
        sim::state::Player::new(Class::Bulwark).health,
        "and nothing has landed yet"
    );
}

#[test]
fn a_part_charge_hits_harder_than_a_tap() {
    let sword = sim::moves::get(Class::Champion, c::SWORD_GROUND);
    let (tap, tap_held) = held_swing(Input::LEFT, 2);
    assert!(!tap_held, "a tap is not a hold");
    assert_eq!(tap, sword.damage);
    let startup = sword.startup as u32;
    let full = sim::tuning::hold_full() as u32;
    let (part, part_held) = held_swing(Input::LEFT, startup + full / 2);
    assert!(part_held, "held past the wind-up, it is held");
    assert!(
        part > tap,
        "a part charge ({part}) hits harder than a tap ({tap})"
    );
}

#[test]
fn tapping_runs_the_string_and_holding_charges_it() {
    // Linear play was a button held down. Holding is the charge now, so it
    // is a button tapped: every link thrown, none of them held.
    let mut w = engaged();
    let mut kinds = Vec::new();
    let mut held = false;
    for f in 0..90 {
        step(&mut w, if f % 4 == 0 { Input::LEFT } else { 0 });
        held |= matches!(w.players[0].action, Action::Channel { .. });
        if let Some(k) = w.players[0].action.attack_kind() {
            if kinds.last() != Some(&k) {
                kinds.push(k);
            }
        }
    }
    assert!(!held, "a tapped string held nothing");
    assert_eq!(
        &kinds[..3],
        &[c::SWORD_GROUND, c::BACKCUT, c::UPCUT],
        "a tapped sword ran {kinds:?}"
    );
}

#[test]
fn a_rush_takes_a_hold_back() {
    let mut w = engaged();
    for _ in 0..40 {
        step(&mut w, Input::MIDDLE);
    }
    assert!(matches!(w.players[0].action, Action::Channel { .. }));
    step(&mut w, Input::MIDDLE | Input::MECHANIC | Input::S);
    assert!(
        !matches!(w.players[0].action, Action::Channel { .. }),
        "the Rush took the hold back: {:?}",
        w.players[0].action
    );
    let Mechanic::Forms { rush, .. } = w.players[0].mechanic else {
        unreachable!()
    };
    assert!(rush > 0, "and he is running");
}
