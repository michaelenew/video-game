//! **The Blood mage: the world bleeds** -- slice 1 of
//! `docs/design/exploration/0038_blood_mage_the_world_bleeds.md`, one test a
//! rule: the Cast embeds and leashes; the wrench shoves toward her and the
//! head reaps her stains on its way home; the zip goes to the head and a jump
//! banked near its end fires on arrival; the spike does nothing on bare
//! ground; her own spike launches her; a plant stands, is solid and lashes;
//! the Sweep through a plant reaps it into a cone; Seize throws onto a plant
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
        // Let go of the scythe, then press it again.
        run(&mut w, 2, 0, 0);
        step2(&mut w, keys::SCYTHE, 0, 0);
        assert!(garden(&w).zipping, "the second press is the zip");
        let mut arrived = None;
        let mut held = keys::SCYTHE;
        for _ in 0..90 {
            let near = w.players[0].haul > 0
                && flat_apart(w.players[0].pos, w.players[0].haul_to)
                    < as_f(t::zip_speed()) / 60.0 * 3.0;
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
fn the_spike_does_nothing_on_bare_ground() {
    let mut w = mage_and(30.0);
    let health = w.players[0].health;
    for pitch in [-2000, -4000] {
        step2(&mut w, keys::SPIKES, pitch, 0);
        assert!(
            !matches!(w.players[0].action, Action::Startup { kind, .. } if kind == b::BLACK_SPIKE),
            "nothing came out"
        );
        run(&mut w, 30, 0, pitch);
    }
    assert_eq!(w.players[0].health, health, "and it cost nothing");
    assert_eq!(plants(&w), 0);
    assert!(
        !w.effects
            .iter()
            .flatten()
            .any(|e| e.kind == EffectKind::BlackSpike)
    );
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
        for pitch in [-1500, -2000, -2500, -3000, -3500, -4000, -1000] {
            let mut trial = w.clone();
            step2(&mut trial, keys::SPIKES, pitch, 0);
            if let Action::Startup { kind, left } = trial.players[0].action
                && kind == b::BLACK_SPIKE
            {
                return left;
            }
        }
        panic!("no pitch found the pool");
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
fn seize_slams_with_no_plant_near_and_throws_onto_one() {
    let seize = |w: &mut World| {
        for _ in 0..40 {
            step2(w, keys::HAND, 0, 0);
        }
        run(w, 30, 0, 0);
    };
    // No plant: slammed, and the floor bleeds in a ring round him.
    let mut w = mage_and(1.4);
    let foe = w.players[1].health;
    seize(&mut w);
    let slam = sim::moves::get(Class::BloodMage, b::SLAM);
    assert!(
        foe - w.players[1].health >= slam.damage,
        "slammed: {}",
        foe - w.players[1].health
    );
    let ring = pools(&w)
        .iter()
        .filter(|p| flat_apart(p.pos, w.players[1].pos) > 0.5)
        .count();
    assert!(ring >= 2, "a ring of blood round him: {ring}");

    // A plant beside him: thrown onto it, worth what it holds, and it drinks.
    let mut w = mage_and(1.4);
    plant_at(&mut w, at(-3.0, 0.0, 10.0), 30, 0);
    let foe = w.players[1].health;
    seize(&mut w);
    let impale = sim::moves::get(Class::BloodMage, b::IMPALE);
    assert!(
        foe - w.players[1].health >= impale.damage + 30,
        "impaled: {}",
        foe - w.players[1].health
    );
    assert!(garden(&w).blood[0] > 30, "and the plant drank");
}

#[test]
fn the_sweep_bleeds_the_floor_and_the_spike_raises_it() {
    let mut w = mage_and(30.0);
    step2(&mut w, keys::SCYTHE, 0, 0);
    run(&mut w, 30, 0, 0);
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
    // And it is a pool to raise.
    let mut raised = false;
    for pitch in [-2000, -3000, -4000, -5000, -6000, -1500] {
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
