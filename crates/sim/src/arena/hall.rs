//! **The tutorial halls** (`crate::tutorial`): one per class, a room after
//! another down `+x`.
//!
//! Every room is [`ROOM_CM`](crate::tutorial::ROOM_CM) long and the hall
//! eighteen metres wide, with a doorway between each two -- a post either
//! side, nine metres open between them -- so walking into a room is plainly
//! walking into it. What a room has besides its floor is its
//! [`Bay`](crate::tutorial::Bay), read off the class's table, so the hall is
//! built from the tutorial and the two cannot disagree: a shelf to get up
//! onto, a wall to kick off, a gap to cross, each with a way round at the
//! north side so nobody is ever stuck below one. The advanced rooms are
//! floored in dressed stone, so the hall says where the basics end.
//!
//! Its edge is a **rim** (`arena::rim`): the land rises too steep to climb off
//! both long sides and both ends, as it does round every fight.
//!
//! Level data, as an arena's table always is: judged by walking it, and by
//! `tests/tutorial.rs` doing every room's task in it.

use super::rim::Rim;
use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use crate::tutorial::{self, Bay, Dummy, ROOM_CM, Room};

/// Half the hall's width.
const HALF: i32 = 900;
/// The doorway's posts: how thick, how tall, and where they stop either side
/// of the middle.
const POST: i32 = 40;
const POST_HIGH: i32 = 300;
const OPENING: i32 = 450;
/// The end walls.
const END: i32 = 80;
const END_HIGH: i32 = 150;
/// The way round a shelf, a wall or a gap, along the north side: from here
/// to the hall's edge, three and a half metres of it inside the doorways.
const LANE: i32 = 100;
/// How high the rim's bank climbs.
const BANK: i32 = 700;
/// How far into a room the dummy stands, and where it stands aside.
const DUMMY_AT: i32 = 1000;
const ASIDE_Z: i32 = 700;

/// How many solids a hall of these rooms has.
pub const fn count(rooms: &[Room]) -> usize {
    let mut n = 2 + 2 * (rooms.len() - 1);
    let mut k = 0;
    while k < rooms.len() {
        n += match rooms[k].bay {
            Bay::Flat => 0,
            Bay::Shelf(_) | Bay::Wall(_) => 1,
            Bay::Gap(..) => 2,
        };
        k += 1;
    }
    n
}

/// How many advanced rooms: each has a stone floor.
pub const fn advanced(rooms: &[Room]) -> usize {
    let mut n = 0;
    let mut k = 0;
    while k < rooms.len() {
        if rooms[k].advanced {
            n += 1;
        }
        k += 1;
    }
    n
}

/// The far end of the hall.
pub const fn length(rooms: &[Room]) -> i32 {
    ROOM_CM * rooms.len() as i32
}

/// **The hall's boxes**: its two ends, the posts of every doorway, and what
/// each room has in it.
pub const fn solids<const N: usize>(rooms: &[Room]) -> [Solid; N] {
    let mut out = [Solid::cm([0, 0, 0], [1, 1, 1], Material::Stone); N];
    let far = length(rooms);
    out[0] = Solid::cm([-END, 0, -HALF], [0, END_HIGH, HALF], Material::Stone);
    out[1] = Solid::cm(
        [far, 0, -HALF],
        [far + END, END_HIGH, HALF],
        Material::Stone,
    );
    let mut n = 2;
    let mut k = 0;
    while k < rooms.len() {
        let x0 = ROOM_CM * k as i32;
        if k > 0 {
            out[n] = Solid::cm(
                [x0 - POST, 0, -HALF],
                [x0 + POST, POST_HIGH, -OPENING],
                Material::Stone,
            );
            out[n + 1] = Solid::cm(
                [x0 - POST, 0, OPENING],
                [x0 + POST, POST_HIGH, HALF],
                Material::Stone,
            );
            n += 2;
        }
        match rooms[k].bay {
            Bay::Flat => {}
            // Across the far third, short of the next doorway's posts.
            Bay::Shelf(h) => {
                out[n] = Solid::cm(
                    [x0 + 1600, 0, -HALF],
                    [x0 + ROOM_CM - 2 * POST, h, LANE],
                    Material::Rock,
                );
                n += 1;
            }
            // Across the room past the dummy, half a metre thick.
            Bay::Wall(h) => {
                out[n] = Solid::cm([x0 + 1700, 0, -HALF], [x0 + 1760, h, LANE], Material::Stone);
                n += 1;
            }
            // Two shelves either side of the gap, low enough to jump onto.
            Bay::Gap(h, w) => {
                let mid = x0 + ROOM_CM / 2;
                out[n] = Solid::cm([x0 + 200, 0, -HALF], [mid - w / 2, h, LANE], Material::Rock);
                out[n + 1] = Solid::cm(
                    [mid + w / 2, 0, -HALF],
                    [x0 + ROOM_CM - 2 * POST, h, LANE],
                    Material::Rock,
                );
                n += 2;
            }
        }
        k += 1;
    }
    assert!(n == N, "a hall's solids are counted by `count`");
    out
}

/// **The advanced rooms' floors**, in dressed stone.
pub const fn regions<const N: usize>(rooms: &[Room]) -> [Region; N] {
    let mut out = [Region {
        area: Area::rect_cm((0, 1), (0, 1)),
        material: Material::Stone,
    }; N];
    let mut n = 0;
    let mut k = 0;
    while k < rooms.len() {
        if rooms[k].advanced {
            let x0 = ROOM_CM * k as i32;
            out[n] = Region {
                area: Area::rect_cm((x0, x0 + ROOM_CM), (-HALF, HALF)),
                material: Material::Stone,
            };
            n += 1;
        }
        k += 1;
    }
    assert!(n == N, "a hall's regions are counted by `advanced`");
    out
}

/// **Where the dummy stands in room `k`**: a third of the way in, facing
/// the door you came through -- or aside, by the north lane.
pub const fn dummy_mark(rooms: &[Room], k: usize) -> Mark {
    let x0 = ROOM_CM * k as i32;
    match rooms[k].dummy {
        Dummy::Stands | Dummy::Swings => Mark::cm(x0 + DUMMY_AT, 0, (x0, 0)),
        Dummy::Aside => Mark::cm(x0 + DUMMY_AT, ASIDE_Z, (x0, ASIDE_Z)),
    }
}

/// A hall, from its class's rooms.
pub const fn hall(
    id: ArenaId,
    name: &'static str,
    rooms: &'static [Room],
    solids: &'static [Solid],
    regions: &'static [Region],
) -> Arena {
    let far = length(rooms);
    Arena {
        id,
        name,
        creature: None,
        bounds: Bounds::cm((-END, far + END), (-HALF, HALF)),
        floor: Material::Ground,
        regions,
        solids,
        spawns: Spawns {
            versus: [Mark::cm(300, 0, (ROOM_CM, 0)), dummy_mark(rooms, 0)],
            hunt: None,
        },
        sites: &[],
        rim: Some(Rim::new(
            (-END, far + END),
            (-HALF, HALF),
            [0, 0, END_HIGH, END_HIGH],
            BANK,
        )),
    }
}

macro_rules! hall {
    ($arena:ident, $solids:ident, $regions:ident, $id:expr, $name:expr, $rooms:path) => {
        const $solids: [Solid; count(&$rooms)] = solids(&$rooms);
        const $regions: [Region; advanced(&$rooms)] = regions(&$rooms);
        pub static $arena: Arena = hall($id, $name, &$rooms, &$solids, &$regions);
    };
}

hall!(
    CHAMPION,
    CHAMPION_SOLIDS,
    CHAMPION_REGIONS,
    ArenaId::HALL_CHAMPION,
    "Champion tutorial",
    tutorial::champion::ROOMS
);
hall!(
    REAVER,
    REAVER_SOLIDS,
    REAVER_REGIONS,
    ArenaId::HALL_REAVER,
    "Reaver tutorial",
    tutorial::reaver::ROOMS
);
hall!(
    ELEMENTALIST,
    ELEMENTALIST_SOLIDS,
    ELEMENTALIST_REGIONS,
    ArenaId::HALL_ELEMENTALIST,
    "Elementalist tutorial",
    tutorial::elementalist::ROOMS
);
hall!(
    BLOOD,
    BLOOD_SOLIDS,
    BLOOD_REGIONS,
    ArenaId::HALL_BLOOD,
    "Blood mage tutorial",
    tutorial::blood::ROOMS
);
hall!(
    DUAL,
    DUAL_SOLIDS,
    DUAL_REGIONS,
    ArenaId::HALL_DUAL,
    "Dual mage tutorial",
    tutorial::dual::ROOMS
);
