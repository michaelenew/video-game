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
}

/// Which mode a stage is played in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Rounds, last one standing, a floating stage you can be knocked off.
    Duel,
    /// King of the Hill: hold the point until your clock runs out.
    Hill,
}

impl Stage {
    /// The pit as a height, if there is one.
    pub fn pit(&self) -> Option<Fx> {
        self.pit.map(cm)
    }
}

/// Every stage, duels first, in the order the picker steps through them.
pub static STAGES: [&Stage; 9] = [
    &crate::arena::stages::ANVIL_STAGE,
    &crate::arena::stages::ZIGGURAT_STAGE,
    &crate::arena::stages::CALDERA_STAGE,
    &crate::arena::stages::TWIN_ISLES_STAGE,
    &crate::arena::hills::QUARRY_STAGE,
    &crate::arena::hills::GLACIER_STAGE,
    &crate::arena::hills::DELTA_STAGE,
    &crate::arena::hills::GORGE_STAGE,
    &crate::arena::hills::HENGE_STAGE,
];

/// The stage an arena is, if it is one.
pub fn of(arena: ArenaId) -> Option<&'static Stage> {
    STAGES.iter().copied().find(|s| s.arena == arena)
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
        let full = Koth::full();
        let rate = |n: u8| -> u32 {
            if n == 0 {
                0
            } else {
                100 + t::hill_second() as u32 * (n as u32 - 1)
            }
        };
        let [a, b] = self.standing;
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
                self.progress = self.progress.saturating_sub(drain);
            }
            Some((s, n)) if s == self.owner => {
                // The holders on their own point push a capture back.
                self.progress = self.progress.saturating_sub(rate(n));
            }
            Some((s, n)) => {
                if self.capper != s && self.progress > 0 {
                    self.progress = self.progress.saturating_sub(rate(n));
                    return;
                }
                self.capper = s;
                self.progress = (self.progress + rate(n)).min(full);
                if self.progress >= full {
                    self.owner = s;
                    self.capper = NOBODY;
                    self.progress = 0;
                }
            }
        }
        if self.progress == 0 {
            self.capper = NOBODY;
        }
    }
}
