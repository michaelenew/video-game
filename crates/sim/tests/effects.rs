//! What the classes leave behind.
//!
//! Three of the six are built on things that outlive the move that made them: a
//! fire pillar that grows where it was planted, a drain field that punishes
//! standing still, structures that change the shape of the arena. A fourth is
//! built on taking someone off the ground with you, and a fifth on holding on
//! to them. These are the assertions for all of it, phrased in terms of what
//! the fighters can do to each other rather than which number moved.

use sim::class::{Class, Mechanic};
use sim::effects::EffectKind;
use sim::state::{Action, MAX_PLAYERS};
use sim::{Input, World};

const Q: u16 = Input::SPECIAL;
const E: u16 = Input::MECHANIC;
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    looking(w, frames, a, 0, b);
}

/// The same, with player one holding a vertical aim as well as a horizontal
/// one. A skillshot aimed level goes where level points, which is over the head
/// of anybody more than a few metres away -- see `aiming_at`.
fn looking(w: &mut World, frames: u32, a: u16, pitch: i16, b: u16) {
    for _ in 0..frames {
        w.advance([
            Input::looking_at(a, LOOK_RIGHT, pitch),
            Input::aimed(b, LOOK_LEFT),
        ]);
    }
}

/// The pitch a player would be holding to put the crosshair on a fighter
/// standing at `target`.
///
/// **Searched rather than computed, and that is the point.** Where the
/// crosshair lands is the aim resolver's business: it depends on the ability's
/// reach, on the camera's zones and on how high the eye sits, and every one of
/// those is a knob. A hard-coded angle is a fixture that silently stops
/// pointing at anything the next time one of them moves.
fn aiming_at(w: &World, slot: u8, target: sim::V3) -> i16 {
    let reach = sim::moves::get(w.players[0].class, slot).reach;
    let field = sim::stones::gather(&w.players);
    let middle = sim::V3::new(
        target.x,
        sim::tuning::body_height().div(sim::fixed::Fx::from_int(2)),
        target.z,
    );
    (0..=80)
        .map(|step| -(step * 200) as i16)
        .min_by_key(|pitch| {
            let at = sim::aim::intent(
                w.players[0].pos,
                Input::looking_at(0, LOOK_RIGHT, *pitch),
                reach,
                false,
                &field,
            );
            at.sub(middle).len().raw()
        })
        .expect("the scan is not empty")
}

/// Press a button, let go, and let the move play out. Holding a button down
/// re-throws the move the frame you become free again, which is fine in a match
/// and useless in a test: two pillars is not the thing being asserted.
fn tap(w: &mut World, button: u16, then: u32) {
    run(w, 2, button, 0);
    run(w, then, 0, 0);
}

fn as_class(one: Class) -> World {
    World::with_classes([one, Class::Bulwark])
}

/// Walk player one into player two, so anything with reach connects.
fn engaged(one: Class) -> World {
    let mut w = as_class(one);
    run(&mut w, 90, Input::W, 0);
    w
}

fn effects_of(w: &World, kind: EffectKind) -> Vec<sim::effects::Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == kind)
        .copied()
        .collect()
}

/// Does player one have a structure standing anywhere on the field?
fn has_structure(w: &World) -> bool {
    matches!(w.players[0].mechanic, Mechanic::Structures(slots) if slots.iter().any(|s| s.is_some()))
}

// ---------------------------------------------------------------------------
// The Elementalist
// ---------------------------------------------------------------------------

#[test]
fn the_fire_pillar_stands_after_the_move_is_over() {
    // The point of the class: the attack is over long before the thing it made
    // stops mattering. If the pillar died with the recovery frames it would
    // just be a slow poke.
    let mut w = engaged(Class::Elementalist);
    tap(&mut w, Q, 60);
    assert!(
        w.players[0].action.actionable(),
        "still swinging 60 frames later, so this proves nothing"
    );
    assert_eq!(
        effects_of(&w, EffectKind::FirePillar).len(),
        1,
        "the pillar did not outlive the move that made it"
    );
}

#[test]
fn the_fire_pillar_spreads_at_the_base_and_climbs_at_the_top() {
    // Two volumes doing two jobs. The base gets wide, which is what catches
    // someone walking past it; the column gets tall, which is what stops them
    // jumping over. If both grew the same way the pillar would be one decision
    // instead of two.
    let mut w = engaged(Class::Elementalist);
    tap(&mut w, Q, 30);
    let young = effects_of(&w, EffectKind::FirePillar)[0].pillar_volumes();
    run(&mut w, 120, 0, 0);
    let old = effects_of(&w, EffectKind::FirePillar)[0].pillar_volumes();

    assert!(
        old.0.radius.raw() > young.0.radius.raw(),
        "the base never spread, so there is nothing to walk around"
    );
    assert!(
        old.1.top.raw() > young.1.top.raw(),
        "the column never climbed, so it can be jumped for free forever"
    );
    let base_grew = old.0.radius.raw() - young.0.radius.raw();
    let column_grew = old.1.radius.raw() - young.1.radius.raw();
    assert!(
        base_grew > column_grew,
        "the column widened as much as the base did; the pillar is one threat, not two"
    );
}

#[test]
fn standing_in_a_fire_pillar_costs_you_and_standing_in_your_own_does_not() {
    // The pillar is placed at range, so both fighters have to walk into it --
    // which is the point: it is terrain, and terrain is something you choose to
    // be in. The caster can stand in her own, or she could never fight beside
    // the thing she just made.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, Q, 20); // the pillar lands well ahead of her
    let caster_before = w.players[0].health;
    let victim_before = w.players[1].health;
    run(&mut w, 140, Input::W, Input::W); // both walk in

    assert!(
        w.players[1].health < victim_before,
        "walking into a fire pillar cost nothing"
    );
    assert_eq!(
        w.players[0].health, caster_before,
        "the Elementalist burned herself on her own pillar, so she cannot fight beside it"
    );
}

#[test]
fn a_bolt_aimed_through_a_fire_pillar_hits_as_a_fire_bolt() {
    // The auto reads what it is aimed through. A fire pillar is a hazard, not
    // a wall, so it charges the shot instead of stopping it -- unlike a
    // structure in the same spot. See docs/design/kits/elementalist.md.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, Q, 20); // plant a pillar ahead, along the same aim as Bolt
    assert_eq!(
        effects_of(&w, EffectKind::FirePillar).len(),
        1,
        "fixture planted no pillar to aim through"
    );

    for _ in 0..60 {
        run(&mut w, 1, Input::LEFT, 0);
        if matches!(w.players[0].action, Action::Active { kind: 0, .. }) {
            break;
        }
    }
    assert!(
        matches!(w.players[0].action, Action::Active { kind: 0, .. }),
        "Bolt never became active"
    );
    assert!(
        w.players[0].bolt_fire,
        "a bolt aimed through a fire pillar was not empowered"
    );
    assert!(
        !w.players[0].bolt_blocked,
        "a fire pillar blocked the shot the way a structure does, which it should not"
    );
}

#[test]
fn a_structure_has_no_clock() {
    // This is the regression. Structures used to live in the effects array,
    // which gave them a lifetime: ten seconds after raising one it silently
    // vanished. A structure is a cap-of-three resource, and the only thing
    // that spends it is raising a fourth.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, E, 4);
    assert!(has_structure(&w), "fixture never raised a structure");
    run(&mut w, 3_000, 0, 0); // fifty seconds of doing nothing
    assert!(has_structure(&w), "the structure went away on its own");
}

#[test]
fn casting_the_pillar_never_costs_a_structure() {
    // The other half of the same bug: effects evicted the oldest of *anything*
    // when the board filled, and a structure was the oldest thing on it. Fire
    // pillar does not even need one out any more, but it still must not eat
    // one that happens to be there.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, E, 4);
    for _ in 0..20 {
        tap(&mut w, Q, 12);
        assert!(has_structure(&w), "casting the pillar ate the structure");
    }
}

#[test]
fn one_fighter_cannot_spam_away_the_other_fighters_field() {
    // Eviction reaches your own effects only. Otherwise holding a button would
    // delete someone else's setup, which is not a decision anybody made.
    let mut w = World::with_classes([Class::Elementalist, Class::BloodMage]);
    for _ in 0..90 {
        w.advance([
            Input::aimed(0, LOOK_RIGHT),
            Input::aimed(Input::W, LOOK_LEFT),
        ]);
    }
    for _ in 0..2 {
        w.advance([Input::aimed(0, LOOK_RIGHT), Input::aimed(E, LOOK_LEFT)]);
    }
    run(&mut w, 60, 0, 0);
    assert_eq!(
        effects_of(&w, EffectKind::BlackSpike).len(),
        1,
        "fixture laid no field"
    );

    tap(&mut w, E, 4);
    for _ in 0..20 {
        tap(&mut w, Q, 4);
    }
    assert_eq!(
        effects_of(&w, EffectKind::BlackSpike).len(),
        1,
        "the Elementalist deleted the Blood mage's field by holding a button"
    );
}

#[test]
fn a_fourth_structure_costs_the_first() {
    // The cap is the resource. It is enforced by the mechanic, which is the
    // only thing that owns structures.
    let mut w = as_class(Class::Elementalist);
    for _ in 0..4 {
        tap(&mut w, E, 2);
        run(&mut w, 10, Input::D, 0); // move, so each one lands somewhere new
    }
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("the Elementalist lost her mechanic");
    };
    assert_eq!(
        slots.iter().flatten().count(),
        sim::class::MAX_STRUCTURES,
        "the cap is not a cap"
    );
}

#[test]
fn the_mechanic_fires_on_the_press_not_while_the_button_is_down() {
    // Held, it used to re-fire every frame, and every class was wrong in its
    // own way: the Champion's form became a function of how many frames you
    // happened to hold it, the Reaver's shadow toggled itself back off, the
    // Bulwark's shield was pinned mid-throw and never planted, and the
    // Elementalist spent all three structures on one spot in three frames.
    for class in sim::class::ALL_CLASSES {
        let mut w = World::with_classes([class, Class::Bulwark]);
        let mut held = w.clone();
        run(&mut held, 40, E, 0);
        run(&mut w, 2, E, 0);
        run(&mut w, 38, 0, 0);
        assert_eq!(
            held.players[0].mechanic,
            w.players[0].mechanic,
            "{}: holding the mechanic did something different from tapping it",
            class.name()
        );
    }
}

#[test]
fn holding_the_mechanic_raises_exactly_one_structure() {
    let mut w = as_class(Class::Elementalist);
    run(&mut w, 40, E, 0);
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("the Elementalist lost her mechanic");
    };
    assert_eq!(
        slots.iter().flatten().count(),
        1,
        "one press should raise one structure"
    );
}

#[test]
fn a_second_press_raises_a_second_structure() {
    // The edge must not latch: letting go and pressing again has to work.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, E, 4);
    tap(&mut w, E, 4);
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("the Elementalist lost her mechanic");
    };
    assert_eq!(
        slots.iter().flatten().count(),
        2,
        "the press edge latched, so the mechanic only ever fires once"
    );
}

#[test]
fn a_structure_climbs_out_of_the_ground_and_then_stops_counting() {
    // It is earth. The age drives the rise and nothing else -- it must not
    // become a lifetime by the back door, so it saturates rather than wrapping.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, E, 0);
    let age_of = |w: &World| {
        let Mechanic::Structures(slots) = w.players[0].mechanic else {
            panic!("no mechanic")
        };
        slots.iter().flatten().next().expect("no structure").age
    };
    let fresh = age_of(&w);
    assert!(
        fresh < sim::tuning::structure_rise(),
        "the structure was already up on the frame it was raised"
    );
    run(&mut w, 60, 0, 0);
    assert!(
        age_of(&w) >= sim::tuning::structure_rise(),
        "the structure never finished rising"
    );
    run(&mut w, 3_000, 0, 0);
    assert!(has_structure(&w), "the age turned back into a lifetime");
}

// ---------------------------------------------------------------------------
// The Blood mage
//
// The class spends its own health to cast and earns it back by connecting, so
// most of what is asserted here is a health bar going the right way.
//
// Two fixtures, and both of them **ask where the ability went** rather than
// assuming. Her two placed abilities reach a long way now, and a fixture that
// hard-codes a distance is a fixture that silently stops testing anything the
// next time somebody retunes the reach.
// ---------------------------------------------------------------------------

/// Cast the spike, then stand the other fighter in whatever it left behind.
fn spiked(w: &mut World) {
    run(w, 2, E, 0);
    for _ in 0..120 {
        run(w, 1, 0, 0);
        if let Some(field) = effects_of(w, EffectKind::BlackSpike).first() {
            w.players[1].pos = sim::V3::new(field.pos.x, w.players[1].pos.y, field.pos.z);
            return;
        }
    }
    panic!("the spike never went into the ground");
}

/// Stand the other fighter at the far end of a Grasp, where its four arms
/// converge, and give back the pitch that points at them.
fn in_the_grasp(w: &mut World) -> i16 {
    let reach = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL).reach;
    w.players[1].pos = sim::V3::new(
        w.players[0].pos.x.add(reach),
        w.players[1].pos.y,
        w.players[0].pos.z,
    );
    aiming_at(w, sim::state::SLOT_SPECIAL, w.players[1].pos)
}

#[test]
fn the_black_spike_is_on_the_mechanic_key() {
    // It used to be shift + click, which is the committed-attack slot on every
    // class. `E` is where the thing only this class does belongs, and the Blood
    // mage has nothing else to spend the key on -- her mechanic is health, and
    // health is not a thing you toggle.
    let mut w = engaged(Class::BloodMage);
    tap(&mut w, Input::SHIFT | Input::LEFT, 60);
    assert!(
        effects_of(&w, EffectKind::BlackSpike).is_empty(),
        "shift + click still casts the spike"
    );

    let mut w = engaged(Class::BloodMage);
    tap(&mut w, E, 60);
    assert_eq!(
        effects_of(&w, EffectKind::BlackSpike).len(),
        1,
        "E did not cast the spike"
    );
}

#[test]
fn the_spike_takes_long_enough_to_be_seen_coming() {
    // The cast is the telegraph. A field you can drop on somebody instantly is
    // not a placement decision, it is a trap, and the whole point of the
    // ability is that the other player gets to walk out of the circle.
    let mut w = engaged(Class::BloodMage);
    let startup = sim::moves::get(Class::BloodMage, sim::state::SLOT_MECHANIC).startup;
    assert!(
        startup > sim::tuning::HUMAN_REACTION_FRAMES,
        "the spike comes out inside reaction time, so nobody can answer it"
    );
    run(&mut w, 2, E, 0);
    run(&mut w, (startup - 1) as u32, 0, 0);
    assert!(
        effects_of(&w, EffectKind::BlackSpike).is_empty(),
        "the spike was already in the ground during the wind-up"
    );
}

#[test]
fn the_spike_reaches_much_further_than_a_swing() {
    // It is placed with the crosshair, so the range is how far across the arena
    // you can put a wall. Pinned against the class's own melee rather than
    // against a number, so retuning either one keeps the relationship honest.
    let spike = sim::moves::get(Class::BloodMage, sim::state::SLOT_MECHANIC);
    let rend = sim::moves::get(Class::BloodMage, sim::state::SLOT_COMMITTED);
    assert!(
        spike.reach.raw() > rend.reach.mul(sim::fixed::Fx::from_int(2)).raw(),
        "the spike lands barely further than a claw does: {} against {}",
        spike.reach.to_f32_for_render(),
        rend.reach.to_f32_for_render()
    );
}

#[test]
fn the_black_spike_drains_and_slows_whoever_stands_in_it() {
    // Not a damage puddle: the slow is what makes it a wall. Leaving costs you
    // time, which is the whole reason to put one between yourself and someone.
    let mut w = as_class(Class::BloodMage);
    let before = w.players[1].health;
    spiked(&mut w);
    run(&mut w, 60, 0, 0);
    assert_eq!(
        effects_of(&w, EffectKind::BlackSpike).len(),
        1,
        "black spike left no field"
    );
    assert!(w.players[1].health < before, "the field drained nobody");
    assert!(
        w.players[1].slowed > 0,
        "the field did not slow, so walking out of it is free"
    );
}

#[test]
fn the_spike_feeds_the_caster_while_it_drains() {
    // The class's whole loop: blood out on the press, blood back while it
    // works. Continuous rather than a lump sum when the field expires, so a
    // Blood mage standing in a fight is being paid the whole time it is up.
    let mut w = as_class(Class::BloodMage);
    let spike = sim::moves::get(Class::BloodMage, sim::state::SLOT_MECHANIC);
    assert!(spike.leech > 0, "the spike returns nothing at all");

    spiked(&mut w);
    // Hurt, so there is room on the bar for the return to show.
    w.players[0].health = sim::tuning::max_health() / 2;
    let paid = w.players[0].health;
    let victim = w.players[1].health;
    run(&mut w, 60, 0, 0);
    assert!(
        w.players[1].health < victim,
        "fixture: the field never drained anybody"
    );
    assert!(
        w.players[0].health > paid,
        "the spike drained {} and gave the caster none of it",
        victim - w.players[1].health
    );
}

#[test]
fn casting_costs_the_blood_mage_health() {
    // Every one of her abilities is paid for out of the bar, which is the class
    // -- see `docs/design/kits/blood-mage.md`. Asserted on all four rather than
    // on one, because "all of them" is the design and a free ability would be
    // the one everybody pressed.
    use sim::state::{SLOT_COMMITTED, SLOT_MECHANIC, SLOT_POKE, SLOT_SPECIAL};
    for (slot, button) in [
        (SLOT_POKE, Input::LEFT),
        (SLOT_COMMITTED, Input::SHIFT | Input::LEFT),
        (SLOT_SPECIAL, Q),
        (SLOT_MECHANIC, E),
    ] {
        let m = sim::moves::get(Class::BloodMage, slot);
        assert!(m.cost > 0, "{} is free to cast", m.name);

        // Cast it at nothing, so the only thing that can move the bar is the
        // price of pressing the button.
        let mut w = as_class(Class::BloodMage);
        let before = w.players[0].health;
        run(&mut w, 2, button, 0);
        assert_eq!(
            w.players[0].health,
            before - m.cost,
            "{} did not cost what the table says",
            m.name
        );
    }
}

#[test]
fn spending_health_cannot_kill_you() {
    // Dying to your own button is not a decision anybody made. The same rule
    // the Dual mage's meter burn already follows.
    let mut w = as_class(Class::BloodMage);
    // Nobody within reach of anything, so the bar can only go one way.
    w.players[1].pos = sim::V3::new(
        sim::fixed::Fx::from_int(-18),
        w.players[1].pos.y,
        w.players[1].pos.z,
    );
    w.players[0].health = 1;
    for _ in 0..20 {
        tap(&mut w, E, 60);
        assert!(
            w.players[0].health > 0,
            "the Blood mage cast herself to death"
        );
    }
    assert_eq!(w.players[0].health, 1, "self-damage did not clamp at one");
}

#[test]
fn the_bloodletter_cuts_on_the_way_out_and_on_the_way_back() {
    // The auto, and the simplest statement of what the class is: a blade goes
    // out, comes back, and the blood comes home with it.
    let mut w = as_class(Class::BloodMage);
    let reach = sim::moves::get(Class::BloodMage, sim::state::SLOT_POKE).reach;
    // Parked halfway along the throw, so the blade passes through them twice.
    w.players[1].pos = sim::V3::new(
        w.players[0]
            .pos
            .x
            .add(reach.div(sim::fixed::Fx::from_int(2))),
        w.players[1].pos.y,
        w.players[0].pos.z,
    );

    let full = w.players[1].health;
    let pitch = aiming_at(&w, sim::state::SLOT_POKE, w.players[1].pos);
    looking(&mut w, 2, Input::LEFT, pitch, 0);
    let mut cuts = 0;
    let mut last = full;
    for _ in 0..120 {
        run(&mut w, 1, 0, 0);
        if w.players[1].health < last {
            cuts += 1;
            last = w.players[1].health;
        }
    }
    assert_eq!(
        cuts, 2,
        "the blade landed {cuts} hits, not one out and one back"
    );
}

#[test]
fn the_bloodletter_pays_out_when_it_is_caught() {
    // Not on contact. The cut lands at once and the health has to survive the
    // flight home, which is what makes an auto attack a small commitment
    // instead of a free poke.
    let mut w = as_class(Class::BloodMage);
    let m = sim::moves::get(Class::BloodMage, sim::state::SLOT_POKE);
    w.players[1].pos = sim::V3::new(
        w.players[0]
            .pos
            .x
            .add(m.reach.div(sim::fixed::Fx::from_int(2))),
        w.players[1].pos.y,
        w.players[0].pos.z,
    );
    // Hurt, so there is room on the bar for the return to be visible.
    w.players[0].health = sim::tuning::max_health() / 2;

    let pitch = aiming_at(&w, sim::state::SLOT_POKE, w.players[1].pos);
    looking(&mut w, 2, Input::LEFT, pitch, 0);
    let flight = sim::tuning::bloodletter_flight();
    run(&mut w, (m.startup + m.active) as u32, 0, 0);
    let mid = w.players[0].health;
    run(&mut w, flight as u32 / 2, 0, 0);
    assert!(
        w.players[1].health < sim::tuning::max_health(),
        "fixture: the blade never cut anybody"
    );
    assert_eq!(
        w.players[0].health, mid,
        "the blade paid out before it came home"
    );
    run(&mut w, flight as u32, 0, 0);
    assert!(
        w.players[0].health > mid,
        "the blade came home and brought nothing with it"
    );
}

#[test]
fn a_grasp_roots_only_when_every_arm_lands() {
    // Four arms, and the root is the price of all four. One or two of them is a
    // glancing blow; standing where the cone closes is a read, and a read is
    // what the design lets a hard stop be bought with.
    let mut w = as_class(Class::BloodMage);
    let pitch = in_the_grasp(&mut w);
    looking(&mut w, 2, Q, pitch, 0);
    run(&mut w, 60, 0, 0);
    assert!(
        w.players[1].rooted > 0,
        "caught by every arm and still walking"
    );

    // Far off to one side: the cone never reaches, so nothing lands.
    let mut w = as_class(Class::BloodMage);
    w.players[1].pos = sim::V3::new(
        w.players[0].pos.x,
        w.players[1].pos.y,
        w.players[0].pos.z.add(sim::fixed::Fx::from_int(9)),
    );
    let full = w.players[1].health;
    tap(&mut w, Q, 60);
    assert_eq!(
        w.players[1].health, full,
        "the arms reached across the arena"
    );
    assert_eq!(w.players[1].rooted, 0, "rooted by a Grasp that missed");
}

#[test]
fn a_rooted_fighter_cannot_walk_dodge_or_jump() {
    // What separates a root from a very heavy slow: it takes the two buttons
    // that would otherwise be the way out. It is not a stun -- you can still
    // turn and swing at whoever put the arms round your legs.
    let mut w = as_class(Class::BloodMage);
    let pitch = in_the_grasp(&mut w);
    looking(&mut w, 2, Q, pitch, 0);
    // Past the arms and past the hitstun they came with. A root that expired
    // inside its own hitstun would never be seen at all, which is why
    // `a_root_outlives_the_hitstun_that_delivers_it` pins the two apart.
    for _ in 0..200 {
        run(&mut w, 1, 0, 0);
        if w.players[1].action.actionable() && w.players[1].rooted > 0 {
            break;
        }
    }
    assert!(w.players[1].rooted > 0, "fixture rooted nobody");
    assert!(
        w.players[1].action.actionable(),
        "the root stunned instead of pinning: you can still swing while held"
    );

    let start = w.players[1].pos;
    run(&mut w, 6, 0, Input::W);
    assert_eq!(
        w.players[1].pos.x.raw(),
        start.x.raw(),
        "a rooted fighter walked"
    );
    run(&mut w, 2, 0, Input::SHIFT | Input::W);
    assert!(
        !matches!(w.players[1].action, Action::Dodge { .. }),
        "a rooted fighter dodged out of it"
    );
    run(&mut w, 2, 0, Input::SPACE);
    assert!(w.players[1].grounded, "a rooted fighter jumped out of it");

    // And it ends.
    run(&mut w, sim::tuning::grasp_root() as u32 + 2, 0, 0);
    assert_eq!(w.players[1].rooted, 0, "the root never wore off");
    run(&mut w, 6, 0, Input::W);
    assert!(
        w.players[1].pos.x.raw() != start.x.raw(),
        "the feet never came back"
    );
}

/// What one swing of `slot` takes off a fighter who has been put into `setup`.
///
/// The comparison the trait is about: the same move, the same distance, the
/// same frame, against a victim whose options are gone and against one whose
/// are not.
fn one_hit(setup: impl Fn(&mut World)) -> i32 {
    let mut w = engaged(Class::BloodMage);
    run(&mut w, 20, 0, 0);
    setup(&mut w);
    let before = w.players[1].health;
    let m = sim::moves::get(Class::BloodMage, sim::state::SLOT_COMMITTED);
    run(&mut w, 2, Input::SHIFT | Input::LEFT, 0);
    run(&mut w, (m.startup + m.active + 2) as u32, 0, 0);
    before - w.players[1].health
}

#[test]
fn a_blood_mage_hits_harder_when_you_cannot_move() {
    // The class's damage identity, out of the archive: *naturally deals
    // increased damage on disabled enemies*. It is what turns the Grasp's root
    // from a small reward into a setup -- four arms is expensive, and it is
    // only worth the cost if something is waiting on the other side of it.
    let free = one_hit(|_| {});
    let rooted = one_hit(|w| w.players[1].root(120));
    assert!(free > 0, "fixture: the claw did not connect at all");
    assert!(
        rooted > free,
        "a rooted fighter took {rooted} where a free one took {free}"
    );

    let expected = sim::fixed::Fx::from_int(free)
        .mul(sim::tuning::disabled_damage_mul())
        .to_int();
    assert!(
        (rooted - expected).abs() <= 1,
        "the bonus is {rooted} against {free}, which is not the knob"
    );
}

#[test]
fn hitstun_is_not_a_disable() {
    // The line the whole definition rests on. Hitstun happens on every hit
    // anybody lands, so counting it would make the trait "increased damage
    // from the second hit onward" -- a flat damage bonus in a costume, and one
    // that would need no read at all.
    let stunned = one_hit(|w| {
        w.players[1].action = Action::HitStun { left: 90 };
    });
    let free = one_hit(|_| {});
    assert_eq!(
        stunned, free,
        "being in hitstun counted as being disabled, so the bonus is free"
    );
}

#[test]
fn nobody_else_preys_on_the_disabled() {
    // A second class quietly acquiring it would mean the trait had stopped
    // being an identity.
    use sim::class::ALL_CLASSES;
    for class in ALL_CLASSES {
        assert_eq!(
            class.preys_on_the_disabled(),
            class == Class::BloodMage,
            "{} hits the disabled harder",
            class.name()
        );
    }
}

#[test]
fn the_grasp_sets_up_its_own_payoff() {
    // The two halves together, which is the point of implementing either. Root
    // them with every arm, then hit them while they are held there.
    let mut w = as_class(Class::BloodMage);
    let pitch = in_the_grasp(&mut w);
    looking(&mut w, 2, Q, pitch, 0);
    run(&mut w, 60, 0, 0);
    assert!(w.players[1].rooted > 0, "fixture rooted nobody");
    assert!(
        w.players[1].disabled(),
        "the root does not count as a disable, so the payoff never fires"
    );
}

#[test]
fn a_slowed_fighter_covers_less_ground() {
    let mut w = as_class(Class::BloodMage);
    spiked(&mut w);
    run(&mut w, 20, 0, 0);
    assert!(w.players[1].slowed > 0, "fixture did not slow anyone");

    let start = w.players[1].pos;
    run(&mut w, 6, 0, Input::W);
    let slowed = w.players[1].pos.sub(start).flat_len();

    // Out of the field, under the same input, for the same number of frames.
    run(&mut w, 240, 0, 0);
    let start = w.players[1].pos;
    run(&mut w, 6, 0, Input::W);
    let free = w.players[1].pos.sub(start).flat_len();

    assert!(
        free.raw() > slowed.raw(),
        "the slow did not slow: {} free vs {} in the field",
        free.to_f32_for_render(),
        slowed.to_f32_for_render()
    );
}

// ---------------------------------------------------------------------------
// The Champion
// ---------------------------------------------------------------------------

#[test]
fn the_uppercut_takes_both_fighters_off_the_ground() {
    // The move is a leap, and it is a leap you bring someone along on. Either
    // half alone is a different move: without the lift it is a launcher you
    // cannot follow up on, and without the launch it is an escape.
    let mut w = engaged(Class::Champion);
    let mut lifted = [false; MAX_PLAYERS];
    run(&mut w, 2, Q, 0);
    for _ in 0..60 {
        w.advance([Input::aimed(0, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        for (i, seen) in lifted.iter_mut().enumerate() {
            *seen |= !w.players[i].grounded;
        }
    }
    assert!(lifted[0], "the uppercut never left the ground");
    assert!(
        lifted[1],
        "the uppercut connected but left its victim standing, so there is nothing to chase"
    );
}

// ---------------------------------------------------------------------------
// The Bulwark
// ---------------------------------------------------------------------------

#[test]
fn the_grapple_holds_on_and_its_victim_cannot_walk_out_of_it() {
    // A grab is not a shove. The victim is pinned to the grabber at arm's
    // length and stays there while they struggle, which is what makes beating
    // guard with it worth the commitment.
    let mut w = engaged(Class::Bulwark);
    run(&mut w, 2, Q, 0);
    let mut grabbed = false;
    for _ in 0..40 {
        w.advance([Input::aimed(0, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        grabbed |= matches!(w.players[1].action, Action::Held { .. });
        if grabbed {
            break;
        }
    }
    assert!(grabbed, "the grapple did not grab anyone");
    assert_eq!(
        w.players[1].held_by, 0,
        "somebody is held and nobody is holding them"
    );

    // Now let the victim try to run. Away is `S` for them -- they spawn facing
    // the other way.
    let gap_before = w.players[1].pos.sub(w.players[0].pos).flat_len();
    run(&mut w, 8, 0, Input::S);
    let gap_after = w.players[1].pos.sub(w.players[0].pos).flat_len();
    assert!(
        matches!(w.players[1].action, Action::Held { .. }),
        "the hold broke the moment its victim pressed a key"
    );
    let drift = (gap_after.raw() - gap_before.raw()).abs();
    assert!(
        drift < (sim::tuning::body_radius().raw() / 4),
        "the victim walked out of the grab: gap went {} -> {}",
        gap_before.to_f32_for_render(),
        gap_after.to_f32_for_render()
    );
}

#[test]
fn being_held_ends_and_hands_you_back_your_feet() {
    let mut w = engaged(Class::Bulwark);
    tap(&mut w, Q, 180);
    assert!(
        !matches!(w.players[1].action, Action::Held { .. }),
        "the grab never let go"
    );
    assert_eq!(
        w.players[1].held_by,
        sim::state::NOBODY,
        "let go but still tethered"
    );
}

// ---------------------------------------------------------------------------
// Rollback
// ---------------------------------------------------------------------------

#[test]
fn a_pillar_is_part_of_the_state_a_rollback_restores() {
    // If effects were not in the checksum, a peer could have a pillar the other
    // does not and neither would notice until someone died to it.
    let mut w = engaged(Class::Elementalist);
    tap(&mut w, E, 4);
    let bare = w.checksum();
    tap(&mut w, Q, 40);
    assert!(
        !effects_of(&w, EffectKind::FirePillar).is_empty(),
        "fixture made no pillar"
    );
    assert_ne!(
        bare,
        w.checksum(),
        "a world with a fire pillar in it hashes the same as one without"
    );

    let saved = w.clone();
    run(&mut w, 30, 0, 0);
    let advanced = w.checksum();
    w = saved.clone();
    assert_eq!(saved.checksum(), w.checksum(), "restore lost the effects");
    assert_ne!(advanced, w.checksum(), "the restore did not go back");
}
