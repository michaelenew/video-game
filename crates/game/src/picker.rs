//! The arena picker: which creature, in which arena (`docs/design/world.md`
//! §6 W0).
//!
//! Two ways in. **At start**, `--hunt <creature>` (or `?hunt=<creature>`) and
//! `--arena <name>` (or `?arena=<name>`) choose the first fight; `--hunt` on
//! its own is the Ridgeback, as it always was. **In game**, `H` swaps between
//! hunting and fighting each other and `Shift+H` steps to the next registered
//! creature -- and those two go through the simulation, as a
//! [`sim::input::Travel`] on the wire, so both peers change arena on the same
//! frame and a rollback that crosses the change replays it. The reasoning is in
//! `docs/design/arenas.md`.
//!
//! Pure functions of what was typed, so the tests at the bottom can check
//! both spellings without a window.

use sim::arena::{self, ArenaId};
use sim::input::Travel;
use sim::species::{self, SpeciesId};
use sim::{Class, World};

/// What a run was asked to start in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Start {
    /// The creature to hunt, or `None` for fighting each other.
    pub hunt: Option<SpeciesId>,
    /// An arena other than the creature's own (or than the proving ground, for
    /// versus). The range, mostly.
    pub arena: Option<ArenaId>,
}

/// Read the picker's two flags. `hunt` is whether `--hunt` was given and
/// `creature` the word after it, if any; `place` the word after `--arena`.
///
/// A creature or arena that is not registered is said so on stderr and
/// ignored -- the Ridgeback for a hunt, the creature's own arena for a place --
/// rather than refusing to start: a link that names a creature from a branch
/// that has not landed yet should still open on something.
pub fn start(hunt: bool, creature: Option<&str>, place: Option<&str>) -> Start {
    let hunt = hunt.then(|| match creature {
        None => SpeciesId::RIDGEBACK,
        Some(name) => species::named(name).map_or_else(
            || {
                eprintln!("no creature called {name:?} yet; hunting the Ridgeback");
                SpeciesId::RIDGEBACK
            },
            |s| s.id,
        ),
    });
    let arena = place.and_then(|name| {
        let found = arena::named(name).map(|a| a.id);
        if found.is_none() {
            eprintln!("no arena called {name:?}; using the fight's own");
        }
        found
    });
    Start { hunt, arena }
}

/// The world a start describes.
pub fn world(start: Start, classes: [Class; 2]) -> World {
    match (start.hunt, start.arena) {
        (Some(s), Some(a)) => World::hunt_in(classes, [Some(s), None], a),
        (Some(s), None) => World::hunt_of(classes, s),
        (None, Some(a)) => World::versus_in(classes, a),
        (None, None) => World::with_classes(classes),
    }
}

/// `H`: hunt the Ridgeback, or go back to fighting each other.
pub fn toggle(w: &World) -> Travel {
    if w.hunting() {
        Travel::VERSUS
    } else {
        Travel::hunt(SpeciesId::RIDGEBACK)
    }
}

/// `Shift+H`: the next registered creature after the one being hunted, in its
/// own arena. From a versus match, the first one.
pub fn next(w: &World) -> Travel {
    let to = match w.monster() {
        Some(m) => species::after(m.species).id,
        None => species::all().next().map_or(SpeciesId::RIDGEBACK, |s| s.id),
    };
    Travel::hunt(to)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Options;

    fn read(o: &Options) -> Start {
        start(o.flag("--hunt"), o.value("--hunt"), o.value("--arena"))
    }

    #[test]
    fn hunt_with_no_creature_is_the_ridgeback() {
        let args = Options::from_args(["--hunt", "--dev"].map(String::from));
        assert_eq!(read(&args).hunt, Some(SpeciesId::RIDGEBACK));
        assert_eq!(
            read(&Options::from_query("?hunt")).hunt,
            Some(SpeciesId::RIDGEBACK)
        );
        assert_eq!(read(&Options::from_query("?dev")).hunt, None);
    }

    #[test]
    fn the_creature_and_the_arena_are_spelled_the_same_two_ways() {
        let args = Options::from_args(
            ["--hunt", "ridgeback", "--arena", "range", "--p1", "reaver"].map(String::from),
        );
        let url = Options::from_query("?hunt=Ridgeback&arena=range&p1=reaver");
        assert_eq!(read(&args), read(&url));
        assert_eq!(
            read(&args),
            Start {
                hunt: Some(SpeciesId::RIDGEBACK),
                arena: Some(ArenaId::RANGE),
            }
        );
        // The proving ground answers to its name with a space, a dash or an
        // underscore.
        for spelled in ["proving_ground", "proving-ground", "Proving Ground"] {
            let o = Options::from_args(["--arena", spelled].map(String::from));
            assert_eq!(read(&o).arena, Some(ArenaId::PROVING_GROUND), "{spelled}");
        }
    }

    #[test]
    fn an_unregistered_creature_is_the_ridgeback_and_an_unknown_arena_is_ignored() {
        let o = Options::from_query("?hunt=no_such_beast&arena=nowhere");
        assert_eq!(
            read(&o),
            Start {
                hunt: Some(SpeciesId::RIDGEBACK),
                arena: None,
            }
        );
    }

    #[test]
    fn a_start_builds_the_world_it_names() {
        let classes = [Class::Bulwark; 2];
        let range = world(
            Start {
                hunt: None,
                arena: Some(ArenaId::RANGE),
            },
            classes,
        );
        assert_eq!(range.arena, ArenaId::RANGE);
        assert!(!range.hunting());
        let hunt = world(
            Start {
                hunt: Some(SpeciesId::RIDGEBACK),
                arena: None,
            },
            classes,
        );
        assert_eq!(hunt.arena, ArenaId::PROVING_GROUND);
        assert!(hunt.hunting());
        assert_eq!(
            world(Start::default(), classes).arena,
            ArenaId::PROVING_GROUND
        );
    }

    #[test]
    fn the_keys_ask_for_registered_creatures_only() {
        let versus = World::with_classes([Class::Bulwark; 2]);
        assert_eq!(toggle(&versus), Travel::hunt(SpeciesId::RIDGEBACK));
        let hunt = World::hunt([Class::Bulwark; 2]);
        assert_eq!(toggle(&hunt), Travel::VERSUS);
        for w in [&versus, &hunt] {
            match next(w).destination() {
                Some(sim::input::Destination::Hunt(s)) => {
                    assert!(species::lookup(s).is_some(), "{s:?} is not registered")
                }
                other => panic!("Shift+H asked for {other:?}"),
            }
        }
    }
}
