//! The Haemorrhage: right click, a bolt that opens a bleed, and a bleed that
//! lays a trail of pools. See `docs/design/kits/blood-mage.md`.

use sim::class::Class;
use sim::effects::{Effect, EffectKind};
use sim::moves::blood as b;
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

fn bolts(w: &World) -> usize {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::Haemorrhage)
        .count()
}

/// A pitch that points the bolt at the other fighter's middle.
fn pitch_at(w: &World, target: V3) -> i16 {
    let m = sim::moves::get(Class::BloodMage, b::HAEMORRHAGE);
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let scene = sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
    };
    let middle = sim::aim::standing_middle(target);
    let miss = |pitch: i16| {
        let look = Input::looking_at(0, 0, pitch);
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

/// Throw the bolt at the dummy standing `away` metres ahead, and run until it
/// has landed or flown its whole reach. Gives back what the bolt itself dealt.
fn bolt_the_dummy(w: &mut World, away: i32) -> i32 {
    let m = sim::moves::get(Class::BloodMage, b::HAEMORRHAGE);
    w.players[1].pos = V3::new(Fx::from_int(away), Fx::ZERO, Fx::ZERO);
    let full = w.players[1].health;
    let pitch = pitch_at(w, w.players[1].pos);
    looking(w, 2, Input::RIGHT, pitch, 0);
    looking(
        w,
        m.startup as u32 + t::haemorrhage_flight() as u32 + 2,
        0,
        pitch,
        0,
    );
    full - w.players[1].health
}

#[test]
fn right_click_throws_a_bolt_that_cuts_and_opens_a_bleed() {
    let m = sim::moves::get(Class::BloodMage, b::HAEMORRHAGE);
    let mut w = mage();
    let paid = w.players[0].cost_of(m.cost);
    let dealt = bolt_the_dummy(&mut w, 4);
    assert!(paid > 0, "the cast is free");
    assert!(
        w.players[0].health <= t::max_health() - paid,
        "the cast cost nothing: {} of {}",
        w.players[0].health,
        t::max_health()
    );
    assert_eq!(
        dealt, m.damage,
        "the bolt dealt {dealt} against a row of {}",
        m.damage
    );
    assert!(
        w.players[1].bleeding > 0,
        "the bolt landed and opened no bleed"
    );
    assert_eq!(w.players[1].bled_by, 0, "the bleed is not hers");
    assert_eq!(bolts(&w), 0, "the bolt flew on after it landed");
    // The cut spilled, like every hit of hers.
    assert_eq!(pools(&w).len(), 1, "the bolt's cut left no pool");
}

#[test]
fn the_bleed_ticks_for_its_whole_length_and_then_stops() {
    let mut w = mage();
    let bolt = bolt_the_dummy(&mut w, 4);
    let full = t::max_health() - bolt;
    // Opened when the bolt landed, part way through its flight.
    let opened = w.players[1].bleeding;
    assert!(
        opened <= t::bleed_lasts() && opened > t::bleed_lasts() - t::haemorrhage_flight() - 4,
        "the bleed opened at {opened} of {}",
        t::bleed_lasts()
    );
    let mut ticks: i32 = 0;
    let mut was = w.players[1].health;
    for _ in 0..t::bleed_lasts() as u32 {
        run(&mut w, 1, 0, 0);
        if w.players[1].health < was {
            ticks += 1;
            assert_eq!(
                was - w.players[1].health,
                t::bleed_damage(),
                "a tick took the wrong amount"
            );
        }
        was = w.players[1].health;
    }
    assert_eq!(w.players[1].bleeding, 0, "the bleed did not stop");
    let expect = (t::bleed_lasts() / t::bleed_tick()) as i32;
    assert_eq!(
        ticks,
        expect,
        "the bleed ticked {ticks} times over {} frames",
        t::bleed_lasts()
    );
    assert_eq!(full - w.players[1].health, ticks * t::bleed_damage());
    // And nothing more once it has run out.
    run(&mut w, 60, 0, 0);
    assert_eq!(
        full - w.players[1].health,
        ticks * t::bleed_damage(),
        "the bleed kept taking"
    );
}

#[test]
fn a_bleeding_fighter_standing_still_pools_where_they_stand() {
    // Every tick spills under the victim; standing still, the ticks merge
    // into the one pool, which grows with what they have bled.
    let mut w = mage();
    let bolt = bolt_the_dummy(&mut w, 4);
    run(&mut w, t::bleed_lasts() as u32, 0, 0);
    let bled = t::max_health() - bolt - w.players[1].health;
    let left = pools(&w);
    assert_eq!(
        left.len(),
        1,
        "a fighter standing still left {} pools",
        left.len()
    );
    let at = w.players[1].pos;
    assert!(left[0].covers(at), "the pool is not under them");
    // What was bled is in it, less what drained while it stood, and the
    // bolt's own cut before it.
    assert!(
        left[0].pool_volume() > bled / 2,
        "bled {bled} and the pool under them holds {}",
        left[0].pool_volume()
    );
}

/// Bolt the dummy and clear the cut's own pool, so what is on the floor
/// afterwards is the bleed's trail and nothing else.
fn bleeding(w: &mut World) {
    bolt_the_dummy(w, 4);
    for slot in w.effects.iter_mut() {
        if slot.is_some_and(|e| e.is_a_pool()) {
            *slot = None;
        }
    }
    assert!(w.players[1].bleeding > 0, "fixture: the bolt missed");
}

#[test]
fn a_bleeding_fighter_walking_lays_a_trail_of_pools_a_stride_apart() {
    // The point of the spell: the victim's own feet make the pools, and a
    // spike on any of them runs the trail. So the trail's pools must land
    // inside a bare spike's eruption of each other. A tick's pool is small
    // and drains away in under a second, so the live trail is the last few
    // strides rather than the whole walk -- which is the part a chain has to
    // cross to reach them anyway.
    let mut w = mage();
    bleeding(&mut w);
    // Walk sideways, so the dummy neither closes on her nor leaves the arena.
    run(&mut w, 90, 0, Input::A);
    let trail = pools(&w);
    assert!(
        trail.len() >= 3,
        "a walking fighter has only {} pools behind them",
        trail.len()
    );
    let mut sorted: Vec<Fx> = trail.iter().map(|p| p.pos.z).collect();
    sorted.sort_by_key(|z| z.raw());
    let spike = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE);
    for pair in sorted.windows(2) {
        let gap = pair[1].sub(pair[0]);
        assert!(
            gap.raw() <= spike.radius.raw(),
            "two pools of the trail are {} m apart, past a bare spike's {} m",
            gap.to_f32_for_render(),
            spike.radius.to_f32_for_render()
        );
    }
    // And the newest is under their feet.
    let feet = w.players[1].pos;
    assert!(
        trail
            .iter()
            .any(|p| p.pos.sub(feet).flat_len().raw() < spike.radius.raw()),
        "no pool of the trail is near the bleeding fighter"
    );
}

#[test]
fn a_spike_on_the_trail_chains_to_the_bleeding_fighter() {
    // The marker at work: put the spike on the pool nearest them and, by the
    // time it comes up, they have walked on and laid two more -- the chain
    // runs along those and catches them at the end of it.
    let mut w = mage();
    bleeding(&mut w);
    run(&mut w, 60, 0, Input::A);
    let feet = w.players[1].pos;
    let newest = pools(&w)
        .into_iter()
        .min_by_key(|p| p.pos.sub(feet).flat_len().raw())
        .expect("a trail");
    w.players[0].health = t::max_health() - 300;
    w.players[0].grey = 300;
    let m = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE);
    // Stand off to the side of the trail and put the crosshair on that pool.
    w.players[0].pos = V3::new(newest.pos.x.sub(Fx::from_int(4)), Fx::ZERO, newest.pos.z);
    let pitch = {
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
                sim::aim::grounded_path(0, look, m.reach, &scene)
                    .to
                    .sub(newest.pos)
                    .len()
                    .raw()
            })
            .expect("the scan is not empty")
    };
    let health = w.players[1].health;
    looking(&mut w, 2, Input::MECHANIC, pitch, Input::A);
    looking(&mut w, m.startup as u32 + 2, 0, pitch, Input::A);
    let eruptions = w
        .effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::BlackSpike && e.erupted())
        .count();
    assert!(
        eruptions >= 3,
        "a spike on the trail set off only {eruptions} eruptions"
    );
    // The chain reached them: an eruption's blow, not just the bleed's ticks.
    let took = health - w.players[1].health;
    let ticks = (m.startup as i32 + 4) / t::bleed_tick() as i32 + 1;
    assert!(
        took > ticks * t::bleed_damage(),
        "the chain ran but never reached the bleeding fighter: they took {took}"
    );
    assert!(
        !w.players[1].grounded || w.players[1].action.stunned(),
        "they were not launched by the eruption that reached them"
    );
}

#[test]
fn the_bolt_is_the_easier_thing_to_land() {
    // Wider than the blade, and it does not need leading the way a spike
    // does: it is the one thing in the kit meant to connect.
    assert!(
        t::haemorrhage_radius().raw() > t::bloodletter_radius().raw(),
        "the bolt is no wider than the blade"
    );
    let m = sim::moves::get(Class::BloodMage, b::HAEMORRHAGE);
    assert_eq!(
        m.drink, 0,
        "the Haemorrhage drinks: it is the setup, not the payoff"
    );
    assert!(m.cost > 0);
}

#[test]
fn a_guarded_bolt_is_spent_and_opens_nothing() {
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(4), Fx::ZERO, Fx::from_int(0));
    let m = sim::moves::get(Class::BloodMage, b::HAEMORRHAGE);
    let pitch = pitch_at(&w, w.players[1].pos);
    looking(&mut w, 2, Input::RIGHT, pitch, Input::RIGHT);
    looking(
        &mut w,
        m.startup as u32 + t::haemorrhage_flight() as u32 + 2,
        0,
        pitch,
        Input::RIGHT,
    );
    assert_eq!(
        w.players[1].health,
        t::max_health(),
        "a guard let the bolt through"
    );
    assert_eq!(w.players[1].bleeding, 0, "a guarded bolt opened a bleed");
    assert_eq!(bolts(&w), 0, "the bolt flew on through a guard");
}

#[test]
fn the_bleed_does_not_kill() {
    let mut w = mage();
    let _ = bolt_the_dummy(&mut w, 4);
    w.players[1].health = 3;
    run(&mut w, t::bleed_lasts() as u32, 0, 0);
    assert!(w.players[1].health >= 0);
}
