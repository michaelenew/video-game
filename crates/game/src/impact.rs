//! **A blow that lands, seen landing** (`docs/design/exploration/0024_a_character_done_properly.md`).
//!
//! When one of the Reaver's own blows connects -- the Reave, the
//! Executioner, the Plunge, Gallows -- a quarter of a second at the point of
//! her blade: a flash of her violet, a ring thrown out square to the blow,
//! and a spray of ink shards flung on along it and falling away. The sound of
//! a blow is its weight (`crates/sound`); this is its picture, and it is what
//! sells a finisher's strength more than any number.
//!
//! **When** is the simulation's: the frame `hit_used` comes up on a move that
//! is out -- the same flag that stops a move hitting twice, so it fires once
//! for one blow on anything, a fighter or a creature. **Where** is the drawn
//! blade's tip (`slash`'s points), so the burst is on the steel. It plays
//! for seventeen frames of the simulation's clock, so it holds through the
//! blow's impact freeze, and a still capture lands on it.
//!
//! **Her thrown blades too**: the Shuriken, the Fan, the Whirl, the
//! Guillotine's lotus, anything of hers that is an effect rather than her
//! arm. The frame a part of one catches somebody new (`Effect::already_hit`
//! comes up), a smaller burst where that part is drawn (`effect_piece`), flung
//! on the way the blade was going.
//!
//! A fixed pool, made at startup: one burst a seat for her blows, and a
//! handful shared by every blade in the air; two materials each that fade.

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::view::NoFrustumCulling;
use sim::effects::{Effect, LOTUS_BLADES, MAX_EFFECTS};
use sim::monster::MAX_MONSTERS;
use sim::state::{Action, MAX_PLAYERS};

/// How long a burst lasts, in seconds of simulation, and how many shards
/// it throws.
const LIFE: f32 = 17.0 / 60.0;
const SHARDS: usize = 10;
/// Her violet, hot: the flash is nearly white at its heart.
const FLASH: [f32; 3] = [1.6, 1.0, 2.6];
const RING: [f32; 3] = [0.9, 0.45, 1.8];
/// Bursts shared by her thrown blades, and how big one is against a blow's.
const SPARKS: usize = 6;
const SPARK: f32 = 0.6;
/// Every burst: a seat's blow first, then the blades'.
const BURSTS: usize = MAX_PLAYERS + SPARKS;
/// Everything a blade can catch: the fighters and the creatures.
const VICTIMS: usize = MAX_PLAYERS + MAX_MONSTERS;

/// One burst: where, which way the blow went, how big, how long ago, and
/// which blow it was for, so one blow fires once (and its shards scatter the
/// same way every time it is drawn).
#[derive(Clone, Copy, Default)]
struct Burst {
    at: Vec3,
    dir: Vec3,
    size: f32,
    start: u32,
    age: f32,
    live: bool,
    blow: Option<(u8, u32)>,
}

/// The bursts, and what each effect slot had caught on the frame before --
/// the effect itself, so a slot taken by a new one is not read as new hits.
#[derive(Resource)]
pub struct Bursts {
    all: [Burst; BURSTS],
    seen: [Option<Effect>; MAX_EFFECTS],
    next: usize,
}

impl Default for Bursts {
    fn default() -> Bursts {
        Bursts {
            all: [Burst::default(); BURSTS],
            seen: [None; MAX_EFFECTS],
            next: 0,
        }
    }
}

/// A piece of a seat's burst.
#[derive(Component, Clone, Copy)]
pub enum Part {
    Flash(usize),
    Ring(usize),
    Shard(usize, usize),
}

/// The two materials of a seat's burst, which fade with it.
#[derive(Resource, Default)]
pub struct Looks(Vec<[Handle<StandardMaterial>; 2]>);

pub struct ImpactPlugin;

impl Plugin for ImpactPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Bursts>()
            .init_resource::<Looks>()
            .add_systems(Startup, spawn)
            .add_systems(
                PostUpdate,
                draw.after(bevy::transform::TransformSystem::TransformPropagate),
            );
    }
}

fn glow(colour: [f32; 3]) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::linear_rgb(colour[0], colour[1], colour[2]),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut looks: ResMut<Looks>,
) {
    let ball = meshes.add(Sphere::new(0.5).mesh().ico(3).unwrap());
    let ring = meshes.add(Torus::new(0.42, 0.5));
    let shard = meshes.add(crate::blades::shard());
    let ink = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.02, 0.01, 0.04),
        emissive: LinearRgba::rgb(0.5, 0.2, 1.2),
        perceptual_roughness: 0.6,
        ..default()
    });
    for owner in 0..BURSTS {
        let pair = [materials.add(glow(FLASH)), materials.add(glow(RING))];
        let hidden = || {
            (
                Transform::default(),
                Visibility::Hidden,
                NotShadowCaster,
                NoFrustumCulling,
            )
        };
        commands.spawn((
            Mesh3d(ball.clone()),
            MeshMaterial3d(pair[0].clone()),
            hidden(),
            Part::Flash(owner),
        ));
        commands.spawn((
            Mesh3d(ring.clone()),
            MeshMaterial3d(pair[1].clone()),
            hidden(),
            Part::Ring(owner),
        ));
        for k in 0..SHARDS {
            commands.spawn((
                Mesh3d(shard.clone()),
                MeshMaterial3d(ink.clone()),
                hidden(),
                Part::Shard(owner, k),
            ));
        }
        looks.0.push(pair);
    }
}

/// Which blow a fighter is in the live frames of and has landed, as the move
/// and the frame it started on; `None` if it has not landed.
fn landed(p: &sim::state::Player, frame: u32) -> Option<(u8, u32)> {
    let Action::Active { kind, left } = p.action else {
        return None;
    };
    if !p.hit_used {
        return None;
    }
    let m = sim::moves::get(p.class, kind);
    let into = (m.startup + m.active.saturating_sub(left)) as u32;
    Some((kind, frame.saturating_sub(into)))
}

/// A number from 0 to 1 for shard `k` of a blow, the same every time.
fn hash(seed: u32, k: usize) -> f32 {
    let mut x = seed.wrapping_mul(0x9E37_79B9) ^ (k as u32).wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

#[allow(clippy::too_many_arguments)] // a Bevy system: its arguments are what it reads
fn draw(
    sim: Res<crate::Sim>,
    rigs: Res<crate::reaver::Rigs>,
    points: Query<&GlobalTransform>,
    fighters: Query<(&crate::Fighter, &GlobalTransform)>,
    looks: Res<Looks>,
    mut bursts: ResMut<Bursts>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut parts: Query<(&Part, &mut Transform, &mut Visibility)>,
) {
    let frame = sim.cur.frame;
    for b in bursts.all.iter_mut() {
        b.age = frame.wrapping_sub(b.start) as f32 / 60.0;
        b.live = b.live && frame >= b.start && b.age < LIFE;
    }
    for (fighter, body) in fighters.iter() {
        let owner = fighter.0;
        let p = &sim.cur.players[owner];
        let b = &mut bursts.all[owner];
        if !crate::reaver::drawn(p.class) {
            continue;
        }
        let Some(blow) = landed(p, sim.cur.frame) else {
            continue;
        };
        if b.blow == Some(blow) {
            continue;
        }
        // The blade tip furthest out along her facing is the one that struck.
        let ahead = body.forward().as_vec3() * -1.0;
        let ahead = if ahead.length_squared() > 0.0 {
            ahead
        } else {
            Vec3::Z
        };
        let mut tip = body.translation() + Vec3::Y * 1.2 + ahead * 0.8;
        if let Some(blades) = rigs.blades[owner] {
            let mut best = f32::MIN;
            for [t, _] in blades {
                if let Ok(g) = points.get(t) {
                    let reach = (g.translation() - body.translation()).dot(ahead);
                    if reach > best {
                        best = reach;
                        tip = g.translation();
                    }
                }
            }
        }
        *b = Burst {
            at: tip,
            dir: ahead,
            size: 1.0,
            start: frame,
            age: 0.0,
            live: true,
            blow: Some(blow),
        };
    }

    // Her blades: a part that has caught somebody it had not a frame ago.
    for slot in 0..MAX_EFFECTS {
        let now = sim.cur.effects[slot];
        let before = std::mem::replace(&mut bursts.seen[slot], now);
        let Some(e) = now.filter(|e| crate::reaver::drawn(e.class)) else {
            continue;
        };
        let before = before.filter(|b| b.kind == e.kind && b.owner == e.owner && b.age <= e.age);
        for part in 0..LOTUS_BLADES {
            let new = (0..VICTIMS).any(|victim| {
                e.already_hit(part, victim) && !before.is_some_and(|b| b.already_hit(part, victim))
            });
            if !new {
                continue;
            }
            let Some(piece) = crate::effect_piece(&e, part).or_else(|| crate::effect_piece(&e, 0))
            else {
                continue;
            };
            let way = crate::fx3(e.dir);
            let k = MAX_PLAYERS + bursts.next % SPARKS;
            bursts.next = bursts.next.wrapping_add(1);
            bursts.all[k] = Burst {
                at: piece.at,
                dir: way.try_normalize().unwrap_or(Vec3::Y),
                size: SPARK,
                start: frame,
                age: 0.0,
                live: true,
                blow: Some((slot as u8, sim.cur.frame.wrapping_add(part as u32 * 7919))),
            };
        }
    }

    for (owner, pair) in looks.0.iter().enumerate() {
        let b = bursts.all[owner];
        let fade = if b.live { 1.0 - b.age / LIFE } else { 0.0 };
        for (k, colour) in [(0, FLASH), (1, RING)] {
            let want = Color::linear_rgb(colour[0] * fade, colour[1] * fade, colour[2] * fade);
            if materials
                .get(&pair[k])
                .is_some_and(|m| m.base_color != want)
            {
                if let Some(m) = materials.get_mut(&pair[k]) {
                    m.base_color = want;
                }
            }
        }
    }

    for (part, mut tf, mut seen) in parts.iter_mut() {
        let owner = match *part {
            Part::Flash(o) | Part::Ring(o) | Part::Shard(o, _) => o,
        };
        let b = bursts.all[owner];
        let want = if b.live {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *seen != want {
            *seen = want;
        }
        if !b.live {
            continue;
        }
        let t = (b.age / LIFE).clamp(0.0, 1.0);
        let out = 1.0 - (1.0 - t) * (1.0 - t);
        match *part {
            Part::Flash(_) => {
                tf.translation = b.at;
                tf.scale = Vec3::splat((0.15 + 0.55 * out.sqrt()) * b.size);
            }
            Part::Ring(_) => {
                tf.translation = b.at + b.dir * 0.1 * out * b.size;
                tf.rotation = Quat::from_rotation_arc(Vec3::Y, b.dir);
                let r = (0.2 + 1.4 * out) * b.size;
                tf.scale = Vec3::new(r, 0.15 * b.size, r);
            }
            Part::Shard(_, k) => {
                let seed = b
                    .blow
                    .map_or(0, |(kind, start)| start.wrapping_mul(31) ^ kind as u32);
                let (u, v, w) = (hash(seed, k), hash(seed, k + 17), hash(seed, k + 41));
                // Flung on along the blow, in a cone, and falling.
                let side = b.dir.cross(Vec3::Y).normalize_or(Vec3::X);
                let up = side.cross(b.dir).normalize_or(Vec3::Y);
                let a = u * std::f32::consts::TAU;
                let spread = 0.35 + 0.55 * v;
                let way = (b.dir + (side * a.cos() + up * a.sin()) * spread).normalize();
                let speed = (2.2 + 2.6 * w) * b.size;
                let at = b.at + way * speed * b.age - Vec3::Y * 4.0 * b.age * b.age;
                tf.translation = at;
                tf.rotation = Quat::from_rotation_arc(Vec3::Y, way);
                let size = (0.06 + 0.08 * w) * (1.0 - t * 0.8) * b.size;
                tf.scale = Vec3::new(size, size * 3.0, size);
            }
        }
    }
}
