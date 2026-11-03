//! **The Warden**: the opening's creature (`docs/design/opening.md` §6), at
//! the bottom of the keep at the end of the way out (`arena::warden`).
//!
//! **Its body is the Ridgeback's, for now.** Its skeleton, its parts and
//! their flags, its legs, its clips and their baked poses are the
//! Ridgeback's tables, read from [`super::ridgeback`]: the plumbing that gives
//! a creature a back to stand on, soft places on top that fill its poise, feet
//! that break and a topple (`docs/design/monster-design.md`, M1). What is its
//! own is in its own tuning (`tuned.rs`): **its size** -- one and a half
//! times the Ridgeback, its back eight metres up -- and **its hide**:
//! its feet take a fraction of what lands on them and its back takes the
//! damage that brings it down (the owner, 2026-11-02). Of the Ridgeback's
//! eight moves it chooses only the ones aimed at the floor (the stomp, the
//! sweep, the slam) and the ones that answer a fighter where it cannot see
//! (the kick, the shake); the bite, the charge and the spray are never
//! chosen. New moves of its own -- a shockwave, a floor taken away, the
//! swat at a perch -- are the next work, and need clips authored for them.
//!
//! **It sleeps until somebody comes into the keep** ([`frame`]): its grace
//! is held, so it stands where it was and throws nothing, while every hunter
//! is still on the way. The first hunter onto the keep's floor wakes it, and
//! a hit wakes it anywhere.
//!
//! **And the climb pays** (the owner, 2026-11-03: the agility section up to
//! the rampart "should teach that the mobility sections offer an extra
//! reward even when optional"). Woken by a blow while a hunter is high --
//! more than [`HIGH`] up, which from outside its floor means the rampart, or
//! a dive off it -- it goes straight over: the fight opens with it toppled,
//! the window a climb onto its back would have to earn. Walked in on the
//! floor, or shot through the gate from the yard, it simply wakes.

mod tuned;

use crate::fixed::Fx;
use crate::species::ridgeback::{self, Clip};
use crate::species::{MoveDecl, Species, SpeciesId, Stock};
use crate::state::World;

/// The Ridgeback's moves, the Warden's choice of them: what it never
/// chooses is still in the table, so the move rows line up with the
/// Ridgeback's and every clip index stays the same.
pub const MOVES: [MoveDecl; ridgeback::MOVE_COUNT] = [
    MoveDecl::new("Bite", Clip::Bite as usize).never_chosen(),
    MoveDecl::new("Stomp", Clip::Stomp as usize),
    MoveDecl::new("Floor sweep", Clip::Sweep as usize).mirrors_to_target_side(),
    MoveDecl::new("Charge", Clip::Charge as usize).never_chosen(),
    MoveDecl::new("Rear and slam", Clip::Slam as usize),
    MoveDecl::new("Shake", Clip::Shake as usize).scaled_by(ridgeback::Knob::ShakeForce as u16),
    MoveDecl::new("Back kick", Clip::Kick as usize),
    MoveDecl::new("Spike spray", Clip::Spray as usize).never_chosen(),
];

pub static SPECIES: Species = Species {
    id: SpeciesId::WARDEN,
    name: "Warden",
    bones: &ridgeback::BONES,
    mirror: &ridgeback::MIRROR,
    neck: &ridgeback::NECK_CHAIN,
    follows: &ridgeback::FOLLOWS,
    parts: &ridgeback::PARTS,
    breakable: crate::beast::breakables(&ridgeback::PARTS),
    legs: &ridgeback::LEGS,
    moves: &MOVES,
    clips: &ridgeback::CLIPS,
    stock: Stock {
        idle: Clip::Idle as usize,
        walk: Clip::Walk as usize,
        gallop: Clip::Gallop as usize,
        flinch: Clip::Flinch as usize,
        stumble: Clip::Stumble as usize,
        topple: Clip::Topple as usize,
        dead: Clip::Dead as usize,
    },
    span: &ridgeback::baked::SPAN,
    rows: ridgeback::baked::ROWS,
    row: |r| &ridgeback::baked::FRAMES[r],
    own: ridgeback::Knob::DECLS,
    tuned: &tuned::KNOBS,
    tuned_path: "crates/sim/src/species/warden/tuned.rs",
    pack: None,
    fight: &FIGHT,
};

/// A body against the solids, as the Ridgeback's is, and the sleep.
pub static FIGHT: crate::species::FightDecl = crate::species::FightDecl {
    row: true,
    collides: true,
    frame: Some(frame),
    ..crate::species::FightDecl::PLAIN
};

/// `Monster::own[0]`: has it woken? Once it has, it stays awake.
const AWAKE: usize = 0;

/// **How high a hunter must be** for a blow that wakes it to topple it, in
/// centimetres: below its back (8.4 m) and above anything on its floor's
/// level outside the keep. Level data, like the keep's own heights.
pub const HIGH: i32 = 600;

/// **The sleep**: while it has not woken, its grace is held (it stands and
/// throws nothing), until a hunter is inside the keep's walls or it has been
/// hit (which ends its grace on its own, `Monster::take_blow`).
pub fn frame(w: &mut World) {
    let (kx, kz) = crate::arena::warden::KEEP;
    let centre = (Fx::ratio(kx, 100), Fx::ratio(kz, 100));
    let inside = Fx::ratio(crate::arena::warden::KEEP_FLOOR, 100);
    let present = w.present();
    // Each axis first: the yard is further from the keep than a square of
    // fixed point holds (`docs/design/fight-spaces.md`, F8), so nothing out
    // there is squared.
    let near = w.players[..present].iter().any(|p| {
        let dx = p.pos.x.sub(centre.0);
        let dz = p.pos.z.sub(centre.1);
        dx.abs().raw() < inside.raw()
            && dz.abs().raw() < inside.raw()
            && dx.mul(dx).add(dz.mul(dz)).raw() < inside.mul(inside).raw()
    });
    let high = w.players[..present]
        .iter()
        .any(|p| p.pos.y.raw() > Fx::ratio(HIGH, 100).raw());
    for m in w.monsters.iter_mut().flatten() {
        if m.species != SpeciesId::WARDEN || m.own[AWAKE] != 0 {
            continue;
        }
        if near {
            m.own[AWAKE] = 1;
            continue;
        }
        // Struck since it last slept: awake -- and over, if the blow came
        // from above.
        if m.brain.grace == 0 {
            m.own[AWAKE] = 1;
            if high {
                m.doing = crate::monster::Doing::Toppled {
                    left: m.sp().topple_frames(),
                };
            }
            continue;
        }
        m.brain.grace = m.brain.grace.max(2);
    }
}
