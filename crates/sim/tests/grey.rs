//! Grey health: the Blood mage's lost health is reclaimable, and a weapon.
//!
//! Every cost she pays and every hit she takes turns red into grey; grey fades
//! on its own clock and nothing else takes it off the bar; only a drink from
//! an essence pool turns it back. These pin the bookkeeping, which is the kind
//! of rule that is only worth having if it is true on every frame -- see
//! `docs/design/blood-mage.md` §"Grey health" and `plans/blood-mage-v1.md` M1.

use sim::class::{ALL_CLASSES, Class};
use sim::moves::blood as b;
use sim::moves::blood::keys;
use sim::state::Action;
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    for _ in 0..frames {
        w.advance([Input::aimed(a, LOOK_RIGHT), Input::aimed(b, LOOK_LEFT)]);
    }
}

fn mage() -> World {
    World::with_classes([Class::BloodMage, Class::Bulwark])
}

/// How much of the bar is neither red nor grey.
fn gone(w: &World) -> i32 {
    t::max_health() - w.players[0].health - w.players[0].grey
}

/// The most the fade can take in one frame.
fn fade_ceiling() -> i32 {
    t::grey_fade() / sim::TICK_HZ as i32 + 1
}

#[test]
fn red_plus_grey_plus_gone_is_the_bar_and_gone_only_grows_by_the_fade() {
    // The invariant, on every frame of an exchange that has her casting
    // everything she has while the other fighter hits her back.
    let mut w = mage();
    w.players[1].pos = w.players[0]
        .pos
        .add(V3::new(Fx::ratio(3, 2), Fx::ZERO, Fx::ZERO));
    let mut was_gone = gone(&w);
    // On the three-clicks map
    // (docs/design/exploration/0009_blood_mage_on_three_clicks.md): the left
    // click on the floor is the Blood nova now, where the right click was
    // Haemorrhage -- another cost, and one that spills a pool of her own
    // blood, which must not heal her either.
    let script = [
        (keys::SCYTHE, 30u32),
        (keys::MY_BLOOD, 60),
        (keys::MY_BLOOD, 30),
        (keys::YOUR_BLOOD, 70),
        (keys::SCYTHE, 30),
    ];
    for (button, then) in script {
        for frame in 0..then + 2 {
            let hers = if frame < 2 { button } else { 0 };
            // The Bulwark bashes whenever it can.
            run(&mut w, 1, hers, Input::LEFT);
            let p = w.players[0];
            assert!(p.grey >= 0 && p.health >= 1, "the bar went negative");
            assert_eq!(
                p.health + p.grey + gone(&w),
                t::max_health(),
                "red {} + grey {} + gone {} is not the bar",
                p.health,
                p.grey,
                gone(&w)
            );
            let now_gone = gone(&w);
            assert!(
                now_gone >= was_gone,
                "health came back from nowhere: gone went {was_gone} -> {now_gone}"
            );
            assert!(
                now_gone - was_gone <= fade_ceiling(),
                "gone jumped by {} in one frame, and only the fade takes grey off",
                now_gone - was_gone
            );
            was_gone = now_gone;
        }
    }
    assert!(
        w.players[0].health < t::max_health() - 100,
        "fixture: the exchange hardly touched her"
    );
}

#[test]
fn a_cast_at_full_health_opens_grey_by_its_cost() {
    // Every press she makes is paid in red, in the air and on the floor
    // (docs/design/exploration/0016_blood_mage_thralls_and_the_well.md).
    for (slot, button, airborne) in [
        (b::HAEMORRHAGE, keys::MY_BLOOD, true),
        (b::HAEMORRHAGE, keys::MY_BLOOD, false),
        (b::COMMAND, keys::SCYTHE, false),
        (b::COAGULATE, keys::GATHER, false),
        (b::SUMMON, keys::RAISE, false),
        (b::BLACK_SPIKE, keys::YOUR_BLOOD, false),
    ] {
        let m = sim::moves::get(Class::BloodMage, slot);
        let mut w = mage();
        if airborne {
            w.players[0].pos.y = Fx::from_int(3);
            w.players[0].grounded = false;
        }
        let paid = w.players[0].cost_of(m.cost);
        assert!(paid > 0, "{}: costs nothing at full health", m.name);
        run(&mut w, 2, button, 0);
        assert_eq!(w.players[0].health, t::max_health() - paid, "{}", m.name);
        assert_eq!(
            w.players[0].grey, paid,
            "{}: the cost did not turn grey",
            m.name
        );
    }
}

#[test]
fn a_hit_from_the_other_fighter_turns_red_into_grey_by_what_it_dealt() {
    let mut w = mage();
    // The Bulwark at arm's length, facing her.
    let bash = sim::moves::get(Class::Bulwark, sim::state::SLOT_POKE);
    w.players[1].pos =
        w.players[0]
            .pos
            .add(V3::new(bash.reach.mul(Fx::ratio(3, 4)), Fx::ZERO, Fx::ZERO));
    let full = w.players[0].health;
    let mut landed = None;
    for _ in 0..60 {
        run(&mut w, 1, 0, Input::LEFT);
        if w.players[0].health < full {
            landed = Some(full - w.players[0].health);
            break;
        }
    }
    let dealt = landed.expect("fixture: the bash never landed");
    assert_eq!(
        w.players[0].grey, dealt,
        "hit for {dealt}, and {} of it turned grey",
        w.players[0].grey
    );
}

#[test]
fn grey_fades_and_nothing_but_the_fade_takes_it() {
    let mut w = mage();
    w.players[0].health = t::max_health() - 200;
    w.players[0].grey = 200;
    run(&mut w, 60, 0, 0);
    let after = w.players[0].grey;
    assert!(after < 200, "grey never faded");
    assert_eq!(
        200 - after,
        t::grey_fade(),
        "a second of fading took {} against a rate of {} a second",
        200 - after,
        t::grey_fade()
    );
    assert_eq!(
        w.players[0].health,
        t::max_health() - 200,
        "the fade moved the red line"
    );
}

#[test]
fn a_committed_casts_worth_of_grey_survives_one_exchange() {
    // The starting fade rate, from the design: the grey a committed cast
    // opens -- the spike, the dearest thing she throws -- should still be more
    // than half there after one whole exchange: its own startup, active and
    // recovery, and a dodge on the end of it. Faster and the blade never gets
    // long enough to matter.
    let spike = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE);
    let exchange = spike.whiff_cost() as u32 + t::dodge_frames() as u32;
    let mut w = mage();
    let paid = w.players[0].cost_of(spike.cost);
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(8));
    run(&mut w, 2, keys::YOUR_BLOOD, 0);
    let opened = w.players[0].grey;
    assert_eq!(opened, paid);
    run(&mut w, exchange, 0, 0);
    assert!(
        w.players[0].grey * 2 > opened,
        "of {opened} grey, {} was left after {exchange} frames: the fade eats a \
         cast before it can be used",
        w.players[0].grey
    );
}

#[test]
fn nobody_else_goes_grey() {
    for class in ALL_CLASSES {
        if class == Class::BloodMage {
            continue;
        }
        let mut w = World::with_classes([class, Class::Bulwark]);
        w.players[1].pos = w.players[0]
            .pos
            .add(V3::new(Fx::ratio(3, 2), Fx::ZERO, Fx::ZERO));
        run(&mut w, 60, Input::LEFT, Input::LEFT);
        assert!(
            w.players[0].health < w.players[0].full_health(),
            "fixture: {} was never hit",
            class.name()
        );
        assert_eq!(w.players[0].grey, 0, "{} has grey health", class.name());
    }
}

#[test]
fn a_tap_is_the_quick_thing_and_a_hold_is_the_field() {
    // The buttons, and what each one throws -- on the map of 0016
    // (docs/design/exploration/0016_blood_mage_thralls_and_the_well.md): the
    // left click tapped is the Haemorrhage, held the Hemoplague; the right
    // click tapped is Command, held the Carpet. `tests/blood_mage.rs` holds the
    // whole map.
    let mut w = mage();
    w.players[0].pos.y = Fx::from_int(3);
    w.players[0].grounded = false;
    run(&mut w, 1, keys::MY_BLOOD, 0);
    assert_eq!(w.players[0].action.attack_kind(), Some(b::HAEMORRHAGE));
    let mut w = mage();
    run(&mut w, 1, keys::SCYTHE, 0);
    assert_eq!(w.players[0].action.attack_kind(), Some(b::COMMAND));
    let start = sim::moves::get(Class::BloodMage, b::HAEMORRHAGE).startup as u32;
    let mut w = mage();
    run(&mut w, start + 2, keys::MY_BLOOD, 0);
    assert_eq!(w.players[0].action.attack_kind(), Some(b::HEMOPLAGUE));
    assert!(matches!(w.players[0].action, Action::Channel { .. }));
}

#[test]
fn a_cast_costs_a_share_of_what_she_has_so_the_wounded_pay_less() {
    // A percentage of current red rather than of the bar: at full health a
    // cast opens a big wound, which is how she gets reach quickly, and at low
    // health it opens a small one, so getting back into the fight never costs
    // the fight. And never the last point.
    let spike = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE);
    let at = |health: i32| {
        let mut w = mage();
        w.players[0].health = health;
        w.players[0].grey = 0;
        run(&mut w, 2, keys::YOUR_BLOOD, 0);
        health - w.players[0].health
    };
    let full = at(t::max_health());
    let low = at(t::max_health() / 5);
    assert!(
        full > low * 3,
        "a spike at full health cost {full} and at a fifth {low}"
    );
    assert_eq!(full, t::max_health() * spike.cost / 100);
    assert_eq!(at(1), 0, "a cast at one health took the last point");
}
