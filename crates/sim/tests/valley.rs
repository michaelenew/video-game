//! **The valley** (`docs/design/valley.md`, `sim::valley`): the places join
//! up, a seam takes both of you on, waystones gate the climb, a death in a
//! reach is a cairn, and the props do what they say.

use sim::arena::{ArenaId, Terrain};
use sim::input::{Destination, Travel};
use sim::species::SpeciesId;
use sim::state::Phase;
use sim::valley::{self, Kind};
use sim::{Class, Fx, Input, V3, World};

const CLASSES: [Class; 2] = [Class::Champion, Class::Elementalist];

fn idle() -> [Input; 2] {
    [Input::new(0), Input::new(0)]
}

/// The world as a seam into `to` at `at` builds it, both seats played.
fn arrived(to: ArenaId, at: Option<u8>) -> World {
    World::arrive(CLASSES, valley::Journey::default(), to, at, 2)
}

/// Where a body would stand in a zone: its middle, on the highest top under
/// the zone's ceiling.
fn standing_in(zone: &valley::Zone, arena: ArenaId) -> V3 {
    let mid = zone.middle();
    let ground = Terrain::bare(arena.get()).floor_below(V3::new(mid.x, zone.max.y, mid.z));
    V3::new(mid.x, ground, mid.z)
}

fn inside_a_solid(arena: ArenaId, p: V3) -> bool {
    let up = V3::new(p.x, p.y.add(Fx::ratio(1, 10)), p.z);
    arena.get().solids().iter().any(|s| {
        up.x.raw() > s.min.x.raw()
            && up.x.raw() < s.max.x.raw()
            && up.y.raw() > s.min.y.raw()
            && up.y.raw() < s.max.y.raw()
            && up.z.raw() > s.min.z.raw()
            && up.z.raw() < s.max.z.raw()
    })
}

#[test]
fn every_seam_leads_to_one_that_leads_back() {
    let mut seams = 0;
    for place in valley::all() {
        for (i, seam) in place.seams.iter().enumerate() {
            seams += 1;
            let there = valley::place(seam.to)
                .unwrap_or_else(|| panic!("{}: seam {i} leads nowhere", place.arena.get().name));
            let back = there.seam(seam.at).unwrap_or_else(|| {
                panic!(
                    "{}: seam {i} comes out of a seam {} has not got",
                    place.arena.get().name,
                    there.arena.get().name
                )
            });
            assert_eq!(
                back.to,
                place.arena,
                "{}: seam {i} and its pair disagree",
                place.arena.get().name
            );
            assert_eq!(
                back.at as usize,
                i,
                "{}: seam {i}'s pair comes back elsewhere",
                place.arena.get().name
            );
        }
    }
    assert!(seams >= 30, "only {seams} seams");
}

#[test]
fn every_seam_can_be_stood_in() {
    for place in valley::all() {
        let id = place.arena;
        for (i, seam) in place.seams.iter().enumerate() {
            let feet = standing_in(&seam.zone, id);
            assert!(
                seam.zone.holds(feet),
                "{}: nowhere to stand in seam {i} ({feet:?})",
                id.get().name
            );
            assert!(
                !inside_a_solid(id, feet),
                "{}: seam {i}'s middle is inside a solid",
                id.get().name
            );
        }
    }
}

#[test]
fn every_arrival_stands_on_footing_out_of_the_rock() {
    for place in valley::all() {
        let id = place.arena;
        let a = id.get();
        for (i, seam) in place.seams.iter().enumerate() {
            for m in seam.marks {
                let feet = V3::new(m.at.x, a.ground_under(m.at), m.at.z);
                let b = a.bounds;
                assert!(
                    feet.x.raw() > b.lo_x.raw()
                        && feet.x.raw() < b.hi_x.raw()
                        && feet.z.raw() > b.lo_z.raw()
                        && feet.z.raw() < b.hi_z.raw(),
                    "{}: seam {i}'s arrival is outside the bounds",
                    a.name
                );
                assert!(
                    !inside_a_solid(id, feet),
                    "{}: seam {i}'s arrival is inside a solid",
                    a.name
                );
                // Off a room's own way out, everywhere else, so arriving never
                // reads as standing in another seam.
                if !matches!(place.kind, Kind::Room(_)) {
                    for (j, other) in place.seams.iter().enumerate() {
                        assert!(
                            !other.zone.holds(feet),
                            "{}: seam {i}'s arrival stands in seam {j}",
                            a.name
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn every_waystone_is_a_solid_by_its_seam() {
    for place in valley::all() {
        for (i, seam) in place.seams.iter().enumerate() {
            let Some(w) = seam.waystone else { continue };
            let s = place.arena.get().solids()[w as usize];
            assert_eq!(
                s.material,
                sim::arena::Material::Stone,
                "{}: seam {i}'s waystone",
                place.arena.get().name
            );
            let mid = seam.zone.middle();
            let dx = (s.min.x.raw() / 2 + s.max.x.raw() / 2 - mid.x.raw()).abs();
            let dz = (s.min.z.raw() / 2 + s.max.z.raw() / 2 - mid.z.raw()).abs();
            assert!(
                dx < Fx::from_int(12).raw() && dz < Fx::from_int(12).raw(),
                "{}: seam {i}'s waystone is far from it",
                place.arena.get().name
            );
            assert!(
                !matches!(seam.gate, valley::Gate::Open),
                "{}: a waystone on an open seam",
                place.arena.get().name
            );
        }
    }
}

#[test]
fn the_valley_starts_in_hearth_at_peace_and_the_ring_is_versus() {
    let w = World::versus_in(CLASSES, valley::START);
    assert!(w.valley.on && w.peaceful() && !w.pvp());
    let ring = arrived(ArenaId::RING, Some(0));
    assert!(ring.valley.on && ring.pvp() && !ring.peaceful());
    // A reach on its own is the valley; a room on its own is not.
    assert!(World::versus_in(CLASSES, ArenaId::MOUTH).valley.on);
    assert!(!World::hunt_of(CLASSES, SpeciesId::HORNBACK).valley.on);
    // Outside the valley nothing changed: versus is versus.
    assert!(World::with_classes(CLASSES).pvp());
}

/// Both fighters at `at`, after a frame outside every seam so they count.
fn walk_both_into(w: &mut World, seam: u8) {
    w.advance(idle());
    let zone = valley::place(w.arena).unwrap().seams[seam as usize].zone;
    let feet = standing_in(&zone, w.arena);
    for p in w.players.iter_mut() {
        p.pos = feet;
        p.vel = V3::ZERO;
    }
}

#[test]
fn both_in_a_seam_go_through_together() {
    let mut w = World::versus_in(CLASSES, valley::START);
    walk_both_into(&mut w, 0);
    w.advance(idle());
    assert_eq!(w.arena, ArenaId::MOUTH);
    assert!(w.valley.on);
    let marks = valley::place(ArenaId::MOUTH).unwrap().seams[0].marks;
    for (p, m) in w.players.iter().zip(marks) {
        assert_eq!((p.pos.x, p.pos.z), (m.at.x, m.at.z));
        assert!(p.health > 0);
    }
}

#[test]
fn one_alone_goes_on_after_holding_the_seam() {
    let mut w = World::versus_in(CLASSES, valley::START);
    w.advance(idle());
    let zone = valley::place(w.arena).unwrap().seams[0].zone;
    w.players[0].pos = standing_in(&zone, w.arena);
    let hold = sim::tuning::seam_hold();
    for _ in 1..hold {
        w.advance(idle());
        assert_eq!(w.arena, ArenaId::HEARTH, "went before the hold was up");
    }
    w.advance(idle());
    assert_eq!(w.arena, ArenaId::MOUTH, "never went");
}

#[test]
fn an_unlit_waystone_holds_its_seam_and_a_trophy_lights_it() {
    let mut w = arrived(ArenaId::BANK, Some(0));
    walk_both_into(&mut w, 2);
    for _ in 0..sim::tuning::seam_hold() + 5 {
        w.advance(idle());
        assert_eq!(w.arena, ArenaId::BANK, "the dark waystone let them through");
    }
    // A trophy, on the wire.
    let credit = Input::new(0).travelling(Travel::credit(SpeciesId::GNAWERS));
    assert_eq!(
        credit.travel.destination(),
        Some(Destination::Credit(SpeciesId::GNAWERS))
    );
    w.advance([credit, Input::new(0)]);
    w.advance(idle());
    assert_eq!(w.arena, ArenaId::SHELVES);
    assert!(
        w.valley.has_beaten(SpeciesId::GNAWERS),
        "the journey forgot"
    );
}

#[test]
fn a_seam_into_a_room_starts_its_hunt_and_back_out_is_open() {
    let mut w = arrived(ArenaId::PINEWOOD, Some(0));
    walk_both_into(&mut w, 3);
    w.advance(idle());
    assert_eq!(w.arena, ArenaId::HIGHLANDS);
    assert!(w.hunting() && w.valley.on && !w.pvp());
    assert_eq!(w.monster().map(|m| m.species), Some(SpeciesId::RIDGEBACK));
    // Arriving stands you in the way out, and that is not leaving.
    for _ in 0..30 {
        w.advance(idle());
    }
    assert_eq!(w.arena, ArenaId::HIGHLANDS);
}

#[test]
fn a_won_hunt_goes_quiet_and_is_credited() {
    let mut w = arrived(ArenaId::HIGHLANDS, Some(0));
    w.monster_mut().unwrap().health = 0;
    w.advance(idle());
    assert!(matches!(w.phase, Phase::RoundOver { .. }));
    assert!(w.valley.has_beaten(SpeciesId::RIDGEBACK));
    for _ in 0..sim::tuning::round_over_frames() + 2 {
        w.advance(idle());
    }
    assert_eq!(w.arena, ArenaId::HIGHLANDS);
    assert!(!w.hunting() && w.peaceful() && matches!(w.phase, Phase::Fighting));
    assert!(w.players.iter().all(|p| p.health > 0));
}

#[test]
fn a_lost_hunt_wakes_you_outside() {
    let mut w = arrived(ArenaId::HIGHLANDS, Some(0));
    for p in w.players.iter_mut() {
        p.health = 0;
    }
    for _ in 0..sim::tuning::round_over_frames() + 3 {
        w.advance(idle());
    }
    assert_eq!(w.arena, ArenaId::PINEWOOD);
    let marks = valley::place(ArenaId::PINEWOOD).unwrap().seams[3].marks;
    assert_eq!(
        (w.players[0].pos.x, w.players[0].pos.z),
        (marks[0].at.x, marks[0].at.z)
    );
    assert!(!w.valley.has_beaten(SpeciesId::RIDGEBACK));
}

#[test]
fn a_death_in_a_reach_stands_you_on_your_last_cairn() {
    let mut w = arrived(ArenaId::MOUTH, Some(0));
    let place = valley::place(ArenaId::MOUTH).unwrap();
    let (k, cairn) = place.cairns().next().expect("the Mouth has cairns");
    w.players[0].pos = V3::new(
        cairn.min.x.add(Fx::ratio(1, 2)),
        cairn.max.y,
        cairn.min.z.add(Fx::ratio(1, 2)),
    );
    w.players[0].health = 10;
    w.advance(idle());
    assert_eq!(w.valley.cairn[0] as usize, k + 1, "the cairn did not count");
    assert!(w.players[0].health > 10, "a cairn rests you");
    w.players[0].pos = V3::new(Fx::from_int(-60), Fx::ZERO, Fx::ZERO);
    w.players[0].health = 0;
    w.advance(idle());
    let p = &w.players[0];
    assert!(p.health > 0);
    assert!(
        cairn.over(p.pos.x, p.pos.z, Fx::ZERO),
        "not stood back on the cairn"
    );
    assert!(
        matches!(w.phase, Phase::Fighting),
        "a death in a reach ended a round"
    );
}

#[test]
fn nobody_can_hurt_anybody_in_a_reach() {
    let mut w = arrived(ArenaId::MOUTH, Some(0));
    w.players[1].pos = w.players[0]
        .pos
        .add(w.players[0].facing.scale(Fx::from_int(1)));
    let before = w.players[1].health;
    for f in 0..240u32 {
        let swing = if f % 30 < 4 { Input::LEFT } else { 0 };
        w.advance([Input::new(swing), Input::new(0)]);
    }
    assert_eq!(w.players[1].health, before, "a blow landed in a reach");
}

#[test]
fn a_vine_is_climbed_with_the_jump_held() {
    let mut w = arrived(ArenaId::MOUTH, Some(0));
    let vine = valley::vines(ArenaId::MOUTH)[0];
    let foot = standing_in(&vine, ArenaId::MOUTH);
    w.players[0].pos = V3::new(foot.x, w.arena().ground_under(foot), foot.z);
    let start = w.players[0].pos.y;
    // Straight up for three seconds: a vine is slow, but it goes.
    for _ in 0..180 {
        w.advance([Input::new(Input::SPACE), Input::new(0)]);
    }
    let climbed = w.players[0].pos.y.sub(start);
    assert!(
        climbed.raw() > Fx::from_int(6).raw(),
        "climbed only {climbed:?}"
    );
    // Let go of everything: it clings.
    let held = w.players[0].pos.y;
    for _ in 0..30 {
        w.advance(idle());
    }
    assert!(
        w.players[0].pos.y.sub(held).abs().raw() < Fx::ratio(1, 10).raw(),
        "it did not cling"
    );
}

#[test]
fn an_updraft_carries_a_body_in_the_air_up() {
    let mut w = arrived(ArenaId::SADDLE, Some(0));
    let vent = valley::vents(ArenaId::SADDLE)[0];
    let at = V3::new(vent.at.0, vent.top.sub(Fx::from_int(4)), vent.at.1);
    w.players[0].pos = at;
    w.players[0].vel = V3::ZERO;
    w.players[0].grounded = false;
    for _ in 0..20 {
        w.advance(idle());
    }
    assert!(
        w.players[0].pos.y.raw() > at.y.raw(),
        "the updraft did not lift"
    );
}

#[test]
fn outside_the_valley_there_are_no_vines_or_seams() {
    let mut w = World::with_classes(CLASSES);
    let before = w.checksum();
    w.advance(idle());
    assert!(!w.valley.on);
    assert_ne!(before, w.checksum());
    assert!(valley::vines(ArenaId::PROVING_GROUND).is_empty());
    assert!(valley::place(ArenaId::PROVING_GROUND).is_none());
}

#[test]
fn the_valley_is_deterministic_across_a_trip() {
    let run = || {
        let mut w = World::versus_in(CLASSES, valley::START);
        walk_both_into(&mut w, 0);
        for f in 0..200u32 {
            let bits = if f % 50 < 20 { Input::W } else { Input::SPACE };
            w.advance([Input::new(bits), Input::new(Input::D)]);
        }
        w.checksum()
    };
    assert_eq!(run(), run());
}

/// **The hops nobody can go round**: the jumps on the valley's way up that
/// have no vine, no stair and no updraft beside them, as gap and rise in
/// centimetres. Every class has to make each of them with a plain running
/// jump -- the airdodge is a spare, not a requirement -- because the main
/// route is for everybody (`docs/design/courses.md` §0).
const REQUIRED: &[(&str, i32, i32)] = &[
    (
        "the Mouth: the second terrace to the traverse's first ledge",
        260,
        150,
    ),
    ("the Mouth: along the traverse", 200, 200),
    ("the Mouth: the last ledge to the Lip", 0, 90),
    ("the Bank: up the stair", 100, 220),
    ("the Pinewood: the Highlands' stair to its notch", 300, 80),
    ("the Pinewood: up the Highlands' stair", 100, 220),
    ("the Ring: over its wall to the door", 0, 150),
];

#[test]
fn every_class_can_make_the_hops_nobody_can_go_round() {
    for class in sim::class::ALL_CLASSES {
        let path = sim::envelope::off_the_edge(class, sim::envelope::Extra::Nothing);
        let apex = sim::envelope::path_apex(&path);
        for (what, gap, rise) in REQUIRED {
            // High enough with room to spare, and far enough at that height:
            // a short hop up is a jump from closer in with less run, which a
            // person does without thinking, so the running path's own width
            // at the rise is what bounds it.
            let (gap, rise) = (Fx::ratio(*gap, 100), Fx::ratio(*rise, 100));
            let widest = sim::envelope::widest(std::slice::from_ref(&path), rise);
            assert!(
                apex.raw() >= rise.add(Fx::ratio(3, 10)).raw()
                    && widest.is_some_and(|w| w.raw() >= gap.raw()),
                "{}: cannot make {what} ({} m out, {} m up)",
                class.name(),
                gap.to_f32_for_render(),
                rise.to_f32_for_render()
            );
        }
    }
}
