//! GGRS wiring.
//!
//! GGRS requires its input type to be `serde`-serialisable. Rather than put a
//! dependency on `sim` -- whose zero-dependency status is a determinism
//! guarantee, not an accident -- the serialisable wrapper lives here.

use ggrs::{Config, GgrsRequest, InputStatus};
use serde::{Deserialize, Serialize};
use sim::state::MAX_PLAYERS;
use sim::{Input, World};

/// Wire format for one player's input. This is the only game data that crosses
/// the network: seven bytes of payload per player per frame.
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
            .travelling(sim::input::Travel((n.0 >> 48) as u16))
    }
}

impl From<Input> for NetInput {
    fn from(i: Input) -> NetInput {
        NetInput(
            (i.travel.0 as u64) << 48
                | (i.bits as u64) << 32
                | (i.aim as u64) << 16
                | i.pitch as u16 as u64,
        )
    }
}

/// Session configuration.
///
/// `Address` is [`crate::Peer`] rather than a socket address because the other
/// player is not always at one: in a browser they are at the far end of a
/// WebRTC data channel, which has no address a page can see. One session type
/// for every way of meeting is what lets `game` hold a single `Driver::Online`.
#[derive(Debug)]
pub struct SessionConfig;

impl Config for SessionConfig {
    type Input = NetInput;
    type State = World;
    type Address = crate::Peer;
}

/// Service one batch of GGRS requests against the simulation.
///
/// This is the whole integration: GGRS decides *when* to save, load and
/// advance; the simulation only has to do those three things correctly.
pub fn handle_requests(world: &mut World, requests: Vec<GgrsRequest<SessionConfig>>) {
    handle_requests_watched(world, requests, |_, _, _| {});
}

/// [`handle_requests`], showing `watch` every frame advanced -- the world
/// before, the inputs that moved it on, and the world after -- predicted or
/// confirmed, re-advanced after a rollback or not. What the game's replay
/// records (by frame number, so the last word on a frame is the confirmed
/// one) and what its sound listens to (once per frame, the first time).
pub fn handle_requests_watched(
    world: &mut World,
    requests: Vec<GgrsRequest<SessionConfig>>,
    mut watch: impl FnMut(&World, [Input; MAX_PLAYERS], &World),
) {
    advance_each(world, requests, |world, inputs| {
        // A flat copy, like every snapshot the session saves: no heap.
        let before = world.clone();
        world.advance(inputs);
        watch(&before, inputs, world);
    });
}

/// [`handle_requests_watched`], **kept on the region ledger**: every frame
/// made goes on its tape, and is fed the crowd field's aggregates due at it
/// (`sim::crowd`, `regions::Ledger::due`) -- re-recorded and re-fed on a
/// rollback's re-advance the way inputs are. What the game's online driver
/// calls.
pub fn handle_requests_kept(
    world: &mut World,
    requests: Vec<GgrsRequest<SessionConfig>>,
    ledger: &mut regions::Ledger,
    mut watch: impl FnMut(&World, [Input; MAX_PLAYERS], &World),
) {
    advance_each(world, requests, |world, inputs| {
        let before = world.clone();
        ledger.record(&before, inputs);
        let (fed, n) = ledger.due(world.frame.wrapping_add(1));
        world.advance_with(inputs, &fed[..n]);
        watch(&before, inputs, world);
    });
}

/// The one loop under both: GGRS's saves and loads, and `advance(world,
/// inputs)` for every frame it asks made, with the inputs as the simulation
/// reads them (a disconnected seat's as a departure).
fn advance_each(
    world: &mut World,
    requests: Vec<GgrsRequest<SessionConfig>>,
    mut advance: impl FnMut(&mut World, [Input; MAX_PLAYERS]),
) {
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
                for (slot, (input, status)) in frame_inputs.iter_mut().zip(inputs.iter()) {
                    // **A seat GGRS has given up on has left the match.** It
                    // marks the seat disconnected from the same frame on
                    // every peer, and from then on hands us a blank input
                    // for it -- which the world used to play as a fighter
                    // standing still and turning to face one way. Said as a
                    // trip instead, the world takes the fighter out of it
                    // (`sim::state::World::away`).
                    *slot = match status {
                        InputStatus::Disconnected => {
                            Input::default().travelling(sim::input::Travel::LEFT)
                        }
                        _ => (*input).into(),
                    };
                }
                advance(world, frame_inputs);
            }
        }
    }
}
