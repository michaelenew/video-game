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
//! # The two kinds, and the one thing that is not one
//!
//! ```text
//!   Grounded    the thing lands on the floor       structures, fire pillar
//!   Skillshot   the thing flies through the air    the Elementalist's auto,
//!                                                   the Blood mage's blade
//!   Swing       not aimed at all                   every melee attack
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
//! the move's own reach. It is in this list so that "which of the three is
//! this move" is a question with an answer for every move rather than a thing
//! each caller decides for itself — see [`crate::moves::Move::aim`].
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

/// Which of the three ways a move is pointed.
///
/// Every move answers this, and [`crate::moves::Move::aim`] is where it is
/// answered. Two of the three are skillshots and go through the raycast above;
/// the third is a body moving and does not.
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
    let eye = crate::camera::eye(caster.pos, look);
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
        // Aimed at the floor. Raised straight up to the height the shot leaves
        // at, so it flies level over the spot the crosshair is on instead of
        // burying itself in the ground a metre in front of her.
        Met::Ground => V3::new(seen.at.x, from.y, seen.at.z),
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
pub fn swing_path(pos: V3, facing: V3, look: Input, grounded: bool, reach: Fx) -> Path {
    // From the hand: an overhead begins at the chest and a rising cut is aimed
    // from there. Where along the body a given weapon actually hinges is
    // `moves::swing_hub`'s business.
    let from = origin(pos);
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
}

impl Targets {
    pub const fn none() -> Targets {
        Targets {
            fighters: false,
            stones: false,
            fire: false,
            quarry: false,
        }
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
    Fighter { index: usize, dist: Fx },
    Stone { index: usize, dist: Fx },
    Fire { dist: Fx },
    Quarry { part: usize, dist: Fx },
}

impl Contact {
    pub fn dist(self) -> Fx {
        match self {
            Contact::Fighter { dist, .. }
            | Contact::Stone { dist, .. }
            | Contact::Fire { dist }
            | Contact::Quarry { dist, .. } => dist,
        }
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
