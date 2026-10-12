//! The Champion's Rush, rebuilt (exploration 0015, C1): fast, and a run that
//! meets a body hits it with the shoulder -- the string's first hit.

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
