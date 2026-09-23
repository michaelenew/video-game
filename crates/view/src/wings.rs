//! The wings on the Dual mage's back: her two bars, drawn.
//!
//! **The body is the meter.** Dark on her left and Light on her right -- the
//! same sides as the arms that goad them -- **three wings a side**, and a bar
//! is read by how many of its three are there. **A wing is a tier**: the
//! first arrives when the bar reaches the blink's threshold, the second at
//! the second jump's, the third at the top the wings read. Those are the
//! marks the HUD's track already carries, so a wing appearing on her back and
//! a tick passing on the bar are the same moment -- the moment something
//! about what her body can do changes. They do not grow: a wing is a wing,
//! and it materialises whole. Lopsided wings are the gap, readable across the
//! arena by both players; three and three is a mage about to ascend, and
//! ascension is all six.
//!
//! **Three sizes, and the biggest comes first.** From the top of her back
//! down: the smallest, the biggest, the middle one -- the seraph's proportions,
//! with the great wing in the middle where the shoulder blades are. They
//! arrive biggest first: the blink puts the great wing out, the second jump
//! the lower one, the top the small one above. So a mage at half already has a
//! wing you can see across the arena, and the last one is a flourish rather
//! than the thing you are waiting for.
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
    ///
    /// The small one **floats**, above her head and clear of the great wing,
    /// bound to nothing: it is the highest presence of that being, not a limb,
    /// and a binding that is not physical is the point of it. The other two
    /// root on the back where the shoulder blades are.
    const fn root_height(self) -> f32 {
        match self {
            Slot::Top => 1.30,
            Slot::Middle => 0.79,
            Slot::Bottom => 0.66,
        }
    }

    /// How far out to its own side it roots, in metres. The floating one
    /// starts off the shoulder rather than on the spine, so it hovers over
    /// the great wing instead of over her head.
    const fn root_out(self) -> f32 {
        match self {
            Slot::Top => 0.45,
            Slot::Middle => 0.0,
            Slot::Bottom => 0.0,
        }
    }

    /// Pitch of the wing's own line above the horizontal, in radians. The arm
    /// rises another thirty-five degrees inside the wing before the wrist, so
    /// these are lower than the wings look: the great one's wrist ends up
    /// about sixty degrees up, the floating one's about fifty, the lower one's
    /// level with the shoulder with its primaries hanging. The first cut had
    /// the small one rooted on the back at seventy, which put its wrist past
    /// vertical and its feathers over her head onto the other side; floating
    /// off the shoulder it can lie lower and still sit clear of the great one.
    const fn pitch(self) -> f32 {
        match self {
            Slot::Top => 0.35,
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

    /// Which tier of the bar puts it out: zero is the first, the blink's.
    pub fn order(self) -> usize {
        ORDER.iter().position(|s| *s == self).unwrap_or(0)
    }
}

/// How far behind her centre line the roots sit, in metres.
const ROOT_BACK: f32 = 0.10;

/// One feather, in the wing's own plane at unit span: `u` out along the wing
/// from the shoulder, `v` across it, up. A rounded slat -- a narrow rectangle
/// with a half-round tip -- so each part is convex and fans from its own
/// middle, and the wing is the overlap of many rather than one sheet.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Feather {
    /// Where its quill roots.
    pub base: [f32; 2],
    /// Which way it points, in radians from `+u`. Negative is down.
    pub angle: f32,
    /// Quill to tip, as a share of the span.
    pub length: f32,
    /// Across the vane.
    pub width: f32,
}

const fn feather(base: [f32; 2], degrees: f32, length: f32, width: f32) -> Feather {
    Feather {
        base,
        angle: degrees * (core::f32::consts::PI / 180.0),
        length,
        width,
    }
}

/// The shoulder, where the arm roots. The origin of the wing's plane.
pub const SHOULDER: [f32; 2] = [0.0, 0.0];

/// The wrist: where the arm stops rising and the primaries fan out from.
/// The highest point of the leading edge, well short of the tip -- that bend
/// is the whole difference between a bird's wing and a butterfly's.
pub const WRIST: [f32; 2] = [0.42, 0.30];

/// The feathers, primaries first. **The primaries fan from the wrist** like
/// fingers, the outermost nearly along the arm's line and reaching the full
/// span, each next one shorter and hung lower, the last hanging straight
/// down. **The secondaries hang from the arm** between wrist and shoulder,
/// overlapping, shorter toward the body. Seen from behind that is an eagle's
/// wing spread; a fan of blades from one point is an insect's.
pub const FEATHERS: [Feather; 12] = [
    feather(WRIST, -8.0, 0.62, 0.085),
    feather(WRIST, -22.0, 0.60, 0.085),
    feather(WRIST, -36.0, 0.57, 0.085),
    feather(WRIST, -50.0, 0.53, 0.085),
    feather(WRIST, -64.0, 0.48, 0.085),
    feather(WRIST, -78.0, 0.43, 0.085),
    feather(WRIST, -92.0, 0.38, 0.085),
    feather([0.36, 0.26], -100.0, 0.36, 0.10),
    feather([0.28, 0.20], -102.0, 0.34, 0.10),
    feather([0.20, 0.14], -104.0, 0.31, 0.10),
    feather([0.12, 0.09], -106.0, 0.27, 0.10),
    feather([0.05, 0.04], -108.0, 0.22, 0.10),
];

/// How many of [`FEATHERS`] are primaries, fanning from the wrist.
pub const PRIMARIES: usize = 7;

/// The coverts: the body of the arm from shoulder to wrist, covering the
/// quills. Convex, so it fans from any interior point.
pub const COVERTS: [[f32; 2]; 5] = [
    [0.00, 0.03],
    [0.42, 0.36],
    [0.58, 0.32],
    [0.48, 0.12],
    [0.14, -0.08],
];

/// How many points a feather's half-round tip is drawn with.
pub const TIP_POINTS: usize = 5;

/// A feather's outline, counter-clockwise: the two long edges and the rounded
/// tip. Convex, so a fan from any interior point fills it.
pub fn feather_outline(f: &Feather) -> [[f32; 2]; 4 + TIP_POINTS] {
    let (sin, cos) = f.angle.sin_cos();
    let d = [cos, sin];
    let n = [-sin, cos];
    let half = f.width * 0.5;
    let shaft = (f.length - half).max(0.0);
    let at = |along: f32, across: f32| {
        [
            f.base[0] + d[0] * along + n[0] * across,
            f.base[1] + d[1] * along + n[1] * across,
        ]
    };
    let mut out = [[0.0; 2]; 4 + TIP_POINTS];
    out[0] = at(0.0, -half);
    out[1] = at(shaft, -half);
    for (i, slot) in out[2..2 + TIP_POINTS].iter_mut().enumerate() {
        // From the right edge round to the left, through the tip.
        let a = -core::f32::consts::FRAC_PI_2
            + core::f32::consts::PI * (i as f32 + 1.0) / (TIP_POINTS as f32 + 1.0);
        *slot = at(shaft + half * a.cos(), half * a.sin());
    }
    out[2 + TIP_POINTS] = at(shaft, half);
    out[3 + TIP_POINTS] = at(0.0, half);
    out
}

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

/// How many of a side's three wings a bar of `bar` shows: how many of the
/// three tiers that bar has reached on its own.
///
/// The blink's threshold, the second jump's and the top the wings read -- the
/// same three numbers `sim::dual::Tier` reads off the *lower* bar and the
/// HUD's track ticks on each side. So a side's wings say how far that being
/// has been fed, and the tier she actually holds is whichever side has fewer.
/// The first cut put them at thirds of the bar, which was a fourth set of
/// marks nobody else used.
pub fn shown_for(bar: sim::Fx) -> usize {
    use sim::dual::Tier;
    [Tier::Blink, Tier::Jump, Tier::Wings]
        .into_iter()
        .filter(|tier| bar.raw() >= sim::Fx::from_int(tier.threshold()).raw())
        .count()
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
        pos[0] - facing[0] * ROOT_BACK + side[0] * slot.root_out(),
        pos[1] + height,
        pos[2] - facing[2] * ROOT_BACK + side[2] * slot.root_out(),
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
