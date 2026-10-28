//! **The region session on the wire** (`net::region`, `docs/design/regions.md`
//! §"The session layer"): four peers on lossy loopback lines, each running
//! only the regions its fighter stands in, walking through three hexagons
//! and meeting on the boundaries. Every peer's checksum zones must equal the
//! global truth's at every frame they are confirmed and complete through;
//! and a fifth who joins late from a snapshot must be exact too.

use net::loopback::{Lossy, Wire};
use net::region::{OUTCOME_EVERY, Session};
use sim::class::Class;
use sim::input::Input;
use sim::math::V3;
use sim::oven::{self, Scalar};
use sim::region::{Grid, RegionId};
use sim::state::MAX_PLAYERS;
use sim::{Fx, World};

const FRAMES: u32 = 300;

fn small_grid() {
    let fx = |v: Fx| v.raw();
    oven::set_scalar(Scalar::RegionReach, fx(Fx::ONE));
    oven::set_scalar(Scalar::RegionSpeed, fx(Fx::from_int(8)));
    oven::set_scalar(Scalar::RegionOverlap, fx(Fx::ONE));
    oven::set_scalar(Scalar::RegionSize, fx(Fx::from_int(8)));
    oven::set_scalar(Scalar::RegionDelay, 8);
}

/// Four fighters along the line through three hexagons: two at each end,
/// walking toward the middle, so pairs meet on both boundaries.
fn start() -> World {
    let mut w = World::with_classes([Class::Champion; 4]);
    let z = Fx::from_int(-8);
    for (i, x) in [-13, -11, 11, 13].into_iter().enumerate() {
        w.players[i].pos = V3::new(Fx::from_int(x), Fx::ZERO, z);
        w.players[i].facing = V3::new(
            if x < 0 { Fx::ONE } else { Fx::ONE.neg() },
            Fx::ZERO,
            Fx::ZERO,
        );
    }
    w
}

fn input_of(seat: usize, frame: u32) -> Input {
    // Walk toward the middle, and stop after a while so the shove settles.
    let aim = if seat < 2 { 0 } else { 1 << 15 };
    if frame < 200 {
        Input::aimed(Input::W, aim)
    } else {
        Input::aimed(0, aim)
    }
}

/// When the late joiner starts walking: it joins at frame 40, and must
/// have its world by then even through loss.
const WALKS_FROM: u32 = 150;

/// The same, with the fourth seat standing still until `WALKS_FROM`: it
/// joins late, and what it does before it has a world is nothing.
fn input_of_late(seat: usize, frame: u32) -> Input {
    let aim = if seat < 2 { 0 } else { 1 << 15 };
    if frame < 200 && (seat != 3 || frame >= WALKS_FROM) {
        Input::aimed(Input::W, aim)
    } else {
        Input::aimed(0, aim)
    }
}

/// A full mesh of lossy lines between `n` peers: `(seat, lines)` each.
fn mesh(n: usize, lose_one_in: usize) -> Vec<Vec<(usize, Lossy<net::loopback::MemoryLine>)>> {
    let mut lines: Vec<Vec<(usize, Lossy<net::loopback::MemoryLine>)>> =
        (0..n).map(|_| Vec::new()).collect();
    let mut seed = 1u64;
    for a in 0..n {
        for b in (a + 1)..n {
            let wire = Wire::default();
            lines[a].push((b, Lossy::new(wire.end(0), lose_one_in, seed)));
            lines[b].push((a, Lossy::new(wire.end(1), lose_one_in, seed + 1)));
            seed += 2;
        }
    }
    lines
}

fn regions_on_the_line(grid: &Grid) -> Vec<RegionId> {
    let mut ids = Vec::new();
    for x in -14..=14 {
        let id = grid.home(V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(-8)));
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

#[test]
fn four_peers_running_only_their_regions_agree_with_the_truth_through_loss() {
    small_grid();
    let grid = Grid::tuned();
    assert!(grid.sound());
    let regions = regions_on_the_line(&grid);
    assert_eq!(regions.len(), 3);
    let truth0 = start();
    let mut truth = truth0.clone();
    let mut sums: Vec<Vec<u64>> = vec![
        regions
            .iter()
            .map(|r| truth.region_checksum(&grid, *r))
            .collect(),
    ];
    let mut lines = mesh(4, 10);
    let mut peers: Vec<Session<_>> = (0..4)
        .map(|me| Session::new(me, grid, truth0.clone(), std::mem::take(&mut lines[me])))
        .collect();
    let mut checked = 0u32;
    for _ in 0..FRAMES {
        let f = truth.frame + 1;
        let mut all = [Input::default(); MAX_PLAYERS];
        for (i, slot) in all.iter_mut().enumerate() {
            *slot = input_of(i, f);
        }
        truth.advance(all);
        sums.push(
            regions
                .iter()
                .map(|r| truth.region_checksum(&grid, *r))
                .collect(),
        );
        // A peer ticks with its own input for the frame it is about to make;
        // one that stalls keeps its input for the frame it will make next.
        for p in peers.iter_mut() {
            let next = p.world.as_ref().unwrap().frame + 1;
            p.tick(input_of(p.me, next));
        }
    }
    oven::reset_to_baked();
    // Every peer, every region it is in, at every frame it is confirmed
    // through and the neighbours' outcomes have reached: the truth's zone.
    for p in &peers {
        let w = p.world.as_ref().unwrap();
        assert!(
            w.frame >= FRAMES - 2 * (net::region::WINDOW),
            "peer {} fell behind: frame {}",
            p.me,
            w.frame
        );
        eprintln!(
            "peer {}: frame {}, confirmed {}, {} rollbacks, {} stalls, in {:?}",
            p.me,
            w.frame,
            p.confirmed,
            p.rollbacks,
            p.stalls,
            p.my_regions()
        );
        checked += agrees(p, &regions, &sums);
        // And the books it heard from the others, held against its own
        // where it is exact, all agreed.
        assert!(p.agreed > 20, "peer {} held only {} books", p.me, p.agreed);
        assert_eq!(
            p.disagreed, 0,
            "peer {} found a desync: {:?}",
            p.me, p.first_disagreement
        );
    }
    assert!(checked > 60, "only {checked} zone-frames checked");
}

#[test]
fn a_peer_whose_world_goes_wrong_is_found_by_its_neighbours_books() {
    small_grid();
    let grid = Grid::tuned();
    let truth0 = start();
    let mut lines = mesh(4, 0);
    let mut peers: Vec<Session<_>> = (0..4)
        .map(|me| Session::new(me, grid, truth0.clone(), std::mem::take(&mut lines[me])))
        .collect();
    let mut found_at = None;
    for f in 1..=FRAMES {
        for p in peers.iter_mut() {
            let next = p.world.as_ref().unwrap().frame + 1;
            p.tick(input_of(p.me, next));
        }
        if f == 150 {
            // A bit flips in one game: its own fighter is a little hurt.
            let w = peers[1].world.as_mut().unwrap();
            w.players[1].health = w.players[1].health.saturating_sub(1);
        }
        if found_at.is_none() && peers.iter().any(|p| p.disagreed > 0) {
            found_at = Some(f);
        }
    }
    oven::reset_to_baked();
    let at = found_at.expect("nobody noticed");
    assert!(at < 150 + 60, "noticed only at frame {at}");
    let who: Vec<usize> = peers
        .iter()
        .filter(|p| p.disagreed > 0)
        .map(|p| p.me)
        .collect();
    eprintln!("desync at frame 150 noticed at {at} by {who:?}");
    assert!(
        who.len() >= 2,
        "only {who:?} noticed: the wrong one and a neighbour both should"
    );
}

/// The frames the tape still holds, up to where the neighbours' outcomes
/// have reached: every one of the peer's regions' zones is the truth's.
fn agrees<L: net::meet::Line>(p: &Session<L>, regions: &[RegionId], sums: &[Vec<u64>]) -> u32 {
    let w = p.world.as_ref().unwrap();
    let through = p
        .confirmed
        .saturating_sub(OUTCOME_EVERY + net::region::WINDOW);
    let mut checked = 0;
    for f in w.frame.saturating_sub(regions::TAPE as u32 - 2)..=through {
        for r in p.my_regions() {
            let Some(k) = regions.iter().position(|x| *x == r) else {
                continue;
            };
            let Some(got) = p.zone_checksum(r, f) else {
                continue;
            };
            assert_eq!(
                got, sums[f as usize][k],
                "peer {} disagrees with the truth about region {r:?} at frame {f}",
                p.me
            );
            checked += 1;
        }
    }
    checked
}

#[test]
fn a_newcomer_takes_a_snapshot_and_is_exact_from_then_on() {
    small_grid();
    let grid = Grid::tuned();
    let regions = regions_on_the_line(&grid);
    let truth0 = start();
    let mut truth = truth0.clone();
    let mut sums: Vec<Vec<u64>> = vec![
        regions
            .iter()
            .map(|r| truth.region_checksum(&grid, *r))
            .collect(),
    ];
    let mut lines = mesh(4, 10);
    let mut peers: Vec<Session<_>> = (0..3)
        .map(|me| Session::new(me, grid, truth0.clone(), std::mem::take(&mut lines[me])))
        .collect();
    // The fourth joins at frame 40 with nothing but its lines, lossy too:
    // a snapshot chunk can go missing, and it asks again.
    let mut late = Session::joining(3, grid, std::mem::take(&mut lines[3]));
    for _ in 0..FRAMES {
        let f = truth.frame + 1;
        let mut all = [Input::default(); MAX_PLAYERS];
        for (i, slot) in all.iter_mut().enumerate() {
            *slot = input_of_late(i, f);
        }
        truth.advance(all);
        sums.push(
            regions
                .iter()
                .map(|r| truth.region_checksum(&grid, *r))
                .collect(),
        );
        for p in peers.iter_mut() {
            let next = p.world.as_ref().unwrap().frame + 1;
            p.tick(input_of_late(p.me, next));
        }
        if f == 40 {
            late.ask_for_the_world();
        }
        if f >= 40 {
            let next = late.world.as_ref().map_or(0, |w| w.frame + 1);
            late.tick(input_of_late(3, next));
        }
        if f == WALKS_FROM {
            assert!(
                late.world.is_some(),
                "no world after {} asks over {} frames",
                late.asked,
                f - 40
            );
        }
    }
    oven::reset_to_baked();
    let w = late.world.as_ref().expect("the newcomer got a world");
    assert!(
        w.frame > 200,
        "the newcomer never caught up: frame {}",
        w.frame
    );
    eprintln!(
        "newcomer: frame {}, confirmed {}, {} rollbacks, {} stalls, asked {} times, in {:?}",
        w.frame,
        late.confirmed,
        late.rollbacks,
        late.stalls,
        late.asked,
        late.my_regions()
    );
    assert!(agrees(&late, &regions, &sums) > 0);
    // And the old hands are exact too, with the newcomer among them.
    for p in &peers {
        assert!(agrees(p, &regions, &sums) > 0);
    }
}

#[test]
fn a_packet_of_inputs_is_small() {
    let p = net::region::Packet::Inputs {
        seat: 1,
        frame: 100,
        inputs: vec![Input::aimed(Input::W, 3); net::region::REDUNDANCY],
        regions: vec![RegionId::new(0, 0); 3],
        until: 100,
    };
    assert_eq!(net::region::Packet::decode(&p.encode()), Some(p.clone()));
    let n = p.encode().len();
    eprintln!(
        "inputs packet {n} bytes; a fighter {} bytes; a world {} bytes",
        net::raw::PLAYER,
        net::raw::WORLD
    );
    assert!(n <= 96, "{n} bytes a frame is more than a few dozen");
}

/// **Far peers hear a heartbeat, and are exact when they meet** (exploration
/// 0029 §9). Two fighters start eighty metres apart on the small grid --
/// far past where they could share a region before a heartbeat's promise
/// runs out -- and walk into each other. While far, each sends the other a
/// declaration a heartbeat instead of every input every frame; as they
/// close they switch to inputs, and every zone each is in agrees with the
/// truth at every frame, all the way through the meeting and the shove.
#[test]
fn far_peers_beat_and_are_exact_when_they_meet() {
    use net::loopback::{Meter, Metered};
    use std::sync::Arc;
    small_grid();
    let grid = Grid::tuned();
    let z = Fx::from_int(-60);
    let mut truth = World::with_classes([Class::Champion; 2]);
    for (i, x) in [-40, 40].into_iter().enumerate() {
        truth.players[i].pos = V3::new(Fx::from_int(x), Fx::ZERO, z);
    }
    let walk = |seat: usize, frame: u32| {
        let aim = if seat == 0 { 0 } else { 1 << 15 };
        Input::aimed(if frame < 560 { Input::W } else { 0 }, aim)
    };
    let mut regions: Vec<RegionId> = Vec::new();
    for x in -46..=46 {
        for dz in -4..=4 {
            let id = grid.home(V3::new(Fx::from_int(x), Fx::ZERO, z.add(Fx::from_int(dz))));
            if !regions.contains(&id) {
                regions.push(id);
            }
        }
    }
    let meters = [Arc::new(Meter::default()), Arc::new(Meter::default())];
    let wire = Wire::default();
    let line = |k: usize| vec![(1 - k, Metered::new(wire.end(k), meters[k].clone()))];
    let start = truth.clone();
    let mut peers: Vec<Session<_>> = (0..2)
        .map(|me| Session::new(me, grid, start.clone(), line(me)))
        .collect();
    let mut sums: Vec<Vec<u64>> = vec![
        regions
            .iter()
            .map(|r| truth.region_checksum(&grid, *r))
            .collect(),
    ];
    let mut early = 0;
    let mut checked = 0;
    for f in 1..=700u32 {
        truth.advance([walk(0, f), walk(1, f)]);
        sums.push(
            regions
                .iter()
                .map(|r| truth.region_checksum(&grid, *r))
                .collect(),
        );
        for p in peers.iter_mut() {
            let next = p.world.as_ref().unwrap().frame + 1;
            p.tick(walk(p.me, next));
        }
        if f == 60 {
            early = meters[0].bytes().0;
        }
        if f % 20 == 0 {
            for p in &peers {
                checked += agrees(p, &regions, &sums);
            }
        }
    }
    oven::reset_to_baked();
    let late = meters[0].bytes().0 - early;
    eprintln!("far: {early} bytes in 60 frames; then {late} in 640; checked {checked} zone-frames");
    // While far: a heartbeat every few frames, not a packet every frame.
    assert!(
        early < 60 * 20,
        "{early} bytes in the first second, far apart: inputs went every frame"
    );
    let d = peers[0].world.as_ref().unwrap().players[0]
        .pos
        .sub(peers[0].world.as_ref().unwrap().players[1].pos)
        .flat_len();
    assert!(
        d.raw() < Fx::from_int(4).raw(),
        "they never met: {d:?} apart"
    );
    assert!(checked > 200, "only {checked} zone-frames checked");
    for p in &peers {
        assert_eq!(
            p.disagreed, 0,
            "peer {} found a desync: {:?}",
            p.me, p.first_disagreement
        );
    }
}
