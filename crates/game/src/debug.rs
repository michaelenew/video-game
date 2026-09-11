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
/// An overhead: it passes over anyone crouching.
const OVERHEAD: Color = Color::srgb(1.0, 0.62, 0.20);
const HURTBOX: Color = Color::srgb(0.45, 0.72, 1.0);
/// Crouching, so overheads miss.
const DUCKED: Color = Color::srgb(0.30, 0.50, 0.78);
const GUARD: Color = Color::srgb(0.21, 0.82, 0.63);
const PARRY: Color = Color::srgb(0.48, 1.0, 0.81);
const FACING: Color = Color::srgb(0.85, 0.88, 0.95);
const SHIELD: Color = Color::srgb(0.98, 0.78, 0.35);

pub fn draw(show: Res<ShowDebug>, sim: Res<crate::Sim>, mut gizmos: Gizmos) {
    if !show.0 {
        return;
    }
    let frame: Frame = view::interpolate(&sim.prev, &sim.cur, sim.clock.alpha());

    for (i, p) in frame.players.iter().enumerate() {
        let pos = Vec3::from_array(p.pos);
        let facing = Vec3::from_array(p.facing);
        let centre = pos + Vec3::Y * 0.9;

        // Hurtbox. Drawn always, and drawn because the hit test adds the
        // defender's body radius to the attack radius -- so "do these two
        // cylinders touch" is precisely "does this connect". Showing only the
        // attack circle would make every hitbox look smaller than it acts.
        let ducked = sim.cur.players[i].crouching;
        cylinder(
            &mut gizmos,
            pos,
            body_radius(),
            body_height(),
            if ducked { DUCKED } else { HURTBOX },
        );
        gizmos.line(centre, centre + facing * 1.2, FACING);

        // The live attack volume, straight from the simulation rather than
        // rebuilt here. Only during active frames: if you can see it, it is out.
        if let Some(hb) = sim::state::hitbox(&sim.cur.players[i]) {
            let colour = hitbox_colour(hb.hits_crouching, hb.spent);
            let at = Vec3::new(
                hb.centre.x.to_f32_for_render(),
                pos.y,
                hb.centre.z.to_f32_for_render(),
            );
            // A cylinder, not a sphere. The test compares *flat* distance and
            // says nothing about height -- you cannot duck under an attack or
            // jump over it, only out-range it, and whether an overhead beats a
            // crouch is a property of the move rather than of its geometry. A
            // sphere would imply a vertical extent the rules do not have.
            cylinder(
                &mut gizmos,
                at,
                hb.radius.to_f32_for_render(),
                body_height(),
                colour,
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

/// What colour an attack volume is drawn.
///
/// Two independent facts, so two independent channels. **Hue** says what kind
/// of attack it is; **brightness** says whether it still has its hit. An
/// earlier version gave "spent" its own colour, which meant the overhead signal
/// vanished the moment a move connected -- precisely when you are stepping
/// through frames to work out what happened.
fn hitbox_colour(hits_crouching: bool, spent: bool) -> Color {
    let base = if hits_crouching { HITBOX } else { OVERHEAD };
    if spent { dim(base) } else { base }
}

/// Same hue, visibly darker. Used for an attack volume that is still out but
/// has already spent its hit.
fn dim(c: Color) -> Color {
    let s = c.to_srgba();
    Color::srgb(s.red * 0.45, s.green * 0.45, s.blue * 0.45)
}

fn body_radius() -> f32 {
    sim::tuning::body_radius().to_f32_for_render()
}

fn body_height() -> f32 {
    sim::tuning::body_height().to_f32_for_render()
}

/// A wireframe cylinder: rings top, middle and bottom, joined by uprights.
///
/// Enough lines to read as a volume at a glance and few enough not to bury the
/// fighters underneath it.
fn cylinder(gizmos: &mut Gizmos, base: Vec3, radius: f32, height: f32, colour: Color) {
    let flat = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    for h in [0.02, height * 0.5, height] {
        gizmos.circle(Isometry3d::new(base + Vec3::Y * h, flat), radius, colour);
    }
    const UPRIGHTS: usize = 8;
    for u in 0..UPRIGHTS {
        let a = u as f32 / UPRIGHTS as f32 * std::f32::consts::TAU;
        let off = Vec3::new(a.cos(), 0.0, a.sin()) * radius;
        gizmos.line(
            base + off + Vec3::Y * 0.02,
            base + off + Vec3::Y * height,
            colour,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(c: Color) -> (f32, f32, f32) {
        let s = c.to_srgba();
        (s.red, s.green, s.blue)
    }

    #[test]
    fn an_overhead_stays_an_overhead_after_it_connects() {
        // The bug this replaced: once a move had spent its hit, the overlay
        // switched to a "spent" colour and stopped saying it was an overhead.
        let live = rgb(hitbox_colour(false, false));
        let used = rgb(hitbox_colour(false, true));
        let normal = rgb(hitbox_colour(true, false));

        // Same hue: the ratio between channels survives the dimming.
        assert!(
            (live.0 / live.1 - used.0 / used.1).abs() < 0.05,
            "spent changed the hue: {live:?} became {used:?}"
        );
        assert!(used.0 < live.0, "spent is not darker");
        // And an overhead is still distinguishable from an ordinary attack.
        assert!(
            (live.1 - normal.1).abs() > 0.2,
            "an overhead looks like an ordinary attack"
        );
    }

    #[test]
    fn a_spent_overhead_is_not_mistaken_for_a_live_normal() {
        // The two dimmest cases must not collide, or the overlay is ambiguous
        // in exactly the state it is most often read in.
        let spent_overhead = rgb(hitbox_colour(false, true));
        let live_normal = rgb(hitbox_colour(true, false));
        let distance = (spent_overhead.0 - live_normal.0).abs()
            + (spent_overhead.1 - live_normal.1).abs()
            + (spent_overhead.2 - live_normal.2).abs();
        assert!(distance > 0.3, "colours too close: {distance}");
    }
}
