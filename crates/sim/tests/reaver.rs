//! The Shadow Reaver's second body.
//!
//! One property under all of it: **the shadow is never nowhere.** It attends
//! her or it stands out on the field, and every option the class has is a
//! function of the line between the two. The old mechanic allowed a third
//! state — no shadow at all — and the class spent half a match with its whole
//! vocabulary greyed out.
//!
//! What is pinned here is what a player would notice if it broke: that holding
//! the shadow is worth a quarter again on every swing, that the copy lands a
//! beat late and from somewhere else, that the recall cuts on its way home, and
//! that the forward dodge is the dash.

use sim::class::{Ghost, Mechanic, Shadow};
use sim::state::{Action, SLOT_COMMITTED, SLOT_MECHANIC, SLOT_POKE, SLOT_SPECIAL};
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

const E: u16 = Input::MECHANIC;
const L: u16 = Input::LEFT;
/// Right click, which on this class sends the shadow -- it is the half of the
/// kit the crosshair aims, and the mouse is where aiming lives.
const R: u16 = Input::RIGHT;
const Q: u16 = Input::SPECIAL;
const W: u16 = Input::W;
const SHIFT: u16 = Input::SHIFT;

fn down(degrees: i32) -> i16 {
    (-degrees * 65536 / 360) as i16
}

fn run(w: &mut World, frames: u32, bits: u16, pitch: i16) {
    for _ in 0..frames {
        w.advance([Input::looking_at(bits, 0, pitch), Input::default()]);
    }
}

fn tap(w: &mut World, bits: u16, pitch: i16, then: u32) {
    run(w, 2, bits, pitch);
    run(w, then, 0, pitch);
}

fn shadow(w: &World) -> Shadow {
    match w.players[0].mechanic {
        Mechanic::Shadow(s) => s,
        _ => panic!("player one is not the Reaver"),
    }
}

/// A Reaver facing down +X with a dummy standing two metres in front of her.
///
/// Two metres is inside her poke and inside her shadow's copy of it, which is
/// the arrangement the damage tests need: both bodies reach the same target.
fn duel() -> World {
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    // Close, because her own hit knocks him back and the copy lands four
    // frames later from a step further away again.
    w.players[1].pos = V3::new(Fx::ratio(-49, 10), Fx::ZERO, Fx::ZERO);
    w.players[1].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    // Let the shadow settle onto her heel before anything is measured.
    run(&mut w, 30, 0, 0);
    w
}

fn hurt(w: &World) -> i32 {
    sim::state::max_health() - w.players[1].health
}

/// Ask `aim` something about a world as it stands. The scene is everything a
/// ray can meet, borrowed from copies exactly as the simulation builds it.
fn with_scene<T>(w: &World, ask: impl FnOnce(&sim::aim::Scene) -> T) -> T {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    ask(&sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
    })
}

/// The pitch, looking down +X, that puts her crosshair on a thing standing at
/// `at`.
///
/// Searched rather than solved. The camera's geometry belongs to the camera,
/// and a test that worked the angle out for itself would be a second copy of
/// it -- which is the whole mistake `aim` exists to stop.
fn crosshair_onto(w: &World, at: V3) -> i16 {
    with_scene(w, |scene| {
        for degrees in -89..=89 {
            let pitch = (degrees * 65536 / 360) as i16;
            if sim::aim::pointing_at(
                0,
                Input::looking_at(0, 0, pitch),
                at,
                t::shadow_lock_cone(),
                scene,
            ) {
                return pitch;
            }
        }
        panic!("no pitch puts the crosshair on it");
    })
}

/// Stand the shadow out on the field at `at`, without playing a throw to get it
/// there. The dash tests are about the dash.
fn put_the_shadow_at(w: &mut World, at: V3) {
    let mut s = shadow(w);
    s.pos = at;
    s.doing = Ghost::Waiting;
    w.players[0].mechanic = Mechanic::Shadow(s);
}

/// A Reaver on open floor, facing down +X, with nothing in front of her and
/// nobody near enough to shove her off a line.
///
/// `z = 8` is past the end of both platforms, which matters more than it
/// sounds: `duel()` stands her on top of one.
fn in_the_open() -> World {
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(8));
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
    run(&mut w, 20, 0, 0);
    w
}

/// The right-hand platform: the dais the third dash test is about.
///
/// Found rather than written down, so the test follows the blockout. The walls
/// are solids too and they sit *outside* the play area, which is what tells
/// them apart from a platform you can stand in front of.
fn dais() -> sim::arena::Solid {
    *sim::arena::SOLIDS
        .iter()
        .filter(|s| s.max.x.raw() < sim::arena::ARENA_HALF.raw())
        .max_by_key(|s| s.min.x.raw())
        .expect("the arena has a platform")
}

/// A Reaver on the floor with that platform four metres in front of her.
fn facing_the_dais() -> World {
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::ZERO);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
    run(&mut w, 20, 0, 0);
    w
}

// ---------------------------------------------------------------------------
// It is always somewhere
// ---------------------------------------------------------------------------

#[test]
fn the_shadow_is_never_nowhere() {
    // The whole mechanic in one assertion. Through a throw, a wait, a recall
    // and the journey home, `placed()` always answers -- which is what lets
    // Guillotine lotus be aimed at the mechanic without a branch for the case
    // where the mechanic does not exist.
    let mut w = duel();
    let mut seen = Vec::new();
    for frame in 0..160 {
        let bits = if frame == 10 || frame == 90 { R } else { 0 };
        run(&mut w, 1, bits, down(15));
        assert!(
            w.players[0].mechanic.placed().is_some(),
            "frame {frame}: the shadow was nowhere"
        );
        let doing = shadow(&w).doing;
        if seen.last() != Some(&std::mem::discriminant(&doing)) {
            seen.push(std::mem::discriminant(&doing));
        }
    }
    // Attending, out, standing, home, attending: every state was visited, so
    // the assertion above was not just watching one of them.
    assert!(
        seen.len() >= 4,
        "the shadow only ever did {} things, so this proves less than it looks",
        seen.len()
    );
}

#[test]
fn the_shadow_stands_behind_her_rather_than_inside_her() {
    // Presentation with teeth: the copy swings from wherever the shadow is, so
    // two bodies in the same place would be two threats a player cannot tell
    // apart -- and one of them hits for a quarter of the other.
    let w = duel();
    let gap = shadow(&w).pos.sub(w.players[0].pos);
    assert!(
        gap.flat_len().raw() > t::shadow_trail().mul(Fx::ratio(1, 2)).raw(),
        "the shadow is standing on top of her"
    );
    assert!(
        gap.dot(w.players[0].facing).raw() < 0,
        "the shadow is standing in front of her, where it will be mistaken for her"
    );
}

// ---------------------------------------------------------------------------
// The copy
// ---------------------------------------------------------------------------

#[test]
fn the_shadow_copies_her_swing_a_beat_late() {
    let mut w = duel();
    let poke = sim::moves::get(Class::ShadowReaver, SLOT_POKE);

    // Her own hit lands first, on its own.
    let mut hers = None;
    for frame in 0..(poke.whiff_cost() as u32 + t::shadow_lag() as u32 + 4) {
        let bits = if frame < 2 { L } else { 0 };
        run(&mut w, 1, bits, 0);
        if hers.is_none() && hurt(&w) > 0 {
            hers = Some(hurt(&w));
        }
    }
    let first = hers.expect("her own poke never landed");
    assert_eq!(
        first, poke.damage,
        "her poke dealt something other than its own damage"
    );
    assert!(
        hurt(&w) > first,
        "the shadow never repeated the swing -- only {first} damage in total"
    );
}

#[test]
fn the_copy_is_worth_a_quarter_of_the_swing_it_copies() {
    // The number the class is balanced around: holding the shadow is a
    // twenty-five per cent damage buff on everything her body does, paid for by
    // not having the second body anywhere useful.
    let mut w = duel();
    let poke = sim::moves::get(Class::ShadowReaver, SLOT_POKE);
    tap(
        &mut w,
        L,
        0,
        poke.whiff_cost() as u32 + t::shadow_lag() as u32 + 8,
    );

    let total = hurt(&w);
    let echoed = total - poke.damage;
    let want = Fx::from_int(poke.damage).mul(t::shadow_echo()).to_int();
    assert_eq!(
        echoed, want,
        "the copy dealt {echoed} where the swing dealt {} -- the quarter is the class",
        poke.damage
    );
}

#[test]
fn a_shadow_out_on_the_field_swings_from_out_there() {
    // The other half of what the copy is for. With the shadow sent away, her
    // swing comes out twice in two places, so a Reaver with the shadow well
    // placed threatens ground she is not standing on.
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    // Out of the way while the shadow is thrown.
    w.players[1].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::from_int(9));
    run(&mut w, 20, 0, 0);

    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    tap(
        &mut w,
        R,
        down(30),
        (send.whiff_cost() + t::shadow_send_frames()) as u32,
    );
    // Then stand him on it, a step in front of where the copy will swing from.
    let out = shadow(&w).pos;
    w.players[1].pos = V3::new(out.x.add(Fx::ONE), Fx::ZERO, out.z);
    let poke = sim::moves::get(Class::ShadowReaver, SLOT_POKE);
    assert!(
        w.players[0].pos.sub(w.players[1].pos).flat_len().raw() > poke.reach.raw() * 2,
        "the fixture left her close enough to reach him herself"
    );

    let before = hurt(&w);
    tap(
        &mut w,
        L,
        0,
        poke.whiff_cost() as u32 + t::shadow_lag() as u32 + 8,
    );
    assert!(
        hurt(&w) > before,
        "she swung at nothing and her shadow, standing on him, swung at nothing too"
    );
}

// ---------------------------------------------------------------------------
// Sending it, and getting it back
// ---------------------------------------------------------------------------

#[test]
fn the_shadow_flies_out_and_then_stops() {
    // "Quickly out, then a pause at the end of it." The pause is what the rest
    // of the kit is aimed at, so the throw has to actually finish rather than
    // drifting.
    let mut w = duel();
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    run(&mut w, 2, R, down(25));
    run(&mut w, send.startup as u32, 0, down(25));

    let mut steps = Vec::new();
    let mut last = shadow(&w).pos;
    for _ in 0..(t::shadow_send_frames() as u32 + 20) {
        run(&mut w, 1, 0, down(25));
        let now = shadow(&w).pos;
        steps.push(now.sub(last).flat_len());
        last = now;
    }
    assert!(
        matches!(shadow(&w).doing, Ghost::Waiting),
        "it never settled"
    );
    let quickest = steps.iter().map(|s| s.raw()).max().unwrap_or(0);
    assert!(
        Fx::from_raw(quickest).raw() > t::move_speed().mul(sim::DT).raw() * 3,
        "the shadow ambled out at walking pace"
    );
    // And it is standing still by the end.
    assert_eq!(steps.last().map(|s| s.raw()), Some(0), "it never stopped");
}

#[test]
fn the_leash_fits_the_throw() {
    // A leash shorter than the throw would have the shadow turn round on the
    // frame it landed, which is the class's setup deleting itself.
    //
    // **Doubled on 2026-09-17**: a leash only a third longer than the throw
    // meant a shadow placed at full range expired almost as soon as it arrived,
    // so a placement was something to spend rather than something to keep. The
    // shadow is this class's movement, and movement you have to use immediately
    // is not a decision. Two throws of slack is the room to leave it out there.
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    let slack = t::shadow_leash().div(send.reach);
    assert!(
        slack.raw() >= Fx::from_int(2).raw(),
        "the shadow can be thrown {} m and is leashed at {} m -- under twice the \
         throw, so a shadow left at range comes home before she has used it",
        send.reach.to_f32_for_render(),
        t::shadow_leash().to_f32_for_render()
    );
}

#[test]
fn the_dash_crosses_the_whole_leash() {
    // **What makes the dash an escape rather than a gamble.** It ends when the
    // dodge does -- it does not outlive it -- so a dash that cannot cover the
    // distance spends the dodge and arrives nowhere, in the open, out of
    // invulnerability. Whether that can happen is a question about the leash
    // and the dodge together, and neither of them knows about the other.
    //
    // Untested until 2026-09-17, when the leash doubled and the only thing
    // holding the property up was a sentence in `tuning::shadow_dash_speed`.
    let covered = t::shadow_dash_speed().mul(Fx::ratio(t::dodge_frames() as i32, 60));
    assert!(
        covered.raw() >= t::shadow_leash().raw(),
        "a dash covers {} m in the {} frames a dodge lasts, against a leash of {} m -- \
         she can be left stranded at the far end of her own mechanic",
        covered.to_f32_for_render(),
        t::dodge_frames(),
        t::shadow_leash().to_f32_for_render()
    );
}

#[test]
fn the_recall_cuts_and_slows_what_it_comes_home_through() {
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    // Out of the way of the throw, and back in the way of the return.
    w.players[1].pos = V3::new(Fx::from_int(-3), Fx::ZERO, Fx::from_int(6));
    run(&mut w, 20, 0, 0);

    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    tap(
        &mut w,
        R,
        down(25),
        (send.whiff_cost() + t::shadow_send_frames()) as u32,
    );
    assert_eq!(hurt(&w), 0, "the throw itself hurt somebody");

    // Stand him on the line home and call it back.
    let out = shadow(&w).pos;
    let midway = V3::new(
        out.x.add(w.players[0].pos.x).mul(Fx::ratio(1, 2)),
        Fx::ZERO,
        out.z.add(w.players[0].pos.z).mul(Fx::ratio(1, 2)),
    );
    w.players[1].pos = midway;
    run(&mut w, 2, R, down(25));
    let mut slowed = false;
    for _ in 0..60 {
        run(&mut w, 1, 0, down(25));
        slowed |= w.players[1].slowed > 0;
    }

    assert_eq!(
        hurt(&w),
        send.damage,
        "the shadow came home through him without cutting him exactly once"
    );
    assert!(
        slowed,
        "the recall cut him but did not slow him, which is the half that matters"
    );
}

// ---------------------------------------------------------------------------
// The dash
// ---------------------------------------------------------------------------

#[test]
fn a_forward_dodge_at_the_shadow_crosses_to_it() {
    let mut w = duel();
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    tap(
        &mut w,
        R,
        down(25),
        (send.whiff_cost() + t::shadow_send_frames()) as u32,
    );
    let out = shadow(&w).pos;
    let stood = w.players[0].pos;
    assert!(
        out.sub(stood).flat_len().raw() > Fx::from_int(4).raw(),
        "the fixture left the shadow within arm's reach, so arriving proves nothing"
    );

    // Shift and forward, looking at it. The camera is behind her and she is
    // facing the shadow, so the crosshair is on it.
    //
    // **Closest approach rather than where she finishes.** Arriving does not
    // stop her -- the speed she crossed at stays under her as the slide, which
    // is the vulnerable tail of the dash and the thing a jump inside the carry
    // window spends (`tuning::shadow_carry`). She therefore goes *through* the
    // shadow and keeps going, and how far past depends on the dash's speed,
    // which was raised on 2026-09-17 when the leash doubled. Where she ends up
    // was never what this test meant.
    let mut sighted = false;
    let mut closest = Fx::from_int(100);
    for _ in 0..(t::dodge_frames() as u32 + 10) {
        run(&mut w, 1, SHIFT | W, down(10));
        sighted |= w.players[0].action.invulnerable();
        closest = closest.min(w.players[0].pos.sub(out).flat_len());
    }
    assert!(sighted, "she was never invulnerable, so it was not a dodge");
    assert!(
        closest.raw() < Fx::from_int(1).raw(),
        "she came no closer than {:.1} m to the shadow she dashed at",
        closest.to_f32_for_render()
    );
    // Arriving picks it up: the loop is throw, act, dash back on, throw again.
    assert!(
        !shadow(&w).is_out(),
        "she crossed to the shadow and left it standing there"
    );
}

#[test]
fn a_forward_air_dodge_at_the_shadow_is_the_dash_too() {
    // Being off the floor is the commonest reason she is not standing where she
    // wants to be, so a mobility option that switched off the moment she jumped
    // would be mobility in the wrong place. The airdodge, aimed, is the dash --
    // and it pays the airdodge, because one commitment per airtime is what
    // stops a jump becoming flight.
    let mut w = in_the_open();
    let out = V3::new(Fx::from_int(7), Fx::ZERO, Fx::from_int(8));
    put_the_shadow_at(&mut w, out);

    run(&mut w, 10, Input::SPACE, 0);
    assert!(!w.players[0].grounded, "she never left the floor");
    // Seven metres, against an airdodge that covers a shade over three. Nothing
    // but the dash reaches from here.
    let reach = t::air_dodge_speed()
        .mul(Fx::from_int(t::air_dodge_frames() as i32))
        .mul(sim::DT);
    assert!(
        out.sub(w.players[0].pos).flat_len().raw() > reach.raw().saturating_mul(2),
        "the fixture left the shadow inside an airdodge's reach, so arriving proves nothing"
    );

    let mut spent_the_airdodge = false;
    let mut closest = Fx::from_int(100);
    for _ in 0..(t::dodge_frames() as u32 + 4) {
        let pitch = crosshair_onto(&w, out);
        run(&mut w, 1, SHIFT | W, pitch);
        spent_the_airdodge |= w.players[0].air_dodged;
        closest = closest.min(w.players[0].pos.sub(out).len());
    }
    // A frame of the dash's own travel, which is the tolerance the arrival test
    // in `shadow::step_her_dash` uses and the finest a per-frame sample can
    // resolve: she is put exactly on the shadow and then slid off it again
    // inside the same frame, so the closest *sampled* position is a step out.
    // It was a body radius until 2026-09-17, which was a number that happened
    // to be larger than a step while the dash was slower.
    let step = t::body_radius().max(t::shadow_dash_speed().mul(sim::DT));
    assert!(
        closest.raw() < step.raw(),
        "she came no closer than {:.1} m to the shadow, so it was an airdodge",
        closest.to_f32_for_render()
    );
    assert!(
        !shadow(&w).is_out(),
        "she crossed to the shadow and left it standing there"
    );
    assert!(
        spent_the_airdodge,
        "the air dash was free -- one commitment per airtime is what stops a \
         jump becoming flight"
    );
}

#[test]
fn the_dash_climbs_to_a_shadow_standing_on_a_dais() {
    // The dash goes to where the shadow *is*, along the straight line between
    // them. It used to drive only her feet, so a shadow a storey up was a
    // shadow she ran at the side of the thing it was standing on.
    let mut w = facing_the_dais();
    let dais = dais();
    let deck = V3::new(
        dais.min.x.add(dais.max.x).mul(Fx::ratio(1, 2)),
        dais.max.y,
        Fx::ZERO,
    );
    put_the_shadow_at(&mut w, deck);
    assert!(
        w.players[0].pos.y.raw() == 0,
        "she is meant to start on the floor, below the thing she is dashing on to"
    );

    let mut arrived = None;
    for _ in 0..(t::dodge_frames() as u32 + 4) {
        let pitch = crosshair_onto(&w, deck);
        run(&mut w, 1, SHIFT | W, pitch);
        if arrived.is_none() && !shadow(&w).is_out() {
            arrived = Some(w.players[0].pos);
        }
    }
    let landed = arrived.expect("she never reached the shadow at all");
    assert!(
        landed.y.raw() >= dais.max.y.raw(),
        "she arrived at {:.2} m up, below the deck at {:.2} m",
        landed.y.to_f32_for_render(),
        dais.max.y.to_f32_for_render()
    );
    assert!(
        landed.x.raw() > dais.min.x.raw() && landed.x.raw() < dais.max.x.raw(),
        "she got the height but not the place: {:.2} m is not over the dais",
        landed.x.to_f32_for_render()
    );
}

#[test]
fn a_stone_across_the_line_leaves_her_with_an_ordinary_dodge() {
    // The one thing that refuses a dash: no line at all. A stone is exactly as
    // tall as a fighter, so one standing between the two bodies blocks every
    // line between them -- which makes denying the Reaver's line a thing the
    // Elementalist can actually do, rather than a rule nothing exercises.
    let mut w = World::with_classes([Class::ShadowReaver, Class::Elementalist]);
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(8));
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
    run(&mut w, 20, 0, 0);
    let out = V3::new(Fx::from_int(8), Fx::ZERO, Fx::from_int(8));
    put_the_shadow_at(&mut w, out);
    let pitch = crosshair_onto(&w, out);

    // Halfway, and squarely on the line.
    let stone = sim::class::Structure {
        age: u16::MAX,
        ..sim::class::Structure::raised(V3::new(Fx::from_int(4), Fx::ZERO, Fx::from_int(8)))
    };
    w.players[1].mechanic = Mechanic::Structures([Some(stone), None, None]);

    let stood = w.players[0].pos;
    let mut sighted = false;
    for _ in 0..(t::dodge_frames() as u32 + 4) {
        run(&mut w, 1, SHIFT | W, pitch);
        sighted |= w.players[0].action.invulnerable();
    }
    assert!(sighted, "she did not dodge at all, so this proves nothing");
    assert!(
        shadow(&w).is_out(),
        "she crossed to a shadow there was no way through to"
    );
    // She still got the dodge. The class's mobility and the universal defensive
    // option are the same button, so a refused dash is the other half of the
    // button rather than a dead press.
    let went = w.players[0].pos.sub(stood).flat_len();
    assert!(
        went.raw() > Fx::ONE.raw(),
        "the dash was refused and so was the dodge -- she did not move at all"
    );
}

#[test]
fn a_jump_inside_the_carry_leaves_with_the_dash_under_her() {
    // Arriving leaves her sliding at the speed she crossed at, and the slide
    // decays. A jump pressed inside that window takes what is left of it up
    // with her; the earlier she finds it, the further she goes.
    let mut w = in_the_open();
    let out = V3::new(Fx::from_int(8), Fx::ZERO, Fx::from_int(8));
    put_the_shadow_at(&mut w, out);
    let pitch = crosshair_onto(&w, out);

    let mut took_off = None;
    for _ in 0..(t::dodge_frames() as u32 * 2) {
        // Shift comes off the moment the window opens, so what is measured is
        // the jump rather than a second dodge thrown on the next frame.
        let carrying = shadow(&w).carry > 0;
        let bits = if carrying {
            W | Input::SPACE
        } else {
            SHIFT | W
        };
        run(&mut w, 1, bits, pitch);
        if carrying && took_off.is_none() && !w.players[0].grounded {
            took_off = Some(w.players[0].vel);
        }
    }
    let leaving = took_off.expect("the jump inside the carry never left the ground");
    let along = V3::new(leaving.x, Fx::ZERO, leaving.z).flat_len();
    assert!(
        leaving.y.raw() > 0,
        "she was airborne without going up, so that was the dash and not a jump"
    );
    assert!(
        along.raw() > t::move_speed().mul(Fx::from_int(2)).raw(),
        "she left the ground at {:.1} m/s, which is a standing jump rather than \
         a jump with the dash under it",
        along.to_f32_for_render()
    );
}

#[test]
fn an_ordinary_dodge_has_no_carry_to_jump_out_of() {
    // The window is a dash's, and only a dash's. Every dodge in the game has a
    // tail you can be punished during, and a jump out of that tail would be a
    // universal escape rather than one class's tech.
    let mut w = in_the_open();
    let stood = w.players[0].pos;
    // Nothing out on the field, so shift and forward is the ordinary dodge.
    assert!(!shadow(&w).is_out());
    let mut airborne = false;
    for frame in 0..t::dodge_frames() as u32 {
        let bits = if frame == 4 {
            SHIFT | W | Input::SPACE
        } else {
            SHIFT | W
        };
        run(&mut w, 1, bits, 0);
        airborne |= !w.players[0].grounded;
    }
    assert!(
        !airborne,
        "she jumped out of an ordinary dodge, so the carry is not the dash's"
    );
    assert!(
        w.players[0].pos.sub(stood).flat_len().raw() > Fx::ONE.raw(),
        "she did not dodge at all, so this proves nothing"
    );
}

#[test]
fn a_forward_dodge_with_the_shadow_at_her_heel_is_just_a_dodge() {
    // The dash is a decision, not a thing that happens to her. With nothing out
    // on the field there is nothing to point at, so shift and forward is the
    // universal defensive option and behaves like everyone else's.
    let mut w = duel();
    let stood = w.players[0].pos;
    run(&mut w, t::dodge_frames() as u32 + 4, SHIFT | W, 0);
    let went = w.players[0].pos.sub(stood).flat_len();
    assert!(
        went.raw() > Fx::ONE.raw(),
        "she did not move at all, so the dodge did not happen"
    );
    assert!(!shadow(&w).is_out());
}

// ---------------------------------------------------------------------------
// In a hunt
// ---------------------------------------------------------------------------

/// A Reaver standing next to a creature that is holding still, so a test about
/// the shadow is about the shadow rather than about chasing.
fn hunting() -> World {
    let mut w = World::hunt([Class::ShadowReaver, Class::Bulwark]);
    let mut beast = w.monster.expect("a hunt has a creature");
    // Close enough that the copy, which swings from a step behind her, reaches
    // it too -- the point of the test is the second body, not her spacing.
    // Placed by its *head* rather than by its centre: it is thirteen metres
    // long, so where the middle of it is says nothing about what either body
    // can reach. Its head goes between the two of them.
    let head = sim::monster::shape(sim::monster::HEAD);
    let mid = head.min.x.add(head.max.x).mul(Fx::ratio(1, 2));
    beast.pos = V3::new(
        mid.sub(t::shadow_trail().mul(Fx::ratio(1, 2))),
        Fx::ZERO,
        Fx::ZERO,
    );
    // Facing back down the arena, and not thinking about anything.
    beast.yaw = Fx::from_raw(1 << 15);
    beast.doing = sim::monster::Doing::Prowl;
    beast.brain.think_left = u16::MAX;
    w.monster = Some(beast);
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::ZERO);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::ZERO);
    run(&mut w, 20, 0, 0);
    w
}

fn beast_health(w: &World) -> i32 {
    w.monster.expect("a hunt has a creature").health
}

#[test]
fn the_shadow_fights_the_creature_too() {
    // A mechanic that only works in versus is half a mechanic, and this is
    // exactly the shape of bug that hid in the Blood mage's fields for months:
    // it is invisible in the mode most tests are written in.
    let mut w = hunting();
    let full = beast_health(&w);
    let poke = sim::moves::get(Class::ShadowReaver, SLOT_POKE);

    // Her own swing lands first, alone.
    let mut hers = None;
    for frame in 0..(poke.whiff_cost() as u32 + t::shadow_lag() as u32 + 8) {
        let bits = if frame < 2 { L } else { 0 };
        run(&mut w, 1, bits, 0);
        if hers.is_none() && beast_health(&w) < full {
            hers = Some(full - beast_health(&w));
        }
    }
    let first = hers.expect("she never reached the creature");
    assert!(
        full - beast_health(&w) > first,
        "the shadow never copied the swing onto the creature -- {first} damage in total"
    );
}

#[test]
fn the_recall_cuts_the_creature_on_its_way_home() {
    let mut w = hunting();
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    // Out past the animal, so the way home crosses it.
    tap(
        &mut w,
        R,
        0,
        (send.whiff_cost() + t::shadow_send_frames()) as u32,
    );
    assert!(shadow(&w).is_waiting(), "the shadow never got out");
    let before = beast_health(&w);
    tap(&mut w, R, 0, 60);
    assert!(
        beast_health(&w) < before,
        "the shadow came home through the creature without touching it"
    );
}

// ---------------------------------------------------------------------------
// Which button is which
// ---------------------------------------------------------------------------
//
// The two are swapped against every other class, and the swap is the point:
// **the mouse means where.** Sending the shadow is the only thing in this kit
// the crosshair aims, so it is on the mouse; Executioner is a swing off the
// body and does not care, so it is on the key.

#[test]
fn right_click_sends_the_shadow() {
    let mut w = duel();
    run(&mut w, 2, R, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_MECHANIC),
        "right click did not throw Send shadow"
    );
}

#[test]
fn the_mechanic_key_throws_her_committed_melee() {
    // The one class where `E` carries a move that is not the mechanic. Worth a
    // test rather than a comment: `on_e` is the only thing that says so, and it
    // is one word away from being the slot everybody else puts there.
    let mut w = duel();
    run(&mut w, 2, E, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_COMMITTED),
        "`E` did not throw Executioner"
    );
}

#[test]
fn shift_and_left_click_no_longer_throws_anything_of_its_own() {
    // Executioner used to answer to shift + left click as well as to `E`, which
    // was the shared grammar's own binding for the committed slot. Shift is only
    // a dodge now -- on every class, see `docs/design/controls.md` -- so that
    // second home is gone and the modifier is simply ignored.
    //
    // **The Reaver is the one class that loses nothing by it**, and this test is
    // where that is recorded: her committed move was already on a key of its
    // own, so the change strands nobody here. The other three are stranded, and
    // finding each a home is a separate job.
    let mut w = duel();
    run(&mut w, 2, SHIFT | L, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(sim::state::SLOT_POKE),
        "shift + left click threw something other than the bare left click's move"
    );
}

#[test]
fn right_click_still_guards_for_the_class_that_has_a_shield() {
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]);
    run(&mut w, 2, R, 0);
    assert!(
        matches!(w.players[0].action, Action::Guard { .. }),
        "the Bulwark stopped guarding when the Reaver took right click"
    );
}

// ---------------------------------------------------------------------------
// The press is not at the mercy of what she is already doing
// ---------------------------------------------------------------------------
//
// Every other button in this game is an attack, and an attack eaten by another
// move's frames is the game correctly telling you that you were busy. Right
// click is not an attack: it is where the second body stands, and the line
// between the two bodies is what the class *is* -- her way out of both the
// ordinary limits on where she can be and the ordinary limits on what she can
// reach.
//
// So it gets two things nothing else gets. It **cuts a recovery short**, the
// way the Champion's Rush does. And the press **outlives the frame it happened
// on**, because a mechanic that answers on one frame in twenty is a timing test
// standing in front of the class rather than the class.
//
// What it does not get is safety, and the last test here is the one that says
// so.

/// Run until the fighter is in `Action::Recovery`, or give up and say so.
fn run_to_recovery(w: &mut World) -> u16 {
    for _ in 0..120 {
        if let Action::Recovery { left, .. } = w.players[0].action {
            return left;
        }
        run(w, 1, 0, 0);
    }
    panic!("she never reached a recovery");
}

#[test]
fn right_click_cuts_a_recovery_short() {
    let mut w = duel();
    // Executioner is the longest commitment in the kit -- sixteen frames of
    // wind-up, four of blade, and twenty-six of standing there afterwards.
    run(&mut w, 2, E, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_COMMITTED),
        "`E` did not throw Executioner"
    );
    let left = run_to_recovery(&mut w);
    assert!(
        left > 12,
        "Executioner's recovery is only {left} frames by the time this test \
         reaches it; there is nothing left to cut short and the test proves \
         nothing"
    );

    run(&mut w, 1, R, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_MECHANIC),
        "right click did not cut Executioner's recovery short"
    );
    // And it is a real send, not just a state change: the second body actually
    // leaves.
    run(&mut w, 30, 0, 0);
    assert!(
        shadow(&w).is_out(),
        "the cancel threw the move but the shadow never went anywhere"
    );
}

#[test]
fn right_click_does_not_cut_a_startup_short() {
    // The other half of the rule, and the half that keeps whiff punishment
    // alive. Recovery is the part that is **over** -- the animation finishing,
    // not the decision. A cancel that reached back into the startup would let
    // her take a committed swing back after throwing it.
    let mut w = duel();
    run(&mut w, 2, E, 0);
    assert!(
        matches!(w.players[0].action, Action::Startup { .. }),
        "Executioner should still be winding up two frames in"
    );
    run(&mut w, 1, R, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_COMMITTED),
        "right click took Executioner back mid-wind-up"
    );
}

#[test]
fn right_click_does_not_cut_a_stun_short() {
    // Nor is it an escape from somebody else's punish. Being hit is the
    // opponent's reward and the one thing in the game that is *supposed* to
    // take the controls away.
    let mut w = duel();
    // Let the dummy hit her: she stands still, he turns round and pokes. The
    // half turn matters -- facing follows the look every frame you are free, so
    // a dummy left on the default aim swings at the wall behind him.
    let at_her = Input::aimed(L, Input::QUARTER_TURN * 2);
    for _ in 0..120 {
        w.advance([Input::default(), at_her]);
        if matches!(w.players[0].action, Action::HitStun { .. }) {
            break;
        }
    }
    let Action::HitStun { left } = w.players[0].action else {
        panic!("the dummy never managed to hit her");
    };
    assert!(left > 2, "the hitstun is too short to test anything");

    run(&mut w, 1, R, 0);
    match w.players[0].action {
        Action::HitStun { left: now } => assert_eq!(
            now,
            left - 1,
            "right click shortened her hitstun; the mechanic is not an escape \
             from being hit"
        ),
        other => panic!("right click cancelled her hitstun into {other:?}"),
    }
}

#[test]
fn a_right_click_thrown_during_a_swing_is_not_dropped() {
    // The complaint this was built for. Slash is the move she chains from
    // constantly; a right click pressed while it is still swinging used to
    // vanish, so sending the shadow meant waiting for the swing to end and
    // hitting a single frame.
    let mut w = duel();
    run(&mut w, 1, L, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_POKE),
        "left click did not throw Slash"
    );
    // One frame of right click, while the swing is still winding up, and then
    // the button is never touched again.
    run(&mut w, 1, R, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_POKE),
        "the press interrupted the swing instead of waiting for it"
    );

    // She is left alone from here. The order still comes out.
    for _ in 0..40 {
        run(&mut w, 1, 0, 0);
        if w.players[0].action.attack_kind() == Some(SLOT_MECHANIC) {
            break;
        }
    }
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_MECHANIC),
        "a right click pressed during Slash was dropped instead of remembered"
    );
    run(&mut w, 30, 0, 0);
    assert!(
        shadow(&w).is_out(),
        "the remembered press threw the move but the shadow never went out"
    );
}

#[test]
fn a_press_thrown_alongside_any_of_her_moves_still_comes_out() {
    // The number, stated as the thing it has to do, and **measured rather than
    // derived** -- how long a move takes to become cancellable is a fact about
    // the state machine's phase changes, and a formula for it here is a second
    // copy of that machine waiting to disagree with the first.
    //
    // Twelve frames was the first value, derived from Slash because Slash is
    // the move she throws most. It passed a test written the same way and
    // silently dropped `Q` then right click -- the lotus drag the kit calls the
    // class's biggest turn -- because the Guillotine takes sixteen frames to
    // become cancellable. Hence every move, and hence pressing rather than
    // arithmetic.
    for (name, bits, slot) in [
        ("Slash", L, SLOT_POKE),
        ("Executioner", E, SLOT_COMMITTED),
        ("Guillotine", Q, SLOT_SPECIAL),
    ] {
        let mut w = duel();
        run(&mut w, 1, bits, 0);
        assert_eq!(
            w.players[0].action.attack_kind(),
            Some(slot),
            "{name} did not come out"
        );
        // One frame of right click at the earliest moment it could possibly be
        // thrown -- the frame after the move started -- and then never again.
        run(&mut w, 1, R, 0);
        run(&mut w, 90, 0, 0);
        assert!(
            shadow(&w).is_out(),
            "a right click thrown at the start of {name} was dropped. The press \
             is remembered for {} frames, which is not long enough to reach the \
             frame {name} becomes cancellable.",
            t::shadow_buffer()
        );
    }
}

#[test]
fn the_lotus_drag_does_not_have_to_wait_for_the_lotus() {
    // The chain the class is named for: `Q` opens the flower wherever the
    // shadow is standing, right click drags it home and the six blades scythe
    // the length of the arena behind it. Two buttons, and the second one used
    // to have to be held back until the first had finished animating.
    let mut w = duel();
    run(&mut w, 2, R, 0);
    run(&mut w, 40, 0, 0);
    assert!(
        shadow(&w).is_out(),
        "the shadow never went out to open a lotus on"
    );

    run(&mut w, 1, Q, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_SPECIAL),
        "`Q` did not open the lotus"
    );
    // Three frames into the eruption -- deep inside a startup that is nine
    // frames long, nowhere near a frame she is free on.
    run(&mut w, 3, 0, 0);
    run(&mut w, 1, R, 0);

    let mut recalled = false;
    for _ in 0..60 {
        run(&mut w, 1, 0, 0);
        if w.players[0].action.attack_kind() == Some(SLOT_MECHANIC) {
            recalled = true;
            break;
        }
    }
    assert!(
        recalled,
        "the recall pressed inside the lotus was dropped, so the class's \
         biggest turn is still a timing test"
    );
    run(&mut w, 60, 0, 0);
    assert!(
        !shadow(&w).is_out(),
        "the recall came out but the shadow never came home"
    );
}

#[test]
fn being_hit_throws_the_remembered_press_away() {
    // The memory exists so her own kit cannot eat the mechanic. A stun is not
    // her own kit -- it is the opponent's reward, and the one thing in the game
    // meant to take the controls off you. Kept across one, a press from before
    // the hit would send the second body away on the frame she is most likely
    // to want it, on an input she gave in a situation that no longer exists.
    let mut w = duel();
    let at_her = Input::aimed(L, Input::QUARTER_TURN * 2);
    let settled = shadow(&w).doing;
    assert!(
        matches!(settled, Ghost::Attending),
        "the fixture did not start with the shadow at her heel"
    );

    // Wind the dummy up and wait for the blow to be **live**, so the press
    // below lands a frame or two before it connects rather than at whatever
    // moment the loop happens to reach. Timing this by luck is how the test
    // came to depend on a hit arriving inside the buffer.
    let mut swinging = false;
    for _ in 0..60 {
        w.advance([Input::default(), at_her]);
        if matches!(w.players[1].action, Action::Active { .. }) {
            swinging = true;
            break;
        }
    }
    assert!(swinging, "the dummy never got a blow out");

    // One frame of right click, into the teeth of it.
    w.advance([Input::new(R), at_her]);
    let mut hit = false;
    for _ in 0..8 {
        w.advance([Input::default(), at_her]);
        if w.players[0].action.stunned() {
            hit = true;
            break;
        }
    }
    assert!(hit, "the blow that was already live never landed");

    // Long enough for the stun to run out and the whole buffer with it.
    run(&mut w, 90, 0, 0);
    assert!(
        matches!(shadow(&w).doing, Ghost::Attending),
        "a right click pressed before she was hit sent the shadow anyway once \
         the stun ran out"
    );
}

#[test]
fn holding_right_click_orders_the_shadow_once() {
    // It is a press, not a held button. Held, it used to re-fire every time she
    // came free -- send, recall, send -- so where the mechanic ended up was a
    // function of how long a finger stayed down. That is the bug `E` had on
    // every class before it grew an edge, and it came back the day this class's
    // mechanic moved onto the mouse.
    let mut w = duel();
    let m = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    let whiff = m.whiff_cost() as u32;
    // Long enough to have toggled several times over.
    run(&mut w, whiff * 4, R, 0);
    assert!(
        shadow(&w).is_out(),
        "holding right click sent the shadow and then called it back again"
    );
}

#[test]
fn cutting_a_recovery_short_leaves_every_move_punishable() {
    // The relationship that keeps the cancel honest, and the reason it needs no
    // charge behind it the way the Champion's does.
    //
    // `feel.rs` pins that every attack in the game is minus on block, which is
    // the single property that makes blocking worth doing. A cancel is a way
    // around a recovery, so it is a way around that property unless the thing
    // she cancels *into* costs about what she skipped -- and this is the class
    // whose biggest move has the longest tail in the kit.
    //
    // So the same arithmetic `Move::on_block` uses, with her best case
    // substituted in: cut the recovery short on its very first frame, and spend
    // the whole of Send shadow instead of the rest of it. Cutting Executioner's
    // twenty-six frame tail to spend Send shadow's twenty-five buys her one
    // frame. That is a recovery traded for the mechanic, not frames bought
    // back, and if Send shadow were ever shortened enough to make it a discount
    // this is what would say so.
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    for m in sim::moves::table(Class::ShadowReaver) {
        let busy = (m.active as i32 - 1) + send.whiff_cost() as i32;
        let on_block = m.blockstun as i32 - busy;
        assert!(
            on_block < 0,
            "{}: {on_block:+} on block once its recovery is cut short into Send \
             shadow. Cancelling is meant to trade a recovery for the mechanic, \
             not to make a blocked move safe.",
            m.name
        );
    }
}

// ---------------------------------------------------------------------------
// It is instant, and in exchange it cannot save her
// ---------------------------------------------------------------------------
//
// Two rules that only make sense as a pair, and the pair is the whole balance
// of the mechanic.
//
// **It comes out on the frame it is asked for.** Anything slower reads as input
// lag rather than as a wind-up, because the shadow is not an attack you are
// committing to -- it is where your second body is, and a delay between asking
// and moving it feels like the game not listening.
//
// **So it may not take an enemy's frames.** Send shadow can already be thrown
// out of any move's recovery. Instant *and* interrupting, it would be the melee
// shine and a worse one: not even her own animations gate it, so she would
// never have to fully commit to anything. Throw the risky move, and when it
// goes wrong, recall through whoever is punishing you and take their attack off
// them. The shadow can hold for an opportune interrupt in case any of her own
// abilities put her in a bad position, and that is precisely the option the
// class must not have.
//
// It cuts, and it slows. What it may not do is hand her the exchange back.

#[test]
fn the_shadow_comes_out_on_the_frame_it_is_asked_for() {
    // One frame of startup, not none and not eight. None is not faster in any
    // way a player can feel -- it is one frame -- and it leaves the animation
    // with nothing to put the release on, so the body has to teleport into the
    // gesture. What matters is that it is *unreactable and unwaitable*: a
    // startup you could see coming would be a startup you could answer, and
    // this is a mechanic rather than an attack.
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    assert_eq!(
        send.startup, 1,
        "Send shadow is {} frames of startup. It is the one input in the kit \
         that is not an attack, and anything past a frame reads as input lag.",
        send.startup
    );

    // And that is what the player gets: press, and the second body is already
    // leaving.
    let mut w = duel();
    run(&mut w, 1, R, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_MECHANIC),
        "right click did not throw Send shadow on the press frame"
    );
    run(&mut w, 2, 0, 0);
    assert!(
        !matches!(shadow(&w).doing, Ghost::Attending),
        "the shadow had not begun to leave two frames after the press"
    );
}

#[test]
fn the_recall_does_not_take_an_enemys_frames() {
    // The load-bearing one. He is mid-swing when the shadow comes home through
    // him: he takes the cut, and he keeps swinging.
    //
    // Note that tuning `hitstun` to zero does **not** buy this. A hit writes
    // `Action::HitStun` over whatever the victim was doing whatever the number
    // is, so `HitStun { left: 0 }` is one frame of nothing and a cancelled
    // attack -- a full interrupt with a zero on it. See `Hit::interrupts`.
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-3), Fx::ZERO, Fx::from_int(6));
    run(&mut w, 20, 0, 0);

    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    tap(
        &mut w,
        R,
        down(25),
        (send.whiff_cost() + t::shadow_send_frames()) as u32,
    );
    // Stand him on the line home, and start him swinging.
    let out = shadow(&w).pos;
    let midway = V3::new(
        out.x.add(w.players[0].pos.x).mul(Fx::ratio(1, 2)),
        Fx::ZERO,
        out.z.add(w.players[0].pos.z).mul(Fx::ratio(1, 2)),
    );
    w.players[1].pos = midway;

    // He throws his special -- the slowest thing he has an input for, so there
    // is plenty of it left to be robbed of. (It was his committed move until
    // shift stopped being an attack modifier and left that slot with no button.)
    let his = sim::moves::get(Class::Bulwark, sim::state::SLOT_SPECIAL);
    w.advance([Input::default(), Input::new(Q)]);
    assert_eq!(
        w.players[1].action.attack_kind(),
        Some(sim::state::SLOT_SPECIAL),
        "the dummy never started his move"
    );

    // And she recalls through him.
    let before = hurt(&w);
    let mut caught_mid_move = false;
    let mut still_swinging = true;
    w.advance([Input::new(R), Input::default()]);
    for _ in 0..(his.whiff_cost() as usize) {
        w.advance([Input::default(), Input::default()]);
        if hurt(&w) > before {
            caught_mid_move = true;
        }
        if caught_mid_move && w.players[1].action.attack_kind().is_none() {
            still_swinging = false;
            break;
        }
    }

    assert!(
        caught_mid_move,
        "the recall never reached him, so this proves nothing"
    );
    assert!(
        still_swinging,
        "the recall cut him and took his move off him. Instant and interrupting \
         is the shine: she could throw anything, and recall out of the punish."
    );
}

#[test]
fn the_recall_still_cuts_and_slows() {
    // The other half, and the reason "no immediate effect" is not "no effect".
    // The mechanic's own description of itself is a second body dashing home
    // through anything in the way, *cutting and slowing it* -- what it gives up
    // is the interrupt, not the damage.
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    assert!(
        send.damage > 0,
        "the recall deals no damage, so the shadow comes home through people \
         without touching them"
    );
    assert_eq!(
        send.hitstun, 0,
        "the recall stuns, which is the interrupt arriving by the front door"
    );
    assert_eq!(
        send.knockback.raw(),
        0,
        "the recall shoves, which is taking somebody's position away in a move \
         that is not allowed to take their frames"
    );
}

#[test]
fn cutting_a_recovery_short_does_not_rescue_her_from_the_punish() {
    // The two rules meeting. She throws Executioner, it is blocked, and she
    // cancels the recovery into a recall aimed through him -- the exact escape
    // the philosophy forbids. He is punishing her before the shadow arrives and
    // he keeps punishing her through it.
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    let exec = sim::moves::get(Class::ShadowReaver, SLOT_COMMITTED);
    // Cancelling swaps the rest of a recovery for the whole of Send shadow, so
    // the frames only come back if Send shadow is shorter than what it cut.
    // That is survivable on its own; what would not be is buying an interrupt
    // with them as well.
    let cancelled_busy = (exec.active as i32 - 1) + send.whiff_cost() as i32;
    assert!(
        exec.blockstun as i32 - cancelled_busy < 0,
        "a blocked Executioner cancelled into Send shadow is {:+} on block -- \
         she is safe, and the cancel has become the escape it is not meant to be",
        exec.blockstun as i32 - cancelled_busy
    );
    // And the shadow it throws cannot close the gap by stopping him.
    assert!(
        send.hitstun == 0 && send.knockback.raw() == 0,
        "the move she cancels into can stun or shove, so she can buy her way \
         out of the punish she cancelled"
    );
}

#[test]
fn the_repeat_lockout_never_holds_up_the_recall() {
    // Where this class's mechanic meets the roster-wide rule that an ability
    // you have just thrown cannot be thrown again for thirty frames.
    //
    // The two would contradict each other if the lockout counted the recall as
    // a second use: the shadow is the class's escape, and an escape you have to
    // wait thirty frames for is the input lag the frame-1 startup exists to
    // avoid -- displaced from the first press to the one that matters. It does
    // not, because a press on a shadow that is already out is the second half
    // of the activation that was paid for when it was sent, not a new one.
    //
    // What the lockout does gate is **sending it again** once it is home, which
    // is the setup and not the escape. That is the ordinary rule and this class
    // has no argument with it.
    let mut w = duel();
    run(&mut w, 2, R, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_MECHANIC),
        "right click did not send the shadow"
    );
    // Out on the field, and her own recovery over.
    run(&mut w, 40, 0, 0);
    assert!(shadow(&w).is_out(), "the shadow never went out");
    assert!(
        w.players[0].locked_out(SLOT_MECHANIC),
        "the send armed no lockout at all, so this proves nothing about it \
         being ignored"
    );

    // And the recall answers anyway.
    run(&mut w, 1, R, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_MECHANIC),
        "the repeat lockout swallowed the recall. The shadow is the escape, \
         and an escape on a thirty-frame gate is the input lag the frame-1 \
         startup exists to avoid."
    );
    run(&mut w, 90, 0, 0);
    assert!(
        !shadow(&w).is_out(),
        "the recall came out but the shadow never came home"
    );
}
