//! Drawing what you carry: eggs and trophies on the floor, each seat's
//! morph at heel, and a few lines saying what you hold and what your morph
//! just became (`sim::keep`).
//!
//! **Nothing here decides anything.** Where an egg lies, what a morph is
//! and where it stands are the snapshot's; this puts a mesh on each. The
//! morph's body is the builder's own (`sim::breed::build`) -- the same
//! boxes its sheets are drawn from -- posed by its gait when it walks.
//!
//! **Obviously something to pick up.** An egg is big, hot pink and
//! glowing -- every egg, a colour nothing else in the valley is -- it bobs
//! and turns, and a pink column of light stands over it that can be seen
//! across a range; a trophy is the same, smaller, in gold. Nothing else in
//! the valley moves like that.

use bevy::prelude::*;
use sim::breed::build::{self, Body};
use sim::breed::morph::{self, Line, Morph};
use sim::breed::{locomotion, posture};
use sim::keep::{Item, LOOSE};
use sim::species::SpeciesId;
use sim::state::MAX_PLAYERS;

use crate::species::Paint;

/// What an egg looks like, whoever laid it: a colour nothing else in the
/// valley is, so it reads as something to pick up from across a range.
const EGG_SHELL: Paint = Paint {
    rgb: [1.0, 0.32, 0.72],
    glow: [0.0; 3],
    roughness: 0.35,
};

/// The most parts a built body has.
const PARTS: usize = sim::beast::MAX_PARTS;

/// One part of a seat's morph.
#[derive(Component)]
pub struct MorphPart(pub usize, pub usize);

/// What lies in a slot on the floor, and its column of light.
#[derive(Component)]
pub struct LooseItem(pub usize);

#[derive(Component)]
pub struct LooseBeam(pub usize);

/// The lines about what you carry.
#[derive(Component)]
pub struct KeepText;

/// Meshes and materials, made once.
#[derive(Resource)]
pub struct Kit {
    part: Handle<Mesh>,
    egg: Handle<Mesh>,
    trophy: Handle<Mesh>,
    beam: Handle<Mesh>,
    /// By species id: a morph's body, its limbs, its head; an egg.
    body: Vec<Handle<StandardMaterial>>,
    limb: Vec<Handle<StandardMaterial>>,
    head: Vec<Handle<StandardMaterial>>,
    egg_of: Vec<Handle<StandardMaterial>>,
    /// A morph that has eaten no egg.
    plain: [Handle<StandardMaterial>; 3],
    gold: Handle<StandardMaterial>,
    beam_egg: Handle<StandardMaterial>,
    beam_trophy: Handle<StandardMaterial>,
}

fn paint(
    materials: &mut Assets<StandardMaterial>,
    p: Paint,
    lift: f32,
    glow: f32,
) -> Handle<StandardMaterial> {
    let [r, g, b] = p.rgb;
    let up = |c: f32| (c + (1.0 - c) * lift).min(1.0);
    materials.add(StandardMaterial {
        base_color: Color::srgb(up(r), up(g), up(b)),
        emissive: LinearRgba::rgb(up(r) * glow, up(g) * glow, up(b) * glow),
        perceptual_roughness: p.roughness,
        ..default()
    })
}

fn beam(materials: &mut Assets<StandardMaterial>, rgb: [f32; 3]) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(rgb[0], rgb[1], rgb[2], 0.35),
        emissive: LinearRgba::rgb(rgb[0] * 2.0, rgb[1] * 2.0, rgb[2] * 2.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    })
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut body = Vec::new();
    let mut limb = Vec::new();
    let mut head = Vec::new();
    let mut egg_of = Vec::new();
    for id in 0..sim::species::COUNT {
        let look = crate::species::look(SpeciesId(id as u8));
        body.push(paint(&mut materials, look.armour, 0.0, 0.0));
        limb.push(paint(&mut materials, look.breakable, 0.0, 0.0));
        head.push(paint(&mut materials, look.weak, 0.0, 0.0));
        // Every egg the one vivid colour, whatever laid it -- a creature's
        // own colour is often a stone's -- with a band of its creature's
        // beneath (`EGG_SHELL`).
        egg_of.push(paint(&mut materials, EGG_SHELL, 0.0, 1.2));
    }
    let plain = Paint {
        rgb: [0.78, 0.74, 0.66],
        glow: [0.0; 3],
        roughness: 0.7,
    };
    let kit = Kit {
        part: meshes.add(crate::shapes::soft_box(Vec3::ONE, 0.5, 10, None)),
        egg: meshes.add(Sphere::new(0.5).mesh().uv(24, 16)),
        trophy: meshes.add(Sphere::new(0.5).mesh().ico(0).unwrap()),
        beam: meshes.add(Cylinder::new(0.12, 1.0)),
        body,
        limb,
        head,
        egg_of,
        plain: [
            paint(&mut materials, plain, 0.0, 0.0),
            paint(&mut materials, plain, 0.2, 0.0),
            paint(&mut materials, plain, 0.4, 0.05),
        ],
        gold: paint(
            &mut materials,
            Paint {
                rgb: [0.95, 0.72, 0.22],
                glow: [0.0; 3],
                roughness: 0.3,
            },
            0.0,
            0.6,
        ),
        beam_egg: beam(&mut materials, EGG_SHELL.rgb),
        beam_trophy: beam(&mut materials, [1.0, 0.8, 0.3]),
    };
    for seat in 0..MAX_PLAYERS {
        for part in 0..PARTS {
            commands.spawn((
                Mesh3d(kit.part.clone()),
                MeshMaterial3d(kit.plain[0].clone()),
                Transform::default(),
                Visibility::Hidden,
                MorphPart(seat, part),
            ));
        }
    }
    for slot in 0..LOOSE {
        commands.spawn((
            Mesh3d(kit.egg.clone()),
            MeshMaterial3d(kit.gold.clone()),
            Transform::default(),
            Visibility::Hidden,
            LooseItem(slot),
        ));
        commands.spawn((
            Mesh3d(kit.beam.clone()),
            MeshMaterial3d(kit.beam_egg.clone()),
            Transform::default(),
            Visibility::Hidden,
            bevy::pbr::NotShadowCaster,
            LooseBeam(slot),
        ));
    }
    commands.insert_resource(kit);
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(0.97, 0.95, 0.88, 0.95)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(96.0),
            left: Val::Px(18.0),
            max_width: Val::Px(520.0),
            ..default()
        },
        KeepText,
    ));
}

fn fx3(v: sim::V3) -> Vec3 {
    Vec3::new(
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    )
}

/// A rotation from two nearly-orthogonal axes, squared up.
fn orient(x: Vec3, y: Vec3) -> Quat {
    let x = x.normalize_or_zero();
    let y = (y - x * x.dot(y)).normalize_or_zero();
    if x == Vec3::ZERO || y == Vec3::ZERO {
        return Quat::IDENTITY;
    }
    Quat::from_mat3(&Mat3::from_cols(x, y, x.cross(y)))
}

/// What a part is, for its colour: the body, a limb, or the head.
fn role(name: &str) -> usize {
    if name.starts_with("head") || name == "jaw" || name == "horns" || name == "crest" {
        2
    } else if name.starts_with("fore")
        || name.starts_with("mid")
        || name.starts_with("hind")
        || name.starts_with("wing_")
    {
        1
    } else {
        0
    }
}

/// Each seat's morph posed: a body built once per frame per seat.
pub fn place_morphs(
    sim: Res<crate::Sim>,
    kit: Res<Kit>,
    mut parts: Query<(
        &MorphPart,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut posed: Local<[Option<(u64, posture::Posture)>; MAX_PLAYERS]>,
) {
    let w = &sim.cur;
    let seats = (w.seats as usize).clamp(1, MAX_PLAYERS);
    let mut bodies: [Option<Body>; MAX_PLAYERS] = [None; MAX_PLAYERS];
    if w.valley.on {
        for seat in 0..seats {
            let m: &Morph = &w.valley.keep.morphs[seat];
            let heel = w.heel[seat];
            if !heel.placed {
                continue;
            }
            let g = m.shown();
            let key = g.fingerprint() ^ (m.growth.raw() as u64) << 40;
            let q = match posed[seat] {
                Some((k, q)) if k == key => q,
                _ => {
                    let q = posture::posture(&g);
                    posed[seat] = Some((key, q));
                    q
                }
            };
            let d = if heel.going {
                let gait = locomotion::at(&g, &q, sim::Fx::ratio(1, 2));
                locomotion::drive(&g, &q, &gait, heel.phase, sim::Fx::ZERO)
            } else {
                locomotion::rest(&g)
            };
            bodies[seat] = Some(build::body_driven(&g, &q, &d));
        }
    }
    for (part, mut transform, mut visible, mut material) in parts.iter_mut() {
        let (seat, i) = (part.0, part.1);
        let Some(body) = bodies.get(seat).copied().flatten() else {
            *visible = Visibility::Hidden;
            continue;
        };
        if i >= body.part_count {
            *visible = Visibility::Hidden;
            continue;
        }
        let heel = w.heel[seat];
        let yaw = heel.facing.to_f32_for_render() * std::f32::consts::TAU;
        // The builder's frame turned toward `+z` by the facing; Bevy's turn
        // about `y` goes the other way.
        let turn = Quat::from_rotation_y(-yaw);
        let shape = body.parts[i].shape;
        let u = body.axes[i];
        let (v, _) = build::frame(u);
        let at = fx3(heel.at) + turn * fx3(body.centre(i));
        *transform = Transform {
            translation: at,
            rotation: turn * orient(fx3(u), fx3(v)),
            scale: fx3(shape.max.sub(shape.min)).max(Vec3::splat(0.02)),
        };
        *visible = Visibility::Inherited;
        let m = &w.valley.keep.morphs[seat];
        let r = role(body.parts[i].name);
        let wanted = if m.of == Morph::UNFORMED {
            &kit.plain[r]
        } else {
            let s = m.of as usize;
            match r {
                0 => &kit.body[s],
                1 => &kit.limb[s],
                _ => &kit.head[s],
            }
        };
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }
}

/// Eggs and trophies on the floor: bobbing, turning, under a column of light.
#[allow(clippy::type_complexity)]
pub fn place_loose(
    sim: Res<crate::Sim>,
    kit: Res<Kit>,
    time: Res<Time>,
    mut items: Query<
        (
            &LooseItem,
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Mesh3d,
        ),
        Without<LooseBeam>,
    >,
    mut beams: Query<
        (
            &LooseBeam,
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        Without<LooseItem>,
    >,
) {
    let t = time.elapsed_secs();
    let w = &sim.cur;
    for (slot, mut transform, mut visible, mut material, mut mesh) in items.iter_mut() {
        let Some(l) = w.loose[slot.0].filter(|_| w.valley.on) else {
            *visible = Visibility::Hidden;
            continue;
        };
        let bob = (t * 2.0 + slot.0 as f32).sin() * 0.12;
        let (scale, want_mesh, want_mat) = match l.item {
            Item::Egg(s) => (
                Vec3::new(0.7, 0.95, 0.7),
                &kit.egg,
                &kit.egg_of[(s.0 as usize).min(kit.egg_of.len() - 1)],
            ),
            Item::Trophy(..) => (Vec3::splat(0.5), &kit.trophy, &kit.gold),
        };
        *transform = Transform {
            translation: fx3(l.at) + Vec3::new(0.0, scale.y * 0.5 + 0.25 + bob, 0.0),
            rotation: Quat::from_rotation_y(t * 1.2 + slot.0 as f32),
            scale,
        };
        *visible = Visibility::Inherited;
        if mesh.0 != *want_mesh {
            mesh.0 = want_mesh.clone();
        }
        if material.0 != *want_mat {
            material.0 = want_mat.clone();
        }
    }
    for (slot, mut transform, mut visible, mut material) in beams.iter_mut() {
        let Some(l) = w.loose[slot.0].filter(|_| w.valley.on) else {
            *visible = Visibility::Hidden;
            continue;
        };
        let (height, want) = match l.item {
            Item::Egg(_) => (14.0, &kit.beam_egg),
            Item::Trophy(..) => (8.0, &kit.beam_trophy),
        };
        let pulse = 1.0 + (t * 3.0 + slot.0 as f32).sin() * 0.15;
        *transform = Transform {
            translation: fx3(l.at) + Vec3::new(0.0, height * 0.5, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(pulse, height, pulse),
        };
        *visible = Visibility::Inherited;
        if material.0 != *want {
            material.0 = want.clone();
        }
    }
}

fn species_name(s: SpeciesId) -> &'static str {
    s.get().name
}

fn station(i: usize) -> &'static str {
    ["front", "middle-front", "middle-back", "back"][i.min(3)]
}

fn lower<T: std::fmt::Debug>(v: T) -> String {
    format!("{v:?}").to_lowercase()
}

/// A change, in words.
fn say(line: &Line) -> String {
    match line {
        Line::Stance(s) => format!("stands {}", lower(s)),
        Line::Head(h) => format!("head {}", lower(h)),
        Line::Tail(t) => match t {
            sim::breed::Tail::None => "lost its tail".into(),
            t => format!("tail {}", lower(t)),
        },
        Line::Segments => "a new body segment".into(),
        Line::Limb(i, b, r) => format!("{} limbs: {} ({})", station(*i), lower(b), lower(r)),
        Line::NewLimb(i, b) => format!("grew {} limbs ({})", station(*i), lower(b)),
        Line::LostLimb(i) => format!("lost its {} limbs", station(*i)),
        Line::Tip(i, t) => format!("{} limbs end in {}", station(*i), lower(t)),
        Line::Jaw(j) => format!("jaw: {}", lower(j)),
        Line::Crown(c) => format!("crown: {}", lower(c)),
        Line::TailTip(t) => format!("tail tip: {}", lower(t)),
        Line::Hide(h) => format!("hide: {}", lower(h)),
        Line::Back => "a new back".into(),
        Line::Flies(true) => "it can fly!".into(),
        Line::Flies(false) => "it can no longer fly".into(),
        Line::Burrows(true) => "it can burrow!".into(),
        Line::Burrows(false) => "it can no longer burrow".into(),
        Line::Size(d) => {
            if d.raw() > 0 {
                "bigger".into()
            } else {
                "smaller".into()
            }
        }
    }
}

fn item_name(it: &Item) -> String {
    match it {
        Item::Egg(s) => format!("{} egg", species_name(*s)),
        Item::Trophy(k, s) => format!("{} {}", species_name(*s), k.name()),
    }
}

/// What the morph just became, kept for a few seconds after a feeding.
#[derive(Default)]
pub struct Told {
    last: Option<(u64, u16, u8)>,
    news: String,
    until: f32,
}

/// The lines: what the local seat carries, its morph, what a feeding just
/// did, and the warning that a creature wants its egg back.
pub fn update_text(
    sim: Res<crate::Sim>,
    time: Res<Time>,
    mut text: Query<&mut Text, With<KeepText>>,
    mut told: Local<Told>,
    mut prev: Local<Option<Morph>>,
    mut shown: Local<Option<u64>>,
) {
    let Ok(mut t) = text.single_mut() else {
        return;
    };
    let w = &sim.cur;
    if !w.valley.on {
        if shown.take().is_some() {
            *t = Text::new("");
        }
        return;
    }
    let me = sim.local_player().min(MAX_PLAYERS - 1);
    let m = w.valley.keep.morphs[me];
    let now_key = (m.genome.fingerprint(), m.generation, m.of);
    let clock = time.elapsed_secs();
    if told.last != Some(now_key) {
        if let (Some(before), Some(_)) = (*prev, told.last) {
            told.news = if m.of != before.of {
                format!(
                    "Your morph ate the {} egg: it is a young {} now.",
                    species_name(SpeciesId(m.of)),
                    species_name(SpeciesId(m.of))
                )
            } else {
                let f = morph::changes(&before.genome, &m.genome, None);
                let said: Vec<String> = f.iter().map(say).collect();
                if said.is_empty() {
                    format!("Generation {}: it grew.", m.generation)
                } else {
                    format!("Generation {}: {}.", m.generation, said.join(", "))
                }
            };
            told.until = clock + 6.0;
        }
        told.last = Some(now_key);
        *prev = Some(m);
    }
    // Rewritten only when what it says changes: what is carried, the
    // morph, the chase, and whether the news is still up.
    let carrier = w.egg_carrier();
    let mut key = now_key.0 ^ (now_key.1 as u64) << 48 ^ (now_key.2 as u64) << 40;
    for (k, it) in w.valley.keep.satchel[me].iter().enumerate() {
        let code = match it {
            None => 0u64,
            Some(Item::Egg(s)) => 1 + s.0 as u64,
            Some(Item::Trophy(t, s)) => 0x100 + (*t as u64) * 0x40 + s.0 as u64,
        };
        key = key.rotate_left(7) ^ code ^ (k as u64) << 20;
    }
    key ^= carrier.map_or(0, |(i, s)| 0x1000_0000 + i as u64 * 0x100 + s.0 as u64);
    key ^= (clock < told.until) as u64 * 0x8000_0000_0000;
    if *shown == Some(key) {
        return;
    }
    *shown = Some(key);
    let mut lines = Vec::new();
    if let Some((i, s)) = carrier {
        if i == me {
            lines.push(format!(
                "The {} wants its egg back. It will not stop until you beat it.",
                species_name(s)
            ));
        }
    }
    let bag: Vec<String> = w.valley.keep.satchel[me]
        .iter()
        .flatten()
        .map(item_name)
        .collect();
    if !bag.is_empty() {
        lines.push(format!("Carrying: {}", bag.join(", ")));
    }
    let what = if m.of == Morph::UNFORMED {
        "an unformed morph".to_string()
    } else {
        format!(
            "a {} morph, generation {}",
            species_name(SpeciesId(m.of)),
            m.generation
        )
    };
    let hint = if bag.is_empty() {
        "find an egg, beat its creature, collect trophies"
    } else {
        "crouch beside it to feed it"
    };
    lines.push(format!("Your morph: {what} -- {hint}"));
    if clock < told.until {
        lines.push(told.news.clone());
    }
    *t = Text::new(lines.join("\n"));
}
