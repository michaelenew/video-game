//! Combat rules. These encode design decisions, so a failure here means either
//! a bug or a decision that changed without the documents changing.

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
