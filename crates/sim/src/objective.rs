//! Objectives (bestiary P7): something besides the hunters that can lose.
//!
//! The Siegeshell walks to a town wall and the hunt is lost when the wall
//! falls; the Hornback's crossing escorts a cart that moves only while a hunter
//! is near it and is lost if the herd breaks it, won if it reaches the far
//! side. Both are **a defended thing with health and a place**: a box standing
//! on the floor, at a fixed point or somewhere along a route, that a
//! creature's attacks can hit.
//!
//! Split the way arenas and species already split a fight:
//!
//! - **Where it is** is the arena's: a [`Site`] -- a route of points (one
//!   point for a wall), and the box's size -- in `Arena::sites`, because the
//!   road and the wall are geometry.
//! - **What it is** is the species': an [`ObjectiveDecl`] in its
//!   `FightDecl`, naming the site, saying whether it moves while escorted,
//!   whether it is a solid, and whether losing it loses the hunt or reaching
//!   the end of its route wins it -- and a row of knobs ([`ObjectiveField`]:
//!   health, speed, escort distance, how much of a creature's blow it takes).
//! - **How it stands** is one cell of the hunt's [`Lore`]: health, how far
//!   along its route, flags, and which creatures' current moves have already
//!   struck it (a move hits it once).
//!
//! The world steps it (a cart rolls), creature attacks reach it through the
//! same volume they reach a fighter with (`Monster::reaches`), it is a solid
//! in [`crate::arena::Terrain`] when it says so, the renderer draws its box,
//! and the fight report counts what it took. `World::advance` ends the hunt on
//! [`lost`] and [`won`].

use crate::arena::{Arena, Material, Solid};
use crate::fixed::Fx;
use crate::lore::{Cell, Lore};
use crate::math::{self, V3};
use crate::oven::{Unit, species_raw};
use crate::species::Species;

/// The most defended things a fight can have standing at once.
pub const MAX_STANDING: usize = 4;

/// Where a defended thing stands, as the arena declares it.
#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub name: &'static str,
    /// Floor points (x, z) in centimetres, in order: where it starts, and the
    /// road it follows. One point for something that stays put.
    pub route: &'static [(i32, i32)],
    /// Half its width (x) and depth (z), and its whole height, in centimetres.
    pub size: (i32, i32, i32),
    /// What it is made of, for the renderer and for a solid's top.
    pub material: Material,
}

impl Site {
    /// The route's length.
    pub fn length(&self) -> Fx {
        let mut total = Fx::ZERO;
        for pair in self.route.windows(2) {
            total = total.add(math::wide_flat_dist(point(pair[0]), point(pair[1])));
        }
        total
    }

    /// The floor point `along` metres down the route, and which way the road
    /// runs there.
    pub fn at(&self, along: Fx) -> (V3, V3) {
        let Some(first) = self.route.first() else {
            return (V3::ZERO, V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
        };
        let mut left = along.max(Fx::ZERO);
        let mut dir = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
        for pair in self.route.windows(2) {
            let (a, b) = (point(pair[0]), point(pair[1]));
            let leg = math::wide_flat_dist(a, b);
            dir = math::wide_normalized(b.sub(a));
            if left.raw() <= leg.raw() {
                return (a.add(dir.scale(left)), dir);
            }
            left = left.sub(leg);
        }
        let last = self.route.last().copied().unwrap_or(*first);
        (point(last), dir)
    }

    /// Half its width, half its depth and its height, in metres.
    pub fn extent(&self) -> V3 {
        V3::new(cm(self.size.0), cm(self.size.2), cm(self.size.1))
    }
}

fn cm(v: i32) -> Fx {
    Fx::ratio(v, 100)
}

fn point(p: (i32, i32)) -> V3 {
    V3::new(cm(p.0), Fx::ZERO, cm(p.1))
}

/// What a defended thing is, as the species whose fight it is declares it.
#[derive(Clone, Copy, Debug)]
pub struct ObjectiveDecl {
    /// What the Oven and the report call it: "the wall", "the cart".
    pub name: &'static str,
    /// Which of the arena's sites it stands at.
    pub site: u8,
    /// It rolls along its route at `Speed` while a hunter is within `Escort`.
    pub escorted: bool,
    /// It is a solid: bodies stop at it, the aiming ray meets it.
    pub solid: bool,
    /// The hunt is lost when it breaks.
    pub loses: bool,
    /// The hunt is won when it reaches the end of its route.
    pub wins: bool,
}

impl ObjectiveDecl {
    /// Something that stands at a site, is solid, and loses the hunt when it
    /// breaks: a wall.
    pub const fn wall(name: &'static str, site: u8) -> ObjectiveDecl {
        ObjectiveDecl {
            name,
            site,
            escorted: false,
            solid: true,
            loses: true,
            wins: false,
        }
    }

    /// Something that rolls along its site's route while escorted, loses the
    /// hunt when it breaks and wins it when it arrives: a cart.
    pub const fn cart(name: &'static str, site: u8) -> ObjectiveDecl {
        ObjectiveDecl {
            name,
            site,
            escorted: true,
            solid: true,
            loses: true,
            wins: true,
        }
    }
}

/// One number about a defended thing: a row per objective, after the
/// species' hazard rows. "Siegeshell · the wall".
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObjectiveField {
    Health,
    /// How fast it rolls while escorted.
    Speed,
    /// How near a hunter has to be for it to roll.
    Escort,
    /// How much of a creature's blow it takes, times.
    Takes,
}

pub const OBJECTIVE_FIELDS: usize = 4;

impl ObjectiveField {
    pub const ALL: &'static [ObjectiveField] = &[
        ObjectiveField::Health,
        ObjectiveField::Speed,
        ObjectiveField::Escort,
        ObjectiveField::Takes,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            ObjectiveField::Health => "Health",
            ObjectiveField::Speed => "Speed, escorted",
            ObjectiveField::Escort => "Escorted within",
            ObjectiveField::Takes => "Takes of a creature's blow (x)",
        }
    }

    pub const fn unit(self) -> Unit {
        match self {
            ObjectiveField::Health => Unit::Int,
            _ => Unit::Fixed,
        }
    }

    pub const fn range(self) -> (i32, i32) {
        const fn fx(n: i32, d: i32) -> i32 {
            Fx::ratio(n, d).raw()
        }
        match self {
            ObjectiveField::Health => (1, 60000),
            ObjectiveField::Speed => (0, fx(12, 1)),
            ObjectiveField::Escort => (0, fx(30, 1)),
            ObjectiveField::Takes => (0, fx(10, 1)),
        }
    }
}

/// One of an objective's numbers, live from the Oven.
pub fn stat(sp: &Species, i: usize, field: ObjectiveField) -> i32 {
    species_raw(sp.id, sp.objective_index(i, field))
}

pub fn stat_fx(sp: &Species, i: usize, field: ObjectiveField) -> Fx {
    Fx::from_raw(stat(sp, i, field))
}

/// How it stands: one cell of the hunt's lore.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Objective {
    /// Present: it has been set up for this fight.
    pub present: bool,
    pub broken: bool,
    pub arrived: bool,
    /// Which creature slots' current moves have already struck it, a bit each;
    /// cleared for a slot when its creature is not mid-move.
    pub struck: u8,
    /// Centimetres along its route.
    pub along: u32,
    pub health: i32,
    /// Everything it has taken, for the report.
    pub taken: i32,
}

mod flag {
    pub const PRESENT: u8 = 1;
    pub const BROKEN: u8 = 2;
    pub const ARRIVED: u8 = 4;
}

impl Objective {
    fn to_cell(self) -> Cell {
        let flags = (self.present as u8) * flag::PRESENT
            | (self.broken as u8) * flag::BROKEN
            | (self.arrived as u8) * flag::ARRIVED;
        [
            u32::from_le_bytes([flags, self.struck, 0, 0]),
            self.along,
            self.health as u32,
            self.taken as u32,
        ]
    }

    fn from_cell(c: Cell) -> Objective {
        let [flags, struck, _, _] = c[0].to_le_bytes();
        Objective {
            present: flags & flag::PRESENT != 0,
            broken: flags & flag::BROKEN != 0,
            arrived: flags & flag::ARRIVED != 0,
            struck,
            along: c[1],
            health: c[2] as i32,
            taken: c[3] as i32,
        }
    }

    /// Metres along its route.
    pub fn along(&self) -> Fx {
        Fx::ratio(self.along.min(i32::MAX as u32 / 2) as i32, 100)
    }
}

/// Objective `i`.
pub fn get(lore: &Lore, i: usize) -> Objective {
    lore.objective_cells()
        .get(i)
        .map_or(Objective::default(), |c| Objective::from_cell(*c))
}

pub fn set(lore: &mut Lore, i: usize, o: Objective) {
    if let Some(c) = lore.objective_cells_mut().get_mut(i) {
        *c = o.to_cell();
    }
}

/// A defended thing as it stands this frame: its declaration, its site, where
/// it is, and its health. What the renderer and the report read.
#[derive(Clone, Copy, Debug)]
pub struct Standing {
    pub index: usize,
    pub decl: &'static ObjectiveDecl,
    pub site: &'static Site,
    /// The middle of its footprint, on the floor.
    pub at: V3,
    /// Which way its route runs here.
    pub dir: V3,
    pub state: Objective,
    pub full: i32,
}

impl Standing {
    /// Its box: an upright box on the floor, axis-aligned. A cart turning a
    /// corner turns its route, not its box -- the box is what a creature's
    /// blow and a body meet, and an axis-aligned one is what the arena's
    /// collision is written for.
    pub fn solid(&self) -> Solid {
        let e = self.site.extent();
        Solid {
            min: V3::new(self.at.x.sub(e.x), self.at.y, self.at.z.sub(e.z)),
            max: V3::new(self.at.x.add(e.x), self.at.y.add(e.y), self.at.z.add(e.z)),
            material: self.site.material,
        }
    }

    /// The radius a creature's blow measures it by: half the longer side.
    pub fn radius(&self) -> Fx {
        let e = self.site.extent();
        e.x.max(e.z)
    }
}

/// Every defended thing in the fight, as it stands.
pub fn standing<'a>(lore: &'a Lore, arena: &'static Arena) -> impl Iterator<Item = Standing> + 'a {
    let sp = lore.owner.map(|s| s.get());
    let decls: &'static [ObjectiveDecl] = sp.map_or(&[], |s| s.fight.objectives);
    decls.iter().enumerate().filter_map(move |(i, decl)| {
        let state = get(lore, i);
        let site = arena.sites.get(decl.site as usize)?;
        if !state.present {
            return None;
        }
        let (at, dir) = site.at(state.along());
        let full = sp.map_or(1, |s| stat(s, i, ObjectiveField::Health));
        Some(Standing {
            index: i,
            decl,
            site,
            at,
            dir,
            state,
            full,
        })
    })
}

/// **Set the fight's objectives up**: each at the start of its site, at full
/// health. An objective whose site the arena does not have is left out.
pub fn set_up(lore: &mut Lore, arena: &Arena) {
    let Some(sp) = lore.owner.map(|s| s.get()) else {
        return;
    };
    for (i, decl) in sp.fight.objectives.iter().enumerate() {
        if arena.sites.get(decl.site as usize).is_none() {
            continue;
        }
        set(
            lore,
            i,
            Objective {
                present: true,
                health: stat(sp, i, ObjectiveField::Health).max(1),
                ..Objective::default()
            },
        );
    }
}

/// One frame: an escorted thing rolls on while a hunter is near it.
pub fn step(lore: &mut Lore, arena: &'static Arena, hunters: &[V3]) {
    let Some(sp) = lore.owner.map(|s| s.get()) else {
        return;
    };
    for (i, decl) in sp.fight.objectives.iter().enumerate() {
        let mut o = get(lore, i);
        if !o.present || o.broken || o.arrived || !decl.escorted {
            continue;
        }
        let Some(site) = arena.sites.get(decl.site as usize) else {
            continue;
        };
        let (at, _) = site.at(o.along());
        let near = stat_fx(sp, i, ObjectiveField::Escort);
        if !hunters
            .iter()
            .any(|h| math::wide_flat_dist(*h, at).raw() <= near.raw())
        {
            continue;
        }
        let step = stat_fx(sp, i, ObjectiveField::Speed).mul(crate::DT);
        let cm = (step.raw() as i64 * 100) >> 16;
        o.along = o.along.saturating_add(cm.max(0) as u32);
        let end = site.length();
        if o.along().raw() >= end.raw() {
            o.along = (end.raw() as i64 * 100 >> 16).max(0) as u32;
            o.arrived = decl.wins;
        }
        set(lore, i, o);
    }
}

/// **A creature's blow lands on objective `i`**: so much damage, scaled by how
/// much of a creature's blow it takes. Returns what went in.
pub fn strike(lore: &mut Lore, i: usize, damage: i32) -> i32 {
    let Some(sp) = lore.owner.map(|s| s.get()) else {
        return 0;
    };
    let mut o = get(lore, i);
    if !o.present || o.broken {
        return 0;
    }
    let dealt = Fx::from_int(damage)
        .mul(stat_fx(sp, i, ObjectiveField::Takes))
        .to_int()
        .max(0);
    o.health = (o.health - dealt).max(0);
    o.taken = o.taken.saturating_add(dealt);
    o.broken = o.health <= 0;
    set(lore, i, o);
    dealt
}

/// Has something that loses the hunt broken?
pub fn lost(lore: &Lore) -> bool {
    let Some(sp) = lore.owner.map(|s| s.get()) else {
        return false;
    };
    sp.fight
        .objectives
        .iter()
        .enumerate()
        .any(|(i, d)| d.loses && get(lore, i).broken)
}

/// Has something that wins the hunt arrived?
pub fn won(lore: &Lore) -> bool {
    let Some(sp) = lore.owner.map(|s| s.get()) else {
        return false;
    };
    sp.fight
        .objectives
        .iter()
        .enumerate()
        .any(|(i, d)| d.wins && get(lore, i).arrived)
}
