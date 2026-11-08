//! The bleed: what the Cast's bite opens (2026-11-04, exploration 0038 -- the
//! Haemorrhage's bolt before it), and a bleed that lays a trail of pools. See
//! `docs/design/kits/blood-mage.md`. The head itself -- the leash, the wrench,
//! the drag home -- is `tests/world_bleeds.rs`; here it is taken off once it
//! has bitten, so what is measured is the bleed alone.

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

fn bolts(w: &World) -> usize {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::ScytheHead)
        .count()
}

/// Is her head in him?
fn bitten(w: &World) -> bool {
    w.effects.iter().flatten().any(|e| {
        e.kind == EffectKind::ScytheHead && e.head_state() == sim::effects::Head::Fighter(1)
    })
}

/// Take her head off, so the bleed is all there is.
fn unhook(w: &mut World) {
    for slot in w.effects.iter_mut() {
        if slot.is_some_and(|e| e.kind == EffectKind::ScytheHead) {
            *slot = None;
        }
    }
}

/// A pitch that points the bolt at the other fighter's middle.
fn pitch_at(w: &World, target: V3) -> i16 {
    let m = sim::moves::get(Class::BloodMage, b::CAST);
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
    let middle = sim::aim::standing_middle(target, sim::tuning::body_height());
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

/// Cast the head along `pitch`, with the other fighter pressing `b`: the
/// scythe held past the tap's window, then let go.
fn cast_bolt(w: &mut World, pitch: i16, b: u16) {
    for _ in 0..t::hold_from() as u32 + 2 {
        looking(w, 1, keys::SCYTHE, pitch, b);
    }
    for _ in 0..60 {
        if bolts(w) > 0 {
            break;
        }
        looking(w, 1, 0, pitch, b);
    }
    assert!(bolts(w) > 0, "fixture: the scythe held threw no head");
}

/// Cast at the dummy standing `away` metres ahead, run until the head has
/// bitten him, and take it off. Gives back what the bite dealt.
fn bolt_the_dummy(w: &mut World, away: i32) -> i32 {
    w.players[1].pos = V3::new(Fx::from_int(away), Fx::ZERO, Fx::ZERO);
    let full = w.players[1].health;
    let pitch = pitch_at(w, w.players[1].pos);
    cast_bolt(w, pitch, 0);
    for _ in 0..30 {
        if bitten(w) {
            break;
        }
        looking(w, 1, 0, pitch, 0);
    }
    unhook(w);
    full - w.players[1].health
}

#[test]
fn the_cast_bites_and_opens_a_bleed() {
    let m = sim::moves::get(Class::BloodMage, b::CAST);
    let mut w = mage();
    let dealt = bolt_the_dummy(&mut w, 4);
    // Free since 2026-11-06: the scythe costs her nothing.
    assert_eq!(m.cost, 0, "the Cast costs her");
    assert_eq!(w.players[0].health, t::max_health(), "the Cast cost her");
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
    // The cut spilled, like every hit of hers.
    assert!(
        pools(&w).iter().any(|p| p.covers(w.players[1].pos)),
        "the bite's cut left no pool under him"
    );
}

#[test]
fn the_bleed_ticks_for_its_whole_length_and_then_stops() {
    let mut w = mage();
    let bolt = bolt_the_dummy(&mut w, 4);
    let full = w.players[1].full_health() - bolt;
    // Opened when the bolt landed, part way through its flight.
    let opened = w.players[1].bleeding;
    assert!(
        opened <= t::bleed_lasts() && opened > t::bleed_lasts() - 30,
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
    let bled = w.players[1].full_health() - bolt - w.players[1].health;
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
    // within what one tick's eruption sets off of each other -- its own size
    // and `tuning::erupt_chain` past its edge. Since the blaster the ticks are
    // fewer and heavier (2026-10-12), so the stains are a stride or two apart
    // and the chain reaches across the gap rather than the stains touching.
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
    let tick = Fx::from_int(t::bleed_damage()).sqrt();
    let sets_off = t::erupt_radius().mul(tick).add(t::erupt_chain());
    for pair in sorted.windows(2) {
        let gap = pair[1].sub(pair[0]);
        assert!(
            gap.raw() <= sets_off.raw(),
            "two pools of the trail are {} m apart, past the {} m one sets off",
            gap.to_f32_for_render(),
            sets_off.to_f32_for_render()
        );
    }
    // And the newest is under their feet.
    let feet = w.players[1].pos;
    assert!(
        trail
            .iter()
            .any(|p| p.pos.sub(feet).flat_len().raw() < sets_off.raw()),
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
            quarry: &w.monsters,
            critters: &sim::critter::Critters::NONE,
            arena: &w.terrain(),
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
    // The same frames with no spike, for what the bleed alone takes.
    let mut alone = w.clone();
    looking(&mut w, 2, keys::SPIKES, pitch, Input::A);
    looking(&mut alone, 2, 0, pitch, Input::A);
    // Link by link since 2026-10-13: each waits `tuning::chain_delay`, and
    // since 2026-10-19 an eruption reaches the next only as it spreads.
    let link = t::chain_delay() as u32 + t::erupt_spread() as u32;
    let mut eruptions = 0;
    let mut launched = false;
    for _ in 0..m.startup as u32 + 4 * link + 2 {
        looking(&mut w, 1, 0, pitch, Input::A);
        launched |= !w.players[1].grounded || w.players[1].action.stunned();
        eruptions += w
            .effects
            .iter()
            .flatten()
            .filter(|e| e.kind == EffectKind::BlackSpike && e.erupted() && e.age == 1)
            .count();
    }
    // Two at least since 2026-11-04: a bleed tick's small pool rises in
    // `tuning::spike_rise_small`, half a fighter's, so the trail laid while
    // it rises is shorter than when every spike took sixteen frames.
    assert!(
        eruptions >= 2,
        "a spike on the trail set off only {eruptions} eruptions"
    );
    // The chain reached them: an eruption's blow, not just the bleed's ticks.
    // Since 2026-10-19 an eruption is worth its stain, and the bleed's are
    // small, so it is weighed against the same frames without the spike.
    looking(
        &mut alone,
        m.startup as u32 + 4 * link + 2,
        0,
        pitch,
        Input::A,
    );
    let took = health - w.players[1].health;
    let bled = health - alone.players[1].health;
    assert!(
        took > bled,
        "the chain ran but never reached the bleeding fighter: they took {took}, the bleed alone {bled}"
    );
    assert!(
        launched,
        "they were not launched by the eruption that reached them"
    );
}

#[test]
fn a_guarded_bolt_is_spent_and_opens_nothing() {
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(4), Fx::ZERO, Fx::from_int(0));
    let pitch = pitch_at(&w, w.players[1].pos);
    // The Bulwark guards on his right click the whole time.
    cast_bolt(&mut w, pitch, Input::RIGHT);
    looking(&mut w, 30, 0, pitch, Input::RIGHT);
    assert_eq!(
        w.players[1].health,
        w.players[1].full_health(),
        "a guard let the bolt through"
    );
    assert_eq!(w.players[1].bleeding, 0, "a guarded bolt opened a bleed");
    assert!(!bitten(&w), "the head bit through a guard");
}

#[test]
fn the_bleed_does_not_kill() {
    let mut w = mage();
    let _ = bolt_the_dummy(&mut w, 4);
    w.players[1].health = 3;
    run(&mut w, t::bleed_lasts() as u32, 0, 0);
    assert!(w.players[1].health >= 0);
}

#[test]
fn a_bite_landed_on_somebody_standing_in_a_pool_drinks_its_share() {
    let m = sim::moves::get(Class::BloodMage, b::CAST);
    let mut w = mage();
    w.players[0].health = t::max_health() - 300;
    w.players[0].grey = 300;
    let at = V3::new(Fx::from_int(4), Fx::ZERO, Fx::ZERO);
    let mut pool = Effect::pool(0, Class::BloodMage, b::CAST, at, 200);
    // Settled: a stain is drunk only once it has (`tuning::stain_settles`).
    pool.age = t::stain_settles();
    w.effects[0] = Some(pool);
    let before = w.players[0].health;
    let paid = w.players[0].cost_of(m.cost);
    bolt_the_dummy(&mut w, 4);
    let got = w.players[0].health - (before - paid);
    // What the cast cost lands where she aimed it, which may be on this
    // stain, so the drink is of it and perhaps the cost (2026-10-13).
    let (least, most) = (m.drinks(200), m.drinks(200 + paid));
    assert!(
        got >= least - 4 && got <= most,
        "the bite drank {got} of a pool of 200 at {}%",
        m.drink
    );
    // One and done: the pool it drank is gone.
    assert!(
        pools(&w).iter().all(|p| p.pool_volume() < 150),
        "the pool it drank is still there"
    );
}
