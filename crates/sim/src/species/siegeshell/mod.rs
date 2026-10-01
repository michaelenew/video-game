//! The Siegeshell: a hill that walks to the wall.
//!
//! A crab-shelled, turtle-headed colossus twenty-four metres to the top of its
//! crown and forty long, on six legs like towers, walking down a valley toward
//! a town wall. See `docs/design/creatures/siegeshell.md` for what the fight is
//! and why; this file is what the animal *is* -- its skeleton, its parts, its
//! legs, its moves, its clips and its own knobs. How it walks and where its
//! feet go is [`gait`]; what it does to the valley, its riders and the wall is
//! [`fight`]; what it wants, and what its parasites want, is [`mind`].
//!
//! ```text
//!                      crown ── anchors (three)
//!                     /
//!   shell.l ── root ── shell.r            neck ── head
//!                |  \____________________/
//!     hip ─ thigh ─ shin ─ ankle   (six legs, three a side)
//! ```
//!
//! **The shell is a stepped dome** of boxes that all reach down to the belly,
//! so it is solid from the plastron up and every tread is a top you can stand
//! on: the rim at 14 m, the flank at 16 and 18, the plateau at 20, the crown
//! at 22 with three anchors standing 2.5 m on it. The rim and the flank of each
//! side hang off that side's own bone, hinged at the plateau's edge, so a shrug
//! rolls one side and leaves the plateau and the crown where they are -- which
//! is the whole of why the crown is calm in a shrug and the rim is not, read
//! off the grip test rather than written down.
//!
//! **The legs hang off the root, not the shell**, from a hip just under the
//! rim's outer edge, and are placed by [`gait::repose`] every frame: the clips
//! never move a leg. Four bones a leg -- the hip turns the leg to its foot, the
//! thigh and shin are a two-link solve in that plane, and the ankle levels the
//! pad -- and four parts: the thigh (a stair, splayed), the shin, the ankle
//! (the breakable box, 0.5 to 3.5 m) and the pad under it.
//!
//! Two generated files sit beside this one: `baked.rs`, its pose table
//! (`cargo run -p anim --bin bake_beast -- --species siegeshell`), and
//! `tuned.rs`, its knobs (`cargo run -p sim --bin bake_tuning`).

pub(crate) mod baked;
pub mod fight;
pub mod gait;
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

/// The Siegeshell's bones, by index. Legs are four bones each, in [`LEGS`]
/// order: fore left, fore right, mid left, mid right, hind left, hind right.
pub mod bones {
    pub const ROOT: usize = 0;
    /// The left side of the shell: the rim and the flank, hinged at the
    /// plateau's left edge.
    pub const SHELL_L: usize = 1;
    pub const SHELL_R: usize = 2;
    /// The crown and the anchors on it.
    pub const CROWN: usize = 3;
    pub const NECK: usize = 4;
    pub const HEAD: usize = 5;
    /// The first leg bone: leg `i`'s hip is `LEG0 + 4 i`, then its thigh, its
    /// shin (whose origin is the knee) and its ankle (the top of the ankle
    /// box).
    pub const LEG0: usize = 6;

    pub const fn hip(leg: usize) -> usize {
        LEG0 + 4 * leg
    }
    pub const fn thigh(leg: usize) -> usize {
        LEG0 + 4 * leg + 1
    }
    pub const fn shin(leg: usize) -> usize {
        LEG0 + 4 * leg + 2
    }
    pub const fn ankle(leg: usize) -> usize {
        LEG0 + 4 * leg + 3
    }

    pub const COUNT: usize = LEG0 + 4 * super::LEG_COUNT;
}

use bones::*;

/// How many legs.
pub const LEG_COUNT: usize = 6;

/// Leg `i`'s side: even legs are its left, odd its right.
pub const fn leg_side(leg: usize) -> i32 {
    if leg % 2 == 0 { -1 } else { 1 }
}

/// Leg `i`'s pair: 0 fore, 1 mid, 2 hind.
pub const fn leg_pair(leg: usize) -> usize {
    leg / 2
}

/// Which tripod a leg walks in: the fore left, mid right and hind left are
/// one (0), the other three the other (1). A tripod lands together.
pub const fn tripod(leg: usize) -> usize {
    (leg_pair(leg) + (leg % 2)) % 2
}

/// Where each pair's hip sits along the body, in centimetres from the root.
pub const HIP_X: [i32; 3] = [900, -100, -1100];
/// The body's measurements, in centimetres: geometry, as a skeleton's bones
/// and an arena's solids are -- data in the table, not knobs.
#[derive(Clone, Copy, Debug)]
pub struct Build {
    /// The hip: how high above the root, and how far out. Just under the
    /// rim's outer edge, so a leg splayed out sideways leaves the shell's
    /// side near the rim's top -- which is what makes it a stair.
    pub hip_up: i32,
    pub hip_out: i32,
    /// The thigh, hip to knee, and the shin, knee to the top of the ankle.
    pub thigh: i32,
    pub shin: i32,
    /// How far out from the middle line a standing foot is planted: the
    /// feet are twenty-eight metres apart.
    pub foot_out: i32,
    /// The ankle box hangs this far below the ankle bone, and the pad under
    /// it is this thick: the ankle is 0.5 to 3.5 m, the pad the floor to
    /// 0.5 m.
    pub ankle_drop: i32,
    pub pad_thick: i32,
    /// The root above the floor, standing: the plastron.
    pub root_up: i32,
    /// Half an anchor across, and how tall it stands on the crown.
    pub anchor_half: i32,
    pub anchor_tall: i32,
}

pub const BUILD: Build = Build {
    hip_up: 400,
    hip_out: 1100,
    thigh: 550,
    shin: 500,
    foot_out: 1400,
    ankle_drop: 300,
    pad_thick: 50,
    root_up: 900,
    anchor_half: 70,
    anchor_tall: 250,
};

const fn leg_bones(leg: usize) -> [Bone; 4] {
    let side = leg_side(leg);
    let x = HIP_X[leg_pair(leg)];
    let names: [[&str; 4]; 6] = [
        ["hip.fl", "thigh.fl", "shin.fl", "ankle.fl"],
        ["hip.fr", "thigh.fr", "shin.fr", "ankle.fr"],
        ["hip.ml", "thigh.ml", "shin.ml", "ankle.ml"],
        ["hip.mr", "thigh.mr", "shin.mr", "ankle.mr"],
        ["hip.hl", "thigh.hl", "shin.hl", "ankle.hl"],
        ["hip.hr", "thigh.hr", "shin.hr", "ankle.hr"],
    ];
    let n = names[leg];
    [
        bone(
            n[0],
            ROOT,
            v((x, 100), (BUILD.hip_up, 100), (side * BUILD.hip_out, 100)),
            side,
        ),
        bone(n[1], hip(leg), v((0, 1), (0, 1), (0, 1)), side),
        bone(
            n[2],
            thigh(leg),
            v((BUILD.thigh, 100), (0, 1), (0, 1)),
            side,
        ),
        bone(n[3], shin(leg), v((BUILD.shin, 100), (0, 1), (0, 1)), side),
    ]
}

const fn skeleton() -> [Bone; COUNT] {
    let mut out = [bone(
        "root",
        NO_PARENT,
        v((0, 1), (BUILD.root_up, 100), (0, 1)),
        0,
    ); COUNT];
    // The two sides of the shell, hinged at the plateau's edges, at its top.
    out[SHELL_L] = bone("shell.l", ROOT, v((0, 1), (1100, 100), (-550, 100)), -1);
    out[SHELL_R] = bone("shell.r", ROOT, v((0, 1), (1100, 100), (550, 100)), 1);
    // The crown's bone at the plateau's top, in its middle.
    out[CROWN] = bone("crown", ROOT, v((0, 1), (1100, 100), (0, 1)), 0);
    // The neck leaves the front of the shell low, and the head hangs below it.
    out[NECK] = bone("neck", ROOT, v((1300, 100), (200, 100), (0, 1)), 0);
    out[HEAD] = bone("head", NECK, v((500, 100), (-300, 100), (0, 1)), 0);
    let mut leg = 0;
    while leg < LEG_COUNT {
        let b = leg_bones(leg);
        out[hip(leg)] = b[0];
        out[thigh(leg)] = b[1];
        out[shin(leg)] = b[2];
        out[ankle(leg)] = b[3];
        leg += 1;
    }
    out
}

/// The skeleton. The root is the middle of the plastron, 9 m up.
pub const BONES: [Bone; COUNT] = skeleton();

const fn mirror_pairs() -> [(usize, usize); 13] {
    let mut out = [(SHELL_L, SHELL_R); 13];
    let mut pair = 0;
    while pair < 3 {
        let (l, r) = (2 * pair, 2 * pair + 1);
        out[1 + 4 * pair] = (hip(l), hip(r));
        out[2 + 4 * pair] = (thigh(l), thigh(r));
        out[3 + 4 * pair] = (shin(l), shin(r));
        out[4 + 4 * pair] = (ankle(l), ankle(r));
        pair += 1;
    }
    out
}

/// The left and right bones a mirrored clip swaps: the shell's two sides and
/// every leg's four.
pub const MIRROR: [(usize, usize); 13] = mirror_pairs();

/// The head turns, a little, toward what is ahead of it.
pub const NECK_CHAIN: [usize; 2] = [bones::NECK, bones::HEAD];

/// Which bone a hit rides: the body, the head (the plough), the crown.
pub const FOLLOWS: [usize; 3] = [ROOT, HEAD, CROWN];

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

/// The three anchors on the crown: fore, left, right. First, so part zero is
/// the thing on the body that takes a blow at full value.
pub const ANCHOR0: usize = 0;
pub const ANCHOR_COUNT: usize = 3;
pub const CROWN_PART: usize = 3;
pub const PLATEAU_FORE: usize = 4;
pub const PLATEAU_MID: usize = 5;
pub const PLATEAU_AFT: usize = 6;
pub const FLANK_UPPER_L: usize = 7;
pub const FLANK_UPPER_R: usize = 8;
pub const FLANK_LOWER_L: usize = 9;
pub const FLANK_LOWER_R: usize = 10;
/// The rim: fore, mid and aft, left before right.
pub const RIM0: usize = 11;
pub const PLASTRON: usize = 17;
pub const NECK_PART: usize = 18;
pub const HEAD_PART: usize = 19;
/// Leg `i`'s thigh is `THIGH0 + i`, its shin `SHIN0 + i`, its ankle
/// `ANKLE0 + i` and its pad `PAD0 + i`.
pub const THIGH0: usize = 20;
pub const SHIN0: usize = 26;
pub const ANKLE0: usize = 32;
pub const PAD0: usize = 38;
pub const PART_COUNT: usize = 44;

pub const fn anchor_part(i: usize) -> usize {
    ANCHOR0 + i
}
pub const fn rim_part(segment: usize, side: i32) -> usize {
    RIM0 + 2 * segment + if side < 0 { 0 } else { 1 }
}
pub const fn thigh_part(leg: usize) -> usize {
    THIGH0 + leg
}
pub const fn shin_part(leg: usize) -> usize {
    SHIN0 + leg
}
pub const fn ankle_part(leg: usize) -> usize {
    ANKLE0 + leg
}
pub const fn pad_part(leg: usize) -> usize {
    PAD0 + leg
}

/// Which anchor a part is, if it is one.
pub const fn anchor_of(part: usize) -> Option<usize> {
    if part < ANCHOR0 + ANCHOR_COUNT {
        Some(part - ANCHOR0)
    } else {
        None
    }
}

/// Which leg's ankle a part is, if it is one.
pub const fn ankle_of(part: usize) -> Option<usize> {
    if part >= ANKLE0 && part < ANKLE0 + LEG_COUNT {
        Some(part - ANKLE0)
    } else {
        None
    }
}

/// Which side of the shell a part is on, for the rim and the flank: `-1`
/// left, `+1` right, `0` neither.
pub const fn shell_side(part: usize) -> i32 {
    match part {
        FLANK_UPPER_L | FLANK_LOWER_L => -1,
        FLANK_UPPER_R | FLANK_LOWER_R => 1,
        p if p >= RIM0 && p < RIM0 + 6 => {
            if (p - RIM0) % 2 == 0 {
                -1
            } else {
                1
            }
        }
        _ => 0,
    }
}

/// Where each anchor stands on the crown, in centimetres in the crown's
/// frame: one ahead on the middle line, two behind it either side -- a
/// triangle round the peak.
pub const ANCHOR_AT: [(i32, i32); 3] = [(300, 0), (-200, -260), (-200, 260)];

const fn anchor(name: &'static str, i: usize) -> Part {
    let (x, z) = ANCHOR_AT[i];
    part(
        name,
        CROWN,
        v(
            (x - BUILD.anchor_half, 100),
            (200, 100),
            (z - BUILD.anchor_half, 100),
        ),
        v(
            (x + BUILD.anchor_half, 100),
            (200 + BUILD.anchor_tall, 100),
            (z + BUILD.anchor_half, 100),
        ),
        Knob::VulnAnchor as u16,
    )
    .sheds()
    .breakable()
}

/// A box on one side's shell bone, mirrored for the right: x from `x0` to
/// `x1`, outward from the hinge `out0` to `out1`, its top `top` below the
/// hinge's height -- all in centimetres -- down to the belly.
const fn side_box(name: &'static str, side: i32, x: (i32, i32), out: (i32, i32), top: i32) -> Part {
    let (z0, z1) = if side < 0 {
        (-out.1, -out.0)
    } else {
        (out.0, out.1)
    };
    part(
        name,
        if side < 0 { SHELL_L } else { SHELL_R },
        v((x.0, 100), (-1100, 100), (z0, 100)),
        v((x.1, 100), (-top, 100), (z1, 100)),
        Knob::VulnShell as u16,
    )
    .mountable()
}

/// Rim segments, fore to aft: x spans in centimetres.
const RIM_X: [(i32, i32); 3] = [(400, 1400), (-600, 400), (-1600, -600)];

const fn body_parts() -> [Part; PART_COUNT] {
    let filler = part(
        "plateau, mid",
        ROOT,
        v((-400, 100), (0, 1), (-550, 100)),
        v((400, 100), (1100, 100), (550, 100)),
        Knob::VulnShell as u16,
    )
    .mountable();
    let mut out = [filler; PART_COUNT];
    out[anchor_part(0)] = anchor("anchor, fore", 0);
    out[anchor_part(1)] = anchor("anchor, left", 1);
    out[anchor_part(2)] = anchor("anchor, right", 2);
    // The crown: 10 m by 9, its top 2 m above the plateau.
    out[CROWN_PART] = part(
        "crown",
        CROWN,
        v((-500, 100), (-300, 100), (-450, 100)),
        v((500, 100), (200, 100), (450, 100)),
        Knob::VulnShell as u16,
    )
    .mountable();
    // The plateau, three plates down the middle at 20 m.
    out[PLATEAU_FORE] = part(
        "plateau, fore",
        ROOT,
        v((400, 100), (0, 1), (-550, 100)),
        v((1300, 100), (1100, 100), (550, 100)),
        Knob::VulnShell as u16,
    )
    .mountable();
    out[PLATEAU_MID] = filler;
    out[PLATEAU_AFT] = part(
        "plateau, aft",
        ROOT,
        v((-1500, 100), (0, 1), (-550, 100)),
        v((-400, 100), (1100, 100), (550, 100)),
        Knob::VulnShell as u16,
    )
    .mountable();
    // The flank: two treads a side, 18 m and 16 m.
    out[FLANK_UPPER_L] = side_box("flank, upper left", -1, (-1400, 1200), (0, 200), 200);
    out[FLANK_UPPER_R] = side_box("flank, upper right", 1, (-1400, 1200), (0, 200), 200);
    out[FLANK_LOWER_L] = side_box("flank, lower left", -1, (-1500, 1300), (200, 400), 400);
    out[FLANK_LOWER_R] = side_box("flank, lower right", 1, (-1500, 1300), (200, 400), 400);
    // The rim at 14 m: three segments a side.
    let rims = [
        "rim, fore left",
        "rim, fore right",
        "rim, mid left",
        "rim, mid right",
        "rim, aft left",
        "rim, aft right",
    ];
    let mut seg = 0;
    while seg < 3 {
        out[rim_part(seg, -1)] = side_box(rims[2 * seg], -1, RIM_X[seg], (400, 650), 600);
        out[rim_part(seg, 1)] = side_box(rims[2 * seg + 1], 1, RIM_X[seg], (400, 650), 600);
        seg += 1;
    }
    // The plastron: the underside, 9 m up. Soft -- nobody reaches it, and
    // it is where the parasites roost.
    out[PLASTRON] = part(
        "plastron",
        ROOT,
        v((-1400, 100), (-40, 100), (-1000, 100)),
        v((1200, 100), (20, 100), (1000, 100)),
        Knob::VulnShell as u16,
    )
    .soft();
    // The neck, out of the front of the shell, and the low heavy head.
    out[NECK_PART] = part(
        "neck",
        NECK,
        v((-100, 100), (-200, 100), (-250, 100)),
        v((500, 100), (200, 100), (250, 100)),
        Knob::VulnLimb as u16,
    );
    out[HEAD_PART] = part(
        "head",
        HEAD,
        v((-100, 100), (-300, 100), (-300, 100)),
        v((500, 100), (150, 100), (300, 100)),
        Knob::VulnLimb as u16,
    );
    let names = [
        ["thigh fl", "shin fl", "ankle fl", "pad fl"],
        ["thigh fr", "shin fr", "ankle fr", "pad fr"],
        ["thigh ml", "shin ml", "ankle ml", "pad ml"],
        ["thigh mr", "shin mr", "ankle mr", "pad mr"],
        ["thigh hl", "shin hl", "ankle hl", "pad hl"],
        ["thigh hr", "shin hr", "ankle hr", "pad hr"],
    ];
    let mut leg = 0;
    while leg < LEG_COUNT {
        let n = names[leg];
        // The thigh, along its bone: a stair when it is splayed, and only
        // then something to stand on (`fight::presence`).
        out[thigh_part(leg)] = part(
            n[0],
            thigh(leg),
            v((0, 1), (-90, 100), (-90, 100)),
            v((BUILD.thigh, 100), (90, 100), (90, 100)),
            Knob::VulnLimb as u16,
        )
        .mountable();
        out[shin_part(leg)] = part(
            n[1],
            shin(leg),
            v((0, 1), (-80, 100), (-80, 100)),
            v((BUILD.shin, 100), (80, 100), (80, 100)),
            Knob::VulnLimb as u16,
        );
        // The ankle: the breakable box, hanging from the ankle bone.
        out[ankle_part(leg)] = part(
            n[2],
            ankle(leg),
            v((-120, 100), (-BUILD.ankle_drop, 100), (-120, 100)),
            v((120, 100), (0, 1), (120, 100)),
            Knob::VulnAnkle as u16,
        )
        .breakable();
        // The pad: solid, and nobody's footing.
        out[pad_part(leg)] = part(
            n[3],
            ankle(leg),
            v(
                (-200, 100),
                (-BUILD.ankle_drop - BUILD.pad_thick, 100),
                (-200, 100),
            ),
            v((200, 100), (-BUILD.ankle_drop, 100), (200, 100)),
            Knob::VulnLimb as u16,
        )
        .sheds();
        leg += 1;
    }
    out
}

pub const PARTS: [Part; PART_COUNT] = body_parts();

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
            upper: thigh_part(leg),
            foot: ankle_part(leg),
            hip: thigh(leg),
            knee: shin(leg),
            front: leg_pair(leg) == 0,
            side: leg_side(leg),
        };
        leg += 1;
    }
    out
}

/// The six legs, fore to hind, left before right.
pub const LEGS: [Leg; LEG_COUNT] = leg_table();

// ---------------------------------------------------------------------------
// Moves
// ---------------------------------------------------------------------------

/// The parasites' moves first, in the gnawer's own order, so a parasite is a
/// gnawer move for move. Its body never chooses them.
pub const DART: u8 = gnawers::DART;
pub const HAMSTRING: u8 = gnawers::HAMSTRING;
pub const PILE_ON: u8 = gnawers::PILE_ON;
pub const GNAW: u8 = gnawers::GNAW;
pub const SCRAMBLE: u8 = gnawers::SCRAMBLE;
/// A tripod landing: the pad, and the ring round it. The gait, never chosen.
pub const FOOTFALL: u8 = 5;
/// One foot lifted, held over somebody, and dropped. The legs' channel.
pub const STAMP: u8 = 6;
/// A planted foot dragged inward along the ground. The legs' channel.
pub const DRAG: u8 = 7;
/// Shell plates shed onto the flank, on one side.
pub const SHED: u8 = 8;
/// The head down, scouring a lane of rubble ahead.
pub const PLOUGH: u8 = 9;
/// The shell rolls away from one side and back: the rim throws.
pub const SHRUG: u8 = 10;
/// The plates round the anchors lift and rattle: the crown throws.
pub const SHIVER: u8 = 11;
/// At the siege line: light through the anchors and the head, into the wall.
pub const BEAM: u8 = 12;

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
    MoveDecl::new("Footfall", Clip::Walk as usize)
        .own_hit()
        .unanimated(),
    MoveDecl::new("Stamp", Clip::Walk as usize)
        .own_hit()
        .unanimated(),
    MoveDecl::new("Ankle drag", Clip::Walk as usize)
        .own_hit()
        .unanimated(),
    MoveDecl::new("Shed", Clip::Shed as usize).own_hit(),
    MoveDecl::new("Plough", Clip::Plough as usize).own_hit(),
    MoveDecl::new("Shrug", Clip::Shrug as usize).scaled_by(Knob::ShrugForce as u16),
    MoveDecl::new("Shiver", Clip::Shiver as usize).scaled_by(Knob::ShiverForce as u16),
    MoveDecl::new("Siege beam", Clip::Beam as usize).own_hit(),
];

// ---------------------------------------------------------------------------
// Clips
// ---------------------------------------------------------------------------

/// Everything the Siegeshell knows how to look like, in the order its baked
/// table lays them out. Authored in `crates/anim/src/beast/siegeshell/`. The
/// legs are never in a clip: [`gait::repose`] places every foot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clip {
    Idle,
    /// The shell's sway and the head's nod over one full gait cycle, by
    /// ground covered.
    Walk,
    /// Hurried: the same, a little lower and heavier.
    Hurry,
    Shed,
    Plough,
    Shrug,
    Shiver,
    Beam,
    Flinch,
    /// A side going down: the shell's half of it (the legs are the gait's).
    Stumble,
    /// An anchor broken: the shell settling and the head down.
    Kneel,
    Dead,
}

pub const CLIP_COUNT: usize = 12;

impl Clip {
    pub const ALL: [Clip; CLIP_COUNT] = [
        Clip::Idle,
        Clip::Walk,
        Clip::Hurry,
        Clip::Shed,
        Clip::Plough,
        Clip::Shrug,
        Clip::Shiver,
        Clip::Beam,
        Clip::Flinch,
        Clip::Stumble,
        Clip::Kneel,
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
    cycle("hurry"),
    attack("shed"),
    attack("plough"),
    attack("shrug"),
    attack("shiver"),
    attack("beam"),
    once("flinch"),
    once("stumble"),
    once("kneel"),
    once("dead"),
];

// ---------------------------------------------------------------------------
// Its own knobs
// ---------------------------------------------------------------------------

crate::species_knobs! {
    /// The knobs only the Siegeshell has. The design document's names for
    /// them are in the comment beside each, where they differ. **Append
    /// only**: the baked file is read by position.
    pub enum Knob;
    // shell_armour
    VulnShell,       "hide",    "Shell (x damage)",                      Fixed, 0, fx(3,1);
    VulnAnkle,       "hide",    "Ankles (x damage)",                     Fixed, 0, fx(3,1);
    VulnAnchor,      "hide",    "Anchors (x damage)",                    Fixed, 0, fx(3,1);
    VulnLimb,        "hide",    "Legs, head and pads (x damage)",        Fixed, 0, fx(3,1);
    AnchorHealth,    "hide",    "An anchor's health",                    Int,   0, 6000;
    // buckle_health
    BuckleHealth,    "hide",    "A broken ankle buckles every",          Int,   0, 2000;
    // opening_damage
    OpenDamage,      "hide",    "An open anchor takes (x)",              Fixed, 0, fx(5,1);
    // limp_per_ankle
    Limp,            "walk",    "Walk per broken ankle (x)",             Fixed, 0, fx(1,1);
    Hurry,           "walk",    "Hurried, walks (x)",                    Fixed, 0, fx(2,1);
    // path_pull
    PathAhead,       "walk",    "Steers at the centre line this far ahead", Fixed, 0, fx(60,1);
    SiegeLine,       "walk",    "Halts with its head this far from the wall", Fixed, 0, fx(60,1);
    StartAt,         "walk",    "Starts this far down the valley",       Fixed, 0, fx(60,1);
    // the gait
    Lift,            "gait",    "A foot rises to",                       Fixed, 0, fx(6,1);
    LiftShare,       "gait",    "A foot is in the air for (share of a cycle)", Fixed, 0, fx(1,2);
    Bob,             "gait",    "The shell bobs on the beat by",         Fixed, 0, fx(1,1);
    Sag,             "gait",    "A side sits lower per broken ankle by", Fixed, 0, fx(3,1);
    // the footfall
    PadRadius,       "footfall", "The pad, radius",                      Fixed, 0, fx(4,1);
    PadDamage,       "footfall", "Under a pad as it lands",              Int,   0, 900;
    RingFrom,        "footfall", "The ring starts at",                   Fixed, 0, fx(6,1);
    RingTo,          "footfall", "The ring reaches",                     Fixed, 0, fx(12,1);
    RingFrames,      "footfall", "The ring sweeps out over",             Frames, 0, 60;
    RingHeight,      "footfall", "The ring stands",                      Fixed, 0, fx(2,1);
    // the stamp
    StampTracking,   "stamp",   "The lifted foot follows at",            Fixed, 0, fx(10,1);
    StampLockout,    "stamp",   "A leg stamps again after",              Frames, 0, 1800;
    StampLift,       "stamp",   "A stamping foot rises to",              Fixed, 0, fx(12,1);
    StampReach,      "stamp",   "Stamps at somebody this near a foot",   Fixed, 0, fx(12,1);
    // cling_frames
    ClingFrames,     "drag",    "Drags a leg clung to for",              Frames, 0, 600;
    ClingRadius,     "drag",    "Clinging is within, of an ankle",       Fixed, 0, fx(8,1);
    DragArc,         "drag",    "The drag sweeps",                       Fixed, 0, fx(12,1);
    // the stumble
    StumbleDrop,     "stumble", "A stumble drops the shell by",          Fixed, 0, fx(12,1);
    StumbleRoll,     "stumble", "A stumble rolls it toward the side (turns)", Fixed, 0, fx(1,8);
    Settle,          "stumble", "Goes down over",                        Frames, 0, 120;
    StairKnee,       "stumble", "A splayed knee's top stands",           Fixed, 0, fx(6,1);
    // the kneel
    KneelDrop,       "kneel",   "A kneel drops the shell by",            Fixed, 0, fx(12,1);
    KneelSettle,     "kneel",   "Kneels over",                           Frames, 0, 240;
    // the anchors and the Opening
    OpenRadius,      "crown",   "Opens an anchor with somebody within",  Fixed, 0, fx(8,1);
    // the vents
    VentCycle,       "vents",   "A grate blows every, walking",          Frames, 0, 900;
    VentCycleRoused, "vents",   "A grate blows every, roused",           Frames, 0, 900;
    VentHiss,        "vents",   "Hisses for",                            Frames, 0, 240;
    VentBlast,       "vents",   "Blows for",                             Frames, 0, 240;
    VentRadius,      "vents",   "A grate, radius",                       Fixed, 0, fx(4,1);
    // the shed
    ShedPlatesFew,   "shed",    "Plates shed, fewest",                   Int,   0, 12;
    ShedPlatesMost,  "shed",    "Plates shed, most",                     Int,   0, 12;
    ShedNear,        "shed",    "Plates land outside the feet from",     Fixed, 0, fx(20,1);
    ShedFar,         "shed",    "Plates land outside the feet to",       Fixed, 0, fx(30,1);
    ShedPlate,       "shed",    "A plate's circle, radius",              Fixed, 0, fx(4,1);
    // the plough
    PloughSpeed,     "plough",  "Rubble flies at",                       Fixed, 0, fx(90,1);
    PloughFrom,      "plough",  "The lane starts ahead at",              Fixed, 0, fx(60,1);
    PloughTo,        "plough",  "The lane ends ahead at",                Fixed, 0, fx(90,1);
    PloughHalf,      "plough",  "The lane, half-width",                  Fixed, 0, fx(10,1);
    // the shrug and the shiver
    ShrugForce,      "aboard",  "The shrug's violence (x clip)",         Fixed, 0, fx(3,1);
    ShiverForce,     "aboard",  "The shiver's violence (x clip)",        Fixed, 0, fx(3,1);
    // the siege
    BreachShare,     "siege",   "A beam takes this share of the wall",   Fixed, 0, fx(1,1);
    // phases
    LockoutWalking,  "phase",   "Walking, Shed and Plough lock out (x)", Fixed, 0, fx(8,1);
    ShiverHurried,   "phase",   "Hurried, the shiver locks out for",     Frames, 0, 900;
    // the brain
    ChannelGap,      "mind",    "Channel gap",                           Frames, 0, 120;
    StampAppetite,   "mind",    "Stamp, somebody under a foot",          Int,   0, 8000;
    DragAppetite,    "mind",    "Drag, somebody clinging",               Int,   0, 8000;
    RimAppetite,     "mind",    "Shrug, per rider on the rim or flank",  Int,   0, 8000;
    CrownAppetite,   "mind",    "Shiver, per rider at the crown",        Int,   0, 8000;
    FlankAppetite,   "mind",    "Shed, somebody on the flank",           Int,   0, 8000;
    AheadAppetite,   "mind",    "Plough, somebody in the lane ahead",    Int,   0, 8000;
    // the parasites: roost_frames, straggler_frames, brood_frames
    RoostFrames,     "parasites", "Drops on somebody under the belly after", Frames, 0, 900;
    StragglerFrames, "parasites", "Drops on a straggler after",          Frames, 0, 900;
    Straggler,       "parasites", "A straggler is this far behind the tail", Fixed, 0, fx(60,1);
    BroodFrames,     "parasites", "A crevice refills one every",         Frames, 0, 3600;
    BroodCap,        "parasites", "Parasites on the shell at most",      Int,   0, 10;
    BroodAtStart,    "parasites", "Parasites when the hunt begins",      Int,   0, 10;
    // the legs' channel, after a move
    FootHome,        "stamp",   "A foot goes home over",                 Frames, 0, 120;
    StampChance,     "stamp",   "Stamps on a beat it can (%)",           Percent, 0, 100;
    LegRest,         "stamp",   "The legs rest after a move for",        Frames, 0, 600;
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
    id: SpeciesId::SIEGESHELL,
    name: "Siegeshell",
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
        gallop: Clip::Hurry as usize,
        flinch: Clip::Flinch as usize,
        stumble: Clip::Stumble as usize,
        topple: Clip::Kneel as usize,
        dead: Clip::Dead as usize,
    },
    span: &baked::SPAN,
    rows: baked::ROWS,
    row: |r| &baked::FRAMES[r],
    own: OWN,
    tuned: &tuned::KNOBS,
    tuned_path: "crates/sim/src/species/siegeshell/tuned.rs",
    pack: Some(&mind::PACK),
    fight: &fight::FIGHT,
};
