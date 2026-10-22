//! Playing against a person.
//!
//! In a browser, every player opens the same link -- two of you, or three or
//! four with `players=` in it (the Esc menu's room carries it):
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
        /// This second of the link, being counted; see [`Meter`].
        meter: Meter,
        /// How many play: two to four.
        players: usize,
        /// Everybody's id in seat order, and the crowd post box: what a cell
        /// leader's aggregates go out by and come in by (`net::cells`,
        /// `net::crowd`).
        roster: Vec<u64>,
        post: net::crowd::Post,
    },
}

/// **The link, counted a second at a time**, onto the tape
/// (`sim::replay::Link`), so a match that felt laggy says on its replay where
/// and why: the line (ping), the friend's machine (stalls while this one ran
/// smoothly), or this one (a slow frame). See `docs/design/replays.md` §6.
#[derive(Default)]
pub struct Meter {
    now: sim::replay::Link,
    /// The frame the second began on.
    since: u32,
}

impl Meter {
    /// A frame drawn, and how long it took on the wall clock.
    pub fn drawn(&mut self, wall: std::time::Duration) {
        self.now.slowest = self.now.slowest.max(wall.as_millis() as u32);
    }
}

/// The world's frames in a second of the link.
const SECOND: u32 = sim::TICK_HZ;

impl Driver {
    /// Which fighter this client drives. Online it is the GGRS handle; locally
    /// it is always player one, with player two on the second key set.
    pub fn local_player(&self) -> usize {
        match self {
            Driver::Online { handle, .. } => (*handle).min(sim::state::MAX_PLAYERS - 1),
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
            Driver::Online {
                gone: true,
                players: 2,
                ..
            } => Some("Your friend disconnected."),
            Driver::Online { gone: true, .. } => {
                Some("A friend disconnected; their fighter has left the fight.")
            }
            Driver::Online { quiet: true, .. } => {
                Some("Waiting for your friend's game: it has gone quiet for a moment…")
            }
            Driver::Online { handle: 0, .. } => Some("Online: you are player one (blue)."),
            Driver::Online { handle: 1, .. } => Some("Online: you are player two (orange)."),
            Driver::Online { handle: 2, .. } => Some("Online: you are player three (green)."),
            Driver::Online { .. } => Some("Online: you are player four (violet)."),
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
    "room", "players", "p1", "p2", "p3", "p4", "hunt", "arena", "temper", "board", "broker",
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
///
/// **For `players`**, two to four, chosen beside the button: a room is for as
/// many as its link says, and the practice only ever has two, so without the
/// choice every room made here was for two and a third friend was turned
/// away (2026-10-22). Seats the practice did not have are the first seat's
/// class, as `--p3` and `--p4` are when not given.
pub fn create(sim: &mut crate::Sim, players: u8) {
    let opts = room_settings(&sim.cur, players);
    enter(sim, &opts);
}

/// The settings a room made from the Esc menu carries: a fresh name and
/// secret, the fight `practice` is, and how many play.
fn room_settings(practice: &World, players: u8) -> Options {
    let mut opts = Options::default();
    opts.set("room", &net::seal::letters(6));
    opts.set("key", &net::seal::letters(26));
    for (name, value) in crate::picker::describe(practice) {
        opts.set(name, &value);
    }
    // Always said, so a room for two made after a match for four is for two.
    let players = players.clamp(2, sim::state::MAX_PLAYERS as u8);
    opts.set("players", &players.to_string());
    // How this run meets: the same brokers it was told to use, if any.
    for name in ["board", "broker"] {
        if let Some(value) = platform::value(name) {
            opts.set(name, value);
        }
    }
    opts
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
    let n = crate::chosen_players(opts) as usize;
    Some(Ok(match (opts.value("--board"), opts.value("--broker")) {
        (Some("tabs"), _) => Box::new(net::browser::tabs(&key, me, &terms).with_players(n)),
        (_, Some(url)) => Box::new(net::browser::brokers(&[url], &key, me, &terms).with_players(n)),
        _ => Box::new(net::browser::public(&key, me, &terms).with_players(n)),
    }))
}

#[cfg(not(target_arch = "wasm32"))]
fn rendezvous(opts: &Options, start: &World) -> Option<Result<Box<dyn net::Rendezvous>, String>> {
    if let Some(key) = room_key(opts) {
        let me = net::meet::fresh_id();
        let terms = terms(start);
        let n = crate::chosen_players(opts) as usize;
        return Some(match (opts.value("--board"), opts.value("--broker")) {
            (Some("tabs"), _) => Err(
                "board=tabs is two tabs of one browser, and this is not a browser. \
                     Drop it from the link to meet through the public brokers."
                    .into(),
            ),
            (_, Some(url)) => Ok(Box::new(
                net::native::brokers(&[url], &key, me, &terms).with_players(n),
            )),
            _ => Ok(Box::new(
                net::native::public(&key, me, &terms).with_players(n),
            )),
        });
    }
    let peer = opts.value("--peer")?;
    // Straight to an address is two machines: a room is how three or four meet.
    if crate::chosen_players(opts) > 2 {
        return Some(Err(
            "--peer reaches one other machine; three or four players meet in a room (--room)."
                .into(),
        ));
    }
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
            let players = seat.players;
            let roster = seat.roster.clone();
            let post = seat.post.clone();
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
                    // The match is a new fight from an agreed start, and so
                    // is its replay.
                    sim.tape = sim::replay::Tape::begin(&start, crate::build());
                    sim.tape.online = Some((handle as u8, delay as u8));
                    sim.cur = start;
                    sim.paused = false;
                    sim.rehearsing = None;
                    // The books are kept for this seat's fighter: its own
                    // regions and their neighbours, nobody else's.
                    sim.ledger.sit(handle);
                    // From the agreed start, as every peer's are.
                    sim.ledger.begin(&sim.cur);
                    Driver::Online {
                        session: Box::new(session),
                        handle,
                        desynced: false,
                        gone: false,
                        room,
                        paced: 0,
                        quiet: false,
                        meter: Meter {
                            since: sim.cur.frame,
                            ..Meter::default()
                        },
                        players,
                        roster,
                        post,
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
    let Driver::Online {
        session,
        paced,
        meter,
        ..
    } = &mut sim.driver
    else {
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
        meter.now.held += 1;
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
pub fn step(
    sim: &mut crate::Sim,
    local: SimInput,
    scape: &mut crate::sound::Soundscape,
    keeping: &mut crate::bag::Keeping,
) {
    let crate::Sim {
        driver,
        cur,
        prev,
        tape,
        ledger,
        ..
    } = sim;
    let Driver::Online {
        session,
        handle,
        desynced,
        roster,
        post,
        gone,
        quiet,
        meter,
        players,
        ..
    } = driver
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
    meter.now.quiet |= *quiet;

    // **Somebody's line closed**: they left. Their seat is given up now --
    // GGRS settles the frame with the others -- rather than after the
    // session's fifteen silent seconds, which froze the match for everybody
    // still in it (2026-10-22). Those seconds stay for a friend who is only
    // busy, drawing an arena for the first time.
    for peer in post.take_closed() {
        let net::Peer::Line(seat) = peer;
        if session.disconnect_player(seat as usize).is_ok() {
            eprintln!("player {} left", seat + 1);
            *gone = true;
        }
    }

    if session.current_state() != net::ggrs::SessionState::Running {
        return;
    }

    // **The region gate**, online as locally: the next frame goes ahead only
    // once every stream the world follows has said what it reads
    // (`docs/design/regions.md`; the crowd field's lookahead, 0018 S1).
    // GGRS is not asked for a frame the ledger would hold, and the books are
    // brought up to what is confirmed first, every tick (`net::may_advance`):
    // a gate that heard only after an advance froze on the first late packet.
    if !net::may_advance(ledger, session.confirmed_frame(), cur) {
        meter.now.stalls += 1;
        return;
    }

    if session
        .add_local_input(*handle, net::NetInput::from(local))
        .is_err()
    {
        // Too far ahead of the peer. Waiting is the correct response.
        meter.now.stalls += 1;
        return;
    }

    let was = cur.clone();
    match session.advance_frame() {
        Ok(requests) => {
            // A rollback is a load and then every frame since played again;
            // the last advance is the new frame, the rest are the replay.
            let loads = requests
                .iter()
                .filter(|r| matches!(r, net::ggrs::GgrsRequest::LoadGameState { .. }))
                .count() as u32;
            if loads > 0 {
                let advances = requests
                    .iter()
                    .filter(|r| matches!(r, net::ggrs::GgrsRequest::AdvanceFrame { .. }))
                    .count() as u32;
                let again = advances.saturating_sub(1);
                meter.now.rollbacks += loads;
                meter.now.resimulated += again;
                meter.now.deepest = meter.now.deepest.max(again);
            }
            let began = bevy::platform::time::Instant::now();
            // Every frame advanced goes on the tape by its frame number, so
            // a rollback's re-advance overwrites the prediction with what the
            // other player really pressed.
            // Kept on the region ledger -- its tape, and the crowd field's
            // due aggregates fed off it -- a re-advance overwriting its
            // prediction the same way the replay's does.
            let me = (*handle).min(sim::state::MAX_PLAYERS - 1);
            net::handle_requests_kept(cur, requests, ledger, |before, inputs, after| {
                tape.record(before.frame, inputs);
                // And sounded, the first time each frame is simulated.
                scape.observe(before, after);
                // And kept: what was picked up and fed, by frame, so a
                // re-advance corrects what a prediction got wrong.
                keeping.record(before, &inputs, after, me);
            });
            // The books are kept for confirmed frames only: what both
            // machines have agreed on, which is what a region's checksum
            // will one day be sent to a neighbour as.
            if let Ok(confirmed) = u32::try_from(session.confirmed_frame()) {
                ledger.confirm_through(confirmed, cur);
            }
            relay_the_crowd(
                cur,
                ledger,
                post,
                roster,
                *handle,
                &meter_peers(*players, *handle),
            );
            meter.now.sim_us = meter.now.sim_us.max(began.elapsed().as_micros() as u32);
            *prev = was;
        }
        Err(net::ggrs::GgrsError::PredictionThreshold) => meter.now.stalls += 1,
        Err(e) => eprintln!("advance failed: {e}"),
    }

    // A second is up: what the line looks like now, and the second's counts.
    if cur.frame >= meter.since + SECOND {
        // The worst line of them: what a laggy match is worth knowing.
        for other in (0..*players).filter(|h| h != handle) {
            if let Ok(stats) = session.network_stats(other) {
                meter.now.ping = meter.now.ping.max(stats.ping as u32);
                meter.now.queue = meter.now.queue.max(stats.send_queue_len as u32);
                meter.now.kbps = meter.now.kbps.max(stats.kbps_sent as u32);
            }
        }
        meter.now.ahead = session.frames_ahead();
        meter.now.frame = cur.frame;
        tape.measure(std::mem::take(&mut meter.now));
        meter.since = cur.frame;
    }
}

/// A frame was drawn online: how long it took, for the link's record.
pub fn drawn(sim: &mut crate::Sim, wall: std::time::Duration) {
    if let Driver::Online { meter, .. } = &mut sim.driver {
        meter.drawn(wall);
    }
}

/// Every other seat: whom a leader may post to.
fn meter_peers(players: usize, me: usize) -> Vec<usize> {
    (0..players).filter(|h| *h != me).collect()
}

/// **A cell leader's duty, and a listener's** (`docs/design/exploration/0018`
/// §5). At an epoch's frame the ledger summed every live region's aggregate;
/// for each one this peer leads this epoch, post it to every other peer that
/// leads a region next to it. And collect what the others posted -- two
/// leaders agreeing, or one unchallenged for an epoch -- and offer it to the
/// ledger, which feeds the field at the lookahead.
///
/// Whom to post to is read off the predicted world rather than a confirmed
/// one: a wrong guess is a packet to the wrong peer, never a different
/// simulation, and the gate holds for a stream that goes unheard.
fn relay_the_crowd(
    cur: &sim::World,
    ledger: &mut regions::Ledger,
    post: &net::crowd::Post,
    roster: &[u64],
    me: usize,
    others: &[usize],
) {
    let grid = sim::region::Grid::tuned();
    let epoch = (cur.frame / sim::tuning::crowd_epoch()) as u16;
    let members: Vec<net::cells::Member> = roster
        .iter()
        .enumerate()
        .map(|(seat, id)| net::cells::Member { seat, id: *id })
        .collect();
    for a in ledger.summed() {
        if !net::cells::leads(cur, &grid, &members, a.region, epoch, me) {
            continue;
        }
        for &other in others {
            let leads_a_neighbour = a
                .region
                .neighbours()
                .iter()
                .any(|n| net::cells::leads(cur, &grid, &members, *n, epoch, other));
            if leads_a_neighbour {
                post.post(net::Peer::Line(other as u8), a);
            }
        }
    }
    for a in post.collect(cur.frame, sim::tuning::crowd_epoch()) {
        ledger.offer(a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A room made from the menu is for the players chosen**, whatever the
    /// practice had: the link says so, and the world it starts seats them all.
    #[test]
    fn a_room_from_the_menu_is_for_as_many_as_were_chosen() {
        let practice = crate::start_world(&Options::default(), crate::Dummy::Human);
        assert_eq!(practice.present(), 2);
        for players in 2..=4u8 {
            let opts = room_settings(&practice, players);
            assert_eq!(crate::chosen_players(&opts), players);
            assert!(
                link_of(&opts).contains(&format!("players={players}")),
                "the link for {players}: {}",
                link_of(&opts)
            );
            let world = crate::start_world(&opts, crate::Dummy::Human);
            assert_eq!(world.present(), players as usize, "{players} seated");
        }
        // And back to two after a match for four.
        let four = crate::start_world(&room_settings(&practice, 4), crate::Dummy::Human);
        let opts = room_settings(&four, 2);
        assert_eq!(crate::start_world(&opts, crate::Dummy::Human).present(), 2);
    }
}
