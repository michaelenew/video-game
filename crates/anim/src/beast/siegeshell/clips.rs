//! Every animation the Siegeshell has, as poses and timing.
//!
//! Baked into `crates/sim/src/species/siegeshell/baked.rs` by
//! `cargo run -p anim --bin bake_beast -- --species siegeshell`.
//!
//! The house style is the Ridgeback's -- **the windup is the move**, **the hit
//! is short and enormous**, **the recovery is the fight** -- at a size where
//! the body is out of frame most of the time, so the silhouette is the
//! *shell's* outline against the sky and the head's against the valley:
//! one side of the shell rearing like a lid is the shrug; the plates round
//! the crown standing up is the shiver; the rim rattling and lifting on one
//! side is the shed; the head going all the way down to the floor is the
//! plough; the head up and the crown lifting, held for twenty seconds, is the
//! beam. The legs are the gait's, placed by the simulation; no clip moves
//! one.

use super::{ShellPose, mark};
use crate::beast::{Feel, Key, Looseness, Pose, Recipe};
use crate::ease::Ease;
use sim::species::siegeshell::{Clip, bones};
use std::f32::consts::TAU;

pub fn all() -> Vec<Recipe> {
    vec![
        idle(),
        walk(Clip::Walk, 1.0, 0.0),
        walk(Clip::Hurry, 1.4, 0.15),
        shed(),
        plough(),
        shrug(),
        shiver(),
        beam(),
        flinch(),
        stumble(),
        kneel(),
        dead(),
    ]
}

fn rest() -> Pose {
    Pose::rest(bones::COUNT)
}

/// A shell that weighs as much as a hill: everything slow to start and
/// damped nearly dead, so the motion is the keys and not the springs -- the
/// grip test reads the shell's motion, and a ringing spring on a forty-metre
/// body is a buck nobody authored.
const MASS: Looseness = Looseness {
    body: Feel::new(2.0, 0.98),
    neck: Feel::new(3.0, 0.8),
    tail: Feel::new(3.0, 0.9),
    legs: Feel::new(1.0, 1.0),
};

/// The shrug and the shiver: fast, but the springs still dead, so the throw
/// is the keys' and its knob scales exactly what was authored.
const WHIP: Looseness = Looseness {
    body: Feel::new(1.0, 1.0),
    neck: Feel::new(2.0, 0.85),
    tail: Feel::new(1.0, 1.0),
    legs: Feel::new(1.0, 1.0),
};

// ---------------------------------------------------------------------------
// Standing and moving
// ---------------------------------------------------------------------------

fn idle() -> Recipe {
    // It breathes through the shell: a hand's width up and down, and the
    // head nods. A hill at rest.
    let breathe = |k: f32| {
        rest()
            .hips(0.0, 0.06 * k, 0.0)
            .neck(-2.0 + 1.5 * k, 1.0 * k, 0.0)
    };
    Recipe::new(
        Clip::Idle,
        vec![Key::at(0.0, breathe(-1.0)), Key::at(0.5, breathe(1.0))],
        MASS,
    )
    .noting("Breathing through the shell; the head nods. The legs are the gait's.".to_string())
}

/// One gait cycle: two beats, each a settle of the shell onto the tripod that
/// lands and a sway toward its side, and the head nodding with it. `sway` is
/// degrees of roll; `low` how much lower the hurried shell carries.
fn walk(clip: Clip, sway: f32, low: f32) -> Recipe {
    let at = |t: f32| {
        // Two beats a cycle: the first tripod lands at nought, the second at
        // a half. The shell settles a hand's width onto each.
        let beat = (2.0 * t * TAU).cos();
        let roll = sway * (t * TAU).sin();
        rest()
            .hips(0.0, -low - 0.08 * (1.0 + beat) * 0.5, 0.0)
            .body(-0.3, 0.0, roll)
            .neck(-3.0 - 2.0 * beat, 1.0 * beat, -roll)
    };
    Recipe::new(
        clip,
        (0..16)
            .map(|i| {
                let t = i as f32 / 16.0;
                Key::eased(t, at(t), Ease::LINEAR)
            })
            .collect(),
        Looseness::GAIT,
    )
    .noting(
        "The shell settles onto each tripod as it lands and sways toward its \
         side, the head nodding on the beat. Small on purpose: the walk never \
         throws a rider, and on a forty-metre body a degree is a lot."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The body's moves
// ---------------------------------------------------------------------------

fn shed() -> Recipe {
    // The rim on the right stands up like a skirt and rattles; the plates go
    // on the hit; it settles. Mirrored to the left when the target is there.
    let m = |phase, u| mark(Clip::Shed, phase, u);
    let skirt = |k: f32| {
        rest()
            .side(true, 6.0 + k)
            .body(0.0, 0.0, -1.5)
            .neck(-4.0, 0.0, 8.0)
    };
    let mut keys = vec![Key::at(0.0, rest())];
    keys.push(Key::eased(m(0, 0.4), skirt(0.0), Ease::OUT));
    for i in 0..6 {
        let u = 0.4 + 0.6 * (i as f32 + 1.0) / 6.0;
        keys.push(Key::at(m(0, u), skirt(if i % 2 == 0 { 1.5 } else { -1.5 })));
    }
    keys.push(Key::eased(
        m(1, 0.3),
        rest().side(true, 9.0).body(0.0, 0.0, -2.0),
        Ease::OUT,
    ));
    keys.push(Key::eased(m(2, 0.5), rest().side(true, 2.0), Ease::SMOOTH));
    keys.push(Key::eased(1.0, rest(), Ease::SMOOTH));
    Recipe::new(Clip::Shed, keys, MASS).noting(
        "One side's rim stands up like a skirt and rattles through the windup -- \
         the plates loosening -- and lifts as they go. The answer is under it."
            .to_string(),
    )
}

fn plough() -> Recipe {
    // The head goes all the way down to the valley floor, and scours forward;
    // the shell tips nose-down with it.
    let m = |phase, u| mark(Clip::Plough, phase, u);
    let low = rest().body(-4.0, 0.0, 0.0).neck(-28.0, -18.0, 0.0);
    let scour = rest()
        .hips(0.6, -0.3, 0.0)
        .body(-6.0, 0.0, 0.0)
        .neck(-32.0, -10.0, 0.0);
    Recipe::new(
        Clip::Plough,
        vec![
            Key::at(0.0, rest()),
            Key::eased(m(0, 0.6), low, Ease::OUT),
            Key::at(m(0, 1.0), low),
            Key::eased(m(1, 0.5), scour, Ease::OUT),
            Key::at(m(2, 0.4), scour),
            Key::eased(1.0, rest(), Ease::SMOOTH),
        ],
        MASS,
    )
    .noting(
        "The head down to the floor and the shell nose-down: from anywhere ahead \
         of it, the head disappearing is the tell. The lane is drawn on the floor."
            .to_string(),
    )
}

fn shrug() -> Recipe {
    // The right side of the shell rears like a lid -- slowly, grinding -- and
    // then throws: up, eighteen degrees, and back, in the active window.
    let m = |phase, u| mark(Clip::Shrug, phase, u);
    Recipe::new(
        Clip::Shrug,
        vec![
            Key::at(0.0, rest()),
            Key::eased(
                m(0, 0.8),
                rest().side(true, 3.0).body(0.0, 0.0, -1.0),
                Ease::OUT,
            ),
            Key::at(m(0, 1.0), rest().side(true, 3.5).body(0.0, 0.0, -1.0)),
            Key::eased(
                m(1, 0.5),
                rest().side(true, 18.0).body(0.0, 0.0, -2.0),
                Ease::OUT,
            ),
            Key::eased(m(1, 1.0), rest().side(true, 1.0), Ease::IN),
            Key::eased(1.0, rest(), Ease::SMOOTH),
        ],
        WHIP,
    )
    .noting(
        "The tell is the lid rising a few degrees and grinding there; the throw \
         is eighteen degrees up and back inside the active frames. A rider on \
         that side's rim or flank is thrown loose and holds braced; the plateau \
         and the crown are on the root, and do not move."
            .to_string(),
    )
}

fn shiver() -> Recipe {
    // A rising hum, the crown's plates lifting a little; then the hackles
    // rattle: eight reversals in the active window, too fast to brace through.
    let m = |phase, u| mark(Clip::Shiver, phase, u);
    let mut keys = vec![
        Key::at(0.0, rest()),
        Key::eased(m(0, 0.9), rest().crown(1.0, 0.0), Ease::OUT),
        Key::at(m(0, 1.0), rest().crown(1.0, 0.0)),
    ];
    for i in 0..8 {
        let u = (i as f32 + 1.0) / 9.0;
        let k = if i % 2 == 0 { 1.0 } else { -1.0 };
        keys.push(Key::at(m(1, u), rest().crown(1.0 + 5.0 * k, 8.0 * k)));
    }
    keys.push(Key::eased(m(1, 1.0), rest().crown(0.5, 0.0), Ease::OUT));
    keys.push(Key::eased(1.0, rest(), Ease::SMOOTH));
    Recipe::new(Clip::Shiver, keys, WHIP).noting(
        "The hum is a slow lift of the crown's plates; the shiver is eight \
         reversals inside the whip, more than a brace holds. Jump it late in \
         the tell."
            .to_string(),
    )
}

fn beam() -> Recipe {
    // The head comes up and the crown lifts, and it holds there, trembling a
    // little, for twenty seconds; the beam; then it settles.
    let m = |phase, u| mark(Clip::Beam, phase, u);
    let up = |k: f32| {
        rest()
            .hips(0.0, 0.3, 0.0)
            .body(2.0, 0.0, 0.0)
            .crown(2.0 + k, 0.0)
            .neck(14.0, 6.0, 0.0)
    };
    let mut keys = vec![
        Key::at(0.0, rest()),
        Key::eased(m(0, 0.1), up(0.0), Ease::OUT),
    ];
    for i in 0..8 {
        let u = 0.1 + 0.9 * (i as f32 + 1.0) / 8.0;
        keys.push(Key::at(m(0, u), up(if i % 2 == 0 { 0.4 } else { -0.4 })));
    }
    keys.push(Key::at(m(1, 1.0), up(1.0)));
    keys.push(Key::eased(1.0, rest(), Ease::SMOOTH));
    Recipe::new(Clip::Beam, keys, MASS).noting(
        "Head up and crown raised, held trembling through the twenty-second \
         charge: the siege's tell, read from anywhere in the valley."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The ladder
// ---------------------------------------------------------------------------

fn flinch() -> Recipe {
    Recipe::new(
        Clip::Flinch,
        vec![
            Key::at(0.0, rest()),
            Key::eased(0.3, rest().neck(-6.0, 4.0, 0.0), Ease::OUT),
            Key::eased(1.0, rest(), Ease::SMOOTH),
        ],
        MASS,
    )
}

fn stumble() -> Recipe {
    // The shell's half of a side going down: the head drops and swings. The
    // lowering and the roll are the simulation's, toward whichever side.
    Recipe::new(
        Clip::Stumble,
        vec![
            Key::at(0.0, rest()),
            Key::eased(0.06, rest().neck(-14.0, -10.0, 0.0), Ease::OUT),
            Key::at(0.9, rest().neck(-12.0, -8.0, 0.0)),
            Key::eased(1.0, rest(), Ease::SMOOTH),
        ],
        Looseness::LIMP,
    )
    .noting(
        "The head drops; the side going down is the gait's lowering, toward its side.".to_string(),
    )
}

fn kneel() -> Recipe {
    Recipe::new(
        Clip::Kneel,
        vec![
            Key::at(0.0, rest()),
            Key::eased(0.08, rest().neck(-20.0, -14.0, 0.0), Ease::OUT),
            Key::at(0.92, rest().neck(-18.0, -12.0, 0.0)),
            Key::eased(1.0, rest(), Ease::SMOOTH),
        ],
        Looseness::LIMP,
    )
    .noting(
        "An anchor broken: the head down on the floor; the shell's settling is the gait's."
            .to_string(),
    )
}

fn dead() -> Recipe {
    Recipe::new(
        Clip::Dead,
        vec![
            Key::at(0.0, rest()),
            Key::eased(1.0, rest().neck(-30.0, -10.0, 0.0), Ease::OUT),
        ],
        Looseness::LIMP,
    )
    .noting("It settles into the valley as a hill, the head down.".to_string())
}
