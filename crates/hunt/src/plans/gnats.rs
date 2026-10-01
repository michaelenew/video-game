//! A hunter for the dev pack (`sim::species::gnats`): the simplest player a
//! pack can be measured against, so the report's pack lines have something to
//! count before any real pack creature is built.
//!
//! What it does is what a person does in their first minute against a crowd:
//! go for the nearest body, and **swing first at one that crouches** -- the
//! windup is the opening, and the hunter sees it `REACTION` frames late like
//! everything else. It aims the way a player does, with the crosshair on the
//! body (`sim::aim::look_onto_closely`), so it exercises the aim change the
//! small bodies forced (bestiary A1) rather than going round it.
//!
//! A real pack creature's plan replaces this with what its document says a
//! decent player learns -- back to a wall, kill one and take the window, go
//! for the leader. See `docs/design/critters.md`.

use sim::critter::{Critter, MAX_CRITTERS, is};
use sim::fixed::Fx;
use sim::math::atan2_turns;
use sim::state::Phase;
use sim::{Input, V3, World};

use crate::{Intent, Plan, REACTION, steer, turns_to_aim};

const CLOSE: Intent = Intent("close");
const SWING: Intent = Intent("swing");
const WAIT: Intent = Intent("wait");

/// A body as the hunter remembers seeing it.
#[derive(Clone, Copy, Default)]
struct Seen {
    at: V3,
    middle: V3,
    alive: bool,
    crouching: bool,
}

pub struct Gnats {
    who: usize,
    memory: Vec<[Seen; MAX_CRITTERS]>,
    at: usize,
    filled: usize,
    intent: Intent,
    /// Frames until it may press attack again: a person does not mash.
    cooldown: u32,
    seed: u32,
}

impl Gnats {
    pub fn new(who: usize, seed: u32) -> Gnats {
        Gnats {
            who,
            memory: vec![[Seen::default(); MAX_CRITTERS]; REACTION + 1],
            at: 0,
            filled: 0,
            intent: WAIT,
            cooldown: 0,
            seed: seed | 1,
        }
    }

    fn recall(&self) -> [Seen; MAX_CRITTERS] {
        let back = REACTION.min(self.filled.saturating_sub(1));
        let idx = (self.at + self.memory.len() - 1 - back) % self.memory.len();
        self.memory[idx]
    }
}

impl Plan for Gnats {
    fn watch(&mut self, w: &World) {
        let sp = w.critters.sp();
        let seen = std::array::from_fn(|i| {
            let c: &Critter = &w.critters[i];
            Seen {
                at: c.pos,
                middle: c.body(sp).middle(),
                alive: c.alive(),
                crouching: c.state == is::STARTUP,
            }
        });
        self.memory[self.at] = seen;
        self.at = (self.at + 1) % self.memory.len();
        self.filled = (self.filled + 1).min(self.memory.len());
    }

    fn act(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        let me = w.players[self.who];
        if me.health <= 0 || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        let seen = self.recall();
        // A crouching body near enough to reach first; otherwise the nearest.
        let near = |s: &Seen| s.at.sub(me.pos).flat_len();
        let pick = seen
            .iter()
            .filter(|s| s.alive && s.crouching && near(s).raw() < Fx::from_int(4).raw())
            .min_by_key(|s| near(s).raw())
            .or_else(|| {
                seen.iter()
                    .filter(|s| s.alive)
                    .min_by_key(|s| near(s).raw())
            });
        let Some(target) = pick.copied() else {
            self.intent = WAIT;
            return Input::default();
        };
        let to = target.at.sub(me.pos);
        let yaw = atan2_turns(to.z, to.x);
        let aim = turns_to_aim(yaw.sub(me.carry_yaw));
        let pitch = sim::aim::look_onto_closely(me.pos, aim, me.aloft, target.middle);
        let reach = sim::moves::get(me.class, 0).reach.max(Fx::from_int(1));
        let mut bits = 0;
        if near(&target).raw() > reach.add(Fx::ratio(1, 2)).raw() {
            self.intent = CLOSE;
            bits |= steer(yaw, to);
        } else if self.cooldown == 0 {
            self.intent = SWING;
            bits |= Input::LEFT;
            // A little irregular, the way a hand is.
            self.seed ^= self.seed << 13;
            self.seed ^= self.seed >> 17;
            self.seed ^= self.seed << 5;
            self.cooldown = 10 + self.seed % 8;
        }
        Input::looking_at(bits, aim, pitch)
    }

    fn intent(&self) -> Intent {
        self.intent
    }
}

pub static CARD: crate::plans::Card = crate::plans::Card {
    species: sim::species::SpeciesId::GNATS,
    plan: |who, seed, _hop| Box::new(Gnats::new(who, seed)),
    bucks: |_| false,
    words: crate::plans::Words {
        weak_hits: "weak-point hits",
        broken: "parts broken",
        into_breakables: "damage into parts",
        into_breakables_why: "nothing to break on a gnat",
        worst: "worst part",
        ride_for: "nothing to ride",
        toppled_pool: "off a pool under a fallen body",
    },
    tally: None,
};
