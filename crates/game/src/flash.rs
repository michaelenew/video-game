//! Bloom, and the light a hit throws.
//!
//! Ability spectacle is an explicit design goal -- [the combat
//! kernel](../../../docs/design/combat-kernel.md) says abilities should be
//! visually rewarding and the camera exists to show them off. This is the
//! cheapest half of delivering that.
//!
//! **Bloom is the whole trick.** A surface that emits more light than white
//! has nowhere to go in an eight-bit image and simply clips; with a
//! high-dynamic-range camera in front of it, the excess spills into the pixels
//! around it instead. That spill is what the eye reads as *brightness* rather
//! than as *pale colour*, and it is the difference between a fire that is
//! orange and a fire that is burning. The fire and arcane materials already
//! emit several times white, so this costs one component and no art.
//!
//! **A hit throws light.** When something connects, a point light appears
//! where it landed and dies over about a fifth of a second. It is close to
//! free, it does more for impact than any amount of texture, and it lights the
//! surroundings -- so a hit next to a wall washes the wall, which is the part
//! that reads as force.
//!
//! ## Which clock this runs on
//!
//! The **wall clock**, deliberately, and it is worth saying why since the rule
//! usually runs the other way.
//!
//! A flash carries no information. It says a hit happened, which the player
//! already knows from the health bar, the hitstun and the knockback. So it is
//! texture, and texture is allowed to pop across a rollback -- which here means
//! a flash may very occasionally fire twice for one hit, over a window of
//! sixteen to a hundred and thirty milliseconds. Nobody will see it.
//!
//! Putting it in the simulation instead would cost a slot in the snapshot, on
//! the rollback path, on every re-simulated frame, to make a light flicker
//! slightly more correctly. That is the wrong trade, and stating it is how the
//! next cosmetic effect gets built in the right place.

use bevy::prelude::*;
use sim::state::Action;

use crate::{MAX_PLAYERS, Sim};

/// How many hits can be lighting the arena at once.
///
/// A fixed pool, spawned once, for the same reason the effect meshes are: a
/// hit is exactly when the frame is busiest, and allocating an entity at that
/// moment is the worst possible time to do it.
const FLASHES: usize = 8;

/// How long a flash lasts, in seconds. Twelve frames at sixty.
const LIFETIME: f32 = 0.2;

/// A hit is hot and slightly warm -- a spark rather than a lamp.
const HIT_COLOUR: Color = Color::linear_rgb(1.0, 0.86, 0.62);

/// Peak intensity, in lumens. Large, because it is competing with daylight --
/// a hit that cannot be seen against the sun is not a hit that reads.
const PEAK: f32 = 900_000.0;

#[derive(Component)]
pub struct HitFlash {
    /// When it was lit, on the renderer's clock. `None` when the slot is free.
    lit_at: Option<f32>,
    colour: Color,
}

/// What each fighter's action was last frame, so a *new* hitstun can be told
/// from an ongoing one.
///
/// Renderer-side rather than in the snapshot, which is the whole point: see
/// the module header. A hit is the frame a fighter enters hitstun, not every
/// frame they spend in it.
#[derive(Resource, Default)]
pub struct WasHit([bool; MAX_PLAYERS]);

pub fn setup(mut commands: Commands) {
    for _ in 0..FLASHES {
        commands.spawn((
            PointLight {
                intensity: 0.0,
                range: 14.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::default(),
            Visibility::Hidden,
            HitFlash {
                lit_at: None,
                colour: Color::WHITE,
            },
        ));
    }
    commands.insert_resource(WasHit::default());
}

/// Light a flash wherever a fighter has just been hit, and fade the rest.
pub fn run(
    sim: Res<Sim>,
    time: Res<Time>,
    mut was_hit: ResMut<WasHit>,
    mut flashes: Query<(
        &mut HitFlash,
        &mut PointLight,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    let now = time.elapsed_secs();

    // Collect this frame's new hits before touching the pool, so that claiming
    // a slot cannot be confused by a slot freed earlier in the same pass.
    let mut landed: Vec<(Vec3, Color)> = Vec::new();
    for owner in 0..MAX_PLAYERS {
        let player = &sim.cur.players[owner];
        let stunned = matches!(
            player.action,
            Action::HitStun { .. } | Action::Stagger { .. }
        );
        if stunned && !was_hit.0[owner] {
            let at = Vec3::new(
                player.pos.x.to_f32_for_render(),
                player.pos.y.to_f32_for_render() + 1.0,
                player.pos.z.to_f32_for_render(),
            );
            // Warm white, and not the attacker's identity colour, which is
            // what this reached for first.
            //
            // The simulation does not record *who* hit you -- only that you
            // are in hitstun. With two fighters "the other one" is a safe
            // guess and it is wrong the moment coop puts four players and
            // several monsters in the arena, at which point the light would be
            // confidently attributing hits to the wrong fighter. Carrying the
            // attacker would cost a slot in the checksummed snapshot to colour
            // a light.
            //
            // So the flash says *a hit landed*, which is true, instead of
            // *who landed it*, which is not known here.
            landed.push((at, HIT_COLOUR));
        }
        was_hit.0[owner] = stunned;
    }

    for (mut flash, mut light, mut tf, mut vis) in flashes.iter_mut() {
        if let Some(lit) = flash.lit_at {
            let age = (now - lit) / LIFETIME;
            if age >= 1.0 {
                flash.lit_at = None;
                light.intensity = 0.0;
                *vis = Visibility::Hidden;
            } else {
                // Cubic falloff, not linear. A light that fades evenly reads as
                // a lamp being turned down; one that drops away fast reads as
                // an impact, which is the same reason fire's brightness is
                // cubed in its temperature ramp.
                let left = 1.0 - age;
                light.intensity = PEAK * left * left * left;
                light.color = flash.colour;
            }
            continue;
        }
        if let Some((at, colour)) = landed.pop() {
            flash.lit_at = Some(now);
            flash.colour = colour;
            light.color = colour;
            light.intensity = PEAK;
            tf.translation = at;
            *vis = Visibility::Inherited;
        }
    }
}
