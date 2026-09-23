//! The wings on the Dual mage's back: her two bars, drawn.
//!
//! **The body is the meter.** Dark on her left and Light on her right -- the
//! same sides as the arms that goad them -- **three wings a side**, and a bar
//! is read by how many of its three are there. Empty is none; a third of the
//! bar and the first wing is there; two thirds, the second; full, the third.
//! They do not grow: a wing is a wing, and it materialises whole. Lopsided
//! wings are the gap, readable across the arena by both players; three and
//! three is a mage about to ascend, and ascension is all six.
//!
//! **Three sizes, and the biggest comes first.** From the top of her back
//! down: the smallest, the biggest, the middle one -- the seraph's proportions,
//! with the great wing in the middle where the shoulder blades are. They
//! arrive biggest first: the first third of a bar puts the great wing out, the
//! second the lower one, the third the small one above. So a mage at a third
//! already has a wing you can see across the arena, and the last one is a
//! flourish rather than the thing you are waiting for.
//!
//! Counting rather than stretching because a wing half a metre long is not a
//! wing, it is a line, and the thing the opponent has to read from across the
//! arena is a count -- one, two or three -- not a length.
//!
//! The rule the overlay lives by holds here in the other direction: what is
//! drawn *is* the bar, by construction, and `view/tests/wings.rs` says so the
//! way `kinematics.rs` checks the blade against the fist. The wings are drawn
//! and not tested against; whether a wingspan reads as a bigger target and
//! lies is an open question in `docs/design/dual-mage.md`.

use crate::fx;
use crate::math::V3;
use sim::aim::{self, Hand};
use sim::class::Force;
use sim::state::Player;

/// How many wings a side has, with its bar full.
pub const PER_SIDE: usize = 3;

/// Where on her back a wing sits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Slot {
    /// The smallest, reaching up. The last to appear.
    Top,
    /// The great wing, reaching out and a little up. The first to appear.
    Middle,
    /// The middle size, reaching down. The second to appear.
    Bottom,
}

/// The three slots in the order they **appear** as the bar rises.
pub const ORDER: [Slot; PER_SIDE] = [Slot::Middle, Slot::Bottom, Slot::Top];

impl Slot {
    /// Root to tip, in metres. Fixed: a wing does not grow.
    pub const fn length(self) -> f32 {
        match self {
            Slot::Top => 1.1,
            Slot::Middle => 2.3,
            Slot::Bottom => 1.7,
        }
    }

    /// Where it roots, as a share of her height.
    const fn root_height(self) -> f32 {
        match self {
            Slot::Top => 0.90,
            Slot::Middle => 0.79,
            Slot::Bottom => 0.66,
        }
    }

    /// Pitch above the horizontal, in radians: the small one reaches up past
    /// the shoulders, the great one up and out, the lower one out and down.
    const fn pitch(self) -> f32 {
        match self {
            Slot::Top => 1.20,
            Slot::Middle => 0.45,
            Slot::Bottom => -0.40,
        }
    }

    /// How far back it sweeps, as a share of its length along the facing.
    /// Little: a wing seen from behind, which is where the player is, has to
    /// be seen face-on, and every share of sweep is a share of the wing
    /// foreshortened away.
    const fn sweep(self) -> f32 {
        match self {
            Slot::Top => 0.15,
            Slot::Middle => 0.10,
            Slot::Bottom => 0.08,
        }
    }

    /// Which third of the bar puts it out: zero is the first.
    pub fn order(self) -> usize {
        ORDER.iter().position(|s| *s == self).unwrap_or(0)
    }
}

/// How far behind her centre line the roots sit, in metres.
const ROOT_BACK: f32 = 0.10;

/// The silhouette of one wing, in its own plane at unit length: `u` out along
/// the wing from the root, `v` across it. A bird's wing spread: a leading edge
/// that runs nearly straight out to the tip, and the long primaries hanging
/// from the outer half, shortening toward the root, each cut from the next by
/// a notch.
///
/// Star-shaped about [`OUTLINE_CENTRE`], so a triangle fan from there fills it
/// without a triangulator: each notch sits at the angular midpoint of the two
/// feather tips beside it, seen from that centre, and well inside them.
pub const OUTLINE: [[f32; 2]; 18] = [
    [0.00, 0.00],
    [0.28, 0.11],
    [0.58, 0.19],
    [0.84, 0.20],
    [1.00, 0.12],
    [0.99, -0.30],
    [0.84, -0.29],
    [0.90, -0.58],
    [0.77, -0.46],
    [0.76, -0.66],
    [0.64, -0.45],
    [0.58, -0.60],
    [0.52, -0.39],
    [0.40, -0.48],
    [0.38, -0.32],
    [0.22, -0.34],
    [0.28, -0.19],
    [0.04, -0.16],
];

/// The point the outline is filled from. See [`OUTLINE`].
pub const OUTLINE_CENTRE: [f32; 2] = [0.60, -0.04];

/// One wing's place in the arena.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Wing {
    pub force: Force,
    pub slot: Slot,
    /// Is it there? The bar has reached this wing's third.
    pub shown: bool,
    /// Root to tip, in metres.
    pub length: f32,
    /// Where it roots.
    pub root: V3,
    /// Unit, out along the wing: the outline's `u`.
    pub along: V3,
    /// Unit, across the wing: the outline's `v`, pointed up so the leading
    /// edge is the upper edge. The wing's face -- `along` crossed with this --
    /// looks forward and back, where the two players are.
    pub across: V3,
}

/// How many of a side's three wings a bar of `bar` shows.
///
/// The first at a third of the top, the second at two thirds, and the third at
/// `tuning::tier_wings` -- the same "full" ascension reads, so three and three
/// on her back means exactly what it looks like. A bar between the top and
/// that threshold shows three too.
pub fn shown_for(bar: sim::Fx) -> usize {
    let top = sim::tuning::meter_max().max(1);
    let full = sim::tuning::tier_wings().min(top);
    let bar = fx(bar);
    let third = top as f32 / PER_SIDE as f32;
    if bar >= full as f32 {
        PER_SIDE
    } else {
        ((bar / third).floor() as usize).min(PER_SIDE - 1)
    }
}

/// All six, the dark side first and each side in the order it appears, if
/// this fighter has bars to draw. `None` for the other five classes, which is
/// what hides them rather than drawing nothing six times.
pub fn wings(p: &Player) -> Option<[Wing; 2 * PER_SIDE]> {
    let (dark, light) = sim::dual::bars(p)?;
    let ascending = sim::dual::ascending(p);
    let shown = |bar: sim::Fx| {
        if ascending { PER_SIDE } else { shown_for(bar) }
    };
    let (d, l) = (shown(dark), shown(light));
    Some([
        wing(p, Force::Dark, ORDER[0], d > 0),
        wing(p, Force::Dark, ORDER[1], d > 1),
        wing(p, Force::Dark, ORDER[2], d > 2),
        wing(p, Force::Light, ORDER[0], l > 0),
        wing(p, Force::Light, ORDER[1], l > 1),
        wing(p, Force::Light, ORDER[2], l > 2),
    ])
}

/// Which arm a force's wings hang beside. The simulation's own answer, so the
/// dark wings are on the side the dark auto comes out of whatever else changes.
pub const fn hand_of(force: Force) -> Hand {
    match force {
        Force::Dark => Hand::Left,
        Force::Light => Hand::Right,
    }
}

fn norm(v: V3) -> V3 {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    [v[0] / n, v[1] / n, v[2] / n]
}

fn wing(p: &Player, force: Force, slot: Slot, shown: bool) -> Wing {
    let pos = [fx(p.pos.x), fx(p.pos.y), fx(p.pos.z)];
    let facing = norm([fx(p.facing.x), 0.0, fx(p.facing.z)]);
    let side = aim::across(p.facing, hand_of(force));
    let side = norm([fx(side.x), 0.0, fx(side.z)]);
    let height = fx(sim::tuning::body_height()) * slot.root_height();
    let root = [
        pos[0] - facing[0] * ROOT_BACK,
        pos[1] + height,
        pos[2] - facing[2] * ROOT_BACK,
    ];
    // Out to the side, pitched up or down, swept back.
    let (sin, cos) = slot.pitch().sin_cos();
    let along = norm([
        side[0] * cos - facing[0] * slot.sweep(),
        sin,
        side[2] * cos - facing[2] * slot.sweep(),
    ]);
    // Across is world-up with its `along` component taken out: the chord runs
    // up the wing, so the vane stands in the vertical plane through `along`
    // and its face looks forward and back.
    let up_along = along[1];
    let across = norm([
        -along[0] * up_along,
        1.0 - along[1] * up_along,
        -along[2] * up_along,
    ]);
    Wing {
        force,
        slot,
        shown,
        length: slot.length(),
        root,
        along,
        across,
    }
}
