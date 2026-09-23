//! The Dual mage's two bars, and the hill between them.
//!
//! She holds two beings apart, and each has a bar: **Dark** and **Light**, from
//! empty to full. Nothing is signed and there is no centre. Goading -- throwing
//! anything -- raises one of them. Every frame the two are compared, and the
//! rest of this module is what happens next:
//!
//! - inside a **band** around equal, nothing moves on its own;
//! - outside it, **the higher bar rises and the lower falls**, faster the
//!   further outside the band they are, up to a cap -- the hill. A small lead
//!   is stable, a large one runs away;
//! - and she **burns**: health drains at a rate that grows with the same
//!   excess, capped, and never past one health;
//! - **calm**: both bars fall slowly on their own, always, so a tier is a thing
//!   she holds by fighting rather than a level she reached.
//!
//! Three quantities fall out and each is read by something different. The
//! **carried bar** is what a cast is worth -- [`depth_at`]. The **lower bar** is
//! what her body can do -- [`Tier`]. The **gap** is how fast she is losing it.
//!
//! All of it is integer arithmetic on fixed point, a dozen operations a frame
//! for one fighter, and every magnitude in it is an Oven knob. See
//! `docs/design/dual-mage.md`, which is the specification.

use crate::DT;
use crate::class::{Force, Mechanic};
use crate::fixed::Fx;
use crate::state::{Action, Player};
use crate::tuning as t;

/// What the **lower** bar has unlocked.
///
/// On the lower bar and not on the sum for one reason: a sum can be reached
/// one-sided. Requiring both to be high is what makes the climb a rhythm of
/// alternating hands, and that rhythm uses the two-form kit in both forms.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Tier {
    /// Below the first threshold: the dodge is a dodge, space in the air does
    /// nothing.
    None,
    /// The dodge is a blink. See `state::step_player`'s dodge branch.
    Blink,
    /// A second jump, once per airtime, and a slower fall.
    Jump,
    /// Both full: ascension. Wings, and a wing beat on every press.
    Wings,
}

impl Tier {
    /// One word for the HUD and the frame table.
    pub const fn name(self) -> &'static str {
        match self {
            Tier::None => "",
            Tier::Blink => "blink",
            Tier::Jump => "second jump",
            Tier::Wings => "WINGS",
        }
    }

    /// The bar value that unlocks this tier, in whole units of the bar.
    pub fn threshold(self) -> i32 {
        match self {
            Tier::None => 0,
            Tier::Blink => t::tier_blink(),
            Tier::Jump => t::tier_jump(),
            Tier::Wings => t::tier_wings(),
        }
    }
}

/// The tier a lower bar of this value holds.
///
/// Held while the bar is at or above the threshold and lost the frame it drops
/// below -- including mid-air, which is what makes a Judgement thrown at three
/// quarters a decision about the jump she is in the middle of.
pub fn tier_of_bar(lower: Fx) -> Tier {
    let at = |tier: Tier| lower.raw() >= Fx::from_int(tier.threshold()).raw();
    if at(Tier::Wings) {
        Tier::Wings
    } else if at(Tier::Jump) {
        Tier::Jump
    } else if at(Tier::Blink) {
        Tier::Blink
    } else {
        Tier::None
    }
}

/// The tier this fighter holds right now. [`Tier::None`] for anybody else.
///
/// Ascending is the top tier for as long as it runs, whatever the bars say:
/// the bars are pinned while it does, and what it grants is the point of it.
pub fn tier(p: &Player) -> Tier {
    let Mechanic::Meter {
        dark,
        light,
        ascending,
        ..
    } = p.mechanic
    else {
        return Tier::None;
    };
    if ascending > 0 {
        return Tier::Wings;
    }
    tier_of_bar(dark.min(light))
}

/// Both bars, dark then light, if this fighter has them.
pub fn bars(p: &Player) -> Option<(Fx, Fx)> {
    match p.mechanic {
        Mechanic::Meter { dark, light, .. } => Some((dark, light)),
        _ => None,
    }
}

/// One bar. Zero for anybody without one.
pub fn bar(p: &Player, force: Force) -> Fx {
    match (p.mechanic, force) {
        (Mechanic::Meter { dark, .. }, Force::Dark) => dark,
        (Mechanic::Meter { light, .. }, Force::Light) => light,
        _ => Fx::ZERO,
    }
}

/// The lower of the two: her **frenzy**, which is what the tiers read.
pub fn lower(p: &Player) -> Fx {
    bars(p).map_or(Fx::ZERO, |(d, l)| d.min(l))
}

/// How far apart the two are.
pub fn gap(p: &Player) -> Fx {
    bars(p).map_or(Fx::ZERO, |(d, l)| d.sub(l).abs())
}

/// Is she inside the band -- level enough that nothing moves on its own?
pub fn level(p: &Player) -> bool {
    gap(p).raw() <= Fx::from_int(t::meter_band()).raw()
}

/// Is she ascending?
pub fn ascending(p: &Player) -> bool {
    matches!(p.mechanic, Mechanic::Meter { ascending, .. } if ascending > 0)
}

/// The top of either bar.
fn top() -> Fx {
    Fx::from_int(t::meter_max().max(1))
}

// ---------------------------------------------------------------------------
// The struggle
// ---------------------------------------------------------------------------

/// One frame of the two bars: calm, the hill, the burn, and the clock while
/// she is ascending. A no-op for anybody else.
pub fn step(p: &mut Player) {
    let Mechanic::Meter {
        dark,
        light,
        colour,
        ascending,
        jumped,
        landed,
        singe,
    } = p.mechanic
    else {
        return;
    };

    if ascending > 0 {
        // The clock. Health is the cost and nothing steers the bars: they are
        // not the resource while it runs, and they are what she gets back --
        // empty -- when it ends.
        singe_health(p, t::ascension_drain().max(1));
        let left = ascending - 1;
        p.mechanic = Mechanic::Meter {
            dark,
            light,
            colour,
            ascending: left,
            jumped,
            landed,
            singe,
        };
        if left == 0 {
            end_ascension(p, landed);
        }
        return;
    }

    // Calm: both fall, always, floored at zero.
    let calm = t::meter_calm().mul(DT);
    let mut dark = dark.sub(calm).max(Fx::ZERO);
    let mut light = light.sub(calm).max(Fx::ZERO);

    // The hill. Compared every frame; inside the band nothing happens.
    let gap = dark.sub(light).abs();
    let excess = gap.sub(Fx::from_int(t::meter_band()));
    let mut singe = singe;
    if excess.raw() > 0 {
        let drift = t::drift_gain().mul(excess).min(t::drift_cap()).mul(DT);
        let (high, low) = if dark.raw() >= light.raw() {
            (&mut dark, &mut light)
        } else {
            (&mut light, &mut dark)
        };
        // The higher bar gains exactly what the lower one lost, so the drift
        // moves what she has from one being to the other and never makes more
        // of it: with no input, `dark + light` can only fall. It is what keeps
        // the hill from ever being a road to the top.
        let taken = drift.min(*low);
        *low = low.sub(taken);
        *high = high.add(taken).min(top());

        // The burn, in health per second, carried forward in `singe` so that a
        // rate under sixty a second is a slow burn and not no burn at all.
        singe = singe.add(t::burn_gain().mul(excess).min(t::burn_cap()).mul(DT));
        let owed = singe.to_int();
        if owed > 0 {
            singe_health(p, owed);
            singe = singe.sub(Fx::from_int(owed));
        }
    }

    // Both goaded to the top together, and it takes her. There is no input for
    // it and there never was; the drift cannot deliver it, because the drift
    // only ever pulls the two apart.
    let ascending = if tier_of_bar(dark.min(light)) == Tier::Wings {
        t::ascension_frames()
    } else {
        0
    };
    p.mechanic = Mechanic::Meter {
        dark,
        light,
        colour,
        ascending,
        jumped,
        landed: 0,
        singe,
    };
}

/// Health comes off, and never past one. Dying to your own bar is not a
/// decision anybody made -- the same rule the Blood mage's costs follow.
fn singe_health(p: &mut Player, amount: i32) {
    if p.health > 1 {
        p.health = (p.health - amount).max(1);
    }
}

/// Ascension is over: both bars empty, and a stagger graduated by how much of
/// the cost she paid back. The vent.
///
/// The share is what the hits she landed refunded against what the whole
/// window drained, so it is the design's own sentence -- *you are staying alive
/// one connection at a time* -- read back as a number. Nothing landed is the
/// ceiling; paying it all back is the floor; a near miss is a short stagger,
/// which is what lets the timing be learnt rather than feared.
fn end_ascension(p: &mut Player, landed: u16) {
    let Mechanic::Meter { colour, .. } = p.mechanic else {
        return;
    };
    let drained = (t::ascension_drain().max(1) * t::ascension_frames() as i32).max(1);
    let refunded = (landed as i32 * t::ascension_refund()).min(drained);
    let share = Fx::ratio(refunded, drained);
    let ceiling = Fx::from_int(t::ascension_stun() as i32);
    let floor = Fx::from_int(t::ascension_stun_floor() as i32).min(ceiling);
    let stun = ceiling.sub(ceiling.sub(floor).mul(share)).to_int().max(0) as u16;
    p.mechanic = Mechanic::Meter {
        dark: Fx::ZERO,
        light: Fx::ZERO,
        colour,
        ascending: 0,
        jumped: false,
        landed: 0,
        singe: Fx::ZERO,
    };
    p.stun_total = stun;
    p.action = Action::Stagger { left: stun };
}

/// Throwing a move goads a bar: an auto its own, by a little; a cast the bar
/// of the force she is carrying, by more; the finisher by a lot.
///
/// **On the press, every time, including the autos.** They used to steer on
/// contact -- the design's own rule -- and with nothing in reach no button on
/// the class moved anything at all. A resource you cannot move without a target
/// is one nobody can learn or tune. See the feel log for 2026-09-13.
///
/// **Which bar comes from the force she is carrying, not from the buttons held
/// down.** Only the autos have a side of their own, and throwing one is what
/// sets which force she carries; everything else is made of that force.
pub fn steer(p: &mut Player, kind: u8) {
    let Mechanic::Meter {
        dark,
        light,
        colour,
        ascending,
        jumped,
        landed,
        singe,
    } = p.mechanic
    else {
        return;
    };
    // Nothing steers during ascension. The bars are not the resource then;
    // the clock is.
    if ascending > 0 {
        return;
    }
    // Three tiers of push, and the order of the arms is the order of the
    // commitment. An auto is the unit the bars are measured in, and it also
    // *sets* which force she is carrying.
    let (push, colour) = match crate::moves::dual::force(kind) {
        Some(thrown) => (t::meter_auto_push(), thrown),
        None if crate::moves::dual::is_the_finisher(kind) => (t::meter_finisher_push(), colour),
        None => (t::meter_cast_push(), colour),
    };
    let goad = |bar: Fx| bar.add(Fx::from_int(push)).min(top());
    let (dark, light) = match colour {
        Force::Dark => (goad(dark), light),
        Force::Light => (dark, goad(light)),
    };
    p.mechanic = Mechanic::Meter {
        dark,
        light,
        colour,
        ascending,
        jumped,
        landed,
        singe,
    };
}

// ---------------------------------------------------------------------------
// The depth curve
// ---------------------------------------------------------------------------

/// What one bar is worth as a multiplier on everything she throws, read this
/// instant.
///
/// A straight line from `tuning::depth_floor` at empty to
/// `tuning::depth_ceiling` at full, with every point on it reachable and
/// nothing anywhere that snaps. **Which bar** is the force the move is made
/// of: an auto reads its own, and everything else reads the bar of the force
/// she is carrying -- `kind` is `None` for a question asked between moves,
/// which is a question about the carried force.
///
/// While ascending it is the top of the curve, because both bars are at the
/// top: that is what the ride is.
///
/// [`Fx::ONE`] for every class but one, so it is safe to ask of anybody.
pub fn depth_at(p: &Player, kind: Option<u8>) -> Fx {
    let Mechanic::Meter {
        dark,
        light,
        colour,
        ascending,
        ..
    } = p.mechanic
    else {
        return Fx::ONE;
    };
    if ascending > 0 {
        return t::depth_ceiling();
    }
    let force = kind.and_then(crate::moves::dual::force).unwrap_or(colour);
    let bar = match force {
        Force::Dark => dark,
        Force::Light => light,
    };
    let out = bar.div(top()).clamp(Fx::ZERO, Fx::ONE);
    crate::math::lerp(t::depth_floor(), t::depth_ceiling(), out)
}

// ---------------------------------------------------------------------------
// What the tiers do to her body
// ---------------------------------------------------------------------------

/// Landing a hit while ascending pulls some health back, and counts toward
/// the stagger on the way out. Nothing happens to anybody else, or to her
/// between ascensions.
pub fn landed_a_hit(p: &mut Player) {
    let Mechanic::Meter {
        ascending, landed, ..
    } = &mut p.mechanic
    else {
        return;
    };
    if *ascending == 0 {
        return;
    }
    *landed = landed.saturating_add(1);
    p.heal(t::ascension_refund());
}

/// Is the dodge a blink right now?
///
/// At the first tier, and **not** while ascending: she flies instead, and loss
/// of control is loss of the option to decline.
pub fn may_blink(p: &Player) -> bool {
    tier(p) == Tier::Blink || tier(p) == Tier::Jump
}

/// May she dodge at all? Refused for the whole of ascension.
pub fn may_dodge(p: &Player) -> bool {
    !ascending(p)
}

/// Is a press of space in the air a jump?
///
/// Once per airtime at the second tier; every press while ascending, which is
/// the wing beat.
pub fn may_beat_wings(p: &Player) -> bool {
    match p.mechanic {
        Mechanic::Meter {
            ascending, jumped, ..
        } => ascending > 0 || (!jumped && tier(p) >= Tier::Jump),
        _ => false,
    }
}

/// Spend the second jump, so it pays for one takeoff and not two.
pub fn spend_wing_beat(p: &mut Player) {
    if let Mechanic::Meter { jumped, .. } = &mut p.mechanic {
        *jumped = true;
    }
}

/// Feet on the floor: the second jump is back.
pub fn feet_down(p: &mut Player) {
    if let Mechanic::Meter { jumped, .. } = &mut p.mechanic {
        *jumped = false;
    }
}

/// What the fall cap is multiplied by: the slow fall at the second tier and
/// above, one for everybody else.
pub fn fall_scale(p: &Player) -> Fx {
    if tier(p) >= Tier::Jump {
        t::slow_fall()
    } else {
        Fx::ONE
    }
}

/// How far the ordinary dodge would have carried her: where a blink goes.
///
/// The dodge is a shove that decays every frame, so its distance is the sum of
/// that series over its own length -- worked out from the same three knobs the
/// dodge is built from, so a blink lands exactly where the dodge would have
/// ended whatever the dodge is tuned to.
pub fn dodge_travel(grounded: bool) -> Fx {
    let (speed, frames) = if grounded {
        (t::dodge_speed(), t::dodge_frames())
    } else {
        (t::air_dodge_speed(), t::air_dodge_frames())
    };
    let mut step = speed.mul(DT);
    let mut total = Fx::ZERO;
    for _ in 0..frames {
        total = total.add(step);
        step = step.mul(t::dodge_decay());
    }
    total
}
