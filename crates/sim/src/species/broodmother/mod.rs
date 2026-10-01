//! The Broodmother: a spider the height of a house, and a clock with legs.
//!
//! Eight legs spanning nine metres, a thorax at a fighter's height in front,
//! and an abdomen carried high behind it with six sacs on its back that ripen
//! while you watch and burst into more of her brood. See
//! `docs/design/creatures/broodmother.md` for what the fight is and why; this
//! file is what the animal *is* -- its skeleton, parts, legs, moves, clips and
//! its own knobs. What she does to the floor, the sacs and the fighters is
//! [`fight`]; what she wants, and what her brood want, is [`mind`].
//!
//! ```text
//!                         fangs
//!                        /
//!   abdomen ─ pedicel ─ root (thorax) ─ head
//!                         |
//!            coxa ─ femur ─ tibia   (eight legs, four a side)
//! ```
//!
//! **Her brood are the Gnawers' gnawer** (`gnawers::GNAWER_KIND`): her move
//! table starts with the gnawer's five moves, in the gnawer's order, so the
//! gnawer's own mind drives them unchanged; her body never chooses them.
//!
//! **Three bones a leg.** The coxa turns the leg to its bearing round the
//! body (yaw), the femur rises from it to a knee above her back, and the tibia
//! comes down from the knee to the floor. Every leg's box is along its own
//! bone, so the shin a fighter on the floor reaches is the tibia's lower end.
//!
//! Two generated files sit beside this one: `baked.rs`, its pose table
//! (`cargo run -p anim --bin bake_beast -- --species broodmother`), and
//! `tuned.rs`, its knobs (`cargo run -p sim --bin bake_tuning`).

pub(crate) mod baked;
pub mod fight;
pub mod legs;
pub mod mind;
mod tuned;

use crate::beast::{Bone, ClipDecl, Leg, NO_PARENT, Part, bone, breakables, part, v};
use crate::fixed::Fx;
use crate::oven::KnobDecl;
use crate::species::gnawers;
use crate::species::{MoveDecl, Species, SpeciesId, Stock};

/// Raw helper for writing fixed-point bounds readably.
const fn fx(num: i32, den: i32) -> i32 {
    Fx::ratio(num, den).raw()
}

// ---------------------------------------------------------------------------
// Bones
// ---------------------------------------------------------------------------

/// The Broodmother's bones, by index. The legs are three bones each, eight
/// legs in [`LEGS`] order: front left, front right, second left, second
/// right, third left, third right, hind left, hind right.
pub mod bones {
    pub const ROOT: usize = 0;
    pub const HEAD: usize = 1;
    pub const FANGS: usize = 2;
    pub const PEDICEL: usize = 3;
    pub const ABDOMEN: usize = 4;
    /// The first leg bone: leg `i`'s coxa is `LEG0 + 3 i`, its femur one
    /// after, its tibia two after.
    pub const LEG0: usize = 5;

    pub const fn coxa(leg: usize) -> usize {
        LEG0 + 3 * leg
    }
    pub const fn femur(leg: usize) -> usize {
        LEG0 + 3 * leg + 1
    }
    pub const fn tibia(leg: usize) -> usize {
        LEG0 + 3 * leg + 2
    }

    pub const COUNT: usize = LEG0 + 3 * 8;
}

use bones::*;

/// How many legs.
pub const LEG_COUNT: usize = 8;

/// Where each leg's coxa sits on the thorax, in metres from its middle, and
/// which side it is on: four a side, front to back.
const HIPS: [(i32, i32); 4] = [(85, 100), (30, 100), (-25, 100), (-80, 100)];

/// The femur, hip to knee, and the tibia, knee to foot, in centimetres: the
/// knee's offset from the femur's root, and the shin's box.
const LIMBS: (i32, i32) = (210, 400);

/// Leg `i`'s side: even legs are her left, odd her right.
pub const fn leg_side(leg: usize) -> i32 {
    if leg % 2 == 0 { -1 } else { 1 }
}

const fn leg_bones(leg: usize) -> [Bone; 3] {
    let side = leg_side(leg);
    let (x, xd) = HIPS[leg / 2];
    let names: [[&str; 3]; 8] = [
        ["coxa.l1", "femur.l1", "tibia.l1"],
        ["coxa.r1", "femur.r1", "tibia.r1"],
        ["coxa.l2", "femur.l2", "tibia.l2"],
        ["coxa.r2", "femur.r2", "tibia.r2"],
        ["coxa.l3", "femur.l3", "tibia.l3"],
        ["coxa.r3", "femur.r3", "tibia.r3"],
        ["coxa.l4", "femur.l4", "tibia.l4"],
        ["coxa.r4", "femur.r4", "tibia.r4"],
    ];
    let n = names[leg];
    [
        // On the thorax's side, a little below its middle.
        bone(n[0], ROOT, v((x, xd), (-20, 100), (side * 85, 100)), side),
        // The femur's root, a short way out along the coxa's bearing.
        bone(n[1], coxa(leg), v((30, 100), (0, 1), (0, 1)), side),
        // The knee: the femur's length along it.
        bone(n[2], femur(leg), v((LIMBS.0, 100), (0, 1), (0, 1)), side),
    ]
}

const fn skeleton() -> [Bone; COUNT] {
    let mut out = [bone("root", NO_PARENT, v((0, 1), (260, 100), (0, 1)), 0); COUNT];
    // The head at the front of the thorax, the fangs under its front.
    out[HEAD] = bone("head", ROOT, v((110, 100), (-20, 100), (0, 1)), 0);
    out[FANGS] = bone("fangs", HEAD, v((70, 100), (-60, 100), (0, 1)), 0);
    // The waist at the back of the thorax, a little above its middle; the
    // abdomen hinges just behind it.
    out[PEDICEL] = bone("pedicel", ROOT, v((-115, 100), (30, 100), (0, 1)), 0);
    // The abdomen's hinge: up and back from the waist, at the front of its
    // underside. It is carried high, and comes down for the slam.
    out[ABDOMEN] = bone("abdomen", PEDICEL, v((-60, 100), (250, 100), (0, 1)), 0);
    let mut leg = 0;
    while leg < LEG_COUNT {
        let b = leg_bones(leg);
        out[coxa(leg)] = b[0];
        out[femur(leg)] = b[1];
        out[tibia(leg)] = b[2];
        leg += 1;
    }
    out
}

/// The skeleton. The root is the middle of the thorax, 2.6 m up.
pub const BONES: [Bone; COUNT] = skeleton();

const fn mirror_pairs() -> [(usize, usize); 12] {
    let mut out = [(0, 0); 12];
    let mut pair = 0;
    while pair < 4 {
        let (l, r) = (2 * pair, 2 * pair + 1);
        out[3 * pair] = (coxa(l), coxa(r));
        out[3 * pair + 1] = (femur(l), femur(r));
        out[3 * pair + 2] = (tibia(l), tibia(r));
        pair += 1;
    }
    out
}

/// The left and right bones a mirrored clip swaps: every leg's three.
pub const MIRROR: [(usize, usize); 12] = mirror_pairs();

/// The head turns to keep you in view; eight eyes do not need it to turn far.
pub const NECK_CHAIN: [usize; 1] = [bones::HEAD];

/// Which bone an attack's hit volume rides: the body, the fangs, or the
/// abdomen (the slam).
pub const FOLLOWS_BODY: u8 = 0;
pub const FOLLOWS_FANGS: u8 = 1;
pub const FOLLOWS_ABDOMEN: u8 = 2;

pub const FOLLOWS: [usize; 3] = [ROOT, FANGS, ABDOMEN];

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

pub const THORAX: usize = 0;
pub const HEAD_PART: usize = 1;
pub const FANG_PART: usize = 2;
pub const PEDICEL_PART: usize = 3;
pub const ABDOMEN_PART: usize = 4;
/// The ridge along the top of the abdomen, between the sacs.
pub const CROWN: usize = 5;
/// The soft underside: what a jumping swing from under her reaches.
pub const UNDERSIDE: usize = 6;
/// The six sacs: fore pair, mid pair, aft pair, left before right.
pub const SAC0: usize = 7;
pub const SAC_COUNT: usize = 6;
/// Leg `i`'s femur is `FEMUR0 + 2 i`, its tibia (the shin) one after.
pub const FEMUR0: usize = SAC0 + SAC_COUNT;

pub const fn femur_part(leg: usize) -> usize {
    FEMUR0 + 2 * leg
}
pub const fn shin_part(leg: usize) -> usize {
    FEMUR0 + 2 * leg + 1
}

pub const PART_COUNT: usize = FEMUR0 + 2 * LEG_COUNT;

/// A sac's part, by site: 0 and 1 the fore pair, 2 and 3 the mid, 4 and 5 the
/// aft.
pub const fn sac_part(site: usize) -> usize {
    SAC0 + site
}

/// Is this part a sac, and which site?
pub const fn sac_site(part: usize) -> Option<usize> {
    if part >= SAC0 && part < SAC0 + SAC_COUNT {
        Some(part - SAC0)
    } else {
        None
    }
}

/// The four middle legs: the only shins that break.
pub const fn middle_leg(leg: usize) -> bool {
    leg >= 2 && leg < 6
}

/// Where each sac sits on the abdomen, in its frame: `(along, up)` in
/// centimetres to its middle. The abdomen's own frame starts at its hinge, at
/// the front of it half way up, 5.4 m standing, and runs back along `-x`; its
/// back is a metre above the hinge. The fore pair sits on the back, the mid pair a little up the
/// rise toward the hump, the aft pair on the hump. **Standing, every sac is
/// at least a metre and a half in from any face a fighter can be beside** --
/// the sides, the front over the thorax, the hump behind -- because that is
/// how far the roster's swings reach out from a body at the top of a hop
/// (`tests/broodmother.rs`, and `beastcheck` prints it).
const SAC_AT: [(i32, i32); 3] = [(-150, 150), (-280, 190), (-385, 270)];
/// How far either side of the abdomen's middle line a pair sits, and half a
/// sac across, in centimetres.
const SAC_SIZE: (i32, i32) = (65, 50);

const fn sac(name: &'static str, site: usize) -> Part {
    let (x, y) = SAC_AT[site / 2];
    let z = if site % 2 == 0 {
        -SAC_SIZE.0
    } else {
        SAC_SIZE.0
    };
    part(
        name,
        ABDOMEN,
        v(
            (x - SAC_SIZE.1, 100),
            (y - SAC_SIZE.1, 100),
            (z - SAC_SIZE.1, 100),
        ),
        v(
            (x + SAC_SIZE.1, 100),
            (y + SAC_SIZE.1, 100),
            (z + SAC_SIZE.1, 100),
        ),
        Knob::VulnSac as u16,
    )
    .soft()
    .breakable()
}

const fn femur_box(name: &'static str, leg: usize) -> Part {
    part(
        name,
        femur(leg),
        v((0, 1), (-18, 100), (-18, 100)),
        v((LIMBS.0, 100), (18, 100), (18, 100)),
        Knob::VulnLeg as u16,
    )
    .soft()
}

/// A shin: from the knee down to the floor along the tibia. Soft -- a body
/// walks between and through her legs -- and on the four middle legs,
/// breakable.
const fn shin_box(name: &'static str, leg: usize) -> Part {
    let p = part(
        name,
        tibia(leg),
        v((0, 1), (-16, 100), (-16, 100)),
        v((LIMBS.1, 100), (16, 100), (16, 100)),
        Knob::VulnLeg as u16,
    )
    .soft();
    if middle_leg(leg) { p.breakable() } else { p }
}

const fn body_parts() -> [Part; PART_COUNT] {
    let filler = part(
        "thorax",
        ROOT,
        v((-110, 100), (-60, 100), (-100, 100)),
        v((110, 100), (60, 100), (100, 100)),
        Knob::VulnHide as u16,
    );
    let mut out = [filler; PART_COUNT];
    // The thorax, 2.0 m to 3.2 m.
    out[THORAX] = filler;
    // The head, 1.8 m to 2.8 m, the front of her.
    out[HEAD_PART] = part(
        "head",
        HEAD,
        v((0, 1), (-60, 100), (-60, 100)),
        v((80, 100), (40, 100), (60, 100)),
        Knob::VulnHide as u16,
    );
    // The fangs, down to 1.2 m: the melee threat is at a fighter's height.
    out[FANG_PART] = part(
        "fangs",
        FANGS,
        v((-20, 100), (-60, 100), (-45, 100)),
        v((40, 100), (10, 100), (45, 100)),
        Knob::VulnHide as u16,
    )
    .soft();
    // The waist, rising from the back of the thorax to the abdomen's hinge:
    // a weak point only when she is low.
    out[PEDICEL_PART] = part(
        "pedicel",
        PEDICEL,
        v((-70, 100), (-30, 100), (-35, 100)),
        v((10, 100), (250, 100), (35, 100)),
        Knob::VulnPedicel as u16,
    )
    .soft();
    // The abdomen, 5.6 m long and five across, 4.5 m to 6.4 m standing:
    // carried high, so that the space under it is somewhere to stand and the
    // sacs on its back are out of a hop.
    out[ABDOMEN_PART] = part(
        "abdomen",
        ABDOMEN,
        v((-560, 100), (-90, 100), (-250, 100)),
        v((0, 1), (100, 100), (250, 100)),
        Knob::VulnHide as u16,
    );
    // The hump at the back of it, that the aft pair sits on.
    out[CROWN] = part(
        "hump",
        ABDOMEN,
        v((-440, 100), (100, 100), (-140, 100)),
        v((-300, 100), (220, 100), (140, 100)),
        Knob::VulnHide as u16,
    );
    // The underside: a soft skin over the bottom of the abdomen.
    out[UNDERSIDE] = part(
        "underside",
        ABDOMEN,
        v((-460, 100), (-95, 100), (-220, 100)),
        v((-40, 100), (-60, 100), (220, 100)),
        Knob::VulnUnder as u16,
    )
    .soft();
    out[sac_part(0)] = sac("fore sac, left", 0);
    out[sac_part(1)] = sac("fore sac, right", 1);
    out[sac_part(2)] = sac("mid sac, left", 2);
    out[sac_part(3)] = sac("mid sac, right", 3);
    out[sac_part(4)] = sac("aft sac, left", 4);
    out[sac_part(5)] = sac("aft sac, right", 5);
    let femurs = [
        "femur l1", "femur r1", "femur l2", "femur r2", "femur l3", "femur r3", "femur l4",
        "femur r4",
    ];
    let shins = [
        "shin l1", "shin r1", "shin l2", "shin r2", "shin l3", "shin r3", "shin l4", "shin r4",
    ];
    let mut leg = 0;
    while leg < LEG_COUNT {
        out[femur_part(leg)] = femur_box(femurs[leg], leg);
        out[shin_part(leg)] = shin_box(shins[leg], leg);
        leg += 1;
    }
    out
}

pub const PARTS: [Part; PART_COUNT] = body_parts();

/// The femur's length, hip to knee, at scale one: the knee's offset.
pub const fn femur_length() -> Fx {
    BONES[bones::tibia(0)].rest.x
}

/// The tibia's length, knee to foot, at scale one: the shin's box.
pub const fn tibia_length() -> Fx {
    PARTS[shin_part(0)].shape.max.x
}

const fn leg_table() -> [Leg; LEG_COUNT] {
    let mut out = [Leg {
        upper: 0,
        foot: 0,
        hip: 0,
        knee: 0,
        front: true,
        side: -1,
    }; LEG_COUNT];
    let mut leg = 0;
    while leg < LEG_COUNT {
        out[leg] = Leg {
            upper: femur_part(leg),
            foot: shin_part(leg),
            hip: femur(leg),
            knee: tibia(leg),
            front: leg < 4,
            side: leg_side(leg),
        };
        leg += 1;
    }
    out
}

/// The eight legs, front to back, left before right.
pub const LEGS: [Leg; LEG_COUNT] = leg_table();

// ---------------------------------------------------------------------------
// Moves
// ---------------------------------------------------------------------------

/// The brood's moves first, in the gnawer's own order, so a broodling is a
/// gnawer move for move. Her body never chooses them.
pub const DART: u8 = gnawers::DART;
pub const HAMSTRING: u8 = gnawers::HAMSTRING;
pub const PILE_ON: u8 = gnawers::PILE_ON;
pub const GNAW: u8 = gnawers::GNAW;
pub const SCRAMBLE: u8 = gnawers::SCRAMBLE;
/// One leg up past the knee and driven down onto a disc beside her.
pub const STAB: u8 = 5;
/// The next stab of a flurry: never chosen, chained on by her rules.
pub const FLURRY: u8 = 6;
/// The thorax drops and the front of her lunges, fangs spread.
pub const LUNGE: u8 = 7;
/// The abdomen up, held, and crashed to the floor: the fight's window.
pub const SLAM: u8 = 8;
/// A scream that calls every broodling home. Always followed by the slam.
pub const SCREECH: u8 = 9;
/// A glob from the spinnerets, behind her, where you will be.
pub const WEB_SHOT: u8 = 10;
/// A line to a wall anchor, and she reels herself across the cave.
pub const WEB_LINE: u8 = 11;
/// A sac bursting: two broodlings and a splash. Never chosen -- the clock.
pub const HATCH: u8 = 12;

pub const MOVE_COUNT: usize = 13;

const fn brood(m: MoveDecl) -> MoveDecl {
    m.unanimated()
}

pub const MOVES: [MoveDecl; MOVE_COUNT] = [
    brood(gnawers::MOVES[0]),
    brood(gnawers::MOVES[1]),
    brood(gnawers::MOVES[2]),
    brood(gnawers::MOVES[3]),
    brood(gnawers::MOVES[4]),
    MoveDecl::new("Leg stab", Clip::Stab as usize).lobbed(),
    MoveDecl::new("Flurry stab", Clip::Stab as usize)
        .lobbed()
        .never_chosen(),
    MoveDecl::new("Fang lunge", Clip::Lunge as usize),
    MoveDecl::new("Sac slam", Clip::Slam as usize),
    MoveDecl::new("Screech", Clip::Screech as usize).then(SLAM),
    MoveDecl::new("Web shot", Clip::WebShot as usize).lobbed(),
    MoveDecl::new("Web line", Clip::WebLine as usize).own_hit(),
    MoveDecl::new("Hatch", Clip::Idle as usize)
        .own_hit()
        .unanimated(),
];

// ---------------------------------------------------------------------------
// Clips
// ---------------------------------------------------------------------------

/// Everything the Broodmother knows how to look like, in the order its baked
/// table lays them out. Authored in `crates/anim/src/beast/broodmother/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clip {
    Idle,
    /// An eight-legged walk, two alternating tetrapods, by ground covered.
    Walk,
    /// The enraged run: the same gait, longer and lower.
    Run,
    /// The body's half of a stab: a lean toward the leg that drives. The leg
    /// itself is lifted by her own layer (`fight::layer`), toward the disc.
    Stab,
    Lunge,
    Slam,
    Screech,
    WebShot,
    WebLine,
    Flinch,
    /// A leg breaking: down on that side for a moment.
    Stumble,
    /// The last sac popped: flat on the floor, legs splayed.
    Collapse,
    Dead,
}

pub const CLIP_COUNT: usize = 13;

impl Clip {
    pub const ALL: [Clip; CLIP_COUNT] = [
        Clip::Idle,
        Clip::Walk,
        Clip::Run,
        Clip::Stab,
        Clip::Lunge,
        Clip::Slam,
        Clip::Screech,
        Clip::WebShot,
        Clip::WebLine,
        Clip::Flinch,
        Clip::Stumble,
        Clip::Collapse,
        Clip::Dead,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn name(self) -> &'static str {
        CLIPS[self as usize].name
    }
}

impl From<Clip> for usize {
    fn from(clip: Clip) -> usize {
        clip.index()
    }
}

const fn cycle(name: &'static str) -> ClipDecl {
    ClipDecl {
        name,
        looping: true,
        phased: false,
    }
}

const fn attack(name: &'static str) -> ClipDecl {
    ClipDecl {
        name,
        looping: false,
        phased: true,
    }
}

const fn once(name: &'static str) -> ClipDecl {
    ClipDecl {
        name,
        looping: false,
        phased: false,
    }
}

pub const CLIPS: [ClipDecl; CLIP_COUNT] = [
    cycle("idle"),
    cycle("walk"),
    cycle("run"),
    attack("stab"),
    attack("lunge"),
    attack("slam"),
    attack("screech"),
    attack("web shot"),
    attack("web line"),
    once("flinch"),
    once("stumble"),
    once("collapse"),
    once("dead"),
];

// ---------------------------------------------------------------------------
// Its own knobs
// ---------------------------------------------------------------------------

crate::species_knobs! {
    /// The knobs only the Broodmother has: her hide, the sac clock, the brood,
    /// the web, her legs, her arc and her own terms in the brain. The design
    /// document's names for them (§5) are in the comment beside each, where
    /// they differ. **Append only**: the baked file is read by position.
    pub enum Knob;
    VulnHide,        "hide",    "Hide (x damage)",                       Fixed, 0, fx(3,1);
    VulnLeg,         "hide",    "Legs (x damage)",                       Fixed, 0, fx(3,1);
    VulnSac,         "hide",    "Sacs (x damage)",                       Fixed, 0, fx(4,1);
    VulnUnder,       "hide",    "Underside (x damage)",                  Fixed, 0, fx(4,1);
    VulnPedicel,     "hide",    "Pedicel (x damage)",                    Fixed, 0, fx(4,1);
    PedicelLow,      "hide",    "Pedicel, while she is down (x)",        Fixed, 0, fx(4,1);
    Steepest,        "hide",    "Nothing on her is floor (cos)",         Fixed, 0, fx(2,1);
    // sac_ripen
    SacRipen,        "sacs",    "A sac swells for",                      Frames, 0, 7200;
    SacAmber,        "sacs",    "Pale until (share)",                    Fixed, 0, fx(1,1);
    SacRed,          "sacs",    "Red and twitching for the last",        Frames, 0, 900;
    // sac_regrow
    SacRegrow,       "sacs",    "A burst site lays again after",         Frames, 0, 3600;
    SacHealth,       "sacs",    "A sac's health",                        Int,   0, 2000;
    PopDamage,       "sacs",    "A pop deals her",                       Int,   0, 2000;
    PopStrain,       "sacs",    "A pop adds strain",                     Int,   0, 4000;
    PopFlinch,       "sacs",    "A pop flinches her for",                Frames, 0, 120;
    // sac_ripen_coop
    SacRipenCoop,    "sacs",    "The clock with two hunters (x)",        Fixed, 0, fx(2,1);
    // brood_cap
    BroodCap,        "brood",   "Brood alive at most",                   Int,   0, 10;
    BroodPerSac,     "brood",   "Broodlings a sac drops",                Int,   0, 4;
    BroodAtStart,    "brood",   "Broodlings when the hunt begins",       Int,   0, 8;
    // brood_guard_radius
    GuardRadius,     "brood",   "Brood guard, broodlings within",        Fixed, 0, fx(20,1);
    GuardFrames,     "brood",   "Brood guard lasts",                     Frames, 0, 600;
    GuardTokens,     "brood",   "Brood guard, more tokens on the attacker", Int, 0, 4;
    // rooted_tokens
    RootedTokens,    "brood",   "Tokens on a rooted target",             Int,   0, 6;
    RecallRadius,    "brood",   "A screech gathers them within, of her", Fixed, 0, fx(12,1);
    // web shot
    WebRoot,         "web",     "A glob roots for",                      Frames, 0, 300;
    CutFree,         "web",     "Each attack cuts this off the root",    Frames, 0, 120;
    GlobRadius,      "web",     "A glob's hit, radius",                  Fixed, 0, fx(3,1);
    PatchRadius,     "web",     "A patch, radius",                       Fixed, 0, fx(4,1);
    WebDown,         "web",     "A glob in the air webs you down at",    Fixed, 0, fx(40,1);
    // web line
    LineSpeed,       "web",     "Reels across at",                       Fixed, 0, fx(40,1);
    LineReach,       "web",     "Reels at most",                         Fixed, 0, fx(40,1);
    LineLane,        "web",     "The lane, half-width",                  Fixed, 0, fx(4,1);
    LineFrom,        "web",     "Fires a line at a target beyond",       Fixed, 0, fx(30,1);
    // strand_life
    StrandHalf,      "strands", "A strand, half-width",                  Fixed, 0, fx(1,1);
    StrandApart,     "strands", "The two strands, apart",                Fixed, 0, fx(6,1);
    TripSpeed,       "strands", "Trips anything crossing faster than",   Fixed, 0, fx(12,1);
    TripDamage,      "strands", "A trip deals",                          Int,   0, 200;
    TripStagger,     "strands", "A trip staggers for",                   Frames, 0, 240;
    // list_drop
    ListDrop,        "legs",    "Two broken on a side: it sits lower by", Fixed, 0, fx(3,1);
    ListLegs,        "legs",    "Broken on a side to list",              Int,   0, 4;
    // clutch_health
    ClutchHealth,    "arc",     "The clutch below (x health)",           Fixed, 0, fx(1,1);
    ClutchFrames,    "arc",     "The clutch ripens every sac over",      Frames, 0, 1200;
    CollapseFrames,  "arc",     "The collapse lasts",                    Frames, 0, 600;
    // enrage_speed
    EnrageSpeed,     "arc",     "Enraged, walks at",                     Fixed, 0, fx(20,1);
    EnrageThink,     "arc",     "Enraged, thinks this much (x)",         Fixed, 0, fx(1,1);
    EnrageFlurry,    "arc",     "Enraged, stabs in a flurry at most",    Int,   0, 6;
    EnrageLunge,     "arc",     "Enraged, the lunge winds up for",       Frames, 0, 60;
    EnrageLines,     "arc",     "Enraged, web lines (x appetite)",       Fixed, 0, fx(4,1);
    // the stab
    StabNear,        "stab",    "An outward stab lands in from its foot", Fixed, 0, fx(6,1);
    StabFar,         "stab",    "An inward stab keeps off her middle by", Fixed, 0, fx(8,1);
    Flurry,          "stab",    "Stabs in a flurry at most",             Int,   0, 6;
    // screech_lockout
    // under(m)
    UnderAppetite,   "mind",    "Under(m): the slam, somebody under her", Int,  0, 8000;
    // aloft(m)
    AloftAppetite,   "mind",    "Aloft(m): the web shot, an airborne target", Int, 0, 4000;
    // recall(m)
    RecallAppetite,  "mind",    "Recall(m): per brood-metre from her / cap", Int, 0, 400;
    ThreatAppetite,  "mind",    "Recall(m): per fighter at a sac",       Int,   0, 4000;
    LineAppetite,    "mind",    "The web line, at a target far off",     Int,   0, 4000;
    ListRoll,        "legs",    "Listing, she rolls toward it (turns)",  Fixed, 0, fx(1,8);
    StabLift,        "stab",    "The stabbing foot rises to",            Fixed, 0, fx(6,1);
    StabSlack,       "stab",    "Stabs at a target this near a disc",    Fixed, 0, fx(4,1);
}

const OWN: &[KnobDecl] = Knob::DECLS;

impl Knob {
    /// The live value, as fixed point.
    pub fn fx(self) -> Fx {
        SPECIES.own_fx(self as usize)
    }

    /// The live value, raw: an integer, or frames.
    pub fn raw(self) -> i32 {
        SPECIES.own_raw(self as usize)
    }
}

// ---------------------------------------------------------------------------
// The species
// ---------------------------------------------------------------------------

pub static SPECIES: Species = Species {
    id: SpeciesId::BROODMOTHER,
    name: "Broodmother",
    bones: &BONES,
    mirror: &MIRROR,
    neck: &NECK_CHAIN,
    follows: &FOLLOWS,
    parts: &PARTS,
    breakable: breakables(&PARTS),
    legs: &LEGS,
    moves: &MOVES,
    clips: &CLIPS,
    stock: Stock {
        idle: Clip::Idle as usize,
        walk: Clip::Walk as usize,
        gallop: Clip::Run as usize,
        flinch: Clip::Flinch as usize,
        stumble: Clip::Stumble as usize,
        topple: Clip::Collapse as usize,
        dead: Clip::Dead as usize,
    },
    span: &baked::SPAN,
    rows: baked::ROWS,
    row: |r| &baked::FRAMES[r],
    own: OWN,
    tuned: &tuned::KNOBS,
    tuned_path: "crates/sim/src/species/broodmother/tuned.rs",
    pack: Some(&mind::PACK),
    fight: &fight::FIGHT,
};
