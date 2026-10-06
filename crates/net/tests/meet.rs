//! The meeting protocol, end to end, with nothing leaving the process.
//!
//! Two rooms on an in-memory board meet, agree who is player one, and then
//! play a real GGRS match over the line the meeting opened. The board loses
//! notes in some of these on purpose: a public broker delivers at most once,
//! and every note in the protocol is repeated until what it asked for has
//! happened.

use net::loopback::{Hall, MemoryBoard, MemoryLine, Wire};
use net::meet::{Rendezvous, Room};
use net::{NetInput, Progress, Seat, handle_requests, p2p};
use sim::{Input, World};

type MemRoom = Room<MemoryBoard, MemoryLine>;

fn room(board: MemoryBoard, wire: &Wire, end: usize, me: u64, terms: &str) -> MemRoom {
    let wire = wire.clone();
    Room::new(board, move || wire.end(end), me, terms)
}

/// Poll both, a frame apart, until both are seated or `frames` run out.
/// `b_from` is the frame the second player opens the link on.
fn meet(
    a: &mut MemRoom,
    b: &mut MemRoom,
    b_from: u64,
    frames: u64,
) -> (Option<Seat>, Option<Seat>, String) {
    let (mut sa, mut sb) = (None, None);
    let mut last = String::new();
    for f in 0..frames {
        let now = f * 16;
        if sa.is_none() {
            match a.poll(now) {
                Progress::Ready(seat) => sa = Some(seat),
                Progress::Waiting(why) => last = why.to_string(),
                Progress::Failed(why) => panic!("a failed: {why}"),
            }
        }
        if sb.is_none() && f >= b_from {
            match b.poll(now) {
                Progress::Ready(seat) => sb = Some(seat),
                Progress::Waiting(_) => {}
                Progress::Failed(why) => panic!("b failed: {why}"),
            }
        }
        if sa.is_some() && sb.is_some() {
            break;
        }
    }
    (sa, sb, last)
}

#[test]
fn whoever_was_waiting_is_player_one_whichever_id_is_lower() {
    // The lower id makes the offer; that must not decide the seats.
    for (id_a, id_b) in [(1, 2), (2, 1)] {
        let hall = Hall::default();
        let wire = Wire::default();
        let mut a = room(hall.board(), &wire, 0, id_a, "t");
        let mut b = room(hall.board(), &wire, 1, id_b, "t");
        let (sa, sb, _) = meet(&mut a, &mut b, 120, 2_000);
        let (sa, sb) = (sa.expect("a seated"), sb.expect("b seated"));
        assert_eq!(sa.handle, 0, "a opened the room first (ids {id_a}, {id_b})");
        assert_eq!(sb.handle, 1);
    }
}

#[test]
fn each_page_has_its_own_clock_and_the_newcomer_is_still_player_two() {
    // Two pages do not share a clock: each counts from when it started, so
    // the newcomer's reads zero while the first has been waiting for half a
    // minute. Whoever decides the seats must not mistake that for a tie. (It
    // did, once: the first two-tab run seated the newcomer first.)
    for (id_a, id_b) in [(1, 2), (2, 1)] {
        let hall = Hall::default();
        let wire = Wire::default();
        let mut a = room(hall.board(), &wire, 0, id_a, "t");
        let mut b = room(hall.board(), &wire, 1, id_b, "t");
        let joins = 30_000 / 16;
        let (mut sa, mut sb) = (None, None);
        for f in 0..joins + 2_000 {
            if sa.is_none()
                && let Progress::Ready(s) = a.poll(f * 16)
            {
                sa = Some(s);
            }
            if f >= joins
                && sb.is_none()
                && let Progress::Ready(s) = b.poll((f - joins) * 16)
            {
                sb = Some(s);
            }
            if sa.is_some() && sb.is_some() {
                break;
            }
        }
        let (sa, sb) = (sa.expect("a seated"), sb.expect("b seated"));
        assert_eq!((sa.handle, sb.handle), (0, 1), "ids {id_a}, {id_b}");
    }
}

#[test]
fn a_board_that_loses_notes_still_gets_them_there() {
    for lose in [2, 3, 5] {
        let hall = Hall::default();
        let wire = Wire::default();
        let mut a = room(hall.losing(lose), &wire, 0, 10, "t");
        let mut b = room(hall.losing(lose), &wire, 1, 20, "t");
        let (sa, sb, _) = meet(&mut a, &mut b, 30, 5_000);
        let (sa, sb) = (sa.expect("a seated"), sb.expect("b seated"));
        assert_ne!(sa.handle, sb.handle, "losing one note in {lose}");
    }
}

#[test]
fn different_terms_never_meet_and_say_why() {
    let hall = Hall::default();
    let wire = Wire::default();
    let mut a = room(hall.board(), &wire, 0, 1, "build-a");
    let mut b = room(hall.board(), &wire, 1, 2, "build-b");
    let (sa, sb, last) = meet(&mut a, &mut b, 0, 600);
    assert!(
        sa.is_none() && sb.is_none(),
        "two different games must not start a match"
    );
    assert!(last.contains("different build"), "said: {last}");
}

#[test]
fn a_friend_who_reloads_is_met_again_as_somebody_new() {
    let hall = Hall::default();
    let wire = Wire::default();
    let mut a = room(hall.board(), &wire, 0, 50, "t");
    // Somebody opens the link, says hello once, and closes the tab before
    // anything else happens. 50 < 60, so `a` makes them an offer nobody will
    // ever answer.
    let mut ghost = room(hall.board(), &wire, 1, 60, "t");
    let _ = a.poll(0);
    let _ = ghost.poll(0);
    drop(ghost);
    // The same person, back after a reload: a new id, on the same link.
    let mut back = room(hall.board(), &wire, 1, 70, "t");
    let (mut sa, mut sb) = (None, None);
    let mut said = Vec::new();
    for f in 1..3_000u64 {
        let now = f * 16;
        if sa.is_none() {
            match a.poll(now) {
                Progress::Ready(s) => sa = Some(s),
                Progress::Waiting(why) => said.push(why),
                Progress::Failed(why) => panic!("{why}"),
            }
        }
        if f > 1_200 && sb.is_none() {
            if let Progress::Ready(s) = back.poll(now) {
                sb = Some(s);
            }
        }
        if sa.is_some() && sb.is_some() {
            break;
        }
    }
    assert!(
        said.iter().any(|w| w.contains("left")),
        "never noticed the ghost go"
    );
    let (sa, sb) = (sa.expect("a seated"), sb.expect("back seated"));
    assert_eq!((sa.handle, sb.handle), (0, 1), "a was there first");
}

#[test]
fn an_answer_to_an_old_offer_is_never_accepted_for_a_new_one() {
    // `a` offers, and `b` stalls before reading it: a frame that takes
    // seconds (shaders compiling, a tab in the background). `a` gives up on
    // it and, when `b` speaks again, offers again on a fresh line. `b` wakes
    // to both offers queued up and answers the old one first. It must then
    // answer the new one on a fresh line of its own, and `a` must not take
    // the old answer for the new offer -- an answer describes one connection,
    // and that one is gone.
    let hall = Hall::default();
    let wire = Wire::default();
    let mut a = room(hall.board(), &wire, 0, 1, "t");
    let mut b = room(hall.board(), &wire, 1, 2, "t");
    let tick = |room: &mut MemRoom, f: u64| match room.poll(f * 16) {
        Progress::Ready(seat) => Some(seat),
        Progress::Failed(why) => panic!("{why}"),
        Progress::Waiting(_) => None,
    };
    let mut seated = (tick(&mut a, 0), tick(&mut b, 0));
    let quiet_until = 20_000 / 16;
    for f in 1..quiet_until + 3_000 {
        if seated.0.is_none() {
            seated.0 = tick(&mut a, f);
        }
        if f >= quiet_until && seated.1.is_none() {
            seated.1 = tick(&mut b, f);
        }
        if seated.0.is_some() && seated.1.is_some() {
            break;
        }
    }
    let (sa, sb) = (seated.0.expect("a seated"), seated.1.expect("b seated"));
    assert_ne!(sa.handle, sb.handle);
}

#[test]
fn a_match_runs_in_sync_over_the_line_the_meeting_opened() {
    let hall = Hall::default();
    let wire = Wire::default();
    let mut a = room(hall.board(), &wire, 0, 7, "t");
    let mut b = room(hall.board(), &wire, 1, 3, "t");
    let (sa, sb, _) = meet(&mut a, &mut b, 10, 2_000);
    let (sa, sb) = (sa.expect("a seated"), sb.expect("b seated"));
    let (ha, hb) = (sa.handle, sb.handle);
    assert_ne!(ha, hb);

    let mut peers = [
        (p2p::start(sa).expect("a session"), World::new(), ha),
        (p2p::start(sb).expect("b session"), World::new(), hb),
    ];
    let mut rng = 0x2545_f491_4f6c_dd1d_u64;
    let mut frames = [0u32; 2];
    for _ in 0..20_000 {
        for (i, (session, world, handle)) in peers.iter_mut().enumerate() {
            session.poll_remote_clients();
            if session.current_state() != net::ggrs::SessionState::Running {
                continue;
            }
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let mut input = Input::aimed((rng & 0x1ff) as u16, (rng >> 20) as u16);
            // Mid-match, whoever is player two changes class: Tab against a
            // person, which travels on the wire and must land on the same
            // frame on both machines, or the desync check below fires.
            if *handle == 1 && frames[i] == 100 {
                input = input.travelling(sim::input::Travel::class(1, sim::Class::Champion));
            }
            let input = NetInput::from(input);
            if session.add_local_input(*handle, input).is_err() {
                continue;
            }
            if let Ok(requests) = session.advance_frame() {
                handle_requests(world, requests);
                frames[i] += 1;
            }
            for event in session.events() {
                if let net::ggrs::GgrsEvent::DesyncDetected { frame, .. } = event {
                    panic!("desync at frame {frame}");
                }
            }
        }
        if frames.iter().all(|&f| f >= 300) {
            break;
        }
    }
    assert!(frames.iter().all(|&f| f >= 300), "frames: {frames:?}");
    for (_, world, handle) in &peers {
        assert_eq!(
            world.players[1].class,
            sim::Class::Champion,
            "player two's class change, as seen by player {}",
            handle + 1
        );
    }
}

/// The desktop's `--port`/`--peer`, over real UDP on this machine.
///
/// The regression this guards: seats used to come from ordering the two
/// addresses, and each machine only knew its own as `127.0.0.1`, so on two
/// real machines both claimed player one. Now nothing about an address enters
/// into it -- these two even share an IP -- and the hello settles it.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn two_desktops_meet_over_udp_and_take_different_seats() {
    use net::direct::Direct;
    let a = Direct::bind(0).expect("bind a");
    let b = Direct::bind(0).expect("bind b");
    let at = |port| std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let (pa, pb) = (a.local_port(), b.local_port());
    let mut a = a.toward(at(pb), 111, "t");
    let mut b = b.toward(at(pa), 222, "t");
    let begun = std::time::Instant::now();
    let now = || begun.elapsed().as_millis() as u64;
    let pause = || std::thread::sleep(std::time::Duration::from_millis(2));

    // a opens the room; b opens it a moment later, so a was waiting.
    let mut sa = None;
    let mut sb = None;
    while sa.is_none() && now() < 10_000 {
        if let Progress::Ready(s) = a.poll(now()) {
            sa = Some(s);
        }
        if now() > 200
            && let Progress::Ready(s) = b.poll(now())
        {
            sb = Some(s);
        }
        pause();
    }
    let sa = sa.expect("a seated");
    let handle_a = sa.handle;
    // a is seated once it has b's answer; b, once a's session speaks.
    let mut session = p2p::start(sa).expect("a session");
    while sb.is_none() && now() < 10_000 {
        session.poll_remote_clients();
        if let Progress::Ready(s) = b.poll(now()) {
            sb = Some(s);
        }
        pause();
    }
    let sb = sb.expect("b seated");
    assert_eq!((handle_a, sb.handle), (0, 1));
}

/// Two desktops over real WebRTC: str0m on both ends, on this machine, meeting
/// through a sealed board. The same line a desktop uses to join a page's room;
/// here it meets itself, so the test needs no browser.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn two_desktops_meet_over_webrtc_through_a_sealed_room_and_play() {
    use net::native::RtcLine;
    use net::seal::{RoomKey, Sealed};
    let hall = Hall::default();
    let key = RoomKey::new("k3x9qw", "a-long-random-secret");
    let mut a = Room::new(Sealed::new(hall.board(), key.clone()), RtcLine::new, 5, "t");
    let mut b = Room::new(Sealed::new(hall.board(), key), RtcLine::new, 9, "t");
    let begun = std::time::Instant::now();
    let now = || begun.elapsed().as_millis() as u64;
    let (mut sa, mut sb) = (None, None);
    while (sa.is_none() || sb.is_none()) && now() < 20_000 {
        if sa.is_none() {
            match a.poll(now()) {
                Progress::Ready(s) => sa = Some(s),
                Progress::Failed(why) => panic!("a: {why}"),
                Progress::Waiting(_) => {}
            }
        }
        if now() > 300 && sb.is_none() {
            match b.poll(now()) {
                Progress::Ready(s) => sb = Some(s),
                Progress::Failed(why) => panic!("b: {why}"),
                Progress::Waiting(_) => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let (sa, sb) = (sa.expect("a seated"), sb.expect("b seated"));
    assert_eq!((sa.handle, sb.handle), (0, 1));
    play(sa, sb, 300);
}

/// Run a real GGRS match between two seats for `frames` frames each, at
/// whatever speed the line allows, and fail on a desync.
#[cfg(not(target_arch = "wasm32"))]
fn play(sa: Seat, sb: Seat, frames: u32) {
    let (ha, hb) = (sa.handle, sb.handle);
    let mut peers = [
        (p2p::start(sa).expect("a session"), World::new(), ha),
        (p2p::start(sb).expect("b session"), World::new(), hb),
    ];
    let mut rng = 0x2545_f491_4f6c_dd1d_u64;
    let mut done = [0u32; 2];
    let begun = std::time::Instant::now();
    while done.iter().any(|&f| f < frames) && begun.elapsed().as_secs() < 60 {
        for (i, (session, world, handle)) in peers.iter_mut().enumerate() {
            session.poll_remote_clients();
            if session.current_state() != net::ggrs::SessionState::Running {
                continue;
            }
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let input = NetInput::from(Input::aimed((rng & 0x1ff) as u16, (rng >> 20) as u16));
            if session.add_local_input(*handle, input).is_err() {
                continue;
            }
            if let Ok(requests) = session.advance_frame() {
                handle_requests(world, requests);
                done[i] += 1;
            }
            for event in session.events() {
                if let net::ggrs::GgrsEvent::DesyncDetected { frame, .. } = event {
                    panic!("desync at frame {frame}");
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(done.iter().all(|&f| f >= frames), "frames: {done:?}");
}
