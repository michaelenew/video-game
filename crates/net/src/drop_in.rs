//! **Playing in a room you come and go from**: a [`Lobby`]'s sessions, one
//! per lineup, and the frames they change at. What the game's online driver
//! holds for a room, and what `tests/drop_in.rs` drives the same way.
//!
//! Each frame the driver calls [`DropIn::tend`] before it advances -- the
//! lobby polled, a session started when one can be, a closed line's seat
//! given up, the host's choice of seat for a newcomer -- and puts
//! [`DropIn::travel`] in its own input ([`DropIn::sent`] once it has gone):
//! the host's admission of a newcomer,
//! so every machine seats them on the same frame. After the advance, and
//! after the books are kept, [`DropIn::after`] looks through the frames now
//! confirmed for that admission, and at it starts the session for the new
//! lineup from the world at that frame -- dropping the few frames played past
//! it -- and the host sends that world to the newcomer.
//!
//! [`Lobby`]: crate::lobby::Lobby

use crate::lobby::Gathering;
use crate::{SessionConfig, p2p};
use ggrs::{P2PSession, SessionState};
use sim::World;
use sim::input::{Destination, Travel};

/// What a call changed about the world the driver holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// Nothing: play on.
    None,
    /// A session started, and the world is the one it starts from: begin the
    /// replay, the books and the interpolation again from it.
    Started,
    /// The lineup changed: a session for the new one, from the world at the
    /// frame somebody was seated -- the same things begun again from it.
    Switched,
}

/// A room's play: its lobby, and the session for its current lineup.
pub struct DropIn {
    pub lobby: Box<dyn Gathering>,
    pub session: Option<P2PSession<SessionConfig>>,
    /// GGRS's players' world seats, in GGRS's order.
    pub seats: Vec<usize>,
    /// This machine's GGRS player.
    pub handle: usize,
    /// Everybody's id, in GGRS's order: what a cell's leaders are ranked by.
    pub roster: Vec<u64>,
    /// The session's post box, for the crowd and for closed lines.
    pub post: Option<crate::crowd::Post>,
    /// The epoch of the lineup the session runs.
    pub epoch: u32,
    /// The world's frame when the session started: GGRS counts its frames
    /// from nought, and its frame `g` is the world's `base + g`.
    pub base: u32,
    delay: usize,
    /// The last confirmed frame looked through for an admission.
    scanned: u32,
    /// **Host**: an admission to go in the next input.
    admit: Option<Travel>,
    settled: bool,
}

impl DropIn {
    /// `delay` is this player's input delay, for every session the room runs.
    pub fn new(lobby: Box<dyn Gathering>, delay: usize) -> DropIn {
        DropIn {
            lobby,
            session: None,
            seats: Vec::new(),
            handle: 0,
            roster: Vec::new(),
            post: None,
            epoch: 0,
            base: 0,
            delay,
            scanned: 0,
            admit: None,
            settled: true,
        }
    }

    /// **The session's confirmed frame, as the world counts**: what the
    /// region gate and the books are told. `-1` before any is confirmed.
    pub fn confirmed(&self) -> i32 {
        match &self.session {
            Some(s) if s.confirmed_frame() >= 0 => self.base as i32 + s.confirmed_frame(),
            _ => -1,
        }
    }

    /// This machine's world seat in the running session.
    pub fn seat(&self) -> usize {
        self.seats.get(self.handle).copied().unwrap_or(0)
    }

    /// GGRS's player for a world seat.
    pub fn handle_of(&self, seat: usize) -> Option<usize> {
        self.seats.iter().position(|&s| s == seat)
    }

    fn start(&mut self, world: &World, ledger: &mut regions::Ledger) -> bool {
        let epoch = self.lobby.lineup().map_or(0, |l| l.epoch);
        let (seat, seats) = self.lobby.session();
        let handle = seat.handle;
        let roster = seat.roster.clone();
        let post = seat.post.clone();
        match p2p::start_with_delay(seat, self.delay) {
            Ok(session) => {
                self.session = Some(session);
                self.seats = seats;
                self.handle = handle;
                self.roster = roster;
                self.post = Some(post);
                self.epoch = epoch;
                self.base = world.frame;
                self.scanned = world.frame;
                self.settled = false;
                ledger.sit(self.seat());
                ledger.begin(world);
                true
            }
            Err(_) => false,
        }
    }

    /// **Before the frame.** `start` is the world a room's first session
    /// starts from: the link's. Replaces `world` when a session starts.
    pub fn tend(
        &mut self,
        now_ms: u64,
        world: &mut World,
        start: &World,
        ledger: &mut regions::Ledger,
    ) -> Change {
        self.lobby.poll(now_ms);
        let me = self.lobby.me();
        let Some(session) = &mut self.session else {
            let Some(lineup) = self.lobby.lineup().cloned() else {
                return Change::None;
            };
            if lineup.newcomer == me {
                // Let into a match already running: from the world the host
                // sends, once it is all here.
                if let Some(w) = self.lobby.take_world() {
                    *world = w;
                    if self.start(world, ledger) {
                        return Change::Started;
                    }
                }
                return Change::None;
            }
            if lineup.go && self.lobby.lines_ready() {
                *world = start.clone();
                if self.start(world, ledger) {
                    return Change::Started;
                }
            }
            return Change::None;
        };
        // **A closed line is somebody gone**: their seat, given up now.
        if let Some(post) = &self.post {
            for peer in post.take_closed() {
                let crate::Peer::Line(handle) = peer;
                let _ = session.disconnect_player(handle as usize);
            }
        }
        if !self.settled && session.current_state() == SessionState::Running {
            self.settled = true;
            self.lobby.settled();
        }
        // **Host**: a seat for whoever is waiting, on the smaller side.
        if let Some((id, _)) = self.lobby.newcomer()
            && let Some(seat) = world.open_seat(id & 1 == 1)
        {
            self.lobby.admit(id, seat);
        }
        if self.admit.is_none()
            && let Some((seat, class)) = self.lobby.admission()
        {
            self.admit = Some(Travel::admit(seat, class));
        }
        Change::None
    }

    /// The travel this machine's input must carry, if any: the host's
    /// admission of a newcomer. It stays until [`DropIn::sent`] says a frame
    /// went out with it.
    pub fn travel(&self) -> Option<Travel> {
        self.admit
    }

    /// The input carrying [`DropIn::travel`] has gone to the session.
    pub fn sent(&mut self) {
        self.admit = None;
    }

    /// **After the frame**, once the books are kept up to the confirmed
    /// frame: an admission confirmed, and the session for the new lineup
    /// started from the world at its frame. Replaces `world` then.
    pub fn after(&mut self, world: &mut World, ledger: &mut regions::Ledger) -> Change {
        let Ok(confirmed) = u32::try_from(self.confirmed()) else {
            return Change::None;
        };
        let confirmed = confirmed.min(world.frame);
        let newer = self.lobby.lineup().is_some_and(|l| l.epoch > self.epoch);
        let mut at = None;
        let mut f = self.scanned.wrapping_add(1);
        while f <= confirmed {
            if let Some(e) = ledger.tape().get(f)
                && e.inputs
                    .iter()
                    .any(|i| matches!(i.travel.destination(), Some(Destination::Admit { .. })))
            {
                at = Some(f);
                break;
            }
            f = f.wrapping_add(1);
        }
        let Some(f) = at else {
            self.scanned = confirmed;
            return Change::None;
        };
        if !newer {
            // Seated before the lineup that seats them reached this machine:
            // looked at again next frame.
            self.scanned = f.wrapping_sub(1);
            return Change::None;
        }
        self.scanned = f;
        // The world at the frame they were seated: the newest, or off the
        // tape when frames past it have been played.
        let seated = if world.frame == f {
            world.clone()
        } else {
            match ledger.tape().get(f.wrapping_add(1)) {
                Some(e) => e.before.clone(),
                None => return Change::None,
            }
        };
        if self.lobby.hosting() {
            self.lobby.send_world(&seated);
        }
        *world = seated;
        self.session = None;
        if self.start(world, ledger) {
            Change::Switched
        } else {
            Change::None
        }
    }
}
