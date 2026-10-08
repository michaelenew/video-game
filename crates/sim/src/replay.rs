//! A replay: a fight's start, and every frame's inputs.
//!
//! The simulation is a pure function of `(state, inputs)`, so a fight is
//! completely described by where it started and what was pressed. That is a
//! few kilobytes for a minute of play, it is exact, and it is what lets a
//! fight somebody played be put back through the harness that judges the
//! scripted hunter -- the inputs of a person, measured the way the bot's are.
//! See `docs/design/replays.md`.
//!
//! Two halves. A [`Start`] is a fight as the settings that start it -- both
//! classes, the creature, its temper, the arena, how many seats are played --
//! and [`Start::world`] is **the** way a world is built from those settings:
//! the game's picker and this file build the same world because they are the
//! same function. A [`Tape`] is a start and the inputs that followed it, with
//! the start world's checksum (so a tape whose start was edited outside the
//! settings -- a capture's `SHOT_BARS`, say -- is known not to replay) and the
//! end's (so a replay can prove it reproduced the fight, bit for bit).
//!
//! **Text, one line per frame**, so a replay can be read, diffed and pasted.
//! Eight hex words a line -- bits, aim, pitch and travel for each fighter --
//! and `xN` where a line repeats, which is most of any stretch without the
//! mouse moving. A ten-minute hunt is about a megabyte; a minute of versus a
//! hundred kilobytes.
//!
//! Nothing here runs inside a frame. Recording is the game's business
//! (`crates/game`), and it records *into* a `Vec` that grows -- which is why
//! this is the one module of `sim` that allocates, and why it is read by the
//! tools and the game's menu rather than by `World::advance`.

use crate::arena::{self, ArenaId};
use crate::class::ALL_CLASSES;
use crate::input::Input;
use crate::species::{self, SpeciesId};
use crate::state::MAX_PLAYERS;
use crate::{Class, World};

/// The format's version, the first line of every tape.
pub const VERSION: u32 = 1;

/// A fight as the settings that start it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Start {
    pub classes: [Class; MAX_PLAYERS],
    /// The creature hunted, or `None` for fighting each other.
    pub hunt: Option<SpeciesId>,
    /// Where. `None` is the fight's own place: the creature's arena, or the
    /// proving ground for versus.
    pub arena: Option<ArenaId>,
    pub temper: u8,
    /// How many fighters are played: two, or one with the second seat out of
    /// any hunt ([`World::seated`]).
    pub seats: u8,
}

impl Start {
    /// **The one way a world is built from its settings.** The game's picker
    /// calls this, and so does a replay, so the two cannot drift.
    pub fn world(&self) -> World {
        let w = match (self.hunt, self.arena) {
            (Some(s), Some(a)) => World::hunt_in(self.classes, herd_of(s), a),
            (Some(s), None) => World::hunt_of(self.classes, s),
            (None, Some(a)) => World::versus_in(self.classes, a),
            (None, None) => World::with_classes(self.classes),
        };
        w.tempered(self.temper).seated(self.seats)
    }

    /// The settings a world was started from, read back off it: the inverse
    /// of [`Start::world`] for any world a start can build. A world built
    /// some other way (the Pair with a third creature) comes out as the
    /// nearest start, which is what the tape's start checksum is for.
    pub fn of(w: &World) -> Start {
        let hunt = w.hunted().into_iter().flatten().next();
        let own = match hunt {
            Some(s) => arena::for_species(s).id,
            None => ArenaId::PROVING_GROUND,
        };
        Start {
            classes: w.players.map(|p| p.class),
            hunt,
            arena: (w.arena != own).then_some(w.arena),
            temper: w.temper(),
            seats: w.seats,
        }
    }

    /// `p1=champion p2=bulwark hunt=ridgeback temper=0 seats=1`, with
    /// `arena=<slug>` when the place is not the fight's own. The names are
    /// the ones the game's flags take, so the line reads like a command.
    pub fn to_line(&self) -> String {
        let mut out = format!(
            "p1={} p2={}",
            class_slug(self.classes[0]),
            class_slug(self.classes[1])
        );
        if let Some(s) = self.hunt {
            out.push_str(&format!(" hunt={}", s.get().slug()));
        }
        if let Some(a) = self.arena {
            out.push_str(&format!(" arena={}", a.get().slug()));
        }
        out.push_str(&format!(" temper={} seats={}", self.temper, self.seats));
        out
    }

    /// The inverse of [`Start::to_line`]. Unknown words are ignored, so a
    /// tape from a build with a flag this one lacks still opens.
    pub fn parse(line: &str) -> Result<Start, String> {
        let mut start = Start {
            classes: [Class::Bulwark; MAX_PLAYERS],
            hunt: None,
            arena: None,
            temper: 0,
            seats: MAX_PLAYERS as u8,
        };
        for word in line.split_whitespace() {
            let Some((key, value)) = word.split_once('=') else {
                continue;
            };
            match key {
                "p1" | "p2" => {
                    let class = class_named(value)
                        .ok_or_else(|| format!("no class is called {value:?}"))?;
                    start.classes[usize::from(key == "p2")] = class;
                }
                "hunt" => {
                    start.hunt = Some(
                        species::named(value)
                            .ok_or_else(|| format!("no creature is called {value:?}"))?
                            .id,
                    );
                }
                "arena" => {
                    start.arena = Some(
                        arena::named(value)
                            .ok_or_else(|| format!("no arena is called {value:?}"))?
                            .id,
                    );
                }
                "temper" => start.temper = value.parse().map_err(|_| "bad temper")?,
                "seats" => start.seats = value.parse().map_err(|_| "bad seats")?,
                _ => {}
            }
        }
        Ok(start)
    }
}

/// As many of a species as its hunt holds: two for the Pair, one for the rest.
pub fn herd_of(species: SpeciesId) -> [Option<SpeciesId>; crate::monster::MAX_MONSTERS] {
    let mut herd = [None; crate::monster::MAX_MONSTERS];
    let bodies = (species.get().fight.bodies as usize).clamp(1, crate::monster::MAX_MONSTERS);
    for slot in herd.iter_mut().take(bodies) {
        *slot = Some(species);
    }
    herd
}

/// A class's name as a flag takes it: lower case, no space.
pub fn class_slug(c: Class) -> String {
    c.name().to_lowercase().replace(' ', "")
}

/// A class by its flag name, or any unambiguous prefix of one; the three
/// short names the game's flags also take.
pub fn class_named(name: &str) -> Option<Class> {
    let n = name.to_lowercase().replace([' ', '_', '-'], "");
    ALL_CLASSES
        .iter()
        .copied()
        .find(|c| class_slug(*c) == n)
        .or_else(|| match n.as_str() {
            "reaver" | "shadow" => Some(Class::ShadowReaver),
            "blood" => Some(Class::BloodMage),
            "dual" => Some(Class::DualMage),
            _ => ALL_CLASSES
                .iter()
                .copied()
                .find(|c| !n.is_empty() && class_slug(*c).starts_with(&n)),
        })
}

/// Something that changed outside the inputs, on a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// The number of played seats changed (the game's dummy keys), which
    /// takes effect at the next fresh fight.
    Seats(u8),
}

/// A fight, recorded: where it started and what was pressed.
#[derive(Clone, Debug, PartialEq)]
pub struct Tape {
    pub start: Start,
    /// The frame the start world was at. Zero for a fight from the top; a
    /// tape begun mid-session keeps the frame count it found.
    pub first_frame: u32,
    /// `World::state_checksum` of the start world, as it was in the game.
    pub start_hash: u64,
    /// The Oven's hash when it was recorded. A different tuning is a
    /// different fight from the same inputs, and this is how a replay says
    /// so rather than merely failing to match.
    pub tuning: u64,
    /// The build that recorded it.
    pub build: String,
    /// `frames[i]` took the world from `first_frame + i` to the next frame.
    pub frames: Vec<[Input; MAX_PLAYERS]>,
    /// Changes outside the inputs, by the frame they happened before.
    pub events: Vec<(u32, Event)>,
    /// The frame and `state_checksum` the fight ended on, once finished.
    pub end: Option<(u32, u64)>,
}

/// About twenty minutes at 60 Hz: what a tape holds before it grows.
const RESERVED: usize = 72_000;

impl Tape {
    /// Begin recording from this world, as it is.
    pub fn begin(w: &World, build: &str) -> Tape {
        Tape {
            start: Start::of(w),
            first_frame: w.frame,
            start_hash: w.state_checksum(),
            tuning: crate::oven::hash(),
            build: build.to_string(),
            frames: Vec::with_capacity(RESERVED),
            events: Vec::new(),
            end: None,
        }
    }

    /// Record the inputs that advance the world from `frame`. By frame rather
    /// than appended, so a rollback that re-advances a frame overwrites what
    /// it predicted with what was confirmed.
    pub fn record(&mut self, frame: u32, inputs: [Input; MAX_PLAYERS]) {
        let Some(i) = frame.checked_sub(self.first_frame) else {
            return;
        };
        let i = i as usize;
        if i < self.frames.len() {
            self.frames[i] = inputs;
        } else {
            self.frames.resize(i, [Input::default(); MAX_PLAYERS]);
            self.frames.push(inputs);
        }
        self.end = None;
    }

    /// Forget everything from `frame` on: the world was rewound to it.
    pub fn rewind_to(&mut self, frame: u32) {
        let keep = frame.saturating_sub(self.first_frame) as usize;
        self.frames.truncate(keep);
        self.events.retain(|(f, _)| (*f as usize) < keep);
        self.end = None;
    }

    /// Note a change made outside the inputs before `frame` is advanced.
    pub fn note(&mut self, frame: u32, event: Event) {
        if let Some(i) = frame.checked_sub(self.first_frame) {
            self.events.push((i, event));
        }
    }

    /// Where the fight stands now, for the end line.
    pub fn finish(&mut self, w: &World) {
        self.end = Some((w.frame, w.state_checksum()));
    }

    /// How many frames are on it.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// The start world, rebuilt from the settings and checked against the
    /// hash of the one the game had. `Err` when the two differ: the game's
    /// start was edited some way the settings do not describe.
    pub fn world(&self) -> Result<World, String> {
        let mut w = self.start.world();
        w.frame = self.first_frame;
        let have = w.state_checksum();
        if have != self.start_hash {
            return Err(format!(
                "the start world does not match: the tape says {:016x}, this build makes {:016x} from `{}`{}",
                self.start_hash,
                have,
                self.start.to_line(),
                if crate::oven::hash() != self.tuning {
                    " (the tuning differs too)"
                } else {
                    ""
                }
            ));
        }
        Ok(w)
    }

    /// Play the tape through, from its start world, showing every frame to
    /// `watch` as the world before, the inputs, and the world after. The
    /// world it ends on is returned; [`Tape::matched`] says whether it is
    /// the one the game ended on.
    pub fn play(
        &self,
        mut watch: impl FnMut(&World, [Input; MAX_PLAYERS], &World),
    ) -> Result<World, String> {
        let mut w = self.world()?;
        let mut events = self.events.iter().peekable();
        for (i, inputs) in self.frames.iter().enumerate() {
            while let Some((at, event)) = events.peek() {
                if *at as usize > i {
                    break;
                }
                match event {
                    Event::Seats(n) => w.seats = *n,
                }
                events.next();
            }
            let before = w.clone();
            w.advance(*inputs);
            watch(&before, *inputs, &w);
        }
        Ok(w)
    }

    /// Did a replay reach the frame and state the game did? `None` for a
    /// tape with no end line.
    pub fn matched(&self, end: &World) -> Option<bool> {
        self.end
            .map(|(frame, hash)| frame == end.frame && hash == end.state_checksum())
    }

    // -----------------------------------------------------------------------
    // Text
    // -----------------------------------------------------------------------

    pub fn to_text(&self) -> String {
        let mut out = String::with_capacity(self.frames.len() * 40 + 512);
        out.push_str(&format!("replay {VERSION}\n"));
        out.push_str(
            "# An arena replay: a fight's start, then one line per frame -- bits aim pitch travel,\n\
             # in hex, for each fighter; xN where a line repeats. cargo run -p hunt --bin replay <file>\n",
        );
        out.push_str(&format!("build {}\n", self.build));
        out.push_str(&format!("tuning {:016x}\n", self.tuning));
        out.push_str(&format!("start {}\n", self.start.to_line()));
        out.push_str(&format!(
            "world {} {:016x}\n",
            self.first_frame, self.start_hash
        ));
        let mut events = self.events.iter().peekable();
        let mut i = 0;
        while i < self.frames.len() {
            while let Some((at, event)) = events.peek() {
                if *at as usize > i {
                    break;
                }
                match event {
                    Event::Seats(n) => out.push_str(&format!("@{at} seats {n}\n")),
                }
                events.next();
            }
            // A run of identical frames, stopped short of the next event so
            // the event keeps its place.
            let stop = events
                .peek()
                .map_or(self.frames.len(), |(at, _)| (*at as usize).max(i + 1));
            let mut n = 1;
            while i + n < stop && self.frames[i + n] == self.frames[i] {
                n += 1;
            }
            write_frame(&mut out, &self.frames[i]);
            if n > 1 {
                out.push_str(&format!(" x{n}"));
            }
            out.push('\n');
            i += n;
        }
        for (at, event) in events {
            match event {
                Event::Seats(n) => out.push_str(&format!("@{at} seats {n}\n")),
            }
        }
        if let Some((frame, hash)) = self.end {
            out.push_str(&format!("end {frame} {hash:016x}\n"));
        }
        out
    }

    pub fn from_text(text: &str) -> Result<Tape, String> {
        let mut tape = Tape {
            start: Start::parse("")?,
            first_frame: 0,
            start_hash: 0,
            tuning: 0,
            build: String::new(),
            frames: Vec::new(),
            events: Vec::new(),
            end: None,
        };
        let mut lines = text.lines().enumerate();
        let mut versioned = false;
        for (n, line) in &mut lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let at = || format!("line {}", n + 1);
            if let Some(rest) = line.strip_prefix('@') {
                let mut words = rest.split_whitespace();
                let frame: u32 = words
                    .next()
                    .and_then(|w| w.parse().ok())
                    .ok_or_else(|| format!("{}: an event wants a frame", at()))?;
                match (
                    words.next(),
                    words.next().and_then(|w| w.parse::<u8>().ok()),
                ) {
                    (Some("seats"), Some(n)) => tape.events.push((frame, Event::Seats(n))),
                    _ => return Err(format!("{}: unknown event {rest:?}", at())),
                }
                continue;
            }
            let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
            match word {
                "replay" => {
                    let v: u32 = rest
                        .trim()
                        .parse()
                        .map_err(|_| format!("{}: bad version", at()))?;
                    if v != VERSION {
                        return Err(format!("replay format {v}; this build reads {VERSION}"));
                    }
                    versioned = true;
                }
                "build" => tape.build = rest.trim().to_string(),
                "tuning" => {
                    tape.tuning = hex(rest.trim()).ok_or_else(|| format!("{}: bad tuning", at()))?
                }
                "start" => tape.start = Start::parse(rest).map_err(|e| format!("{}: {e}", at()))?,
                "world" => {
                    let mut words = rest.split_whitespace();
                    tape.first_frame = words
                        .next()
                        .and_then(|w| w.parse().ok())
                        .ok_or_else(|| format!("{}: bad start frame", at()))?;
                    tape.start_hash = words
                        .next()
                        .and_then(hex)
                        .ok_or_else(|| format!("{}: bad start hash", at()))?;
                }
                "end" => {
                    let mut words = rest.split_whitespace();
                    let frame = words
                        .next()
                        .and_then(|w| w.parse().ok())
                        .ok_or_else(|| format!("{}: bad end frame", at()))?;
                    let hash = words
                        .next()
                        .and_then(hex)
                        .ok_or_else(|| format!("{}: bad end hash", at()))?;
                    tape.end = Some((frame, hash));
                }
                _ => {
                    if !versioned {
                        return Err("not a replay: no `replay <version>` line first".to_string());
                    }
                    let (inputs, repeat) =
                        read_frame(line).ok_or_else(|| format!("{}: bad frame", at()))?;
                    for _ in 0..repeat {
                        tape.frames.push(inputs);
                    }
                }
            }
        }
        if !versioned {
            return Err("not a replay: no `replay <version>` line".to_string());
        }
        tape.events.sort_by_key(|(f, _)| *f);
        Ok(tape)
    }
}

fn write_frame(out: &mut String, inputs: &[Input; MAX_PLAYERS]) {
    for (i, input) in inputs.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&format!(
            "{:04x} {:04x} {:04x} {:02x}",
            input.bits, input.aim, input.pitch as u16, input.travel.0
        ));
    }
}

fn read_frame(line: &str) -> Option<([Input; MAX_PLAYERS], u32)> {
    let mut words = line.split_whitespace();
    let mut inputs = [Input::default(); MAX_PLAYERS];
    for input in inputs.iter_mut() {
        input.bits = u16::from_str_radix(words.next()?, 16).ok()?;
        input.aim = u16::from_str_radix(words.next()?, 16).ok()?;
        input.pitch = u16::from_str_radix(words.next()?, 16).ok()? as i16;
        input.travel = crate::input::Travel(u8::from_str_radix(words.next()?, 16).ok()?);
    }
    let repeat = match words.next() {
        None => 1,
        Some(w) => w.strip_prefix('x')?.parse().ok()?,
    };
    Some((inputs, repeat))
}

fn hex(word: &str) -> Option<u64> {
    u64::from_str_radix(word, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn script(frames: u32, seed: u64) -> Vec<[Input; MAX_PLAYERS]> {
        let mut rng = seed;
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
                    Input::aimed((next() & 0x1ff) as u16, next() as u16),
                ]
            })
            .collect()
    }

    #[test]
    fn a_tape_round_trips_through_its_text() {
        let w = World::hunt_of([Class::Champion, Class::Bulwark], SpeciesId::RIDGEBACK)
            .tempered(2)
            .seated(1);
        let mut tape = Tape::begin(&w, "test");
        let mut w = w;
        for (i, inputs) in script(300, 7).into_iter().enumerate() {
            if i == 100 {
                tape.note(w.frame, Event::Seats(2));
                w.seats = 2;
            }
            tape.record(w.frame, inputs);
            w.advance(inputs);
        }
        tape.finish(&w);
        let text = tape.to_text();
        let back = Tape::from_text(&text).expect("parses");
        assert_eq!(back, tape);
    }

    #[test]
    fn a_replay_reproduces_the_fight() {
        let w = World::hunt_of([Class::Champion, Class::Bulwark], SpeciesId::RIDGEBACK).seated(1);
        let mut tape = Tape::begin(&w, "test");
        let mut w = w;
        for inputs in script(600, 99) {
            tape.record(w.frame, inputs);
            w.advance(inputs);
        }
        tape.finish(&w);
        let text = tape.to_text();
        let back = Tape::from_text(&text).expect("parses");
        let end = back
            .play(|_, _, _| {})
            .expect("the start is one a start describes");
        assert_eq!(back.matched(&end), Some(true));
        assert_eq!(end, w);
    }

    #[test]
    fn repeats_are_collapsed_and_expanded() {
        let w = World::new();
        let mut tape = Tape::begin(&w, "test");
        let same = [Input::aimed(Input::W, 100), Input::default()];
        for f in 0..50 {
            tape.record(f, same);
        }
        tape.record(50, [Input::default(); MAX_PLAYERS]);
        let text = tape.to_text();
        assert!(text.contains(" x50\n"), "{text}");
        assert_eq!(Tape::from_text(&text).expect("parses").frames.len(), 51);
    }

    #[test]
    fn a_rewind_forgets_the_frames_after_it() {
        let w = World::new();
        let mut tape = Tape::begin(&w, "test");
        for f in 0..40 {
            tape.record(f, [Input::default(); MAX_PLAYERS]);
        }
        tape.note(30, Event::Seats(1));
        tape.rewind_to(25);
        assert_eq!(tape.len(), 25);
        assert!(tape.events.is_empty());
    }

    #[test]
    fn a_frame_advanced_again_is_overwritten_not_appended() {
        let w = World::new();
        let mut tape = Tape::begin(&w, "test");
        for f in 0..10 {
            tape.record(f, [Input::default(); MAX_PLAYERS]);
        }
        let corrected = [Input::aimed(Input::LEFT, 5), Input::default()];
        tape.record(6, corrected);
        assert_eq!(tape.len(), 10);
        assert_eq!(tape.frames[6], corrected);
    }

    #[test]
    fn the_start_reads_back_off_every_world_a_start_builds() {
        for sp in species::all() {
            for seats in 1..=2u8 {
                let start = Start {
                    classes: [Class::DualMage, Class::ShadowReaver],
                    hunt: Some(sp.id),
                    arena: None,
                    temper: 1,
                    seats,
                };
                let w = start.world();
                assert_eq!(Start::of(&w), start, "{}", sp.name);
                assert_eq!(Start::parse(&start.to_line()), Ok(start));
            }
        }
        for a in arena::all() {
            let start = Start {
                classes: [Class::Bulwark, Class::Elementalist],
                hunt: None,
                arena: (a.id != ArenaId::PROVING_GROUND).then_some(a.id),
                temper: 0,
                seats: 2,
            };
            let w = start.world();
            assert_eq!(Start::of(&w), start, "{}", a.name);
            assert_eq!(Start::parse(&start.to_line()), Ok(start));
        }
    }

    #[test]
    fn an_edited_start_is_refused_with_a_reason() {
        let mut w = World::new();
        w.players[0].health = 1;
        let tape = Tape::begin(&w, "test");
        let err = tape
            .world()
            .expect_err("a start the settings cannot describe");
        assert!(err.contains("does not match"), "{err}");
    }

    #[test]
    fn class_names_read_the_way_the_flags_take_them() {
        assert_eq!(class_named("champion"), Some(Class::Champion));
        assert_eq!(class_named("Shadow Reaver"), Some(Class::ShadowReaver));
        assert_eq!(class_named("shadowreaver"), Some(Class::ShadowReaver));
        assert_eq!(class_named("reaver"), Some(Class::ShadowReaver));
        assert_eq!(class_named("blood"), Some(Class::BloodMage));
        assert_eq!(class_named("ele"), Some(Class::Elementalist));
        assert_eq!(class_named("nobody"), None);
    }
}
