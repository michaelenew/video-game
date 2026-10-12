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

// -- The full charges (C5) -----------------------------------------------

use sim::effects::EffectKind;

/// A Champion with nobody near, at `place` in the string, the Bulwark parked
/// `ahead` metres down his facing: room for a full charge to do its work.
fn open_floor(place: u8, ahead: i32) -> World {
    let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
    let y = w.players[0].pos.y;
    w.players[0].pos = sim::V3::new(sim::Fx::from_int(-6), y, sim::Fx::from_int(8));
    w.players[1].pos = sim::V3::new(sim::Fx::from_int(-6 + ahead), y, sim::Fx::from_int(8));
    for _ in 0..2 {
        step(&mut w, 0);
    }
    if let Mechanic::Forms {
        chain, chain_left, ..
    } = &mut w.players[0].mechanic
    {
        *chain = place;
        *chain_left = 400;
    }
    w
}

/// Hold `button` to a full charge and let go. Returns the world on the frame
/// after the release, and the move it threw.
fn full_charge(w: &mut World, button: u16) -> Option<u8> {
    let full = sim::tuning::hold_full() as u32;
    for _ in 0..(full + 30) {
        step(w, button);
        if let Action::Channel { held, .. } = w.players[0].action {
            if held as u32 > full {
                break;
            }
        }
    }
    step(w, 0);
    w.players[0].action.attack_kind()
}

fn live(w: &World, kind: EffectKind) -> bool {
    w.effects.iter().flatten().any(|e| e.kind == kind)
}

#[test]
fn every_link_held_to_full_is_its_weapons_full_charge() {
    for (place, button, want) in [
        (0, Input::LEFT, c::CRESCENT),
        (0, Input::MIDDLE, c::SHOCKWAVE),
        (0, Input::RIGHT, c::COMET),
        (1, Input::LEFT, c::CRESCENT),
        (2, Input::LEFT, c::THOUSAND_CUTS),
        (2, Input::MIDDLE, c::METEOR),
        (2, Input::RIGHT, c::HEAVEN_PIERCER),
    ] {
        let mut w = open_floor(place, 12);
        assert_eq!(
            full_charge(&mut w, button),
            Some(want),
            "link {place} of {button:#x} held to full"
        );
    }
}

#[test]
fn the_crescent_cuts_somebody_well_past_the_swords_reach() {
    let mut w = open_floor(0, 7);
    let before = w.players[1].health;
    assert_eq!(full_charge(&mut w, Input::LEFT), Some(c::CRESCENT));
    let mut flew = false;
    for _ in 0..40 {
        step(&mut w, 0);
        flew |= live(&w, EffectKind::Crescent);
    }
    assert!(flew, "no crescent left the blade");
    assert!(
        w.players[1].health < before,
        "the crescent never reached somebody seven metres away"
    );
}

#[test]
fn the_shockwave_throws_up_somebody_outside_the_hammers_reach() {
    let mut w = open_floor(0, 4);
    let before = w.players[1].health;
    assert_eq!(full_charge(&mut w, Input::MIDDLE), Some(c::SHOCKWAVE));
    let mut up = false;
    for _ in 0..40 {
        step(&mut w, 0);
        up |= !w.players[1].grounded;
    }
    assert!(w.players[1].health < before, "the ring never reached them");
    assert!(up, "and did not throw them up");
}

#[test]
fn the_comet_crosses_eight_metres() {
    let mut w = open_floor(0, 30);
    let from = w.players[0].pos;
    assert_eq!(full_charge(&mut w, Input::RIGHT), Some(c::COMET));
    for _ in 0..40 {
        step(&mut w, 0);
    }
    let gone = w.players[0].pos.sub(from).flat_len().to_f32_for_render();
    let want = sim::tuning::comet_reach().to_f32_for_render();
    assert!(
        gone > want * 0.8,
        "the Comet lunged {gone:.1} m of its {want:.1}"
    );
}

#[test]
fn thousand_cuts_runs_through_them_and_the_cuts_land_after() {
    let mut w = open_floor(2, 3);
    let before = w.players[1].health;
    assert_eq!(full_charge(&mut w, Input::LEFT), Some(c::THOUSAND_CUTS));
    let mut cuts = 0;
    let mut last = before;
    let mut thrown = false;
    for _ in 0..90 {
        step(&mut w, 0);
        if w.players[1].health < last {
            cuts += 1;
            last = w.players[1].health;
        }
        thrown |= !w.players[1].grounded;
    }
    assert!(cuts >= 3, "the cuts landed {cuts} times");
    assert!(thrown, "and the last did not throw them up");
    let past = w.players[0].pos.x.to_f32_for_render() > w.players[1].pos.x.to_f32_for_render();
    assert!(past, "he ran through them");
}

#[test]
fn the_meteor_comes_down_where_the_crosshair_was() {
    let mut w = open_floor(2, 6);
    let before = w.players[1].health;
    // Pointed down at the floor in front of him.
    let full = sim::tuning::hold_full() as u32;
    let look = |bits| Input::looking_at(bits, LOOK_RIGHT, -3000);
    for _ in 0..(full + 30) {
        w.advance([look(Input::MIDDLE), Input::aimed(0, LOOK_LEFT)]);
        if matches!(w.players[0].action, Action::Channel { held, .. } if held as u32 > full) {
            break;
        }
    }
    w.advance([look(0), Input::aimed(0, LOOK_LEFT)]);
    assert_eq!(w.players[0].action.attack_kind(), Some(c::METEOR));
    let mut flew = false;
    let mut ring = false;
    for _ in 0..120 {
        step(&mut w, 0);
        flew |= !w.players[0].grounded;
        ring |= live(&w, EffectKind::Shockwave);
    }
    assert!(flew, "the Meteor never left the floor");
    assert!(ring, "and broke no ground where it came down");
    assert!(w.players[1].health < before, "the crater reached nobody");
}

#[test]
fn the_heaven_piercer_pins_them_and_he_tears_it_out() {
    let mut w = open_floor(2, 10);
    let before = w.players[1].health;
    assert_eq!(full_charge(&mut w, Input::RIGHT), Some(c::HEAVEN_PIERCER));
    let mut pinned = false;
    let mut thrown = false;
    let mut pinned_at = w.players[1].pos;
    for _ in 0..120 {
        step(&mut w, 0);
        if matches!(w.players[1].action, Action::Held { .. }) {
            pinned = true;
            pinned_at = w.players[1].pos;
        }
        thrown |= !w.players[1].grounded;
    }
    assert!(pinned, "the javelin pinned nobody");
    assert!(w.players[1].health < before);
    assert!(thrown, "the tear-out did not throw them up");
    assert!(
        !live(&w, EffectKind::Javelin),
        "the spear is back in his hands"
    );
    let near = w.players[0]
        .pos
        .sub(pinned_at)
        .flat_len()
        .to_f32_for_render();
    assert!(
        near < sim::tuning::tear_radius().to_f32_for_render(),
        "he went after it and stopped where it was: {near:.1} m from it"
    );
}

#[test]
fn a_grand_finisher_that_lands_gives_the_rush_back() {
    let mut w = open_floor(2, 3);
    if let Mechanic::Forms { recharge, .. } = &mut w.players[0].mechanic {
        *recharge = 200;
    }
    full_charge(&mut w, Input::MIDDLE);
    let mut back = false;
    for _ in 0..120 {
        step(&mut w, 0);
        back |= matches!(w.players[0].mechanic, Mechanic::Forms { recharge: 0, .. });
    }
    assert!(back, "the Meteor landed and the Rush stayed spent");
}

// -- Momentum (C4) -------------------------------------------------------

fn fury(w: &World) -> sim::class::Fury {
    match w.players[0].mechanic {
        Mechanic::Forms { fury, .. } => fury,
        _ => unreachable!(),
    }
}

fn set_momentum(w: &mut World, to: i32) {
    if let Mechanic::Forms { fury, .. } = &mut w.players[0].mechanic {
        fury.momentum = to as u16;
        fury.momentum_left = 200;
    }
}

#[test]
fn landed_hits_feed_the_meter_and_mixing_feeds_it_more() {
    let t = sim::tuning::momentum_hit();
    let mut same = engaged();
    for f in 0..60 {
        step(&mut same, if f % 4 == 0 { Input::LEFT } else { 0 });
    }
    let mut mixed = engaged();
    for f in 0..60 {
        let b = if f < 20 { Input::LEFT } else { Input::RIGHT };
        step(&mut mixed, if f % 4 == 0 { b } else { 0 });
    }
    assert!(
        fury(&same).momentum as i32 >= t * 2,
        "two sword hits fed {}",
        fury(&same).momentum
    );
    assert!(
        fury(&mixed).momentum > fury(&same).momentum,
        "sword into spear fed {} and sword into sword {}",
        fury(&mixed).momentum,
        fury(&same).momentum
    );
}

#[test]
fn the_meter_holds_and_then_drains() {
    let mut w = engaged();
    set_momentum(&mut w, 50);
    if let Mechanic::Forms { fury, .. } = &mut w.players[0].mechanic {
        fury.momentum_left = 10;
    }
    for _ in 0..10 {
        step(&mut w, 0);
    }
    assert_eq!(fury(&w).momentum, 50, "it holds");
    for _ in 0..30 {
        step(&mut w, 0);
    }
    assert!(fury(&w).momentum < 50, "and then drains");
}

#[test]
fn a_full_meter_makes_a_tap_a_full_charge_and_spends_it() {
    let mut w = open_floor(0, 12);
    set_momentum(&mut w, sim::tuning::momentum_max());
    assert_eq!(fury(&w).tier(), 3);
    step(&mut w, Input::LEFT);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(c::CRESCENT),
        "a tap at full momentum is the full charge"
    );
    assert_eq!(fury(&w).momentum, 0, "and spends the meter");
}

#[test]
fn heated_and_blazing_fill_a_hold_faster() {
    let held_for = |m: i32| {
        let mut w = open_floor(0, 30);
        set_momentum(&mut w, m);
        let mut f = 0;
        loop {
            step(&mut w, Input::MIDDLE);
            f += 1;
            if let Action::Channel { .. } = w.players[0].action {
                break;
            }
            assert!(f < 60);
        }
        // Hold, a frame at a time, until letting go would be the full charge.
        let mut held = 0;
        loop {
            let mut probe = w.clone();
            step(&mut probe, 0);
            if probe.players[0].action.attack_kind() == Some(c::SHOCKWAVE) {
                return held;
            }
            step(&mut w, Input::MIDDLE);
            held += 1;
            assert!(held < 200);
        }
    };
    let cold = held_for(0);
    let max = sim::tuning::momentum_max();
    let heated = held_for(max / 3 + 1);
    let blazing = held_for(max * 2 / 3 + 1);
    assert!(
        cold > heated && heated > blazing,
        "full at {cold}, {heated} heated and {blazing} blazing"
    );
}

#[test]
fn a_blow_taken_costs_a_share_of_the_meter() {
    let mut w = World::with_classes([Class::Bulwark, Class::Champion]);
    for _ in 0..90 {
        w.advance([
            Input::aimed(Input::W, LOOK_RIGHT),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
    if let Mechanic::Forms { fury, .. } = &mut w.players[1].mechanic {
        fury.momentum = 60;
        fury.momentum_left = 400;
    }
    for f in 0..40 {
        let b = if f == 0 { Input::LEFT } else { 0 };
        w.advance([Input::aimed(b, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
    }
    let Mechanic::Forms { fury, .. } = w.players[1].mechanic else {
        unreachable!()
    };
    assert_eq!(
        fury.momentum as i32,
        60 - 60 * sim::tuning::momentum_hurt() / 100,
        "the Bash cost him a share"
    );
}

// -- Break and the Clash (C6, C7) ----------------------------------------

#[test]
fn the_champions_heavy_blows_weigh_on_a_creature_more_than_they_cut() {
    use sim::species::SpeciesId;
    // The hammer on the Ridgeback: on the frame it lands, the animal's strain
    // climbs by more than its health falls.
    let mut w = World::hunt_of([Class::Champion; 2], SpeciesId::RIDGEBACK).seated(1);
    let beast = w.monster().unwrap().pos;
    let y = w.players[0].pos.y;
    w.players[0].pos = sim::V3::new(beast.x.sub(sim::Fx::from_int(3)), y, beast.z);
    let mut seen = None;
    for f in 0..60 {
        let before = *w.monster().unwrap();
        let aim = Input::looking_at(if f == 0 { Input::MIDDLE } else { 0 }, LOOK_RIGHT, -2000);
        w.advance([aim, Input::default()]);
        let after = w.monster().unwrap();
        if after.health < before.health {
            seen = Some((before.health - after.health, after.strain - before.strain));
            break;
        }
    }
    let (cut, weighed) = seen.expect("the hammer never reached the Ridgeback");
    assert!(weighed > cut, "the hammer cut {cut} and weighed {weighed}");
}

#[test]
fn a_heavy_blow_meeting_a_lighter_one_wins_it_outright() {
    // Two Champions in reach: one swinging the sword, one in the Shockwave's
    // active frames. The heavier blow lands; the sword is cancelled and its
    // thrower staggered.
    let mut w = World::with_classes([Class::Champion, Class::Champion]);
    for _ in 0..90 {
        w.advance([
            Input::aimed(Input::W, LOOK_RIGHT),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
    let shockwave = sim::moves::get(Class::Champion, c::SHOCKWAVE);
    let sword = sim::moves::get(Class::Champion, c::SWORD_GROUND);
    w.players[0].action = Action::Active {
        kind: c::SHOCKWAVE,
        left: shockwave.active,
    };
    w.players[1].action = Action::Active {
        kind: c::SWORD_GROUND,
        left: sword.active,
    };
    let (h0, h1) = (w.players[0].health, w.players[1].health);
    w.advance([Input::aimed(0, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
    assert_eq!(w.players[0].health, h0, "the lighter blow was cancelled");
    assert!(w.players[1].health < h1, "the heavier one landed");
    assert!(
        matches!(
            w.players[1].action,
            Action::Stagger { .. } | Action::HitStun { .. }
        ),
        "and the lighter's thrower is in no state to go on: {:?}",
        w.players[1].action
    );
}
