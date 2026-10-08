//! A scripted hunter, and the report that says whether the fight it plays is
//! any good.
//!
//! Its own crate rather than a module of `sim`, because it is not simulation:
//! it is a **player**, standing in for a person so that a fight can be run a
//! thousand times and measured. The numbers in here are the measuring
//! instrument, not feel values, which is exactly why they should not be in the
//! Oven -- and keeping them out of `crates/sim/src` is what stops the knob
//! guard from having to be argued with.
//!
//! Two rules make the measurement worth anything.
//!
//! **The hunter reacts late.** It sees the world as it was `REACTION` frames
//! ago and never sooner. A bot that dodges on the first frame of a startup
//! proves nothing about whether that move was readable; one that cannot see the
//! present at all proves quite a lot.
//!
//! **It does not read the creature's mind.** It gets what a player gets from
//! the screen -- where the animal is, which way it is pointed, and what it has
//! visibly begun -- and nothing about which move the control algorithm is about
//! to pick.
//!
//! ## One plan per creature
//!
//! A person learns each fight separately, and so does this. What the hunter
//! does against a creature is that creature's **plan**, in its own file under
//! [`plans`] -- the Ridgeback's is `plans/ridgeback.rs` -- registered on a
//! [`plans::Card`] beside the words its report uses. Everything here is what
//! every plan shares: the reaction delay, the keyboard, the jump, and the loop
//! that plays a hunt and hands back a [`Report`].

pub mod class;
pub mod duel;
pub mod plans;
pub mod replay;
pub mod report;

pub use class::{Hands, Uses};
pub use duel::{Duelist, Level};
pub use report::{Outcome, Report};

use sim::fixed::Fx;
use sim::species::SpeciesId;
use sim::state::{MAX_PLAYERS, Phase};
use sim::{Input, V3, World};

/// Frames of delay between the world changing and the hunter knowing.
///
/// Human reaction at 60 Hz is roughly this. It is the reason the report's
/// "reactable share" means something: below this a move genuinely cannot be
/// answered on sight, and the hunter is held to the same limit a person is.
pub const REACTION: usize = 15;

/// What a hunter is trying to do, by name. Committing to an intent for a
/// while is what stops it dithering, and it is also what a person does.
///
/// A name rather than a shared enum because each creature's plan has its own
/// -- the Ridgeback's hunter climbs and works the ridge, and nothing else
/// does -- and a list every plan appended to would be one file every creature
/// branch edited.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Intent(pub &'static str);

impl std::fmt::Debug for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/// One creature's hunter: what a decent player does against it. See
/// [`plans`].
pub trait Plan {
    /// Record this frame. Called before [`Plan::act`], every tick, so the
    /// delay line stays honest.
    fn watch(&mut self, w: &World);
    /// One frame of decision.
    fn act(&mut self, w: &World) -> Input;
    /// What it is trying to do this frame, for the report's play sequence.
    fn intent(&self) -> Intent;
    /// **Its class, in its hands** (`crate::class`), for a plan that has
    /// wired one in: [`Hunter::act`] gives every frame's input to
    /// [`Hands::finish`] after the plan, and the report counts what it did.
    fn hands(&mut self) -> Option<&mut Hands> {
        None
    }
    /// The same, to read.
    fn hands_ref(&self) -> Option<&Hands> {
        None
    }
}

/// The plan of a person: none the report can read. See [`Hunter::person`].
struct Person;

impl Plan for Person {
    fn watch(&mut self, _: &World) {}
    fn act(&mut self, _: &World) -> Input {
        Input::default()
    }
    fn intent(&self) -> Intent {
        Intent("a person")
    }
}

/// One scripted fighter, playing its creature's plan.
pub struct Hunter {
    pub who: usize,
    /// The creature its plan is for.
    pub species: SpeciesId,
    plan: Box<dyn Plan + Send + Sync>,
    /// What it pressed last, and so where it was looking: the camera the
    /// report asks what was on screen (`sim::aim::in_view`).
    pub last: Input,
}

impl Hunter {
    /// A hunter for the Ridgeback, with its default timing and no idea how
    /// high it jumps. What the running game's scripted partner starts as.
    pub fn new(who: usize) -> Hunter {
        Hunter {
            who,
            species: SpeciesId::RIDGEBACK,
            plan: Box::new(plans::ridgeback::Ridgeback::new(who)),
            last: Input::default(),
        }
    }

    /// A hunter for whatever species, if it has a plan: default timing, and
    /// no idea how high it jumps -- what the running game swaps in when the
    /// creature in front of it changes.
    pub fn for_species(species: SpeciesId, who: usize) -> Option<Hunter> {
        plans::card(species).map(|card| Hunter::of(card, who, 0, Fx::ZERO))
    }

    /// A hunter playing a species' plan, seeded and knowing its own hop.
    pub fn of(card: &plans::Card, who: usize, seed: u32, hop: Fx) -> Hunter {
        Hunter {
            who,
            species: card.species,
            plan: (card.plan)(who, seed, hop),
            last: Input::default(),
        }
    }

    /// **A person**, for the report: a hunter whose every frame is read off
    /// a replay rather than decided (`crate::replay`). The report asks a
    /// hunter three things -- which seat, what it last pressed (its camera),
    /// and what it meant -- and a person answers the first two exactly and
    /// the third not at all.
    pub fn person(who: usize, species: SpeciesId) -> Hunter {
        Hunter {
            who,
            species,
            plan: Box::new(Person),
            last: Input::default(),
        }
    }

    /// One frame from a replay, in place of [`Hunter::act`]: the input is
    /// what the person pressed, remembered as the camera the report reads.
    pub fn playback(&mut self, input: Input) -> Input {
        self.last = input;
        input
    }

    pub fn watch(&mut self, w: &World) {
        self.plan.watch(w);
    }

    pub fn act(&mut self, w: &World) -> Input {
        let raw = self.plan.act(w);
        let me = w.players[self.who];
        self.last = match self.plan.hands() {
            Some(hands) => hands.finish(w, &me, raw),
            None => raw,
        };
        self.last
    }

    /// What it did with its class, if its plan plays one (`crate::class`).
    pub fn uses(&self) -> Option<Uses> {
        self.plan.hands_ref().map(|h| h.uses)
    }

    pub fn intent(&self) -> Intent {
        self.plan.intent()
    }
}

/// A quarter turn, in `Fx`.
pub const QUARTER: Fx = Fx::from_raw(1 << 14);
/// Half of one, for splitting a range span.
pub const HALF: Fx = Fx::from_raw(1 << 15);
/// Stick deadzone when turning a direction into four keys.
const DEAD: Fx = Fx::ratio(3, 10);

/// A look angle, as the wire's sixteen bits.
pub fn turns_to_aim(turns: Fx) -> u16 {
    (turns.raw() as u32 & 0xFFFF) as u16
}

/// Four keys from a direction, given where the fighter is looking.
///
/// The inverse of `state::move_dir`, and deliberately lossy in the same way a
/// keyboard is: eight directions, not three hundred and sixty.
pub fn steer(aim: Fx, want: V3) -> u16 {
    let want = V3::new(want.x, Fx::ZERO, want.z).normalized();
    if want.flat_len().raw() == 0 {
        return 0;
    }
    let forward = V3::from_turns(aim);
    let right = V3::from_turns(aim.add(QUARTER));
    let f = want.dot(forward);
    let r = want.dot(right);
    let mut bits = 0;
    if f.raw() > DEAD.raw() {
        bits |= Input::W;
    } else if f.raw() < DEAD.neg().raw() {
        bits |= Input::S;
    }
    if r.raw() > DEAD.raw() {
        bits |= Input::D;
    } else if r.raw() < DEAD.neg().raw() {
        bits |= Input::A;
    }
    bits
}

/// Which button throws the class's committed attack.
///
/// Everywhere but the Champion it is shift plus left click, per `controls.md`.
/// The Champion's three mouse buttons are three weapons instead, and the
/// committed one is the hammer on middle click -- so this hunter punishes an
/// opening with a hammer, and never with the class's *biggest* hits, which all
/// live behind a Rush charge it deliberately does not spend. It is here to
/// measure the creature, not to show off a kit.
///
/// The Bulwark's is on middle click too, since 2026-09-23: Slam, which spends
/// the shield's weight, so a hunter that has blocked a stomp punishes with
/// what the stomp put into it. The other classes' committed moves have had no
/// input since shift stopped modifying clicks, and for them this is the poke.
pub fn heavy(class: sim::Class) -> u16 {
    match class {
        sim::Class::Champion | sim::Class::Bulwark => Input::MIDDLE,
        // The Haemorrhage, on right click: a bolt that cuts and gores, and
        // the one right-click ability in the roster since shift stopped
        // modifying clicks. The creature does not bleed, so on a hunt it is
        // the ranged punish and nothing more.
        sim::Class::BloodMage => Input::RIGHT,
        _ => Input::LEFT | Input::SHIFT,
    }
}

/// How long the committed attack keeps the hunter busy: the frames until it is
/// free to move again. What an opening has to be longer than to be worth it.
pub fn heavy_commitment(class: sim::Class) -> usize {
    let m = sim::moves::get(class, sim::state::SLOT_COMMITTED);
    (m.startup + m.active + m.recovery) as usize
}

/// Apex of a full hop above the feet, for one class.
///
/// Measured by running the simulation, the way `beastcheck` does, because the
/// sustain window makes the closed form wrong -- and the number is what
/// decides which of the creature's surfaces the hunter treats as a route.
pub fn jump_apex(class: sim::Class) -> Fx {
    let mut w = World::with_classes([class; MAX_PLAYERS]);
    for _ in 0..40 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let floor = w.players[0].pos.y;
    let mut best = floor;
    let held = Input::default().with(Input::SPACE);
    for _ in 0..240 {
        w.advance([held, Input::default()]);
        best = best.max(w.players[0].pos.y);
    }
    best.sub(floor)
}

/// Run a whole hunt and hand back the report.
///
/// One hunter and one idle second fighter by default: the numbers are set for a
/// single player, and a monster tuned for two is a different monster.
///
/// `seed` starts the creature's generator somewhere else. A hunt is completely
/// deterministic, so without it every repeat is the same fight -- which tells
/// you nothing about the spread, and the spread is most of what you want to
/// know about a creature that chooses.
pub fn play(classes: [sim::Class; MAX_PLAYERS], partners: usize, limit: u32, seed: u32) -> Report {
    play_watched(classes, partners, limit, seed, |_| {})
}

/// [`play`], against a creature of any species that has a plan.
pub fn play_species(
    species: SpeciesId,
    classes: [sim::Class; MAX_PLAYERS],
    partners: usize,
    limit: u32,
    seed: u32,
) -> Report {
    play_species_watched(species, classes, partners, limit, seed, |_| {})
}

/// [`play`], showing every frame of the world to `watch` as it goes. What the
/// bit-identity pin hashes (`tests/pin.rs`).
pub fn play_watched(
    classes: [sim::Class; MAX_PLAYERS],
    partners: usize,
    limit: u32,
    seed: u32,
    watch: impl FnMut(&World),
) -> Report {
    play_species_watched(SpeciesId::RIDGEBACK, classes, partners, limit, seed, watch)
}

/// The whole loop, against any species with a plan. See [`play`].
pub fn play_species_watched(
    species: SpeciesId,
    classes: [sim::Class; MAX_PLAYERS],
    partners: usize,
    limit: u32,
    seed: u32,
    watch: impl FnMut(&World),
) -> Report {
    play_tempered(species, 0, classes, partners, limit, seed, watch)
}

/// The whole loop, at a temper (`sim::temper`): what `fight --temper` runs, so
/// a temper's difficulty is a number the harness reports rather than a guess.
pub fn play_tempered(
    species: SpeciesId,
    temper: u8,
    classes: [sim::Class; MAX_PLAYERS],
    partners: usize,
    limit: u32,
    seed: u32,
    watch: impl FnMut(&World),
) -> Report {
    let card = plans::card(species).expect("no hunter plan is registered for that species");
    play_card(card, temper, classes, partners, limit, seed, watch)
}

/// [`play_tempered`] in an arena other than the creature's own: what
/// `fight --arena` runs -- the Hornback's crossing, mostly.
#[allow(clippy::too_many_arguments)]
pub fn play_in(
    species: SpeciesId,
    arena: Option<sim::arena::ArenaId>,
    temper: u8,
    classes: [sim::Class; MAX_PLAYERS],
    partners: usize,
    limit: u32,
    seed: u32,
    watch: impl FnMut(&World),
) -> Report {
    let card = plans::card(species).expect("no hunter plan is registered for that species");
    play_card_in(card, arena, temper, classes, partners, limit, seed, watch)
}

/// The whole loop, with a card in hand rather than a species: what
/// `fight --gamble` runs, with the card's second plan in its first's place.
pub fn play_card(
    card: &'static plans::Card,
    temper: u8,
    classes: [sim::Class; MAX_PLAYERS],
    partners: usize,
    limit: u32,
    seed: u32,
    watch: impl FnMut(&World),
) -> Report {
    play_card_in(card, None, temper, classes, partners, limit, seed, watch)
}

/// [`play_card`] in a chosen arena, or the creature's own for `None`.
#[allow(clippy::too_many_arguments)]
pub fn play_card_in(
    card: &'static plans::Card,
    arena: Option<sim::arena::ArenaId>,
    temper: u8,
    classes: [sim::Class; MAX_PLAYERS],
    partners: usize,
    limit: u32,
    seed: u32,
    mut watch: impl FnMut(&World),
) -> Report {
    let species = card.species;
    let w = match arena {
        Some(place) => World::hunt_in(classes, [Some(species), None], place),
        None => World::hunt_of(classes, species),
    };
    let mut w = w.tempered(temper);
    for beast in w.monsters.iter_mut().flatten() {
        beast.brain.rng = seed | 1;
    }
    if let Some(pack) = w.pack.as_mut() {
        pack.rng = seed | 1;
    }
    let count = partners.clamp(1, MAX_PLAYERS);
    let mut bots: Vec<Hunter> = (0..count)
        .map(|who| Hunter::of(card, who, seed, jump_apex(classes[who])))
        .collect();
    // A fighter nobody is driving is not a fighter standing very still: it is a
    // target the creature will happily charge across the arena at for twenty
    // minutes, which is what the first long run of this harness actually
    // measured. Absent hunters are out of the hunt.
    for p in w.players.iter_mut().skip(count) {
        p.health = 0;
    }
    let mut report = Report::new(card);
    while w.frame < limit && matches!(w.phase, Phase::Fighting) {
        let mut inputs = [Input::default(); MAX_PLAYERS];
        for bot in bots.iter_mut() {
            bot.watch(&w);
        }
        for bot in bots.iter_mut() {
            inputs[bot.who] = bot.act(&w);
        }
        let before = w.clone();
        w.advance(inputs);
        watch(&w);
        report.observe(&before, &w, &bots);
    }
    report.finish(&w);
    report
}
