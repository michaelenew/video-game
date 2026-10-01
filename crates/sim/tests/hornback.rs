//! The Hornback herd (`docs/design/creatures/hornback.md`): the rules of the
//! fight about rocks, pinned as relationships rather than values. Each test
//! is one line of the document's §10 list, or a rule it rests on.
//!
//! The machinery underneath -- critters, the pack brain, hazards that stand
//! as solids -- is pinned in `tests/critters.rs` and `tests/hazards.rs`; what
//! is here is the herd's own: alarm, the charge and its stop, the boulders
//! spent, wariness, the stampede held to its lane, the close game, the guard
//! and its horns, the kick, the ride and the end.

use sim::critter::{CritterField, MAX_CRITTERS, is};
use sim::fixed::Fx;
use sim::hazard;
use sim::pack::{self, mood};
use sim::species::SpeciesId;
use sim::species::hornback::{self as h, Knob};
use sim::state::{Action, MAX_PLAYERS};
use sim::{Class, Input, V3, World};

fn hunt(class: Class) -> World {
    let mut w = World::hunt_of([class; MAX_PLAYERS], SpeciesId::HORNBACK);
    // One hunter: the second fighter is out of it, as the harness has it.
    w.players[1].health = 0;
    w
}

fn idle() -> [Input; MAX_PLAYERS] {
    [Input::default(); MAX_PLAYERS]
}

fn keep_up(w: &mut World) {
    w.players[0].health = w.players[0].full_health();
}

fn at(x: i32, z: i32) -> V3 {
    V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z))
}

fn knob(k: Knob) -> i32 {
    h::knob(k)
}

fn knob_fx(k: Knob) -> Fx {
    h::knob_fx(k)
}

fn bull(w: &World) -> usize {
    (0..MAX_CRITTERS)
        .find(|i| w.critters[*i].kind == h::BULL)
        .expect("a bull")
}

fn cows(w: &World) -> Vec<usize> {
    (0..MAX_CRITTERS)
        .filter(|i| w.critters[*i].kind == h::COW && w.critters[*i].alive())
        .collect()
}

/// Step until the boulders are down and the herd has had a frame.
fn settle(w: &mut World) {
    for _ in 0..2 {
        w.advance(idle());
    }
}

/// Alarm the herd now, with its grace spent: a fight already under way.
fn alarmed(w: &mut World) {
    settle(w);
    let mut p = w.pack.unwrap();
    p.mood = mood::HUNTING;
    w.pack = Some(p);
    w.advance(idle());
    let mut p = w.pack.unwrap();
    p.grace = 0;
    w.pack = Some(p);
}

/// Put the bull somewhere, facing a point, standing.
fn stand_bull(w: &mut World, pos: V3, face: V3) {
    let b = bull(w);
    let c = &mut w.critters[b];
    c.pos = pos;
    c.vel = V3::ZERO;
    c.yaw = pack::yaw_of(face.sub(pos));
    c.state = is::PROWL;
    c.timer = 0;
}

/// Make the bull throw a move now.
fn throw(w: &mut World, act: u8) {
    let b = bull(w);
    let sp = w.critters.sp();
    let c = &mut w.critters[b];
    c.state = is::STARTUP;
    c.act = act;
    c.timer = sp.attack(act).startup.max(1);
    c.set(sim::critter::flag::HIT_USED, false);
}

/// Move every cow out of the way: far off at home, not in the fight.
fn cows_away(w: &mut World) {
    for (k, i) in cows(w).into_iter().enumerate() {
        w.critters[i].pos = at(18 - 2 * (k as i32 % 3), -16 + 2 * (k as i32 / 3));
    }
}

/// The boulders standing, as solids.
fn standing_boulders(w: &World) -> Vec<(usize, u8)> {
    hazard::all(&w.lore)
        .filter(|(_, hz)| hz.index() == h::BOULDER || hz.index() == h::CRACKED)
        .map(|(i, hz)| (i, hz.index()))
        .collect()
}

fn boulder_at(w: &World, slot: usize) -> V3 {
    let hz = hazard::get(&w.lore, slot);
    let c = hz.centre();
    V3::new(c.x, Fx::ZERO, c.z)
}

// ---------------------------------------------------------------------------
// Alarm
// ---------------------------------------------------------------------------

#[test]
fn the_herd_is_not_hostile_until_somebody_comes_close_or_hits_one() {
    // Somebody keeping their distance is watched and not fought.
    let mut w = hunt(Class::Champion);
    settle(&mut w);
    w.players[0].pos = V3::new(Fx::from_int(-20), Fx::ZERO, Fx::from_int(-2));
    for _ in 0..600 {
        keep_up(&mut w);
        w.advance(idle());
    }
    assert_eq!(w.pack.unwrap().mood, mood::CALM, "nobody came near");
    assert!(
        w.critters.iter().all(|c| !c.attacking()),
        "a calm herd throws nothing"
    );

    // Walking within the alarm radius of any animal starts it.
    let c = w.critters[cows(&w)[0]].pos;
    let near = pack_alarm();
    w.players[0].pos = V3::new(c.x.sub(near.sub(Fx::ONE)), Fx::ZERO, c.z);
    for _ in 0..30 {
        w.advance(idle());
    }
    assert_eq!(w.pack.unwrap().mood, mood::HUNTING, "close enough to alarm");

    // So does a hit on any one of them, from anywhere.
    let mut w = hunt(Class::Champion);
    settle(&mut w);
    let i = cows(&w)[3];
    pack::hurt(
        &mut w.pack,
        &mut w.critters,
        i,
        10,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
        Fx::ZERO,
    );
    w.advance(idle());
    assert_eq!(w.pack.unwrap().mood, mood::HUNTING, "a hit alarms the herd");
    // And the alarm buys a grace: interposing, nothing thrown.
    for _ in 0..(knob_grace() - 10) {
        keep_up(&mut w);
        w.advance(idle());
        assert!(
            w.critters.iter().all(|c| !c.attacking()),
            "nothing is thrown in the grace after the alarm"
        );
    }
}

fn pack_alarm() -> Fx {
    sim::species::lookup(SpeciesId::HORNBACK)
        .unwrap()
        .pack_fx(pack::PackKnob::Alarm)
}

fn knob_grace() -> i32 {
    sim::species::lookup(SpeciesId::HORNBACK)
        .unwrap()
        .pack_raw(pack::PackKnob::Grace)
}

#[test]
fn a_herd_left_alone_long_enough_calms_down() {
    let mut w = hunt(Class::Champion);
    alarmed(&mut w);
    w.players[0].pos = V3::new(Fx::from_int(-22), Fx::ZERO, Fx::from_int(18));
    cows_away(&mut w);
    stand_bull(&mut w, at(16, -12), at(0, 0));
    let far = knob_fx(Knob::CalmRadius);
    let mut calm_at = None;
    for f in 0..(knob(Knob::CalmFrames) + 400) {
        // Holding the herd where it is: the test is about the clock.
        cows_away(&mut w);
        stand_bull(&mut w, at(16, -12), at(0, 0));
        keep_up(&mut w);
        w.advance(idle());
        let near = w
            .critters
            .iter()
            .filter(|c| c.alive())
            .any(|c| c.pos.sub(w.players[0].pos).flat_len().raw() <= far.raw());
        assert!(!near, "the hunter is kept far away");
        if w.pack.unwrap().mood == mood::CALM && calm_at.is_none() {
            calm_at = Some(f);
        }
    }
    let calm_at = calm_at.expect("a herd left alone calms down");
    assert!(
        calm_at + 2 >= knob(Knob::CalmFrames),
        "not before CalmFrames ({calm_at})"
    );
}

// ---------------------------------------------------------------------------
// The charge
// ---------------------------------------------------------------------------

/// The bull 11 m from the hunter, facing them, the hunter standing still.
fn charge_scene(class: Class, hunter: V3, bull_at: V3) -> World {
    let mut w = hunt(class);
    alarmed(&mut w);
    cows_away(&mut w);
    w.players[0].pos = hunter;
    stand_bull(&mut w, bull_at, hunter);
    w
}

#[test]
fn the_charge_follows_you_until_the_second_paw_and_not_after() {
    let sp = sim::species::lookup(SpeciesId::HORNBACK).unwrap();
    let lock = sim::critter::stat(sp, h::BULL, CritterField::Lock) as u16;
    let startup = sp.attack(h::CHARGE).startup;
    assert!(
        lock > 0 && lock < startup,
        "it tracks for a while and then locks"
    );

    // Sidestep before the lock: the bull turns to follow.
    let mut w = charge_scene(Class::Champion, at(-10, 4), at(1, 4));
    throw(&mut w, h::CHARGE);
    w.advance(idle());
    let b = bull(&w);
    let yaw0 = w.critters[b].yaw;
    w.players[0].pos = at(-10, 0);
    for _ in 0..(startup - lock - 4) {
        w.advance(idle());
    }
    let followed = w.critters[b].yaw;
    assert_ne!(yaw0, followed, "before the second paw it turns to follow");

    // Sidestep after the lock: it does not.
    let mut w = charge_scene(Class::Champion, at(-10, 4), at(1, 4));
    throw(&mut w, h::CHARGE);
    for _ in 0..(startup - lock + 2) {
        w.advance(idle());
    }
    let b = bull(&w);
    let locked = w.critters[b].yaw;
    w.players[0].pos = at(-10, -2);
    for _ in 0..(lock - 3) {
        w.advance(idle());
        assert_eq!(
            w.critters[b].yaw, locked,
            "after the second paw it cannot turn"
        );
    }
}

#[test]
fn a_charge_that_meets_a_solid_stuns_for_the_whole_window() {
    let mut w = hunt(Class::Champion);
    settle(&mut w);
    let (slot, _) = standing_boulders(&w)[0];
    let rock = boulder_at(&w, slot);
    // The hunter three metres in front of the rock, the bull 10 m beyond.
    let hunter = rock.add(V3::new(Fx::from_int(4), Fx::ZERO, Fx::ZERO));
    let mut w = charge_scene(Class::Champion, hunter, hunter.add(at(10, 0)));
    throw(&mut w, h::CHARGE);
    // Walk out of the lane after the lock, toward the next rock: here, aside.
    let mut stunned_for = 0;
    let mut longest = 0;
    let mut frames = 0;
    for f in 0..400 {
        keep_up(&mut w);
        let inp = if f > 20 && f < 50 {
            [Input::aimed(Input::D, 0), Input::default()]
        } else {
            idle()
        };
        w.advance(inp);
        let b = bull(&w);
        if h::stunned(&w.critters[b]) {
            stunned_for += 1;
            longest = longest.max(stunned_for);
        } else {
            stunned_for = 0;
        }
        frames = f;
    }
    let _ = frames;
    assert_eq!(
        longest,
        knob(Knob::StunFrames),
        "a clean charge into a rock stuns for the whole window"
    );
    // And the rock is cracked.
    let hz = hazard::get(&w.lore, slot);
    assert_eq!(hz.index(), h::CRACKED, "the first charge cracks it");
}

#[test]
fn a_charge_pulls_up_short_of_the_arena_edge() {
    // In open grass near the east edge, facing it: the run ends a margin
    // short of the bounds, with no stun.
    let mut w = charge_scene(Class::Champion, at(22, 0), at(10, 0));
    w.players[0].pos = at(-10, 10);
    let b = bull(&w);
    w.critters[b].yaw = pack::yaw_of(V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
    throw(&mut w, h::CHARGE);
    let mut most = Fx::ZERO;
    let mut ever_stunned = false;
    for _ in 0..200 {
        keep_up(&mut w);
        w.advance(idle());
        let c = &w.critters[b];
        most = most.max(c.pos.x);
        ever_stunned |= h::stunned(c);
        // Hold its facing: this test is about the edge, not the target.
        if c.state == is::STARTUP {
            w.critters[b].yaw = pack::yaw_of(V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
        }
    }
    let sp = sim::species::lookup(SpeciesId::HORNBACK).unwrap();
    let half = sim::critter::stat_fx(sp, h::BULL, CritterField::Length).mul(Fx::ratio(1, 2));
    let edge = w.terrain().bounds.hi_x;
    assert!(!ever_stunned, "the edge is not a solid it is stunned by");
    assert!(
        most.add(half).raw() <= edge.sub(knob_fx(Knob::EdgeMargin)).add(Fx::ONE).raw(),
        "its nose stops short of the edge: {} + {} against {}",
        most.to_f32_for_render(),
        half.to_f32_for_render(),
        edge.to_f32_for_render()
    );
}

/// Charge the bull from `from` straight at `rock`, with nobody in the way,
/// and run until it is stunned or not.
fn charge_into(w: &mut World, rock: V3, from: V3) -> bool {
    w.players[0].pos = at(-20, 18);
    stand_bull(w, from, rock);
    throw(w, h::CHARGE);
    let b = bull(w);
    for _ in 0..120 {
        keep_up(w);
        if w.critters[b].state == is::STARTUP {
            w.critters[b].yaw = pack::yaw_of(rock.sub(w.critters[b].pos));
        }
        w.advance(idle());
        if h::stunned(&w.critters[b]) {
            return true;
        }
    }
    false
}

#[test]
fn a_boulder_takes_two_charges_and_the_second_breaks_it() {
    let mut w = hunt(Class::Champion);
    alarmed(&mut w);
    cows_away(&mut w);
    let (slot, _) = standing_boulders(&w)[1];
    let rock = boulder_at(&w, slot);
    let from = rock.add(at(9, 0));
    assert!(charge_into(&mut w, rock, from), "the first charge stuns");
    assert_eq!(hazard::get(&w.lore, slot).index(), h::CRACKED);
    assert!(
        w.terrain()
            .raised()
            .iter()
            .any(|s| { s.min.x.raw() < rock.x.raw() && s.max.x.raw() > rock.x.raw() }),
        "a cracked boulder still stands"
    );
    assert!(charge_into(&mut w, rock, from), "and the second");
    assert_eq!(
        hazard::get(&w.lore, slot).index(),
        h::RUBBLE,
        "the second charge shatters it"
    );
    assert!(
        !w.terrain().raised().iter().any(|s| {
            s.min.x.raw() < rock.x.raw()
                && s.max.x.raw() > rock.x.raw()
                && s.min.z.raw() < rock.z.raw()
                && s.max.z.raw() > rock.z.raw()
        }),
        "rubble is not a solid"
    );
    // A third charge down the same line runs on through it.
    assert!(
        !charge_into(&mut w, rock, from),
        "rubble does not stop a charge"
    );
}

#[test]
fn a_stone_and_a_planted_shield_stop_the_bull_like_a_boulder() {
    // A stone: raised in open grass, the bull charging it.
    let mut w = hunt(Class::Elementalist);
    alarmed(&mut w);
    cows_away(&mut w);
    let spot = at(-4, -2);
    let mut stone = sim::class::Structure::raised(spot);
    stone.age = 200;
    sim::stones::raise(&mut w.players[0], stone);
    assert!(
        charge_into(&mut w, spot, spot.add(at(9, 0))),
        "a stone stops it"
    );
    let left = sim::stones::gather(&w.players).iter().flatten().count();
    assert_eq!(left, 0, "and collapses doing it");

    // A planted shield: the Bulwark's wall stops it and takes the charge's
    // weight.
    let mut w = hunt(Class::Bulwark);
    alarmed(&mut w);
    cows_away(&mut w);
    let wall = at(-4, -2);
    w.players[0].mechanic = sim::class::Mechanic::Shield(sim::class::Shield::Planted {
        pos: wall,
        weight: Fx::from_int(100),
    });
    let before = sim::bulwark::weight(&w.players[0]);
    assert!(
        charge_into(&mut w, wall, wall.add(at(9, 0))),
        "a planted shield stops it"
    );
    let after = sim::bulwark::weight(&w.players[0]);
    assert!(
        after.raw() > before.raw(),
        "the shield is loaded by the charge"
    );
}

#[test]
fn the_bull_does_not_charge_the_solid_it_last_hit_while_wary() {
    let mut w = hunt(Class::Champion);
    alarmed(&mut w);
    cows_away(&mut w);
    let (slot, _) = standing_boulders(&w)[0];
    let rock = boulder_at(&w, slot);
    assert!(charge_into(&mut w, rock, rock.add(at(9, 0))));
    // Out of the stun and its lockout, the hunter in front of the same rock:
    // nothing. In front of another: a charge.
    let b = bull(&w);
    let stun = knob(Knob::StunFrames) + knob(Knob::ChargeLockout) + 5;
    let hunter = rock.add(at(-3, 0));
    let mut charged_same = false;
    for f in 0..(stun + 300) {
        keep_up(&mut w);
        cows_away(&mut w);
        w.players[0].pos = hunter;
        if f > stun {
            // Held where a charge at the hunter passes over the same rock.
            stand_bull_keep_state(&mut w, rock.add(at(9, 0)), hunter);
        }
        w.advance(idle());
        let c = w.critters[b];
        if c.attacking() && c.act == h::CHARGE {
            charged_same = true;
        }
    }
    assert!(!charged_same, "it does not charge the same rock while wary");
    // The same bull, a hunter in front of nothing it remembers: charged.
    let elsewhere = rock.add(at(9, -10));
    let mut charged = false;
    for _ in 0..300 {
        keep_up(&mut w);
        cows_away(&mut w);
        w.players[0].pos = elsewhere;
        stand_bull_keep_state(&mut w, rock.add(at(9, 0)), elsewhere);
        w.advance(idle());
        let c = w.critters[b];
        charged |= c.attacking() && c.act == h::CHARGE;
    }
    assert!(charged, "a lane that misses the rock it learned is charged");
}

/// Hold the bull's position and facing without touching what it is doing.
fn stand_bull_keep_state(w: &mut World, pos: V3, face: V3) {
    let b = bull(w);
    let c = &mut w.critters[b];
    if c.state == is::PROWL {
        c.pos = pos;
        c.vel = V3::ZERO;
        c.yaw = pack::yaw_of(face.sub(pos));
    }
}

// ---------------------------------------------------------------------------
// The close game
// ---------------------------------------------------------------------------

#[test]
fn the_trample_lands_no_sooner_than_a_reaction_after_you_can_move() {
    let mut w = hunt(Class::Champion);
    alarmed(&mut w);
    cows_away(&mut w);
    let hunter = at(-6, 0);
    w.players[0].pos = hunter;
    stand_bull(&mut w, hunter.add(at(3, 0)), hunter);
    // On the floor for a long while: the trample must wait it out.
    let down = 70u16;
    w.players[0].action = Action::Stagger { left: down };
    throw(&mut w, h::TRAMPLE);
    let b = bull(&w);
    let mut free_at = None;
    let mut lands_at = None;
    for f in 0..200u32 {
        w.advance(idle());
        if free_at.is_none() && !matches!(w.players[0].action, Action::Stagger { .. }) {
            free_at = Some(f);
        }
        let c = w.critters[b];
        if lands_at.is_none() && c.state == is::ACTIVE && c.act == h::TRAMPLE {
            lands_at = Some(f);
        }
    }
    let (free, lands) = (free_at.unwrap(), lands_at.expect("it tramples"));
    assert!(
        lands >= free + knob(Knob::TrampleAfter) as u32 - 1,
        "it lands {} frames after you could move",
        lands as i32 - free as i32
    );
}

#[test]
fn any_hit_in_the_shoulders_lean_interrupts_it() {
    let mut w = hunt(Class::Champion);
    alarmed(&mut w);
    cows_away(&mut w);
    let hunter = at(-6, 2);
    w.players[0].pos = hunter;
    stand_bull(&mut w, at(-6, 0), at(0, 0));
    throw(&mut w, h::SHOULDER);
    w.advance(idle());
    // A light hit, far below any strain threshold.
    let b = bull(&w);
    pack::hurt(
        &mut w.pack,
        &mut w.critters,
        b,
        5,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
        Fx::ZERO,
    );
    let c = w.critters[b];
    assert_eq!(c.state, is::FLINCH, "a hit in the lean flinches it");
    // While the same hit in a hook's windup does not.
    throw(&mut w, h::HOOK);
    w.advance(idle());
    let mut p = w.pack.unwrap();
    p.memo[h::memo::STRAIN] = 0;
    w.pack = Some(p);
    pack::hurt(
        &mut w.pack,
        &mut w.critters,
        b,
        5,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
        Fx::ZERO,
    );
    assert_eq!(
        w.critters[b].state,
        is::STARTUP,
        "a hook's windup takes a poke"
    );
}

#[test]
fn cows_do_no_damage_outside_a_stampede() {
    // Somebody standing in the middle of a grazing herd, walking about in it,
    // is pushed about and never hurt by a touch -- only a kick from a cow
    // they stood behind, which is a move with a tell.
    let mut w = hunt(Class::Champion);
    alarmed(&mut w);
    let b = bull(&w);
    w.critters[b].pos = at(-20, 18);
    let home = w.pack.unwrap().home;
    w.players[0].pos = home;
    let mut kicked = 0;
    for f in 0..600 {
        let before = w.players[0].health;
        let kicking = w
            .critters
            .iter()
            .any(|c| c.kind == h::COW && c.state == is::ACTIVE && c.act == h::KICK);
        // Keep the bull out of it, and the hunter turning about.
        w.critters[b].pos = at(-20, 18);
        w.critters[b].state = is::PROWL;
        let aim = ((f * 300) % 65536) as u16;
        w.advance([Input::aimed(Input::W, aim), Input::default()]);
        let lost = before - w.players[0].health;
        if lost > 0 {
            assert!(
                kicking
                    || w.critters
                        .iter()
                        .any(|c| c.kind == h::COW && c.state == is::ACTIVE && c.act == h::KICK),
                "a cow hurt somebody without a kick"
            );
            kicked += 1;
        }
        keep_up(&mut w);
    }
    let _ = kicked;
}
