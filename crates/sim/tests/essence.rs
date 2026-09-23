//! Essence pools: the other fighter's blood on the floor, and the drink.
//!
//! Every hit the Blood mage lands spills a pool under the target, sized by the
//! damage; pools drain, merge and are capped; and the only heal she has is a
//! move put through one. See `docs/design/blood-mage.md` §"Essence pools" and
//! §"Double duty", and `plans/blood-mage-v1.md` M2.

use sim::class::{ALL_CLASSES, Class};
use sim::effects::{Effect, EffectKind};
use sim::moves::blood as b;
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

/// Stand the dummy inside the scythe's reach.
fn in_reach(w: &mut World) {
    let sweep = sim::moves::get(Class::BloodMage, b::SWEEP);
    w.players[1].pos = w.players[0].pos.add(V3::new(
        sweep.reach.mul(Fx::ratio(1, 2)),
        Fx::ZERO,
        Fx::ZERO,
    ));
}

/// Open some grey on her, so a drink has something to convert.
fn wounded(w: &mut World, grey: i32) {
    w.players[0].health = t::max_health() - grey;
    w.players[0].grey = grey;
}

/// A pitch that points the Bloodletter at the other fighter's middle.
fn pitch_at(w: &World, target: V3) -> i16 {
    let m = sim::moves::get(Class::BloodMage, b::BLOODLETTER);
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let scene = sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
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
    run(&mut w, 2, Input::LEFT, 0);
    run(&mut w, 20, 0, 0);
    let dealt = full - w.players[1].health;
    assert!(dealt > 0, "fixture: the sweep never landed");
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
    // Sized by the damage, less what has drained in the frames since.
    assert!(
        pool.pool_volume() <= dealt && pool.pool_volume() > dealt * 3 / 4,
        "hit for {dealt} and the pool holds {}",
        pool.pool_volume()
    );
}

#[test]
fn a_pools_size_and_life_are_monotone_in_the_damage_that_made_it() {
    let sweep = sim::moves::get(Class::BloodMage, b::SWEEP);
    let spike = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE);
    let life_of = |volume: i32| -> (Fx, u32) {
        let mut w = mage();
        w.effects[0] = Some(Effect::pool(
            0,
            Class::BloodMage,
            b::SWEEP,
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
    let (small_r, small_life) = life_of(sweep.damage);
    let (big_r, big_life) = life_of(spike.damage);
    assert!(
        big_r.raw() > small_r.raw(),
        "a spike's pool is no wider than a sweep's"
    );
    // A spike's pool outlives a sweep's by at least the ratio of their damage:
    // the drain is a rate, so life is volume over rate.
    assert!(
        big_life * sweep.damage as u32 >= small_life * spike.damage as u32,
        "a spike's pool lives {big_life}f and a sweep's {small_life}f, against a \
         damage ratio of {}:{}",
        spike.damage,
        sweep.damage
    );
}

#[test]
fn hits_that_land_on_one_spot_make_one_pool() {
    // Four arms of a Grasp converge on one point and each spills there. One
    // figure, not four -- and the Grasp is the move to ask, because its four
    // hits land on one spot in one frame: a hit an effect delivers drinks
    // only pools older than the effect, so the arms cannot each drink what
    // the arm before spilled, and their four spills gather into one figure.
    // (A sweep repeated on one spot leaves one fresh pool each time rather
    // than a growing one, because each drinks the last one's first.)
    let mut w = mage();
    let grasp = sim::moves::get(Class::BloodMage, b::GRASP);
    // At the reach a tap converges on, so no hold is needed.
    w.players[1].pos = w.players[0]
        .pos
        .add(V3::new(grasp.channel_from, Fx::ZERO, Fx::ZERO));
    let pitch = pitch_at(&w, w.players[1].pos);
    let full = w.players[1].health;
    looking(&mut w, 2, Input::SPECIAL, pitch, 0);
    looking(
        &mut w,
        grasp.whiff_cost() as u32 + t::grasp_flight() as u32,
        0,
        pitch,
        0,
    );
    let dealt = full - w.players[1].health;
    assert!(
        dealt > grasp.damage * 2,
        "fixture: only {dealt} landed of four arms"
    );
    let made = pools(&w);
    assert_eq!(
        made.len(),
        1,
        "four arms on one spot made {} pools",
        made.len()
    );
    assert!(
        made[0].pool_volume() > grasp.damage * 2,
        "the pool did not gather the arms: {} after {dealt} dealt",
        made[0].pool_volume()
    );
}

#[test]
fn a_fifth_pool_merges_into_the_newest() {
    let mut w = mage();
    let far = |i: i32| V3::new(Fx::from_int(-12 + i * 3), Fx::ZERO, Fx::from_int(6));
    for i in 0..t::pool_cap() as i32 {
        let mut pool = Effect::pool(0, Class::BloodMage, b::SWEEP, far(i), 100);
        // Older the lower the index, so the newest is the last one placed.
        pool.age = (t::pool_cap() as i32 - i) as u16 * 10;
        w.effects[i as usize] = Some(pool);
    }
    let newest = w.effects[t::pool_cap() - 1].expect("placed").pool_volume();
    in_reach(&mut w);
    run(&mut w, 2, Input::LEFT, 0);
    run(&mut w, 20, 0, 0);
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
fn her_own_costs_never_pool() {
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-18), Fx::ZERO, Fx::ZERO);
    for button in [
        Input::LEFT,
        Input::RIGHT,
        Input::MIDDLE,
        Input::SPECIAL,
        Input::MECHANIC,
    ] {
        run(&mut w, 2, button, 0);
        run(&mut w, 70, 0, 0);
    }
    assert!(w.players[0].grey > 0, "fixture: nothing cost her anything");
    assert!(
        pools(&w).is_empty(),
        "casting at nothing left blood on the floor"
    );
}

#[test]
fn nobody_else_spills_anybody() {
    for class in ALL_CLASSES {
        if class == Class::BloodMage {
            continue;
        }
        let mut w = World::with_classes([class, Class::BloodMage]);
        w.players[1].pos = w.players[0]
            .pos
            .add(V3::new(Fx::ratio(3, 2), Fx::ZERO, Fx::ZERO));
        run(&mut w, 60, Input::LEFT, 0);
        assert!(
            w.players[1].health < t::max_health(),
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
fn a_move_landed_over_a_pool_drinks_its_share_and_the_pool_is_gone() {
    let mut w = mage();
    in_reach(&mut w);
    wounded(&mut w, 300);
    let at = w.players[1].pos;
    w.effects[0] = Some(Effect::pool(0, Class::BloodMage, b::SWEEP, at, 200));
    let sweep = sim::moves::get(Class::BloodMage, b::SWEEP);
    let before = w.players[0].health;
    let paid = w.players[0].cost_of(sweep.cost);
    run(&mut w, 2, Input::LEFT, 0);
    run(&mut w, sweep.startup as u32 + 4, 0, 0);
    let got = w.players[0].health - (before - paid);
    let expect = sweep.drinks(200);
    assert!(
        got >= expect - 4 && got <= expect,
        "the sweep drank {got} of a pool of 200 at {}%",
        sweep.drink
    );
    // One and done: the pool she drank is gone. What is on the floor now is
    // only what the sweep spilled after it, which is a fresh, smaller figure.
    let dealt = t::max_health() - w.players[1].health;
    let left = pools(&w);
    assert_eq!(
        left.len(),
        1,
        "the pool she drank is still there beside the one she spilled"
    );
    assert!(
        left[0].pool_volume() <= dealt && left[0].pool_volume() > dealt - 4,
        "the drunk pool survived: {} on the floor after a spill of {dealt}",
        left[0].pool_volume()
    );
    // Red plus grey is still the bar, less what the fade took meanwhile.
    let bar = w.players[0].health + w.players[0].grey;
    assert!(
        bar <= t::max_health() && bar >= t::max_health() - 6,
        "the drink made health out of nothing: red + grey is {bar}"
    );
}

#[test]
fn a_drink_is_capped_by_grey_and_the_pool_is_spent_regardless() {
    let mut w = mage();
    in_reach(&mut w);
    let at = w.players[1].pos;
    w.effects[0] = Some(Effect::pool(0, Class::BloodMage, b::SWEEP, at, 200));
    let sweep = sim::moves::get(Class::BloodMage, b::SWEEP);
    // At full health the only grey she has is the cast's own cost, less what
    // faded during the wind-up.
    run(&mut w, 2, Input::LEFT, 0);
    run(&mut w, sweep.startup as u32 + 4, 0, 0);
    assert!(
        w.players[0].health <= t::max_health()
            && w.players[0].health >= t::max_health() - t::max_health() * sweep.cost / 100,
        "she healed past the top of the bar, or not to it: {}",
        w.players[0].health
    );
    assert_eq!(
        w.players[0].grey, 0,
        "there was grey left with a pool to drink"
    );
    // And the pool is spent all the same: what she could not fill is lost
    // with it. Only the Reap's own spill is left on the floor.
    let dealt = t::max_health() - w.players[1].health;
    let left = pools(&w);
    assert_eq!(left.len(), 1);
    assert!(
        left[0].pool_volume() <= dealt,
        "the pool kept what she could not drink: {} on the floor",
        left[0].pool_volume()
    );
}

#[test]
fn a_hit_on_bare_floor_returns_nothing_on_the_frame_it_lands() {
    let mut w = mage();
    in_reach(&mut w);
    wounded(&mut w, 300);
    let sweep = sim::moves::get(Class::BloodMage, b::SWEEP);
    run(&mut w, 2, Input::LEFT, 0);
    let paid = w.players[0].health;
    run(&mut w, sweep.startup as u32 + 4, 0, 0);
    assert!(
        w.players[1].health < t::max_health(),
        "fixture: the sweep missed"
    );
    assert_eq!(w.players[0].health, paid, "a hit on bare floor healed her");
    assert_eq!(pools(&w).len(), 1, "and it left no pool for the next one");
}

#[test]
fn the_blade_drinks_from_pools_it_crosses_on_the_way_home() {
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(8));
    wounded(&mut w, 300);
    let m = sim::moves::get(Class::BloodMage, b::BLOODLETTER);
    // A pool along the throw, a metre and a half out, with nobody in it.
    let along = w.players[0]
        .pos
        .add(V3::new(Fx::ratio(3, 2), Fx::ZERO, Fx::ZERO));
    w.effects[0] = Some(Effect::pool(0, Class::BloodMage, b::SWEEP, along, 100));
    looking(&mut w, 2, Input::MIDDLE, 0, 0);
    let paid = w.players[0].health;
    let flight = t::bloodletter_flight() as u32;
    // The way out drinks nothing.
    run(&mut w, m.startup as u32 + flight / 4, 0, 0);
    assert_eq!(w.players[0].health, paid, "the blade drank on the way out");
    run(&mut w, flight, 0, 0);
    let got = w.players[0].health - paid;
    assert!(
        got > 0,
        "the blade crossed a pool on the way home and drank nothing"
    );
    assert!(
        got <= m.drinks(100),
        "the blade drank {got} from one pool, which is more than its share"
    );
    assert!(
        pools(&w).is_empty(),
        "the pool the blade drank is still there"
    );
}

#[test]
fn a_blood_mage_ability_landed_over_a_pool_returns_more_than_it_cost() {
    // The feel relationship, rewritten from *thrown perfectly returns more
    // than it cost*: landed on bare floor it returns nothing (the test above),
    // and landed over a pool of its own making it returns more than it cost.
    // The first cast makes the pool; the second, thrown as soon as she is
    // free, is the one measured.
    let cases = [(b::SWEEP, Input::LEFT), (b::BLOODLETTER, Input::MIDDLE)];
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
    assert!(std::mem::size_of::<World>() <= 4096);
    assert_eq!(EffectKind::Pool.name(), "essence pool");
}

#[test]
fn a_grasp_landed_on_somebody_standing_in_a_pool_drinks_its_share_once() {
    // Four arms, one drink: the share of the pool that was there before the
    // Grasp was thrown, and nothing of what its own arms spill.
    let mut w = mage();
    wounded(&mut w, 400);
    let grasp = sim::moves::get(Class::BloodMage, b::GRASP);
    w.players[1].pos = w.players[0]
        .pos
        .add(V3::new(grasp.channel_from, Fx::ZERO, Fx::ZERO));
    let at = w.players[1].pos;
    w.effects[0] = Some(Effect::pool(0, Class::BloodMage, b::SWEEP, at, 200));
    let pitch = pitch_at(&w, w.players[1].pos);
    let before = w.players[0].health;
    let paid = w.players[0].cost_of(grasp.cost);
    let full = w.players[1].health;
    looking(&mut w, 2, Input::SPECIAL, pitch, 0);
    looking(
        &mut w,
        grasp.whiff_cost() as u32 + t::grasp_flight() as u32,
        0,
        pitch,
        0,
    );
    let dealt = full - w.players[1].health;
    assert!(dealt > grasp.damage * 2, "fixture: only {dealt} landed");
    let got = w.players[0].health - (before - paid);
    let expect = grasp.drinks(200);
    assert!(
        got >= expect - 8 && got <= expect,
        "the Grasp drank {got} of a pool of 200 at {}%, against a share of {expect}",
        grasp.drink
    );
    // And what its arms spilled is still on the floor, in one figure.
    let left = pools(&w);
    assert_eq!(left.len(), 1, "{} pools after a Grasp over one", left.len());
    assert!(
        left[0].pool_volume() > grasp.damage * 2,
        "the arms' own spill was drunk back: {} on the floor after {dealt} dealt",
        left[0].pool_volume()
    );
}
