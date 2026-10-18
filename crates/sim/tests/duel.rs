//! A crossbred creature fighting: its actor and brain against a fighter
//! that stands still, and against one that answers every move the way the
//! move says it is answered, a reaction after its tell starts.
//!
//! This is the bestiary contract played rather than measured. Standing
//! still, you get hit -- the creature can hurt you. Answering, you do not
//! get hit by anything answered by reacting -- every one of those moves
//! can be read and beaten by a player with a human reaction and the
//! game's own dodge, jump and crouch.

use sim::breed::actor::{Act, Actor, spin};
use sim::breed::archetypes::CAST;
use sim::breed::brain::{Brain, Choice};
use sim::breed::build::{self, MoveKind};
use sim::breed::strike::{self, Answer};
use sim::breed::{Genome, posture, verdict};
use sim::fixed::Fx;
use sim::math::V3;
use sim::tuning as t;

const fn fx(num: i32, den: i32) -> Fx {
    Fx::ratio(num, den)
}

fn per_frame(speed: Fx) -> Fx {
    speed.div(Fx::from_int(60))
}

/// A fighter: a column on the floor (or off it, jumping).
#[derive(Clone, Copy)]
struct Fighter {
    at: V3,
    post: V3,
    vy: Fx,
    crouch: bool,
    /// Invulnerable through this frame (a dodge's).
    iframes_until: i32,
    /// Moving this way each frame until `until`.
    run: V3,
    run_until: i32,
    /// A jump waiting for its frame.
    jump_at: i32,
    velocity: V3,
}

impl Fighter {
    fn new(at: V3) -> Fighter {
        Fighter {
            at,
            post: at,
            vy: Fx::ZERO,
            crouch: false,
            iframes_until: -1,
            run: V3::ZERO,
            run_until: -1,
            jump_at: -1,
            velocity: V3::ZERO,
        }
    }

    fn height(&self) -> Fx {
        if self.crouch {
            t::body_height().mul(t::crouch_height_scale())
        } else {
            t::body_height()
        }
    }

    fn step(&mut self, now: i32) {
        let before = self.at;
        if now <= self.run_until {
            self.at = self.at.add(self.run);
        } else if self.at.y == Fx::ZERO {
            // Back to its post, at a walk.
            let to = self.post.sub(self.at);
            let d = to.len();
            let walk = per_frame(t::move_speed());
            if d > walk {
                self.at = self.at.add(to.scale(walk.div(d)));
            }
        }
        if self.at.y > Fx::ZERO || self.vy > Fx::ZERO {
            self.at.y = self.at.y.add(per_frame(self.vy));
            self.vy = self.vy.add(per_frame(t::gravity()));
            if self.at.y <= Fx::ZERO {
                self.at.y = Fx::ZERO;
                self.vy = Fx::ZERO;
            }
        }
        self.velocity = self.at.sub(before);
    }

    fn touched_by(&self, s: &strike::Sphere, now: i32) -> bool {
        if now <= self.iframes_until {
            return false;
        }
        let flat = V3::new(s.at.x.sub(self.at.x), Fx::ZERO, s.at.z.sub(self.at.z)).len();
        flat <= s.radius.add(t::body_radius())
            && s.top() >= self.at.y
            && s.bottom() <= self.at.y.add(self.height())
    }
}

#[derive(Default, Debug)]
struct Tally {
    thrown: Vec<MoveKind>,
    /// The longest it stayed under the floor, and whether it was ever
    /// exposed while it was.
    under_most: i32,
    hit_while_under: bool,
    /// Moves that landed, and how they are answered.
    hits: Vec<(MoveKind, Answer)>,
}

/// Answered by reacting, with the fighter's own dodge, jump or crouch.
fn reactive(a: Answer) -> bool {
    matches!(
        a,
        Answer::Sidestep
            | Answer::Dodge
            | Answer::LeaveLane
            | Answer::Jump
            | Answer::Crouch
            | Answer::OffTheMark
    )
}

fn fight(g: &Genome, answering: bool, frames: i32, seed: u64) -> Tally {
    fight_from(g, answering, frames, seed, Fx::from_int(2))
}

/// A fight with the fighter standing `ahead` metres in front of its nose.
fn fight_from(g: &Genome, answering: bool, frames: i32, seed: u64, ahead: Fx) -> Tally {
    let mut me = Actor::new(g, V3::ZERO, Fx::ZERO);
    let rest = build::body_in(g, &me.posture);
    let e = verdict::extent(&rest, g.stance == sim::breed::Stance::Limbless);
    let mut you = Fighter::new(V3::new(e.nose.add(ahead), Fx::ZERO, Fx::ZERO));
    let mut brain = Brain::new(&me, seed);
    let mut tally = Tally::default();
    let mut landed_this_move = false;
    let reaction = t::HUMAN_REACTION_FRAMES as i32;
    for now in 0..frames {
        brain.glance(now, you.at, you.velocity);
        let choice = brain.think(&me, now);
        if let Choice::Strike { index, .. } = choice
            && let Some((spec, _)) = me.kit[index]
        {
            tally.thrown.push(spec.kind);
            landed_this_move = false;
        }
        Brain::act(choice, &mut me);
        me.step();
        // The answer, a reaction after the tell starts, timed as a player
        // who has learned the move times it.
        if answering && let Some((spec, frame, motion)) = me.striking() {
            let start = now - frame;
            let answer = strike::answer(spec.kind);
            if frame == reaction {
                let strike_at = start + spec.startup;
                match answer {
                    Answer::Sidestep | Answer::Dodge | Answer::LeaveLane => {
                        // Off the line the move was pointed along, to the
                        // side you are already on, the dodge's i-frames
                        // over the strike's first frames, then running on
                        // until it is over.
                        let line = spin(
                            V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
                            me.facing.add(motion.struck.yaw),
                        );
                        let rel = you.at.sub(me.at);
                        let side = if line.x.mul(rel.z).sub(line.z.mul(rel.x)) >= Fx::ZERO {
                            Fx::ONE
                        } else {
                            Fx::ONE.neg()
                        };
                        let across = V3::new(line.z.neg(), Fx::ZERO, line.x).scale(side);
                        let go = (strike_at - 2).max(now);
                        you.iframes_until = go + t::dodge_iframes() as i32;
                        you.run = across.scale(per_frame(t::dodge_speed().mul(fx(1, 2))));
                        you.run_until = strike_at + spec.active;
                    }
                    Answer::Jump => {
                        // Highest in the middle of the strike.
                        let apex = t::jump_speed()
                            .div(t::gravity().neg())
                            .mul(Fx::from_int(60))
                            .to_int();
                        let jump = (strike_at + spec.active / 2 - apex).max(now);
                        you.jump_at = jump;
                    }
                    Answer::Crouch => you.crouch = true,
                    // Straight off the churning ground, at a run.
                    Answer::OffTheMark => {
                        if let Some(mark) = me.mark() {
                            let away =
                                V3::new(you.at.x.sub(mark.at.x), Fx::ZERO, you.at.z.sub(mark.at.z));
                            let len = away.len();
                            let away = if len > fx(1, 100) {
                                away.scale(Fx::ONE.div(len))
                            } else {
                                V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE)
                            };
                            you.run = away.scale(per_frame(t::move_speed()));
                            you.run_until = strike_at + spec.active;
                        }
                    }
                    _ => {}
                }
            }
            if frame == spec.startup + spec.active {
                you.crouch = false;
            }
        }
        if now == you.jump_at {
            you.vy = t::jump_speed();
            you.at.y = fx(1, 100);
        }
        you.step(now);
        if let Act::Under { frames, .. } = me.act {
            tally.under_most = tally.under_most.max(frames);
            tally.hit_while_under |= me.exposed();
        }
        if !landed_this_move {
            let hit = me.hit();
            if hit.iter().any(|s| you.touched_by(s, now))
                && let Some((spec, _, _)) = me.striking()
            {
                tally.hits.push((spec.kind, strike::answer(spec.kind)));
                landed_this_move = true;
            }
        }
    }
    tally
}

#[test]
fn standing_still_you_get_hit() {
    for (_, name, g, _) in CAST {
        let tally = fight(g, false, 1800, 3);
        let mut kinds = tally.thrown.clone();
        kinds.sort_by_key(|k| *k as u8);
        kinds.dedup();
        assert!(
            !tally.hits.is_empty(),
            "{name} never hit somebody standing in front of it: threw {:?}",
            tally.thrown
        );
        assert!(kinds.len() >= 2, "{name} threw only {kinds:?}");
    }
}

#[test]
fn answering_every_move_on_sight_you_do_not_get_hit_by_the_reactable_ones() {
    for (_, name, g, _) in CAST {
        let tally = fight(g, true, 1800, 3);
        let bad: Vec<_> = tally.hits.iter().filter(|(_, a)| reactive(*a)).collect();
        assert!(
            bad.is_empty(),
            "{name}: hit through its answer by {bad:?} (threw {:?})",
            tally.thrown
        );
    }
}

#[test]
fn a_creature_decides_on_what_it_saw_a_reaction_ago() {
    let g = CAST[0].2;
    let me = Actor::new(g, V3::ZERO, Fx::ZERO);
    let mut brain = Brain::new(&me, 1);
    brain.glance(0, V3::new(Fx::from_int(5), Fx::ZERO, Fx::ZERO), V3::ZERO);
    assert_eq!(
        brain.believes(10),
        None,
        "it acted on a glance ten frames old"
    );
    assert!(brain.believes(t::HUMAN_REACTION_FRAMES as i32).is_some());
    // Somebody who moves is seen to move only a reaction later.
    for now in (6..60).step_by(6) {
        brain.glance(
            now,
            V3::new(Fx::from_int(5), Fx::ZERO, Fx::from_int(10)),
            V3::ZERO,
        );
        let Some(seen) = brain.believes(now) else {
            continue;
        };
        if now < t::HUMAN_REACTION_FRAMES as i32 + 6 {
            assert_eq!(seen.z, Fx::ZERO, "frame {now}: it saw the move too soon");
        }
    }
}

#[test]
fn a_move_leaves_the_body_where_it_carried_it() {
    let g = CAST[0].2; // the Ridgeback, whose charge runs forty metres
    let mut me = Actor::new(g, V3::ZERO, Fx::ZERO);
    let k = (0..me.kit_len)
        .find(|&k| me.kit[k].is_some_and(|(s, _)| s.kind == MoveKind::Charge))
        .unwrap();
    let (spec, m) = me.kit[k].unwrap();
    assert!(me.strike(k, Fx::ZERO));
    for _ in 0..strike::frames(&spec) {
        me.step();
    }
    assert!(matches!(me.act, Act::Stand));
    let travelled = me.at.x;
    assert!(
        (travelled.sub(m.struck.surge)).abs() < fx(1, 10),
        "{travelled:?} vs {:?}",
        m.struck.surge
    );
    // A tail sweep leaves it turned.
    let sweep = (0..me.kit_len)
        .find(|&k| me.kit[k].is_some_and(|(s, _)| s.kind == MoveKind::TailSweep))
        .unwrap();
    let (spec, m) = me.kit[sweep].unwrap();
    let facing = me.facing;
    me.strike(sweep, Fx::ZERO);
    for _ in 0..strike::frames(&spec) {
        me.step();
    }
    assert_eq!(me.facing, facing.add(m.struck.yaw));
}

#[test]
fn a_still_body_is_built_once() {
    let g = CAST[0].2;
    let mut me = Actor::new(g, V3::ZERO, Fx::ZERO);
    for _ in 0..100 {
        me.step();
        let _ = me.body();
    }
    assert_eq!(me.builds, 1);
}

#[test]
fn a_giant_turns_slower_than_a_gnawer() {
    let giant = Actor::new(
        CAST.iter().find(|c| c.1 == "Siegeshell").unwrap().2,
        V3::ZERO,
        Fx::ZERO,
    );
    let gnawer = Actor::new(
        CAST.iter().find(|c| c.1 == "Gnawers").unwrap().2,
        V3::ZERO,
        Fx::ZERO,
    );
    assert!(giant.turn_rate().mul(Fx::from_int(3)) < gnawer.turn_rate());
}

#[test]
fn a_fight_is_the_same_every_time() {
    let g = CAST[1].2;
    let a = fight(g, true, 900, 9);
    let b = fight(g, true, 900, 9);
    assert_eq!(a.thrown, b.thrown);
    assert_eq!(a.hits, b.hits);
    let _ = posture::posture(g);
}

#[test]
fn the_answer_is_what_saves_you() {
    // The same fights, standing: the moves an answer beats do land on
    // somebody who does not answer them, for most of the cast.
    let mut creatures = 0;
    for (_, _, g, _) in CAST {
        let standing = fight(g, false, 1800, 3);
        if standing.hits.iter().any(|(_, a)| reactive(*a)) {
            creatures += 1;
        }
    }
    assert!(
        creatures >= 8,
        "only {creatures} creatures land a reactable move on somebody standing"
    );
}

#[test]
fn bred_creatures_can_be_answered_too() {
    use sim::breed::Rng;
    use sim::breed::cross::{self, Rates, Trophy, TrophyKind};
    let cast = |n: &str| *CAST.iter().find(|c| c.1 == n).unwrap().2;
    let lines = [
        ("Ridgeback", TrophyKind::Claw, "Mantis"),
        ("Hornback", TrophyKind::Feather, "Galewing"),
        ("Broodmother", TrophyKind::Horn, "Hornback"),
    ];
    for (k, (egg, kind, donor)) in lines.into_iter().enumerate() {
        let mut rng = Rng::new(40 + k as u64);
        let mut g = cast(egg);
        for generation in 1..=30 {
            let t = Trophy {
                kind,
                donor: cast(donor),
            };
            g = cross::graft(&g, &t, &Rates::DEFAULT, &mut rng).genome;
            if generation % 10 == 0 {
                let tally = fight(&g, true, 1200, generation as u64);
                let bad: Vec<_> = tally.hits.iter().filter(|(_, a)| reactive(*a)).collect();
                assert!(
                    bad.is_empty(),
                    "{egg} line, generation {generation}: hit through its answer by {bad:?}"
                );
            }
        }
    }
}

#[test]
fn a_move_turns_to_where_it_was_aimed() {
    use sim::math::atan2_turns;
    let g = CAST[0].2;
    for aim in [fx(-1, 10), Fx::ZERO, fx(1, 10)] {
        let mut me = Actor::new(g, V3::ZERO, Fx::ZERO);
        let k = (0..me.kit_len)
            .find(|&k| me.kit[k].is_some_and(|(s, _)| s.kind == MoveKind::Bite))
            .unwrap();
        let (spec, _) = me.kit[k].unwrap();
        me.strike(k, aim);
        let mut sum = V3::ZERO;
        let mut n = 0;
        for _ in 0..spec.startup + spec.active {
            me.step();
            for s in me.hit().iter() {
                sum = sum.add(s.at);
                n += 1;
            }
        }
        assert!(n > 0);
        let landed = atan2_turns(sum.z, sum.x);
        assert!(
            landed.sub(aim).abs() < fx(1, 40),
            "aimed {aim:?}, landed {landed:?}"
        );
    }
}

fn sandworm() -> &'static Genome {
    CAST.iter().find(|c| c.1 == "Sandmaw").unwrap().2
}

#[test]
fn the_sandworm_goes_under_and_comes_up_under_you() {
    // Twenty metres off: going under is one of its ways in.
    let tally = fight_from(sandworm(), false, 1800, 5, Fx::from_int(20));
    assert!(
        tally.thrown.contains(&MoveKind::Burrow),
        "threw {:?}",
        tally.thrown
    );
    assert!(
        tally.thrown.contains(&MoveKind::Erupt),
        "threw {:?}",
        tally.thrown
    );
    assert!(
        tally.hits.iter().any(|(k, _)| *k == MoveKind::Erupt),
        "it never came up under somebody standing still: {tally:?}"
    );
    assert!(!tally.hit_while_under, "it could be hit while under");
    assert!(
        tally.under_most <= t::breed_under_most(),
        "under for {} frames",
        tally.under_most
    );
}

#[test]
fn stepping_off_the_mark_answers_an_eruption() {
    for seed in 1..6 {
        let tally = fight_from(sandworm(), true, 1800, seed, Fx::from_int(20));
        assert!(
            tally.thrown.contains(&MoveKind::Erupt),
            "seed {seed}: threw {:?}",
            tally.thrown
        );
        let bad: Vec<_> = tally.hits.iter().filter(|(_, a)| reactive(*a)).collect();
        assert!(
            bad.is_empty(),
            "seed {seed}: hit through its answer by {bad:?}"
        );
    }
}

#[test]
fn a_mole_goes_under_and_can_be_answered() {
    use sim::breed::{Build, Tip};
    let mut mole = *CAST.iter().find(|c| c.1 == "Hornback").unwrap().2;
    let p = mole.limbs[0].as_mut().unwrap();
    p.build = Build::Spade;
    p.tip = Tip::Claw;
    let standing = fight_from(&mole, false, 1800, 2, Fx::from_int(20));
    assert!(
        standing.thrown.contains(&MoveKind::Burrow),
        "threw {:?}",
        standing.thrown
    );
    assert!(
        standing.thrown.contains(&MoveKind::Erupt),
        "threw {:?}",
        standing.thrown
    );
    assert!(!standing.hit_while_under);
    for seed in 1..4 {
        let tally = fight_from(&mole, true, 1800, seed, Fx::from_int(20));
        let bad: Vec<_> = tally.hits.iter().filter(|(_, a)| reactive(*a)).collect();
        assert!(
            bad.is_empty(),
            "seed {seed}: hit through its answer by {bad:?}"
        );
    }
}
