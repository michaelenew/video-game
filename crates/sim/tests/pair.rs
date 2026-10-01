//! The Pair: the rules `docs/design/creatures/the-pair.md` pins, as sentences.
//!
//! Most of these set up one cat (the other slot emptied, so the pair brain
//! plays it alone) in front of one fighter, start a move, and play the answer
//! the design gives -- and the wrong one -- with the fighter's buttons.

use sim::fixed::Fx;
use sim::monster::{Doing, Monster};
use sim::species::SpeciesId;
use sim::species::pair::{self, Knob, fight};
use sim::state::{MAX_PLAYERS, Player};
use sim::{Class, Input, V3, World};

/// A hunt against the Pair, the second fighter out of it.
fn hunt() -> World {
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::PAIR);
    w.players[1].health = 0;
    w
}

/// Where the duels are fought: a clear strip of the Den, south of the
/// platforms and between the stones.
fn base() -> V3 {
    V3::new(Fx::from_int(-4), Fx::ZERO, Fx::from_int(-9))
}

/// A point `x` metres east and `z` south of [`base`].
fn at(x: i32, z: i32) -> V3 {
    base().add(V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z)))
}

/// One cat at [`base`] facing +x, a fighter `at` metres down +x facing it,
/// and the other cat gone. Both awake.
fn duel(dist: i32) -> World {
    let mut w = hunt();
    w.monsters[1] = None;
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = base();
    m.yaw = Fx::ZERO;
    m.brain.grace = 0;
    m.brain.think_left = 600;
    w.players[0].pos = at(dist, 0);
    w.players[0].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    m.brain.seen = w.players[0].pos;
    // Settle: a few frames of nothing so the pair brain has written its
    // first lore and the cat has glanced.
    for _ in 0..10 {
        w.advance([idle(); MAX_PLAYERS]);
        hold_still(&mut w);
    }
    w
}

/// Keep the cat where the test put it, thinking about nothing.
fn hold_still(w: &mut World) {
    if let Some(m) = w.monsters[0].as_mut() {
        m.brain.think_left = m.brain.think_left.max(60);
        if m.doing.free() {
            m.pos = base();
            m.yaw = Fx::ZERO;
            m.yaw_rate = Fx::ZERO;
            m.speed = Fx::ZERO;
        }
    }
}

/// Looking at the cat (down -x), no keys.
fn idle() -> Input {
    Input::aimed(0, 0x8000)
}

fn cat(w: &World) -> Monster {
    w.monsters[0].unwrap()
}

fn me(w: &World) -> Player {
    w.players[0]
}

/// Start a move on the cat now, as its brain would.
fn throw(w: &mut World, kind: u8) {
    let m = w.monsters[0].as_mut().unwrap();
    m.doing = Doing::Startup {
        kind,
        left: pair::SPECIES.attack(kind).startup,
    };
    m.hit_used = false;
    m.brain.think_left = 600;
    if pair::MOVES[kind as usize].lobbed {
        m.lob(kind);
    }
    // The ambush is aimed past its target, as its commit aims it.
    if kind == pair::AMBUSH {
        let past = m.brain.seen.sub(m.pos);
        let far = sim::math::wide_flat_len(past).add(Knob::AmbushPast.fx());
        let at = m.pos.add(sim::math::wide_normalized(past).scale(far));
        m.aim_at(at);
    }
}

/// Play the move out: `press` from frame `at` of it, for `hold` frames.
/// Returns the fighter's health lost.
fn play(w: &mut World, kind: u8, at: u32, press: Input, hold: u32) -> i32 {
    let before = me(w).health;
    let total = pair::SPECIES.attack(kind).total() as u32;
    for f in 0..total + 4 {
        let input = if f >= at && f < at + hold {
            press
        } else {
            idle()
        };
        w.advance([input, Input::default()]);
        if let Some(m) = w.monsters[0].as_mut() {
            m.brain.think_left = m.brain.think_left.max(60);
        }
    }
    before - me(w).health
}

#[test]
fn two_cats_are_hunted_together() {
    let w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::PAIR);
    assert!(w.monsters[0].is_some_and(|m| m.species == SpeciesId::PAIR));
    assert!(w.monsters[1].is_some_and(|m| m.species == SpeciesId::PAIR));
}

#[test]
fn the_pounce_is_dodged_toward_not_away() {
    // The fighter sees the flick on frame thirteen, fifteen frames late: it
    // dodges on frame twenty-eight, after the leap has left the ground.
    let at = 13 + 15;
    let toward = Input::aimed(Input::SHIFT | Input::W, 0x8000);
    let away = Input::aimed(Input::SHIFT | Input::S, 0x8000);

    let mut w = duel(7);
    throw(&mut w, pair::POUNCE);
    let lost = play(&mut w, pair::POUNCE, at, toward, 2);
    assert_eq!(lost, 0, "a dodge toward it, under the arc, is the answer");

    let mut w = duel(7);
    throw(&mut w, pair::POUNCE);
    let lost = play(&mut w, pair::POUNCE, at, away, 2);
    assert!(lost > 0, "a dodge away is caught by the skid");

    let mut w = duel(7);
    throw(&mut w, pair::POUNCE);
    let lost = play(&mut w, pair::POUNCE, at, idle(), 2);
    assert!(lost > 0, "standing still is the pounce landing");
}

#[test]
fn the_ambush_is_stepped_out_of_sideways() {
    // The fighter seven metres on with their back to it, walking sideways
    // from the moment a person could have seen the lane.
    let at = 15;
    let mut w = duel(7);
    throw(&mut w, pair::AMBUSH);
    let lost = play(&mut w, pair::AMBUSH, at, Input::aimed(Input::D, 0), 60);
    assert_eq!(lost, 0, "walking at right angles clears the lane");

    let mut w = duel(7);
    throw(&mut w, pair::AMBUSH);
    let lost = play(&mut w, pair::AMBUSH, at, Input::aimed(Input::W, 0), 60);
    assert!(lost > 0, "running along it does not");
}

#[test]
fn the_tail_trip_is_jumped() {
    let beside = |w: &mut World| {
        w.players[0].pos = at(-2, 2);
        let m = w.monsters[0].as_mut().unwrap();
        m.brain.seen = w.players[0].pos;
    };
    let mut w = duel(3);
    beside(&mut w);
    throw(&mut w, pair::TRIP);
    let at = pair::SPECIES.attack(pair::TRIP).startup as u32 - 6;
    let lost = play(&mut w, pair::TRIP, at, Input::aimed(Input::SPACE, 0), 30);
    assert_eq!(lost, 0, "a jump clears the low sweep");

    let mut w = duel(3);
    beside(&mut w);
    throw(&mut w, pair::TRIP);
    let lost = play(&mut w, pair::TRIP, 0, idle(), 0);
    assert!(lost > 0, "standing beside its hips is tripped");
}

#[test]
fn the_feint_and_the_pounce_differ_by_frame_ten() {
    let pose_at = |kind: u8, frame: u16| {
        let mut m = Monster::new(SpeciesId::PAIR);
        let a = pair::SPECIES.attack(kind);
        m.doing = Doing::Startup {
            kind,
            left: a.startup - frame,
        };
        m.pose()
    };
    // The tail's lift, all four bones of it.
    let tail = |p: &sim::beast::Pose| {
        [
            pair::bones::TAIL1,
            pair::bones::TAIL2,
            pair::bones::TAIL3,
            pair::bones::TAIL4,
        ]
        .iter()
        .fold(Fx::ZERO, |sum, b| sum.add(p.bone[*b].x))
    };
    for f in 0..=10 {
        let (p, q) = (pose_at(pair::POUNCE, f), pose_at(pair::FEINT, f));
        let d = tail(&p).sub(tail(&q)).abs();
        assert!(
            d.raw() < Fx::ratio(1, 100).raw(),
            "frame {f}: the coil is the same coil until the flick ({})",
            d.to_f32_for_render()
        );
    }
    let (p, q) = (pose_at(pair::POUNCE, 16), pose_at(pair::FEINT, 16));
    let d = tail(&p).sub(tail(&q)).abs();
    assert!(
        d.raw() > Fx::ratio(1, 12).raw(),
        "by frame sixteen the pounce's tail is up and over and the feint's is down ({})",
        d.to_f32_for_render()
    );
}

#[test]
fn the_rake_hold_is_drawn_when_the_second_hit_commits() {
    let mut holds = Vec::new();
    for seed in 0..12u32 {
        let mut w = duel(3);
        let m = w.monsters[0].as_mut().unwrap();
        m.brain.rng = (0x1234_5679 ^ seed.wrapping_mul(0x9E37_79B9)) | 1;
        throw(&mut w, pair::RAKE);
        // The swipe out: the hold is not drawn until it is over.
        let first = pair::SPECIES.attack(pair::RAKE);
        let rng_at_commit = cat(&w).brain.rng;
        for _ in 0..(first.startup + first.active) {
            w.advance([Input::aimed(Input::S, 0x8000), Input::default()]);
            assert_eq!(
                cat(&w).brain.rng,
                rng_at_commit,
                "nothing drawn during the first"
            );
        }
        let mut hold = 0;
        for _ in 0..80 {
            w.advance([Input::aimed(Input::S, 0x8000), Input::default()]);
            match cat(&w).doing {
                Doing::Startup {
                    kind: pair::COCK, ..
                } => hold += 1,
                Doing::Startup {
                    kind: pair::RAKE2, ..
                } => break,
                _ => {}
            }
        }
        assert!(
            (Knob::HoldMin.raw()..=Knob::HoldMax.raw() + 1).contains(&hold),
            "hold {hold}"
        );
        holds.push(hold);
    }
    holds.sort();
    holds.dedup();
    assert!(holds.len() > 3, "the hold varies: {holds:?}");
}

#[test]
fn nobody_stands_on_a_cat() {
    let w = duel(5);
    let m = cat(&w);
    // Put down on its back from above, a fighter is pushed off the side,
    // never held up.
    for x in [-1, 0, 1] {
        let above = base().add(V3::new(Fx::ratio(x * 4, 10), Fx::ratio(17, 10), Fx::ZERO));
        let c = m.resolve(
            above,
            sim::tuning::body_radius(),
            sim::tuning::body_height(),
        );
        assert!(c.landed.is_none(), "nothing on a cat is a floor");
        assert!(m.surface_under(above, sim::tuning::body_radius()).is_none());
    }
    // A fighter jumping onto it from the side comes down on the floor.
    let mut w = duel(3);
    w.players[0].pos = base().add(V3::new(Fx::ratio(15, 10), Fx::ZERO, Fx::ZERO));
    for f in 0..90 {
        let bits = if f < 30 {
            Input::SPACE | Input::W
        } else {
            Input::W
        };
        w.advance([Input::aimed(bits, 0x8000), Input::default()]);
        hold_still(&mut w);
        assert!(!me(&w).aboard(), "nobody mounts a cat");
    }
    assert!(
        me(&w).pos.y.raw() < Fx::ratio(1, 10).raw(),
        "and they end on the floor"
    );
}

#[test]
fn a_cat_that_cannot_see_you_does_not_sample_you() {
    // The Den's standing stone at (9.5, 9.5), between them.
    let mut w = hunt();
    w.monsters[1] = None;
    let at = V3::new(Fx::from_int(7), Fx::ZERO, Fx::from_int(7));
    let pin = |w: &mut World| {
        let m = w.monsters[0].as_mut().unwrap();
        m.brain.think_left = 9999;
        m.pos = at;
        m.speed = Fx::ZERO;
    };
    let m = w.monsters[0].as_mut().unwrap();
    m.brain.grace = 0;
    m.brain.seen = V3::ZERO;
    w.players[0].pos = V3::new(Fx::from_int(12), Fx::ZERO, Fx::from_int(12));
    for _ in 0..30 {
        w.advance([idle(), Input::default()]);
        pin(&mut w);
    }
    assert_eq!(cat(&w).brain.seen, V3::ZERO, "the stone hides you");
    assert!(fight::unseen(&cat(&w)) > 20);
    // Step out from behind it, and it sees you at its next glance.
    w.players[0].pos = V3::new(Fx::from_int(12), Fx::ZERO, Fx::from_int(4));
    for _ in 0..10 {
        w.advance([idle(), Input::default()]);
        pin(&mut w);
    }
    assert!(
        sim::math::wide_flat_dist(cat(&w).brain.seen, me(&w).pos).raw() < Fx::ONE.raw(),
        "in sight, it is sampled"
    );
}

#[test]
fn a_scarred_cat_does_not_see_you_on_its_blind_side() {
    let mut w = duel(5);
    let m = w.monsters[0].as_mut().unwrap();
    fight::scar_eye(m, 1);
    m.brain.seen = V3::ZERO;
    // Square off its right side: blind.
    w.players[0].pos = at(0, 5);
    for _ in 0..20 {
        w.advance([idle(), Input::default()]);
        hold_still(&mut w);
    }
    assert!(
        sim::math::wide_flat_dist(cat(&w).brain.seen, me(&w).pos).raw() > Fx::from_int(2).raw(),
        "on its blind side it does not sample you"
    );
    // Its good side: seen.
    w.players[0].pos = at(0, -5);
    for _ in 0..20 {
        w.advance([idle(), Input::default()]);
        hold_still(&mut w);
    }
    assert!(
        sim::math::wide_flat_dist(cat(&w).brain.seen, me(&w).pos).raw() < Fx::ONE.raw(),
        "on its good side it does"
    );
}

#[test]
fn three_hundred_and_fifty_into_a_head_scars_the_eye_on_that_side() {
    let mut w = duel(5);
    let mut m = cat(&w);
    let mut dealt = 0;
    while dealt < Knob::ScarDamage.raw() {
        dealt += m.take_hit(pair::HEAD, 40);
        m.doing = Doing::Prowl;
    }
    w.monsters[0] = Some(m);
    // The fighter is on its right.
    w.players[0].pos = at(1, 2);
    w.advance([idle(), Input::default()]);
    assert_eq!(fight::scar(&cat(&w)), 1);
}

/// A fixed stream of buttons and looks for both fighters, as the Ridgeback's
/// pin drives its hunt.
fn script(frames: u32, seed: u64) -> Vec<[Input; MAX_PLAYERS]> {
    let mut rng = seed | 1;
    let mut next = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    (0..frames)
        .map(|_| {
            [
                Input::aimed((next() & 0x1ff) as u16, next() as u16),
                Input::default(),
            ]
        })
        .collect()
}

/// A long hunt against both cats, the fighter kept alive so the cats keep
/// working. `each` sees every frame, before and after.
fn scripted(seed: u64, frames: u32, mut each: impl FnMut(&World, &World)) {
    let mut w = hunt();
    let full = w.players[0].health;
    for input in script(frames, seed) {
        let before = w.clone();
        w.advance(input);
        each(&before, &w);
        keep(&mut w, full);
    }
}

/// The scripted fighter kept alive and inside the Den: a random walk is
/// over a 1.5 m wall in a minute, and a cat cannot follow it there.
fn keep(w: &mut World, full: i32) {
    let p = &mut w.players[0];
    p.health = full;
    let edge = Fx::from_int(12);
    p.pos.x = p.pos.x.clamp(edge.neg(), edge);
    p.pos.z = p.pos.z.clamp(edge.neg(), edge);
}

/// Did cat `s` land a hit that hurts on this frame?
fn landed(before: &World, after: &World, s: usize) -> Option<u8> {
    let (Some(was), Some(now)) = (before.monsters[s], after.monsters[s]) else {
        return None;
    };
    let kind = now.doing.attacking()?;
    (!was.hit_used && now.hit_used && pair::SPECIES.attack(kind).damage > 0).then_some(kind)
}

#[test]
fn the_brain_decides_the_same_whatever_buttons_were_pressed() {
    // Two hunts whose fighter stands in the same place and looks anywhere:
    // the look, the facing and the camera differ every frame, the quarry
    // never does. The cats must not be able to tell.
    let mut a = hunt();
    let mut b = hunt();
    let looks = script(1800, 0x5eed);
    let others = script(1800, 0xfeed);
    let mut differed = 0;
    for f in 0..1800 {
        let (la, lb) = (looks[f][0], others[f][0]);
        a.advance([Input::aimed(0, la.aim), Input::default()]);
        b.advance([Input::aimed(0, lb.aim), Input::default()]);
        if a.players[0].facing != b.players[0].facing {
            differed += 1;
        }
        assert_eq!(
            a.players[0].pos, b.players[0].pos,
            "frame {f}: the quarry is the same"
        );
        assert_eq!(a.monsters, b.monsters, "frame {f}: so are the cats");
    }
    assert!(differed > 1000, "the looks really did differ ({differed})");
}

#[test]
fn every_hit_lands_at_least_fifteen_frames_after_the_glance_that_chose_it() {
    // Below a reaction, only the swat -- positional, and short enough that
    // what it reaches was already in reach (Swat in §2).
    for kind in 0..pair::MOVES.len() as u8 {
        let a = pair::SPECIES.attack(kind);
        if a.damage > 0 && a.startup < 15 {
            assert_eq!(
                kind,
                pair::SWAT,
                "{} tells under a reaction",
                pair::MOVES[kind as usize].name
            );
            assert!(
                a.ideal_range.add(a.range_span).raw() < a.hit_x.add(a.hit_radius).raw(),
                "and the swat is only thrown at what its paw already reaches"
            );
        }
    }
    // Every other hit: at least fifteen frames after its commit, and after
    // the last frame anything it aims at moved.
    let mut hits = 0;
    for seed in 1..=4u64 {
        let mut commit = [0u32; 2];
        let mut aimed = [0u32; 2];
        scripted(seed * 0x9E37_79B9, 3600, |before, after| {
            for s in 0..2 {
                let (Some(was), Some(now)) = (before.monsters[s], after.monsters[s]) else {
                    continue;
                };
                let started = matches!(now.doing, Doing::Startup { .. })
                    && (was.doing.attacking() != now.doing.attacking()
                        || !matches!(was.doing, Doing::Startup { .. }));
                if started {
                    commit[s] = after.frame;
                    aimed[s] = after.frame;
                }
                if now.doing.attacking().is_some() && now.aimed_at() != was.aimed_at() {
                    aimed[s] = after.frame;
                }
                if let Some(kind) = landed(before, after, s) {
                    if kind == pair::SWAT {
                        continue;
                    }
                    hits += 1;
                    let name = pair::MOVES[kind as usize].name;
                    assert!(
                        after.frame - commit[s] >= 15,
                        "{name}: {} after its commit",
                        after.frame - commit[s]
                    );
                    assert!(
                        after.frame - aimed[s] >= 15,
                        "{name}: {} after its aim last moved",
                        after.frame - aimed[s]
                    );
                }
            }
        });
    }
    assert!(hits > 10, "enough hits to mean something ({hits})");
}

#[test]
fn the_two_never_land_within_the_gap_except_the_twin_pounce() {
    let gap = Knob::StaggerGap.raw() as u32;
    let mut windows = 0;
    for seed in 1..=4u64 {
        // The last frame each cat was live with something that hurts.
        let mut live: [Option<u32>; 2] = [None; 2];
        scripted(seed * 0x2545_F491, 3600, |_, after| {
            for s in 0..2 {
                let Some(m) = after.monsters[s] else { continue };
                let Doing::Active { kind, .. } = m.doing else {
                    continue;
                };
                if kind == pair::TWIN || pair::SPECIES.attack(kind).damage <= 0 {
                    continue;
                }
                if live[s] != Some(after.frame - 1) {
                    windows += 1;
                }
                if let Some(o) = live[1 - s] {
                    assert!(
                        after.frame - o > gap,
                        "frame {}: {} live {} frames after the other",
                        after.frame,
                        pair::MOVES[kind as usize].name,
                        after.frame - o
                    );
                }
                live[s] = Some(after.frame);
            }
        });
    }
    assert!(windows > 20, "enough moves to mean something ({windows})");
}

/// Both cats either side of the fighter, coiled for the twin pounce at them.
fn twin_coiled() -> World {
    let mut w = duel(7);
    let mut other = cat(&w);
    other.pos = at(14, 0);
    other.yaw = Fx::ratio(1, 2);
    w.monsters[1] = Some(other);
    let mid = me(&w).pos;
    let ground = w.terrain();
    for s in 0..2 {
        let m = w.monsters[s].as_mut().unwrap();
        m.brain.seen = mid;
        m.doing = Doing::Startup {
            kind: pair::TWIN,
            left: pair::SPECIES.attack(pair::TWIN).startup,
        };
        m.hit_used = false;
        m.lob(pair::TWIN);
        fight::mark_at(m, mid, &ground, &[None; sim::stones::MAX_STONES]);
    }
    w
}

/// Play the twin pounce out with `press` from frame `at`.
fn play_twin(w: &mut World, at: u32, press: Input) -> (i32, bool) {
    let before = me(w).health;
    let total = pair::SPECIES.attack(pair::TWIN).total() as u32;
    let mut crashed = false;
    for f in 0..total + 4 {
        let input = if f >= at && f < at + 2 {
            press
        } else {
            Input::aimed(0, 0)
        };
        w.advance([input, Input::default()]);
        for m in w.monsters.iter_mut().flatten() {
            m.brain.think_left = m.brain.think_left.max(60);
        }
        crashed |= w
            .monsters
            .iter()
            .flatten()
            .all(|m| matches!(m.doing, Doing::Toppled { .. }));
    }
    (before - me(w).health, crashed)
}

#[test]
fn a_late_dodge_crashes_the_twin_pounce_and_an_early_one_does_not() {
    // Sideways, across the line between them (the fighter faces +z here).
    let side = Input::aimed(Input::SHIFT | Input::D, 0x4000);
    let leave = Knob::TwinLeave.raw() as u32;

    // Late: after they have left the ground, the mark cannot follow.
    let mut w = twin_coiled();
    let (lost, crashed) = play_twin(&mut w, leave + 2, side);
    assert_eq!(lost, 0, "a late dodge is out from under both");
    assert!(crashed, "and they land on each other");

    // Early: the one mark is still following, and finds you.
    let mut w = twin_coiled();
    let (lost, crashed) = play_twin(&mut w, 2, side);
    assert!(lost > 0, "an early dodge is followed");
    assert!(!crashed, "and nobody crashes");
}

#[test]
fn hurting_the_wounded_cat_brings_the_other_between() {
    let mut w = hunt();
    let full = w.monsters[0].unwrap().health;
    // The fighter beside the wounded one; its mate off to the side.
    w.players[0].pos = at(3, 0);
    let wounded = at(0, 0);
    let guard = at(4, -8);
    for (s, pos) in [(0, wounded), (1, guard)] {
        let m = w.monsters[s].as_mut().unwrap();
        m.pos = pos;
        m.brain.grace = 0;
        m.brain.seen = at(3, 0);
    }
    w.monsters[0].as_mut().unwrap().health = full / 5;
    let mut between = false;
    for _ in 0..240 {
        w.advance([Input::aimed(0, 0x8000), Input::default()]);
        w.players[0].pos = at(3, 0);
        let g = w.monsters[1].unwrap();
        let w0 = w.monsters[0].unwrap();
        let mid = sim::math::lerp3(w0.pos, me(&w).pos, sim::math::half(Fx::ONE));
        let near_line = sim::math::wide_flat_dist(g.pos, mid).raw() < Fx::from_int(2).raw();
        if g.doing.attacking() == Some(pair::INTERPOSE) || near_line {
            between = true;
            break;
        }
    }
    assert!(between, "the healthy one comes between you and its mate");
}

#[test]
fn the_survivor_enrages_and_no_tell_goes_below_fifteen_frames() {
    let mut tells = 0;
    for seed in 1..=3u64 {
        let mut w = hunt();
        w.monsters[1].as_mut().unwrap().health = 0;
        let full = w.players[0].health;
        let mut enraged = false;
        for input in script(3600, seed * 0x51ED) {
            let before = w.clone();
            w.advance(input);
            keep(&mut w, full);
            let (Some(was), Some(now)) = (before.monsters[0], w.monsters[0]) else {
                continue;
            };
            enraged |= fight::enraged(&now);
            if let Doing::Startup { kind, left } = now.doing {
                let fresh = was.doing.attacking() != Some(kind)
                    || !matches!(was.doing, Doing::Startup { .. });
                if fresh && pair::SPECIES.attack(kind).damage > 0 {
                    tells += 1;
                    let tell = left + 1;
                    if kind == pair::SWAT {
                        assert!(
                            tell >= pair::SPECIES.attack(kind).startup,
                            "the swat stays itself"
                        );
                    } else {
                        assert!(
                            tell >= 15,
                            "{} told in {tell}",
                            pair::MOVES[kind as usize].name
                        );
                    }
                }
            }
        }
        assert!(enraged, "the survivor enrages");
    }
    assert!(tells > 10, "enough tells to mean something ({tells})");
}

#[test]
fn the_ridgeback_is_bit_identical_with_two_slots() {
    // The second slot is there in every hunt and empty in the Ridgeback's:
    // nothing in the pair brain runs, and nothing fills it. The hash itself
    // is pinned by `ridgeback_pin.rs`.
    let mut w = World::hunt([Class::Champion; MAX_PLAYERS]);
    for input in script(1200, 0x1d) {
        w.advance(input);
        assert!(w.monsters[1].is_none());
    }
    assert!(w.monsters[0].is_some_and(|m| m.species == SpeciesId::RIDGEBACK));
}
