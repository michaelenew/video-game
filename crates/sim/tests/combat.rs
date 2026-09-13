//! Combat rules. These encode design decisions, so a failure here means either
//! a bug or a decision that changed without the documents changing.

use sim::class::{ALL_CLASSES, Class};
use sim::fixed::Fx;
use sim::math::V3;
use sim::moves;
use sim::state::{Action, Phase, Shield, max_health};
use sim::{Input, World};

const L: u16 = Input::LEFT;
const R: u16 = Input::RIGHT;
/// The class special -- Q -- and the class mechanic -- E.
const Q: u16 = Input::SPECIAL;
const E: u16 = Input::MECHANIC;
const SHIFT: u16 = Input::SHIFT;

/// Aim angles for two fighters looking at each other, which is how they spawn.
/// Movement and attacks are camera-relative now, so a test that does not say
/// where a fighter is looking is not saying what its buttons mean.
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    for _ in 0..frames {
        w.advance([Input::aimed(a, LOOK_RIGHT), Input::aimed(b, LOOK_LEFT)]);
    }
}

/// Walk player one into range so attacks connect. They start eight units
/// apart at seven units per second, so this needs more than a second.
fn engaged() -> World {
    let mut w = World::new();
    run(&mut w, 90, Input::W, 0);
    let gap = w.players[1]
        .pos
        .sub(w.players[0].pos)
        .flat_len()
        .to_f32_for_render();
    assert!(gap < 2.0, "fixture failed to close the distance: gap {gap}");
    w
}

#[test]
fn an_unguarded_hit_deals_damage() {
    let mut w = engaged();
    let before = w.players[1].health;
    run(&mut w, 20, L, 0);
    assert!(w.players[1].health < before, "no damage dealt");
}

#[test]
fn blocking_costs_no_health_but_does_cost_a_vulnerable_window() {
    // No chip damage, per defense.md. The cost is knockback plus stunlock.
    let mut w = engaged();
    run(&mut w, 30, L, R);
    assert_eq!(
        w.players[1].health,
        max_health(),
        "blocking took chip damage"
    );
    assert!(
        w.players[1].action.frames_left().max(u16::from(matches!(
            w.players[1].action,
            Action::BlockStun { .. }
        ))) > 0
            || matches!(w.players[1].action, Action::Guard { .. }),
        "block produced no state at all"
    );
}

#[test]
fn a_grapple_goes_through_guard() {
    // The answer to a turtling opponent. Without this, blocking is solved.
    let mut w = engaged();
    let before = w.players[1].health;
    run(&mut w, 40, Q, R);
    assert!(
        w.players[1].health < before,
        "grapple was blocked; guard is unbeatable"
    );
}

#[test]
fn a_dodge_has_invulnerable_frames_then_stops_having_them() {
    let mut w = World::new();
    w.advance([
        Input::aimed(Input::SHIFT | Input::W, LOOK_RIGHT),
        Input::aimed(0, LOOK_LEFT),
    ]);
    assert!(
        w.players[0].action.invulnerable(),
        "dodge did not start invulnerable"
    );
    run(&mut w, 20, Input::SHIFT | Input::W, 0);
    assert!(
        !w.players[0].action.invulnerable(),
        "dodge stayed invulnerable to the end -- nothing would ever punish it"
    );
}

#[test]
fn dodging_evades_an_attack_that_would_otherwise_land() {
    let mut baseline = engaged();
    run(&mut baseline, 20, L, 0);
    assert!(
        baseline.players[1].health < max_health(),
        "setup did not connect"
    );

    let mut dodged = engaged();
    run(&mut dodged, 20, L, Input::SHIFT | Input::S);
    assert_eq!(
        dodged.players[1].health,
        max_health(),
        "dodge failed to evade"
    );
}

#[test]
fn throwing_the_shield_gives_up_blocking() {
    let mut w = World::new();
    run(&mut w, 2, E, 0);
    assert!(
        !w.players[0].shield().unwrap().in_hand(),
        "shield did not leave the hand"
    );
    run(&mut w, 20, R, 0);
    assert!(
        !w.players[0].action.guarding(),
        "guarded without a shield in hand"
    );
}

#[test]
fn a_thrown_shield_plants_and_can_be_recalled() {
    let mut w = World::new();
    run(&mut w, 2, E, 0);
    run(&mut w, 60, 0, 0);
    assert!(
        matches!(w.players[0].shield(), Some(Shield::Planted { .. })),
        "shield never came to rest: {:?}",
        w.players[0].shield()
    );
    run(&mut w, 2, E, 0);
    run(&mut w, 90, 0, 0);
    assert!(
        w.players[0].shield().unwrap().in_hand(),
        "recall never returned the shield: {:?}",
        w.players[0].shield()
    );
}

#[test]
fn a_knockout_ends_the_round_and_awards_it() {
    let mut w = engaged();
    w.players[1].health = 1;
    run(&mut w, 30, L, 0);
    assert!(
        matches!(w.phase, Phase::RoundOver { winner: 0, .. }),
        "phase after knockout: {:?}",
        w.phase
    );
    assert_eq!(w.players[0].rounds_won, 1);
}

#[test]
fn the_next_round_starts_fresh() {
    let mut w = engaged();
    w.players[1].health = 1;
    run(&mut w, 30, L, 0);
    run(&mut w, 200, 0, 0);
    assert!(matches!(w.phase, Phase::Fighting), "round never restarted");
    assert_eq!(w.players[1].health, max_health(), "health did not reset");
    assert_eq!(w.players[0].rounds_won, 1, "round wins were lost on reset");
}

#[test]
fn walls_keep_players_inside_the_arena() {
    let mut w = World::new();
    run(&mut w, 400, Input::W, Input::W);
    for (i, p) in w.players.iter().enumerate() {
        let x = p.pos.x.to_f32_for_render();
        assert!(x.abs() < 14.5, "player {i} escaped the arena at x={x}");
    }
}

#[test]
fn players_can_stand_on_the_platforms() {
    let mut w = World::new();
    // Walk onto the left platform and jump.
    run(&mut w, 60, Input::S, 0);
    run(&mut w, 40, Input::SPACE, 0);
    run(&mut w, 60, Input::W, 0);
    let y = w.players[0].pos.y.to_f32_for_render();
    assert!(y >= 0.0, "fell through the floor to y={y}");
}

#[test]
fn crouching_ducks_an_overhead_but_not_a_mid() {
    // Slam is an overhead; Bash is a mid. Crouch beats one and loses to the
    // other, which is what stops it being a free defensive option.
    let mut standing = engaged();
    run(&mut standing, 45, L | SHIFT, 0);
    assert!(
        standing.players[1].health < max_health(),
        "setup did not connect while standing"
    );

    let mut mid = engaged();
    run(&mut mid, 20, L, Input::CROUCH);
    assert!(
        mid.players[1].health < max_health(),
        "a mid was ducked; crouch beats everything"
    );

    let mut ducked = engaged();
    run(&mut ducked, 45, L | SHIFT, Input::CROUCH);
    assert_eq!(
        ducked.players[1].health,
        max_health(),
        "crouch failed to duck the overhead"
    );
}

#[test]
fn crouching_is_slower_than_walking() {
    let mut walk = World::new();
    run(&mut walk, 30, Input::W, 0);
    let mut duck = World::new();
    run(&mut duck, 30, Input::W | Input::CROUCH, 0);
    assert!(
        duck.players[0].pos.x.raw() < walk.players[0].pos.x.raw(),
        "crouch-walking was not slower"
    );
}

#[test]
fn crouch_releases_when_the_key_does() {
    let mut w = World::new();
    run(&mut w, 5, Input::CROUCH, 0);
    assert!(w.players[0].crouching);
    run(&mut w, 5, 0, 0);
    assert!(!w.players[0].crouching, "stayed crouched after release");
}

// ---------------------------------------------------------------------------
// Aim -- movement and attacks resolve relative to where you are looking
// ---------------------------------------------------------------------------

/// Direction player one sets off in, given a button and an aim.
///
/// One frame, and the velocity rather than the displacement: walk far enough
/// and the arena walls have an opinion, which is correct behaviour but not what
/// is under test here.
fn walk_dir(bits: u16, aim: u16) -> (f32, f32) {
    let mut w = World::new();
    w.advance([Input::aimed(bits, aim), Input::aimed(0, LOOK_LEFT)]);
    let v = w.players[0].vel;
    let (x, z) = (v.x.to_f32_for_render(), v.z.to_f32_for_render());
    let len = (x * x + z * z).sqrt();
    assert!(len > 0.5, "did not set off at all: {len}");
    (x / len, z / len)
}

#[test]
fn forward_is_wherever_you_are_looking() {
    // The whole point of camera-relative movement. W is not a world direction.
    for eighth in 0..8u32 {
        let aim = (eighth * 65536 / 8) as u16;
        let turns = Input::aimed(0, aim).aim_turns();
        let want = sim::state::move_dir(turns, 0, 1);
        let (ux, uz) = walk_dir(Input::W, aim);
        let (wx, wz) = (want.x.to_f32_for_render(), want.z.to_f32_for_render());
        assert!(
            (ux - wx).abs() < 0.05 && (uz - wz).abs() < 0.05,
            "aim {aim}: walked ({ux:.2}, {uz:.2}), expected ({wx:.2}, {wz:.2})"
        );
    }
}

#[test]
fn strafing_is_sideways_not_forwards() {
    // D at aim zero should move along +Z, not +X. If these ever collapse into
    // each other, circle-strafing turns into walking in.
    let (fx, fz) = walk_dir(Input::W, 0);
    let (sx, sz) = walk_dir(Input::D, 0);
    assert!(
        fx > 0.9 && fz.abs() < 0.1,
        "forward went ({fx:.2}, {fz:.2})"
    );
    assert!(sz > 0.9 && sx.abs() < 0.1, "strafe went ({sx:.2}, {sz:.2})");
}

#[test]
fn you_attack_where_you_look() {
    // Aim away and the same button that would have landed a hit whiffs.
    let mut facing = engaged();
    run(&mut facing, 20, L, 0);
    assert!(
        facing.players[1].health < max_health(),
        "a poke at point blank did not connect"
    );

    let mut away = engaged();
    for _ in 0..20 {
        // Looking backwards, the opponent is behind you.
        away.advance([Input::aimed(L, LOOK_LEFT), Input::aimed(0, LOOK_LEFT)]);
    }
    assert_eq!(
        away.players[1].health,
        max_health(),
        "an attack aimed away from the opponent still hit them"
    );
}

#[test]
fn facing_locks_once_a_move_has_started() {
    // Otherwise the mouse drags a live hitbox around during its active frames,
    // and any whiff can be rescued after the fact by turning -- which takes
    // whiff punishment, most of the game, off the table.
    let mut w = engaged();
    w.advance([Input::aimed(L, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
    let committed = w.players[0].facing;
    // Now swing the aim hard while the move runs.
    for step in 1..12 {
        let spun = (step * 5000) as u16;
        w.advance([Input::aimed(0, spun), Input::aimed(0, LOOK_LEFT)]);
    }
    let drifted = committed.sub(w.players[0].facing).flat_len();
    assert!(
        drifted.to_f32_for_render() < 0.01,
        "facing moved {} during the move",
        drifted.to_f32_for_render()
    );
}

#[test]
fn a_guard_cannot_spin_to_cover_everything() {
    // Guard is an arc, and an arc you can flip instantly is a bubble.
    let mut w = engaged();
    run(&mut w, 5, R, 0);
    assert!(w.players[0].action.guarding(), "fixture is not guarding");
    let before = w.players[0].facing;
    // Whip the aim a full half turn and hold it there for a few frames.
    for _ in 0..4 {
        w.advance([
            Input::aimed(R, LOOK_RIGHT.wrapping_add(1 << 15)),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
    let turned = before.dot(w.players[0].facing).to_f32_for_render();
    assert!(
        turned > 0.5,
        "guard turned {turned} of the way round in four frames"
    );
}

#[test]
fn aim_survives_a_rollback() {
    // Aim is input, so it has to replay exactly like a button press does.
    let script: Vec<[Input; 2]> = (0..60u32)
        .map(|i| {
            [
                Input::aimed(if i % 7 == 0 { L } else { Input::W }, (i * 997) as u16),
                Input::aimed(0, LOOK_LEFT),
            ]
        })
        .collect();
    let mut live = World::new();
    for i in &script[..30] {
        live.advance(*i);
    }
    let snapshot = live.clone();
    for i in &script[30..] {
        live.advance(*i);
    }
    let mut replay = snapshot;
    for i in &script[30..] {
        replay.advance(*i);
    }
    assert_eq!(live.checksum(), replay.checksum(), "rollback diverged");
}

// ---------------------------------------------------------------------------
// How much a move hinders you
// ---------------------------------------------------------------------------

/// Horizontal speed after holding `bits` for `frames` from a standing start.
fn speed_after(bits: u16, frames: u32) -> f32 {
    let mut w = World::new();
    for _ in 0..frames {
        w.advance([Input::aimed(bits, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
    }
    let v = w.players[0].vel;
    (v.x.to_f32_for_render().powi(2) + v.z.to_f32_for_render().powi(2)).sqrt()
}

#[test]
fn poking_slows_you_without_stopping_you() {
    // Reported as jarring: every basic attack snapped you from a full walk to a
    // dead stop, and the basic attack is the move you throw constantly.
    let walking = speed_after(Input::W, 10);
    let poking = speed_after(Input::W | L, 10);
    assert!(walking > 6.0, "fixture is not walking: {walking}");
    assert!(
        poking > 1.0,
        "a poke still stops you dead: {poking} while walking is {walking}"
    );
    assert!(
        poking < walking * 0.85,
        "a poke costs nothing: {poking} against a walk of {walking}"
    );
}

#[test]
fn a_committed_move_still_roots_you() {
    // The other half of the rule. Rooting is what commitment *means*; if the
    // heavy moves stopped rooting, spacing would stop mattering.
    let slam = speed_after(Input::W | SHIFT | L, 20);
    assert!(
        slam < 0.3,
        "a committed move let you keep walking at {slam}"
    );
}

#[test]
fn coming_to_rest_inside_a_move_is_not_instant() {
    // The snap was the jarring part, not the rooting. A rooting move should
    // bleed the speed off over a few frames.
    let mut w = World::new();
    for _ in 0..10 {
        w.advance([
            Input::aimed(Input::W, LOOK_RIGHT),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
    w.advance([
        Input::aimed(SHIFT | L, LOOK_RIGHT),
        Input::aimed(0, LOOK_LEFT),
    ]);
    let v = w.players[0].vel;
    let first = (v.x.to_f32_for_render().powi(2) + v.z.to_f32_for_render().powi(2)).sqrt();
    assert!(
        first > 1.0,
        "velocity snapped to {first} on the first frame of the move"
    );

    for _ in 0..8 {
        w.advance([
            Input::aimed(SHIFT | L, LOOK_RIGHT),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
    let v = w.players[0].vel;
    let settled = (v.x.to_f32_for_render().powi(2) + v.z.to_f32_for_render().powi(2)).sqrt();
    assert!(settled < 0.3, "never came to rest: {settled}");
}

#[test]
fn releasing_a_direction_still_stops_you_crisply() {
    // The decay is for moves that root you, not for ordinary walking. Letting
    // go of W while free should stop you on the spot, or movement turns to ice.
    let mut w = World::new();
    for _ in 0..10 {
        w.advance([
            Input::aimed(Input::W, LOOK_RIGHT),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
    w.advance([Input::aimed(0, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
    let v = w.players[0].vel;
    let after = (v.x.to_f32_for_render().powi(2) + v.z.to_f32_for_render().powi(2)).sqrt();
    assert_eq!(after, 0.0, "walking now coasts");
}

// ---------------------------------------------------------------------------
// The hitbox the overlay draws
// ---------------------------------------------------------------------------

/// The buttons that throw this class's fastest move with a volume of its own.
///
/// Not every poke has one. A move whose radius is zero places something and
/// lets the thing it placed do the hitting -- the Blood mage's thrown blade is
/// the whole reason the distinction exists -- so a test about attack volumes
/// has to be pointed at a move that has one.
fn swings_with(class: sim::class::Class) -> u16 {
    if sim::moves::get(class, sim::state::SLOT_POKE).strikes() {
        L
    } else {
        SHIFT | L
    }
}

/// A world with player one mid-swing and player two parked at `gap` past the
/// far end of the attack volume, along the line the weapon lies on.
///
/// The **far end** rather than the centre, because most of what the game throws
/// is a line. The Elementalist's auto is a beam out of her chest, and a
/// defender standing at the middle of it is inside it however far away they are
/// put; the Champion's weapons are capsules that sweep, and the only boundary a
/// capsule has that does not depend on which way you approach it is the one off
/// its end. For a bubble the two points are the same, so it means that too.
fn swinging_at(class: sim::class::Class, gap_factor: f32) -> World {
    use sim::state::hitbox;

    let button = swings_with(class);
    let mut w = World::with_classes([class, class]);
    // Out of the way while the swing starts, so nothing connects early.
    w.players[1].pos = sim::V3::new(Fx::from_int(30), w.players[1].pos.y, Fx::ZERO);
    let held = [Input::aimed(button, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)];

    // The volume the **next** frame will test with, found by stepping a copy.
    //
    // Reading the current one and then advancing was fine while every attack
    // was a disc that sat still for its whole active window. A swing does not:
    // it has moved by the time the hit is resolved, so the box to park a
    // defender against is the one the hit test is about to use, not the one on
    // screen a frame earlier.
    let mut found = None;
    for _ in 0..40 {
        let mut peek = w.clone();
        peek.advance(held);
        if let Some(next) = hitbox(&peek.players[0]) {
            found = Some(next);
            break;
        }
        w.advance(held);
    }
    let hb = found.expect("never reached an active frame");

    // Exactly the threshold the hit test uses: the attack radius plus the
    // defender's body radius, which is why the overlay draws both cylinders.
    let threshold = (hb.radius.to_f32_for_render()
        + sim::tuning::body_radius().to_f32_for_render())
        * gap_factor;
    let step = Fx::ratio((threshold * 1000.0) as i32, 1000);
    // Out along the volume's own direction. For a bubble that is the body's
    // facing; for anything with a length of its own it is that line, which may
    // be pointing anywhere at all -- the Elementalist's beam ends wherever the
    // crosshair was, and the Champion's weapons sweep through an arc.
    let along = if hb.is_a_beam() {
        hb.to.sub(hb.from).normalized()
    } else {
        w.players[0].facing
    };
    let spot = hb.to.add(along.scale(step));
    // Placed by the middle of the body rather than by the feet, so "just past
    // the end of the volume" means the same thing at any height.
    let half = sim::tuning::body_height().div(Fx::from_int(2));
    w.players[1].pos = sim::V3::new(spot.x, spot.y.sub(half).max(Fx::ZERO), spot.z);
    w.advance(held);
    w
}

#[test]
fn a_move_with_no_volume_draws_nothing_and_touches_nobody() {
    // The other half of the rule below. An ability that puts something into the
    // world and lets it do the hitting must not *also* poke whoever happens to
    // be standing next to the caster: the blade is out there, and the caster is
    // here with empty hands.
    use sim::state::hitbox;
    let mut w = World::with_classes([sim::class::Class::BloodMage; 2]);
    w.players[1].pos = w.players[0].pos;
    let before = w.players[1].health;
    for _ in 0..40 {
        w.advance([Input::aimed(L, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        assert!(
            hitbox(&w.players[0]).is_none(),
            "the thrown blade drew a hitbox on the caster"
        );
    }
    assert!(
        w.players[1].health < before,
        "fixture: the blade never came round to hit anybody"
    );
}

#[test]
fn the_drawn_hitbox_is_the_one_that_hits() {
    // The overlay draws `state::hitbox`, and the hit test uses it too. This
    // pins that they agree at the boundary, for every class -- including the
    // Champion, whose swings are capsules that move over their active frames
    // and which an overlay rebuilding the box from the move table on its own
    // would get wrong on every frame but the first.
    for class in ALL_CLASSES {
        let inside = swinging_at(class, 0.8);
        assert!(
            inside.players[1].health < max_health(),
            "{class:?}: a defender well inside the drawn box was not hit"
        );
        let outside = swinging_at(class, 1.3);
        assert_eq!(
            outside.players[1].health,
            max_health(),
            "{class:?}: a defender well outside the drawn box was hit anyway"
        );
    }
}

#[test]
fn nothing_is_drawn_when_no_attack_is_out() {
    // If the overlay could draw a box outside active frames it would be
    // claiming a threat that does not exist.
    use sim::state::hitbox;
    let mut w = World::new();
    assert!(hitbox(&w.players[0]).is_none(), "idle fighter has a hitbox");

    let mut seen_active = false;
    let mut seen_gap = false;
    for _ in 0..40 {
        w.advance([Input::aimed(L, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        let out = hitbox(&w.players[0]).is_some();
        let active = matches!(w.players[0].action, sim::state::Action::Active { .. });
        assert_eq!(out, active, "hitbox and active frames disagree");
        seen_active |= active;
        seen_gap |= seen_active && !active;
    }
    assert!(seen_active && seen_gap, "fixture never covered both states");
}

#[test]
fn spent_distinguishes_a_whiff_from_a_landed_hit() {
    // The overlay dims a volume that is still out but can no longer hit.
    //
    // Worth being precise about what that shows. At point blank a move
    // connects on the very frame its box appears, so a landed hit is drawn
    // dim for every frame you can see it. The bright state is therefore what a
    // *whiff* looks like: out, and still looking for someone. That is the more
    // useful reading anyway -- "did that touch anything" is the question you
    // are asking when you step through the frames.
    use sim::state::hitbox;

    let mut whiff = World::new();
    let mut whiff_live = 0;
    let mut whiff_spent = 0;
    for _ in 0..30 {
        whiff.advance([Input::aimed(L, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        if let Some(hb) = hitbox(&whiff.players[0]) {
            if hb.spent {
                whiff_spent += 1;
            } else {
                whiff_live += 1;
            }
        }
    }
    assert!(
        whiff_live > 0,
        "a whiffing attack never showed a live volume"
    );
    assert_eq!(
        whiff_spent, 0,
        "an attack that hit nothing was marked as having spent its hit"
    );

    let mut landed = engaged();
    let mut landed_spent = 0;
    for _ in 0..30 {
        landed.advance([Input::aimed(L, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        if hitbox(&landed.players[0]).is_some_and(|hb| hb.spent) {
            landed_spent += 1;
        }
    }
    assert!(
        landed_spent > 0,
        "a connecting attack never marked its volume spent"
    );
}

// ---------------------------------------------------------------------------
// Jump, dodge, airdodge
// ---------------------------------------------------------------------------

fn press(w: &mut World, bits: u16, frames: u32) {
    for _ in 0..frames {
        w.advance([Input::aimed(bits, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
    }
}

#[test]
fn space_always_jumps_even_while_moving() {
    // The whole complaint: space plus a direction used to dodge, so pressing
    // jump while walking -- which is most of the time -- did not jump.
    for bits in [
        Input::SPACE,
        Input::SPACE | Input::W,
        Input::SPACE | Input::A,
        Input::SPACE | Input::S,
        Input::SPACE | Input::D,
    ] {
        let mut w = World::new();
        press(&mut w, bits, 2);
        assert!(
            !w.players[0].grounded && w.players[0].vel.y.raw() > 0,
            "space with {bits:b} did not leave the ground"
        );
        assert!(
            !matches!(w.players[0].action, Action::Dodge { .. }),
            "space with {bits:b} produced a dodge"
        );
    }
}

#[test]
fn shift_plus_a_direction_dodges() {
    let mut w = World::new();
    press(&mut w, Input::SHIFT | Input::W, 1);
    assert!(
        matches!(w.players[0].action, Action::Dodge { .. }),
        "shift plus a direction did not dodge: {:?}",
        w.players[0].action
    );
    assert!(w.players[0].grounded, "a grounded dodge left the ground");
}

#[test]
fn a_click_beats_a_dodge_when_shift_is_held() {
    // Shift is overloaded: with a click it is the stronger version of that
    // attack, with only a direction it is a dodge. The click has to win, or
    // every committed move thrown while walking would come out as a dodge.
    let mut w = World::new();
    press(&mut w, Input::SHIFT | Input::W | L, 1);
    assert!(
        matches!(w.players[0].action, Action::Startup { .. }),
        "shift+direction+click dodged instead of attacking: {:?}",
        w.players[0].action
    );
}

#[test]
fn you_can_airdodge_once_per_jump() {
    let mut w = World::new();
    press(&mut w, Input::SPACE, 2);
    assert!(!w.players[0].grounded, "fixture never left the ground");

    press(&mut w, Input::SHIFT | Input::W, 1);
    assert!(
        matches!(w.players[0].action, Action::Dodge { .. }),
        "could not airdodge"
    );
    let first = w.players[0].vel.x.to_f32_for_render();
    assert!(first > 5.0, "the airdodge carried no speed: {first}");

    assert!(w.players[0].air_dodged, "the airdodge was not recorded");
}

#[test]
fn a_second_airdodge_in_the_same_jump_is_refused() {
    // Tested through the flag rather than by waiting out the first dodge:
    // wiping vertical speed means you land before a second attempt is even
    // possible, so a timing-based fixture would pass without exercising the
    // gate at all.
    let mut w = World::new();
    press(&mut w, Input::SPACE, 2);
    assert!(!w.players[0].grounded, "fixture never left the ground");
    w.players[0].air_dodged = true;

    press(&mut w, Input::SHIFT | Input::W, 1);
    assert!(
        !matches!(w.players[0].action, Action::Dodge { .. }),
        "airdodged twice in one jump, which is flight"
    );
}

#[test]
fn landing_restores_the_airdodge() {
    let mut w = World::new();
    press(&mut w, Input::SPACE, 2);
    press(&mut w, Input::SHIFT | Input::W, 1);
    assert!(
        w.players[0].air_dodged,
        "airdodge was not recorded as spent"
    );
    press(&mut w, 0, 120);
    assert!(w.players[0].grounded, "never came back down");
    assert!(
        !w.players[0].air_dodged,
        "the airdodge was not restored on landing"
    );
}

#[test]
fn an_airdodge_is_not_a_second_jump() {
    // It wipes vertical speed rather than adding to it, so it commits you
    // sideways and can never be used to climb.
    let mut w = World::new();
    press(&mut w, Input::SPACE, 2);
    let apex = w.players[0].pos.y;
    press(&mut w, Input::SHIFT | Input::W, 1);
    assert!(
        w.players[0].vel.y.raw() <= 0,
        "the airdodge left upward momentum"
    );
    press(&mut w, 0, 10);
    assert!(
        w.players[0].pos.y.raw() < apex.raw() + Fx::from_int(2).raw(),
        "the airdodge gained height"
    );
}

/// Apex height and airtime in frames for a jump held for `hold` frames.
fn jump_profile(class: sim::class::Class, hold: u32) -> (f32, u32) {
    let mut w = World::with_classes([class, class]);
    let mut apex = 0.0f32;
    let mut frames = 0;
    for i in 0..240 {
        let bits = if i < hold { Input::SPACE } else { 0 };
        w.advance([Input::aimed(bits, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        let p = &w.players[0];
        apex = apex.max(p.pos.y.to_f32_for_render());
        if i > 0 && p.grounded {
            frames = i;
            break;
        }
    }
    (apex, frames)
}

#[test]
fn holding_the_jump_button_goes_higher() {
    // Variable jump height. Without it there is one jump arc, and every
    // approach through the air is the same approach.
    for class in ALL_CLASSES {
        let (short, short_time) = jump_profile(class, 1);
        let (full, full_time) = jump_profile(class, 60);
        assert!(
            full > short * 1.35,
            "{}: full hop {full:.2}m is barely taller than the short hop {short:.2}m",
            class.name()
        );
        assert!(
            full_time > short_time,
            "{}: holding the button did not buy any airtime",
            class.name()
        );
    }
}

/// Frames before the fighter's feet clear a standing opponent's head.
fn frames_to_clear_a_head(class: sim::class::Class) -> u32 {
    let head = sim::tuning::body_height();
    let mut w = World::with_classes([class, class]);
    for i in 0..200 {
        w.advance([
            Input::aimed(Input::SPACE, LOOK_RIGHT),
            Input::aimed(0, LOOK_LEFT),
        ]);
        if w.players[0].pos.y.raw() > head.raw() {
            return i;
        }
    }
    u32::MAX
}

/// How far sideways a fighter gets by strafing for a whole jump.
fn strafe_across_a_jump(class: sim::class::Class) -> f32 {
    let mut w = World::with_classes([class, class]);
    let start = w.players[0].pos;
    for i in 0..200u32 {
        let bits = if i < 30 {
            Input::SPACE | Input::D
        } else {
            Input::D
        };
        w.advance([Input::aimed(bits, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        if i > 2 && w.players[0].grounded {
            break;
        }
    }
    w.players[0].pos.sub(start).flat_len().to_f32_for_render()
}

// The four tests below replaced a single assertion that a full hop lasted
// between 40 and 110 frames. That was a number with no argument behind it: it
// caught an order-of-magnitude mistake and otherwise just went off whenever
// someone tuned the jump, which is the opposite of useful. What a jump has to
// be is four separate things, and each of them is worth stating.

#[test]
fn a_short_hop_is_a_fraction_of_a_full_one() {
    // The whole point of a variable jump. With only the gravity sustain
    // separating them, the short hop was *exactly* the sustain multiplier of
    // the full one -- about two thirds, which is not a second option, it is the
    // same jump slightly lower. A platform fighter wants a quarter or less.
    for class in ALL_CLASSES {
        let (short, _) = jump_profile(class, 1);
        let (full, _) = jump_profile(class, 60);
        let ratio = short / full;
        assert!(
            (0.12..=0.33).contains(&ratio),
            "{}: a short hop is {:.0}% of a full one ({short:.1}m against {full:.1}m)",
            class.name(),
            ratio * 100.0
        );
    }
}

#[test]
fn a_short_hop_is_long_enough_to_throw_an_aerial() {
    // It is deliberately too low to cross over someone -- that is what the full
    // hop is for -- so what makes it worth having is that an aerial fits inside
    // it. If the fastest poke cannot start and finish before you land, the
    // short hop is just a stumble.
    let shortest = ALL_CLASSES
        .iter()
        .map(|c| sim::moves::get(*c, 0).whiff_cost())
        .min()
        .unwrap() as u32;
    for class in ALL_CLASSES {
        let (_, airtime) = jump_profile(class, 1);
        assert!(
            airtime > shortest,
            "{}: a short hop lasts {airtime} frames, less than the {shortest} a poke needs",
            class.name()
        );
    }
}

#[test]
fn a_full_hop_reaches_platform_fighter_heights() {
    // Smash characters jump four or more times their own height. Every class
    // should manage twice, and the floaty end should reach the four that makes
    // the genre's verticality read as generous rather than as a hop.
    let head = sim::tuning::body_height().to_f32_for_render();
    let mut tallest: f32 = 0.0;
    for class in ALL_CLASSES {
        let (apex, _) = jump_profile(class, 60);
        assert!(
            apex > head * 2.0,
            "{}: a full hop reaches {:.1} body heights",
            class.name(),
            apex / head
        );
        tallest = tallest.max(apex / head);
    }
    assert!(
        tallest >= 4.0,
        "nobody jumps four body heights; the tallest manages {tallest:.1}"
    );
}

#[test]
fn the_rise_is_fast_enough_not_to_be_a_sitting_duck() {
    // Vulnerability while jumping is about how long you spend at head height
    // where you can be hit, not about how long you are airborne in total --
    // which is exactly why the airtime ceiling this replaced was measuring the
    // wrong thing.
    for class in ALL_CLASSES {
        let frames = frames_to_clear_a_head(class);
        assert!(
            frames <= 12,
            "{}: takes {frames} frames to get above a standing fighter",
            class.name()
        );
    }
}

#[test]
fn strafing_can_carry_you_clear_of_where_you_took_off() {
    // A jump you cannot steer out of is a commitment with no counterplay of its
    // own. Two body widths is enough to leave a melee hitbox that was aimed
    // where you started.
    let clear = sim::tuning::body_radius().to_f32_for_render() * 4.0;
    for class in ALL_CLASSES {
        let across = strafe_across_a_jump(class);
        assert!(
            across > clear,
            "{}: strafing a whole jump moves you {across:.1}m, under the {clear:.1}m needed",
            class.name()
        );
    }
}

#[test]
fn a_full_hop_can_still_be_punished() {
    // Slow enough that an opponent can see it, react, and land something before
    // you are back down. Measured on the *full* hop, because that is the
    // committed option: a short hop being hard to react to is correct -- it is
    // the fast, low-commitment one, and platform fighters lean on exactly that.
    //
    // A floor, with deliberately no ceiling. How high you go is a design choice
    // and a test should not be quietly capping it.
    let fastest = ALL_CLASSES
        .iter()
        .map(|c| {
            let m = sim::moves::get(*c, 0);
            m.startup + m.active
        })
        .min()
        .unwrap() as u32;
    let needed = sim::tuning::HUMAN_REACTION_FRAMES as u32 + fastest;
    for class in ALL_CLASSES {
        let (_, airtime) = jump_profile(class, 60);
        assert!(
            airtime > needed,
            "{}: a full hop lasts {airtime} frames, too short to react to and punish ({needed})",
            class.name()
        );
    }
}

#[test]
fn releasing_the_jump_button_is_final() {
    // Otherwise the sustain is a button to mash rather than a decision, and
    // jump height stops being something you chose.
    // Measured only until the fighter lands. Running longer catches the *next*
    // jump, which a still-held button starts immediately -- the first draft of
    // this test failed on exactly that and was measuring two jumps against one.
    let apex_of = |resurrect: bool| {
        let mut w = World::new();
        let mut apex = 0.0f32;
        for i in 0..120 {
            let held = i < 4 || (resurrect && i > 6);
            w.advance([
                Input::aimed(if held { Input::SPACE } else { 0 }, LOOK_RIGHT),
                Input::aimed(0, LOOK_LEFT),
            ]);
            if i > 0 && w.players[0].grounded {
                break;
            }
            apex = apex.max(w.players[0].pos.y.to_f32_for_render());
        }
        apex
    };
    let apex = (apex_of(false), apex_of(true));
    assert!(
        (apex.0 - apex.1).abs() < 0.01,
        "re-pressing jump resurrected a cut jump: {:.2}m against {:.2}m",
        apex.1,
        apex.0
    );
}

/// Horizontal speed and heading after jumping forward and then holding `bits`.
fn air_drift(bits: u16, frames: u32) -> (f32, f32) {
    let mut w = World::new();
    // Walk up to speed, then take off carrying it.
    press(&mut w, Input::W, 12);
    press(&mut w, Input::SPACE | Input::W, 1);
    press(&mut w, bits, frames);
    let v = w.players[0].vel;
    let (x, z) = (v.x.to_f32_for_render(), v.z.to_f32_for_render());
    ((x * x + z * z).sqrt(), z.atan2(x).to_degrees())
}

#[test]
fn holding_forward_in_the_air_buys_nothing() {
    // The Quake property. Input accelerates you along the component of your
    // motion you have *not* already spent, so pointing where you are already
    // going has nothing left to add.
    let (fwd_speed, fwd_heading) = air_drift(Input::W, 20);
    let (drift_speed, drift_heading) = air_drift(0, 20);
    assert!(
        (fwd_speed - drift_speed).abs() < 0.05,
        "holding forward added {:.2} of speed",
        fwd_speed - drift_speed
    );
    assert!(
        (fwd_heading - drift_heading).abs() < 0.5,
        "holding forward turned you {:.1} degrees",
        fwd_heading - drift_heading
    );
}

#[test]
fn strafing_redirects_you_when_holding_forward_cannot() {
    // Same twenty frames, same fighter, same speed. The only difference is
    // which direction the input points relative to the motion already there.
    let baseline = air_drift(0, 20);
    let forward = air_drift(Input::W, 20);
    let strafe = air_drift(Input::D, 20);

    let moved = |a: (f32, f32), b: (f32, f32)| (a.1 - b.1).abs();
    assert!(
        moved(strafe, baseline) > 5.0,
        "strafing turned you only {:.1} degrees",
        moved(strafe, baseline)
    );
    assert!(
        moved(strafe, baseline) > moved(forward, baseline) * 10.0,
        "strafing is not meaningfully better than holding forward"
    );
}

#[test]
fn turning_while_strafing_compounds_the_redirect() {
    // Where the skill actually lives. A single strafe spends a fixed budget and
    // stops; turning the camera as you hold it keeps redefining which direction
    // counts as "perpendicular", so the budget refills against the new heading.
    // A player who does not turn gets one nudge; a player who does can carve.
    let fixed = air_drift(Input::D, 30).1;

    let mut w = World::new();
    press(&mut w, Input::W, 12);
    press(&mut w, Input::SPACE | Input::W, 1);
    for i in 0..30 {
        // Sweep the aim while holding the strafe, which is the technique.
        let aim = (i as u32 * 380) as u16;
        w.advance([Input::aimed(Input::D, aim), Input::aimed(0, LOOK_LEFT)]);
    }
    let v = w.players[0].vel;
    let carved =
        v.z.to_f32_for_render()
            .atan2(v.x.to_f32_for_render())
            .to_degrees();
    let baseline = air_drift(0, 30).1;

    assert!(
        (carved - baseline).abs() > (fixed - baseline).abs() * 1.5,
        "turning while strafing bought nothing: carved {:.1}, fixed {:.1}, baseline {:.1}",
        carved,
        fixed,
        baseline
    );
}

#[test]
fn momentum_carries_in_the_air() {
    // Letting go of the stick mid-jump should not stop you dead. This is the
    // difference between a jump being a commitment and a jump being a hover.
    let (drift, _) = air_drift(0, 20);
    assert!(drift > 5.0, "momentum evaporated in the air: {drift:.2}");
}

#[test]
fn air_speed_has_a_ceiling() {
    // A deliberate divergence from Source. A player who can cross the whole
    // arena from anywhere has removed spacing from the game.
    let cap = (sim::tuning::move_speed().to_f32_for_render()
        * sim::tuning::air_speed_cap().to_f32_for_render())
        + 0.1;
    // Alternate strafes, which is how you build speed if it can be built.
    let mut w = World::new();
    press(&mut w, Input::W, 12);
    press(&mut w, Input::SPACE | Input::W, 1);
    for i in 0..60 {
        let bits = if (i / 4) % 2 == 0 { Input::D } else { Input::A };
        press(&mut w, bits, 1);
        let v = w.players[0].vel;
        let speed = (v.x.to_f32_for_render().powi(2) + v.z.to_f32_for_render().powi(2)).sqrt();
        assert!(
            speed <= cap,
            "air speed reached {speed:.2}, over the {cap:.2} cap"
        );
    }
}

#[test]
fn an_aerial_slows_the_rise_rather_than_deleting_it() {
    // Setting vertical speed to zero read as the game snatching the jump out
    // from under you mid-rise. The control over jump height is worth keeping;
    // it has to arrive as a slowing.
    let mut w = World::new();
    press(&mut w, Input::SPACE, 1);
    let rising = w.players[0].vel.y;
    assert!(rising.raw() > 0, "fixture is not rising");

    press(&mut w, L, 1);
    let after = w.players[0].vel.y;
    assert!(
        after.raw() > 0,
        "the attack deleted the rise outright: {}",
        after.to_f32_for_render()
    );
    assert!(
        after.raw() < rising.raw(),
        "the attack did not slow the rise at all"
    );

    // And it keeps bleeding, rather than holding at whatever it landed on.
    press(&mut w, 0, 3);
    assert!(
        w.players[0].vel.y.raw() < after.raw(),
        "the rise stopped bleeding after one frame"
    );
}

/// Horizontal speed one frame after `bits`, having just jumped.
///
/// Compared against the *same direction without the attack*, because holding a
/// direction in the air accelerates you anyway — the first version of these
/// tests measured against no input at all and was reading ordinary air control
/// as the shove.
fn airborne_nudge(bits: u16) -> i32 {
    let mut w = World::new();
    press(&mut w, Input::SPACE, 1);
    press(&mut w, bits, 1);
    w.players[0].vel.x.raw()
}

#[test]
fn a_poke_in_the_air_shoves_you_the_way_you_are_holding() {
    // What makes attacking part of air movement rather than a pause in it.
    let drifting = airborne_nudge(Input::W);
    let shoved = airborne_nudge(L | Input::W);
    assert!(
        shoved > drifting,
        "a poke in the air added nothing over holding the direction: {shoved} against {drifting}"
    );
}

#[test]
fn the_shove_needs_a_direction_and_belongs_to_the_poke() {
    // No direction held, no push -- it is a shove *somewhere*, not free speed.
    assert_eq!(
        airborne_nudge(L),
        airborne_nudge(0),
        "a poke with no direction still pushed"
    );

    // And the committed move gets none: it is already a commitment, and one
    // that also repositioned you would be strictly better than a poke.
    assert_eq!(
        airborne_nudge(SHIFT | L | Input::W),
        airborne_nudge(Input::W),
        "the committed move repositioned you as well"
    );
}

#[test]
fn the_shove_cannot_break_the_air_speed_cap() {
    // Otherwise poking repeatedly is a way across the arena, and spacing stops
    // meaning anything.
    let cap = (sim::tuning::move_speed().to_f32_for_render()
        * sim::tuning::air_speed_cap().to_f32_for_render())
        + 0.1;
    let mut w = World::new();
    press(&mut w, Input::W, 12);
    press(&mut w, Input::SPACE | Input::W, 1);
    for _ in 0..20 {
        press(&mut w, L | Input::W, 1);
        press(&mut w, Input::W, 2);
        let v = w.players[0].vel;
        let speed = (v.x.to_f32_for_render().powi(2) + v.z.to_f32_for_render().powi(2)).sqrt();
        assert!(
            speed <= cap,
            "poking reached {speed:.2}, over the {cap:.2} cap"
        );
    }
}

#[test]
fn an_aerial_hangs_you_where_you_are() {
    // Per-move, and the reason the air is worth attacking from at all.
    let mut plain = World::new();
    let mut striking = World::new();
    press(&mut plain, Input::SPACE, 1);
    press(&mut striking, Input::SPACE, 1);
    // Let both rise, then one of them attacks.
    press(&mut plain, 0, 20);
    press(&mut striking, 0, 19);
    press(&mut striking, L, 1);
    assert!(
        striking.players[0].air_stall > 0,
        "an aerial armed no hang at all"
    );
    press(&mut plain, 0, 6);
    press(&mut striking, 0, 6);
    assert!(
        striking.players[0].pos.y.raw() > plain.players[0].pos.y.raw(),
        "the aerial did not delay the fall"
    );
}

#[test]
fn a_grounded_attack_does_not_hang_anything() {
    let mut w = World::new();
    press(&mut w, L, 1);
    assert_eq!(
        w.players[0].air_stall, 0,
        "a grounded attack armed an air stall"
    );
}

// ---------------------------------------------------------------------------
// Stun
// ---------------------------------------------------------------------------
//
// Every point of damage freezes, stuns, interrupts and shoves. See
// `docs/design/stun.md` for why the rules are what they are; these check the
// game actually does it.

/// Two fighters nose to nose with clear floor behind the victim.
///
/// `engaged()` walks them together on the spawn marks, which happen to sit at
/// the edge of a platform -- fine for "did it connect", useless for "how far
/// did it send them", because the platform stops the knockback dead and the
/// test then measures the platform. Anything about distance uses this instead.
///
/// The victim is always the middleweight. A class measured against itself is
/// really being measured against how launchable *it* is, which is a different
/// question and, for the lightest class on the roster, a much harder one.
fn in_the_open(class: Class) -> World {
    let mut w = World::with_classes([class, Class::Champion]);
    run(&mut w, 30, 0, 0);
    w.players[0].pos = V3::new(Fx::from_int(-3), Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-2), Fx::ZERO, Fx::ZERO);
    w
}

/// Advance holding `a` until player two's health drops. False if it never does.
fn until_hit(w: &mut World, a: u16) -> bool {
    let before = w.players[1].health;
    for _ in 0..120 {
        run(w, 1, a, 0);
        if w.players[1].health < before {
            return true;
        }
    }
    false
}

fn flat_speed(p: &sim::state::Player) -> f32 {
    V3::new(p.vel.x, Fx::ZERO, p.vel.z)
        .flat_len()
        .to_f32_for_render()
}

/// Is the victim still held by the last thing that hit them?
fn still_held(w: &World) -> bool {
    let p = &w.players[1];
    p.hitlag > 0
        || matches!(
            p.action,
            Action::HitStun { .. } | Action::Held { .. } | Action::Stagger { .. }
        )
}

#[test]
fn a_hit_interrupts_whatever_the_victim_was_doing() {
    // The whole reason stun is a mechanic. A hit that merely subtracted health
    // would leave both fighters swinging past each other, and the winner of an
    // exchange would be whoever pressed first rather than whoever hit first.
    let mut w = engaged();
    // Player two starts the slowest thing they have; player one pokes it out.
    run(&mut w, 1, 0, SHIFT | L);
    assert!(
        matches!(w.players[1].action, Action::Startup { .. }),
        "fixture failed to start a move on the victim"
    );
    assert!(until_hit(&mut w, L), "the poke never connected");
    assert!(
        matches!(w.players[1].action, Action::HitStun { .. }),
        "being hit did not take the victim out of their move: {:?}",
        w.players[1].action
    );
    // And it stays taken away: the interrupted move must not resume.
    for _ in 0..40 {
        run(&mut w, 1, 0, 0);
        assert!(
            !matches!(w.players[1].action, Action::Active { .. }),
            "the interrupted move came back out"
        );
    }
}

#[test]
fn contact_freezes_both_fighters_and_costs_neither_of_them_tempo() {
    // Hitlag. The frames are handed to both sides, so it is punctuation rather
    // than frame advantage: what changes is that a hit reads as a collision,
    // not who gets to move first afterwards.
    let mut w = in_the_open(Class::Bulwark);
    assert!(until_hit(&mut w, L), "the poke never connected");
    let freeze = w.players[0].hitlag;
    assert!(freeze > 0, "a hit produced no contact freeze");
    assert_eq!(
        w.players[1].hitlag, freeze,
        "the two sides of one blow froze for different lengths"
    );

    let frozen = w.players;
    for _ in 0..freeze {
        run(&mut w, 1, 0, 0);
        for (i, (now, then)) in w.players.iter().zip(frozen.iter()).enumerate() {
            assert_eq!(now.pos, then.pos, "player {i} moved while frozen");
            assert_eq!(
                now.action.frames_left(),
                then.action.frames_left(),
                "player {i}'s frames advanced while frozen"
            );
        }
    }
    assert!(
        w.players.iter().all(|p| p.hitlag == 0),
        "the freeze did not end"
    );
    run(&mut w, 1, 0, 0);
    assert!(
        w.players[1].action.frames_left() < frozen[1].action.frames_left(),
        "the stun never started counting down"
    );
}

#[test]
fn the_freeze_does_not_swallow_what_you_pressed_during_it() {
    // Four to eight frames of input being ignored is long enough to lose a
    // button in, and the Champion's leap is pressed inside exactly that window.
    // Whatever was held during a freeze is handed back on the first frame that
    // is not frozen.
    let mut w = in_the_open(Class::Bulwark);
    assert!(until_hit(&mut w, L), "the poke never connected");
    let freeze = w.players[1].hitlag;
    assert!(freeze >= 2, "no freeze to press anything during");
    // Player two taps guard entirely inside the freeze, then lets go.
    for _ in 0..freeze {
        run(&mut w, 1, 0, R);
    }
    assert_ne!(
        w.players[1].buffered & R,
        0,
        "the press was not remembered across the freeze"
    );
}

#[test]
fn damage_already_taken_makes_the_next_hit_send_you_further() {
    // The swell, and the single most Smash-like thing in the system: the same
    // move does the same damage all round and throws you steadily further as
    // the round goes on. It is what gives a match an arc instead of sixty
    // seconds of identical exchanges.
    let speed_at = |health: i32| {
        let mut w = in_the_open(Class::Champion);
        w.players[1].health = health;
        assert!(until_hit(&mut w, L), "the poke never connected");
        flat_speed(&w.players[1])
    };
    let fresh = speed_at(max_health());
    let hurt = speed_at(max_health() / 5);
    assert!(
        hurt > fresh * 2.0,
        "a hurt fighter is thrown {hurt:.1} m/s against {fresh:.1} for an untouched one; \
         the swell is not doing anything"
    );
}

#[test]
fn a_heavy_is_shoved_less_far_than_a_light_one() {
    // Weight, and the reason heavies are combo food: the stun is the same
    // length for everyone, so a fighter who travels less out of it is easier to
    // stay on top of.
    let shove = |victim: sim::class::Class| {
        let mut w = World::with_classes([Class::Champion, victim]);
        run(&mut w, 30, 0, 0);
        w.players[0].pos = V3::new(Fx::from_int(-3), Fx::ZERO, Fx::ZERO);
        w.players[1].pos = V3::new(Fx::from_int(-2), Fx::ZERO, Fx::ZERO);
        assert!(until_hit(&mut w, L), "the poke never connected");
        flat_speed(&w.players[1])
    };
    let heavy = shove(Class::Bulwark);
    let light = shove(Class::DualMage);
    assert!(
        light > heavy,
        "the Dual mage ({light:.1} m/s) is not thrown further than the Bulwark ({heavy:.1}), \
         so weight does not mean anything"
    );
}

#[test]
fn a_launched_fighter_still_steers_and_never_slows_themselves_down() {
    // **The answer to Smash's directional influence, and it is not a second
    // mechanic.** A hit takes away everything that makes you dangerous and
    // leaves the one tool that only repositions: air control. Holding a
    // direction across a launch bends where you land and must never shorten how
    // far you go, or pressing a direction would be a way to take less
    // knockback -- which is a defensive option that costs nothing and is always
    // correct.
    let launched = |held: u16| {
        let mut w = in_the_open(Class::Champion);
        w.players[1].health = max_health() / 2;
        // Put the victim in the air first, so this is a launch and not a skid.
        w.players[1].vel.y = Fx::from_int(6);
        w.players[1].grounded = false;
        assert!(until_hit(&mut w, L), "the poke never connected");
        run(&mut w, 12, 0, held);
        (
            flat_speed(&w.players[1]),
            w.players[1].vel.z.to_f32_for_render(),
        )
    };
    let (straight_speed, straight_z) = launched(0);
    let (steered_speed, steered_z) = launched(Input::A);
    assert!(
        (steered_z - straight_z).abs() > 0.5,
        "steering in the air changed nothing about where they were going: \
         {steered_z:.2} against {straight_z:.2}"
    );
    assert!(
        steered_speed >= straight_speed - 0.05,
        "steering cost them speed -- {steered_speed:.2} m/s against {straight_speed:.2} \
         doing nothing -- so holding a direction is a way to take less knockback"
    );
}

#[test]
fn steering_out_of_a_launch_is_an_air_privilege() {
    // On the ground a stun is the plain tax it has always been. Skidding along
    // the floor is not a moment anybody is flying through, and letting it be
    // steered would quietly hand every grounded hit an escape.
    let skid = |held: u16| {
        let mut w = in_the_open(Class::Champion);
        w.players[1].health = max_health() / 2;
        assert!(until_hit(&mut w, L), "the poke never connected");
        assert!(w.players[1].grounded, "fixture put them in the air");
        run(&mut w, 12, 0, held);
        w.players[1].vel.z.to_f32_for_render()
    };
    assert_eq!(
        skid(0),
        skid(Input::A),
        "a grounded fighter steered out of their own hitstun"
    );
}

#[test]
fn a_hit_moves_somebody_from_where_the_round_starts() {
    // **The marks must not put a fighter's back against a wall.**
    //
    // They used to. The two platforms stand five metres out on either side and
    // the marks sat eight metres apart on the centre line, so each fighter
    // began flush against one: every shove drove the victim into it and the
    // arena zeroed the velocity. Measured from the marks, *every move in the
    // game* moved the opponent exactly zero metres -- including the ones that
    // throw people four metres in open ground.
    //
    // Nothing about the stun system was wrong, and nothing about it could be
    // seen either, which is the worse failure of the two: the first thing
    // anybody does is hit the training dummy on its mark.
    let mut w = World::new();
    run(&mut w, 90, Input::W, 0);
    let before = w.players[1].health;
    let mut landed = false;
    for _ in 0..120 {
        run(&mut w, 1, SHIFT | L, 0);
        if w.players[1].health < before {
            landed = true;
            break;
        }
    }
    assert!(landed, "the fixture never connected");
    let from = w.players[1].pos;
    run(&mut w, 60, 0, 0);
    let shoved = V3::new(
        w.players[1].pos.x.sub(from.x),
        Fx::ZERO,
        w.players[1].pos.z.sub(from.z),
    )
    .flat_len()
    .to_f32_for_render();
    assert!(
        shoved > 1.0,
        "a committed hit moved them {shoved:.2} m from the spawn mark -- less than one body \
         width, so knockback cannot be seen at all where every session starts"
    );
}

#[test]
fn a_grab_that_carries_you_up_throws_you_at_the_end_of_it() {
    // The Champion's uppercut, whose kit entry reads "launch, and hold on".
    // It could not do both: a held fighter is pinned to their captor by
    // `drag_the_held`, which zeroes their velocity every frame, so the launch
    // handed to them on contact was overwritten before it moved anything and
    // they were set back down motionless at the top of the carry. The launch
    // waits for the release instead.
    let mut w = in_the_open(Class::Champion);
    run(&mut w, 2, E, 0); // Rush
    run(&mut w, 1, 0, 0);
    run(&mut w, 2, Input::MIDDLE, 0); // and the uppercut off the dash
    let (mut carried, mut thrown) = (0.0f32, 0.0f32);
    let mut released = false;
    for _ in 0..120 {
        run(&mut w, 1, 0, 0);
        let y = w.players[1].pos.y.to_f32_for_render();
        if matches!(w.players[1].action, Action::Held { .. }) {
            carried = carried.max(y);
        } else if carried > 0.0 {
            released = true;
            thrown = thrown.max(y);
        }
    }
    assert!(carried > 1.0, "the uppercut never took them off the ground");
    assert!(released, "the hold never ended");
    assert!(
        thrown > carried + 0.5,
        "they were carried to {carried:.2} m and let go at {thrown:.2} -- the grab set them \
         down where it picked them up rather than throwing them"
    );
}

#[test]
fn combos_open_up_mid_round_and_close_again() {
    // The shape the whole system exists to produce, on the class built for it.
    //
    // The Dual mage's two autos are its mechanic: left steers dark, right
    // steers light, so alternating them is how the meter is driven while
    // staying on offence. That alternation is the combo worth having, and it is
    // the one measured here.
    //
    // At full health a hit resets neutral -- the stun is short and the exchange
    // starts over. As damage accumulates the stun grows and the autos start to
    // link. Keep going and the knockback, which grows faster, puts the victim
    // out of reach before their attacker can follow, and the link is a launch
    // instead.
    let alternations = |pct: i32| {
        let hp = max_health() * pct / 100;
        [(L, R), (R, L), (L, L), (R, R)]
            .iter()
            .filter(|(a, b)| links(Class::DualMage, *a, *b, hp))
            .count()
    };
    // A band rather than a single reading. Where exactly the window sits is a
    // tuning question that moves every time a kit is touched; that there *is*
    // one, that it is not open at the start and not open at the end, is the
    // design.
    let fresh = alternations(100);
    let middle: usize = [70, 55, 45, 30, 25].iter().map(|p| alternations(*p)).sum();
    let dying = alternations(8);
    assert_eq!(
        fresh, 0,
        "the autos already link at full health, so neutral never resets"
    );
    assert!(
        middle > 0,
        "nothing links anywhere in the middle of a round: the stun system enables no combos"
    );
    assert_eq!(
        dying, 0,
        "the autos still link with the opponent nearly dead ({dying} alternations); \
         knockback has stopped outgrowing hitstun and the round has no arc"
    );
}

/// Does `b` connect while the victim is still held by `a`?
///
/// That *is* the definition of a combo: the follow-up lands during a window in
/// which the victim had no legal input. The attacker holds forward as well,
/// because a player chasing is the situation being asked about.
fn links(class: sim::class::Class, a: u16, b: u16, at_health: i32) -> bool {
    let mut w = in_the_open(class);
    w.players[1].health = at_health;
    let chase = a | Input::W;
    if !until_hit(&mut w, chase) {
        return false;
    }
    let after = w.players[1].health;
    for _ in 0..120 {
        // Read before advancing: the question is whether the victim was still
        // held at the start of the frame the second hit landed.
        let was_held = still_held(&w);
        run(&mut w, 1, b | Input::W, 0);
        if w.players[1].health < after {
            return was_held;
        }
    }
    false
}

#[test]
fn nothing_links_into_itself_while_the_victim_could_live_through_it() {
    // The stunlock rule. Every class has a fast move, and a fast move whose
    // stun outgrows its own recovery loops on itself for the rest of the round
    // -- which is a match decided by the first hit of it.
    //
    // Late in a round the same loop is a **kill confirm**, and that is a
    // different thing: it only runs as long as it does because the victim dies
    // partway through it. Measured from half health, where they would not.
    //
    // The bound is what the *loop* deals -- everything after the first press.
    // One read landing is allowed to be worth a lot; a read that then repeats
    // itself until they are dead is not a read, it is a match decided by one
    // button.
    for class in ALL_CLASSES {
        for kind in 0..moves::table(class).len() as u8 {
            let Some(bits) = press_for(class, kind) else {
                continue;
            };
            let health = max_health() / 2;
            let (repeats, looped) = mashed_chain(class, bits, health);
            // A third of a full bar, measured against the bar rather than
            // against the fixture's starting health: what matters is how much
            // of the match one read decides, and the player reads the bar.
            // Thirty-odd percent is about where a platform fighter's combos
            // land, and a sixty-second time to kill has room for three of them.
            assert!(
                looped * 3 < max_health(),
                "{}: {} mashed from half health repeats {repeats} more times for {looped} \
                 further damage, over a third of a {} bar -- one read decides the round",
                class.name(),
                moves::get(class, kind).name,
                max_health(),
            );
        }
    }
}

/// The button that throws a given move, where a plain press will do it.
///
/// Read out of the game's own binding table rather than written out again here,
/// so a move that moves buttons does not quietly drop out of the test. `None`
/// for the ones that need a state first -- a Rush charge, both feet off the
/// ground -- because those are not reachable by holding one key, and a mapping
/// that pretended otherwise would be measuring nothing while looking thorough.
fn press_for(class: sim::class::Class, kind: u8) -> Option<u16> {
    match moves::binding(class, kind as usize) {
        "LMB" => Some(L),
        "MMB" => Some(Input::MIDDLE),
        "RMB" => Some(R),
        "Shift+LMB" => Some(SHIFT | L),
        "Q" => Some(Q),
        "E" | "E, Shift+LMB" => Some(E),
        _ => None,
    }
}

/// How many times a mashed move **links into itself**, and what the repeats deal.
///
/// A repeat has to clear two bars at once, and each is there for a false
/// positive the other lets through:
///
/// - The damage lands while the victim is **still held** by the previous hit.
///   Sampling that when the button goes down instead of when the blow connects
///   counts a swing thrown during hitstun that arrives after it, which is a
///   race rather than a link -- and it made the Bulwark's Bash read as a nine
///   hit touch of death that it turns out not to have.
/// - It comes from a **later press**. Half the abilities in the game land
///   several contacts from one throw -- a lotus is six blades, a Grasp is four
///   arms -- and counting those would flag every multi-hit move on the roster
///   as a stunlock while missing the thing being asked, which is whether a
///   *second throw* connects on somebody the *first* is still holding.
///
/// The first hit is free: one read landing is allowed to be worth a lot. What
/// is bounded is everything after it.
fn mashed_chain(class: sim::class::Class, bits: u16, health: i32) -> (u32, i32) {
    let mut w = in_the_open(class);
    w.players[1].health = health;
    let (mut repeats, mut looped): (u32, i32) = (0, 0);
    let mut last_health = w.players[1].health;
    let (mut press, mut swinging) = (0u32, false);
    let mut hit_on_press: Option<u32> = None;
    for _ in 0..600 {
        let held = still_held(&w);
        run(&mut w, 1, bits | Input::W, 0);
        let now_swinging = matches!(w.players[0].action, Action::Startup { .. });
        if now_swinging && !swinging {
            press += 1;
        }
        swinging = now_swinging;
        if w.players[1].health < last_health {
            let dealt = last_health - w.players[1].health;
            last_health = w.players[1].health;
            match hit_on_press {
                // Another contact from the throw that is already counted.
                Some(p) if p == press => {}
                Some(_) if held => {
                    repeats += 1;
                    looped += dealt;
                    hit_on_press = Some(press);
                }
                // They got a turn back in between, so the string is over.
                Some(_) => break,
                None => hit_on_press = Some(press),
            }
        }
        if w.players[1].health <= 0 {
            break;
        }
    }
    (repeats, looped)
}
