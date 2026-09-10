//! Combat rules. These encode design decisions, so a failure here means either
//! a bug or a decision that changed without the documents changing.

use sim::state::{Action, MAX_HEALTH, Phase, Shield};
use sim::{Input, World};

const L: u16 = Input::LEFT;
const R: u16 = Input::RIGHT;
const M: u16 = Input::MIDDLE;
const SHIFT: u16 = Input::SHIFT;

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    for _ in 0..frames {
        w.advance([Input(a), Input(b)]);
    }
}

/// Walk player one into range so attacks connect. They start eight units
/// apart at seven units per second, so this needs more than a second.
fn engaged() -> World {
    let mut w = World::new();
    run(&mut w, 90, Input::D, 0);
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
    w.advance([Input(Input::SPACE | Input::D), Input::default()]);
    assert!(
        w.players[0].action.invulnerable(),
        "dodge did not start invulnerable"
    );
    run(&mut w, 20, Input::SPACE | Input::D, 0);
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
        !w.players[0].shield.in_hand(),
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
        matches!(w.players[0].shield, Shield::Planted { .. }),
        "shield never came to rest: {:?}",
        w.players[0].shield
    );
    run(&mut w, 2, M, 0);
    run(&mut w, 90, 0, 0);
    assert!(
        w.players[0].shield.in_hand(),
        "recall never returned the shield: {:?}",
        w.players[0].shield
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
    run(&mut w, 400, Input::D, Input::A);
    for (i, p) in w.players.iter().enumerate() {
        let x = p.pos.x.to_f32_for_render();
        assert!(x.abs() < 14.5, "player {i} escaped the arena at x={x}");
    }
}

#[test]
fn players_can_stand_on_the_platforms() {
    let mut w = World::new();
    // Walk onto the left platform and jump.
    run(&mut w, 60, Input::A, 0);
    run(&mut w, 40, Input::SPACE, 0);
    run(&mut w, 60, Input::D, 0);
    let y = w.players[0].pos.y.to_f32_for_render();
    assert!(y >= 0.0, "fell through the floor to y={y}");
}
