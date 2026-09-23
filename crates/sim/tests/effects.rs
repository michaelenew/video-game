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
use sim::moves;
use sim::state::{Action, MAX_PLAYERS};
use sim::{Fx, Input, World};

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
    let m = sim::moves::get(w.players[0].class, slot);
    let reach = m.reach;
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let scene = sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
    };
    // Their middle, measured from *their* feet. Not from the world's floor: a
    // fighter standing on a platform is a metre and a half up, and aiming at
    // half a body height above the arena floor is aiming into the platform
    // they are standing on.
    let middle = sim::V3::new(
        target.x,
        target
            .y
            .add(sim::tuning::body_height().div(sim::fixed::Fx::from_int(2))),
        target.z,
    );
    // How close the *line* passes to them, rather than how close its far end
    // lands. Bodies came off the aiming ray on 2026-09-13 -- a creature up
    // close fills the screen and the reticle ends up on its chest, metres above
    // the thing you meant to hit -- so a shot aimed at somebody now ends on the
    // floor behind them and passes through them on the way. Nearest approach is
    // what "the reticle is on them" has always meant to the player; it used to
    // be possible to spell it as "the ray stopped there".
    // Measured along the move's *own* flight, which is the crosshair's direction
    // for the move's full reach -- "a blade thrown at something four metres away
    // still flies its full distance; the crosshair picked the line." So a wall
    // cutting the aiming ray short does not move the line the blade takes.
    // Through the move's **own** kind of aiming, which is declared in the
    // table. A fixture that always asked `skillshot_path` would be a second
    // aiming model living in the tests, and the first move to change its
    // column would start missing for reasons nothing in the game shares.
    let miss = |pitch: i16| {
        let look = Input::looking_at(0, LOOK_RIGHT, pitch);
        let one = &players[0];
        let path = match m.aim() {
            sim::aim::Kind::Swing => {
                sim::aim::swing_path(one.pos, one.facing, look, one.grounded, reach, m.hand)
            }
            _ => sim::aim::skillshot_path(0, look, reach, &scene),
        };
        let dir = path.dir();
        let toward = middle.sub(path.from);
        let down = toward.dot(dir).max(sim::fixed::Fx::ZERO).min(reach);
        path.from.add(dir.scale(down)).sub(middle).len().raw()
    };
    // Both ways from level. A swing pitched by the dead zone is flat through
    // the whole of the first sweep, so a target standing above the caster can
    // only be found by looking up at it.
    (-40..=80)
        .map(|step| -(step * 200) as i16)
        .min_by_key(|pitch| miss(*pitch))
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
fn an_auto_aimed_through_a_fire_pillar_lights_a_fire_bolt() {
    // The auto reads what it is aimed through. A fire pillar is a hazard, not
    // a wall, so it does not stop the beam -- it lights one, and what leaves
    // the pillar is a real projectile with a speed and a long range. See
    // docs/design/kits/elementalist.md.
    let mut w = as_class(Class::Elementalist);
    // Clear of the raised platforms, which reach four metres either side of
    // the middle: a pillar planted on one stands a platform's height up and a
    // level shot correctly passes underneath it. And the other fighter well
    // out of the way, so what the beam meets first is the fire.
    w.players[0].pos = sim::V3::new(Fx::from_int(-10), Fx::ZERO, Fx::from_int(8));
    w.players[1].pos = sim::V3::new(Fx::from_int(12), Fx::ZERO, Fx::from_int(8));
    tap(&mut w, Q, 60); // plant a pillar ahead, and let her recover from it
    let pillars = effects_of(&w, EffectKind::FirePillar);
    assert_eq!(pillars.len(), 1, "fixture planted no pillar to aim through");
    assert!(
        w.bolts.iter().all(|b| b.is_none()),
        "fixture started with a bolt already in the air"
    );

    // The pillar reaches further than the beam does, so she has to close
    // before she can shoot through her own fire. Walked rather than assumed:
    // both ranges are tuned, and a fixture that took the gap on faith would
    // start passing or failing for reasons that have nothing to do with it.
    let reach = sim::moves::get(Class::Elementalist, 0).reach;
    let gap = |w: &World| pillars[0].pos.sub(w.players[0].pos).flat_len();
    for _ in 0..240 {
        if gap(&w).raw() < reach.raw() {
            break;
        }
        run(&mut w, 1, Input::W, 0);
    }
    assert!(
        gap(&w).raw() < reach.raw(),
        "fixture never got within the beam's own range of the pillar"
    );

    for _ in 0..60 {
        run(&mut w, 1, Input::LEFT, 0);
        if matches!(w.players[0].action, Action::Active { kind: 0, .. }) {
            break;
        }
    }
    assert!(
        matches!(w.players[0].action, Action::Active { kind: 0, .. }),
        "the auto never became active"
    );
    let lit = w.bolts.iter().flatten().next().copied();
    let lit = lit.expect("aiming the auto through a fire pillar lit no fire bolt");

    // It comes *from the fire*, not from her hand. Anything else and the
    // interaction is invisible: the player would see a bolt leave the
    // Elementalist and have no way to know the pillar had anything to do
    // with it.
    let live = effects_of(&w, EffectKind::FirePillar)[0];
    let edge = live.pillar_volumes().0.radius;
    let off = lit.pos.sub(live.pos).flat_len();
    assert!(
        off.raw() <= edge.add(sim::tuning::fire_bolt_radius()).raw(),
        "the bolt was lit {} m from the pillar, whose base is {} m across",
        off.to_f32_for_render(),
        edge.to_f32_for_render()
    );

    // And it flies. The beam is instant; this is the one part with a speed.
    let before = lit.pos;
    run(&mut w, 3, 0, 0);
    let moved = w.bolts.iter().flatten().next().copied();
    let moved = moved.expect("the fire bolt vanished before it had gone anywhere");
    assert!(
        moved.pos.sub(before).len().raw() > 0,
        "the fire bolt never left the pillar"
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
    // Let the world catch up before aiming. A full grasp's reach from the spawn
    // mark lands on top of one of the raised platforms, so a fighter put there
    // is standing a metre and a half up by the next frame -- and aiming at
    // where they were placed rather than where they end up is aiming at the
    // platform's flank.
    looking(w, 2, 0, 0, 0);
    aiming_at(w, sim::state::SLOT_SPECIAL, w.players[1].pos)
}

/// Wind the Grasp all the way out and let go.
///
/// A channelled move is thrown by the **release**, and how long the button was
/// held is its range -- see `state::step_channel`. Every fixture here wants the
/// far end of that slider, because that is where `in_the_grasp` stands the
/// other fighter. Pressing and letting go, which is how every other move in
/// these tests is thrown, would send the arms out three metres.
fn grasp(w: &mut World, pitch: i16) {
    let hold = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL).channel;
    looking(w, hold as u32 + 1, Q, pitch, 0);
    looking(w, 1, 0, pitch, 0);
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
    // Rend is in the list and has no button: shift stopped being an attack
    // modifier, so the committed slot is stranded on the three classes that
    // still keep a move there. The table is still the design -- every one of
    // her abilities is paid for -- so the cost is asserted on all four and only
    // the three with an input are thrown. See `docs/design/controls.md`.
    for (slot, button) in [
        (SLOT_POKE, Some(Input::LEFT)),
        (SLOT_COMMITTED, None),
        (SLOT_SPECIAL, Some(Q)),
        (SLOT_MECHANIC, Some(E)),
    ] {
        let m = sim::moves::get(Class::BloodMage, slot);
        assert!(m.cost > 0, "{} is free to cast", m.name);
        let Some(button) = button else { continue };

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
fn the_blade_comes_back_to_the_mage_and_not_to_the_spot_she_threw_it_from() {
    // A catch happens between two objects. A blade returning to a patch of air
    // its caster walked out of three quarters of a second ago has not been
    // caught by anybody, and paying for one that was not caught makes the
    // flight home a formality rather than the risk it is meant to be.
    //
    // Measured as *where the blade ends up*, against where it would have ended
    // up under the old rule: the throw point is still there to compare with.
    let mut w = as_class(Class::BloodMage);
    let m = sim::moves::get(Class::BloodMage, sim::state::SLOT_POKE);
    looking(&mut w, 2, Input::LEFT, 0, 0);
    run(&mut w, (m.startup + m.active) as u32, 0, 0);
    let thrown_from = effects_of(&w, EffectKind::Bloodletter)
        .first()
        .expect("the blade never left")
        .pos;

    // Walk sideways for the whole of the flight, so the mage is nowhere near
    // where the blade left her hand by the time it arrives.
    let flight = sim::tuning::bloodletter_flight();
    let mut last = None;
    for _ in 0..flight * 2 {
        run(&mut w, 1, 0, 0);
        looking(&mut w, 1, Input::D, 0, 0);
        if let Some(blade) = effects_of(&w, EffectKind::Bloodletter).first() {
            last = Some((blade.blade_at(), w.players[0].pos));
        }
    }
    let (blade, mage) = last.expect("the blade was never in the air");
    let walked = mage.sub(thrown_from).flat_len();
    assert!(
        walked.raw() > sim::fixed::Fx::from_int(2).raw(),
        "fixture: the mage only moved {} m, which is not far enough to tell the \
         two answers apart",
        walked.to_f32_for_render()
    );
    let to_mage = blade.sub(mage).flat_len();
    let to_throw = blade.sub(thrown_from).flat_len();
    assert!(
        to_mage.raw() < to_throw.raw(),
        "the blade finished {} m from the mage and {} m from where she threw \
         it, so it came home to the spot rather than to her",
        to_mage.to_f32_for_render(),
        to_throw.to_f32_for_render()
    );
}

#[test]
fn the_blade_tracks_the_mage_the_whole_way_home_rather_than_snapping_to_her() {
    // "Continually tracks" rather than "ends up in the right place". A return
    // that flew to the old spot and then jumped the last few metres would pass
    // the test above and read, in the hand, as a bug.
    let mut w = as_class(Class::BloodMage);
    let m = sim::moves::get(Class::BloodMage, sim::state::SLOT_POKE);
    looking(&mut w, 2, Input::LEFT, 0, 0);
    run(&mut w, (m.startup + m.active) as u32, 0, 0);

    let flight = sim::tuning::bloodletter_flight();
    let mut gaps = Vec::new();
    let mut steps = Vec::new();
    let mut was: Option<sim::V3> = None;
    for _ in 0..flight * 2 {
        looking(&mut w, 1, Input::D, 0, 0);
        let out = effects_of(&w, EffectKind::Bloodletter);
        let Some(blade) = out.first() else {
            continue;
        };
        let at = blade.blade_at();
        if blade.returning() {
            gaps.push(at.sub(w.players[0].pos).flat_len());
        }
        if let Some(was) = was {
            steps.push(at.sub(was).len());
        }
        was = Some(at);
    }
    assert!(gaps.len() > 4, "fixture: the blade never came back");

    // Closing on her every frame of the way in, never opening up.
    for pair in gaps.windows(2) {
        assert!(
            pair[1].raw() <= pair[0].raw(),
            "the gap went from {} m to {} m on the way home",
            pair[0].to_f32_for_render(),
            pair[1].to_f32_for_render()
        );
    }

    // And no frame that is a teleport. The blade covers its reach in half its
    // life, so a step worth more than three of those is not a blade flying.
    let even = m.reach.div(sim::fixed::Fx::from_int(flight as i32 / 2));
    for step in &steps {
        assert!(
            step.raw() < even.raw() * 3,
            "one frame moved the blade {} m against an even {} m",
            step.to_f32_for_render(),
            even.to_f32_for_render()
        );
    }
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
        w.players[1].health < w.players[1].full_health(),
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
fn holding_the_grasp_longer_sends_it_further() {
    // The channel **is** the aiming, and it is the only place in the game where
    // the cast button chooses a range. So the property is the plain one: the
    // arms converge further out the longer the wind-up, from the move's near
    // knob at no hold to its own reach at the cap.
    let m = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL);
    assert!(m.channel > 0, "the Grasp does not channel");

    let thrown = |hold: u16| {
        let mut w = as_class(Class::BloodMage);
        let pitch = in_the_grasp(&mut w);
        looking(&mut w, hold as u32 + 1, Q, pitch, 0);
        looking(&mut w, 1, 0, pitch, 0);
        for _ in 0..90 {
            run(&mut w, 1, 0, 0);
            if let Some(arms) = effects_of(&w, EffectKind::Grasp).first() {
                return arms.reach;
            }
        }
        panic!("the arms never came out");
    };

    let (near, half, far) = (thrown(0), thrown(m.channel / 2), thrown(m.channel));
    assert!(
        near.raw() < half.raw() && half.raw() < far.raw(),
        "no hold reaches {} m, half {} m, full {} m -- the slider is not a slider",
        near.to_f32_for_render(),
        half.to_f32_for_render(),
        far.to_f32_for_render()
    );
    assert!(
        (far.sub(m.reach)).abs().raw() < sim::fixed::Fx::ratio(1, 4).raw(),
        "a full hold reaches {} m against a move that says {} m",
        far.to_f32_for_render(),
        m.reach.to_f32_for_render()
    );
    assert!(
        (near.sub(m.channel_from)).abs().raw() < sim::fixed::Fx::ratio(1, 4).raw(),
        "no hold at all reaches {} m against a near end of {} m",
        near.to_f32_for_render(),
        m.channel_from.to_f32_for_render()
    );
}

#[test]
fn a_still_mouse_holds_the_line_and_only_the_marker_moves() {
    // What the channel is *for*, and the one thing it has to do to be readable.
    //
    // The aim is solved at the move's **full** reach every frame and the hold
    // picks a point along the line that comes back. Solving at the wound-up
    // range instead -- which is what it did first -- means the raycast's own
    // answer changes as the range grows: the far end walks off the floor and
    // onto a wall and back, so a player holding the mouse perfectly still
    // watched the marker jump about while choosing a depth. With the line held
    // still, the only thing moving is the thing the player is moving.
    let m = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL);
    for pitch in [4096i16, 0, -4096, -8192, -12000] {
        let mut w = as_class(Class::BloodMage);
        let mut line: Option<sim::V3> = None;
        let mut was = sim::fixed::Fx::ZERO;
        for held in 0..=m.channel {
            looking(&mut w, 1, Q, pitch, 0);
            if w.players[0].action.channelling().is_none() {
                continue;
            }
            let path = w.players[0].aim_path;
            let (dir, far) = (path.dir(), path.length());
            if let Some(line) = line {
                // The same line, frame after frame. A hair of rounding is fine;
                // a surface change is not, and shows up here as a direction
                // that swings.
                let swing = dir.sub(line).len();
                assert!(
                    swing.raw() < sim::fixed::Fx::ratio(1, 100).raw(),
                    "held {held} at pitch {pitch}: the line moved by {} while \
                     the mouse was still",
                    swing.to_f32_for_render()
                );
                // And the marker walks out, never back, never in a jump.
                assert!(
                    far.raw() >= was.raw(),
                    "held {held} at pitch {pitch}: the marker went from {} m \
                     back to {} m",
                    was.to_f32_for_render(),
                    far.to_f32_for_render()
                );
                let step = far.sub(was);
                assert!(
                    step.raw() < sim::fixed::Fx::ratio(1, 2).raw(),
                    "held {held} at pitch {pitch}: one frame moved the marker \
                     {} m, which is a jump rather than a walk",
                    step.to_f32_for_render()
                );
            }
            line = Some(dir);
            was = far;
        }
        assert!(line.is_some(), "the channel never opened at pitch {pitch}");
    }
}

#[test]
fn the_marker_is_as_far_out_as_the_hold_and_nothing_else() {
    // The other half of the split. The crosshair answers *which way*; the hold
    // answers *how far*, on its own, and it does not ask what the ray stopped
    // on. Aim at a wall two metres away and wind to full range and it is still
    // a ten-metre Grasp -- it goes through the wall. A wall is a thing to punch
    // an ability through, not a shorter version of the ability.
    //
    // Swept across pitches that put wildly different things under the reticle:
    // open air at the top, the floor a few metres out at the bottom. The
    // answer has to be the same number every time.
    let m = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL);
    for pitch in [4096i16, 0, -4096, -8192, -12000, -16384] {
        let mut w = as_class(Class::BloodMage);
        for held in 0..=m.channel {
            looking(&mut w, 1, Q, pitch, 0);
            let Some((_, wound)) = w.players[0].action.channelling() else {
                continue;
            };
            let want = m.reach_after(wound);
            let got = w.players[0].aim_path.length();
            assert!(
                got.sub(want).abs().raw() < sim::fixed::Fx::ratio(1, 100).raw(),
                "held {held} at pitch {pitch}: the marker is {} m out where the \
                 hold says {} m",
                got.to_f32_for_render(),
                want.to_f32_for_render()
            );
        }
    }
}

#[test]
fn a_grasp_wound_to_full_range_reaches_full_range_through_a_wall() {
    // The same rule, stated where it actually bites: pointed straight down at
    // the floor a metre and a half in front of her, where the crosshair's ray
    // stops almost immediately. The arms still go out ten metres along that
    // line, which is under the arena rather than along it -- and that is
    // correct. What the player asked for was a depth, and the depth is the
    // hold.
    let m = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL);
    let mut w = as_class(Class::BloodMage);
    // Straight down: whatever the ray meets, it meets it at once.
    let pitch = -16384;
    looking(&mut w, m.channel as u32 + 1, Q, pitch, 0);
    let wound = w.players[0].aim_path.length();
    assert!(
        wound.sub(m.reach).abs().raw() < sim::fixed::Fx::ratio(1, 100).raw(),
        "aimed at the floor underfoot, a full hold reached {} m of {}",
        wound.to_f32_for_render(),
        m.reach.to_f32_for_render()
    );
    // And the arms that come out of it carry the same number.
    for _ in 0..90 {
        run(&mut w, 1, 0, 0);
        if let Some(arms) = effects_of(&w, EffectKind::Grasp).first() {
            assert!(
                arms.reach.sub(m.reach).abs().raw() < sim::fixed::Fx::ratio(1, 100).raw(),
                "the marker wound to {} m and the arms came out at {} m",
                wound.to_f32_for_render(),
                arms.reach.to_f32_for_render()
            );
            return;
        }
    }
    panic!("the arms never came out");
}

#[test]
fn the_marker_starts_inside_melee_range() {
    // "Start at the character and move outward." A slider whose near end is
    // already mid-range gives the player no sense that holding is doing
    // anything -- the marker appears out in the arena and creeps, rather than
    // leaving the body and travelling. Inside the reach of her own melee is the
    // test of that, because Rend is what "right in front of me" means for this
    // class.
    let grasp = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL);
    let rend = sim::moves::get(Class::BloodMage, sim::state::SLOT_COMMITTED);
    assert!(
        grasp.reach_after(0).raw() < rend.reach.raw(),
        "a tapped Grasp reaches {} m against {} m of Rend, so it does not start \
         at the caster",
        grasp.reach_after(0).to_f32_for_render(),
        rend.reach.to_f32_for_render()
    );
    // And the far end is still a long way past it, or the slider has no travel.
    assert!(
        grasp.reach.raw() > rend.reach.raw() * 3,
        "the slider runs {} m to {} m, which is not a range worth choosing",
        grasp.reach_after(0).to_f32_for_render(),
        grasp.reach.to_f32_for_render()
    );
}

#[test]
fn a_channel_that_is_never_released_throws_itself() {
    // Holding the button is buying reach, and the reach runs out. Past the cap
    // the move comes out on its own rather than sitting there paid for: an
    // ability you can hold indefinitely is a threat with no clock on it, and
    // the other player has no way to wait one out.
    let m = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL);
    let mut w = as_class(Class::BloodMage);
    let pitch = in_the_grasp(&mut w);
    let mut out = None;
    for f in 0..(m.channel as u32 * 3) {
        looking(&mut w, 1, Q, pitch, 0);
        if out.is_none() && !effects_of(&w, EffectKind::Grasp).is_empty() {
            out = Some(f);
        }
    }
    let f = out.expect("the button was held for three channels and the arms never came out");
    assert!(
        f <= (m.channel + m.startup + m.active) as u32,
        "the arms came out on frame {f}, which is later than the cap plus the          move's own wind-up -- the hold is being stored somewhere"
    );
}

#[test]
fn a_grasp_binds_before_it_hauls_and_the_haul_covers_ground() {
    // The shape of the catch, which is the whole of how the move reads: the
    // arms close and for a moment **nothing happens** -- you are held where
    // they caught you -- and only then are you dragged back, across real
    // distance, at a speed you can watch. It used to be a teleport, and a
    // teleport reads as the game moving somebody rather than as an ability
    // landing on them.
    //
    // Gated on catching somebody with **every** arm, for a mechanical reason as
    // much as a design one: the haul would otherwise pull its victim out from
    // under the other three arms -- the bottom pair connect a frame before the
    // top pair -- and the catch would cancel itself.
    let mut w = as_class(Class::BloodMage);
    let pitch = in_the_grasp(&mut w);
    let grabs = sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL).grabs;
    assert!(grabs > 0, "the table says this move does not grab");

    let apart = |w: &World| w.players[1].pos.sub(w.players[0].pos).flat_len();
    let before = apart(&w);
    grasp(&mut w, pitch);

    // Every frame of the hold, in order, so the two phases can be told apart.
    let mut gaps = Vec::new();
    let mut disabled = false;
    for _ in 0..120 {
        run(&mut w, 1, 0, 0);
        if matches!(w.players[1].action, Action::Held { .. }) {
            gaps.push(apart(&w));
            disabled |= w.players[1].disabled();
        }
    }
    assert!(!gaps.is_empty(), "every arm landed and nobody was caught");
    assert!(disabled, "the payoff window is not a payoff");

    // The bind. Caught where they stood, not snapped to the caster.
    let bind = sim::tuning::grasp_bind() as usize;
    assert!(
        bind > 0,
        "there is no bind, so the haul is a teleport again"
    );
    for (f, gap) in gaps.iter().take(bind).enumerate() {
        assert!(
            gap.raw() * 4 > before.raw() * 3,
            "frame {f} of the bind and they have already travelled {} m of {}",
            before.sub(*gap).to_f32_for_render(),
            before.to_f32_for_render()
        );
    }

    // The haul. Never a single step, and it finishes.
    let biggest = gaps
        .windows(2)
        .map(|w| w[0].sub(w[1]).raw())
        .max()
        .expect("the hold lasted a frame");
    assert!(
        biggest * 3 < before.raw(),
        "one frame of the haul covered {} m of the {} m trip, which is a blink",
        sim::fixed::Fx::from_raw(biggest).to_f32_for_render(),
        before.to_f32_for_render()
    );
    let arrived = *gaps.last().expect("the hold lasted a frame");
    assert!(
        arrived.raw() * 4 < before.raw(),
        "caught from {} m away and left standing {} m away",
        before.to_f32_for_render(),
        arrived.to_f32_for_render()
    );
}

#[test]
fn the_grasp_gives_the_feet_back_the_frame_it_ends() {
    // The other half of the trade. The catch is expensive and short, and what
    // makes it fair is that it is *over* when it is over: no root left on the
    // end of it, no recovery to sit through. A victim who is still stuck after
    // the hands let go cannot tell where the ability stopped, which is how a
    // brief hard stop turns into one that feels like a stunlock.
    let mut w = as_class(Class::BloodMage);
    let pitch = in_the_grasp(&mut w);
    grasp(&mut w, pitch);
    for _ in 0..120 {
        run(&mut w, 1, 0, 0);
        if matches!(w.players[1].action, Action::Held { .. }) {
            break;
        }
    }
    let Action::Held { left } = w.players[1].action else {
        panic!("every arm landed and nobody was caught");
    };
    run(&mut w, left as u32 + 1, 0, 0);
    assert!(
        w.players[1].action.actionable(),
        "the hold ran out and they are still in {:?}",
        w.players[1].action
    );
    let start = w.players[1].pos;
    run(&mut w, 6, 0, Input::W);
    assert!(
        w.players[1].pos.sub(start).flat_len().raw() > 0,
        "the feet never came back"
    );
}

#[test]
fn only_a_full_grasp_catches_anybody() {
    // The rule, swept rather than staged. One or two arms is damage and nothing
    // else: if a single arm could grab, the ability would be a ten-metre pull on
    // any contact at all, which is not a read but a tax on being in front of a
    // Blood mage.
    //
    // A sweep because the interesting positions are a hand's width apart -- the
    // arms converge, so the difference between four and two is about a metre --
    // and a fixture that picked one of them would be pinned to today's cone
    // width rather than to the rule.
    let mut seen_full = false;
    let mut seen_glancing = false;
    for tenth in 0..30 {
        let mut w = as_class(Class::BloodMage);
        let pitch = in_the_grasp(&mut w);
        let off = sim::fixed::Fx::ratio(tenth, 10);
        w.players[1].pos = sim::V3::new(
            w.players[1].pos.x,
            w.players[1].pos.y,
            w.players[1].pos.z.add(off),
        );
        let full = w.players[1].health;
        grasp(&mut w, pitch);

        let mut held = false;
        for _ in 0..120 {
            run(&mut w, 1, 0, 0);
            held |= matches!(w.players[1].action, Action::Held { .. });
        }
        let arms = w.players[1].health.abs_diff(full) as i32
            / sim::moves::get(Class::BloodMage, sim::state::SLOT_SPECIAL)
                .damage
                .max(1);
        let all_four = arms >= sim::effects::GRASP_ARMS as i32;
        let at = off.to_f32_for_render();

        assert_eq!(
            held, all_four,
            "{at} m off centre: {arms} arms landed, held = {held}"
        );
        seen_full |= all_four;
        seen_glancing |= arms > 0 && !all_four;
    }
    assert!(seen_full, "the sweep never caught anybody with all four");
    assert!(
        seen_glancing,
        "the sweep never found a glancing hit, so it proves only one half"
    );
}

#[test]
fn a_grasp_catches_only_when_every_arm_lands() {
    // Four arms, and the catch is the price of all four. One or two of them is
    // a glancing blow; standing where the cone closes is a read, and a read is
    // what the design lets a hard stop be bought with.
    let mut w = as_class(Class::BloodMage);
    let pitch = in_the_grasp(&mut w);
    grasp(&mut w, pitch);
    let mut caught = false;
    for _ in 0..90 {
        run(&mut w, 1, 0, 0);
        caught |= matches!(w.players[1].action, Action::Held { .. });
    }
    assert!(caught, "caught by every arm and still walking");

    // Far off to one side: the cone never reaches, so nothing lands.
    let mut w = as_class(Class::BloodMage);
    w.players[1].pos = sim::V3::new(
        w.players[0].pos.x,
        w.players[1].pos.y,
        w.players[0].pos.z.add(sim::fixed::Fx::from_int(9)),
    );
    let full = w.players[1].health;
    grasp(&mut w, 0);
    run(&mut w, 60, 0, 0);
    assert_eq!(
        w.players[1].health, full,
        "the arms reached across the arena"
    );
    assert!(
        !matches!(w.players[1].action, Action::Held { .. }),
        "caught by a Grasp that missed"
    );
}

#[test]
fn a_caught_fighter_cannot_walk_dodge_or_jump_out_of_it() {
    // The hold is a hard stop, and the design allows one only behind a hard
    // condition (`ability-spec.md`) -- here, landing every arm of a Grasp.
    // Having paid for it, it has to actually hold: a catch you can dodge or
    // jump out of is a very expensive way to deal one hit of damage.
    let mut w = as_class(Class::BloodMage);
    let pitch = in_the_grasp(&mut w);
    grasp(&mut w, pitch);
    for _ in 0..120 {
        run(&mut w, 1, 0, 0);
        if matches!(w.players[1].action, Action::Held { .. }) {
            break;
        }
    }
    assert!(
        matches!(w.players[1].action, Action::Held { .. }),
        "fixture caught nobody"
    );

    // Held still for the bind, whatever they press.
    let start = w.players[1].pos;
    run(&mut w, 2, 0, Input::SHIFT | Input::W);
    assert!(
        !matches!(w.players[1].action, Action::Dodge { .. }),
        "a caught fighter dodged out of it"
    );
    run(&mut w, 2, 0, Input::SPACE);
    assert!(w.players[1].grounded, "a caught fighter jumped out of it");
    assert_eq!(
        w.players[1].pos.sub(start).flat_len().raw(),
        0,
        "a caught fighter walked out of the bind"
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
    // increased damage on disabled enemies*. It is what turns the Grasp's catch
    // from a small reward into a setup -- four arms is expensive, and it is
    // only worth the cost if something is waiting on the other side of it.
    let free = one_hit(|_| {});
    // Bound for the whole window rather than hauled, so the fixture measures
    // the multiplier and not the distance the victim covered while it swung.
    let caught = one_hit(|w| w.players[1].seized(0, 120, 120));
    assert!(free > 0, "fixture: the claw did not connect at all");
    assert!(
        caught > free,
        "a caught fighter took {caught} where a free one took {free}"
    );

    let expected = sim::fixed::Fx::from_int(free)
        .mul(sim::tuning::disabled_damage_mul())
        .to_int();
    assert!(
        (caught - expected).abs() <= 1,
        "the bonus is {caught} against {free}, which is not the knob"
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
    // The two halves together, which is the point of implementing either.
    // Catch them with every arm, and the window that opens is the one the
    // class's damage bonus is waiting on.
    let mut w = as_class(Class::BloodMage);
    let pitch = in_the_grasp(&mut w);
    grasp(&mut w, pitch);
    let mut disabled = false;
    for _ in 0..90 {
        run(&mut w, 1, 0, 0);
        disabled |= w.players[1].disabled();
    }
    assert!(
        disabled,
        "the catch does not count as a disable, so the payoff never fires"
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

const LMB: u16 = Input::LEFT;
const MMB: u16 = Input::MIDDLE;
const RMB: u16 = Input::RIGHT;

/// A Champion nose to nose with a Bulwark, already mid-Rush.
///
/// Every Rush move needs the dash under it, and the dash is one charge on a
/// long recharge, so the fixture spends it rather than each test doing so.
fn rushing(toward: u16) -> World {
    let mut w = engaged(Class::Champion);
    run(&mut w, 1, E, 0);
    run(&mut w, 1, 0, 0);
    let _ = toward;
    w
}

/// Off the ground and **past the takeoff window**, so a click gives an aerial
/// rather than the move that leaves the floor.
///
/// Every airborne test needs this now: the window is a few frames wide by
/// design, and a script that jumps and immediately attacks is asking for a
/// takeoff whether or not it meant to. Which is the right answer to that input
/// -- it is what a player pressing both means -- so the fixture waits it out
/// rather than the rule being softened.
fn airborne() -> World {
    let mut w = engaged(Class::Champion);
    run(&mut w, 1, Input::SPACE, 0);
    run(&mut w, sim::tuning::takeoff_window() as u32 + 2, 0, 0);
    assert!(!w.players[0].grounded, "the fixture never left the ground");
    w
}

#[test]
fn the_uppercut_takes_both_fighters_off_the_ground() {
    // The move is a leap, and it is a leap you bring someone along on. Either
    // half alone is a different move: without the lift it is a launcher you
    // cannot follow up on, and without the launch it is an escape.
    //
    // It is the **hammer's takeoff** now -- middle click on the same press as
    // jump -- which puts the launcher on a button the class always has rather
    // than behind a charge it may have spent. See `docs/design/champion.md`.
    let mut w = engaged(Class::Champion);
    let mut lifted = [false; MAX_PLAYERS];
    run(&mut w, 2, MMB | Input::SPACE, 0);
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

#[test]
fn pressing_jump_inside_an_uppercut_takes_the_pair_of_you_higher() {
    // "We are settling this in the air." The leap is the whole reason the
    // uppercut holds on to somebody rather than merely launching them: you get
    // to decide, after it connects, how high this exchange is going to happen.
    let climb = |leap: bool| {
        let mut w = engaged(Class::Champion);
        run(&mut w, 2, MMB | Input::SPACE, 0);
        let mut best = 0.0f32;
        let mut off = 0;
        for _ in 0..70 {
            // The leap is only there to be pressed once the uppercut has
            // actually taken you up, which is its first active frame -- so the
            // script waits for the feet to leave the floor rather than counting
            // to a number that a retune would move. Tapped rather than held: it
            // is an input, and holding it down would be one press.
            if !w.players[0].grounded {
                off += 1;
            }
            let space = if leap && off == 2 { Input::SPACE } else { 0 };
            w.advance([Input::aimed(space, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
            best = best.max(w.players[1].pos.y.to_f32_for_render());
        }
        best
    };
    let plain = climb(false);
    let leapt = climb(true);
    assert!(
        leapt > plain + 0.2,
        "the victim topped out at {leapt:.2} m with the leap and {plain:.2} m without it"
    );
}

#[test]
fn the_three_buttons_are_three_weapons() {
    // The whole input scheme in one assertion: a click is a weapon, and which
    // of that weapon's moves comes out is decided by where your feet are. If
    // this ever starts passing by accident -- two buttons throwing one move --
    // the class is back to being a mode toggle.
    use sim::class::Form;
    let thrown = |button: u16| {
        let mut w = engaged(Class::Champion);
        run(&mut w, 2, button, 0);
        let kind = w.players[0].action.attack_kind().expect("nothing came out");
        let form = match w.players[0].mechanic {
            Mechanic::Forms { form, .. } => form,
            _ => panic!("the Champion lost its mechanic"),
        };
        (kind, form)
    };
    assert_eq!(thrown(LMB), (0, Form::Sword));
    assert_eq!(thrown(MMB), (1, Form::Hammer));
    assert_eq!(thrown(RMB), (2, Form::Spear));
}

#[test]
fn the_same_button_is_a_different_move_in_the_air() {
    // Aerials are variants of their grounded counterpart rather than a separate
    // list -- the direction `controls.md` has had open since the dodge moved.
    let mut w = airborne();
    run(&mut w, 2, LMB, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(sim::moves::champion::AIR_SWORD),
        "left click in the air still threw the standing sword"
    );
}

#[test]
fn a_spear_put_into_the_ground_vaults_and_one_levelled_at_someone_stabs() {
    // Two moves on one button, separated by where you are pointing. The
    // separator has to be legible, so it is the most physical one available: a
    // pole vault *is* a spear planted in the floor.
    let down = -(1 << 13); // an eighth of a turn below the horizon
    let mut w = engaged(Class::Champion);
    run(&mut w, 1, E, 0);
    for _ in 0..2 {
        w.advance([
            Input::looking_at(RMB, LOOK_RIGHT, down),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(sim::moves::champion::POLE_VAULT),
        "a spear aimed at the floor mid-Rush did not vault"
    );

    let mut w = engaged(Class::Champion);
    run(&mut w, 1, E, 0);
    run(&mut w, 2, RMB, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(sim::moves::champion::RUSH_STAB),
        "a spear levelled mid-Rush did not stab"
    );
}

#[test]
fn the_vault_trades_the_dash_for_height() {
    // It is movement, not an attack: it has no hit volume at all, and what it
    // buys is a way into the air that a jump cannot reach.
    let down = -(1 << 13);
    let mut w = engaged(Class::Champion);
    run(&mut w, 1, E, 0);
    let mut top = 0.0f32;
    for f in 0..80 {
        let bits = if (1..3).contains(&f) { RMB } else { 0 };
        w.advance([
            Input::looking_at(bits, LOOK_RIGHT, down),
            Input::aimed(0, LOOK_LEFT),
        ]);
        top = top.max(w.players[0].pos.y.to_f32_for_render());
        assert!(
            sim::state::hitbox(&w.players[0]).is_none() || f > 40,
            "the vault put a hitbox in the world"
        );
    }
    let jump = {
        let mut w = engaged(Class::Champion);
        let mut top = 0.0f32;
        for _ in 0..80 {
            w.advance([
                Input::aimed(Input::SPACE, LOOK_RIGHT),
                Input::aimed(0, LOOK_LEFT),
            ]);
            top = top.max(w.players[0].pos.y.to_f32_for_render());
        }
        top
    };
    assert!(
        top > jump,
        "the vault reached {top:.2} m and a plain jump reaches {jump:.2} m, so planting the \
         spear bought nothing"
    );
}

#[test]
fn the_hammer_out_of_a_rush_goes_under_a_sword() {
    // The Rush row is three answers to "what does this weapon do to somebody
    // you are running at", and the hammer's is the one that goes underneath.
    // Its volume has to reach the floor, and the sword's beside it has to not
    // -- otherwise the row is one move with three damage numbers.
    // The lowest point the volume reaches over the whole swing, in metres
    // above the floor.
    let floor_of = |button: u16| {
        let mut w = rushing(LOOK_RIGHT);
        let mut lowest = 9.0f32;
        for f in 0..30 {
            let bits = if f < 2 { button } else { 0 };
            w.advance([Input::aimed(bits, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
            if let Some(b) = sim::state::hitbox(&w.players[0]) {
                let deepest = b.from.y.min(b.to.y).sub(b.radius);
                lowest = lowest.min(deepest.to_f32_for_render());
            }
        }
        lowest
    };
    let hammer = floor_of(MMB);
    let sword = floor_of(LMB);
    assert!(
        hammer < 0.2,
        "the hammer's run-through never reaches the floor: {hammer:.2} m"
    );
    assert!(
        sword > hammer,
        "the sword's run-through goes as low as the hammer's ({sword:.2} against {hammer:.2})"
    );
}

#[test]
fn the_sword_keeps_cutting_while_the_dash_runs() {
    // The run-through: the samurai walks past a line of people and they fall
    // over afterwards. One long active window that re-arms, rather than a
    // stronger single hit, because the point is that you were *moving* through
    // them the whole time.
    let mut w = engaged(Class::Champion);
    run(&mut w, 1, E, 0);
    let before = w.players[1].health;
    let single = sim::moves::get(Class::Champion, sim::moves::champion::RUSH_SLASH).damage;
    run(&mut w, 2, LMB, 0);
    run(&mut w, 40, 0, 0);
    let dealt = before - w.players[1].health;
    assert!(
        dealt > single,
        "one pass of the run-through dealt {dealt}, which is one cut's {single}"
    );
}

#[test]
fn an_airborne_target_is_driven_into_the_floor() {
    // The other half of the air game. Hitting somebody up is only worth doing
    // if there is something to do to them up there, and the aerial hammer is
    // it: a spike, and then the ground charges for the landing.
    let mut w = engaged(Class::Champion);
    // The Champion dropping from above, the victim still on the way up. Set
    // rather than played out, because what is being asserted is the spike and
    // not the twenty-frame combo that gets you here.
    w.players[1].pos.y = sim::Fx::from_int(3);
    w.players[1].vel.y = sim::Fx::from_int(14);
    w.players[1].grounded = false;
    w.players[0].pos.y = sim::Fx::from_int(4);
    w.players[0].grounded = false;
    let before = w.players[1].health;
    let down = -(1 << 14); // straight down
    let mut staggered = false;
    for f in 0..90 {
        let bits = if (0..2).contains(&f) { MMB } else { 0 };
        w.advance([
            Input::looking_at(bits, LOOK_RIGHT, down),
            Input::aimed(0, LOOK_LEFT),
        ]);
        staggered |= matches!(w.players[1].action, Action::Stagger { .. });
    }
    let dealt = before - w.players[1].health;
    let swing = sim::moves::get(Class::Champion, sim::moves::champion::AIR_HAMMER).damage;
    assert!(
        dealt > swing,
        "the spike dealt {dealt} and the swing alone is {swing}, so the floor charged nothing"
    );
    assert!(
        staggered,
        "they got up off the floor as if they had walked into it"
    );
}

/// Park a defender at `(ahead, up, across)` metres from the Champion, throw
/// `button`, and say whether it connected.
///
/// In the Champion's own frame: `ahead` along the facing, `across` to its
/// right. Positions rather than distances, because what is being asserted is
/// that the three weapons own three different *pieces of space* -- which no
/// single reach number can say.
fn reaches(button: u16, ahead: f32, across: f32, ducking: bool) -> bool {
    let m = |v: f32| sim::Fx::ratio((v * 1000.0) as i32, 1000);
    let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
    // Player one spawns looking down +x, so ahead is +x and its right is +z.
    let base = w.players[0].pos;
    let park = |w: &mut World| {
        w.players[1].pos = sim::V3::new(base.x.add(m(ahead)), base.y, m(across));
    };
    park(&mut w);
    let duck = if ducking { Input::CROUCH } else { 0 };
    let before = w.players[1].health;
    for f in 0..50 {
        let bits = if f < 2 { button } else { 0 };
        w.advance([
            Input::aimed(bits, LOOK_RIGHT),
            Input::aimed(duck, LOOK_LEFT),
        ]);
        // Nobody is allowed to walk into anybody: the question is what the
        // swing covers from where it was thrown.
        park(&mut w);
        if w.players[1].health < before {
            return true;
        }
    }
    false
}

#[test]
fn the_three_weapons_own_three_different_pieces_of_space() {
    // The point of the whole rebuild, as one assertion. A reach number cannot
    // say any of this -- it takes a shape, and since 2026-09-15 it also takes
    // the ground the move closes: the sword's opener steps in, so what it
    // threatens is its reach *plus* its step, and the measurements below are of
    // the threat rather than of the table.
    //
    // Sword: **across, and it comes to you**. The only one of the three that
    // catches somebody standing well off the line it was thrown down, and the
    // only one whose two openers cover opposite sides -- the first cut comes
    // down from the right shoulder and the second from the left, so a pair of
    // them covers a fan a single swing could not.
    assert!(
        reaches(LMB, 0.9, 1.4, false),
        "the sword does not cover its own flank"
    );
    assert!(
        reaches(LMB, 0.9, -1.4, false),
        "the sword covers one flank and not the other"
    );
    assert!(
        !reaches(LMB, 3.8, 0.0, false),
        "the sword reaches as far as a spear, step and all"
    );

    // Spear: **out**. The longest line in the game, and a thin one -- it misses
    // the same flank the sword owns, and it is the band between those two
    // numbers that is the spear's alone.
    assert!(reaches(RMB, 3.8, 0.0, false), "the spear does not reach");
    assert!(
        !reaches(RMB, 0.9, 1.8, false),
        "the spear covers the flank too"
    );

    // Hammer: **down**. Shortest of the three, and the only one that arrives at
    // floor level -- so it is the answer to somebody ducking under the spear's
    // line, which is the one thing a longer weapon cannot do for you.
    assert!(
        reaches(MMB, 1.4, 0.0, false),
        "the hammer does not reach in front of itself"
    );
    assert!(
        !reaches(MMB, 3.2, 0.0, false),
        "the hammer reaches as far as a spear"
    );
    assert!(
        reaches(MMB, 1.4, 0.0, true) && !reaches(RMB, 1.4, 0.0, true),
        "the hammer and the spear agree about a crouching opponent"
    );
}

#[test]
fn the_spear_pokes_from_where_it_stands_and_the_sword_has_to_come_in() {
    // The other half of "the spear owns distance", and the half a reach number
    // *can* say -- but only now that a move is allowed to carry the body. The
    // two weapons' threat ranges are a bit under a metre apart and the sword's
    // step is most of what closes that gap, so the honest difference between
    // them is no longer only how far they reach: it is **where you are standing
    // when it is over**.
    //
    // A spear poke leaves you exactly where you were, which is the whole of what
    // a spacing tool is. A sword cut spends better than half a metre of ground
    // to land, and you are still standing on it afterwards -- inside their reach,
    // which is the price of being the weapon that closes.
    use sim::moves::champion as c;
    let sword = moves::get(Class::Champion, c::SWORD_GROUND);
    let spear = moves::get(Class::Champion, c::SPEAR_GROUND);
    assert_eq!(
        spear.step.raw(),
        0,
        "the spear's opener carries you {} m forward, so it is not a poke from \
         where you stand any more",
        spear.step.to_f32_for_render()
    );
    assert!(
        sword.step.raw() > 0,
        "the sword's opener does not step, so the class's close-in weapon does \
         not close"
    );
    assert!(
        spear.reach.raw() > sword.reach.add(sword.step).raw(),
        "the spear reaches {:.2} m and the sword reaches {:.2} m and then steps \
         {:.2} of them, which is not a spacing weapon and a closing one",
        spear.reach.to_f32_for_render(),
        sword.reach.to_f32_for_render(),
        sword.step.to_f32_for_render()
    );
}

#[test]
fn the_air_spear_pays_its_shove_out_on_contact() {
    // The fan is a repositioning tool you have to earn: catching somebody with
    // it kicks you the way you are holding. Thrown at nothing it is just a
    // poke, which is what stops it being free flight.
    let speed = |hit: bool| {
        let mut w = airborne();
        if !hit {
            // Out of the way, so the identical script whiffs.
            w.players[1].pos =
                sim::V3::new(sim::Fx::from_int(25), w.players[1].pos.y, sim::Fx::ZERO);
        }
        let mut best = 0.0f32;
        for f in 0..30 {
            let bits = if (0..2).contains(&f) { RMB } else { 0 } | Input::W;
            w.advance([Input::aimed(bits, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
            let v = w.players[0].vel;
            best = best
                .max((v.x.to_f32_for_render().powi(2) + v.z.to_f32_for_render().powi(2)).sqrt());
        }
        best
    };
    let landed = speed(true);
    let whiffed = speed(false);
    assert!(
        landed > whiffed + 1.0,
        "the fan reached {landed:.2} m/s on a hit and {whiffed:.2} m/s on a whiff"
    );
}

#[test]
fn the_combo_the_class_is_built_around_is_playable() {
    // Hammer, let the chain carry itself into the second hit, take the pair of
    // you up with the hammer's takeoff, leap higher, and spike them back into
    // the floor. Every piece of this is tested on its own above; this is the
    // assertion that they **connect** -- that a connected link really does cut
    // its own recovery short, that the launcher catches somebody already in
    // hitstun, that the carry lasts long enough to throw a twenty-two frame
    // startup off the end of, and that the spike lands before they hit the
    // ground on their own.
    //
    // A loop that works step by step and does not join up is the usual way a
    // combo class turns out not to be one.
    let mut w = engaged(Class::Champion);
    let start = w.players[1].health;

    // Hammer, and keep the button down: a link that lands cuts its own tail
    // short, so the string walks itself into the second hit.
    let mut second = false;
    for _ in 0..50 {
        w.advance([Input::aimed(MMB, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        second |= w.players[0].action.attack_kind() == Some(moves::champion::UPROOT);
        if second {
            break;
        }
    }
    assert!(
        second,
        "the hammer never chained into its second hit: {:?}",
        w.players[0].action
    );
    // Let the second hit finish. Bounded, so a stuck fixture fails rather than
    // hangs.
    for _ in 0..60 {
        if w.players[0].action.actionable() {
            break;
        }
        run(&mut w, 1, 0, 0);
    }

    run(&mut w, 2, MMB | Input::SPACE, 0); // the uppercut: grabs, and lifts
    for _ in 0..30 {
        if !w.players[0].grounded {
            break;
        }
        run(&mut w, 1, 0, 0);
    }
    run(&mut w, 1, 0, 0);
    run(&mut w, 1, Input::SPACE, 0); // both of you, higher
    // Up until the uppercut's own tail gives the controls back.
    for _ in 0..40 {
        if w.players[0].action.actionable() {
            break;
        }
        run(&mut w, 1, 0, 0);
    }
    assert!(
        !w.players[1].grounded && w.players[1].pos.y.to_f32_for_render() > 3.0,
        "the uppercut did not take them anywhere: {:.2} m",
        w.players[1].pos.y.to_f32_for_render()
    );

    // And the spike, aimed down, thrown the first frame the uppercut's own tail
    // gives the controls back.
    let down = -(1 << 13);
    let mut staggered = false;
    let mut thrown = false;
    for _ in 0..80 {
        let bits = if !thrown && w.players[0].action.actionable() {
            thrown = true;
            MMB
        } else {
            0
        };
        w.advance([
            Input::looking_at(bits, LOOK_RIGHT, down),
            Input::aimed(0, LOOK_LEFT),
        ]);
        staggered |= matches!(w.players[1].action, Action::Stagger { .. });
    }
    assert!(staggered, "they were never put on the floor");

    let dealt = start - w.players[1].health;
    let biggest = moves::table(Class::Champion)
        .iter()
        .map(|m| m.damage)
        .max()
        .unwrap();
    assert!(
        dealt > biggest,
        "the whole loop dealt {dealt}, and one Rush stab deals {biggest} -- the combo is \
         not worth doing"
    );
    // And it is not a round-ender either: a combo that takes most of a health
    // bar would make the first read the whole match.
    assert!(
        dealt < sim::tuning::max_health() / 2,
        "the loop deals {dealt} of {} health in one go",
        sim::tuning::max_health()
    );
}

// --- The chain -------------------------------------------------------------
//
// Three hits deep on the ground, and every hit is a free choice of all three
// weapons. What these check is the *grammar* of that rather than the numbers:
// which move a press produces, what ends a string, and what a string costs when
// the defender answers it.

/// Hold one button down and collect every move it throws, in order.
fn strung(w: &mut World, button: u16, frames: u32) -> Vec<u8> {
    let mut seen: Vec<u8> = Vec::new();
    for _ in 0..frames {
        w.advance([Input::aimed(button, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        if let Some(kind) = w.players[0].action.attack_kind() {
            if seen.last() != Some(&kind) {
                seen.push(kind);
            }
        }
    }
    seen
}

#[test]
fn one_weapon_held_down_walks_the_whole_chain_and_then_starts_again() {
    // Linear play is meant to be strong, so the simplest possible input -- hold
    // a button -- has to produce the whole string rather than the first hit
    // three times.
    use moves::champion as c;
    let mut w = engaged(Class::Champion);
    let seen = strung(&mut w, LMB, 180);
    assert_eq!(
        &seen[..3],
        &[c::SWORD_GROUND, c::BACKCUT, c::UPCUT],
        "holding left click did not walk the sword chain: {seen:?}"
    );
    assert_eq!(
        seen.get(3),
        Some(&c::SWORD_GROUND),
        "the chain did not go back to its opener after the finisher: {seen:?}"
    );
}

#[test]
fn every_hit_of_the_chain_is_a_free_choice_of_weapon() {
    // **The class fantasy, as an assertion.** Sword into spear into hammer is
    // an ordinary thing to do, and nothing about the second hit remembers what
    // the first one was made of.
    use moves::champion as c;
    let mut w = engaged(Class::Champion);
    let mut seen = Vec::new();
    let mut want = [LMB, RMB, MMB].into_iter().peekable();
    let mut button = want.next().unwrap();
    for _ in 0..140 {
        w.advance([Input::aimed(button, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        if let Some(kind) = w.players[0].action.attack_kind() {
            if seen.last() != Some(&kind) {
                seen.push(kind);
                if let Some(next) = want.next() {
                    button = next;
                }
            }
        }
    }
    assert_eq!(
        &seen[..3],
        &[c::SWORD_GROUND, c::SKEWER, c::EARTHBREAKER],
        "a mixed string did not come out as one chain: {seen:?}"
    );
}

#[test]
fn the_two_sword_openers_cut_opposite_diagonals() {
    // **The sword owns the diagonal**, and the two openers own opposite ones:
    // the first comes down from over the right shoulder and the second from over
    // the left. A player reads which hit they are looking at off where the blade
    // started, so the two must not be the same stroke played twice -- and a pair
    // of them covers a fan that one swing could not.
    //
    // Measured off the hit volume rather than off the clip, because the volume is
    // what decides the fight: `hitbox`'s first frame is the start of the arc.
    let opened = |button: u16, second: bool| {
        let mut w = engaged(Class::Champion);
        // Nothing in reach, so the chain is walked by hand: what is being looked
        // at is the shape, and a victim standing in it would change the spacing.
        w.players[1].pos = sim::V3::new(sim::Fx::from_int(11), w.players[1].pos.y, sim::Fx::ZERO);
        if second {
            if let Mechanic::Forms {
                chain, chain_left, ..
            } = &mut w.players[0].mechanic
            {
                *chain = 1;
                *chain_left = 200;
            }
        }
        for _ in 0..40 {
            w.advance([Input::aimed(button, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
            if let Some(h) = sim::state::hitbox(&w.players[0]) {
                let up = h.to.y.sub(h.from.y);
                // The facing is +x, so its own sides are ±z.
                let side = h.to.z.sub(h.from.z);
                return (up.to_f32_for_render(), side.to_f32_for_render());
            }
        }
        panic!("the cut put nothing out");
    };
    let (first_up, first_side) = opened(LMB, false);
    let (second_up, second_side) = opened(LMB, true);
    assert!(
        first_up > 0.4 && second_up > 0.4,
        "a descending cut has to start above the hands: the openers start \
         {first_up:.2} m and {second_up:.2} m up"
    );
    assert!(
        first_side * second_side < 0.0,
        "both sword openers start over the same shoulder ({first_side:.2} and \
         {second_side:.2} off the centre line), so there is no telling them apart"
    );
    assert!(
        first_side.abs() > 0.4 && second_side.abs() > 0.4,
        "the openers start {:.2} m and {:.2} m off the centre line, which is a \
         vertical swing rather than a diagonal one",
        first_side.abs(),
        second_side.abs()
    );
}

#[test]
fn a_step_is_finished_by_the_time_the_weapon_lands() {
    // **The rule that makes a stepping attack a step rather than a slide.** The
    // body covers the move's own `step` down the facing it locked, and it has
    // covered all of it on the frame the hitbox appears -- so the weight arrives
    // with the weapon. What happens afterwards is momentum bleeding off through
    // the recovery, which is the follow-through.
    //
    // It used to run for the whole active window, and the hammer's finisher is
    // what found the difference: it put its head through the floor on the first
    // active frame and then carried the body most of another metre past it.
    use moves::champion as c;
    for (name, button, kind) in [
        ("the sword's opener", LMB, c::SWORD_GROUND),
        ("the hammer's finisher", MMB, c::EARTHBREAKER),
    ] {
        let m = moves::get(Class::Champion, kind);
        let want = m.step.to_f32_for_render();
        assert!(want > 0.0, "{name} does not step at all");
        // From the spawn rather than from `engaged`, and with nobody in front:
        // what is being measured is ground covered, so there must be clear floor
        // to cover it on. `engaged` walks the fighter into the side of a platform,
        // and a step into a wall is the arena's answer rather than the move's.
        let mut w = as_class(Class::Champion);
        w.players[1].pos = sim::V3::new(sim::Fx::from_int(11), w.players[1].pos.y, sim::Fx::ZERO);
        if let Mechanic::Forms {
            chain, chain_left, ..
        } = &mut w.players[0].mechanic
        {
            *chain = c::link_of(kind).expect("a chain link");
            *chain_left = 400;
        }
        run(&mut w, 12, 0, 0);
        let from = w.players[0].pos.x;
        let mut at_contact = None;
        for _ in 0..80 {
            w.advance([Input::aimed(button, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
            let live = matches!(w.players[0].action, sim::state::Action::Active { .. });
            if live && at_contact.is_none() {
                at_contact = Some(w.players[0].pos.x.sub(from).to_f32_for_render());
            }
        }
        let gone = at_contact.unwrap_or_else(|| panic!("{name} never came out"));
        assert!(
            (gone - want).abs() < 0.08,
            "{name} says it steps {want:.2} m and had covered {gone:.2} m by the \
             frame its volume appeared"
        );
    }
}

#[test]
fn nothing_that_does_not_declare_a_step_moves_on_its_own() {
    // The other side of it. A move without a `step` must not carry you an inch,
    // or the field is decorative and every class's spacing has quietly changed.
    // The spear's opener is the one to check, because the spear is the spacing
    // weapon and being carried forward is the one thing that would stop it being
    // one -- see `the_spear_pokes_from_where_it_stands_and_the_sword_has_to_come_in`.
    let mut w = as_class(Class::Champion);
    w.players[1].pos = sim::V3::new(sim::Fx::from_int(11), w.players[1].pos.y, sim::Fx::ZERO);
    run(&mut w, 12, 0, 0);
    let from = w.players[0].pos;
    for _ in 0..40 {
        w.advance([Input::aimed(RMB, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
    }
    let gone = w.players[0].pos.sub(from).flat_len().to_f32_for_render();
    assert!(
        gone < 0.05,
        "the spear's opener walked {gone:.2} m on its own"
    );
}

#[test]
fn jumping_into_the_hammer_finisher_takes_both_of_you_up() {
    // The decision the hammer's finisher offers, and it is a decision because
    // both halves of it cost something. Throw it and they go up and you stay on
    // the floor, which is a knock-up and a reset. Press jump while it is winding
    // up and they go *higher* and you leave the floor on the frame it lands, so
    // the exchange continues in the air -- where the air hammer is waiting and
    // where a whiff leaves you falling with nothing.
    //
    // Paid on contact rather than on the press, which is the rule the aerial
    // fan's shove already follows: a whiffed finisher that launched you anyway
    // would be a free escape bolted to the most punishable move in the kit.
    let exchange = |jump: bool| {
        let mut w = engaged(Class::Champion);
        if let Mechanic::Forms {
            chain, chain_left, ..
        } = &mut w.players[0].mechanic
        {
            *chain = 2;
            *chain_left = 400;
        }
        let mut theirs = 0.0f32;
        let mut mine = 0.0f32;
        for f in 0..90 {
            // The press lands in the middle of the wind-up, which is where a
            // player deciding "I am going with this one" would put it.
            let space = if jump && (8..10).contains(&f) {
                Input::SPACE
            } else {
                0
            };
            w.advance([
                Input::aimed(MMB | space, LOOK_RIGHT),
                Input::aimed(0, LOOK_LEFT),
            ]);
            theirs = theirs.max(w.players[1].vel.y.to_f32_for_render());
            mine = mine.max(w.players[0].vel.y.to_f32_for_render());
        }
        (theirs, mine)
    };
    let (up_alone, me_alone) = exchange(false);
    let (up_together, me_together) = exchange(true);
    assert!(
        up_alone > 1.0,
        "the hammer's finisher does not knock anybody up at all"
    );
    assert!(
        me_alone < 1.0,
        "the Champion left the floor without asking to: {me_alone:.1} m/s"
    );
    assert!(
        up_together > up_alone,
        "going up with them knocked them up {up_together:.1} m/s against \
         {up_alone:.1} alone, so the jump bought nothing for them"
    );
    assert!(
        me_together > 1.0,
        "the jump was pressed during the finisher and the Champion stayed on the \
         floor"
    );
}

#[test]
fn the_leap_is_paid_on_contact_and_not_on_the_press() {
    // The guard on the rule above. A finisher thrown at nobody has to leave you
    // standing in your own recovery, whatever you pressed during it -- otherwise
    // the jump is an escape from the whiff punishment rather than a commitment to
    // the exchange.
    let mut w = engaged(Class::Champion);
    // Out of reach entirely, so the finisher connects with nothing.
    w.players[1].pos = sim::V3::new(sim::Fx::from_int(11), w.players[1].pos.y, sim::Fx::ZERO);
    if let Mechanic::Forms {
        chain, chain_left, ..
    } = &mut w.players[0].mechanic
    {
        *chain = 2;
        *chain_left = 400;
    }
    let mut highest = 0.0f32;
    for f in 0..90 {
        let space = if (8..10).contains(&f) {
            Input::SPACE
        } else {
            0
        };
        w.advance([
            Input::aimed(MMB | space, LOOK_RIGHT),
            Input::aimed(0, LOOK_LEFT),
        ]);
        highest = highest.max(w.players[0].vel.y.to_f32_for_render());
    }
    assert!(
        highest < 1.0,
        "a whiffed finisher still threw the Champion into the air at \
         {highest:.1} m/s"
    );
}

#[test]
fn a_blocked_link_pays_its_whole_recovery() {
    // **The chain is a hit confirm**, and this is the half of it the defender
    // owns. Blocking one hit of a string does not merely reduce the damage, it
    // takes the string's rhythm away -- which is what keeps every number the
    // frame table prints about a Champion move true against somebody who
    // answered it.
    let reached = |guarding: bool| {
        let mut w = engaged(Class::Champion);
        let block = if guarding { Input::RIGHT } else { 0 };
        let mut at = 0u32;
        for f in 0..60 {
            w.advance([
                Input::aimed(LMB, LOOK_RIGHT),
                Input::aimed(block, LOOK_LEFT),
            ]);
            if w.players[0].action.attack_kind() == Some(moves::champion::BACKCUT) {
                at = f;
                break;
            }
        }
        at
    };
    let open = reached(false);
    let blocked = reached(true);
    assert!(open > 0, "the sword never chained at all");
    assert!(
        blocked > open,
        "the second hit arrived on frame {blocked} against a guard and frame {open} \
         against nobody -- blocking bought no time"
    );
}

#[test]
fn swapping_weapons_flows_faster_than_swinging_the_same_one_twice() {
    // The nonlinear incentive, and it is deliberately small: one haft with
    // three heads, and the head is re-formed out of the follow-through rather
    // than re-chambered. Linear play stays completely viable -- see
    // `tuning::chain_cancel_swapped`.
    let arrives = |second: u16, kind: u8| {
        let mut w = engaged(Class::Champion);
        run(&mut w, 1, LMB, 0);
        for f in 1..60u32 {
            w.advance([Input::aimed(second, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
            if w.players[0].action.attack_kind() == Some(kind) {
                return f;
            }
        }
        u32::MAX
    };
    let repeated = arrives(LMB, moves::champion::BACKCUT);
    let swapped = arrives(RMB, moves::champion::SKEWER);
    assert!(
        swapped < repeated,
        "a swap reached the second hit on frame {swapped} and a repeat on frame \
         {repeated}; the swap buys nothing"
    );
}

#[test]
fn leaving_the_ground_ends_a_string() {
    // A chain you could park in the air and come back to would make the grace
    // window mean nothing, and would let the class hold a finisher over
    // somebody indefinitely.
    let mut w = engaged(Class::Champion);
    run(&mut w, 1, LMB, 0);
    run(&mut w, 30, 0, 0);
    run(&mut w, 1, Input::SPACE, 0);
    run(&mut w, sim::tuning::takeoff_window() as u32 + 2, 0, 0);
    // Back on the floor, and whatever the string was is over.
    for _ in 0..90 {
        if w.players[0].grounded {
            break;
        }
        run(&mut w, 1, 0, 0);
    }
    run(&mut w, 2, LMB, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(moves::champion::SWORD_GROUND),
        "the chain survived a trip into the air"
    );
}

#[test]
fn a_string_dies_if_you_stop_swinging() {
    let mut w = engaged(Class::Champion);
    run(&mut w, 1, LMB, 0);
    // Long enough for the sword to finish and the grace to run all the way out.
    run(&mut w, 40 + sim::tuning::chain_grace() as u32, 0, 0);
    run(&mut w, 2, LMB, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(moves::champion::SWORD_GROUND),
        "the chain outlived its grace window"
    );
}

// --- Leaving the floor -----------------------------------------------------

#[test]
fn each_weapon_has_its_own_way_off_the_ground() {
    // Three takeoffs on the three buttons, and the window is wide enough that
    // "jump and attack" does not have to be one frame.
    use moves::champion as c;
    let thrown = |button: u16, wait: u32| {
        let mut w = engaged(Class::Champion);
        run(&mut w, 1, Input::SPACE, 0);
        run(&mut w, wait, 0, 0);
        run(&mut w, 1, button, 0);
        w.players[0].action.attack_kind()
    };
    assert_eq!(thrown(LMB, 0), Some(c::RISING_CUT));
    assert_eq!(thrown(MMB, 0), Some(c::UPPERCUT));
    assert_eq!(thrown(RMB, 0), Some(c::POLE_DRIVE));
    // A few frames late still means what the player meant.
    assert_eq!(thrown(LMB, 3), Some(c::RISING_CUT));
    // And past the window it is an ordinary aerial again.
    assert_eq!(
        thrown(LMB, sim::tuning::takeoff_window() as u32 + 2),
        Some(c::AIR_SWORD)
    );
}

#[test]
fn a_takeoff_costs_the_jump_it_came_out_of() {
    // One jump buys one of them. Without this the window would be a few frames
    // of free launchers rather than a way to spend a jump.
    let mut w = engaged(Class::Champion);
    run(&mut w, 1, MMB | Input::SPACE, 0);
    run(&mut w, 40, 0, 0);
    assert!(!w.players[0].grounded, "the uppercut never left the floor");
    run(&mut w, 2, LMB, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(moves::champion::AIR_SWORD),
        "a second takeoff came out of one jump"
    );
}

#[test]
fn the_spear_takeoff_buys_height_and_a_direction() {
    // It is a jump with a weapon in it: the plant supplies height a jump cannot
    // reach, and the run you were holding comes with you. Both halves, or it is
    // only half a move.
    let plain = {
        let mut w = engaged(Class::Champion);
        w.players[1].pos = sim::V3::new(sim::Fx::from_int(40), w.players[1].pos.y, sim::Fx::ZERO);
        let mut top = 0.0f32;
        for f in 0..80 {
            let bits = if f < 3 { Input::SPACE } else { 0 };
            w.advance([Input::aimed(bits, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
            top = top.max(w.players[0].pos.y.to_f32_for_render());
        }
        top
    };
    let (driven, ground) = {
        let mut w = engaged(Class::Champion);
        // Nobody in the way: this is about how far the plant carries you, and a
        // body you are already touching is a wall.
        w.players[1].pos = sim::V3::new(sim::Fx::from_int(40), w.players[1].pos.y, sim::Fx::ZERO);
        let from = w.players[0].pos;
        let (mut top, mut far) = (0.0f32, 0.0f32);
        for f in 0..80 {
            let bits = if f < 3 { Input::SPACE | RMB } else { 0 } | Input::W;
            w.advance([Input::aimed(bits, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
            top = top.max(w.players[0].pos.y.to_f32_for_render());
            far = far.max(w.players[0].pos.sub(from).flat_len().to_f32_for_render());
        }
        (top, far)
    };
    assert!(
        driven > plain + 0.5,
        "the pole drive reached {driven:.2} m and a plain jump {plain:.2} m"
    );
    assert!(
        ground > 2.0,
        "the pole drive went {ground:.2} m along the floor, so it has no direction in it"
    );
}

#[test]
fn the_sword_takeoff_is_the_hardest_hitting_of_the_three() {
    // The three takeoffs are a real choice, and the sword's is the one you
    // throw when you have read them: no grab, no boost, just the biggest number
    // in the row and a long fall if it misses.
    use moves::champion as c;
    let damage = |kind: u8| moves::get(Class::Champion, kind).damage;
    assert!(
        damage(c::RISING_CUT) > damage(c::UPPERCUT)
            && damage(c::RISING_CUT) > damage(c::POLE_DRIVE),
        "the sword's takeoff does not hit hardest"
    );
    assert!(
        moves::get(Class::Champion, c::UPPERCUT).grabs > 0,
        "the uppercut does not hold on to anybody, so there is nothing to follow"
    );
    assert!(
        moves::get(Class::Champion, c::POLE_DRIVE).self_lift.raw()
            > moves::get(Class::Champion, c::RISING_CUT).self_lift.raw(),
        "the spear's takeoff does not go highest"
    );
}

#[test]
fn rush_cancels_a_recovery() {
    // The class's "break my own pattern" tool, and the reason one charge is a
    // real decision. Without it every combo ends where the frame data says it
    // ends.
    let mut w = engaged(Class::Champion);
    run(&mut w, 2, MMB, 0);
    // Deep into the hammer's recovery, which is the longest tail it has.
    run(&mut w, 24, 0, 0);
    assert!(
        matches!(w.players[0].action, Action::Recovery { .. }),
        "the fixture is not in a recovery: {:?}",
        w.players[0].action
    );
    run(&mut w, 1, E, 0);
    assert!(
        matches!(w.players[0].action, Action::Free),
        "Rush did not cancel the recovery: {:?}",
        w.players[0].action
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

// ---------------------------------------------------------------------------
// The Guillotine lotus, as a shape
// ---------------------------------------------------------------------------
//
// Six blades out of the Reaver's shadow and six back into it. What follows is
// the *flower* rather than the cast -- where it is aimed is `aiming.rs`'s, and
// what it does to somebody is the hit test's. These are the three things a
// player reads off the screen while deciding whether they are standing in one.
//
// It used to be a different shape in all three respects: the blades left the
// shadow's feet, arched up over the eruption and came back down to the floor at
// full extension, and then retraced that arm exactly on the way home. The arc
// left the volume half buried in the floor at precisely the reach where it does
// the most work, and a flower whose height changes while it turns is hard to
// read as a plane being swept.

/// A lotus standing at the origin, at whatever age we want to look at it.
fn flower(age: u16) -> sim::effects::Effect {
    let mut e = sim::effects::Effect::cast(
        EffectKind::GuillotineLotus,
        0,
        Class::ShadowReaver,
        sim::state::SLOT_SPECIAL,
        sim::V3::ZERO,
        sim::V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
    );
    e.age = age;
    e
}

/// One blade's bearing around the centre, in turns, unwrapped against the
/// previous sample so a sweep past the seam reads as a sweep.
fn bearing(at: sim::V3) -> f64 {
    let x = at.x.to_f32_for_render() as f64;
    let z = at.z.to_f32_for_render() as f64;
    z.atan2(x) / std::f64::consts::TAU
}

fn reach(at: sim::V3) -> f64 {
    at.flat_len().to_f32_for_render() as f64
}

#[test]
fn the_flower_is_flat() {
    // One horizontal plane, for the whole life, for every blade. This is the
    // property a player actually uses: a volume at a fixed height is one you
    // can decide about once, and a volume that rises and falls while it turns
    // is one you have to keep re-reading.
    let life = EffectKind::GuillotineLotus.life();
    let want = sim::tuning::lotus_height();
    for age in 0..=life {
        for blade in 0..sim::effects::LOTUS_BLADES {
            let y = flower(age).lotus_at(blade, sim::V3::ZERO).y;
            assert_eq!(
                y.raw(),
                want.raw(),
                "blade {blade} at age {age} is {:.2} m up, not the {:.2} m the \
                 flower lies at -- the lotus is meant to be planar",
                y.to_f32_for_render(),
                want.to_f32_for_render()
            );
        }
    }
}

#[test]
fn the_blades_leave_from_the_midriff_and_not_from_the_feet() {
    // They erupt *out of the shadow*, so they start inside its body rather than
    // at the floor under it. Bracketed against the body rather than pinned to a
    // number: the point is that the plane is inside a standing fighter, which
    // is what makes the flower something you are caught *in*.
    let height = sim::tuning::lotus_height();
    let body = sim::tuning::body_height();
    assert!(
        height.raw() > 0,
        "the flower lies on the floor; the blades are supposed to come out of \
         the shadow's midriff"
    );
    assert!(
        height.raw() < body.raw(),
        "the flower lies at {:.2} m, over the head of a {:.2} m fighter",
        height.to_f32_for_render(),
        body.to_f32_for_render()
    );
    // Below the chest a cast comes out of: these are off the waist, not the
    // hands.
    assert!(
        height.raw() < sim::tuning::cast_height().raw(),
        "the flower lies at chest height or above, where a cast comes from, \
         rather than at the midriff"
    );
    // And a blade's own thickness does not put it underground at the start.
    assert!(
        height.raw() > sim::tuning::lotus_blade_radius().raw(),
        "a blade at rest is wider than the flower is high, so it starts half \
         buried in the floor"
    );
}

#[test]
fn the_blades_spiral_out_rather_than_running_straight() {
    // Each leaves on its own sixth of the circle and keeps turning as it goes,
    // which is what makes the six of them open like petals instead of as spokes
    // of a wheel. Reach and bearing have to advance *together* for that: a
    // radius with an angle bolted on somewhere else is a spoke that happens to
    // be rotating.
    let erupt = sim::tuning::lotus_erupt();
    let mut last_reach = -1.0;
    let mut last_bearing = bearing(flower(0).lotus_at(0, sim::V3::ZERO));
    let mut turned = 0.0;
    for age in 1..=erupt {
        let at = flower(age).lotus_at(0, sim::V3::ZERO);
        let (r, b) = (reach(at), bearing(at));
        assert!(
            r > last_reach,
            "the blade stopped reaching outward at age {age}"
        );
        turned += b - last_bearing;
        last_reach = r;
        last_bearing = b;
    }
    assert!(
        turned.abs() > 0.01,
        "the blade travelled {turned:.3} turns on the way out -- it went \
         straight, so this is a starburst and not a lotus"
    );
}

#[test]
fn the_way_home_turns_the_other_way_and_past_where_it_started() {
    // The fix this test exists for. The return used to be the eruption played
    // backwards, so the blade unwound onto the exact bearing it left on and the
    // flower rewound rather than closing. It now comes home on a spiral of its
    // own, turning against the way it opened and carrying on past the start.
    let centre = sim::V3::ZERO;
    let erupt = sim::tuning::lotus_erupt();
    let life = EffectKind::GuillotineLotus.life();

    let opened_on = bearing(flower(0).lotus_at(0, centre));
    let out_to = bearing(flower(erupt).lotus_at(0, centre));
    // The last frame with any reach left to measure a bearing from.
    let home = bearing(flower(life - 1).lotus_at(0, centre));

    let went = out_to - opened_on;
    let came = home - out_to;
    assert!(
        went * came < 0.0,
        "out {went:+.3} turns and home {came:+.3} turns: the way back does not \
         turn against the way out"
    );
    assert!(
        came.abs() > went.abs(),
        "out {went:+.3} turns and home {came:+.3} turns. The return unwinds by \
         no more than it wound, so it retraces the arm it came out on instead \
         of closing on a spiral of its own."
    );
    // Which is the same thing said as a place rather than as a rotation: it
    // finishes on the far side of the bearing it opened on.
    assert!(
        (home - opened_on) * went < 0.0,
        "the blade finished back on the side it opened towards, so it never \
         crossed its own starting bearing"
    );
}

#[test]
fn the_way_home_sweeps_floor_the_way_out_never_touched() {
    // And this is why that is a hit test rather than a look. Ground a blade has
    // already crossed is ground whose occupants have been cut once and have had
    // the whole hold to walk off it, so a retraced return can only catch
    // somebody who stepped back into the same line. Compared at matched reach,
    // because that is the only fair comparison: the two passes are at the same
    // distance from the shadow but should not be at the same bearing.
    let centre = sim::V3::ZERO;
    let erupt = sim::tuning::lotus_erupt();
    let life = EffectKind::GuillotineLotus.life();
    let full = reach(flower(erupt).lotus_at(0, centre));

    let mut checked = 0;
    for age in 1..erupt {
        let out = flower(age).lotus_at(0, centre);
        let r = reach(out);
        // Skip the ends, where both passes are necessarily near the centre.
        if r < full * 0.25 || r > full * 0.9 {
            continue;
        }
        // The return frame that is at about this same distance out.
        let back = (erupt..life)
            .map(|a| flower(a).lotus_at(0, centre))
            .min_by(|x, y| {
                (reach(*x) - r)
                    .abs()
                    .partial_cmp(&(reach(*y) - r).abs())
                    .expect("reaches compare")
            })
            .expect("the return has frames");
        let apart = (bearing(back) - bearing(out)).abs();
        assert!(
            apart > 0.02,
            "at {r:.1} m out the return is only {:.1}° from the arm the \
             eruption drew. The way home is retracing already-cut ground.",
            apart * 360.0
        );
        checked += 1;
    }
    assert!(checked > 0, "the fixture compared nothing");
}

// ---------------------------------------------------------------------------
// The blades are blades
// ---------------------------------------------------------------------------

/// Cast a lotus with the victim standing `feet` metres off the floor, and say
/// how much the flower took off them.
///
/// She casts it at her own shadow, which is at her heel, so the blades sweep
/// out through anybody standing inside `lotus_radius` of her.
fn lotus_takes_off_a_victim_at(feet: Fx) -> i32 {
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = sim::V3::ZERO;
    w.players[0].facing = sim::V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    // Well inside the blades' reach, and well outside her own body.
    let stand = sim::V3::new(Fx::from_int(3), feet, Fx::ZERO);
    // Held there every frame, falling or not. This is a question about the
    // shape of the volume, not about how long somebody can hang in the air --
    // without the pin the airborne case simply lands before the blades arrive
    // and the test passes for the wrong reason.
    let hold = |w: &mut World, bits: u16| {
        w.players[1].pos = stand;
        w.players[1].vel = sim::V3::ZERO;
        w.advance([Input::new(bits), Input::default()]);
    };
    for _ in 0..20 {
        hold(&mut w, 0);
    }

    let start = w.players[1].health;
    let cast = moves::get(Class::ShadowReaver, sim::state::SLOT_SPECIAL);
    let life = EffectKind::GuillotineLotus.life() as u32;
    for f in 0..(cast.whiff_cost() as u32 + life + 10) {
        hold(&mut w, if f < 2 { Q } else { 0 });
    }
    start - w.players[1].health
}

#[test]
fn a_blade_is_a_disc_and_not_a_ball() {
    // A shuriken thrown flat: wide in the plane the flower lies in, and barely
    // there across it. As a sphere at the old radius it reached from a standing
    // fighter's shins to their chest, which is not a blade and is not a shape
    // anybody can do anything about.
    let wide = sim::tuning::lotus_blade_radius();
    let thick = sim::tuning::lotus_blade_thickness();
    assert!(
        wide.raw() > thick.raw() * 2,
        "a blade is {:.2} m wide and {:.2} m thick either side -- that is a \
         ball, not a disc",
        wide.to_f32_for_render(),
        thick.to_f32_for_render()
    );
    // And narrower than the body it cuts, or it is a wall with a spin on it.
    assert!(
        wide.raw() < sim::tuning::body_radius().raw(),
        "a blade is {:.2} m wide against a {:.2} m body. Six of these read as \
         beach balls, which is what the count and the size were changed for.",
        wide.to_f32_for_render(),
        sim::tuning::body_radius().to_f32_for_render()
    );
}

#[test]
fn the_flower_opens_twelve_blades() {
    assert_eq!(
        sim::effects::LOTUS_BLADES,
        12,
        "the flower's blade count changed; the hit mask, the damage per blade \
         and the renderer all read it"
    );
    // The bookkeeping has to be able to address all of them. This is a
    // compile-time assertion in `effects.rs` as well, because at six blades the
    // mask fitted a `u32` exactly and twelve would have truncated in silence --
    // a blade that shares a bit with another goes quiet the moment that one
    // lands.
    assert!(
        sim::effects::LOTUS_BLADES * sim::effects::VICTIMS <= 64,
        "the hit mask cannot address every blade against every victim"
    );
}

#[test]
fn a_single_blade_is_a_scratch_and_the_flower_is_the_threat() {
    // Twelve smaller blades at half the damage rather than six big ones. The
    // ability's danger is meant to be the shape it sweeps, not any one thing
    // landing -- which is also what makes a partial clip through the edge of it
    // feel like a graze instead of a punish.
    let blade = moves::get(Class::ShadowReaver, sim::state::SLOT_SPECIAL).damage;
    let poke = moves::get(Class::ShadowReaver, sim::state::SLOT_POKE).damage;
    assert!(
        blade * 3 < poke,
        "one blade takes {blade} against a {poke} poke. A single blade should \
         be a scratch."
    );
}

#[test]
fn the_flower_cuts_what_is_standing_in_it() {
    let dealt = lotus_takes_off_a_victim_at(Fx::ZERO);
    assert!(
        dealt > 0,
        "a fighter standing inside the flower took nothing at all"
    );
}

#[test]
fn the_flower_can_be_jumped() {
    // The counterplay the shape implies and a ball never allowed. The flower is
    // planar and at waist height, and a blade is a thin slab -- so being off
    // the ground above it is being out of it. A ball of the old radius reached
    // the shins to the chest and there was nothing to do about it but leave
    // sideways.
    let plane = sim::tuning::lotus_height();
    let thick = sim::tuning::lotus_blade_thickness();
    // Feet clear of the top of the slab, by a comfortable margin.
    let over = plane.add(thick).add(Fx::ratio(1, 2));
    // Stated against a body rather than against a jump arc: the clearance has
    // to be something a fighter can obviously get over, and her short hop is
    // 1.9 m by the frame table -- twice a body and well past this.
    assert!(
        over.raw() < sim::tuning::body_height().raw(),
        "clearing the flower needs {:.2} m, which is over a fighter's own \
         height -- 'jumpable' would be a claim about nothing",
        over.to_f32_for_render()
    );
    assert_eq!(
        lotus_takes_off_a_victim_at(over),
        0,
        "a fighter in the air above the flower was cut anyway -- the blades are \
         still behaving like balls in the vertical"
    );
}
