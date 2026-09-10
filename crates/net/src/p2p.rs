//! Peer-to-peer sessions over UDP.
//!
//! No server. Two peers exchange an address and connect directly, which is all
//! a prototype needs and all a LAN match ever needs.
//!
//! **Honest about "no server ever":** direct connections between arbitrary home
//! networks eventually need NAT traversal, which means a small STUN or relay
//! service. Not needed now; worth not being surprised by later.

use ggrs::{
    DesyncDetection, GgrsError, P2PSession, PlayerType, SessionBuilder, UdpNonBlockingSocket,
};
use std::net::SocketAddr;

/// Why a session could not start. Binding is our problem; everything else is
/// GGRS reporting a misconfigured session.
#[derive(Debug)]
pub enum StartError {
    Bind(std::io::Error),
    Session(GgrsError),
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartError::Bind(e) => write!(f, "could not bind the UDP socket: {e}"),
            StartError::Session(e) => write!(f, "could not start the session: {e}"),
        }
    }
}

impl std::error::Error for StartError {}

impl From<GgrsError> for StartError {
    fn from(e: GgrsError) -> Self {
        StartError::Session(e)
    }
}

use crate::SessionConfig;

/// How often peers exchange state checksums. GGRS raises a desync event when
/// they disagree, which is the whole reason the simulation is integer-only.
const DESYNC_CHECK_INTERVAL: u32 = 30;

/// Frames of input delay. Higher hides more latency at the cost of feeling
/// less responsive; two is the usual starting point for a fighting game.
pub const INPUT_DELAY: usize = 2;

/// Start a two-player session.
///
/// `local_handle` is 0 or 1 and decides which side of the arena you are. Both
/// peers must agree, which is what the room handshake settles -- see
/// `handshake` below.
pub fn start(
    local_port: u16,
    remote: SocketAddr,
    local_handle: usize,
) -> Result<P2PSession<SessionConfig>, StartError> {
    let socket = UdpNonBlockingSocket::bind_to_port(local_port).map_err(StartError::Bind)?;

    Ok(SessionBuilder::<SessionConfig>::new()
        .with_num_players(2)
        .with_input_delay(INPUT_DELAY)
        .with_desync_detection_mode(DesyncDetection::On {
            interval: DESYNC_CHECK_INTERVAL,
        })
        .add_player(PlayerType::Local, local_handle)?
        .add_player(PlayerType::Remote(remote), 1 - local_handle)?
        .start_p2p_session(socket)?)
}

/// Decide who is player one without a server.
///
/// Both peers know both addresses, so ordering them gives the same answer on
/// both machines. Crude, deterministic, and enough until there is matchmaking.
pub fn local_handle_for(local: SocketAddr, remote: SocketAddr) -> usize {
    let key = |a: SocketAddr| (a.ip().to_string(), a.port());
    if key(local) <= key(remote) { 0 } else { 1 }
}
