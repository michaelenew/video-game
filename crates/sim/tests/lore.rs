//! The hunt's own room in the snapshot (`sim::lore`): one region, sized to the
//! largest fight, that each fight lays out differently.
//!
//! What is pinned: every registered species' layout fits it; the `World`
//! with every cell of it full still fits the 4 KiB snapshot, with room left
//! over; a species' own words round-trip and never spill into the generic
//! lists; and a fight that uses none of it hashes as before there was any.

use sim::hazard;
use sim::lore::{CELLS, Lore};
use sim::species::{self, SpeciesId, sentinel};
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, World};

/// The snapshot's cap: see `tests/budget.rs`.
const SNAPSHOT_CAP: usize = 4096;

#[test]
fn every_species_layout_fits_the_region() {
    for sp in species::all() {
        let l = sp.fight.layout;
        assert!(
            l.cells() <= CELLS,
            "{} lays out {} cells of {CELLS}",
            sp.name,
            l.cells()
        );
        assert!(l.hazards as usize <= hazard::MAX_HAZARDS, "{}", sp.name);
        if !sp.fight.hazards.is_empty() {
            assert!(
                l.hazards > 0,
                "{} has hazard kinds and no room for one",
                sp.name
            );
        }
        assert!(
            l.objectives as usize >= sp.fight.objectives.len(),
            "{} declares {} objectives and room for {}",
            sp.name,
            sp.fight.objectives.len(),
            l.objectives
        );
        if sp.fight.hears {
            assert!(l.noises > 0, "{} hears and keeps no ring", sp.name);
            assert!(sp.fight.row, "{} hears with no loudness row", sp.name);
        }
        if sp.fight.collides {
            assert!(sp.fight.row, "{} collides with no body radius", sp.name);
        }
    }
}

#[test]
fn the_snapshot_fits_with_every_cell_of_the_region_full() {
    let size = std::mem::size_of::<World>();
    let region = std::mem::size_of::<Lore>();
    println!(
        "World {size} B of {SNAPSHOT_CAP}; the region {region} B ({CELLS} cells); {} B to spare",
        SNAPSHOT_CAP - size
    );
    assert!(size <= SNAPSHOT_CAP);
    assert!(region >= CELLS * 16);
    // Every cell full, in a fight with every list: the size does not move,
    // because it is a fixed array, and it still clones without allocating.
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::SENTINEL);
    for i in 0..CELLS * 4 {
        w.lore.set_word(i, 0xA5A5_A5A5);
    }
    w.advance([Input::default(); MAX_PLAYERS]);
    let copy = w.clone();
    assert_eq!(copy.state_checksum(), w.state_checksum());
    // A creature document's itemised worst case, after what the world
    // already holds for every fight (monsters, critters, the pack): what the
    // region has to hold, in cells, against what it has.
    for (who, cells) in [
        ("Mireback: sixteen pools, braziers, swallow", 18),
        (
            "Siegeshell: vents, falling things, anchors, siege, the wall",
            19,
        ),
        ("Veilstalker: footfalls, paint, veil, four hazards", 15),
        (
            "Sandmaw: noise ring, sinkholes, spine, attention, swallow",
            17,
        ),
        ("Broodmother: patches, strands, sacs, list", 13),
    ] {
        assert!(cells < CELLS, "{who} needs {cells} cells of {CELLS}");
    }
}

#[test]
fn a_species_own_words_round_trip_and_never_spill() {
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::SENTINEL);
    let l = sentinel::FIGHT.layout;
    let generic = (l.hazards + l.noises + l.objectives) as usize;
    let own = CELLS - generic;
    for i in 0..own * 4 {
        w.lore.set_int(i, -(i as i32) - 1);
    }
    for i in 0..own * 4 {
        assert_eq!(w.lore.int(i), -(i as i32) - 1);
    }
    // A write past the end is dropped, a read past it is zero.
    w.lore.set_word(own * 4 + 3, 7);
    assert_eq!(w.lore.word(own * 4 + 3), 0);
    // The generic lists are untouched by the species' words.
    assert_eq!(hazard::all(&w.lore).count(), 0);
    assert_eq!(sim::noise::all(&w.lore).count(), 0);
}

#[test]
fn a_fight_that_uses_none_of_it_keeps_it_blank() {
    for id in [SpeciesId::RIDGEBACK, SpeciesId::GNATS] {
        let mut w = World::hunt_of([Class::Elementalist; MAX_PLAYERS], id);
        assert_eq!(w.lore.owner, Some(id));
        for f in 0..300u32 {
            w.advance(
                [Input::aimed(Input::W | (f as u16 & Input::SPACE), (f * 211) as u16); MAX_PLAYERS],
            );
        }
        assert!(w.lore.is_blank(), "{id:?}");
        assert_eq!(w.lore.layout(), sim::lore::Layout::NONE);
    }
}
