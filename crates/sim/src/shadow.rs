//! The Reaver's second body.
//!
//! **The shadow is never absent.** It is attending her — a translucent copy a
//! step behind, repeating what she does a beat late — or it is out on the
//! field. Everything the class has is a function of the line between the two,
//! and a line needs both ends: the old design let the shadow be *nowhere*, so
//! half a match was spent with the class's whole vocabulary greyed out, and
//! every ability that read the mechanic carried a branch for the case where it
//! did not exist.
//!
//! ## What lives here
//!
//! Where the shadow is, what it is doing, and what it copies. What it *hits* is
//! [`crate::state`]'s, because landing a blow needs both fighters and the
//! creature — see `World::step_shadows`.
//!
//! ## Three rules worth knowing
//!
//! **The flight out is a function of age.** `Ghost::Casting` stores where it
//! left, where it is going and how many frames ago it started, and the position
//! is worked out from those every frame. Same rule as [`crate::effects`], same
//! reason: a rollback landing in the middle of a flight reproduces it exactly
//! rather than re-integrating it and finishing somewhere near.
//!
//! **The way home is a speed, not a curve.** The return chases her, and she
//! moves, so there is no fixed distance to be a fraction of. It is still pure —
//! the position it starts from is in the snapshot and every step is the same
//! arithmetic — but it is written as a chase because that is what it is.
//!
//! **The echo is two numbers.** Which move she threw, and how many frames ago.
//! The shadow's own startup, active and recovery are *derived* from those
//! against the same move table she used, so there is no second state machine to
//! disagree with hers and nothing to restore on a rollback beyond a `u8` and a
//! `u16`.

use crate::DT;
use crate::aim;
use crate::class::{Class, Ghost, Mechanic, NO_ECHO, Shadow};
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::{self, V3};
use crate::moves;
use crate::state::{Action, Player};
use crate::tuning as t;

/// This fighter's shadow, if this fighter is the one that has one.
pub fn of(p: &Player) -> Option<Shadow> {
    match p.mechanic {
        Mechanic::Shadow(shadow) => Some(shadow),
        _ => None,
    }
}

fn put(p: &mut Player, shadow: Shadow) {
    p.mechanic = Mechanic::Shadow(shadow);
}

/// Where the attending shadow wants to stand: behind her, at her shoulder.
///
/// Behind rather than on top of her so that the two bodies can be told apart at
/// a glance, which matters more than it sounds — the copy swings too, and a
/// player has to be able to see which of the two threats is which.
fn heel(p: &Player) -> V3 {
    p.pos
        .sub(V3::new(p.facing.x, Fx::ZERO, p.facing.z).scale(t::shadow_trail()))
}

// ---------------------------------------------------------------------------
// The tick
// ---------------------------------------------------------------------------

/// Move the shadow, age its echo, and answer the leash.
///
/// Called from `state::step_mechanic`, which is the one place per-frame
/// mechanic upkeep happens for every class.
pub fn step(p: &mut Player) {
    let Some(mut shadow) = of(p) else { return };

    // It turns the way she turns. It is her shadow: it does what she does, and
    // that includes which way it is pointed, which is what makes its copy of a
    // swing come out along the same line hers did.
    //
    // **Except out on the field, in the middle of a copy.** There it has turned
    // to whoever is in reach -- see `copy_needs_a_target` -- and it holds that
    // until the copy is over. Eased back towards her mid-swing, the arc would
    // bend round to point wherever her shoulders are, which is the miss the
    // turn exists to fix.
    if !fighting_out_there(shadow) {
        shadow.facing = ease_to(shadow.facing, p.facing).normalized();
    }

    match shadow.doing {
        Ghost::Attending => {
            // Eased rather than pinned, so it swings out wide behind her when
            // she turns and drifts back in when she stops. A shadow welded to a
            // fixed offset reads as a decal on the floor.
            shadow.pos = ease_to(shadow.pos, heel(p));
        }
        Ghost::Casting { from, to, age } => {
            let span = t::shadow_send_frames();
            // The frame it *will* be on, not the one it was: a flight whose
            // first step is a no-op reads as a hitch before the throw.
            let next = age.saturating_add(1);
            shadow.pos = math::lerp3(
                from,
                to,
                math::ease_out(Fx::ratio(next as i32, span as i32)),
            );
            // Out fast, and then it simply stops. The pause at the end of the
            // throw is the whole point of the ability: what the rest of the kit
            // is aimed at is a second body standing still somewhere useful.
            shadow.doing = if next >= span {
                shadow.pos = to;
                Ghost::Waiting
            } else {
                Ghost::Casting {
                    from,
                    to,
                    age: next,
                }
            };
        }
        Ghost::Waiting => {
            // The leash. Walk out of it and the shadow comes and finds you,
            // cutting whatever is between the two of you -- which is the
            // mechanic's own description of itself, and the reason straying is
            // a decision rather than a mistake.
            if shadow.pos.sub(p.pos).flat_len().raw() > t::shadow_leash().raw() {
                shadow.doing = Ghost::Returning { struck: 0 };
            }
        }
        Ghost::Returning { struck } => {
            let home = heel(p);
            let gap = home.sub(shadow.pos);
            let step = t::shadow_home_speed().mul(DT);
            if gap.len().raw() <= step.raw() {
                shadow.pos = home;
                shadow.doing = Ghost::Attending;
            } else {
                shadow.pos = shadow.pos.add(gap.normalized().scale(step));
                shadow.doing = Ghost::Returning { struck };
            }
        }
    }

    age_the_echo(p, &mut shadow);
    put(p, shadow);
    step_her_dash(p);
}

/// Out on the field and copying a swing: the one time the shadow is pointed by
/// something other than her.
fn fighting_out_there(shadow: Shadow) -> bool {
    matches!(shadow.doing, Ghost::Waiting) && shadow.echo != NO_ECHO
}

/// A step of the ease every following thing here uses.
fn ease_to(from: V3, to: V3) -> V3 {
    from.add(to.sub(from).scale(t::shadow_follow()))
}

// ---------------------------------------------------------------------------
// The echo
// ---------------------------------------------------------------------------

/// She threw a move; the shadow throws the same one, `shadow_lag` frames later.
///
/// **Swings only.** The other two things she can throw are already the
/// shadow's: the Guillotine erupts *at* it and the mechanic move *is* it, and a
/// copy of either would be the same ability fired twice from the same place.
pub fn begin_echo(p: &mut Player, kind: u8) {
    let Some(mut shadow) = of(p) else { return };
    if moves::get(p.class, kind).aim() != aim::Kind::Swing {
        return;
    }
    shadow.echo = kind;
    shadow.echo_age = 0;
    shadow.echo_used = false;
    put(p, shadow);
}

/// Where the shadow is in the move it is copying, or `None` when it is copying
/// nothing.
///
/// Derived rather than stored. The three phases come out of the same move table
/// hers do, so retuning a startup in the Oven moves both bodies at once and
/// there is no second machine to fall out of step.
pub fn echo_action(class: Class, shadow: Shadow) -> Option<Action> {
    if shadow.echo == NO_ECHO {
        return None;
    }
    let elapsed = shadow.echo_age.checked_sub(t::shadow_lag())?;
    let m = moves::get(class, shadow.echo);
    let kind = shadow.echo;
    let active_ends = m.startup + m.active;
    if elapsed < m.startup {
        Some(Action::Startup {
            kind,
            left: m.startup - elapsed,
        })
    } else if elapsed < active_ends {
        Some(Action::Active {
            kind,
            left: active_ends - elapsed,
        })
    } else if elapsed < active_ends + m.recovery {
        Some(Action::Recovery {
            kind,
            left: active_ends + m.recovery - elapsed,
        })
    } else {
        None
    }
}

fn age_the_echo(p: &Player, shadow: &mut Shadow) {
    if shadow.echo == NO_ECHO {
        return;
    }
    shadow.echo_age = shadow.echo_age.saturating_add(1);
    // Still waiting to begin. `echo_action` says "nothing out" both before the
    // copy starts and after it finishes, and only the second of those means the
    // echo is over -- reading the first as the end cancelled every copy on the
    // frame after it was scheduled.
    if shadow.echo_age < t::shadow_lag() {
        return;
    }
    // The copy is a swing, and a swing lands once. The flag is cleared on the
    // frame the volume appears rather than when the move starts, so a shadow
    // that caught somebody during a previous copy does not carry that over.
    match echo_action(p.class, *shadow) {
        Some(Action::Active { left, .. }) if left == moves::get(p.class, shadow.echo).active => {
            shadow.echo_used = false;
        }
        None => {
            shadow.echo = NO_ECHO;
            shadow.echo_age = 0;
        }
        _ => {}
    }
}

/// The shadow as a body, for the one frame's worth of questions that need one.
///
/// A whole `Player` rather than a bespoke shape, and that is the point: the hit
/// test, the debug overlay and the renderer all ask *the same* functions about
/// it that they ask about her — `state::hitbox` above all, which is the one
/// description of an attack's volume. A second, smaller struct would mean a
/// second description of the same swing, and the two would disagree the first
/// time either was touched.
///
/// Her aim path is **carried over** to the shadow rather than recomputed: the
/// copy is her swing thrown from somewhere else, so it keeps her line -- the
/// pitch she committed to, the shape, the frames. At her heel it keeps her yaw
/// too, and is a plain translation. **Out on the field it keeps the yaw the
/// shadow turned to**, which is whoever was in reach when the copy wound up --
/// see [`aim::shadow_faces`] and [`aim::copied_swing`].
pub fn echo_body(p: &Player) -> Option<Player> {
    let shadow = of(p)?;
    let action = echo_action(p.class, shadow)?;
    if p.health <= 0 {
        return None;
    }
    let yaw = if matches!(shadow.doing, Ghost::Waiting) {
        shadow.facing
    } else {
        p.facing
    };
    Some(Player {
        pos: shadow.pos,
        vel: V3::ZERO,
        facing: shadow.facing,
        action,
        hit_used: shadow.echo_used,
        grounded: true,
        crouching: false,
        aim_path: aim::copied_swing(p.aim_path, p.pos, p.facing, shadow.pos, yaw),
        ..*p
    })
}

/// If the shadow is out on the field and winding up a copy, where it is and how
/// far the copy reaches -- the two things [`aim::shadow_faces`] needs to pick a
/// target. `None` the rest of the time.
///
/// Through the wind-up rather than on one frame of it, so a target that steps
/// across it during the startup is followed the way she would follow them
/// herself. The moment the copy is out it holds, because a swing that turned
/// to track somebody mid-arc would be homing.
pub fn copy_needs_a_target(p: &Player) -> Option<(V3, Fx)> {
    let shadow = of(p)?;
    if !fighting_out_there(shadow) || p.health <= 0 {
        return None;
    }
    match echo_action(p.class, shadow)? {
        Action::Startup { kind, .. } => Some((shadow.pos, moves::get(p.class, kind).reach)),
        _ => None,
    }
}

/// Point the shadow. Called with whatever [`aim::shadow_faces`] answered, or her
/// own facing when it answered nothing.
pub fn turn_to(p: &mut Player, facing: V3) {
    let Some(mut shadow) = of(p) else { return };
    shadow.facing = facing;
    put(p, shadow);
}

/// Mark the copy as having connected, so it lands once.
pub fn echo_landed(p: &mut Player) {
    let Some(mut shadow) = of(p) else { return };
    shadow.echo_used = true;
    put(p, shadow);
}

// ---------------------------------------------------------------------------
// The press that orders it
// ---------------------------------------------------------------------------
//
// Right click is the one button in this kit that is not an attack, and it is
// read differently from the rest for that reason. Two rules, and both were
// paid for:
//
// **It is a press, not a held button.** Held, it used to re-fire every time she
// came free -- send, recall, send -- so the mechanic's position became a
// function of how long a finger stayed down. That is the same bug `E` had on
// every class before it grew an edge, and it came back the day the Reaver's
// mechanic moved onto the mouse, because a click has no edge of its own here.
//
// **The press outlives the frame it happened on.** An attack that lands on a
// busy frame is correctly eaten -- the game is telling you that you were busy.
// The shadow is not an attack: it is where the second body stands, which is
// the class's escape from both the ordinary limits on where she can be and the
// ordinary limits on what she can reach. A press that only answers on the one
// frame in twenty when she happens to be free is a timing test standing in
// front of the mechanic. So it is remembered, briefly -- see
// `tuning::shadow_buffer` -- and spent on the first frame she can take it.

/// Read the right click, and remember it for a few frames if she cannot act on
/// it yet.
///
/// Called once per fighter per frame, from both input paths. The edge lives on
/// the fighter rather than in a renderer-side "just pressed" for the reason
/// every other edge here does: rollback re-runs these frames, so an edge
/// remembered outside the snapshot is an edge that disappears the first time a
/// frame is replayed.
pub fn queue_order(p: &mut Player, input: Input) {
    let pressed = input.has(Input::RIGHT) && !p.right_held;
    p.right_held = input.has(Input::RIGHT);
    // **Being hit throws the press away.** The memory exists so that her own
    // kit cannot eat the mechanic; a stun is not her own kit, it is the
    // opponent's reward and the one thing in the game that is meant to take the
    // controls off you. Kept, a press from before the hit would fire the moment
    // she recovered -- sending the second body away on the frame she is most
    // likely to want it, and doing it on an input she gave in a situation that
    // no longer exists.
    if p.action.stunned() {
        p.shadow_queued = 0;
        return;
    }
    // Armed only for the fighter who has a shadow to order about. The edge
    // above is read for everybody, so that picking the class mid-match starts
    // from a button that is down rather than from a press that never happened.
    if pressed && of(p).is_some() {
        p.shadow_queued = t::shadow_buffer();
    } else {
        p.shadow_queued = p.shadow_queued.saturating_sub(1);
    }
}

/// Is there a right click waiting to be spent?
pub fn order_queued(p: &Player) -> bool {
    p.shadow_queued > 0
}

/// Spend it, on the frame Send shadow actually comes out.
///
/// A no-op for every other move and every other class. It has to be spent
/// rather than left to expire: the memory is several frames long and Send
/// shadow's own startup is shorter than that, so a press left armed would
/// order the shadow again the moment the send could be cancelled -- which is
/// the held-button bug with extra steps.
pub fn spend_order(p: &mut Player, kind: u8) {
    if kind == crate::state::SLOT_MECHANIC && of(p).is_some() {
        p.shadow_queued = 0;
    }
}

// ---------------------------------------------------------------------------
// Sending it, and getting it back
// ---------------------------------------------------------------------------

/// What pressing `E` does, given where the shadow already is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Order {
    /// It was with her, and now it is going out.
    Sent,
    /// It was out, and now it is coming home through anybody in the way.
    Recalled,
}

/// Throw the shadow at `to`, or call it home if it is already out.
///
/// One button, two meanings, decided by where the second body is — the same
/// shape as the Bulwark's shield, and for the same reason: the mechanic has a
/// position, so the only thing the key can mean is "change it".
pub fn order(p: &mut Player, to: V3) -> Option<Order> {
    let mut shadow = of(p)?;
    let order = match shadow.doing {
        Ghost::Attending => {
            shadow.doing = Ghost::Casting {
                from: shadow.pos,
                to,
                age: 0,
            };
            Order::Sent
        }
        // Already on its way home. Pressing again does not hurry it, but the
        // press is still a recall as far as everything downstream is concerned
        // -- the lotus reads the order, not the state.
        Ghost::Returning { .. } => Order::Recalled,
        Ghost::Casting { .. } | Ghost::Waiting => {
            shadow.doing = Ghost::Returning { struck: 0 };
            Order::Recalled
        }
    };
    put(p, shadow);
    Some(order)
}

/// Has the return already caught this fighter?
pub fn already_cut(shadow: Shadow, victim: usize) -> bool {
    match shadow.doing {
        Ghost::Returning { struck } => struck & (1 << victim) != 0,
        _ => false,
    }
}

/// Remember that the return caught this fighter, so it cuts them once.
pub fn mark_cut(p: &mut Player, victim: usize) {
    let Some(mut shadow) = of(p) else { return };
    if let Ghost::Returning { struck } = shadow.doing {
        shadow.doing = Ghost::Returning {
            struck: struck | (1 << victim),
        };
        put(p, shadow);
    }
}

// ---------------------------------------------------------------------------
// Her dash to it
// ---------------------------------------------------------------------------

/// Should a dodge thrown *now* be a dash to the shadow instead?
///
/// Forward, the crosshair on the shadow, and a way through to it. The first two
/// are what make it a decision: forward is what keeps the other three dodges as
/// dodges, and the crosshair is what stops the dash happening to her whenever
/// the shadow is roughly ahead. The third is the world's answer rather than
/// hers -- see [`aim::clear_between`], and note how blunt it is. Nothing
/// *partial* stops a dash. A ledge, a lip, a stone she would have to go round:
/// she goes over or past all of them, because the dash is a straight line to
/// wherever the second body is standing and the only thing that can refuse it
/// is having no line at all.
///
/// Both questions are asked of [`crate::aim`] rather than answered here, which
/// is the standing rule about who decides where something goes. The angle
/// worked out beside the dodge, and the flat distance measured beside it, are
/// the same mistake twice.
///
/// Refused, she gets the ordinary dodge. The class's mobility and the universal
/// defensive option are the same button on purpose, so the failure case is the
/// other half of the button rather than a dead press.
pub fn dash_is_asked_for(
    p: &Player,
    who: usize,
    input: crate::input::Input,
    forward: bool,
    scene: &aim::Scene,
) -> bool {
    let Some(shadow) = of(p) else { return false };
    forward
        && shadow.is_out()
        && aim::pointing_at(who, input, shadow.pos, t::shadow_lock_cone(), scene)
        && aim::clear_between(p.pos, shadow.pos, scene)
}

/// Start the dash. It rides along with `Action::Dodge`, which is where the
/// invulnerability comes from.
pub fn begin_dash(p: &mut Player) {
    let Some(mut shadow) = of(p) else { return };
    shadow.dash = t::dodge_frames();
    put(p, shadow);
}

/// The velocity her dash is driving this frame, or `None` if she is not on one.
///
/// A constant speed rather than a decaying shove, unlike every other dodge.
/// The dash has somewhere to *be*, and a dodge that decays covers a distance
/// that depends on the tuning of the decay -- so a dash built out of one either
/// falls short of the shadow or overshoots it, and which of those it does
/// changes every time somebody drags a slider.
///
/// **All three axes.** The dash goes to where the shadow *is*, not to the patch
/// of floor underneath it: a shadow standing on a dais is up there, and a dash
/// that drove only `x` and `z` walked her into the side of it and stopped. The
/// caller turns gravity and the arena off for the duration -- see
/// `state::step_player` -- so the line she flies is the line this returns.
pub fn dash_drive(p: &Player) -> Option<V3> {
    let shadow = of(p)?;
    if shadow.dash == 0 {
        return None;
    }
    Some(
        shadow
            .pos
            .sub(p.pos)
            .normalized()
            .scale(t::shadow_dash_speed()),
    )
}

/// Is the carry live: has a dash just arrived, with its speed still under her?
///
/// The window a jump can be thrown into. See [`Shadow::carry`].
pub fn carrying_a_dash(p: &Player) -> bool {
    of(p).is_some_and(|shadow| shadow.carry > 0)
}

/// Is she asking to swing out of the carry?
///
/// **The cash-in is a strike on arrival**, and this is what makes it one. The
/// dash leaves her sliding at the speed she crossed at, a few metres past the
/// shadow, and a swing that had to wait for the dodge's tail to run out came
/// out that far from anybody standing beside it: the whole pattern -- send,
/// mark, cross, cash -- missed at the last step, every time, in `tally`. So a
/// swing pressed inside the carry cuts the tail short the way a jump does,
/// and comes out with her still on the shadow.
///
/// **The jump keeps the slide and the swing spends it.** The jump out of the
/// carry is for going somewhere, so it takes the speed up with her; the swing
/// is for striking *here*, so she plants -- and turns to the crosshair, the way
/// any fighter free to act does, since the dodge she is cutting short had her
/// facing fixed along the dash.
///
/// It buys no safety. The tail it cuts is traded for the swing's own frames,
/// and a blocked Slash is still minus five.
pub fn swing_out_of_the_carry(p: &mut Player, input: Input) -> bool {
    if !(carrying_a_dash(p) && (input.has(Input::LEFT) || input.has(Input::MECHANIC))) {
        return false;
    }
    p.action = Action::Free;
    p.vel = V3::new(Fx::ZERO, p.vel.y, Fx::ZERO);
    p.facing = V3::from_turns(input.aim_turns());
    spend_carry(p);
    true
}

/// Spend the carry on a jump, so it pays for one takeoff and not two.
pub fn spend_carry(p: &mut Player) {
    let Some(mut shadow) = of(p) else { return };
    shadow.carry = 0;
    put(p, shadow);
}

/// Being hit ends the dash and the carry both.
///
/// The same rule the Champion's Rush has, and for the same reason: a commitment
/// you can be hit out of and keep is a commitment with invulnerability attached.
pub fn broken_by_a_hit(p: &mut Player) {
    let Some(mut shadow) = of(p) else { return };
    shadow.dash = 0;
    shadow.carry = 0;
    put(p, shadow);
}

/// Count the dash down, and end it on arrival.
///
/// Arriving **collects** the shadow. Going and getting it is the other half of
/// throwing it out: the loop the class plays is send, act off the line, dash
/// back onto it, send again -- and a dash that left the shadow standing where
/// she now is would leave the pair of them in the same place with the line
/// between them gone and no way to say so.
fn step_her_dash(p: &mut Player) {
    let Some(mut shadow) = of(p) else { return };
    // The carry runs down whether or not a dash is still going: it is what a
    // dash leaves behind, and the frame it was opened on is one of its own.
    // The cash-in window is the same kind of thing and runs down beside it.
    // See [`Shadow::cash`].
    shadow.carry = shadow.carry.saturating_sub(1);
    shadow.cash = shadow.cash.saturating_sub(1);
    if shadow.dash == 0 {
        put(p, shadow);
        return;
    }
    shadow.dash -= 1;
    // Within a body, or within one frame's worth of travel -- whichever is
    // larger. A tolerance smaller than the step would let her cross the shadow
    // and turn round to come back at it. Measured in three dimensions now that
    // the dash flies in three: a flat gap reads a shadow a storey up as
    // arrived at, from the floor below it.
    let close = t::body_radius().max(t::shadow_dash_speed().mul(DT));
    let arrived = shadow.pos.sub(p.pos).len().raw() <= close.raw();
    // Ends on arrival, and ends anyway the moment the dodge does -- getting
    // hit out of it, or simply running out of frames. The dash is the dodge; it
    // does not outlive it.
    if arrived || !matches!(p.action, Action::Dodge { .. }) {
        shadow.dash = 0;
    }
    if arrived {
        // **She lands on it, not near it.** The tolerance above is half a metre
        // wide and the shadow is standing somewhere she can stand, so closing
        // the last of the gap outright is both the honest reading of "the dash
        // brings her to the shadow" and what makes arriving on a dais put her
        // on the deck rather than half a metre short of the lip, where the
        // arena would push her off again.
        p.pos = shadow.pos;
        // The line is over, so the world takes her back. Left alone, the rise
        // that carried her up would keep carrying her off the top of it.
        p.vel.y = Fx::ZERO;
        shadow.carry = t::shadow_carry();
        // **And the cash-in window.** The same arrival, and only this one:
        // the recall and the leash bring the shadow home too, and neither is
        // her crossing to it. See [`cash_open`].
        shadow.cash = t::cash_window();
        // The window is the same length however far she came. What is left of
        // the dodge usually *is* that window -- she arrived early and the rest
        // is the slide -- but a dash that spent the whole dodge crossing would
        // leave none, so the dodge is topped up to fit. Never shortened: a
        // short dash keeps the tail it has always had.
        if let Action::Dodge { left } = p.action {
            if left < shadow.carry {
                p.action = Action::Dodge { left: shadow.carry };
            }
        }
        if shadow.is_out() {
            shadow.doing = Ghost::Attending;
        }
    }
    put(p, shadow);
}

// ---------------------------------------------------------------------------
// The tally, and cashing it
// ---------------------------------------------------------------------------
//
// v2 of the damage pattern, 2026-09-23 -- `docs/design/shadow-reaver-v2.md`.
// The shadow already paid movement, damage and a thin slice of utility. This
// makes it pay **burst**: every hit it lands from the field is counted on the
// victim, and crossing to it by dash opens a window in which her first swing
// to connect spends the count. Nothing new is put in the world to get it; the
// marks are a count of what the shadow already does, and the cash-in is a
// multiplier on a swing she already has.

/// Put one mark on a fighter the shadow has just hit from the field.
///
/// Once per victim per *event* -- a copy, a lotus pass, a recall -- and the
/// callers are what make it once: each already remembers whom it has cut.
/// A new mark restarts the fade, so a victim the shadow is still working does
/// not lose the count it is building.
pub fn mark(victim: &mut Player) {
    victim.marks = victim.marks.saturating_add(1).min(t::mark_cap());
    victim.mark_clock = t::mark_fade();
}

/// Run a fighter's marks down: one every `mark_fade` frames since the last
/// mark or the last fade.
pub fn fade_mark(p: &mut Player) {
    if p.marks == 0 {
        p.mark_clock = 0;
        return;
    }
    p.mark_clock = p.mark_clock.saturating_sub(1);
    if p.mark_clock == 0 {
        p.marks -= 1;
        if p.marks > 0 {
            p.mark_clock = t::mark_fade();
        }
    }
}

/// She has just thrown `kind`. If it is a swing and the window is open, this
/// is the swing that cashes, and the window is spent on it; anything she throws
/// afterwards is an ordinary swing.
///
/// The first swing *thrown*, not the first that pays: a Slash whiffed inside
/// the window has had its chance, and so has one thrown at somebody with no
/// marks on them.
pub fn arm_the_cash(p: &mut Player, kind: u8) {
    let Some(mut shadow) = of(p) else { return };
    let swing = moves::get(p.class, kind).aim() == aim::Kind::Swing;
    shadow.cashing = if swing && shadow.cash > 0 {
        shadow.cash = 0;
        kind
    } else {
        NO_ECHO
    };
    put(p, shadow);
}

/// Does the blow she is landing right now cash the tally?
pub fn cashing(p: &Player) -> bool {
    let Some(shadow) = of(p) else { return false };
    shadow.cashing != NO_ECHO && p.action.attack_kind() == Some(shadow.cashing)
}

/// It connected: it has cashed, blocked or not, and cannot again.
pub fn cashed(p: &mut Player) {
    let Some(mut shadow) = of(p) else { return };
    shadow.cashing = NO_ECHO;
    put(p, shadow);
}

/// What a swing that spends `marks` is worth, as a multiple of itself: one,
/// plus `mark_worth` a mark.
pub fn cash_multiple(marks: u8) -> Fx {
    Fx::ONE.add(t::mark_worth().mul(Fx::from_int(marks as i32)))
}

/// Is this a full tally -- the one that also staggers?
pub fn full_tally(marks: u8) -> bool {
    marks >= t::mark_cap()
}
