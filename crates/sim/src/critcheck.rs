//! **Can every move touch something short?** The instrument the Gnawers' §10
//! M2 asks for, made generic: `cargo run -p sim --bin critcheck` prints it,
//! and `tests/critters.rs` holds the roster to it.
//!
//! The Ridgeback found that a level shot goes under a belly; a knee-high body
//! is the mirror image, and a level shot goes over its back. Arguments about
//! that conducted in prose go wrong, so this stands one critter of a pack's
//! kind in front of a fighter at a few distances, puts the crosshair's ray
//! through the middle of it ([`crate::aim::look_onto`]), **presses** every
//! move the class has ([`World::press`], the real `begin_move`) and runs the
//! world until the move is over. Two answers per trial:
//!
//! - **hurt**: the critter lost health -- by the swing's volume, a beam, a
//!   bolt, a thrown blade, a field, anything. The fight's own answer.
//! - **drawn**: on some frame, the volume [`state::hitbox`] describes -- the
//!   one the debug overlay draws -- touched the critter's box. For a swing the
//!   two must agree, which is the overlay rule; for a move whose damage is
//!   carried by something else (a bolt, an effect) only `hurt` can say.
//!
//! Float-free and deterministic like the rest of the crate. It builds worlds,
//! so it allocates; it is a tool, not a frame.

use crate::class::Class;
use crate::critter::{Critter, is};
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::V3;
use crate::species::SpeciesId;
use crate::state::{self, Action, MAX_PLAYERS, World};

/// The distances a critter is stood at, in metres: close, a step, a lunge,
/// and the Gnawers' ring.
pub const METRES: [i32; 4] = [1, 2, 3, 5];

/// What one trial found.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Trial {
    /// The critter lost health.
    pub hurt: bool,
    /// The overlay's volume touched its box on some frame.
    pub drawn: bool,
    /// **The control**: the same move, the same distance, the crosshair on the
    /// middle of a *fighter* standing there instead. What the aim model is
    /// held to is that a short body is touched wherever a fighter would be.
    pub fighter: bool,
    /// The move carried the fighter into or past the critter before it
    /// landed. Fighters pass through critters (a pack must not be able to wall
    /// anybody in) and are stopped by a fighter, so a lunge that hits a
    /// fighter at a step's distance runs over a gnawer there. Geometry of the
    /// lunge, not of the aim, and reported apart from "over it".
    pub ran_through: bool,
}

/// One move against one kind of critter at every distance.
#[derive(Clone, Debug)]
pub struct Row {
    pub class: Class,
    pub kind: u8,
    pub name: &'static str,
    pub aim: crate::aim::Kind,
    pub trials: [Trial; METRES.len()],
}

/// Where every trial stands: open floor in the proving ground, clear of both
/// platforms and the walls out past the farthest of [`METRES`]. The fighter is
/// here, looking down +X; the body is that many metres along it. At `y = 0`:
/// a creature's own arena may have relief under this lane
/// (`arena::relief`), and [`on_the_floor`] puts a body down on it.
pub fn lane() -> V3 {
    V3::new(Fx::from_int(-6), Fx::ZERO, Fx::from_int(-9))
}

/// A point put down on the arena's floor, wherever the floor is there.
fn on_the_floor(w: &World, at: V3) -> V3 {
    V3::new(at.x, w.arena().ground_under(at), at.z)
}

/// Put fighter 0 at the start of the lane looking down +X, with her second body -- if
/// she has one -- attending at her shoulder, as a round starts.
fn stand_at_origin(w: &mut World) {
    let at = on_the_floor(w, lane());
    let p = &mut w.players[0];
    p.pos = at;
    p.facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    if let crate::class::Mechanic::Shadow(_) = p.mechanic {
        p.mechanic =
            crate::class::Mechanic::Shadow(crate::class::Shadow::attending(p.pos, p.facing));
    }
}

/// Stand an idle fighter `metres` in front of a fighter of `class`,
/// crosshair on its middle, press `move_kind`, and say whether it landed. The
/// control every critter trial is read against.
pub fn fighter_trial(class: Class, move_kind: u8, metres: Fx) -> bool {
    let mut w = World::with_classes([class; MAX_PLAYERS]);
    stand_at_origin(&mut w);
    w.players[1].pos = on_the_floor(&w, lane().add(V3::new(metres, Fx::ZERO, Fx::ZERO)));
    w.players[1].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    let idle = Input::default();
    for _ in 0..2 {
        w.advance([Input::looking_at(0, 0, 0), idle]);
    }
    let p1 = w.players[1].pos;
    let middle = crate::aim::standing_middle(p1, crate::tuning::body_height());
    let pitch = crate::aim::look_onto_closely(w.players[0].pos, 0, w.players[0].aloft, middle);
    let look = Input::looking_at(0, 0, pitch);
    let before = w.players[1].health;
    w.press(0, move_kind, look);
    for _ in 0..240 {
        w.advance([look, idle]);
        if w.players[1].health < before {
            return true;
        }
        let busy = matches!(
            w.players[0].action,
            Action::Startup { .. }
                | Action::Active { .. }
                | Action::Recovery { .. }
                | Action::Channel { .. }
        );
        let lingering = w.effects.iter().any(Option::is_some)
            || w.bolts.iter().any(Option::is_some)
            || w.gusts.iter().any(Option::is_some);
        if !busy && !lingering {
            return false;
        }
    }
    false
}

impl Row {
    /// Touched at some distance, by the fight's own answer.
    pub fn touches(&self) -> bool {
        self.trials.iter().any(|t| t.hurt)
    }

    /// Distances where a fighter was touched and the critter was not: **over
    /// its back**. What the aim model must keep at none.
    pub fn over(&self) -> impl Iterator<Item = i32> + '_ {
        self.trials
            .iter()
            .zip(METRES)
            .filter(|(t, _)| t.fighter && !t.hurt && !t.ran_through)
            .map(|(_, m)| m)
    }

    /// Distances where the move carried the fighter through the critter
    /// before it could land, where a fighter would have stopped it.
    pub fn ran_through(&self) -> impl Iterator<Item = i32> + '_ {
        self.trials
            .iter()
            .zip(METRES)
            .filter(|(t, _)| t.fighter && !t.hurt && t.ran_through)
            .map(|(_, m)| m)
    }

    /// Does the move do damage at all, by its own row? A move that is
    /// movement or a toggle has nothing to touch anything with.
    pub fn is_an_attack(&self) -> bool {
        let m = crate::moves::get(self.class, self.kind);
        m.damage > 0 || crate::effects::EffectKind::from_code(m.effect).is_some()
    }
}

/// Stand one critter of `kind` (of `species`' pack) `metres` in front of a
/// fighter of `class`, crosshair on its middle, and press `move_kind`.
pub fn trial(species: SpeciesId, kind: u8, class: Class, move_kind: u8, metres: Fx) -> Trial {
    let mut w = World::hunt_of([class; MAX_PLAYERS], species);
    let sp = species.get();
    // One body, held where it is: a flinch nothing will end before the move
    // does, and more health than any move takes.
    for c in w.critters.iter_mut() {
        *c = Critter::EMPTY;
    }
    let mut body = Critter::new(
        sp,
        kind,
        on_the_floor(&w, lane().add(V3::new(metres, Fx::ZERO, Fx::ZERO))),
        1 << 15,
    );
    body.state = is::FLINCH;
    body.timer = u16::MAX;
    body.health = i16::MAX;
    w.critters[0] = body;
    if let Some(pack) = w.pack.as_mut() {
        pack.grace = u16::MAX;
    }
    // The fighter at the origin looking down +X; the other well out of it.
    stand_at_origin(&mut w);
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(12));
    let idle = Input::default();
    for _ in 0..2 {
        w.advance([Input::looking_at(0, 0, 0), idle]);
    }
    let middle = w.critters[0].body(sp).middle();
    let pitch = crate::aim::look_onto_closely(w.players[0].pos, 0, w.players[0].aloft, middle);
    let look = Input::looking_at(0, 0, pitch);
    let before = w.critters[0].health;
    w.press(0, move_kind, look);
    let mut out = Trial::default();
    for _ in 0..240 {
        w.advance([look, idle]);
        let now = w.critters[0];
        if let Some(hb) = state::hitbox(&w.players[0]) {
            if now.alive() && now.body(sp).touched_by(&hb) {
                out.drawn = true;
            }
        }
        if now.health < before {
            out.hurt = true;
        } else if w.players[0].pos.x.raw() >= now.body(sp).foot.x.sub(now.body(sp).half_len).raw() {
            out.ran_through = true;
        }
        let busy = matches!(
            w.players[0].action,
            Action::Startup { .. }
                | Action::Active { .. }
                | Action::Recovery { .. }
                | Action::Channel { .. }
        );
        // Over, and nothing it left behind is still out there working.
        let lingering = w.effects.iter().any(Option::is_some)
            || w.bolts.iter().any(Option::is_some)
            || w.gusts.iter().any(Option::is_some);
        if out.hurt || (!busy && !lingering) {
            break;
        }
    }
    out
}

/// Every class's every move against one kind of critter.
pub fn table(species: SpeciesId, kind: u8) -> Vec<Row> {
    let mut rows = Vec::new();
    for class in crate::class::ALL_CLASSES {
        for k in 0..crate::moves::slots(class) {
            let m = crate::moves::get(class, k as u8);
            let mut trials = [Trial::default(); METRES.len()];
            for (t, metres) in trials.iter_mut().zip(METRES) {
                *t = trial(species, kind, class, k as u8, Fx::from_int(metres));
                t.fighter = fighter_trial(class, k as u8, Fx::from_int(metres));
            }
            rows.push(Row {
                class,
                kind: k as u8,
                name: m.name,
                aim: m.aim(),
                trials,
            });
        }
    }
    rows
}

/// **Can a class jump onto a mountable critter's back?** (The Hornback's
/// "every class can get onto a cow's back", §1.) Stands one critter of `kind`
/// still in its species' hunt, puts a fighter of `class` beside its flank,
/// and holds jump while walking at it; the answer is how high the back was
/// if the fighter ended up riding it, or `None`. The real jump, the real
/// landing (`state::meet_the_critters`): the same question a person asks
/// with the keyboard.
pub fn lands_on(class: Class, species: SpeciesId, kind: u8) -> Option<Fx> {
    // A person times the step: try every length of push toward it, held from
    // the takeoff, and say yes if any of them lands on its back.
    (0..40).find_map(|push| landed_with(class, species, kind, push))
}

/// One try at [`lands_on`]: jump from against its flank, holding the push
/// toward it for `push` frames from the takeoff.
fn landed_with(class: Class, species: SpeciesId, kind: u8, push: u32) -> Option<Fx> {
    let mut w = World::hunt_of([class; MAX_PLAYERS], species);
    w.players[1].health = 0;
    let sp = species.get();
    let slot = w.critters.iter().position(|c| c.kind == kind)?;
    if let Some(p) = w.pack.as_mut() {
        p.grace = u16::MAX;
    }
    // The body stood still at the middle, everything else away: the question
    // is geometry.
    let stand = |w: &mut World| {
        for (i, c) in w.critters.iter_mut().enumerate() {
            if i == slot {
                c.pos = V3::ZERO;
                c.vel = V3::ZERO;
                c.yaw = 0;
                c.state = is::PROWL;
                c.timer = 0;
            } else if c.present() {
                c.pos = V3::new(Fx::from_int(-20), Fx::ZERO, Fx::from_int(i as i32 * 3 - 15));
            }
        }
    };
    stand(&mut w);
    let body: Critter = w.critters[slot];
    let half_wid = body.body(sp).half_wid;
    w.players[0].pos = V3::new(
        Fx::ZERO,
        Fx::ZERO,
        half_wid.add(crate::tuning::body_radius()).neg(),
    );
    for f in 0..120u32 {
        stand(&mut w);
        let mut press = if f < 40 { Input::SPACE } else { 0 };
        if f < push {
            press |= Input::W;
        }
        w.advance([Input::aimed(press, Input::QUARTER_TURN), Input::default()]);
        if crate::critter::ridden(w.players[0].mount) == Some(slot) {
            return Some(body.body(sp).crown());
        }
        if w.players[0].grounded && f > 4 {
            return None;
        }
    }
    let _ = (state::hitbox, Action::Free);
    None
}
