//! **Versus modes** (2026-10-21): what a stage's rules are beyond "last one
//! standing". See `docs/design/versus.md`.
//!
//! A stage is an arena built for fighting each other. It names its rules
//! here, in a [`Stage`], the way a course names its pit in `crate::course`:
//!
//! - **A pit** -- a height below which a fighter is out. The floating
//!   stages hang over open air, and a body knocked off the edge falls
//!   through this and is knocked out, exactly as if their health had run
//!   out. Smash's ring-out, which is what makes spacing near an edge matter.
//! - **A hill** -- King of the Hill, Team Fortress 2's way: one capture
//!   point in the middle, **locked** for a while at the start so both sides
//!   can reach it; standing on it alone **captures** it, faster with a
//!   team-mate; both sides on it is **contested** and nothing moves; once a
//!   side owns it **their clock runs down** while they hold it, and it is
//!   theirs until the other side captures it back. The side whose clock
//!   reaches zero wins -- unless somebody of the other side is standing on
//!   the point, which is **overtime**, and lasts until the last of them
//!   steps off (or is knocked off) or they take it. The dead stand back up
//!   at their own side's start after a wait.
//!
//! **Sides are seats taken alternately**: seats 0 and 2 against 1 and 3
//! ([`side`]), so a duel is seat 0 against seat 1 as it always was. On a
//! hill the two of a side cannot hurt each other (`World::foes`); on a
//! duel stage it is everybody against everybody, as versus always has been.
//!
//! Every number that is a feel -- how long a capture takes, how long a clock
//! runs -- is the Oven's ("Match" family). Where the point and the pit are
//! is level data, in each stage's file under `crate::arena`.

use crate::arena::{ArenaId, cm};
use crate::fixed::Fx;
use crate::math::V3;
use crate::state::MAX_PLAYERS;
use crate::tuning as t;

/// The two sides of a team game. Seats alternate between them.
pub const SIDES: usize = 2;

/// Nobody: an owner or a capper that is neither side.
pub const NOBODY: u8 = u8::MAX;

/// Which side seat `i` plays on: seats alternate, so a duel is seat 0
/// against seat 1 and four fighters are two against two.
pub const fn side(i: usize) -> u8 {
    (i % SIDES) as u8
}

/// **A capture point**: a disc on the floor of a stage, and **the column of
/// air over it** (2026-10-21, from play: "jump around on the point without
/// losing capture, and hold it even in the air"). A body counts as on the
/// point with its feet anywhere over the disc from a little under its
/// surface up to `King of the hill, the point counts up to` over it -- so a
/// hop, a launch or an aerial exchange on the point keeps capturing -- but
/// never as high as `room`, the map's own headroom: somebody standing on a
/// bridge over the point is not on it, however the knob is tuned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hill {
    /// The middle of the disc, on the surface it lies on.
    pub at: V3,
    pub radius: Fx,
    /// Feet this far under the surface still count (a step down off it).
    pub below: Fx,
    /// The most air over the surface that may count: under whatever stands
    /// over the point.
    pub room: Fx,
}

impl Hill {
    /// From centimetres: a disc of `radius` round (x, z) on a surface at
    /// height `y`, counting feet from `below` under it to `room` over it at
    /// the most.
    pub const fn cm(x: i32, y: i32, z: i32, radius: i32, below: i32, room: i32) -> Hill {
        Hill {
            at: V3::new(cm(x), cm(y), cm(z)),
            radius: cm(radius),
            below: cm(below),
            room: cm(room),
        }
    }

    /// The highest feet that count, over the surface.
    pub fn reach(&self) -> Fx {
        t::hill_column().min(self.room)
    }

    /// Is a body with its feet here on the point: over the disc, from a
    /// little under its surface up through the column of air over it?
    pub fn holds(&self, feet: V3) -> bool {
        let up = feet.y.sub(self.at.y);
        up.raw() >= self.below.neg().raw()
            && up.raw() <= self.reach().raw()
            && feet.sub(self.at).flat_len().raw() <= self.radius.raw()
    }
}

/// **A stage's rules**: what an arena built for versus asks of the fight
/// beyond last one standing.
#[derive(Clone, Copy, Debug)]
pub struct Stage {
    pub arena: ArenaId,
    /// What kind of stage it is, for the picker and the HUD.
    pub kind: Kind,
    /// One line on what the place is.
    pub theme: &'static str,
    /// Below this height a fighter is knocked out (centimetres), or `None`
    /// for a stage on solid ground.
    pub pit: Option<i32>,
    /// The capture point, on a King of the Hill map.
    pub hill: Option<Hill>,
    /// **The line of five points**, on a five-point map ([`Kind::Front`]).
    pub front: Option<&'static Line>,
    /// **Its updrafts** (2026-11-02, `docs/design/fight-spaces.md`): columns
    /// of rising air, read by the same movement as the valley's
    /// (`crate::valley::vents` falls back to these) -- a way up to a tier
    /// that is not a stair, and on a floating stage a way back from a
    /// knock off the edge.
    pub vents: &'static [crate::valley::Vent],
    /// **Its boxes that move** (2026-11-02): lifts and ferries, exactly the
    /// valley's (`crate::drift`) -- where one is is a function of the frame,
    /// so nothing about it is in the snapshot.
    pub drifts: &'static [crate::drift::Drift],
    /// **The ways it promises** (2026-11-02): every climb, crossing and
    /// hidden way the map is built to have, each walked end to end by every
    /// class that moves (`tests/versus.rs`), and counted by the fight-space
    /// rules (`tests/fight_space.rs`).
    pub ways: &'static [Way],
}

/// **A way across a stage**: stood at `from`, walking to each point of
/// `route` in turn -- forward held, jumping when the next point is higher
/// or the walk is blocked. Points are centimetres, `(x, y, z)`, `y` the
/// height the feet arrive at.
#[derive(Clone, Copy, Debug)]
pub struct Way {
    /// What it is, for a failure's message and the audit.
    pub says: &'static str,
    pub from: (i32, i32, i32),
    pub route: &'static [(i32, i32, i32)],
}

impl Way {
    pub const fn new(
        says: &'static str,
        from: (i32, i32, i32),
        route: &'static [(i32, i32, i32)],
    ) -> Way {
        Way { says, from, route }
    }
}

/// Which mode a stage is played in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Rounds, last one standing, a floating stage you can be knocked off.
    Duel,
    /// King of the Hill: hold the point until your clock runs out.
    Hill,
    /// **Five points in a row** (2026-11-03, Team Fortress 2's five-point
    /// maps): each side owns its own two, the middle is nobody's, a side
    /// takes only the next point along its line, and the side that owns all
    /// five wins. A side respawns a point behind its front.
    Front,
}

impl Stage {
    /// The pit as a height, if there is one.
    pub fn pit(&self) -> Option<Fx> {
        self.pit.map(cm)
    }
}

/// Every stage, duels first, in the order the picker steps through them:
/// each King of the Hill map big, bigger and biggest (`arena::hills::sizes`),
/// so `Shift+N` flips through one map's sizes before the next map.
pub static STAGES: [&Stage; 20] = {
    use crate::arena::hills::{self, sizes};
    [
        &crate::arena::stages::ANVIL_STAGE,
        &crate::arena::stages::ZIGGURAT_STAGE,
        &crate::arena::stages::CALDERA_STAGE,
        &crate::arena::stages::TWIN_ISLES_STAGE,
        &hills::QUARRY_STAGE,
        &sizes::QUARRY_BIGGER_STAGE,
        &sizes::QUARRY_BIGGEST_STAGE,
        &hills::GLACIER_STAGE,
        &sizes::GLACIER_BIGGER_STAGE,
        &sizes::GLACIER_BIGGEST_STAGE,
        &hills::DELTA_STAGE,
        &sizes::DELTA_BIGGER_STAGE,
        &sizes::DELTA_BIGGEST_STAGE,
        &hills::GORGE_STAGE,
        &sizes::GORGE_BIGGER_STAGE,
        &sizes::GORGE_BIGGEST_STAGE,
        &hills::HENGE_STAGE,
        &sizes::HENGE_BIGGER_STAGE,
        &sizes::HENGE_BIGGEST_STAGE,
        &crate::arena::kilnworks::STAGE,
    ]
};

/// The stage an arena is, if it is one.
pub fn of(arena: ArenaId) -> Option<&'static Stage> {
    STAGES.iter().copied().find(|s| s.arena == arena)
}

/// The updrafts of the stage an arena is, if it is one; none otherwise.
pub fn vents(arena: ArenaId) -> &'static [crate::valley::Vent] {
    of(arena).map_or(&[], |s| s.vents)
}

/// The drifts of the stage an arena is, if it is one; none otherwise.
pub fn drifts(arena: ArenaId) -> &'static [crate::drift::Drift] {
    of(arena).map_or(&[], |s| s.drifts)
}

/// The stage after this arena in [`STAGES`], wrapping; the first from
/// anywhere that is not a stage.
pub fn after(arena: ArenaId) -> &'static Stage {
    match STAGES.iter().position(|s| s.arena == arena) {
        Some(k) => STAGES[(k + 1) % STAGES.len()],
        None => STAGES[0],
    }
}

/// The capture point's state, and the respawns of a King of the Hill game.
/// All zero (and not hashed) on every arena that has no hill.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Koth {
    /// Is a game on the hill being played at all?
    pub on: bool,
    /// Frames until the point can be captured. Counts down from the round's
    /// start.
    pub locked: u16,
    /// The side that owns the point, or [`NOBODY`].
    pub owner: u8,
    /// The side whose capture is under way, or [`NOBODY`].
    pub capper: u8,
    /// How far that capture is, in hundredths of a frame of one capper
    /// standing on it: done at `Capture takes` × 100.
    pub progress: u32,
    /// Each side's clock: frames it must still hold the point for.
    pub clock: [u16; SIDES],
    /// How many of each side stood on the point this frame.
    pub standing: [u8; SIDES],
    /// Each seat's wait to stand back up, in frames: zero while alive.
    pub respawn: [u16; MAX_PLAYERS],
}

impl Koth {
    pub const NONE: Koth = Koth {
        on: false,
        locked: 0,
        owner: NOBODY,
        capper: NOBODY,
        progress: 0,
        clock: [0; SIDES],
        standing: [0; SIDES],
        respawn: [0; MAX_PLAYERS],
    };

    /// A game on the hill from the top: the point locked and nobody's, and
    /// both clocks full. [`Koth::NONE`] on an arena with no hill.
    pub fn start(arena: ArenaId) -> Koth {
        if of(arena).and_then(|s| s.hill).is_none() {
            return Koth::NONE;
        }
        Koth {
            on: true,
            locked: t::hill_unlock(),
            clock: [t::hill_clock(); SIDES],
            ..Koth::NONE
        }
    }

    /// A capture is complete at this much progress.
    pub fn full() -> u32 {
        t::hill_capture() as u32 * 100
    }

    /// How far the capture is, 0 to 100.
    pub fn percent(&self) -> u32 {
        (self.progress * 100 / Koth::full().max(1)).min(100)
    }

    /// Both sides on the point at once: nothing moves.
    pub fn contested(&self) -> bool {
        self.standing.iter().all(|&n| n > 0)
    }

    /// **Overtime**: a clock is out, but somebody of the other side is on the
    /// point, so the game goes on until the last of them is off it.
    pub fn overtime(&self) -> bool {
        self.owner != NOBODY && self.clock[self.owner as usize] == 0 && self.challenged()
    }

    /// **Somebody of the side that does not own the point is on it**, alive:
    /// what holds a round open in overtime (2026-10-21, from play: "overtime
    /// until the losing team steps off"). A capture they had under way and
    /// left does not -- the moment the last of them is off, the clock that
    /// ran out wins.
    fn challenged(&self) -> bool {
        let other = (self.owner as usize + 1) % SIDES;
        self.standing[other] > 0
    }

    /// **One frame of the hill.** `on[i]` is whether seat `i` is alive and
    /// standing on the point. Returns the side that has won, if one has.
    pub fn step(&mut self, on: [bool; MAX_PLAYERS]) -> Option<u8> {
        if !self.on {
            return None;
        }
        self.standing = [0; SIDES];
        for (i, &here) in on.iter().enumerate() {
            if here {
                self.standing[side(i) as usize] += 1;
            }
        }
        if self.locked > 0 {
            self.locked -= 1;
            return None;
        }
        self.capture();
        // The holder's clock runs while they hold it, and they win when it
        // is out and nobody is challenging them for it.
        if self.owner != NOBODY {
            let o = self.owner as usize;
            self.clock[o] = self.clock[o].saturating_sub(1);
            if self.clock[o] == 0 && !self.challenged() {
                return Some(self.owner);
            }
        }
        None
    }

    /// The capture: who is on it moves the progress.
    fn capture(&mut self) {
        capture(
            &mut self.owner,
            &mut self.capper,
            &mut self.progress,
            self.standing,
        );
    }
}

/// **A capture, one frame**: who is standing on a point (`standing`, each
/// side's count) moves its progress -- King of the Hill's point and each of
/// a five-point map's ([`Front`]) alike. Returns whether it changed hands.
fn capture(owner: &mut u8, capper: &mut u8, progress: &mut u32, standing: [u8; SIDES]) -> bool {
    let full = Koth::full();
    let rate = |n: u8| -> u32 {
        if n == 0 {
            0
        } else {
            100 + t::hill_second() as u32 * (n as u32 - 1)
        }
    };
    let [a, b] = standing;
    let mut taken = false;
    // One side alone on it, and it is not theirs: their capture. A side
    // stepping onto somebody else's capture first takes it back to
    // nothing, at their own rate, before starting their own.
    let alone = match (a > 0, b > 0) {
        (true, false) => Some((0u8, a)),
        (false, true) => Some((1u8, b)),
        _ => None,
    };
    match alone {
        // Contested: nothing moves.
        None if a > 0 => {}
        // Nobody on it: a capture under way drains away slowly.
        None => {
            let drain = t::hill_decay() as u32;
            *progress = progress.saturating_sub(drain);
        }
        Some((s, n)) if s == *owner => {
            // The holders on their own point push a capture back.
            *progress = progress.saturating_sub(rate(n));
        }
        Some((s, n)) => {
            if *capper != s && *progress > 0 {
                *progress = progress.saturating_sub(rate(n));
                return false;
            }
            *capper = s;
            *progress = (*progress + rate(n)).min(full);
            if *progress >= full {
                *owner = s;
                *capper = NOBODY;
                *progress = 0;
                taken = true;
            }
        }
    }
    if *progress == 0 {
        *capper = NOBODY;
    }
    taken
}

// ---------------------------------------------------------------------------
// Five points in a row
// ---------------------------------------------------------------------------

/// How many points a five-point map has.
pub const POINTS: usize = 5;

/// **A five-point map's line** (2026-11-03): its five points from the west
/// side's last to the east side's last, and where the west side stands back
/// up when its spawn is at each of them. The map is turned half a turn about
/// its middle, so the east side's spawn at point `k` is the west side's at
/// `4 - k`, turned ([`Line::spawn`]).
#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub points: [Hill; POINTS],
    /// The west side's spawn while its spawn is at each point: behind the
    /// point, on the west side's own ground.
    pub spawns: [crate::arena::Mark; POINTS],
}

impl Line {
    /// Where `side` stands back up while its spawn is at point `k`.
    pub fn spawn(&self, side: u8, k: usize) -> crate::arena::Mark {
        if side == 0 {
            self.spawns[k]
        } else {
            let m = self.spawns[POINTS - 1 - k];
            crate::arena::Mark {
                at: V3::new(m.at.x.neg(), m.at.y, m.at.z.neg()),
                facing: V3::new(m.facing.x.neg(), m.facing.y, m.facing.z.neg()),
            }
        }
    }
}

/// **A game on a five-point map**: who owns each point, each capture under
/// way, the round's clock and the respawns. All zero (and not hashed) on
/// every arena that is not one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Front {
    pub on: bool,
    /// Frames until anything can be captured: the middle point is locked at
    /// the start so both sides can reach it.
    pub locked: u16,
    pub owner: [u8; POINTS],
    pub capper: [u8; POINTS],
    pub progress: [u32; POINTS],
    /// How many of each side stood on each point this frame -- counting only
    /// those who may take it or hold it ([`Front::open`]).
    pub standing: [[u8; SIDES]; POINTS],
    /// Frames left in the round. Every capture adds to it.
    pub clock: u32,
    /// Each seat's wait to stand back up, in frames: zero while alive.
    pub respawn: [u16; MAX_PLAYERS],
}

impl Front {
    pub const NONE: Front = Front {
        on: false,
        locked: 0,
        owner: [NOBODY; POINTS],
        capper: [NOBODY; POINTS],
        progress: [0; POINTS],
        standing: [[0; SIDES]; POINTS],
        clock: 0,
        respawn: [0; MAX_PLAYERS],
    };

    /// A game from the top: each side owns its own two points, the middle is
    /// nobody's and locked, the clock full. [`Front::NONE`] on an arena that
    /// is not a five-point map.
    pub fn start(arena: ArenaId) -> Front {
        if of(arena).and_then(|s| s.front).is_none() {
            return Front::NONE;
        }
        Front {
            on: true,
            locked: t::front_unlock(),
            owner: [0, 0, NOBODY, 1, 1],
            clock: t::front_clock(),
            ..Front::NONE
        }
    }

    /// **May `side` take point `k`?** It is not theirs, and the point on
    /// their side of it is theirs -- or it is the far end of the other
    /// side's line. Only the next point along a side's line is open to it.
    pub fn open(&self, k: usize, side: u8) -> bool {
        if self.owner[k] == side {
            return false;
        }
        if side == 0 {
            k > 0 && self.owner[k - 1] == 0
        } else {
            k < POINTS - 1 && self.owner[k + 1] == 1
        }
    }

    /// The point `side`'s front is at: the furthest of its own along its
    /// line, unbroken from its last.
    pub fn front(&self, side: u8) -> usize {
        if side == 0 {
            (0..POINTS)
                .take_while(|&k| self.owner[k] == 0)
                .last()
                .unwrap_or(0)
        } else {
            (0..POINTS)
                .rev()
                .take_while(|&k| self.owner[k] == 1)
                .last()
                .unwrap_or(POINTS - 1)
        }
    }

    /// **Where `side` stands back up**: a point behind its front, and never
    /// behind its own last -- so taking the middle moves a side's spawn up to
    /// its second point, and taking the other side's second to the middle.
    pub fn spawn_at(&self, side: u8) -> usize {
        let f = self.front(side);
        if side == 0 {
            f.saturating_sub(1)
        } else {
            (f + 1).min(POINTS - 1)
        }
    }

    /// How many points `side` owns.
    pub fn held(&self, side: u8) -> usize {
        self.owner.iter().filter(|&&o| o == side).count()
    }

    /// How far a capture on point `k` is, 0 to 100.
    pub fn percent(&self, k: usize) -> u32 {
        (self.progress[k] * 100 / Koth::full().max(1)).min(100)
    }

    /// **One frame of the line.** `on[i]` is which point seat `i` is
    /// standing on, alive, if any. Returns the round's result if it is over:
    /// `Some(side)` when a side owns all five or holds more when the clock
    /// runs out, `Some(NOBODY)` when the clock runs out level.
    pub fn step(&mut self, on: [Option<usize>; MAX_PLAYERS]) -> Option<u8> {
        if !self.on {
            return None;
        }
        self.standing = [[0; SIDES]; POINTS];
        for (i, at) in on.iter().enumerate() {
            let Some(k) = *at else { continue };
            let s = side(i);
            // Only those who may take it or hold it count on it: standing on
            // a point that is not next along your line does nothing.
            if self.owner[k] == s || self.open(k, s) {
                self.standing[k][s as usize] += 1;
            }
        }
        if self.locked > 0 {
            self.locked -= 1;
            return None;
        }
        for k in 0..POINTS {
            let (mut o, mut c, mut p) = (self.owner[k], self.capper[k], self.progress[k]);
            if capture(&mut o, &mut c, &mut p, self.standing[k]) {
                self.clock = self
                    .clock
                    .saturating_add(t::front_bonus())
                    .min(t::front_clock());
            }
            (self.owner[k], self.capper[k], self.progress[k]) = (o, c, p);
        }
        for s in 0..SIDES as u8 {
            if self.held(s) == POINTS {
                return Some(s);
            }
        }
        // A capture under way holds the round open, as overtime does.
        let under_way = self.progress.iter().any(|&p| p > 0);
        self.clock = self.clock.saturating_sub(1);
        if self.clock == 0 && !under_way {
            return Some(match self.held(0).cmp(&self.held(1)) {
                core::cmp::Ordering::Greater => 0,
                core::cmp::Ordering::Less => 1,
                core::cmp::Ordering::Equal => NOBODY,
            });
        }
        None
    }
}
