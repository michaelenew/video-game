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
use std::time::Duration;

/// How often peers exchange state checksums. GGRS raises a desync event when
/// they disagree, which is the whole reason the simulation is integer-only.
const DESYNC_CHECK_INTERVAL: u32 = 30;

/// Frames before your own input takes effect. **None**: a press acts on the
/// next frame, exactly as it does playing locally, and rollback hides the
/// network -- which is the reason to have rollback at all.
///
/// It was two, the usual starting point for a fighting game, and the first
/// people to play online felt it at once against the local game. The price
/// of zero is that every surprise from the far side is corrected rather than
/// waited for: rollbacks are as long as half the round trip instead of two
/// frames shorter, which the frame budget allows for up to the session's
/// eight-frame prediction window (`docs/design/architecture.md`). A link that
/// rolls back so much the other fighter jitters can trade some of it back with
/// `delay=` (`--delay`), per player, since each side's delay is its own.
pub const INPUT_DELAY: usize = 0;

/// How long the other side may go silent before it counts as gone. GGRS's
/// default is two seconds, and a page goes silent for longer than that drawing
/// the first frame of an arena it has not drawn before -- every new material
/// is a shader compiled on the page's one thread. So pressing H or N
/// disconnected your friend: the side that pressed said so and played on with
/// blank inputs for them, and the other side, still in the match, saw that as
/// a desync. Fifteen seconds outlasts any load, and a friend who has really
/// gone is still noticed.
const DISCONNECT_AFTER: Duration = Duration::from_secs(15);

/// When a silence is worth saying something about (`NetworkInterrupted`).
const SILENT_AFTER: Duration = Duration::from_secs(1);

/// Start a two-player session. `seat.handle` is 0 or 1 and decides which side
/// of the arena you are; the meeting has already made both peers agree on it.
pub fn start(seat: Seat) -> Result<P2PSession<SessionConfig>, GgrsError> {
    start_with_delay(seat, INPUT_DELAY)
}

/// [`start`], with `delay` frames of input delay instead of [`INPUT_DELAY`].
pub fn start_with_delay(seat: Seat, delay: usize) -> Result<P2PSession<SessionConfig>, GgrsError> {
    SessionBuilder::<SessionConfig>::new()
        .with_num_players(2)
        .with_input_delay(delay)
        .with_disconnect_timeout(DISCONNECT_AFTER)
        .with_disconnect_notify_delay(SILENT_AFTER)
        .with_desync_detection_mode(DesyncDetection::On {
            interval: DESYNC_CHECK_INTERVAL,
        })
        .add_player(PlayerType::Local, seat.handle)?
        .add_player(PlayerType::Remote(seat.remote), 1 - seat.handle)?
        .start_p2p_session(seat.socket)
}
