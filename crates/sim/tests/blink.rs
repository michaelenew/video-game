//! The spike on a pool.
//!
//! A pool is a heal and a plant. The blink to a pool was cut on 2026-11-04
//! (exploration 0038: the zip replaced it, `tests/world_bleeds.rs`), and the
//! file keeps its name for the spike on a pool it always held: the Black
//! spike erupts a pool it is raised out of, at the pool's size, and the
//! eruption sets off every pool it covers. See `docs/design/blood-mage.md`.

use sim::class::Class;
use sim::effects::{Effect, EffectKind};
use sim::moves::blood as b;
use sim::moves::blood::keys;
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

/// The pitch that puts the crosshair on a spot on the floor.
fn pitch_onto(w: &World, at: V3) -> i16 {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let scene = sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: &w.monsters,
        critters: &sim::critter::Critters::NONE,
        arena: &w.terrain(),
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

#[test]
fn a_spike_on_a_pool_erupts_at_the_pools_radius_and_drinks_all_of_it() {
    let w = mage();
    let spike = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE);
    // A spot on the floor eight metres out, and the crosshair on it.
    let at = w.players[0]
        .pos
        .add(w.players[0].facing.scale(Fx::from_int(8)));
    let pitch = pitch_onto(&w, at);

    // Now a big pool there, the dummy standing where the eruption reaches
    // but the spike's own disc does not, and grey to fill.
    let mut w = mage();
    w.players[0].health = t::max_health() - 500;
    w.players[0].grey = 500;
    let pool = Effect::pool(0, Class::BloodMage, b::CAST, at, 300);
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
    looking(&mut w, 2, keys::SPIKES, pitch, 0);
    // Past the frames it spreads out over (2026-10-19), so it has reached
    // its edge.
    run(
        &mut w,
        t::spike_rise_big() as u32 + 2 + t::erupt_spread() as u32,
        0,
        0,
    );
    let erupted = w
        .effects
        .iter()
        .flatten()
        .find(|e| e.kind == EffectKind::BlackSpike)
        .expect("the eruption is standing");
    assert!(erupted.erupted(), "the spike on a pool did not erupt");
    assert!(
        (erupted.spike_volume().radius.sub(radius)).abs().raw() < Fx::ratio(1, 10).raw(),
        "spread to {} m against {} m for a pool of 300",
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
fn an_eruption_sets_off_every_pool_it_covers() {
    // The chain: a spike on one pool among others brings up all of them, each
    // at its own size. The skill is in arranging the pools; the spike is what
    // cashes them in at once.
    let w = mage();
    let at = w.players[0]
        .pos
        .add(w.players[0].facing.scale(Fx::from_int(8)));
    let pitch = pitch_onto(&w, at);
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(8));
    w.players[0].health = t::max_health() - 600;
    w.players[0].grey = 600;
    // Three pools: one under the spike, one a stride away, one further still
    // -- inside the second's eruption but outside the first's.
    let first = t::erupt_radius().mul(Fx::from_int(300).sqrt());
    w.effects[0] = Some(Effect::pool(0, Class::BloodMage, b::CAST, at, 300));
    w.effects[1] = Some(Effect::pool(
        0,
        Class::BloodMage,
        b::CAST,
        V3::new(at.x, Fx::ZERO, at.z.add(first.sub(Fx::ratio(1, 2)))),
        300,
    ));
    w.effects[2] = Some(Effect::pool(
        0,
        Class::BloodMage,
        b::CAST,
        V3::new(
            at.x,
            Fx::ZERO,
            at.z.add(first.mul(Fx::from_int(2)).sub(Fx::from_int(1))),
        ),
        100,
    ));
    let before = w.players[0].health;
    looking(&mut w, 2, keys::SPIKES, pitch, 0);
    // Link by link since 2026-10-13: each waits `tuning::chain_delay`, and
    // since 2026-10-19 an eruption reaches the next only as it spreads.
    let link = t::chain_delay() as u32 + t::erupt_spread() as u32;
    let mut eruptions = 0;
    for _ in 0..t::spike_rise_big() as u32 + 4 * link + 4 {
        run(&mut w, 1, 0, 0);
        eruptions += w
            .effects
            .iter()
            .flatten()
            .filter(|e| e.kind == EffectKind::BlackSpike && e.erupted() && e.age == 1)
            .count();
    }
    // Three.
    assert!(
        eruptions >= 3,
        "three pools in a chain made {eruptions} eruptions"
    );
    // Nothing is left on the floor: the spike leaves no stain of its own
    // (2026-10-19).
    assert!(pools(&w).is_empty(), "a pool survived the chain");
    assert!(
        w.players[0].health - before > 600 - 20,
        "the chain drank only {} of 700 across three pools",
        w.players[0].health - before
    );
}
