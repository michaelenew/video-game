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

use sim::critter;
use sim::fixed::Fx;
use sim::monster::{self, Doing};
use sim::species::{MAX_MOVES, Species};
use sim::state::{MAX_PLAYERS, Phase, QUARRY};
use sim::{V3, World};

use crate::plans::Card;
use crate::{Hunter, Intent, REACTION};

/// Before the first move of a hunt: what the play sequence names the hunter's
/// intent when there is no hunter to ask.
const NOBODY: Intent = Intent("Circle");

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
    /// A hit landing rather than a move starting: what the hunter was in the
    /// middle of when it did, which is the difference between "the telegraph
    /// is too short" and "the bot swung a hammer into the next move".
    pub hit: Option<HunterState>,
}

/// What the first hunter was doing on the frame a hit landed on it.
#[derive(Clone, Copy, Debug)]
pub struct HunterState {
    /// It was on the animal and came off: the fall's damage, not the move's.
    pub bucked: bool,
    pub airborne: bool,
    /// Locked into an attack or a recovery of its own.
    pub busy: bool,
    pub dodging: bool,
    pub health: i32,
}

pub struct Report {
    /// Which creature, and what its card calls things. Every per-move and
    /// per-part line below is read off the species' own table.
    pub species: &'static Species,
    card: &'static Card,
    pub frames: u32,
    pub outcome: Outcome,

    /// Every move it started, by index, and how many of those could be answered
    /// on sight. Room for the species with the most moves; the rest stay zero.
    pub starts: [u32; MAX_MOVES],
    /// How many times each move actually connected with a hunter. Beside
    /// `starts` this is the move's hit rate, which is what says whether a
    /// move's telegraph is doing its job or whether it is a free throw.
    pub landed: [u32; MAX_MOVES],
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
    /// Frames of the fight proper: after the moment the hunt opens with, in
    /// which it stands and takes you in. Every rate and share of the rhythm is
    /// over these, because three seconds of an animal that is not allowed to
    /// do anything is not three seconds of how it fights -- and in a fight won
    /// inside a minute it was enough to read as a creature that stands about.
    pub fought: u32,

    /// **The fight divided by what it lets you do**, frame by frame, by how
    /// long until it can start its next move. See [`Threat`]. The design asks
    /// for about four tenths threatening, a fifth safe to walk up on, and
    /// most of the rest open only to a poke or a fast way in.
    pub threat: [u32; 4],

    /// Riding.
    pub ride_frames: u32,
    pub rides: u32,
    pub longest_ride: u32,
    run_ride: u32,
    was_aboard: bool,
    pub thrown: u32,
    /// Rides that ended, not by a buck, while a bucking move was in progress
    /// or had just been: the rider read it and left. The buck working, by the
    /// other route.
    pub fled: u32,
    /// The last frame a bucking move was winding up or out.
    last_buck: u32,

    /// Attacks the hunters started, and frames on which one connected with
    /// the creature. The ratio is whether the hunter is hitting what it is
    /// swinging at, which "damage dealt" on its own cannot say: a low number
    /// is either a cautious hunter or one swinging at air.
    pub swings: u32,
    pub connected: u32,
    /// Damage, both ways.
    pub dealt: i32,
    pub taken: i32,
    pub hits_taken: u32,
    /// The Bulwark's shield: blows it took, the weight they stored, and the
    /// weight its Slams spent. Zero for every other class. Whether the loop
    /// the class is built on -- take a blow, give it back -- happens at all in
    /// a hunt is a question only these three answer.
    pub guarded: u32,
    pub stored: i32,
    pub spent: i32,
    pub unanswerable: u32,
    /// Hits on a weak point: the poise pool filling. The Ridgeback's ridge
    /// and nape.
    pub ridge_hits: u32,
    pub topples: u32,
    /// Breakable parts broken: the Ridgeback's feet.
    pub legs_broken: u32,
    /// Damage put into the breakable parts -- the Ridgeback's four feet -- and
    /// into the worst-hit one of them.
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

    /// The Blood mage's essence, if a hunter is one: pools her hits spilled
    /// under the creature, and the health she got back by putting a move
    /// through them -- all of it, and the share of it taken while the
    /// creature was on its side, which is the pool the climb exists to earn.
    /// Zero on any other class, since nobody else spills or drinks.
    pub pools_made: u32,
    pub drank: i32,
    pub drank_toppled: i32,
    was_pools: usize,

    /// What the creature was committed to, and how far away the hunters were
    /// when it committed. The second is what makes "unanswerable" checkable
    /// rather than arguable.
    commit_kind: u8,
    commit_range: [Fx; MAX_PLAYERS],
    /// Where the creature stood when it committed.
    commit_at: V3,

    pub timeline: Vec<Beat>,

    /// The small bodies, when the fight has a pack (`sim::pack`). Counted for
    /// every pack creature the same way, so a creature's plan and its report
    /// lines can lean on them.
    pub pack: PackTally,

    /// What the fight put on the ground and asked the hunters to defend, and
    /// what falling cost them (bestiary P4, P6, P7). Printed only when there
    /// is something in it, so a fight with none of it reads as it always did.
    pub ground: GroundTally,

    /// **Its own lines** (P8): what the species' card counts that no other
    /// creature has -- the Mireback's tar and fire. `None` for a card with
    /// none, which prints exactly what it always did.
    pub extra: Option<Box<dyn Tally>>,
}

/// **A species' own report lines**: made by its card (`plans::Card::tally`),
/// shown every tick of the hunt after the shared measures, and printed after
/// the shared sections.
pub trait Tally: Send + Sync {
    fn observe(&mut self, before: &World, after: &World);
    /// Hits its own rule says were unanswerable that the shared rule could
    /// not see: added to the report's count when the hunt ends.
    fn unanswerable(&self) -> u32 {
        0
    }
    /// Its section of the report.
    fn render(&self, report: &Report) -> String;
}

/// What the report counts about the hunt's lore: the defended things, falls,
/// and the floor.
#[derive(Clone, Debug, Default)]
pub struct GroundTally {
    /// Each defended thing: its name, what it took, what it had at the end,
    /// and whether it broke or arrived.
    pub objectives: Vec<(String, i32, i32, bool, bool)>,
    /// Landings that cost a hunter something, and what they cost.
    pub falls: u32,
    pub fall_damage: i32,
    /// Frames a hunter spent standing in a hazard, and the most hazards on
    /// the floor at once.
    pub hazard_frames: u32,
    pub most_hazards: u32,
}

impl GroundTally {
    fn any(&self) -> bool {
        !self.objectives.is_empty() || self.falls > 0 || self.most_hazards > 0
    }
}

/// What the report counts about a pack: who died, who bit, how many at once.
#[derive(Clone, Debug, Default)]
pub struct PackTally {
    /// The fight had one at all.
    pub present: bool,
    /// Critters killed, and how many of those led the pack.
    pub kills: u32,
    pub leaders_killed: u32,
    /// Critters that came into the fight after it began: spawned into a slot.
    pub spawned: u32,
    /// Blows the hunters landed on critters (a body losing health).
    pub struck: u32,
    /// Hits critters landed on the hunters, and what those moves list as
    /// their damage.
    pub hits_taken: u32,
    pub damage_taken: i32,
    /// The most critters winding up or out at once, and frames with two or
    /// more doing it: what the tokens are for.
    pub most_at_once: u32,
    pub frames_two_or_more: u32,
    /// Frames with every token held, out of the frames it was hunting: the
    /// Gnawers' "tokens live".
    pub tokens_full: u32,
    pub hunting_frames: u32,
    /// Times it scattered, routed, came back, and broke for good.
    pub scatters: u32,
    pub routs: u32,
    pub regroups: u32,
    pub broke: u32,
}

impl Report {
    pub fn new(card: &'static Card) -> Report {
        Report {
            species: card.species.get(),
            card,
            frames: 0,
            outcome: Outcome::Unresolved,
            starts: [0; MAX_MOVES],
            landed: [0; MAX_MOVES],
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
            fought: 0,
            threat: [0; 4],
            ride_frames: 0,
            rides: 0,
            longest_ride: 0,
            run_ride: 0,
            was_aboard: false,
            thrown: 0,
            fled: 0,
            last_buck: 0,
            swings: 0,
            connected: 0,
            dealt: 0,
            taken: 0,
            hits_taken: 0,
            guarded: 0,
            stored: 0,
            spent: 0,
            unanswerable: 0,
            ridge_hits: 0,
            topples: 0,
            legs_broken: 0,
            foot_damage: 0,
            worst_foot: 0,
            lo: V3::ZERO,
            hi: V3::ZERO,
            spread: Fx::ZERO,
            pools_made: 0,
            drank: 0,
            drank_toppled: 0,
            was_pools: 0,
            commit_kind: monster::NO_PART,
            commit_range: [Fx::ZERO; MAX_PLAYERS],
            commit_at: V3::ZERO,
            timeline: Vec::new(),
            pack: PackTally::default(),
            ground: GroundTally::default(),
            extra: card.tally.map(|make| make()),
        }
    }

    /// Fold one tick of the pack into the report, if there is one.
    fn observe_pack(&mut self, before: &World, after: &World) {
        let (Some(was), Some(now)) = (before.pack, after.pack) else {
            return;
        };
        let t = &mut self.pack;
        t.present = true;
        let sp = now.sp();
        let mut at_once = 0;
        for (b, a) in before.critters.iter().zip(after.critters.iter()) {
            if a.alive() && !b.present() {
                t.spawned += 1;
            }
            if b.alive() && !a.alive() && a.state == critter::is::DEAD {
                t.kills += 1;
                if b.has(critter::flag::LEADER) {
                    t.leaders_killed += 1;
                }
            }
            if b.alive() && a.health < b.health {
                t.struck += 1;
            }
            if a.alive() && a.attacking() {
                at_once += 1;
            }
            // A move of its connecting: its hit spent on this frame.
            if a.state == critter::is::ACTIVE
                && a.has(critter::flag::HIT_USED)
                && !(b.state == critter::is::ACTIVE && b.has(critter::flag::HIT_USED))
            {
                t.hits_taken += 1;
                t.damage_taken += sp.attack(a.act).damage;
                if (a.act as usize) < MAX_MOVES {
                    self.landed[a.act as usize] += 1;
                }
            }
            // A move starting: counted with the creature's own, by the
            // species' move numbering, so the report's move table covers a
            // pack's moves too.
            if a.state == critter::is::STARTUP && b.state != critter::is::STARTUP {
                if (a.act as usize) < MAX_MOVES {
                    self.starts[a.act as usize] += 1;
                }
                let m = sp.attack(a.act);
                if m.damage > 0 {
                    self.committed += 1;
                    if m.startup as usize >= REACTION {
                        self.reactable += 1;
                    }
                }
            }
        }
        t.most_at_once = t.most_at_once.max(at_once);
        if at_once >= 2 {
            t.frames_two_or_more += 1;
        }
        if now.mood == sim::pack::mood::HUNTING {
            t.hunting_frames += 1;
            let held = after
                .critters
                .iter()
                .filter(|c| c.has(critter::flag::TOKEN))
                .count();
            if held >= now.token_cap() && now.token_cap() > 0 {
                t.tokens_full += 1;
            }
        }
        if was.mood != now.mood {
            match now.mood {
                sim::pack::mood::SCATTERED => t.scatters += 1,
                sim::pack::mood::ROUTED => t.routs += 1,
                sim::pack::mood::BROKEN => t.broke += 1,
                sim::pack::mood::HUNTING if was.mood == sim::pack::mood::ROUTED => t.regroups += 1,
                _ => {}
            }
        }
        if after.monster().is_none() {
            self.frames = after.frame;
            if now.grace == 0 {
                self.fought += 1;
            }
        }
    }

    /// The ground: falls, hazards underfoot, and the defended things.
    fn observe_ground(&mut self, before: &World, after: &World) {
        let g = &mut self.ground;
        for (was, now) in before.players.iter().zip(after.players.iter()) {
            let paid = sim::state::landing_damage(was, now);
            if paid > 0 {
                g.falls += 1;
                g.fall_damage += paid;
            }
        }
        if after.lore.owner.is_none() {
            return;
        }
        let ground = after.terrain();
        let hazards = ground.floor.iter().count() as u32;
        g.most_hazards = g.most_hazards.max(hazards);
        if hazards > 0 {
            for p in after.players.iter().filter(|p| p.health > 0) {
                if ground.floor.iter().any(|h| h.holds(p.pos)) {
                    g.hazard_frames += 1;
                }
            }
        }
        g.objectives = sim::objective::standing(&after.lore, after.arena())
            .map(|o| {
                (
                    o.decl.name.to_string(),
                    o.state.taken,
                    o.state.health,
                    o.state.broken,
                    o.state.arrived,
                )
            })
            .collect();
    }

    /// Fold one tick into the report.
    pub fn observe(&mut self, before: &World, after: &World, bots: &[Hunter]) {
        if let Some(extra) = self.extra.as_mut() {
            extra.observe(before, after);
        }
        self.observe_pack(before, after);
        self.observe_ground(before, after);
        // The first creature. A fight against two (the Pair) reports on the
        // first; a report per creature is that fight's to add.
        let (Some(&was), Some(&now)) = (before.monster(), after.monster()) else {
            return;
        };
        let sp = self.species;
        self.frames = after.frame;
        let fighting = now.brain.grace == 0;
        if fighting {
            self.fought += 1;
        }
        if now.alive() && fighting {
            self.threat[Threat::of(now.frames_until_free() as i32) as usize] += 1;
        }

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
            let m = sp.attack(kind);
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
                self.commit_at = now.pos;
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
                    intent: bots.first().map(|b| b.intent()).unwrap_or(NOBODY),
                    aboard: after.players.iter().any(|p| p.aboard()),
                    range: nearest,
                    hit: None,
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

        if matches!(now.doing, Doing::Prowl) && fighting {
            self.idle_frames += 1;
        }
        if matches!(now.doing, Doing::Toppled { .. }) && !matches!(was.doing, Doing::Toppled { .. })
        {
            self.topples += 1;
        }
        for part in sp.breakables() {
            let (then, next) = (was.part_health(part), now.part_health(part));
            if then > 0 && next <= 0 {
                self.legs_broken += 1;
            }
            let into = (then - next).max(0);
            self.foot_damage += into;
            if next < then {
                self.worst_foot = self.worst_foot.max(sp.part_health() - next);
            }
        }
        if now.poise > was.poise {
            self.ridge_hits += 1;
        }
        self.dealt += (was.health - now.health).max(0);
        if now.health < was.health {
            self.connected += 1;
        }
        for i in 0..MAX_PLAYERS {
            let (a, b) = (&before.players[i], &after.players[i]);
            if matches!(b.action, sim::state::Action::Startup { .. })
                && !matches!(a.action, sim::state::Action::Startup { .. })
            {
                self.swings += 1;
            }
        }

        // Riding, and being removed from the ride.
        //
        // A bucking move seen begin is a reason to leave even once it has been
        // cut short: a rider who read a slam's rear and jumped, a moment
        // after their own hit on the ridge flinched it out of the slam, left
        // because of the slam. Counted from the last frame one was running,
        // for as long as it takes to see it.
        if now.doing.attacking().is_some_and(self.card.bucks)
            && !matches!(now.doing, Doing::Recovery { .. })
        {
            self.last_buck = after.frame;
        }
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
            } else if after.frame.saturating_sub(self.last_buck) <= REACTION as u32 * 2 {
                self.fled += 1;
            }
        }
        self.was_aboard = aboard;

        // The shield. A rise in weight is a blow taken on it; a fall bigger
        // than a frame of drain is a Slam spending it.
        for i in 0..MAX_PLAYERS {
            let was = sim::bulwark::weight(&before.players[i]);
            let now = sim::bulwark::weight(&after.players[i]);
            let moved = now.sub(was).to_int();
            if moved > 0 {
                self.guarded += 1;
                self.stored += moved;
            } else if moved < -1 {
                self.spent -= moved;
            }
        }

        // Damage to the hunters, and whether they had any way to answer it.
        // The blood on the floor, and what came back off it. A pool that
        // merged into another is not a new pool; one that drained away is
        // not counted against the ones made.
        let pools = after
            .effects
            .iter()
            .flatten()
            .filter(|e| e.is_a_pool())
            .count();
        if pools > self.was_pools {
            self.pools_made += (pools - self.was_pools) as u32;
        }
        self.was_pools = pools;
        for i in 0..MAX_PLAYERS {
            let back = after.players[i].health - before.players[i].health;
            if back > 0 {
                self.drank += back;
                if matches!(now.doing, Doing::Toppled { .. }) {
                    self.drank_toppled += back;
                }
            }
        }

        // **A fight with a floor burns people too.** Where the species lays
        // hazards, health lost on a frame the creature's move did not
        // connect is the floor's, not a hit: counted in what was taken and
        // nowhere else, or every tick of burning tar under a hunter would read
        // as the move in progress landing. A fight without a floor counts as
        // it always did.
        let floored = !sp.fight.hazards.is_empty();
        let connected = !was.hit_used && now.hit_used;
        for i in 0..MAX_PLAYERS {
            let lost = before.players[i].health - after.players[i].health;
            if lost <= 0 {
                continue;
            }
            self.taken += lost;
            if floored && !connected {
                continue;
            }
            self.hits_taken += 1;
            if self.commit_kind == monster::NO_PART {
                continue;
            }
            let p = &after.players[i];
            // A rider bucked off during the move's active window lost health
            // to the fall, not to the volume: that is `thrown`, above.
            let bucked = before.players[i].aboard();
            self.timeline.push(Beat {
                frame: after.frame,
                kind: self.commit_kind,
                intent: bots.first().map(|b| b.intent()).unwrap_or(NOBODY),
                aboard: p.aboard(),
                range: self.commit_range[i],
                hit: Some(HunterState {
                    bucked,
                    airborne: !before.players[i].grounded,
                    busy: !before.players[i].action.actionable(),
                    dodging: matches!(before.players[i].action, sim::state::Action::Dodge { .. }),
                    health: p.health,
                }),
            });
            let m = sp.attack(self.commit_kind);
            // A buck's fall is not a hit; it is the ride's own cost, and it
            // is counted under `thrown`. Only damage while a volume is out
            // is the move's.
            if matches!(now.doing, Doing::Active { .. }) && m.damage > 0 && !bucked {
                self.landed[self.commit_kind as usize] += 1;
            }
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
            //
            // **Measured from where the animal stood when it committed, to
            // where the hunter is when it lands.** Measured from the hunter's
            // position at the commit instead, it blamed the creature for a
            // hunter walking into a stomp at a full run -- which is the one
            // thing the stomp exists to punish, and the answer to it is a
            // position the hunter chose. What is unanswerable is the animal
            // closing a gap you had left it, not you closing one.
            let reach = m
                .hit_x
                .abs()
                .add(m.hit_radius)
                .add(sim::tuning::body_radius());
            let from_commit = V3::new(
                after.players[i].pos.x.sub(self.commit_at.x),
                Fx::ZERO,
                after.players[i].pos.z.sub(self.commit_at.z),
            )
            .flat_len();
            if (m.startup as usize) < REACTION
                && self.commit_range[i].raw() > reach.raw()
                && from_commit.raw() > reach.raw()
            {
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
        if let Some(extra) = self.extra.as_ref() {
            self.unanswerable += extra.unanswerable();
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
        self.species
            .attacks()
            .filter(|m| m.damage > 0 && m.startup as usize >= REACTION)
            .count()
    }

    /// How many of its moves do damage at all. The Ridgeback's shake does not.
    pub fn damaging_moves(&self) -> usize {
        self.species.attacks().filter(|m| m.damage > 0).count()
    }

    /// Moves it started per minute. The rhythm of the fight, as one number.
    pub fn moves_per_minute(&self) -> f32 {
        let total: u32 = self.starts.iter().sum();
        if self.fought == 0 {
            return 0.0;
        }
        total as f32 * 3600.0 / self.fought as f32
    }

    pub fn openings_per_minute(&self) -> f32 {
        if self.fought == 0 {
            return 0.0;
        }
        self.openings as f32 * 3600.0 / self.fought as f32
    }

    /// Average punish window, in frames. Shorter than the hunter's fastest
    /// meaningful attack and it is not an opening, it is a tease.
    pub fn mean_opening(&self) -> f32 {
        ratio(self.open_frames, self.openings)
    }

    /// The share of the fight it spent in one kind of window.
    pub fn threat_share(&self, t: Threat) -> f32 {
        let total: u32 = self.threat.iter().sum();
        ratio(self.threat[t as usize], total)
    }

    pub fn idle_share(&self) -> f32 {
        ratio(self.idle_frames, self.fought)
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
        h / (self.species.moves.len() as f32).log2()
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

        out.push_str("\nWHAT IT DID                thrown   landed   frames\n");
        for (kind, count) in self.starts[..self.species.moves.len()].iter().enumerate() {
            let m = self.species.attack(kind as u8);
            out.push_str(&format!(
                "  {:<16}        {:>3}      {:>3}   {:>2}/{:>2}/{:>2}   {}\n",
                m.name,
                count,
                self.landed[kind],
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
        for (t, what) in [
            (Threat::Threatening, "nothing lands before it answers"),
            (Threat::PokeOnly, "a poke from where you stand"),
            (Threat::Skilled, "a dash or a leap gets you in"),
            (Threat::WalkUp, "walk in and swing"),
        ] {
            line(
                &mut out,
                t.label(),
                format!("{:.0}%", self.threat_share(t) * 100.0),
                what,
            );
        }
        line(
            &mut out,
            "idle share",
            format!("{:.0}%", self.idle_share() * 100.0),
            "doing nothing at all",
        );
        line(
            &mut out,
            "move coverage",
            format!("{}/{}", self.coverage(), self.species.moves.len()),
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
            self.card.words.ride_for,
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
            "left under a buck",
            format!("{}", self.fled),
            "read it and jumped, or stepped off",
        );
        line(
            &mut out,
            self.card.words.weak_hits,
            format!("{}", self.ridge_hits),
            "damage on the weak point",
        );
        line(
            &mut out,
            "topples",
            format!("{}", self.topples),
            "poise broken",
        );
        let words = &self.card.words;
        line(&mut out, words.broken, format!("{}", self.legs_broken), "");
        line(
            &mut out,
            words.into_breakables,
            format!("{}", self.foot_damage),
            words.into_breakables_why,
        );
        line(
            &mut out,
            words.worst,
            format!("{} of {}", self.worst_foot, self.species.part_health()),
            "how close one came to going",
        );

        out.push_str("\nWAS IT FAIR\n");
        line(
            &mut out,
            "swings",
            format!("{}", self.swings),
            "attacks the hunters started",
        );
        line(
            &mut out,
            "connected",
            format!("{}", self.connected),
            "frames one of them landed",
        );
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
            "blows guarded",
            format!("{}", self.guarded),
            "taken on a Bulwark's shield",
        );
        line(
            &mut out,
            "weight stored",
            format!("{}", self.stored),
            "and spent by Slam",
        );
        line(&mut out, "weight spent", format!("{}", self.spent), "");
        line(
            &mut out,
            "unanswerable hits",
            format!("{}", self.unanswerable),
            "too fast to read, from outside its range",
        );

        if self.pack.present {
            let t = &self.pack;
            out.push_str("\nTHE PACK\n");
            line(
                &mut out,
                "killed",
                format!("{}", t.kills),
                if t.leaders_killed > 0 {
                    "the leader among them"
                } else {
                    "small bodies down"
                },
            );
            line(
                &mut out,
                "spawned",
                format!("{}", t.spawned),
                "came into the fight after it began",
            );
            line(
                &mut out,
                "blows landed on them",
                format!("{}", t.struck),
                "the hunters' hits on small bodies",
            );
            line(
                &mut out,
                "hits taken from them",
                format!("{}", t.hits_taken),
                "small bodies' moves that connected",
            );
            line(
                &mut out,
                "damage from them",
                format!("{}", t.damage_taken),
                "as their moves list it",
            );
            line(
                &mut out,
                "most at once",
                format!("{}", t.most_at_once),
                "winding up or out together: the tokens' job",
            );
            line(
                &mut out,
                "two or more",
                format!("{:.0}%", ratio(t.frames_two_or_more, self.fought) * 100.0),
                "of the fight",
            );
            line(
                &mut out,
                "tokens live",
                format!("{:.0}%", ratio(t.tokens_full, t.hunting_frames) * 100.0),
                "every token held, of the time it hunted",
            );
            line(
                &mut out,
                "scatters / routs",
                format!("{} / {}", t.scatters, t.routs),
                "deaths that broke the ring, and the pack",
            );
            line(
                &mut out,
                "regroups / broke",
                format!("{} / {}", t.regroups, t.broke),
                "came back, and left for good",
            );
        }

        if self.ground.any() {
            let g = &self.ground;
            out.push_str("\nTHE GROUND\n");
            for (name, taken, left, broke, arrived) in &g.objectives {
                line(
                    &mut out,
                    name,
                    format!("{taken} / {left}"),
                    if *broke {
                        "taken / left: it broke"
                    } else if *arrived {
                        "taken / left: it arrived"
                    } else {
                        "taken / left"
                    },
                );
            }
            if g.most_hazards > 0 {
                line(
                    &mut out,
                    "hazards, most at once",
                    format!("{}", g.most_hazards),
                    "on the floor together",
                );
                line(
                    &mut out,
                    "frames in one",
                    format!("{}", g.hazard_frames),
                    "a hunter standing in a hazard",
                );
            }
            line(
                &mut out,
                "falls that hurt",
                format!("{} / {}", g.falls, g.fall_damage),
                "landings past the free height, and their cost",
            );
        }

        // Only when there is blood to report: the section is the Blood mage's,
        // and printing zeroes for every other class would say she was there.
        if self.pools_made > 0 || self.drank > 0 {
            out.push_str("\nTHE BLOOD\n");
            line(
                &mut out,
                "pools made",
                format!("{}", self.pools_made),
                "spilled under the creature",
            );
            line(
                &mut out,
                "drank",
                format!("{}", self.drank),
                "health reclaimed through a pool",
            );
            line(
                &mut out,
                "drank while toppled",
                format!("{}", self.drank_toppled),
                self.card.words.toppled_pool,
            );
        }
        if let Some(extra) = self.extra.as_ref() {
            out.push_str(&extra.render(self));
        }
        out
    }

    /// The fight, one line per committed move. This is the play sequence made
    /// readable: what it threw, from how far, and what the hunter was in the
    /// middle of trying to do about it.
    pub fn trace(&self) -> String {
        let mut out = String::from("\nTHE PLAY SEQUENCE\n");
        for beat in &self.timeline {
            if let Some(hit) = beat.hit {
                out.push_str(&format!(
                    "  {:>5}    -> {:<13} {}; hunter was {:?}{}{}{}, {} health left\n",
                    beat.frame,
                    self.species.attack(beat.kind).name,
                    if hit.bucked {
                        "bucked it off"
                    } else {
                        "landed"
                    },
                    beat.intent,
                    if hit.busy { ", mid-attack" } else { "" },
                    if hit.airborne { ", in the air" } else { "" },
                    if hit.dodging { ", dodging" } else { "" },
                    hit.health
                ));
                continue;
            }
            out.push_str(&format!(
                "  {:>5}  {:<16} at {:>6?} m   hunter: {:?}{}\n",
                beat.frame,
                self.species.attack(beat.kind).name,
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

/// What a window lets a hunter do, by how many frames it has.
///
/// Every threshold is a hunter's own reaction plus the swing, plus whatever
/// it takes to get there from the standoff -- nothing for a hunter already in
/// reach or shooting from range, a dash for one with a way in, a walk for
/// anybody. The gap is [`APPROACH_GAP`], the ground between waiting just
/// outside its reach and a foot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Threat {
    /// Shorter than a reaction and a swing. It answers anything that closes.
    Threatening,
    /// Long enough to swing from where you are. A poke already in reach, or a
    /// shot from range.
    PokeOnly,
    /// Long enough to get in with the fastest thing a fighter has, and not
    /// on foot: a dodge, a vault, a structure jump, a dash to a shadow.
    Skilled,
    /// Long enough to walk in from the standoff and swing.
    WalkUp,
}

/// The ground between waiting at the edge of its close reach and a foot.
pub const APPROACH_GAP: Fx = Fx::from_raw(9 << 15);

/// The frames a swing takes to land: a quick poke's startup and first active
/// frame. Deliberately the fastest in the roster rather than any one class's,
/// so that a window called too short is too short for everybody.
const SWING: i32 = 9;

impl Threat {
    pub fn of(frames: i32) -> Threat {
        let step = |speed: Fx| APPROACH_GAP.div(speed.mul(sim::DT)).to_int();
        let poke = REACTION as i32 + SWING;
        let skilled = poke + step(sim::tuning::dodge_speed());
        let walk = poke + step(sim::tuning::move_speed());
        if frames < poke {
            Threat::Threatening
        } else if frames < skilled {
            Threat::PokeOnly
        } else if frames < walk {
            Threat::Skilled
        } else {
            Threat::WalkUp
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Threat::Threatening => "threatening",
            Threat::PokeOnly => "open to a poke",
            Threat::Skilled => "open to a way in",
            Threat::WalkUp => "safe to walk up",
        }
    }
}
