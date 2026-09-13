//! The Elementalist's auto, as a line.
//!
//! The complaint these were written for: *"even when I aim upwards, the auto
//! attack still just follows along the ground."* It did, and then it did
//! something subtler and just as wrong — it flew along the look angle from her
//! chest, which is a ray parallel to the crosshair's and never meets it.
//!
//! So every test here aims the way a player does: **it puts the crosshair on
//! the thing and then shoots**, rather than typing in an angle. Where the
//! camera sits is the camera's business, and a test that hard-coded an angle
//! would be asserting something about the rig instead of about the aim.

use sim::class::{Class, Mechanic};
use sim::state::{Action, max_health};
use sim::tuning as t;
use sim::{Fx, Input, V3, World, aim};

const L: u16 = Input::LEFT;
const E: u16 = Input::MECHANIC;
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn metres(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}

fn at(x: f32, y: f32, z: f32) -> V3 {
    V3::new(metres(x), metres(y), metres(z))
}

/// An Elementalist against someone with no answer to anything, clear of the
/// raised platforms.
fn elementalist() -> World {
    let mut w = World::with_classes([Class::Elementalist, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[1].pos = at(2.0, 0.0, 8.0);
    w
}

fn run(w: &mut World, frames: u32, buttons: u16, pitch: i16) {
    for _ in 0..frames {
        w.advance([
            Input::looking_at(buttons, LOOK_RIGHT, pitch),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
}

/// Throw the auto at a given pitch and let it play out.
fn shoot(w: &mut World, pitch: i16) {
    run(w, 1, L, pitch);
    run(w, 24, 0, pitch);
}

fn beam_reach() -> Fx {
    sim::moves::get(Class::Elementalist, 0).reach
}

fn with_scene<T>(w: &World, ask: impl FnOnce(&aim::Scene) -> T) -> T {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    ask(&aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
    })
}

/// One tenth of a degree, in the wire's own unit.
fn tenths(v: i32) -> i16 {
    (v * 65536 / 3600) as i16
}

/// The pitch that puts the crosshair on something the predicate accepts.
///
/// This is the fixture the whole file rests on: *look at the thing*. It sweeps
/// from well below the horizon to well above it and returns the first angle
/// whose sighted point the caller is happy with.
fn crosshair_onto(w: &World, want: impl Fn(&aim::Sighted) -> bool) -> Option<i16> {
    (-700..800).find_map(|step| {
        let pitch = tenths(step);
        let look = Input::looking_at(0, LOOK_RIGHT, pitch);
        let seen = with_scene(w, |scene| aim::sight(0, look, beam_reach(), scene));
        want(&seen).then_some(pitch)
    })
}

/// The pitch that puts the crosshair on the other fighter.
///
/// Bodies are not on the aiming ray -- they came off it on 2026-09-13, because
/// a creature up close fills the screen and the reticle ends up on its chest
/// three metres above the thing you meant to hit. So "the crosshair is on them"
/// cannot be read off [`aim::Sighted`] any more, and the honest reading is the
/// one the player would give: **the line the reticle picks goes through them.**
///
/// Which is what the reticle looked like it promised all along. The ray stopping
/// on a body was only ever a way of guessing at this.
fn crosshair_onto_the_other_fighter(w: &World) -> Option<i16> {
    (-700..800).find_map(|step| {
        let pitch = tenths(step);
        let look = Input::looking_at(0, LOOK_RIGHT, pitch);
        let met = with_scene(w, |scene| {
            aim::first_along(
                aim::skillshot_path(0, look, beam_reach(), scene),
                Fx::ZERO,
                0,
                scene,
                aim::Targets::none().fighters(true),
            )
        });
        matches!(met, Some(aim::Contact::Fighter { index: 1, .. })).then_some(pitch)
    })
}

/// Is it on a stone standing at `stone`?
fn on_the_stone(stone: V3, seen: &aim::Sighted) -> bool {
    let span = t::structure_radius().add(metres(0.2));
    seen.met == aim::Met::Solid && seen.at.sub(stone).flat_len().raw() <= span.raw()
}

fn stones_of(w: &World) -> Vec<sim::class::Structure> {
    match w.players[0].mechanic {
        Mechanic::Structures(slots) => slots.iter().flatten().copied().collect(),
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Where the shot goes
// ---------------------------------------------------------------------------

#[test]
fn the_shot_hits_whoever_the_crosshair_is_on() {
    // The headline, and the one property a player can actually check: put the
    // reticle on somebody and the shot reaches them.
    let mut w = elementalist();
    let pitch = crosshair_onto_the_other_fighter(&w)
        .expect("no angle put the crosshair on the other fighter at all");
    shoot(&mut w, pitch);
    assert!(
        w.players[1].health < max_health(),
        "the crosshair was on them and the shot went somewhere else"
    );
}

#[test]
fn aiming_over_someone_shoots_over_them() {
    // The other half. The shot is a line at the angle it was fired, so lifting
    // the reticle off somebody lifts the shot off them too.
    let base = elementalist();
    let pitch = crosshair_onto_the_other_fighter(&base)
        .expect("no angle put the crosshair on the other fighter at all");

    let mut high = elementalist();
    shoot(&mut high, pitch + tenths(350));
    assert_eq!(
        high.players[1].health,
        max_health(),
        "a shot aimed thirty-five degrees above a fighter still hit them, so the aim \
         is decoration and the shot is following the ground"
    );
}

#[test]
fn aiming_up_reaches_someone_standing_above_her() {
    // Height is real in both directions. Someone on a platform is somewhere the
    // crosshair can be put, and putting it there has to be enough.
    let mut w = elementalist();
    w.players[0].pos = at(-1.0, 0.0, 0.0);
    // On the raised platform at the far end, which is a body's height up.
    w.players[1].pos = at(6.0, 1.5, 0.0);

    let pitch = crosshair_onto_the_other_fighter(&w)
        .expect("no angle put the crosshair on a fighter standing on the platform");
    shoot(&mut w, pitch);
    assert!(
        w.players[1].health < max_health(),
        "aiming up at a fighter standing above her did not reach them"
    );
}

#[test]
fn the_drawn_beam_ends_where_the_shot_stopped() {
    // The overlay draws `state::hitbox`, and for this move that is the line
    // itself. A beam stopped by a stone is as long as the gap to the stone; one
    // that met nothing runs to the edge of its range. Drawing either as the
    // other is how you end up unable to tell why a shot did not connect.
    let mut w = elementalist();
    run(&mut w, 2, E, 0); // a stone, a few metres ahead
    run(&mut w, 30, 0, 0);
    let stone = stones_of(&w)[0].at;

    let pitch = crosshair_onto(&w, |seen| on_the_stone(stone, seen))
        .expect("no angle put the crosshair on the stone");

    let mut seen = None;
    for _ in 0..30 {
        run(&mut w, 1, L, pitch);
        if let Some(hb) = sim::state::hitbox(&w.players[0]) {
            seen = Some(hb);
            break;
        }
    }
    let hb = seen.expect("the auto never put a volume out");
    assert!(hb.is_a_beam(), "the auto's volume is not drawn as a line");

    let drawn = hb.to.sub(hb.from).len();
    let to_stone = stone.sub(hb.from).flat_len();
    assert!(
        drawn.raw() <= to_stone.raw(),
        "the beam was drawn {} m long past a stone standing {} m away",
        drawn.to_f32_for_render(),
        to_stone.to_f32_for_render()
    );

    // And a shot with nothing in front of it draws its whole range.
    let mut clear = elementalist();
    clear.players[1].pos = at(13.0, 0.0, -13.0);
    let mut seen = None;
    for _ in 0..30 {
        run(&mut clear, 1, L, 0);
        if let Some(hb) = sim::state::hitbox(&clear.players[0]) {
            seen = Some(hb);
            break;
        }
    }
    let hb = seen.expect("the auto never put a volume out");
    let drawn = hb.to.sub(hb.from).len();
    assert!(
        beam_reach().sub(drawn).abs().raw() < metres(0.2).raw(),
        "a shot that met nothing was drawn {} m long against a range of {} m",
        drawn.to_f32_for_render(),
        beam_reach().to_f32_for_render()
    );
}

// ---------------------------------------------------------------------------
// What it does to a fighter
// ---------------------------------------------------------------------------

#[test]
fn the_shot_takes_the_charge_and_gives_the_frames_straight_back() {
    // Small damage, the wind-up they were partway through, and nothing else. No
    // hitstun, no stagger, no shove -- being poked by the auto costs you the
    // move you were holding and not your turn.
    let mut w = elementalist();
    let pitch = crosshair_onto_the_other_fighter(&w)
        .expect("no angle put the crosshair on the other fighter at all");

    let mut caught = None;
    let mut wound_up = false;
    for _ in 0..40 {
        // Player two winds up their slowest move once, then lets go, so what
        // the beam interrupts is not immediately thrown again.
        let them = if !wound_up && w.players[1].action == Action::Free {
            wound_up = true;
            Input::SHIFT | Input::LEFT
        } else {
            0
        };
        let before = w.players[1];
        w.advance([
            Input::looking_at(L, LOOK_RIGHT, pitch),
            Input::aimed(them, LOOK_LEFT),
        ]);
        if w.players[1].health < before.health && caught.is_none() {
            caught = Some((before.action, w.players[1]));
        }
    }
    let (was, now) = caught.expect("the auto never connected");

    assert!(
        matches!(was, Action::Startup { .. }),
        "fixture did not catch player two mid-charge; they were {was:?}"
    );
    assert_eq!(
        now.action,
        Action::Free,
        "the auto left its victim in {:?} -- it is meant to have no stagger at all",
        now.action
    );
    assert_eq!(
        (now.vel.x.raw(), now.vel.z.raw()),
        (0, 0),
        "the auto shoved its victim, which is knockback by another name"
    );
    assert!(
        max_health() - now.health <= max_health() / 10,
        "the auto took {} of {} health; it is meant to be small",
        max_health() - now.health,
        max_health()
    );
}

// ---------------------------------------------------------------------------
// What it does to a structure
// ---------------------------------------------------------------------------

#[test]
fn a_stone_goes_where_on_it_you_were_pointing() {
    // "Along the ground if aiming down, an impulse up into the air if aiming
    // up." With the shot leaving her chest and ending where the crosshair is,
    // that is one rule rather than two: the reticle on the upper part of a
    // stone is a line angled upward, and the stone takes that line.
    let raised = || {
        let mut w = elementalist();
        run(&mut w, 2, E, 0);
        run(&mut w, 30, 0, 0); // let it finish climbing out
        w
    };

    // The stone as it leaves, rather than wherever it has got to afterwards:
    // gravity is pulling on it from the next frame onward.
    let kicked = |w: &mut World, pitch: i16| {
        run(w, 1, L, pitch);
        for _ in 0..24 {
            run(w, 1, 0, pitch);
            let stone = stones_of(w)[0];
            if stone.launched {
                return Some(stone);
            }
        }
        None
    };

    let start = raised();
    let stone = stones_of(&start)[0].at;
    let hand = sim::aim::origin(start.players[0].pos).y;

    let mut up = raised();
    let high = crosshair_onto(&up, |seen| {
        on_the_stone(stone, seen) && seen.at.y.raw() > hand.add(metres(0.25)).raw()
    })
    .expect("no angle put the crosshair on the stone above the height she casts from");
    let lifted = kicked(&mut up, high).expect("the crosshair was on the stone and the shot missed");
    assert!(
        lifted.vel.y.raw() > 0,
        "the crosshair was above her hand on the stone's face and the stone went flat"
    );

    let mut level = raised();
    let low = crosshair_onto(&level, |seen| {
        on_the_stone(stone, seen) && seen.at.y.sub(hand).abs().raw() < metres(0.12).raw()
    })
    .expect("no angle put the crosshair level with her hand on the stone");
    let flat = kicked(&mut level, low).expect("the crosshair was on the stone and the shot missed");
    assert!(
        flat.vel.y.raw() <= 0,
        "a stone shot level left the ground, so where on it you point decides nothing"
    );
}

#[test]
fn a_shot_aimed_over_a_stone_passes_over_it() {
    // A structure is a solid, so it blocks the shot -- but only when the shot
    // actually meets it. Aiming above one is how you shoot past your own
    // terrain, and it could not be done while the trace ignored height.
    let mut w = elementalist();
    run(&mut w, 2, E, 0);
    run(&mut w, 30, 0, 0);
    shoot(&mut w, tenths(400));

    assert!(
        !stones_of(&w)[0].launched,
        "a shot aimed forty degrees above a stone still kicked it"
    );
}

// ---------------------------------------------------------------------------
// What it does to fire
// ---------------------------------------------------------------------------

#[test]
fn a_fire_bolt_carries_the_shot_past_the_beams_own_range() {
    // The beam is short. The bolt it lights is not -- that is the whole trade,
    // and the reason a pillar is worth putting between you and them.
    //
    // Fire is deliberately *not* on the aiming ray's list: you can see through
    // flame, so a pillar never steals the crosshair. She aims at the ground
    // beyond it, and the shot passes through the fire on its way.
    let mut w = elementalist();
    w.players[0].pos = at(-10.0, 0.0, 8.0);
    w.players[1].pos = at(10.0, 0.0, 8.0);

    run(&mut w, 2, Input::SPECIAL, 0); // plant a pillar ahead of her
    run(&mut w, 60, 0, 0); // and let her recover from planting it
    let pillar = w.effects.iter().flatten().next().copied();
    let pillar = pillar.expect("fixture planted no pillar");

    // Close until the pillar is well inside the beam's own range -- the pillar
    // reaches further than the beam does, and both are tuned -- and far enough
    // inside that there is still ground beyond it left to aim at.
    let want = beam_reach().div(Fx::from_int(2));
    let gap = |w: &World| pillar.pos.sub(w.players[0].pos).flat_len();
    for _ in 0..240 {
        if gap(&w).raw() < want.raw() {
            break;
        }
        run(&mut w, 1, Input::W, 0);
    }
    assert!(
        gap(&w).raw() < want.raw(),
        "fixture never got within half the beam's range of the pillar"
    );
    let reach = w.players[1].pos.sub(w.players[0].pos).flat_len();
    assert!(
        reach.raw() > beam_reach().raw(),
        "fixture put the target inside the beam's own range"
    );

    // The ground past the pillar, which is what she is looking at through it.
    let pitch = crosshair_onto(&w, |seen| {
        seen.met == aim::Met::Ground && seen.at.x.raw() > pillar.pos.x.raw()
    })
    .expect("no angle put the crosshair on the ground beyond the pillar");

    let before = w.players[1].health;
    run(&mut w, 1, L, pitch);
    let mut staggered = false;
    for _ in 0..120 {
        run(&mut w, 1, 0, pitch);
        staggered |= w.players[1].action.stunned();
    }

    assert!(
        w.players[1].health < before,
        "a bolt lit at a pillar never reached the fighter beyond it"
    );
    assert!(
        staggered,
        "the fire bolt is the part of the auto that staggers, and it did not"
    );
    assert!(
        w.bolts.iter().all(|b| b.is_none()),
        "the bolt carried on past the fighter it hit"
    );
}
