//! Playing against a person.
//!
//! In a browser, both players open the same link:
//!
//!     https://…/?room=k3x9qw              (the page's "Play with a friend")
//!     https://…/?room=k3x9qw&board=tabs   (two tabs of one browser)
//!     https://…/?room=k3x9qw&broker=wss://… (one broker of your choosing)
//!
//! A desktop joins the same room with the same link, pasted:
//!
//!     game --join 'https://…/?room=k3x9qw#key=…'
//!
//! or, on a LAN, two desktops are given each other's address:
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
//! and it is chosen in [`start`]: a room is `net::browser` in a page and
//! `net::native` on a desktop -- the same brokers, the same sealed notes, the
//! same WebRTC data channel, built from different parts -- and only a desktop
//! can also go straight to an address over UDP. Past that the two are
//! one `Box<dyn net::Rendezvous>`, and the rest of the crate does not know
//! which it got. See `crates/net/src/meet.rs` for the seams and
//! [`docs/design/web.md`](../../../docs/design/web.md) for the reasoning.

use crate::platform::{self, Options};
use sim::{Input as SimInput, World};

/// A room this client is in: what to call it, and the link that brings a
/// friend into it. Shown in the Esc menu (`menu.rs`) with a Copy button.
#[derive(Clone, Debug, PartialEq)]
pub struct Room {
    pub name: String,
    pub link: String,
}

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
        /// `None` for two desktops meeting by address.
        room: Option<Room>,
    },
    /// Meeting failed, so this is training, and this is why -- and the
    /// meeting's own account of it ([`net::Rendezvous::report`]), kept for the
    /// Esc menu in dev mode, since the meeting itself is gone.
    Alone { why: String, report: Vec<String> },
    Online {
        session: Box<net::ggrs::P2PSession<net::SessionConfig>>,
        handle: usize,
        desynced: bool,
        gone: bool,
        room: Option<Room>,
        /// Frames with ticks since one was held back for being ahead; see
        /// [`pace`].
        paced: u32,
        /// The friend's game has gone quiet for a moment: loading an arena,
        /// say, or in a window the system has stopped drawing.
        quiet: bool,
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

    /// The room this client is in, while it is meeting or playing in one.
    pub fn room(&self) -> Option<&Room> {
        match self {
            Driver::Meeting { room, .. } | Driver::Online { room, .. } => room.as_ref(),
            _ => None,
        }
    }

    /// How meeting is going, in detail: every meeting point's story, who has
    /// been heard from, the direct line. For dev mode's Esc menu. Empty in
    /// training, and in a match (which is past meeting).
    pub fn report(&self) -> Vec<String> {
        match self {
            Driver::Meeting { rendezvous, .. } => rendezvous.report(),
            Driver::Alone { report, .. } => report.clone(),
            _ => Vec::new(),
        }
    }

    /// One line for the HUD, and for the page around the canvas.
    pub fn status(&self) -> Option<&str> {
        match self {
            Driver::Local => None,
            Driver::Meeting { said, .. } => Some(said),
            Driver::Alone { why, .. } => Some(why),
            Driver::Online { desynced: true, .. } => {
                Some("DESYNC: the two games disagree about the fight. A bug -- please report it.")
            }
            Driver::Online { gone: true, .. } => Some("Your friend disconnected."),
            Driver::Online { quiet: true, .. } => {
                Some("Waiting for your friend's game: it has gone quiet for a moment…")
            }
            Driver::Online { handle: 0, .. } => Some("Online: you are player one (blue)."),
            Driver::Online { .. } => Some("Online: you are player two (orange)."),
        }
    }
}

/// Do these settings ask for a person to play against? Asked before the
/// world is built, because a world for two people seats player two even in a
/// hunt.
pub fn wanted(opts: &Options) -> bool {
    opts.value("--room").is_some() || opts.value("--peer").is_some()
}

/// What two clients must agree on to play: the build, and the world the match
/// starts from. The checksum covers the arena, both classes and every tuned
/// number, so a different link or a different Oven is caught here, before a
/// frame is played, rather than as a desync.
fn terms(start: &World) -> String {
    // The commit this was built from (`build.rs`), so a desktop built from
    // the commit the page was deployed from is the same build as the page.
    let build = env!("ARENA_BUILD");
    format!("{build}-{:016x}", start.checksum())
}

/// Start meeting the other player if these settings ask for one, and train
/// otherwise. `start` is the world the match will begin from: built from the
/// same settings, which is what makes it the same world on both machines.
pub fn start(opts: &Options, start: &World) -> Driver {
    match rendezvous(opts, start) {
        None => Driver::Local,
        Some(Ok(rendezvous)) => Driver::Meeting {
            rendezvous,
            start: Box::new(start.clone()),
            said: "Starting…",
            room: room_of(opts),
        },
        Some(Err(why)) => {
            eprintln!("{why}");
            Driver::Alone {
                why,
                report: Vec::new(),
            }
        }
    }
}

/// The settings a room's link carries, in the order they are written. The
/// room and its secret, how the two meet, and everything the starting world
/// is built from -- so whoever opens the link builds the same world.
const LINKED: &[&str] = &[
    "room", "p1", "p2", "hunt", "arena", "temper", "board", "broker",
];

/// The link that brings a friend into the room these settings describe.
fn link_of(opts: &Options) -> String {
    let query: Vec<String> = LINKED
        .iter()
        .filter_map(|name| opts.value(name).map(|v| format!("{name}={v}")))
        .collect();
    let mut link = format!("{}?{}", platform::page_url(), query.join("&"));
    if let Some(key) = opts.value("--key") {
        link.push_str("#key=");
        link.push_str(key);
    }
    link
}

fn room_of(opts: &Options) -> Option<Room> {
    Some(Room {
        name: opts.value("--room")?.to_string(),
        link: link_of(opts),
    })
}

/// **Create a room**, from the Esc menu: a fresh name and secret, and the
/// fight being practised -- both classes, the creature, the arena, the temper
/// -- written into the link. The match then starts from the world that link
/// describes, built the way the friend who opens it will build it, so the two
/// agree even where the link could not say everything about the practice.
pub fn create(sim: &mut crate::Sim) {
    let mut opts = Options::default();
    opts.set("room", &net::seal::letters(6));
    opts.set("key", &net::seal::letters(26));
    for (name, value) in crate::picker::describe(&sim.cur) {
        opts.set(name, &value);
    }
    // How this run meets: the same brokers it was told to use, if any.
    for name in ["board", "broker"] {
        if let Some(value) = platform::value(name) {
            opts.set(name, value);
        }
    }
    enter(sim, &opts);
}

/// **Join a room** from a link a friend sent, pasted into the Esc menu.
pub fn join(sim: &mut crate::Sim, link: &str) -> Result<(), String> {
    let mut opts = Options::default();
    opts.absorb_link(link.trim());
    if opts.value("--room").is_none() {
        return Err("That is not a room link: it has no room= in it. Paste the whole link.".into());
    }
    enter(sim, &opts);
    Ok(())
}

/// **Leave** the room, or the match: back to training, the room forgotten.
pub fn leave(sim: &mut crate::Sim) {
    sim.driver = Driver::Local;
    platform::show_room(None);
}

/// Start meeting in the room these settings name, from the world they
/// describe, with player two seated as a person.
fn enter(sim: &mut crate::Sim, opts: &Options) {
    let world = crate::start_world(opts, crate::Dummy::Human);
    sim.dummy = crate::Dummy::Human;
    sim.driver = start(opts, &world);
    platform::show_room(sim.driver.room().map(|r| r.link.as_str()));
}

/// The room these settings name, if any: its name, and the secret after the
/// link's `#` that seals it (`net::seal`). On a desktop both arrive in
/// `--join <link>`, which `platform` unpacks into the same two names.
fn room_key(opts: &Options) -> Option<net::seal::RoomKey> {
    let room = opts.value("--room")?;
    Some(net::seal::RoomKey::new(
        room,
        opts.value("--key").unwrap_or(""),
    ))
}

#[cfg(target_arch = "wasm32")]
fn rendezvous(opts: &Options, start: &World) -> Option<Result<Box<dyn net::Rendezvous>, String>> {
    let key = room_key(opts)?;
    let me = net::meet::fresh_id();
    let terms = terms(start);
    Some(Ok(match (opts.value("--board"), opts.value("--broker")) {
        (Some("tabs"), _) => Box::new(net::browser::tabs(&key, me, &terms)),
        (_, Some(url)) => Box::new(net::browser::brokers(&[url], &key, me, &terms)),
        _ => Box::new(net::browser::public(&key, me, &terms)),
    }))
}

#[cfg(not(target_arch = "wasm32"))]
fn rendezvous(opts: &Options, start: &World) -> Option<Result<Box<dyn net::Rendezvous>, String>> {
    if let Some(key) = room_key(opts) {
        let me = net::meet::fresh_id();
        let terms = terms(start);
        return Some(match (opts.value("--board"), opts.value("--broker")) {
            (Some("tabs"), _) => Err(
                "board=tabs is two tabs of one browser, and this is not a browser. \
                     Drop it from the link to meet through the public brokers."
                    .into(),
            ),
            (_, Some(url)) => Ok(Box::new(net::native::brokers(&[url], &key, me, &terms))),
            _ => Ok(Box::new(net::native::public(&key, me, &terms))),
        });
    }
    let peer = opts.value("--peer")?;
    let Ok(peer) = peer.parse::<std::net::SocketAddr>() else {
        return Some(Err(format!(
            "--peer {peer} is not an address; it wants ip:port, like 192.168.1.20:47812"
        )));
    };
    let port = opts
        .value("--port")
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
            // Printed whether or not dev mode is on: a console or a terminal
            // is where somebody looks once it has failed, and by then it is
            // too late to switch anything on and try again for the same story.
            let report = rendezvous.report();
            platform::log(&format!("meeting failed: {why}"));
            for line in &report {
                platform::log(line);
            }
            sim.driver = Driver::Alone { why, report };
        }
        net::Progress::Ready(seat) => {
            let handle = seat.handle;
            let Driver::Meeting { start, room, .. } =
                std::mem::replace(&mut sim.driver, Driver::Local)
            else {
                unreachable!("matched above");
            };
            // `delay=` (`--delay`) trades some rollback back for input delay,
            // on this side only. None is the default: see `net::p2p`.
            let delay = platform::value("--delay")
                .and_then(|d| d.parse().ok())
                .unwrap_or(net::p2p::INPUT_DELAY);
            sim.driver = match net::p2p::start_with_delay(seat, delay) {
                Ok(session) => {
                    eprintln!(
                        "online: you are player {}, input delay {delay} frames",
                        handle + 1
                    );
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
                        room,
                        paced: 0,
                        quiet: false,
                    }
                }
                Err(e) => Driver::Alone {
                    why: format!("could not start the match: {e}"),
                    report: Vec::new(),
                },
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

/// How many of this frame's `ticks` to run online: all of them, unless this
/// side is ahead of the other.
///
/// Two clocks never agree exactly, and the one that runs fast gets ahead: its
/// predictions reach further, its rollbacks get longer, and at the session's
/// prediction limit it stops dead until the other catches up -- a stall you
/// feel as lag. GGRS measures the gap (`frames_ahead`) and leaves closing it
/// to the game, so the side that is ahead holds back one tick in every
/// [`PACE_EVERY`] frames: about a tenth slower, for as long as it is ahead,
/// which is too little to see and closes a frame's gap in under a second.
pub fn pace(sim: &mut crate::Sim, ticks: u32) -> u32 {
    let Driver::Online { session, paced, .. } = &mut sim.driver else {
        return ticks;
    };
    if ticks == 0 {
        return 0;
    }
    *paced += 1;
    // Far ahead -- the other side has just come back from a long frame --
    // closes faster: every other frame, rather than one in ten.
    let every = if session.frames_ahead() > FAR_AHEAD {
        2
    } else {
        PACE_EVERY
    };
    if session.frames_ahead() > 0 && *paced >= every {
        *paced = 0;
        return ticks - 1;
    }
    ticks
}

/// See [`pace`].
const PACE_EVERY: u32 = 10;
const FAR_AHEAD: i32 = 3;

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
        quiet,
        ..
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
            net::ggrs::GgrsEvent::NetworkInterrupted { .. } => {
                eprintln!("peer interrupted");
                *quiet = true;
            }
            net::ggrs::GgrsEvent::NetworkResumed { .. } => {
                eprintln!("peer resumed");
                *quiet = false;
            }
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
