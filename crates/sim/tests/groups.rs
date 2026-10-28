//! **Groups: a world changes hands** (`sim::group`, exploration 0029 §12).
//!
//! - A merge keeps every fighter where they were on the map, with their ids,
//!   what they own and what they say of each other, and the merged world
//!   plays each group's fighters on exactly as their own world would have.
//! - A split is the world copied and the other side fed `Travel::LEFT`:
//!   they are gone at once, the rest exactly where they were, and a merge
//!   puts everybody back where their own group had them.
//! - Two hunts, two maps or too many fighters are refused.

use sim::group::{self, Refused};
use sim::input::Travel;
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};

fn on_map(w: &World, i: usize) -> V3 {
    w.players[i].pos.add(w.map_origin())
}

/// A group of two arrived at `place` of the valley, ids from `first`.
fn group(place: sim::arena::ArenaId, classes: [Class; 2], first: u64) -> World {
    let mut w = World::arrive(classes, sim::valley::Journey::default(), place, None, 2);
    w.ids[0] = first;
    w.ids[1] = first + 1;
    w
}

/// A seat's hands on frame `f`: walking in circles, swinging now and then.
fn hands(seat: usize, f: u32) -> Input {
    let aim = ((f * 97 + seat as u32 * 16_000) % 65_536) as u16;
    let bits = match (f / 30 + seat as u32) % 4 {
        0 => Input::W,
        1 => Input::W | Input::LEFT,
        2 => Input::RIGHT,
        _ => 0,
    };
    Input::aimed(bits, aim)
}

fn play(w: &mut World, frames: u32, seats: &[usize]) {
    for _ in 0..frames {
        let f = w.frame;
        let mut inputs = [Input::default(); MAX_PLAYERS];
        for &s in seats {
            inputs[s] = hands(s, f);
        }
        w.advance(inputs);
    }
}

#[test]
fn a_merge_keeps_every_fighter_where_they_were_and_plays_them_on() {
    // One group in the town; another up on the High Plane, 300 m up and
    // half a kilometre off, in a frame of its own, fifty frames ahead.
    let mut a = group(
        sim::valley::START,
        [Class::Champion, Class::Elementalist],
        10,
    );
    let mut b = group(
        sim::arena::ArenaId::HIGH_PLANE,
        [Class::BloodMage, Class::ShadowReaver],
        20,
    );
    play(&mut a, 120, &[0, 1]);
    play(&mut b, 170, &[0, 1]);
    assert_ne!(a.map_origin(), b.map_origin(), "two frames of reference");
    let owned_b = b.effects.iter().flatten().filter(|e| e.owner < 2).count();
    let m = group::merge(&a, &b).expect("two groups, one hunt between them: none");
    let w = &m.world;
    // Both of a's where they were; b's in the free slots, where they were
    // on the map.
    for j in 0..2 {
        let ka = m.from_a[j].expect("a's fighter kept");
        let kb = m.from_b[j].expect("b's fighter moved");
        assert_eq!(on_map(w, ka), on_map(&a, j));
        assert_eq!(on_map(w, kb), on_map(&b, j), "b's {j} kept its place");
        assert_eq!(w.players[kb].health, b.players[j].health);
        assert_eq!(w.players[kb].class, b.players[j].class);
        assert_eq!(w.ids[ka], a.ids[j]);
        assert_eq!(w.ids[kb], b.ids[j]);
        assert!(w.is_present(kb));
    }
    // What b's fighters had out, theirs still.
    let moved: Vec<u8> = (0..2).map(|j| m.from_b[j].unwrap() as u8).collect();
    let owned = w
        .effects
        .iter()
        .flatten()
        .filter(|e| moved.contains(&e.owner))
        .count();
    assert_eq!(owned, owned_b, "b's effects came with them");

    // **Played on as they would have been**: b's fighters, with the inputs
    // they would have pressed in b, go exactly where they go in b -- a world
    // apart from a's fighters, at a frame fifty behind b's.
    let mut alone = b.clone();
    let mut together = m.world.clone();
    let kb: Vec<usize> = (0..2).map(|j| m.from_b[j].unwrap()).collect();
    for _ in 0..300 {
        let fb = alone.frame;
        let fa = together.frame;
        alone.advance([hands(0, fb), hands(1, fb)]);
        let mut inputs = [Input::default(); MAX_PLAYERS];
        inputs[0] = hands(0, fa);
        inputs[1] = hands(1, fa);
        inputs[kb[0]] = hands(0, fb);
        inputs[kb[1]] = hands(1, fb);
        together.advance(inputs);
        for (j, &k) in kb.iter().enumerate() {
            assert_eq!(
                on_map(&together, k),
                on_map(&alone, j),
                "b's fighter {j} parted from its own world at b's frame {}",
                alone.frame
            );
        }
    }
    // A pure function: the same two worlds make the same one.
    let again = group::merge(&a, &b).unwrap();
    assert_eq!(again.world.checksum(), m.world.checksum());
}

/// A world of four, the others fed as gone from a group's copy of it.
fn split(w: &World, keep: &[usize]) -> World {
    let mut w = w.clone();
    let f = w.frame;
    let inputs: [Input; MAX_PLAYERS] = std::array::from_fn(|s| {
        if keep.contains(&s) {
            hands(s, f)
        } else {
            Input::default().travelling(Travel::LEFT)
        }
    });
    w.advance(inputs);
    w
}

#[test]
fn a_split_and_a_merge_give_everybody_back_where_they_are() {
    // Four in the town, two of whom go off on their own: the group splits.
    // (In one world a group a hundred metres off walking into a range starts
    // its hunt and moves the frame for everybody -- the coupling groups are
    // for, found by this test's first version.)
    let mut w = World::arrive(
        [
            Class::Champion,
            Class::Elementalist,
            Class::DualMage,
            Class::ShadowReaver,
        ],
        sim::valley::Journey::default(),
        sim::valley::START,
        None,
        4,
    );
    for (i, id) in w.ids.iter_mut().enumerate().take(4) {
        *id = 100 + i as u64;
    }
    play(&mut w, 60, &[0, 1, 2, 3]);
    let mut ours = split(&w, &[0, 1]);
    let mut theirs = split(&w, &[2, 3]);
    // The others gone at once, and nobody who stayed moved by it.
    let mut whole = w.clone();
    play(&mut whole, 1, &[0, 1, 2, 3]);
    for i in 0..2 {
        assert!(ours.is_present(i) && !ours.is_present(i + 2));
        assert!(theirs.is_present(i + 2) && !theirs.is_present(i));
        assert_eq!(on_map(&ours, i), on_map(&whole, i));
        assert_eq!(on_map(&theirs, i + 2), on_map(&whole, i + 2));
    }
    // Apart a while, each its own fight, and back together: everybody where
    // their own group had them, by id.
    let left = Input::default().travelling(Travel::LEFT);
    for _ in 0..200 {
        let f = ours.frame;
        let mut a = [left; MAX_PLAYERS];
        a[0] = hands(0, f);
        a[1] = hands(1, f);
        ours.advance(a);
        let f = theirs.frame;
        let mut b = [left; MAX_PLAYERS];
        b[2] = hands(2, f);
        b[3] = hands(3, f);
        theirs.advance(b);
    }
    let m = group::merge(&ours, &theirs).expect("neither in a hunt");
    for j in 0..2 {
        let k = m.from_a[j].unwrap();
        assert_eq!(on_map(&m.world, k), on_map(&ours, j));
        assert_eq!(m.world.ids[k], 100 + j as u64);
    }
    for j in 2..4 {
        let k = m.from_b[j].unwrap();
        assert_eq!(on_map(&m.world, k), on_map(&theirs, j));
        assert_eq!(m.world.ids[k], 100 + j as u64);
    }
    assert_eq!(
        (0..MAX_PLAYERS).filter(|&i| m.world.is_present(i)).count(),
        4,
        "four again"
    );
}

#[test]
fn two_hunts_two_maps_and_too_many_are_refused() {
    let hunt = World::hunt_of([Class::Champion; 2], sim::species::SpeciesId::RIDGEBACK);
    let other = World::hunt_of([Class::Champion; 2], sim::species::SpeciesId::GNAWERS);
    assert_eq!(
        group::merge(&hunt, &other).err(),
        Some(Refused::BothHunting)
    );
    let town = group(sim::valley::START, [Class::Champion; 2], 1);
    assert_eq!(group::merge(&town, &hunt).err(), Some(Refused::NotOneMap));
    let five = |first: u64| {
        let mut w = World::arrive(
            [Class::Champion; 5],
            sim::valley::Journey::default(),
            sim::valley::START,
            None,
            5,
        );
        for (i, id) in w.ids.iter_mut().enumerate().take(5) {
            *id = first + i as u64;
        }
        w
    };
    assert_eq!(group::merge(&five(1), &five(10)).err(), Some(Refused::Full));
}
