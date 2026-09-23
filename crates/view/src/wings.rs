//! The wings on the Dual mage's back: her two bars, drawn.
//!
//! **The body is the meter.** Dark on her left and Light on her right -- the
//! same sides as the arms that goad them -- each as long as its bar. Lopsided
//! wings are the gap, readable across the arena by both players; full span is
//! ascension. The HUD carries the two bars too, but the wings are the display,
//! and the thing the tiers unlock is the thing everyone is already looking at.
//!
//! The rule the overlay lives by holds here in the other direction: the drawn
//! span *is* the bar, by construction, and `view/tests/wings.rs` says so the way
//! `kinematics.rs` checks the blade against the fist. The wings are drawn and
//! not tested against; whether a wingspan reads as a bigger target and lies is
//! an open question in `docs/design/dual-mage.md`.

use crate::fx;
use crate::math::V3;
use sim::aim::{self, Hand};
use sim::class::Force;
use sim::state::Player;

/// How long one wing is with its bar full, in metres.
///
/// Presentation, not gameplay: it changes what is seen and nothing that
/// happens, which is why it is a constant here rather than an Oven knob.
pub const FULL_SPAN: f32 = 1.6;

/// Where on her back the wings root, as a share of her height.
const ROOT_HEIGHT: f32 = 0.78;

/// How far behind her centre line the root sits, in metres.
const ROOT_BACK: f32 = 0.12;

/// How far the tip of a wing rises above its root, as a share of its span.
/// Swept up rather than held level, so the two read as wings and not as arms.
const RAKE: f32 = 0.45;

/// One wing: where it roots, where its tip is, and how long it is.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Wing {
    pub force: Force,
    pub root: V3,
    pub tip: V3,
    /// Metres, from root to tip.
    pub span: f32,
}

/// Both wings, dark then light, if this fighter has bars to draw. `None` for
/// the other five classes, which is what hides them rather than drawing two
/// stubs.
pub fn wings(p: &Player) -> Option<[Wing; 2]> {
    let (dark, light) = sim::dual::bars(p)?;
    let ascending = sim::dual::ascending(p);
    let top = sim::tuning::meter_max().max(1) as f32;
    let span_of = |bar: sim::Fx| {
        if ascending {
            FULL_SPAN
        } else {
            (fx(bar) / top).clamp(0.0, 1.0) * FULL_SPAN
        }
    };
    Some([
        wing(p, Force::Dark, span_of(dark)),
        wing(p, Force::Light, span_of(light)),
    ])
}

/// Which arm a force's wing hangs beside. The simulation's own answer, so the
/// dark wing is on the side the dark auto comes out of whatever else changes.
pub const fn hand_of(force: Force) -> Hand {
    match force {
        Force::Dark => Hand::Left,
        Force::Light => Hand::Right,
    }
}

fn wing(p: &Player, force: Force, span: f32) -> Wing {
    let pos = [fx(p.pos.x), fx(p.pos.y), fx(p.pos.z)];
    let facing = [fx(p.facing.x), fx(p.facing.y), fx(p.facing.z)];
    let across = aim::across(p.facing, hand_of(force));
    let across = [fx(across.x), fx(across.y), fx(across.z)];
    let height = fx(sim::tuning::body_height()) * ROOT_HEIGHT;
    let root = [
        pos[0] - facing[0] * ROOT_BACK,
        pos[1] + height,
        pos[2] - facing[2] * ROOT_BACK,
    ];
    // Out along the side, up by the rake, and scaled so the root-to-tip
    // distance is exactly the span.
    let scale = span / (1.0 + RAKE * RAKE).sqrt();
    let tip = [
        root[0] + across[0] * scale,
        root[1] + RAKE * scale,
        root[2] + across[2] * scale,
    ];
    Wing {
        force,
        root,
        tip,
        span,
    }
}
