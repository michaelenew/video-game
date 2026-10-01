//! Species: what one kind of creature *is*, as data.
//!
//! The machinery that makes a creature stand, walk, collide, get hit, carry a
//! rider and decide what to do is shared -- [`crate::beast`] for the skeleton,
//! [`crate::monster`] for the body, the moves and the mind. What differs from
//! one creature to the next is a table, and this module is where the tables
//! live: one file (or folder) per species, each exporting one [`Species`].
//!
//! A [`Species`] is:
//!
//! - **a skeleton** -- bones, their parents, their rest offsets, which side
//!   each is on, and the pairs a mirrored clip swaps;
//! - **parts** -- one box per part on a bone, with flags that say what the box
//!   is: mountable, solid, breakable, a weak point, and which knob holds its
//!   damage multiplier;
//! - **legs** -- any number, none included;
//! - **moves** -- a name, a clip, and the flags that used to be special cases
//!   by name ([`MoveDecl`]); the frame data and hit volumes are knobs;
//! - **clips** -- its own baked table, written by `bake_beast`;
//! - **knobs** -- the numbers every creature has ([`Common`]), then its own,
//!   then its move table, all baked to its own `tuned.rs`.
//!
//! `Monster` carries a [`SpeciesId`] in the snapshot and reads everything else
//! from here, so a second species is a new file rather than a change to the
//! machinery. The recipe for adding one is in `docs/design/species.md`.
//!
//! ## Why the registry has a line for every creature already
//!
//! Ten creatures are planned after the Ridgeback (`docs/design/bestiary.md`),
//! and they will be built two at a time on separate branches. Every one of
//! them has an id and a commented-out line in each of the few places a species
//! is registered, **one blank line apart**. Building a creature means
//! uncommenting its own lines, and two branches doing that for two different
//! creatures touch lines git sees as unrelated -- so they merge without
//! conflicting.

pub mod common;

pub use common::Common;

use crate::beast::{Bone, ClipDecl, Leg, MAX_BREAKABLE, MAX_PARTS, Part, Shape};
use crate::fixed::Fx;
use crate::math::V3;
use crate::oven::KnobDecl;

// ---------------------------------------------------------------------------
// The registry
// ---------------------------------------------------------------------------

// One line per species, each followed by a blank line. See the module docs for
// why every planned creature already has one.

pub mod ridgeback;

/// A dev pack, not a creature anybody fights for a trophy: the smallest pack
/// that exercises every piece of the critter machinery, for its tests and for
/// `critcheck`. It is to packs what the range is to arenas.
pub mod gnats;

pub mod sentinel;

pub mod gnawers;

pub mod hornback;

pub mod mireback;

pub mod sandmaw;

pub mod pair;

// pub mod broodmother;

pub mod veilstalker;

// pub mod mantis;

// pub mod galewing;

// pub mod siegeshell;

/// Which kind of creature. The one byte of species a `Monster` keeps in the
/// snapshot; everything else is looked up.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct SpeciesId(pub u8);

impl SpeciesId {
    pub const RIDGEBACK: SpeciesId = SpeciesId(0);
    pub const GNAWERS: SpeciesId = SpeciesId(1);
    pub const HORNBACK: SpeciesId = SpeciesId(2);
    pub const MIREBACK: SpeciesId = SpeciesId(3);
    pub const SANDMAW: SpeciesId = SpeciesId(4);
    pub const PAIR: SpeciesId = SpeciesId(5);
    pub const BROODMOTHER: SpeciesId = SpeciesId(6);
    pub const VEILSTALKER: SpeciesId = SpeciesId(7);
    pub const MANTIS: SpeciesId = SpeciesId(8);
    pub const GALEWING: SpeciesId = SpeciesId(9);
    pub const SIEGESHELL: SpeciesId = SpeciesId(10);
    /// The dev pack: see [`gnats`].
    pub const GNATS: SpeciesId = SpeciesId(11);
    /// The dev creature for the senses, the floor and the defended things:
    /// see [`sentinel`].
    pub const SENTINEL: SpeciesId = SpeciesId(12);

    /// The table. Every registered id has one; asking for an unregistered one
    /// is a bug in whoever built the monster, and gets the Ridgeback rather
    /// than a crash in the middle of a rollback.
    pub fn get(self) -> &'static Species {
        lookup(self).unwrap_or(&ridgeback::SPECIES)
    }
}

/// How many ids there are, registered or not.
pub const COUNT: usize = 13;

/// The species registered under an id, if one is.
pub const fn lookup(id: SpeciesId) -> Option<&'static Species> {
    match id {
        SpeciesId::RIDGEBACK => Some(&ridgeback::SPECIES),

        SpeciesId::GNATS => Some(&gnats::SPECIES),

        SpeciesId::SENTINEL => Some(&sentinel::SPECIES),

        SpeciesId::GNAWERS => Some(&gnawers::SPECIES),

        SpeciesId::HORNBACK => Some(&hornback::SPECIES),
        SpeciesId::MIREBACK => Some(&mireback::SPECIES),

        SpeciesId::SANDMAW => Some(&sandmaw::SPECIES),

        SpeciesId::PAIR => Some(&pair::SPECIES),

        // SpeciesId::BROODMOTHER => Some(&broodmother::SPECIES),

        SpeciesId::VEILSTALKER => Some(&veilstalker::SPECIES),

        // SpeciesId::MANTIS => Some(&mantis::SPECIES),

        // SpeciesId::GALEWING => Some(&galewing::SPECIES),

        // SpeciesId::SIEGESHELL => Some(&siegeshell::SPECIES),
        _ => None,
    }
}

/// Every registered species, in id order.
pub fn all() -> impl Iterator<Item = &'static Species> {
    (0..COUNT as u8).filter_map(|i| lookup(SpeciesId(i)))
}

/// Find a registered species by name, ignoring case. For the tools that take
/// `--species`.
pub fn named(name: &str) -> Option<&'static Species> {
    all().find(|s| s.name.eq_ignore_ascii_case(name) || s.slug() == name.to_lowercase())
}

/// The next registered species after this one, in id order, wrapping round:
/// what the picker's cycle steps to. Unregistered ids are skipped, so a
/// creature whose branch has not landed is never offered.
pub fn after(id: SpeciesId) -> &'static Species {
    (1..=COUNT as u8)
        .filter_map(|step| lookup(SpeciesId((id.0.wrapping_add(step)) % COUNT as u8)))
        .next()
        .unwrap_or(&ridgeback::SPECIES)
}

// ---------------------------------------------------------------------------
// The table
// ---------------------------------------------------------------------------

/// The most moves a species has. The brain keeps a lockout per move in the
/// snapshot, so this is bytes.
pub const MAX_MOVES: usize = 16;

/// One of a species' moves, as far as the code is concerned.
///
/// The move's *numbers* -- frames, damage, hit volume, what the brain wants --
/// are knobs (`oven::MonsterField`), tuned per species. What is here is the
/// shape of the move, and the **flags that used to be special cases by name**:
/// declared in the table, the way `Move::aim()` is declared for a fighter's
/// moves rather than inferred from what they leave behind.
#[derive(Clone, Copy, Debug)]
pub struct MoveDecl {
    pub name: &'static str,
    /// The clip it plays, as an index into the species' clips.
    pub clip: usize,
    /// **Played toward the side its target is on.** The clip is baked going
    /// one way; when the move commits, the creature looks at which side its
    /// target is on and plays it mirrored if that is the other one. Decided
    /// once and held for the move, because a swing that changed its mind
    /// mid-whip would be a telegraph that lied. The Ridgeback's tail sweep.
    pub mirrors_to_target_side: bool,
    /// **Its violence is a knob.** The species' own knob, by index, that the
    /// baked clip is scaled by -- the one kind of animation number that is
    /// gameplay rather than content, because it decides whether a braced
    /// rider stays on. The Ridgeback's shake, by its shake force.
    pub scaled_by: Option<u16>,
    /// **Lobbed at a point**: its volume lands on the floor at the creature's
    /// aim point (`monster::Brain::aim`), chosen when it commits and held -- the
    /// lead point, kept inside the move's range, unless the species' `commit`
    /// hook says otherwise. Its `HitX`/`HitZ` are not read. The Mireback's glob
    /// and its belly flop: the telegraph is drawn where it will land from the
    /// first frame of the tell, because that is where it will land.
    pub lobbed: bool,
    /// **Its travel stops at the aim point**: a volume that `travels` goes no
    /// further along the facing than the creature's aim point, which the species
    /// keeps on the first solid in the way. The Mireback's tongue, which slag
    /// or a stone or a planted shield stops.
    pub stops_at_aim: bool,
    /// **It lands for no damage.** A move whose zero damage would otherwise
    /// mean it has no volume at all: it still reaches, is still drawn, and its
    /// landing is still handed to the species (`FightDecl::landed`). The
    /// Mireback's Backwash, which tars rather than hurts.
    pub harmless: bool,
    /// **The brain never picks it**: a state the species puts the creature in
    /// from its own hooks, played and timed as a move -- the Mireback's
    /// swallow, its gag. It scores nothing.
    pub never_chosen: bool,
    /// **Its species tests its hit itself**: the move has no cylinder, so
    /// [`crate::monster::Monster::hit_volume`] and the telegraph have nothing
    /// for it, and the species' frame hook decides who it reaches and draws
    /// the same shape through its `marks`. A shape that is not a cylinder --
    /// the Sandmaw's cone of spray, its tail's half-ring -- or a reach a solid
    /// can stop, which only the world can ask about.
    pub own_hit: bool,
}

impl MoveDecl {
    pub const fn new(name: &'static str, clip: usize) -> MoveDecl {
        MoveDecl {
            name,
            clip,
            mirrors_to_target_side: false,
            scaled_by: None,
            lobbed: false,
            stops_at_aim: false,
            harmless: false,
            never_chosen: false,
            own_hit: false,
        }
    }

    pub const fn lobbed(mut self) -> MoveDecl {
        self.lobbed = true;
        self
    }

    pub const fn stops_at_aim(mut self) -> MoveDecl {
        self.stops_at_aim = true;
        self
    }

    pub const fn harmless(mut self) -> MoveDecl {
        self.harmless = true;
        self
    }

    pub const fn never_chosen(mut self) -> MoveDecl {
        self.never_chosen = true;
        self
    }

    pub const fn own_hit(mut self) -> MoveDecl {
        self.own_hit = true;
        self
    }

    pub const fn mirrors_to_target_side(mut self) -> MoveDecl {
        self.mirrors_to_target_side = true;
        self
    }

    pub const fn scaled_by(mut self, knob: u16) -> MoveDecl {
        self.scaled_by = Some(knob);
        self
    }
}

/// The clips every creature plays outside its moves, by index into its clip
/// table. The same clip may stand for two of them.
#[derive(Clone, Copy, Debug)]
pub struct Stock {
    /// Standing. Loops, and breathes.
    pub idle: usize,
    /// One full cycle of the walk. Indexed by **ground covered** rather than
    /// by time -- a cycle on a fixed cadence skates the moment the body moves
    /// at any other speed.
    pub walk: usize,
    /// The fast gait, blended from the walk above the walking speed.
    pub gallop: usize,
    pub flinch: usize,
    /// Down on a knee: what a broken leg and a landed piece of crowd control
    /// both produce.
    pub stumble: usize,
    pub topple: usize,
    pub dead: usize,
}

/// What a species does with the press of a fighter inside it: the world, the
/// creature's slot, the fighter and what they sent; what they may still do.
pub type FromInside = fn(&mut crate::state::World, usize, usize, crate::Input) -> crate::Input;

/// What a species brings to a fight besides its body and its pack: the
/// shared machinery of bestiary P4, P5 and P7, and the hooks a creature's own
/// file plugs into. Every field defaults to nothing ([`FightDecl::PLAIN`]), and
/// a species that says nothing is fought exactly as the Ridgeback always was.
///
/// See `docs/design/hazards.md` for the recipe.
pub struct FightDecl {
    /// How it lays out the hunt's lore: hazard cells, noise cells, objective
    /// cells, and its own. See [`crate::lore`].
    pub layout: crate::lore::Layout,
    /// Whether it has the **senses and body row** of knobs ([`FightField`]):
    /// its sight cone and blind arc, what it feels, how loud each noise is to
    /// it, and how big it is against the arena's solids.
    pub row: bool,
    /// Its hazard kinds, in the order the cells' `kind` byte counts them.
    pub hazards: &'static [crate::hazard::HazardDecl],
    /// The things in its fight that can lose: a wall, a cart.
    pub objectives: &'static [crate::objective::ObjectiveDecl],
    /// **Its perception filter**: does it perceive this fighter now? See
    /// [`crate::perception`]; `sees_all` is the Ridgeback's.
    pub perceives: crate::perception::Perceive,
    /// It hears the noise ring: when its glance perceives no body, it samples
    /// the loudest noise that reached its head since its last glance.
    pub hears: bool,
    /// It collides with the arena's solids (its row's `BodyRadius`, stepping
    /// over anything under `StepOver`). Off, it is only kept inside the
    /// bounds -- which is what the Ridgeback has always had, and what keeps it
    /// bit-identical.
    pub collides: bool,
    /// **Its parts come down on bodies** -- a belly flop -- so a body standing
    /// on the floor under a part is shoved out sideways rather than into the
    /// ground. Off, the collision is least penetration as it always was,
    /// which keeps the Ridgeback bit-identical.
    pub lands_on_bodies: bool,
    /// **It rolls onto its back**, so a part's top face can point at the
    /// floor -- and a face pointing at the floor is nobody's to stand on.
    /// Off, every top face is a surface whichever way it points, as it always
    /// was: the Ridgeback's shake turns its shoulders far enough that the
    /// rule would cost a braced rider their footing.
    pub rolls_over: bool,
    /// **The steepest face that is still somewhere to stand**: the species'
    /// own knob, by index, holding the cosine of that slope. A part whose top
    /// face tilts further from level is a wall, not a floor -- the Sandmaw's
    /// column, six metres of worm leaning out of the sand. `None`, every top
    /// face is a surface whichever way it tilts, as it always was.
    pub steepest: Option<u16>,
    /// Called when it walks into a solid, with the push that got it out: the
    /// Hornback's charge into a rock is a stun.
    pub bumped: Option<fn(&mut crate::monster::Monster, crate::math::V3)>,
    /// Called once a frame, after the creatures, the pack and the hazards have
    /// stepped and before the fighters do: the species' own rules over its
    /// lore -- where tar spreads, when a vent blows, what a strand trips.
    pub frame: Option<fn(&mut crate::state::World)>,
    /// How much of a creature part can be seen, nought to one: what the
    /// renderer draws it at, and what the report and the scripted hunter call
    /// visible. `None` is always fully. The Veilstalker's veil.
    pub shown: Option<fn(&crate::state::World, usize, usize) -> Fx>,
    /// **What it draws on the floor besides its bodies' telegraphs**: a
    /// stampede's lane and its lees, the solid a charge will stop at, a
    /// guard. See [`crate::sign`]. `None` draws nothing more.
    pub signs: Option<fn(&crate::state::World, &mut crate::sign::Signs)>,

    // ---- the brain's seams: see `monster::Mind` ----
    /// **Its own terms in the scoring**, after the shared ones: handed a move,
    /// the score the shared brain gave it, and what the brain may read, and
    /// returns the score. The Mireback's floor, kindle, coat, crowd and flee.
    pub appetite: Option<AppetiteFn>,
    /// **Where it walks when it is free**, if not at its target: the Mireback
    /// walks to the middle of its own tar. `None` from the hook is "at the
    /// target", as every other creature does.
    pub prowl_to: Option<fn(&crate::monster::Monster, &crate::monster::Mind) -> Option<V3>>,
    /// Called on the frame a move commits, after the shared brain has set it
    /// up: where a lobbed move lands (`monster::Brain::aim`), which way it
    /// leaps.
    pub commit: Option<fn(&mut crate::monster::Monster, u8, &crate::monster::Mind)>,

    // ---- the body's seams: state in `Monster::own` ----
    /// **What its hide is worth on a part this frame**, times the part's own
    /// multiplier: the Mireback's tar coat, the throat sac shown or hidden.
    pub hide: Option<fn(&crate::monster::Monster, usize) -> Fx>,
    /// **A hit has landed**, for this much, after health and strain and before
    /// the shared ladder (breaks, topple, interrupt, flinch). Returning true
    /// says the species has decided what the hit did and the ladder is
    /// skipped: the Mireback's sac tearing, its wallow broken, the warts.
    pub struck: Option<fn(&mut crate::monster::Monster, usize, i32) -> bool>,

    // ---- the fight's seams ----
    /// **One of its moves has landed on a fighter** (creature slot, fighter,
    /// move, whether it was guarded), after the hit itself is dealt: what the
    /// move does besides hurt. The tongue's grab, the Backwash's tar.
    pub landed: Option<LandedFn>,
    /// **What it draws besides its hazards and its telegraph**: rings on the
    /// floor for what a move will leave or light, and the things it owns in
    /// the arena. Read by the renderer and the overlay, from the snapshot, so
    /// what is drawn is what the fight will do. See [`Mark`].
    pub marks: Option<fn(&crate::state::World, &mut Marks)>,

    // ---- a body that is not always there (the Sandmaw) ----
    /// **Which parts have no body this frame** -- no hurtbox, not solid, not
    /// mountable, not drawn -- and **which nobody can stand on** though they
    /// are there: a bit per part each. Handed the creature and its rig as
    /// built; `None` is every part always there. The Sandmaw under the sand,
    /// and only ridden when it is beached.
    pub presence:
        Option<fn(&crate::monster::Monster, &crate::beast::Rig) -> crate::beast::Presence>,
    /// **What it looks like, when its posture says so**: a pose (sampled from
    /// its own clips, `beast::sample`) in place of the shared choice between
    /// idle, walk and gallop and the stock states -- `None` from the hook is
    /// the shared choice. A worm swimming slowly under the sand is still
    /// under it.
    pub clip: Option<fn(&crate::monster::Monster) -> Option<crate::beast::Pose>>,
    /// **How far it hears this frame**, times its row's loudness: hunger, or
    /// deafness. `None` is one.
    pub hearing: Option<fn(&crate::monster::Monster, &crate::lore::Lore) -> Fx>,
    /// **A fighter inside one of its hollow parts pressed something**: the
    /// world, the creature's slot, the fighter and what they sent, and it
    /// returns what they may still do. The Sandmaw's swallow reads the escape
    /// off it and lets them do nothing else.
    pub from_inside: Option<FromInside>,
    /// **A move's radius this frame**, from the radius its knobs give: a
    /// consequence that changes the size of a move for the rest of the
    /// fight -- the Sandmaw's broken tooth ring, which shrinks its rise-bite.
    /// Read by the hit volume, so the telegraph and the hit change together.
    pub radius: Option<fn(&crate::monster::Monster, u8, Fx) -> Fx>,

    // ---- two bodies (the Pair) ----
    /// **How many of it a hunt holds**: one for every creature but the Pair,
    /// who are two. `World::hunt_of` puts this many in the slots, so the
    /// picker, `?hunt=` and the harness all get the whole fight.
    pub bodies: u8,
    /// **Its height is its own**: the walk leaves `Monster::pos.y` where the
    /// species' hooks put it rather than on the floor, and the fence reads
    /// it -- a cat standing on a wall top. Off, it is on the floor, as every
    /// creature always was.
    pub keeps_height: bool,
    /// **Its pace this frame**, times its speed: the Pair's survivor,
    /// enraged, runs past its gallop. `None` is one.
    pub pace: Option<fn(&crate::monster::Monster) -> Fx>,
    /// **Frames between its glances**, from its own state, before its
    /// temper: the Pair glance quicker with two hunters. `None` is its
    /// knob.
    pub glance: Option<fn(&crate::monster::Monster) -> u16>,
    /// **How high a lobbed move lands**: the top its aim point is on -- a cat
    /// pouncing onto a platform lands on the platform, and its circle is
    /// drawn there. `None` is the floor.
    pub lob_height: Option<fn(&crate::monster::Monster) -> Fx>,
}

/// Something a species draws beyond its hazards and its telegraph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mark {
    /// The middle of it, on the floor or wherever it stands.
    pub at: V3,
    pub radius: Fx,
    /// Zero: a ring on the floor. Above zero: a column this tall -- a thing
    /// standing in the arena.
    pub height: Fx,
    pub look: MarkLook,
    /// How far through what it warns of, nought to one: a ring that closes,
    /// a glow that brightens.
    pub progress: Fx,
}

/// How a [`Mark`] is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkLook {
    /// **Something lands here**: a telegraph ring, in the telegraph's colour.
    Warning,
    /// **This will catch**: a hazard that is about to ignite, glowing from
    /// inside.
    Kindling,
    /// Fire standing: a brazier's flame.
    Flame,
    /// A dark thing standing: iron, a plinth's load.
    Iron,
    /// Embers: a thing that is coming back.
    Embers,
    /// **Raised sand**: a mound over something moving under the floor, the
    /// height of the column it is drawn as. The Sandmaw's wake.
    Sand,
    /// A fin cutting through the sand: a dark blade standing over the head.
    Fin,
    /// **A noise it heard**: a ring in the sand where it was made, fading as
    /// `progress` runs to one. Only noises the creature heard are drawn.
    Heard,
    /// **What it can feel**: a faint disc on the floor, the radius the
    /// simulation feels a body within.
    Feel,
}

/// [`FightDecl::appetite`]: the creature, a move, the shared brain's score
/// for it, and what the brain may read.
pub type AppetiteFn = fn(&crate::monster::Monster, u8, i32, &crate::monster::Mind) -> i32;

/// [`FightDecl::landed`]: the world, the creature's slot, the fighter, the
/// move, and whether it was guarded.
pub type LandedFn = fn(&mut crate::state::World, usize, usize, u8, bool);

/// The most marks a species draws at once.
pub const MAX_MARKS: usize = 32;

/// A fixed list of marks, filled by a species' `marks` hook.
#[derive(Clone, Copy, Debug)]
pub struct Marks {
    pub items: [Option<Mark>; MAX_MARKS],
}

impl Marks {
    pub const NONE: Marks = Marks {
        items: [None; MAX_MARKS],
    };

    /// Add one; past the end it is dropped.
    pub fn push(&mut self, m: Mark) {
        if let Some(slot) = self.items.iter_mut().find(|s| s.is_none()) {
            *slot = Some(m);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Mark> {
        self.items.iter().flatten()
    }
}

impl std::fmt::Debug for FightDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FightDecl")
            .field("layout", &self.layout)
            .field("row", &self.row)
            .field("hazards", &self.hazards.len())
            .field("objectives", &self.objectives.len())
            .field("hears", &self.hears)
            .field("collides", &self.collides)
            .finish()
    }
}

impl FightDecl {
    /// Nothing: no lore, no hazards, no objectives, sees everybody, collides
    /// with nothing but the bounds, no hooks.
    pub const PLAIN: FightDecl = FightDecl {
        layout: crate::lore::Layout::NONE,
        row: false,
        hazards: &[],
        objectives: &[],
        perceives: crate::perception::sees_all,
        hears: false,
        collides: false,
        lands_on_bodies: false,
        rolls_over: false,
        steepest: None,
        bumped: None,
        frame: None,
        shown: None,
        signs: None,
        appetite: None,
        prowl_to: None,
        commit: None,
        hide: None,
        struck: None,
        landed: None,
        marks: None,
        presence: None,
        clip: None,
        hearing: None,
        from_inside: None,
        radius: None,
        bodies: 1,
        keeps_height: false,
        pace: None,
        glance: None,
        lob_height: None,
    };
}

/// One field of a species' **senses and body row**, for a species whose
/// `FightDecl::row` is set. "<Species> · senses".
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FightField {
    /// How wide it is against the arena's solids, when it collides with them.
    BodyRadius,
    /// Solids lower than this it walks over.
    StepOver,
    /// Half the angle it sees across, in turns: a half or more is all round.
    SightCone,
    /// Half the width of a blind arc, in turns, centred square off one side.
    BlindArc,
    /// How far from its head it feels a body on the floor.
    FeelRadius,
    /// How far off the floor a body can be and still be felt.
    FeelHeight,
    /// How far each noise carries to it, in metres.
    Footfall,
    FootfallHard,
    Landing,
    LandingPerMetre,
    LandingMost,
    Dodge,
    Hit,
    Rush,
    Stone,
    Shield,
    Quake,
}

pub const FIGHT_FIELDS: usize = 17;

impl FightField {
    pub const ALL: &'static [FightField] = &[
        FightField::BodyRadius,
        FightField::StepOver,
        FightField::SightCone,
        FightField::BlindArc,
        FightField::FeelRadius,
        FightField::FeelHeight,
        FightField::Footfall,
        FightField::FootfallHard,
        FightField::Landing,
        FightField::LandingPerMetre,
        FightField::LandingMost,
        FightField::Dodge,
        FightField::Hit,
        FightField::Rush,
        FightField::Stone,
        FightField::Shield,
        FightField::Quake,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            FightField::BodyRadius => "Body radius, against solids",
            FightField::StepOver => "Steps over solids under",
            FightField::SightCone => "Sight, half-angle (turns)",
            FightField::BlindArc => "Blind arc, half-width (turns)",
            FightField::FeelRadius => "Feels a body within",
            FightField::FeelHeight => "Feels a body no higher than",
            FightField::Footfall => "Hears a footfall at",
            FightField::FootfallHard => "Hears a footfall on rock at",
            FightField::Landing => "Hears a landing at",
            FightField::LandingPerMetre => "Landing, more per metre fallen",
            FightField::LandingMost => "Landing, at most",
            FightField::Dodge => "Hears a dodge at",
            FightField::Hit => "Hears a hit at",
            FightField::Rush => "Hears a Rush at",
            FightField::Stone => "Hears a stone at",
            FightField::Shield => "Hears a shield planted at",
            FightField::Quake => "Hears a quake at",
        }
    }

    pub const fn unit(self) -> crate::oven::Unit {
        crate::oven::Unit::Fixed
    }

    pub const fn range(self) -> (i32, i32) {
        const fn fx(n: i32, d: i32) -> i32 {
            Fx::ratio(n, d).raw()
        }
        match self {
            FightField::SightCone => (0, fx(1, 2)),
            FightField::BlindArc => (0, fx(1, 4)),
            FightField::BodyRadius | FightField::StepOver | FightField::FeelHeight => {
                (0, fx(20, 1))
            }
            FightField::LandingPerMetre => (0, fx(10, 1)),
            _ => (0, fx(200, 1)),
        }
    }
}

/// One kind of creature. See the module docs.
#[derive(Debug)]
pub struct Species {
    pub id: SpeciesId,
    /// As a person says it: "Ridgeback". Also the family name of its knobs in
    /// the Oven, so it is what the palette groups them under.
    pub name: &'static str,

    // ---- the skeleton ----
    pub bones: &'static [Bone],
    /// Left and right bones that swap when a clip is mirrored.
    pub mirror: &'static [(usize, usize)],
    /// The bones the head's tracking is spread across, base to tip. Empty for
    /// a creature that does not turn its head to watch you.
    pub neck: &'static [usize],
    /// Which bone a move's hit volume rides, by the move table's `Follows`
    /// number: `follows[0]` is the body's, and the rest are the species' own.
    pub follows: &'static [usize],

    // ---- the body ----
    pub parts: &'static [Part],
    /// Which parts are breakable, in part order, and how many: worked out from
    /// the flags by `beast::breakables`.
    pub breakable: ([u8; MAX_BREAKABLE], usize),
    pub legs: &'static [Leg],

    // ---- what it does ----
    pub moves: &'static [MoveDecl],
    pub clips: &'static [ClipDecl],
    pub stock: Stock,
    /// `(first row, how many)` per clip, from the species' baked file.
    pub span: &'static [(u16, u16)],
    /// How many baked rows there are.
    pub rows: usize,
    /// One baked row: `beast::channels(bones)` numbers, hips then three per
    /// bone. A function rather than a slice so the baked file can keep its
    /// table as an array of rows, one per line.
    pub row: fn(usize) -> &'static [i32],

    // ---- its numbers ----
    /// Knobs this species has that others do not: after [`Common`] in its
    /// store, before its moves.
    pub own: &'static [KnobDecl],
    /// What its `tuned.rs` holds: common, own, then moves.
    pub tuned: &'static [i32],
    /// Where that file is, from the repository root, for the bake.
    pub tuned_path: &'static str,

    // ---- its small bodies ----
    /// **The pack it brings**, if any: its kinds of critter, who it starts
    /// with, and the mind that steers them (`crate::pack`). `None` for a
    /// creature that is one body, like the Ridgeback. A creature can be a pack
    /// and nothing else -- the Gnawers, the Hornback herd -- in which case it
    /// has no skeleton at all ([`Species::has_body`]); or a body that owns a
    /// pack, as the Broodmother does her brood. See `docs/design/critters.md`.
    pub pack: Option<&'static crate::pack::PackDecl>,

    // ---- what it brings to the fight besides its body ----
    /// Hazards, objectives, senses, the room it keeps in the hunt's lore, and
    /// the hooks the shared machinery calls it by. [`FightDecl::PLAIN`] for a
    /// creature that brings none of it, which is the Ridgeback.
    pub fight: &'static FightDecl,
}

impl Species {
    /// **A creature that is only a pack**: no skeleton, no parts, no clips --
    /// its moves are its critters' moves and everything else is its
    /// [`crate::pack::PackDecl`]. The Gnawers and the Hornback herd are this;
    /// so is the dev pack, [`gnats`]. Written once here so a pack's table is
    /// its moves, its own knobs and its pack rather than a page of empty
    /// skeleton.
    #[allow(clippy::too_many_arguments)]
    pub const fn pack_only(
        id: SpeciesId,
        name: &'static str,
        moves: &'static [MoveDecl],
        own: &'static [KnobDecl],
        tuned: &'static [i32],
        tuned_path: &'static str,
        pack: &'static crate::pack::PackDecl,
    ) -> Species {
        Species {
            id,
            name,
            bones: &[],
            mirror: &[],
            neck: &[],
            follows: &[0],
            parts: &[],
            breakable: ([u8::MAX; MAX_BREAKABLE], 0),
            legs: &[],
            moves,
            clips: &[],
            stock: Stock {
                idle: 0,
                walk: 0,
                gallop: 0,
                flinch: 0,
                stumble: 0,
                topple: 0,
                dead: 0,
            },
            span: &[],
            rows: 0,
            row: |_| &[],
            own,
            tuned,
            tuned_path,
            pack: Some(pack),
            fight: &FightDecl::PLAIN,
        }
    }

    /// The same table, bringing this to the fight: hazards, objectives,
    /// senses, lore. `Species::pack_only(..).fighting(&FIGHT)`.
    pub const fn fighting(mut self, fight: &'static FightDecl) -> Species {
        self.fight = fight;
        self
    }

    /// Does it have a skeleton -- is there a `Monster` to build? A species that
    /// is only a pack has no bones, and the world builds its critters instead.
    pub fn has_body(&self) -> bool {
        !self.bones.is_empty()
    }

    /// One of its pack's kinds of critter. A species with no pack, or a kind
    /// past the end, gets the first kind of the dev pack rather than a panic in
    /// the middle of a rollback; asking is a bug in whoever built the critter.
    pub fn kind(&self, kind: u8) -> &'static crate::critter::CritterKind {
        let kinds = self.pack.map_or(gnats::PACK.kinds, |p| p.kinds);
        &kinds[(kind as usize).min(kinds.len() - 1)]
    }

    /// Where the pack's own knobs start in the store: after the moves.
    pub fn pack_base(&self) -> usize {
        Common::ALL.len() + self.own.len() + self.moves.len() * crate::oven::MONSTER_FIELDS
    }

    /// Where one of the pack's own knobs sits in the store.
    pub fn pack_index(&self, k: crate::pack::PackKnob) -> usize {
        self.pack_base() + k as usize
    }

    /// Where one field of one kind of critter sits in the store.
    pub fn critter_index(&self, kind: usize, field: crate::critter::CritterField) -> usize {
        self.pack_base()
            + crate::pack::PackKnob::ALL.len()
            + kind * crate::critter::CRITTER_FIELDS
            + field as usize
    }

    /// One of the pack's own knobs, live.
    pub fn pack_raw(&self, k: crate::pack::PackKnob) -> i32 {
        crate::oven::species_raw(self.id, self.pack_index(k))
    }

    /// One of the pack's own knobs, as fixed point.
    pub fn pack_fx(&self, k: crate::pack::PackKnob) -> Fx {
        Fx::from_raw(self.pack_raw(k))
    }

    /// The name as an identifier: lower case, spaces to underscores.
    pub fn slug(&self) -> String {
        self.name.to_lowercase().replace(' ', "_")
    }

    pub fn part_count(&self) -> usize {
        self.parts.len().min(MAX_PARTS)
    }

    /// The part's box at the species' tuned scale.
    pub fn shape(&self, index: usize) -> Shape {
        let s = self.parts[index.min(self.parts.len() - 1)].shape;
        let k = self.scale();
        Shape {
            min: s.min.scale(k),
            max: s.max.scale(k),
            ..s
        }
    }

    /// A bone's rest offset at the species' tuned scale.
    pub fn rest(&self, bone: usize) -> V3 {
        self.bones[bone.min(self.bones.len() - 1)]
            .rest
            .scale(self.scale())
    }

    /// How much of an attack's damage a part passes through. Below one is
    /// armour; a weak point is above it, which is the whole reason to climb.
    pub fn vulnerability(&self, part: usize) -> Fx {
        let p = self.parts[part.min(self.parts.len() - 1)];
        Fx::from_raw(self.own_raw(p.vuln as usize))
    }

    /// Damage here fills the poise pool.
    pub fn is_weak_point(&self, part: usize) -> bool {
        self.parts.get(part).is_some_and(|p| p.weak)
    }

    /// The health slot a breakable part keeps, if it is one.
    pub fn break_slot(&self, part: usize) -> Option<usize> {
        let (slots, count) = self.breakable;
        slots[..count].iter().position(|p| *p as usize == part)
    }

    /// The breakable parts, in part order.
    pub fn breakables(&self) -> impl Iterator<Item = usize> + '_ {
        let (slots, count) = &self.breakable;
        slots[..*count].iter().map(|p| *p as usize)
    }

    /// The bone a move's `Follows` number means.
    pub fn follow_bone(&self, follows: u8) -> usize {
        self.follows
            .get(follows as usize)
            .copied()
            .unwrap_or(self.follows[0])
    }

    /// Index of a part by name, for tools and tests that talk about parts.
    pub fn part_named(&self, name: &str) -> Option<usize> {
        self.parts.iter().position(|p| p.name == name)
    }

    /// Index of a move by name.
    pub fn move_named(&self, name: &str) -> Option<u8> {
        self.moves
            .iter()
            .position(|m| m.name == name)
            .map(|i| i as u8)
    }

    // ---- the knob store ----

    /// How many knobs this species keeps: [`Common`], its own, then one row
    /// of `oven::MonsterField` per move.
    ///
    /// Then, after the pack's, what the species brings to the fight
    /// ([`FightDecl`]): its senses and body row if it has one, a row of
    /// `hazard::HazardField` per hazard kind, and a row of
    /// `objective::ObjectiveField` per defended thing. **Appended after
    /// everything that was there before**, so a species that brings none of
    /// them keeps every index it had and its baked file does not move.
    pub fn knob_count(&self) -> usize {
        self.objective_base() + self.fight.objectives.len() * crate::objective::OBJECTIVE_FIELDS
    }

    /// Where the fight's rows start: after the pack's.
    pub fn fight_base(&self) -> usize {
        self.pack_base()
            + self.pack.map_or(0, |p| {
                crate::pack::PackKnob::ALL.len() + p.kinds.len() * crate::critter::CRITTER_FIELDS
            })
    }

    /// Where one field of the senses and body row sits. Only meaningful when
    /// `fight.row` is set; [`Species::fight_raw`] answers zero otherwise.
    pub fn fight_index(&self, f: FightField) -> usize {
        self.fight_base() + f as usize
    }

    /// One field of the senses and body row: zero for a species without one.
    pub fn fight_raw(&self, f: FightField) -> i32 {
        if !self.fight.row {
            return 0;
        }
        crate::oven::species_raw(self.id, self.fight_index(f))
    }

    pub fn fight_fx(&self, f: FightField) -> Fx {
        Fx::from_raw(self.fight_raw(f))
    }

    /// Where the hazard rows start.
    pub fn hazard_base(&self) -> usize {
        self.fight_base() + if self.fight.row { FIGHT_FIELDS } else { 0 }
    }

    /// Where one field of one hazard kind sits.
    pub fn hazard_index(&self, kind: usize, f: crate::hazard::HazardField) -> usize {
        self.hazard_base()
            + kind.min(self.fight.hazards.len().saturating_sub(1)) * crate::hazard::HAZARD_FIELDS
            + f as usize
    }

    /// Where the objective rows start.
    pub fn objective_base(&self) -> usize {
        self.hazard_base() + self.fight.hazards.len() * crate::hazard::HAZARD_FIELDS
    }

    /// Where one field of one objective sits.
    pub fn objective_index(&self, i: usize, f: crate::objective::ObjectiveField) -> usize {
        self.objective_base()
            + i.min(self.fight.objectives.len().saturating_sub(1))
                * crate::objective::OBJECTIVE_FIELDS
            + f as usize
    }

    /// One of the numbers every creature has.
    pub fn common(&self, k: Common) -> i32 {
        crate::oven::species_raw(self.id, k as usize)
    }

    /// One of this species' own knobs, by its index in [`Species::own`].
    pub fn own_raw(&self, k: usize) -> i32 {
        crate::oven::species_raw(self.id, Common::ALL.len() + k)
    }

    /// One of this species' own knobs, as fixed point.
    pub fn own_fx(&self, k: usize) -> Fx {
        Fx::from_raw(self.own_raw(k))
    }

    /// Where a move's field sits in the store.
    pub fn move_index(&self, slot: usize, field: crate::oven::MonsterField) -> usize {
        Common::ALL.len()
            + self.own.len()
            + slot.min(self.moves.len() - 1) * crate::oven::MONSTER_FIELDS
            + field as usize
    }
}
