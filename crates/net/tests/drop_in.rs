//! **A room you come and go from**, end to end with nothing leaving the
//! process: lobbies on an in-memory board, lines on a switchboard, each peer
//! driven the way the game's online driver drives one (`net::drop_in`) --
//! the lobby tended, the session advanced through the region gate with the
//! crowd's books kept, the lineup switched at the frame somebody is seated.
//!
//! - Two start; a third and a fourth come in mid-match, each on the smaller
//!   side, and all four agree on every frame once they are all in.
//! - One of four leaves: the rest play on at once, and the next to come takes
//!   the seat that was theirs.

use net::drop_in::{Change, DropIn};
use net::lobby::Lobby;
use net::loopback::{Hall, Switchboard};
use net::{NetInput, handle_requests_seated};
use sim::{Class, Input, World};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The world a room for four in the town starts from, as a link builds it.
fn start() -> World {
    sim::replay::Start::parse("p1=champion p2=champion arena=hearth temper=0 seats=4")
        .expect("start")
        .world()
}

struct Peer {
    name: &'static str,
    play: DropIn,
    world: World,
    ledger: regions::Ledger,
    /// Each frame's checksum, the last word on it.
    sums: HashMap<u32, u64>,
    /// Every line this peer made, so it can hang them all up.
    made: Arc<Mutex<Vec<u32>>>,
    gone: bool,
    switches: u32,
    /// Press "Start match" on the next frame made.
    ready: bool,
    rng: u64,
}

struct Room {
    hall: Hall,
    switch: Switchboard,
    start: World,
    peers: Vec<Peer>,
    began: Instant,
}

impl Room {
    fn new() -> Room {
        Room {
            hall: Hall::default(),
            switch: Switchboard::default(),
            start: start(),
            peers: Vec::new(),
            began: Instant::now(),
        }
    }

    fn arrive(&mut self, name: &'static str, id: u64, class: Class) {
        let switch = self.switch.clone();
        let made: Arc<Mutex<Vec<u32>>> = Arc::default();
        let mine = made.clone();
        let lobby = Lobby::new(
            self.hall.board(),
            move || {
                let line = switch.line();
                mine.lock().unwrap().push(line.serial());
                line
            },
            id,
            "t",
            4,
            class,
        );
        let world = self.start.clone();
        let ledger = regions::Ledger::new(&world, 0, false);
        self.peers.push(Peer {
            name,
            play: DropIn::new(Box::new(lobby), 2),
            world,
            ledger,
            sums: HashMap::new(),
            made,
            gone: false,
            switches: 0,
            ready: false,
            rng: id.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1,
        });
    }

    fn leave(&mut self, name: &str) {
        let p = self.peers.iter_mut().find(|p| p.name == name).unwrap();
        for &serial in p.made.lock().unwrap().iter() {
            self.switch.hang_up(serial);
        }
        // Lines the others made to it are hung up from their end too.
        p.gone = true;
    }

    /// One round: every peer that is here tends its room and plays a frame.
    fn round(&mut self) {
        let now = self.began.elapsed().as_millis() as u64;
        for p in self.peers.iter_mut().filter(|p| !p.gone) {
            let change = p.play.tend(now, &mut p.world, &self.start, &mut p.ledger);
            if change == Change::Started {
                p.sums.clear();
            }
            let travel = p.play.travel();
            let handle = p.play.handle;
            let Some(session) = &mut p.play.session else {
                continue;
            };
            session.poll_remote_clients();
            for event in session.events() {
                if let net::ggrs::GgrsEvent::DesyncDetected { frame, .. } = event {
                    panic!("{}: desync at frame {frame}", p.name);
                }
            }
            if session.current_state() != net::ggrs::SessionState::Running {
                continue;
            }
            let base = p.play.base as i32;
            let world_confirmed = |c: i32| if c < 0 { c } else { base + c };
            if !net::may_advance(
                &mut p.ledger,
                world_confirmed(session.confirmed_frame()),
                &p.world,
            ) {
                continue;
            }
            p.rng ^= p.rng << 13;
            p.rng ^= p.rng >> 7;
            p.rng ^= p.rng << 17;
            let mut input = Input::aimed((p.rng & 0x1ff) as u16, (p.rng >> 20) as u16);
            if let Some(t) = travel {
                input = input.travelling(t);
            } else if p.ready {
                input = input.travelling(sim::input::Travel::START);
            }
            if session
                .add_local_input(handle, NetInput::from(input))
                .is_err()
            {
                continue;
            }
            let Ok(requests) = session.advance_frame() else {
                continue;
            };
            let confirmed = world_confirmed(session.confirmed_frame());
            let advanced = requests
                .iter()
                .any(|r| matches!(r, net::ggrs::GgrsRequest::AdvanceFrame { .. }));
            if travel.is_some() && advanced {
                p.play.sent();
            } else if p.ready && advanced {
                p.ready = false;
            }
            let sums = &mut p.sums;
            let seats = p.play.seats.clone();
            handle_requests_seated(
                &mut p.world,
                requests,
                &mut p.ledger,
                &seats,
                |_, _, after| {
                    sums.insert(after.frame, after.checksum());
                },
            );
            if let Ok(c) = u32::try_from(confirmed) {
                p.ledger.confirm_through(c, &p.world);
            }
            // The crowd's leaders, as the game relays them.
            if let Some(post) = p.play.post.clone() {
                let grid = sim::region::Grid::tuned();
                let epoch = (p.world.frame / sim::tuning::crowd_epoch()) as u16;
                let members: Vec<net::cells::Member> = p
                    .play
                    .roster
                    .iter()
                    .zip(&p.play.seats)
                    .map(|(id, &seat)| net::cells::Member { seat, id: *id })
                    .collect();
                let me = p.play.seat();
                for a in p.ledger.summed() {
                    if !net::cells::leads(&p.world, &grid, &members, a.region, epoch, me) {
                        continue;
                    }
                    for (player, &other) in p.play.seats.iter().enumerate() {
                        if player != handle
                            && a.region.neighbours().iter().any(|n| {
                                net::cells::leads(&p.world, &grid, &members, *n, epoch, other)
                            })
                        {
                            post.post(net::Peer::Line(player as u8), a);
                        }
                    }
                }
                for a in post.collect(p.world.frame, sim::tuning::crowd_epoch()) {
                    p.ledger.offer_heard(a);
                }
            }
            if p.play.after(&mut p.world, &mut p.ledger) == Change::Switched {
                p.switches += 1;
                // The frames past the switch were dropped with the old
                // session: what is kept from here is the new one's.
                let from = p.world.frame;
                p.sums.retain(|f, _| *f <= from);
            }
        }
    }

    fn run_until(&mut self, limit: Duration, done: impl Fn(&Room) -> bool) {
        let until = Instant::now() + limit;
        while Instant::now() < until {
            self.round();
            if done(self) {
                return;
            }
            std::thread::sleep(Duration::from_micros(500));
        }
        let state: Vec<String> = self
            .peers
            .iter()
            .map(|p| {
                format!(
                    "{}: frame {} epoch {} seats {:?} running {} state {:?} confirmed {} -- {}",
                    p.name,
                    p.world.frame,
                    p.play.epoch,
                    p.play.seats,
                    p.play.session.is_some(),
                    p.play.session.as_ref().map(|s| s.current_state()),
                    p.play.confirmed(),
                    p.play.lobby.status()
                )
            })
            .collect();
        let reports: Vec<String> = self
            .peers
            .iter()
            .map(|p| format!("{}: {}", p.name, p.play.lobby.report().join(" | ")))
            .collect();
        panic!("timed out:\n{}\n{}", state.join("\n"), reports.join("\n"));
    }

    fn here(&self) -> impl Iterator<Item = &Peer> {
        self.peers.iter().filter(|p| !p.gone)
    }

    /// Everybody here plays the same lineup and has made `frames` frames in it.
    fn settled(&self, players: usize, frames: u32) -> bool {
        let here: Vec<&Peer> = self.here().collect();
        let Some(first) = here.first() else {
            return false;
        };
        // Frames made in the session running now: past where it began.
        here.iter().all(|p| {
            p.play.seats.len() == players
                && p.play.epoch == first.play.epoch
                && p.play.session.is_some()
                && p.world.frame > p.play.base + frames
        })
    }

    /// Every frame everybody here has confirmed and played again since, the
    /// same on all of them.
    fn agree(&self) {
        let here: Vec<&Peer> = self.here().collect();
        let confirmed = here
            .iter()
            .filter(|p| p.play.session.is_some())
            .map(|p| p.play.confirmed())
            .min()
            .unwrap_or(0)
            .max(0) as u32;
        let from = here
            .iter()
            .map(|p| p.sums.keys().min().copied().unwrap_or(0))
            .max()
            .unwrap_or(0);
        let mut checked = 0;
        for f in from..confirmed.saturating_sub(12) {
            let sums: Vec<Option<&u64>> = here.iter().map(|p| p.sums.get(&f)).collect();
            if sums.iter().all(Option::is_some) {
                checked += 1;
                assert!(
                    sums.windows(2).all(|w| w[0] == w[1]),
                    "frame {f}: {:?}",
                    here.iter()
                        .map(|p| (p.name, p.sums.get(&f)))
                        .collect::<Vec<_>>()
                );
            }
        }
        assert!(checked > 30, "only {checked} frames to compare");
    }
}

#[test]
fn two_start_and_a_third_and_fourth_drop_in_on_the_smaller_side() {
    let mut room = Room::new();
    room.arrive("a", 0x10, Class::Champion);
    room.arrive("b", 0x20, Class::Elementalist);
    // Long enough that the crowd's aggregates are due to every frame: a
    // switch has to leave none waited for (`regions::Ledger::begin`).
    room.run_until(Duration::from_secs(20), |r| r.settled(2, 400));
    room.agree();

    room.arrive("c", 0x30, Class::DualMage);
    room.run_until(Duration::from_secs(20), |r| r.settled(3, 120));
    room.agree();

    room.arrive("d", 0x40, Class::BloodMage);
    room.run_until(Duration::from_secs(20), |r| r.settled(4, 150));
    room.agree();

    // Everybody presses "Start match": the versus match starts, on every
    // machine alike, and they play on in step.
    for p in room.peers.iter_mut() {
        p.ready = true;
    }
    room.run_until(Duration::from_secs(20), |r| {
        r.here().all(|p| sim::versus::of(p.world.arena).is_some())
    });
    let at = room.peers[0].world.frame;
    room.run_until(Duration::from_secs(20), |r| {
        r.here().all(|p| p.world.frame > at + 400)
    });
    room.agree();
    for p in room.here() {
        let w = &p.world;
        for i in 0..4 {
            assert!(w.is_present(i), "{}: seat {i} is empty", p.name);
        }
        let sides = (0..4).filter(|&i| sim::versus::side(i) == 0).count();
        assert_eq!(sides, 2, "two a side");
        assert_eq!(p.play.seats, vec![0, 1, 2, 3]);
    }
    // The newcomers came as what they chose.
    let a = &room.peers[0].world;
    let classes: Vec<Class> = (0..4).map(|i| a.players[i].class).collect();
    assert!(classes.contains(&Class::DualMage) && classes.contains(&Class::BloodMage));
}

#[test]
fn one_leaves_the_rest_play_on_and_the_next_takes_their_seat() {
    let mut room = Room::new();
    for (name, id, class) in [
        ("a", 0x10, Class::Champion),
        ("b", 0x20, Class::Elementalist),
        ("c", 0x30, Class::DualMage),
        ("d", 0x40, Class::ShadowReaver),
    ] {
        room.arrive(name, id, class);
    }
    room.run_until(Duration::from_secs(30), |r| r.settled(4, 400));
    let c_seat = room.peers[2].play.seat();

    room.leave("c");
    let at = room.peers[0].world.frame;
    // The three play on, at once: no fifteen frozen seconds.
    room.run_until(Duration::from_secs(5), |r| {
        r.here()
            .all(|p| p.world.frame > at + 120 && !p.world.is_present(c_seat))
    });

    room.arrive("e", 0x50, Class::BloodMage);
    room.run_until(Duration::from_secs(30), |r| r.settled(4, 150));
    room.agree();
    for p in room.here() {
        assert!(
            p.world.is_present(c_seat),
            "{}: the seat is taken again",
            p.name
        );
        assert_eq!(p.world.players[c_seat].class, Class::BloodMage);
    }
}

#[test]
fn a_full_room_turns_the_fifth_away() {
    let mut room = Room::new();
    for (name, id) in [("a", 0x10), ("b", 0x20), ("c", 0x30), ("d", 0x40)] {
        room.arrive(name, id, Class::Champion);
    }
    room.run_until(Duration::from_secs(30), |r| r.settled(4, 60));
    room.arrive("e", 0x50, Class::Champion);
    room.run_until(Duration::from_secs(10), |r| {
        r.peers[4].play.lobby.status().contains("full")
    });
    assert!(room.peers[4].play.session.is_none());
    assert!(room.here().take(4).all(|p| p.play.seats.len() == 4));
}

/// **A newcomer with the lowest id of anybody** still comes into the match
/// going on, rather than taking the hellos it heard before the host's lineup
/// for a room nobody has started and making a first lineup of its own -- a
/// session of its own from the link's world, alone (found playing three
/// desktops, 2026-10-22).
#[test]
fn a_newcomer_with_the_lowest_id_joins_the_match_going_on() {
    let mut room = Room::new();
    room.arrive("a", 0x10, Class::Champion);
    room.arrive("b", 0x20, Class::Elementalist);
    room.run_until(Duration::from_secs(20), |r| r.settled(2, 300));
    room.arrive("c", 0x01, Class::DualMage);
    room.run_until(Duration::from_secs(20), |r| r.settled(3, 120));
    room.agree();
    assert!(
        room.peers[2].play.base > 300,
        "c started from the match, not the link"
    );
}
