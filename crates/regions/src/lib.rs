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
//!   fighter is a member) and, for this peer's own regions and their
//!   neighbours only, what each one's checksum is.
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

pub mod session;
mod watch;

pub use watch::{Finding, Issue, Watch};

use sim::World;
use sim::crowd::{Aggregate, MAX_STREAMS};
use sim::input::{Input, Travel};
use sim::region::{Grid, RegionId};
use sim::state::MAX_PLAYERS;
use sim::tuning as t;

/// How many aggregates the ledger keeps in hand, by the frame they are due:
/// every followed stream's, for as many epochs as the lookahead spans, with
/// room for a rollback's worth of re-feeding behind.
pub const CROWD_TAPE: usize = 256;

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

impl Default for Tape {
    fn default() -> Tape {
        Tape::new()
    }
}

impl Tape {
    pub fn new() -> Tape {
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

/// A region in this peer's books -- one its own fighter is a member of, or
/// a neighbour of one -- and its checksum at the last confirmed frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Live {
    pub region: RegionId,
    pub checksum: u64,
    /// Whether this peer's own fighter is a member: the region is its own,
    /// rather than next door.
    pub mine: bool,
}

/// The most regions a peer's books hold: at most three of its own, each
/// with six neighbours, fewer shared.
pub const MAX_BOOKS: usize = 16;

/// A stream the field follows, and the newest epoch an aggregate has been
/// offered for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeardCrowd {
    pub region: RegionId,
    pub epoch: Option<u16>,
    /// No peer behind it: the ledger says what it would say -- nobody there.
    pub simulated: bool,
}

/// The books.
pub struct Ledger {
    grid: Grid,
    tape: Box<Tape>,
    /// **The crowd tape**: every aggregate offered and not yet past its
    /// due frame by more than a rollback, kept to be fed at that frame
    /// (`World::advance_with`) and fed again if the frame is run again.
    crowd: Box<[Option<Aggregate>; CROWD_TAPE]>,
    /// The streams the last confirmed world follows, and what has been
    /// heard on each: what the gate holds for.
    followed: [HeardCrowd; MAX_STREAMS],
    followed_len: usize,
    /// The live regions' aggregates summed at the last epoch's frame: what
    /// a leader of each posts to the leaders next door (`net::crowd`).
    summed: [Aggregate; MAX_LIVE],
    summed_len: usize,
    /// The newest frame the books have been kept for.
    confirmed: Option<u32>,
    /// This peer's seat: whose feet decide which regions are its own.
    me: usize,
    /// Every region anybody is a member of, at the last confirmed frame.
    live: [RegionId; MAX_LIVE],
    live_len: usize,
    /// The books: this peer's own regions and their neighbours, with their
    /// checksums. Nothing further away is ever summed up.
    books: [Live; MAX_BOOKS],
    books_len: usize,
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
    /// Books starting at `world`, kept for the fighter in seat `me`.
    /// `watch` turns the watchdog on: dev mode.
    pub fn new(world: &World, me: usize, watch: bool) -> Ledger {
        Ledger {
            grid: Grid::tuned(),
            tape: Box::new(Tape::new()),
            confirmed: None,
            me: me.min(MAX_PLAYERS - 1),
            live: [RegionId::default(); MAX_LIVE],
            live_len: 0,
            books: [Live {
                region: RegionId::default(),
                checksum: 0,
                mine: false,
            }; MAX_BOOKS],
            books_len: 0,
            heard: [Heard {
                region: RegionId::default(),
                through: 0,
                simulated: true,
            }; MAX_TRACKED],
            heard_len: 0,
            clock: world.frame,
            held_since: None,
            watch: watch.then(|| Box::new(Watch::new())),
            crowd: Box::new([None; CROWD_TAPE]),
            followed: [HeardCrowd {
                region: RegionId::default(),
                epoch: None,
                simulated: true,
            }; MAX_STREAMS],
            followed_len: 0,
            summed: [Aggregate::NONE; MAX_LIVE],
            summed_len: 0,
        }
    }

    /// The live regions' aggregates as summed at the last epoch's frame.
    pub fn summed(&self) -> &[Aggregate] {
        &self.summed[..self.summed_len]
    }

    /// **An aggregate arrived**, from a leader or from this ledger's own
    /// reckoning: kept until the frame it is due, replacing an earlier copy
    /// of the same region and epoch (a repeat on the wire is nothing). One
    /// already past its due frame by more than the tape holds is dropped:
    /// the gate would have held for it, so it is not missed, only late.
    pub fn offer(&mut self, a: Aggregate) {
        let due = a.due();
        if let Some(c) = self.confirmed
            && due.wrapping_add(TAPE as u32) < c
        {
            return;
        }
        let slot = &mut self.crowd[due as usize % CROWD_TAPE];
        match slot {
            Some(have) if have.due() == due && have.region != a.region => {
                // Two regions due the same frame share a slot only by
                // collision of the ring; keep the newer offer in the first
                // free slot after it.
                let mut k = (due as usize + 1) % CROWD_TAPE;
                while k != due as usize % CROWD_TAPE {
                    match &self.crowd[k] {
                        Some(o) if o.due() == due && o.region != a.region => {}
                        Some(o) if o.due() == due => break,
                        Some(o)
                            if o.due().wrapping_add(TAPE as u32) < self.confirmed.unwrap_or(0) =>
                        {
                            break;
                        }
                        None => break,
                        _ => {}
                    }
                    k = (k + 1) % CROWD_TAPE;
                }
                self.crowd[k] = Some(a);
            }
            _ => *slot = Some(a),
        }
        for f in self.followed[..self.followed_len].iter_mut() {
            if f.region == a.region {
                f.epoch = Some(f.epoch.map_or(a.epoch, |e| e.max(a.epoch)));
            }
        }
    }

    /// **What is due at `frame`**: every aggregate kept whose due frame is
    /// it, in region order, for `World::advance_with`. A fixed array and a
    /// count, so a frame allocates nothing.
    pub fn due(&self, frame: u32) -> ([Aggregate; MAX_STREAMS], usize) {
        let mut out = [Aggregate::NONE; MAX_STREAMS];
        let mut n = 0;
        for a in self.crowd.iter().flatten() {
            if a.due() == frame && n < MAX_STREAMS {
                out[n] = *a;
                n += 1;
            }
        }
        out[..n].sort_by_key(|a| (a.region.q, a.region.r));
        (out, n)
    }

    /// The streams followed, and what has been heard on each.
    pub fn followed(&self) -> &[HeardCrowd] {
        &self.followed[..self.followed_len]
    }

    /// **A peer is behind this region** (or no longer is): a cell leader's
    /// line opened (or closed), so the ledger stops (or starts) speaking for
    /// it, and the gate holds for its aggregates. What the session layer
    /// says as lines come and go. Remembered for a region not yet followed.
    pub fn peer_behind(&mut self, region: RegionId, real: bool) {
        if let Some(f) = self.followed[..self.followed_len]
            .iter_mut()
            .find(|f| f.region == region)
        {
            f.simulated = !real;
        } else if self.followed_len < MAX_STREAMS {
            self.followed[self.followed_len] = HeardCrowd {
                region,
                epoch: None,
                simulated: !real,
            };
            self.followed_len += 1;
        }
    }

    /// The epoch whose aggregate frame `next` needs, if it needs one: the
    /// one due at or before it, once the lookahead has passed.
    fn epoch_needed(next: u32) -> Option<u16> {
        let (e, l) = (t::crowd_epoch(), t::crowd_lookahead());
        next.checked_sub(l).map(|f| (f / e) as u16)
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn confirmed(&self) -> Option<u32> {
        self.confirmed
    }

    /// Every region anybody is a member of, at the last confirmed frame.
    pub fn live(&self) -> &[RegionId] {
        &self.live[..self.live_len]
    }

    /// **The books**: this peer's own regions and their neighbours, with
    /// their checksums at the last confirmed frame. The only regions it
    /// ever sums up.
    pub fn books(&self) -> &[Live] {
        &self.books[..self.books_len]
    }

    /// Whose fighter the books are kept for.
    pub fn seat(&self) -> usize {
        self.me
    }

    /// The books change seats: a match seats this machine.
    pub fn sit(&mut self, me: usize) {
        self.me = me.min(MAX_PLAYERS - 1);
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
        // A followed stream with no peer behind it says, on the same clock,
        // that nobody is there: an aggregate of nothing for every epoch its
        // clock has reached.
        let e = t::crowd_epoch();
        let reached = (through / e) as u16;
        for k in 0..self.followed_len {
            let f = self.followed[k];
            if !f.simulated {
                continue;
            }
            let from = f.epoch.map_or(reached.saturating_sub(1), |have| have + 1);
            let mut epoch = from;
            while epoch <= reached {
                self.offer(Aggregate {
                    region: f.region,
                    epoch,
                    ..Aggregate::NONE
                });
                epoch += 1;
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
        // The crowd's gate: every stream the world follows has to have said
        // its aggregate for the epoch frame `next` reads (0018, S1). A
        // simulated stream has, by its own clock.
        let crowd_lagging = Self::epoch_needed(next).and_then(|e| {
            self.followed[..self.followed_len]
                .iter()
                .find(|f| f.epoch.is_none_or(|have| have < e))
                .copied()
        });
        let lagging = lagging.or(crowd_lagging.map(|f| Heard {
            region: f.region,
            through: 0,
            simulated: f.simulated,
        }));
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
        *self.crowd = [None; CROWD_TAPE];
        self.followed_len = 0;
        self.confirmed = None;
        self.live_len = 0;
        self.books_len = 0;
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

        // Live: every region a fighter is a member of. No checksum: these
        // are for the aggregates, which every peer of a match feeds alike.
        self.live_len = 0;
        let origin = world.map_origin();
        for p in &world.players {
            for &r in grid.regions_of(p.pos.add(origin)).as_slice() {
                if self.live_len < MAX_LIVE && !self.live().contains(&r) {
                    self.live[self.live_len] = r;
                    self.live_len += 1;
                }
            }
        }

        // The books: my own regions, then their neighbours, each summed up.
        // A region further off is somebody else's to sum: the cost of the
        // books is the cost of one neighbourhood, however many are playing.
        self.books_len = 0;
        let mine = grid.regions_of(world.players[self.me].pos.add(origin));
        let mut book = |region: RegionId, mine: bool| {
            if self.books_len < MAX_BOOKS
                && !self.books[..self.books_len]
                    .iter()
                    .any(|b| b.region == region)
            {
                self.books[self.books_len] = Live {
                    region,
                    checksum: world.region_checksum(&grid, region),
                    mine,
                };
                self.books_len += 1;
            }
        };
        for &r in mine.as_slice() {
            book(r, true);
        }
        for &r in mine.as_slice() {
            for n in r.neighbours() {
                book(n, false);
            }
        }

        // Followed: my own regions, heard through this frame, and every
        // region next to one -- heard through this frame too if somebody
        // stands in it (a match has its inputs), else by heartbeat.
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
        for b in &self.books[..self.books_len] {
            if b.mine || self.live[..self.live_len].contains(&b.region) {
                follow(b.region, true, world.frame);
            } else {
                // A neighbour already followed keeps how recently it was heard.
                let kept = self.heard[..self.heard_len]
                    .iter()
                    .find(|h| h.region == b.region && h.simulated)
                    .map_or(simulated, |h| h.through);
                follow(b.region, false, kept);
            }
        }
        self.heard = next;
        self.heard_len = len;

        // The crowd: what the world follows, and which of those this ledger
        // speaks for (no live region, so no peer). And at an epoch's frame,
        // every live region's aggregate, summed from this confirmed world --
        // what a leader would send, offered here to the world's own field.
        let mut followed = [HeardCrowd {
            region: RegionId::default(),
            epoch: None,
            simulated: true,
        }; MAX_STREAMS];
        let mut followed_len = 0;
        for s in world.crowd.streams.iter().filter(|s| s.on) {
            let kept = self.followed[..self.followed_len]
                .iter()
                .find(|f| f.region == s.region)
                .copied();
            let live = self.live[..self.live_len].contains(&s.region);
            // Spoken for unless somebody is behind it: a live region (this
            // peer is), or a peer the session layer said has a line open.
            let real = live || kept.is_some_and(|k| !k.simulated);
            followed[followed_len] = HeardCrowd {
                region: s.region,
                epoch: kept.and_then(|k| k.epoch),
                simulated: !real,
            };
            followed_len += 1;
        }
        self.followed = followed;
        self.followed_len = followed_len;
        let e = t::crowd_epoch();
        if world.frame % e == 0 {
            let epoch = (world.frame / e) as u16;
            self.summed_len = 0;
            for k in 0..self.live_len {
                let region = self.live[k];
                let a = world.aggregate(&grid, region, epoch);
                self.offer(a);
                if self.summed_len < MAX_LIVE {
                    self.summed[self.summed_len] = a;
                    self.summed_len += 1;
                }
            }
        }

        if let Some(mut w) = self.watch.take() {
            w.confirm(&grid, world, &self.tape, self.books());
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
