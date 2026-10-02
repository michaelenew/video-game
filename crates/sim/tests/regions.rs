//! The region grid and region checksums: `docs/design/regions.md`.
//!
//! The ledger that keeps the books is `crates/regions`; these are the promises
//! its books rest on. A body is always in one to three regions and one to three
//! checksum zones while the grid is sound. A region's checksum moves with
//! what stands in its zone and nothing else. The state that belongs to no body
//! is counted once, in the fight's home region.

use sim::math::V3;
use sim::region::{Grid, RegionId};
use sim::state::Phase;
use sim::{Fx, World};

fn at(x: i32, z: i32) -> V3 {
    V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z))
}

#[test]
fn the_tuned_grid_is_sound() {
    let g = Grid::tuned();
    assert!(
        g.sound(),
        "h + overlap must stay under half a hexagon: {g:?}"
    );
    assert_eq!(
        g.horizon(),
        Fx::from_int(32),
        "24 m of reach and 8 frames at 60 m/s"
    );
}

#[test]
fn the_origin_is_a_corner_of_three() {
    let g = Grid::tuned();
    let zones = g.zones_of(V3::ZERO);
    assert_eq!(zones.len(), 3, "{zones:?}");
    assert!(zones.contains(g.fight_home()));
    // And the whole proving ground is in all three regions.
    for (x, z) in [(-14, -14), (14, -14), (-14, 14), (14, 14), (0, 0)] {
        let regions = g.regions_of(at(x, z));
        assert_eq!(regions.len(), 3, "({x}, {z}): {regions:?}");
    }
}

#[test]
fn every_point_is_in_one_to_three_regions_and_zones() {
    let g = Grid::tuned();
    let mut seen_three = false;
    for x in (-300..=300).step_by(7) {
        for z in (-300..=300).step_by(7) {
            let p = at(x, z);
            let regions = g.regions_of(p).len();
            let zones = g.zones_of(p).len();
            assert!(
                (1..=3).contains(&regions),
                "({x}, {z}) in {regions} regions"
            );
            assert!((1..=3).contains(&zones), "({x}, {z}) in {zones} zones");
            assert!(g.regions_of(p).contains(g.home(p)));
            seen_three |= regions == 3;
        }
    }
    assert!(
        seen_three,
        "a sweep that never meets a corner is not testing much"
    );
}

#[test]
fn a_hexagon_centre_is_in_its_own_hexagon() {
    let g = Grid::tuned();
    for q in -4..=4 {
        for r in -4..=4 {
            let (x, z) = sim::math::hex_centre(q, r, g.size);
            // The grid is shifted so a corner is the origin.
            let p = V3::new(x, Fx::ZERO, z.sub(g.size));
            assert_eq!(g.home(p), RegionId::new(q as i16, r as i16));
            assert_eq!(g.gap(g.home(p), p), Fx::ZERO);
        }
    }
}

#[test]
fn the_gap_is_the_distance_to_the_nearest_edge() {
    let g = Grid::tuned();
    // Hexagon (0, 0) has its +z corner on the origin, so it lies towards -z:
    // ten metres the other way, along the edge between its two neighbours, is
    // ten metres from it, and ten metres into it is no distance at all.
    let h00 = RegionId::new(0, 0);
    let gap = g.gap(h00, at(0, 10));
    assert!((gap.raw() - Fx::from_int(10).raw()).abs() < 64, "{gap:?}");
    assert_eq!(g.gap(h00, at(0, -10)), Fx::ZERO);
    let home = g.fight_home();
    // And a point a long way off is a long way from it.
    assert!(g.gap(home, at(500, 500)).raw() > Fx::from_int(300).raw());
}

#[test]
fn an_unsound_grid_puts_somebody_in_four() {
    let g = Grid {
        size: Fx::from_int(40),
        ..Grid::tuned()
    };
    assert!(!g.sound());
    let crowded = (-60..=60)
        .step_by(3)
        .flat_map(|x| (-60..=60).step_by(3).map(move |z| (x, z)))
        .any(|(x, z)| g.regions_of(at(x, z)).len() >= 4);
    assert!(crowded, "the watchdog's first check has to be able to fail");
}

#[test]
fn a_region_checksum_is_what_stands_in_its_zone() {
    let g = Grid::tuned();
    let w = World::new();
    let zones = g.zones_of(V3::ZERO);
    for &r in zones.as_slice() {
        assert_eq!(w.region_checksum(&g, r), w.clone().region_checksum(&g, r));
    }

    // Move fighter one a long way off: every region it left changes, and a
    // region it never stood in does not.
    let far = RegionId::new(20, -20);
    let mut moved = w.clone();
    moved.players[1].pos = at(2000, 2000);
    assert_eq!(w.region_checksum(&g, far), moved.region_checksum(&g, far));
    for &r in zones.as_slice() {
        if g.in_zone(r, w.players[1].pos) {
            assert_ne!(
                w.region_checksum(&g, r),
                moved.region_checksum(&g, r),
                "{r:?}"
            );
        }
    }
}

#[test]
fn state_that_belongs_to_nobody_is_counted_once_in_the_home_region() {
    let g = Grid::tuned();
    let w = World::new();
    let mut over = w.clone();
    over.phase = Phase::RoundOver { winner: 0, left: 9 };
    for &r in g.zones_of(V3::ZERO).as_slice() {
        let same = w.region_checksum(&g, r) == over.region_checksum(&g, r);
        assert_eq!(same, r != g.fight_home(), "{r:?}");
    }
}
