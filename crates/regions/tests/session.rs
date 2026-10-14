//! **The region experiment** (`docs/design/exploration/0005`, "the experiment
//! that would settle it"; `0018` §7.1): one global simulation, and three
//! regions each run by a peer that has only its members' inputs and the
//! others' outcomes `delay` frames late. Two fighters walk through all three
//! regions, meet in the middle and shove. Every region's checksum zone must
//! equal the global run's, at every frame an outcome has arrived for; and
//! with the reach declared too small, it must not.
//!
//! One test, because it moves Oven knobs and the Oven is shared.

use regions::session::{Outcome, Peer};
use sim::class::Class;
use sim::input::Input;
use sim::math::V3;
use sim::oven::{self, Scalar};
use sim::region::{Grid, RegionId};
use sim::state::MAX_PLAYERS;
use sim::{Fx, World};

const FRAMES: u32 = 240;

fn fx(v: Fx) -> i32 {
    v.raw()
}

/// Hexagons eight metres across the proving ground, sound for a fight that
/// only walks: reach one metre (a shove), speed eight (a run is seven).
fn small_grid(reach: Fx, speed: Fx, overlap: Fx) {
    oven::set_scalar(Scalar::RegionReach, fx(reach));
    oven::set_scalar(Scalar::RegionSpeed, fx(speed));
    oven::set_scalar(Scalar::RegionOverlap, fx(overlap));
    oven::set_scalar(Scalar::RegionSize, fx(Fx::from_int(8)));
    oven::set_scalar(Scalar::RegionDelay, 8);
}

/// Two fighters on a line through three hexagons, facing each other, placed
/// so that they meet on the line between two of them -- where a shove from
/// a body one region has not heard from is the thing the overlap exists to
/// account for.
fn start() -> World {
    let mut w = World::with_classes([Class::Champion; 2]);
    // Hexagon (0, 0)'s centre is at z = -size on the floor: a line through it
    // along x crosses it and its two neighbours on that row, and the edge
    // between (0, 0) and (1, 0) is at x = size * root three / 2 = 6.93.
    w.players[0].pos = V3::new(Fx::ONE, Fx::ZERO, Fx::from_int(-8));
    w.players[1].pos = V3::new(Fx::from_int(13), Fx::ZERO, Fx::from_int(-8));
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    w
}

/// Walk at each other: forward, looking along the line.
fn walking() -> [Input; 2] {
    [Input::aimed(Input::W, 0), Input::aimed(Input::W, 1 << 15)]
}

/// Run the experiment: how many frames every region's zone agreed with the
/// truth, and the first frame one did not.
fn run(grid: Grid) -> (u32, Option<(u32, RegionId)>) {
    let truth0 = start();
    let regions: Vec<RegionId> = {
        let mut ids = Vec::new();
        for x in -13..=13 {
            let id = grid.home(V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(-8)));
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        ids
    };
    assert_eq!(regions.len(), 3, "{regions:?}");
    let mut peers: Vec<Peer> = regions
        .iter()
        .map(|r| Peer::new(*r, grid, truth0.clone()))
        .collect();
    let mut truth = truth0;
    // The truth's zone checksums, by frame then region.
    let mut sums: Vec<Vec<u64>> = vec![
        regions
            .iter()
            .map(|r| truth.region_checksum(&grid, *r))
            .collect(),
    ];
    let delay = grid.delay;
    let mut agreed = 0;
    let mut first_wrong = None;
    for _ in 0..FRAMES {
        let inputs = walking();
        let mut all = [Input::default(); MAX_PLAYERS];
        all[..2].copy_from_slice(&inputs);
        // Membership, decided from the truth: what the session layer will
        // decide from confirmed positions, the same on every machine.
        let origin = truth.map_origin();
        let members: Vec<[bool; MAX_PLAYERS]> = peers
            .iter()
            .map(|p| std::array::from_fn(|i| p.admits(truth.players[i].pos.add(origin))))
            .collect();
        truth.advance(all);
        sums.push(
            regions
                .iter()
                .map(|r| truth.region_checksum(&grid, *r))
                .collect(),
        );
        for (peer, m) in peers.iter_mut().zip(&members) {
            peer.advance(*m, all);
        }
        // Outcomes, `delay` frames late, from everybody to everybody.
        if truth.frame >= delay {
            let fo = truth.frame - delay;
            let outs: Vec<Outcome> = peers.iter().filter_map(|p| p.outcome(fo)).collect();
            for peer in peers.iter_mut() {
                for o in &outs {
                    peer.accept(o);
                }
            }
            // Every zone, exact at the frame the outcomes covered.
            for (k, peer) in peers.iter().enumerate() {
                let got = peer.zone_checksum(fo).expect("on the tape");
                let want = sums[fo as usize][k];
                if got == want {
                    agreed += 1;
                } else if first_wrong.is_none() {
                    first_wrong = Some((fo, peer.region));
                }
            }
        }
    }
    (agreed, first_wrong)
}

#[test]
fn three_regions_fed_outcomes_late_agree_with_the_truth_and_a_lie_about_reach_does_not() {
    // Sound, and honest about what a walking fight does.
    small_grid(Fx::ONE, Fx::from_int(8), Fx::ONE);
    let grid = Grid::tuned();
    assert!(grid.sound(), "{grid:?}");
    let (agreed, wrong) = run(grid);
    oven::reset_to_baked();
    assert!(agreed > 600, "only {agreed} zone-frames checked");
    assert_eq!(
        wrong, None,
        "a zone disagreed with the truth, after {agreed} agreed"
    );

    // Declare no reach at all, and a thin overlap: a shove from a body just
    // outside the membership radius is then an influence nobody accounted
    // for, and the inequality says a zone must come out wrong. (A thin
    // overlap, because an outcome *overwrites* the bodies in it: a wide one
    // repairs a lie about reach as far as it is wide, which is worth knowing
    // and is why the honest run above does not need to be told.)
    small_grid(Fx::ZERO, Fx::ratio(1, 10), Fx::ratio(1, 10));
    let grid = Grid::tuned();
    assert!(grid.sound(), "{grid:?}");
    let (_, wrong) = run(grid);
    oven::reset_to_baked();
    assert!(
        wrong.is_some(),
        "declaring no reach cannot survive two bodies shoving"
    );
}
