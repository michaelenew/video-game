//! Starting a rollback session -- two to four players -- on a seat a meeting
//! found.
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

/// Frames before your own input takes effect, when nothing else says.
///
/// It was two, the usual starting point for a fighting game, and the first
/// people to play online felt it at once against the local game; then none,
/// so a press acted on the next frame and rollback hid the network. **Two
/// again since 2026-10-22**, and a setting in the Esc menu
/// (`game::settings::Settings::input_delay`): with four in a room the far
/// inputs arrived about seven frames late, every hiccup stalled the match,
/// and two frames of delay take two off every input's lateness -- for
/// thirty-three milliseconds on your own presses. Each side's delay is its
/// own; `delay=` (`--delay`) overrides it for a run.
pub const INPUT_DELAY: usize = 2;

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

/// Start a session for everybody the meeting seated. `seat.handle` is which
/// player you are, and decides where in the arena you stand; the meeting has
/// already made every peer agree on it.
pub fn start(seat: Seat) -> Result<P2PSession<SessionConfig>, GgrsError> {
    start_with_delay(seat, INPUT_DELAY)
}

/// [`start`], with `delay` frames of input delay instead of [`INPUT_DELAY`].
pub fn start_with_delay(seat: Seat, delay: usize) -> Result<P2PSession<SessionConfig>, GgrsError> {
    start_with(seat, delay, DISCONNECT_AFTER)
}

/// [`start_with_delay`], giving up on a silent peer after `disconnect_after`
/// rather than [`DISCONNECT_AFTER`]: what lets a test watch somebody leave
/// without waiting a quarter of a minute for it.
pub fn start_with(
    seat: Seat,
    delay: usize,
    disconnect_after: Duration,
) -> Result<P2PSession<SessionConfig>, GgrsError> {
    let mut builder = SessionBuilder::<SessionConfig>::new()
        .with_num_players(seat.players)
        .with_input_delay(delay)
        .with_max_prediction_window(crate::MAX_ROLLBACK_FRAMES)
        .with_disconnect_timeout(disconnect_after)
        .with_disconnect_notify_delay(SILENT_AFTER)
        .with_desync_detection_mode(DesyncDetection::On {
            interval: DESYNC_CHECK_INTERVAL,
        })
        .add_player(PlayerType::Local, seat.handle)?;
    for (handle, peer) in seat.remotes {
        builder = builder.add_player(PlayerType::Remote(peer), handle)?;
    }
    builder.start_p2p_session(seat.socket)
}
