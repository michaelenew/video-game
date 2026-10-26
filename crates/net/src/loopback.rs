//! A board and a line that never leave the process.
//!
//! For tests: two [`Room`](crate::meet::Room)s that meet through a [`Hall`]
//! and play over a [`Wire`] are the whole protocol and a real GGRS session,
//! with no sockets, no browser and no broker -- and with as many notes lost as
//! a test cares to lose.

use crate::meet::{Board, Line, LineState, Reach};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A room's worth of notes, in the order they were posted.
#[derive(Clone, Default)]
pub struct Hall(Arc<Mutex<Vec<String>>>);

impl Hall {
    /// A board on this hall that reads every note.
    pub fn board(&self) -> MemoryBoard {
        self.losing(0)
    }

    /// A board that loses one note in `n`, at random; 0 loses none. At
    /// random rather than every `n`th, because the protocol's notes repeat on
    /// fixed periods and a fixed loss pattern can alias with them -- losing
    /// the same answer every time, which no real network does.
    /// Every note posted to a public broker can be lost, and the protocol has
    /// to survive it.
    ///
    /// It reads from now on, the way a subscriber to a broker topic hears
    /// nothing that was published before it subscribed.
    pub fn losing(&self, n: usize) -> MemoryBoard {
        MemoryBoard {
            hall: self.clone(),
            read: lock(&self.0).len(),
            lose_one_in: n as u64,
            rng: 0x9e37_79b9_7f4a_7c15 ^ n as u64,
        }
    }
}

pub struct MemoryBoard {
    hall: Hall,
    read: usize,
    lose_one_in: u64,
    rng: u64,
}

impl Board for MemoryBoard {
    /// Including to the poster, the way a broker echoes a publish back to a
    /// subscriber.
    fn post(&mut self, note: &str) {
        lock(&self.hall.0).push(note.to_string());
    }

    fn take(&mut self) -> Option<String> {
        loop {
            let note = lock(&self.hall.0).get(self.read).cloned()?;
            self.read += 1;
            self.rng ^= self.rng << 13;
            self.rng ^= self.rng >> 7;
            self.rng ^= self.rng << 17;
            if self.lose_one_in == 0 || self.rng % self.lose_one_in != 0 {
                return Some(note);
            }
        }
    }

    fn reach(&mut self, _now_ms: u64) -> Reach {
        Reach::Up
    }
}

#[derive(Default)]
struct Ends {
    /// Packets waiting at each end.
    queues: [VecDeque<Vec<u8>>; 2],
    /// The offering side has accepted an answer to its current offer.
    connected: bool,
    /// Lines made on this wire so far; each offer is named after its line.
    made: u32,
}

/// Two ends of one connection, before either is handed to a room.
#[derive(Clone, Default)]
pub struct Wire(Arc<Mutex<Ends>>);

impl Wire {
    /// End 0 or end 1. Which end makes the offer is the room's business.
    pub fn end(&self, which: usize) -> MemoryLine {
        let serial = {
            let mut ends = lock(&self.0);
            ends.made += 1;
            ends.made
        };
        MemoryLine {
            wire: self.clone(),
            at: which & 1,
            serial,
            described: None,
        }
    }
}

/// One end. Strict the way WebRTC is: an answer connects only the offer it
/// was written for, so a protocol that hands an old answer to a new offer
/// never opens -- which is the mistake this exists to catch.
pub struct MemoryLine {
    wire: Wire,
    at: usize,
    serial: u32,
    described: Option<String>,
}

impl Line for MemoryLine {
    fn offer(&mut self) {
        self.described = Some(format!("offer {}", self.serial));
    }

    fn answer(&mut self, offer: &str) {
        self.described = Some(format!("answer to {offer}"));
    }

    fn accept(&mut self, answer: &str) {
        if answer == format!("answer to offer {}", self.serial) {
            lock(&self.wire.0).connected = true;
        }
    }

    fn description(&mut self, _now_ms: u64) -> Option<String> {
        self.described.clone()
    }

    fn state(&mut self) -> LineState {
        if self.described.is_some() && lock(&self.wire.0).connected {
            LineState::Open
        } else {
            LineState::Negotiating
        }
    }

    fn send(&mut self, packet: &[u8]) {
        lock(&self.wire.0).queues[1 - self.at].push_back(packet.to_vec());
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        lock(&self.wire.0).queues[self.at].pop_front()
    }
}

/// **Lines that pair the way WebRTC's do**: through what they describe. A
/// line made here offers under a serial of its own, an answer names the offer
/// it answers, and the two ends join when the offerer accepts that answer --
/// so any number of rooms can make lines off one switchboard, a fresh one per
/// partner, without anybody saying in advance who is wired to whom. What a
/// room of three or four meets over in a test.
#[derive(Clone, Default)]
pub struct Switchboard(Arc<Mutex<Exchange>>);

#[derive(Default)]
struct Exchange {
    /// Lines made so far; each is named by its number.
    made: u32,
    /// Packets waiting at each line, by its number.
    queues: Vec<VecDeque<Vec<u8>>>,
    /// The line at the other end of each, once paired.
    far: Vec<Option<u32>>,
    /// Lines whose offer has been accepted: open, both ends.
    open: Vec<bool>,
    /// Lines hung up, both ends: somebody left.
    hung_up: Vec<bool>,
}

impl Switchboard {
    pub fn line(&self) -> SwitchedLine {
        let mut x = lock(&self.0);
        let serial = x.made;
        x.made += 1;
        x.queues.push(VecDeque::new());
        x.far.push(None);
        x.open.push(false);
        x.hung_up.push(false);
        SwitchedLine {
            board: self.clone(),
            serial,
            described: None,
        }
    }
}

pub struct SwitchedLine {
    board: Switchboard,
    serial: u32,
    described: Option<String>,
}

impl Switchboard {
    /// **Hang up a line**, both ends, the way a data channel closes when the
    /// page at one end does: each end reads [`LineState::Failed`] and
    /// nothing more is carried.
    pub fn hang_up(&self, serial: u32) {
        let mut x = lock(&self.0);
        let far = x.far.get(serial as usize).copied().flatten();
        for end in [Some(serial), far].into_iter().flatten() {
            if let Some(h) = x.hung_up.get_mut(end as usize) {
                *h = true;
            }
        }
    }
}

impl SwitchedLine {
    /// Which line this is, for [`Switchboard::hang_up`].
    pub fn serial(&self) -> u32 {
        self.serial
    }
}

impl Line for SwitchedLine {
    fn offer(&mut self) {
        self.described = Some(format!("offer {}", self.serial));
    }

    fn answer(&mut self, offer: &str) {
        let Some(theirs) = offer
            .strip_prefix("offer ")
            .and_then(|n| n.parse::<u32>().ok())
        else {
            return;
        };
        let mut x = lock(&self.board.0);
        if (theirs as usize) < x.far.len() {
            x.far[self.serial as usize] = Some(theirs);
            x.far[theirs as usize] = Some(self.serial);
        }
        drop(x);
        self.described = Some(format!("answer {} to offer {theirs}", self.serial));
    }

    fn accept(&mut self, answer: &str) {
        let mut x = lock(&self.board.0);
        let far = x.far[self.serial as usize];
        if let Some(far) = far
            && answer == format!("answer {far} to offer {}", self.serial)
        {
            x.open[self.serial as usize] = true;
            x.open[far as usize] = true;
        }
    }

    fn description(&mut self, _now_ms: u64) -> Option<String> {
        self.described.clone()
    }

    fn state(&mut self) -> LineState {
        let x = lock(&self.board.0);
        if x.hung_up[self.serial as usize] {
            LineState::Failed
        } else if x.open[self.serial as usize] {
            LineState::Open
        } else {
            LineState::Negotiating
        }
    }

    fn send(&mut self, packet: &[u8]) {
        let mut x = lock(&self.board.0);
        if x.hung_up[self.serial as usize] {
            return;
        }
        if let Some(far) = x.far[self.serial as usize] {
            x.queues[far as usize].push_back(packet.to_vec());
        }
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        lock(&self.board.0).queues[self.serial as usize].pop_front()
    }
}

/// **A line that loses packets**: one in `n` of what is sent, at random,
/// the way a data channel with retransmits off does. 0 loses none. For the
/// tests that have to survive it.
pub struct Lossy<L: Line> {
    pub line: L,
    lose_one_in: u64,
    rng: u64,
}

impl<L: Line> Lossy<L> {
    pub fn new(line: L, lose_one_in: usize, seed: u64) -> Lossy<L> {
        Lossy {
            line,
            lose_one_in: lose_one_in as u64,
            rng: 0x9e37_79b9_7f4a_7c15 ^ seed,
        }
    }
}

impl<L: Line> Line for Lossy<L> {
    fn offer(&mut self) {
        self.line.offer();
    }
    fn answer(&mut self, offer: &str) {
        self.line.answer(offer);
    }
    fn accept(&mut self, answer: &str) {
        self.line.accept(answer);
    }
    fn description(&mut self, now_ms: u64) -> Option<String> {
        self.line.description(now_ms)
    }
    fn state(&mut self) -> LineState {
        self.line.state()
    }
    fn send(&mut self, packet: &[u8]) {
        if self.lose_one_in > 0 {
            self.rng ^= self.rng << 13;
            self.rng ^= self.rng >> 7;
            self.rng ^= self.rng << 17;
            if self.rng % self.lose_one_in == 0 {
                return;
            }
        }
        self.line.send(packet);
    }
    fn recv(&mut self) -> Option<Vec<u8>> {
        self.line.recv()
    }
}

/// **What crossed a line**, counted: bytes and packets each way. Shared, so
/// the load harness (`bin/load.rs`) reads a peer's totals while its lines
/// are inside the session.
#[derive(Default, Debug)]
pub struct Meter {
    pub sent_bytes: std::sync::atomic::AtomicU64,
    pub sent_packets: std::sync::atomic::AtomicU64,
    pub got_bytes: std::sync::atomic::AtomicU64,
    pub got_packets: std::sync::atomic::AtomicU64,
    /// Bytes sent, by the packet's first byte (its kind, in `net::region`'s
    /// packets), the first eight kinds.
    pub sent_by_kind: [std::sync::atomic::AtomicU64; 8],
}

impl Meter {
    /// Bytes sent and received, so far.
    pub fn bytes(&self) -> (u64, u64) {
        use std::sync::atomic::Ordering::Relaxed;
        (self.sent_bytes.load(Relaxed), self.got_bytes.load(Relaxed))
    }

    /// Packets sent and received, so far.
    pub fn packets(&self) -> (u64, u64) {
        use std::sync::atomic::Ordering::Relaxed;
        (
            self.sent_packets.load(Relaxed),
            self.got_packets.load(Relaxed),
        )
    }
}

/// **A line that counts what crosses it** into a [`Meter`].
pub struct Metered<L: Line> {
    pub line: L,
    meter: Arc<Meter>,
}

impl<L: Line> Metered<L> {
    pub fn new(line: L, meter: Arc<Meter>) -> Metered<L> {
        Metered { line, meter }
    }
}

impl<L: Line> Line for Metered<L> {
    fn offer(&mut self) {
        self.line.offer();
    }
    fn answer(&mut self, offer: &str) {
        self.line.answer(offer);
    }
    fn accept(&mut self, answer: &str) {
        self.line.accept(answer);
    }
    fn description(&mut self, now_ms: u64) -> Option<String> {
        self.line.description(now_ms)
    }
    fn state(&mut self) -> LineState {
        self.line.state()
    }
    fn send(&mut self, packet: &[u8]) {
        use std::sync::atomic::Ordering::Relaxed;
        self.meter
            .sent_bytes
            .fetch_add(packet.len() as u64, Relaxed);
        self.meter.sent_packets.fetch_add(1, Relaxed);
        if let Some(k) = packet
            .first()
            .and_then(|&k| self.meter.sent_by_kind.get(k as usize))
        {
            k.fetch_add(packet.len() as u64, Relaxed);
        }
        self.line.send(packet);
    }
    fn recv(&mut self) -> Option<Vec<u8>> {
        use std::sync::atomic::Ordering::Relaxed;
        let got = self.line.recv()?;
        self.meter.got_bytes.fetch_add(got.len() as u64, Relaxed);
        self.meter.got_packets.fetch_add(1, Relaxed);
        Some(got)
    }
}

/// **A clock of ticks** that [`Delayed`] lines read: the load harness moves
/// it on once a round, so a delay is in frames whatever the machine's speed.
#[derive(Clone, Default)]
pub struct Ticks(Arc<std::sync::atomic::AtomicU32>);

impl Ticks {
    pub fn now(&self) -> u32 {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn tick(&self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

/// **A line with a trip time**: what is sent is held `delay` ticks of a
/// [`Ticks`] clock before it goes, the way a real line takes half a ping.
pub struct Delayed<L: Line> {
    pub line: L,
    clock: Ticks,
    delay: u32,
    held: VecDeque<(u32, Vec<u8>)>,
}

impl<L: Line> Delayed<L> {
    pub fn new(line: L, clock: Ticks, delay: u32) -> Delayed<L> {
        Delayed {
            line,
            clock,
            delay,
            held: VecDeque::new(),
        }
    }

    /// Everything whose time has come, onto the line.
    fn release(&mut self) {
        let now = self.clock.now();
        while self.held.front().is_some_and(|(at, _)| *at <= now) {
            if let Some((_, p)) = self.held.pop_front() {
                self.line.send(&p);
            }
        }
    }
}

impl<L: Line> Line for Delayed<L> {
    fn offer(&mut self) {
        self.line.offer();
    }
    fn answer(&mut self, offer: &str) {
        self.line.answer(offer);
    }
    fn accept(&mut self, answer: &str) {
        self.line.accept(answer);
    }
    fn description(&mut self, now_ms: u64) -> Option<String> {
        self.line.description(now_ms)
    }
    fn state(&mut self) -> LineState {
        self.line.state()
    }
    fn send(&mut self, packet: &[u8]) {
        self.held
            .push_back((self.clock.now() + self.delay, packet.to_vec()));
        self.release();
    }
    fn recv(&mut self) -> Option<Vec<u8>> {
        self.release();
        self.line.recv()
    }
}
