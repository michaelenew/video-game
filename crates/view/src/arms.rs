//! The Champion's three weapons, as things you can see.
//!
//! The class is three weapons on three buttons, and for most of its life the
//! only one of them on the screen was the hit capsule drawn as a glowing rod:
//! the sword, the hammer and the spear were the same stick at three lengths,
//! which is how a player comes to say he cannot feel the difference between
//! them. A weapon's silhouette is the cheapest tell there is -- it is read from
//! across the arena before a single frame of the swing has been -- so each one
//! is drawn as the thing it is, and they are drawn to be **unlike each other at
//! a glance**:
//!
//! - the **sword** is a longsword: a pommel, a two-hand grip, a crossguard and
//!   a straight blade tapering to a point. Middling in every dimension.
//! - the **hammer** is short and top-heavy: a stout haft with a big square iron
//!   head set across its end, flared at both faces. The only weapon with mass
//!   at the end of it.
//! - the **spear** is nearly twice either of them, thin the whole way, with a
//!   leaf-shaped head and a red tassel under it.
//!
//! **All three are always on her.** The one in her hands is the running move's
//! weapon, or the last one she threw when nothing is running -- the same
//! `Form` the HUD names. The other two are stowed: the spear slung across her
//! back, the hammer hung at her right hip, the sword in a scabbard at her
//! left. So a glance at her says which weapon is out, and a glance at the rest
//! of her says which two are not.
//!
//! **The drawn weapon is where the hit test is.** On an active frame the tip is
//! the far end of `state::hitbox`, exactly, and the weapon runs back from it
//! through her leading hand -- the rule the Blood mage's scythe keeps, for the
//! reason `CLAUDE.md` gives the debug overlay. The weapon's own size is fixed,
//! and what gives when the volume reaches further than the weapon at rest is
//! the part a hand could slide along: a spear's shaft or a hammer's haft runs
//! through the leading hand rather than being held at one point, and the
//! sword's blade -- which a hand cannot slide along -- is drawn to the length
//! of the cut. Through the wind-up and the recovery the weapon simply lies
//! along the line of her two hands, head beyond the right one, which is the
//! convention `anim::clips::champion::weapon` authors every grip in.
//!
//! [`trail`] is the arc of the cut, re-asked of the hit test for each active
//! frame so far: a band across the outer part of the capsule, per frame, fading
//! through the first frames of the recovery. It is what replaced the rod.
//!
//! Everything here is a pure function of simulation state plus the drawn hand
//! and torso positions, allocation-free, so `tests/arms.rs` can hold it to the
//! hit test without a renderer.

use crate::math::{self, Quat, V3};
use sim::class::{Form, Mechanic};
use sim::moves::champion::{self, HAMMER, SPEAR, SWORD};
use sim::state::{self, Action, Player};

/// Pieces per weapon. Every weapon has this many entities in the renderer;
/// a piece a weapon does not use is simply never shown.
pub const PIECES: usize = 7;

/// The weapons, in button order: [`SWORD`], [`HAMMER`], [`SPEAR`].
pub const WEAPONS: usize = 3;

/// How many active frames of a swing the trail remembers.
pub const TRAIL: usize = 10;

/// How many recovery frames the trail lingers for, fading.
pub const TRAIL_GHOST: u16 = 7;

/// What a piece is shaped like. The renderer holds one unit mesh of each.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    /// A unit cube.
    Block,
    /// A unit cylinder standing on `along`.
    Rod,
    /// A unit cone, apex toward `along`.
    Point,
    /// A unit sphere.
    Knob,
}

/// What a piece is made of. The renderer holds one material of each.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stuff {
    /// Bright, polished: the sword's blade and the spear's head.
    Steel,
    /// Dark, heavy, forged: the hammer's head.
    Iron,
    /// The fittings: crossguard, pommel.
    Brass,
    /// Hafts and shafts.
    Wood,
    /// Grips and the scabbard.
    Leather,
    /// The spear's tassel.
    Cloth,
}

/// What each piece of each weapon is, fixed so the renderer can make its
/// meshes once. Indexed `[weapon][piece]`.
pub const MAKE: [[(Shape, Stuff); PIECES]; WEAPONS] = [
    // Sword: pommel, grip, crossguard, blade, point, scabbard, scabbard's chape.
    [
        (Shape::Knob, Stuff::Brass),
        (Shape::Rod, Stuff::Leather),
        (Shape::Block, Stuff::Brass),
        (Shape::Block, Stuff::Steel),
        (Shape::Point, Stuff::Steel),
        (Shape::Block, Stuff::Leather),
        (Shape::Block, Stuff::Brass),
    ],
    // Hammer: haft, grip, butt cap, langets, head, the two faces.
    [
        (Shape::Rod, Stuff::Wood),
        (Shape::Rod, Stuff::Leather),
        (Shape::Rod, Stuff::Iron),
        (Shape::Block, Stuff::Iron),
        (Shape::Block, Stuff::Iron),
        (Shape::Block, Stuff::Steel),
        (Shape::Block, Stuff::Steel),
    ],
    // Spear: shaft, grip, butt spike, socket, the head's two halves, tassel.
    [
        (Shape::Rod, Stuff::Wood),
        (Shape::Rod, Stuff::Leather),
        (Shape::Point, Stuff::Steel),
        (Shape::Rod, Stuff::Steel),
        (Shape::Point, Stuff::Steel),
        (Shape::Point, Stuff::Steel),
        (Shape::Block, Stuff::Cloth),
    ],
];

/// One piece of one weapon, placed in the arena.
///
/// `along` is the piece's own length axis (the mesh's `+Y`), `across` the
/// direction of its breadth (the mesh's `+Z`, square to `along`), and `size`
/// is `[thickness, length, breadth]` -- the mesh's `x`, `y`, `z` scale.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Piece {
    pub shown: bool,
    pub centre: V3,
    pub along: V3,
    pub across: V3,
    pub size: V3,
}

const HIDDEN: Piece = Piece {
    shown: false,
    centre: math::ZERO,
    along: [0.0, 1.0, 0.0],
    across: [0.0, 0.0, 1.0],
    size: [0.0; 3],
};

/// One weapon, as a line and a plane: butt to tip, and the direction its
/// breadth lies in -- the sword's edges, the hammer head's long axis, the
/// spear's leaf.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Weapon {
    pub butt: V3,
    pub tip: V3,
    /// Unit, square to the axis.
    pub flat: V3,
    /// In her hands, rather than on her back or at her hip.
    pub held: bool,
}

impl Weapon {
    pub fn axis(&self) -> V3 {
        math::normalize_or(math::sub(self.tip, self.butt), [0.0, 1.0, 0.0])
    }

    /// Butt to tip.
    pub fn length(&self) -> f32 {
        math::length(math::sub(self.tip, self.butt))
    }
}

/// Where the Champion's upper body is drawn this frame, in the arena: the
/// chest and the hips, each a point and a rotation from the body's own frame
/// (`+X` her left, `+Y` up, `+Z` forward) into the arena's. The stowed
/// weapons ride these, so they lean and turn with her.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Torso {
    pub chest: V3,
    pub chest_turn: Quat,
    pub hips: V3,
    pub hips_turn: Quat,
}

/// Everything the Champion carries: the three weapons, indexed by
/// [`SWORD`], [`HAMMER`] and [`SPEAR`], and the scabbard at her hip.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Arms {
    pub weapons: [Weapon; WEAPONS],
    /// The scabbard's mouth, its axis toward the chape, and the plane of it.
    pub scabbard: Weapon,
    /// Which one is in her hands.
    pub held: u8,
}

// -- The three weapons' dimensions ----------------------------------------
//
// Metres. The body is about 1.8 m tall, which is what these are read against.

/// Pommel to the right hand, on the sword: the two hands both fit on the grip
/// behind it, the right one against the crossguard.
const SWORD_HOLD: f32 = 0.31;
/// The right hand to the tip, with nothing out: a longsword's blade.
const SWORD_REACH: f32 = 1.2;
/// The shortest and longest the blade is drawn, hand to tip, while a cut is
/// out. The cut decides; these only stop a degenerate frame drawing a dagger
/// or a pike.
const SWORD_FLEX: (f32, f32) = (0.9, 2.1);
const SWORD_POINT: f32 = 0.2;
const SWORD_BREADTH: f32 = 0.085;
const SWORD_THICK: f32 = 0.024;
const SWORD_GUARD: f32 = 0.38;

/// Butt to the far face of the head.
const HAMMER_LENGTH: f32 = 1.35;
/// How far past its own length the haft may be drawn when a swing reaches
/// further than the hands can slide.
const HAMMER_STRETCH: f32 = 1.75;
/// The head: across the haft, along it, and its depth.
const HAMMER_HEAD: V3 = [0.28, 0.28, 0.52];
const HAMMER_HAFT: f32 = 0.075;

const SPEAR_LENGTH: f32 = 2.9;
const SPEAR_STRETCH: f32 = 1.3;
/// Butt to the right hand, with nothing out: the spear is held back of its
/// middle, most of it ahead.
const SPEAR_HOLD: f32 = 0.95;
const SPEAR_SHAFT: f32 = 0.055;
const SPEAR_HEAD: f32 = 0.44;
const SPEAR_BREADTH: f32 = 0.16;

/// How far behind the trailing hand the butt of a sliding weapon is kept.
const PAST_THE_HAND: f32 = 0.12;

/// Which weapon is in her hands: the running move's, or the last one thrown.
pub fn in_hand(p: &Player) -> Option<u8> {
    if p.class != sim::Class::Champion {
        return None;
    }
    if let Some(kind) = p.action.attack_kind() {
        return Some(champion::weapon(kind));
    }
    Some(match p.mechanic {
        Mechanic::Forms { form, .. } => match form {
            Form::Sword => SWORD,
            Form::Hammer => HAMMER,
            Form::Spear => SPEAR,
        },
        _ => SWORD,
    })
}

/// Where the Champion's three weapons are, if this fighter is a Champion.
///
/// `left` and `right` are where the body's drawn hands are, and `torso` where
/// its chest and hips are: the renderer knows those and the simulation does
/// not. They decide where each weapon is held or hung, never how far a swing
/// reaches.
pub fn champion_arms(p: &Player, left: V3, right: V3, torso: &Torso) -> Option<Arms> {
    let held = in_hand(p)?;
    let facing = math::normalize_or(
        [crate::fx(p.facing.x), 0.0, crate::fx(p.facing.z)],
        [1.0, 0.0, 0.0],
    );
    let mut weapons = [
        stowed_sword(torso),
        stowed_hammer(torso),
        stowed_spear(torso),
    ];
    weapons[held as usize] = in_the_hands(p, held, left, right, facing);
    Some(Arms {
        weapons,
        scabbard: scabbard(torso),
        held,
    })
}

/// The weapon in her hands.
fn in_the_hands(p: &Player, which: u8, left: V3, right: V3, facing: V3) -> Weapon {
    if p.action.attack_kind().is_none() {
        return carried(p, which, right, facing);
    }
    let apart = math::length(math::sub(right, left));
    let hands = math::normalize_or(math::sub(right, left), facing);
    // While the volume is out, the tip is its far end and the weapon points at
    // it from the leading hand. A volume that is a disc has no far end to
    // point at, and the weapon stays in the hands.
    let target = state::hitbox(p)
        .map(|hb| (fx3(hb.from), fx3(hb.to)))
        .filter(|(from, to)| math::length(math::sub(*to, *from)) > 0.05);
    let (butt, tip) = match which {
        SWORD => {
            let (axis, reach) = match target {
                Some((_, to)) => {
                    let toward = math::sub(to, right);
                    let d = math::length(toward).clamp(SWORD_FLEX.0, SWORD_FLEX.1);
                    (math::normalize_or(toward, hands), d)
                }
                None => (hands, SWORD_REACH),
            };
            (
                math::sub(right, math::scale(axis, SWORD_HOLD)),
                math::add(right, math::scale(axis, reach)),
            )
        }
        _ => {
            let (length, stretch, hold) = if which == HAMMER {
                // Rested, the right hand is the length of the two hands and a
                // bit from the butt: both hands at the end of the haft.
                (HAMMER_LENGTH, HAMMER_STRETCH, apart + PAST_THE_HAND)
            } else {
                (SPEAR_LENGTH, SPEAR_STRETCH, SPEAR_HOLD.max(apart + 0.15))
            };
            match target {
                Some((_, to)) => slide(right, to, hands, length, stretch),
                None => {
                    let butt = math::sub(right, math::scale(hands, hold));
                    (butt, math::add(butt, math::scale(hands, length)))
                }
            }
        }
    };
    let axis = math::normalize_or(math::sub(tip, butt), facing);
    let flat = held_flat(p, which, axis, facing);
    Weapon {
        butt,
        tip,
        flat,
        held: true,
    }
}

/// The weapon in her hand between moves.
///
/// The Champion has no idle of her own -- she stands and walks on the clips
/// every class shares, whose hands are not on anything -- so between moves
/// the weapon is carried the way that weapon is carried, off her right hand
/// alone, and each carry says what the weapon is from across the arena: the
/// sword up and forward in a low guard, the hammer's head resting on the
/// floor at her side with the haft leaning up into her hand, and the spear
/// stood upright, butt on the floor, head well over hers.
fn carried(p: &Player, which: u8, right: V3, facing: V3) -> Weapon {
    let up = [0.0, 1.0, 0.0];
    let side = math::cross(facing, up);
    let floor = crate::fx(p.pos.y);
    let lean = |f: f32, u: f32, s: f32| {
        math::normalize_or(
            math::add(
                math::add(math::scale(facing, f), math::scale(up, u)),
                math::scale(side, s),
            ),
            facing,
        )
    };
    let (butt, tip) = match which {
        SWORD => {
            let axis = lean(0.8, 0.45, -0.12);
            (
                math::sub(right, math::scale(axis, SWORD_HOLD)),
                math::add(right, math::scale(axis, SWORD_REACH)),
            )
        }
        HAMMER => {
            // The head's centre on the floor, ahead and out to her right,
            // and the haft from it up into the hand, which holds it a hand's
            // width from the butt.
            let head = HAMMER_HEAD[1] * 0.5;
            let reach = HAMMER_LENGTH - PAST_THE_HAND - head;
            let drop = (right[1] - floor - HAMMER_HEAD[0] * 0.5).clamp(0.0, reach);
            let out = (reach * reach - drop * drop).max(0.0).sqrt();
            let flat = math::normalize_or(
                math::add(math::scale(facing, 0.75), math::scale(side, 0.45)),
                facing,
            );
            let axis = math::normalize_or(
                math::sub(math::scale(flat, out), math::scale(up, drop)),
                facing,
            );
            let butt = math::sub(right, math::scale(axis, PAST_THE_HAND));
            (butt, math::add(butt, math::scale(axis, HAMMER_LENGTH)))
        }
        _ => {
            let axis = lean(0.08, 1.0, 0.03);
            let butt = [right[0], floor, right[2]];
            (butt, math::add(butt, math::scale(axis, SPEAR_LENGTH)))
        }
    };
    let axis = math::normalize_or(math::sub(tip, butt), facing);
    Weapon {
        butt,
        tip,
        flat: held_flat(p, which, axis, facing),
        held: true,
    }
}

/// A weapon a hand slides along, pointed from `right` at `to`: the tip on
/// `to`, and the shaft running back through the hand. It is only drawn longer
/// than itself when the hand has slid all the way to the butt and the volume
/// still reaches further; past `stretch` times its length the tip falls short.
fn slide(right: V3, to: V3, fallback: V3, length: f32, stretch: f32) -> (V3, V3) {
    let toward = math::sub(to, right);
    let axis = math::normalize_or(toward, fallback);
    let d = math::length(toward);
    let needed = d + PAST_THE_HAND;
    if needed <= length {
        return (math::sub(to, math::scale(axis, length)), to);
    }
    let drawn = needed.min(length * stretch);
    let butt = math::sub(right, math::scale(axis, PAST_THE_HAND));
    (butt, math::add(butt, math::scale(axis, drawn)))
}

/// The plane the held weapon's breadth lies in.
///
/// On an active frame the sword's edge and the hammer's face lead the cut:
/// the way the tip is moving, asked of the hit test one frame back. Otherwise
/// the sword and hammer stand their breadth up in the vertical plane of the
/// weapon, and the spear lays its leaf flat so it reads from above and behind,
/// where the camera is.
fn held_flat(p: &Player, which: u8, axis: V3, facing: V3) -> V3 {
    let up = [0.0, 1.0, 0.0];
    let side = math::cross(facing, up);
    if which == SPEAR {
        return square(axis, side)
            .or_else(|| square(axis, up))
            .unwrap_or(side);
    }
    if let Action::Active { kind, left } = p.action {
        let m = sim::moves::get(p.class, kind);
        if left < m.active {
            let mut before = *p;
            before.action = Action::Active {
                kind,
                left: left + 1,
            };
            if let (Some(now), Some(then)) = (state::hitbox(p), state::hitbox(&before)) {
                let moved = math::sub(fx3(now.to), fx3(then.to));
                if let Some(v) = square(axis, moved) {
                    return v;
                }
            }
        }
    }
    square(axis, up)
        .or_else(|| square(axis, facing))
        .unwrap_or(side)
}

/// A point `local` in a body frame, into the arena.
fn on(at: V3, turn: Quat, local: V3) -> V3 {
    math::add(at, turn.rotate(local))
}

fn dir(turn: Quat, local: V3) -> V3 {
    math::normalize_or(turn.rotate(local), [0.0, 1.0, 0.0])
}

/// The sword at rest: in its scabbard at her left hip, hilt forward and up,
/// point down and back.
fn stowed_sword(t: &Torso) -> Weapon {
    let s = scabbard(t);
    let axis = s.axis();
    // The hilt stands out of the scabbard's mouth; the blade is inside it.
    let butt = math::sub(s.butt, math::scale(axis, SWORD_HOLD + 0.06));
    Weapon {
        butt,
        tip: math::add(butt, math::scale(axis, SWORD_HOLD + SWORD_REACH)),
        flat: s.flat,
        held: false,
    }
}

/// The scabbard: hung from the left hip, angled down and back.
fn scabbard(t: &Torso) -> Weapon {
    let mouth = on(t.hips, t.hips_turn, [0.24, 0.02, 0.10]);
    let axis = dir(t.hips_turn, [0.12, -0.5, -0.86]);
    let flat = square(axis, t.hips_turn.rotate([0.0, 0.0, 1.0])).unwrap_or([0.0, 1.0, 0.0]);
    Weapon {
        butt: mouth,
        tip: math::add(mouth, math::scale(axis, SWORD_REACH - 0.02)),
        flat,
        held: false,
    }
}

/// The hammer at rest: hung at her right hip, head down beside the thigh and
/// the haft up behind the hip -- low, where a camera behind her looks past it
/// rather than through it, and where the one heavy thing she owns hangs the
/// way something heavy hangs.
fn stowed_hammer(t: &Torso) -> Weapon {
    let tip = on(t.hips, t.hips_turn, [-0.34, -0.62, -0.10]);
    let axis = dir(t.hips_turn, [-0.06, -1.0, 0.16]);
    let butt = math::sub(tip, math::scale(axis, HAMMER_LENGTH));
    let flat = square(axis, t.hips_turn.rotate([0.0, 0.0, 1.0])).unwrap_or([1.0, 0.0, 0.0]);
    Weapon {
        butt,
        tip,
        flat,
        held: false,
    }
}

/// The spear at rest: across her back the other way, head high over her left
/// shoulder.
fn stowed_spear(t: &Torso) -> Weapon {
    // Its middle between her shoulder blades and a hand behind them, and
    // steep enough that the butt clears the floor: the head ends up high and
    // off to her left, out from under the crosshair.
    let middle = on(t.chest, t.chest_turn, [0.0, 0.1, -0.3]);
    let axis = dir(t.chest_turn, [0.75, 1.0, -0.08]);
    let tip = math::add(middle, math::scale(axis, SPEAR_LENGTH * 0.5));
    let butt = math::sub(middle, math::scale(axis, SPEAR_LENGTH * 0.5));
    let flat = square(axis, t.chest_turn.rotate([0.0, 0.0, 1.0])).unwrap_or([1.0, 0.0, 0.0]);
    Weapon {
        butt,
        tip,
        flat,
        held: false,
    }
}

/// Every piece of one weapon, placed. `which` is [`SWORD`], [`HAMMER`] or
/// [`SPEAR`]; what shape and stuff each index is is [`MAKE`].
pub fn pieces(arms: &Arms, which: u8) -> [Piece; PIECES] {
    let w = arms.weapons[which as usize];
    let mut out = [HIDDEN; PIECES];
    match which {
        SWORD => sword(&w, &arms.scabbard, &mut out),
        HAMMER => hammer(&w, &mut out),
        _ => spear(&w, &mut out),
    }
    out
}

/// A piece between two distances along a weapon, measured from its butt.
fn span(w: &Weapon, from: f32, to: f32, thick: f32, breadth: f32) -> Piece {
    let axis = w.axis();
    let a = math::add(w.butt, math::scale(axis, from));
    let b = math::add(w.butt, math::scale(axis, to));
    between(a, b, w.flat, thick, breadth)
}

/// A piece from `a` to `b`, its breadth toward `flat`.
fn between(a: V3, b: V3, flat: V3, thick: f32, breadth: f32) -> Piece {
    let along = math::sub(b, a);
    let length = math::length(along);
    if length < 0.005 {
        return HIDDEN;
    }
    let along = math::scale(along, 1.0 / length);
    Piece {
        shown: true,
        centre: math::scale(math::add(a, b), 0.5),
        along,
        across: square(along, flat).unwrap_or(flat),
        size: [thick, length, breadth],
    }
}

fn sword(w: &Weapon, sheath: &Weapon, out: &mut [Piece; PIECES]) {
    let len = w.length();
    let guard = SWORD_HOLD + 0.04;
    // Pommel: a knob at the very end.
    out[0] = span(w, 0.0, 0.085, 0.085, 0.085);
    // Grip: room for both hands.
    out[1] = span(w, 0.06, guard - 0.02, 0.055, 0.055);
    // Crossguard: a bar across the blade's own plane.
    out[2] = span(w, guard - 0.022, guard + 0.022, 0.05, SWORD_GUARD);
    if w.held {
        // Blade, then the point it tapers to.
        out[3] = span(
            w,
            guard + 0.02,
            len - SWORD_POINT,
            SWORD_THICK,
            SWORD_BREADTH,
        );
        out[4] = span(w, len - SWORD_POINT, len, SWORD_THICK, SWORD_BREADTH * 1.15);
    }
    // The scabbard is always at her hip, empty or not.
    let sl = sheath.length();
    out[5] = span(sheath, 0.0, sl - 0.06, 0.05, SWORD_BREADTH + 0.05);
    out[6] = span(sheath, sl - 0.07, sl, 0.06, SWORD_BREADTH + 0.06);
}

fn hammer(w: &Weapon, out: &mut [Piece; PIECES]) {
    let len = w.length();
    let [thick, deep, across] = HAMMER_HEAD;
    let head = len - deep * 0.5;
    out[0] = span(w, 0.02, head, HAMMER_HAFT, HAMMER_HAFT);
    out[1] = span(w, 0.05, 0.5, HAMMER_HAFT * 1.3, HAMMER_HAFT * 1.3);
    out[2] = span(w, 0.0, 0.06, HAMMER_HAFT * 1.5, HAMMER_HAFT * 1.5);
    // Langets: iron straps down the haft under the head.
    out[3] = span(w, len - deep - 0.26, len - deep + 0.02, 0.1, 0.1);
    // The head, square across the end of the haft.
    out[4] = span(w, len - deep, len, thick, across);
    // Its two faces, flared: a little bigger than the head, at each end of it.
    let centre = math::add(w.butt, math::scale(w.axis(), head));
    let axis = w.axis();
    for (i, sign) in [(5, 1.0f32), (6, -1.0)] {
        let at = math::add(centre, math::scale(w.flat, sign * (across * 0.5 + 0.02)));
        let a = math::sub(at, math::scale(axis, deep * 0.62));
        let b = math::add(at, math::scale(axis, deep * 0.62));
        out[i] = between(a, b, w.flat, thick * 1.24, 0.07);
    }
}

fn spear(w: &Weapon, out: &mut [Piece; PIECES]) {
    let len = w.length();
    let widest = len - SPEAR_HEAD + 0.12;
    out[0] = span(w, 0.08, len - SPEAR_HEAD, SPEAR_SHAFT, SPEAR_SHAFT);
    out[1] = span(w, 0.5, 1.15, SPEAR_SHAFT * 1.25, SPEAR_SHAFT * 1.25);
    // The butt spike, pointing back: a cone runs from its base to its apex, so
    // it is drawn from the shaft to the butt.
    out[2] = between(
        math::add(w.butt, math::scale(w.axis(), 0.12)),
        w.butt,
        w.flat,
        SPEAR_SHAFT * 1.2,
        SPEAR_SHAFT * 1.2,
    );
    out[3] = span(
        w,
        len - SPEAR_HEAD - 0.02,
        len - SPEAR_HEAD + 0.08,
        SPEAR_SHAFT * 1.4,
        SPEAR_SHAFT * 1.4,
    );
    // The leaf: a short cone widening from the socket to the widest point,
    // then a long one tapering to the tip.
    let axis = w.axis();
    let at = |d: f32| math::add(w.butt, math::scale(axis, d));
    out[4] = between(
        at(widest),
        at(len - SPEAR_HEAD + 0.06),
        w.flat,
        0.035,
        SPEAR_BREADTH,
    );
    out[5] = between(at(widest), at(len), w.flat, 0.035, SPEAR_BREADTH);
    // A red tassel under the socket: colour on the one weapon that is
    // otherwise a long thin line.
    out[6] = span(
        w,
        len - SPEAR_HEAD - 0.16,
        len - SPEAR_HEAD - 0.02,
        0.09,
        0.09,
    );
}

/// The arc of a swing: the outer part of the hit capsule on each active frame
/// so far, oldest first, and how solid to draw it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Trail {
    /// Which weapon threw it, for its colour.
    pub which: u8,
    /// The inner and outer edge of the band on each remembered frame.
    pub inner: [V3; TRAIL],
    pub outer: [V3; TRAIL],
    /// How many of the entries are real.
    pub count: usize,
    /// One while the volume is out, running to nothing over the first
    /// [`TRAIL_GHOST`] frames of the recovery.
    pub fade: f32,
}

/// How far out along the capsule the trail's band starts, per weapon: a broad
/// crescent for the sword, a short heavy one at the head of the hammer, and a
/// streak at the spear's point.
pub const fn band(which: u8) -> f32 {
    match which {
        SWORD => 0.42,
        HAMMER => 0.62,
        _ => 0.82,
    }
}

/// The trail of the Champion's swing this frame, if there is one to draw.
pub fn trail(p: &Player) -> Option<Trail> {
    if p.class != sim::Class::Champion {
        return None;
    }
    let (kind, now, fade) = match p.action {
        Action::Active { kind, left } => (kind, left, 1.0),
        Action::Recovery { kind, left } => {
            let m = sim::moves::get(p.class, kind);
            let gone = m.recovery.saturating_sub(left);
            if gone >= TRAIL_GHOST {
                return None;
            }
            (kind, 1, 1.0 - gone as f32 / TRAIL_GHOST as f32)
        }
        _ => return None,
    };
    let m = sim::moves::get(p.class, kind);
    let which = champion::weapon(kind);
    let k = band(which);
    let mut t = Trail {
        which,
        inner: [math::ZERO; TRAIL],
        outer: [math::ZERO; TRAIL],
        count: 0,
        fade,
    };
    // Active frames count `left` down from `m.active` to one. The newest
    // `TRAIL` of those already thrown, oldest first.
    let first = m.active.min(now.saturating_add(TRAIL as u16 - 1));
    let mut left = first;
    while left >= now.max(1) && t.count < TRAIL {
        let mut then = *p;
        then.action = Action::Active { kind, left };
        if let Some(hb) = state::hitbox(&then) {
            let (from, to) = (fx3(hb.from), fx3(hb.to));
            if math::length(math::sub(to, from)) > 0.05 {
                t.inner[t.count] = math::add(from, math::scale(math::sub(to, from), k));
                t.outer[t.count] = to;
                t.count += 1;
            }
        }
        if left == 0 {
            break;
        }
        left -= 1;
    }
    (t.count >= 2).then_some(t)
}

/// The part of `v` square to `axis`, unit, or `None` if there is none.
fn square(axis: V3, v: V3) -> Option<V3> {
    let flat = math::sub(v, math::scale(axis, math::dot(v, axis)));
    (math::length(flat) > 1e-4).then(|| math::normalize_or(flat, axis))
}

fn fx3(v: sim::V3) -> V3 {
    [crate::fx(v.x), crate::fx(v.y), crate::fx(v.z)]
}
