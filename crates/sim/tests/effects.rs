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
use sim::{Input, World};

const Q: u16 = Input::SPECIAL;
const E: u16 = Input::MECHANIC;
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    for _ in 0..frames {
        w.advance([Input::aimed(a, LOOK_RIGHT), Input::aimed(b, LOOK_LEFT)]);
    }
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
        w.advance([
            Input::aimed(0, LOOK_RIGHT),
            Input::aimed(Input::SHIFT | Input::LEFT, LOOK_LEFT),
        ]);
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
// ---------------------------------------------------------------------------

#[test]
fn the_black_spike_drains_and_slows_whoever_stands_in_it() {
    // Not a damage puddle: the slow is what makes it a wall. Leaving costs you
    // time, which is the whole reason to put one between yourself and someone.
    let mut w = engaged(Class::BloodMage);
    let before = w.players[1].health;
    tap(&mut w, Input::SHIFT | Input::LEFT, 80);
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
fn a_slowed_fighter_covers_less_ground() {
    let mut w = engaged(Class::BloodMage);
    tap(&mut w, Input::SHIFT | Input::LEFT, 80);
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

#[test]
fn the_uppercut_takes_both_fighters_off_the_ground() {
    // The move is a leap, and it is a leap you bring someone along on. Either
    // half alone is a different move: without the lift it is a launcher you
    // cannot follow up on, and without the launch it is an escape.
    //
    // It is a **Rush move** now -- middle click during the dash -- which is
    // what makes it a combo rather than a button. See `docs/design/champion.md`.
    let mut w = rushing(LOOK_RIGHT);
    let mut lifted = [false; MAX_PLAYERS];
    run(&mut w, 2, MMB, 0);
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
        let mut w = rushing(LOOK_RIGHT);
        run(&mut w, 2, MMB, 0);
        let mut best = 0.0f32;
        for f in 0..70 {
            // Held down it would be one press; tapped, it is an input.
            let space = if leap && (10..12).contains(&f) {
                Input::SPACE
            } else {
                0
            };
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
    let mut w = engaged(Class::Champion);
    run(&mut w, 4, Input::SPACE, 0);
    assert!(!w.players[0].grounded, "the fixture never left the ground");
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
    // say any of this -- it takes a shape.
    //
    // Sword: **across**. It catches somebody standing well off to the side and
    // it does not reach far in front.
    assert!(
        reaches(LMB, 0.9, 1.4, false),
        "the sword does not cover its own flank"
    );
    assert!(
        !reaches(LMB, 3.2, 0.0, false),
        "the sword reaches as far as a spear"
    );

    // Spear: **out**. The longest line in the game, and a thin one -- it misses
    // the same flank the sword owns.
    assert!(reaches(RMB, 3.2, 0.0, false), "the spear does not reach");
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
fn the_air_spear_pays_its_shove_out_on_contact() {
    // The fan is a repositioning tool you have to earn: catching somebody with
    // it kicks you the way you are holding. Thrown at nothing it is just a
    // poke, which is what stops it being free flight.
    let speed = |hit: bool| {
        let mut w = engaged(Class::Champion);
        if !hit {
            // Out of the way, so the identical script whiffs.
            w.players[1].pos =
                sim::V3::new(sim::Fx::from_int(25), w.players[1].pos.y, sim::Fx::ZERO);
        }
        run(&mut w, 4, Input::SPACE, 0);
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
    // Hammer, cancel the recovery with Rush, uppercut into the air, leap
    // higher, and spike them back into the floor. Every piece of this is tested
    // on its own above; this is the assertion that they **connect** -- that the
    // hammer's advantage is long enough to Rush out of, that the uppercut
    // catches somebody already in hitstun, that the carry lasts long enough to
    // throw a twenty-two frame startup off the end of, and that the spike lands
    // before they hit the ground on their own.
    //
    // A loop that works step by step and does not join up is the usual way a
    // combo class turns out not to be one.
    let mut w = engaged(Class::Champion);
    let start = w.players[1].health;

    run(&mut w, 2, MMB, 0); // hammer
    run(&mut w, 22, 0, 0); // into its recovery
    run(&mut w, 1, E, 0); // Rush, cancelling it
    run(&mut w, 2, MMB, 0); // uppercut, which grabs and lifts
    run(&mut w, 8, 0, 0);
    run(&mut w, 1, Input::SPACE, 0); // both of you, higher
    run(&mut w, 24, 0, 0);
    assert!(
        !w.players[1].grounded && w.players[1].pos.y.to_f32_for_render() > 3.0,
        "the uppercut did not take them anywhere: {:.2} m",
        w.players[1].pos.y.to_f32_for_render()
    );

    // And the spike, aimed down.
    let down = -(1 << 13);
    let mut staggered = false;
    for f in 0..80 {
        let bits = if f < 2 { MMB } else { 0 };
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
