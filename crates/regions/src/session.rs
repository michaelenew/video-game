//! **A peer of the open world, running one region**: what the session layer
//! GGRS cannot provide will be built from (`docs/design/exploration/0005`,
//! §"What it would need"; `0018` §1). This is the shape, and the experiment
//! that proves the shape exact, with no sockets:
//!
//! - A [`Peer`] holds one region's world. It has the **inputs of its
//!   members** -- the fighters within the grid's membership radius of its
//!   hexagon -- and nothing from anybody else, whose inputs it takes as
//!   nothing pressed.
//! - A neighbour says what happened in *its* checksum zone as an
//!   [`Outcome`]: the bodies standing there at a frame, exactly. A peer that
//!   receives one **rolls back** to that frame, puts the outcome's bodies
//!   where the neighbour says they were, and runs forward again with the
//!   inputs it recorded. So the strip between a peer's zone and its
//!   membership radius is driven by the neighbours' outcomes, up to `delay`
//!   frames late, and the zone is exact at every frame an outcome has
//!   arrived for -- because nothing outside the strip can reach the zone in
//!   `delay` frames (`Grid::horizon`).
//!
//! Nothing here is read by the game yet. It is the experiment the design
//! asked for, kept as a test (`tests/session.rs`), and the piece the wire
//! will carry.

use crate::{TAPE, Tape};
use sim::World;
use sim::input::Input;
use sim::region::{Grid, RegionId};
use sim::state::{MAX_PLAYERS, Player};

/// What a region says happened in its checksum zone at one frame: every
/// fighter standing there, exactly. (Effects, bolts and creatures are the
/// next thing an outcome carries; a fighter is the first, because a fighter
/// is the one body whose future depends on an input the peer does not have.)
#[derive(Clone)]
pub struct Outcome {
    pub region: RegionId,
    pub frame: u32,
    pub fighters: [Option<Player>; MAX_PLAYERS],
}

/// One region, run by one peer.
pub struct Peer {
    pub region: RegionId,
    pub grid: Grid,
    pub world: World,
    tape: Box<Tape>,
    /// Which seats this peer has the inputs of, as it last decided.
    members: [bool; MAX_PLAYERS],
    /// The newest frame every neighbour's outcome has been taken for: what
    /// the zone is exact through.
    pub exact_through: u32,
}

impl Peer {
    /// A peer for `region`, starting from `world`.
    pub fn new(region: RegionId, grid: Grid, world: World) -> Peer {
        Peer {
            region,
            grid,
            exact_through: world.frame,
            world,
            tape: Box::new(Tape::new()),
            members: [false; MAX_PLAYERS],
        }
    }

    /// Whether a fighter at `at` (on the map) is a member: its inputs are
    /// this peer's to have. **One horizon further out than the zone is
    /// exact**: a member's inputs have to be in hand for the `delay` frames
    /// *before* the frame it could first touch the zone, because the outcome
    /// that puts its state right arrives that late and is played forward
    /// with the inputs recorded since. So inputs flow from `membership +
    /// horizon` out; the exactness argument needs only `membership`.
    pub fn admits(&self, at: sim::math::V3) -> bool {
        self.grid.gap(self.region, at).raw()
            <= self.grid.membership().add(self.grid.horizon()).raw()
    }

    /// **One frame**, given every seat's input: the peer keeps the members'
    /// and takes the rest as nothing pressed, which is what it would have on
    /// the wire. `members` is who is a member this frame -- decided by the
    /// session layer the same way on every machine.
    pub fn advance(&mut self, members: [bool; MAX_PLAYERS], inputs: [Input; MAX_PLAYERS]) {
        self.members = members;
        let mine: [Input; MAX_PLAYERS] = std::array::from_fn(|i| {
            if members[i] {
                inputs[i]
            } else {
                Input::default()
            }
        });
        self.tape.record(&self.world, mine);
        self.world.advance(mine);
    }

    /// The world as it was at `frame`: now, or off the tape.
    fn world_at(&self, frame: u32) -> Option<World> {
        if frame == self.world.frame {
            Some(self.world.clone())
        } else {
            self.tape
                .get(frame.wrapping_add(1))
                .map(|e| e.before.clone())
        }
    }

    /// What this region says happened at `frame`: its zone's fighters.
    pub fn outcome(&self, frame: u32) -> Option<Outcome> {
        let w = self.world_at(frame)?;
        let origin = w.map_origin();
        let fighters = std::array::from_fn(|i| {
            let p = w.players[i];
            self.grid
                .in_zone(self.region, p.pos.add(origin))
                .then_some(p)
        });
        Some(Outcome {
            region: self.region,
            frame,
            fighters,
        })
    }

    /// **Take a neighbour's outcome**: roll back to its frame, put its
    /// fighters where it says, and play forward again with the inputs
    /// recorded. A fighter this peer has the inputs of is left alone -- it
    /// is exact here already, and the neighbour agrees. Returns whether the
    /// outcome could be taken (the tape still had its frame).
    pub fn accept(&mut self, outcome: &Outcome) -> bool {
        if outcome.frame > self.world.frame || outcome.region == self.region {
            return false;
        }
        let Some(mut w) = self.world_at(outcome.frame) else {
            return false;
        };
        // Every fighter the outcome reports, members included: a member in
        // the neighbour's zone is exact on both sides and the overwrite is
        // no change; a fighter that has only just become a member was
        // predicted with nothing pressed until it did, and this is where
        // its true path arrives.
        let mut changed = false;
        for (i, p) in outcome.fighters.iter().enumerate() {
            if let Some(p) = p
                && w.players[i] != *p
            {
                w.players[i] = *p;
                changed = true;
            }
        }
        if changed {
            // Forward again, from the frame after the outcome's to now,
            // with the inputs each frame was played with.
            let mut f = outcome.frame;
            while f < self.world.frame {
                let next = f.wrapping_add(1);
                let Some(e) = self.tape.get(next) else {
                    return false;
                };
                let inputs = e.inputs;
                self.tape.record(&w, inputs);
                w.advance(inputs);
                f = next;
            }
            self.world = w;
        }
        self.exact_through = self.exact_through.max(outcome.frame);
        true
    }

    /// The region's checksum at `frame`, off the tape.
    pub fn zone_checksum(&self, frame: u32) -> Option<u64> {
        self.world_at(frame)
            .map(|w| w.region_checksum(&self.grid, self.region))
    }

    /// How many frames the tape holds, which is how late an outcome may be.
    pub const fn tape_len() -> usize {
        TAPE
    }
}
