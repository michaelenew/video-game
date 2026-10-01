//! Noise (bestiary P5): the ring of loud things a creature that hears can
//! perceive.
//!
//! **The game has no sound**, so a noise is a drawn thing and a simulated one:
//! a place, a kind, who made it, how big (metres fallen, a shield's weight),
//! and when. Fighters and the world make them where they already do the loud
//! thing -- a footfall at a walk, a landing, a dodge starting and ending, a hit
//! landing, a Rush, a stone coming up, a shield planted, a quake -- and each
//! goes into a fixed ring in the hunt's [`Lore`], overwriting the oldest. A
//! fight whose layout has no noise cells makes none, so nothing here costs a
//! byte or a cycle in a fight that does not hear.
//!
//! **Where** a noise is comes from what the simulation already decided: the
//! end of the path `aim.rs` solved for a shot, the centre of a swing's own
//! volume, the stone's own position. Nothing here works out an aim.
//!
//! **How loud** is the hearing species' business: a row of loudness knobs per
//! kind ([`crate::species::FightField`]), read by [`loudness`]. A creature
//! hears a noise whose loudness, in metres, is more than its distance from the
//! creature's head -- see `monster::Heard` and `World::heard_by`.

use crate::arena::Material;
use crate::fixed::Fx;
use crate::lore::{self, Cell, Lore};
use crate::math::V3;
use crate::species::{FightField, Species};
use crate::state::{Action, MAX_PLAYERS, Player};

/// What made it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum NoiseKind {
    /// A step at a walk or faster. A crouch-walk makes none.
    Footfall = 1,
    /// A step on rock or stone, which carries further than one on sand.
    FootfallHard,
    /// Touching down; `amount` is the metres fallen.
    Landing,
    /// A dodge starting, or ending.
    Dodge,
    /// A blow landing, at the impact.
    Hit,
    /// The Champion's Rush, every stride of it.
    Rush,
    /// A stone coming up out of the floor.
    Stone,
    /// A shield planted; `amount` is its weight.
    Shield,
    /// A quake or a tremor going off.
    Quake,
}

impl NoiseKind {
    pub const ALL: [NoiseKind; 9] = [
        NoiseKind::Footfall,
        NoiseKind::FootfallHard,
        NoiseKind::Landing,
        NoiseKind::Dodge,
        NoiseKind::Hit,
        NoiseKind::Rush,
        NoiseKind::Stone,
        NoiseKind::Shield,
        NoiseKind::Quake,
    ];

    pub fn from_byte(b: u8) -> Option<NoiseKind> {
        NoiseKind::ALL.iter().copied().find(|k| *k as u8 == b)
    }

    pub const fn name(self) -> &'static str {
        match self {
            NoiseKind::Footfall => "footfall",
            NoiseKind::FootfallHard => "footfall on rock",
            NoiseKind::Landing => "landing",
            NoiseKind::Dodge => "dodge",
            NoiseKind::Hit => "hit",
            NoiseKind::Rush => "rush",
            NoiseKind::Stone => "stone",
            NoiseKind::Shield => "shield",
            NoiseKind::Quake => "quake",
        }
    }
}

/// Nobody: a noise the world made rather than a fighter.
pub const NOBODY: u8 = u8::MAX;

/// One noise: a cell of the ring.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Noise {
    pub kind: NoiseKind,
    /// The fighter who made it, or [`NOBODY`].
    pub who: u8,
    /// How big: metres fallen for a landing, a shield's weight; else zero.
    pub amount: u8,
    /// The frame it was made on.
    pub born: u32,
    /// Centimetres.
    pub at: [i16; 3],
}

impl Noise {
    pub fn pos(&self) -> V3 {
        lore::cm_point(self.at)
    }

    fn to_cell(self) -> Cell {
        [
            u32::from_le_bytes([self.kind as u8, self.who, self.amount, 0]),
            self.born,
            lore::halves(self.at[0], self.at[1]),
            lore::halves(self.at[2], 0),
        ]
    }

    fn from_cell(c: Cell) -> Option<Noise> {
        let [kind, who, amount, _] = c[0].to_le_bytes();
        Some(Noise {
            kind: NoiseKind::from_byte(kind)?,
            who,
            amount,
            born: c[1],
            at: [lore::lo(c[2]), lore::hi(c[2]), lore::lo(c[3])],
        })
    }
}

/// Every noise in the ring.
pub fn all(lore: &Lore) -> impl Iterator<Item = Noise> + '_ {
    lore.noise_cells()
        .iter()
        .filter_map(|c| Noise::from_cell(*c))
}

/// The noise in one cell of the ring, by the cell's index, if it holds one.
/// What a species that marks which noises it heard keeps its bits against.
pub fn nth(lore: &Lore, cell: usize) -> Option<Noise> {
    lore.noise_cells()
        .get(cell)
        .and_then(|c| Noise::from_cell(*c))
}

/// Does this fight hear at all?
pub fn heard_here(lore: &Lore) -> bool {
    !lore.noise_cells().is_empty()
}

/// **Make a noise**: into an empty cell, or over the oldest. Nothing in a fight
/// whose layout has no ring.
pub fn make(lore: &mut Lore, kind: NoiseKind, at: V3, who: u8, amount: u8, frame: u32) {
    let cells = lore.noise_cells_mut();
    if cells.is_empty() {
        return;
    }
    let slot = cells
        .iter()
        .position(|c| Noise::from_cell(*c).is_none())
        .unwrap_or_else(|| {
            // The oldest: the one longest ago, measured backwards from now so
            // the frame counter wrapping does not make the oldest look newest.
            (0..cells.len())
                .max_by_key(|i| {
                    let born = cells[*i][1];
                    (frame.wrapping_sub(born), usize::MAX - i)
                })
                .unwrap_or(0)
        });
    cells[slot] = Noise {
        kind,
        who,
        amount,
        born: frame,
        at: lore::point_cm(at),
    }
    .to_cell();
}

/// **How far a noise carries**, in metres, to the species listening: its row's
/// loudness for the kind, and for a landing so much more per metre fallen up to
/// its most. Zero for a species with no row.
pub fn loudness(sp: &Species, n: &Noise) -> Fx {
    if !sp.fight.row {
        return Fx::ZERO;
    }
    let f = |field| sp.fight_fx(field);
    match n.kind {
        NoiseKind::Footfall => f(FightField::Footfall),
        NoiseKind::FootfallHard => f(FightField::FootfallHard),
        NoiseKind::Landing => f(FightField::Landing)
            .add(f(FightField::LandingPerMetre).mul(Fx::from_int(n.amount as i32)))
            .min(f(FightField::LandingMost)),
        NoiseKind::Dodge => f(FightField::Dodge),
        NoiseKind::Hit => f(FightField::Hit),
        NoiseKind::Rush => f(FightField::Rush),
        NoiseKind::Stone => f(FightField::Stone),
        NoiseKind::Shield => f(FightField::Shield),
        NoiseKind::Quake => f(FightField::Quake),
    }
}

/// What a fighter did this frame that makes a noise, read off the difference
/// between their body before the frame and after it. Pushed into the ring.
///
/// Diffing rather than a call at each loud place is deliberate: the places a
/// fighter does something loud are spread across a five-thousand-line step,
/// and a hook in each is a hook somebody forgets. A transition is one rule.
pub(crate) fn listen(
    lore: &mut Lore,
    before: &[Player; MAX_PLAYERS],
    after: &[Player; MAX_PLAYERS],
    floor_of: impl Fn(V3) -> Material,
    frame: u32,
) {
    if !heard_here(lore) {
        return;
    }
    let field_was = crate::stones::gather(before);
    let field_now = crate::stones::gather(after);
    for i in 0..MAX_PLAYERS {
        let (was, now) = (&before[i], &after[i]);
        if now.health <= 0 && was.health <= 0 {
            continue;
        }
        let who = i as u8;
        // A step: the stride crossing a half-cycle, feet down, not crouched.
        // The renderer puts a foot down on the same half-cycles.
        let stepped = (was.stride ^ now.stride) & 0x8000 != 0;
        if stepped && now.grounded && !now.crouching && !now.aboard() {
            let hard = matches!(floor_of(now.pos), Material::Rock | Material::Stone);
            let kind = if hard {
                NoiseKind::FootfallHard
            } else {
                NoiseKind::Footfall
            };
            make(lore, kind, now.pos, who, 0, frame);
        }
        // Touching down.
        if !was.grounded && now.grounded {
            let fell = fallen_by_speed(was.vel.y);
            make(lore, NoiseKind::Landing, now.pos, who, fell, frame);
        }
        // A dodge, at both ends.
        let dodging = |p: &Player| matches!(p.action, Action::Dodge { .. });
        if dodging(was) != dodging(now) {
            make(lore, NoiseKind::Dodge, now.pos, who, 0, frame);
        }
        // A blow landing: at the swing's own volume, or where the shot's path
        // ends.
        if !was.hit_used && now.hit_used {
            let at = crate::state::hitbox(now).map_or(now.aim_path.to, |b| b.centre());
            make(lore, NoiseKind::Hit, at, who, 0, frame);
        }
        // The Rush, once a stride.
        if crate::state::rushing(now).is_some() && stepped {
            make(lore, NoiseKind::Rush, now.pos, who, 0, frame);
        }
    }
    // Stones up, and shields down: a slot of the field filled this frame.
    for (slot, (w, n)) in field_was.iter().zip(field_now.iter()).enumerate() {
        let (None, Some(stone)) = (w, n) else {
            continue;
        };
        let owner = slot / crate::class::MAX_STRUCTURES;
        let who = owner as u8;
        match after.get(owner).map(|p| p.mechanic) {
            Some(crate::class::Mechanic::Shield(s)) => {
                let weight = s.weight().to_int().clamp(0, u8::MAX as i32) as u8;
                make(lore, NoiseKind::Shield, stone.at, who, weight, frame);
            }
            _ => make(lore, NoiseKind::Stone, stone.at, who, 0, frame),
        }
    }
}

/// Metres a body landing at this vertical speed has fallen, as far as speed
/// can tell: `v² / 2g`, which saturates at terminal velocity -- about four
/// metres, which is past the most a landing's loudness grows for.
fn fallen_by_speed(vy: Fx) -> u8 {
    let v = vy.neg().max(Fx::ZERO);
    let g = crate::tuning::gravity().abs().max(Fx::ONE);
    let metres = v.mul(v).div(g.add(g));
    metres.to_int().clamp(0, u8::MAX as i32) as u8
}
