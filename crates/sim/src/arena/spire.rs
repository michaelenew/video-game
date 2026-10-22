//! **The Spire**: Hearth's tower (exploration 0019 §2a, built 2026-10-15).
//! The owner: "an extremely tall narrow tower with a very large inverted dome
//! at the top (like the DBZ Korin tower, but no rounded top) with some ruined
//! buildings atop it and roosting flying monsters on top of the dome".
//!
//! It stands on a plaza just outside the town's south wall -- the square is
//! too full of town for a shaft nine metres across -- on the Wood's road, so
//! the dome's rim hangs over the town's south roofs. A place of its own, so
//! the screen says where you are: **the Needle**, which is also what
//! `--arena needle` starts at (the jump course is already called the Spire).
//!
//! | Part | What it is |
//! | --- | --- |
//! | **The shaft** | Round, pale stone, nine metres across at its foot and under eight at its top, 315 m tall |
//! | **The stair** | Up its south face to 40 m, zig-zagging: stones four metres apart, a catch at the hands for the plainest jump and a landing for the best, a wide landing with a cairn at the top |
//! | **The lifts** | Two, each a pair of cars south of the face (one up while the other comes down): 40 to 120 m, and 170 to 276 m |
//! | **The round** | 120 to 170 m: fifteen floating stones circling the tower nearly twice, about ten metres out, no two hops alike |
//! | **The crown of the shaft** | 276 to 315 m: a shuttle jumped off at its speed, a stack of three tunnels turned round by wall jumps, a ferry caught going by |
//! | **The corkscrew** | Stones jutting from the shaft a quarter turn and eight metres apart: the mechanic band |
//! | **The balconies** | Every forty metres, three boards round the shaft (never across the stair), a cairn on the north one every other time |
//! | **The lantern** | Four columns on the shaft's top carrying the dome, gaps between them to walk through |
//! | **The dome** | Inverted, 90 m across and 35 m deep, its point on the lantern: eight slabs of a sphere, every one a ring round **the chimney** up its stem |
//! | **The chimney** | Nine metres across, 38 m from the shaft's top to the dome's: ledges spiralling up its wall, a catch apart |
//! | **The ruins** | On the dome's top, 353 m up: broken walls, column stumps, an arch, a fallen column, a cistern, a cairn by the chimney's mouth |
//!
//! The roost -- Galewings on the dome, fought there -- is a range's
//! (exploration 0019 §2e), not this table's. Since 2026-10-22 about half
//! the climb is carried (`crate::drift`); until then the stair went the
//! whole height of the shaft, seventy-eight stones alike.

use super::{Arena, ArenaId, Bounds, Mark, Material, Shape, Solid, Spawns};
use crate::drift::Drift;
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::land::{Point, at};
use crate::valley::{Gate, Kind, Place, Seam, Zone};

/// Where the shaft stands on the map, centimetres.
pub const AT: (i32, i32) = (1000, -6000);

pub static ARENA: Arena = Arena {
    id: ArenaId::SPIRE,
    name: "Needle",
    creature: None,
    bounds: Bounds::cm((-4400, 4400), (-14000, -3900)),
    floor: Material::Stone,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::cm(2400, -6000, (1000, -6000)),
            Mark::cm(2400, -6400, (1000, -6000)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

const SOLIDS: [Solid; 0] = [];

/// **The plaza** at the shaft's foot: a way short and wide, on the Wood's
/// road (`super::wood::WAY` runs through it).
pub const PLAZA: [Point; 2] = [at(500, -5800, 0, 1500), at(1500, -6200, 0, 1500)];

/// The shaft: its radius at the foot and at the top, and how tall.
pub const SHAFT: (i32, i32, i32) = (450, 380, 31500);
/// The dome: how far above the shaft's top its point is (the lantern), how
/// deep it is and how far across at its rim (a radius), and the chimney's
/// radius.
pub const LANTERN: i32 = 350;
pub const DEPTH: i32 = 3500;
pub const RIM: i32 = 4500;
pub const CHIMNEY: i32 = 450;
/// The dome's point and its top, above the plaza.
pub const DOME_FROM: i32 = SHAFT.2 + LANTERN;
pub const TOP: i32 = DOME_FROM + DEPTH;
const SLABS: i32 = 8;

/// **The stair** (2026-10-16, where a vine hung until then): the bearing it
/// climbs on (south), in thousandths of a turn, and how far either side of
/// it nothing else is put.
const STAIR_BEARING: i32 = 750;
const STAIR_CLEAR: i32 = 230;
/// How far each of its stones is above the last: past most classes'
/// landing, inside the Bulwark's catch. Every tenth is a landing, forty
/// metres apart, as the balconies are.
pub const STAIR_RISE: i32 = 400;
/// Where the stair ends, at its landing: the first lift's foot.
pub const STAIR_TOP: i32 = 4000;

/// How the Spire joins the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::SPIRE,
    kind: Kind::Reach,
    seams: &SEAMS,
    vents: &[],
};

const SEAMS: [Seam; 2] = [
    // Up the road to the Mouth, toward the town's gate.
    Seam {
        zone: Zone::cm([3900, -300, -5900], [4400, 700, -4900]),
        to: ArenaId::MOUTH,
        at: 3,
        marks: [
            Mark::cm(3500, -5500, (1000, -6000)),
            Mark::cm(3500, -5800, (1000, -6000)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Mouth",
    },
    // On down the road to the Wood.
    Seam {
        zone: Zone::cm([800, -300, -14000], [2000, 700, -13500]),
        to: ArenaId::WOOD,
        at: 0,
        marks: [
            Mark::cm(1200, -11800, (1000, -6000)),
            Mark::cm(1500, -11800, (1000, -6000)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Great Wood",
    },
];

// ---------------------------------------------------------------------------
// Built on the land
// ---------------------------------------------------------------------------

fn cm(v: i32) -> Fx {
    super::cm(v)
}

fn turns(milli: i32) -> Fx {
    Fx::ratio(milli.rem_euclid(1000), 1000)
}

/// Is a bearing clear of the stair's?
fn off_the_stair(bearing: i32) -> bool {
    let d = (bearing - STAIR_BEARING).rem_euclid(1000);
    d > STAIR_CLEAR && d < 1000 - STAIR_CLEAR
}

/// **A step of a climb**, as a block in the shaft's own frame (centimetres
/// from its axis, `x` east and `z` north): its corners across, its top over
/// the plaza, and whether a cairn stands on it.
#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub x: (i32, i32),
    pub z: (i32, i32),
    pub top: i32,
    pub cairn: bool,
}

impl Step {
    /// The block on the map, on a shaft standing on `base`, `deep` from its
    /// top down.
    pub fn solid(&self, base: Fx, deep: i32) -> Solid {
        Solid {
            min: V3::new(
                cm(AT.0 + self.x.0),
                base.add(cm(self.top - deep)),
                cm(AT.1 + self.z.0),
            ),
            max: V3::new(
                cm(AT.0 + self.x.1),
                base.add(cm(self.top)),
                cm(AT.1 + self.z.1),
            ),
            material: Material::Stone,
            shape: Shape::Box,
        }
    }
}

/// How far the stair's stones stand out from the shaft, and how wide they
/// are; how far apart the two sides it zig-zags between are.
const STAIR_OUT: i32 = 250;
const STAIR_WIDE: i32 = 240;
const STAIR_APART: i32 = 140;

/// **The stair's stones**, foot to its landing: every [`STAIR_RISE`] from
/// the plaza to [`STAIR_TOP`], out of its south face, zig-zagging east and
/// west of the south so none is over the one under it -- the crags' way of
/// going up. The last is a landing, standing a metre further out, with a
/// cairn: the foot of the first lift.
///
/// Until 2026-10-22 it went on the whole height of the shaft, seventy-eight
/// stones alike; the owner: "keep the current all-the-way-up pattern for the
/// first little section, then add a moving platform".
pub fn stair() -> impl Iterator<Item = Step> {
    (1..).take_while(|k| STAIR_RISE * k <= STAIR_TOP).map(|k| {
        let y = STAIR_RISE * k;
        let r = shaft_at(y);
        let landing = k % 10 == 0;
        // A landing is wider, away from the middle, for its cairn.
        let wide = STAIR_WIDE + if landing { 120 } else { 0 };
        let (x0, x1) = if k % 2 == 0 {
            (-STAIR_APART / 2 - wide, -STAIR_APART / 2)
        } else {
            (STAIR_APART / 2, STAIR_APART / 2 + wide)
        };
        let out = STAIR_OUT + if landing { 100 } else { 0 };
        Step {
            x: (x0, x1),
            z: (-r - out, -r + 150),
            top: y,
            cairn: landing,
        }
    })
}

/// How far apart the chimney's ledges are, one over the next, and how far
/// one stands out of its wall.
const CHIMNEY_RISE: i32 = 400;
const CHIMNEY_OUT: i32 = 260;

/// **The chimney's ledges**: up its north and south walls by turns, from
/// just over the shaft's top to a step under the dome's -- every one a
/// catch across from the last.
pub fn chimney() -> impl Iterator<Item = Step> {
    (0..)
        .map(|n| (n, SHAFT.2 + 350 + CHIMNEY_RISE * n))
        .take_while(|&(_, y)| y <= TOP - 250)
        .map(|(n, y)| {
            let z = if n % 2 == 0 {
                (-CHIMNEY - 100, -CHIMNEY + CHIMNEY_OUT)
            } else {
                (CHIMNEY - CHIMNEY_OUT, CHIMNEY + 100)
            };
            Step {
                x: (-150, 150),
                z,
                top: y,
                cairn: false,
            }
        })
}

// ---------------------------------------------------------------------------
// The climb since 2026-10-22: short sections, and lifts between them
// ---------------------------------------------------------------------------
//
// The owner: "A lot of the sections are super long, as I requested, but
// they're repetitive ... keep the current all-the-way-up pattern for the
// first little section, then add a moving platform that speeds the player up
// the tower ... After that ... floating stones and slightly more challenging
// jumps ... navigate around the tower ... Then another elevator, then a
// slightly more challenging section, then the top. Overall the user
// shouldn't traverse but maybe half the current height of the tower
// manually." So, over the plaza:
//
// | From | To | What |
// | --- | --- | --- |
// | 0 | 40 m | **the stair**, as it was, to its first landing (a cairn) |
// | 40 | 120 m | **the first lift**: two cars, one up while the other comes down |
// | 120 | 170 m | **the round**: fifteen floating stones circling the tower nearly twice, no two hops alike |
// | 170 | 276 m | **the second lift** |
// | 276 | 315 m | **the crown of the shaft**: a stone to jump off at speed, a stack of tunnels turned round by wall jumps, a fast ferry to catch going by |
// | 315 | 353 m | **the chimney**, as it was, to the ruins |
//
// About 165 m of 353 climbed by hand, where it was all of it.

/// Where every lift's cars run, across (shaft frame, centimetres): south of
/// the face, clear of the stair, the balconies and the corkscrew. Two cars
/// side by side, a car's deck three metres square.
const CAR_Z: (i32, i32) = (-1140, -840);
const CAR_X: [(i32, i32); 2] = [(-400, -100), (-60, 240)];
/// How deep a car's deck is.
const CAR_DEEP: i32 = 40;

/// **A lift**: its foot and its head (centimetres over the plaza), and how
/// long it takes, in frames -- the wait at each end, up, and down.
#[derive(Clone, Copy, Debug)]
pub struct Lift {
    pub foot: i32,
    pub head: i32,
    pub wait: u16,
    pub up: u16,
    pub down: u16,
}

/// **The lifts.** Each car waits two seconds at either end, goes up in nine
/// (about nine metres a second at its fastest) and comes down in six; the
/// second car is half a cycle behind the first, so one is never more than
/// half a cycle away.
pub const LIFTS: [Lift; 2] = [
    Lift {
        foot: STAIR_TOP,
        head: ROUND_FROM,
        wait: 120,
        up: 540,
        down: 360,
    },
    Lift {
        foot: ROUND_TO,
        head: CROWN_FROM,
        wait: 120,
        up: 660,
        down: 420,
    },
];

/// Where the round starts and ends, and the crown of the shaft starts.
pub const ROUND_FROM: i32 = 12000;
pub const ROUND_TO: i32 = 17000;
pub const CROWN_FROM: i32 = 27600;

impl Lift {
    /// Its two cars, on the map, over a plaza at `base`.
    pub fn cars(&self, base: Fx) -> [Drift; 2] {
        let period = 2 * self.wait as u32 + self.up as u32 + self.down as u32;
        let car = |k: usize| {
            let deck = Step {
                x: CAR_X[k],
                z: CAR_Z,
                top: self.foot,
                cairn: false,
            };
            Drift {
                solid: Solid {
                    material: Material::Wood,
                    ..deck.solid(base, CAR_DEEP)
                },
                travel: V3::new(Fx::ZERO, cm(self.head - self.foot), Fx::ZERO),
                wait: self.wait,
                go: self.up,
                stay: self.wait,
                back: self.down,
                phase: (k as u32 * period / 2) as u16,
            }
        };
        [car(0), car(1)]
    }

    /// **What it is stepped off on to at its head**: a platform between the
    /// cars and the shaft, flush with a car at the top, a cairn on it. (Its
    /// foot is the stair's landing, or the round's last stone's.)
    pub fn arrival(&self) -> Step {
        Step {
            x: (-450, 300),
            z: (CAR_Z.1 + 5, -shaft_at(self.head) + 100),
            top: self.head,
            cairn: true,
        }
    }

    /// **What it is stepped on from at its foot**, when that is not the
    /// stair's landing: the same platform at the foot.
    pub fn departure(&self) -> Step {
        Step {
            top: self.foot,
            cairn: true,
            ..self.arrival()
        }
    }
}

/// **The round** (2026-10-22): from the first lift's head, fifteen floating
/// stones circling the tower counterclockwise from above, about ten metres
/// out from its axis -- outside the corkscrew and the balconies -- and
/// nearly twice round, to the second lift's foot. "A little more interest
/// with some floating stones and slightly more challenging jumps ... a little
/// higher, a little further, a little narrower ... navigate around the
/// tower." Where the stair is four metres up and a metre and a half across,
/// every time, these are:
///
/// - **gaps of three to six and a half metres** and rises of two to five, at
///   the edge of the shortest jumpers' catch (the Champion's and the Blood
///   mage's: six metres at four up, five and a half at five);
/// - **five pins** a metre and a quarter across among stones of two and a
///   half;
/// - **one long level-ish hop** (five and a half metres, three up), **one
///   drop** (six and a half metres out and three down, onto a pin), **one
///   catch of five up**, and **a rest** with a cairn halfway.
///
/// Laid out by bearing and distance from the axis so the gaps are what the
/// table below says, edge to edge (`tests/spire.rs` reads them back): each
/// as its middle (shaft frame, centimetres), its top and half its side.
pub const ROUND: [(i32, i32, i32, i32); 15] = [
    (882, -471, 12300, 130),
    (1026, 223, 12550, 60),
    (596, 740, 12950, 130),
    (-213, 1079, 13150, 130),
    (-707, 707, 13600, 60),
    (-1150, 22, 13900, 130),
    (-844, -536, 14350, 130),
    (-299, -1006, 14700, 60),
    (411, -1020, 15000, 250),
    (936, -160, 15400, 130),
    (865, 680, 15100, 60),
    (327, 945, 15600, 130),
    (-318, 1001, 15950, 60),
    (-792, 524, 16350, 130),
    (-1022, -242, 16650, 130),
];

/// The round's stones, as steps: a cairn on the rest.
pub fn round() -> impl Iterator<Item = Step> {
    ROUND.iter().map(|&(x, z, top, half)| Step {
        x: (x - half, x + half),
        z: (z - half, z + half),
        top,
        cairn: half >= 250,
    })
}

/// How deep a floating stone of the round or the crown is: a slab under its
/// top, thicker the wider.
pub const fn floating_deep(half: i32) -> i32 {
    120 + half / 3
}

// ---- The crown of the shaft ----------------------------------------------

/// **The crown of the shaft** (2026-10-22): from the second lift's head,
/// 276 m up, to the shaft's top -- "a slightly more challenging section",
/// and the owner's three ideas for stones that move and walls that turn you,
/// in the order they come:
///
/// 1. **The shuttle**: "you have to time a jump off a stone so its momentum
///    carries you across a gap". Two stones east to a takeoff; off its east
///    edge a stone four metres long runs seven metres east and back, fast
///    (over two-thirds of a second, fifteen metres a second at its
///    fastest); from where it stops it is **twelve and a half metres** to
///    the rest across the gap, past the Dual mage's plain catch (ten). From
///    standing still on it, nobody makes it. Run along it as it goes and
///    jump off its front at its fastest, and you carry its speed over.
/// 2. **The stack**: "turn-around challenges ... jump to a wall then wall
///    jump to change directions then make it to a platform on the other side
///    (... make the coming-from section a tunnel so the player can't just
///    jump over a dividing wall)". Three tunnels stacked, each 6.5 m over
///    the last, the first going north out of the rest: out of a tunnel's
///    mouth, a wall across the way two and a half metres on, kicked off and
///    back over the mouth on to the tunnel's own roof -- which is the next
///    tunnel's floor, going back the way you came. Measured on the bench
///    (`arena::bench::TURNS`): every mover makes a turn of 6.5 m off a wall
///    2.5 m out, and none comes back over that without the wall.
/// 3. **The ferry**: "a particularly fast moving stone you have to time a
///    jump onto at a weird angle, then the stone carries you quickly across a
///    long stretch". From the stack's top, a stone four metres square goes
///    by, south of you and below, at eleven metres a second, rising west
///    across sixty metres of air to the tower's south face. It waits out of
///    reach to the east; you catch it going past, sideways. It stops by
///    the last stone, which a catch puts you on, and the shaft's top is a
///    catch from that.
pub const CROWN_STEPS: [(i32, i32, i32, i32); 2] =
    [(850, -650, 27900, 130), (1500, -650, 28100, 150)];

/// The crown's two stones to the shuttle's takeoff, as steps.
pub fn crown_steps() -> impl Iterator<Item = Step> {
    CROWN_STEPS.iter().map(|&(x, z, top, half)| Step {
        x: (x - half, x + half),
        z: (z - half, z + half),
        top,
        cairn: false,
    })
}

/// The shuttle's deck at its near stop (shaft frame), how far east it runs,
/// and its clock in frames: the wait at each end, out and back.
pub const SHUTTLE: Step = Step {
    x: (1680, 2080),
    z: (-800, -500),
    top: 28100,
    cairn: false,
};
pub const SHUTTLE_RUN: i32 = 1000;
pub const SHUTTLE_CLOCK: (u16, u16, u16, u16) = (90, 70, 60, 90);

/// **The rest across the shuttle's gap**: a cairn on it, and the stack's
/// foot.
pub const REST: Step = Step {
    x: (4280, 4880),
    z: (-950, -350),
    top: 28100,
    cairn: true,
};

/// The stack: how many tunnels, how far each floor is over the last, how
/// far its walls stand off the mouths, and its floors' and walls' thickness.
pub const TIERS: i32 = 3;
pub const TIER_RISE: i32 = 650;
pub const TURN_GAP: i32 = 250;
const TIER_DEEP: i32 = 80;
const TUNNEL_SIDE: i32 = 100;
/// The tunnels across (x, shaft frame) and along (z): north out of the rest.
const TUNNEL_X: (i32, i32) = (4430, 4730);
const TUNNEL_Z: (i32, i32) = (-350, 550);
/// How far a floor stops short of the mouth of the tunnel it roofs, so a
/// jump out of the mouth is not a jump into the roof.
const ROOF_SHORT: i32 = 60;

/// **The stack's floors**, bottom first: the first tunnel's floor (on the
/// rest's north edge), each next one its roof, and the last the top --
/// tunnel `k` goes north out of its mouth when `k` is even and south when
/// odd, so each floor stops short of the end the tunnel under it leaves by.
pub fn stack_floors() -> impl Iterator<Item = Step> {
    (0..=TIERS).map(|k| {
        let z = if k == 0 {
            TUNNEL_Z
        } else if k % 2 == 1 {
            // Over a tunnel going north: short of the north end.
            (TUNNEL_Z.0, TUNNEL_Z.1 - ROOF_SHORT)
        } else {
            (TUNNEL_Z.0 + ROOF_SHORT, TUNNEL_Z.1)
        };
        Step {
            x: TUNNEL_X,
            z,
            top: REST.top + TIER_RISE * k,
            cairn: false,
        }
    })
}

/// The stack's top: where the ferry is caught from.
pub fn stack_top() -> Step {
    stack_floors().last().unwrap_or(REST)
}

/// A box in the shaft's frame: its x and z ranges and the heights over the
/// plaza it runs between (centimetres).
pub type Walled = ((i32, i32), (i32, i32), (i32, i32));

/// **The stack's walls**: its two sides the whole height, and the two walls
/// across the ends it is turned round at -- north, off the first and third
/// mouths; south, off the second, hung over the rest so the rest is walked
/// under it. In the shaft's frame, as (x, z) ranges and the heights over the
/// plaza they run between.
pub fn stack_walls() -> [Walled; 4] {
    let top = stack_top().top;
    let (x0, x1) = TUNNEL_X;
    let (z0, z1) = TUNNEL_Z;
    let side = TUNNEL_SIDE;
    [
        ((x0 - side, x0), (z0, z1), (REST.top - 200, top)),
        ((x1, x1 + side), (z0, z1), (REST.top - 200, top)),
        (
            (x0 - side, x1 + side),
            (z1 + TURN_GAP, z1 + TURN_GAP + side),
            (REST.top - 400, top + 250),
        ),
        (
            (x0 - side, x1 + side),
            (z0 - TURN_GAP - side, z0 - TURN_GAP),
            (REST.top + 250, REST.top + 2 * TIER_RISE + 300),
        ),
    ]
}

/// **The ferry**: its deck where it waits out of reach, east of the stack
/// (shaft frame); how far it runs, west and up, to its stop by the last
/// stone; and its clock in frames -- a wait at the east end, the run, a
/// longer wait at the tower to step off, the way back.
pub const FERRY: Step = Step {
    x: (6000, 6400),
    z: (-1150, -750),
    top: 29600,
    cairn: false,
};
pub const FERRY_RUN: (i32, i32) = (-6200, 1250);
pub const FERRY_CLOCK: (u16, u16, u16, u16) = (60, 420, 150, 300);

/// **The last stone**: out of the south face, three metres over the
/// ferry's stop, and a catch under the shaft's top.
pub fn last_stone() -> Step {
    let top = SHAFT.2 - 350;
    let r = shaft_at(top);
    Step {
        x: (-120, 120),
        z: (-r - 250, -r + 150),
        top,
        cairn: false,
    }
}

/// A drift of the crown, from a step at its first stop, run by `travel`
/// (x, y, z, centimetres) on its clock: the wait there, the way, the stay
/// at the far end and the way back, in frames.
fn drift_of(
    deck: Step,
    base: Fx,
    deep: i32,
    travel: (i32, i32, i32),
    clock: (u16, u16, u16, u16),
) -> Drift {
    Drift {
        solid: deck.solid(base, deep),
        travel: V3::new(cm(travel.0), cm(travel.1), cm(travel.2)),
        wait: clock.0,
        go: clock.1,
        stay: clock.2,
        back: clock.3,
        phase: 0,
    }
}

/// **The shuttle**, on the map, over a plaza at `base`.
pub fn shuttle(base: Fx) -> Drift {
    drift_of(SHUTTLE, base, 60, (SHUTTLE_RUN, 0, 0), SHUTTLE_CLOCK)
}

/// **The ferry**, on the map, over a plaza at `base`: a second out east,
/// seven seconds across, two and a half at the tower to step off, five back.
pub fn ferry(base: Fx) -> Drift {
    drift_of(FERRY, base, 60, (FERRY_RUN.0, FERRY_RUN.1, 0), FERRY_CLOCK)
}

/// **Every drift of the Needle**, on the map, over a plaza at `base`: both
/// lifts' cars, the shuttle and the ferry.
pub fn drifts(base: Fx) -> Vec<Drift> {
    let mut out = Vec::new();
    for lift in &LIFTS {
        out.extend(lift.cars(base));
    }
    out.push(shuttle(base));
    out.push(ferry(base));
    out
}

/// Where a point a distance out from the axis on a bearing is on the map.
pub fn out_at(bearing: i32, dist: i32) -> (Fx, Fx) {
    let d = V3::from_turns(turns(bearing));
    (
        cm(AT.0).add(d.x.mul(cm(dist))),
        cm(AT.1).add(d.z.mul(cm(dist))),
    )
}

/// The shaft's radius at a height above the plaza.
const fn shaft_at(y: i32) -> i32 {
    SHAFT.0 - (SHAFT.0 - SHAFT.1) * y / SHAFT.2
}

/// The dome's radius at a height above its point: a slice of a sphere
/// through the point and the rim.
fn dome_at(h: i32) -> i32 {
    let r = (RIM as i64 * RIM as i64 + DEPTH as i64 * DEPTH as i64) / (2 * DEPTH as i64);
    let k = r - h as i64;
    crate::math::isqrt_wide((r * r - k * k).max(0) as u128) as i32
}

/// A board from `inner` to `outer` out from the axis along a bearing, its top
/// at `top`, `wide` across, `thick` deep.
#[allow(clippy::too_many_arguments)]
fn board(
    base: Fx,
    bearing: i32,
    inner: i32,
    outer: i32,
    top: i32,
    wide: i32,
    thick: i32,
    m: Material,
) -> Solid {
    let d = V3::from_turns(turns(bearing));
    let mid = cm((inner + outer) / 2);
    Solid::turned(
        cm(AT.0).add(d.x.mul(mid)),
        cm(AT.1).add(d.z.mul(mid)),
        base.add(cm(top - thick)),
        base.add(cm(top)),
        cm((outer - inner) / 2),
        cm(wide / 2),
        turns(bearing),
        m,
    )
}

/// A cairn: a slab of snow two metres across on a top at `y`.
fn cairn_on(x: Fx, z: Fx, y: Fx) -> Solid {
    Solid {
        min: V3::new(x.sub(Fx::ONE), y.sub(Fx::ratio(3, 10)), z.sub(Fx::ONE)),
        max: V3::new(x.add(Fx::ONE), y.add(Fx::ratio(2, 5)), z.add(Fx::ONE)),
        material: Material::Snow,
        shape: Shape::Box,
    }
}

/// **The ruins on the dome's top**: what is left of a village 353 metres up.
/// Each as a bearing and a distance from the axis (centimetres, thousandths
/// of a turn), and what it is.
#[derive(Clone, Copy)]
enum Ruin {
    /// A broken wall: along a bearing for a length, so high.
    Wall { length: i32, high: i32 },
    /// A column's stump, so high.
    Stump { high: i32 },
    /// A column fallen along a bearing, so long.
    Fallen { long: i32 },
    /// Two piers and a lintel across them, along a bearing.
    Arch,
    /// A cistern's round wall.
    Cistern,
}

const RUINS: [(i32, i32, Ruin); 18] = [
    (
        50,
        2600,
        Ruin::Wall {
            length: 700,
            high: 380,
        },
    ),
    (
        120,
        3300,
        Ruin::Wall {
            length: 500,
            high: 220,
        },
    ),
    (180, 2200, Ruin::Stump { high: 420 }),
    (230, 3700, Ruin::Arch),
    (
        300,
        2900,
        Ruin::Wall {
            length: 900,
            high: 300,
        },
    ),
    (360, 1800, Ruin::Stump { high: 180 }),
    (410, 3500, Ruin::Fallen { long: 650 }),
    (480, 2500, Ruin::Cistern),
    (
        540,
        3800,
        Ruin::Wall {
            length: 600,
            high: 450,
        },
    ),
    (600, 2000, Ruin::Stump { high: 320 }),
    (
        640,
        3000,
        Ruin::Wall {
            length: 400,
            high: 150,
        },
    ),
    (700, 3600, Ruin::Stump { high: 500 }),
    (
        800,
        2700,
        Ruin::Wall {
            length: 800,
            high: 260,
        },
    ),
    (850, 3900, Ruin::Fallen { long: 500 }),
    (900, 1900, Ruin::Stump { high: 250 }),
    (950, 3300, Ruin::Arch),
    (20, 1500, Ruin::Stump { high: 140 }),
    (
        680,
        1500,
        Ruin::Wall {
            length: 300,
            high: 120,
        },
    ),
];

fn ruin(base: Fx, bearing: i32, dist: i32, r: Ruin, out: &mut Vec<Solid>) {
    let floor = base.add(cm(TOP));
    let (x, z) = out_at(bearing, dist);
    match r {
        Ruin::Wall { length, high } => {
            // Square to the bearing: a wall across it.
            out.push(Solid::turned(
                x,
                z,
                floor.sub(Fx::ratio(1, 5)),
                floor.add(cm(high)),
                cm(length / 2),
                cm(40),
                turns(bearing + 250),
                Material::Stone,
            ));
        }
        Ruin::Stump { high } => {
            out.push(Solid::round(
                x,
                z,
                floor.sub(Fx::ratio(1, 5)),
                floor.add(cm(high)),
                cm(75),
                cm(60),
                Material::Stone,
            ));
        }
        Ruin::Fallen { long } => {
            out.push(Solid::turned(
                x,
                z,
                floor,
                floor.add(cm(120)),
                cm(long / 2),
                cm(60),
                turns(bearing + 130),
                Material::Stone,
            ));
        }
        Ruin::Arch => {
            let across = V3::from_turns(turns(bearing + 250));
            for side in [-1, 1] {
                let off = cm(200 * side);
                out.push(Solid::round(
                    x.add(across.x.mul(off)),
                    z.add(across.z.mul(off)),
                    floor.sub(Fx::ratio(1, 5)),
                    floor.add(cm(520)),
                    cm(70),
                    cm(60),
                    Material::Stone,
                ));
            }
            out.push(Solid::turned(
                x,
                z,
                floor.add(cm(520)),
                floor.add(cm(620)),
                cm(290),
                cm(60),
                turns(bearing + 250),
                Material::Stone,
            ));
        }
        Ruin::Cistern => {
            out.push(Solid::ring(
                x,
                z,
                floor.sub(Fx::ratio(1, 5)),
                floor.add(cm(130)),
                (cm(330), cm(320)),
                cm(260),
                Material::Stone,
            ));
        }
    }
}

/// **The Spire's solids**, standing on the land (`height` is the ground at
/// a map point).
pub fn build(height: &dyn Fn(Fx, Fx) -> Fx, out: &mut Vec<Solid>, moving: &mut Vec<Drift>) {
    let (x, z) = (cm(AT.0), cm(AT.1));
    let base = height(x, z);
    moving.extend(drifts(base));
    // The shaft.
    out.push(Solid::round(
        x,
        z,
        base.sub(Fx::from_int(2)),
        base.add(cm(SHAFT.2)),
        cm(SHAFT.0),
        cm(SHAFT.1),
        Material::Stone,
    ));
    // The corkscrew, and the balconies every forty metres.
    let mut bearing = 0;
    let mut y = 1200;
    while y < SHAFT.2 - 600 {
        let balcony = 4000;
        let near_balcony = (y % balcony) < 300 || (y % balcony) > balcony - 300;
        if off_the_stair(bearing) && !near_balcony {
            let r = shaft_at(y);
            out.push(board(
                base,
                bearing,
                r - 100,
                r + 320,
                y,
                260,
                90,
                Material::Stone,
            ));
        }
        bearing += 280;
        y += 800;
    }
    let mut k = 0;
    let mut b = 4000;
    while b < SHAFT.2 - 600 {
        let r = shaft_at(b);
        for bearing in [0, 250, 500] {
            out.push(board(
                base,
                bearing,
                r - 100,
                r + 280,
                b,
                480,
                100,
                Material::Stone,
            ));
        }
        if k % 2 == 1 {
            let (cx, cz) = out_at(250, r + 160);
            out.push(cairn_on(cx, cz, base.add(cm(b))));
        }
        k += 1;
        b += 4000;
    }

    // The lantern: four columns on the shaft's top, into the dome's stem.
    for c in 0..4 {
        let (cx, cz) = out_at(125 + 250 * c, CHIMNEY + 30);
        out.push(Solid::round(
            cx,
            cz,
            base.add(cm(SHAFT.2 - 50)),
            base.add(cm(DOME_FROM + 250)),
            cm(130),
            cm(120),
            Material::Stone,
        ));
    }

    // The dome: eight slabs of a sphere, each a ring round the chimney.
    for s in 0..SLABS {
        let (h0, h1) = (DEPTH * s / SLABS, DEPTH * (s + 1) / SLABS);
        let r0 = dome_at(h0).max(CHIMNEY + 250);
        let r1 = dome_at(h1).max(CHIMNEY + 300);
        out.push(Solid::ring(
            x,
            z,
            base.add(cm(DOME_FROM + h0)),
            base.add(cm(DOME_FROM + h1)),
            (cm(r0), cm(r1)),
            cm(CHIMNEY),
            Material::Stone,
        ));
    }

    // The stair up the south face, and the chimney's ledges.
    for s in stair() {
        // Deep, so a jump rising beside one meets its face rather than
        // going up under it.
        out.push(s.solid(base, 200));
        if s.cairn {
            // At the end away from the middle: the climb comes up the other.
            let cx = if s.x.0 < 0 { s.x.0 + 110 } else { s.x.1 - 110 };
            let (cx, cz) = (AT.0 + cx, AT.1 + s.z.0 + 150);
            out.push(cairn_on(cm(cx), cm(cz), base.add(cm(s.top))));
        }
    }
    for s in chimney() {
        out.push(s.solid(base, 150));
    }
    // The lifts' platforms, the round, and the crown of the shaft.
    let mut stones: Vec<(Step, i32)> = Vec::new();
    stones.push((LIFTS[0].arrival(), 100));
    stones.push((LIFTS[1].departure(), 100));
    stones.push((LIFTS[1].arrival(), 100));
    for (k, s) in round().enumerate() {
        stones.push((s, floating_deep(ROUND[k].3)));
    }
    for (k, s) in crown_steps().enumerate() {
        stones.push((s, floating_deep(CROWN_STEPS[k].3)));
    }
    stones.push((REST, 150));
    for s in stack_floors() {
        stones.push((s, TIER_DEEP));
    }
    stones.push((last_stone(), 200));
    for (s, deep) in stones {
        out.push(s.solid(base, deep));
        if s.cairn {
            let (cx, cz) = (AT.0 + (s.x.0 + s.x.1) / 2, AT.1 + (s.z.0 + s.z.1) / 2);
            out.push(cairn_on(cm(cx), cm(cz), base.add(cm(s.top))));
        }
    }
    for ((x0, x1), (z0, z1), (y0, y1)) in stack_walls() {
        out.push(Solid::cm(
            [AT.0 + x0, 0, AT.1 + z0],
            [AT.0 + x1, 0, AT.1 + z1],
            Material::Stone,
        ));
        let last = out.len() - 1;
        out[last].min.y = base.add(cm(y0));
        out[last].max.y = base.add(cm(y1));
    }
    // **The lip stone**: a block set into the chimney's mouth over the last
    // ledge, its top the dome's. A ring's edge is not caught from inside its
    // hole; a block's is, so the plainest jump hangs from it and climbs out.
    if let Some(last) = chimney().last() {
        let z = if last.z.0 < 0 {
            (-CHIMNEY - 150, -CHIMNEY + 10)
        } else {
            (CHIMNEY - 10, CHIMNEY + 150)
        };
        let lip = Step {
            x: (-150, 150),
            z,
            top: TOP,
            cairn: false,
        };
        out.push(lip.solid(base, 150));
    }

    // The ruins on the top, and a cairn by the chimney's mouth.
    for &(bearing, dist, r) in &RUINS {
        ruin(base, bearing, dist, r, out);
    }
    let (cx, cz) = out_at(250, CHIMNEY + 250);
    out.push(cairn_on(cx, cz, base.add(cm(TOP))));
}
