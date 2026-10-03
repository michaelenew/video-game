//! Playing against a person.
//!
//! In a browser, both players open the same link:
//!
//!     https://…/?room=k3x9qw              (the page's "Play with a friend")
//!     https://…/?room=k3x9qw&board=tabs   (two tabs of one browser)
//!     https://…/?room=k3x9qw&broker=wss://… (one broker of your choosing)
//!
//! On a desktop, each is given the other's address:
//!
//!     game --port 47811 --peer 192.168.1.20:47812
//!
//! Either way the run starts in [`Driver::Meeting`]: the fight runs as
//! training while `net::meet` finds the other player, checks that the two of
//! you are running the same build on the same settings, and agrees who is
//! player one. Then both clients start the match from the same world and
//! every tick goes through GGRS, which owns when to save, load and advance --
//! the simulation only has to do those three things correctly.
//!
//! **Which way of meeting is the one thing here that differs by platform**,
//! and it is chosen in [`start`]: a public broker and WebRTC in a page (which
//! cannot open a UDP socket), a UDP port on a desktop. Past that the two are
//! one `Box<dyn net::Rendezvous>`, and the rest of the crate does not know
//! which it got. See `crates/net/src/meet.rs` for the seams and
//! [`docs/design/web.md`](../../../docs/design/web.md) for the reasoning.

use crate::platform;
use sim::{Input as SimInput, World};

/// How the match is being driven.
pub enum Driver {
    /// The training mode.
    Local,
    /// Looking for the other player. Played as training meanwhile; the match
    /// starts from `start`, not from wherever practice got to.
    Meeting {
        rendezvous: Box<dyn net::Rendezvous>,
        start: Box<World>,
        said: &'static str,
    },
    /// Meeting failed, so this is training, and this is why.
    Alone(String),
    Online {
        session: Box<net::ggrs::P2PSession<net::SessionConfig>>,
        handle: usize,
        desynced: bool,
        gone: bool,
    },
}

impl Driver {
    /// Which fighter this client drives. Online it is the GGRS handle; locally
    /// it is always player one, with player two on the second key set.
    pub fn local_player(&self) -> usize {
        match self {
            Driver::Online { handle, .. } => (*handle).min(1),
            _ => 0,
        }
    }

    /// Is a match against a person running? Anything that edits the world
    /// outside a tick -- a restart, a class change, a rewind -- would edit it
    /// on one machine only, so all of those ask this first.
    pub fn online(&self) -> bool {
        matches!(self, Driver::Online { .. })
    }

    /// One line for the HUD, and for the page around the canvas.
    pub fn status(&self) -> Option<&str> {
        match self {
            Driver::Local => None,
            Driver::Meeting { said, .. } => Some(said),
            Driver::Alone(why) => Some(why),
            Driver::Online { desynced: true, .. } => {
                Some("DESYNC: the two games disagree about the fight. A bug -- please report it.")
            }
            Driver::Online { gone: true, .. } => Some("Your friend disconnected."),
            Driver::Online { handle: 0, .. } => Some("Online: you are player one (blue)."),
            Driver::Online { .. } => Some("Online: you are player two (orange)."),
        }
    }
}

/// Did this run ask for a person to play against? Asked before the world is
/// built, because a world for two people seats player two even in a hunt.
pub fn wanted() -> bool {
    platform::value("--room").is_some() || platform::value("--peer").is_some()
}

/// What two clients must agree on to play: the build, and the world the match
/// starts from. The checksum covers the arena, both classes and every tuned
/// number, so a different link or a different Oven is caught here, before a
/// frame is played, rather than as a desync.
fn terms(start: &World) -> String {
    // Stamped by `crates/web/build-game.sh`. A desktop built by hand says
    // `dev`, and two of those are told apart by the checksum alone.
    let build = option_env!("ARENA_BUILD").unwrap_or("dev");
    format!("{build}-{:016x}", start.checksum())
}

/// Start meeting the other player if this run was given a way to, and train
/// otherwise. `start` is the world the match will begin from.
pub fn start(start: &World) -> Driver {
    match rendezvous(start) {
        None => Driver::Local,
        Some(Ok(rendezvous)) => Driver::Meeting {
            rendezvous,
            start: Box::new(start.clone()),
            said: "Starting…",
        },
        Some(Err(why)) => {
            eprintln!("{why}");
            Driver::Alone(why)
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn rendezvous(start: &World) -> Option<Result<Box<dyn net::Rendezvous>, String>> {
    let room = platform::value("--room")?;
    let me = net::meet::fresh_id();
    let terms = terms(start);
    Some(Ok(
        match (platform::value("--board"), platform::value("--broker")) {
            (Some("tabs"), _) => Box::new(net::browser::tabs(room, me, &terms)),
            (_, Some(url)) => Box::new(net::browser::brokers(&[url], room, me, &terms)),
            _ => Box::new(net::browser::public(room, me, &terms)),
        },
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn rendezvous(start: &World) -> Option<Result<Box<dyn net::Rendezvous>, String>> {
    if platform::value("--room").is_some() {
        return Some(Err(
            "Rooms are the browser build's for now: a desktop has no WebRTC yet. \
             On a desktop, use --port and --peer."
                .into(),
        ));
    }
    let peer = platform::value("--peer")?;
    let Ok(peer) = peer.parse::<std::net::SocketAddr>() else {
        return Some(Err(format!(
            "--peer {peer} is not an address; it wants ip:port, like 192.168.1.20:47812"
        )));
    };
    let port = platform::value("--port")
        .and_then(|p| p.parse().ok())
        .unwrap_or(peer.port());
    Some(match net::direct::Direct::bind(port) {
        Ok(direct) => Ok(Box::new(direct.toward(
            peer,
            net::meet::fresh_id(),
            &terms(start),
        ))),
        Err(e) => Err(format!("could not open UDP port {port}: {e}")),
    })
}

/// Advance the meeting by a frame, and start the match the moment it is
/// ready: both clients from the agreed world, practice thrown away.
pub fn meet(sim: &mut crate::Sim, now_ms: u64) {
    let Driver::Meeting {
        rendezvous, said, ..
    } = &mut sim.driver
    else {
        return;
    };
    match rendezvous.poll(now_ms) {
        net::Progress::Waiting(why) => *said = why,
        net::Progress::Failed(why) => {
            eprintln!("{why}");
            sim.driver = Driver::Alone(why);
        }
        net::Progress::Ready(seat) => {
            let handle = seat.handle;
            let Driver::Meeting { start, .. } = std::mem::replace(&mut sim.driver, Driver::Local)
            else {
                unreachable!("matched above");
            };
            sim.driver = match net::p2p::start(seat) {
                Ok(session) => {
                    eprintln!("online: you are player {}", handle + 1);
                    let start = *start;
                    sim.prev = start.clone();
                    sim.history = crate::Rewind::new(&start);
                    sim.cur = start;
                    sim.paused = false;
                    sim.rehearsing = None;
                    Driver::Online {
                        session: Box::new(session),
                        handle,
                        desynced: false,
                        gone: false,
                    }
                }
                Err(e) => Driver::Alone(format!("could not start the match: {e}")),
            };
        }
    }
}

/// Tell the page, or the terminal, when the line changes.
pub fn announce(sim: bevy::prelude::Res<crate::Sim>, mut last: bevy::prelude::Local<String>) {
    let now = sim.driver.status().unwrap_or("");
    if *last != now {
        now.clone_into(&mut last);
        platform::announce(now);
    }
}

/// One networked tick.
///
/// GGRS decides when to save, load and advance; `handle_requests` services
/// those against the simulation. Rollbacks land here as a load followed by
/// several advances, all inside one call.
pub fn step(sim: &mut crate::Sim, local: SimInput) {
    let Driver::Online {
        session,
        handle,
        desynced,
        gone,
    } = &mut sim.driver
    else {
        return;
    };

    session.poll_remote_clients();
    for event in session.events() {
        match event {
            net::ggrs::GgrsEvent::DesyncDetected { frame, .. } => {
                // Should be impossible: the simulation is integer-only and
                // SyncTest covers it. If it happens, say so loudly rather than
                // letting the two players drift apart in silence.
                eprintln!("DESYNC at frame {frame}");
                *desynced = true;
            }
            net::ggrs::GgrsEvent::Disconnected { .. } => {
                eprintln!("peer disconnected");
                *gone = true;
            }
            net::ggrs::GgrsEvent::NetworkInterrupted { .. } => eprintln!("peer interrupted"),
            _ => {}
        }
    }

    if session.current_state() != net::ggrs::SessionState::Running {
        return;
    }

    if session
        .add_local_input(*handle, net::NetInput::from(local))
        .is_err()
    {
        // Too far ahead of the peer. Waiting is the correct response.
        return;
    }

    let prev = sim.cur.clone();
    match session.advance_frame() {
        Ok(requests) => {
            net::handle_requests(&mut sim.cur, requests);
            sim.prev = prev;
        }
        Err(net::ggrs::GgrsError::PredictionThreshold) => {}
        Err(e) => eprintln!("advance failed: {e}"),
    }
}
