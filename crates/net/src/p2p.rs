//! Starting a two-player rollback session on a seat a meeting found.
//!
//! How the two players found each other is [`crate::meet`]'s business; this
//! file only knows that there is a socket, a far end, and a side of the arena.
//!
//! **Honest about "no server ever":** two players still need somewhere to
//! leave each other a note before they can talk directly. On a LAN, or with a
//! typed-in address, that is the UDP port itself ([`crate::direct`]). Between
//! two browsers it is a public message broker, and the direct connection is
//! WebRTC, which asks a public STUN server how this machine looks from outside
//! ([`crate::browser`]). Neither carries a frame of the match. Some network
//! pairs will not take a direct connection at all; those would need a relay
//! (TURN), which does carry the match and is not free to run.

use crate::SessionConfig;
use crate::meet::Seat;
use ggrs::{DesyncDetection, GgrsError, P2PSession, PlayerType, SessionBuilder};

/// How often peers exchange state checksums. GGRS raises a desync event when
/// they disagree, which is the whole reason the simulation is integer-only.
const DESYNC_CHECK_INTERVAL: u32 = 30;

/// Frames of input delay. Higher hides more latency at the cost of feeling
/// less responsive; two is the usual starting point for a fighting game.
pub const INPUT_DELAY: usize = 2;

/// Start a two-player session. `seat.handle` is 0 or 1 and decides which side
/// of the arena you are; the meeting has already made both peers agree on it.
pub fn start(seat: Seat) -> Result<P2PSession<SessionConfig>, GgrsError> {
    SessionBuilder::<SessionConfig>::new()
        .with_num_players(2)
        .with_input_delay(INPUT_DELAY)
        .with_desync_detection_mode(DesyncDetection::On {
            interval: DESYNC_CHECK_INTERVAL,
        })
        .add_player(PlayerType::Local, seat.handle)?
        .add_player(PlayerType::Remote(seat.remote), 1 - seat.handle)?
        .start_p2p_session(seat.socket)
}
