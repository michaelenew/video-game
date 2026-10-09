//! The Blood mage on three clicks: what a player would notice if any of it
//! broke.
//!
//! `docs/design/exploration/0009_blood_mage_on_three_clicks.md` is the
//! direction: left is *my blood*, middle is *your blood*, right is the scythe
//! and how she moves -- each on the floor, in the air and with space -- and
//! the combos are built out of those pieces. Each test below is one piece, and
//! the last is the Hanging, the combo the pieces were chosen for.

use sim::class::Class;
use sim::effects::{Effect, EffectKind};
use sim::moves::blood as b;
use sim::moves::blood::keys;
use sim::state::{Action, NAILED};
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

const SPACE: u16 = Input::SPACE;
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn metres(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}

fn at(x: f32, y: f32, z: f32) -> V3 {
    V3::new(metres(x), metres(y), metres(z))
}

fn as_f(v: Fx) -> f32 {
    v.raw() as f32 / 65536.0
}

/// A Blood mage facing +X in the open, and a Bulwark far enough off to be out
/// of everything unless a test moves him.
fn mage() -> World {
    let mut w = World::with_classes([Class::BloodMage, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[1].pos = at(11.0, 0.0, 8.0);
    w
}

fn step_looking(w: &mut World, bits: u16, pitch: i16) {
    w.advance([
        Input::looking_at(bits, LOOK_RIGHT, pitch),
        Input::looking_at(0, LOOK_LEFT, 0),
    ]);
}

fn step(w: &mut World, bits: u16) {
    step_looking(w, bits, 0);
}

fn run(w: &mut World, frames: u32, bits: u16) {
    for _ in 0..frames {
        step(w, bits);
    }
}

fn aloft(w: &mut World, who: usize, height: f32) {
    w.players[who].pos.y = metres(height);
    w.players[who].grounded = false;
}

fn doing(w: &World) -> Option<u8> {
    match w.players[0].action {
        Action::Startup { kind, .. }
        | Action::Active { kind, .. }
        | Action::Recovery { kind, .. }
        | Action::Channel { kind, .. } => Some(kind),
        _ => None,
    }
}

fn pools(w: &World) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.is_a_pool())
        .copied()
        .collect()
}

fn place(w: &mut World, effect: Effect) {
    let slot = w.effects.iter().position(|s| s.is_none()).expect("room");
    w.effects[slot] = Some(effect);
}

// ---------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------

#[test]
fn on_the_floor_the_clicks_are_my_blood_your_blood_and_the_scythe() {
    for (button, wanted) in [
        (keys::MY_BLOOD, b::BLOOD_NOVA),
        (keys::YOUR_BLOOD, b::GRASP),
        (keys::SCYTHE, b::SWEEP),
        (keys::BLOODLETTER, b::BLOODLETTER),
        (keys::SPIKE, b::BLACK_SPIKE),
    ] {
        let mut w = mage();
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} on the floor");
    }
}

#[test]
fn in_the_air_the_clicks_are_haemorrhage_the_nail_and_the_hook() {
    for (button, wanted) in [
        (keys::MY_BLOOD, b::HAEMORRHAGE),
        (keys::YOUR_BLOOD, b::NAIL),
        (keys::SCYTHE, b::HOOK),
    ] {
        let mut w = mage();
        aloft(&mut w, 0, 3.0);
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} in the air");
    }
}

#[test]
fn space_and_a_click_is_a_takeoff() {
    for (button, wanted) in [
        (keys::MY_BLOOD, b::BLOOD_JET),
        (keys::YOUR_BLOOD, b::MARIONETTE),
        (keys::SCYTHE, b::HARVEST),
    ] {
        let mut w = mage();
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x}");
    }
}

// ---------------------------------------------------------------------------
// My blood
// ---------------------------------------------------------------------------

/// Hold the nova for `hold` frames and let go; how much grey she has, and the
/// nova's radius.
fn nova(hold: u32) -> (i32, Fx, World) {
    let mut w = mage();
    for _ in 0..hold.max(1) {
        step(&mut w, keys::MY_BLOOD);
    }
    // The burst comes on the move's first active frame, after its wind-up.
    let startup = sim::moves::get(Class::BloodMage, b::BLOOD_NOVA).startup;
    run(&mut w, startup as u32 + 2, 0);
    let grey = w.players[0].grey;
    let radius = w
        .effects
        .iter()
        .flatten()
        .find(|e| e.kind == EffectKind::Nova)
        .map(|e| e.reach)
        .expect("the nova burst");
    (grey, radius, w)
}

#[test]
fn a_longer_nova_is_bigger_and_costs_more_blood() {
    let (grey_tap, small, _) = nova(1);
    let (grey_held, big, _) = nova(50);
    assert!(big.raw() > small.raw(), "held, the burst is bigger");
    assert!(grey_held > grey_tap, "and she paid more for it");
}

#[test]
fn the_nova_leaves_a_pool_of_her_own_blood_that_does_not_heal_her() {
    let (_, _, mut w) = nova(40);
    let own: Vec<Effect> = pools(&w)
        .into_iter()
        .filter(|p| b::own_blood(p.slot))
        .collect();
    assert_eq!(own.len(), 1, "a pool of her own blood where she stood");
    // Sweep straight through it: it is a door, not a heal.
    let red = w.players[0].health;
    let grey = w.players[0].grey;
    run(&mut w, 30, 0);
    step(&mut w, keys::SCYTHE);
    run(&mut w, 30, 0);
    assert!(
        w.players[0].health <= red,
        "her own blood healed her: {} -> {}",
        red,
        w.players[0].health
    );
    assert!(w.players[0].grey <= grey);
}

#[test]
fn the_nova_throws_off_whoever_is_close() {
    let mut w = mage();
    w.players[1].pos = at(-4.5, 0.0, 8.0);
    let before = w.players[1].health;
    for _ in 0..30 {
        step(&mut w, keys::MY_BLOOD);
    }
    run(&mut w, 20, 0);
    assert!(w.players[1].health < before, "he was hurt");
    assert!(
        w.players[1].pos.x.raw() > metres(-4.0).raw(),
        "and thrown off: he is at {}",
        as_f(w.players[1].pos.x)
    );
}

#[test]
fn the_jet_drives_her_up_along_her_aim_and_leaves_a_door_home() {
    let mut w = mage();
    let from = w.players[0].pos;
    // Aimed up and forward.
    step_looking(&mut w, SPACE | keys::MY_BLOOD, 6000);
    for _ in 0..30 {
        step_looking(&mut w, keys::MY_BLOOD, 6000);
    }
    let startup = sim::moves::get(Class::BloodMage, b::BLOOD_JET).startup;
    run(&mut w, startup as u32 + 2, 0);
    let p = w.players[0];
    assert!(
        p.pos.y.raw() > metres(3.0).raw(),
        "she went up: {}",
        as_f(p.pos.y)
    );
    assert!(p.pos.x.raw() > from.x.add(metres(2.0)).raw(), "and along");
    let own: Vec<Effect> = pools(&w)
        .into_iter()
        .filter(|p| b::own_blood(p.slot))
        .collect();
    assert_eq!(own.len(), 1, "a pool of her own blood where she took off");
    let gap = V3::new(own[0].pos.x.sub(from.x), Fx::ZERO, own[0].pos.z.sub(from.z));
    assert!(gap.flat_len().raw() < metres(0.5).raw());
}

// ---------------------------------------------------------------------------
// Your blood
// ---------------------------------------------------------------------------

#[test]
fn marionette_lifts_him() {
    let mut w = mage();
    w.players[1].pos = at(-4.6, 0.0, 8.0);
    step(&mut w, SPACE | keys::YOUR_BLOOD);
    let mut lifted = false;
    for _ in 0..30 {
        step(&mut w, 0);
        if w.players[1].pos.y.raw() > metres(1.0).raw() {
            lifted = true;
            break;
        }
    }
    assert!(lifted, "his own blood took him up");
}

/// The Nail thrown from above at a body `height` up, `ahead` metres off.
fn nail_at(height: f32) -> World {
    let mut w = mage();
    aloft(&mut w, 0, 4.0);
    w.players[1].pos = at(-2.0, height, 8.0);
    w.players[1].grounded = height <= 0.0;
    let hold = |w: &mut World| {
        w.players[0].pos.y = metres(4.0);
        w.players[0].vel.y = Fx::ZERO;
        if !w.players[1].grounded && !matches!(w.players[1].action, Action::Held { .. }) {
            w.players[1].pos.y = metres(height);
            w.players[1].vel.y = Fx::ZERO;
        }
    };
    // Aimed down at him a little.
    hold(&mut w);
    step_looking(&mut w, keys::YOUR_BLOOD, -2000);
    for _ in 0..30 {
        hold(&mut w);
        step(&mut w, 0);
        if matches!(w.players[1].action, Action::Held { .. }) {
            break;
        }
    }
    w
}

#[test]
fn the_nail_pins_somebody_in_the_air() {
    let mut w = nail_at(3.0);
    let him = w.players[1];
    assert!(
        matches!(him.action, Action::Held { .. }) && him.held_by == NAILED,
        "pinned: {:?}",
        him.action
    );
    let y = him.pos.y;
    run(&mut w, (t::nail_pin() / 2) as u32, 0);
    assert_eq!(w.players[1].pos.y, y, "held where he was, not falling");
    // And his blood fell to the floor below him.
    assert!(
        pools(&w)
            .iter()
            .any(|p| p.pos.y.raw() < metres(0.5).raw() && !b::own_blood(p.slot)),
        "his blood is on the floor under him"
    );
}

#[test]
fn the_nail_only_hits_somebody_on_the_floor() {
    let w = nail_at(0.0);
    assert_ne!(
        w.players[1].held_by, NAILED,
        "nobody is pinned on the floor"
    );
}

// ---------------------------------------------------------------------------
// The scythe
// ---------------------------------------------------------------------------

#[test]
fn the_hook_pulls_her_to_what_it_catches() {
    let mut w = mage();
    aloft(&mut w, 0, 1.0);
    w.players[1].pos = at(2.0, 0.0, 8.0);
    let before = as_f(w.players[1].pos.x.sub(w.players[0].pos.x));
    // The crosshair on him, as a player puts it there: the line from her hand
    // to a level crosshair passes over his head.
    let after = [-2000i16, -3000, -4000]
        .into_iter()
        .map(|pitch| {
            let mut trial = w.clone();
            step_looking(&mut trial, keys::SCYTHE, pitch);
            run(&mut trial, 30, 0);
            as_f(trial.players[1].pos.x.sub(trial.players[0].pos.x))
        })
        .fold(f32::MAX, f32::min);
    assert!(
        after < before - 4.0,
        "she was pulled to him: {before} m apart, then {after}"
    );
}

#[test]
fn harvest_drinks_the_pools_it_passes_over() {
    let mut w = mage();
    // Hurt, so there is grey to drink back.
    w.players[0].spend_health(40);
    let red = w.players[0].health;
    let feet = w.players[0].pos;
    place(
        &mut w,
        Effect::pool(
            0,
            Class::BloodMage,
            b::SWEEP,
            feet.add(at(1.0, 0.0, 0.0)),
            100,
        ),
    );
    run(&mut w, 2, 0);
    step(&mut w, SPACE | keys::SCYTHE);
    run(&mut w, 40, 0);
    assert!(w.players[0].health > red, "she drank on the way up");
    assert!(pools(&w).is_empty(), "and the pool is gone");
}

// ---------------------------------------------------------------------------
// The creature bleeds
// ---------------------------------------------------------------------------

#[test]
fn a_creature_bleeds_from_the_haemorrhage() {
    let mut w = World::hunt([Class::BloodMage; 2]);
    let beast = w.monsters[0].expect("a creature").pos;
    // A bolt cast where the creature is.
    let bolt = Effect::cast(
        EffectKind::Haemorrhage,
        0,
        Class::BloodMage,
        b::HAEMORRHAGE,
        beast.add(at(0.0, 1.5, 0.0)),
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
    );
    place(&mut w, bolt);
    let mut bled = false;
    for _ in 0..30 {
        w.advance([Input::default(); 2]);
        if w.monsters[0].is_some_and(|m| m.bleeding > 0) {
            bled = true;
            break;
        }
    }
    assert!(bled, "the creature bleeds");
    let before = pools(&w).len();
    for _ in 0..(t::bleed_tick() as u32 * 3) {
        w.advance([Input::default(); 2]);
    }
    assert!(
        pools(&w).len() >= before.max(1),
        "and its bleed spills under it"
    );
}

// ---------------------------------------------------------------------------
// The Hanging
// ---------------------------------------------------------------------------

#[test]
fn the_hanging_marionette_then_the_nail() {
    let mut w = mage();
    w.players[1].pos = at(-4.6, 0.0, 8.0);
    // Marionette: he goes up, and so does she.
    step(&mut w, SPACE | keys::YOUR_BLOOD);
    let mut ready = false;
    for _ in 0..40 {
        step(&mut w, 0);
        let me = w.players[0];
        if !me.grounded && me.action.actionable() && !w.players[1].grounded {
            ready = true;
            break;
        }
    }
    assert!(ready, "both in the air and her free to act");
    // The Nail at him, from wherever she is: pinned.
    let mut pinned = false;
    for pitch in [-3000, -1500, 0, 1500, 3000] {
        let mut trial = w.clone();
        step_looking(&mut trial, keys::YOUR_BLOOD, pitch);
        for _ in 0..30 {
            step(&mut trial, 0);
            if trial.players[1].held_by == NAILED {
                pinned = true;
                break;
            }
        }
        if pinned {
            break;
        }
    }
    assert!(pinned, "a Nail after Marionette pins him in the air");
}
