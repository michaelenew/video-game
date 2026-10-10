//! **The link, laid beside the fight**: where a match against a person got
//! laggy, and why (`docs/design/replays.md` §6).
//!
//! A tape saved online carries one [`Link`] a second, as the machine that
//! saved it saw the connection: the ping, the rollbacks, the ticks spent
//! waiting for the friend, the slowest frame it drew. None of that says
//! *where* the two of you were; the replay does, because the replay rebuilds
//! the fight. So the harness plays the tape, notes for every frame how far
//! apart the fighters stood and what it cost to simulate here, and puts the
//! two side by side: the rough stretches, each with its likeliest cause, and
//! the whole match cut by distance apart.
//!
//! Lag has three places it can come from, and they leave different marks:
//!
//! - **the line** -- the round trip is long: a high ping, deep rollbacks,
//!   and stalls when the friend's inputs run out of time to arrive;
//! - **the friend's machine** -- it fell behind (a slow frame there, a page
//!   loading a place): this one stalls waiting while its own frames are fast
//!   and the ping is ordinary;
//! - **this machine** -- a frame took too long to draw: the slowest frame
//!   column, whatever the line was doing.
//!
//! Each machine's tape is its own side of the line. Both saving the same
//! match is what tells the second from the third for certain.

use sim::arena::ArenaId;
use sim::replay::{Link, Tape};
use sim::state::MAX_PLAYERS;
use sim::{Fx, V3, World};

/// A frame drawn in more than this is a hitch you feel: three frames' time.
const SLOW_FRAME_MS: u32 = 50;
/// A round trip this long is a line you feel, even with rollback hiding it.
const HIGH_PING_MS: u32 = 150;
/// The furthest a rollback reaches before the session stalls instead
/// (`net::MAX_ROLLBACK_FRAMES`; this crate does not depend on `net`).
const ROLLBACK_LIMIT: u32 = 8;
/// Rollbacks this deep, of the session's eight, make the far fighter jump.
const DEEP_ROLLBACK: u32 = 6;
/// Distances apart the match is cut by, in metres: close, within a fight,
/// across a place, in different places.
const APART: [f32; 4] = [0.0, 10.0, 40.0, 150.0];

/// What the replay knew about one frame that the tape's link did not.
#[derive(Clone, Copy, Debug)]
struct Frame {
    /// Metres between the two fighters on the ground, when both are up.
    apart: Option<f32>,
    place: ArenaId,
    /// Microseconds this frame took to simulate, here.
    cost_us: u32,
}

/// One second of the link, with what the replay says about it.
#[derive(Clone, Copy, Debug)]
pub struct Second {
    pub link: Link,
    /// The furthest apart the two fighters were in the second.
    pub apart: Option<f32>,
    pub place: ArenaId,
    /// The most one frame of it cost to simulate on the machine judging it.
    pub cost_us: u32,
}

impl Second {
    /// Why this second was rough, or `None` if it was not.
    fn rough(&self) -> Option<&'static str> {
        let l = &self.link;
        if l.quiet {
            Some("the friend's game went silent")
        } else if l.slowest > SLOW_FRAME_MS {
            Some("this machine: a frame took too long to draw")
        } else if l.stalls > 0 && l.ping > HIGH_PING_MS {
            Some("the line: a long round trip, and the game waited on it")
        } else if l.stalls > 0 {
            Some("waiting on the friend's machine: the line and this one were fine")
        } else if l.ping > HIGH_PING_MS {
            Some("the line: a long round trip")
        } else if l.deepest >= DEEP_ROLLBACK {
            Some("deep rollbacks: guesses about the friend wrong for a long time")
        } else {
            None
        }
    }
}

/// The match's link, second by second, beside the fight.
#[derive(Clone, Debug, Default)]
pub struct Lag {
    frames: Vec<Frame>,
    first: u32,
    /// Filled by [`Lag::join`]: the tape's link seconds.
    pub seconds: Vec<Second>,
}

impl Lag {
    pub fn new(first_frame: u32) -> Lag {
        Lag {
            frames: Vec::new(),
            first: first_frame,
            seconds: Vec::new(),
        }
    }

    /// One frame of the replay, and what it cost to simulate.
    pub fn observe(&mut self, after: &World, cost_us: u32) {
        let seats = (after.seats as usize).clamp(1, MAX_PLAYERS);
        let [a, b] = [after.players[0], after.players[1]];
        let apart = (seats == MAX_PLAYERS && a.health > 0 && b.health > 0).then(|| {
            let d = a.pos.sub(b.pos);
            sim::math::wide_len(V3::new(d.x, Fx::ZERO, d.z)).to_f32_for_render()
        });
        // The first frame's time is the replay setting up, not a frame.
        let cost_us = if self.frames.is_empty() { 0 } else { cost_us };
        self.frames.push(Frame {
            apart,
            place: after.arena,
            cost_us,
        });
    }

    /// Put the tape's link beside the frames the replay saw.
    pub fn join(&mut self, tape: &Tape) {
        let mut from = 0usize;
        self.seconds = tape
            .link
            .iter()
            .map(|link| {
                let to = (link.frame.saturating_sub(self.first) as usize).min(self.frames.len());
                let span = &self.frames[from.min(to)..to];
                from = to;
                Second {
                    link: *link,
                    apart: span.iter().filter_map(|f| f.apart).reduce(f32::max),
                    place: span
                        .last()
                        .or(self.frames.last())
                        .map_or(ArenaId::PROVING_GROUND, |f| f.place),
                    cost_us: span.iter().map(|f| f.cost_us).max().unwrap_or(0),
                }
            })
            .collect();
    }

    /// The section: the link at a glance, the rough stretches, and the match
    /// cut by distance apart. Empty for a fight with one fighter and no link.
    pub fn render(&self, tape: &Tape) -> String {
        let two = self.frames.iter().any(|f| f.apart.is_some());
        if self.seconds.is_empty() && !two {
            return String::new();
        }
        let clock = |frame: u32| {
            let s = frame.saturating_sub(self.first) / sim::TICK_HZ;
            format!("{}:{:02}", s / 60, s % 60)
        };
        let mut out = String::new();
        let Some((seat, delay)) = tape.online.filter(|_| !self.seconds.is_empty()) else {
            out.push_str(
                "\nTHE LINK  not on this tape: it was not a match against a person, or the build\n  \
                 that saved it did not keep the link yet. What the replay can still say is what\n  \
                 the simulation costs, here, at each distance apart:\n",
            );
            out.push_str(&self.by_distance(false));
            return out;
        };
        let s = &self.seconds;
        let n = s.len() as u32;
        let mut pings: Vec<u32> = s.iter().map(|x| x.link.ping).collect();
        pings.sort_unstable();
        let worst_ping = s.iter().max_by_key(|x| x.link.ping).expect("a second");
        let worst_frame = s.iter().max_by_key(|x| x.link.slowest).expect("a second");
        let stalls: u32 = s.iter().map(|x| x.link.stalls).sum();
        let rollbacks: u32 = s.iter().map(|x| x.link.rollbacks).sum();
        let resim: u32 = s.iter().map(|x| x.link.resimulated).sum();
        let at_limit = s
            .iter()
            .filter(|x| x.link.deepest >= ROLLBACK_LIMIT)
            .count();
        out.push_str(&format!(
            "\nTHE LINK  as player {}'s machine saw it, input delay {delay}  --  {}:{:02} online\n",
            seat + 1,
            n / 60,
            n % 60
        ));
        out.push_str(&format!(
            "  ping            {} ms typical, {} ms at worst ({})\n",
            pings[pings.len() / 2],
            worst_ping.link.ping,
            clock(worst_ping.link.frame)
        ));
        out.push_str(&format!(
            "  rollbacks       {rollbacks}, {:.1} frames replayed each; {at_limit} seconds reached the limit of eight\n",
            if rollbacks == 0 {
                0.0
            } else {
                resim as f32 / rollbacks as f32
            }
        ));
        out.push_str(&format!(
            "  waiting         {stalls} ticks stood still waiting on the friend ({:.1} s)\n",
            stalls as f32 / sim::TICK_HZ as f32
        ));
        out.push_str(&format!(
            "  slowest frame   {} ms ({})\n",
            worst_frame.link.slowest,
            clock(worst_frame.link.frame)
        ));

        // The rough stretches: runs of rough seconds, a second's grace
        // between them, each named by the cause most of its seconds share.
        let mut stretches: Vec<(usize, usize)> = Vec::new();
        for (i, sec) in s.iter().enumerate() {
            if sec.rough().is_none() {
                continue;
            }
            match stretches.last_mut() {
                Some((_, end)) if i <= *end + 2 => *end = i,
                _ => stretches.push((i, i)),
            }
        }
        if stretches.is_empty() {
            out.push_str(
                "  no rough stretches: nothing stalled, hitched or went past the thresholds\n",
            );
        } else {
            let rough = s.iter().filter(|x| x.rough().is_some()).count();
            out.push_str(&format!(
                "\n  ROUGH STRETCHES  {} of them, {rough} s in all\n",
                stretches.len()
            ));
            out.push_str(
                "  WHEN          PING  WAITED  DEEPEST  SLOWEST  APART  WHERE            LIKELY\n",
            );
            let mut shown: Vec<_> = stretches.iter().collect();
            // The worst first, by ticks waited and then the slowest frame.
            shown.sort_by_key(|(a, b)| {
                let span = &s[*a..=*b];
                std::cmp::Reverse((
                    span.iter().map(|x| x.link.stalls).sum::<u32>(),
                    span.iter().map(|x| x.link.slowest).max().unwrap_or(0),
                ))
            });
            for (a, b) in shown.iter().take(12) {
                let span = &s[*a..=*b];
                let mut causes: Vec<(&str, usize)> = Vec::new();
                for why in span.iter().filter_map(Second::rough) {
                    match causes.iter_mut().find(|(c, _)| *c == why) {
                        Some((_, k)) => *k += 1,
                        None => causes.push((why, 1)),
                    }
                }
                let likely = causes
                    .iter()
                    .max_by_key(|(_, k)| *k)
                    .map_or("", |(c, _)| *c);
                let start = if *a == 0 {
                    self.first
                } else {
                    s[a - 1].link.frame
                };
                out.push_str(&format!(
                    "  {:>5}-{:<5}  {:>4}  {:>6}  {:>7}  {:>5}ms  {:>5}  {:<15}  {likely}\n",
                    clock(start),
                    clock(span[span.len() - 1].link.frame),
                    span.iter().map(|x| x.link.ping).max().unwrap_or(0),
                    span.iter().map(|x| x.link.stalls).sum::<u32>(),
                    span.iter().map(|x| x.link.deepest).max().unwrap_or(0),
                    span.iter().map(|x| x.link.slowest).max().unwrap_or(0),
                    metres(span.iter().filter_map(|x| x.apart).reduce(f32::max)),
                    span[span.len() - 1].place.get().name,
                ));
            }
            if stretches.len() > 12 {
                out.push_str(&format!("  … and {} more\n", stretches.len() - 12));
            }
        }
        out.push_str(&self.by_distance(true));
        out
    }

    /// The match cut by how far apart the fighters were: does the link, or
    /// this machine, or the simulation, get worse with distance?
    fn by_distance(&self, linked: bool) -> String {
        let band_of = |apart: f32| APART.iter().rposition(|&m| apart >= m).unwrap_or(0);
        let mut bands = [Band::default(); APART.len()];
        if linked {
            for sec in &self.seconds {
                let Some(apart) = sec.apart else { continue };
                let b = &mut bands[band_of(apart)];
                b.seconds += 1;
                b.rough += u32::from(sec.rough().is_some());
                b.ping += u64::from(sec.link.ping);
                b.stalls += sec.link.stalls;
                b.slowest = b.slowest.max(sec.link.slowest);
                b.rollbacks += sec.link.rollbacks;
                b.resim += sec.link.resimulated;
                b.cost_us = b.cost_us.max(sec.cost_us);
            }
        } else {
            for chunk in self.frames.chunks(sim::TICK_HZ as usize) {
                let Some(apart) = chunk.iter().filter_map(|f| f.apart).reduce(f32::max) else {
                    continue;
                };
                let b = &mut bands[band_of(apart)];
                b.seconds += 1;
                b.cost_us = b
                    .cost_us
                    .max(chunk.iter().map(|f| f.cost_us).max().unwrap_or(0));
            }
        }
        let mut out = String::from("\n  BY DISTANCE APART\n");
        out.push_str(if linked {
            "  APART       TIME   ROUGH   PING  WAITED/MIN  SLOWEST  ROLLBACK  SIM HERE\n"
        } else {
            "  APART       TIME   SIM HERE (the most one frame cost)\n"
        });
        for (i, b) in bands.iter().enumerate() {
            let label = match APART.get(i + 1) {
                Some(to) => format!("{}-{} m", APART[i], to),
                None => format!("{}+ m", APART[i]),
            };
            if b.seconds == 0 {
                out.push_str(&format!("  {label:<9}      -\n"));
                continue;
            }
            let time = format!("{}:{:02}", b.seconds / 60, b.seconds % 60);
            if linked {
                out.push_str(&format!(
                    "  {label:<9}  {time:>5}   {:>4.0}%  {:>4}  {:>10.1}  {:>5}ms  {:>8.1}  {:>6}us\n",
                    b.rough as f32 * 100.0 / b.seconds as f32,
                    b.ping / u64::from(b.seconds),
                    b.stalls as f32 * 60.0 / b.seconds as f32,
                    b.slowest,
                    if b.rollbacks == 0 {
                        0.0
                    } else {
                        b.resim as f32 / b.rollbacks as f32
                    },
                    b.cost_us
                ));
            } else {
                out.push_str(&format!("  {label:<9}  {time:>5}   {:>6}us\n", b.cost_us));
            }
        }
        out.push_str(&format!(
            "  (SIM HERE is this machine, now, without rollback: a rollback plays up to eight\n  \
             frames inside one, so eight times it has to fit in a {:.1} ms frame.)\n",
            1000.0 / sim::TICK_HZ as f32
        ));
        if linked {
            out.push_str(&reading(&bands));
        }
        out
    }
}

/// The seconds a distance apart, added up.
#[derive(Default, Clone, Copy)]
struct Band {
    seconds: u32,
    rough: u32,
    ping: u64,
    stalls: u32,
    slowest: u32,
    rollbacks: u32,
    resim: u32,
    cost_us: u32,
}

impl Band {
    fn and(self, b: Band) -> Band {
        Band {
            seconds: self.seconds + b.seconds,
            rough: self.rough + b.rough,
            ping: self.ping + b.ping,
            stalls: self.stalls + b.stalls,
            slowest: self.slowest.max(b.slowest),
            rollbacks: self.rollbacks + b.rollbacks,
            resim: self.resim + b.resim,
            cost_us: self.cost_us.max(b.cost_us),
        }
    }
    fn rough(&self) -> f32 {
        self.rough as f32 / self.seconds as f32
    }
    fn ping(&self) -> f32 {
        self.ping as f32 / self.seconds as f32
    }
    fn waits(&self) -> f32 {
        self.stalls as f32 * 60.0 / self.seconds as f32
    }
}

/// One plain sentence on what the cut by distance shows, when it shows
/// something: near against far, and which column moved.
fn reading(bands: &[Band; APART.len()]) -> String {
    // Near is within a fight (under forty metres), far is past it.
    let sum = |r: std::ops::Range<usize>| bands[r].iter().fold(Band::default(), |a, b| a.and(*b));
    let (near, far) = (sum(0..2), sum(2..APART.len()));
    if near.seconds < 10 || far.seconds < 10 {
        return String::new();
    }
    if far.rough() <= near.rough() * 1.5 + 0.02 {
        return "  Far apart was no rougher than close together.\n".to_string();
    }
    let mut out = format!(
        "Far apart (over 40 m) was rougher: {:.0}% of seconds against {:.0}% close.",
        far.rough() * 100.0,
        near.rough() * 100.0
    );
    if far.ping() > near.ping() * 1.3 + 10.0 {
        out.push_str(&format!(
            " The ping rose with it ({:.0} ms against {:.0}), so the line itself was worse then.",
            far.ping(),
            near.ping()
        ));
    } else if far.slowest > SLOW_FRAME_MS && far.slowest > near.slowest * 2 {
        out.push_str(&format!(
            " The ping did not move ({:.0} ms against {:.0}) but this machine's slowest frame did \
             ({} ms against {}): it is drawing, not the network.",
            far.ping(),
            near.ping(),
            far.slowest,
            near.slowest
        ));
    } else if far.waits() > near.waits() * 1.5 {
        out.push_str(&format!(
            " The ping did not move ({:.0} ms against {:.0}) and this machine kept up, but it \
             waited on the friend more ({:.0} ticks a minute against {:.0}): their machine was \
             falling behind. Their tape of the same match will show it.",
            far.ping(),
            near.ping(),
            far.waits(),
            near.waits()
        ));
    }
    wrap(&out)
}

/// A sentence folded to the width of the tables above it.
fn wrap(text: &str) -> String {
    let mut out = String::new();
    let mut line = String::from(" ");
    for word in text.split_whitespace() {
        if line.len() + 1 + word.len() > 90 {
            out.push_str(&line);
            out.push('\n');
            line = String::from(" ");
        }
        line.push(' ');
        line.push_str(word);
    }
    out.push_str(&line);
    out.push('\n');
    out
}

fn metres(m: Option<f32>) -> String {
    m.map_or("-".to_string(), |m| format!("{m:.0}m"))
}
