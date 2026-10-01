//! Every animation the Mireback has, as poses and timing.
//!
//! Baked into `crates/sim/src/species/mireback/baked.rs` by
//! `cargo run -p anim --bin bake_beast -- --species mireback`.
//!
//! The house style is the Ridgeback's: **the windup is the move** (most of
//! the frames before the hit, the silhouette changing early and a lot), **the
//! hit is short and enormous**, and **the recovery is the fight**. A toad has
//! almost no neck and a body that is one dome, so its silhouette changes come
//! from the whole body: it sinks for the flop, rears its head for the spew,
//! drops its jaw for the tongue, hangs its throat out for the belch, lifts its
//! rump for the backwash, swells for the inflate and rolls over for the
//! wallow. A player across the arena reads those from the shape of the dome.
//!
//! Two of its states are written against the rules that matter for a climb:
//! the flop's recovery is **pressed flat** -- the rim down 0.6 m, inside a
//! standing hop for the Elementalist, the Reaver and the Dual mage -- and
//! **winded** sags the rim to 3.8 m. `beastcheck` prints both.

use super::{FORE_REACH, HIND_REACH, MirebackPose, mark};
use crate::beast::{Key, Looseness, Pose, Recipe};
use crate::ease::Ease;
use sim::species::mireback::Clip;
use std::f32::consts::TAU;

pub fn all() -> Vec<Recipe> {
    vec![
        idle(),
        walk(),
        spew(),
        flop(),
        tongue(),
        swallow(),
        belch(),
        backwash(),
        inflate(),
        wallow(),
        gag(),
        flinch(),
        winded(),
        gutted(),
        dead(),
    ]
}

// ---------------------------------------------------------------------------
// Standing and moving
// ---------------------------------------------------------------------------

fn idle() -> Recipe {
    // Breathing through the throat, the way a toad does: the sac pulses and
    // the dome rises a hand's width with it.
    let breathe = |k: f32| {
        Pose::rest(sim::species::mireback::bones::COUNT)
            .hips(0.0, 0.03 * k, 0.0)
            .root(0.6 * k, 0.0, 0.0)
            .head(1.0 * k, 0.0, 0.0)
            .sac(3.0 + 3.0 * k)
            .planted()
    };
    Recipe::new(
        Clip::Idle,
        vec![
            Key::at(0.0, breathe(-1.0)),
            Key::at(0.5, breathe(1.0)),
            Key::at(0.8, breathe(0.2)),
        ],
        Looseness::BEAST,
    )
    .noting(
        "Breathing through the throat. The sac pulses and the dome rises a \
         hand's width: a creature this still between moves still has to read \
         as alive."
            .to_string(),
    )
}

/// One leg at a point in its own cycle: planted for most of it, the body
/// travelling over the foot, then a short low swing forward.
fn stride(p: Pose, which: usize, u: f32, home: f32, reach: f32, lift: f32) -> Pose {
    const STANCE: f32 = 0.7;
    let u = u.rem_euclid(1.0);
    if u < STANCE {
        let k = u / STANCE;
        p.plant(which, home + reach * (1.0 - 2.0 * k), 0.0)
    } else {
        let k = (u - STANCE) / (1.0 - STANCE);
        let arc = (k * std::f32::consts::PI).sin();
        p.plant(which, home + reach * (2.0 * k - 1.0), lift * arc)
    }
}

fn walk() -> Recipe {
    // A waddle: a heavy four-beat with the dome rolling over each planted
    // side. Three feet down at all times; the roll is what makes it a toad
    // and not a table.
    const OFFSET: [f32; 4] = [0.5, 0.0, 0.75, 0.25];
    let at = |t: f32| {
        let roll = 3.5 * (t * TAU).sin();
        let bob = -0.06 * (2.0 * (t * TAU).sin().abs());
        let mut p = Pose::rest(sim::species::mireback::bones::COUNT)
            .hips(0.0, bob, 0.0)
            .root(0.8 * (t * TAU * 2.0).cos(), 0.0, roll)
            .head(-1.0, -roll * 0.6, -roll * 0.4)
            .sac(3.0);
        for (which, offset) in OFFSET.into_iter().enumerate() {
            let home = if which < 2 { FORE_REACH } else { HIND_REACH };
            p = stride(p, which, t + offset, home, 0.55, 0.35);
        }
        p
    };
    Recipe::new(
        Clip::Walk,
        (0..8)
            .map(|i| {
                let t = i as f32 / 8.0;
                Key::eased(t, at(t), Ease::LINEAR)
            })
            .collect(),
        Looseness::GAIT,
    )
    .noting(
        "A waddle at two metres a second, rolling over each planted side. It \
         has no faster gait: it relocates by leaping."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The moves
// ---------------------------------------------------------------------------

fn spew() -> Recipe {
    let ready = Pose::standing();
    // Cheeks full and head tipped back: the dome rears at the front, which
    // reads from anywhere in the arena as "something is coming out of that".
    let loaded = Pose::standing()
        .hips(-0.1, 0.25, 0.0)
        .root(3.0, 0.0, 0.0)
        .head(24.0, 0.0, 0.0)
        .sac(14.0)
        .jaw(0.0)
        .plant_fore(FORE_REACH - 0.2, 0.0)
        .plant_hind(HIND_REACH, 0.0);
    let thrown = Pose::standing()
        .hips(0.15, 0.1, 0.0)
        .root(-2.0, 0.0, 0.0)
        .head(-14.0, 0.0, 0.0)
        .jaw(28.0)
        .sac(2.0)
        .plant_fore(FORE_REACH + 0.1, 0.0)
        .plant_hind(HIND_REACH, 0.0);
    Recipe::new(
        Clip::Spew,
        vec![
            Key::eased(0.0, ready, Ease::ANTICIPATE),
            Key::eased(mark(Clip::Spew, 0, 0.6), loaded, Ease::SNAP),
            Key::eased(mark(Clip::Spew, 1, 0.0), thrown, Ease::STRIKE),
            Key::eased(mark(Clip::Spew, 2, 0.3), thrown, Ease::OUT),
            Key::eased(mark(Clip::Spew, 2, 0.75), ready, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "Cheeks fill and the head tips back for most of the startup, then the \
         glob is thrown with the whole front of the dome. The landing is marked \
         on the floor from the commit; this is what tells you to look for it."
            .to_string(),
    )
}

fn flop() -> Recipe {
    let ready = Pose::standing();
    // Sunk: the whole dome down onto its folded legs, the floor rippling round
    // it. Held -- this is the long tell.
    let sunk = Pose::standing()
        .hips(0.0, -0.7, 0.0)
        .head(-6.0, 0.0, 0.0)
        .spread_all(25.0)
        .plant_fore(FORE_REACH + 0.2, 0.0)
        .plant_hind(HIND_REACH + 0.2, 0.0);
    // In the air (the simulation carries it there): legs thrown out behind
    // and in front, the dome flattening for the landing.
    let flying = Pose::standing()
        .hips(0.0, 0.1, 0.0)
        .root(4.0, 0.0, 0.0)
        .head(6.0, 0.0, 0.0)
        .leg(0, 55.0, -30.0)
        .leg(1, 55.0, -30.0)
        .leg(2, -70.0, 20.0)
        .leg(3, -70.0, 20.0);
    // The crash: pressed flat, legs splayed -- and it stays down for most of
    // the recovery, the rim within a standing hop for the longer jumpers.
    let pressed = Pose::standing()
        .hips(0.0, -0.6, 0.0)
        .root(-1.0, 0.0, 0.0)
        .head(-8.0, 0.0, 0.0)
        .jaw(10.0)
        .spread_all(45.0)
        .plant_fore(FORE_REACH + 0.5, 0.0)
        .plant_hind(HIND_REACH + 0.4, 0.0);
    let air = 20.0 / 50.0;
    Recipe::new(
        Clip::Flop,
        vec![
            Key::eased(0.0, ready, Ease::IN),
            Key::eased(mark(Clip::Flop, 0, 0.45), sunk, Ease::HOLD),
            Key::eased(mark(Clip::Flop, 0, 1.0 - air), sunk, Ease::SNAP),
            Key::eased(mark(Clip::Flop, 0, 1.0 - air * 0.5), flying, Ease::IN),
            Key::eased(mark(Clip::Flop, 1, 0.0), pressed, Ease::OUT),
            Key::eased(mark(Clip::Flop, 2, 0.7), pressed, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "Sink, leap, crash, and stay down. The sink is held for most of the \
         fifty frames -- the toad flat on the floor is the tell -- and the leap \
         is the last twenty, carried by the simulation. Pressed flat after the \
         crash, its rim comes down within a standing hop for the Elementalist, \
         the Reaver and the Dual mage: the window a flop opens."
            .to_string(),
    )
}

fn tongue() -> Recipe {
    let ready = Pose::standing();
    // The throat pulses and the jaw drops: a croak, then a mouth a metre
    // open pointed at the floor ahead.
    let pulse = Pose::standing().head(4.0, 0.0, 0.0).sac(16.0).jaw(6.0);
    let open = Pose::standing()
        .hips(0.05, 0.05, 0.0)
        .root(-2.0, 0.0, 0.0)
        .head(-8.0, 0.0, 0.0)
        .jaw(38.0)
        .sac(4.0)
        .tongue(-10.0)
        .planted();
    let out = Pose::standing()
        .hips(0.0, 0.05, 0.0)
        .root(-2.0, 0.0, 0.0)
        .head(-10.0, 0.0, 0.0)
        .jaw(40.0)
        .tongue(-4.0)
        .planted();
    Recipe::new(
        Clip::Tongue,
        vec![
            Key::eased(0.0, ready, Ease::SMOOTH),
            Key::eased(mark(Clip::Tongue, 0, 0.4), pulse, Ease::SNAP),
            Key::eased(mark(Clip::Tongue, 0, 0.9), open, Ease::STRIKE),
            Key::eased(mark(Clip::Tongue, 1, 0.5), out, Ease::OUT),
            Key::eased(
                mark(Clip::Tongue, 2, 0.6),
                out.blend(&ready, 0.5),
                Ease::SMOOTH,
            ),
            Key::at(1.0, ready),
        ],
        Looseness::SNAP,
    )
    .noting(
        "A croak -- the throat pulses -- then the jaw drops and the tongue goes \
         out along its line at eighty metres a second. The line is drawn from \
         the hit test, stopping at the first solid; the clip's job is the \
         mouth, which is what a player sees turn toward them."
            .to_string(),
    )
}

fn swallow() -> Recipe {
    // Somebody inside. Gulping: the throat working and the dome heaving, so a
    // partner outside can see it is busy and where to hit.
    let gulp = |k: f32| {
        Pose::standing()
            .hips(0.0, 0.06 + 0.06 * k, 0.0)
            .root(0.5 * k, 0.0, 1.0 * k)
            .head(3.0 * k, 0.0, 0.0)
            .sac(10.0 + 8.0 * k)
            .planted()
    };
    let spit = Pose::standing()
        .hips(0.2, 0.1, 0.0)
        .root(-2.0, 0.0, 0.0)
        .head(-12.0, 0.0, 0.0)
        .jaw(42.0)
        .sac(2.0)
        .planted();
    let mut keys = vec![Key::eased(0.0, gulp(-1.0), Ease::SMOOTH)];
    for i in 1..8 {
        let u = i as f32 / 8.0;
        let k = if i % 2 == 0 { -1.0 } else { 1.0 };
        keys.push(Key::eased(mark(Clip::Swallow, 1, u), gulp(k), Ease::SMOOTH));
    }
    keys.push(Key::eased(
        mark(Clip::Swallow, 1, 1.0),
        gulp(0.0),
        Ease::SNAP,
    ));
    keys.push(Key::eased(mark(Clip::Swallow, 2, 0.25), spit, Ease::OUT));
    keys.push(Key::at(1.0, Pose::standing()));
    Recipe::new(Clip::Swallow, keys, Looseness::BEAST).noting(
        "Gulping, two seconds of it, and a spit. The throat and the dome heave \
         in turn; the belly bulging where a hit lands is the renderer's."
            .to_string(),
    )
}

fn belch() -> Recipe {
    let ready = Pose::standing();
    // The throat lights and swells -- the sac hangs right out -- and the head
    // comes up over it. A minute of difference from anything else it does,
    // and the one move whose tell is drawn on every pool it will light.
    let swollen = Pose::standing()
        .hips(0.0, 0.25, 0.0)
        .root(3.0, 0.0, 0.0)
        .head(18.0, 0.0, 0.0)
        .sac(42.0)
        .jaw(4.0)
        .planted();
    let blown = Pose::standing()
        .hips(0.2, 0.1, 0.0)
        .root(-2.0, 0.0, 0.0)
        .head(-10.0, 0.0, 0.0)
        .jaw(44.0)
        .sac(20.0)
        .planted();
    Recipe::new(
        Clip::Belch,
        vec![
            Key::eased(0.0, ready, Ease::IN),
            Key::eased(mark(Clip::Belch, 0, 0.5), swollen, Ease::HOLD),
            Key::eased(mark(Clip::Belch, 0, 0.9), swollen, Ease::SNAP),
            Key::eased(mark(Clip::Belch, 1, 0.0), blown, Ease::STRIKE),
            Key::eased(mark(Clip::Belch, 2, 0.4), blown, Ease::OUT),
            Key::at(1.0, ready),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "Sixty frames of a swelling throat, the sac out where a blow can reach \
         it: hit it hard enough and the spark goes off in its own mouth."
            .to_string(),
    )
}

fn backwash() -> Recipe {
    let ready = Pose::standing();
    // The hind end lifts and the hind feet gather under it: the rump is
    // where the person this is aimed at is looking.
    let lifted = Pose::standing()
        .hips(0.0, 0.2, 0.0)
        .root(-4.0, 0.0, 0.0)
        .head(8.0, 0.0, 0.0)
        .plant_fore(FORE_REACH, 0.0)
        .plant_hind(HIND_REACH + 0.4, 0.0);
    // Churned: the hind feet thrown back, a low sheet of tar behind.
    let churned = Pose::standing()
        .hips(0.1, 0.1, 0.0)
        .root(-4.0, 0.0, 0.0)
        .plant_fore(FORE_REACH, 0.0)
        .leg(2, -60.0, 20.0)
        .leg(3, -60.0, 20.0);
    Recipe::new(
        Clip::Backwash,
        vec![
            Key::eased(0.0, ready, Ease::ANTICIPATE),
            Key::eased(mark(Clip::Backwash, 0, 0.7), lifted, Ease::SNAP),
            Key::eased(mark(Clip::Backwash, 1, 0.2), churned, Ease::OUT),
            Key::eased(
                mark(Clip::Backwash, 1, 0.6),
                lifted.blend(&churned, 0.4),
                Ease::OUT,
            ),
            Key::eased(mark(Clip::Backwash, 1, 1.0), churned, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        Looseness::SNAP,
    )
    .noting(
        "The rump lifts and the hind feet churn. Shin-high and only ever thrown \
         at somebody behind it: jump it."
            .to_string(),
    )
}

fn inflate() -> Recipe {
    let ready = Pose::standing();
    // Swelling: the whole dome rises on splayed legs. The renderer rounds it
    // out by half; the shove is the volume the hit test uses.
    let swelling = Pose::standing()
        .hips(0.0, 0.35, 0.0)
        .root(2.0, 0.0, 0.0)
        .sac(20.0)
        .spread_all(20.0)
        .planted();
    // The pop: a jolt up and out that throws an unbraced rider.
    let popped = Pose::standing()
        .hips(0.0, 0.95, 0.0)
        .root(-3.0, 0.0, 0.0)
        .sac(34.0)
        .spread_all(36.0)
        .planted();
    let puffed = Pose::standing()
        .hips(0.0, 0.7, 0.0)
        .sac(30.0)
        .spread_all(30.0)
        .planted();
    Recipe::new(
        Clip::Inflate,
        vec![
            Key::eased(0.0, ready, Ease::IN),
            Key::eased(mark(Clip::Inflate, 0, 0.85), swelling, Ease::SNAP),
            Key::eased(mark(Clip::Inflate, 1, 0.3), popped, Ease::OUT),
            Key::eased(mark(Clip::Inflate, 2, 0.1), puffed, Ease::HOLD),
            Key::eased(mark(Clip::Inflate, 2, 0.75), puffed, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        Looseness::SNAP,
    )
    .noting(
        "Swell, pop, and stay puffed with the sac out. The pop is the buck: a \
         braced rider rides it and an unbraced one is thrown. The ninety frames \
         puffed are the sac's window."
            .to_string(),
    )
}

fn wallow() -> Recipe {
    let ready = Pose::standing();
    let grind = |k: f32| Pose::belly_up(30.0 * k).root(0.0, 10.0 * k, 180.0 + 8.0 * k);
    let mut keys = vec![
        Key::eased(0.0, ready, Ease::IN),
        Key::eased(
            mark(Clip::Wallow, 0, 0.5),
            Pose::standing().root(0.0, 0.0, 70.0).hips(0.0, 1.2, 0.0),
            Ease::LINEAR,
        ),
        Key::eased(mark(Clip::Wallow, 1, 0.0), grind(-1.0), Ease::SMOOTH),
    ];
    for i in 1..6 {
        let k = if i % 2 == 0 { -1.0 } else { 1.0 };
        keys.push(Key::eased(
            mark(Clip::Wallow, 1, i as f32 / 6.0),
            grind(k),
            Ease::SMOOTH,
        ));
    }
    keys.push(Key::eased(mark(Clip::Wallow, 1, 1.0), grind(0.0), Ease::IN));
    keys.push(Key::eased(
        mark(Clip::Wallow, 2, 0.5),
        Pose::standing().root(0.0, 0.0, 70.0).hips(0.0, 1.2, 0.0),
        Ease::SMOOTH,
    ));
    keys.push(Key::at(1.0, ready));
    Recipe::new(Clip::Wallow, keys, Looseness::HEAVY).noting(
        "Over onto its back in its own tar, grinding, legs flailing. Rolling \
         over throws everybody aboard -- leave before it. Belly up, it is \
         soaked: set the pools under it alight now and it is gutted."
            .to_string(),
    )
}

fn gag() -> Recipe {
    let shake = |k: f32| {
        Pose::standing()
            .head(-6.0, 16.0 * k, 4.0 * k)
            .jaw(32.0)
            .sac(30.0)
            .root(0.0, 3.0 * k, 0.0)
    };
    let mut keys = vec![Key::eased(0.0, shake(0.0), Ease::OUT)];
    for i in 1..6 {
        let k = if i % 2 == 0 { -1.0 } else { 1.0 };
        keys.push(Key::eased(i as f32 / 7.0, shake(k), Ease::SMOOTH));
    }
    keys.push(Key::at(1.0, Pose::standing()));
    Recipe::new(Clip::Gag, keys, Looseness::SNAP)
        .noting("A shield in its throat: the head shaking, the jaw wide, the sac out.".to_string())
}

// ---------------------------------------------------------------------------
// Being in trouble
// ---------------------------------------------------------------------------

fn flinch() -> Recipe {
    let hit = Pose::standing()
        .hips(-0.1, -0.05, 0.0)
        .root(5.0, -3.0, 4.0)
        .head(-10.0, -6.0, 0.0)
        .jaw(14.0)
        .planted();
    Recipe::new(
        Clip::Flinch,
        vec![
            Key::eased(0.0, hit, Ease::OUT),
            Key::eased(0.45, hit.blend(&Pose::standing(), 0.5), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::LIMP,
    )
    .noting(
        "Staggered: a backfire, a broken wallow, a wart bursting. Held at the \
         start and bled off."
            .to_string(),
    )
}

fn winded() -> Recipe {
    // The sac torn: it sags, the rim down to 3.8 m, the throat hanging out.
    let sagged = Pose::standing()
        .hips(0.0, -1.4, 0.0)
        .head(-12.0, 0.0, 0.0)
        .jaw(20.0)
        .sac(36.0)
        .spread_all(55.0)
        .plant_fore(FORE_REACH + 0.6, 0.0)
        .plant_hind(HIND_REACH + 0.5, 0.0);
    Recipe::new(
        Clip::Winded,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::OUT),
            Key::eased(0.15, sagged, Ease::HOLD),
            Key::eased(0.8, sagged, Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::COLLAPSE,
    )
    .noting(
        "Winded. Sagged flat on splayed legs for most of the window, the rim \
         within a standing hop for all but the Bulwark."
            .to_string(),
    )
}

fn gutted() -> Recipe {
    let mut keys = vec![Key::eased(0.0, Pose::belly_up(0.0), Ease::OUT)];
    for i in 1..9 {
        let k = if i % 2 == 0 { -1.0 } else { 1.0 };
        keys.push(Key::eased(
            i as f32 / 10.0,
            Pose::belly_up(40.0 * k).root(0.0, 6.0 * k, 180.0 + 6.0 * k),
            Ease::SMOOTH,
        ));
    }
    keys.push(Key::eased(
        0.92,
        Pose::standing().root(0.0, 0.0, 70.0).hips(0.0, 1.2, 0.0),
        Ease::SMOOTH,
    ));
    keys.push(Key::at(1.0, Pose::standing()));
    Recipe::new(Clip::Gutted, keys, Looseness::LIMP).noting(
        "Gutted: on its back in its own fire, flailing without threat, the \
         whole hide soft. It rolls upright at the very end, coat gone and \
         ringed by the slag its own pools left."
            .to_string(),
    )
}

fn dead() -> Recipe {
    let down = Pose::standing()
        .hips(0.0, -1.3, 0.0)
        .root(-4.0, 0.0, 18.0)
        .head(-14.0, 0.0, 0.0)
        .jaw(30.0)
        .sac(30.0)
        .spread_all(60.0)
        .plant_fore(FORE_REACH + 0.8, 0.0)
        .plant_hind(HIND_REACH + 0.6, 0.0);
    Recipe::new(Clip::Dead, vec![Key::at(0.0, down)], Looseness::LIMP)
}
