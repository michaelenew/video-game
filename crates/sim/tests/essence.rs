//! Essence pools: the other fighter's blood on the floor, and the drink.
//!
//! Every hit the Blood mage lands spills a pool under the target, sized by the
//! damage; pools drain, merge and are capped; and the only heal she has is a
//! move put through one. See `docs/design/blood-mage.md` §"Essence pools" and
//! §"Double duty", and `plans/blood-mage-v1.md` M2.

use sim::class::{ALL_CLASSES, Class};
use sim::effects::{Effect, EffectKind};
use sim::moves::blood as b;
use sim::moves::blood::keys;
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    looking(w, frames, a, 0, b);
}

fn looking(w: &mut World, frames: u32, a: u16, pitch: i16, b: u16) {
    for _ in 0..frames {
        w.advance([
            Input::looking_at(a, LOOK_RIGHT, pitch),
            Input::aimed(b, LOOK_LEFT),
        ]);
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

/// Stand the dummy a few metres off, inside the Haemorrhage's reach.
fn in_reach(w: &mut World) {
    w.players[1].pos = w.players[0]
        .pos
        .add(V3::new(Fx::from_int(5), Fx::ZERO, Fx::ZERO));
}

/// The Haemorrhage, tapped at the dummy's middle, and the frames it takes to
/// land.
fn bolt_at_him(w: &mut World) {
    let pitch = pitch_at(w, w.players[1].pos);
    looking(w, 2, keys::MY_BLOOD, pitch, 0);
    looking(w, 30, 0, pitch, 0);
}

/// Open some grey on her, so a drink has something to convert.
fn wounded(w: &mut World, grey: i32) {
    w.players[0].health = t::max_health() - grey;
    w.players[0].grey = grey;
}

/// A pitch that points the Bloodletter at the other fighter's middle.
fn pitch_at(w: &World, target: V3) -> i16 {
    let m = sim::moves::get(Class::BloodMage, b::HAEMORRHAGE);
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
    let middle = V3::new(
        target.x,
        target.y.add(t::body_height().div(Fx::from_int(2))),
        target.z,
    );
    let miss = |pitch: i16| {
        let look = Input::looking_at(0, LOOK_RIGHT, pitch);
        let path = sim::aim::skillshot_path(0, look, m.reach, &scene);
        let dir = path.dir();
        let toward = middle.sub(path.from);
        let down = toward.dot(dir).max(Fx::ZERO).min(m.reach);
        path.from.add(dir.scale(down)).sub(middle).len().raw()
    };
    (-40..=80)
        .map(|step| -(step * 200) as i16)
        .min_by_key(|pitch| miss(*pitch))
        .expect("the scan is not empty")
}

#[test]
fn every_hit_she_lands_spills_a_pool_under_the_target() {
    let mut w = mage();
    in_reach(&mut w);
    let full = w.players[1].health;
    bolt_at_him(&mut w);
    let dealt = full - w.players[1].health;
    assert!(dealt > 0, "fixture: the bolt never landed");
    let made = pools(&w);
    assert_eq!(made.len(), 1, "one hit made {} pools", made.len());
    let pool = made[0];
    assert_eq!(pool.owner, 0);
    let under = pool.pos.sub(w.players[1].pos).flat_len();
    assert!(
        under.raw() < Fx::ratio(1, 2).raw(),
        "the pool is {} m from the fighter it came out of",
        under.to_f32_for_render()
    );
    // Sized by the damage, less what has drained in the frames since -- and
    // her cost, which landed where she aimed, joined it.
    assert!(
        pool.pool_volume() > dealt * 3 / 4,
        "hit for {dealt} and the pool holds {}",
        pool.pool_volume()
    );
}

#[test]
fn a_pools_size_and_life_are_monotone_in_the_damage_that_made_it() {
    let small = sim::moves::get(Class::BloodMage, b::UPHEAVAL);
    let spike = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE);
    let life_of = |volume: i32| -> (Fx, u32) {
        let mut w = mage();
        w.effects[0] = Some(Effect::pool(
            0,
            Class::BloodMage,
            b::HAEMORRHAGE,
            V3::ZERO,
            volume,
        ));
        let radius = w.effects[0].expect("placed").pool_radius();
        for frame in 1..20_000 {
            run(&mut w, 1, 0, 0);
            if pools(&w).is_empty() {
                return (radius, frame);
            }
        }
        panic!("a pool of {volume} never drained");
    };
    let (small_r, small_life) = life_of(small.damage);
    let (big_r, big_life) = life_of(spike.damage);
    assert!(
        big_r.raw() > small_r.raw(),
        "a spike's pool is no wider than an Upheaval's"
    );
    // A spike's pool outlives an Upheaval's by at least the ratio of their damage:
    // the drain is a rate, so life is volume over rate.
    assert!(
        big_life * small.damage as u32 >= small_life * spike.damage as u32,
        "a spike's pool lives {big_life}f and an Upheaval's {small_life}f, against a \
         damage ratio of {}:{}",
        spike.damage,
        small.damage
    );
}

#[test]
fn hits_that_land_on_one_spot_make_one_pool() {
    // The Hemoplague's ticks land on one body standing in it, again and again,
    // and every one spills to the cloud's centre. One stain, not one a tick --
    // the merge is what keeps a fight that stays in one place from becoming
    // a floor of clutter.
    let mut w = mage();
    let spot = w.players[0]
        .pos
        .add(V3::new(Fx::from_int(4), Fx::ZERO, Fx::ZERO));
    w.players[1].pos = spot;
    let plague = sim::moves::get(Class::BloodMage, b::HEMOPLAGUE);
    let slot = w.effects.iter().position(|e| e.is_none()).expect("room");
    w.effects[slot] = Some(Effect::cast(
        EffectKind::Plague,
        0,
        Class::BloodMage,
        b::HEMOPLAGUE,
        spot,
        V3::ZERO,
        plague.radius,
    ));
    let full = w.players[1].health;
    for _ in 0..(t::plague_tick() as u32 * 3 + 1) {
        w.players[1].pos = spot;
        run(&mut w, 1, 0, 0);
    }
    let dealt = full - w.players[1].health;
    assert!(
        dealt >= plague.damage * 3,
        "fixture: only {dealt} landed of three ticks"
    );
    let made = pools(&w);
    assert_eq!(made.len(), 1, "three ticks made {} pools", made.len());
    assert!(
        made[0].pool_volume() > plague.damage * 2,
        "the stain did not gather the ticks: {} after {dealt} dealt",
        made[0].pool_volume()
    );
}

#[test]
fn a_fifth_pool_merges_into_the_newest() {
    let mut w = mage();
    let far = |i: i32| V3::new(Fx::from_int(-12 + i * 3), Fx::ZERO, Fx::from_int(6));
    for i in 0..t::pool_cap() as i32 {
        let mut pool = Effect::pool(0, Class::BloodMage, b::HAEMORRHAGE, far(i), 100);
        // Older the lower the index, so the newest is the last one placed.
        pool.age = (t::pool_cap() as i32 - i) as u16 * 10;
        w.effects[i as usize] = Some(pool);
    }
    let newest = w.effects[t::pool_cap() - 1].expect("placed").pool_volume();
    in_reach(&mut w);
    bolt_at_him(&mut w);
    let made = pools(&w);
    assert_eq!(
        made.len(),
        t::pool_cap(),
        "the cap did not hold: {} pools",
        made.len()
    );
    let grown = w.effects[t::pool_cap() - 1]
        .expect("still there")
        .pool_volume();
    assert!(
        grown > newest - 10,
        "the fifth hit did not merge into the newest pool ({newest} -> {grown})"
    );
}

#[test]
fn every_cost_she_pays_is_a_stain() {
    // Since 2026-10-13 nothing she spends is lost: what a press costs lands
    // where she aimed it as a stain like any other, hers to take back
    // (docs/design/exploration/0016_blood_mage_thralls_and_the_well.md).
    for (button, airborne) in [
        (keys::SCYTHE, false),
        (keys::MY_BLOOD, true),
        (keys::MY_BLOOD, false),
        (keys::YOUR_BLOOD, false),
    ] {
        let mut w = mage();
        w.players[1].pos = V3::new(Fx::from_int(-18), Fx::ZERO, Fx::ZERO);
        if airborne {
            w.players[0].pos.y = Fx::from_int(3);
            w.players[0].grounded = false;
        }
        looking(&mut w, 2, button, -2000, 0);
        run(&mut w, 70, 0, 0);
        assert!(
            w.players[0].grey > 0,
            "fixture: {button:#x} cost her nothing"
        );
        assert!(
            !pools(&w).is_empty(),
            "{button:#x}: what it cost is not on the floor"
        );
    }
}

#[test]
fn nobody_else_spills_anybody() {
    for class in ALL_CLASSES {
        // The Elementalist's left click goes through bodies since 2026-10-02
        // (`bolt::targets`): it never hits her, so it has nothing to say here.
        if class == Class::BloodMage || class == Class::Elementalist {
            continue;
        }
        let mut w = World::with_classes([class, Class::BloodMage]);
        w.players[1].pos = w.players[0]
            .pos
            .add(V3::new(Fx::ratio(3, 2), Fx::ZERO, Fx::ZERO));
        // Pressed, then let go: a Champion's weapon held down is a charge.
        run(&mut w, 1, Input::LEFT, 0);
        run(&mut w, 59, 0, 0);
        assert!(
            w.players[1].health < w.players[1].full_health(),
            "fixture: {} never hit her",
            class.name()
        );
        assert!(
            pools(&w).is_empty(),
            "{} spilled the Blood mage onto the floor",
            class.name()
        );
    }
}

#[test]
fn a_blood_mage_ability_landed_over_a_pool_returns_more_than_it_cost() {
    // The feel relationship, rewritten from *thrown perfectly returns more
    // than it cost*: landed on bare floor it returns nothing (the test above),
    // and landed over a pool of its own making it returns more than it cost.
    // The first cast makes the pool; the second, thrown as soon as she is
    // free, is the one measured.
    let cases = [(b::HAEMORRHAGE, keys::MY_BLOOD)];
    for (slot, button) in cases {
        let m = sim::moves::get(Class::BloodMage, slot);
        let mut w = mage();
        in_reach(&mut w);
        wounded(&mut w, 300);
        let pitch = pitch_at(&w, w.players[1].pos);
        // First cast: make the pool. Then wait out the move and its repeat
        // lockout, so the second press is a press she can spend.
        looking(&mut w, 2, button, pitch, 0);
        looking(
            &mut w,
            m.whiff_cost() as u32 + m.repeat_idle() as u32 + 2,
            0,
            pitch,
            0,
        );
        // Hold the dummy where it was, so the second cast lands on the same
        // floor, and re-open the same headroom.
        in_reach(&mut w);
        w.players[1].vel = V3::ZERO;
        w.players[1].action = sim::state::Action::Free;
        wounded(&mut w, 300);
        assert!(
            !pools(&w).is_empty(),
            "{}: the first cast left no pool",
            m.name
        );
        let before = w.players[0].health;
        let paid = w.players[0].cost_of(m.cost);
        looking(&mut w, 2, button, pitch, 0);
        looking(
            &mut w,
            m.whiff_cost() as u32 + t::bloodletter_flight() as u32,
            0,
            pitch,
            0,
        );
        let returned = w.players[0].health - (before - paid);
        assert!(
            returned > paid,
            "{}: landed over its own pool it returned {returned} and cost {paid}, so playing \
             well still loses you the fight",
            m.name
        );
    }
}

#[test]
fn a_pool_is_an_effect_and_the_world_still_fits_its_budget() {
    // The plan's measured criterion: nothing allocates and the snapshot is
    // under its cap. `budget.rs` holds both; this only says which change is
    // being asked about.
    assert!(std::mem::size_of::<World>() <= 20 * 1024);
    assert_eq!(EffectKind::Pool.name(), "essence pool");
}
