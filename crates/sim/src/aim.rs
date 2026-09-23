//! Where an ability goes. **The only place in the game that decides.**
//!
//! # The rule
//!
//! The player's whole frame of reference is the crosshair. So:
//!
//! > **Every skillshot starts with one raycast, from the camera through the
//! > crosshair, ignoring anything behind the character model. Take the first
//! > thing it meets. That point is what the player is pointing at, and the
//! > ability goes there.**
//!
//! That ray meets **terrain, structures, and the ability's own max-range
//! sphere** — one list, a property of the world rather than of the ability
//! doing the aiming. Whatever it reaches first wins.
//!
//! **Bodies are not on that list.** Neither other fighters nor the creature.
//! The ray is answering *which place is the player pointing at*, and a body is
//! a thing standing in a place rather than a place of its own; what it is in
//! the way of is a separate question, asked of the path afterwards by
//! [`first_along`]. It reads as a fine distinction and is not: a creature up
//! close fills the screen, so the crosshair sits on its chest several metres
//! up, and an ability that went *there* sailed over it. See [`sight`].
//!
//! # The two kinds, and the two things that are not one
//!
//! ```text
//!   Grounded    the thing lands on the floor       structures, fire pillar
//!   Skillshot   the thing flies through the air    the Elementalist's auto,
//!                                                   the Blood mage's blade
//!   Swing       not aimed at all                   every melee attack
//!   Mechanic    wherever the mechanic is standing  the Reaver's blades
//! ```
//!
//! **Grounded** — [`grounded_path`]:
//!
//! - Hit the ground, and it is cast *exactly* there. Not a pixel different.
//! - Hit the max-range sphere, and it is cast at max range on the ground, in
//!   the direction the mouse is facing.
//! - Anything else — a body, a wall, a stone — drops to whatever is underneath
//!   it, because the thing being placed comes out of the floor.
//! - If it travels, it travels from the character to that point.
//!
//! **Skillshot** — [`skillshot_path`]:
//!
//! - Hit the ground, and the target is that spot raised straight up to the
//!   height the ability leaves the caster at. The shot flies level over the
//!   place the crosshair is on rather than diving into the dirt.
//! - Hit anything else — terrain that is not ground, a structure, or the
//!   max-range sphere — and the target is the point of intersection exactly.
//! - Either way the ability travels in a straight line from the caster to that
//!   point, and that line is its whole reach.
//!
//! **Swing** is not aimed. A sword is a body moving, and pointing the camera at
//! the floor must not put the blade there; a swing comes out along `facing`, at
//! the move's own reach — from one shoulder rather than from the chest if the
//! move is thrown with one arm, which is [`Hand`]. It is in this list so that
//! "which of the four is this move" is a question with an answer for every move
//! rather than a thing each caller decides for itself — see
//! [`crate::moves::Move::aim`].
//!
//! # Why this file exists, and why nothing else may do this
//!
//! Because the two rays are not the same ray. The camera sits behind and above
//! the shoulder, so a ray from the *chest* along the *look direction* is
//! parallel to the crosshair's and never converges with it: the reticle sits on
//! one spot and the ability goes to another, by metres, and the error grows
//! with distance. Every version of this bug has been someone writing their own
//! intersection logic next to the ability that needed it.
//!
//! So: **all ray-against-shape arithmetic lives in [`crate::math`], and all
//! decisions about where an ability goes live here.** `crates/sim/tests/one_aim.rs`
//! fails the build if another module reaches for the primitives directly.
//! Adding an ability means calling [`grounded_path`] or [`skillshot_path`]; if
//! neither fits, change them, and the change is then true of every ability at
//! once.
//!
//! The cost is that this has to know where the eye is, so the camera's geometry
//! is simulation state and its numbers are in the desync checksum. See
//! `crate::camera`, which says what that bought and what it cost.

use crate::arena;
use crate::class::{Mechanic, Structure};
use crate::effects::{EffectKind, Effects};
use crate::fixed::{Fx, cos_turns, sin_turns};
use crate::input::Input;
use crate::math::V3;
use crate::monster::Monster;
use crate::state::{MAX_PLAYERS, Player};
use crate::stones::Field;
use crate::tuning as t;

/// A straight line something follows, from where it leaves to where it ends.
///
/// Both kinds of skillshot produce one, so the thing that travels and the thing
/// the overlay draws are the same object and cannot disagree about direction.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Path {
    pub from: V3,
    pub to: V3,
}

impl Path {
    /// Unit vector along the path. Zero for a path of no length.
    pub fn dir(self) -> V3 {
        self.to.sub(self.from).normalized()
    }

    pub fn length(self) -> Fx {
        self.to.sub(self.from).len()
    }

    /// A point some distance along it.
    pub fn at(self, dist: Fx) -> V3 {
        self.from.add(self.dir().scale(dist))
    }
}

/// Which of the four ways a move is pointed.
///
/// Every move answers this, and [`crate::moves::Move::aim`] is where it is
/// answered. Two of the four are skillshots and go through the raycast above;
/// the others are pointed by something the player decided earlier -- which way
/// their body is facing, or where they put the mechanic.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// Lands on the floor. [`grounded_path`].
    Grounded,
    /// Flies to the point the crosshair is on. [`skillshot_path`].
    Skillshot,
    /// A body moving: out along `facing`, at the move's own reach, tilted by
    /// the camera's pitch outside a dead zone. [`swing_path`].
    Swing,
    /// Wherever the class mechanic is standing. The player aimed when they put
    /// it there. [`mechanic_path`].
    AtTheMechanic,
}

impl Kind {
    /// Does the crosshair's raycast decide where this goes?
    ///
    /// Two of the four. The other two are pointed by something the player
    /// already decided -- which way their body is facing, or where they put the
    /// mechanic -- and consult nothing.
    pub const fn is_a_skillshot(self) -> bool {
        matches!(self, Kind::Grounded | Kind::Skillshot)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Kind::Grounded => "grounded",
            Kind::Skillshot => "skillshot",
            Kind::Swing => "swing",
            Kind::AtTheMechanic => "mechanic",
        }
    }

    /// How the move table stores it. Unknown codes read as a swing, which is
    /// the one that asks nothing of the world and so cannot be wrong by
    /// accident.
    pub const fn from_code(code: u8) -> Kind {
        match code {
            1 => Kind::Grounded,
            2 => Kind::Skillshot,
            3 => Kind::AtTheMechanic,
            _ => Kind::Swing,
        }
    }

    pub const fn code(self) -> u8 {
        match self {
            Kind::Swing => 0,
            Kind::Grounded => 1,
            Kind::Skillshot => 2,
            Kind::AtTheMechanic => 3,
        }
    }
}

/// Everything a ray can meet.
///
/// One bundle rather than an argument each, because "what can be aimed at" is a
/// property of the world: an ability that quietly left bodies out of its own
/// trace would aim through people.
pub struct Scene<'a> {
    pub stones: &'a Field,
    pub players: &'a [Player; MAX_PLAYERS],
    pub effects: &'a Effects,
    pub quarry: Option<&'a Monster>,
}

/// Where a fighter standing at `pos` casts from: the height abilities leave at.
pub fn origin(pos: V3) -> V3 {
    V3::new(pos.x, pos.y.add(t::cast_height()), pos.z)
}

/// The point a shot aimed at a patch of *floor* should actually go to: the
/// middle of a fighter standing on it.
///
/// The floor is never the target. Aiming at somebody puts the crosshair through
/// them and onto the ground behind -- bodies are not on the ray -- so a ground
/// hit means "there", and a shot sent to the dirt at `there` passes under
/// whoever is standing on it.
///
/// **Half a body up from the ground it hit**, not up to the caster's own cast
/// height, which is what this used to be. The two are the same number on flat
/// ground and nothing like it off it: from the top of a dais the caster's cast
/// height is three metres above the arena floor, so every skillshot aimed at
/// somebody below flew level out of the platform and over their head, and the
/// only way to land one was to aim at a patch of floor well in front of them.
/// Measuring from the ground the ray met makes the rule true from any height.
pub fn standing_middle(ground: V3) -> V3 {
    V3::new(
        ground.x,
        ground.y.add(t::body_height().div(Fx::from_int(2))),
        ground.z,
    )
}

/// Which arm a move comes out of.
///
/// Most moves have no answer worth giving -- a two-handed overhead, a gesture
/// that plants something -- and those are [`Hand::Centre`], on the body's own
/// line, which is where every volume in the game sat before this existed. It is
/// here because one class is *built* on the distinction: the Dual mage holds two
/// forces apart, one in each arm, and "left is dark, right is light" is the
/// whole of how the meter is steered. A hitbox on the centre line cannot say
/// which of the two just landed.
///
/// Declared per move in [`crate::moves::hand`], next to the shape, for the same
/// reason the shape is: which arm throws a punch is what the move **is**, not a
/// number to drag.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hand {
    Left,
    Right,
    /// Both, or neither. The body's centre line.
    Centre,
}

impl Hand {
    pub const fn name(self) -> &'static str {
        match self {
            Hand::Left => "left",
            Hand::Right => "right",
            Hand::Centre => "centre",
        }
    }

    /// Which way is *away from the body* for this hand, as a sign against the
    /// strafe-right axis.
    ///
    /// One number, in one place, because everything sided reads it: where the
    /// hand is, and which way a volume thrown from it opens. Two mirrored moves
    /// then share one set of tuned numbers -- the arc of a wing is a magnitude
    /// and this is its direction -- rather than being a sign apart in the Oven,
    /// where a tuner can flip one of them and not the other.
    ///
    /// **Left is negative**, because away-from-the-body on the left hand is the
    /// way opposite to strafing right. The signs were the other way round until
    /// 2026-09-16, to follow a skeleton that was itself mirrored; both are
    /// fixed and `view/src/skeleton.rs` carries the account of how the two
    /// managed to agree with each other while disagreeing with the body.
    pub const fn outward(self) -> i32 {
        match self {
            Hand::Left => -1,
            Hand::Right => 1,
            Hand::Centre => 0,
        }
    }
}

/// A quarter turn. An angle unit, not a quantity.
const QUARTER_TURN: Fx = Fx::from_raw(1 << 14);

/// The unit vector from the body's centre line out to one hand, level.
///
/// Zero for [`Hand::Centre`], which is what makes an unsided move come out of
/// the middle of the chest exactly as it always has.
///
/// **The left hand is where a body's left hand is**: `up` crossed with the
/// facing, which for a fighter looking down `+X` is `-Z`. The renderer's
/// `Joint::ArmL` is drawn there too, and `view/tests/kinematics.rs` fails if the
/// two ever part company.
///
/// That sentence used to read the other way round and be defended at length.
/// The body was authored mirrored -- `Joint::ArmL` at `-X` in a frame where the
/// left arm is at `+X` -- and this function was written to follow it, on the
/// reasoning that what a hitbox and an animation have to agree about is which
/// arm the player can see. The reasoning is right and it was being used to hold
/// two wrongs in place: both frames agreed with each other and both were the
/// mirror of the body, so left click came out of the right hand. Fixed on
/// 2026-09-16 in both places at once; see `view/src/skeleton.rs`.
pub fn across(facing: V3, hand: Hand) -> V3 {
    match hand.outward() {
        0 => V3::ZERO,
        sign => {
            let turn = QUARTER_TURN.mul(Fx::from_int(sign));
            let (c, s) = (cos_turns(turn), sin_turns(turn));
            V3::new(
                facing.x.mul(c).sub(facing.z.mul(s)),
                Fx::ZERO,
                facing.x.mul(s).add(facing.z.mul(c)),
            )
        }
    }
}

/// Where a fighter's hand is: the cast origin, stepped out to that shoulder.
///
/// The height is [`origin`]'s, so a one-armed move leaves from the same level a
/// two-armed one does and only the side changes.
pub fn hand_origin(pos: V3, facing: V3, hand: Hand) -> V3 {
    origin(pos).add(across(facing, hand).scale(t::hand_offset()))
}

// ---------------------------------------------------------------------------
// The one raycast
// ---------------------------------------------------------------------------

/// What the crosshair's ray met.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Met {
    /// A surface you could stand on: the floor, the top of a platform, the top
    /// of a stone. The one case the two kinds of ability treat differently.
    Ground,
    /// Anything else solid — a wall, the side of a platform, the side of a
    /// stone. Not a body: bodies are not on the ray at all.
    Solid,
    /// Nothing at all within the ability's reach, so the max-range sphere.
    Reach,
}

/// Where the crosshair is pointing, and what is there.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sighted {
    pub at: V3,
    /// Distance along the camera's ray, for comparing two candidates.
    pub dist: Fx,
    pub met: Met,
}

/// **The** raycast: from the camera, through the crosshair, out to the edge of
/// what this ability can reach.
///
/// `who` is the caster's index. Nothing between the camera and the character is
/// a candidate — the eye sits behind the shoulder, and a wall the camera
/// happens to be looking through is not a thing the player is aiming at.
///
/// # What it meets, and what it passes through
///
/// Terrain, structures, and the reach sphere. **Not bodies** — not other
/// fighters and not the creature.
///
/// This is the one place the model says something the player would not guess,
/// so it is worth being exact about what the ray is *for*. It answers **which
/// place in the world is under the crosshair**, and then the ability goes to
/// that place and hits whatever is standing there on the way. A body is not a
/// place; it is a thing occupying one. So the ray goes through it and stops on
/// the geometry behind it, and the ability crosses the ground the body is
/// standing on -- which is the same thing, only pointed at properly.
///
/// Bodies *were* on the list, and the creature is what proved they should not
/// be. It is large. Up close it fills the screen, so the crosshair lands on its
/// chest or its head, several metres up and only a metre or two away. The ray
/// stopped there, the ability was aimed at that point, and every shot went over
/// the animal at exactly the range where you cannot miss. Fighters have the
/// same shape of problem in miniature -- stand nose to nose and the camera,
/// which sits above the shoulder, puts the reticle on the top of their head.
///
/// Nothing is lost by it. What a shot runs into is [`first_along`]'s question,
/// asked along the path rather than along the camera's ray, and it has always
/// been the separate question: the camera is behind and above, so the two lines
/// were never the same line and a body the camera could not see was always
/// still a body the shot went through.
pub fn sight(who: usize, look: Input, reach: Fx, scene: &Scene) -> Sighted {
    let caster = &scene.players[who];
    let eye = crate::camera::eye(caster.pos, look, caster.aloft);
    let dir = look.look_dir();
    let cast = origin(caster.pos);

    // The near clip: the front of the character model, along the ray.
    let near = cast.sub(eye).dot(dir).sub(t::body_radius()).max(Fx::ZERO);
    // The far clip: the ability's own range, as a sphere about the caster
    // rather than a length along the ray, because the range belongs to the
    // ability and the ability starts at the fighter. Met from within or
    // without, and it is the *far* crossing that counts -- the near one is the
    // ray on its way past the caster.
    let sphere = reach_hit(eye, dir, cast, reach);
    let limit = sphere.unwrap_or(Fx::MAX);

    let mut best: Option<(Fx, Met)> = None;
    let mut keep = |hit: Option<Fx>, met: Met| {
        if let Some(d) = hit {
            if d.raw() >= near.raw()
                && d.raw() <= limit.raw()
                && best.is_none_or(|(b, _)| d.raw() < b.raw())
            {
                best = Some((d, met));
            }
        }
    };

    // Terrain. Ground is whatever faces upward, which is what decides whether
    // a skillshot flies level over the spot or straight at it.
    keep(floor_hit(eye, dir), Met::Ground);
    for solid in arena::SOLIDS.iter() {
        let hit = crate::math::ray_hits_box(eye, dir, solid.min, solid.max);
        keep(hit, facing(hit, eye, dir, solid.max.y));
    }
    for stone in scene.stones.iter().flatten() {
        let hit = stone_hit(eye, dir, stone);
        keep(hit, facing(hit, eye, dir, stone.top()));
    }
    // **Bodies are not on this list, and that is deliberate.** See the note on
    // this function: the ray is asking which *place* the player is pointing at,
    // and a creature is a thing standing in a place rather than the place
    // itself. Putting it on the ray made close-quarters aim at the creature
    // unusable, because up close it fills the screen and the crosshair sits on
    // its chest three metres up.

    match (best, sphere) {
        (Some((dist, met)), _) => Sighted {
            at: eye.add(dir.scale(dist)),
            dist,
            met,
        },
        (None, Some(edge)) => Sighted {
            at: eye.add(dir.scale(edge)),
            dist: edge,
            met: Met::Reach,
        },
        // The ray leaves the world without meeting anything, and without even
        // crossing the reach sphere -- which needs the eye to be outside it and
        // pointing away. Nothing to aim at, so the ability goes as far as it can
        // along the line anyway.
        (None, None) => Sighted {
            at: cast.add(dir.scale(reach)),
            dist: reach,
            met: Met::Reach,
        },
    }
}

/// Does a hit land on the upward face of a shape whose top is at `top`?
fn facing(hit: Option<Fx>, from: V3, dir: V3, top: Fx) -> Met {
    match hit {
        Some(d) if from.y.add(dir.y.mul(d)).sub(top).abs().raw() <= arena::SKIN.raw() => {
            Met::Ground
        }
        _ => Met::Solid,
    }
}

// ---------------------------------------------------------------------------
// The two kinds of skillshot
// ---------------------------------------------------------------------------

/// Where a **grounded** ability lands, and the line it takes to get there.
///
/// A structure, a fire pillar: things that come out of the floor, whatever the
/// caster was doing when they cast them. `from` is the character, so an ability
/// that races along the ground has its path already.
pub fn grounded_path(who: usize, look: Input, reach: Fx, scene: &Scene) -> Path {
    let caster = &scene.players[who];
    let seen = sight(who, look, reach, scene);
    let to = match seen.met {
        // Out at the edge of the ability's range: max range on the ground, in
        // the direction the mouse is facing. Settling the sphere's own point
        // instead would make an upward aim land short, which reads as the
        // ability refusing to go where it was pointed.
        Met::Reach => {
            let dir = look.look_dir();
            let flat = V3::new(dir.x, Fx::ZERO, dir.z).normalized();
            settle(origin(caster.pos).add(flat.scale(reach)), scene.stones)
        }
        // On the ground, exactly there -- `settle` is a no-op on a surface
        // something already stands on. On a wall, the floor beneath it,
        // because that is where the thing being placed can exist.
        Met::Ground | Met::Solid => settle(seen.at, scene.stones),
    };
    Path {
        from: caster.pos,
        to,
    }
}

/// The line a **skillshot** flies along: from the caster's ability origin to
/// the point the crosshair is on.
///
/// Its length is the ability's whole reach. There is no separate range: the
/// max-range sphere is part of the raycast, so a shot that meets nothing ends
/// on the sphere and a shot that meets something ends on that.
pub fn skillshot_path(who: usize, look: Input, reach: Fx, scene: &Scene) -> Path {
    let from = origin(scene.players[who].pos);
    let seen = sight(who, look, reach, scene);
    let to = match seen.met {
        // Aimed at the floor, which is never really the target: raised to the
        // middle of a fighter standing there, so it goes through whoever is on
        // that spot instead of burying itself in the dirt. See
        // [`standing_middle`].
        Met::Ground => standing_middle(seen.at),
        // A wall, the side of a stone, the edge of the range: the point
        // itself, because that is the thing the player is looking at.
        Met::Solid | Met::Reach => seen.at,
    };
    Path { from, to }
}

/// The line a **swing** comes out along: the body's own direction, tilted by
/// how far the camera is looking up or down.
///
/// No raycast, because a swing is not aimed *at* anything — it is a body
/// moving, and it stops where the weapon stops rather than where the crosshair
/// lands. What it takes from the crosshair is the **plane**. The yaw is the
/// facing, because a cut goes where your shoulders are; the pitch is the
/// camera's, because melee happens in the air and on slopes and a swing pinned
/// to the horizontal misses things that are plainly in front of you. It is most
/// of what the Champion is — a hammer comes down in the plane you are aiming
/// along, and its aerials are thrown at the floor or at the sky on purpose.
///
/// **There is a dead zone below the horizon, and it is the whole trick.** The
/// camera sits above the shoulder, so looking at somebody standing at your own
/// height means looking slightly *down* at them; a swing that followed that
/// exactly would tilt into the floor every time you fought anyone. So:
///
/// ```text
///   pitch above the horizon    the swing follows it exactly
///   the first N degrees below  the swing stays level -- the standard arc
///   further down than that     the swing follows what is left over
/// ```
///
/// At the dead zone's edge the tilt is still zero and it moves a degree per
/// degree from there, so there is no step at the boundary. `N` is
/// [`crate::tuning::swing_level_to`].
///
/// **`hand` moves where it starts, never where it points.** A punch thrown with
/// one arm leaves from that shoulder rather than from the middle of the chest,
/// which is the difference between a hitbox that comes out of the arm the player
/// can see swinging and one that comes out of the sternum. The direction is the
/// same either way: a body turns as one piece.
///
/// **Standing only.** The correction it makes is about two fighters sharing a
/// floor; off the floor the thing under the reticle really is below you, so
/// `grounded == false` follows the pitch exactly all the way down. It is also
/// the difference between the air game working and not: the look-down limit is
/// 85 degrees, and a 45-degree dead zone would cap a falling fighter's tilt at
/// 40 when the Champion's spike needs 45.
///
/// It is here rather than beside the move for the same reason everything else
/// in this file is: the look direction is one of the two ingredients of the
/// mistake this module exists to prevent, so the places that turn it into a
/// line are all in one file where they can be compared.
pub fn swing_path(pos: V3, facing: V3, look: Input, grounded: bool, reach: Fx, hand: Hand) -> Path {
    // From the hand: an overhead begins at the chest and a rising cut is aimed
    // from there, and a one-armed move begins a shoulder's width to one side of
    // both. Where along the body a given weapon actually hinges is
    // `moves::swing_hub`'s business; which side of it is [`Hand`].
    let from = hand_origin(pos, facing, hand);
    let tilt = swing_tilt(look, grounded);
    let flat = cos_turns(tilt);
    let dir = V3::new(facing.x.mul(flat), sin_turns(tilt), facing.z.mul(flat));
    Path {
        from,
        to: from.add(dir.scale(reach)),
    }
}

/// How far a swing tilts, given where the camera is looking.
///
/// Written as a sum rather than a branch so the two halves cannot disagree
/// about the boundary: above the horizon the first term is the pitch and the
/// second is zero, below it the first is zero and the second is whatever is
/// left after the dead zone is spent.
///
/// **The dead zone is a standing rule.** Its whole reason is that you fight
/// people at your own height by looking slightly down at them, and that is a
/// thing that happens with your feet on the floor. Off the ground you are
/// genuinely above what you are hitting, so the swing follows the camera all
/// the way down -- the same split `moves::swing_base` already makes for the
/// plane a weapon sweeps in.
fn swing_tilt(look: Input, grounded: bool) -> Fx {
    let pitch = look.pitch_turns();
    if !grounded {
        return pitch;
    }
    let dead = Fx::ratio(t::swing_level_to(), 360);
    pitch.max(Fx::ZERO).add(pitch.add(dead).min(Fx::ZERO))
}

/// Is the crosshair on the thing standing at `at`?
///
/// The same ray [`sight`] uses -- from the eye, through the crosshair -- asked
/// a yes-or-no question about one object instead of "what is the nearest thing
/// in the world". `slack` swells the column it is tested against, so this is
/// "near enough", not "exactly on".
///
/// It exists for one input: the Reaver's forward dodge becomes a dash to her
/// shadow when she is looking at it. That is a **decision about where an
/// ability goes**, so it belongs in this file with the rest of them -- the
/// alternative was an angle between the look direction and the line to the
/// shadow, worked out beside the dodge, which is the parallel-ray mistake in
/// its usual disguise: it agrees with the crosshair at long range and is out by
/// a body at short.
///
/// Nothing occludes it. A shadow is a shadow: you can point at one through a
/// wall, and whether she can actually *get* there is a separate question asked
/// of the world rather than of the camera -- see [`clear_between`].
pub fn pointing_at(who: usize, look: Input, at: V3, slack: Fx, scene: &Scene) -> bool {
    let eye = crate::camera::eye(scene.players[who].pos, look, scene.players[who].aloft);
    let column = t::body_radius().add(slack);
    let foot = V3::new(at.x, at.y.sub(slack), at.z);
    crate::math::ray_hits_cylinder(
        eye,
        look.look_dir(),
        foot,
        column,
        t::body_height().add(slack.add(slack)),
    )
    .is_some()
}

/// Is the crosshair on a **disc on the floor** -- an essence pool?
///
/// The same question as [`pointing_at`], asked about the Blood mage's object
/// rather than the Reaver's: the ray from the eye through the crosshair,
/// against a short cylinder standing on the disc. The disc's own radius is
/// the width, because a pool is drawn at exactly that and a player aims at
/// what they can see; `height` is the slack above it, since a reticle that
/// had to be on the floor itself would make a blink at anything past a few
/// metres a pixel-hunt. Bodies are not on the ray here either.
pub fn pointing_at_disc(
    who: usize,
    look: Input,
    at: V3,
    radius: Fx,
    height: Fx,
    scene: &Scene,
) -> bool {
    let eye = crate::camera::eye(scene.players[who].pos, look, scene.players[who].aloft);
    crate::math::ray_hits_cylinder(eye, look.look_dir(), at, radius, height).is_some()
}

/// Is there a straight line from one body to another that nothing solid
/// crosses?
///
/// The Reaver's dash to her shadow is the one thing that asks, and the rule it
/// is asking about is deliberately blunt: **the dash goes to wherever the
/// shadow is, along the straight line between them, and only a total
/// obstruction stops it.** Total means what it says -- not "a ledge is in the
/// way of her feet", which is every dash onto anything, but "there is no way
/// through at all".
///
/// `from` and `to` are the two sets of feet. Both bodies are upright columns
/// standing over a fixed spot, so **every line between them has the same
/// horizontal projection** and they differ only in how they rise. That makes
/// the four corner lines -- feet to feet, feet to head, head to feet, head to
/// head -- the extremes of the whole family, and a solid that crosses all four
/// crosses everything in between. So: clear if any one of the four is clear.
///
/// It is what lets her dash *up*. Standing at the foot of a platform with the
/// shadow on the deck, the line from her feet is through the wall of it and the
/// line from her head is over the lip -- she can see a way up, so she takes it.
/// Standing on the floor with the shadow on the far side of that platform, all
/// four lines go into it, and the dodge stays an ordinary dodge.
///
/// The four lines are measured from just above the soles and just below the
/// crown, by the collision skin. A body standing on a surface is standing
/// *exactly* on it, so a line taken from the soles themselves grazes the thing
/// it is standing on and reads as a wall. Trimming the body rather than the
/// world is what keeps two stacked solids one obstruction instead of two with a
/// hairline gap between them.
pub fn clear_between(from: V3, to: V3, scene: &Scene) -> bool {
    let sole = arena::SKIN;
    let crown = t::body_height().sub(arena::SKIN);
    [(sole, sole), (sole, crown), (crown, sole), (crown, crown)]
        .into_iter()
        .any(|(lift_a, lift_b)| {
            nothing_between(
                V3::new(from.x, from.y.add(lift_a), from.z),
                V3::new(to.x, to.y.add(lift_b), to.z),
                scene,
            )
        })
}

/// Where a **blink** ends: `reach` along the flat of `dir` from `from`, or
/// short of that, against the first thing solid in the way.
///
/// The Dual mage's dodge at the first tier is the ordinary dodge with the
/// travelling taken out: she is put where it would have ended, on its first
/// frame. Where that is has to be decided here for the same reason the dash
/// to the shadow is -- it is a body being sent somewhere, and the answer to
/// "is anything in the way" is ray-against-shape work that belongs in one
/// place. Bodies are not on the line, as ever: a blink passes through a
/// fighter to the spot behind them, and it is the arena and the stones that
/// stop it.
///
/// **Stricter than [`clear_between`], on purpose.** The dash goes wherever the
/// shadow is, up included, so a way through for any part of the body is a way
/// through; that is why one clear line out of four is enough there. A blink
/// goes along the floor, to a spot at her own height, and a low platform her
/// head would clear is still a wall to her feet -- so she stops where the
/// **first** of the four corner lines meets something, less her own radius, and
/// arrives against the obstacle rather than inside it or on top of it.
pub fn blink_to(from: V3, dir: V3, reach: Fx, scene: &Scene) -> V3 {
    let flat = V3::new(dir.x, Fx::ZERO, dir.z).normalized();
    if reach.raw() <= 0 {
        return from;
    }
    let to = from.add(flat.scale(reach));
    let sole = arena::SKIN;
    let crown = t::body_height().sub(arena::SKIN);
    // The share of the way each line gets before it is stopped, as a fraction
    // of its own length -- which is the same fraction of the flat distance,
    // because all four share one horizontal projection.
    let mut share = Fx::ONE;
    for (lift_a, lift_b) in [(sole, sole), (sole, crown), (crown, sole), (crown, crown)] {
        let a = V3::new(from.x, from.y.add(lift_a), from.z);
        let b = V3::new(to.x, to.y.add(lift_b), to.z);
        if let Some(along) = first_solid_between(a, b, scene) {
            share = share.min(along);
        }
    }
    if share.raw() >= Fx::ONE.raw() {
        return to;
    }
    let short = reach
        .mul(share)
        .sub(t::body_radius().add(arena::SKIN))
        .max(Fx::ZERO);
    from.add(flat.scale(short))
}

/// Where along one line, as a share of its length, the first solid is -- or
/// `None` if nothing solid crosses it before the far end.
fn first_solid_between(a: V3, b: V3, scene: &Scene) -> Option<Fx> {
    let span = b.sub(a);
    let reach = span.len();
    if reach.raw() <= 0 {
        return None;
    }
    let dir = span.normalized();
    let mut nearest: Option<Fx> = None;
    let mut consider = |hit: Option<Fx>| {
        if let Some(d) = hit {
            if d.raw() < reach.raw() && nearest.is_none_or(|n| d.raw() < n.raw()) {
                nearest = Some(d);
            }
        }
    };
    for solid in arena::SOLIDS.iter() {
        consider(crate::math::ray_hits_box(a, dir, solid.min, solid.max));
    }
    for stone in scene.stones.iter().flatten() {
        consider(crate::math::ray_hits_cylinder(
            a,
            dir,
            stone.at,
            t::structure_radius(),
            stone.standing_height(),
        ));
    }
    nearest.map(|d| d.div(reach))
}

/// One line of [`clear_between`], against the terrain and the structures on it.
///
/// The ground plane is not consulted: every surface a body can stand on is at
/// or above it, so the floor is never *between* two of them.
fn nothing_between(a: V3, b: V3, scene: &Scene) -> bool {
    let span = b.sub(a);
    let reach = span.len();
    if reach.raw() <= 0 {
        return true;
    }
    let dir = span.normalized();
    // Short of the far end, so a line that arrives exactly on the surface the
    // other body is standing on has not been stopped by it.
    let stopped = |hit: Option<Fx>| hit.is_some_and(|d| d.raw() < reach.raw());
    for solid in arena::SOLIDS.iter() {
        if stopped(crate::math::ray_hits_box(a, dir, solid.min, solid.max)) {
            return false;
        }
    }
    for stone in scene.stones.iter().flatten() {
        if stopped(crate::math::ray_hits_cylinder(
            a,
            dir,
            stone.at,
            t::structure_radius(),
            stone.standing_height(),
        )) {
            return false;
        }
    }
    true
}

/// Where the class mechanic is standing.
///
/// One move works this way -- the Reaver's Guillotine lotus, whose blades erupt
/// at the shadow. The player *did* aim it, with the crosshair, when they placed
/// the shadow; throwing the move only cashes that in. Re-aiming it here would
/// quietly delete the reason shadow placement is a decision.
///
/// Falls back to the caster's own feet when the mechanic is nowhere, which the
/// move table's `needs_mechanic` should already have prevented.
pub fn mechanic_path(from: V3, mechanic: &Mechanic) -> Path {
    Path {
        from,
        to: mechanic.placed().unwrap_or(from),
    }
}

/// Where a move plants something **in front of the body**, on the floor.
///
/// Not a fifth line of effect, and it is worth being exact about why. The four
/// above answer *where does this ability go*, and a move declares which of them
/// it uses. This answers a narrower question that one move asks in addition to
/// its own line of effect: the Elementalist's Landfall is a [`Kind::Swing`] --
/// a body arriving, its volume on the floor at her own feet -- and the slab of
/// rock it drives up is a second thing, put down a fixed distance in front of
/// her rather than anywhere the crosshair chose. She is landing, not aiming.
///
/// It is here rather than beside the move for the reason [`pointing_at`] and
/// [`clear_between`] are: it decides where something in the world ends up, and
/// the alternative is a facing, a distance and a floor query written next to
/// the ability, which is exactly the shape of the mistake this file exists to
/// prevent. The facing is used **flat**: a plunge that put its stone nearer
/// because she happened to be looking down would be aiming after all.
///
/// [`settle`] does the last step, so the stone comes up on top of whatever is
/// under that spot -- another stone included -- rather than inside it.
pub fn planted_ahead(pos: V3, facing: V3, ahead: Fx, stones: &Field) -> V3 {
    let flat = V3::new(facing.x, Fx::ZERO, facing.z).normalized();
    settle(pos.add(flat.scale(ahead)), stones)
}

/// Which way the Reaver's shadow, **out on the field**, throws its copy of
/// her swing: flat, at the nearest body inside `reach` -- or `None`, and it
/// keeps her yaw.
///
/// Not a fifth line of effect either. The copy is still a [`Kind::Swing`] and
/// keeps everything about hers but the yaw: the pitch she committed to, the
/// shape, the frames. What this answers is the one thing a copy thrown from
/// somewhere else cannot take from her -- which way is *forward* over there.
/// With her yaw, a copy six metres away was a quarter-damage arc pointed
/// wherever her shoulders happened to be, and landed only on somebody standing
/// at exactly her offset from it. See `docs/design/shadow-reaver-v2.md`.
///
/// "In reach" is the copied move's own reach from the shadow's feet to the
/// nearest edge of a body, plus `tuning::shadow_aim_slack`. A fighter counts
/// by the edge of their column; the creature by the nearest point of whichever
/// part of it is closest to the height the swing leaves at, so a shadow standing at a Ridgeback's flank cuts the
/// flank rather than turning to face the middle of the animal. The owner is
/// never a candidate, nor anybody already down, and `fighters` is false in a
/// hunt -- the copy cannot hurt a partner, so it does not turn to one.
///
/// Here rather than beside the shadow because it is a decision about where
/// something goes, and the alternative -- a yaw to the nearest body worked out
/// in `shadow.rs` -- is the thing this file exists to stop.
pub fn shadow_faces(
    from: V3,
    owner: usize,
    reach: Fx,
    fighters: bool,
    scene: &Scene,
) -> Option<V3> {
    if !t::shadow_aims() {
        return None;
    }
    let limit = reach.add(t::shadow_aim_slack());
    let mut best: Option<(V3, Fx)> = None;
    let mut consider = |at: V3, gap: Fx| {
        if gap.raw() <= limit.raw() && best.is_none_or(|(_, seen)| gap.raw() < seen.raw()) {
            best = Some((at, gap));
        }
    };
    for (i, body) in scene.players.iter().enumerate() {
        if !fighters || i == owner || body.health <= 0 {
            continue;
        }
        let gap = body.pos.sub(from).flat_len().sub(t::body_radius());
        consider(body.pos, gap);
    }
    // From the height the swing leaves at, and in three dimensions: a part
    // overhead is not in reach however close its footprint is.
    if let Some(beast) = scene.quarry.filter(|b| b.alive()) {
        let hub = origin(from);
        let at = beast.nearest_to(hub);
        consider(at, crate::math::big_len(at.sub(hub)));
    }
    let (at, _) = best?;
    let flat = V3::new(at.x.sub(from.x), Fx::ZERO, at.z.sub(from.z));
    if flat.flat_len().raw() <= 0 {
        return None;
    }
    Some(flat.normalized())
}

/// Her swing's line, **thrown from the shadow** at `at` and turned onto
/// `facing`.
///
/// The copy keeps her line: where along her body it leaves, how far it goes,
/// the pitch it goes at. Only two things change, which place it starts from
/// and which way is forward. Turned rather than recomputed, because the pitch
/// on her path is what she committed to when she threw the move, and a copy
/// that re-read the camera would be a second swing rather than a copy of hers.
///
/// With `facing` equal to her own this is a plain translation, which is what an
/// attending shadow gets -- it is at her heel copying her, not fighting on its
/// own.
pub fn copied_swing(path: Path, her_pos: V3, her_facing: V3, at: V3, facing: V3) -> Path {
    let turn = |p: V3| {
        let v = p.sub(her_pos);
        // Her frame: along her facing, and across it. Then the same two
        // amounts along the new facing and across that. No angles, so no
        // trigonometry and nothing to round differently on two machines.
        let across_hers = V3::new(her_facing.z.neg(), Fx::ZERO, her_facing.x);
        let across_new = V3::new(facing.z.neg(), Fx::ZERO, facing.x);
        let along = V3::new(v.x, Fx::ZERO, v.z).dot(her_facing);
        let side = V3::new(v.x, Fx::ZERO, v.z).dot(across_hers);
        let flat = facing.scale(along).add(across_new.scale(side));
        at.add(V3::new(flat.x, v.y, flat.z))
    };
    Path {
        from: turn(path.from),
        to: turn(path.to),
    }
}

// ---------------------------------------------------------------------------
// What a path runs into
// ---------------------------------------------------------------------------

/// Which kinds of thing a travelling ability can hit.
///
/// Stated per ability rather than assumed, because they genuinely differ: the
/// Elementalist's beam lights a fire pillar it passes through, and the bolt
/// that comes out of the pillar does not.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Targets {
    pub fighters: bool,
    pub stones: bool,
    pub fire: bool,
    pub quarry: bool,
    /// The arena itself -- walls, the sides and tops of platforms.
    ///
    /// **Off for every ability that existed before 2026-09-17**, and that is
    /// not an oversight being preserved: the crosshair's ray already stops on
    /// terrain ([`sight`]), so a shot is aimed at a point the geometry allows
    /// and asking a second time along its own path would only ever agree. What
    /// wants this is an ability asking *whether there is something to pull on*,
    /// which is a question about the anchor rather than about the flight -- the
    /// Blood mage's Grasp, and her blink looking for the wall it must not go
    /// through.
    pub terrain: bool,
}

impl Targets {
    pub const fn none() -> Targets {
        Targets {
            fighters: false,
            stones: false,
            fire: false,
            quarry: false,
            terrain: false,
        }
    }

    pub const fn terrain(mut self) -> Targets {
        self.terrain = true;
        self
    }

    pub const fn fighters(mut self, yes: bool) -> Targets {
        self.fighters = yes;
        self
    }

    pub const fn stones(mut self) -> Targets {
        self.stones = true;
        self
    }

    pub const fn fire(mut self) -> Targets {
        self.fire = true;
        self
    }

    pub const fn quarry(mut self, yes: bool) -> Targets {
        self.quarry = yes;
        self
    }
}

/// What a travelling ability meets, and how far along its path it sits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Contact {
    Fighter {
        index: usize,
        dist: Fx,
    },
    Stone {
        index: usize,
        dist: Fx,
    },
    Fire {
        dist: Fx,
    },
    Quarry {
        part: usize,
        dist: Fx,
    },
    /// A wall, or the side or top of a platform. Only ever reported when
    /// [`Targets::terrain`] asked for it.
    Terrain {
        dist: Fx,
    },
}

impl Contact {
    pub fn dist(self) -> Fx {
        match self {
            Contact::Fighter { dist, .. }
            | Contact::Stone { dist, .. }
            | Contact::Fire { dist }
            | Contact::Quarry { dist, .. }
            | Contact::Terrain { dist } => dist,
        }
    }

    /// Is this something a thrown rope could hold on to?
    ///
    /// Terrain, a structure and the creature are anchors; a fighter and a fire
    /// are not -- one of them moves and the other is not there. It is a
    /// question about the world rather than about the Blood mage, so it is
    /// answered here beside the enum rather than beside her ability.
    pub fn is_an_anchor(self) -> bool {
        matches!(
            self,
            Contact::Terrain { .. } | Contact::Stone { .. } | Contact::Quarry { .. }
        )
    }
}

/// The first thing a straight path runs into, or `None` if it reaches its end
/// clear.
///
/// Separate from [`sight`] and deliberately so: the camera's ray says *where
/// the player is pointing*, and this says *what is actually in the way of the
/// thing they threw*. The two lines are not the same line, and a body the
/// camera could not see is still a body the shot passes through.
///
/// `girth` is the travelling thing's own radius, added to whatever it is tested
/// against, so "do these two volumes touch" is one ray against one shape.
pub fn first_along(
    path: Path,
    girth: Fx,
    owner: u8,
    scene: &Scene,
    targets: Targets,
) -> Option<Contact> {
    let (from, dir, limit) = (path.from, path.dir(), path.length());
    let mut best: Option<Contact> = None;
    let mut keep = |found: Contact| {
        if found.dist().raw() <= limit.raw()
            && best.is_none_or(|b| found.dist().raw() < b.dist().raw())
        {
            best = Some(found);
        }
    };

    if targets.fighters {
        for (index, p) in scene.players.iter().enumerate() {
            if index as u8 == owner || p.health <= 0 || p.action.invulnerable() {
                continue;
            }
            if let Some(dist) = body_hit(from, dir, p, girth) {
                keep(Contact::Fighter { index, dist });
            }
        }
    }
    if targets.stones {
        for (index, slot) in scene.stones.iter().enumerate() {
            let Some(stone) = slot else { continue };
            if let Some(dist) = crate::math::ray_hits_cylinder(
                from,
                dir,
                stone.at,
                t::structure_radius().add(girth),
                stone.standing_height(),
            ) {
                keep(Contact::Stone { index, dist });
            }
        }
    }
    if targets.fire {
        for slot in scene.effects.iter() {
            let Some(e) = slot else { continue };
            if e.kind != EffectKind::FirePillar {
                continue;
            }
            // Both of a pillar's volumes: the wide base you walk into and the
            // column above it that stops you jumping over.
            let (base, column) = e.pillar_volumes();
            for slab in [base, column] {
                let foot = V3::new(e.pos.x, e.pos.y.add(slab.bottom), e.pos.z);
                if let Some(dist) = crate::math::ray_hits_cylinder(
                    from,
                    dir,
                    foot,
                    slab.radius.add(girth),
                    slab.top.sub(slab.bottom),
                ) {
                    keep(Contact::Fire { dist });
                }
            }
        }
    }
    if targets.quarry {
        if let Some((part, dist)) = scene
            .quarry
            .and_then(|b| b.part_struck_along(from, dir, limit, girth))
        {
            keep(Contact::Quarry { part, dist });
        }
    }
    if targets.terrain {
        // The blockout's own boxes, grown by the travelling thing's girth in
        // all three axes -- the same trick the stones use, so "do these two
        // volumes touch" stays one ray against one shape.
        let fat = V3::new(girth, girth, girth);
        for solid in arena::SOLIDS.iter() {
            if let Some(dist) =
                crate::math::ray_hits_box(from, dir, solid.min.sub(fat), solid.max.add(fat))
            {
                keep(Contact::Terrain { dist });
            }
        }
    }
    best
}

/// Drop a point onto whatever it would stand on.
pub fn settle(at: V3, stones: &Field) -> V3 {
    let mut floor = arena::ground_under(at);
    for stone in stones.iter().flatten() {
        let apart = V3::new(at.x.sub(stone.at.x), Fx::ZERO, at.z.sub(stone.at.z)).flat_len();
        if apart.raw() < t::structure_radius().raw() && stone.top().raw() > floor.raw() {
            floor = stone.top();
        }
    }
    V3::new(at.x, floor, at.z)
}

// ---------------------------------------------------------------------------
// The shapes
// ---------------------------------------------------------------------------

/// How far along the ray the ability's own range runs out.
fn reach_hit(from: V3, dir: V3, centre: V3, radius: Fx) -> Option<Fx> {
    let m = from.sub(centre);
    let b = m.dot(dir);
    let under = b.mul(b).sub(m.len_sq().sub(radius.mul(radius)));
    if under.raw() < 0 {
        return None;
    }
    let far = under.sqrt().sub(b);
    (far.raw() > 0).then_some(far)
}

/// The arena floor. A plane rather than a box, because that is what the
/// simulation collides against -- `arena::resolve` treats `y <= 0` as the
/// ground and never consults `SOLIDS` for it.
fn floor_hit(from: V3, dir: V3) -> Option<Fx> {
    if dir.y.raw() >= 0 || from.y.raw() < 0 {
        return None;
    }
    Some(from.y.div(dir.y.neg()))
}

/// A stone: an upright cylinder standing on its base, with both end caps.
///
/// The top cap is not a detail. Aiming at the top of a stone is how you put the
/// next thing on top of it, and a cylinder without caps is a tube the ray goes
/// straight down.
fn stone_hit(from: V3, dir: V3, stone: &Structure) -> Option<Fx> {
    crate::math::ray_hits_cylinder(
        from,
        dir,
        stone.at,
        t::structure_radius(),
        // Still buried, so there is nothing there to hit.
        stone.standing_height(),
    )
}

/// A fighter: the upright cylinder their hurtbox already is, swollen by
/// `girth`. Crouching lowers it, which is what lets a crouch duck a shot aimed
/// over the head.
fn body_hit(from: V3, dir: V3, victim: &Player, girth: Fx) -> Option<Fx> {
    crate::math::ray_hits_cylinder(
        from,
        dir,
        victim.pos,
        t::body_radius().add(girth),
        victim.hurt_height(),
    )
}
