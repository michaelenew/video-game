//! **What the Veilstalker touches**, drawn (`veilstalker.md` §6): the trail in
//! the snow and the ash, the paint on its hide, its breath, the ripples
//! round its legs in the stream, and its quills in flight.
//!
//! Everything here is read from the snapshot -- the footfall ring, the paint
//! ring, the quills' launch, the parts' places -- or derived from it and the
//! frame count, so a rollback redraws the same trail and nothing is drawn
//! that the simulation does not know. The body itself is `crate::beast`'s,
//! drawn at the strength `World::shown` gives each part; so is the mimic's
//! ghost (`World::apparition`). A fixed pool of meshes, spawned once: nothing
//! is spawned in a fight.
//!
//! **Prints are the creature's alone** (§6): fighters leave none, so every
//! print on the floor means the same thing. Three-toed and dark, nothing else
//! in the arena is either. On ash a fresh print glows with stirred embers for
//! `EmberGlow` frames; through blood, a print is red.

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use sim::arena::Material;
use sim::monster::MAX_MONSTERS;
use sim::species::SpeciesId;
use sim::species::veilstalker::{self as vs, Knob, fight};

/// One piece of what is drawn, by pool and index.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trace {
    /// A print's toe: `(print, toe)`.
    Toe(usize, usize),
    Paint(usize),
    Breath(usize),
    Ripple(usize),
    Quill(usize),
}

#[derive(Resource)]
pub struct Inks {
    print: Handle<StandardMaterial>,
    red: Handle<StandardMaterial>,
    ember: Handle<StandardMaterial>,
    paint: Handle<StandardMaterial>,
    breath: Handle<StandardMaterial>,
    ripple: Handle<StandardMaterial>,
    quill: Handle<StandardMaterial>,
}

/// Three toes a print.
const TOES: usize = 3;
/// Breath puffs alive at once.
const PUFFS: usize = 3;
/// A puff every this many frames, from the head.
const BREATH_EVERY: u32 = 90;
/// How long a puff lasts.
const PUFF_LIFE: u32 = 50;
/// Legs that can be in the stream.
const RIPPLES: usize = 4;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut flat = |rgb: [f32; 3], glow: [f32; 3], alpha: f32| {
        materials.add(StandardMaterial {
            base_color: Color::srgba(rgb[0], rgb[1], rgb[2], alpha),
            emissive: LinearRgba::rgb(glow[0], glow[1], glow[2]),
            alpha_mode: if alpha < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            perceptual_roughness: 1.0,
            ..default()
        })
    };
    let inks = Inks {
        print: flat([0.16, 0.15, 0.17], [0.0; 3], 0.85),
        red: flat([0.55, 0.05, 0.06], [0.15, 0.0, 0.0], 0.9),
        ember: flat([0.95, 0.45, 0.10], [2.2, 0.7, 0.1], 0.95),
        paint: flat([1.0, 0.25, 0.65], [2.6, 0.5, 1.4], 1.0),
        breath: flat([0.94, 0.95, 0.98], [0.1, 0.1, 0.12], 0.45),
        ripple: flat([0.70, 0.82, 0.92], [0.05, 0.08, 0.10], 0.6),
        quill: flat([0.22, 0.20, 0.18], [0.0; 3], 1.0),
    };
    let disc = meshes.add(Cylinder::new(0.5, 1.0));
    let ball = meshes.add(Sphere::new(0.5).mesh().ico(1).unwrap());
    let ring = meshes.add(Torus::new(0.40, 0.48));
    let stick = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mut spawn = |mesh: &Handle<Mesh>, material: &Handle<StandardMaterial>, t: Trace| {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Visibility::Hidden,
            NotShadowCaster,
            t,
        ));
    };
    for p in 0..fight::PRINTS {
        for toe in 0..TOES {
            spawn(&disc, &inks.print, Trace::Toe(p, toe));
        }
    }
    for i in 0..fight::PAINTS {
        spawn(&ball, &inks.paint, Trace::Paint(i));
    }
    for i in 0..PUFFS {
        spawn(&ball, &inks.breath, Trace::Breath(i));
    }
    for i in 0..RIPPLES {
        spawn(&ring, &inks.ripple, Trace::Ripple(i));
    }
    for i in 0..fight::QUILL_COUNT {
        spawn(&stick, &inks.quill, Trace::Quill(i));
    }
    commands.insert_resource(inks);
}

fn fx3(v: sim::V3) -> Vec3 {
    Vec3::new(
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    )
}

/// Put every trace where the snapshot says it is.
pub fn place(
    sim: Res<crate::Sim>,
    inks: Res<Inks>,
    mut traces: Query<(
        &Trace,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let w = &sim.cur;
    let beast = (0..MAX_MONSTERS).find_map(|s| {
        w.monsters[s]
            .filter(|m| m.species == SpeciesId::VEILSTALKER)
            .map(|m| (s, m))
    });
    let Some((_slot, m)) = beast else {
        for (_, _, mut visible, _) in traces.iter_mut() {
            *visible = Visibility::Hidden;
        }
        return;
    };
    // The rings read once, on the stack.
    let mut prints = [None; fight::PRINTS];
    for (i, p) in fight::prints(w).enumerate().take(fight::PRINTS) {
        prints[i] = Some(p);
    }
    let mut paints = [None; fight::PAINTS];
    for (i, p) in fight::paints(w).enumerate().take(fight::PAINTS) {
        paints[i] = Some(p);
    }
    let mut quills = [None; fight::QUILL_COUNT];
    for q in fight::quills_in_flight(w) {
        quills[q.index] = Some(q);
    }
    let glow = Knob::EmberGlow.raw().max(0) as u32;
    let head = m.head();
    let rig = m.rig();
    for (trace, mut transform, mut visible, mut material) in traces.iter_mut() {
        let mut show = |t: Transform, ink: &Handle<StandardMaterial>| {
            *transform = t;
            *visible = Visibility::Inherited;
            if material.0 != *ink {
                material.0 = ink.clone();
            }
        };
        match *trace {
            Trace::Toe(i, toe) => {
                let Some(p) = prints[i] else {
                    *visible = Visibility::Hidden;
                    continue;
                };
                let life = fight::print_life(w, &p).max(1);
                // A print fades over its last third.
                let left = 1.0 - p.age as f32 / life as f32;
                let fade = (left * 3.0).clamp(0.0, 1.0);
                if fade <= 0.0 {
                    *visible = Visibility::Hidden;
                    continue;
                }
                let ash = w.arena().floor_at(p.at.x, p.at.z) == Material::Ash;
                let ink = if p.red {
                    &inks.red
                } else if ash && p.age < glow {
                    &inks.ember
                } else {
                    &inks.print
                };
                // Three toes splayed forward of the pad, toward the next print
                // laid after it -- the way it was going. The ring keeps no
                // heading: the trail is its own heading.
                let next = prints
                    .iter()
                    .flatten()
                    .filter(|o| o.age < p.age)
                    .max_by_key(|o| o.age);
                let ahead = next
                    .map(|o| fx3(o.at) - fx3(p.at))
                    .unwrap_or_else(|| fx3(sim::V3::from_turns(m.yaw)))
                    .with_y(0.0)
                    .normalize_or(Vec3::X);
                let side = Vec3::new(-ahead.z, 0.0, ahead.x);
                let spread = (toe as f32 - 1.0) * 0.11;
                let at = fx3(p.at)
                    + ahead * (0.08 + 0.04 * (1.0 - (toe as f32 - 1.0).abs()))
                    + side * spread;
                let size = 0.11 * (0.5 + 0.5 * fade);
                show(
                    Transform {
                        translation: Vec3::new(at.x, 0.012, at.z),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(size, 0.01, size * 1.6),
                    },
                    ink,
                );
            }
            Trace::Paint(i) => {
                let Some(p) = paints[i] else {
                    *visible = Visibility::Hidden;
                    continue;
                };
                let k = fight::paint_strength(&p).to_f32_for_render();
                if k <= 0.0 || !m.alive() {
                    *visible = Visibility::Hidden;
                    continue;
                }
                let at = fx3(rig.part_to_world(p.part, p.local));
                show(
                    Transform::from_translation(at).with_scale(Vec3::splat(0.22 + 0.18 * k)),
                    &inks.paint,
                );
            }
            Trace::Breath(i) => {
                // A puff every `BREATH_EVERY` frames from its head, rising and
                // spreading as it fades: derived from the frame and the head,
                // nothing stored.
                let f = w.frame;
                let beat = f / BREATH_EVERY;
                if beat < i as u32 || !m.alive() {
                    *visible = Visibility::Hidden;
                    continue;
                }
                let born = (beat - i as u32) * BREATH_EVERY;
                let age = f - born;
                if age >= PUFF_LIFE {
                    *visible = Visibility::Hidden;
                    continue;
                }
                let u = age as f32 / PUFF_LIFE as f32;
                let ahead = fx3(sim::V3::from_turns(m.yaw))
                    .with_y(0.0)
                    .normalize_or(Vec3::X);
                let at = fx3(head) + ahead * (0.6 + 0.5 * u) + Vec3::Y * (0.2 * u);
                show(
                    Transform::from_translation(at).with_scale(Vec3::splat(0.25 + 0.7 * u)),
                    &inks.breath,
                );
            }
            Trace::Ripple(i) => {
                let leg = vs::SPECIES.legs[i.min(vs::SPECIES.legs.len() - 1)];
                let at = rig.part_to_world(leg.foot, sim::V3::ZERO);
                let wet = w.arena().floor_at(at.x, at.z) == Material::Water
                    && at.y.raw() < sim::Fx::ratio(1, 2).raw();
                if !wet || !m.alive() {
                    *visible = Visibility::Hidden;
                    continue;
                }
                let pulse = 1.0 + 0.4 * ((w.frame % 40) as f32 / 40.0);
                let a = fx3(at);
                show(
                    Transform::from_translation(Vec3::new(a.x, 0.03, a.z))
                        .with_scale(Vec3::new(pulse, 1.0, pulse)),
                    &inks.ripple,
                );
            }
            Trace::Quill(i) => {
                let Some(q) = quills[i] else {
                    *visible = Visibility::Hidden;
                    continue;
                };
                let (at, was) = (fx3(q.at), fx3(q.was));
                let dir = (at - was).normalize_or(Vec3::X);
                show(
                    Transform {
                        translation: at,
                        rotation: Quat::from_rotation_arc(Vec3::X, dir),
                        scale: Vec3::new(0.9, 0.05, 0.05),
                    },
                    &inks.quill,
                );
            }
        }
    }
}
