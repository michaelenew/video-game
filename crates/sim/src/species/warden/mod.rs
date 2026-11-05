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

use crate::arena::warden::{FALLEN, SAFE, TOP};
use crate::fixed::Fx;
use crate::species::ridgeback::{self, Clip};
use crate::species::{MoveDecl, Species, SpeciesId, Stock};
use crate::state::{Player, World};

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
    // Its height is the island's ([`stand_back`]): a body that climbs what
    // it meets, it would otherwise take the island's top for something to
    // climb and sink through it to the floor far below.
    keeps_height: true,
    frame: Some(frame),
    ..crate::species::FightDecl::PLAIN
};

/// `Monster::own[0]`: has it woken? Once it has, it stays awake.
const AWAKE: usize = 0;

/// **How high a hunter must be** for a blow that wakes it to topple it, in
/// centimetres over the island's top: below its back (8.4 m) and above
/// anything on the floor's level outside the keep. Level data, like the
/// keep's own heights.
pub const HIGH: i32 = 600;

/// **What it does to the place every frame**: the fall off the island
/// ([`stand_back`]), then the sleep -- while it has not woken, its grace is
/// held (it stands and throws nothing), until a hunter is inside the keep's
/// walls or it has been hit (which ends its grace on its own,
/// `Monster::take_blow`).
pub fn frame(w: &mut World) {
    stand_back(w);
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
        .any(|p| p.pos.y.raw() > Fx::ratio(TOP + HIGH, 100).raw());
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

/// **What a fall off the island costs**, in health: about a stomp's worth.
/// Level data: what the drop is worth, judged with the drop.
pub const FALL_COST: i32 = 120;

/// **A fall off the island is not a death** (`docs/design/level-design.md`,
/// L8): a fighter more than [`FALLEN`] below the island's top is stood on
/// the nearest safe spot ([`SAFE`]), as a jump course stands you on its last
/// gate (`World::step_course`) -- fresh, with whatever she had out taken
/// back -- less [`FALL_COST`] of the health she had, never below one. And
/// the Warden stands on the island's top, and if it walks out over the drop
/// is back in the middle of its keep.
fn stand_back(w: &mut World) {
    let pit = Fx::ratio(TOP - FALLEN, 100);
    let top = Fx::ratio(TOP, 100);
    let present = w.present();
    for i in 0..present {
        let p = w.players[i];
        if p.pos.y.raw() >= pit.raw() {
            continue;
        }
        // The nearest by the two flat distances added: nothing here is
        // squared, at the island's size (`fight-spaces.md`, F8).
        let near = |s: &(i32, i32)| {
            let dx = Fx::ratio(s.0, 100).sub(p.pos.x).abs();
            let dz = Fx::ratio(s.1, 100).sub(p.pos.z).abs();
            dx.add(dz).raw()
        };
        // On the side of the curtain wall the fall was on: it runs down
        // the island's face, so nobody falls past it, and a fall is not a
        // way through it (`arena::warden::CURTAIN`).
        let (west, _, _) = crate::arena::warden::CURTAIN;
        let west_of = |x: Fx| x.raw() < Fx::ratio(west, 100).raw();
        let side = west_of(p.pos.x);
        let Some(&(x, z)) = SAFE
            .iter()
            .filter(|s| west_of(Fx::ratio(s.0, 100)) == side)
            .min_by_key(|s| near(s))
        else {
            continue;
        };
        let (wins, class) = (p.rounds_won, p.class);
        let health = (p.health - FALL_COST).max(1);
        let stand = crate::math::V3::new(Fx::ratio(x, 100), top, Fx::ratio(z, 100));
        let q = &mut w.players[i];
        *q = Player {
            pos: stand,
            facing: p.facing,
            rounds_won: wins,
            ..Player::new(class)
        };
        q.health = health;
        if let crate::class::Mechanic::Shadow(_) = q.mechanic {
            q.mechanic =
                crate::class::Mechanic::Shadow(crate::class::Shadow::attending(q.pos, q.facing));
        }
    }
    // The Warden stands on the island's top wherever the island is under
    // it; walked out over the drop, it is back in the middle of its keep.
    let (kx, kz) = crate::arena::warden::KEEP;
    let ground = w.terrain();
    let mut over_the_drop = [false; crate::monster::MAX_MONSTERS];
    for (i, m) in w.monsters.iter().enumerate() {
        if let Some(m) = m.filter(|m| m.species == SpeciesId::WARDEN) {
            let probe = crate::math::V3::new(m.pos.x, top.add(Fx::ONE), m.pos.z);
            over_the_drop[i] = ground.ground_under(probe).raw() < pit.raw();
        }
    }
    for (i, m) in w.monsters.iter_mut().enumerate() {
        let Some(m) = m.as_mut().filter(|m| m.species == SpeciesId::WARDEN) else {
            continue;
        };
        if over_the_drop[i] {
            m.pos = crate::math::V3::new(Fx::ratio(kx, 100), top, Fx::ratio(kz, 100));
            m.speed = Fx::ZERO;
        } else {
            m.pos.y = top;
        }
    }
}
