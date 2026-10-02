//! The region ledger: the books every build keeps, one confirmed frame at a
//! time. `docs/design/regions.md` is the specification.
//!
//! The simulation never reads this. A fight plays out the same with it or
//! without it. What it does is watch confirmed frames the way a peer in the
//! open world will have to:
//!
//! - **The tape** keeps the last few frames, each the world before it and the
//!   inputs that advanced it, so a frame can be run again.
//! - **The books** say, for each confirmed frame, which regions are live (a
//!   fighter is a member) and what each live region's checksum is.
//! - **Heartbeats**: every region next to a live one has to have said what
//!   happened in it, recently enough. A region with a fighter in it is heard
//!   through the frame just confirmed, because its inputs are in hand. Every
//!   other one has no peer behind it in a one-arena game, so the ledger plays
//!   one: a clock that ticks once per wall tick, heard through that clock less
//!   the Oven's *neighbour lag*.
//! - **The gate**: [`Ledger::may_advance`] holds the next frame until every
//!   such region is heard through at least `next − delay`. Both drivers ask it.
//! - **The watchdog** ([`Watch`]), in dev mode, checks the promises the books
//!   rest on and records every broken one as an [`Issue`].
//!
//! Allocates when it is made -- the tape is boxed -- and never after.

mod watch;

pub use watch::{Finding, Issue, Watch};

use sim::World;
use sim::input::{Input, Travel};
use sim::region::{Grid, RegionId};
use sim::state::MAX_PLAYERS;

/// The most frames the tape holds, and so the longest delay the Oven's
/// *delay* knob may ask for (its bound is one less).
pub const TAPE: usize = 31;

/// The most live regions the books keep, and the most regions they follow
/// heartbeats for. A sound grid puts two fighters in at most six live regions
/// with at most eighteen neighbours between them; the room past that is for an
/// unsound grid, which the watchdog reports rather than the books dropping.
pub const MAX_LIVE: usize = 16;
pub const MAX_TRACKED: usize = 64;

/// One frame on the tape: the world before it and what advanced it.
#[derive(Clone)]
pub struct Entry {
    pub before: World,
    pub inputs: [Input; MAX_PLAYERS],
}

impl Entry {
    /// The frame this entry produces.
    pub fn frame(&self) -> u32 {
        self.before.frame.wrapping_add(1)
    }

    /// Whether this frame went somewhere else: a fresh fight, which no region
    /// re-run can reproduce from the frame before.
    pub fn travelled(&self) -> bool {
        self.inputs.iter().any(|i| i.travel != Travel::NONE)
    }
}

/// The last [`TAPE`] frames, by frame number. A frame run again -- a rollback
/// -- overwrites what was there, so the tape always holds the latest version
/// of each frame.
pub struct Tape {
    slots: [Option<Entry>; TAPE],
}

impl Tape {
    fn new() -> Tape {
        Tape {
            slots: std::array::from_fn(|_| None),
        }
    }

    /// Keep the frame `before` is about to be advanced into, by `inputs`.
    pub fn record(&mut self, before: &World, inputs: [Input; MAX_PLAYERS]) {
        let frame = before.frame.wrapping_add(1);
        let slot = &mut self.slots[frame as usize % TAPE];
        match slot {
            Some(e) => {
                e.before.clone_from(before);
                e.inputs = inputs;
            }
            None => {
                *slot = Some(Entry {
                    before: before.clone(),
                    inputs,
                })
            }
        }
    }

    /// The entry that produced `frame`, if the tape still has it.
    pub fn get(&self, frame: u32) -> Option<&Entry> {
        self.slots[frame as usize % TAPE]
            .as_ref()
            .filter(|e| e.frame() == frame)
    }

    fn clear(&mut self) {
        // Kept allocated; marked stale by a frame no lookup asks for.
        for e in self.slots.iter_mut().flatten() {
            e.before.frame = u32::MAX - 1;
        }
    }
}

/// A region the books follow, and how recently it has been heard from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Heard {
    pub region: RegionId,
    /// The newest frame it has said what happened in.
    pub through: u32,
    /// No fighter stands in it, so its heartbeat is the ledger's own.
    pub simulated: bool,
}

/// A live region and its checksum at the last confirmed frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Live {
    pub region: RegionId,
    pub checksum: u64,
}

/// The books.
pub struct Ledger {
    grid: Grid,
    tape: Box<Tape>,
    /// The newest frame the books have been kept for.
    confirmed: Option<u32>,
    live: [Live; MAX_LIVE],
    live_len: usize,
    heard: [Heard; MAX_TRACKED],
    heard_len: usize,
    /// The simulated neighbours' clock: one tick per wall tick.
    clock: u32,
    /// Whether the gate is holding, and since when -- so a stall is one issue
    /// with a length, not one per tick.
    held_since: Option<u32>,
    watch: Option<Box<Watch>>,
}

impl Ledger {
    /// Books starting at `world`. `watch` turns the watchdog on: dev mode.
    pub fn new(world: &World, watch: bool) -> Ledger {
        Ledger {
            grid: Grid::tuned(),
            tape: Box::new(Tape::new()),
            confirmed: None,
            live: [Live {
                region: RegionId::default(),
                checksum: 0,
            }; MAX_LIVE],
            live_len: 0,
            heard: [Heard {
                region: RegionId::default(),
                through: 0,
                simulated: true,
            }; MAX_TRACKED],
            heard_len: 0,
            clock: world.frame,
            held_since: None,
            watch: watch.then(|| Box::new(Watch::new())),
        }
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn confirmed(&self) -> Option<u32> {
        self.confirmed
    }

    /// The live regions at the last confirmed frame, with their checksums.
    pub fn live(&self) -> &[Live] {
        &self.live[..self.live_len]
    }

    /// Every region the books follow, and how recently each was heard from.
    pub fn heard(&self) -> &[Heard] {
        &self.heard[..self.heard_len]
    }

    pub fn tape(&self) -> &Tape {
        &self.tape
    }

    /// The watchdog, when it is on.
    pub fn watch(&self) -> Option<&Watch> {
        self.watch.as_deref()
    }

    /// Keep a frame on the tape: called with the world just before it is
    /// advanced and the inputs that will advance it. Every advance, including
    /// one run again by a rollback.
    pub fn record(&mut self, before: &World, inputs: [Input; MAX_PLAYERS]) {
        self.tape.record(before, inputs);
    }

    /// One wall tick: the simulated neighbours move on, whether or not this
    /// peer's world did -- a real one keeps going while you wait for it.
    pub fn tick(&mut self) {
        self.clock = self.clock.wrapping_add(1);
        let through = self.simulated_through();
        for h in self.heard[..self.heard_len].iter_mut() {
            if h.simulated {
                h.through = through;
            }
        }
    }

    /// Whether frame `next` may be simulated: every region next to a live one
    /// has been heard through at least `next − delay`. A held frame is one
    /// [`Finding::Held`] when the hold ends, naming the region that held it.
    pub fn may_advance(&mut self, next: u32) -> bool {
        let need = next.saturating_sub(self.grid.delay);
        let lagging = self.heard[..self.heard_len]
            .iter()
            .filter(|h| h.through < need)
            .min_by_key(|h| h.through)
            .copied();
        match (lagging, self.held_since) {
            (Some(_), None) => {
                self.held_since = Some(self.clock);
                false
            }
            (Some(_), Some(_)) => false,
            (None, Some(since)) => {
                self.held_since = None;
                let behind = self.clock.wrapping_sub(since);
                let region = self.slowest().map_or(RegionId::default(), |h| h.region);
                if let Some(w) = self.watch.as_deref_mut() {
                    w.report(
                        next,
                        Finding::Held {
                            region,
                            ticks: behind,
                        },
                    );
                }
                true
            }
            (None, None) => true,
        }
    }

    /// Keep the books for every frame up to `frame` not yet kept. `now` is
    /// the world as it stands -- at `frame` itself for the local driver; ahead
    /// of it online, where the frames between come off the tape.
    pub fn confirm_through(&mut self, frame: u32, now: &World) {
        if self.confirmed.is_some_and(|c| c > now.frame) {
            // The world went backwards: a fresh fight. New books.
            self.restart(now);
        }
        let first = self.confirmed.map_or(frame, |c| c.wrapping_add(1));
        let mut f = first;
        while f <= frame {
            if f == now.frame {
                self.confirm(now);
            } else if let Some(next) = self.tape.get(f.wrapping_add(1)) {
                let world = next.before.clone();
                self.confirm(&world);
            }
            f = f.wrapping_add(1);
        }
    }

    fn restart(&mut self, now: &World) {
        self.tape.clear();
        self.confirmed = None;
        self.live_len = 0;
        self.heard_len = 0;
        self.clock = now.frame;
        self.held_since = None;
        if let Some(w) = self.watch.as_deref_mut() {
            w.forget();
        }
    }

    /// The books for one confirmed frame.
    fn confirm(&mut self, world: &World) {
        let grid = Grid::tuned();
        if grid != self.grid {
            // A knob moved: the old books were kept against another grid.
            self.grid = grid;
            self.heard_len = 0;
            if let Some(w) = self.watch.as_deref_mut() {
                w.forget();
            }
        }
        self.confirmed = Some(world.frame);

        // Live: every region a fighter is a member of.
        self.live_len = 0;
        for p in &world.players {
            for &r in grid.regions_of(p.pos).as_slice() {
                if self.live_len < MAX_LIVE && !self.live().iter().any(|l| l.region == r) {
                    self.live[self.live_len] = Live {
                        region: r,
                        checksum: world.region_checksum(&grid, r),
                    };
                    self.live_len += 1;
                }
            }
        }

        // Followed: the live regions, heard through this frame, and every
        // region next to one, heard from by heartbeat.
        let mut next = [Heard {
            region: RegionId::default(),
            through: 0,
            simulated: true,
        }; MAX_TRACKED];
        let mut len = 0;
        let mut follow = |region: RegionId, real: bool, through: u32| {
            if let Some(h) = next[..len].iter_mut().find(|h| h.region == region) {
                if real {
                    h.simulated = false;
                    h.through = through;
                }
            } else if len < MAX_TRACKED {
                next[len] = Heard {
                    region,
                    through,
                    simulated: !real,
                };
                len += 1;
            }
        };
        let simulated = self.simulated_through();
        for l in &self.live[..self.live_len] {
            follow(l.region, true, world.frame);
        }
        for l in &self.live[..self.live_len] {
            for n in l.region.neighbours() {
                // A neighbour already followed keeps how recently it was heard.
                let kept = self.heard[..self.heard_len]
                    .iter()
                    .find(|h| h.region == n && h.simulated)
                    .map_or(simulated, |h| h.through);
                follow(n, false, kept);
            }
        }
        self.heard = next;
        self.heard_len = len;

        if let Some(mut w) = self.watch.take() {
            w.confirm(&grid, world, &self.tape, self.live());
            self.watch = Some(w);
        }
    }

    fn simulated_through(&self) -> u32 {
        self.clock
            .saturating_sub(sim::tuning::region_neighbour_lag())
    }

    fn slowest(&self) -> Option<Heard> {
        self.heard[..self.heard_len]
            .iter()
            .min_by_key(|h| h.through)
            .copied()
    }
}
