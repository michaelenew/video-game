//! Debug overlay.
//!
//! Tuning happens where you play, so the things the browser frame-data tool
//! shows have to exist here too: hitboxes, guard arcs, the shield, and where
//! each fighter is facing.
//!
//! Toggle with F1.

use bevy::prelude::*;
use sim::state::{Action, Shield, move_frames};
use view::Frame;

#[derive(Resource)]
pub struct ShowDebug(pub bool);

impl Default for ShowDebug {
    fn default() -> Self {
        // On by default under DEMO so headless screenshots show the overlay.
        ShowDebug(std::env::var("DEBUG_OVERLAY").is_ok_and(|v| v == "1"))
    }
}

const HITBOX: Color = Color::srgb(1.0, 0.23, 0.31);
const GUARD: Color = Color::srgb(0.21, 0.82, 0.63);
const PARRY: Color = Color::srgb(0.48, 1.0, 0.81);
const FACING: Color = Color::srgb(0.85, 0.88, 0.95);
const SHIELD: Color = Color::srgb(0.98, 0.78, 0.35);
const BODY: Color = Color::srgb(0.35, 0.42, 0.52);

pub fn draw(show: Res<ShowDebug>, sim: Res<crate::Sim>, mut gizmos: Gizmos) {
    if !show.0 {
        return;
    }
    let frame: Frame = view::interpolate(&sim.prev, &sim.cur, sim.clock.alpha());

    for (i, p) in frame.players.iter().enumerate() {
        let pos = Vec3::from_array(p.pos);
        let facing = Vec3::from_array(p.facing);
        let centre = pos + Vec3::Y * 0.9;

        // Body cylinder, drawn as two rings so overlap is visible.
        for h in [0.05, 1.75] {
            gizmos.circle(
                Isometry3d::new(
                    pos + Vec3::Y * h,
                    Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                ),
                0.5,
                BODY,
            );
        }
        gizmos.line(centre, centre + facing * 1.2, FACING);

        // Live hitbox. Only drawn during active frames, which is the point --
        // if you can see it, it can hit you.
        if let Action::Active { kind, .. } = p.action {
            let (reach, radius) = hitbox_shape(kind);
            gizmos.sphere(
                Isometry3d::from_translation(centre + facing * reach),
                radius,
                HITBOX,
            );
        }

        // Guard arc: 120 degrees, and brighter during the parry window.
        if let Action::Guard { held } = p.action {
            let parrying = held < sim::state::parry_window();
            let colour = if parrying { PARRY } else { GUARD };
            let base = facing.z.atan2(facing.x);
            let steps = 14;
            let mut prev = None;
            for s in 0..=steps {
                let a = base - std::f32::consts::FRAC_PI_3
                    + (s as f32 / steps as f32) * (2.0 * std::f32::consts::FRAC_PI_3);
                let point = centre + Vec3::new(a.cos(), 0.0, a.sin()) * 1.5;
                if let Some(q) = prev {
                    gizmos.line(q, point, colour);
                }
                prev = Some(point);
            }
            gizmos.line(centre, centre + facing * 1.5, colour);
        }

        // The shield, wherever it is.
        if let Some(sp) = shield_pos(&sim.cur.players[i].shield) {
            gizmos.sphere(Isometry3d::from_translation(sp), 0.45, SHIELD);
        }
    }
}

fn shield_pos(s: &Shield) -> Option<Vec3> {
    s.world_pos().map(|v| {
        Vec3::new(
            v.x.to_f32_for_render(),
            v.y.to_f32_for_render(),
            v.z.to_f32_for_render(),
        )
    })
}

/// Mirrors the reach and radius in `sim::state`. Derived from the frame table
/// so the two cannot silently drift apart on timing.
fn hitbox_shape(kind: u8) -> (f32, f32) {
    let _ = move_frames(kind);
    match kind {
        sim::state::MOVE_BASH => (1.5, 0.9),
        sim::state::MOVE_SLAM => (2.0, 1.4),
        _ => (1.1, 0.8),
    }
}
