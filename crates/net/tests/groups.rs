//! **Groups on the wire** (`net::group`, exploration 0029 §13): four peers
//! on lines that lose one packet in twenty, two groups of two, each in a
//! world of its own. One group is in the town and the other on the High
//! Plane, half a kilometre off; the second fast-travels to a cairn beside
//! the first, and the two **merge** into one group of four; then it
//! fast-travels far away again, and they **split** back into two. At every
//! stage every member of a group holds the same world at the frames they
//! have all confirmed, and while apart each group's members are its own two.

use net::group::Session;
use net::loopback::{Lossy, MemoryLine, Wire};
use sim::input::{Input, Travel};
use sim::{Class, World};

type L = Lossy<MemoryLine>;

fn mesh(ids: &[u16], lose_one_in: usize) -> Vec<Vec<(u16, L)>> {
    let mut lines: Vec<Vec<(u16, L)>> = ids.iter().map(|_| Vec::new()).collect();
    let mut seed = 1;
    for a in 0..ids.len() {
        for b in a + 1..ids.len() {
            let wire = Wire::default();
            lines[a].push((ids[b], Lossy::new(wire.end(0), lose_one_in, seed)));
            lines[b].push((ids[a], Lossy::new(wire.end(1), lose_one_in, seed + 1)));
            seed += 2;
        }
    }
    lines
}

fn group(place: sim::arena::ArenaId, classes: [Class; 2], ids: [u16; 2]) -> World {
    let mut w = World::arrive(classes, sim::valley::Journey::default(), place, None, 2);
    w.ids[0] = ids[0];
    w.ids[1] = ids[1];
    w
}

/// The cairn nearest a point of the map that wakes nothing when a group
/// arrives at it, and the one furthest from it.
fn cairns_by(at: sim::V3, from: &World) -> (u8, u8) {
    let atlas = sim::atlas::valley();
    let d = |k: &usize| {
        let top = sim::valley::top_of(&atlas.solids[sim::valley::cairns()[*k] as usize]);
        // In three dimensions: the Needle's top is near the town on the
        // flat and 350 m up, on a roost that wakes as anybody arrives.
        sim::math::wide_len(top.sub(at)).raw()
    };
    // A cairn beside a den wakes its pack a while after a group arrives
    // (cairn 7's gnawers come out some five hundred frames in), and a hunt
    // is never split: the meeting is at a cairn that stays peaceful for as
    // long as the test plays there.
    let quiet = |k: &usize| {
        let mut w = from.clone();
        let go = Input::default().travelling(Travel::cairn(*k as u8));
        w.advance([go, go]);
        for _ in 0..1500 {
            w.advance([Input::default(); 2]);
        }
        !w.hunting()
    };
    let mut all: Vec<usize> = (0..sim::valley::cairns().len()).collect();
    all.sort_by_key(|k| d(k));
    let near = *all.iter().find(|k| quiet(k)).expect("a quiet cairn");
    let far = *all.last().unwrap();
    (near as u8, far as u8)
}

/// Every member of a group holds the same world at a frame all of them have
/// confirmed and still have.
fn agree(peers: &[Session<L>], who: &[usize]) -> bool {
    let f = who.iter().map(|&i| peers[i].confirmed).min().unwrap();
    let sums: Vec<Option<u64>> = who
        .iter()
        .map(|&i| peers[i].world_at(f).map(|w| w.checksum()))
        .collect();
    sums.iter().all(|s| s.is_some() && *s == sums[0])
}

#[test]
fn two_groups_merge_when_they_meet_and_split_when_they_part() {
    let ids = [1u16, 2, 3, 4];
    let town = group(
        sim::valley::START,
        [Class::Champion, Class::Elementalist],
        [1, 2],
    );
    let plane = group(
        sim::arena::ArenaId::HIGH_PLANE,
        [Class::DualMage, Class::ShadowReaver],
        [3, 4],
    );
    let (near, far) = cairns_by(town.players[0].pos.add(town.map_origin()), &plane);
    let mut lines = mesh(&ids, 20);
    let mut peers: Vec<Session<L>> = ids
        .iter()
        .enumerate()
        .map(|(k, &id)| {
            let w = if id <= 2 { town.clone() } else { plane.clone() };
            Session::new(id, w, std::mem::take(&mut lines[k]))
        })
        .collect();
    let tick = |peers: &mut Vec<Session<L>>, trip: Option<(u16, Travel)>| {
        for p in peers.iter_mut() {
            let mut input = Input::default();
            if let Some((id, t)) = trip
                && id == p.me
            {
                input = input.travelling(t);
            }
            p.tick(input);
        }
    };
    // Apart: two groups, and each agrees within itself.
    for _ in 0..120 {
        tick(&mut peers, None);
    }
    assert_eq!(peers[0].members(), vec![1, 2]);
    assert_eq!(peers[2].members(), vec![3, 4]);
    assert!(agree(&peers, &[0, 1]) && agree(&peers, &[2, 3]));
    // The plane's pair to the cairn by the town (each presses it; the first
    // press of the pair is what moves them both into its frame).
    tick(&mut peers, Some((3, Travel::cairn(near))));
    tick(&mut peers, Some((4, Travel::cairn(near))));
    let mut merged_at = None;
    for t in 0..900 {
        tick(&mut peers, None);
        if merged_at.is_none() && peers.iter().all(|p| p.members().len() == 4) {
            merged_at = Some(t);
        }
    }
    let merged_at = merged_at.expect("the two groups never merged");
    eprintln!("merged {merged_at} ticks after they met");
    for p in &peers {
        assert_eq!(p.members(), vec![1, 2, 3, 4], "peer {}", p.me);
    }
    assert!(agree(&peers, &[0, 1, 2, 3]), "one group, one world");
    // And apart again: the pair to the furthest cairn.
    tick(&mut peers, Some((3, Travel::cairn(far))));
    tick(&mut peers, Some((4, Travel::cairn(far))));
    for _ in 0..300 {
        tick(&mut peers, None);
    }
    assert_eq!(peers[0].members(), vec![1, 2]);
    assert_eq!(peers[1].members(), vec![1, 2]);
    assert_eq!(peers[2].members(), vec![3, 4]);
    assert_eq!(peers[3].members(), vec![3, 4]);
    assert!(agree(&peers, &[0, 1]) && agree(&peers, &[2, 3]));
    for p in &peers {
        eprintln!(
            "peer {}: {} merges, {} splits, {} rollbacks, {} stalls",
            p.me, p.merges, p.splits, p.rollbacks, p.stalls
        );
    }
}
