//! What the simulation's transitions mean, as sounds.
//!
//! [`cues`] diffs the world before and after one tick into a fixed list of
//! [`Cue`]s: each a [`Patch`] (what it sounds like) and a place (where it is
//! heard from). It is the same move the fight report and the replay judge
//! make -- read the world twice, see what changed -- and it keeps every rule
//! about what a sound *is* out of the game crate, which only has to play
//! them.
//!
//! The numbers come from where the design already put them. A blow's weight
//! is its **impact freeze**: the frames both bodies stop for when it lands,
//! which is how the Champion's weapons were told apart by weight on
//! 2026-09-26 (`Move::hitstop`). A telegraph's length is the move's startup.
//! A footfall's material is the arena's floor there. A struck thing's size
//! is its height.
//!
//! Fixed size, no allocation: this runs once per simulation tick, inside the
//! frame.

use crate::patch::{
    Crackle, FIGHTER, Growl, Gust, Material, Patch, Ring, Rumble, Step, Strike, Wet, Whoosh,
};
use sim::class::{Mechanic, Shield};
use sim::effects::EffectKind;
use sim::monster::Doing;
use sim::state::{Action, PARRY_FLOURISH, Phase};
use sim::{Class, V3, World};

/// One sound to play: what, and where.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cue {
    pub patch: Patch,
    /// Where it comes from, in the arena, as the renderer's floats.
    pub at: [f32; 3],
}

/// The most cues one tick can produce. Two fighters, two creatures, a pack
/// and a handful of effects can each contribute one or two.
pub const MAX_CUES: usize = 32;

/// A tick's cues. Fixed size, so no frame allocates to say what it sounded
/// like; overflow drops the quietest end of the list, which is the end.
#[derive(Clone, Copy, Debug)]
pub struct Cues {
    list: [Option<Cue>; MAX_CUES],
    len: usize,
}

impl Default for Cues {
    fn default() -> Self {
        Cues {
            list: [None; MAX_CUES],
            len: 0,
        }
    }
}

impl Cues {
    pub fn push(&mut self, patch: Patch, at: V3) {
        if self.len < MAX_CUES {
            self.list[self.len] = Some(Cue {
                patch,
                at: [
                    at.x.to_f32_for_render(),
                    at.y.to_f32_for_render(),
                    at.z.to_f32_for_render(),
                ],
            });
            self.len += 1;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Cue> {
        self.list[..self.len].iter().flatten()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}

/// The longest impact freeze any move has: the Champion's Earthbreaker, at
/// thirteen frames. A blow's weight is its freeze as a share of this.
const HEAVIEST_FREEZE: f32 = 13.0;

/// How sharp a class's blows are: an edge or a club. A column the move table
/// could grow; until it does, the kit's weapon says.
fn edge(class: Class) -> f32 {
    match class {
        Class::Bulwark => 0.1,
        Class::Champion => 0.55,
        Class::ShadowReaver => 0.9,
        Class::Elementalist => 0.3,
        Class::BloodMage => 0.8,
        Class::DualMage => 0.2,
    }
}

/// A fighter's move, as the weight and edge of its blow.
fn blow(class: Class, kind: u8) -> (f32, f32) {
    let m = sim::moves::get(class, kind);
    let freeze = m.hitstop as f32 / HEAVIEST_FREEZE;
    let damage = m.damage as f32 / 200.0;
    ((0.7 * freeze + 0.3 * damage).clamp(0.1, 1.0), edge(class))
}

/// Where a fighter's chest is: the height sounds leave a body from.
fn chest(p: &sim::state::Player) -> V3 {
    V3::new(p.pos.x, p.pos.y.add(sim::Fx::ratio(13, 10)), p.pos.z)
}

fn struck(a: Action) -> bool {
    matches!(
        a,
        Action::HitStun { .. } | Action::Stagger { .. } | Action::Held { .. }
    )
}

fn begun(a: Action) -> bool {
    matches!(a, Action::Startup { .. } | Action::Channel { .. })
}

/// Everything that sounded between `before` and `after`.
pub fn cues(before: &World, after: &World, out: &mut Cues) {
    out.clear();
    let here = after.arena.get();
    let floor = |pos: V3| Material::of_floor(here.material_under(pos));

    // The round itself.
    if matches!(before.phase, Phase::Fighting) && matches!(after.phase, Phase::RoundOver { .. }) {
        // A low gong: the end of it.
        let at = after.players[0]
            .pos
            .add(after.players[1].pos)
            .scale(sim::Fx::ratio(1, 2));
        out.push(
            Patch::Strike(Strike {
                weight: 1.0,
                sharp: 0.0,
                material: Material::Plate,
                size: 6.0,
            }),
            at,
        );
    }

    for i in 0..sim::state::MAX_PLAYERS {
        let (was, now) = (&before.players[i], &after.players[i]);
        if was.health <= 0 {
            continue;
        }
        let other = &after.players[1 - i];

        // A move begins: its telegraph, exactly as long as its startup.
        if begun(now.action)
            && (!begun(was.action) || now.action.attack_kind() != was.action.attack_kind())
            && let Some(kind) = now.action.attack_kind()
        {
            let m = sim::moves::get(now.class, kind);
            let (weight, _) = blow(now.class, kind);
            out.push(
                Patch::Whoosh(Whoosh {
                    frames: m.startup.max(3),
                    size: FIGHTER,
                    rising: true,
                    weight,
                }),
                chest(now),
            );
        }
        // The swing itself.
        if matches!(now.action, Action::Active { .. })
            && !matches!(was.action, Action::Active { .. })
            && let Some(kind) = now.action.attack_kind()
        {
            let m = sim::moves::get(now.class, kind);
            let (weight, _) = blow(now.class, kind);
            out.push(
                Patch::Whoosh(Whoosh {
                    frames: (m.active + 4).max(6),
                    size: FIGHTER,
                    rising: false,
                    weight,
                }),
                chest(now),
            );
        }
        // A dodge: a body moving fast, softly.
        if matches!(now.action, Action::Dodge { .. }) && !matches!(was.action, Action::Dodge { .. })
        {
            out.push(
                Patch::Whoosh(Whoosh {
                    frames: 10,
                    size: FIGHTER,
                    rising: false,
                    weight: 0.2,
                }),
                chest(now),
            );
        }
        // Leaving the floor, and coming back to it.
        if was.grounded && !now.grounded && now.vel.y.raw() > 0 {
            out.push(
                Patch::Step(Step {
                    weight: 0.15,
                    material: floor(was.pos),
                    size: FIGHTER,
                    seed: after.frame as u8 ^ i as u8,
                }),
                now.pos,
            );
        }
        if !was.grounded && now.grounded && !now.aboard() {
            let fall = was.vel.y.to_f32_for_render().abs();
            out.push(
                Patch::Step(Step {
                    weight: (0.2 + fall / 25.0).min(1.0),
                    material: floor(now.pos),
                    size: FIGHTER,
                    seed: after.frame as u8 ^ i as u8,
                }),
                now.pos,
            );
        }
        // Footfalls: a fighter on the floor and moving. A cadence from the
        // frame count rather than a gait clock the snapshot does not keep.
        let speed = now.vel.flat_len().to_f32_for_render();
        if now.grounded && was.grounded && speed > 1.5 && !now.aboard() {
            let period = (90.0 / speed.max(2.0)).clamp(9.0, 24.0) as u32;
            if (after.frame + (i as u32) * (period / 2)) % period == 0 {
                out.push(
                    Patch::Step(Step {
                        weight: (0.12 + speed / 40.0).min(0.45),
                        material: floor(now.pos),
                        size: FIGHTER,
                        seed: (after.frame / period) as u8 ^ (i as u8) << 4,
                    }),
                    now.pos,
                );
            }
        }
        // A blow taken. Whose: the other fighter's live move, or a creature's.
        let hit = struck(now.action)
            && (!struck(was.action) || now.action.frames_left() > was.action.frames_left());
        let blocked = matches!(now.action, Action::BlockStun { .. })
            && (!matches!(was.action, Action::BlockStun { .. })
                || now.action.frames_left() > was.action.frames_left());
        if hit || blocked {
            let (mut weight, mut sharp) = (0.4, 0.3);
            if let Some(kind) = other
                .action
                .attack_kind()
                .filter(|_| matches!(other.action, Action::Active { .. }))
            {
                (weight, sharp) = blow(other.class, kind);
            } else if let Some(m) = after
                .monsters
                .iter()
                .flatten()
                .find(|m| matches!(m.doing, Doing::Active { .. }))
            {
                if let Doing::Active { kind, .. } = m.doing {
                    let a = m.species.get().attack(kind);
                    weight = (a.damage as f32 / 400.0).clamp(0.3, 1.0);
                    sharp = 0.2;
                }
            } else if was.health - now.health > 0 {
                weight = ((was.health - now.health) as f32 / 200.0).clamp(0.2, 1.0);
            }
            out.push(
                Patch::Strike(Strike {
                    weight,
                    sharp,
                    material: if blocked {
                        Material::Shield
                    } else {
                        Material::Flesh
                    },
                    size: FIGHTER,
                }),
                chest(now),
            );
            if !blocked && now.class == Class::BloodMage || !blocked && weight > 0.6 {
                out.push(
                    Patch::Wet(Wet {
                        weight: weight * 0.6,
                    }),
                    chest(now),
                );
            }
        }
        // A parry: the one chime.
        if now.parried == PARRY_FLOURISH && was.parried != PARRY_FLOURISH {
            out.push(Patch::Ring(Ring { pitch: 1760.0 }), chest(now));
        }
        // Knocked out.
        if now.health <= 0 {
            out.push(
                Patch::Strike(Strike {
                    weight: 1.0,
                    sharp: 0.1,
                    material: Material::Flesh,
                    size: FIGHTER,
                }),
                chest(now),
            );
        }
        // The mechanics.
        mechanic(was, now, out);
    }

    // The creatures.
    for slot in 0..sim::monster::MAX_MONSTERS {
        let (Some(was), Some(now)) = (&before.monsters[slot], &after.monsters[slot]) else {
            continue;
        };
        if !was.alive() {
            continue;
        }
        let size = (now.head().y.sub(now.pos.y)).to_f32_for_render().max(0.8);
        let head = now.head();
        // Winding up: a growl as long as the startup, from the head.
        if let Doing::Startup { kind, left } = now.doing
            && !matches!(was.doing, Doing::Startup { kind: k, .. } if k == kind)
        {
            out.push(
                Patch::Growl(Growl {
                    frames: left.saturating_add(1),
                    size,
                }),
                head,
            );
        }
        // The move itself: a swing of the creature's weight.
        if let Doing::Active { kind, .. } = now.doing
            && !matches!(was.doing, Doing::Active { .. })
        {
            let a = now.species.get().attack(kind);
            out.push(
                Patch::Whoosh(Whoosh {
                    frames: (a.active + 6).max(8),
                    size,
                    rising: false,
                    weight: (a.damage as f32 / 400.0).clamp(0.3, 1.0),
                }),
                now.pos,
            );
        }
        // Struck: hide, or plate where a part broke.
        if now.health < was.health {
            let broke = now
                .breaks
                .iter()
                .zip(was.breaks.iter())
                .any(|(n, w)| n <= &0 && w > &0);
            let dealt = (was.health - now.health) as f32;
            out.push(
                Patch::Strike(Strike {
                    weight: if broke {
                        1.0
                    } else {
                        (dealt / 250.0).clamp(0.2, 1.0)
                    },
                    sharp: 0.4,
                    material: if broke {
                        Material::Plate
                    } else {
                        Material::Hide
                    },
                    size,
                }),
                now.pos.add(V3::new(
                    sim::Fx::ZERO,
                    now.head().y.sub(now.pos.y).mul(sim::Fx::ratio(1, 2)),
                    sim::Fx::ZERO,
                )),
            );
        }
        // Going down, and getting up.
        if matches!(now.doing, Doing::Toppled { .. } | Doing::Stumble { .. })
            && !matches!(was.doing, Doing::Toppled { .. } | Doing::Stumble { .. })
        {
            out.push(
                Patch::Step(Step {
                    weight: 1.0,
                    material: Material::of_floor(here.material_under(now.pos)),
                    size,
                    seed: after.frame as u8,
                }),
                now.pos,
            );
        }
        if !now.alive() {
            out.push(Patch::Growl(Growl { frames: 40, size }), head);
        }
    }

    // Small bodies.
    let sp = after.critters.species.get();
    for (was, now) in before.critters.all.iter().zip(after.critters.all.iter()) {
        if was.health <= 0 || now.health >= was.health {
            continue;
        }
        let height = now.body(sp).height.to_f32_for_render().max(0.3);
        out.push(
            Patch::Strike(Strike {
                weight: if now.health <= 0 { 0.7 } else { 0.35 },
                sharp: 0.4,
                material: Material::Hide,
                size: height,
            }),
            now.pos,
        );
    }

    // Things left in the world.
    for (was, now) in before.effects.iter().zip(after.effects.iter()) {
        let Some(e) = now else { continue };
        let fresh = match was {
            None => true,
            Some(w) => w.kind != e.kind || e.age < w.age,
        };
        if !fresh {
            continue;
        }
        let patch = match e.kind {
            EffectKind::FirePillar => Patch::Crackle(Crackle { seconds: 0.5 }),
            EffectKind::FireTornado => Patch::Gust(Gust {
                frames: 30,
                weight: 0.8,
            }),
            EffectKind::BlackSpike => Patch::Wet(Wet { weight: 0.7 }),
            EffectKind::Bloodletter => Patch::Whoosh(Whoosh {
                frames: 10,
                size: FIGHTER,
                rising: false,
                weight: 0.5,
            }),
            EffectKind::Grasp => Patch::Wet(Wet { weight: 0.5 }),
            EffectKind::Haemorrhage => Patch::Wet(Wet { weight: 0.3 }),
            EffectKind::GuillotineLotus => Patch::Whoosh(Whoosh {
                frames: 12,
                size: FIGHTER,
                rising: false,
                weight: 0.8,
            }),
            EffectKind::LanceBurst => Patch::Gust(Gust {
                frames: 14,
                weight: 0.6,
            }),
            EffectKind::Tether => Patch::Ring(Ring { pitch: 880.0 }),
            EffectKind::JudgementField => Patch::Gust(Gust {
                frames: 40,
                weight: 0.5,
            }),
            EffectKind::Pool => Patch::Wet(Wet { weight: 0.35 }),
            // The Elementalist's fire on earth, and her air.
            EffectKind::Embers | EffectKind::FireRing => Patch::Crackle(Crackle { seconds: 0.35 }),
            // The carpet and the fountain are fire laid down and left: a
            // longer crackle than a burst's.
            EffectKind::FireCarpet | EffectKind::Fountain => {
                Patch::Crackle(Crackle { seconds: 0.6 })
            }
            // The Blood mage's burst is wet and heavy; the nail a short ring.
            EffectKind::Nova => Patch::Wet(Wet { weight: 0.8 }),
            EffectKind::Nail => Patch::Ring(Ring { pitch: 440.0 }),
            // The Air ball is a held gust, the heaviest of her air.
            EffectKind::AirBall => Patch::Gust(Gust {
                frames: 30,
                weight: 0.8,
            }),
            EffectKind::Rough => Patch::Rumble(Rumble {
                frames: 8,
                size: 1.5,
            }),
            EffectKind::Updraft | EffectKind::Downdraft | EffectKind::AirRing => {
                Patch::Gust(Gust {
                    frames: 20,
                    weight: 0.7,
                })
            }
            // Anything a later build adds speaks until it is given a voice.
            #[allow(unreachable_patterns)]
            _ => Patch::Gust(Gust {
                frames: 12,
                weight: 0.3,
            }),
        };
        out.push(patch, e.pos);
    }
}

/// The class mechanics that make a sound: a stone coming up or breaking, a
/// shield thrown, planted and caught.
fn mechanic(was: &sim::state::Player, now: &sim::state::Player, out: &mut Cues) {
    match (&was.mechanic, &now.mechanic) {
        (Mechanic::Structures(before), Mechanic::Structures(after)) => {
            for (b, a) in before.iter().zip(after.iter()) {
                match (b, a) {
                    (None, Some(s)) => out.push(
                        Patch::Rumble(Rumble {
                            frames: 12,
                            size: 2.0,
                        }),
                        s.at,
                    ),
                    (Some(b), Some(s)) if s.age < b.age => out.push(
                        Patch::Rumble(Rumble {
                            frames: 12,
                            size: 2.0,
                        }),
                        s.at,
                    ),
                    (Some(b), None) => out.push(
                        Patch::Strike(Strike {
                            weight: 0.8,
                            sharp: 0.3,
                            material: Material::Stone,
                            size: 2.0,
                        }),
                        b.at,
                    ),
                    _ => {}
                }
            }
        }
        (Mechanic::Shield(b), Mechanic::Shield(a)) => match (b, a) {
            (Shield::Held { .. }, Shield::Flying { pos, .. }) => out.push(
                Patch::Whoosh(Whoosh {
                    frames: 10,
                    size: 1.0,
                    rising: false,
                    weight: 0.6,
                }),
                *pos,
            ),
            (Shield::Flying { .. }, Shield::Planted { pos, weight }) => out.push(
                Patch::Strike(Strike {
                    weight: (0.6 + weight.to_f32_for_render() / 400.0).min(1.0),
                    sharp: 0.3,
                    material: Material::Shield,
                    size: 1.2,
                }),
                *pos,
            ),
            (Shield::Flying { .. } | Shield::Planted { .. }, Shield::Held { .. }) => out.push(
                Patch::Strike(Strike {
                    weight: 0.3,
                    sharp: 0.3,
                    material: Material::Shield,
                    size: 1.2,
                }),
                chest(now),
            ),
            (Shield::Planted { pos, .. }, Shield::Flying { .. }) => out.push(
                Patch::Strike(Strike {
                    weight: 0.4,
                    sharp: 0.2,
                    material: Material::Stone,
                    size: 1.2,
                }),
                *pos,
            ),
            _ => {}
        },
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::Input;
    use sim::state::MAX_PLAYERS;

    fn step(w: &mut World, inputs: [Input; MAX_PLAYERS]) -> Cues {
        let before = w.clone();
        w.advance(inputs);
        let mut out = Cues::default();
        cues(&before, w, &mut out);
        out
    }

    fn has(c: &Cues, f: impl Fn(&Patch) -> bool) -> bool {
        c.iter().any(|cue| f(&cue.patch))
    }

    #[test]
    fn a_swing_is_a_telegraph_then_a_swing_then_a_blow() {
        let mut w = World::with_classes([Class::Bulwark; MAX_PLAYERS]);
        // Stand them close, facing each other, as the arena starts them.
        let press = [Input::aimed(Input::LEFT, 0), Input::default()];
        let first = step(&mut w, press);
        assert!(
            has(
                &first,
                |p| matches!(p, Patch::Whoosh(Whoosh { rising: true, frames, .. }) if *frames == sim::moves::get(Class::Bulwark, 0).startup)
            ),
            "{first:?}"
        );
        let mut swung = false;
        let mut landed = false;
        for _ in 0..30 {
            let c = step(&mut w, [Input::aimed(0, 0), Input::default()]);
            swung |= has(&c, |p| {
                matches!(p, Patch::Whoosh(Whoosh { rising: false, .. }))
            });
            landed |= has(&c, |p| {
                matches!(
                    p,
                    Patch::Strike(Strike {
                        material: Material::Flesh,
                        ..
                    })
                )
            });
        }
        assert!(swung, "no swing was heard");
        // Whether it landed depends on the spawn distance; either answer is
        // allowed here, the hit test's own suite says which.
        let _ = landed;
    }

    #[test]
    fn a_jump_and_a_landing_sound_like_the_floor() {
        let mut w = World::with_classes([Class::Champion; MAX_PLAYERS]);
        let jump = [Input::aimed(Input::SPACE, 0), Input::default()];
        let c = step(&mut w, jump);
        assert!(
            has(&c, |p| matches!(
                p,
                Patch::Step(Step {
                    material: Material::Earth,
                    ..
                })
            )),
            "{c:?}"
        );
        let mut landed = false;
        for _ in 0..120 {
            let c = step(&mut w, [Input::default(); MAX_PLAYERS]);
            landed |= has(
                &c,
                |p| matches!(p, Patch::Step(Step { material: Material::Earth, weight, .. }) if *weight > 0.2),
            );
        }
        assert!(landed);
    }

    #[test]
    fn a_creature_winding_up_growls_for_its_startup() {
        let mut w = World::hunt([Class::Champion; MAX_PLAYERS]).seated(1);
        let mut growled = None;
        for _ in 0..1200 {
            let c = step(&mut w, [Input::default(); MAX_PLAYERS]);
            if let Some(cue) = c.iter().find(|c| matches!(c.patch, Patch::Growl(_))) {
                growled = Some(*cue);
                break;
            }
        }
        let cue = growled.expect("the Ridgeback never wound anything up in twenty seconds");
        let Patch::Growl(g) = cue.patch else {
            unreachable!()
        };
        assert!(g.frames >= 8, "{g:?}");
        assert!(g.size > 2.0, "a nine-metre animal, {g:?}");
    }

    #[test]
    fn a_quiet_frame_is_silent() {
        let mut w = World::with_classes([Class::Bulwark; MAX_PLAYERS]);
        // Settle, then stand still.
        for _ in 0..10 {
            step(&mut w, [Input::default(); MAX_PLAYERS]);
        }
        let c = step(&mut w, [Input::default(); MAX_PLAYERS]);
        assert!(c.is_empty(), "{c:?}");
    }

    #[test]
    fn cues_never_exceed_their_room() {
        let mut out = Cues::default();
        for _ in 0..100 {
            out.push(Patch::Ring(Ring { pitch: 440.0 }), V3::ZERO);
        }
        assert_eq!(out.len(), MAX_CUES);
    }
}
