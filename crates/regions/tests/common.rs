//! A hunt driven the way the game's local driver drives one: a wall tick, the
//! gate, the tape, the advance, the books.

use hunt::Hunter;
use regions::Ledger;
use sim::input::Input;
use sim::species::SpeciesId;
use sim::state::MAX_PLAYERS;
use sim::{Class, World};

pub struct Run {
    pub world: World,
    pub ledger: Ledger,
    bots: Vec<Hunter>,
}

impl Run {
    pub fn hunt(classes: [Class; MAX_PLAYERS], seed: u32) -> Run {
        let mut world = World::hunt_of(classes, SpeciesId::RIDGEBACK);
        for beast in world.monsters.iter_mut().flatten() {
            beast.brain.rng = seed | 1;
        }
        let bots = (0..MAX_PLAYERS)
            .map(|who| Hunter::for_species(SpeciesId::RIDGEBACK, who).expect("a plan"))
            .collect();
        let ledger = Ledger::new(&world, true);
        Run {
            world,
            ledger,
            bots,
        }
    }

    /// One wall tick. Returns whether the world advanced.
    pub fn tick(&mut self) -> bool {
        self.ledger.tick();
        if !self.ledger.may_advance(self.world.frame + 1) {
            return false;
        }
        let mut inputs = [Input::default(); MAX_PLAYERS];
        for bot in self.bots.iter_mut() {
            bot.watch(&self.world);
        }
        for (who, bot) in self.bots.iter_mut().enumerate() {
            inputs[who] = bot.act(&self.world);
        }
        self.ledger.record(&self.world, inputs);
        self.world.advance(inputs);
        self.ledger.confirm_through(self.world.frame, &self.world);
        true
    }
}
