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
/// she is ascending -- and the Veil's clock. A no-op for anybody else.
///
/// **True on the frame ascension begins**, which is the world's cue for
/// Apotheosis: the nova that bursts from her as the wings erupt. See
/// `docs/design/exploration/0015_dual_mage_spellbook.md` §"Ascending at the
/// right moment".
pub fn step(p: &mut Player) -> bool {
    let Mechanic::Meter {
        dark,
        light,
        ascending,
        landed,
        singe,
        veil,
        ..
    } = p.mechanic
    else {
        return false;
    };
    // The Veil's clock runs whatever the bars are doing.
    if let Mechanic::Meter { veil: v, .. } = &mut p.mechanic {
        *v = veil.saturating_sub(1);
    }

    if ascending > 0 {
        // The clock. Health is the cost and nothing steers the bars: they are
        // not the resource while it runs, and they are what she gets back --
        // empty -- when it ends.
        singe_health(p, t::ascension_drain().max(1));
        let left = ascending - 1;
        if let Mechanic::Meter { ascending, .. } = &mut p.mechanic {
            *ascending = left;
        }
        if left == 0 {
            end_ascension(p, landed);
        }
        return false;
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
    let ascends = tier_of_bar(dark.min(light)) == Tier::Wings;
    if let Mechanic::Meter {
        dark: d,
        light: l,
        ascending,
        landed,
        singe: s,
        ..
    } = &mut p.mechanic
    {
        *d = dark;
        *l = light;
        *ascending = if ascends { t::ascension_frames() } else { 0 };
        *landed = 0;
        *s = singe;
    }
    ascends
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
    let drained = (t::ascension_drain().max(1) * t::ascension_frames() as i32).max(1);
    let refunded = (landed as i32 * t::ascension_refund()).min(drained);
    let share = Fx::ratio(refunded, drained);
    let ceiling = Fx::from_int(t::ascension_stun() as i32);
    let floor = Fx::from_int(t::ascension_stun_floor() as i32).min(ceiling);
    let stun = ceiling.sub(ceiling.sub(floor).mul(share)).to_int().max(0) as u16;
    if let Mechanic::Meter {
        dark,
        light,
        ascending,
        jumped,
        landed,
        singe,
        ..
    } = &mut p.mechanic
    {
        *dark = Fx::ZERO;
        *light = Fx::ZERO;
        *ascending = 0;
        *jumped = false;
        *landed = 0;
        *singe = Fx::ZERO;
    }
    p.stun_total = stun;
    p.action = Action::Stagger { left: stun };
}

/// Throwing a move moves a bar: an auto or a takeoff fills its own, a spell
/// or a long hold vents its own, and a major spends a great deal of its own.
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
        ..
    } = p.mechanic
    else {
        return;
    };
    // Nothing steers during ascension. The bars are not the resource then;
    // the clock is.
    if ascending > 0 {
        return;
    }
    // **Autos fill, spells vent** (2026-10-12, the second pass's second
    // day): the wings and the moves that leave the floor are how she builds,
    // each goading the bar of its force by `meter, an auto moves` (twilight
    // both at once, so it never widens the gap); a spell vents its own bar
    // by `meter, a spell vents`, a long hold by `meter, a long hold vents`
    // (the twilight one both bars alike); and a major spends `meter, a major
    // spends` of its own -- worth the bar it was thrown from, since
    // `Player::thrown_at` was read before this, and the way back into the
    // band when one bar has run away. The force thrown is the one she is
    // carrying after; twilight leaves that as it was.
    use crate::moves::dual as d;
    let builds =
        d::is_an_auto(kind) || d::is_takeoff(kind) || (d::twilight(kind) && !d::is_a_vent(kind));
    let by = if builds {
        t::meter_auto_push()
    } else if d::is_the_finisher(kind) {
        -t::meter_finisher_push()
    } else if d::is_a_vent(kind) {
        -t::hold_vents()
    } else {
        -t::meter_cast_push()
    };
    let goad = |bar: Fx, by: i32| bar.add(Fx::from_int(by)).clamp(Fx::ZERO, top());
    let (mut dark, mut light, colour) = match d::force(kind) {
        Some(Force::Dark) => (goad(dark, by), light, Force::Dark),
        Some(Force::Light) => (dark, goad(light, by), Force::Light),
        None => (goad(dark, by), goad(light, by), colour),
    };
    // **Equinox levels her** (2026-10-12): the one spell aimed at her own
    // runaway. What it moves goes from the higher bar to the lower, and never
    // more than half the gap, so it can close the gap and cannot cross it.
    if kind == crate::moves::dual::EQUINOX {
        let half = dark.sub(light).abs().mul(Fx::ratio(1, 2));
        let by = Fx::from_int(t::equinox_levels()).min(half);
        if dark.raw() >= light.raw() {
            dark = dark.sub(by);
            light = light.add(by);
        } else {
            light = light.sub(by);
            dark = dark.add(by);
        }
    }
    if let Mechanic::Meter {
        dark: d,
        light: l,
        colour: c,
        ..
    } = &mut p.mechanic
    {
        *d = dark;
        *l = light;
        *c = colour;
    }
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
    // A twilight move is both forces at once, and is worth her lower bar:
    // the balance is what it spends.
    let bar = match kind {
        Some(kind) if crate::moves::dual::twilight(kind) => dark.min(light),
        _ => match kind.and_then(crate::moves::dual::force).unwrap_or(colour) {
            Force::Dark => dark,
            Force::Light => light,
        },
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

/// Feet on the floor: the second jump is back, and so is the air hop.
pub fn feet_down(p: &mut Player) {
    if let Mechanic::Meter { jumped, hopped, .. } = &mut p.mechanic {
        *jumped = false;
        *hopped = false;
    }
}

/// Does an auto thrown in the air rise like a jump? Once an airtime, and every
/// time while ascending. See `state::hop_after_the_auto`.
pub fn may_air_hop(p: &Player) -> bool {
    match p.mechanic {
        Mechanic::Meter {
            ascending, hopped, ..
        } => ascending > 0 || !hopped,
        _ => false,
    }
}

/// Spend the air hop.
pub fn spend_air_hop(p: &mut Player) {
    if let Mechanic::Meter { hopped, .. } = &mut p.mechanic {
        *hopped = true;
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

// ---------------------------------------------------------------------------
// The curse, and the purge
// ---------------------------------------------------------------------------
//
// Her spellbook (2026-10-12): every dark spell that lands lays stacks of a
// curse on the body, up to `curse, most stacks`; they tick, drain to her, and
// fade one at a time on one clock. A light spell that lands on a curse
// **purges** it -- every stack at once, each worth more than the last. So the
// curse is built by one hand and cashed by the other, and both bars rise. See
// `docs/design/exploration/0015_dual_mage_spellbook.md` §"The two things she
// leaves in the world".
//
// A curse is one byte on the body: the stacks in the low three bits, and
// whose curse it is -- the seat of the mage who laid it -- in the top one.
// The same byte the hex was, because the snapshot is at its cap.

/// No curse on a body.
pub const NO_CURSE: u8 = 0;

/// How many stacks a curse byte holds.
pub const fn stacks(curse: u8) -> u8 {
    curse & 0x07
}

/// Whose curse it is: the seat of the mage who laid the last stack.
pub const fn cursed_by(curse: u8) -> usize {
    (curse >> 7) as usize
}

/// A curse byte of `n` stacks, laid by seat `by`.
pub const fn curse_of(n: u8, by: usize) -> u8 {
    if n == 0 {
        NO_CURSE
    } else {
        (n & 0x07) | (((by & 1) as u8) << 7)
    }
}

/// Lay `n` more stacks of `by`'s curse on a body, capped, and start its clock
/// again. A curse is one mage's: another's stacks replace it. Returns the
/// stacks it holds now.
pub fn lay_curse(curse: &mut u8, clock: &mut u16, by: usize, n: u8) -> u8 {
    if n == 0 {
        return stacks(*curse);
    }
    let had = if stacks(*curse) > 0 && cursed_by(*curse) == by {
        stacks(*curse)
    } else {
        0
    };
    let now = had.saturating_add(n).min(t::curse_most().clamp(1, 7));
    *curse = curse_of(now, by);
    *clock = t::curse_fades();
    now
}

/// One frame of a curse's clock: when it runs out one stack goes, and the
/// clock starts again for the rest.
pub fn fade_curse(curse: &mut u8, clock: &mut u16) {
    if stacks(*curse) == 0 {
        *clock = 0;
        return;
    }
    *clock = clock.saturating_sub(1);
    if *clock == 0 {
        let left = stacks(*curse) - 1;
        *curse = curse_of(left, cursed_by(*curse));
        if left > 0 {
            *clock = t::curse_fades();
        }
    }
}

/// What purging `n` stacks is worth, as a multiple of `purge, damage per
/// stack`: `n(1 + c·n)`, so the fifth is worth the most. Convex for the
/// Reaver's reason: building to the cap and then cashing has to be the
/// biggest thing the class does, and "one more, or now" has to be a real
/// question.
pub fn purge_multiple(n: u8) -> Fx {
    let n = Fx::from_int(n as i32);
    n.mul(Fx::ONE.add(t::purge_curve().mul(n)))
}

/// What a purge of `n` stacks deals, thrown at `power`.
pub fn purge_damage(n: u8, power: Fx) -> i32 {
    Fx::from_int(t::purge_damage())
        .mul(purge_multiple(n))
        .mul(power)
        .to_int()
        .max(0)
}

/// Does a purge of `n` stacks stagger a fighter?
pub fn purge_staggers(n: u8) -> bool {
    n > 0 && n >= t::purge_stagger_from().max(1)
}

/// What one tick of `n` stacks deals, at the power of whoever laid them.
pub fn curse_tick_damage(n: u8, power: Fx) -> i32 {
    Fx::from_int(t::curse_tick() * n as i32)
        .mul(power)
        .to_int()
        .max(0)
}

/// Is this the frame a curse ticks, and a sigil burns? One clock for the whole
/// world, on the frame counter, so every curse ticks together and rollback
/// reproduces it exactly.
pub fn ticks_on(frame: u32) -> bool {
    frame % (t::curse_every().max(1) as u32) == 0
}

/// What a curse's tick is worth: the Dark power of whoever laid it, read live,
/// so a curse laid from an empty bar grows if she feeds her dark hand.
pub fn curse_power(p: &Player) -> Fx {
    depth_at(p, Some(crate::moves::dual::DARK_AUTO))
}

// ---------------------------------------------------------------------------
// The hop after an auto
// ---------------------------------------------------------------------------
//
// Since the second pass of 2026-10-12 her autos carry her: after each one she
// hops a short way in the direction she is holding -- Kalista's step -- and up
// a little, more the fuller her lower bar. So a flurry of hits lifts her off
// the floor as the beings inside her wake, which is the class's fantasy in the
// owner's words: "a flurry of hits that come from the beings inside them,
// slowly being lifted off the ground as they dash about". See
// `docs/design/exploration/0016_dual_mage_controls.md`.

/// How high an auto's hop lifts her, in metres: `auto, hop lifts` at an empty
/// lower bar, and `auto, hop lifts at full` more at a full one, in proportion
/// between. The lower bar because it is the frenzy -- what her body can do --
/// and both beings have to be awake for her to rise.
pub fn hop_lift(p: &Player) -> Fx {
    t::auto_hop_lift().add(t::auto_hop_lift_full().mul(balance(p)))
}

// ---------------------------------------------------------------------------
// The Veil and the sigil's ward
// ---------------------------------------------------------------------------

/// She rises as shadow: untouchable for `veil, untouchable for`.
pub fn veil(p: &mut Player) {
    if let Mechanic::Meter { veil, .. } = &mut p.mechanic {
        *veil = t::veil_for();
    }
}

/// Untouchable for `frames`, as a Veil is, or longer if one already runs:
/// the Flurry dash's run down its own line.
pub fn veil_for(p: &mut Player, frames: u8) {
    if let Mechanic::Meter { veil, .. } = &mut p.mechanic {
        *veil = (*veil).max(frames);
    }
}

/// Is she inside a Veil right now? Read by `state::Player::untouchable`.
pub fn veiled(p: &Player) -> bool {
    matches!(p.mechanic, Mechanic::Meter { veil, .. } if veil > 0)
}

/// Is she standing in her own sigil this frame?
pub fn warded(p: &Player) -> bool {
    matches!(p.mechanic, Mechanic::Meter { warded: true, .. })
}

/// The world says, once a frame, whether she stands in one of her sigils.
pub fn set_warded(p: &mut Player, now: bool) {
    if let Mechanic::Meter { warded, .. } = &mut p.mechanic {
        *warded = now;
    }
}

/// A wound, as her ward lets it through: cut by `sigil, wards her by` while
/// she stands in her own sigil, whole otherwise and for anybody else.
pub fn through_the_ward(p: &Player, damage: i32) -> i32 {
    if damage <= 0 || !warded(p) {
        return damage;
    }
    let kept = 100 - t::sigil_wards().clamp(0, 100);
    damage * kept / 100
}

// ---------------------------------------------------------------------------
// The majors grow along their bar
// ---------------------------------------------------------------------------

/// How far up its curve a power is, nought at `depth_floor` to one at
/// `depth_ceiling`: the share of the bar a move was thrown from, read back
/// off the power it carries.
pub fn share_of(power: Fx) -> Fx {
    let span = t::depth_ceiling().sub(t::depth_floor());
    if span.raw() <= 0 {
        return Fx::ONE;
    }
    power
        .sub(t::depth_floor())
        .div(span)
        .clamp(Fx::ZERO, Fx::ONE)
}

/// How big a major arrives, as a multiple of its row, from the power it was
/// thrown with: `majors, size at empty` to `majors, size at full`, a steeper
/// line than the depth curve, so a major from an empty bar is a modest spell
/// and one from a full bar the biggest thing on the screen.
pub fn major_size(power: Fx) -> Fx {
    crate::math::lerp(t::major_size_empty(), t::major_size_full(), share_of(power))
}

/// How many columns Judgement brings down at this power: one from empty, up
/// to `judgement, columns at full`.
pub fn judgement_columns(power: Fx) -> u8 {
    let most = t::judgement_columns().max(1);
    let extra = Fx::from_int((most - 1) as i32).mul(share_of(power));
    // Rounded, so the second column arrives a quarter of the way up rather
    // than only at full.
    1 + extra
        .add(Fx::ratio(1, 2))
        .to_int()
        .clamp(0, (most - 1) as i32) as u8
}

/// How level she is: her lower bar as a share of full, nought to one. What
/// Equinox's height reads.
pub fn balance(p: &Player) -> Fx {
    lower(p).div(top()).clamp(Fx::ZERO, Fx::ONE)
}
