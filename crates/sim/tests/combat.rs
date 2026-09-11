//! Combat rules. These encode design decisions, so a failure here means either
//! a bug or a decision that changed without the documents changing.

use sim::class::ALL_CLASSES;
use sim::fixed::Fx;
use sim::state::{Action, MAX_HEALTH, Phase, Shield};
use sim::{Input, World};

const L: u16 = Input::LEFT;
const R: u16 = Input::RIGHT;
const M: u16 = Input::MIDDLE;
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
    assert_eq!(w.players[1].health, MAX_HEALTH, "blocking took chip damage");
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
    run(&mut w, 40, M | SHIFT, R);
    assert!(
        w.players[1].health < before,
        "grapple was blocked; guard is unbeatable"
    );
}

#[test]
fn a_dodge_has_invulnerable_frames_then_stops_having_them() {
    let mut w = World::new();
    w.advance([
        Input::aimed(Input::SPACE | Input::W, LOOK_RIGHT),
        Input::aimed(0, LOOK_LEFT),
    ]);
    assert!(
        w.players[0].action.invulnerable(),
        "dodge did not start invulnerable"
    );
    run(&mut w, 20, Input::SPACE | Input::W, 0);
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
        baseline.players[1].health < MAX_HEALTH,
        "setup did not connect"
    );

    let mut dodged = engaged();
    run(&mut dodged, 20, L, Input::SPACE | Input::S);
    assert_eq!(
        dodged.players[1].health, MAX_HEALTH,
        "dodge failed to evade"
    );
}

#[test]
fn throwing_the_shield_gives_up_blocking() {
    let mut w = World::new();
    run(&mut w, 2, M, 0);
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
    run(&mut w, 2, M, 0);
    run(&mut w, 60, 0, 0);
    assert!(
        matches!(w.players[0].shield(), Some(Shield::Planted { .. })),
        "shield never came to rest: {:?}",
        w.players[0].shield()
    );
    run(&mut w, 2, M, 0);
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
    assert_eq!(w.players[1].health, MAX_HEALTH, "health did not reset");
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
        standing.players[1].health < MAX_HEALTH,
        "setup did not connect while standing"
    );

    let mut mid = engaged();
    run(&mut mid, 20, L, Input::CROUCH);
    assert!(
        mid.players[1].health < MAX_HEALTH,
        "a mid was ducked; crouch beats everything"
    );

    let mut ducked = engaged();
    run(&mut ducked, 45, L | SHIFT, Input::CROUCH);
    assert_eq!(
        ducked.players[1].health, MAX_HEALTH,
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
        facing.players[1].health < MAX_HEALTH,
        "a poke at point blank did not connect"
    );

    let mut away = engaged();
    for _ in 0..20 {
        // Looking backwards, the opponent is behind you.
        away.advance([Input::aimed(L, LOOK_LEFT), Input::aimed(0, LOOK_LEFT)]);
    }
    assert_eq!(
        away.players[1].health, MAX_HEALTH,
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

/// A world with player one mid-swing and player two parked at `gap` from the
/// centre of the attack volume, along the attack direction.
fn swinging_at(class: sim::class::Class, gap_factor: f32) -> World {
    use sim::state::hitbox;

    let mut w = World::with_classes([class, class]);
    // Out of the way while the swing starts, so nothing connects early.
    w.players[1].pos = sim::V3::new(Fx::from_int(30), w.players[1].pos.y, Fx::ZERO);

    for _ in 0..40 {
        w.advance([Input::aimed(L, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        if hitbox(&w.players[0]).is_some() {
            break;
        }
    }
    let hb = hitbox(&w.players[0]).expect("never reached an active frame");

    // Exactly the threshold the hit test uses: the attack radius plus the
    // defender's body radius, which is why the overlay draws both cylinders.
    let threshold =
        (hb.radius.to_f32_for_render() + sim::tuning::BODY_RADIUS.to_f32_for_render()) * gap_factor;
    w.players[1].pos = sim::V3::new(
        hb.centre
            .x
            .add(Fx::ratio((threshold * 1000.0) as i32, 1000)),
        w.players[1].pos.y,
        hb.centre.z,
    );
    w.advance([Input::aimed(L, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
    w
}

#[test]
fn the_drawn_hitbox_is_the_one_that_hits() {
    // The overlay draws `state::hitbox`, and the hit test uses it too. This
    // pins that they agree at the boundary, for every class -- including the
    // Bellator, whose weapon form multiplies reach and which an overlay
    // rebuilding the box from the move table on its own would get wrong.
    for class in ALL_CLASSES {
        let inside = swinging_at(class, 0.8);
        assert!(
            inside.players[1].health < MAX_HEALTH,
            "{class:?}: a defender well inside the drawn box was not hit"
        );
        let outside = swinging_at(class, 1.3);
        assert_eq!(
            outside.players[1].health, MAX_HEALTH,
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
