//! **The region books online, through rollback**: two peers keep the books
//! exactly as the game's online driver does (`crates/game/src/online.rs`) --
//! the gate before every frame, the crowd's aggregates fed by due frame, the
//! books kept after every advance, the leaders' posts -- over a line that
//! drops a third of the polls, so rollbacks are constant. Their worlds must
//! agree on every confirmed frame, and GGRS must never see a desync, with or
//! without a trip to another arena.
//!
//! What this caught (2026-10-22), each a desync a few seconds into a match
//! with friends, felt as "it desynced when I changed arenas":
//!
//! - the books kept *before* the advance, from frames the rollback had not
//!   yet corrected (`Ledger::inputs_through` is what the gate needs instead);
//! - the "nobody there" aggregate of a stream with no peer behind it said on
//!   each machine's wall clock (`Ledger::confirm_through` says it now);
//! - the books begun from whichever frame a peer first confirmed
//!   (`Ledger::begin`, from the agreed start).

use net::loopback::Hall;
use net::meet::{Rendezvous, Room};
use net::{NetInput, Progress, Seat, handle_requests_kept, p2p};
use sim::input::Travel;
use sim::{Input, World};
use std::collections::HashMap;

/// `n` rooms for `n` players on one hall and one switchboard, each opening
/// the link a little after the last; their seats, by handle.
fn seats(n: usize) -> Vec<Seat> {
    use net::loopback::Switchboard;
    let hall = Hall::default();
    let switch = Switchboard::default();
    let mut rooms: Vec<_> = (0..n)
        .map(|i| {
            let switch = switch.clone();
            Room::new(
                hall.board(),
                move || switch.line(),
                11 * (i as u64 + 1),
                "t",
            )
            .with_players(n)
        })
        .collect();
    let mut seats: Vec<Option<Seat>> = (0..n).map(|_| None).collect();
    for f in 0..6_000u64 {
        for (i, room) in rooms.iter_mut().enumerate() {
            if seats[i].is_some() || f < i as u64 * 150 {
                continue;
            }
            if let Progress::Ready(s) = room.poll(f * 16) {
                seats[i] = Some(s);
            }
        }
        if seats.iter().all(Option::is_some) {
            break;
        }
    }
    let mut seats: Vec<Seat> = seats
        .into_iter()
        .enumerate()
        .map(|(i, s)| s.unwrap_or_else(|| panic!("room {i} never seated")))
        .collect();
    seats.sort_by_key(|s| s.handle);
    seats
}

/// The start the owner's replay began from: the town, two Champions.
const HEARTH: &str = "p1=champion p2=champion arena=hearth temper=0";

/// Play `frames` frames on `n` peers, `trip(handle, frame)` asked for by
/// that player on that frame of theirs; `seed` decides which polls drop, and
/// `slow` is how many polls in eight the last peer drops (the rest drop one
/// in three): a machine behind the others, as a slow one is.
fn play(n: usize, slow: u64, seed: u64, frames: u32, trip: impl Fn(usize, u32) -> Travel) {
    let start = sim::replay::Start::parse(&format!("{HEARTH} seats={n}"))
        .expect("start")
        .world();
    struct Peer {
        session: net::ggrs::P2PSession<net::SessionConfig>,
        world: World,
        handle: usize,
        done: u32,
        ledger: regions::Ledger,
        roster: Vec<u64>,
        post: net::crowd::Post,
        /// Each frame's checksum, the last word on it: a re-advance overwrites.
        sums: HashMap<u32, u64>,
    }
    let mut peers: Vec<Peer> = seats(n)
        .into_iter()
        .map(|s| {
            let handle = s.handle;
            let roster = s.roster.clone();
            let post = s.post.clone();
            // As the game does as a match starts.
            let mut ledger = regions::Ledger::new(&start, 0, false);
            ledger.sit(handle);
            ledger.begin(&start);
            Peer {
                session: p2p::start(s).expect("session"),
                world: start.clone(),
                handle,
                done: 0,
                ledger,
                roster,
                post,
                sums: HashMap::new(),
            }
        })
        .collect();
    let grid = sim::region::Grid::tuned();
    let epoch_frames = sim::tuning::crowd_epoch();
    let mut rng = 0x2545_f491_4f6c_dd1d_u64 ^ seed.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    for _ in 0..400_000 {
        for p in peers.iter_mut() {
            // A third of the polls never happen: the line is late, often.
            let drop = if p.handle == n - 1 && slow > 0 {
                (next() >> 40) % 8 < slow
            } else {
                (next() >> 40) % 3 == 0
            };
            if drop {
                continue;
            }
            p.session.poll_remote_clients();
            for event in p.session.events() {
                if let net::ggrs::GgrsEvent::DesyncDetected { frame, .. } = event {
                    panic!("seed {seed}: desync at frame {frame}");
                }
            }
            if p.session.current_state() != net::ggrs::SessionState::Running || p.done >= frames {
                continue;
            }
            if !net::may_advance(&mut p.ledger, p.session.confirmed_frame(), &p.world) {
                continue;
            }
            let r = next();
            let input = Input::aimed((r & 0x1ff) as u16, (r >> 20) as u16)
                .travelling(trip(p.handle, p.done));
            if p.session
                .add_local_input(p.handle, NetInput::from(input))
                .is_err()
            {
                continue;
            }
            let Ok(requests) = p.session.advance_frame() else {
                continue;
            };
            let sums = &mut p.sums;
            handle_requests_kept(&mut p.world, requests, &mut p.ledger, |_, _, after| {
                sums.insert(after.frame, after.checksum());
            });
            // Frames made, not calls answered: a session that may not advance
            // still answers, with nothing in it.
            p.done = p.world.frame;
            if let Ok(confirmed) = u32::try_from(p.session.confirmed_frame()) {
                p.ledger.confirm_through(confirmed, &p.world);
            }
            // The leader's duty, as the game does it.
            let epoch = (p.world.frame / epoch_frames) as u16;
            let members: Vec<net::cells::Member> = p
                .roster
                .iter()
                .enumerate()
                .map(|(seat, id)| net::cells::Member { seat, id: *id })
                .collect();
            for a in p.ledger.summed() {
                if !net::cells::leads(&p.world, &grid, &members, a.region, epoch, p.handle) {
                    continue;
                }
                for other in (0..n).filter(|o| *o != p.handle) {
                    let next_door =
                        a.region.neighbours().iter().any(|n| {
                            net::cells::leads(&p.world, &grid, &members, *n, epoch, other)
                        });
                    if next_door {
                        p.post.post(net::Peer::Line(other as u8), a);
                    }
                }
            }
            for a in p.post.collect(p.world.frame, epoch_frames) {
                p.ledger.offer(a);
            }
        }
        if peers.iter().all(|p| p.done >= frames) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_micros(300));
    }
    let done: Vec<u32> = peers.iter().map(|p| p.done).collect();
    assert!(
        done.iter().all(|&d| d >= frames),
        "seed {seed}: stalled at {done:?}"
    );
    // Every frame both have confirmed and played again since -- short of the
    // last few, which the peer that stopped first never re-advanced.
    let confirmed = peers
        .iter()
        .map(|p| p.session.confirmed_frame())
        .min()
        .unwrap_or(0) as u32;
    let parted =
        (1..confirmed.saturating_sub(10)).find(|f| peers[0].sums.get(f) != peers[1].sums.get(f));
    assert_eq!(parted, None, "seed {seed}: the worlds parted at that frame");
}

#[test]
fn the_books_keep_two_peers_in_step_through_rollback() {
    for seed in 0..8 {
        play(2, 0, seed, 600, |_, _| Travel::NONE);
    }
}

#[test]
fn a_trip_from_the_town_under_rollback_does_not_desync() {
    for seed in 0..8 {
        for who in 0..2 {
            play(2, 0, seed, 600, |h, f| {
                if h == who && f == 300 {
                    Travel::VERSUS
                } else {
                    Travel::NONE
                }
            });
        }
    }
}

#[test]
fn four_with_one_slow_machine_stay_in_step() {
    for seed in 0..4 {
        play(4, 6, seed, 900, |h, f| {
            if h == 3 && f == 400 {
                Travel::VERSUS
            } else {
                Travel::NONE
            }
        });
    }
}
