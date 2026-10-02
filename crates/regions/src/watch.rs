//! The watchdog: the region logic's promises, checked on every confirmed
//! frame in dev mode, and a short memory of every one found broken.
//!
//! `docs/design/regions.md` §"The watchdog". In order of how cheap they are:
//!
//! 1. **Sound**: the grid keeps every body in at most three regions.
//! 2. **Crowded**: a body is in one to three regions and one to three zones.
//! 3. **Too fast**: no body moved further in one frame than the declared speed.
//! 4. **Outran**: one live region a frame, in turn, run again from the tape
//!    with every fighter who was not a member pressing nothing. If its
//!    checksum differs from the real one, something outside reached in faster
//!    than the declared reach and speed allow.
//! 5. **Held** (reported by the ledger): a frame the gate held, and for how
//!    long.

use crate::{Live, Tape};
use sim::input::Input;
use sim::math::V3;
use sim::region::{Body, Grid, RegionId};
use sim::state::{MAX_PLAYERS, Phase};
use sim::{Fx, TICK_HZ, World};

/// How many issues the watchdog remembers: the latest, newest first.
pub const MEMORY: usize = 16;

/// What was found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finding {
    /// `h + overlap` is not under half a hexagon: a body can be in four.
    Unsound { membership: Fx, size: Fx },
    /// A body in too many (or no) regions or zones.
    Crowded { body: Body, regions: u8, zones: u8 },
    /// A body that moved further in a frame than the declared speed allows.
    TooFast { body: Body, moved: Fx, allowed: Fx },
    /// A region whose checksum came out differently with only its members'
    /// inputs: something outside reached in. `body` is the first body in its
    /// zone that differs, or none when what differs is the state that belongs
    /// to no body (the home region's phase, pack or lore).
    Outran {
        region: RegionId,
        body: Option<Body>,
    },
    /// The gate held for this many ticks, waiting on a region's heartbeat.
    Held { region: RegionId, ticks: u32 },
}

impl Finding {
    /// The kinds, by name, in the order [`Watch::counts`] keeps them.
    pub const KINDS: [&'static str; 5] = ["unsound", "crowded", "too fast", "outran", "held"];

    pub const fn kind(self) -> usize {
        match self {
            Finding::Unsound { .. } => 0,
            Finding::Crowded { .. } => 1,
            Finding::TooFast { .. } => 2,
            Finding::Outran { .. } => 3,
            Finding::Held { .. } => 4,
        }
    }
}

/// One finding, when it was last seen, and how many times in a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Issue {
    pub frame: u32,
    pub finding: Finding,
    pub repeats: u32,
    /// Counts up across every issue ever recorded, so a reader can tell which
    /// are new since it last looked.
    pub seq: u32,
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = |v: Fx| v.raw() as f64 / 65536.0;
        write!(f, "frame {}: ", self.frame)?;
        match self.finding {
            Finding::Unsound { membership, size } => write!(
                f,
                "grid unsound: reach + speed x delay + overlap = {:.1} m, half a hexagon is {:.1} m",
                m(membership),
                m(size) / 2.0
            )?,
            Finding::Crowded {
                body,
                regions,
                zones,
            } => write!(f, "{body:?} in {regions} regions, {zones} zones")?,
            Finding::TooFast {
                body,
                moved,
                allowed,
            } => write!(
                f,
                "{body:?} moved {:.2} m in a frame, the speed allows {:.2}",
                m(moved),
                m(allowed)
            )?,
            Finding::Outran { region, body } => match body {
                Some(b) => write!(
                    f,
                    "region ({}, {}) outrun: {b:?} differs without outsiders' inputs",
                    region.q, region.r
                )?,
                None => write!(
                    f,
                    "region ({}, {}) outrun: its global state differs without outsiders' inputs",
                    region.q, region.r
                )?,
            },
            Finding::Held { region, ticks } => write!(
                f,
                "held {ticks} ticks waiting on region ({}, {})",
                region.q, region.r
            )?,
        }
        if self.repeats > 1 {
            write!(f, " (x{})", self.repeats)?;
        }
        Ok(())
    }
}

/// The most bodies a world has: every slot of every kind.
const BODIES: usize = 160;

/// The watchdog's state between frames.
pub struct Watch {
    issues: [Option<Issue>; MEMORY],
    /// Index of the newest issue in `issues`.
    newest: usize,
    seq: u32,
    /// Where every body stood at the frame before, for the speed check.
    before: [(u32, V3); BODIES],
    before_len: usize,
    before_frame: Option<u32>,
    before_phase: Option<Phase>,
    /// The grid last judged, so an unsound grid is one issue, not one a frame.
    judged: Option<Grid>,
    /// Which live region the locality check runs next.
    turn: usize,
    /// How many locality checks have run, for the HUD: a test that never runs
    /// passes very convincingly.
    pub reruns: u32,
    /// Every finding ever recorded, by kind, in [`Finding::KINDS`] order: the
    /// memory holds sixteen lines, and a fight that breaks one promise a frame
    /// would push everything else out of it.
    pub counts: [u32; Finding::KINDS.len()],
}

impl Default for Watch {
    fn default() -> Watch {
        Watch::new()
    }
}

impl Watch {
    pub fn new() -> Watch {
        Watch {
            issues: [None; MEMORY],
            newest: 0,
            seq: 0,
            before: [(0, V3::ZERO); BODIES],
            before_len: 0,
            before_frame: None,
            before_phase: None,
            judged: None,
            turn: 0,
            reruns: 0,
            counts: [0; Finding::KINDS.len()],
        }
    }

    /// The remembered issues, newest first.
    pub fn issues(&self) -> impl Iterator<Item = &Issue> {
        (0..MEMORY).filter_map(move |k| self.issues[(self.newest + MEMORY - k) % MEMORY].as_ref())
    }

    /// How many issues have ever been recorded.
    pub fn seq(&self) -> u32 {
        self.seq
    }

    /// Record a finding. The same finding as the newest one is a repeat of it.
    pub fn report(&mut self, frame: u32, finding: Finding) {
        self.counts[finding.kind()] += 1;
        if let Some(last) = self.issues[self.newest].as_mut() {
            if same(last.finding, finding) {
                last.frame = frame;
                last.finding = finding;
                last.repeats += 1;
                self.seq += 1;
                last.seq = self.seq;
                return;
            }
        }
        self.seq += 1;
        self.newest = (self.newest + 1) % MEMORY;
        self.issues[self.newest] = Some(Issue {
            frame,
            finding,
            repeats: 1,
            seq: self.seq,
        });
    }

    /// Start again from nothing remembered about the frame before: a fresh
    /// fight, or a grid that changed under the books. Issues are kept.
    pub fn forget(&mut self) {
        self.before_len = 0;
        self.before_frame = None;
        self.before_phase = None;
        self.judged = None;
    }

    /// Every check, for one confirmed frame.
    pub fn confirm(&mut self, grid: &Grid, world: &World, tape: &Tape, live: &[Live]) {
        let frame = world.frame;
        if self.judged != Some(*grid) {
            self.judged = Some(*grid);
            if !grid.sound() {
                self.report(
                    frame,
                    Finding::Unsound {
                        membership: grid.membership(),
                        size: grid.size,
                    },
                );
            }
        }

        // Crowded, and the speed check against the frame before.
        let continuous = self.before_frame == Some(frame.wrapping_sub(1))
            && self.before_phase == Some(world.phase)
            && tape.get(frame).is_none_or(|e| !e.travelled());
        let allowed = grid.speed.div(Fx::from_int(TICK_HZ as i32));
        let mut now = [(0u32, V3::ZERO); BODIES];
        let mut now_len = 0;
        let mut findings = [None; 8];
        let mut found = 0;
        each_moving(world, |body, at, fresh| {
            let regions = grid.regions_of(at).len() as u8;
            let zones = grid.zones_of(at).len() as u8;
            let crowded = !(1..=3).contains(&regions) || !(1..=3).contains(&zones);
            if crowded && found < findings.len() {
                findings[found] = Some(Finding::Crowded {
                    body,
                    regions,
                    zones,
                });
                found += 1;
            }
            if continuous && !fresh {
                let was = self.before[..self.before_len]
                    .iter()
                    .find(|(w, _)| *w == body.word());
                if let Some(&(_, then)) = was {
                    let moved = at.sub(then).len();
                    if moved.raw() > allowed.raw() && found < findings.len() {
                        findings[found] = Some(Finding::TooFast {
                            body,
                            moved,
                            allowed,
                        });
                        found += 1;
                    }
                }
            }
            if now_len < BODIES {
                now[now_len] = (body.word(), at);
                now_len += 1;
            }
        });
        for f in findings.into_iter().flatten() {
            self.report(frame, f);
        }
        self.before = now;
        self.before_len = now_len;
        self.before_frame = Some(frame);
        self.before_phase = Some(world.phase);

        // Locality: one live region this frame.
        if !live.is_empty() {
            self.turn = (self.turn + 1) % live.len();
            let region = live[self.turn].region;
            if let Some(outcome) = rerun(grid, world, tape, region) {
                self.reruns += 1;
                if let Some(finding) = outcome {
                    self.report(frame, finding);
                }
            }
        }
    }
}

/// Two findings that are the same problem: same kind, same body or region.
fn same(a: Finding, b: Finding) -> bool {
    match (a, b) {
        (Finding::Unsound { .. }, Finding::Unsound { .. }) => true,
        (Finding::Crowded { body: x, .. }, Finding::Crowded { body: y, .. }) => x == y,
        (Finding::TooFast { body: x, .. }, Finding::TooFast { body: y, .. }) => x == y,
        (Finding::Outran { region: x, body: a }, Finding::Outran { region: y, body: b }) => {
            x == y && a == b
        }
        (Finding::Held { region: x, .. }, Finding::Held { region: y, .. }) => x == y,
        _ => false,
    }
}

/// Every body, where it stands, and whether it is new this frame -- a bolt
/// just thrown or an effect just placed appears where it appears, and that is
/// not a speed.
fn each_moving(world: &World, mut visit: impl FnMut(Body, V3, bool)) {
    for (i, p) in world.players.iter().enumerate() {
        visit(Body::Fighter(i as u8), p.pos, false);
    }
    for (i, m) in world.monsters.iter().enumerate() {
        if let Some(m) = m {
            visit(Body::Creature(i as u8), m.pos, false);
        }
    }
    for (i, c) in world.critters.iter().enumerate() {
        if c.present() {
            visit(Body::Critter(i as u8), c.pos, false);
        }
    }
    for (i, b) in world.bolts.iter().enumerate() {
        if let Some(b) = b {
            visit(Body::Bolt(i as u8), b.pos, b.travelled == Fx::ZERO);
        }
    }
    for (i, d) in world.debris.iter().enumerate() {
        if let Some(d) = d {
            visit(Body::Shard(i as u8), d.pos, d.travelled == Fx::ZERO);
        }
    }
    for (i, g) in world.gusts.iter().enumerate() {
        if let Some(g) = g {
            visit(Body::Gust(i as u8), g.pos, g.travelled == Fx::ZERO);
        }
    }
    for (i, e) in world.effects.iter().enumerate() {
        if let Some(e) = e {
            visit(Body::Effect(i as u8), e.pos, e.age == 0);
        }
    }
}

/// The locality check for one region: run the last `delay` frames again from
/// the tape with only its members' inputs, and compare its checksum with the
/// real one. `None` if the tape cannot run it (too short, or the window holds a
/// trip to a fresh fight); `Some(None)` if they agree.
fn rerun(grid: &Grid, world: &World, tape: &Tape, region: RegionId) -> Option<Option<Finding>> {
    let frame = world.frame;
    let first = frame.checked_sub(grid.delay)?.wrapping_add(1);
    let start = tape.get(first)?;
    let mut f = first;
    while f <= frame {
        if tape.get(f).is_none_or(|e| e.travelled()) {
            return None;
        }
        f += 1;
    }
    let members: [bool; MAX_PLAYERS] =
        std::array::from_fn(|i| grid.in_region(region, start.before.players[i].pos));
    let mut w = start.before.clone();
    let mut f = first;
    while f <= frame {
        let e = tape.get(f)?;
        let inputs: [Input; MAX_PLAYERS] = std::array::from_fn(|i| {
            if members[i] {
                e.inputs[i]
            } else {
                Input::default()
            }
        });
        w.advance(inputs);
        f += 1;
    }
    if w.region_checksum(grid, region) == world.region_checksum(grid, region) {
        return Some(None);
    }
    // Which body: the first in the zone, in either world, whose hash differs.
    let mut real = [(0u32, 0u64); BODIES];
    let mut real_len = 0;
    world.each_body(|body, at, own| {
        if grid.in_zone(region, at) && real_len < BODIES {
            real[real_len] = (body.word(), own);
            real_len += 1;
        }
    });
    let mut odd = None;
    w.each_body(|body, at, own| {
        if odd.is_none()
            && grid.in_zone(region, at)
            && !real[..real_len].contains(&(body.word(), own))
        {
            odd = Some(body);
        }
    });
    if odd.is_none() {
        // Something in the real zone that the re-run does not have.
        let mut rerun = [(0u32, 0u64); BODIES];
        let mut len = 0;
        w.each_body(|body, at, own| {
            if grid.in_zone(region, at) && len < BODIES {
                rerun[len] = (body.word(), own);
                len += 1;
            }
        });
        world.each_body(|body, at, own| {
            if odd.is_none()
                && grid.in_zone(region, at)
                && !rerun[..len].contains(&(body.word(), own))
            {
                odd = Some(body);
            }
        });
    }
    Some(Some(Finding::Outran { region, body: odd }))
}
