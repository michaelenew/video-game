//! Playing the game's voice.
//!
//! What a thing sounds like is decided in `crates/sound`, from the
//! simulation's own numbers; this file only plays it. Each simulation tick is
//! diffed into its cues ([`sound::cues`]) as it is advanced -- inside the
//! tick loop, because one picture can carry several ticks and a rollback
//! re-advances frames -- and [`play`] turns the cues into sources at their
//! places in the arena, heard by a listener on the camera.
//!
//! **Under rollback a frame is sounded once**: the first time it is
//! simulated, and never again when it is re-simulated with the other
//! player's real inputs. A mispredicted hit therefore makes its sound; that
//! is the price every rollback game pays, and the alternative -- waiting for
//! confirmation -- would put the whole rollback window between a parry and
//! its chime. See `docs/design/sound.md`.
//!
//! Every source is rendered on the spot from its patch: a blow is a few
//! hundred milliseconds of samples, well under a millisecond to make, and a
//! fight makes a few a second. A bank, keyed by a patch's rounded numbers,
//! is the obvious next step if that ever shows in the profile.

use bevy::prelude::*;
use sound::{Cue, Cues};

/// The cues gathered by this picture's ticks, and how far they have been
/// sounded.
#[derive(Resource, Default)]
pub struct Soundscape {
    /// Cues waiting to be played, each with the frame it belongs to.
    pending: Vec<(u32, Cue)>,
    /// The newest frame whose cues have been sounded. A frame simulated
    /// again (a rollback, a rewind and a step) is not sounded again.
    sounded_to: u32,
    /// Scratch for one tick's cues, so a tick allocates nothing to say what
    /// it sounded like.
    scratch: Cues,
}

impl Soundscape {
    /// One simulation tick happened: `before` became `after`. Called from the
    /// tick loop, for every tick, predicted or not.
    pub fn observe(&mut self, before: &sim::World, after: &sim::World) {
        if after.frame <= self.sounded_to {
            return;
        }
        sound::cues(before, after, &mut self.scratch);
        for cue in self.scratch.iter() {
            self.pending.push((after.frame, *cue));
        }
    }

    /// The world was put back to an earlier frame on purpose (`[`): what
    /// comes after it is new again.
    pub fn rewound_to(&mut self, frame: u32) {
        self.sounded_to = self.sounded_to.min(frame);
        self.pending.retain(|(f, _)| *f <= frame);
    }

    /// A fresh fight from the top: nothing has sounded yet.
    pub fn restart(&mut self, frame: u32) {
        self.sounded_to = frame;
        self.pending.clear();
    }
}

/// The ears: on the camera, a head's width apart.
#[derive(Component)]
pub struct Ears;

/// Put the listener on the camera, once it exists.
pub fn setup(mut commands: Commands, cameras: Query<Entity, Added<crate::MainCamera>>) {
    for camera in &cameras {
        commands
            .entity(camera)
            .insert((SpatialListener::new(0.3), Ears));
    }
}

/// Positions are in metres and the arena is thirty of them across, with the
/// camera a dozen back from the fight. Rodio's spatial sink attenuates by
/// distance from one unit, so metres are scaled down to keep the fight
/// audible from where the camera is, and the fall-off below is the one the
/// ear gets instead.
const SPATIAL_SCALE: f32 = 0.08;

/// How loud a cue is at a distance from the ears: full to eight metres, then
/// down to a quarter at forty.
fn falloff(distance: f32) -> f32 {
    if distance <= 8.0 {
        1.0
    } else {
        (1.0 - (distance - 8.0) / 32.0 * 0.75).max(0.25)
    }
}

/// Sound the pending cues: one source each, at its place, despawned when it
/// has played.
pub fn play(
    mut commands: Commands,
    mut scape: ResMut<Soundscape>,
    mut sources: ResMut<Assets<AudioSource>>,
    settings: Res<crate::settings::Settings>,
    ears: Query<&GlobalTransform, With<Ears>>,
) {
    if scape.pending.is_empty() {
        return;
    }
    let at_ears = ears.single().map(|t| t.translation()).unwrap_or(Vec3::ZERO);
    let master = settings.volume;
    let pending = std::mem::take(&mut scape.pending);
    // Many sources in one tick (a pack, a landing on a crowd) are capped, so
    // a busy frame is loud rather than a wall.
    let mut budget = 12;
    for (frame, cue) in pending {
        scape.sounded_to = scape.sounded_to.max(frame);
        if budget == 0 || master <= 0.0 {
            continue;
        }
        budget -= 1;
        let samples = cue.patch.render();
        let bytes = sound::wav::encode(&samples);
        let handle = sources.add(AudioSource {
            bytes: bytes.into(),
        });
        let at = Vec3::new(cue.at[0], cue.at[1], cue.at[2]);
        let gain = falloff(at.distance(at_ears)) * master;
        commands.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: bevy::audio::Volume::Linear(gain),
                spatial: true,
                spatial_scale: Some(bevy::audio::SpatialScale::new(SPATIAL_SCALE)),
                ..PlaybackSettings::DESPAWN
            },
            Transform::from_translation(at),
            GlobalTransform::from_translation(at),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_is_sounded_once() {
        let mut scape = Soundscape::default();
        let a = sim::World::with_classes([sim::Class::Champion; 2]);
        let mut b = a.clone();
        b.advance([
            sim::Input::aimed(sim::Input::SPACE, 0),
            sim::Input::default(),
        ]);
        scape.observe(&a, &b);
        let first = scape.pending.len();
        assert!(first > 0, "a jump makes a sound");
        // Played.
        for (f, _) in scape.pending.drain(..) {
            scape.sounded_to = scape.sounded_to.max(f);
        }
        // Re-simulated after a rollback: silent.
        scape.observe(&a, &b);
        assert!(scape.pending.is_empty());
        // Rewound on purpose and stepped again: heard again.
        scape.rewound_to(a.frame);
        scape.observe(&a, &b);
        assert_eq!(scape.pending.len(), first);
    }

    #[test]
    fn the_falloff_is_full_close_and_never_silent() {
        assert_eq!(falloff(2.0), 1.0);
        assert!(falloff(20.0) < 1.0 && falloff(20.0) > 0.25);
        assert_eq!(falloff(100.0), 0.25);
    }
}
