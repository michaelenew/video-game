//! Rollback session management.
//!
//! The simulation knows nothing about networking; this crate knows nothing
//! about gameplay. The whole contract is `Rollback` below.
//!
//! Shaped deliberately like the GGRS session handler so that dropping in
//! `ggrs` locally is an impl block, not a redesign.

use sim::state::MAX_PLAYERS;
use sim::{Input, World};

/// What a rollback session needs from a simulation.
pub trait Rollback {
    type Snapshot: Clone;

    fn advance(&mut self, inputs: [Input; MAX_PLAYERS]);
    fn save(&self) -> Self::Snapshot;
    fn load(&mut self, snap: &Self::Snapshot);
    fn checksum(&self) -> u64;
}

impl Rollback for World {
    type Snapshot = World;

    fn advance(&mut self, inputs: [Input; MAX_PLAYERS]) {
        World::advance(self, inputs);
    }
    fn save(&self) -> World {
        self.clone()
    }
    fn load(&mut self, snap: &World) {
        *self = snap.clone();
    }
    fn checksum(&self) -> u64 {
        World::checksum(self)
    }
}

/// Number of frames of history kept. Bounds how far a peer can be behind
/// before the session stalls instead of rolling back.
pub const MAX_ROLLBACK_FRAMES: usize = 8;

/// A local stand-in for a networked session.
///
/// Runs the real prediction-and-rollback loop against a simulated peer, so
/// rollback correctness can be tested with no sockets and no second machine.
/// This is the harness that catches non-determinism early, when it is still
/// cheap to fix.
pub struct LocalSession<S: Rollback> {
    sim: S,
    history: Vec<(u32, S::Snapshot)>,
    /// Inputs confirmed by the peer, oldest first.
    confirmed: Vec<[Input; MAX_PLAYERS]>,
    /// Inputs we predicted and have not yet confirmed.
    predicted: Vec<[Input; MAX_PLAYERS]>,
    frame: u32,
    pub rollbacks: u32,
}

impl<S: Rollback> LocalSession<S> {
    pub fn new(sim: S) -> Self {
        let snap = sim.save();
        LocalSession {
            sim,
            history: vec![(0, snap)],
            confirmed: Vec::new(),
            predicted: Vec::new(),
            frame: 0,
            rollbacks: 0,
        }
    }

    pub fn frame(&self) -> u32 {
        self.frame
    }

    pub fn sim(&self) -> &S {
        &self.sim
    }

    /// Advance one frame using a predicted remote input.
    pub fn predict_and_advance(&mut self, inputs: [Input; MAX_PLAYERS]) {
        self.predicted.push(inputs);
        self.sim.advance(inputs);
        self.frame += 1;
        self.history.push((self.frame, self.sim.save()));
        if self.history.len() > MAX_ROLLBACK_FRAMES + 1 {
            self.history.remove(0);
        }
    }

    /// The peer's real inputs arrive. If a prediction was wrong, roll back to
    /// the last agreed frame and re-simulate forward.
    ///
    /// Returns true if a rollback occurred.
    pub fn confirm(&mut self, actual: &[[Input; MAX_PLAYERS]]) -> bool {
        let mismatch = self
            .predicted
            .iter()
            .zip(actual.iter())
            .position(|(p, a)| p != a);

        let Some(offset) = mismatch else {
            self.confirmed.extend_from_slice(actual);
            self.predicted
                .drain(..actual.len().min(self.predicted.len()));
            return false;
        };

        let target = self.frame - self.predicted.len() as u32 + offset as u32;
        let snap = self
            .history
            .iter()
            .find(|(f, _)| *f == target)
            .map(|(_, s)| s.clone())
            .expect("rolled back further than history allows");

        self.sim.load(&snap);
        self.frame = target;
        self.history.retain(|(f, _)| *f <= target);
        self.rollbacks += 1;

        let replay: Vec<_> = actual[offset..].to_vec();
        self.predicted.clear();
        self.confirmed.extend_from_slice(&actual[..offset]);
        for inputs in replay {
            self.confirmed.push(inputs);
            self.sim.advance(inputs);
            self.frame += 1;
            self.history.push((self.frame, self.sim.save()));
        }
        true
    }
}
