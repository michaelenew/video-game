//! Going somewhere else mid-match, against a person: H, Shift+H and N.
//!
//! Online is pass-the-keyboard with the friend on another machine, so
//! anything one player can do to the fight on a shared keyboard has to happen
//! on both machines, on the same frame. These play a real rollback match over
//! an in-memory line, with one side asking for a trip mid-match, and fail on
//! the session's own desync check.

use net::loopback::{Hall, MemoryBoard, MemoryLine, Wire};
use net::meet::{Rendezvous, Room};
use net::{NetInput, Progress, Seat, handle_requests, p2p};
use sim::input::Travel;
use sim::{Input, World};

fn seats() -> (Seat, Seat) {
    let hall = Hall::default();
    let wire = Wire::default();
    let room = |board: MemoryBoard, end: usize, me: u64| -> Room<MemoryBoard, MemoryLine> {
        let wire = wire.clone();
        Room::new(board, move || wire.end(end), me, "t")
    };
    let (mut a, mut b) = (room(hall.board(), 0, 7), room(hall.board(), 1, 3));
    let (mut sa, mut sb) = (None, None);
    for f in 0..2_000u64 {
        if sa.is_none()
            && let Progress::Ready(s) = a.poll(f * 16)
        {
            sa = Some(s);
        }
        if f >= 10
            && sb.is_none()
            && let Progress::Ready(s) = b.poll(f * 16)
        {
            sb = Some(s);
        }
    }
    (sa.expect("a seated"), sb.expect("b seated"))
}

/// Play `frames` frames each; `trip(handle, frame)` is what that player asks
/// for on that frame of theirs. The two worlds at the end, by handle.
fn play(frames: u32, trip: impl Fn(usize, u32) -> Travel) -> [World; 2] {
    play_with_hitch(frames, trip, None)
}

/// [`play`], with player `who` frozen for `for_ms` of wall-clock time after
/// its frame `at`: not polled, not sending, the way a page is while it
/// compiles the shaders of an arena it has not drawn before.
fn play_with_hitch(
    frames: u32,
    trip: impl Fn(usize, u32) -> Travel,
    hitch: Option<(usize, u32, u64)>,
) -> [World; 2] {
    let mut frozen_until: Option<std::time::Instant> = None;
    let (sa, sb) = seats();
    let mut peers = [sa, sb].map(|s| {
        let h = s.handle;
        (p2p::start(s).expect("session"), World::new(), h, 0u32)
    });
    let mut rng = 0x2545_f491_4f6c_dd1d_u64;
    for _ in 0..400_000 {
        if frozen_until.is_some_and(|t| std::time::Instant::now() < t) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        for (session, world, handle, done) in peers.iter_mut() {
            if let Some((who, at, for_ms)) = hitch
                && *handle == who
                && *done == at
            {
                let until = *frozen_until.get_or_insert_with(|| {
                    std::time::Instant::now() + std::time::Duration::from_millis(for_ms)
                });
                if std::time::Instant::now() < until {
                    continue;
                }
            }
            session.poll_remote_clients();
            for event in session.events() {
                match event {
                    net::ggrs::GgrsEvent::DesyncDetected { frame, .. } => {
                        panic!("desync at frame {frame}")
                    }
                    net::ggrs::GgrsEvent::Disconnected { .. } => panic!("disconnected"),
                    _ => {}
                }
            }
            if session.current_state() != net::ggrs::SessionState::Running {
                continue;
            }
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let input = Input::aimed((rng & 0x1ff) as u16, (rng >> 20) as u16)
                .travelling(trip(*handle, *done));
            if session
                .add_local_input(*handle, NetInput::from(input))
                .is_err()
            {
                continue;
            }
            if let Ok(requests) = session.advance_frame() {
                handle_requests(world, requests);
                *done += 1;
            }
        }
        if peers.iter().all(|p| p.3 >= frames) {
            break;
        }
    }
    assert!(peers.iter().all(|p| p.3 >= frames), "stalled");
    let mut out = [World::new(), World::new()];
    for (_, world, handle, _) in peers {
        out[handle] = world;
    }
    out
}

#[test]
fn a_hunt_and_back_from_either_side() {
    for who in 0..2 {
        let worlds = play(400, |h, f| match (h == who, f) {
            (true, 100) => Travel::hunt(sim::species::SpeciesId::RIDGEBACK),
            (true, 250) => Travel::VERSUS,
            _ => Travel::NONE,
        });
        assert!(worlds.iter().all(|w| !w.hunting()));
    }
}

#[test]
fn every_creature_from_either_side() {
    for s in sim::species::all() {
        for who in 0..2 {
            let worlds = play(250, |h, f| {
                if h == who && f == 100 {
                    Travel::hunt(s.id)
                } else {
                    Travel::NONE
                }
            });
            assert!(worlds.iter().all(|w| w.hunting()), "{:?}", s.id);
        }
    }
}

#[test]
fn every_course_from_either_side() {
    for c in sim::course::all() {
        for who in 0..2 {
            let worlds = play(250, |h, f| {
                if h == who && f == 100 {
                    Travel::arena(c.arena)
                } else {
                    Travel::NONE
                }
            });
            assert!(worlds.iter().all(|w| w.arena == c.arena));
        }
    }
}

#[test]
fn a_long_hitch_after_a_trip_is_not_a_disconnect() {
    // The first frame of an arena a page has not drawn before can take
    // seconds: every new material is a shader to compile, and WebGL compiles
    // on the main thread. Meanwhile it sends nothing. The session used to
    // count two seconds of that as a disconnect -- the side that pressed H
    // said "your friend disconnected", and went on with blank inputs for them,
    // which the other side then saw as a desync.
    for who in 0..2 {
        let worlds = play_with_hitch(
            300,
            |h, f| {
                if h == 0 && f == 100 {
                    Travel::hunt(sim::species::SpeciesId::RIDGEBACK)
                } else {
                    Travel::NONE
                }
            },
            Some((who, 101, 3_500)),
        );
        assert!(worlds.iter().all(|w| w.hunting()));
    }
}

#[test]
fn pause_step_and_restart_from_either_side() {
    for who in 0..2 {
        let worlds = play(400, |h, f| match (h == who, f) {
            (true, 100) => Travel::PAUSE,
            (true, 130) => Travel::STEP,
            (true, 160) => Travel::PAUSE,
            (true, 200) => Travel::RESTART,
            (true, 260) => Travel::PAUSE,
            _ => Travel::NONE,
        });
        assert!(worlds.iter().all(|w| w.paused), "paused at the end on both");
    }
}
