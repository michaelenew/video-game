//! Playing against a person.
//!
//!     game --port 47811 --peer 192.168.1.20:47812
//!
//! Online routes every tick through GGRS, which owns when to save, load and
//! advance — the simulation only has to do those three things correctly. Both
//! peers derive who is player one from the two addresses, so there is no server
//! and no lobby.
//!
//! **The browser build has no peer, and this file is where that is decided.** A
//! page cannot open a UDP socket, so on wasm `Driver` has one variant and the
//! rest of the crate does not notice it went. That is the entire cost of the
//! decision: the browser is one player and a training dummy, which is what a
//! link is for — somebody trying the thing out, not a match. Rollback over
//! WebRTC data channels is the route back if it is ever wanted; see
//! [`docs/design/web.md`](../../../docs/design/web.md).

// Both of these belong to the peer, which the browser does not have.
#[cfg(not(target_arch = "wasm32"))]
use crate::platform;
#[cfg(not(target_arch = "wasm32"))]
use sim::Input as SimInput;

/// How the match is being driven.
///
/// Local is the training mode.
pub enum Driver {
    Local,
    #[cfg(not(target_arch = "wasm32"))]
    Online {
        session: Box<net::ggrs::P2PSession<net::SessionConfig>>,
        handle: usize,
        desynced: bool,
    },
}

impl Driver {
    /// Which fighter this client drives. Online it is the GGRS handle; locally
    /// it is always player one, with player two on the second key set.
    pub fn local_player(&self) -> usize {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Driver::Online { handle, .. } => (*handle).min(1),
            Driver::Local => 0,
        }
    }
}

/// Open a session if this run was given a peer, and fall back to training mode
/// if it was not, or if the socket would not open.
#[cfg(not(target_arch = "wasm32"))]
pub fn start() -> Driver {
    let Some((port, peer)) = peer_from_options() else {
        return Driver::Local;
    };
    let local: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().expect("local addr");
    let handle = net::p2p::local_handle_for(local, peer);
    match net::p2p::start(port, peer, handle) {
        Ok(session) => {
            eprintln!(
                "online: port {port} to {peer}, you are player {}",
                handle + 1
            );
            Driver::Online {
                session: Box::new(session),
                handle,
                desynced: false,
            }
        }
        Err(e) => {
            eprintln!("could not start the session ({e}); falling back to training mode");
            Driver::Local
        }
    }
}

/// Training mode, always. There is nothing else a browser can do.
#[cfg(target_arch = "wasm32")]
pub fn start() -> Driver {
    Driver::Local
}

#[cfg(not(target_arch = "wasm32"))]
fn peer_from_options() -> Option<(u16, std::net::SocketAddr)> {
    let port: u16 = platform::value("--port")?.parse().ok()?;
    let peer: std::net::SocketAddr = platform::value("--peer")?.parse().ok()?;
    Some((port, peer))
}

/// One networked tick.
///
/// GGRS decides when to save, load and advance; `handle_requests` services
/// those against the simulation. Rollbacks land here as a load followed by
/// several advances, all inside one call.
#[cfg(not(target_arch = "wasm32"))]
pub fn step(sim: &mut crate::Sim, local: SimInput) {
    let Driver::Online {
        session,
        handle,
        desynced,
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
            net::ggrs::GgrsEvent::Disconnected { addr } => eprintln!("peer {addr} disconnected"),
            net::ggrs::GgrsEvent::NetworkInterrupted { addr, .. } => {
                eprintln!("peer {addr} interrupted")
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
