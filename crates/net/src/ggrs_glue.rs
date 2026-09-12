//! GGRS wiring.
//!
//! GGRS requires its input type to be `serde`-serialisable. Rather than put a
//! dependency on `sim` -- whose zero-dependency status is a determinism
//! guarantee, not an accident -- the serialisable wrapper lives here.

use ggrs::{Config, GgrsRequest};
use serde::{Deserialize, Serialize};
use sim::state::MAX_PLAYERS;
use sim::{Input, World};

/// Wire format for one player's input. This is the only game data that crosses
/// the network: six bytes of payload per player per frame.
///
/// Buttons, then yaw, then pitch, each in its own sixteen bits. All three are on
/// the wire because movement and abilities resolve relative to where the player
/// is looking, so the look direction is gameplay -- and gameplay has to match on
/// both peers to the bit. Carrying it as input means rollback predicts and
/// corrects it with the same machinery as a button press, and the camera never
/// enters the snapshot.
///
/// Pitch joined it when abilities started landing where the crosshair is. A
/// crosshair is a line in space and a line needs two angles; with only the yaw
/// on the wire, two peers would disagree about where a fire pillar went.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
pub struct NetInput(pub u64);

impl From<NetInput> for Input {
    fn from(n: NetInput) -> Input {
        Input::looking_at((n.0 >> 32) as u16, (n.0 >> 16) as u16, n.0 as u16 as i16)
    }
}

impl From<Input> for NetInput {
    fn from(i: Input) -> NetInput {
        NetInput((i.bits as u64) << 32 | (i.aim as u64) << 16 | i.pitch as u16 as u64)
    }
}

/// Session configuration. `Address` is a plain socket address because the
/// prototype connects peers directly by IP with no matchmaking server.
#[derive(Debug)]
pub struct SessionConfig;

impl Config for SessionConfig {
    type Input = NetInput;
    type State = World;
    type Address = std::net::SocketAddr;
}

/// Service one batch of GGRS requests against the simulation.
///
/// This is the whole integration: GGRS decides *when* to save, load and
/// advance; the simulation only has to do those three things correctly.
pub fn handle_requests(world: &mut World, requests: Vec<GgrsRequest<SessionConfig>>) {
    for request in requests {
        match request {
            GgrsRequest::SaveGameState { cell, frame } => {
                debug_assert_eq!(frame, world.frame as i32);
                let checksum = world.checksum();
                cell.save(frame, Some(world.clone()), Some(checksum as u128));
            }
            GgrsRequest::LoadGameState { cell, .. } => {
                *world = cell.load().expect("ggrs asked to load an empty cell");
            }
            GgrsRequest::AdvanceFrame { inputs } => {
                let mut frame_inputs = [Input::default(); MAX_PLAYERS];
                for (slot, (input, _status)) in frame_inputs.iter_mut().zip(inputs.iter()) {
                    *slot = (*input).into();
                }
                world.advance(frame_inputs);
            }
        }
    }
}
