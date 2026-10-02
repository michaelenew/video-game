//! The watchdog: quiet on a real fight with the tuned grid, and biting when
//! the grid is declared smaller than what the fight does.
//! `docs/design/regions.md` §"The watchdog". One test, because it moves Oven
//! knobs and the Oven is shared by everything in the process.

mod common;

use common::Run;
use regions::Finding;
use sim::oven::{self, Scalar};
use sim::{Class, Fx};

const FRAMES: u32 = 1500;

fn play(classes: [Class; 2], seed: u32) -> Run {
    let mut run = Run::hunt(classes, seed);
    for _ in 0..FRAMES {
        run.tick();
        if !matches!(run.world.phase, sim::state::Phase::Fighting) {
            break;
        }
    }
    run
}

#[test]
fn quiet_when_tuned_and_biting_when_the_speed_of_light_is_declared_too_low() {
    // As tuned, the whole arena is in all three regions round the origin, so
    // every fighter is a member of every live region: the re-run has every
    // input and must agree exactly, and nothing may be crowded or unsound.
    let run = play([Class::Champion, Class::Elementalist], 11);
    let watch = run.ledger.watch().unwrap();
    assert!(
        watch.reruns > 100,
        "only {} locality checks ran",
        watch.reruns
    );
    let wrong: Vec<String> = watch
        .issues()
        .filter(|i| !matches!(i.finding, Finding::TooFast { .. }))
        .map(|i| i.to_string())
        .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
    for i in watch.issues() {
        eprintln!("tuned: {i}");
    }
    eprintln!(
        "tuned: {} locality checks, frame {}",
        watch.reruns, run.world.frame
    );

    // Declare a speed of light far below what a fight does -- two metres of
    // reach, five metres a second -- on a grid small enough to stay sound.
    // Now the fighters stand outside each other's regions, and the re-run
    // without the outsiders' inputs has to come out differently somewhere.
    let fx = |v: i32| Fx::from_int(v).raw();
    oven::set_scalar(Scalar::RegionReach, fx(2));
    oven::set_scalar(Scalar::RegionSpeed, fx(5));
    oven::set_scalar(Scalar::RegionOverlap, fx(1));
    oven::set_scalar(Scalar::RegionSize, fx(16));
    let tight = play([Class::Champion, Class::Elementalist], 11);
    oven::reset_to_baked();
    let watch = tight.ledger.watch().unwrap();
    let all: Vec<String> = watch.issues().map(|i| i.to_string()).collect();
    for i in &all {
        eprintln!("tight: {i}");
    }
    assert!(
        watch
            .issues()
            .any(|i| matches!(i.finding, Finding::Outran { .. })),
        "a speed of light of 5 m/s cannot survive a fight: {all:#?}"
    );
    assert!(
        watch
            .issues()
            .any(|i| matches!(i.finding, Finding::TooFast { .. })),
        "nor can it survive a dash: {all:#?}"
    );
    assert!(
        !watch
            .issues()
            .any(|i| matches!(i.finding, Finding::Unsound { .. } | Finding::Crowded { .. })),
        "the small grid is still sound: {all:#?}"
    );
}
