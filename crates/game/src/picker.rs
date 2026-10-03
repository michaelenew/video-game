//! The arena picker: which creature, in which arena (`docs/design/world.md`
//! §6 W0).
//!
//! Two ways in. **At start**, `--hunt <creature>` (or `?hunt=<creature>`) and
//! `--arena <name>` (or `?arena=<name>`) choose the first fight; `--hunt` on
//! its own is the Ridgeback, as it always was. A creature's second mode is
//! its name, a dash and the mode: `--hunt hornback-escort` (or
//! `?hunt=hornback-escort`) is the Hornback's crossing. **In game**, `H` swaps between
//! hunting and fighting each other and `Shift+H` steps to the next registered
//! creature, and `T` steps the creature being hunted to its next temper on
//! offer (world W2) -- and those three go through the simulation, as a
//! [`sim::input::Travel`] on the wire, so both peers change arena on the same
//! frame and a rollback that crosses the change replays it. The temper rides
//! in the same byte. The reasoning is in `docs/design/arenas.md`; tempers and
//! trophies are `docs/design/world.md` §6.
//!
//! Pure functions of what was typed, so the tests at the bottom can check
//! both spellings without a window.

use crate::trophies::Trophies;
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
    /// `--temper <n>`: the temper to start the hunt at, regardless of what the
    /// player has earned. A dev's flag; zero, as tuned, without it.
    pub temper: u8,
}

/// Read the picker's two flags. `hunt` is whether `--hunt` was given and
/// `creature` the word after it, if any; `place` the word after `--arena`.
///
/// A creature or arena that is not registered is said so on stderr and
/// ignored -- the Ridgeback for a hunt, the creature's own arena for a place --
/// rather than refusing to start: a link that names a creature from a branch
/// that has not landed yet should still open on something.
pub fn start(
    hunt: bool,
    creature: Option<&str>,
    place: Option<&str>,
    temper: Option<&str>,
) -> Start {
    // A creature's second mode is its name, a dash, and the mode:
    // `hornback-escort` is the Hornback on its crossing.
    if let Some((species, arena)) = creature.and_then(mode) {
        let mut s = start(hunt, Some(species.name), place, temper);
        s.arena = s.arena.or(Some(arena));
        return s;
    }
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
    let temper = temper.map_or(0, |n| match n.parse::<u8>() {
        Ok(n) if n <= sim::temper::HIGHEST => n,
        _ => {
            eprintln!(
                "--temper wants 0 to {}; got {n:?}, using the highest",
                sim::temper::HIGHEST
            );
            sim::temper::HIGHEST
        }
    });
    Start {
        hunt,
        arena,
        temper,
    }
}

/// **A creature's other mode**, spelled `<creature>-<mode>`: the creature,
/// and one of the arenas that names it -- by its own name, or by `escort` for
/// the one with something to defend in it (bestiary P7). `hornback-escort`
/// and `hornback-crossing` are both the Hornback's crossing. `None` for a
/// plain creature's name, or a mode it does not have.
pub fn mode(name: &str) -> Option<(&'static species::Species, ArenaId)> {
    if species::named(name).is_some() {
        return None;
    }
    let (who, how) = name.split_once(['-', '_', ' '])?;
    let sp = species::named(who)?;
    let how = how.to_lowercase();
    arena::all()
        .filter(|a| a.creature == Some(sp.id))
        .find(|a| {
            a.slug() == how.replace(['-', ' '], "_") || (how == "escort" && !a.sites.is_empty())
        })
        .map(|a| (sp, a.id))
}

/// The world a start describes.
pub fn world(start: Start, classes: [Class; 2]) -> World {
    let w = match (start.hunt, start.arena) {
        (Some(s), Some(a)) => World::hunt_in(classes, [Some(s), None], a),
        (Some(s), None) => World::hunt_of(classes, s),
        (None, Some(a)) => World::versus_in(classes, a),
        (None, None) => World::with_classes(classes),
    };
    w.tempered(start.temper)
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
/// own arena. From a versus match, the first one. The dev species (the gnats
/// and the sentinel, `SpeciesId::is_dev`) are not in the cycle; `--hunt`
/// reaches them by name.
pub fn next(w: &World) -> Travel {
    // What is hunted, not `w.monster()`: a pack creature (the Gnawers, the
    // herd) has no monster, and asking for one sent `Shift+H` from the
    // Gnawers back to the Ridgeback, so the cycle never got past them.
    let to = match w.hunted().into_iter().flatten().next() {
        Some(s) => species::after(s).id,
        None => species::shown()
            .next()
            .map_or(SpeciesId::RIDGEBACK, |s| s.id),
    };
    Travel::hunt(to)
}

/// `N`: the next jump course, in order of difficulty -- easy, hard, barely
/// possible -- wrapping, and the first from anywhere that is not a course.
/// The same trip on the wire as `H`, with the arena in the byte
/// (`sim::input::Travel::arena`). See `docs/design/courses.md`.
pub fn next_course(w: &World) -> Travel {
    Travel::arena(sim::course::after(w.arena).arena)
}

/// The course panel, in a course: every course, which one this is, and how
/// the run is going -- the furthest checkpoint, the clock, the falls.
pub fn course_panel(w: &World) -> Option<String> {
    let here = sim::course::of(w.arena)?;
    let run = w.course[0];
    let mut out = String::from("N next course   Backspace restart\n");
    for c in sim::course::all() {
        out.push_str(&format!(
            "{} {:<14} {}{}\n",
            if c.arena == w.arena { ">" } else { " " },
            c.name,
            c.tier.name(),
            c.for_class
                .map_or(String::new(), |k| format!(" ({})", k.name())),
        ));
    }
    let tenths = run.clock(w.frame) * 10 / sim::TICK_HZ;
    out.push_str(&format!(
        "\n{} -- {}\n{}  {}.{}s  falls {}\n",
        here.name,
        here.tier.name(),
        if run.finished() {
            "FINISHED".to_string()
        } else {
            format!("checkpoint {}/{}", run.reached, here.finish())
        },
        tenths / 10,
        tenths % 10,
        run.falls,
    ));
    Some(out)
}

/// `T`: the creature being hunted again, at the next temper on offer --
/// wrapping back to as tuned. `any` is `--temper`, which offers every one.
/// Nothing outside a hunt, where there is nothing to temper.
pub fn temper(w: &World, trophies: &Trophies, any: bool) -> Travel {
    let Some(hunted) = w.hunted().into_iter().flatten().next() else {
        return Travel::NONE;
    };
    Travel::tempered(hunted, trophies.next_temper(hunted, w.temper(), any))
}

/// The roman numeral a temper is called by: as tuned has none.
pub fn numeral(t: u8) -> &'static str {
    ["-", "I", "II", "III"][(t as usize).min(3)]
}

/// The picker's list, as the HUD shows it: every creature a player is shown,
/// its trophies, the tempers on offer, and which one is being hunted now. A
/// dev species is listed only while it is the one being hunted.
pub fn listing(w: &World, trophies: &Trophies, any: bool) -> String {
    let hunted = w.hunted().into_iter().flatten().next();
    if let Some(panel) = course_panel(w) {
        return panel;
    }
    let mut out = String::from("H hunt/versus   Shift+H next   T temper   N courses\n");
    for sp in species::all().filter(|s| !s.id.is_dev() || hunted == Some(s.id)) {
        let here = hunted == Some(sp.id);
        let won: Vec<&str> = (0..sim::temper::TEMPERS)
            .filter(|t| trophies.beaten(sp.id, *t))
            .map(|t| if t == 0 { "0" } else { numeral(t) })
            .collect();
        let offered: Vec<&str> = (0..sim::temper::TEMPERS)
            .filter(|t| any || trophies.offered(sp.id, *t))
            .map(|t| if t == 0 { "0" } else { numeral(t) })
            .collect();
        out.push_str(&format!(
            "{} {:<10} trophies {:<9} tempers {}{}\n",
            if here { ">" } else { " " },
            sp.name,
            if won.is_empty() {
                "-".to_string()
            } else {
                won.join(" ")
            },
            offered.join(" "),
            if here && w.temper() > 0 {
                format!("   now {}", numeral(w.temper()))
            } else {
                String::new()
            },
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Options;

    fn read(o: &Options) -> Start {
        start(
            o.flag("--hunt"),
            o.value("--hunt"),
            o.value("--arena"),
            o.value("--temper"),
        )
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
                temper: 0,
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
    fn a_creatures_escort_mode_is_its_name_a_dash_and_escort() {
        for o in [
            Options::from_args(["--hunt", "hornback-escort"].map(String::from)),
            Options::from_query("?hunt=hornback-escort"),
            Options::from_args(["--hunt", "hornback", "--arena", "crossing"].map(String::from)),
        ] {
            assert_eq!(
                read(&o),
                Start {
                    hunt: Some(SpeciesId::HORNBACK),
                    arena: Some(ArenaId::HORNBACK_CROSSING),
                    temper: 0,
                }
            );
        }
        // A mode it does not have is the creature's name not found.
        let o = Options::from_query("?hunt=ridgeback-escort");
        assert_eq!(read(&o).hunt, Some(SpeciesId::RIDGEBACK));
        assert_eq!(read(&o).arena, None);
    }

    #[test]
    fn an_unregistered_creature_is_the_ridgeback_and_an_unknown_arena_is_ignored() {
        let o = Options::from_query("?hunt=no_such_beast&arena=nowhere");
        assert_eq!(
            read(&o),
            Start {
                hunt: Some(SpeciesId::RIDGEBACK),
                arena: None,
                temper: 0,
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
                temper: 0,
            },
            classes,
        );
        assert_eq!(range.arena, ArenaId::RANGE);
        assert!(!range.hunting());
        let hunt = world(
            Start {
                hunt: Some(SpeciesId::RIDGEBACK),
                arena: None,
                temper: 0,
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
                Some(sim::input::Destination::Hunt(s, _)) => {
                    assert!(species::lookup(s).is_some(), "{s:?} is not registered")
                }
                other => panic!("Shift+H asked for {other:?}"),
            }
        }
    }

    #[test]
    fn the_temper_is_spelled_the_same_two_ways_and_starts_the_hunt_at_it() {
        let args = Options::from_args(["--hunt", "ridgeback", "--temper", "2"].map(String::from));
        let url = Options::from_query("?hunt=ridgeback&temper=2");
        assert_eq!(read(&args), read(&url));
        assert_eq!(read(&args).temper, 2);
        assert_eq!(read(&Options::from_query("?hunt")).temper, 0);
        assert_eq!(read(&Options::from_query("?hunt&temper=9")).temper, 3);
        let w = world(read(&args), [Class::Bulwark; 2]);
        assert_eq!(w.temper(), 2);
        // A temper asked of a versus match is nothing at all.
        let v = world(read(&Options::from_query("?temper=3")), [Class::Bulwark; 2]);
        assert_eq!(v, World::with_classes([Class::Bulwark; 2]));
    }

    #[test]
    fn t_offers_only_what_has_been_earned_unless_the_flag_says_otherwise() {
        let hunt = World::hunt([Class::Bulwark; 2]);
        let mut trophies = Trophies::default();
        let at = |t: Travel| match t.destination() {
            Some(sim::input::Destination::Hunt(s, n)) => (s, n),
            other => panic!("T asked for {other:?}"),
        };
        assert_eq!(
            at(temper(&hunt, &trophies, false)),
            (SpeciesId::RIDGEBACK, 0)
        );
        assert_eq!(
            at(temper(&hunt, &trophies, true)),
            (SpeciesId::RIDGEBACK, 1)
        );
        trophies.record(SpeciesId::RIDGEBACK, 0);
        assert_eq!(
            at(temper(&hunt, &trophies, false)),
            (SpeciesId::RIDGEBACK, 1)
        );
        let versus = World::with_classes([Class::Bulwark; 2]);
        assert_eq!(temper(&versus, &trophies, true), Travel::NONE);
        let list = listing(&hunt.tempered(1), &trophies, false);
        assert!(list.contains("> Ridgeback"), "{list}");
        assert!(list.contains("now I"), "{list}");
    }

    #[test]
    fn the_list_and_the_cycle_hold_every_creature_and_no_dev_species() {
        let trophies = Trophies::default();
        let versus = World::with_classes([Class::Bulwark; 2]);
        let list = listing(&versus, &trophies, false);
        for sp in species::all() {
            assert_eq!(
                list.contains(sp.name),
                !sp.id.is_dev(),
                "{} in the list:\n{list}",
                sp.name
            );
        }
        // Shift+H, from versus and round the cycle, visits every one.
        let mut w = versus;
        let mut seen = Vec::new();
        for _ in 0..species::COUNT {
            let Some(sim::input::Destination::Hunt(s, _)) = next(&w).destination() else {
                panic!("Shift+H went nowhere");
            };
            assert!(!s.is_dev(), "Shift+H offered {s:?}");
            seen.push(s);
            w = World::hunt_of([Class::Bulwark; 2], s);
        }
        for sp in species::shown() {
            assert!(seen.contains(&sp.id), "Shift+H never reached {}", sp.name);
        }
        // A dev species reached by name is listed while it is hunted.
        let gnats = World::hunt_of([Class::Bulwark; 2], SpeciesId::GNATS);
        assert!(listing(&gnats, &trophies, false).contains("> Gnats"));
        assert_ne!(
            next(&gnats).destination(),
            Some(sim::input::Destination::Hunt(SpeciesId::GNATS, 0))
        );
    }

    #[test]
    fn n_steps_through_the_courses_and_the_panel_follows() {
        let versus = World::with_classes([Class::Champion; 2]);
        let mut w = versus;
        let mut seen = Vec::new();
        for _ in 0..sim::course::all().count() {
            let Some(sim::input::Destination::Arena(a)) = next_course(&w).destination() else {
                panic!("N went nowhere");
            };
            seen.push(a);
            w = World::versus_in([Class::Champion; 2], a);
            let panel = course_panel(&w).expect("a course without a panel");
            let here = sim::course::of(a).expect("N went somewhere that is not a course");
            assert!(panel.contains(&format!("> {}", here.name)), "{panel}");
            assert!(panel.contains("checkpoint 0/"), "{panel}");
        }
        for c in sim::course::all() {
            assert!(seen.contains(&c.arena), "N never reached {}", c.name);
        }
        // Outside a course the creature list is shown, and offers the key.
        let list = listing(
            &World::with_classes([Class::Champion; 2]),
            &Trophies::default(),
            false,
        );
        assert!(list.contains("N courses"), "{list}");
        // And the names the flags take start one.
        let s = start(false, None, Some("climb"), None);
        assert_eq!(s.arena, Some(sim::arena::ArenaId::CLIMB_CLIMB));
    }
}
