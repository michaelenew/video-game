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
use sim::state::Phase;
use sim::{Class, Input, V3, World};

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
    let a = spar(classes, [Level::Normal; 2], MINUTE, 99);
    let b = spar(classes, [Level::Normal; 2], MINUTE, 99);
    assert_eq!(a, b);
    let c = spar(classes, [Level::Normal; 2], MINUTE, 100);
    assert_ne!(a, c, "a different seed played exactly the same minute");
}

/// **It does not do one thing.** Over a fight it tries most of its plans, and
/// none of them takes up most of its time.
#[test]
fn it_mixes_it_up() {
    for class in ALL_CLASSES {
        let mut frames = [[0u32; 9]; 2];
        spar_traced(
            [class, class],
            [Level::Normal; 2],
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
            [Level::Normal; 2],
            3 * MINUTE,
            11,
            |w, _| {
                if matches!(w.phase, Phase::Fighting) {
                    outside += w.players[..w.present()]
                        .iter()
                        .filter(|p| !inside(p))
                        .count() as u32;
                }
            },
        );
        for i in 0..hunt::duel::DUEL {
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
    let half = sim::arena::proving_ground::half();
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
        let bout = spar([class, class], [Level::Normal; 2], 3 * MINUTE, 5);
        for i in 0..hunt::duel::DUEL {
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
        [Level::Normal; 2],
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

/// The sparring bot's walls are the arena's: in the range, two hundred metres
/// long, the middle of the floor is nowhere near one, and in the proving
/// ground the test is the one it always was.
#[test]
fn the_bot_reads_the_walls_off_the_arena() {
    use hunt::duel::{by_the_wall, to_the_wall};
    use sim::arena::{proving_ground, range};
    use sim::fixed::Fx;
    let middle_of_the_range = V3::new(Fx::from_int(100), Fx::ZERO, Fx::ZERO);
    assert!(!by_the_wall(
        middle_of_the_range,
        Fx::from_int(4),
        &range::ARENA.bounds
    ));
    assert!(by_the_wall(
        middle_of_the_range,
        Fx::from_int(4),
        &proving_ground::ARENA.bounds
    ));
    let near = V3::new(Fx::from_int(118), Fx::ZERO, Fx::from_int(3));
    assert!(by_the_wall(near, Fx::from_int(4), &range::ARENA.bounds));
    assert_eq!(to_the_wall(near, &range::ARENA.bounds), Fx::from_int(2));
    // In the proving ground, exactly the old arithmetic: |x| against the
    // half-width, either side.
    let half = proving_ground::half();
    for (x, z) in [(-13, 2), (12, -11), (0, 0), (14, 14), (-7, 13)] {
        let p = V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z));
        let old = p.x.abs().sub(half).abs().min(p.z.abs().sub(half).abs());
        assert_eq!(to_the_wall(p, &proving_ground::ARENA.bounds), old);
        let old_by = |m: Fx| {
            let h = half.sub(m);
            p.x.abs().raw() > h.raw() || p.z.abs().raw() > h.raw()
        };
        for m in [0, 2, 4] {
            let m = Fx::from_int(m);
            assert_eq!(by_the_wall(p, m, &proving_ground::ARENA.bounds), old_by(m));
        }
    }
}

/// **The levels climb.** Each sees, notices, punishes, aims and turns at least
/// as well as the one below it -- and none of them reacts under fifteen
/// frames, which is about as quick as a person waiting for something.
#[test]
fn the_levels_climb() {
    for pair in Level::ALL.windows(2) {
        let (lo, hi) = (pair[0].skill(), pair[1].skill());
        let names = (pair[0].name(), pair[1].name());
        assert!(
            hi.react_min <= lo.react_min && hi.react_max <= lo.react_max,
            "{names:?}"
        );
        assert!(
            hi.notice >= lo.notice && hi.punish >= lo.punish,
            "{names:?}"
        );
        assert!(hi.aim_error.raw() <= lo.aim_error.raw(), "{names:?}");
        assert!(hi.turn.raw() >= lo.turn.raw(), "{names:?}");
        assert!(
            hi.think_max <= lo.think_max && hi.slop <= lo.slop,
            "{names:?}"
        );
        assert!(hi.lead >= lo.lead, "{names:?}");
    }
    for level in Level::ALL {
        assert!(
            level.skill().react_min >= 15,
            "{} reacts too fast",
            level.name()
        );
    }
}

/// **It knows what hurts.** The Elementalist's two pushes go straight through
/// a body (2026-10-02), so measured on somebody they take nothing, and the
/// pillar takes plenty. Thrown things -- the Shuriken, the Thorn lash, which
/// have no damage of their own in the move table -- are measured at what the
/// thing they throw does.
#[test]
fn it_knows_what_hurts() {
    use hunt::duel::Kit;
    use sim::moves::elementalist::keys;
    let kit = Kit::measure(Class::Elementalist);
    let on = |bits: u16| kit.iter().find(|t| t.bits == bits).unwrap();
    assert_eq!(on(keys::WEAK_PUSH).hurt, 0);
    assert_eq!(on(keys::STRONG_PUSH).hurt, 0);
    assert!(on(keys::FIRE).hurt > 100, "{kit:?}");
    let reaver = Kit::measure(Class::ShadowReaver);
    let shuriken = reaver
        .iter()
        .find(|t| t.kind == sim::moves::reaver::SHURIKEN)
        .unwrap();
    assert!(shuriken.hurt > 0 && shuriken.far.raw() >= sim::Fx::from_int(6).raw());
    let blood = Kit::measure(Class::BloodMage);
    let lash = blood
        .iter()
        .find(|t| t.kind == sim::moves::blood::THORN_LASH)
        .unwrap();
    assert!(lash.hurt > 0, "{blood:?}");
}

/// **It does not push at nobody.** The Elementalist's pushes go through a
/// body, so she throws them to kick one of her stones into somebody: over
/// three minutes against a Champion, nearly every push she throws has a stone
/// of hers near the line between them. Until 2026-10-20 she threw them all
/// fight with nothing to push, and did nothing with them.
///
/// Near rather than on the line, by two metres: what she aims at is where she
/// saw him a reaction ago, and he has moved since.
#[test]
fn the_elementalist_pushes_stones() {
    use sim::class::Mechanic;
    use sim::moves::elementalist::keys;
    let kit = hunt::duel::Kit::measure(Class::Elementalist);
    let pushes: Vec<u8> = kit
        .iter()
        .filter(|t| t.bits == keys::WEAK_PUSH || t.bits == keys::STRONG_PUSH)
        .map(|t| t.kind)
        .collect();
    let (mut aimed, mut blind) = (0u32, 0u32);
    let mut was = None;
    spar_traced(
        [Class::Elementalist, Class::Champion],
        [Level::Normal; 2],
        3 * MINUTE,
        8,
        |w, _| {
            let (me, them) = (w.players[0], w.players[1]);
            let now = match me.action {
                sim::state::Action::Startup { kind, .. } => Some(kind),
                _ => None,
            };
            if let Some(k) = now
                && now != was
                && pushes.contains(&k)
            {
                let line = them.pos.sub(me.pos);
                let (dist, dir) = (line.flat_len(), line.normalized());
                let Mechanic::Structures(stones) = me.mechanic else {
                    unreachable!()
                };
                let near = stones.iter().flatten().any(|s| {
                    let rel = s.at.sub(me.pos);
                    let along = rel.dot(dir);
                    let off = rel.sub(dir.scale(along)).flat_len();
                    along.raw() > 0
                        && along.raw() < dist.raw()
                        && off.raw() < sim::Fx::from_int(2).raw()
                });
                if near {
                    aimed += 1;
                } else {
                    blind += 1;
                }
            }
            was = now;
        },
    );
    assert!(aimed >= 10, "she pushed a stone at him only {aimed} times");
    assert!(
        blind * 3 < aimed,
        "{blind} pushes with no stone near the line, {aimed} with one"
    );
}
