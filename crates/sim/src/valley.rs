//! **The valley** (`docs/design/valley.md`): a world to walk through, made
//! of arenas.
//!
//! One long climb from Hearth's gate to the far end, loaded a **reach** at a
//! time, with the creatures' own arenas as rooms off it. Every place is an
//! arena, as it always was; what this module adds is how they join.
//!
//! - **A seam** is a box at the edge of a place -- a notch in a wall, a gate,
//!   the mouth of a cave -- that leads to a seam of another place. Seams come
//!   in pairs: this one's `to` and `at` name the seam you come out of, and that
//!   one's name this. Walking into a seam is a trip: a fresh world, in the
//!   place it leads to, with the journey carried across.
//! - **Both of you go together.** A seam fires on the frame every fighter
//!   still on their feet is in it, or once one has held it for
//!   `tuning::seam_hold` -- the second is how a pair says "come on" and how
//!   one player alone goes on at all. A fighter who arrives standing in a seam
//!   has to step out of it before it counts them, so arriving is never
//!   leaving.
//! - **A waystone** gates a seam that leads on: it is lit when enough of a
//!   tier's creatures have been beaten, counted from the journey, and an unlit
//!   one does not fire. Going back down is never gated.
//! - **A cairn** is a top that is a checkpoint: a fighter who touches one is
//!   rested there (health back), and one who dies anywhere in a reach is stood
//!   on the last cairn they touched, or where they came in. A fall costs
//!   what the fall rule says; it never ends anything.
//! - **Peace**: in the town and the reaches the two cannot hurt each other.
//!   The Ring is the one place for that, and plays as versus does.
//! - **A room** is a creature's own arena, and walking into it starts the
//!   hunt. Win it and the place goes quiet: walk out the way you came. Lose it
//!   and you wake outside, at the seam you went in by.
//!
//! The journey -- [`Journey`] -- is a few bytes of the snapshot: whether the
//! world is the valley at all, which creatures have been beaten (from fights
//! won on the way, and from each player's trophies, sent on the wire as
//! [`crate::input::Destination::Credit`] so both peers agree), which seam you
//! came in by, and each fighter's last cairn. Outside the valley it is all
//! zero and not hashed, so every fight that is not in it hashes as it did.

use crate::arena::{ArenaId, Mark, Solid};
use crate::fixed::Fx;
use crate::math::V3;
use crate::species::SpeciesId;
use crate::state::MAX_PLAYERS;

/// A box of the world a body's feet can be in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Zone {
    pub min: V3,
    pub max: V3,
}

impl Zone {
    /// From centimetres: two opposite corners.
    pub const fn cm(min: [i32; 3], max: [i32; 3]) -> Zone {
        let s = Solid::cm(min, max, crate::arena::Material::Ground);
        Zone {
            min: s.min,
            max: s.max,
        }
    }

    /// Are these feet in it?
    pub fn holds(&self, feet: V3) -> bool {
        feet.x.raw() >= self.min.x.raw()
            && feet.x.raw() <= self.max.x.raw()
            && feet.z.raw() >= self.min.z.raw()
            && feet.z.raw() <= self.max.z.raw()
            && feet.y.raw() >= self.min.y.raw()
            && feet.y.raw() <= self.max.y.raw()
    }

    /// The middle of its floor plane, at its bottom.
    pub fn middle(&self) -> V3 {
        V3::new(
            Fx::from_raw(self.min.x.raw() / 2 + self.max.x.raw() / 2),
            self.min.y,
            Fx::from_raw(self.min.z.raw() / 2 + self.max.z.raw() / 2),
        )
    }
}

/// What lets a seam fire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    Open,
    /// A waystone: lit once `need` of these creatures have been beaten.
    Waystone {
        need: u8,
        of: &'static [SpeciesId],
        /// What the HUD calls the tier.
        tier: &'static str,
    },
}

impl Gate {
    /// How many of its creatures have been beaten.
    pub fn count(&self, beaten: u32) -> u8 {
        match self {
            Gate::Open => 0,
            Gate::Waystone { of, .. } => of
                .iter()
                .filter(|s| beaten & (1 << s.0.min(31)) != 0)
                .count() as u8,
        }
    }

    pub fn lit(&self, beaten: u32) -> bool {
        match self {
            Gate::Open => true,
            Gate::Waystone { need, .. } => self.count(beaten) >= *need,
        }
    }
}

/// **The tiers** (`docs/design/world.md` §3), as the waystones read them.
pub mod tier {
    use super::Gate;
    use crate::species::SpeciesId as S;

    /// The Commons: the herd, the den. One of two opens the Shelves.
    pub const ONE: Gate = Gate::Waystone {
        need: 1,
        of: &[S::HORNBACK, S::GNAWERS],
        tier: "the Commons",
    };
    /// The lowlands: the Pan, the Mire. One of two opens the Pinewood.
    pub const TWO: Gate = Gate::Waystone {
        need: 1,
        of: &[S::SANDMAW, S::MIREBACK],
        tier: "the lowlands",
    };
    /// The heights and the woods. Two of three opens the Saddle.
    pub const THREE: Gate = Gate::Waystone {
        need: 2,
        of: &[S::RIDGEBACK, S::PAIR, S::BROODMOTHER],
        tier: "the heights and the woods",
    };
    /// The edges: the Cliffs, the Ashwood. One of two opens the Shrine.
    pub const FOUR: Gate = Gate::Waystone {
        need: 1,
        of: &[S::GALEWING, S::VEILSTALKER],
        tier: "the edges",
    };
    /// The Shrine's: the Mantis opens the Long Valley, where the
    /// Siegeshell walks.
    pub const FIVE: Gate = Gate::Waystone {
        need: 1,
        of: &[S::MANTIS],
        tier: "the Shrine",
    };
}

/// One way out of a place.
#[derive(Clone, Copy, Debug)]
pub struct Seam {
    /// Where your feet have to be.
    pub zone: Zone,
    /// The place it leads to...
    pub to: ArenaId,
    /// ...and which of that place's seams you come out of.
    pub at: u8,
    /// Where the two of you stand when you come *in* through this seam,
    /// facing into the place.
    pub marks: [Mark; MAX_PLAYERS],
    pub gate: Gate,
    /// The solid that is its waystone, if it has one: drawn lit or dark.
    pub waystone: Option<u8>,
    /// What the HUD says it leads to.
    pub says: &'static str,
}

/// A cylinder of rising air: anybody in the air inside it is carried up, to
/// its top. What courses.md §4 called an updraft vent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vent {
    pub at: (Fx, Fx),
    pub radius: Fx,
    /// The height it lets go at.
    pub top: Fx,
}

impl Vent {
    /// From centimetres.
    pub const fn cm(x: i32, z: i32, radius: i32, top: i32) -> Vent {
        Vent {
            at: (Fx::ratio(x, 100), Fx::ratio(z, 100)),
            radius: Fx::ratio(radius, 100),
            top: Fx::ratio(top, 100),
        }
    }

    /// Is a body at `p` in its column, below the top?
    pub fn carries(&self, p: V3) -> bool {
        let dx = p.x.sub(self.at.0);
        let dz = p.z.sub(self.at.1);
        if dx.abs().raw() > self.radius.raw() || dz.abs().raw() > self.radius.raw() {
            return false;
        }
        dx.mul(dx).add(dz.mul(dz)).raw() <= self.radius.mul(self.radius).raw()
            && p.y.raw() < self.top.raw()
    }
}

/// What kind of place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Hearth: nobody fights.
    Town,
    /// The one place in the valley where the two of you may fight each other:
    /// versus, round by round.
    Ring,
    /// A stretch of the climb: nobody fights, and a death is a cairn.
    Reach,
    /// A creature's arena: walking in starts its hunt.
    Room(SpeciesId),
}

/// One place of the valley.
#[derive(Debug)]
pub struct Place {
    pub arena: ArenaId,
    pub kind: Kind,
    pub seams: &'static [Seam],
    /// Climbable faces: a box in front of a face, its feet anywhere in which
    /// can climb with the jump held.
    pub vines: &'static [Zone],
    pub vents: &'static [Vent],
}

impl Place {
    pub fn seam(&self, i: u8) -> Option<&'static Seam> {
        self.seams.get(i as usize)
    }

    /// **Its cairns**: every solid whose top is snow, by index -- the same
    /// reading the jump courses give snow, where it is a checkpoint. Only a
    /// reach's or the town's count; a room's snow is a creature's floor.
    pub fn cairns(&self) -> impl Iterator<Item = (usize, &'static Solid)> {
        let counts = matches!(self.kind, Kind::Reach | Kind::Town);
        self.arena
            .get()
            .solids()
            .iter()
            .enumerate()
            .filter(move |(_, s)| counts && s.material == crate::arena::Material::Snow)
    }

    /// Is the world peaceful here: the town, a reach -- anywhere but the Ring
    /// and a room with its creature still in it.
    pub fn peaceful(&self) -> bool {
        !matches!(self.kind, Kind::Ring)
    }
}

/// Are these feet on this top: over it, and no higher above it than three
/// bodies -- a hop on the spot is still on a cairn.
pub fn on_top(s: &Solid, feet: V3) -> bool {
    let body = crate::tuning::body_height();
    s.over(feet.x, feet.z, Fx::ZERO)
        && feet.y.raw() >= s.max.y.sub(crate::arena::SKIN).raw()
        && feet.y.raw() <= s.max.y.add(body).add(body).add(body).raw()
}

/// Where the valley starts: Hearth's square.
pub const START: ArenaId = ArenaId::HEARTH;

/// The place an arena is, if it is one.
pub fn place(id: ArenaId) -> Option<&'static Place> {
    use crate::arena;
    Some(match id {
        ArenaId::HEARTH => &arena::hearth::PLACE,
        ArenaId::RING => &arena::ring::PLACE,
        ArenaId::MOUTH => &arena::mouth::PLACE,
        ArenaId::BANK => &arena::bank::PLACE,
        ArenaId::SHELVES => &arena::shelves::PLACE,
        ArenaId::PINEWOOD => &arena::pinewood::PLACE,
        ArenaId::SADDLE => &arena::saddle::PLACE,
        ArenaId::HIGHLANDS => &rooms::HIGHLANDS,
        ArenaId::HORNBACK => &rooms::MEADOW,
        ArenaId::GNAWERS => &rooms::COMMONS,
        ArenaId::MIREBACK => &rooms::MIRE,
        ArenaId::SANDMAW => &rooms::PAN,
        ArenaId::PAIR => &rooms::DEN,
        ArenaId::BROODMOTHER => &rooms::HOLLOWS,
        ArenaId::GALEWING => &rooms::CLIFFS,
        ArenaId::VEILSTALKER => &rooms::ASHWOOD,
        ArenaId::MANTIS => &rooms::SHRINE,
        ArenaId::SIEGESHELL => &rooms::LONG_VALLEY,
        _ => return None,
    })
}

/// Every place, in id order.
pub fn all() -> impl Iterator<Item = &'static Place> {
    (0..crate::arena::COUNT as u8).filter_map(|i| place(ArenaId(i)))
}

/// The vines and vents an arena has, for the movement that reads them --
/// whether or not the world is the valley, since they are part of the
/// ground.
pub fn vines(id: ArenaId) -> &'static [Zone] {
    place(id).map_or(&[], |p| p.vines)
}

pub fn vents(id: ArenaId) -> &'static [Vent] {
    place(id).map_or(&[], |p| p.vents)
}

/// The vine a body's feet are in, if any.
pub fn vine_at(id: ArenaId, feet: V3) -> Option<&'static Zone> {
    vines(id).iter().find(|v| v.holds(feet))
}

/// The vent carrying a body, if any.
pub fn vent_at(id: ArenaId, feet: V3) -> Option<&'static Vent> {
    vents(id).iter().find(|v| v.carries(feet))
}

/// "No seam" and "no cairn", in the journey's bytes.
pub const NONE: u8 = 0;

/// **The journey**: what the valley keeps across trips. Sixteen bytes of the
/// snapshot; all zero, and not hashed, outside the valley.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Journey {
    /// The world is the valley.
    pub on: bool,
    /// The creatures beaten, by species id: a bit each.
    pub beaten: u32,
    /// The seam of this place you came in by, plus one; [`NONE`] for the
    /// place's own start.
    pub came: u8,
    /// The seam somebody is holding, plus one, and for how long.
    pub held: u8,
    pub hold: u16,
    /// A bit per fighter: has stepped out of every seam since arriving, so a
    /// seam now counts them.
    pub armed: u8,
    /// Each fighter's last cairn, plus one.
    pub cairn: [u8; MAX_PLAYERS],
}

impl Journey {
    /// Has this creature been beaten on the way, or credited from a trophy?
    pub fn has_beaten(&self, s: SpeciesId) -> bool {
        self.beaten & (1 << s.0.min(31)) != 0
    }

    pub fn credit(&mut self, s: SpeciesId) {
        self.beaten |= 1 << s.0.min(31);
    }
}

/// The creatures' rooms: each one's arena as it is, with one seam out of it
/// where the hunters stand when it starts -- the way you came in.
pub mod rooms {
    use super::{Kind, Place, Seam, Zone};
    use crate::arena::{ArenaId, Mark};
    use crate::species::SpeciesId;

    const fn out(zone: Zone, to: ArenaId, at: u8, marks: [Mark; 2], says: &'static str) -> Seam {
        Seam {
            zone,
            to,
            at,
            marks,
            gate: super::Gate::Open,
            waystone: None,
            says,
        }
    }

    const fn room(arena: ArenaId, species: SpeciesId, seams: &'static [Seam]) -> Place {
        Place {
            arena,
            kind: Kind::Room(species),
            seams,
            vines: &[],
            vents: &[],
        }
    }

    pub static MEADOW: Place = room(
        ArenaId::HORNBACK,
        SpeciesId::HORNBACK,
        &[out(
            Zone::cm([-2390, -100, -400], [-1900, 600, 400]),
            ArenaId::MOUTH,
            1,
            [
                Mark::cm(-2000, -200, (0, -200)),
                Mark::cm(-2000, 200, (0, 200)),
            ],
            "the Mouth",
        )],
    );

    pub static COMMONS: Place = room(
        ArenaId::GNAWERS,
        SpeciesId::GNAWERS,
        &[out(
            Zone::cm([-400, -100, -1490], [400, 600, -1050]),
            ArenaId::BANK,
            1,
            [
                Mark::cm(-150, -1150, (-150, 0)),
                Mark::cm(150, -1150, (150, 0)),
            ],
            "the Bank",
        )],
    );

    pub static MIRE: Place = room(
        ArenaId::MIREBACK,
        SpeciesId::MIREBACK,
        &[out(
            Zone::cm([-1700, -100, -400], [-1100, 600, 400]),
            ArenaId::SHELVES,
            1,
            [
                Mark::cm(-1200, -200, (0, -200)),
                Mark::cm(-1200, 200, (0, 200)),
            ],
            "the Shelves",
        )],
    );

    pub static PAN: Place = room(
        ArenaId::SANDMAW,
        SpeciesId::SANDMAW,
        &[out(
            Zone::cm([-1750, -100, -400], [-1200, 600, 400]),
            ArenaId::SHELVES,
            2,
            [
                Mark::cm(-1300, -200, (0, -200)),
                Mark::cm(-1300, 200, (0, 200)),
            ],
            "the Shelves",
        )],
    );

    pub static DEN: Place = room(
        ArenaId::PAIR,
        SpeciesId::PAIR,
        &[out(
            Zone::cm([-1450, -100, -400], [-900, 600, 400]),
            ArenaId::PINEWOOD,
            1,
            [
                Mark::cm(-1000, -150, (0, -150)),
                Mark::cm(-1000, 150, (0, 150)),
            ],
            "the Pinewood",
        )],
    );

    pub static HOLLOWS: Place = room(
        ArenaId::BROODMOTHER,
        SpeciesId::BROODMOTHER,
        &[out(
            Zone::cm([-1450, -100, -400], [-1050, 600, 400]),
            ArenaId::PINEWOOD,
            2,
            [
                Mark::cm(-1150, -200, (0, -200)),
                Mark::cm(-1150, 200, (0, 200)),
            ],
            "the Pinewood",
        )],
    );

    /// The Ridgeback has no arena of its own (it is hunted in the proving
    /// ground); in the valley it is met on the Highlands, which are the
    /// proving ground's floor plan on a moor.
    pub static HIGHLANDS: Place = room(
        ArenaId::HIGHLANDS,
        SpeciesId::RIDGEBACK,
        &[out(
            Zone::cm([-1390, -100, -300], [-1100, 600, 300]),
            ArenaId::PINEWOOD,
            3,
            [
                Mark::cm(-1000, -200, (0, -200)),
                Mark::cm(-1000, 200, (0, 200)),
            ],
            "the Pinewood",
        )],
    );

    /// The Cliffs: the hunters start on the plateau, twelve metres up.
    pub static CLIFFS: Place = room(
        ArenaId::GALEWING,
        SpeciesId::GALEWING,
        &[out(
            Zone::cm([-400, 1100, -1950], [400, 1800, -1350]),
            ArenaId::SADDLE,
            1,
            [
                Mark::cm(-150, -1500, (0, 900)),
                Mark::cm(150, -1500, (0, 900)),
            ],
            "the Saddle",
        )],
    );

    pub static ASHWOOD: Place = room(
        ArenaId::VEILSTALKER,
        SpeciesId::VEILSTALKER,
        &[out(
            Zone::cm([-1640, -100, -400], [-1100, 600, 400]),
            ArenaId::SADDLE,
            2,
            [
                Mark::cm(-1200, -150, (0, -150)),
                Mark::cm(-1200, 150, (0, 150)),
            ],
            "the Saddle",
        )],
    );

    pub static SHRINE: Place = room(
        ArenaId::MANTIS,
        SpeciesId::MANTIS,
        &[out(
            Zone::cm([-1600, -100, -400], [-1000, 600, 400]),
            ArenaId::SADDLE,
            3,
            [
                Mark::cm(-1100, -150, (0, -150)),
                Mark::cm(-1100, 150, (0, 150)),
            ],
            "the Saddle",
        )],
    );

    /// The Long Valley runs from the far end of the climb back down to
    /// Hearth's wall: in at the west from the Saddle, out at the east through
    /// the wall's gate to Hearth. The Siegeshell walks it whichever end you
    /// come in by.
    pub static LONG_VALLEY: Place = room(
        ArenaId::SIEGESHELL,
        SpeciesId::SIEGESHELL,
        &[
            out(
                Zone::cm([13700, -100, -300], [14300, 600, 300]),
                ArenaId::HEARTH,
                2,
                [
                    Mark::cm(13400, -200, (0, -200)),
                    Mark::cm(13400, 200, (0, 200)),
                ],
                "Hearth",
            ),
            out(
                Zone::cm([-14990, -100, -400], [-14400, 600, 400]),
                ArenaId::SADDLE,
                4,
                [
                    Mark::cm(-14000, -200, (0, -200)),
                    Mark::cm(-14000, 200, (0, 200)),
                ],
                "the Saddle",
            ),
        ],
    );
}
