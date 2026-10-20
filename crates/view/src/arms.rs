//! The Champion's weapon, as a thing you can see.
//!
//! **She has one weapon, and it is a liquid.** Between moves it is a coil of
//! golden quicksilver wound round her right forearm, which is none of the
//! three. In the middle of a move it runs off her hand into the shape of the
//! weapon that threw it -- the sword, the hammer or the spear -- and keeps that
//! shape for a few seconds after the move ends, so whoever is watching gets a
//! good look at what it *is*, before it runs slowly back up her arm. Calling a
//! different weapon while one is out takes a few frames: the sword's blade
//! runs into the hammer's head, and the hammer is solid by the first active
//! frame of the move that called it. [`Liquid`] is that, worked out from the
//! clocks in `sim::class::Quicksilver`, which are in the snapshot because a
//! morph that ran off the renderer's clock would replay differently every
//! time a rollback re-simulated the frame it began on.
//!
//! The shapes it takes are drawn to be **unlike each other at a glance**,
//! because a silhouette is the cheapest tell there is -- it is read from across
//! the arena before a single frame of the swing has been:
//!
//! - the **sword** is a longsword: a pommel, a two-hand grip, a crossguard and
//!   a straight blade tapering to a point. Middling in every dimension.
//! - the **hammer** is short and top-heavy: a stout haft with a big square head
//!   set across its end, flared at both faces. The only shape with mass at the
//!   end of it.
//! - the **spear** is nearly twice either of them, thin the whole way, with a
//!   leaf-shaped head and a collar under it.
//!
//! The liquid between them is [`BEADS`] drops, each a weighted blend of where
//! it sits in the coil and where it sits in each weapon: so the coil unwinds
//! off the wrist first and the drops nearest the elbow arrive last, and the
//! solid pieces of the weapon thicken out of the line the drops have drawn.
//! Melting is the same thing backwards, tip first.
//!
//! **The drawn weapon is where the hit test is.** On an active frame the tip is
//! the far end of `state::hitbox`, exactly, and the weapon runs back from it
//! through her leading hand -- the rule the Blood mage's scythe keeps, for the
//! reason `CLAUDE.md` gives the debug overlay -- and on an active frame the
//! weapon is always whole. The weapon's own size is fixed, and what gives when
//! the volume reaches further than the weapon at rest is the part a hand could
//! slide along: a spear's shaft or a hammer's haft runs through the leading
//! hand rather than being held at one point, and the sword's blade -- which a
//! hand cannot slide along -- is drawn to the length of the cut. Through the
//! wind-up and the recovery the weapon simply lies along the line of her two
//! hands, head beyond the right one, which is the convention
//! `anim::clips::champion::weapon` authors every grip in.
//!
//! [`trail`] is the arc of the cut, re-asked of the hit test for each active
//! frame so far: a band across the outer part of the capsule, per frame, fading
//! through the first frames of the recovery. It is what replaced the rod.
//!
//! Everything here is a pure function of simulation state plus the drawn hand
//! and torso positions, allocation-free, so `tests/arms.rs` can hold it to the
//! hit test without a renderer.

use crate::math::{self, Quat, V3};
use sim::class::{Form, Mechanic, Quicksilver};
use sim::moves::champion::{self, HAMMER, SPEAR, SWORD};
use sim::state::{self, Action, Player};

/// Pieces per weapon. Every weapon has this many entities in the renderer;
/// a piece a weapon does not use is simply never shown.
pub const PIECES: usize = 7;

/// Drops of liquid the coil is made of, and that run between one shape and
/// the next. Each is a [`Shape::Knob`] of [`Stuff::Liquid`].
pub const BEADS: usize = 34;

/// Frames the liquid takes to run into a weapon's shape when a move calls it:
/// a few, and never more than the move's startup, so the weapon is whole by
/// its first active frame.
pub const FORM_IN: u16 = 5;

/// Frames a weapon keeps its shape after the move that called it has ended:
/// a few seconds, long enough to see what it is.
pub const HOLD: u16 = 180;

/// Frames it takes to run back up her arm into the coil once it lets go:
/// slow, so the shape lingers as it goes.
pub const MELT: u16 = 80;

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

/// What a piece is made of. The renderer holds one material of each. All of
/// it is the same gold: the weapon is one liquid, and these are only how it
/// catches the light where it has set into an edge, a body or a grip.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stuff {
    /// Set to an edge: pale, polished, the brightest thing on her. Blades,
    /// points, a hammer's faces.
    Edge,
    /// Set to a body: the hammer's head, a crossguard, a pommel.
    Gold,
    /// Set to a haft or a grip: deeper, duller, where hands go.
    Deep,
    /// Still running: the coil and the drops between shapes.
    Liquid,
}

/// What each piece of each weapon is, fixed so the renderer can make its
/// meshes once. Indexed `[weapon][piece]`.
pub const MAKE: [[(Shape, Stuff); PIECES]; WEAPONS] = [
    // Sword: pommel, grip, crossguard, blade, point. Two unused.
    [
        (Shape::Knob, Stuff::Gold),
        (Shape::Rod, Stuff::Deep),
        (Shape::Block, Stuff::Gold),
        (Shape::Block, Stuff::Edge),
        (Shape::Point, Stuff::Edge),
        (Shape::Knob, Stuff::Gold),
        (Shape::Knob, Stuff::Gold),
    ],
    // Hammer: haft, grip, butt cap, langets, head, the two faces.
    [
        (Shape::Rod, Stuff::Deep),
        (Shape::Rod, Stuff::Deep),
        (Shape::Rod, Stuff::Gold),
        (Shape::Block, Stuff::Gold),
        (Shape::Block, Stuff::Gold),
        (Shape::Block, Stuff::Edge),
        (Shape::Block, Stuff::Edge),
    ],
    // Spear: shaft, grip, butt spike, socket, the head's two halves, collar.
    [
        (Shape::Rod, Stuff::Deep),
        (Shape::Rod, Stuff::Deep),
        (Shape::Point, Stuff::Gold),
        (Shape::Rod, Stuff::Gold),
        (Shape::Point, Stuff::Edge),
        (Shape::Point, Stuff::Edge),
        (Shape::Knob, Stuff::Gold),
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
/// (`+X` her left, `+Y` up, `+Z` forward) into the arena's, and the shoulder
/// and elbow of the arm the liquid coils round -- the one that holds the
/// weapon.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Torso {
    pub chest: V3,
    pub chest_turn: Quat,
    pub hips: V3,
    pub hips_turn: Quat,
    /// The weapon arm's shoulder and elbow: the coil runs from the one, by
    /// the other, to the hand.
    pub shoulder: V3,
    pub elbow: V3,
}

/// How much of the liquid is in each shape, worked out from the clocks.
///
/// The liquid is a blend of four shapes -- the coil and the three weapons --
/// and these are the weapons' shares of it; the coil has what is left.
/// `weapon` is the one being called, or last called, and `was` the shape it
/// was called out of, which is still running out of the picture for the
/// first [`FORM_IN`] frames.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Liquid {
    pub weapon: u8,
    /// How much of `weapon` there is: zero is the coil, one is all of it.
    pub solid: f32,
    pub was: Option<u8>,
    /// How much of `was` there was when `weapon` was called.
    pub was_solid: f32,
}

impl Liquid {
    /// Each weapon's share of the liquid, by [`SWORD`], [`HAMMER`], [`SPEAR`].
    pub fn weights(&self) -> [f32; WEAPONS] {
        let mut w = [0.0; WEAPONS];
        w[self.weapon as usize] += self.solid;
        if let Some(was) = self.was {
            w[was as usize] += (1.0 - self.solid) * self.was_solid;
        }
        w
    }
}

/// How far into its shape the liquid has run, `forged` frames after it was
/// called with `within` frames to get there.
fn grown(forged: u16, within: u16) -> f32 {
    (forged as f32 / within.max(1) as f32).min(1.0)
}

/// How much of a shape is still held, `rest` frames after the move that
/// called it ended.
fn kept(rest: u16) -> f32 {
    if rest <= HOLD {
        return 1.0;
    }
    1.0 - ((rest - HOLD) as f32 / MELT as f32).min(1.0)
}

fn index(form: Form) -> u8 {
    match form {
        Form::Sword => SWORD,
        Form::Hammer => HAMMER,
        Form::Spear => SPEAR,
    }
}

/// How much of which weapon the Champion's liquid is in, this frame.
pub fn liquid(p: &Player) -> Option<Liquid> {
    let weapon = in_hand(p)?;
    let (formed, q) = match p.mechanic {
        Mechanic::Forms { form, liquid, .. } => (index(form), liquid),
        _ => (weapon, Quicksilver::AT_REST),
    };
    let within = match p.action.attack_kind() {
        Some(kind) => FORM_IN.min(sim::moves::get(p.class, kind).startup),
        None => FORM_IN,
    };
    if weapon != formed {
        // A move that called no weapon: whatever threw it, it is whole.
        return Some(Liquid {
            weapon,
            solid: 1.0,
            was: None,
            was_solid: 0.0,
        });
    }
    let t = grown(q.forged(), within);
    let solid = smooth(t).min(kept(q.rest()));
    // What it ran out of only matters while it is still running out of it.
    let was = q.was().map(index).filter(|_| t < 1.0);
    let was_solid = match was {
        Some(_) => grown(q.was_forged(), FORM_IN).min(kept(q.was_rest())),
        None => 0.0,
    };
    Some(Liquid {
        weapon,
        solid,
        was,
        was_solid,
    })
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Everything the Champion carries: where each of the three shapes would be
/// in her hands this frame, indexed by [`SWORD`], [`HAMMER`] and [`SPEAR`],
/// how much of the liquid is in each, and the drops of it that are not.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Arms {
    pub weapons: [Weapon; WEAPONS],
    /// Each weapon's share of the liquid: one is that weapon, whole.
    pub weight: [f32; WEAPONS],
    /// The weapon being called, or last called.
    pub held: u8,
    /// The drops: the coil round her forearm, or the liquid between shapes.
    /// None is shown once the weapon is whole.
    pub beads: [Piece; BEADS],
    /// The hand the weapon is held in.
    pub hand: V3,
}

impl Arms {
    /// The weapon is whole: no liquid is anywhere but in its shape.
    pub fn whole(&self) -> bool {
        self.weight[self.held as usize] >= 0.999
    }

    /// The far end of whatever she is holding: the weapon's tip, or her
    /// hand while it is mostly coil.
    pub fn head(&self) -> V3 {
        let w = self.weight[self.held as usize];
        if w >= 0.5 {
            self.weapons[self.held as usize].tip
        } else {
            self.hand
        }
    }
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

/// Where the Champion's weapon is and what shape it is in, if this fighter
/// is a Champion.
///
/// `left` and `right` are where the body's drawn hands are, and `torso` where
/// its chest, hips and weapon elbow are: the renderer knows those and the
/// simulation does not. They decide where the weapon is held and where the
/// coil winds, never how far a swing reaches.
pub fn champion_arms(p: &Player, left: V3, right: V3, torso: &Torso) -> Option<Arms> {
    let liquid = liquid(p)?;
    let held = liquid.weapon;
    let weight = liquid.weights();
    let facing = math::normalize_or(
        [crate::fx(p.facing.x), 0.0, crate::fx(p.facing.z)],
        [1.0, 0.0, 0.0],
    );
    let mut weapons = [Weapon {
        butt: right,
        tip: right,
        flat: [0.0, 1.0, 0.0],
    }; WEAPONS];
    for (which, w) in weapons.iter_mut().enumerate() {
        if weight[which] > 0.0 || which == held as usize {
            *w = in_the_hands(p, which as u8, left, right, facing);
        }
    }
    let mut arms = Arms {
        weapons,
        weight,
        held,
        beads: [HIDDEN; BEADS],
        hand: right,
    };
    if !arms.whole() {
        arms.beads = beads(&arms, torso, right);
    }
    Some(arms)
}

/// The drops of liquid: each one a blend of where it sits in the coil and
/// where it sits in each weapon, by the weapons' weights.
///
/// Drop `i` runs from the shoulder end of the coil to the butt of the weapon,
/// and the wrist end to the tip, and the wrist end goes first: the coil
/// unwinds off the hand and reaches out into the shape, and melts back the
/// same way, the tip arriving at the wrist last.
fn beads(arms: &Arms, torso: &Torso, hand: V3) -> [Piece; BEADS] {
    let total: f32 = arms.weight.iter().sum();
    let mut out = [HIDDEN; BEADS];
    for (i, bead) in out.iter_mut().enumerate() {
        let t = i as f32 / (BEADS - 1) as f32;
        // How far this drop has run out of the coil: the wrist end leads.
        let lag = 0.55 * (1.0 - t);
        let out_of_coil = smooth((total * (1.0 + lag) - lag).clamp(0.0, 1.0));
        let coil = coiled(torso.shoulder, torso.elbow, hand, t);
        let mut centre = math::scale(coil.centre, 1.0 - out_of_coil);
        let mut along = math::scale(coil.along, 1.0 - out_of_coil);
        let mut across = math::scale(coil.across, 1.0 - out_of_coil);
        let mut size = math::scale(coil.size, 1.0 - out_of_coil);
        if total > 0.0 {
            for which in 0..WEAPONS {
                let share = out_of_coil * arms.weight[which] / total;
                if share <= 0.0 {
                    continue;
                }
                let d = in_weapon(&arms.weapons[which], which as u8, t);
                centre = math::add(centre, math::scale(d.centre, share));
                along = math::add(along, math::scale(d.along, share));
                across = math::add(across, math::scale(d.across, share));
                size = math::add(size, math::scale(d.size, share));
            }
        }
        // In flight a drop rounds off and stretches a little along the way it
        // is going: the liquid runs, rather than sliding as flat pieces.
        let running = (out_of_coil * std::f32::consts::PI).sin();
        let round = (size[0] + size[2]) * 0.5;
        size[0] += (round - size[0]) * running;
        size[2] += (round - size[2]) * running;
        size[1] = size[1].max(round) * (1.0 + 0.5 * running);
        let along = math::normalize_or(along, [0.0, 1.0, 0.0]);
        *bead = Piece {
            shown: true,
            centre,
            along,
            across: square(along, across).unwrap_or([0.0, 0.0, 1.0]),
            size,
        };
    }
    out
}

/// Drop `t` of the coil: a helix wound up the arm, from just below the
/// shoulder by the elbow to the wrist, each drop long enough to overlap the
/// next so the coil reads as one band of metal. `t` is zero at the shoulder.
fn coiled(shoulder: V3, elbow: V3, hand: V3, t: f32) -> Piece {
    let step = 1.0 / (BEADS - 1) as f32;
    let here = wound(shoulder, elbow, hand, t);
    let ahead = wound(shoulder, elbow, hand, t + step * 0.5);
    let behind = wound(shoulder, elbow, hand, t - step * 0.5);
    let tangent = math::sub(ahead, behind);
    let length = math::length(tangent) * 1.5;
    let along = math::normalize_or(tangent, [0.0, -1.0, 0.0]);
    // The band's breadth lies along the arm, so it wraps flat round it.
    let (_, axis) = on_the_arm(shoulder, elbow, hand, t);
    let across = square(along, axis).unwrap_or([0.0, 1.0, 0.0]);
    Piece {
        shown: true,
        centre: here,
        along,
        across,
        size: [0.04, length.max(0.04), 0.052],
    }
}

/// How many times the coil goes round the arm, shoulder to wrist.
const TURNS: f32 = 3.0;

/// A point `t` of the way along the coil's helix.
fn wound(shoulder: V3, elbow: V3, hand: V3, t: f32) -> V3 {
    let (at, axis) = on_the_arm(shoulder, elbow, hand, t);
    // Square to the arm: one direction off the vertical (or off the arena's
    // side, for an arm that hangs straight), and the other across both.
    let u = square(axis, [0.0, 1.0, 0.0])
        .or_else(|| square(axis, [1.0, 0.0, 0.0]))
        .unwrap_or([0.0, 0.0, 1.0]);
    let v = math::cross(axis, u);
    let angle = t * TURNS * std::f32::consts::TAU;
    // Thicker round the upper arm than round the wrist.
    let radius = 0.085 - 0.025 * t;
    math::add(
        at,
        math::add(
            math::scale(u, radius * angle.cos()),
            math::scale(v, radius * angle.sin()),
        ),
    )
}

/// The point `t` of the way down the arm the coil covers, and the arm's
/// direction there: from a little below the shoulder, by the elbow, to the
/// wrist.
fn on_the_arm(shoulder: V3, elbow: V3, hand: V3, t: f32) -> (V3, V3) {
    let upper = math::length(math::sub(elbow, shoulder));
    let lower = math::length(math::sub(hand, elbow));
    let from = upper * 0.2;
    let to = upper + lower * 0.85;
    let d = from + (to - from) * t.clamp(0.0, 1.0);
    if d <= upper {
        let axis = math::normalize_or(math::sub(elbow, shoulder), [0.0, -1.0, 0.0]);
        (math::add(shoulder, math::scale(axis, d)), axis)
    } else {
        let axis = math::normalize_or(math::sub(hand, elbow), [0.0, -1.0, 0.0]);
        (math::add(elbow, math::scale(axis, d - upper)), axis)
    }
}

/// Drop `t` of a weapon's shape: spread butt to tip, each as thick as the
/// weapon is where it sits, so the drops draw the weapon's outline before
/// its pieces set.
fn in_weapon(w: &Weapon, which: u8, t: f32) -> Piece {
    let len = w.length().max(0.05);
    let axis = w.axis();
    let at = len * t;
    let (thick, breadth) = match which {
        SWORD if at < SWORD_HOLD + 0.04 => (0.06, 0.06),
        SWORD => (0.035, SWORD_BREADTH),
        HAMMER if at > len - HAMMER_HEAD[1] * 1.3 => (HAMMER_HEAD[0] * 0.8, HAMMER_HEAD[2] * 0.8),
        HAMMER => (HAMMER_HAFT, HAMMER_HAFT),
        _ if at > len - SPEAR_HEAD => (0.04, SPEAR_BREADTH * 0.8),
        _ => (SPEAR_SHAFT, SPEAR_SHAFT),
    };
    Piece {
        shown: true,
        centre: math::add(w.butt, math::scale(axis, at)),
        along: axis,
        across: w.flat,
        size: [thick, (len / (BEADS - 1) as f32) * 1.6, breadth],
    }
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
    Weapon { butt, tip, flat }
}

/// The weapon in her hand between moves, for the seconds it keeps its shape.
///
/// The Champion has no idle of her own -- she stands and walks on the clips
/// every class shares, whose hands are not on anything -- so while the shape
/// holds it is carried the way that weapon is carried, off her right hand
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

/// Every piece of one weapon, placed, as far as it has set. `which` is
/// [`SWORD`], [`HAMMER`] or [`SPEAR`]; what shape and stuff each index is is
/// [`MAKE`].
///
/// A piece sets out of the line the drops have drawn: it thickens from
/// nothing to its full breadth as the weapon's share of the liquid rises, the
/// pieces nearest the hand first, and thins away again tip first as it melts.
pub fn pieces(arms: &Arms, which: u8) -> [Piece; PIECES] {
    let w = arms.weapons[which as usize];
    let weight = arms.weight[which as usize];
    let mut out = [HIDDEN; PIECES];
    if weight <= 0.0 {
        return out;
    }
    match which {
        SWORD => sword(&w, &mut out),
        HAMMER => hammer(&w, &mut out),
        _ => spear(&w, &mut out),
    }
    let len = w.length().max(0.05);
    for (i, piece) in out.iter_mut().enumerate() {
        if !piece.shown {
            continue;
        }
        let far = (math::dot(math::sub(piece.centre, w.butt), w.axis()) / len).clamp(0.0, 1.0);
        let set = smooth((weight * 1.5 - 0.3 - 0.2 * far) / 0.8);
        if set <= 0.01 {
            *piece = HIDDEN;
            continue;
        }
        piece.size[0] *= set;
        piece.size[2] *= set;
        if MAKE[which as usize][i].0 == Shape::Knob {
            piece.size[1] *= set;
        }
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

fn sword(w: &Weapon, out: &mut [Piece; PIECES]) {
    let len = w.length();
    let guard = SWORD_HOLD + 0.04;
    // Pommel: a knob at the very end.
    out[0] = span(w, 0.0, 0.085, 0.085, 0.085);
    // Grip: room for both hands.
    out[1] = span(w, 0.06, guard - 0.02, 0.055, 0.055);
    // Crossguard: a bar across the blade's own plane.
    out[2] = span(w, guard - 0.022, guard + 0.022, 0.05, SWORD_GUARD);
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
    // A collar under the socket, where the liquid gathers before it runs
    // out into the leaf.
    out[6] = span(
        w,
        len - SPEAR_HEAD - 0.1,
        len - SPEAR_HEAD - 0.01,
        0.085,
        0.085,
    );
}

/// The arc of a swing: the outer part of the hit capsule on each active frame
/// so far, oldest first, and how solid to draw it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Trail {
    /// Which weapon threw it, for its colour.
    pub which: u8,
    /// Where in the string the swing stands: 0, 1 or 2 for its first, second
    /// and third hit, [`FINISHER`] for a finisher. Each is drawn bigger than
    /// the one before, and the third in gold (2026-10-20, from play): a
    /// player counting to three should see the count, not only feel it.
    pub place: u8,
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

/// [`Trail::place`] for a finisher.
pub const FINISHER: u8 = 3;

/// The band at a place in the string: the first hit's is the outer half of
/// the weapon's own, the second's the weapon's own, and the third's -- and a
/// finisher's -- reaches further in, so the three read as a climb.
pub fn band_at(which: u8, place: u8) -> f32 {
    let own = band(which);
    match place {
        0 => own + (1.0 - own) * 0.5,
        1 => own,
        _ => own * 0.6,
    }
}

/// Where a Champion's move stands in his string, for the trail and the
/// third hit's flare: a link's own place, an aerial's or a takeoff's the one
/// it was thrown at, [`FINISHER`] for a finisher, and 0 for a Rush move.
pub fn place_in_string(p: &Player, kind: u8) -> u8 {
    if champion::is_full(kind) {
        return FINISHER;
    }
    if let Some(link) = champion::link_of(kind) {
        return link;
    }
    match p.mechanic {
        Mechanic::Forms { fury, .. } if champion::in_string(kind) => fury.place,
        _ => 0,
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
    let place = place_in_string(p, kind);
    let k = band_at(which, place);
    let mut t = Trail {
        which,
        place,
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
