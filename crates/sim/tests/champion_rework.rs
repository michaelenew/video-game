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
    // Both landed off the same opener, so the same momentum is in both: the
    // difference is the beat's alone.
    let want = late * (100 + sim::tuning::true_link_damage() as i32) / 100;
    assert!(
        (beat - want).abs() <= 1,
        "the true one hits harder: {beat} against {late}, {want} wanted"
    );
}

/// Press a weapon fresh and keep it down for `frames`, then let go and let
/// the swing play out. What it dealt, and the moves it threw.
fn held_swing(button: u16, frames: u32) -> (i32, Vec<u8>) {
    let mut w = engaged();
    let before = w.players[1].health;
    let mut kinds = Vec::new();
    for f in 0..(frames + 60) {
        step(&mut w, if f < frames { button } else { 0 });
        if let Some(k) = w.players[0].action.attack_kind() {
            if kinds.last() != Some(&k) {
                kinds.push(k);
            }
        }
    }
    (before - w.players[1].health, kinds)
}

#[test]
fn a_press_a_little_long_is_still_the_tap() {
    // A click is six to eight frames long. The sword winds up in six, so a
    // press that outlasts its wind-up by a frame or two is still the sword.
    let (dealt, kinds) = held_swing(Input::LEFT, sim::tuning::hold_from() as u32 - 2);
    assert_eq!(kinds, [c::SWORD_GROUND], "{kinds:?}");
    assert_eq!(
        dealt,
        sim::moves::get(Class::Champion, c::SWORD_GROUND).damage
    );
}

#[test]
fn held_any_link_is_its_weapons_finisher() {
    // Tap to combo, hold to finish: held, the opener is the finisher, at once.
    for (button, want) in [
        (Input::LEFT, c::CRESCENT),
        (Input::MIDDLE, c::SHOCKWAVE),
        (Input::RIGHT, c::REPEL),
    ] {
        let (_, kinds) = held_swing(button, 40);
        assert!(kinds.contains(&want), "{button:#x} held threw {kinds:?}");
    }
}

#[test]
fn a_finisher_ends_the_string() {
    let mut w = engaged();
    for _ in 0..40 {
        step(&mut w, Input::MIDDLE);
    }
    let Mechanic::Forms { chain, .. } = w.players[0].mechanic else {
        unreachable!()
    };
    assert_eq!(chain, 0, "the next press opens a new string");
}

#[test]
fn tapping_runs_the_string_and_holding_finishes_it() {
    // Linear play: a button tapped throws every link, none of them a
    // finisher.
    let mut w = engaged();
    let mut kinds = Vec::new();
    for f in 0..90 {
        step(&mut w, if f % 4 == 0 { Input::LEFT } else { 0 });
        if let Some(k) = w.players[0].action.attack_kind() {
            if kinds.last() != Some(&k) {
                kinds.push(k);
            }
        }
    }
    assert!(
        kinds.iter().all(|k| !c::is_full(*k)),
        "a tapped string threw {kinds:?}"
    );
    assert_eq!(
        &kinds[..3],
        &[c::SWORD_GROUND, c::BACKCUT, c::UPCUT],
        "{kinds:?}"
    );
}

#[test]
fn a_rush_takes_a_hold_back() {
    // The sword's wind-up is shorter than a hold, so it waits at the top a
    // few frames for the press to mean it: a Rush in those takes it back.
    let mut w = engaged();
    let startup = sim::moves::get(Class::Champion, c::SWORD_GROUND).startup;
    for _ in 0..=startup + 1 {
        step(&mut w, Input::LEFT);
    }
    assert!(
        matches!(w.players[0].action, Action::Channel { .. }),
        "{:?}",
        w.players[0].action
    );
    step(&mut w, Input::LEFT | Input::MECHANIC);
    assert!(!matches!(w.players[0].action, Action::Channel { .. }));
    let Mechanic::Forms { recharge, .. } = w.players[0].mechanic else {
        unreachable!()
    };
    // Into the Bulwark in front of him, which the shoulder may already have
    // ended: the charge is spent either way.
    assert!(recharge > 0, "and he ran");
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

/// Hold `button` until the link it opens is its finisher, and let go.
/// Returns the move it threw.
fn full_charge(w: &mut World, button: u16) -> Option<u8> {
    for _ in 0..60 {
        step(w, button);
        if w.players[0].action.attack_kind().is_some_and(c::is_full) {
            break;
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
        (0, Input::RIGHT, c::REPEL),
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
fn the_repel_shoves_them_away_and_hurts_little() {
    // The spear's first finisher is room, not a kill: it hits for less than
    // the spear's own opener and throws them well clear.
    let mut w = open_floor(0, 2);
    let (before, from) = (w.players[1].health, w.players[1].pos);
    assert_eq!(full_charge(&mut w, Input::RIGHT), Some(c::REPEL));
    for _ in 0..40 {
        step(&mut w, 0);
    }
    let dealt = before - w.players[1].health;
    assert!(dealt > 0, "the Repel missed somebody two metres away");
    assert!(
        dealt <= sim::moves::get(Class::Champion, c::SPEAR_GROUND).damage,
        "the Repel dealt {dealt}"
    );
    let pushed = w.players[1].pos.sub(from).flat_len().to_f32_for_render();
    assert!(pushed > 2.0, "and moved them {pushed:.1} m");
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
    let look = |bits| Input::looking_at(bits, LOOK_RIGHT, -3000);
    for _ in 0..60 {
        w.advance([look(Input::MIDDLE), Input::aimed(0, LOOK_LEFT)]);
        if w.players[0].action.attack_kind().is_some_and(c::is_full) {
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
fn momentum_makes_every_hit_worth_more() {
    let tap = |m: i32| {
        let mut w = engaged();
        set_momentum(&mut w, m);
        let before = w.players[1].health;
        for f in 0..30 {
            step(&mut w, if f == 0 { Input::LEFT } else { 0 });
        }
        before - w.players[1].health
    };
    let cold = tap(0);
    let full = tap(sim::tuning::momentum_max());
    assert!(
        full > cold,
        "a sword cut dealt {full} at a full meter and {cold} at none"
    );
}

#[test]
fn a_finisher_spends_the_meter_and_is_worth_what_it_spent() {
    let shockwave = |m: i32| {
        let mut w = open_floor(0, 4);
        set_momentum(&mut w, m);
        let before = w.players[1].health;
        assert_eq!(full_charge(&mut w, Input::MIDDLE), Some(c::SHOCKWAVE));
        assert_eq!(fury(&w).momentum, 0, "a finisher spends the meter");
        let mut staggered = 0;
        for _ in 0..60 {
            step(&mut w, 0);
            if let Action::Stagger { left } = w.players[1].action {
                staggered = staggered.max(left);
            }
        }
        (before - w.players[1].health, staggered, fury(&w).spent)
    };
    let (cold, cold_stagger, nothing) = shockwave(0);
    let (half, half_stagger, some) = shockwave(sim::tuning::momentum_max() / 2);
    let (full, full_stagger, all) = shockwave(sim::tuning::momentum_max());
    assert_eq!((nothing, some, all), (0, 50, 100), "what each spent");
    assert!(cold < half && half < full, "dealt {cold}, {half}, {full}");
    assert!(
        cold_stagger <= half_stagger && half_stagger < full_stagger,
        "staggered {cold_stagger}, {half_stagger}, {full_stagger}"
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
        fury.momentum_left = 250;
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

#[test]
fn any_blow_that_meets_his_shoulder_is_spent_and_its_thrower_staggered() {
    // The generic Clash: a Bulwark's Bash from behind, landing in the last
    // frames of a Rush running away from him. No line of the Bash's own says
    // it can be clashed -- the blow meets the shoulder where every blow lands.
    let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
    let y = w.players[0].pos.y;
    w.players[0].pos = sim::V3::new(sim::Fx::ZERO, y, sim::Fx::from_int(8));
    w.players[1].pos = sim::V3::new(sim::Fx::from_int(-1), y, sim::Fx::from_int(8));
    if let Mechanic::Forms { rush, rush_vel, .. } = &mut w.players[0].mechanic {
        // The run's last frames, held where he is so the Bash reaches.
        *rush = sim::tuning::rush_eat_frames();
        *rush_vel = sim::V3::ZERO;
    }
    let bash = sim::moves::get(Class::Bulwark, 0);
    w.players[1].facing = sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO);
    w.players[1].action = Action::Active {
        kind: 0,
        left: bash.active,
    };
    let health = w.players[0].health;
    let mut staggered = false;
    let mut clashed = false;
    for _ in 0..3 {
        w.advance([Input::aimed(0, LOOK_RIGHT), Input::aimed(0, LOOK_RIGHT)]);
        staggered |= matches!(w.players[1].action, Action::Stagger { .. });
        clashed |= w.players[0].parried > 0;
    }
    assert!(clashed, "the Bash never met him");
    assert_eq!(
        w.players[0].health, health,
        "the Bash took nothing from him"
    );
    assert!(
        staggered,
        "and its thrower was knocked out of it: {:?}",
        w.players[1].action
    );
}

// -- From play, 2026-10-13 --------------------------------------------------

/// Jump, full height, and let go of the button: in the air, with the
/// takeoff behind him, so a click is an aerial.
fn jump(w: &mut World) {
    for _ in 0..12 {
        step(w, Input::SPACE);
    }
    step(w, 0);
    assert!(!w.players[0].grounded, "the jump never left the floor");
}

fn chain(w: &World) -> u8 {
    match w.players[0].mechanic {
        Mechanic::Forms { chain, .. } => chain,
        _ => unreachable!(),
    }
}

#[test]
fn clicking_fast_never_holds() {
    // A second press during a link's wind-up is the next link, never this
    // one held: whatever rhythm the clicks come in, a tapped string throws
    // no finisher.
    for button in [Input::LEFT, Input::MIDDLE, Input::RIGHT] {
        for period in 3..9 {
            for down in 1..period {
                let mut w = engaged();
                for f in 0..150 {
                    step(&mut w, if f % period < down { button } else { 0 });
                    let k = w.players[0].action.attack_kind();
                    assert!(
                        !k.is_some_and(c::is_full),
                        "{button:#x} clicked {down} of every {period} frames threw {k:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_meter_waits_while_he_winds_up_a_hold() {
    let mut w = engaged();
    set_momentum(&mut w, 50);
    if let Mechanic::Forms { fury, .. } = &mut w.players[0].mechanic {
        fury.momentum_left = 0;
    }
    // The press itself is a frame of the meter's ordinary clock; from the
    // first frame of the wind-up it waits.
    step(&mut w, Input::LEFT);
    let pressed = fury(&w).momentum;
    for _ in 1..sim::tuning::hold_from() - 1 {
        step(&mut w, Input::LEFT);
    }
    assert!(pressed > 0);
    assert_eq!(
        fury(&w).momentum,
        pressed,
        "the meter ran down under a hold"
    );
}

#[test]
fn an_aerial_is_the_next_hit_of_the_string() {
    let mut w = open_floor(1, 12);
    jump(&mut w);
    step(&mut w, Input::LEFT);
    assert_eq!(w.players[0].action.attack_kind(), Some(c::AIR_SWORD));
    assert_eq!(chain(&w), 2, "ground, jump: two hits");
}

#[test]
fn two_hits_behind_him_an_aerial_is_the_third() {
    // It is still the air's move, and it is the string's third: the string
    // is over after it, a landed one gives the Rush back, and held it is the
    // grand finisher (`an_aerial_held_is_its_weapons_finisher`).
    let mut w = open_floor(2, 12);
    if let Mechanic::Forms { recharge, .. } = &mut w.players[0].mechanic {
        *recharge = 200;
    }
    jump(&mut w);
    step(&mut w, Input::MIDDLE);
    assert_eq!(w.players[0].action.attack_kind(), Some(c::AIR_HAMMER));
    assert_eq!(
        chain(&w),
        c::DEPTH,
        "ground, jump, third: the string is spent"
    );
    assert_eq!(fury(&w).place, 2);
}

#[test]
fn a_jump_does_not_end_the_string() {
    let mut w = open_floor(1, 12);
    if let Mechanic::Forms { chain_left, .. } = &mut w.players[0].mechanic {
        *chain_left = sim::tuning::chain_grace();
    }
    jump(&mut w);
    assert_eq!(chain(&w), 1, "the string was dropped in the air");
}

#[test]
fn an_aerial_held_is_its_weapons_finisher() {
    for (place, button, want) in [
        (0, Input::LEFT, c::CRESCENT),
        (0, Input::MIDDLE, c::SHOCKWAVE),
        (0, Input::RIGHT, c::REPEL),
        (2, Input::LEFT, c::THOUSAND_CUTS),
        (2, Input::MIDDLE, c::METEOR),
    ] {
        let mut w = open_floor(place, 12);
        jump(&mut w);
        assert_eq!(
            full_charge(&mut w, button),
            Some(want),
            "{button:#x} held in the air at {place}"
        );
    }
}

#[test]
fn the_crescent_off_a_jump_keeps_the_jumps_speed() {
    let mut w = open_floor(0, 12);
    for _ in 0..12 {
        step(&mut w, Input::SPACE | Input::W);
    }
    step(&mut w, Input::W);
    let carried = w.players[0].vel.flat_len().to_f32_for_render();
    assert_eq!(full_charge(&mut w, Input::LEFT), Some(c::CRESCENT));
    let kept = w.players[0].vel.flat_len().to_f32_for_render();
    assert!(
        kept > carried * 0.8,
        "a crescent thrown at {carried:.1} m/s left him at {kept:.1}"
    );
}

#[test]
fn the_hammer_held_in_the_air_slams_him_back_down_and_breaks_the_ground() {
    let mut w = open_floor(0, 4);
    jump(&mut w);
    let before = w.players[1].health;
    assert_eq!(full_charge(&mut w, Input::MIDDLE), Some(c::SHOCKWAVE));
    let mut landed_at = None;
    let mut ring = false;
    for f in 0..60 {
        step(&mut w, 0);
        if w.players[0].grounded && landed_at.is_none() {
            landed_at = Some(f);
        }
        ring |= live(&w, EffectKind::Shockwave);
    }
    let landed = landed_at.expect("he never came down");
    assert!(landed < 15, "he took {landed} frames to come down");
    assert!(ring, "and broke no ground");
    assert!(w.players[1].health < before, "the ring reached nobody");
}

#[test]
fn the_rush_goes_where_the_crosshair_is_not_where_the_stick_is() {
    let mut w = open_floor(0, 30);
    let from = w.players[0].pos;
    // Holding back and to the side, looking ahead.
    step(&mut w, Input::S | Input::A | Input::MECHANIC);
    for _ in 0..6 {
        step(&mut w, Input::S | Input::A);
    }
    let gone = w.players[0].pos.sub(from);
    assert!(
        gone.x.to_f32_for_render() > 2.0 && gone.z.to_f32_for_render().abs() < 0.5,
        "the Rush went {:.1}, {:.1}",
        gone.x.to_f32_for_render(),
        gone.z.to_f32_for_render()
    );
}
