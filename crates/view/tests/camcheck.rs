//! **The camera check judges the camera, not the route**
//! (`view::camcheck`, `docs/design/level-design.md`, "The play check").

use sim::{Class, World};
use view::camcheck::{Kind, walk};

fn terrain(name: &str) -> sim::arena::Terrain {
    let a = sim::arena::named(name).expect("a registered arena");
    World::versus_in([Class::Champion; sim::state::MAX_PLAYERS], a.id).terrain()
}

/// The Shrine is an open court, walled low, with four columns the camera
/// passes: a walk across it and a turn at each stop is a clean pass.
#[test]
fn an_open_court_passes() {
    let r = walk(
        &terrain("shrine"),
        &[
            [-11.0, 0.0, 2.0],
            [0.0, -5.0, 2.0],
            [5.0, 5.0, 2.0],
            [-5.0, 8.0, 2.0],
        ],
        &Default::default(),
    );
    assert!(r.passes(), "{}", r.text());
    assert_eq!(r.count(Kind::Jump), 0, "{}", r.text());
}

/// A route drawn through the side of a platform puts the fighter where a
/// body cannot be. Those frames are counted apart and not judged, so a bad
/// route is reported as one rather than as the camera's fault.
#[test]
fn a_route_through_a_solid_is_skipped_not_blamed() {
    // The proving ground's west platform runs x -9 to -5, z -4 to 4, 1.5 m
    // tall; this stop stands on its edge, half inside it.
    let r = walk(
        &terrain("proving_ground"),
        &[[0.0, 0.0, 1.0], [-5.0, 0.0, 1.0]],
        &Default::default(),
    );
    assert!(r.inside > 0, "{}", r.text());
    assert!(r.passes(), "{}", r.text());
}
