//! Debug overlay.
//!
//! Tuning happens where you play, so the things the browser frame-data tool
//! shows have to exist here too: hitboxes, guard arcs, the shield, and where
//! each fighter is facing.
//!
//! Toggle with F1.

use bevy::prelude::*;
use sim::class::Mechanic;
use sim::state::Action;
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
            // Straight from the move table, so the overlay cannot drift from
            // what actually hits.
            let m = sim::moves::get(sim.cur.players[i].class, kind);
            gizmos.sphere(
                Isometry3d::from_translation(centre + facing * m.reach.to_f32_for_render()),
                m.radius.to_f32_for_render(),
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

        // The class mechanic, wherever it lives in the world.
        for spot in mechanic_markers(&sim.cur.players[i].mechanic) {
            gizmos.sphere(Isometry3d::from_translation(spot), 0.45, SHIELD);
        }
    }
}

/// Anything the class mechanic has placed in the world: a thrown shield, a
/// shadow, structures. Drawn identically because the point is "your mechanic
/// is over there", not what shape it is.
fn mechanic_markers(m: &Mechanic) -> Vec<Vec3> {
    let v3 = |v: sim::V3| {
        Vec3::new(
            v.x.to_f32_for_render(),
            v.y.to_f32_for_render(),
            v.z.to_f32_for_render(),
        )
    };
    match m {
        Mechanic::Shield(s) => s.world_pos().map(v3).into_iter().collect(),
        Mechanic::Shadow { at } => at.map(v3).into_iter().collect(),
        Mechanic::Structures(slots) => slots.iter().filter_map(|s| s.map(v3)).collect(),
        _ => Vec::new(),
    }
}
