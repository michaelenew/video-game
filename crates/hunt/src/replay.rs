//! A person's fight, judged the way the scripted hunter's is.
//!
//! The game records every fight as a [`sim::replay::Tape`]: where it started
//! and what was pressed. This puts one back through the simulation and hands
//! the frames to the same [`Report`] that judges the bot, so the eight or nine
//! numbers that say whether a fight is any good can be read off a fight
//! somebody actually played -- and beside them, what the person did with
//! their hands, move by move, which the bot's report never needed because the
//! bot's plan already says.
//!
//! ```text
//! cargo run -p hunt --bin replay -- <file> [--trace]
//! ```
//!
//! The replay is **checked**: the tape carries the frame and checksum the game
//! ended on, and the judgement says whether this build reached the same place.
//! A mismatch is a different build or a different tuning (the tape names both),
//! and the numbers under it are of a fight nobody played.

use sim::replay::Tape;
use sim::state::{Action, MAX_PLAYERS};
use sim::{Class, Input, World};

use crate::{Hunter, Report};

/// What one fighter did, counted off the world rather than off a plan --
/// **one table per class they played**, because a class changes inside a
/// fight (a trip, the class keys) and a move is only a slot number until its
/// class names it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hands {
    /// The class on the last frame alive.
    pub class: Option<Class>,
    /// Frames alive and in the fight, every class together.
    pub frames: u32,
    /// Each class played, in the order it was first played.
    pub stints: Vec<Stint>,
}

/// One class's share of a fighter's fight.
#[derive(Clone, Debug, PartialEq)]
pub struct Stint {
    pub class: Class,
    /// Moves started, by kind, and the frames one of them was out when a
    /// blow of this fighter's landed on anybody.
    pub thrown: Vec<(u8, u32)>,
    pub landed: Vec<(u8, u32)>,
    pub dodges: u32,
    pub guards: u32,
    pub jumps: u32,
    /// Blows taken: this fighter going into hitstun, a stagger or a hold.
    pub struck: u32,
    pub damage_taken: i32,
    /// Frames alive and in the fight as this class.
    pub frames: u32,
    pub airborne: u32,
}

impl Stint {
    fn new(class: Class) -> Stint {
        Stint {
            class,
            thrown: Vec::new(),
            landed: Vec::new(),
            dodges: 0,
            guards: 0,
            jumps: 0,
            struck: 0,
            damage_taken: 0,
            frames: 0,
            airborne: 0,
        }
    }

    fn bump(list: &mut Vec<(u8, u32)>, kind: u8) {
        match list.iter_mut().find(|(k, _)| *k == kind) {
            Some((_, n)) => *n += 1,
            None => list.push((kind, 1)),
        }
    }

    fn count(list: &[(u8, u32)], kind: u8) -> u32 {
        list.iter().find(|(k, _)| *k == kind).map_or(0, |(_, n)| *n)
    }

    /// The table, move by move.
    fn render(&self, who: usize) -> String {
        let mut out = format!(
            "\nPLAYER {}  {}  --  {} frames, {:.1} s, {:.0}% airborne\n",
            who + 1,
            self.class.name(),
            self.frames,
            self.frames as f32 / 60.0,
            if self.frames == 0 {
                0.0
            } else {
                self.airborne as f32 * 100.0 / self.frames as f32
            }
        );
        out.push_str("  WHAT THEY THREW           thrown   landed\n");
        let mut kinds: Vec<u8> = self.thrown.iter().map(|(k, _)| *k).collect();
        kinds.sort_unstable();
        for kind in kinds {
            let m = sim::moves::get(self.class, kind);
            out.push_str(&format!(
                "  {:<24}   {:>4}     {:>4}\n",
                m.name,
                Stint::count(&self.thrown, kind),
                Stint::count(&self.landed, kind)
            ));
        }
        out.push_str(&format!(
            "  dodges {}   guards raised {}   jumps {}\n  blows taken {}   damage taken {}\n",
            self.dodges, self.guards, self.jumps, self.struck, self.damage_taken
        ));
        out
    }
}

impl Hands {
    /// The stint of the class this fighter is now, started if it is new.
    fn stint(&mut self, class: Class) -> &mut Stint {
        let at = match self.stints.iter().position(|s| s.class == class) {
            Some(at) => at,
            None => {
                self.stints.push(Stint::new(class));
                self.stints.len() - 1
            }
        };
        &mut self.stints[at]
    }

    /// One frame: what changed for seat `who`. A frame is counted to the
    /// class the fighter is *after* it, except a blow landed, which is the
    /// move that was out before it, so the class that threw it.
    fn observe(&mut self, who: usize, before: &World, input: Input, after: &World) {
        let (was, now) = (before.players[who], after.players[who]);
        if was.health <= 0 {
            return;
        }
        self.class = Some(now.class);
        self.frames += 1;
        let changed = was.class != now.class;
        let s = self.stint(now.class);
        s.frames += 1;
        if !now.grounded {
            s.airborne += 1;
        }
        let begun = |a: Action| matches!(a, Action::Startup { .. } | Action::Channel { .. });
        if begun(now.action)
            && (changed
                || !begun(was.action)
                || now.action.attack_kind() != was.action.attack_kind())
            && let Some(kind) = now.action.attack_kind()
        {
            Stint::bump(&mut s.thrown, kind);
        }
        if matches!(now.action, Action::Dodge { .. }) && !matches!(was.action, Action::Dodge { .. })
        {
            s.dodges += 1;
        }
        if now.action.guarding() && !was.action.guarding() {
            s.guards += 1;
        }
        if was.grounded && !now.grounded && input.bits & Input::SPACE != 0 {
            s.jumps += 1;
        }
        let struck = |a: Action| {
            matches!(
                a,
                Action::HitStun { .. } | Action::Stagger { .. } | Action::Held { .. }
            )
        };
        if struck(now.action)
            && (!struck(was.action) || now.action.frames_left() > was.action.frames_left())
        {
            s.struck += 1;
        }
        if now.health < was.health {
            s.damage_taken += was.health - now.health;
        }
        // A blow landing while one of mine is out: the other fighter struck,
        // or a creature or small body losing health.
        if let Some(kind) = was
            .action
            .attack_kind()
            .filter(|_| matches!(was.action, Action::Active { .. }))
        {
            let other = 1 - who;
            let (ow, on) = (before.players[other].action, after.players[other].action);
            let hit_fighter = struck(on) && (!struck(ow) || on.frames_left() > ow.frames_left());
            let hit_creature =
                before
                    .monsters
                    .iter()
                    .zip(after.monsters.iter())
                    .any(|(b, a)| match (b, a) {
                        (Some(b), Some(a)) => a.health < b.health,
                        _ => false,
                    });
            let hit_critter = before
                .critters
                .all
                .iter()
                .zip(after.critters.all.iter())
                .any(|(b, a)| a.health < b.health);
            if hit_fighter || hit_creature || hit_critter {
                Stint::bump(&mut self.stint(was.class).landed, kind);
            }
        }
    }

    /// One table per class played, in the order they were played.
    pub fn render(&self, who: usize) -> String {
        if self.stints.is_empty() {
            return format!("\nPLAYER {}  sat out\n", who + 1);
        }
        self.stints.iter().map(|s| s.render(who)).collect()
    }
}

/// **One place of the valley, as a replay went through it**
/// (`docs/design/valley.md`): when you came in and went on, what the
/// climbing cost, and whether the two of you went on together.
#[derive(Clone, Debug, PartialEq)]
pub struct Leg {
    pub arena: sim::arena::ArenaId,
    pub entered: u32,
    pub left: Option<u32>,
    /// Damage landings: in a peaceful place, the only damage there is is a
    /// fall's.
    pub falls: [u32; MAX_PLAYERS],
    /// Times stood back on a cairn after a death.
    pub stood_back: [u32; MAX_PLAYERS],
    /// Cairns touched, in order, by their index on the valley's map (each
    /// new one once).
    pub cairns: Vec<u16>,
    /// Where it went on to, and whether everybody walked in together or one
    /// went ahead.
    pub out: Option<(&'static str, bool)>,
}

/// The valley, place by place. Empty for a replay that never was in it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Trek {
    pub legs: Vec<Leg>,
}

impl Trek {
    fn observe(&mut self, before: &World, after: &World) {
        if !after.valley.on && !before.valley.on {
            return;
        }
        if self.legs.is_empty() || before.arena != after.arena {
            if let Some(last) = self.legs.last_mut() {
                last.left = Some(after.frame);
                // The seam it went on by is the one of the last place that
                // leads here; it went together if everybody on their feet
                // was in the new place when the world moved into it.
                let seam = sim::valley::place(before.arena)
                    .and_then(|p| p.seams.iter().find(|s| s.to == after.arena));
                if let Some(seam) = seam {
                    let origin = after.map_origin();
                    let seats = (after.seats as usize).clamp(1, MAX_PLAYERS);
                    let together = sim::atlas::valley().placed(after.arena).is_some_and(|to| {
                        after.players[..seats]
                            .iter()
                            .filter(|p| p.health > 0)
                            .all(|p| {
                                let f = p.pos.add(origin);
                                to.holds(f.x, f.z)
                            })
                    });
                    last.out = Some((seam.says, together));
                }
            }
            self.legs.push(Leg {
                arena: after.arena,
                entered: after.frame,
                left: None,
                falls: [0; MAX_PLAYERS],
                stood_back: [0; MAX_PLAYERS],
                cairns: Vec::new(),
                out: None,
            });
            return;
        }
        let leg = self.legs.last_mut().expect("a leg");
        if !after.peaceful() {
            return;
        }
        for i in 0..MAX_PLAYERS {
            let (was, now) = (before.players[i], after.players[i]);
            if was.health <= 0 {
                continue;
            }
            let jumped = now.pos.sub(was.pos).flat_len().raw() > sim::Fx::from_int(3).raw();
            if jumped && now.health >= was.health {
                leg.stood_back[i] += 1;
            } else if now.health < was.health {
                leg.falls[i] += 1;
            }
            let c = after.valley.cairn[i];
            if c != before.valley.cairn[i] && c > 0 && !leg.cairns.contains(&(c - 1)) {
                leg.cairns.push(c - 1);
            }
        }
    }

    /// The table, place by place.
    pub fn render(&self, end: u32) -> String {
        if self.legs.is_empty() {
            return String::new();
        }
        let first = self.legs[0].entered;
        let clock = |f: u32| {
            let s = f.saturating_sub(first) / 60;
            format!("{}:{:02}", s / 60, s % 60)
        };
        let total = end.saturating_sub(first) / 60;
        let mut out = format!(
            "\nTHE VALLEY  {} places, {}:{:02} in all\n",
            self.legs.len(),
            total / 60,
            total % 60
        );
        out.push_str(
            "  PLACE            IN     OUT     TIME   FALLS   STOOD BACK   CAIRNS   ON TO\n",
        );
        for leg in &self.legs {
            let left = leg.left.unwrap_or(end);
            let on = match leg.out {
                Some((to, true)) => format!("{to}, together"),
                Some((to, false)) => format!("{to}, apart"),
                None if leg.left.is_some() => "(a trip)".to_string(),
                None => "-".to_string(),
            };
            out.push_str(&format!(
                "  {:<15}  {:>5}  {:>5}  {:>5} s   {}/{}       {}/{}         {:>2}     {on}\n",
                leg.arena.get().name,
                clock(leg.entered),
                clock(left),
                (left - leg.entered) / 60,
                leg.falls[0],
                leg.falls[1],
                leg.stood_back[0],
                leg.stood_back[1],
                leg.cairns.len(),
            ));
        }
        out
    }
}

/// **Something a body's movement did that walking should not**, found on a
/// tape: what `--motion` lists. Each is a frame a person can go to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Snag {
    /// Walking at speed on the floor, free to move and still steering, and
    /// stopped dead in a frame: a lip, a seam, ground judged too steep.
    Stopped,
    /// Off the floor with no jump, and back on it within a few frames: the
    /// feet losing a slope they were walking down.
    Hopped,
    /// Rising faster than twice a jump: two takeoffs stacked into one.
    Rocketed,
}

impl Snag {
    fn says(self) -> &'static str {
        match self {
            Snag::Stopped => "stopped dead walking into something",
            Snag::Hopped => "left the floor with no jump",
            Snag::Rocketed => "rose at over twice a jump",
        }
    }
}

/// How many frames off the floor still counts as a hop rather than a fall.
const HOP: u32 = 4;

/// **How the fighters moved**: every [`Snag`] on the tape, and the frames of
/// a window as a table -- where, how fast, on the floor or not, doing what --
/// for `cargo run -p hunt --bin replay -- <file> --motion <from>[..<to>]`.
/// Written for a report like "the slopes at the back of this replay": the
/// snags say where to look, and the window says what the body did there.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Motion {
    /// The frames to print, first and last, both included.
    pub window: Option<(u32, u32)>,
    /// `(frame, who, what, where on the map, the place)`.
    pub snags: Vec<(u32, usize, Snag, sim::V3, sim::arena::ArenaId)>,
    /// The window's frames, a line each.
    pub lines: Vec<String>,
    /// For each fighter, the frame it left the floor without a jump.
    aloft: [Option<u32>; MAX_PLAYERS],
}

impl Motion {
    pub fn new(window: Option<(u32, u32)>) -> Motion {
        Motion {
            window,
            ..Motion::default()
        }
    }

    fn observe(&mut self, who: usize, before: &World, input: Input, after: &World) {
        let (was, now) = (before.players[who], after.players[who]);
        let here = now.pos.add(after.map_origin());
        let moved = |p: &sim::state::Player| sim::V3::new(p.vel.x, sim::Fx::ZERO, p.vel.z);
        // A new fight, a class changed or a place crossed moves a body
        // without it walking anywhere.
        let same = before.arena == after.arena && was.class == now.class && now.health > 0;
        if let Some((from, to)) = self.window
            && (from..=to).contains(&after.frame)
        {
            let f = |x: sim::Fx| x.to_f32_for_render();
            self.lines.push(format!(
                "  {:>7}  P{}  {:<13}  ({:8.2} {:7.2} {:8.2})  ({:6.2} {:6.2} {:6.2})  {}  {:>4x}  {:?}",
                after.frame,
                who + 1,
                after.arena.get().name,
                f(here.x),
                f(here.y),
                f(here.z),
                f(now.vel.x),
                f(now.vel.y),
                f(now.vel.z),
                if now.grounded { "floor" } else { "air  " },
                input.bits,
                now.action
            ));
        }
        if !same {
            self.aloft[who] = None;
            return;
        }
        let steering = input.bits & (Input::W | Input::A | Input::S | Input::D) != 0;
        let walking = sim::tuning::move_speed().mul(sim::Fx::ratio(9, 10));
        if steering
            && was.grounded
            && now.grounded
            && was.action.actionable()
            && now.action.actionable()
            && moved(&was).flat_len().raw() >= walking.raw()
            && moved(&now).flat_len().raw() < walking.mul(sim::Fx::ratio(1, 4)).raw()
        {
            self.snags
                .push((after.frame, who, Snag::Stopped, here, after.arena));
        }
        let jumped = input.bits & Input::SPACE != 0;
        if was.grounded && !now.grounded && !jumped && now.action.actionable() {
            self.aloft[who] = Some(after.frame);
        }
        if now.grounded {
            if let Some(left) = self.aloft[who].take()
                && after.frame - left <= HOP
            {
                self.snags
                    .push((left, who, Snag::Hopped, here, after.arena));
            }
        } else if self.aloft[who].is_some_and(|left| after.frame - left > HOP) {
            self.aloft[who] = None;
        }
        let twice = sim::tuning::jump_speed().mul(sim::Fx::from_int(2));
        if now.vel.y.raw() > twice.raw() && was.vel.y.raw() <= twice.raw() {
            self.snags
                .push((after.frame, who, Snag::Rocketed, here, after.arena));
        }
    }

    /// The snags, counted and the first of each listed, then the window.
    pub fn render(&self) -> String {
        let mut out = String::from("\nMOVEMENT\n");
        for kind in [Snag::Stopped, Snag::Hopped, Snag::Rocketed] {
            let of: Vec<_> = self.snags.iter().filter(|s| s.2 == kind).collect();
            out.push_str(&format!("  {:<30} {:>5}\n", kind.says(), of.len()));
            for (frame, who, _, at, arena) in of.iter().take(12) {
                let f = |x: sim::Fx| x.to_f32_for_render();
                out.push_str(&format!(
                    "    frame {:>7}  P{}  {:<13} ({:.2}, {:.2}, {:.2})\n",
                    frame,
                    who + 1,
                    arena.get().name,
                    f(at.x),
                    f(at.y),
                    f(at.z)
                ));
            }
            if of.len() > 12 {
                out.push_str(&format!("    ... and {} more\n", of.len() - 12));
            }
        }
        if let Some((from, to)) = self.window {
            out.push_str(&format!(
                "\n  FRAMES {from} TO {to}: where on the map, velocity, floor or air, the buttons, doing\n"
            ));
            for line in &self.lines {
                out.push_str(line);
                out.push('\n');
            }
        }
        out
    }
}

/// A replay, judged.
pub struct Judgement {
    /// The creature's report, in a hunt whose creature has a plan card -- the
    /// same report the bot's fight gets, over the person's frames.
    pub report: Option<Report>,
    /// Each fighter's hands.
    pub hands: [Hands; MAX_PLAYERS],
    /// Rounds each side won, in versus.
    pub rounds: [u8; MAX_PLAYERS],
    /// The valley, place by place, when the replay was in it.
    pub trek: Trek,
    /// The link, beside where the fighters were, when it was online.
    pub lag: crate::link::Lag,
    /// How the fighters moved: `--motion`.
    pub motion: Motion,
    /// The world the replay ended on.
    pub end: World,
    /// Whether it is the one the game ended on: `None` for a tape with no
    /// end line.
    pub matched: Option<bool>,
}

/// Play a tape and judge it. `Err` when the tape's start is not one the
/// settings rebuild.
pub fn judge(tape: &Tape) -> Result<Judgement, String> {
    judge_moving(tape, None)
}

/// [`judge`], with the frames of `window` kept as a table of how the
/// fighters moved ([`Motion`]).
pub fn judge_moving(tape: &Tape, window: Option<(u32, u32)>) -> Result<Judgement, String> {
    let species = tape.start.hunt;
    let card = species.and_then(crate::plans::card);
    let mut report = card.map(Report::new);
    let mut people: Vec<Hunter> = (0..tape.start.seats as usize)
        .map(|who| Hunter::person(who, species.unwrap_or(sim::species::SpeciesId::RIDGEBACK)))
        .collect();
    let mut hands: [Hands; MAX_PLAYERS] = Default::default();
    let mut trek = Trek::default();
    let mut motion = Motion::new(window);
    let mut lag = crate::link::Lag::new(tape.first_frame);
    // What a frame cost to simulate: the time from the end of one watch to
    // the start of the next is the copy of the world and its `advance`.
    let mut mark = std::time::Instant::now();
    let end = tape.play(|before, inputs, after| {
        lag.observe(after, mark.elapsed().as_micros() as u32);
        trek.observe(before, after);
        for person in people.iter_mut() {
            person.playback(inputs[person.who]);
        }
        for (who, hand) in hands.iter_mut().enumerate() {
            hand.observe(who, before, inputs[who], after);
            if who < (after.seats as usize).max(1) {
                motion.observe(who, before, inputs[who], after);
            }
        }
        if let Some(report) = report.as_mut() {
            report.observe(before, after, &people);
        }
        mark = std::time::Instant::now();
    })?;
    lag.join(tape);
    if let Some(report) = report.as_mut() {
        report.finish(&end);
    }
    let matched = tape.matched(&end);
    Ok(Judgement {
        report,
        hands,
        rounds: end.players.map(|p| p.rounds_won),
        trek,
        lag,
        motion,
        matched,
        end,
    })
}

impl Judgement {
    /// Everything, as text: the check, the creature's report if there is
    /// one, and each fighter's hands.
    pub fn render(&self, tape: &Tape, trace: bool) -> String {
        self.render_moving(tape, trace, false)
    }

    /// [`Judgement::render`], with how the fighters moved ([`Motion`]) at
    /// the end when `moving`.
    pub fn render_moving(&self, tape: &Tape, trace: bool, moving: bool) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "REPLAY  {}  --  {} frames from {}, build {}\n",
            tape.start.to_line(),
            tape.len(),
            tape.first_frame,
            if tape.build.is_empty() {
                "?"
            } else {
                &tape.build
            }
        ));
        match self.matched {
            Some(true) => out.push_str(
                "  reproduced bit for bit: this build reached the frame and state the game did\n",
            ),
            Some(false) => {
                out.push_str(
                    "  DOES NOT REPRODUCE: this build ended somewhere else than the game did",
                );
                if tape.tuning != sim::oven::hash() {
                    out.push_str(
                        " -- the tuning differs (the tape's Oven hash is not this build's)",
                    );
                }
                out.push_str("; the numbers below are of a fight nobody played\n");
                // **Replay it where it was recorded.** A tape is the inputs,
                // so it only plays back on the build that made it -- which
                // is a commit, and a worktree of it is a minute away.
                if !tape.build.is_empty() && tape.build != "?" {
                    out.push_str(&format!(
                        "  it was recorded at build {b}; to play it back there:\n    \
                         git worktree add ../at-{b} {b} && cd ../at-{b} && \
                         cargo run --release -p hunt --bin replay -- <this file>\n",
                        b = tape.build
                    ));
                }
            }
            None => out.push_str("  unchecked: the tape has no end line\n"),
        }
        out.push_str(&self.trek.render(self.end.frame));
        out.push_str(&self.lag.render(tape));
        if tape.start.hunt.is_none() && self.trek.legs.is_empty() {
            out.push_str(&format!(
                "  rounds: player one {}, player two {}\n",
                self.rounds[0], self.rounds[1]
            ));
        }
        if let Some(report) = &self.report {
            out.push_str(&report.render());
            if trace {
                out.push_str(&report.trace());
            }
        }
        for (who, hand) in self.hands.iter().enumerate() {
            if who < tape.start.seats as usize || hand.frames > 0 {
                out.push_str(&hand.render(who));
            }
        }
        if moving {
            out.push_str(&self.motion.render());
        }
        out
    }
}
