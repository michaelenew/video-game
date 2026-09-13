//! Judging whether the fight is any good.
//!
//! "Feels good" is not a measurement. What a good fight *needs* can be
//! measured, and these are the proxies -- each one chosen because a fight that
//! scores badly on it is a fight somebody would complain about, in words you
//! could predict.
//!
//! The numbers are observations, not verdicts. `crates/hunt/tests/fight.rs`
//! turns the handful that are actual design decisions into assertions, in the
//! style of the rest of the feel harness: relationships, not values.

use sim::fixed::Fx;
use sim::monster::{self, Doing, MOVES};
use sim::state::{MAX_PLAYERS, Phase, QUARRY};
use sim::{V3, World};

use crate::{Hunter, Intent, REACTION};

/// What the reaction threshold means, for the failure message of the test that
/// checks it. Kept next to the measure so the explanation cannot drift from it.
pub const REACTION_NOTE: &str = "A startup below about fifteen frames cannot be answered on sight at 60 Hz,      so a move set made of those is one you memorise rather than read.";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    /// The hunters killed it, on this frame.
    Killed(u32),
    /// It killed them.
    Died,
    /// Neither, inside the frame budget. A fight that cannot end is its own
    /// kind of failure, which is why this is not folded into "died".
    Unresolved,
}

/// One thing worth seeing in the timeline.
#[derive(Clone, Copy, Debug)]
pub struct Beat {
    pub frame: u32,
    pub kind: u8,
    pub intent: Intent,
    pub aboard: bool,
    /// Flat distance from the nearest hunter when the creature committed.
    pub range: Fx,
}

pub struct Report {
    pub frames: u32,
    pub outcome: Outcome,

    /// Every move it started, by index, and how many of those could be answered
    /// on sight.
    pub starts: [u32; MOVES],
    pub reactable: u32,
    pub committed: u32,
    pub longest_repeat: u32,
    run_kind: u8,
    run_len: u32,

    /// Frames it spent punishable, and how many separate windows that was.
    pub open_frames: u32,
    pub openings: u32,
    pub shortest_opening: u32,
    run_open: u32,
    was_open: bool,

    /// Frames it was doing nothing at all.
    pub idle_frames: u32,

    /// Riding.
    pub ride_frames: u32,
    pub rides: u32,
    pub longest_ride: u32,
    run_ride: u32,
    was_aboard: bool,
    pub thrown: u32,

    /// Damage, both ways.
    pub dealt: i32,
    pub taken: i32,
    pub hits_taken: u32,
    pub unanswerable: u32,
    pub ridge_hits: u32,
    pub topples: u32,
    pub legs_broken: u32,
    /// Damage put into the four feet, and into the worst-hit one of them.
    ///
    /// It exists because "legs broken: 0" is two completely different findings
    /// -- the hunter never swung at a leg, or it swung at them constantly and
    /// they are too tough -- and nothing else in the report tells them apart.
    /// The ground game is the only way onto the animal that does not need a
    /// platform or a piece of luck, so a hunt that never touches a foot is a
    /// hunt playing half the fight.
    pub foot_damage: i32,
    pub worst_foot: i32,

    /// How much of the arena the fight used.
    lo: V3,
    hi: V3,
    pub spread: Fx,

    /// What the creature was committed to, and how far away the hunters were
    /// when it committed. The second is what makes "unanswerable" checkable
    /// rather than arguable.
    commit_kind: u8,
    commit_range: [Fx; MAX_PLAYERS],

    pub timeline: Vec<Beat>,
}

impl Default for Report {
    fn default() -> Report {
        Report::new()
    }
}

impl Report {
    pub fn new() -> Report {
        Report {
            frames: 0,
            outcome: Outcome::Unresolved,
            starts: [0; MOVES],
            reactable: 0,
            committed: 0,
            longest_repeat: 0,
            run_kind: monster::NO_PART,
            run_len: 0,
            open_frames: 0,
            openings: 0,
            shortest_opening: u32::MAX,
            run_open: 0,
            was_open: false,
            idle_frames: 0,
            ride_frames: 0,
            rides: 0,
            longest_ride: 0,
            run_ride: 0,
            was_aboard: false,
            thrown: 0,
            dealt: 0,
            taken: 0,
            hits_taken: 0,
            unanswerable: 0,
            ridge_hits: 0,
            topples: 0,
            legs_broken: 0,
            foot_damage: 0,
            worst_foot: 0,
            lo: V3::ZERO,
            hi: V3::ZERO,
            spread: Fx::ZERO,
            commit_kind: monster::NO_PART,
            commit_range: [Fx::ZERO; MAX_PLAYERS],
            timeline: Vec::new(),
        }
    }

    /// Fold one tick into the report.
    pub fn observe(&mut self, before: &World, after: &World, bots: &[Hunter]) {
        let (Some(was), Some(now)) = (before.monster, after.monster) else {
            return;
        };
        self.frames = after.frame;

        // Where the fight is happening.
        if self.frames == 1 {
            self.lo = now.pos;
            self.hi = now.pos;
        }
        self.lo = V3::new(self.lo.x.min(now.pos.x), Fx::ZERO, self.lo.z.min(now.pos.z));
        self.hi = V3::new(self.hi.x.max(now.pos.x), Fx::ZERO, self.hi.z.max(now.pos.z));

        // A move beginning. `left` still equal to the whole startup is the one
        // frame it can be said to have started on.
        if let Doing::Startup { kind, left } = now.doing {
            let m = monster::attack(kind);
            if left == m.startup && was.doing.attacking() != Some(kind) {
                self.starts[kind as usize] += 1;
                if m.damage > 0 {
                    self.committed += 1;
                    if m.startup as usize >= REACTION {
                        self.reactable += 1;
                    }
                }
                if kind == self.run_kind {
                    self.run_len += 1;
                } else {
                    self.run_kind = kind;
                    self.run_len = 1;
                }
                self.longest_repeat = self.longest_repeat.max(self.run_len);

                self.commit_kind = kind;
                let mut nearest = Fx::MAX;
                for i in 0..MAX_PLAYERS {
                    let d = V3::new(
                        after.players[i].pos.x.sub(now.pos.x),
                        Fx::ZERO,
                        after.players[i].pos.z.sub(now.pos.z),
                    )
                    .flat_len();
                    self.commit_range[i] = d;
                    nearest = nearest.min(d);
                }
                self.timeline.push(Beat {
                    frame: after.frame,
                    kind,
                    intent: bots.first().map(|b| b.intent).unwrap_or(Intent::Circle),
                    aboard: after.players.iter().any(|p| p.aboard()),
                    range: nearest,
                });
            }
        }

        // Openings: contiguous windows where it cannot answer.
        let open = now.doing.open();
        if open {
            self.open_frames += 1;
            self.run_open += 1;
        }
        if self.was_open && !open {
            self.openings += 1;
            self.shortest_opening = self.shortest_opening.min(self.run_open);
            self.run_open = 0;
        }
        self.was_open = open;

        if matches!(now.doing, Doing::Prowl) {
            self.idle_frames += 1;
        }
        if matches!(now.doing, Doing::Toppled { .. }) && !matches!(was.doing, Doing::Toppled { .. })
        {
            self.topples += 1;
        }
        for part in 0..monster::PARTS {
            if was.part_health[part] > 0 && now.part_health[part] <= 0 {
                self.legs_broken += 1;
            }
            let into = (was.part_health[part] - now.part_health[part]).max(0);
            self.foot_damage += into;
            if now.part_health[part] < was.part_health[part] {
                self.worst_foot = self
                    .worst_foot
                    .max(sim::tuning::limb_health() - now.part_health[part]);
            }
        }
        if now.poise > was.poise {
            self.ridge_hits += 1;
        }
        self.dealt += (was.health - now.health).max(0);

        // Riding, and being removed from the ride.
        let aboard = after.players.iter().any(|p| p.aboard());
        if aboard {
            self.ride_frames += 1;
            self.run_ride += 1;
        }
        if self.was_aboard && !aboard {
            self.rides += 1;
            self.longest_ride = self.longest_ride.max(self.run_ride);
            self.run_ride = 0;
            // Thrown rather than stepped off: you are helpless on the way down.
            if after
                .players
                .iter()
                .any(|p| matches!(p.action, sim::state::Action::HitStun { .. }))
            {
                self.thrown += 1;
            }
        }
        self.was_aboard = aboard;

        // Damage to the hunters, and whether they had any way to answer it.
        for i in 0..MAX_PLAYERS {
            let lost = before.players[i].health - after.players[i].health;
            if lost <= 0 {
                continue;
            }
            self.taken += lost;
            self.hits_taken += 1;
            if self.commit_kind == monster::NO_PART {
                continue;
            }
            let m = monster::attack(self.commit_kind);
            // **Unanswerable**: too fast to answer on sight, *and* it reached
            // somewhere the hunter could not have known was inside it.
            //
            // The threshold is the move's own volume -- how far the hit
            // actually extends from the creature's centre -- rather than the
            // distance the move prefers. A player learns a monster's reach and
            // is expected to; being hit from inside a reach you have seen is a
            // decision you made. Being hit from *outside* it, by something you
            // could not react to, is not, and it happens when the animal walks
            // into you during a startup too short to answer.
            let reach = m
                .hit_x
                .abs()
                .add(m.hit_radius)
                .add(sim::tuning::body_radius());
            if (m.startup as usize) < REACTION && self.commit_range[i].raw() > reach.raw() {
                self.unanswerable += 1;
            }
        }
    }

    pub fn finish(&mut self, w: &World) {
        if self.was_aboard {
            self.rides += 1;
            self.longest_ride = self.longest_ride.max(self.run_ride);
        }
        if self.was_open {
            self.openings += 1;
            self.shortest_opening = self.shortest_opening.min(self.run_open);
        }
        if self.shortest_opening == u32::MAX {
            self.shortest_opening = 0;
        }
        self.spread = self.hi.sub(self.lo).flat_len();
        self.outcome = match w.phase {
            Phase::RoundOver { winner, .. } if winner == QUARRY => Outcome::Died,
            Phase::RoundOver { .. } => Outcome::Killed(w.frame),
            Phase::Fighting => Outcome::Unresolved,
        };
    }

    // -----------------------------------------------------------------------
    // Derived measures
    // -----------------------------------------------------------------------

    /// Share of the attacks it actually threw that a person could answer on
    /// sight -- what the fight *felt* like, which depends on the mix.
    ///
    /// All of them and the fight is a metronome; none of them and it is
    /// memorisation. It wants to be a majority and not a totality.
    pub fn reactable_share(&self) -> f32 {
        ratio(self.reactable, self.committed)
    }

    /// How many of the six moves could be answered on sight -- what the move
    /// *set* is, independent of how often each came up. Reported next to the
    /// share because they say different things and it is easy to read one as
    /// the other.
    pub fn reactable_moves(&self) -> usize {
        (0..MOVES)
            .map(|k| monster::attack(k as u8))
            .filter(|m| m.damage > 0 && m.startup as usize >= REACTION)
            .count()
    }

    /// How many of the six do damage at all. The shake does not.
    pub fn damaging_moves(&self) -> usize {
        (0..MOVES)
            .map(|k| monster::attack(k as u8))
            .filter(|m| m.damage > 0)
            .count()
    }

    /// Moves it started per minute. The rhythm of the fight, as one number.
    pub fn moves_per_minute(&self) -> f32 {
        let total: u32 = self.starts.iter().sum();
        if self.frames == 0 {
            return 0.0;
        }
        total as f32 * 3600.0 / self.frames as f32
    }

    pub fn openings_per_minute(&self) -> f32 {
        if self.frames == 0 {
            return 0.0;
        }
        self.openings as f32 * 3600.0 / self.frames as f32
    }

    /// Average punish window, in frames. Shorter than the hunter's fastest
    /// meaningful attack and it is not an opening, it is a tease.
    pub fn mean_opening(&self) -> f32 {
        ratio(self.open_frames, self.openings)
    }

    pub fn idle_share(&self) -> f32 {
        ratio(self.idle_frames, self.frames)
    }

    pub fn ride_share(&self) -> f32 {
        ratio(self.ride_frames, self.frames)
    }

    pub fn mean_ride(&self) -> f32 {
        ratio(self.ride_frames, self.rides)
    }

    /// How many of its six moves it ever actually used. A move it never throws
    /// is a design fiction.
    pub fn coverage(&self) -> usize {
        self.starts.iter().filter(|n| **n > 0).count()
    }

    /// Share of its moves taken by its favourite one. A one-note monster scores
    /// near one however hard it hits.
    pub fn dominant_share(&self) -> f32 {
        let total: u32 = self.starts.iter().sum();
        ratio(self.starts.iter().copied().max().unwrap_or(0), total)
    }

    /// Shannon entropy of the move distribution, as a share of the most it
    /// could be. One is a perfectly even mix; zero is one move forever.
    ///
    /// Reported for a person to read rather than asserted on: the share above
    /// and the longest repeat are the integer versions, and those are what the
    /// tests pin.
    pub fn entropy(&self) -> f32 {
        let total: u32 = self.starts.iter().sum();
        if total == 0 {
            return 0.0;
        }
        let mut h = 0.0f32;
        for n in self.starts.iter().filter(|n| **n > 0) {
            let p = *n as f32 / total as f32;
            h -= p * p.log2();
        }
        h / (MOVES as f32).log2()
    }

    /// The whole thing, as text.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("\nTHE HUNT\n");
        let seconds = self.frames as f32 / 60.0;
        out.push_str(&format!(
            "  {}  --  {} frames, {:.1} s\n",
            match self.outcome {
                Outcome::Killed(f) => format!("killed at frame {f}"),
                Outcome::Died => "the hunters went down".to_string(),
                Outcome::Unresolved => "nobody won inside the budget".to_string(),
            },
            self.frames,
            seconds
        ));

        out.push_str("\nWHAT IT DID\n");
        for (kind, count) in self.starts.iter().enumerate() {
            let m = monster::attack(kind as u8);
            out.push_str(&format!(
                "  {:<16} {:>3}   {:>2}/{:>2}/{:>2}   {}\n",
                m.name,
                count,
                m.startup,
                m.active,
                m.recovery,
                if (m.startup as usize) < REACTION {
                    "unreactable"
                } else {
                    ""
                }
            ));
        }

        out.push_str("\nDOES THE FIGHT HAVE DECISIONS IN IT\n");
        let line = |out: &mut String, name: &str, value: String, why: &str| {
            out.push_str(&format!("  {name:<26} {value:>9}   {why}\n"));
        };
        line(
            &mut out,
            "reactable share",
            format!("{:.0}%", self.reactable_share() * 100.0),
            "answerable on sight, not from memory",
        );
        line(
            &mut out,
            "reactable moves",
            format!("{}/{}", self.reactable_moves(), self.damaging_moves()),
            "of the move set, not of what it threw",
        );
        line(
            &mut out,
            "moves per minute",
            format!("{:.1}", self.moves_per_minute()),
            "the rhythm",
        );
        line(
            &mut out,
            "openings per minute",
            format!("{:.1}", self.openings_per_minute()),
            "how often you get a turn",
        );
        line(
            &mut out,
            "mean opening",
            format!("{:.0}f", self.mean_opening()),
            "long enough to punish?",
        );
        line(
            &mut out,
            "shortest opening",
            format!("{}f", self.shortest_opening),
            "the worst case",
        );
        line(
            &mut out,
            "idle share",
            format!("{:.0}%", self.idle_share() * 100.0),
            "doing nothing at all",
        );
        line(
            &mut out,
            "move coverage",
            format!("{}/{}", self.coverage(), MOVES),
            "moves it ever used",
        );
        line(
            &mut out,
            "favourite move share",
            format!("{:.0}%", self.dominant_share() * 100.0),
            "one-note?",
        );
        line(
            &mut out,
            "move entropy",
            format!("{:.2}", self.entropy()),
            "1.00 is an even mix",
        );
        line(
            &mut out,
            "longest repeat",
            format!("{}", self.longest_repeat),
            "same move in a row",
        );
        line(
            &mut out,
            "arena used",
            format!("{:?}", self.spread),
            "metres the creature ranged over",
        );

        out.push_str("\nTHE CLIMB\n");
        line(
            &mut out,
            "rides",
            format!("{}", self.rides),
            "times anyone got on",
        );
        line(
            &mut out,
            "ride share",
            format!("{:.0}%", self.ride_share() * 100.0),
            "of the fight spent aboard",
        );
        line(
            &mut out,
            "mean ride",
            format!("{:.0}f", self.mean_ride()),
            "long enough to reach the ridge?",
        );
        line(
            &mut out,
            "longest ride",
            format!("{}f", self.longest_ride),
            "the best one",
        );
        line(
            &mut out,
            "thrown off",
            format!("{}", self.thrown),
            "ended by a buck, not a jump",
        );
        line(
            &mut out,
            "ridge hits",
            format!("{}", self.ridge_hits),
            "damage on the weak point",
        );
        line(
            &mut out,
            "topples",
            format!("{}", self.topples),
            "poise broken",
        );
        line(&mut out, "legs broken", format!("{}", self.legs_broken), "");
        line(
            &mut out,
            "damage into feet",
            format!("{}", self.foot_damage),
            "the ground game",
        );
        line(
            &mut out,
            "worst foot",
            format!("{} of {}", self.worst_foot, sim::tuning::limb_health()),
            "how close one came to going",
        );

        out.push_str("\nWAS IT FAIR\n");
        line(
            &mut out,
            "damage dealt",
            format!("{}", self.dealt),
            "to the creature",
        );
        line(
            &mut out,
            "damage taken",
            format!("{}", self.taken),
            "by the hunters",
        );
        line(&mut out, "hits taken", format!("{}", self.hits_taken), "");
        line(
            &mut out,
            "unanswerable hits",
            format!("{}", self.unanswerable),
            "too fast to read, from outside its range",
        );
        out
    }

    /// The fight, one line per committed move. This is the play sequence made
    /// readable: what it threw, from how far, and what the hunter was in the
    /// middle of trying to do about it.
    pub fn trace(&self) -> String {
        let mut out = String::from("\nTHE PLAY SEQUENCE\n");
        for beat in &self.timeline {
            out.push_str(&format!(
                "  {:>5}  {:<16} at {:>6?} m   hunter: {:?}{}\n",
                beat.frame,
                monster::attack(beat.kind).name,
                beat.range,
                beat.intent,
                if beat.aboard { "  [aboard]" } else { "" }
            ));
        }
        out
    }
}

fn ratio(a: u32, b: u32) -> f32 {
    if b == 0 { 0.0 } else { a as f32 / b as f32 }
}
