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
//! Counting rather than stretching because a wing half a metre long is not a
//! wing, it is a line, and the thing the opponent has to read from across the
//! arena is a count -- one, two or three -- not a length. Three per side is the
//! seraph's six, and it is the number of thresholds a player can hold in mind.
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

/// How long one wing is, root to tip, in metres. Fixed: a wing does not grow.
pub const LENGTH: f32 = 1.15;

/// Where on her back the wings root, as a share of her height: the top pair,
/// and how much lower each pair below it roots.
const ROOT_HEIGHT: f32 = 0.84;
const ROOT_DROP: f32 = 0.11;

/// How far behind her centre line the roots sit, in metres.
const ROOT_BACK: f32 = 0.10;

/// Each wing's pitch above the horizontal, top pair first, in radians. The
/// top pair reaches up, the middle pair out, the bottom pair down: the fan of
/// a seraph rather than three bars stacked.
const PITCH: [f32; PER_SIDE] = [0.85, 0.15, -0.55];

/// How far back each wing sweeps, as a share of its length along the facing.
/// The top pair sweeps back the most, so the three read as a fan from behind
/// as well as from the side.
const SWEEP: [f32; PER_SIDE] = [0.55, 0.35, 0.25];

/// The silhouette of one wing, in its own plane: `u` out along the wing from
/// the root, `v` across it, as shares of [`LENGTH`]. A leading edge that
/// rises and falls to the tip, and a trailing edge cut into five feathers.
///
/// Star-shaped about [`OUTLINE_CENTRE`], so a triangle fan from there fills it
/// without a triangulator: every edge is seen face-on from that point, because
/// each notch of the trailing edge sits inward of the feather tip before it.
pub const OUTLINE: [[f32; 2]; 16] = [
    [0.00, 0.02],
    [0.18, 0.13],
    [0.42, 0.21],
    [0.68, 0.22],
    [0.88, 0.16],
    [1.00, 0.07],
    // The trailing edge, tip to root: feather tip, notch, feather tip ...
    [0.97, -0.10],
    [0.84, -0.09],
    [0.86, -0.27],
    [0.70, -0.19],
    [0.64, -0.38],
    [0.50, -0.24],
    [0.40, -0.40],
    [0.30, -0.22],
    [0.16, -0.30],
    [0.00, -0.10],
];

/// The point the outline is filled from. See [`OUTLINE`].
pub const OUTLINE_CENTRE: [f32; 2] = [0.45, -0.02];

/// One wing's place in the arena.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Wing {
    pub force: Force,
    /// Which of the side's three: zero is the top pair.
    pub index: usize,
    /// Is it there? The bar has reached this wing's third.
    pub shown: bool,
    /// Where it roots.
    pub root: V3,
    /// Unit, out along the wing: the outline's `u`.
    pub along: V3,
    /// Unit, across the wing: the outline's `v`. Up-ish, so the leading edge
    /// is the upper edge.
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

/// All six, dark side first, if this fighter has bars to draw. `None` for the
/// other five classes, which is what hides them rather than drawing nothing
/// six times.
pub fn wings(p: &Player) -> Option<[Wing; 2 * PER_SIDE]> {
    let (dark, light) = sim::dual::bars(p)?;
    let ascending = sim::dual::ascending(p);
    let shown = |bar: sim::Fx| {
        if ascending { PER_SIDE } else { shown_for(bar) }
    };
    let (d, l) = (shown(dark), shown(light));
    Some([
        wing(p, Force::Dark, 0, d > 0),
        wing(p, Force::Dark, 1, d > 1),
        wing(p, Force::Dark, 2, d > 2),
        wing(p, Force::Light, 0, l > 0),
        wing(p, Force::Light, 1, l > 1),
        wing(p, Force::Light, 2, l > 2),
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

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn wing(p: &Player, force: Force, index: usize, shown: bool) -> Wing {
    let pos = [fx(p.pos.x), fx(p.pos.y), fx(p.pos.z)];
    let facing = norm([fx(p.facing.x), 0.0, fx(p.facing.z)]);
    let side = aim::across(p.facing, hand_of(force));
    let side = norm([fx(side.x), 0.0, fx(side.z)]);
    let height = fx(sim::tuning::body_height()) * (ROOT_HEIGHT - ROOT_DROP * index as f32);
    let root = [
        pos[0] - facing[0] * ROOT_BACK,
        pos[1] + height,
        pos[2] - facing[2] * ROOT_BACK,
    ];
    // Out to the side, pitched up or down, swept back.
    let (sin, cos) = PITCH[index].sin_cos();
    let along = norm([
        side[0] * cos - facing[0] * SWEEP[index],
        sin,
        side[2] * cos - facing[2] * SWEEP[index],
    ]);
    // The wing is a flat vane whose face looks forward and back -- where the
    // two players are -- so its plane holds the sideways direction, and
    // `across` is what is left: perpendicular to `along` inside that plane,
    // pointed up so the leading edge is the upper edge.
    let normal = norm(cross(along, side));
    let a = norm(cross(normal, along));
    let across = if a[1] < 0.0 { [-a[0], -a[1], -a[2]] } else { a };
    Wing {
        force,
        index,
        shown,
        root,
        along,
        across,
    }
}
