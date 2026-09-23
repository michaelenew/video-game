//! Grey health: the Blood mage's lost health is reclaimable, and a weapon.
//!
//! Every cost she pays and every hit she takes turns red into grey; grey fades
//! on its own clock and nothing else takes it off the bar; only a drink from
//! an essence pool turns it back. These pin the bookkeeping, which is the kind
//! of rule that is only worth having if it is true on every frame -- see
//! `docs/design/blood-mage.md` §"Grey health" and `plans/blood-mage-v1.md` M1.

use sim::class::{ALL_CLASSES, Class};
use sim::moves::blood as b;
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
    let script = [
        (Input::LEFT, 30u32),
        (Input::RIGHT, 60),
        (Input::MIDDLE, 30),
        (Input::MECHANIC, 70),
        (Input::LEFT, 30),
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
    for (slot, button) in [
        (b::SWEEP, Input::LEFT),
        (b::HAEMORRHAGE, Input::RIGHT),
        (b::BLOODLETTER, Input::MIDDLE),
        (b::GRASP, Input::SPECIAL),
        (b::BLACK_SPIKE, Input::MECHANIC),
    ] {
        let m = sim::moves::get(Class::BloodMage, slot);
        let mut w = mage();
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
    run(&mut w, 2, Input::MECHANIC, 0);
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
fn the_scythe_is_at_least_half_again_as_long_at_full_grey() {
    // The number to read across the arena. Below this the scaling is not
    // worth the exception it makes to the aiming rules.
    assert!(
        t::grey_reach().raw() >= Fx::ratio(3, 2).raw(),
        "reach at full grey is only x{}",
        t::grey_reach().to_f32_for_render()
    );
    {
        let m = sim::moves::get(Class::BloodMage, b::SWEEP);
        let mut p = sim::state::Player::new(Class::BloodMage);
        let base = sim::state::live_reach(&p, &m);
        assert_eq!(
            base.raw(),
            m.reach.raw(),
            "{}: no grey, and the reach moved",
            m.name
        );
        p.health = 1;
        p.grey = t::max_health() - 1;
        let long = sim::state::live_reach(&p, &m);
        // A bar of grey is one short of full -- red clamps at one -- so the
        // line is read a hair under its top.
        assert!(
            long.raw() >= base.mul(Fx::ratio(149, 100)).raw(),
            "{}: {} m at a full bar of grey against {} m at none",
            m.name,
            long.to_f32_for_render(),
            base.to_f32_for_render()
        );
    }
    // And nothing else she throws grows.
    for slot in [b::BLOODLETTER, b::HAEMORRHAGE, b::GRASP, b::BLACK_SPIKE] {
        let m = sim::moves::get(Class::BloodMage, slot);
        let mut p = sim::state::Player::new(Class::BloodMage);
        p.health = 1;
        p.grey = t::max_health() - 1;
        assert_eq!(
            sim::state::live_reach(&p, &m).raw(),
            m.reach.raw(),
            "{}: grey lengthened something that is not the scythe",
            m.name
        );
    }
}

#[test]
fn the_scythe_hits_harder_the_greyer_she_is() {
    let hit_for = |grey: i32| {
        let mut w = mage();
        w.players[0].health = t::max_health() - grey;
        w.players[0].grey = grey;
        let sweep = sim::moves::get(Class::BloodMage, b::SWEEP);
        w.players[1].pos = w.players[0].pos.add(V3::new(
            sweep.reach.mul(Fx::ratio(1, 2)),
            Fx::ZERO,
            Fx::ZERO,
        ));
        let full = w.players[1].health;
        run(&mut w, 2, Input::LEFT, 0);
        run(&mut w, 30, 0, 0);
        full - w.players[1].health
    };
    let fresh = hit_for(0);
    let grey = hit_for(600);
    assert!(fresh > 0, "fixture: the sweep never landed");
    assert!(
        grey > fresh,
        "at six hundred grey the sweep hit for {grey} against {fresh} at none"
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
            w.players[0].health < t::max_health(),
            "fixture: {} was never hit",
            class.name()
        );
        assert_eq!(w.players[0].grey, 0, "{} has grey health", class.name());
    }
}

#[test]
fn right_click_is_the_haemorrhage_and_left_is_the_sweep() {
    // The three clicks, and what each one throws.
    let mut w = mage();
    run(&mut w, 1, Input::RIGHT, 0);
    assert_eq!(w.players[0].action.attack_kind(), Some(b::HAEMORRHAGE));
    let mut w = mage();
    run(&mut w, 1, Input::LEFT, 0);
    assert_eq!(w.players[0].action.attack_kind(), Some(b::SWEEP));
    let mut w = mage();
    run(&mut w, 1, Input::MIDDLE, 0);
    assert_eq!(w.players[0].action.attack_kind(), Some(b::BLOODLETTER));
    assert!(matches!(w.players[0].action, Action::Startup { .. }));
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
        run(&mut w, 2, Input::MECHANIC, 0);
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

#[test]
fn the_sweeps_volume_widens_with_grey_and_the_row_does_not() {
    // The second thing grey buys, beside the reach: the hit volume's radius,
    // which is what makes a grey mage's swing collect the pools it passes
    // near. Doubled at a full bar is the first number.
    assert!(
        t::grey_width().raw() >= Fx::ratio(3, 2).raw(),
        "width at full grey is only x{}",
        t::grey_width().to_f32_for_render()
    );
    let m = sim::moves::get(Class::BloodMage, b::SWEEP);
    let mut p = sim::state::Player::new(Class::BloodMage);
    let base = sim::state::live_radius(&p, &m);
    assert_eq!(base.raw(), m.radius.raw(), "no grey, and the radius moved");
    p.health = 1;
    p.grey = t::max_health() - 1;
    let wide = sim::state::live_radius(&p, &m);
    assert!(
        wide.raw() >= base.mul(Fx::ratio(149, 100)).raw(),
        "{} m wide at a full bar of grey against {} m at none",
        wide.to_f32_for_render(),
        base.to_f32_for_render()
    );
    // And it is the volume that widens, not just a number beside it.
    p.action = Action::Active {
        kind: b::SWEEP,
        left: m.active,
    };
    let hb = sim::state::hitbox(&p).expect("a swing has a volume");
    assert_eq!(
        hb.radius.raw(),
        wide.raw(),
        "the hit test does not read the live radius"
    );
    for slot in [b::BLOODLETTER, b::HAEMORRHAGE, b::GRASP, b::BLACK_SPIKE] {
        let m = sim::moves::get(Class::BloodMage, slot);
        assert_eq!(
            sim::state::live_radius(&p, &m).raw(),
            m.radius.raw(),
            "{} widened with grey",
            m.name
        );
    }
}
