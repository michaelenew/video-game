//! **The High Plane**: a separate world over the valley (exploration 0019
//! §2c, built 2026-10-15). The owner: "sections that lead up and down to full
//! separate planes with different creatures there. The routes up need to be
//! treacherous floating stones or stones held by vines with big gaps between
//! them, assorted smaller stones for precision jumping" -- and, asked whether
//! the plane above should be a table mountain or islands in the sky, "both".
//!
//! - **The plane** is a table mountain: a place keeping its own floor 300 m up,
//!   north of the Bank behind the valley's mountains, which top out near 190.
//!   Its rim holds the land up to its edge, a lip a metre and a half high, and
//!   the land falls away three in one below it: from the valley it is a flat
//!   top over the ridges. Its creatures are its own ([`crate::valley::ranges`]).
//! - **The Sky Road** ([`sky_road`]) is the islands: off the Needle's dome,
//!   353 m up, a chain of turf islands, stones and precision stones over the
//!   town, the Mouth and the mountains, coming down gently to the plane's
//!   south-west corner. Gaps of seven to twenty-six metres, stones from a
//!   metre and a half across to twenty; a cairn on every big island.
//! - **The Hanging Stones** ([`LADDER`], [`SPAN`]) climb from the Bank's road:
//!   a column -- a turn of stones, a lift, **the riser** (two walls rising
//!   together, kicked between), a short climb -- then a span north over the
//!   mountains: a gully (twelve metres of drop between two walls, crossed by
//!   jumping from wall to wall, [`super::gully`]) and a stretch of stones,
//!   **a ferry** caught going by, two **turn-arounds** out of tunnels, a lift
//!   and the last stones (2026-10-22, `crate::drift`). Until 2026-10-16 it
//!   was stones seven metres apart with vines between them, which the owner
//!   found awful to get stuck in; until 2026-10-20, fifty stones each 4.2 m
//!   over the last and a span of identical steps, sized for the Bulwark's
//!   catch; until 2026-10-22, forty hops round a square and a span of
//!   twenty-eight, every metre by hand.
//!
//! **Since 2026-10-20 the Hanging Stones are at the Sky Road's difficulty,
//! and two routes are harder still.** The owner, on the Sky Road: "the only
//! thing resembling challenging. I want more jump sections to be closer to
//! that difficulty, and a few that are more difficult still than that. It's
//! an actually fun challenge because there's variety in the jumps and it
//! forces you to learn class's mobility tools." And the Bulwark is no longer
//! a mover (`crate::class::MOVERS`), so nothing here is sized to it. Every
//! route but the Sky Road is now a table of [`Hop`]s, each declaring what it
//! asks ([`Need`]): a plain jump every mover makes -- most at the edge of
//! the shortest jumpers' catch -- or a class's tool, past every plain jump.
//! A rest with a cairn stands close before every tool hop of the Hanging
//! Stones. Both harder routes are optional:
//!
//! - **The Crown** ([`CROWN`]): off the plane's floor to a turf island fifty
//!   metres over it, the highest place in the valley: fifteen- to
//!   twenty-two-metre hops onto one- and two-metre stones, and two rises of
//!   ten, one rest halfway.
//! - **The High Line** ([`high_line`]): a shortcut across the Sky Road's
//!   middle, six stones a metre or two across, about twenty metres apart.
//!
//! The difficulty of every one of them is a guess, unplayed: sized against
//! `crate::envelope`, which measures three of the classes' tools.
//!
//! The Sky Road's last stretch, over the mountains, is a place of its own
//! (`ArenaId::SKY_ROAD`): the land under it is nobody's, and its cairns have
//! to belong to somewhere.

use super::rim::Rim;
use super::{Arena, ArenaId, Bounds, Mark, Material, Shape, Solid, Spawns};
use crate::drift::Drift;
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::{Kind, Place};

/// The plane's middle on the map and its floor's height (centimetres).
pub const PLANE_AT: [i32; 3] = [42000, 30000, 38000];
/// Half the plane: x and z.
pub const PLANE_HALF: (i32, i32) = (13000, 11000);

pub static PLANE: Arena = Arena {
    id: ArenaId::HIGH_PLANE,
    name: "High Plane",
    creature: None,
    bounds: Bounds::cm((-PLANE_HALF.0, PLANE_HALF.0), (-PLANE_HALF.1, PLANE_HALF.1)),
    floor: Material::Grass,
    regions: &[],
    solids: &PLANE_SOLIDS,
    spawns: Spawns {
        // On the south edge, where the Hanging Stones arrive.
        versus: [
            Mark::cm(-2300, -PLANE_HALF.1 + 600, (-2300, 0)),
            Mark::cm(-1900, -PLANE_HALF.1 + 600, (-1900, 0)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: Some(Rim::new(
        (-PLANE_HALF.0, PLANE_HALF.0),
        (-PLANE_HALF.1, PLANE_HALF.1),
        [0, 0, 0, 0],
        150,
    )),
};

/// What stands on the plane: standing stones and boulders, and a cairn
/// where each route arrives.
const PLANE_SOLIDS: [Solid; 9] = [
    Solid::cm([-12400, -30, -10400], [-12200, 40, -10200], Material::Snow),
    Solid::cm([-2200, -30, -10400], [-2000, 40, -10200], Material::Snow),
    Solid::cm([-300, 0, -300], [300, 900, 300], Material::Stone),
    Solid::cm([2500, 0, 1500], [3100, 700, 2100], Material::Stone),
    Solid::cm([-1800, 0, 2600], [-1200, 1100, 3200], Material::Stone),
    Solid::cm([6000, 0, -4000], [7600, 300, -2600], Material::Rock),
    Solid::cm([-8000, 0, 5000], [-6400, 400, 6800], Material::Rock),
    Solid::cm([9000, 0, 6000], [10200, 250, 7000], Material::Rock),
    Solid::cm([-5000, 0, -7000], [-4000, 200, -6200], Material::Rock),
];

/// How the plane joins the valley: by nothing but the routes up.
pub static PLANE_PLACE: Place = Place {
    arena: ArenaId::HIGH_PLANE,
    kind: Kind::Reach,
    seams: &[],
    vents: &[],
};

/// The Sky Road's last stretch: the mountains between the valley and the
/// plane, under the stones.
pub static SKY_ROAD: Arena = Arena {
    id: ArenaId::SKY_ROAD,
    name: "Sky Road",
    creature: None,
    bounds: Bounds::cm((0, 56000), (14000, PLANE_AT[2] - PLANE_HALF.1)),
    floor: Material::Rock,
    regions: &[],
    solids: &[],
    spawns: Spawns {
        versus: [
            Mark::cm(SPAN_FROM.0, 14500, (SPAN_FROM.0, 20000)),
            Mark::cm(SPAN_FROM.0 + 300, 14500, (SPAN_FROM.0 + 300, 20000)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

pub static SKY_ROAD_PLACE: Place = Place {
    arena: ArenaId::SKY_ROAD,
    kind: Kind::Reach,
    seams: &[],
    vents: &[],
};

// ---------------------------------------------------------------------------
// A route of stones, as a table of hops
// ---------------------------------------------------------------------------

/// **A stone of a route**: its middle on the map, its top, half its side, and
/// whether it stands on the land rather than floats.
#[derive(Clone, Copy, Debug)]
pub struct Stone {
    pub x: i32,
    pub z: i32,
    pub top: i32,
    pub half: i32,
    pub grounded: bool,
}

/// **Which way a hop goes** on the map: `x` is east and `z` north.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    N,
    E,
    S,
    W,
}

impl Way {
    /// The flat axis it goes along.
    pub const fn step(self) -> (i32, i32) {
        match self {
            Way::N => (0, 1),
            Way::E => (1, 0),
            Way::S => (0, -1),
            Way::W => (-1, 0),
        }
    }
}

/// **What a hop asks** (2026-10-20): a plain running jump, landed or caught
/// at the hands, that every class that moves makes (`crate::class::MOVERS`)
/// -- or a class's tool, because no plain jump reaches it, airdodge and all.
/// Declared in the table, so the tests hold each hop to what it was laid out
/// to be (`tests/heights.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Need {
    Jump,
    Tool,
}

/// **A hop of a route**: the next stone's half-side, which way it is, the gap
/// to it edge to edge, how far its top is above the last's (less than
/// nothing: below), and what it asks -- or a gully along a way: a drop
/// between walls ([`super::gully`]) onto a landing with a cairn, crossed by
/// kicking from wall to wall.
#[derive(Clone, Copy, Debug)]
pub enum Hop {
    Stone {
        half: i32,
        gap: i32,
        rise: i32,
        way: Way,
        need: Need,
    },
    Gully {
        way: Way,
    },
    /// **A lift** (2026-10-22, `crate::drift`): two cars side by side
    /// beyond the last stone along `way`, carrying `rise` up to a rest
    /// beyond them, one up while the other comes down.
    Lift {
        way: Way,
        rise: i32,
    },
    /// **A riser** (2026-10-22): two walls across a slot beyond the last
    /// stone along `way`, rising `rise` together; jumped into at their foot
    /// and ridden up by kicking from one to the other, because a slide down
    /// a wall that rises slower than it slides leaves you behind. Off at the
    /// top on to a stone beyond the slot. The owner: "a pair of vertically
    /// moving walls requires you to wall jump between them to ascend".
    Riser {
        way: Way,
        rise: i32,
    },
    /// **A turn-around** (2026-10-22): a tunnel out of the last stone along
    /// `way`, a wall across its mouth's way a gap past it, and its roof the
    /// landing -- out, off the wall and back over the mouth (`super::spire`'s
    /// stack, one tier). The landing here is the roof's mouth end.
    Turn {
        way: Way,
    },
    /// **A ferry** (2026-10-22): a stone that goes by beside the last one
    /// along `way` from out of reach behind it, `run` long and rising
    /// `rise`, caught going past and ridden to a stone a hop over where it
    /// stops.
    Ferry {
        way: Way,
        run: i32,
        rise: i32,
    },
}

impl Hop {
    /// What it asks, of a stone: a plain jump or a tool. Anything else -- a
    /// gully, a lift, a riser, a turn-around, a ferry -- asks what it is.
    pub const fn need(self) -> Option<Need> {
        match self {
            Hop::Stone { need, .. } => Some(need),
            _ => None,
        }
    }
}

use Way::{E, N, S, W};

/// A stone of the ordinary size, three metres across, a plain jump away.
const fn on(way: Way, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half: 150,
        gap,
        rise,
        way,
        need: Need::Jump,
    }
}

/// **A precision stone**: a metre across, room for the feet and no more, a
/// plain jump away.
const fn pin(way: Way, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half: 50,
        gap,
        rise,
        way,
        need: Need::Jump,
    }
}

/// **A rest**: six metres across, a cairn on it, a plain jump away -- put in
/// front of a hard hop, so a miss costs that hop and no more, and wide
/// enough for any class to wind its tool up on.
const fn rest(way: Way, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half: REST,
        gap,
        rise,
        way,
        need: Need::Jump,
    }
}

/// **A hop for a class's tool**, onto a stone `half` across: a gap or a rise
/// no plain jump reaches.
const fn tool(way: Way, half: i32, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half,
        gap,
        rise,
        way,
        need: Need::Tool,
    }
}

/// A gully along `way`.
const fn gully(way: Way) -> Hop {
    Hop::Gully { way }
}

/// A lift along `way`, `rise` up.
const fn lift(way: Way, rise: i32) -> Hop {
    Hop::Lift { way, rise }
}

/// A riser along `way`, `rise` up.
const fn riser(way: Way, rise: i32) -> Hop {
    Hop::Riser { way, rise }
}

/// A turn-around along `way`.
const fn turn(way: Way) -> Hop {
    Hop::Turn { way }
}

/// A ferry along `way`, `run` long, `rise` up.
const fn ferry(way: Way, run: i32, rise: i32) -> Hop {
    Hop::Ferry { way, run, rise }
}

// ---- What moves, and what turns you ---------------------------------------

/// A lift's cars: how long along the way, how wide across (two side by
/// side), and how far from the stones either end; its clock, in frames a
/// metre up and down (about eight metres a second up, twelve down) and the
/// wait at either end.
pub const CAR_LONG: i32 = 300;
pub const CAR_WIDE: i32 = 290;
const CAR_CLEAR: i32 = 10;
const CAR_FRAMES_A_METRE: (i32, i32) = (75, 50);
const CAR_WAIT: u16 = 120;

/// A riser's slot: how long along the way, how wide inside, its walls'
/// thickness, and how far they run under and over the stone it is entered
/// from at their foot; how much higher than that the stone off its top is;
/// its clock -- the wait at the foot, the way up (frames a metre: four
/// metres a second, six at its fastest, past a slide's 3.6), the stay at the
/// top, the way down.
pub const SLOT_LONG: i32 = 500;
pub const SLOT_WIDE: i32 = 260;
const SLOT_WALL: i32 = 100;
pub const SLOT_UNDER: i32 = 150;
pub const SLOT_OVER: i32 = 900;
pub const RISER_EXIT: i32 = 200;
const RISER_FRAMES_A_METRE: i32 = 15;
const RISER_CLOCK: (u16, u16, u16) = (150, 180, 240);

/// A turn-around: the tunnel's length, its width inside, how far its roof is
/// over its floor, how far the roof stops short of the mouth, and how far
/// the wall stands off the mouth -- the Needle's stack's, measured on the
/// bench (`super::bench::TURNS`).
pub const TUNNEL_LONG: i32 = 800;
pub const TUNNEL_WIDE: i32 = 300;
pub const ROOF_UP: i32 = super::spire::TIER_RISE;
const ROOF_SHORT: i32 = 60;
const ROOF_DEEP: i32 = 80;
const TURN_GAP: i32 = super::spire::TURN_GAP;

/// A ferry's deck, how far behind the stone it is caught from it waits (past
/// the Dual mage's plain jump with its airdodge), how
/// far out to the side it goes by, how far over its stop the stone off it
/// is, and its clock: the wait out of reach, the run (frames a metre: ten
/// metres a second, fifteen at its fastest), the stay at the far end, the way
/// back.
pub const FERRY_DECK: i32 = 400;
pub const FERRY_BACK: i32 = 2000;
pub const FERRY_SIDE: i32 = 300;
pub const FERRY_UP: i32 = 300;
const FERRY_FRAMES_A_METRE: i32 = 6;
const FERRY_CLOCK: (u16, u16, u16) = (60, 150, 360);

/// Across a way, to its right, looking down from above with north up:
/// north's right is east, east's is south.
const fn right(way: Way) -> (i32, i32) {
    let (dx, dz) = way.step();
    (dz, -dx)
}

/// **A stone this wide carries a cairn** (half its side, centimetres): a
/// rest, a gully's landing, the Crown. The Sky Road's islands have their own
/// rule ([`build`]).
pub const REST: i32 = 300;

/// How wide a gully's landing is: room to come down from a run of kicks
/// still carrying it, and a checkpoint.
pub const LANDING_HALF: i32 = 400;

/// Does a stone of the Hanging Stones or the Crown carry a cairn?
pub const fn rests(s: &Stone) -> bool {
    s.half >= REST
}

/// **The stones a table of hops lays**, from `from`: each the hop's gap past
/// the last along its way, or a gully's landing past its drop. The last
/// `grounded` stand on the land.
pub const fn lay<const K: usize>(from: Stone, hops: &[Hop; K], grounded: usize) -> [Stone; K] {
    let mut out = [from; K];
    let mut last = from;
    let mut k = 0;
    while k < K {
        let next = match hops[k] {
            Hop::Stone {
                half,
                gap,
                rise,
                way,
                ..
            } => {
                let (dx, dz) = way.step();
                let d = last.half + gap + half;
                Stone {
                    x: last.x + dx * d,
                    z: last.z + dz * d,
                    top: last.top + rise,
                    half,
                    grounded: k + grounded >= K,
                }
            }
            Hop::Gully { way } => {
                let to = super::gully::landing(takeoff(&last, way), way.step());
                let (dx, dz) = way.step();
                Stone {
                    x: to[0] + dx * LANDING_HALF,
                    z: to[2] + dz * LANDING_HALF,
                    top: to[1],
                    half: LANDING_HALF,
                    grounded: false,
                }
            }
            Hop::Lift { way, rise } => {
                let (dx, dz) = way.step();
                let d = last.half + CAR_CLEAR + CAR_LONG + CAR_CLEAR + REST;
                Stone {
                    x: last.x + dx * d,
                    z: last.z + dz * d,
                    top: last.top + rise,
                    half: REST,
                    grounded: false,
                }
            }
            Hop::Riser { way, rise } => {
                let (dx, dz) = way.step();
                let d = last.half + CAR_CLEAR + SLOT_LONG + CAR_CLEAR + 150;
                Stone {
                    x: last.x + dx * d,
                    z: last.z + dz * d,
                    top: last.top + rise + RISER_EXIT,
                    half: 150,
                    grounded: false,
                }
            }
            Hop::Turn { way } => {
                let (dx, dz) = way.step();
                let d = last.half + TUNNEL_LONG - ROOF_SHORT - 150;
                Stone {
                    x: last.x + dx * d,
                    z: last.z + dz * d,
                    top: last.top + ROOF_UP,
                    half: 150,
                    grounded: false,
                }
            }
            Hop::Ferry { way, run, rise } => {
                let (dx, dz) = way.step();
                let (rx, rz) = right(way);
                let side = last.half + FERRY_SIDE + FERRY_DECK / 2;
                let along = run - FERRY_BACK + FERRY_DECK / 2 + CAR_CLEAR * 3 + 150;
                Stone {
                    x: last.x + dx * along + rx * side,
                    z: last.z + dz * along + rz * side,
                    top: ferry_top(&last, run, rise) + rise + FERRY_UP,
                    half: 150,
                    grounded: false,
                }
            }
        };
        out[k] = next;
        last = next;
        k += 1;
    }
    out
}

/// Where a ferry's deck's top is when it waits out of reach: low enough that
/// it goes by the stone it is caught from half a metre under its top -- its
/// height and its way along are eased alike, so it is there when it is a
/// [`FERRY_BACK`] of its run along.
pub const fn ferry_top(last: &Stone, run: i32, rise: i32) -> i32 {
    last.top - 50 - rise * FERRY_BACK / run
}

/// A box on the map from its corners (centimetres).
const fn boxed(min: [i32; 3], max: [i32; 3], m: Material) -> Solid {
    Solid::cm(min, max, m)
}

/// A box `along` a way and `across` it from a point, its heights `y`: the
/// ranges are from the point, along the way's step and its right.
const fn laid(
    at: (i32, i32),
    way: Way,
    along: (i32, i32),
    across: (i32, i32),
    y: (i32, i32),
    m: Material,
) -> Solid {
    let (dx, dz) = way.step();
    let (rx, rz) = right(way);
    let (ax, az) = (
        at.0 + dx * along.0 + rx * across.0,
        at.1 + dz * along.0 + rz * across.0,
    );
    let (bx, bz) = (
        at.0 + dx * along.1 + rx * across.1,
        at.1 + dz * along.1 + rz * across.1,
    );
    let (x0, x1) = if ax < bx { (ax, bx) } else { (bx, ax) };
    let (z0, z1) = if az < bz { (az, bz) } else { (bz, az) };
    boxed([x0, y.0, z0], [x1, y.1, z1], m)
}

/// The middle of a stone's edge on `way`.
const fn edge_of(s: &Stone, way: Way) -> (i32, i32) {
    let (dx, dz) = way.step();
    (s.x + dx * s.half, s.z + dz * s.half)
}

/// A drift from a box at its first stop, `rise` up or `travel` (x, y, z)
/// away, on a clock of (wait, go, stay, back) frames, `phase` into it.
fn moving(solid: Solid, travel: (i32, i32, i32), clock: (u16, u16, u16, u16), phase: u16) -> Drift {
    Drift {
        solid,
        travel: V3::new(cm(travel.0), cm(travel.1), cm(travel.2)),
        wait: clock.0,
        go: clock.1,
        stay: clock.2,
        back: clock.3,
        phase,
    }
}

/// **A lift's two cars**, from the stone before it: side by side across the
/// way, one half a cycle behind the other.
pub fn lift_cars(last: &Stone, way: Way, rise: i32) -> [Drift; 2] {
    let at = edge_of(last, way);
    let up = (rise / 100 * CAR_FRAMES_A_METRE.0) as u16;
    let down = (rise / 100 * CAR_FRAMES_A_METRE.1) as u16;
    let clock = (CAR_WAIT, up, CAR_WAIT, down);
    let period = 2 * CAR_WAIT as u32 + up as u32 + down as u32;
    let car = |k: i32| {
        let across = if k == 0 {
            (-CAR_WIDE, -5)
        } else {
            (5, CAR_WIDE)
        };
        let deck = laid(
            at,
            way,
            (CAR_CLEAR, CAR_CLEAR + CAR_LONG),
            across,
            (last.top - 40, last.top),
            Material::Wood,
        );
        moving(deck, (0, rise, 0), clock, (k as u32 * period / 2) as u16)
    };
    [car(0), car(1)]
}

/// **A riser's two walls**, from the stone before it, at their foot.
pub fn riser_walls(last: &Stone, way: Way, rise: i32) -> [Drift; 2] {
    let at = edge_of(last, way);
    let (wait, stay, back) = RISER_CLOCK;
    let go = (rise / 100 * RISER_FRAMES_A_METRE) as u16;
    let h = SLOT_WIDE / 2;
    let wall = |across: (i32, i32)| {
        let w = laid(
            at,
            way,
            (CAR_CLEAR, CAR_CLEAR + SLOT_LONG),
            across,
            (last.top - SLOT_UNDER, last.top + SLOT_OVER),
            Material::Stone,
        );
        moving(w, (0, rise, 0), (wait, go, stay, back), 0)
    };
    [wall((-h - SLOT_WALL, -h)), wall((h, h + SLOT_WALL))]
}

/// **A turn-around's boxes**, from the stone before it: the tunnel's floor
/// and its two sides, its roof (the landing), and the wall across its
/// mouth's way.
pub fn turn_parts(last: &Stone, way: Way) -> [Solid; 5] {
    let at = edge_of(last, way);
    let h = TUNNEL_WIDE / 2;
    let (floor, roof) = (last.top, last.top + ROOF_UP);
    let side = 100;
    [
        laid(
            at,
            way,
            (0, TUNNEL_LONG),
            (-h, h),
            (floor - 120, floor),
            Material::Rock,
        ),
        laid(
            at,
            way,
            (0, TUNNEL_LONG - ROOF_SHORT),
            (-h, h),
            (roof - ROOF_DEEP, roof),
            Material::Grass,
        ),
        laid(
            at,
            way,
            (0, TUNNEL_LONG),
            (-h - side, -h),
            (floor - 120, roof),
            Material::Rock,
        ),
        laid(
            at,
            way,
            (0, TUNNEL_LONG),
            (h, h + side),
            (floor - 120, roof),
            Material::Rock,
        ),
        laid(
            at,
            way,
            (TUNNEL_LONG + TURN_GAP, TUNNEL_LONG + TURN_GAP + side),
            (-h - 200, h + 200),
            (floor - 400, roof + 300),
            Material::Rock,
        ),
    ]
}

/// **A ferry**, from the stone it is caught from: where it waits, behind and
/// to the right, and its run along the way and up.
pub fn ferry_of(last: &Stone, way: Way, run: i32, rise: i32) -> Drift {
    let (dx, dz) = way.step();
    let (rx, rz) = right(way);
    let side = last.half + FERRY_SIDE + FERRY_DECK / 2;
    let (cx, cz) = (
        last.x - dx * FERRY_BACK + rx * side,
        last.z - dz * FERRY_BACK + rz * side,
    );
    let top = ferry_top(last, run, rise);
    let h = FERRY_DECK / 2;
    let deck = boxed(
        [cx - h, top - 60, cz - h],
        [cx + h, top, cz + h],
        Material::Sand,
    );
    let (wait, stay, back) = FERRY_CLOCK;
    let go = (run / 100 * FERRY_FRAMES_A_METRE) as u16;
    moving(deck, (dx * run, rise, dz * run), (wait, go, stay, back), 0)
}

/// **Every drift a route's hops put down**, from its first stone.
pub fn drifts_of(from: Stone, hops: &[Hop], stones: &[Stone]) -> Vec<Drift> {
    let mut out = Vec::new();
    for (k, h) in hops.iter().enumerate() {
        let last = if k == 0 { from } else { stones[k - 1] };
        match *h {
            Hop::Lift { way, rise } => out.extend(lift_cars(&last, way, rise)),
            Hop::Riser { way, rise } => out.extend(riser_walls(&last, way, rise)),
            Hop::Ferry { way, run, rise } => out.push(ferry_of(&last, way, run, rise)),
            _ => {}
        }
    }
    out
}

/// The middle of a stone's edge on `way`, at its top: where a gully leaves.
const fn takeoff(s: &Stone, way: Way) -> [i32; 3] {
    let (dx, dz) = way.step();
    [s.x + dx * s.half, s.top, s.z + dz * s.half]
}

/// **A route's gullies**: for each, the hop it is (an index into the route's
/// hops and stones) and its two walls, from the stone before it.
pub fn gullies_of(
    from: Stone,
    hops: &'static [Hop],
    stones: &'static [Stone],
) -> impl Iterator<Item = (usize, [Solid; 2])> {
    (0..hops.len()).filter_map(move |k| match hops[k] {
        Hop::Gully { way } => {
            let last = if k == 0 { from } else { stones[k - 1] };
            Some((
                k,
                super::gully::walls(takeoff(&last, way), way.step(), Material::Rock),
            ))
        }
        _ => None,
    })
}

/// **How far apart two stones are**, edge to edge on the flat, in
/// centimetres: what a hop between them crosses.
pub fn gap(a: &Stone, b: &Stone) -> i32 {
    let apart = |p: i32, q: i32| ((p - q).abs() - a.half - b.half).max(0) as i64;
    let (gx, gz) = (apart(a.x, b.x), apart(a.z, b.z));
    crate::math::isqrt_wide((gx * gx + gz * gz) as u128) as i32
}

// ---------------------------------------------------------------------------
// The Hanging Stones
// ---------------------------------------------------------------------------

/// **The column's foot**, by the Bank's road: a stone standing on the land.
pub const FOOT: Stone = Stone {
    x: 38000,
    z: 500,
    top: 4100,
    half: 200,
    grounded: true,
};

/// **The column** (2026-10-20; cut short and broken up 2026-10-22): from
/// the foot by the Bank's road, 41 m, to 203 m. Until 2026-10-22 it was
/// forty hops round a square; the owner, on the long sections: "they're
/// repetitive and don't give enough variety ... Short to medium length,
/// interesting sections that give some variety, then a moving platform, then
/// another section." So, in three:
///
/// - **The first turn**, as it was: plain catches three to five and a half
///   metres out and three to five up, precision stones a metre across, a
///   rest and a high tool hop, to a rest 88 m up.
/// - **A lift**, 62 m, to a rest at 150 m -- and straight off it **the
///   riser**: two walls rising together 24 m, ridden by kicking between
///   them, to a stone two metres over where they stop.
/// - **The top**: a catch, a pin, a rest, a long tool hop of thirteen
///   metres, two pins and a catch, to the column's top.
///
/// Every turn of the old square is gone above the first; the hops by hand are
/// eighteen, where they were forty.
pub const LADDER_HOPS: [Hop; 22] = [
    // The first turn: plain, then a rest and the first high hop.
    on(W, 500, 400),
    pin(W, 450, 300),
    on(W, 550, 300),
    rest(N, 300, 400),
    tool(N, 200, 250, 900),
    on(N, 450, 450),
    pin(E, 500, 200),
    pin(E, 450, 300),
    on(E, 400, 450),
    on(S, 550, 300),
    pin(S, 400, 400),
    rest(W, 300, 300),
    // Up: a lift, and the riser off its head.
    lift(W, 6200),
    riser(W, 2400),
    // The top.
    on(N, 450, 450),
    pin(N, 450, 300),
    rest(E, 300, 400),
    tool(E, 200, 1300, 0),
    pin(N, 400, 350),
    on(N, 400, 450),
    pin(W, 450, 300),
    rest(N, 300, 400),
];

/// The column, its foot first.
pub const LADDER: [Stone; LADDER_HOPS.len()] = lay(FOOT, &LADDER_HOPS, 0);

/// **The span** (2026-10-20; broken up 2026-10-22): north from the
/// column's top over the mountains to the plane's south edge, 203 m to
/// 305 m, in four:
///
/// - **Over the low ground**, as it was: a gully straight off the column's
///   top, a long hop east and up, three precision stones zig-zagging, a
///   rest, seventeen metres and four up, a catch.
/// - **The ferry**: caught going by on your right, sideways, at twelve
///   metres a second, and ridden ninety-five metres up the mountainside,
///   thirty up, to a stone a hop over where it stops.
/// - **Two turn-arounds** across the slope: out of a tunnel east, off a wall
///   across its way and back over its mouth on to its roof, 6.5 m up; a
///   catch north; and the same going west.
/// - **A lift**, 24 m, and **on to the lip**: a pin, five catches and
///   stones, the last two standing on the land the plane's rim holds up.
pub const SPAN_HOPS: [Hop; 20] = [
    // Over the low ground.
    gully(N),
    on(N, 500, 400),
    tool(E, 150, 1400, 200),
    pin(N, 500, 300),
    pin(N, 500, 200),
    pin(W, 550, 100),
    rest(N, 450, 400),
    tool(N, 200, 1700, 400),
    on(N, 400, 500),
    // The ferry, up the mountainside.
    ferry(N, 9500, 3000),
    // The turn-arounds, along the slope: east, and back west.
    turn(E),
    on(N, 400, 300),
    turn(W),
    // A lift, and on to the lip.
    lift(N, 2400),
    pin(N, 400, 300),
    on(N, 500, 300),
    on(N, 500, 300),
    on(N, 500, 300),
    on(N, 450, 200),
    on(N, 400, 100),
];

/// Where the span starts: over the column's last stone.
pub const SPAN_FROM: (i32, i32) = (LADDER[LADDER.len() - 1].x, LADDER[LADDER.len() - 1].z);

/// How many of the span's last stones stand on the land rather than float:
/// the plane's rim holds the land up to its edge, so they are over it.
const SPAN_GROUNDED: usize = 2;

/// The span's stones, in order: each laid from the last by its hop.
pub const SPAN: [Stone; SPAN_HOPS.len()] = lay(LADDER[LADDER.len() - 1], &SPAN_HOPS, SPAN_GROUNDED);

/// The span's gullies: for each, the hop it is (an index into [`SPAN`]) and
/// its walls.
pub fn gullies() -> impl Iterator<Item = (usize, [Solid; 2])> {
    gullies_of(LADDER[LADDER.len() - 1], &SPAN_HOPS, &SPAN)
}

// ---------------------------------------------------------------------------
// Harder than the Sky Road: the Crown and the High Line
// ---------------------------------------------------------------------------

/// **The Crown's foot**: a pillar four metres tall standing on the plane,
/// in its south-west quarter, clear of every range on it.
pub const CROWN_FOOT: Stone = Stone {
    x: PLANE_AT[0] - 9500,
    z: PLANE_AT[2] - 7500,
    top: PLANE_AT[1] + 400,
    half: 200,
    grounded: true,
};

/// **The Crown** (2026-10-20, the owner: "a few that are more difficult
/// still than that"): an optional climb off the High Plane's floor to a turf
/// island fifty metres over it, 354 m up -- the highest place in the valley,
/// a hand over the Needle's dome, with a cairn on it and the whole valley
/// under it. Harder than the Sky Road on every count: the Sky Road's worst,
/// by its table, is eighteen metres and two up onto an island twenty metres
/// across; here the long hops are fifteen to twenty-two metres, level to
/// four up, onto stones one and two metres across, and two high ones ten
/// up, past the Champion's vault. One rest, halfway. The plain hops between
/// are at the very edge of the shortest jumpers' catch.
pub const CROWN_HOPS: [Hop; 14] = [
    pin(E, 550, 350),
    tool(E, 50, 1700, 300),
    pin(N, 600, 200),
    tool(N, 100, 300, 1000),
    rest(W, 400, 300),
    tool(W, 100, 2000, 200),
    pin(W, 600, 200),
    pin(S, 600, 100),
    tool(S, 50, 2200, 0),
    tool(E, 100, 300, 1000),
    pin(E, 550, 300),
    tool(N, 50, 1800, 400),
    on(N, 500, 400),
    tool(W, 600, 1500, 300),
];

/// The Crown's stones, its foot first; the last the island.
pub const CROWN: [Stone; CROWN_HOPS.len()] = lay(CROWN_FOOT, &CROWN_HOPS, 0);

/// **The High Line** (2026-10-20): a harder way across the Sky Road's middle,
/// off its sixth stone (its second island with a cairn) to its fourteenth
/// (its fourth), past the seven between, bowed out thirty metres to the
/// north-west of the road it cuts short. Seven hops, every one a class's
/// tool: nineteen to twenty-two metres each, two of them rising two, onto
/// stones one and two metres across -- where the stretch it skips, by its
/// table, has one gap of twenty-one metres and three of thirteen to
/// eighteen, onto stones one and a half to sixteen metres across, and a run
/// of precision stones a plain jump apart. As the Sky Road's, the gaps are a
/// pattern, stretched or squeezed alike to fit between the two islands and
/// read off the map by the tests: for each stone, its half-side, the gap
/// onto it and how far it is up from the last (less than nothing: down);
/// and then the gap onto the island it joins, whose top is the Sky Road's.
const HIGH_LINE_HOPS: [(i32, i32, i32); 6] = [
    (50, 1750, 200),
    (100, 2050, -300),
    (50, 2300, -200),
    (100, 2150, 200),
    (50, 2200, -300),
    (100, 2150, -300),
];
const HIGH_LINE_LAST: i32 = 1950;

/// Which of the Sky Road's islands the High Line leaves and joins.
pub const HIGH_LINE_FROM: usize = 5;
pub const HIGH_LINE_TO: usize = 13;

/// How far out of the road's line the High Line bows at its middle.
const HIGH_LINE_BOW: i64 = 3000;

/// **The High Line's stones**, from the first after the island it leaves to
/// the last before the island it joins.
pub fn high_line() -> Vec<Stone> {
    let road = sky_road();
    let (a, b) = (road[HIGH_LINE_FROM], road[HIGH_LINE_TO]);
    let (dx, dz) = ((b.x - a.x) as i64, (b.z - a.z) as i64);
    let len = crate::math::isqrt_wide((dx * dx + dz * dz) as u128) as i64;
    // Along the line from middle to middle: the islands' halves, the
    // stones, and the gaps stretched alike to fill the rest.
    let stones: i64 = HIGH_LINE_HOPS.iter().map(|h| 2 * h.0 as i64).sum();
    let gaps: i64 = HIGH_LINE_HOPS.iter().map(|h| h.1 as i64).sum::<i64>() + HIGH_LINE_LAST as i64;
    let room = (len - a.half as i64 - b.half as i64 - stones).max(gaps / 2);
    let mut along = a.half as i64;
    let mut top = a.top;
    let mut out = Vec::new();
    for &(half, gap, rise) in &HIGH_LINE_HOPS {
        along += gap as i64 * room / gaps + half as i64;
        top += rise;
        // Bowed out to the left of the line (north-west), most at the
        // middle: four times t(1 - t), t in thousandths of the way.
        let t = along * 1000 / len;
        let bow = HIGH_LINE_BOW * 4 * t * (1000 - t) / 1_000_000;
        out.push(Stone {
            x: (a.x as i64 + (dx * along - dz * bow) / len) as i32,
            z: (a.z as i64 + (dz * along + dx * bow) / len) as i32,
            top,
            half,
            grounded: false,
        });
        along += half as i64;
    }
    out
}

// ---------------------------------------------------------------------------
// Built
// ---------------------------------------------------------------------------

fn cm(v: i32) -> Fx {
    super::cm(v)
}

/// A stone's solid: floating, a slab under its top; grounded, from the land
/// up. Rock for the small ones, turf for an island.
fn stone(s: &Stone, base: Fx, out: &mut Vec<Solid>) {
    let thick = if s.half >= 500 { 400 } else { 120 + s.half / 3 };
    let m = if s.half >= 500 {
        Material::Grass
    } else {
        Material::Rock
    };
    let bottom = if s.grounded {
        base.sub(Fx::ONE)
    } else {
        cm(s.top - thick)
    };
    out.push(Solid {
        min: V3::new(cm(s.x - s.half), bottom, cm(s.z - s.half)),
        max: V3::new(cm(s.x + s.half), cm(s.top), cm(s.z + s.half)),
        material: m,
        shape: Shape::Box,
    });
}

/// A cairn on a top at `y`.
fn cairn_on(x: i32, z: i32, y: i32) -> Solid {
    Solid::cm(
        [x - 100, y - 30, z - 100],
        [x + 100, y + 40, z + 100],
        Material::Snow,
    )
}

/// **The Sky Road**: the islands and stones from the Needle's dome to the
/// plane, as the pattern of hops it is made of -- a stone's half-side and
/// the gap to the next, edge to edge (centimetres), and the drop (or, less
/// than nothing, the rise) to it.
const SKY_HOPS: [(i32, i32, i32); 22] = [
    (800, 1400, 250),
    (200, 1000, 200),
    (80, 700, 150),
    (80, 700, 150),
    (200, 2000, 300),
    (600, 2400, 250),
    (150, 900, 200),
    (70, 900, -300),
    (70, 900, 200),
    (200, 2600, 400),
    (800, 1600, 250),
    (200, 1800, 300),
    (150, 2200, -200),
    (1000, 1200, 200),
    (80, 800, 150),
    (80, 800, 150),
    (80, 900, 150),
    (200, 2000, 300),
    (600, 1800, 250),
    (150, 1400, 200),
    (200, 1600, 300),
    (1000, 1600, 0),
];

/// The Sky Road's stones, from the dome's rim outward.
pub fn sky_road() -> Vec<Stone> {
    use super::spire;
    let from = (spire::AT.0 + 2950, spire::AT.1 + 3510);
    let to = (
        PLANE_AT[0] - PLANE_HALF.0 + 1200,
        PLANE_AT[2] - PLANE_HALF.1 - 1500,
    );
    let (dx, dz) = ((to.0 - from.0) as i64, (to.1 - from.1) as i64);
    let len = crate::math::isqrt_wide((dx * dx + dz * dz) as u128) as i64;
    // The gaps are stretched or squeezed, all alike, so the last island's
    // far edge is a jump short of the plane: the table says how the hops
    // go, the map how far it is.
    let stones: i64 = SKY_HOPS.iter().map(|h| 2 * h.0 as i64).sum();
    let gaps: i64 = SKY_HOPS[..SKY_HOPS.len() - 1]
        .iter()
        .map(|h| h.1 as i64)
        .sum();
    let room = (len - stones).max(gaps / 2);
    let mut along = 0i64;
    let mut top = spire::TOP - 200;
    let mut out = Vec::new();
    let mut prev_half = 0;
    for (k, &(half, gap, drop)) in SKY_HOPS.iter().enumerate() {
        along += (prev_half + half) as i64;
        let gap = gap as i64 * room / gaps.max(1);
        // A little off the line, side to side, so it is not a ruler.
        let wobble = [0, 400, -300, 500, -500, 200, -200, 600][k % 8] as i64;
        let (ux, uz) = (dx * 1000 / len, dz * 1000 / len);
        let x = from.0 as i64 + ux * along / 1000 - uz * wobble / 1000;
        let z = from.1 as i64 + uz * along / 1000 + ux * wobble / 1000;
        out.push(Stone {
            x: x as i32,
            z: z as i32,
            top,
            half,
            grounded: false,
        });
        along += gap;
        top -= drop;
        prev_half = half;
    }
    out
}

/// **The High Plane's solids, and its routes'**: the plane's own are its
/// table; these are the stones of both routes up, the gullies' walls, the
/// Crown and the High Line, and a cairn on every rest, every gully's
/// landing, the Crown's island and every one of the Sky Road's islands.
pub fn build(height: &dyn Fn(Fx, Fx) -> Fx, out: &mut Vec<Solid>, moving: &mut Vec<Drift>) {
    let hanging = LADDER_HOPS
        .iter()
        .zip(LADDER.iter())
        .chain(SPAN_HOPS.iter().zip(SPAN.iter()))
        // A turn-around's landing is its tunnel's roof, built with it.
        .filter(|(h, _)| !matches!(h, Hop::Turn { .. }))
        .map(|(_, s)| s);
    let crown = core::iter::once(&CROWN_FOOT).chain(CROWN.iter());
    for s in core::iter::once(&FOOT).chain(hanging).chain(crown) {
        stone(s, height(cm(s.x), cm(s.z)), out);
        if rests(s) {
            out.push(cairn_on(s.x, s.z, s.top));
        }
    }
    for (_, walls) in gullies() {
        out.extend(walls);
    }
    for (_, parts) in turns() {
        out.extend(parts);
    }
    moving.extend(drifts());
    for s in sky_road() {
        stone(&s, Fx::ZERO, out);
        if s.half >= 600 {
            out.push(cairn_on(s.x, s.z, s.top));
        }
    }
    for s in high_line() {
        stone(&s, Fx::ZERO, out);
    }
}

/// **The Hanging Stones' turn-arounds**: for each, the hop it is (an index
/// into the column's hops, then on into the span's) and its boxes.
pub fn turns() -> impl Iterator<Item = (usize, [Solid; 5])> {
    let col = LADDER_HOPS.iter().zip(core::iter::once(FOOT).chain(LADDER));
    let span = SPAN_HOPS
        .iter()
        .zip(core::iter::once(LADDER[LADDER.len() - 1]).chain(SPAN));
    col.chain(span)
        .enumerate()
        .filter_map(|(k, (h, last))| match *h {
            Hop::Turn { way } => Some((k, turn_parts(&last, way))),
            _ => None,
        })
}

/// **The Hanging Stones' drifts**: the column's and the span's lifts, the
/// riser's walls and the ferry, in that order.
pub fn drifts() -> Vec<Drift> {
    let mut out = drifts_of(FOOT, &LADDER_HOPS, &LADDER);
    out.extend(drifts_of(LADDER[LADDER.len() - 1], &SPAN_HOPS, &SPAN));
    out
}
