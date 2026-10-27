//! **A room of groups** (`net::group::Room`, exploration 0029 §14), end to
//! end with nothing leaving the process: lobbies on an in-memory board,
//! lines on a switchboard, each peer driven the way the game drives one.
//!
//! - Two open the link: each starts as a group of one in the town, and the
//!   two are one group moments later, holding one world.
//! - A third comes in while they play: a group of one, then one of three --
//!   nobody seated it, and nobody sent it a world.
//! - One of them closes their game: the rest are a group of two at once,
//!   and play on.

use net::group::Room;
use net::lobby::Lobby;
use net::loopback::{Hall, Switchboard};
use sim::{Class, Input, World};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Peer {
    name: &'static str,
    room: Room,
    class: Class,
    made: Arc<Mutex<Vec<u32>>>,
    gone: bool,
    rng: u64,
}

struct Rooms {
    hall: Hall,
    switch: Switchboard,
    peers: Vec<Peer>,
    began: Instant,
}

/// My world when my group starts: me alone, in the town.
fn alone(class: Class) -> World {
    World::arrive(
        [class],
        sim::valley::Journey::default(),
        sim::valley::START,
        None,
        1,
    )
}

impl Rooms {
    fn new() -> Rooms {
        Rooms {
            hall: Hall::default(),
            switch: Switchboard::default(),
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
            8,
            class,
        );
        self.peers.push(Peer {
            name,
            room: Room::new(Box::new(lobby)),
            class,
            made,
            gone: false,
            rng: id.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1,
        });
    }

    fn leave(&mut self, name: &str) {
        let p = self.peers.iter_mut().find(|p| p.name == name).unwrap();
        for &serial in p.made.lock().unwrap().iter() {
            self.switch.hang_up(serial);
        }
        p.gone = true;
    }

    fn round(&mut self) {
        let now = self.began.elapsed().as_millis() as u64;
        for p in self.peers.iter_mut().filter(|p| !p.gone) {
            let class = p.class;
            p.room.tend(now, || alone(class));
            let Some(session) = &mut p.room.session else {
                continue;
            };
            // Milling about the square: never far from where they began.
            p.rng ^= p.rng << 13;
            p.rng ^= p.rng >> 7;
            p.rng ^= p.rng << 17;
            let bits = if p.rng & 3 == 0 { Input::W } else { 0 };
            session.tick(Input::aimed(bits, (p.rng >> 20) as u16));
        }
    }

    fn run_until(&mut self, limit: Duration, done: impl Fn(&Rooms) -> bool) {
        let until = Instant::now() + limit;
        while Instant::now() < until {
            self.round();
            if done(self) {
                return;
            }
            std::thread::sleep(Duration::from_micros(300));
        }
        let state: Vec<String> = self
            .here()
            .map(|p| match &p.room.session {
                Some(s) => format!(
                    "{}: {:?} members {:?} peers {:?}",
                    p.name,
                    s,
                    s.members(),
                    s.peers()
                ),
                None => format!("{}: no session -- {}", p.name, p.room.lobby.status()),
            })
            .collect();
        panic!("timed out:\n{}", state.join("\n"));
    }

    fn here(&self) -> impl Iterator<Item = &Peer> {
        self.peers.iter().filter(|p| !p.gone)
    }

    /// Everybody here is one group of everybody here.
    fn one_group(&self) -> bool {
        let mut ids: Vec<u64> = self.here().map(|p| p.room.me()).collect();
        ids.sort();
        self.here()
            .all(|p| p.room.session.as_ref().is_some_and(|s| s.members() == ids))
    }

    /// Everybody here holds the same world at a frame all have confirmed.
    fn agree(&self) {
        let sessions: Vec<_> = self
            .here()
            .map(|p| p.room.session.as_ref().unwrap())
            .collect();
        let f = sessions.iter().map(|s| s.confirmed).min().unwrap();
        let sums: Vec<_> = sessions
            .iter()
            .map(|s| s.world_at(f).map(|w| w.checksum()))
            .collect();
        assert!(
            sums[0].is_some() && sums.iter().all(|s| *s == sums[0]),
            "at frame {f}: {sums:?}"
        );
    }
}

#[test]
fn a_room_of_groups_gathers_takes_in_a_newcomer_and_lets_one_go() {
    let mut r = Rooms::new();
    r.arrive("ann", 0x0101_0000_0000_00a1, Class::Champion);
    r.arrive("bo", 0x0202_0000_0000_00b2, Class::Elementalist);
    r.run_until(Duration::from_secs(20), |r| r.one_group());
    r.agree();
    // Played on a while together, then somebody new.
    for _ in 0..200 {
        r.round();
    }
    r.arrive("cy", 0x0303_0000_0000_00c3, Class::DualMage);
    r.run_until(Duration::from_secs(20), |r| r.one_group());
    for _ in 0..100 {
        r.round();
    }
    r.agree();
    // By merging: every one of them took part in at least one.
    for p in r.here() {
        let s = p.room.session.as_ref().unwrap();
        assert!(s.merges >= 1, "{} never merged", p.name);
        eprintln!("{}: {} merges, frame {}", p.name, s.merges, s.world.frame);
    }
    // And one goes.
    let frame = r.peers[0].room.session.as_ref().unwrap().world.frame;
    r.leave("bo");
    r.run_until(Duration::from_secs(20), |r| {
        r.one_group()
            && r.here()
                .all(|p| p.room.session.as_ref().unwrap().world.frame > frame + 200)
    });
    r.agree();
}

/// **Two players are two members, whatever their ids** (2026-10-27): these
/// two folded to the same sixteen-bit id when the world kept one, and the
/// room played them as a single member.
#[test]
fn two_whose_ids_once_folded_together_are_two_members() {
    let (a, b) = (0x0001_0000_0000_0000_u64, 0x0000_0000_0000_0001_u64);
    let fold = |id: u64| (id ^ (id >> 16) ^ (id >> 32) ^ (id >> 48)) as u16;
    assert_eq!(
        fold(a),
        fold(b),
        "the pair the old fold could not tell apart"
    );
    let mut r = Rooms::new();
    r.arrive("ann", a, Class::Champion);
    r.arrive("bo", b, Class::Elementalist);
    r.run_until(Duration::from_secs(20), |r| r.one_group());
    for _ in 0..100 {
        r.round();
    }
    for p in r.here() {
        let s = p.room.session.as_ref().unwrap();
        assert_eq!(s.members(), vec![b, a], "{}", p.name);
        let present = (0..sim::state::MAX_PLAYERS)
            .filter(|&k| s.world.is_present(k))
            .count();
        assert_eq!(present, 2, "{}: two fighters in the world", p.name);
    }
    r.agree();
}
