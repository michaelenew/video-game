//! **The load harness** (exploration 0029, step 1): bot peers on simulated
//! lines, no window, and what each one pays.
//!
//!     cargo run --release -p net --bin load
//!     cargo run --release -p net --bin load -- --frames 1800 --delay 4 --loss 20 --apart 300
//!
//! Every peer plays the valley's town with a bot on its keys -- an input held
//! for a quarter to two-thirds of a second, then another, the way hands
//! change their minds -- over lines that hold each packet `--delay` frames
//! (half a ping) and lose one in `--loss`. Two layouts: the fighters
//! **together**, a few metres apart, and **apart**, `--apart` metres between
//! each, in different regions. Two netcodes: **GGRS**, what the game plays
//! on (everybody runs everything and waits on everybody), and **regions**,
//! `net::region`'s session (`docs/design/regions.md`). For each, per peer:
//! what it sent and received a second, how often it rolled back, how often
//! it stood waiting, and what one tick cost to simulate.
//!
//! What it is for: every step of 0029 is judged against these numbers. A
//! netcode that scales is one whose *apart* row does not grow with the
//! number of peers.

use net::loopback::{Delayed, Hall, Lossy, Meter, Metered, Switchboard, Ticks, Wire};
use net::meet::{Rendezvous, Room};
use net::{NetInput, Progress, Seat, handle_requests_kept, p2p};
use sim::region::Grid;
use sim::state::MAX_PLAYERS;
use sim::{Class, Fx, Input, V3, World};
use std::sync::Arc;
use std::time::Instant;

struct Options {
    frames: u32,
    delay: u32,
    loss: usize,
    apart: i32,
    peers: usize,
}

fn options() -> Options {
    let args: Vec<String> = std::env::args().collect();
    let value = |name: &str, default: i64| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(default)
    };
    Options {
        frames: value("--frames", 1200) as u32,
        delay: value("--delay", 3) as u32,
        loss: value("--loss", 30) as usize,
        apart: value("--apart", 450) as i32,
        peers: (value("--peers", MAX_PLAYERS as i64) as usize).clamp(2, MAX_PLAYERS),
    }
}

/// What one peer paid over a run.
#[derive(Default, Clone)]
struct Bill {
    frames: u32,
    sent: u64,
    got: u64,
    packets: u64,
    rollbacks: u64,
    replayed: u64,
    stalls: u64,
    ticks_us: Vec<u64>,
    /// Bytes sent by kind of region packet: inputs, outcomes, wants,
    /// snapshots, books.
    kinds: [u64; 5],
}

impl Bill {
    fn seconds(&self) -> f64 {
        self.frames.max(1) as f64 / sim::TICK_HZ as f64
    }
}

/// The valley, `n` fighters: `gap` metres apart, on a grid of three by
/// three from the town (nine fit on the map up to about 450 m).
fn town(n: usize, gap: i32) -> World {
    let classes = [
        Class::Champion,
        Class::ShadowReaver,
        Class::Elementalist,
        Class::DualMage,
        Class::BloodMage,
        Class::Champion,
        Class::ShadowReaver,
        Class::Elementalist,
    ];
    let mut w = World::versus_in(classes, sim::valley::START).seated(n as u8);
    let ground = w.terrain();
    let origin = w.players[0].pos;
    // Together, a row; apart, a grid of three by three, from the town's
    // south-west so every point is on the map at up to 450 m.
    let (x0, z0) = if gap > 50 { (-300, -450) } else { (0, 0) };
    for i in 0..n {
        let (cx, cz) = if gap > 50 {
            ((i % 3) as i32 * gap, (i / 3) as i32 * gap)
        } else {
            (i as i32 * gap, 0)
        };
        let x = origin.x.add(Fx::from_int(x0 + cx));
        let z = origin.z.add(Fx::from_int(z0 + cz));
        let up = V3::new(x, Fx::from_int(1000), z);
        w.players[i].pos = V3::new(x, ground.floor_below(up), z);
    }
    w
}

/// **A bot's hands**: an input held 15 to 40 frames, then another --
/// walking, turning, jumping, swinging -- from a seed of its own.
struct Hands {
    rng: u64,
    held: Input,
    left: u32,
    aim: u16,
}

impl Hands {
    fn new(seed: u64) -> Hands {
        Hands {
            rng: 0x9e37_79b9_7f4a_7c15 ^ seed.wrapping_mul(0x2545_f491_4f6c_dd1d),
            held: Input::default(),
            left: 0,
            aim: (seed as u16).wrapping_mul(9_973),
        }
    }

    fn next(&mut self) -> Input {
        if self.left == 0 {
            self.rng ^= self.rng << 13;
            self.rng ^= self.rng >> 7;
            self.rng ^= self.rng << 17;
            let r = self.rng;
            let bits = [
                0,
                Input::W,
                Input::W,
                Input::W | Input::SPACE,
                Input::LEFT,
                Input::A,
                Input::D,
                Input::RIGHT,
            ][(r % 8) as usize];
            self.aim = self.aim.wrapping_add((r >> 8) as u16 % 8_000);
            self.held = Input::aimed(bits, self.aim);
            self.left = 15 + ((r >> 24) % 26) as u32;
        }
        self.left -= 1;
        self.held
    }
}

// ---------------------------------------------------------------------------
// GGRS: what the game plays on
// ---------------------------------------------------------------------------

fn ggrs(o: &Options, n: usize, gap: i32) -> Vec<Bill> {
    let clock = Ticks::default();
    let hall = Hall::default();
    let switch = Switchboard::default();
    let meters: Vec<Arc<Meter>> = (0..n).map(|_| Arc::new(Meter::default())).collect();
    let mut rooms: Vec<_> = (0..n)
        .map(|i| {
            let (switch, meter, clock, loss, delay) = (
                switch.clone(),
                meters[i].clone(),
                clock.clone(),
                o.loss,
                o.delay,
            );
            let serial = std::sync::atomic::AtomicU64::new(0);
            Room::new(
                hall.board(),
                move || {
                    let k = serial.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    let line = Lossy::new(switch.line(), loss, k * 7 + i as u64);
                    Metered::new(Delayed::new(line, clock.clone(), delay), meter.clone())
                },
                11 * (i as u64 + 1),
                "load",
            )
            .with_players(n)
        })
        .collect();
    let mut seats: Vec<Option<Seat>> = (0..n).map(|_| None).collect();
    for f in 0..20_000u64 {
        clock.tick();
        std::thread::sleep(std::time::Duration::from_micros(200));
        for (i, room) in rooms.iter_mut().enumerate() {
            if seats[i].is_none()
                && let Progress::Ready(s) = room.poll(f * 16)
            {
                seats[i] = Some(s);
            }
        }
        if seats.iter().all(Option::is_some) {
            break;
        }
    }
    let mut seats: Vec<Seat> = seats
        .into_iter()
        .map(|s| s.expect("every peer seated"))
        .collect();
    seats.sort_by_key(|s| s.handle);
    let start = town(n, gap);
    // What the meeting cost is not the match's.
    let before: Vec<(u64, u64, u64)> = meters
        .iter()
        .map(|m| {
            let (s, g) = m.bytes();
            (s, g, m.packets().0)
        })
        .collect();
    struct Peer {
        session: net::ggrs::P2PSession<net::SessionConfig>,
        world: World,
        ledger: regions::Ledger,
        hands: Hands,
        handle: usize,
        bill: Bill,
    }
    let mut peers: Vec<Peer> = seats
        .into_iter()
        .map(|s| {
            let handle = s.handle;
            let mut ledger = regions::Ledger::new(&start, 0, false);
            ledger.sit(handle);
            ledger.begin(&start);
            Peer {
                session: p2p::start(s).expect("a session"),
                world: start.clone(),
                ledger,
                hands: Hands::new(handle as u64 + 1),
                handle,
                bill: Bill::default(),
            }
        })
        .collect();
    let mut rounds = 0u32;
    while peers.iter().any(|p| p.world.frame < o.frames) && rounds < o.frames * 20 {
        rounds += 1;
        clock.tick();
        // GGRS keeps its handshake, its pings and its time sync on the wall
        // clock: a round has to take some of it.
        std::thread::sleep(std::time::Duration::from_micros(500));
        for p in peers.iter_mut() {
            p.session.poll_remote_clients();
            for _ in p.session.events() {}
            if p.world.frame >= o.frames
                || p.session.current_state() != net::ggrs::SessionState::Running
            {
                continue;
            }
            if !net::may_advance(&mut p.ledger, p.session.confirmed_frame(), &p.world) {
                p.bill.stalls += 1;
                continue;
            }
            let input = p.hands.next();
            if p.session
                .add_local_input(p.handle, NetInput::from(input))
                .is_err()
            {
                p.bill.stalls += 1;
                continue;
            }
            let began = Instant::now();
            match p.session.advance_frame() {
                Ok(requests) => {
                    let loads = requests
                        .iter()
                        .filter(|r| matches!(r, net::ggrs::GgrsRequest::LoadGameState { .. }))
                        .count() as u64;
                    let advances = requests
                        .iter()
                        .filter(|r| matches!(r, net::ggrs::GgrsRequest::AdvanceFrame { .. }))
                        .count() as u64;
                    if loads > 0 {
                        p.bill.rollbacks += loads;
                        p.bill.replayed += advances.saturating_sub(1);
                    }
                    handle_requests_kept(&mut p.world, requests, &mut p.ledger, |_, _, _| {});
                    if let Ok(c) = u32::try_from(p.session.confirmed_frame()) {
                        p.ledger.confirm_through(c, &p.world);
                    }
                    if advances > 0 {
                        p.bill.ticks_us.push(began.elapsed().as_micros() as u64);
                    }
                }
                Err(_) => p.bill.stalls += 1,
            }
        }
    }
    peers
        .into_iter()
        .map(|p| {
            let m = &meters[p.handle];
            let (s, g) = m.bytes();
            let (s0, g0, k0) = before[p.handle];
            Bill {
                frames: p.world.frame,
                sent: s - s0,
                got: g - g0,
                packets: m.packets().0 - k0,
                ..p.bill
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The region session: what 0029 proposes the game moves onto
// ---------------------------------------------------------------------------

fn regions(o: &Options, n: usize, gap: i32) -> Vec<Bill> {
    let clock = Ticks::default();
    let meters: Vec<Arc<Meter>> = (0..n).map(|_| Arc::new(Meter::default())).collect();
    type L = Metered<Delayed<Lossy<net::loopback::MemoryLine>>>;
    let mut lines: Vec<Vec<(usize, L)>> = (0..n).map(|_| Vec::new()).collect();
    let mut seed = 1u64;
    for a in 0..n {
        for b in (a + 1)..n {
            let wire = Wire::default();
            let end = |k: usize, me: usize, seed: u64| {
                Metered::new(
                    Delayed::new(
                        Lossy::new(wire.end(k), o.loss, seed),
                        clock.clone(),
                        o.delay,
                    ),
                    meters[me].clone(),
                )
            };
            lines[a].push((b, end(0, a, seed)));
            lines[b].push((a, end(1, b, seed + 1)));
            seed += 2;
        }
    }
    let start = town(n, gap);
    let grid = Grid::tuned();
    let mut peers: Vec<(net::region::Session<L>, Hands, Bill)> = (0..n)
        .map(|me| {
            (
                net::region::Session::new(me, grid, start.clone(), std::mem::take(&mut lines[me])),
                Hands::new(me as u64 + 1),
                Bill::default(),
            )
        })
        .collect();
    let mut rounds = 0u32;
    let frame = |s: &net::region::Session<L>| s.world.as_ref().map_or(0, |w| w.frame);
    while peers.iter().any(|(s, _, _)| frame(s) < o.frames) && rounds < o.frames * 20 {
        rounds += 1;
        clock.tick();
        for (s, hands, bill) in peers.iter_mut() {
            s.receive();
            if frame(s) >= o.frames {
                continue;
            }
            let input = hands.next();
            let began = Instant::now();
            if s.tick(input) {
                bill.ticks_us.push(began.elapsed().as_micros() as u64);
            } else {
                bill.stalls += 1;
            }
        }
    }
    peers
        .into_iter()
        .map(|(s, _, bill)| {
            let m = &meters[s.me];
            let (sent, got) = m.bytes();
            Bill {
                frames: frame(&s),
                sent,
                got,
                packets: m.packets().0,
                rollbacks: s.rollbacks as u64,
                kinds: std::array::from_fn(|k| {
                    m.sent_by_kind[k + 1].load(std::sync::atomic::Ordering::Relaxed)
                }),
                ..bill
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------

fn row(netcode: &str, layout: &str, n: usize, bills: &[Bill]) {
    let k = bills.len().max(1) as f64;
    let per = |f: &dyn Fn(&Bill) -> f64| bills.iter().map(f).sum::<f64>() / k;
    let up = per(&|b| b.sent as f64 / b.seconds() / 1024.0);
    let down = per(&|b| b.got as f64 / b.seconds() / 1024.0);
    let packets = per(&|b| b.packets as f64 / b.seconds());
    let rollbacks = per(&|b| b.rollbacks as f64 / b.seconds());
    let replayed = per(&|b| b.replayed as f64 / b.seconds());
    let stalls = per(&|b| b.stalls as f64 / b.seconds());
    let mut all: Vec<u64> = bills.iter().flat_map(|b| b.ticks_us.clone()).collect();
    all.sort_unstable();
    let p = |q: f64| {
        all.get(((all.len().max(1) - 1) as f64 * q) as usize)
            .copied()
            .unwrap_or(0)
    };
    let frames = bills.iter().map(|b| b.frames).min().unwrap_or(0);
    println!(
        "{netcode:<8} {layout:<9} {n:>2} {frames:>6} {up:>8.2} {down:>8.2} {packets:>7.0} {rollbacks:>9.1} {replayed:>9.1} {stalls:>7.1} {:>6} {:>6}",
        p(0.5),
        p(0.99)
    );
    if bills.iter().any(|b| b.kinds.iter().any(|&k| k > 0)) {
        let kind = |k: usize| per(&|b| b.kinds[k] as f64 / b.seconds() / 1024.0);
        println!(
            "{:>21} up by kind, KB/s: inputs {:.2}  outcomes {:.2}  wants {:.2}  snapshots {:.2}  books {:.2}",
            "",
            kind(0),
            kind(1),
            kind(2),
            kind(3),
            kind(4)
        );
    }
}

fn main() {
    let o = options();
    println!(
        "{} frames a run, lines {} frames each way ({} ms ping), losing one packet in {}; apart is {} m",
        o.frames,
        o.delay,
        o.delay * 2 * 1000 / sim::TICK_HZ,
        o.loss,
        o.apart
    );
    println!(
        "per peer: KB/s up and down, packets/s up, rollbacks/s, frames replayed/s, ticks held/s, one tick's cost (us, median and 99th)"
    );
    println!(
        "{:<8} {:<9} {:>2} {:>6} {:>8} {:>8} {:>7} {:>9} {:>9} {:>7} {:>6} {:>6}",
        "netcode",
        "layout",
        "n",
        "frames",
        "up",
        "down",
        "pkt/s",
        "rollback",
        "replayed",
        "held",
        "p50",
        "p99"
    );
    for (layout, gap) in [("together", 3), ("apart", o.apart)] {
        for n in 2..=o.peers {
            row("GGRS", layout, n, &ggrs(&o, n, gap));
            row("regions", layout, n, &regions(&o, n, gap));
        }
    }
}
