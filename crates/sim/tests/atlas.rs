//! **The atlas** (`sim::atlas`, `docs/design/atlas.md`): many places on one
//! map, and the tiles that let a query read only what is near it.
//!
//! What the tiles promise is that they never miss anything: a query over a
//! box gets every box that overlaps it, in map order, each once. What the
//! valley's layout promises is that no two places sit on each other and every
//! doorway is cut. Walking through the doorways is `tests/valley.rs`.

use sim::arena::{ArenaId, Material, Solid, Terrain};
use sim::atlas::{self, Atlas, DoorPlan, Plan};
use sim::{Fx, V3};

/// A small deterministic stream of numbers, for scattering boxes.
struct Dice(u64);

impl Dice {
    fn roll(&mut self, below: i32) -> i32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % below as u64) as i32
    }
}

fn overlaps(s: &Solid, lo: (Fx, Fx), hi: (Fx, Fx)) -> bool {
    s.min.x.raw() <= hi.0.raw()
        && s.max.x.raw() >= lo.0.raw()
        && s.min.z.raw() <= hi.1.raw()
        && s.max.z.raw() >= lo.1.raw()
}

/// Everything a query should find, by walking every box.
fn brute(atlas: &Atlas, lo: (Fx, Fx), hi: (Fx, Fx)) -> Vec<u32> {
    (0..atlas.solids.len() as u32)
        .filter(|&i| overlaps(&atlas.solids[i as usize], lo, hi))
        .collect()
}

/// A map of a thousand boxes scattered over a square kilometre and a half,
/// some long, some tiny, some on tile lines -- and no places at all.
fn scattered(n: usize) -> Atlas {
    let mut dice = Dice(0x5EED_CAFE);
    let mut extras = Vec::with_capacity(n);
    for _ in 0..n {
        let (x, z) = (dice.roll(1200) - 600, dice.roll(1200) - 600);
        let long = dice.roll(10) == 0;
        let (w, d) = (1 + dice.roll(if long { 90 } else { 6 }), 1 + dice.roll(6));
        let h = 1 + dice.roll(8);
        extras.push(Solid::cm(
            [x * 100, 0, z * 100],
            [(x + w) * 100, h * 100, (z + d) * 100],
            Material::Rock,
        ));
    }
    Atlas::compose(&Plan {
        extras,
        void: Fx::from_int(-100),
        ..Plan::default()
    })
}

#[test]
fn a_query_finds_every_box_that_overlaps_it_in_order_and_once() {
    let atlas = scattered(1000);
    let mut dice = Dice(42);
    for _ in 0..2000 {
        let (x, z) = (dice.roll(1300) - 650, dice.roll(1300) - 650);
        let (w, d) = (dice.roll(40), dice.roll(40));
        let lo = (Fx::from_int(x), Fx::from_int(z));
        let hi = (Fx::from_int(x + w), Fx::from_int(z + d));
        let found: Vec<u32> = atlas.near(lo, hi).collect();
        assert!(
            found.windows(2).all(|p| p[0] < p[1]),
            "out of order, or twice: {found:?}"
        );
        for i in brute(&atlas, lo, hi) {
            assert!(
                found.contains(&i),
                "box {i} overlaps ({lo:?}, {hi:?}) and the tiles missed it"
            );
        }
    }
}

#[test]
fn a_query_wider_than_the_tiles_it_may_merge_still_finds_everything() {
    let atlas = scattered(300);
    let lo = (Fx::from_int(-700), Fx::from_int(-700));
    let hi = (Fx::from_int(700), Fx::from_int(700));
    let found: Vec<u32> = atlas.near(lo, hi).collect();
    assert_eq!(found, brute(&atlas, lo, hi));
}

/// **The cost of a query is what is near it, not how big the map is.** A map
/// a hundred times larger answers a body-sized question by reading as many
/// boxes as a small one does.
#[test]
fn a_bigger_map_costs_a_query_nothing_more() {
    let small = scattered(1000);
    let read = |atlas: &Atlas| {
        let mut dice = Dice(7);
        let mut total = 0;
        for _ in 0..1000 {
            let (x, z) = (dice.roll(1000) - 500, dice.roll(1000) - 500);
            let at = (Fx::from_int(x), Fx::from_int(z));
            total += atlas
                .near(at, (at.0.add(Fx::ONE), at.1.add(Fx::ONE)))
                .count();
        }
        total
    };
    // The same density, ten times as far each way: a hundred thousand boxes.
    let mut dice = Dice(0x5EED_CAFE);
    let mut extras = Vec::with_capacity(100_000);
    for _ in 0..100_000 {
        let (x, z) = (dice.roll(12_000) - 6000, dice.roll(12_000) - 6000);
        let (w, d) = (1 + dice.roll(6), 1 + dice.roll(6));
        extras.push(Solid::cm(
            [x * 100, 0, z * 100],
            [(x + w) * 100, 300, (z + d) * 100],
            Material::Rock,
        ));
    }
    let big = Atlas::compose(&Plan {
        extras,
        void: Fx::from_int(-100),
        ..Plan::default()
    });
    let (a, b) = (read(&small), read(&big));
    assert!(
        b <= a * 2,
        "a body-sized query read {b} boxes on the big map and {a} on the small one"
    );
}

#[test]
fn a_doorway_cuts_what_it_passes_through_and_leaves_the_rest() {
    let wall = Solid::cm([0, 0, -1000], [100, 500, 1000], Material::Rock);
    let atlas = Atlas::compose(&Plan {
        extras: vec![wall],
        doors: vec![DoorPlan {
            a: (ArenaId::HEARTH, 0),
            b: (ArenaId::MOUTH, 0),
            cut: Solid::cm([-100, 50, -200], [200, 300, 200], Material::Ground),
        }],
        void: Fx::from_int(-100),
        ..Plan::default()
    });
    // Either side of the door, the sill under it and the lintel over it.
    assert_eq!(atlas.solids.len(), 4);
    let inside = V3::new(Fx::ratio(1, 2), Fx::from_int(1), Fx::ZERO);
    for s in &atlas.solids {
        let holds = inside.x.raw() > s.min.x.raw()
            && inside.x.raw() < s.max.x.raw()
            && inside.y.raw() > s.min.y.raw()
            && inside.y.raw() < s.max.y.raw()
            && inside.z.raw() > s.min.z.raw()
            && inside.z.raw() < s.max.z.raw();
        assert!(!holds, "the doorway is still solid: {s:?}");
    }
    // And the pieces add up to the wall less the hole.
    let volume = |s: &Solid| {
        let d = s.max.sub(s.min);
        (d.x.raw() as i128) * (d.y.raw() as i128) * (d.z.raw() as i128)
    };
    let hole = Solid::cm([0, 50, -200], [100, 300, 200], Material::Ground);
    assert_eq!(
        atlas.solids.iter().map(volume).sum::<i128>(),
        volume(&wall) - volume(&hole)
    );
}

#[test]
fn no_two_places_of_the_valley_sit_on_each_other() {
    let atlas = atlas::valley();
    assert!(atlas.places.len() >= 18, "{} places", atlas.places.len());
    for (i, p) in atlas.places.iter().enumerate() {
        for q in &atlas.places[i + 1..] {
            let x = p.lo.0.raw() < q.hi.0.raw() && q.lo.0.raw() < p.hi.0.raw();
            let z = p.lo.1.raw() < q.hi.1.raw() && q.lo.1.raw() < p.hi.1.raw();
            assert!(
                !(x && z),
                "{} and {} overlap on the map",
                p.get().name,
                q.get().name
            );
        }
    }
}

#[test]
fn every_seam_with_a_door_is_on_the_map_and_every_place_is_reached() {
    let atlas = atlas::valley();
    for place in sim::valley::all() {
        assert!(
            atlas.placed(place.arena).is_some(),
            "{} is a place of the valley and not on its map",
            place.arena.get().name
        );
    }
    assert_eq!(atlas.doors.len(), sim::valley::layout::JOINTS.len());
}

/// More than an arena alone may have, by far: the cap was per table, and the
/// valley is one map.
#[test]
fn the_valley_is_one_map_of_more_boxes_than_an_arena_may_hold() {
    let atlas = atlas::valley();
    assert!(atlas.solids.len() > 8 * sim::arena::MAX_SOLIDS);
    // And still only a handful in any one tile: what a body's query reads.
    assert!(
        atlas.busiest_tile() <= 24,
        "a tile holds {} boxes",
        atlas.busiest_tile()
    );
}

/// The terrain on a map answers in the place's own coordinates: the floor of
/// the Bank, asked from the Mouth's, is twenty metres up and a reach along.
#[test]
fn a_neighbour_is_ground_in_this_place_s_coordinates() {
    let atlas = atlas::valley();
    let mouth = Terrain::placed(atlas, ArenaId::MOUTH.get());
    let bank = Terrain::placed(atlas, ArenaId::BANK.get());
    let d = atlas
        .placed(ArenaId::BANK)
        .unwrap()
        .at
        .sub(atlas.placed(ArenaId::MOUTH).unwrap().at);
    for (x, z) in [(-60, 10), (-20, -15), (40, 0), (70, 18)] {
        let at = V3::new(Fx::from_int(x), Fx::from_int(60), Fx::from_int(z));
        assert_eq!(
            mouth.ground_under(at.add(d)),
            bank.ground_under(at).add(d.y),
            "the Bank's ground at ({x}, {z}) is somewhere else from the Mouth"
        );
    }
}
