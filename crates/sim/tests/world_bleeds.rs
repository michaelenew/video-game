//! **The Blood mage: the world bleeds** -- slice 1 of
//! `docs/design/exploration/0038_blood_mage_the_world_bleeds.md`, one test a
//! rule: the Cast embeds and leashes; the wrench shoves toward her and the
//! head reaps her stains on its way home; the zip goes to the head and a jump
//! banked near its end fires on arrival; the spike does nothing on bare
//! ground; her own spike launches her; a plant stands, is solid and lashes;
//! the Sweep through a plant reaps it into a cone; the chain and the Red
//! Hand of round three (2026-11-07) below that; Seize, cut that day, threw onto a plant
//! or slams; and the Sweep bleeds the floor.

use sim::class::{Class, Garden, Mechanic, Structure};
use sim::effects::{Effect, EffectKind, Head};
use sim::moves::blood as b;
use sim::moves::blood::keys;
use sim::state::Action;
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

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

/// A Blood mage facing +X in the open, and a Bulwark `apart` metres in front
/// of her, facing her.
fn mage_and(apart: f32) -> World {
    let mut w = World::with_classes([Class::BloodMage, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[1].pos = at(-6.0 + apart, 0.0, 8.0);
    w
}

/// Both of them, one frame: her buttons and pitch, his buttons.
fn step2(w: &mut World, bits: u16, pitch: i16, his: u16) {
    w.advance([
        Input::looking_at(bits, LOOK_RIGHT, pitch),
        Input::looking_at(his, LOOK_LEFT, 0),
    ]);
}

fn run(w: &mut World, frames: u32, bits: u16, pitch: i16) {
    for _ in 0..frames {
        step2(w, bits, pitch, 0);
    }
}

fn head(w: &World) -> Option<Effect> {
    w.effects
        .iter()
        .flatten()
        .find(|e| e.kind == EffectKind::ScytheHead)
        .copied()
}

fn garden(w: &World) -> Garden {
    match w.players[0].mechanic {
        Mechanic::Blood(g) => g,
        _ => panic!("not a Blood mage"),
    }
}

fn plants(w: &World) -> usize {
    garden(w).plants.iter().flatten().count()
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

/// A stain of hers at `pos`, settled.
fn stain(w: &mut World, pos: V3, essence: i32) {
    let mut e = Effect::pool(0, Class::BloodMage, b::CAST, pos, essence);
    e.age = t::stain_settles() + 1;
    place(w, e);
}

/// A plant of hers at `pos`, holding `blood` of which she paid `paid`.
fn plant_at(w: &mut World, pos: V3, blood: u16, paid: u16) {
    if let Mechanic::Blood(g) = &mut w.players[0].mechanic {
        g.plants[0] = Some(Structure::planted(pos));
        g.blood[0] = blood;
        g.paid[0] = paid;
    }
}

fn flat_apart(a: V3, b: V3) -> f32 {
    let (dx, dz) = (as_f(a.x.sub(b.x)), as_f(a.z.sub(b.z)));
    (dx * dx + dz * dz).sqrt()
}

/// Hold the scythe until the head is out, then let go: the Cast.
fn cast(w: &mut World, pitch: i16) {
    let hold = t::hold_from() as u32 + 2;
    run(w, hold, keys::SCYTHE, pitch);
    for _ in 0..60 {
        if head(w).is_some() {
            break;
        }
        step2(w, 0, pitch, 0);
    }
}

/// Cast at each of a few pitches, and keep the first world where the head
/// bit what `bit` asks for.
fn cast_until(w: &World, bit: impl Fn(Head) -> bool) -> World {
    for pitch in [0, -500, 500, -1000, 1000, -2000, -3000, -4000, -5000] {
        let mut trial = w.clone();
        cast(&mut trial, pitch);
        for _ in 0..40 {
            if head(&trial).is_some_and(|e| bit(e.head_state())) {
                return trial;
            }
            step2(&mut trial, 0, pitch, 0);
        }
    }
    panic!("no pitch put the head where it was wanted");
}

// ---------------------------------------------------------------------------

#[test]
fn the_cast_embeds_in_a_fighter_and_leashes_them() {
    let w = mage_and(8.0);
    let mut w = cast_until(&w, |h| h == Head::Fighter(1));
    let e = head(&w).unwrap();
    let leash = as_f(e.reach);
    assert!(
        (leash - 8.0).abs() < 1.5,
        "the leash is the distance at the bite: {leash}"
    );
    assert!(w.players[1].health < w.players[1].full_health(), "it cut");
    assert!(w.players[1].bleeding > 0, "and opened the bleed");
    // He walks away, and cannot get further from her than the chain.
    for _ in 0..30 {
        step2(&mut w, 0, 0, Input::S);
        if head(&w).is_some_and(|e| e.head_state() == Head::Fighter(1)) {
            assert!(w.players[1].slowed > 0, "held, he is slowed");
            let apart = flat_apart(w.players[0].pos, w.players[1].pos);
            assert!(apart <= leash + 0.05, "{apart} past the leash {leash}");
        }
    }
}

#[test]
fn the_wrench_shoves_them_toward_her_and_the_head_reaps_her_stains_home() {
    let w = mage_and(8.0);
    let mut w = cast_until(&w, |h| h == Head::Fighter(1));
    w.players[0].wound(300);
    // A stain on the floor between them, on the head's way home.
    stain(&mut w, at(-2.0, 0.0, 8.0), 120);
    let grey = w.players[0].grey;
    assert!(grey > 0);
    let mut wrenched = false;
    for _ in 0..t::cast_holds() as u32 + 4 {
        step2(&mut w, 0, 0, 0);
        if matches!(w.players[1].action, Action::Stagger { .. }) && !wrenched {
            wrenched = true;
            assert!(
                as_f(w.players[1].vel.x) < -1.0,
                "shoved toward her: {}",
                as_f(w.players[1].vel.x)
            );
        }
    }
    assert!(wrenched, "it wrenched free and staggered him");
    let health = w.players[0].health;
    for _ in 0..90 {
        step2(&mut w, 0, 0, 0);
    }
    assert!(head(&w).is_none(), "the head came home");
    assert!(
        pools(&w)
            .iter()
            .all(|p| flat_apart(p.pos, at(-2.0, 0.0, 8.0)) > 1.0 || p.banked < 100),
        "the stain on its line was drunk"
    );
    assert!(w.players[0].health > health, "and gave her back her grey");
}

#[test]
fn the_zip_goes_to_the_head_and_a_banked_jump_fires_on_arrival() {
    let w = mage_and(30.0);
    let bit = cast_until(&w, |h| h == Head::World);
    let to = head(&bit).unwrap().home;
    let from = bit.players[0].pos;
    assert!(flat_apart(from, to) > 3.0, "the head is out in the world");
    let zip = |jump: bool| {
        let mut w = bit.clone();
        // Let go of the scythe, then tap it again: the zip (2026-11-10).
        run(&mut w, 2, 0, 0);
        step2(&mut w, keys::SCYTHE, 0, 0);
        step2(&mut w, 0, 0, 0);
        assert!(garden(&w).zipping, "a tap of right is the zip");
        let mut arrived = None;
        let mut held = 0;
        for _ in 0..90 {
            // Near the end of the pull, at the speed she is going (a pull
            // since 2026-11-07, not a haul).
            let near = garden(&w).zipping
                && flat_apart(w.players[0].pos, w.players[0].haul_to)
                    < as_f(w.players[0].vel.len()) / 60.0 * 3.0;
            let space = if jump && near { Input::SPACE } else { 0 };
            step2(&mut w, held | space, 0, 0);
            held = 0;
            if !garden(&w).zipping {
                arrived = Some(w.players[0]);
                break;
            }
        }
        arrived.expect("she arrived")
    };
    let plain = zip(false);
    assert!(
        flat_apart(plain.pos, to) < 1.5,
        "the zip took her to the head"
    );
    assert!(
        as_f(plain.vel.x) > as_f(t::zip_speed()) * 0.5,
        "and she keeps its speed: {}",
        as_f(plain.vel.x)
    );
    let jumped = zip(true);
    assert!(
        as_f(jumped.vel.y) > as_f(plain.vel.y) + 3.0,
        "a jump banked near the end fires on arrival: {} against {}",
        as_f(jumped.vel.y),
        as_f(plain.vel.y)
    );
}

#[test]
fn the_spike_on_bare_ground_throws_nothing() {
    // From play, 2026-11-10: "The best move is still to spam middle click
    // which we need to change. Middle click should only happen when there's
    // a bloodstain on the ground." (It raised her own blood there from
    // 2026-11-06.)
    let mut w = mage_and(30.0);
    let health = w.players[0].health;
    for f in 0..120u32 {
        step2(&mut w, if f % 4 < 2 { keys::SPIKES } else { 0 }, -3000, 0);
        assert!(
            w.players[0].action.attack_kind().is_none(),
            "a press on bare ground threw {:?}",
            w.players[0].action
        );
    }
    assert_eq!(plants(&w), 0, "no plant");
    assert_eq!(w.players[0].health, health, "and it cost nothing");
}

#[test]
fn the_rising_spike_on_bare_ground_is_her_own_blood_and_the_plant_holds_it() {
    // From play, 2026-11-06: on bare ground the Rising spike pays the whole
    // price in her own blood -- her way up -- the plant holds it as grey that
    // does not fade, and reaping the plant gives it back.
    let mut w = mage_and(30.0);
    let health = w.players[0].health;
    step2(&mut w, Input::SPACE, 0, 0);
    step2(&mut w, Input::SPACE | keys::SPIKES, 0, 0);
    run(&mut w, 40, 0, -3000);
    assert_eq!(plants(&w), 1, "a plant stands on bare ground");
    let price = health - w.players[0].health;
    assert!(price > 0, "and she paid for it");
    assert_eq!(w.players[0].grey, price, "as grey");
    run(&mut w, 400, 0, -3000);
    assert_eq!(plants(&w), 1, "it is still standing, seven seconds on");
    assert_eq!(
        w.players[0].grey, price,
        "and what it holds has not faded off her bar"
    );
    // Walk to it and sweep through it: reaped, she drinks it back.
    let plant = garden(&w).plants[0].unwrap().at;
    w.players[0].pos = at(as_f(plant.x) - 2.0, 0.0, as_f(plant.z));
    step2(&mut w, keys::SCYTHE, 0, 0);
    run(&mut w, 20, 0, 0);
    assert_eq!(plants(&w), 0, "reaped");
    assert_eq!(w.players[0].health, health, "and her blood came back");
}

#[test]
fn a_pool_pays_toward_the_spike_and_she_pays_what_it_lacks() {
    let price_over = |volume: i32| {
        let mut w = mage_and(30.0);
        let health = w.players[0].health;
        if volume > 0 {
            stain(&mut w, at(-1.0, 0.0, 8.0), volume);
        }
        let pitch = aim_onto(&w, at(-1.0, 0.0, 8.0));
        step2(&mut w, keys::SPIKES, pitch, 0);
        health - w.players[0].health
    };
    // The whole price, less the one drop of the smallest pool: on bare
    // ground the spike throws nothing (2026-11-10).
    assert_eq!(price_over(0), 0, "bare ground: nothing, and free");
    let full = price_over(1) + 1;
    assert!(full > 1);
    assert_eq!(
        price_over(full / 2),
        full - full / 2,
        "half a pool, half the price"
    );
    assert_eq!(price_over(full * 3), 0, "a big pool pays it all");
}

/// The pitch that puts her crosshair's floor point nearest `spot`: asked of
/// the spike's own line (`aim::grounded_path`), since a press on bare ground
/// throws nothing (2026-11-10) and so aims nothing.
fn aim_onto(w: &World, spot: V3) -> i16 {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let terrain = w.terrain();
    let scene = sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: &w.monsters,
        critters: &sim::critter::Critters::NONE,
        arena: &terrain,
    };
    let reach = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE).reach;
    let mut best = (f32::MAX, 0i16);
    for pitch in (500..14000).step_by(100).map(|p| -(p as i16)) {
        let look = Input::looking_at(keys::SPIKES, LOOK_RIGHT, pitch);
        let to = sim::aim::grounded_path(0, look, reach, &scene).to;
        let far = flat_apart(to, spot);
        if far < best.0 {
            best = (far, pitch);
        }
    }
    best.1
}

#[test]
fn her_own_spike_launches_her_and_stands_as_a_plant() {
    let mut w = mage_and(30.0);
    let feet = w.players[0].pos;
    stain(&mut w, feet, 60);
    step2(&mut w, Input::SPACE, 0, 0);
    step2(&mut w, Input::SPACE | keys::SPIKES, 0, 0);
    let mut highest = Fx::ZERO;
    let mut up = Fx::ZERO;
    for _ in 0..40 {
        step2(&mut w, 0, 0, 0);
        highest = highest.max(w.players[0].pos.y);
        up = up.max(w.players[0].vel.y);
    }
    assert!(
        as_f(up) >= as_f(t::spike_launches_her()) * 0.9,
        "her own spike threw her up: {}",
        as_f(up)
    );
    assert!(as_f(highest) > 2.0, "high: {}", as_f(highest));
    assert_eq!(plants(&w), 1, "and it stands");
    assert!(pools(&w).is_empty(), "the pool it rose out of is spent");
}

#[test]
fn a_spike_rises_slower_out_of_more_blood() {
    let rise = |volume: i32| {
        let mut w = mage_and(30.0);
        stain(&mut w, at(-1.0, 0.0, 8.0), volume);
        let pitch = aim_onto(&w, at(-1.0, 0.0, 8.0));
        step2(&mut w, keys::SPIKES, pitch, 0);
        match w.players[0].action {
            Action::Startup { kind, left } if kind == b::BLACK_SPIKE => left,
            other => panic!("no spike: {other:?}"),
        }
    };
    let (small, body, big) = (rise(10), rise(t::pool_full()), rise(600));
    assert!(small < body && body < big, "{small} {body} {big}");
    assert_eq!(small, t::spike_rise_small());
    assert_eq!(big, t::spike_rise_big());
}

#[test]
fn a_plant_stands_is_solid_and_lashes() {
    let mut w = mage_and(30.0);
    let spot = at(0.0, 0.0, 8.0);
    plant_at(&mut w, spot, 0, 0);
    // An enemy beside it.
    w.players[1].pos = at(1.2, 0.0, 8.0);
    let health = w.players[1].health;
    run(&mut w, t::plant_lash_every() as u32 + 1, 0, 0);
    assert!(w.players[1].health < health, "it lashed him");
    assert!(w.players[1].slowed > 0, "and slowed him");
    // Solid: dropped on to it, a body stands on its top.
    let top = Structure::planted(spot).top();
    w.players[1].pos = V3::new(spot.x, top.add(Fx::ONE), spot.z);
    w.players[1].grounded = false;
    run(&mut w, 40, 0, 0);
    assert!(
        (as_f(w.players[1].pos.y) - as_f(top)).abs() < 0.1,
        "he stands on it: {} against its top {}",
        as_f(w.players[1].pos.y),
        as_f(top)
    );
    // And it withers.
    run(&mut w, t::plant_stands() as u32, 0, 0);
    assert_eq!(plants(&w), 0, "a plant stands only so long");
}

#[test]
fn the_sweep_through_a_plant_reaps_it_into_a_cone() {
    let mut w = mage_and(6.0);
    plant_at(&mut w, at(-4.4, 0.0, 8.0), 40, 30);
    w.players[0].wound(200);
    let (health, foe) = (w.players[0].health, w.players[1].health);
    step2(&mut w, keys::SCYTHE, 0, 0);
    run(&mut w, 20, 0, 0);
    assert_eq!(plants(&w), 0, "the scythe broke it");
    let m = sim::moves::get(Class::BloodMage, b::THORNS);
    assert!(
        foe - w.players[1].health >= m.damage + 40,
        "its thorns flew on through him, worth what it held: {}",
        foe - w.players[1].health
    );
    let sweep = sim::moves::get(Class::BloodMage, b::SWEEP);
    let cost = w.players[0].full_health() * sweep.cost as i32 / 100;
    assert!(
        w.players[0].health >= health + 30 - cost,
        "and what she paid came back: {} from {health}",
        w.players[0].health
    );
}

#[test]
fn the_sweep_bleeds_the_floor_and_the_spike_raises_it() {
    let mut w = mage_and(30.0);
    step2(&mut w, keys::SCYTHE, 0, 0);
    // Since 2026-11-08 a tap begins as the button comes up, a frame on.
    run(&mut w, 32, 0, 0);
    // Two: the floor the Sweep cut, and what the Sweep cost her, a stain at
    // the blade's end as every cost is.
    let cut: Vec<Effect> = pools(&w)
        .into_iter()
        .filter(|p| p.is_world_blood())
        .collect();
    assert_eq!(cut.len(), 1, "the floor the Sweep cut bled");
    assert!(
        cut[0].banked > t::sweep_bleeds() / 2,
        "what it cut, less a little drain"
    );
    assert!(
        flat_apart(cut[0].pos, w.players[0].pos) > 0.5,
        "in front of her"
    );
    // And it is a pool to raise -- the crosshair on it (on bare ground the
    // spike throws nothing since 2026-11-10, so the aim has to find it).
    let mut raised = false;
    let onto = aim_onto(&w, cut[0].pos);
    for pitch in [onto, onto - 300, onto + 300, onto - 600, onto + 600] {
        let mut trial = w.clone();
        step2(&mut trial, keys::SPIKES, pitch, 0);
        run(&mut trial, 40, 0, pitch);
        if plants(&trial) == 1 {
            raised = true;
            break;
        }
    }
    assert!(raised, "the spike raised the floor's blood");
}

// ---------------------------------------------------------------------------
// From play, 2026-11-06
// ---------------------------------------------------------------------------

#[test]
fn a_press_while_the_head_flies_is_kept_and_zips_when_it_bites() {
    // "it pulls the mage a few inches towards it": a press a beat early was
    // dropped, and the next one threw a fresh Sweep.
    let w = mage_and(30.0);
    let mut w = w;
    let hold = t::hold_from() as u32 + 2;
    run(&mut w, hold, keys::SCYTHE, -3000);
    let from = w.players[0].pos;
    // Pressed again at once -- before the head has bitten anything.
    step2(&mut w, 0, -3000, 0);
    step2(&mut w, keys::SCYTHE, -3000, 0);
    let mut bit = None;
    for _ in 0..90 {
        step2(&mut w, 0, -3000, 0);
        if let Some(h) = head(&w).filter(|h| h.head_state() != Head::Flying) {
            bit.get_or_insert(h.home);
        }
    }
    let bit = bit.expect("the head bit the floor");
    let gone = flat_apart(from, w.players[0].pos);
    let out = flat_apart(from, bit);
    assert!(out > 5.0, "the head went out: {out}");
    assert!(
        gone > out - 2.0,
        "and the early press took her all the way to it: {gone} of {out}"
    );
}

#[test]
fn a_zip_onto_a_fighter_stops_her_beside_him() {
    let mut w = mage_and(8.0);
    let hold = t::hold_from() as u32 + 2;
    run(&mut w, hold, keys::SCYTHE, 0);
    step2(&mut w, 0, 0, 0);
    step2(&mut w, keys::SCYTHE, 0, 0);
    run(&mut w, 60, 0, 0);
    let gap = flat_apart(w.players[0].pos, w.players[1].pos);
    let reach = as_f(sim::moves::get(Class::BloodMage, b::REND).reach);
    assert!(
        gap < reach,
        "she arrived inside the Red Hand's reach of him and stayed: {gap} m"
    );
}

#[test]
fn the_rising_reap_takes_her_far_higher_than_a_jump() {
    let top = |bits: u16| {
        let mut w = mage_and(30.0);
        let mut top = Fx::ZERO;
        for f in 0..90 {
            step2(&mut w, if f < 12 { bits } else { 0 }, 0, 0);
            top = top.max(w.players[0].pos.y);
        }
        as_f(top)
    };
    let (jump, reap) = (top(Input::SPACE), top(Input::SPACE | keys::SCYTHE));
    assert!(
        reap > jump * 2.0,
        "the Rising reap {reap} m against a jump {jump} m"
    );
}

#[test]
fn taking_off_from_a_plant_never_reaps_it() {
    // From play, 2026-11-07: "jumping off a tree that spawned from a blood pool
    // causes it to explode into thorns" -- a Rising reap or a Sweep thrown
    // standing on a plant cut down through the plant under her feet.
    for bits in [Input::SPACE, Input::SPACE | keys::SCYTHE, keys::SCYTHE] {
        let mut w = mage_and(30.0);
        let mut f = 0;
        loop {
            step2(
                &mut w,
                if f < 3 {
                    Input::SPACE | keys::SPIKES
                } else {
                    0
                },
                0,
                0,
            );
            f += 1;
            if (f > 20 && w.players[0].grounded) || f > 400 {
                break;
            }
        }
        assert_eq!(plants(&w), 1, "fixture: she is standing on her plant");
        run(&mut w, 3, bits, 0);
        run(&mut w, 40, 0, 0);
        assert_eq!(plants(&w), 1, "{bits:#x} off its top reaped it");
    }
}

// ---------------------------------------------------------------------------
// Round three, 2026-11-07: the owner's tweaks (exploration 0038, the promise
// card). `cargo run --release -p sim --example kitcheck` measures the same.
// ---------------------------------------------------------------------------

/// The head in a fighter 10 m off.
fn hooked() -> World {
    for pitch in (-1500..1500).step_by(150) {
        let mut w = mage_and(10.0);
        run(
            &mut w,
            t::hold_from() as u32 + 2,
            keys::SCYTHE,
            pitch as i16,
        );
        for _ in 0..40 {
            step2(&mut w, 0, pitch as i16, 0);
            if head(&w).is_some_and(|h| matches!(h.head_state(), Head::Fighter(_))) {
                return w;
            }
        }
    }
    panic!("no throw bit the fighter");
}

#[test]
fn the_head_is_a_weight_thrown_on_a_chain() {
    // Thrown level at nothing, it rises, falls, and lands on the floor.
    let mut w = mage_and(60.0);
    run(&mut w, t::hold_from() as u32 + 2, keys::SCYTHE, 0);
    let (mut fell, mut was, mut landed) = (false, 0.0f32, None);
    for _ in 0..120 {
        step2(&mut w, 0, 0, 0);
        if let Some(h) = head(&w) {
            let y = as_f(h.home.y);
            fell |= h.head_state() == Head::Flying && y < was;
            was = y;
            if h.head_state() == Head::World {
                landed.get_or_insert(y);
            }
        }
    }
    assert!(fell, "it fell");
    assert!(
        landed.is_some_and(|y| y < 0.3),
        "and landed on the floor: {landed:?}"
    );
    // And aimed at a fighter, it still meets him.
    let _ = hooked();
}

#[test]
fn right_on_the_chain_pulls_her_reels_in_or_rips_it_out() {
    // From play, 2026-11-10: "When I rclick to send out the scythe head, I
    // then have to lclick to zip to it. That's backwards, it's rclick that
    // controls the scythe, and rclick does nothing while it's out."
    let mut w = hooked();
    step2(&mut w, keys::SCYTHE, 0, 0);
    step2(&mut w, 0, 0, 0);
    assert!(garden(&w).zipping, "a tap of right is the zip");
    run(&mut w, 40, 0, 0);
    assert!(
        flat_apart(w.players[0].pos, w.players[1].pos) < 2.6,
        "a tap pulled her to him"
    );

    let mut w = hooked();
    run(&mut w, t::reel_full() as u32 + 2, keys::SCYTHE, 0);
    run(&mut w, 90, 0, 0);
    assert!(
        flat_apart(w.players[0].pos, w.players[1].pos) < 2.5,
        "held, she reeled him in"
    );

    let mut w = hooked();
    let his = w.players[1].health;
    run(&mut w, t::hold_from() as u32 + 4, keys::SCYTHE, 0);
    step2(&mut w, 0, 0, 0);
    assert!(
        w.players[1].health < his,
        "let go early, it ripped out of him"
    );
    assert!(w.players[1].bleeding > 0);

    // In the world, held, the chain brings the head home -- and no press on
    // the chain swings the bladeless scythe.
    let mut w = cast_until(&mage_and(30.0), |h| h == Head::World);
    run(&mut w, 2, 0, 0);
    for _ in 0..t::reel_full() as u32 + 4 {
        step2(&mut w, keys::SCYTHE, 0, 0);
        assert!(
            !matches!(w.players[0].action, Action::Channel { kind, .. } | Action::Startup { kind, .. } if kind == b::SWEEP),
            "the chain's press threw a Sweep"
        );
    }
    assert!(
        head(&w).is_some_and(|h| h.head_state() == Head::Home),
        "held, the chain reeled the head home"
    );
    assert!(!garden(&w).zipping, "and did not zip her");

    // And the left hand is the Red Hand while the head is out.
    let mut w = hooked();
    while w.players[0].action != Action::Free {
        step2(&mut w, 0, 0, 0);
    }
    assert!(
        head(&w).is_some_and(|h| h.is_a_head_holding()),
        "fixture: still in him"
    );
    step2(&mut w, keys::HAND, 0, 0);
    run(&mut w, 2, 0, 0);
    assert!(
        matches!(w.players[0].action, Action::Startup { kind, .. } | Action::Active { kind, .. } if kind == b::REND),
        "left with the head out is Rend: {:?}",
        w.players[0].action
    );
}

#[test]
fn the_shrike_impales_a_body_reeled_into_her_plant() {
    let mut w = hooked();
    plant_at(&mut w, at(-1.0, 0.0, 8.0), 0, 0);
    let mut pinned = false;
    for f in 0..120u32 {
        step2(
            &mut w,
            if f < t::reel_full() as u32 + 2 {
                keys::SCYTHE
            } else {
                0
            },
            0,
            0,
        );
        pinned |= matches!(w.players[1].action, Action::Held { .. })
            && w.players[1].held_by == sim::state::NAILED;
    }
    assert!(pinned, "he was impaled on the plant between them");
}

#[test]
fn the_red_claw_lands_further_and_bigger_for_the_hold_and_slows() {
    let claw_at = |held: u32| {
        let mut w = mage_and(40.0);
        let from = w.players[0].pos;
        run(&mut w, held, keys::HAND, -2000);
        for _ in 0..30 {
            step2(&mut w, 0, -2000, 0);
            if let Some(c) = w
                .effects
                .iter()
                .flatten()
                .find(|e| e.kind == EffectKind::Claw)
            {
                return (flat_apart(from, c.pos), as_f(c.reach));
            }
        }
        panic!("no claw came out");
    };
    let (near, small) = claw_at(t::hold_from() as u32 + 8);
    let (far, big) = claw_at(t::hold_from() as u32 + 45);
    assert!(
        far > near + 3.0 && big > small,
        "{near} {small} -> {far} {big}"
    );
    // Held too long, it lands past a fighter 3 m off.
    let mut w = mage_and(3.0);
    let his = w.players[1].health;
    run(&mut w, t::hold_from() as u32 + 70, keys::HAND, -2000);
    run(&mut w, 30, 0, -2000);
    assert_eq!(w.players[1].health, his, "over-held, it missed him");
}

#[test]
fn rend_strung_is_three_blows_each_bigger_and_the_hand_feeds_a_plant() {
    let mut w = mage_and(2.2);
    let (mut last, mut hits) = (w.players[1].health, Vec::new());
    for f in 0..80u32 {
        step2(
            &mut w,
            if matches!(f, 0 | 16 | 32) {
                keys::HAND
            } else {
                0
            },
            0,
            0,
        );
        if w.players[1].health != last {
            hits.push(last - w.players[1].health);
            last = w.players[1].health;
        }
    }
    assert!(
        hits.len() == 3 && hits[0] < hits[1] && hits[1] < hits[2],
        "{hits:?}"
    );
    let mut w = mage_and(40.0);
    plant_at(&mut w, at(-4.4, 0.0, 8.0), 0, 0);
    let before = garden(&w).plants[0].unwrap().scale;
    step2(&mut w, keys::HAND, 0, 0);
    run(&mut w, 20, 0, 0);
    assert!(
        garden(&w).plants[0].unwrap().scale.raw() > before.raw(),
        "fed"
    );
}

#[test]
fn the_charge_jump_leaps_further_for_the_hold_and_the_claw_rides_it() {
    let leap = |held: u32| {
        let mut w = mage_and(40.0);
        let from = w.players[0].pos;
        for f in 0..120u32 {
            let bits = if f < 2 {
                Input::SPACE | keys::HAND
            } else if f < 2 + held {
                keys::HAND
            } else {
                0
            };
            step2(&mut w, bits, 0, 0);
        }
        flat_apart(from, w.players[0].pos)
    };
    assert!(leap(t::leap_hold() as u32 + 4) > leap(0) + 2.0);
    let mut w = mage_and(5.0);
    let his = w.players[1].health;
    for f in 0..80u32 {
        let bits = if f < 2 {
            Input::SPACE | keys::HAND
        } else if f < 2 + t::leap_hold() as u32 {
            keys::HAND
        } else {
            0
        };
        step2(&mut w, bits, 0, 0);
    }
    assert!(w.players[1].health < his, "it hit him on the way");
}

// ---------------------------------------------------------------------------
// From play, 2026-11-08: one or the other, and a held button does not repeat
// ---------------------------------------------------------------------------

#[test]
fn a_held_click_is_the_held_move_and_never_the_tap_first_nor_again() {
    // "Both l and r click, when holding, the short version comes out then
    // the long one. That's not right, it's one or the other."
    for (click, tap) in [(keys::HAND, b::REND), (keys::SCYTHE, b::SWEEP)] {
        let mut w = mage_and(30.0);
        for _ in 0..t::hold_from() as u32 + 30 {
            step2(&mut w, click, 0, 0);
            assert!(
                !matches!(
                    w.players[0].action,
                    Action::Startup { kind, .. } | Action::Active { kind, .. } | Action::Recovery { kind, .. }
                        if kind == tap
                ),
                "held, {:?} wound up the tap first",
                w.players[0].action
            );
        }
    }
    // "just holding [the click] while aiming up shoots the scythe head, then
    // it disappears, then shoots, over and over."
    let mut w = mage_and(30.0);
    let mut throws = 0;
    let mut had = false;
    for _ in 0..300 {
        step2(&mut w, keys::SCYTHE, 3000, 0);
        let out = head(&w).is_some_and(|h| h.age <= 1 && h.head_state() == Head::Flying);
        if out && !had {
            throws += 1;
        }
        had = out;
    }
    assert_eq!(throws, 1, "held five seconds, it threw once");
    // And a tap is still the tap.
    let mut w = mage_and(30.0);
    step2(&mut w, keys::SCYTHE, 0, 0);
    run(&mut w, 12, 0, 0);
    assert!(
        matches!(w.players[0].action, Action::Active { kind, .. } | Action::Recovery { kind, .. } if kind == b::SWEEP),
        "a tap swept"
    );
}
