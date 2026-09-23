//! The blink, the haul onto her feet, and the spike on a pool.
//!
//! A pool is a heal or a door. A dodge thrown with the crosshair on one of
//! her pools puts the Blood mage in it and spends it; the Grasp hauls a
//! victim to her feet, or her to the creature; and the Black spike seeds a
//! pool on bare floor or erupts one it is cast on. See
//! `docs/design/blood-mage.md` and `plans/blood-mage-v1.md` M3.

use sim::class::Class;
use sim::effects::{Effect, EffectKind};
use sim::moves::blood as b;
use sim::state::Action;
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

const LOOK_LEFT: u16 = 1 << 15;

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    looking(w, frames, a, 0, b);
}

fn looking(w: &mut World, frames: u32, a: u16, pitch: i16, b: u16) {
    for _ in 0..frames {
        w.advance([Input::looking_at(a, 0, pitch), Input::aimed(b, LOOK_LEFT)]);
    }
}

fn mage() -> World {
    World::with_classes([Class::BloodMage, Class::Bulwark])
}

fn pools(w: &World) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.is_a_pool())
        .copied()
        .collect()
}

/// A pool of hers `far` metres ahead, and nobody in the way.
fn pool_ahead(w: &mut World, far: i32) -> V3 {
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(8));
    let at = w.players[0]
        .pos
        .add(w.players[0].facing.scale(Fx::from_int(far)));
    w.effects[0] = Some(Effect::pool(0, Class::BloodMage, b::SWEEP, at, 200));
    at
}

/// The pitch that puts the crosshair on a spot on the floor.
fn pitch_onto(w: &World, at: V3) -> i16 {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let scene = sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
    };
    (-40..=80)
        .map(|step| -(step * 200) as i16)
        .min_by_key(|pitch| {
            let look = Input::looking_at(0, 0, *pitch);
            sim::aim::grounded_path(0, look, Fx::from_int(20), &scene)
                .to
                .sub(at)
                .len()
                .raw()
        })
        .expect("the scan is not empty")
}

/// Dodge forward with the crosshair on `at`.
fn dodge_at(w: &mut World, at: V3) {
    let pitch = pitch_onto(w, at);
    looking(w, 1, Input::SHIFT | Input::W, pitch, 0);
}

#[test]
fn a_dodge_with_the_crosshair_on_a_pool_puts_her_in_it_and_spends_it() {
    let mut w = mage();
    let at = pool_ahead(&mut w, 8);
    dodge_at(&mut w, at);
    let landed = w.players[0].pos.sub(at).flat_len();
    assert!(
        landed.raw() < Fx::ratio(1, 4).raw(),
        "she is {} m from the pool she blinked to",
        landed.to_f32_for_render()
    );
    assert!(
        matches!(w.players[0].action, Action::Dodge { .. }),
        "the blink is not a dodge"
    );
    assert!(pools(&w).is_empty(), "the pool survived the blink");
}

#[test]
fn the_blink_is_refused_with_no_pool_under_the_crosshair() {
    let mut w = mage();
    let at = pool_ahead(&mut w, 8);
    // Look well off it: up, into the sky. Negative pitch is down, the way
    // the scan in `pitch_onto` finds the floor.
    looking(&mut w, 1, Input::SHIFT | Input::W, 8000, 0);
    let moved = w.players[0].pos.sub(at).flat_len();
    assert!(
        moved.raw() > Fx::from_int(6).raw(),
        "she blinked with the crosshair nowhere near the pool"
    );
    assert!(
        matches!(w.players[0].action, Action::Dodge { .. }),
        "an ordinary dodge was refused too"
    );
    assert_eq!(pools(&w).len(), 1, "a refused blink spent the pool");
}

#[test]
fn the_blink_is_refused_when_a_stone_blocks_the_line() {
    let mut w = World::with_classes([Class::BloodMage, Class::Elementalist]);
    let at = pool_ahead(&mut w, 8);
    // A stone in the way, raised by the Elementalist between the two.
    let between = w.players[0]
        .pos
        .add(w.players[0].facing.scale(Fx::from_int(4)));
    sim::stones::raise(&mut w.players[1], sim::class::Structure::raised(between));
    for _ in 0..t::structure_rise() as u32 + 2 {
        run(&mut w, 1, 0, 0);
    }
    let start = w.players[0].pos;
    dodge_at(&mut w, at);
    let crossed = w.players[0].pos.sub(start).flat_len();
    assert!(
        crossed.raw() < Fx::from_int(2).raw(),
        "she blinked {} m through a stone",
        crossed.to_f32_for_render()
    );
    assert_eq!(pools(&w).len(), 1, "a refused blink spent the pool");
}

#[test]
fn the_blink_is_taken_in_the_air_and_refused_once_the_airdodge_is_spent() {
    // Off the floor the blink is the airdodge, aimed: it costs the one
    // airborne commitment, so a fighter who has already spent it stays where
    // she is. A pool is on the floor, so a blink taken in the air lands her
    // -- and landing hands the airdodge back, the way it always does.
    let mut w = mage();
    let at = pool_ahead(&mut w, 8);
    run(&mut w, 1, Input::SPACE, 0);
    run(&mut w, 6, 0, 0);
    assert!(!w.players[0].grounded, "fixture: she never left the floor");
    dodge_at(&mut w, at);
    let landed = w.players[0].pos.sub(at).flat_len();
    assert!(
        landed.raw() < Fx::ratio(1, 4).raw(),
        "the airborne blink left her {} m from the pool",
        landed.to_f32_for_render()
    );
    assert!(
        pools(&w).is_empty(),
        "the airborne blink did not spend the pool"
    );

    // Again, with the airdodge already spent this airtime.
    let mut w = mage();
    let at = pool_ahead(&mut w, 8);
    run(&mut w, 1, Input::SPACE, 0);
    run(&mut w, 6, 0, 0);
    w.players[0].air_dodged = true;
    let before = w.players[0].pos;
    dodge_at(&mut w, at);
    assert!(
        w.players[0].pos.sub(before).flat_len().raw() < Fx::from_int(2).raw(),
        "she blinked in the air with the airdodge already spent"
    );
    assert_eq!(pools(&w).len(), 1, "a refused blink spent the pool");
}

#[test]
fn nobody_else_blinks_to_her_pools() {
    let mut w = World::with_classes([Class::Bulwark, Class::BloodMage]);
    let at = w.players[0]
        .pos
        .add(w.players[0].facing.scale(Fx::from_int(8)));
    w.effects[0] = Some(Effect::pool(1, Class::BloodMage, b::SWEEP, at, 200));
    let start = w.players[0].pos;
    dodge_at(&mut w, at);
    assert!(
        w.players[0].pos.sub(start).flat_len().raw() < Fx::from_int(2).raw(),
        "the Bulwark blinked"
    );
}

#[test]
fn a_spike_on_a_pool_erupts_at_the_pools_radius_and_drinks_all_of_it() {
    let mut w = mage();
    let spike = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE);
    // Where the spike lands with the crosshair level: find it once.
    looking(&mut w, 2, Input::MECHANIC, 0, 0);
    let mut where_ = None;
    for _ in 0..spike.startup as u32 + 4 {
        run(&mut w, 1, 0, 0);
        if let Some(e) = w
            .effects
            .iter()
            .flatten()
            .find(|e| e.kind == EffectKind::BlackSpike)
        {
            where_ = Some(e.pos);
            break;
        }
    }
    let at = where_.expect("the spike never came up");

    // Now a big pool there, the dummy standing where the eruption reaches
    // but the spike's own disc does not, and grey to fill.
    let mut w = mage();
    w.players[0].health = t::max_health() - 500;
    w.players[0].grey = 500;
    let pool = Effect::pool(0, Class::BloodMage, b::SWEEP, at, 300);
    w.effects[0] = Some(pool);
    let radius = t::erupt_radius().mul(Fx::from_int(300).sqrt());
    assert!(
        radius.raw() > spike.radius.add(t::body_radius()).raw(),
        "fixture: the eruption is no wider than the bare spike"
    );
    w.players[1].pos = V3::new(at.x, Fx::ZERO, at.z.add(radius.sub(t::body_radius())));
    let full = w.players[1].health;
    let before = w.players[0].health;
    let paid = w.players[0].cost_of(spike.cost);
    looking(&mut w, 2, Input::MECHANIC, 0, 0);
    run(&mut w, spike.startup as u32 + 4, 0, 0);
    let erupted = w
        .effects
        .iter()
        .flatten()
        .find(|e| e.kind == EffectKind::BlackSpike)
        .expect("the eruption is standing");
    assert!(erupted.erupted(), "the spike on a pool did not erupt");
    assert!(
        (erupted.spike_volume().radius.sub(radius)).abs().raw() < Fx::ratio(1, 10).raw(),
        "erupted at {} m against {} m for a pool of 300",
        erupted.spike_volume().radius.to_f32_for_render(),
        radius.to_f32_for_render()
    );
    assert!(
        erupted.spike_volume().top.raw() > t::spike_height().raw(),
        "the eruption stands no taller than a bare spike"
    );
    assert!(
        w.players[1].health < full,
        "the dummy at the pool's edge, well outside the spike's own disc, was not hit"
    );
    assert!(w.players[1].slowed > 0, "the eruption did not slow");
    assert!(
        !w.players[1].grounded || w.players[1].vel.y.raw() > 0 || spike.launch.raw() == 0,
        "the eruption did not launch"
    );
    let got = w.players[0].health - (before - paid);
    assert!(got >= 300 - 8, "the eruption drank {got} of a pool of 300");
    // The pool is spent, whatever the hit spilled since.
    assert!(
        pools(&w).iter().all(|p| p.pool_volume() < 300 - 100),
        "the pool the spike erupted from is still there"
    );
}

#[test]
fn a_spike_on_bare_floor_returns_nothing_and_leaves_a_pool_where_it_hit() {
    let mut w = mage();
    let spike = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE);
    w.players[0].health = t::max_health() - 500;
    w.players[0].grey = 500;
    // Find where it lands, then stand the dummy there.
    looking(&mut w, 2, Input::MECHANIC, 0, 0);
    let mut at = None;
    for _ in 0..spike.startup as u32 + 4 {
        run(&mut w, 1, 0, 0);
        if let Some(e) = w
            .effects
            .iter()
            .flatten()
            .find(|e| e.kind == EffectKind::BlackSpike)
        {
            at = Some(e.pos);
            break;
        }
    }
    let at = at.expect("the spike never came up");
    let mut w = mage();
    w.players[0].health = t::max_health() - 500;
    w.players[0].grey = 500;
    w.players[1].pos = V3::new(at.x, Fx::ZERO, at.z);
    let before = w.players[0].health;
    let paid = w.players[0].cost_of(spike.cost);
    looking(&mut w, 2, Input::MECHANIC, 0, 0);
    run(&mut w, spike.startup as u32 + 4, 0, 0);
    assert!(
        w.players[1].health < t::max_health(),
        "fixture: the spike missed"
    );
    assert_eq!(
        w.players[0].health,
        before - paid,
        "a spike on bare floor returned health"
    );
    assert!(w.players[1].slowed > 0, "the spike did not slow");
    let made = pools(&w);
    assert_eq!(made.len(), 1, "the spike left {} pools", made.len());
    assert!(
        made[0].pos.sub(at).flat_len().raw() < Fx::from_int(1).raw(),
        "the pool is not where the spike hit"
    );
}

#[test]
fn a_victim_hauled_by_the_grasp_stands_at_her_feet_when_the_hold_ends() {
    // The catch hauls to her feet -- touching -- so a mage standing in a
    // pool when the arms close has the victim on the essence, held, and
    // **the Grasp drinks that pool as they land on it**: the kit's own
    // sentence is "Grasp them onto the pool you are standing in".
    let mut w = mage();
    w.players[0].health = t::max_health() - 300;
    w.players[0].grey = 300;
    let grasp = sim::moves::get(Class::BloodMage, b::GRASP);
    w.players[1].pos = w.players[0].pos.add(V3::new(
        grasp.reach.sub(Fx::from_int(1)),
        Fx::ZERO,
        Fx::ZERO,
    ));
    // A pool under her own feet.
    w.effects[0] = Some(Effect::pool(
        0,
        Class::BloodMage,
        b::SWEEP,
        w.players[0].pos,
        150,
    ));
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let scene = sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: None,
    };
    let middle = sim::aim::standing_middle(w.players[1].pos);
    let pitch = (-40..=80)
        .map(|step| -(step * 200) as i16)
        .min_by_key(|pitch| {
            let look = Input::looking_at(0, 0, *pitch);
            let path = sim::aim::skillshot_path(0, look, grasp.reach, &scene);
            let dir = path.dir();
            let down = middle
                .sub(path.from)
                .dot(dir)
                .max(Fx::ZERO)
                .min(grasp.reach);
            path.from.add(dir.scale(down)).sub(middle).len().raw()
        })
        .expect("the scan is not empty");
    let before = w.players[0].health;
    let paid = w.players[0].cost_of(grasp.cost);
    looking(&mut w, grasp.channel as u32 + 1, Input::SPECIAL, pitch, 0);
    looking(&mut w, 1, 0, pitch, 0);
    let mut held = false;
    let mut let_go_at = None;
    for _ in 0..200 {
        run(&mut w, 1, 0, 0);
        if matches!(w.players[1].action, Action::Held { .. }) {
            held = true;
        } else if held {
            let_go_at = Some(w.players[1].pos);
            break;
        }
    }
    assert!(held, "fixture: the Grasp never caught");
    let at = let_go_at.expect("the hold never ended");
    let gap = at.sub(w.players[0].pos).flat_len();
    assert!(
        gap.raw() <= t::body_radius().mul(Fx::from_int(2)).raw() + Fx::ratio(1, 10).raw(),
        "let go {} m from her, not at her feet",
        gap.to_f32_for_render()
    );
    // Onto the pool she stands in -- and the pool is drunk as they land on
    // it: gone, and her share of it back.
    assert!(
        w.effects[0].is_none_or(|e| !e.is_a_pool()),
        "hauled onto her feet and the pool she stands in was not drunk"
    );
    let got = w.players[0].health - (before - paid);
    let expect = grasp.drinks(150);
    assert!(
        got >= expect - 10 && got <= expect,
        "the haul onto her pool drank {got} against a share of {expect}"
    );
}

#[test]
fn all_four_arms_on_the_creature_haul_her_to_it() {
    use sim::monster::Doing;
    // Along a lane clear of the arena's two platforms, which sit either side
    // of the middle: a haul into the side of one stops there, correctly.
    let mut w = World::hunt([Class::BloodMage, Class::BloodMage]);
    let mut beast = w.monster.expect("a hunt has a creature");
    beast.pos = V3::new(Fx::from_int(4), Fx::ZERO, Fx::from_int(7));
    beast.yaw = Fx::ratio(1, 2);
    beast.doing = Doing::Prowl;
    beast.brain.think_left = u16::MAX;
    w.monster = Some(beast);
    w.players[0].pos = V3::new(Fx::from_int(-4), Fx::ZERO, Fx::from_int(7));
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-8));
    let grasp = sim::moves::get(Class::BloodMage, b::GRASP);
    let start = w.players[0].pos;
    // Wound the Grasp all the way out, level at the flank.
    looking(&mut w, grasp.channel as u32 + 1, Input::SPECIAL, 0, 0);
    looking(&mut w, 1, 0, 0, 0);
    let mut hauled = false;
    for _ in 0..120 {
        run(&mut w, 1, 0, 0);
        if w.players[0].haul > 0 {
            hauled = true;
        }
    }
    assert!(
        hauled,
        "all four arms landed on the creature and nothing hauled her"
    );
    let crossed = w.players[0].pos.sub(start).flat_len();
    assert!(
        crossed.raw() > Fx::from_int(3).raw(),
        "she was hauled only {} m toward the creature",
        crossed.to_f32_for_render()
    );
}

#[test]
fn an_eruption_sets_off_every_pool_it_covers() {
    // The chain: a spike on one pool among others brings up all of them, each
    // at its own size. The skill is in arranging the pools; the spike is what
    // cashes them in at once.
    let mut w = mage();
    let spike = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE);
    looking(&mut w, 2, Input::MECHANIC, 0, 0);
    let mut at = None;
    for _ in 0..spike.startup as u32 + 4 {
        run(&mut w, 1, 0, 0);
        if let Some(e) = w
            .effects
            .iter()
            .flatten()
            .find(|e| e.kind == EffectKind::BlackSpike)
        {
            at = Some(e.pos);
            break;
        }
    }
    let at = at.expect("the spike never came up");
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(8));
    w.players[0].health = t::max_health() - 600;
    w.players[0].grey = 600;
    // Three pools: one under the spike, one a stride away, one further still
    // -- inside the second's eruption but outside the first's.
    let first = t::erupt_radius().mul(Fx::from_int(300).sqrt());
    w.effects[0] = Some(Effect::pool(0, Class::BloodMage, b::SWEEP, at, 300));
    w.effects[1] = Some(Effect::pool(
        0,
        Class::BloodMage,
        b::SWEEP,
        V3::new(at.x, Fx::ZERO, at.z.add(first.sub(Fx::ratio(1, 2)))),
        300,
    ));
    w.effects[2] = Some(Effect::pool(
        0,
        Class::BloodMage,
        b::SWEEP,
        V3::new(
            at.x,
            Fx::ZERO,
            at.z.add(first.mul(Fx::from_int(2)).sub(Fx::from_int(1))),
        ),
        100,
    ));
    let before = w.players[0].health;
    looking(&mut w, 2, Input::MECHANIC, 0, 0);
    run(&mut w, spike.startup as u32 + 2, 0, 0);
    let eruptions = w
        .effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::BlackSpike && e.erupted())
        .count();
    assert_eq!(
        eruptions, 3,
        "three pools in a chain made {eruptions} eruptions"
    );
    assert!(pools(&w).is_empty(), "a pool survived the chain");
    assert!(
        w.players[0].health - before > 600 - 20,
        "the chain drank only {} of 700 across three pools",
        w.players[0].health - before
    );
}

#[test]
fn the_scythe_collects_a_pool_it_passes_over_with_nobody_in_it() {
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(8));
    w.players[0].health = t::max_health() - 300;
    w.players[0].grey = 300;
    let sweep = sim::moves::get(Class::BloodMage, b::SWEEP);
    let ahead = w.players[0]
        .pos
        .add(w.players[0].facing.scale(sweep.reach.mul(Fx::ratio(1, 2))));
    w.effects[0] = Some(Effect::pool(0, Class::BloodMage, b::SWEEP, ahead, 100));
    let before = w.players[0].health;
    let paid = w.players[0].cost_of(sweep.cost);
    run(&mut w, 2, Input::LEFT, 0);
    run(&mut w, sweep.whiff_cost() as u32, 0, 0);
    assert!(
        pools(&w).is_empty(),
        "the sweep passed over the pool and left it"
    );
    assert!(
        w.players[0].health > before - paid,
        "the sweep collected the pool and drank nothing"
    );
}
