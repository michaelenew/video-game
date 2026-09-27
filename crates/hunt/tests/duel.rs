//! What the sparring bot promises, held.
//!
//! None of these say it is *good*. That is a question for somebody playing
//! against it; `cargo run -p hunt --bin duel` prints the numbers that inform
//! it. These say it keeps the rules that make it fair and make it worth
//! playing twice: it never sees the present, it is not the same every time,
//! and it stays in the fight.

use hunt::duel::{Plan, spar, spar_traced};
use hunt::{Duelist, Level};
use sim::class::ALL_CLASSES;
use sim::state::{MAX_PLAYERS, Phase};
use sim::{Class, Input, World};

const MINUTE: u32 = 60 * 60;

/// **It never sees the present.** Whatever it acts on is at least its
/// quickest reaction old, every frame of a fight, at every level.
#[test]
fn it_sees_late() {
    for level in Level::ALL {
        let floor = level.skill().react_min as u32;
        let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
        let mut bot = Duelist::new(1, level, 7);
        let mut saw_anything = false;
        for _ in 0..MINUTE {
            bot.watch(&w);
            let two = bot.act(&w);
            if let Some(seen) = bot.seeing() {
                saw_anything = true;
                assert!(
                    w.frame - seen >= floor,
                    "{} bot saw frame {seen} on frame {}, under its {floor}-frame floor",
                    level.name(),
                    w.frame
                );
            }
            // Something to look at: player one walks about and swings.
            let one = if w.frame % 90 < 40 {
                Input::new(Input::W | Input::D)
            } else if w.frame % 90 < 43 {
                Input::new(Input::LEFT)
            } else {
                Input::default()
            };
            w.advance([one, two]);
        }
        assert!(saw_anything, "{} bot never saw anything", level.name());
    }
}

/// The same seed is the same fight, so a harness run means something.
#[test]
fn one_seed_is_one_fight() {
    let classes = [Class::Champion, Class::Elementalist];
    let a = spar(classes, [Level::Normal; MAX_PLAYERS], MINUTE, 99);
    let b = spar(classes, [Level::Normal; MAX_PLAYERS], MINUTE, 99);
    assert_eq!(a, b);
    let c = spar(classes, [Level::Normal; MAX_PLAYERS], MINUTE, 100);
    assert_ne!(a, c, "a different seed played exactly the same minute");
}

/// **It does not do one thing.** Over a fight it tries most of its plans, and
/// none of them takes up most of its time.
#[test]
fn it_mixes_it_up() {
    for class in ALL_CLASSES {
        let mut frames = [[0u32; 9]; MAX_PLAYERS];
        spar_traced(
            [class, class],
            [Level::Normal; MAX_PLAYERS],
            3 * MINUTE,
            3,
            |_, bots| {
                for (i, bot) in bots.iter().enumerate() {
                    frames[i][plan_index(bot.plan)] += 1;
                }
            },
        );
        for (i, f) in frames.iter().enumerate() {
            let total: u32 = f.iter().sum();
            let tried = f.iter().filter(|n| **n > 0).count();
            let most = *f.iter().max().unwrap();
            assert!(
                tried >= 6,
                "{} bot {i} only tried {tried} plans: {f:?}",
                class.name()
            );
            assert!(
                most * 2 < total,
                "{} bot {i} spent more than half its time on one plan: {f:?}",
                class.name()
            );
        }
    }
}

fn plan_index(p: Plan) -> usize {
    match p {
        Plan::Press => 0,
        Plan::Footsies => 1,
        Plan::Circle => 2,
        Plan::Retreat => 3,
        Plan::Zone => 4,
        Plan::DashIn => 5,
        Plan::JumpIn => 6,
        Plan::Bait => 7,
        Plan::Wait => 8,
    }
}

/// **It fights.** Every class, against its own mirror, throws, lands and
/// finishes a round inside three minutes -- and stays inside the walls, which
/// are lower than a jump.
#[test]
fn every_class_fights() {
    for class in ALL_CLASSES {
        let mut outside = 0u32;
        let bout = spar_traced(
            [class, class],
            [Level::Normal; MAX_PLAYERS],
            3 * MINUTE,
            11,
            |w, _| {
                if matches!(w.phase, Phase::Fighting) {
                    outside += w.players.iter().filter(|p| !inside(p)).count() as u32;
                }
            },
        );
        for i in 0..MAX_PLAYERS {
            assert!(bout.thrown[i] >= 40, "{}: {bout:?}", class.name());
            assert!(bout.landed[i] >= 10, "{}: {bout:?}", class.name());
        }
        assert!(
            bout.rounds[0] + bout.rounds[1] >= 1,
            "{}: nobody won a round in three minutes: {bout:?}",
            class.name()
        );
        assert!(
            outside < 10 * 60,
            "{}: {outside} fighter-frames outside the walls",
            class.name()
        );
    }
}

fn inside(p: &sim::state::Player) -> bool {
    let half = sim::arena::ARENA_HALF;
    p.pos.x.abs().raw() <= half.raw() && p.pos.z.abs().raw() <= half.raw()
}

/// **The levels mean something.** Across every pairing, a hard bot takes a
/// good deal more rounds off an easy one than it gives up -- and not all of
/// them, because an easy bot that never wins is not somebody to practise on.
#[test]
fn harder_is_better() {
    let (mut hard, mut easy) = (0, 0);
    for (n, a) in ALL_CLASSES.into_iter().enumerate() {
        for b in ALL_CLASSES {
            let bout = spar([a, b], [Level::Hard, Level::Easy], 3 * MINUTE, n as u32);
            hard += bout.rounds[0] as u32;
            easy += bout.rounds[1] as u32;
        }
    }
    assert!(
        hard * 2 > easy * 3 && easy > 0,
        "hard took {hard} rounds and easy took {easy}"
    );
}

/// **It plays its class.** Every class uses the thing that makes it that
/// class, over and over, against its own mirror: the Rush, the shield, the
/// shadow, the stones, the pools, the bars.
#[test]
fn every_class_uses_its_mechanic() {
    for class in ALL_CLASSES {
        let bout = spar([class, class], [Level::Normal; MAX_PLAYERS], 3 * MINUTE, 5);
        for i in 0..MAX_PLAYERS {
            assert!(
                bout.mechanic[i] >= 10,
                "{} bot {i} used its mechanic {} times in three minutes: {bout:?}",
                class.name(),
                bout.mechanic[i]
            );
        }
    }
}

/// **The Dual mage keeps her bars level.** Outside the band she burns her
/// own health, so a bot that throws whatever it likes kills itself slowly.
/// Most of the fight is inside it, and she gets high enough to use what the
/// bars unlock.
#[test]
fn the_dual_mage_keeps_her_balance() {
    let band = sim::Fx::from_int(sim::tuning::meter_band());
    let (mut inside, mut total, mut best) = (0u32, 0u32, sim::Fx::ZERO);
    spar_traced(
        [Class::DualMage, Class::Champion],
        [Level::Normal; MAX_PLAYERS],
        3 * MINUTE,
        8,
        |w, _| {
            let me = &w.players[0];
            if !matches!(w.phase, Phase::Fighting) || sim::dual::ascending(me) {
                return;
            }
            total += 1;
            if sim::dual::gap(me).raw() <= band.raw() {
                inside += 1;
            }
            best = best.max(sim::dual::lower(me));
        },
    );
    assert!(
        inside * 10 >= total * 8,
        "inside the band for {inside} of {total} frames"
    );
    assert!(
        best.raw() >= sim::Fx::from_int(sim::tuning::tier_blink()).raw(),
        "her lower bar never reached the blink: best {}",
        best.to_int()
    );
}
