//! **What you carry, and every seat's morph: this game's own** (2026-10-19).
//!
//! None of it is in the world the players agree on. The owner: *"using an
//! item is something we can treat like a move / input. It just happens
//! when the user clicks the key, and then both players agree that the item
//! was used rather than needing to agree that the item was always there."*
//! So the world keeps only what the fight reads -- what lies on the floor,
//! and who carries which egg (`sim::keep`) -- and this keeps the rest, from
//! what the world says happened:
//!
//! - **A trophy picked up** is in `World::picked` on the frame it happens;
//!   it goes in this player's bag. (An egg is carried in the world, so the
//!   bag shows the eggs the world says this player has.)
//! - **A feeding** is an input naming the item (`Travel::feed`): every
//!   seat's morph eats what its player fed it, on every machine, so each
//!   draws every morph the same.
//!
//! **A rollback is a correction, not a second time.** The game sees every
//! frame it advances, predicted or confirmed, and a rollback advances some
//! again with what the other player really pressed. So what happened is
//! kept by the frame it happened on -- the last word on a frame is the one
//! that stands, as the replay's tape keeps inputs -- and settled into the
//! bag and the morphs only once it is too old for a rollback to reach.

use bevy::prelude::*;
use sim::breed::archetypes;
use sim::breed::morph::Morph;
use sim::input::{Destination, Input};
use sim::keep::Item;
use sim::species::SpeciesId;
use sim::state::{MAX_PLAYERS, World};

/// Frames back a rollback can reach, with room to spare: what happened
/// this long ago is settled.
const WINDOW: u32 = 64;

/// What happened on one frame that this keeps.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct Happened {
    /// What each seat picked up, or was given.
    picked: [Option<Item>; MAX_PLAYERS],
    /// What each seat fed its morph.
    fed: [Option<Item>; MAX_PLAYERS],
}

impl Happened {
    fn is_empty(&self) -> bool {
        self.picked
            .iter()
            .chain(self.fed.iter())
            .all(|x| x.is_none())
    }
}

/// The bag and the morphs, as of some frame.
#[derive(Clone, Debug)]
pub struct Held {
    /// This player's trophies, in the order they were picked up.
    pub bag: Vec<Item>,
    pub morphs: [Morph; MAX_PLAYERS],
    /// The frame each seat's morph last ate on.
    pub fed_at: [Option<u32>; MAX_PLAYERS],
}

impl Default for Held {
    fn default() -> Held {
        // A morph that has eaten no egg: the smallest body there is,
        // young -- drawn as a blob until it eats one.
        let base = archetypes::of_species(SpeciesId::GNAWERS)
            .copied()
            .unwrap_or(archetypes::RIDGEBACK);
        Held {
            bag: Vec::new(),
            morphs: core::array::from_fn(|i| Morph::hatch(&base, i as u8)),
            fed_at: [None; MAX_PLAYERS],
        }
    }
}

impl Held {
    /// One frame's happenings, for the player in seat `me`.
    fn apply(&mut self, frame: u32, h: &Happened, me: usize) {
        if let Some(Item::Trophy(k, s)) = h.picked[me] {
            self.bag.push(Item::Trophy(k, s));
        }
        for (i, fed) in h.fed.iter().enumerate() {
            let Some(item) = *fed else { continue };
            if i == me
                && let Some(at) = self.bag.iter().position(|b| *b == item)
            {
                self.bag.remove(at);
            }
            match item {
                Item::Egg(s) => {
                    if let Some(g) = archetypes::of_species(s) {
                        self.morphs[i].eat_egg(g, s.0);
                    }
                }
                Item::Trophy(kind, s) => {
                    if let Some(g) = archetypes::of_species(s) {
                        self.morphs[i].eat(kind, g);
                    }
                }
            }
            self.fed_at[i] = Some(frame);
        }
    }
}

/// The ledger: what is settled, what is recent enough to be corrected,
/// and the two together.
#[derive(Resource, Default)]
pub struct Keeping {
    /// The seat this game plays.
    me: usize,
    settled: Held,
    /// By the frame it happened on (the world's frame before it), oldest
    /// first.
    pending: Vec<(u32, Happened)>,
    newest: u32,
    now: Held,
}

impl Keeping {
    /// The bag and the morphs now.
    pub fn held(&self) -> &Held {
        &self.now
    }

    /// One frame advanced: `before` became `after` on `inputs`, for the
    /// player in seat `me`. Every frame, predicted or confirmed, and again
    /// after a rollback.
    pub fn record(
        &mut self,
        before: &World,
        inputs: &[Input; MAX_PLAYERS],
        after: &World,
        me: usize,
    ) {
        let frame = before.frame;
        // A world far behind the last one (a fresh fight, a new place): what
        // came before is history, not something to correct.
        if frame.saturating_add(WINDOW) < self.newest {
            self.settle_all();
        }
        self.me = me.min(MAX_PLAYERS - 1);
        self.newest = self.newest.max(frame);
        let mut h = Happened::default();
        if before.valley.on {
            h.picked = after.picked;
            for (i, (fed, input)) in h.fed.iter_mut().zip(inputs).enumerate() {
                *fed = match input.travel.destination() {
                    // An egg eaten is an egg let go of, which the world
                    // decides: not while its creature hunts them for it.
                    Some(Destination::Feed(Item::Egg(s))) => {
                        (before.refusal(i, Item::Egg(s)).is_none()
                            && !after.valley.keep.carries_egg(i, s))
                        .then_some(Item::Egg(s))
                    }
                    Some(Destination::Feed(item)) => Some(item),
                    _ => None,
                };
            }
        }
        let at = self.pending.binary_search_by_key(&frame, |(f, _)| *f);
        let changed = match (at, h.is_empty()) {
            (Ok(k), true) => {
                self.pending.remove(k);
                true
            }
            (Ok(k), false) => {
                let same = self.pending[k].1 == h;
                self.pending[k].1 = h;
                !same
            }
            (Err(k), false) => {
                self.pending.insert(k, (frame, h));
                true
            }
            (Err(_), true) => false,
        };
        let settled = self.settle_older_than(self.newest.saturating_sub(WINDOW));
        if changed || settled {
            self.refresh();
        }
    }

    /// The world was put back to `frame` (`[`): what happened from it on
    /// did not.
    pub fn rewound_to(&mut self, frame: u32) {
        self.pending.retain(|(f, _)| *f < frame);
        self.newest = frame;
        self.refresh();
    }

    /// A fresh world from the top: everything so far is history.
    pub fn restart(&mut self) {
        self.settle_all();
        self.newest = 0;
    }

    fn settle_older_than(&mut self, frame: u32) -> bool {
        let n = self.pending.iter().take_while(|(f, _)| *f < frame).count();
        for (f, h) in self.pending.drain(..n) {
            self.settled.apply(f, &h, self.me);
        }
        n > 0
    }

    fn settle_all(&mut self) {
        self.settle_older_than(u32::MAX);
        self.refresh();
    }

    fn refresh(&mut self) {
        let mut now = self.settled.clone();
        for (f, h) in &self.pending {
            now.apply(*f, h, self.me);
        }
        self.now = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::breed::cross::TrophyKind;
    use sim::class::Class;
    use sim::input::Travel;
    use sim::valley;

    const FEATHER: Item = Item::Trophy(TrophyKind::Feather, SpeciesId::MANTIS);

    fn hearth() -> World {
        World::arrive(
            [Class::Champion],
            valley::Journey::default(),
            valley::START,
            None,
            1,
        )
    }

    fn inputs(travel: Travel) -> [Input; MAX_PLAYERS] {
        let mut i = [Input::default(); MAX_PLAYERS];
        i[0] = Input::new(0).travelling(travel);
        i
    }

    /// Advance `w` on `travel`, kept.
    fn advance(k: &mut Keeping, w: &mut World, travel: Travel) {
        let before = w.clone();
        let i = inputs(travel);
        w.advance([i[0]]);
        k.record(&before, &i, w, 0);
    }

    #[test]
    fn a_trophy_given_is_in_the_bag_and_fed_is_eaten() {
        let mut k = Keeping::default();
        let mut w = hearth();
        advance(&mut k, &mut w, Travel::grant(FEATHER));
        assert_eq!(k.held().bag, vec![FEATHER]);
        let before = k.held().morphs[0];
        advance(&mut k, &mut w, Travel::feed(FEATHER));
        assert!(k.held().bag.is_empty());
        assert_eq!(k.held().morphs[0].generation, before.generation + 1);
        // Long after, settled, it is still so.
        for _ in 0..2 * WINDOW {
            advance(&mut k, &mut w, Travel::NONE);
        }
        assert!(k.pending.is_empty());
        assert_eq!(k.held().morphs[0].generation, before.generation + 1);
    }

    #[test]
    fn a_rollback_that_undoes_a_pickup_undoes_it_in_the_bag() {
        let mut k = Keeping::default();
        let mut w = hearth();
        let start = w.clone();
        // Predicted: given a feather on this frame.
        advance(&mut k, &mut w, Travel::grant(FEATHER));
        assert_eq!(k.held().bag, vec![FEATHER]);
        // Corrected: the same frame again, and nothing given.
        let mut again = start.clone();
        advance(&mut k, &mut again, Travel::NONE);
        assert!(k.held().bag.is_empty(), "{:?}", k.held().bag);
        // And played a third time, it is given once, not twice.
        let mut third = start;
        advance(&mut k, &mut third, Travel::grant(FEATHER));
        assert_eq!(k.held().bag, vec![FEATHER]);
    }

    #[test]
    fn another_seats_feeding_grows_their_morph_not_my_bag() {
        let mut k = Keeping::default();
        let w = hearth();
        let mut i = [Input::default(); MAX_PLAYERS];
        i[1] = Input::new(0).travelling(Travel::feed(FEATHER));
        let mut after = w.clone();
        after.frame += 1;
        k.record(&w, &i, &after, 0);
        assert_eq!(k.held().morphs[1].generation, 1);
        assert_eq!(k.held().morphs[0].generation, 0);
        assert!(k.held().bag.is_empty());
    }
}
