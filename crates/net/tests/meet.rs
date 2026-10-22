//! The meeting protocol, end to end, with nothing leaving the process.
//!
//! Two rooms on an in-memory board meet, agree who is player one, and then
//! play a real GGRS match over the line the meeting opened. The board loses
//! notes in some of these on purpose: a public broker delivers at most once,
//! and every note in the protocol is repeated until what it asked for has
//! happened.

use net::loopback::{Hall, MemoryBoard, MemoryLine, Wire};
use net::meet::{Rendezvous, Room};
use net::{NetInput, Progress, Seat, handle_requests, handle_requests_kept, p2p};
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

/// `n` rooms for `n` players on one hall and one switchboard, the first
/// opening the link first and each of the rest a little after.
fn mesh(n: usize, ids: &[u64]) -> Vec<Seat> {
    use net::loopback::Switchboard;
    let hall = Hall::default();
    let switch = Switchboard::default();
    let mut rooms: Vec<_> = ids
        .iter()
        .map(|&id| {
            let switch = switch.clone();
            Room::new(hall.board(), move || switch.line(), id, "t").with_players(n)
        })
        .collect();
    let mut seats: Vec<Option<Seat>> = (0..n).map(|_| None).collect();
    for f in 0..6_000u64 {
        for (i, room) in rooms.iter_mut().enumerate() {
            if seats[i].is_some() || f < i as u64 * 150 {
                continue;
            }
            match room.poll(f * 16) {
                Progress::Ready(s) => seats[i] = Some(s),
                Progress::Failed(why) => panic!("room {i} failed: {why}"),
                Progress::Waiting(_) => {}
            }
        }
        if seats.iter().all(Option::is_some) {
            break;
        }
    }
    seats
        .into_iter()
        .enumerate()
        .map(|(i, s)| s.unwrap_or_else(|| panic!("room {i} never seated")))
        .collect()
}

#[test]
fn three_and_four_meet_and_whoever_came_first_sits_first() {
    for ids in [
        vec![30, 10, 20],
        vec![1, 2, 3],
        vec![40, 30, 20, 10],
        vec![7, 99, 3, 50],
    ] {
        let n = ids.len();
        let seats = mesh(n, &ids);
        let handles: Vec<usize> = seats.iter().map(|s| s.handle).collect();
        assert_eq!(handles, (0..n).collect::<Vec<_>>(), "ids {ids:?}");
        for s in &seats {
            assert_eq!(s.players, n);
            let mut far: Vec<usize> = s.remotes.iter().map(|(h, _)| *h).collect();
            far.sort();
            let want: Vec<usize> = (0..n).filter(|&h| h != s.handle).collect();
            assert_eq!(far, want, "seat {} reaches everybody else", s.handle);
        }
    }
}

#[test]
fn a_room_for_four_waits_for_the_fourth() {
    use net::loopback::Switchboard;
    let hall = Hall::default();
    let switch = Switchboard::default();
    let mut rooms: Vec<_> = [1u64, 2, 3]
        .into_iter()
        .map(|id| {
            let switch = switch.clone();
            Room::new(hall.board(), move || switch.line(), id, "t").with_players(4)
        })
        .collect();
    let mut said = String::new();
    for f in 0..600u64 {
        for room in rooms.iter_mut() {
            match room.poll(f * 16) {
                Progress::Ready(_) => panic!("three started a match for four"),
                Progress::Failed(why) => panic!("{why}"),
                Progress::Waiting(why) => said = why.to_string(),
            }
        }
    }
    assert!(said.contains("one more"), "said: {said}");
}

#[test]
fn four_play_a_match_in_sync_over_the_mesh() {
    let seats = mesh(4, &[11, 22, 33, 44]);
    let mut peers: Vec<_> = seats
        .into_iter()
        .map(|s| {
            let h = s.handle;
            let start = World::with_classes([sim::Class::Bulwark; 4]);
            (p2p::start(s).expect("session"), start, h, 0u32)
        })
        .collect();
    let mut rng = 0x2545_f491_4f6c_dd1d_u64;
    for _ in 0..40_000 {
        for (session, world, handle, frames) in peers.iter_mut() {
            session.poll_remote_clients();
            if session.current_state() != net::ggrs::SessionState::Running {
                continue;
            }
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let mut input = Input::aimed((rng & 0x1ff) as u16, (rng >> 20) as u16);
            // Player four changes class mid-match: it travels on the wire
            // and must land on the same frame on all four machines.
            if *handle == 3 && *frames == 100 {
                input = input.travelling(sim::input::Travel::class(3, sim::Class::Champion));
            }
            if session
                .add_local_input(*handle, NetInput::from(input))
                .is_err()
            {
                continue;
            }
            if let Ok(requests) = session.advance_frame() {
                handle_requests(world, requests);
                *frames += 1;
            }
            for event in session.events() {
                if let net::ggrs::GgrsEvent::DesyncDetected { frame, .. } = event {
                    panic!("desync at frame {frame}");
                }
            }
        }
        if peers.iter().all(|p| p.3 >= 300) {
            break;
        }
    }
    let frames: Vec<u32> = peers.iter().map(|p| p.3).collect();
    assert!(frames.iter().all(|&f| f >= 300), "frames: {frames:?}");
    for (_, world, handle, _) in &peers {
        assert_eq!(world.present(), 4);
        assert_eq!(
            world.players[3].class,
            sim::Class::Champion,
            "player four's class change, as seen by player {}",
            handle + 1
        );
    }
}

/// **Somebody leaves, and the rest play on in step without them.** Three on
/// the mesh; the third stops answering a few seconds in. The two left behind
/// give the seat up on the same frame -- GGRS sees to that -- and from it the
/// third fighter is out of the world on both machines (`World::away`),
/// rather than standing there as a target with its look turned to zero. The
/// two worlds agree, frame for frame, all the way to the end.
#[test]
fn three_play_and_one_leaves_and_the_two_agree() {
    use std::time::{Duration, Instant};
    const LEAVES_AT: u32 = 200;
    const UNTIL: u32 = 700;
    let seats = mesh(3, &[11, 22, 33]);
    let mut peers: Vec<_> = seats
        .into_iter()
        .map(|s| {
            let h = s.handle;
            let start = World::with_classes([sim::Class::Champion; 3]);
            let session = p2p::start_with(s, 2, Duration::from_millis(500)).expect("session");
            (session, start, h, 0u32, false, None::<u64>)
        })
        .collect();
    let mut rng = 0x2545_f491_4f6c_dd1d_u64;
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(30) {
        let mut moved = false;
        for (session, world, handle, frames, saw_leave, at_end) in peers.iter_mut() {
            // The third player is gone: never polled again.
            if *handle == 2 && *frames >= LEAVES_AT {
                continue;
            }
            session.poll_remote_clients();
            for event in session.events() {
                match event {
                    net::ggrs::GgrsEvent::DesyncDetected { frame, .. } => {
                        panic!("desync at frame {frame}")
                    }
                    net::ggrs::GgrsEvent::Disconnected { .. } => *saw_leave = true,
                    _ => {}
                }
            }
            if session.current_state() != net::ggrs::SessionState::Running || *frames >= UNTIL {
                continue;
            }
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let input = Input::aimed((rng & 0x1ff) as u16, (rng >> 20) as u16);
            if session
                .add_local_input(*handle, NetInput::from(input))
                .is_err()
            {
                continue;
            }
            if let Ok(requests) = session.advance_frame() {
                // A call that was over the prediction window plays nothing:
                // the world's own frame is what counts.
                handle_requests(world, requests);
                moved |= world.frame != *frames;
                *frames = world.frame;
                if *frames == UNTIL {
                    *at_end = Some(world.checksum());
                }
            }
        }
        let stayed = peers.iter().filter(|p| p.2 != 2);
        if stayed.clone().all(|p| p.3 >= UNTIL && p.4) {
            break;
        }
        if !moved {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    let stayed: Vec<_> = peers.iter().filter(|p| p.2 != 2).collect();
    for (_, world, handle, frames, saw_leave, _) in &stayed {
        assert!(
            *saw_leave,
            "player {} never heard the third leave",
            handle + 1
        );
        assert_eq!(*frames, UNTIL, "player {} stalled", handle + 1);
        assert!(
            world.away.has(2),
            "player {} still has the third in the fight",
            handle + 1
        );
        assert!(!world.is_present(2));
        assert_eq!(world.players[2].health, 0);
    }
    assert_eq!(
        stayed[0].5, stayed[1].5,
        "the two who stayed disagree about the world without the third"
    );
}

/// **Four cells' worth of leaders on the mesh** (`net::cells`, `net::crowd`,
/// 0018 §5): four play in one session; each keeps the region books, and at
/// every epoch the leaders of each live region post its aggregate over the
/// lines to the leaders next door, who collect it by agreement. In one
/// session everybody sums the same confirmed world, so what arrives over the
/// wire must equal what each ledger summed itself, two leaders must never
/// be at odds, and every peer must follow the same streams.
#[test]
fn leaders_post_their_aggregates_over_the_mesh_and_the_rest_collect_them_in_agreement() {
    use sim::tuning as t;
    let seats = mesh(4, &[11, 22, 33, 44]);
    let mut peers: Vec<_> = seats
        .into_iter()
        .map(|s| {
            let h = s.handle;
            let roster = s.roster.clone();
            let post = s.post.clone();
            let start = World::with_classes([sim::Class::Bulwark; 4]);
            let ledger = regions::Ledger::new(&start, h, false);
            (
                p2p::start(s).expect("session"),
                start,
                h,
                ledger,
                post,
                roster,
                0u32,
                0u32,
            )
        })
        .collect();
    let grid = sim::region::Grid::tuned();
    let epoch_frames = t::crowd_epoch();
    let until = 4 * epoch_frames + 30;
    for _ in 0..60_000 {
        for (session, world, handle, ledger, post, roster, heard, posted) in peers.iter_mut() {
            session.poll_remote_clients();
            if session.current_state() != net::ggrs::SessionState::Running || world.frame >= until {
                continue;
            }
            if !net::may_advance(ledger, session.confirmed_frame(), world) {
                continue;
            }
            let input = Input::aimed(Input::W, (*handle as u16) << 14);
            if session
                .add_local_input(*handle, NetInput::from(input))
                .is_err()
            {
                continue;
            }
            if let Ok(requests) = session.advance_frame() {
                handle_requests_kept(world, requests, ledger, |_, _, _| {});
                if let Ok(confirmed) = u32::try_from(session.confirmed_frame()) {
                    ledger.confirm_through(confirmed, world);
                }
                // The leader's duty, as the game does it.
                let epoch = (world.frame / epoch_frames) as u16;
                let members: Vec<net::cells::Member> = roster
                    .iter()
                    .enumerate()
                    .map(|(seat, id)| net::cells::Member { seat, id: *id })
                    .collect();
                for a in ledger.summed() {
                    if !net::cells::leads(world, &grid, &members, a.region, epoch, *handle) {
                        continue;
                    }
                    for other in (0..4).filter(|o| o != handle) {
                        let next_door =
                            a.region.neighbours().iter().any(|n| {
                                net::cells::leads(world, &grid, &members, *n, epoch, other)
                            });
                        if next_door {
                            post.post(net::Peer::Line(other as u8), a);
                            *posted += 1;
                        }
                    }
                }
                for a in post.collect(world.frame, epoch_frames) {
                    // What came over the wire is what this ledger summed
                    // itself for the same region and epoch, byte for byte.
                    let own = ledger
                        .summed()
                        .iter()
                        .find(|s| s.region == a.region && s.epoch == a.epoch);
                    if let Some(own) = own {
                        assert_eq!(*own, a, "player {} heard a different sum", *handle + 1);
                    }
                    ledger.offer(a);
                    *heard += 1;
                }
            }
            for event in session.events() {
                if let net::ggrs::GgrsEvent::DesyncDetected { frame, .. } = event {
                    panic!("desync at frame {frame}");
                }
            }
        }
        if peers.iter().all(|p| p.1.frame >= until) {
            break;
        }
    }
    let frames: Vec<u32> = peers.iter().map(|p| p.1.frame).collect();
    assert!(frames.iter().all(|&f| f >= until), "frames: {frames:?}");
    let posted: u32 = peers.iter().map(|p| p.7).sum();
    let heard: u32 = peers.iter().map(|p| p.6).sum();
    assert!(posted > 0, "nobody led a cell");
    assert!(heard > 0, "nothing arrived over the lines: {posted} posted");
    for (_, world, handle, ledger, post, ..) in &peers {
        assert_eq!(
            post.disputes(),
            0,
            "player {} saw leaders at odds",
            handle + 1
        );
        assert!(
            !ledger.followed().is_empty(),
            "player {} follows nothing",
            handle + 1
        );
        let _ = world;
    }
    // Every peer follows the same streams: the field is a function of the
    // world, and the world is one.
    let first: Vec<_> = peers[0].3.followed().iter().map(|f| f.region).collect();
    for (_, _, handle, ledger, ..) in &peers[1..] {
        let mine: Vec<_> = ledger.followed().iter().map(|f| f.region).collect();
        assert_eq!(mine, first, "player {} follows other streams", handle + 1);
    }
}

/// **A friend's inputs arrive late, once, and the match goes on.** Two play,
/// keeping the region books as the game does; a little way in, player two's
/// machine goes quiet for a fifth of a second -- one slow packet on an
/// ordinary line -- so player one runs ahead of what is confirmed by more
/// than the region gate's delay. When the inputs arrive, both carry on.
///
/// They did not, once: the books were brought up to the confirmed frame only
/// after a frame was advanced, so the gate, shut for the late inputs, never
/// heard they had come and never opened again. Every match with a friend
/// froze a few seconds in, on the first hiccup (2026-10-22).
#[test]
fn a_late_packet_does_not_shut_the_region_gate_for_good() {
    use std::time::{Duration, Instant};
    const QUIET_AT: u32 = 100;
    const QUIET: Duration = Duration::from_millis(200);
    const UNTIL: u32 = 400;
    let seats = mesh(2, &[11, 22]);
    let mut peers: Vec<_> = seats
        .into_iter()
        .map(|s| {
            let h = s.handle;
            let start = World::with_classes([sim::Class::Champion; 2]);
            let ledger = regions::Ledger::new(&start, h, false);
            (p2p::start(s).expect("session"), start, h, ledger)
        })
        .collect();
    let mut quiet_until: Option<Instant> = None;
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(20) {
        for (session, world, handle, ledger) in peers.iter_mut() {
            if *handle == 1 {
                if world.frame == QUIET_AT && quiet_until.is_none() {
                    quiet_until = Some(Instant::now() + QUIET);
                }
                if quiet_until.is_some_and(|t| Instant::now() < t) {
                    continue;
                }
            }
            session.poll_remote_clients();
            if session.current_state() != net::ggrs::SessionState::Running || world.frame >= UNTIL {
                continue;
            }
            if !net::may_advance(ledger, session.confirmed_frame(), world) {
                continue;
            }
            let input = Input::aimed(Input::W, (*handle as u16) << 14);
            if session
                .add_local_input(*handle, NetInput::from(input))
                .is_err()
            {
                continue;
            }
            if let Ok(requests) = session.advance_frame() {
                handle_requests_kept(world, requests, ledger, |_, _, _| {});
                if let Ok(confirmed) = u32::try_from(session.confirmed_frame()) {
                    ledger.confirm_through(confirmed, world);
                }
            }
        }
        if peers.iter().all(|p| p.1.frame >= UNTIL) {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(quiet_until.is_some(), "player two never went quiet");
    let frames: Vec<u32> = peers.iter().map(|p| p.1.frame).collect();
    assert!(
        frames.iter().all(|&f| f >= UNTIL),
        "the match froze after a late packet: frames {frames:?}"
    );
}

/// **Somebody closes the game, and the rest play on at once.** Three on the
/// mesh with the session's real silence timeout (fifteen seconds); the third
/// hangs up every line, the way closing a page closes its data channels. The
/// two left behind must give the seat up within a couple of seconds -- not
/// after fifteen, frozen, which is what four friends got (2026-10-22) -- and
/// agree, frame for frame, on the world without them.
#[test]
fn a_player_who_closes_the_game_is_let_go_at_once() {
    use net::loopback::Switchboard;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    const LEAVES_AT: u32 = 200;
    const UNTIL: u32 = 500;
    let hall = Hall::default();
    let switch = Switchboard::default();
    // Every line each room makes, so the third's can be hung up.
    let made: Vec<Arc<Mutex<Vec<u32>>>> = (0..3).map(|_| Arc::default()).collect();
    let mut rooms: Vec<_> = [11u64, 22, 33]
        .into_iter()
        .zip(&made)
        .map(|(id, mine)| {
            let switch = switch.clone();
            let mine = mine.clone();
            Room::new(
                hall.board(),
                move || {
                    let line = switch.line();
                    mine.lock().unwrap().push(line.serial());
                    line
                },
                id,
                "t",
            )
            .with_players(3)
        })
        .collect();
    let mut seats: Vec<Option<Seat>> = vec![None, None, None];
    for f in 0..6_000u64 {
        for (i, room) in rooms.iter_mut().enumerate() {
            if seats[i].is_none()
                && f >= i as u64 * 150
                && let Progress::Ready(s) = room.poll(f * 16)
            {
                seats[i] = Some(s);
            }
        }
        if seats.iter().all(Option::is_some) {
            break;
        }
    }
    let mut peers: Vec<_> = seats
        .into_iter()
        .enumerate()
        .map(|(room, s)| {
            let s = s.expect("seated");
            let (h, post) = (s.handle, s.post.clone());
            let start = World::with_classes([sim::Class::Champion; 3]);
            let session = p2p::start(s).expect("session");
            (
                session,
                start,
                h,
                0u32,
                post,
                room,
                None::<Instant>,
                HashMapSums::default(),
            )
        })
        .collect();
    let leaver = peers.iter().position(|p| p.2 == 2).expect("a third seat");
    let mut hung_up: Option<Instant> = None;
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(30) {
        for (k, (session, world, handle, frames, post, room, let_go, sums)) in
            peers.iter_mut().enumerate()
        {
            if k == leaver {
                if *frames >= LEAVES_AT && hung_up.is_none() {
                    for &serial in made[*room].lock().unwrap().iter() {
                        switch.hang_up(serial);
                    }
                    hung_up = Some(Instant::now());
                }
                if hung_up.is_some() {
                    continue;
                }
            }
            session.poll_remote_clients();
            // What the game's driver does with a closed line.
            for net::Peer::Line(seat) in post.take_closed() {
                if session.disconnect_player(seat as usize).is_ok() && let_go.is_none() {
                    *let_go = Some(Instant::now());
                }
            }
            for event in session.events() {
                if let net::ggrs::GgrsEvent::DesyncDetected { frame, .. } = event {
                    panic!("desync at frame {frame}");
                }
            }
            if session.current_state() != net::ggrs::SessionState::Running || *frames >= UNTIL {
                continue;
            }
            let input = Input::aimed(Input::W, (*handle as u16) << 14);
            if session
                .add_local_input(*handle, NetInput::from(input))
                .is_err()
            {
                continue;
            }
            if let Ok(requests) = session.advance_frame() {
                net::handle_requests_watched(world, requests, |_, _, after| {
                    sums.0.insert(after.frame, after.checksum());
                });
                // Frames made, not calls answered: a session that may not
                // advance still answers, with nothing in it.
                *frames = world.frame;
            }
        }
        let stayed = peers.iter().enumerate().filter(|(k, _)| *k != leaver);
        if stayed.clone().all(|(_, p)| p.3 >= UNTIL) {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let hung_up = hung_up.expect("the third never hung up");
    let stayed: Vec<_> = peers
        .iter()
        .enumerate()
        .filter(|(k, _)| *k != leaver)
        .map(|(_, p)| p)
        .collect();
    for (_, world, handle, frames, _, _, let_go, _) in &stayed {
        let let_go =
            let_go.unwrap_or_else(|| panic!("player {} never let the third go", handle + 1));
        assert!(
            let_go.duration_since(hung_up) < Duration::from_secs(2),
            "player {} took {:?} to let the third go",
            handle + 1,
            let_go.duration_since(hung_up)
        );
        assert_eq!(*frames, UNTIL, "player {} stalled", handle + 1);
        assert!(
            !world.is_present(2),
            "player {} still has the third",
            handle + 1
        );
    }
    let (a, b) = (&stayed[0].7.0, &stayed[1].7.0);
    let parted = (1..UNTIL - 10).find(|f| a.get(f) != b.get(f));
    assert_eq!(parted, None, "the two who stayed parted at that frame");
}

/// Each frame's checksum, the last word on it.
#[derive(Default)]
struct HashMapSums(std::collections::HashMap<u32, u64>);
