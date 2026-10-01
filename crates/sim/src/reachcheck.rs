//! **Can a fighter on the floor strike that part?** The instrument the
//! Broodmother's §1 asks for, made generic: `beastcheck` prints it for any
//! breakable part of any species, and `tests/broodmother.rs` holds her sacs to
//! the rules her document writes.
//!
//! A creature's heights against a hop's apex are only half the question --
//! what a swing reaches from the top of a hop is the class's own shape (the
//! Champion's sword goes three and a half metres over his feet; the Bulwark's
//! hammer a metre and a half), and how close a body can get to a part is the
//! creature's (its solid boxes push you off). Arguments about that conducted
//! in prose go wrong, so this **plays it**: the creature held in one state,
//! a fighter of one class put on the floor beside the part -- in eight
//! directions, as near as the creature's solid boxes let a body stand, and
//! half a metre further out -- walking in toward it with the crosshair on the
//! part's middle (`aim::look_onto_closely`), and either swinging there or
//! hopping and swinging at the top of the hop, each auto in turn. A trial
//! counts when the part loses health: the fight's own answer.
//!
//! Float-free and deterministic like the rest of the crate. It builds worlds,
//! so it allocates; it is a tool, not a frame.

use crate::class::Class;
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::V3;
use crate::monster::Doing;
use crate::species::SpeciesId;
use crate::state::{MAX_PLAYERS, World};

/// What a class can do to one part.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reach {
    /// From the floor beside it, swinging where it stands.
    pub standing: bool,
    /// From the top of a full hop taken beside it.
    pub hop: bool,
    /// Where the first hop that reached it was taken from.
    pub hop_from: Option<V3>,
    /// From the floor straight under it, standing or hopping: a place the
    /// creature's own body overhangs.
    pub under: bool,
}

/// How many bodies tall the column over a footing is asked about: the Dual
/// mage's float tops out under six metres, and her head is a body above that.
const COLUMN: usize = 5;

/// The eight directions a fighter is put on, in turns.
const BEARINGS: [i32; 8] = [0, 8192, 16384, 24576, 32768, 40960, 49152, 57344];

/// The world, with the creature in the middle of its arena held in `doing`,
/// facing along `+x`, nothing else in it but the fighter.
pub fn stage(species: SpeciesId, class: Class) -> World {
    let mut w = World::hunt_of([class; MAX_PLAYERS], species);
    w.pack = None;
    for c in w.critters.iter_mut() {
        *c = crate::critter::Critter::EMPTY;
    }
    // The second fighter out of the way: in the corner of the arena, past the
    // creature's keep-out from the wall.
    let b = w.arena().bounds;
    w.players[1].pos = V3::new(b.lo_x, Fx::ZERO, b.lo_z);
    w
}

/// Pin the creature where the trial wants it, every frame: still, in the
/// state being measured, not thinking.
pub fn pin(w: &mut World, doing: Doing, at: V3) {
    pin_with(w, doing, at, |_| {});
}

/// [`pin`], and then `prepare` on the creature: what a trial wants of it
/// besides its state -- the Broodmother with only one sac on her back, so a
/// swing that touches two is asked about the one being measured.
pub fn pin_with(w: &mut World, doing: Doing, at: V3, prepare: fn(&mut crate::monster::Monster)) {
    if let Some(m) = w.monsters[0].as_mut() {
        m.pos = at;
        m.yaw = Fx::ZERO;
        m.yaw_rate = Fx::ZERO;
        m.speed = Fx::ZERO;
        m.doing = doing;
        m.brain.think_left = 600;
        m.brain.grace = 600;
        m.rooted = 2;
        m.health = m.sp().health();
        prepare(m);
    }
}

/// Where the creature stands for a trial: the middle of its arena's hunt
/// marks, or the origin.
pub fn centre(w: &World) -> V3 {
    w.monsters[0].map_or(V3::ZERO, |m| V3::new(m.pos.x, Fx::ZERO, m.pos.z))
}

/// The world middle of a part, the creature posed in `doing` at `at`.
pub fn middle(w: &World, part: usize) -> V3 {
    let Some(m) = w.monsters[0] else {
        return V3::ZERO;
    };
    let sh = m.sp().shape(part);
    let mid = |a: Fx, b: Fx| a.add(b).mul(Fx::ratio(1, 2));
    m.rig().part_to_world(
        part,
        V3::new(
            mid(sh.min.x, sh.max.x),
            mid(sh.min.y, sh.max.y),
            mid(sh.min.z, sh.max.z),
        ),
    )
}

/// Where a fighter can stand on the floor nearest `toward` along a bearing,
/// **with nothing over them**: the first point, walking in from five metres
/// out, at which a body in the column above it -- from the floor up five
/// bodies, past where any hop tops out -- would be inside one of the creature's
/// solid boxes, less a step back. A body under an overhang hits its head on
/// it and never reaches the top of its hop; [`UNDER`] is that spot, tried
/// on its own ([`under`]).
pub fn footing(w: &World, toward: V3, bearing: i32) -> V3 {
    let Some(m) = w.monsters[0] else {
        return toward;
    };
    let r = crate::tuning::body_radius();
    let h = crate::tuning::body_height();
    let under = V3::new(toward.x, Fx::ZERO, toward.z);
    let blocked = |p: V3| {
        (0..COLUMN).any(|k| {
            let at = V3::new(p.x, h.mul(Fx::from_int(k as i32)), p.z);
            let c = m.resolve(at, r, h);
            c.shoved || c.landed.is_some() || c.pos != at
        })
    };
    // From as far out as the creature keeps from a wall -- its own measure of
    // how far its body reaches -- walking in half a body at a time.
    let dir = V3::from_turns(Fx::from_raw(bearing));
    let step = r.mul(Fx::ratio(1, 2));
    let mut d = m.sp().margin();
    let mut last = under.add(dir.scale(d));
    while d.raw() > 0 {
        let p = under.add(dir.scale(d));
        if blocked(p) {
            return last;
        }
        last = p;
        d = d.sub(step);
    }
    last
}

/// The floor straight under a part, if a body standing there is clear of the
/// creature's solid boxes.
pub fn under(w: &World, toward: V3) -> Option<V3> {
    let m = w.monsters[0]?;
    let p = V3::new(toward.x, Fx::ZERO, toward.z);
    let c = m.resolve(
        p,
        crate::tuning::body_radius(),
        crate::tuning::body_height(),
    );
    (!c.shoved && c.landed.is_none() && c.pos == p).then_some(p)
}

/// The yaw, in turns, from `from` to `to` in the floor plane.
fn yaw_to(from: V3, to: V3) -> u16 {
    let d = to.sub(from);
    (crate::math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16
}

/// One trial: a fighter of `class` at `from`, the crosshair on the part,
/// swinging `button` where it stands -- after walking in toward the part --
/// or from the top of a hop taken on the spot, drifting toward it only once
/// off the floor. True if the part lost health.
#[allow(clippy::too_many_arguments)]
pub fn trial(
    base: &World,
    doing: Doing,
    at: V3,
    part: usize,
    from: V3,
    button: u16,
    hop: bool,
    prepare: fn(&mut crate::monster::Monster),
) -> bool {
    let mut w = base.clone();
    pin_with(&mut w, doing, at, prepare);
    let target = middle(&w, part);
    w.players[0].pos = from;
    w.players[0].vel = V3::ZERO;
    let Some(slot) = w.monsters[0].and_then(|m| m.sp().break_slot(part)) else {
        return false;
    };
    let before = w.monsters[0].map_or(0, |m| m.breaks[slot]);
    let mut pressed = false;
    let mut rising = false;
    for frame in 0..110 {
        pin_with(&mut w, doing, at, prepare);
        let p = &w.players[0];
        let aim = yaw_to(p.pos, target);
        let pitch = crate::aim::look_onto_closely(p.pos, aim, p.aloft, target);
        let mut bits = if !hop || !p.grounded { Input::W } else { 0 };
        // Held while rising: the Dual mage's float is a held jump.
        if hop && !pressed && (frame < 4 || p.vel.y.raw() > 0) {
            bits |= Input::SPACE;
        }
        if p.vel.y.raw() > 0 {
            rising = true;
        }
        let fire = if hop {
            rising && p.vel.y.raw() <= 0 && !pressed
        } else {
            frame == 12
        };
        if fire {
            bits |= button;
            pressed = true;
        }
        let look = Input::looking_at(bits, aim, pitch);
        w.advance([look, Input::default()]);
        let now = w.monsters[0].map_or(before, |m| m.breaks[slot]);
        if now < before {
            return true;
        }
    }
    false
}

/// **Can `class` strike `part` of a `species` held in `doing`** -- from the
/// floor beside it, from the top of a hop beside it, and from the floor
/// straight under it? Every bearing, two distances, both autos, until one
/// lands.
pub fn reach(species: SpeciesId, doing: Doing, class: Class, part: usize) -> Reach {
    reach_with(species, doing, class, part, |_| {})
}

/// [`reach`], with `prepare` done to the creature every frame of every trial
/// (see [`pin_with`]).
pub fn reach_with(
    species: SpeciesId,
    doing: Doing,
    class: Class,
    part: usize,
    prepare: fn(&mut crate::monster::Monster),
) -> Reach {
    let mut base = stage(species, class);
    let at = centre(&base);
    pin_with(&mut base, doing, at, prepare);
    // A frame for the world to settle with the fighter out of the way, where
    // the other one is.
    base.players[0].pos = base.players[1].pos;
    base.advance([Input::default(); MAX_PLAYERS]);
    pin_with(&mut base, doing, at, prepare);
    let target = middle(&base, part);
    let mut out = Reach::default();
    let buttons = [Input::LEFT, Input::RIGHT];
    if let Some(below) = under(&base, target) {
        out.under = buttons.iter().any(|b| {
            trial(&base, doing, at, part, below, *b, false, prepare)
                || trial(&base, doing, at, part, below, *b, true, prepare)
        });
    }
    for bearing in BEARINGS {
        let near = footing(&base, target, bearing);
        let dir = V3::from_turns(Fx::from_raw(bearing));
        for from in [near, near.add(dir.scale(Fx::ratio(1, 2)))] {
            for button in buttons {
                if !out.standing && trial(&base, doing, at, part, from, button, false, prepare) {
                    out.standing = true;
                }
                if !out.hop && trial(&base, doing, at, part, from, button, true, prepare) {
                    out.hop = true;
                    out.hop_from = Some(from.sub(at));
                }
                if out.standing && out.hop {
                    return out;
                }
            }
        }
    }
    out
}
