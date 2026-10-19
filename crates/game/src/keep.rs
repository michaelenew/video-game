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
//!
//! **An unformed morph is a blob**: pink, glossy, floating at shoulder
//! height with two big eyes, a mouth and nothing else -- it is not
//! anything yet. Fed an egg, it melts into that creature's young: the blob
//! shrinks away as the body grows in where it hung. The body is still
//! there underneath, walking (the snapshot's heel), so it melts into a
//! creature already in step.
//!
//! **Moving, the blob leaves a trail of puffs** off its tail, like a
//! cartoon's propulsion: each a quarter of its size, a little lumpy, left
//! where it was made and shrinking to nothing in a second. A creature
//! leaves none -- once fed, it moves as that creature does.
//!
//! **With `--dev`**, a second window beside the bag lists every item in
//! the game under a search box; "Add" puts one in the bag (`Travel::grant`).
//!
//! **The bag** (B) is an egui window of what you carry: click an item, then
//! "Feed to morph". Feeding is a trip on the wire (`Travel::feed`), so it
//! lands on the same frame on both machines.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use sim::breed::build::{self, Body};
use sim::breed::morph::{self, Line, Morph};
use sim::breed::{locomotion, posture};
use sim::keep::{Item, LOOSE, Refusal};
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

/// One piece of a seat's blob (`BLOB` names them).
#[derive(Component)]
pub struct BlobPiece(pub usize, pub usize);

/// The blob's pieces: where each sits on the body, in the body's frame
/// (`+z` forward, a unit body), how big it is, and what it is drawn in --
/// 0 the body's pink, 1 white, 2 black, 3 the mouth, 4 a blush.
///
/// **Cute, not a blobfish** (2026-10-19). What read as a blobfish was a
/// squat body tipped nose-down, its face slid to the bottom. So it is a
/// little taller than wide and upright, its face high on it: big eyes
/// close together, big pupils each with a highlight, a small smile, pink
/// cheeks, short nubs for arms, and a tail tapering behind it that the
/// puffs come off.
const BLOB: [([f32; 3], [f32; 3], usize); 15] = [
    ([0.0, 0.0, 0.0], [0.92, 1.0, 0.9], 0),
    // The tail, tapering back and a little down.
    ([0.0, -0.1, -0.4], [0.52, 0.48, 0.56], 0),
    ([0.0, -0.18, -0.66], [0.28, 0.26, 0.34], 0),
    // Eyes, pupils, and a highlight in each.
    ([-0.17, 0.1, 0.36], [0.3, 0.36, 0.2], 1),
    ([0.17, 0.1, 0.36], [0.3, 0.36, 0.2], 1),
    ([-0.16, 0.08, 0.44], [0.2, 0.25, 0.1], 2),
    ([0.16, 0.08, 0.44], [0.2, 0.25, 0.1], 2),
    ([-0.2, 0.14, 0.5], [0.07, 0.08, 0.04], 1),
    ([0.12, 0.14, 0.5], [0.07, 0.08, 0.04], 1),
    // A small smile.
    ([0.0, -0.12, 0.45], [0.1, 0.05, 0.04], 3),
    // Cheeks.
    ([-0.31, -0.06, 0.33], [0.13, 0.08, 0.06], 4),
    ([0.31, -0.06, 0.33], [0.13, 0.08, 0.06], 4),
    // Nubs.
    ([-0.44, -0.04, 0.02], [0.18, 0.16, 0.18], 0),
    ([0.44, -0.04, 0.02], [0.18, 0.16, 0.18], 0),
    // Where the puffs leave from: the tail's tip, never drawn.
    ([0.0, -0.22, -0.84], [0.0, 0.0, 0.0], 0),
];

/// The tail's tip, in `BLOB`.
const TAIL_TIP: usize = 14;

/// The puffs a moving blob leaves (2026-10-19, the owner's spec): off its
/// tail, a quarter of its size, a little irregular, fixed where they were
/// left, shrinking evenly to nothing in a second. The renderer's own: a
/// puff decides nothing.
const PUFF_LIFE: f32 = 1.0;
const PUFF_SHARE: f32 = 0.25;
/// One every this many seconds while it moves faster than `PUFF_SPEED`
/// (m/s); a jump further than `PUFF_JUMP` (m) in a frame is a catch-up,
/// not a flight, and leaves nothing between.
const PUFF_EVERY: f32 = 0.06;
const PUFF_SPEED: f32 = 0.4;
const PUFF_JUMP: f32 = 3.0;
/// As many as a second of them, a seat.
const PUFFS: usize = 20;
/// Differently lumpy puff meshes.
const PUFF_SHAPES: usize = 4;

#[derive(Clone, Copy)]
struct Puff {
    at: Vec3,
    born: f32,
    /// Its size at birth, a scale a side, and a turn: the irregularity.
    scale: Vec3,
    turn: Quat,
}

/// The puffs, a seat's ring of them, and where each tail was last frame.
#[derive(Resource, Default)]
pub struct Puffs {
    ring: [[Option<Puff>; PUFFS]; MAX_PLAYERS],
    next: [usize; MAX_PLAYERS],
    last: [Option<(Vec3, f32)>; MAX_PLAYERS],
    left_at: [f32; MAX_PLAYERS],
    origin: Option<Vec3>,
    dice: u32,
}

impl Puffs {
    /// Nought to one, cheaply: the irregularity needs no more.
    fn roll(&mut self) -> f32 {
        self.dice = self
            .dice
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.dice >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// One puff on screen: a seat's, a slot in its ring.
#[derive(Component)]
pub struct PuffPiece(pub usize, pub usize);

/// A ball pushed in and out by a few bumps: a puff, `k` of them.
fn lumpy(k: usize) -> Mesh {
    let mut mesh = Sphere::new(0.5).mesh().ico(2).unwrap();
    let f = k as f32 * 1.91 + 0.37;
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(ps)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for p in ps.iter_mut() {
            let v = Vec3::from(*p);
            let d = v.normalize_or_zero();
            let bump =
                (d.x * 3.1 + f).sin() * (d.y * 2.7 + f * 1.3).sin() + 0.6 * (d.z * 4.3 - f).sin();
            *p = (v * (1.0 + 0.14 * bump)).to_array();
        }
    }
    mesh.compute_smooth_normals();
    mesh
}

/// How wide the blob is (m), and how high its middle floats.
const BLOB_SIZE: f32 = 0.85;
const BLOB_FLOAT: f32 = 1.5;

/// How long it takes to melt from a blob into a creature (s).
const MELT: f32 = 1.2;

/// How far along each seat's melt is: 1 a blob, 0 the creature. The
/// renderer's own, since it is how a change is shown, not what it is.
#[derive(Resource, Default)]
pub struct Melt {
    blob: [f32; MAX_PLAYERS],
    /// What each seat was of last frame, and when it last changed.
    of: [Option<u8>; MAX_PLAYERS],
    since: [f32; MAX_PLAYERS],
}

/// The bag: whether it is open and which item is picked.
#[derive(Resource, Default)]
pub struct Bag {
    pub open: bool,
    chosen: Option<usize>,
    /// The `--dev` item panel's search.
    search: String,
    /// A feeding chosen while another trip was on the wire, sent next.
    pending: Option<u8>,
}

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
    /// The blob's pink, its eyes' white and black, its mouth, its blush.
    blob: [Handle<StandardMaterial>; 5],
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
        blob: [
            materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.62, 0.82),
                emissive: LinearRgba::rgb(0.2, 0.07, 0.13),
                perceptual_roughness: 0.12,
                reflectance: 0.8,
                clearcoat: 1.0,
                clearcoat_perceptual_roughness: 0.05,
                ..default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.98, 0.98, 0.98),
                perceptual_roughness: 0.15,
                ..default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.02, 0.02, 0.03),
                perceptual_roughness: 0.1,
                ..default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.42, 0.06, 0.16),
                perceptual_roughness: 0.6,
                ..default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.36, 0.58),
                emissive: LinearRgba::rgb(0.25, 0.04, 0.1),
                perceptual_roughness: 0.5,
                ..default()
            }),
        ],
    };
    let puff: Vec<Handle<Mesh>> = (0..PUFF_SHAPES).map(|k| meshes.add(lumpy(k))).collect();
    for seat in 0..MAX_PLAYERS {
        for i in 0..PUFFS {
            commands.spawn((
                Mesh3d(puff[i % PUFF_SHAPES].clone()),
                MeshMaterial3d(kit.blob[0].clone()),
                Transform::default(),
                Visibility::Hidden,
                bevy::pbr::NotShadowCaster,
                PuffPiece(seat, i),
            ));
        }
    }
    for seat in 0..MAX_PLAYERS {
        for (piece, (_, _, m)) in BLOB.iter().enumerate().take(TAIL_TIP) {
            commands.spawn((
                Mesh3d(kit.egg.clone()),
                MeshMaterial3d(kit.blob[*m].clone()),
                Transform::default(),
                Visibility::Hidden,
                BlobPiece(seat, piece),
            ));
        }
    }
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

/// Each seat's morph posed: a body built once per frame per seat, grown
/// in as its blob melts away.
#[allow(clippy::type_complexity)]
pub fn place_morphs(
    sim: Res<crate::Sim>,
    kit: Res<Kit>,
    time: Res<Time>,
    mut melt: ResMut<Melt>,
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
    let clock = time.elapsed_secs();
    for seat in 0..MAX_PLAYERS {
        let of = w.valley.keep.morphs[seat].of;
        // A change of what it is of -- an egg eaten -- melts it again; the
        // first sight of a morph does not.
        match melt.of[seat] {
            Some(was) if was != of => melt.since[seat] = clock,
            None => melt.since[seat] = clock - MELT,
            _ => {}
        }
        melt.of[seat] = Some(of);
        melt.blob[seat] = if of == Morph::UNFORMED {
            1.0
        } else {
            let t = ((clock - melt.since[seat]) / MELT).clamp(0.0, 1.0);
            // Eased: it holds as a blob a moment, then goes.
            1.0 - t * t * (3.0 - 2.0 * t)
        };
    }
    if w.valley.on {
        for seat in 0..seats {
            let m: &Morph = &w.valley.keep.morphs[seat];
            let heel = w.heel[seat];
            if !heel.placed || melt.blob[seat] >= 1.0 {
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
        // Grown in from where the blob hung as it melts.
        let grown = 1.0 - melt.blob[seat];
        let hung = fx3(heel.at) + Vec3::Y * BLOB_FLOAT;
        let at = fx3(heel.at) + turn * fx3(body.centre(i));
        *transform = Transform {
            translation: hung.lerp(at, grown),
            rotation: turn * orient(fx3(u), fx3(v)),
            scale: fx3(shape.max.sub(shape.min)).max(Vec3::splat(0.02)) * grown,
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

/// Each seat's blob: floating, bobbing, wobbling, turned the way its heel
/// faces, and shrinking away as it melts into a creature; and, while it is
/// a blob and moving, the puffs off its tail.
#[allow(clippy::type_complexity)]
pub fn place_blobs(
    sim: Res<crate::Sim>,
    time: Res<Time>,
    melt: Res<Melt>,
    mut puffs: ResMut<Puffs>,
    mut pieces: Query<(&BlobPiece, &mut Transform, &mut Visibility), Without<PuffPiece>>,
    mut drawn: Query<(&PuffPiece, &mut Transform, &mut Visibility), Without<BlobPiece>>,
) {
    let w = &sim.cur;
    let seats = (w.seats as usize).clamp(1, MAX_PLAYERS);
    let t = time.elapsed_secs();
    // The world moved into the next place's coordinates: the puffs, which
    // stay where they were left, move with it.
    let o = fx3(w.map_origin());
    if let Some(last) = puffs.origin.replace(o)
        && last != o
    {
        let by = last - o;
        for p in puffs.ring.iter_mut().flatten().flatten() {
            p.at += by;
        }
        for l in puffs.last.iter_mut().flatten() {
            l.0 += by;
        }
    }
    // Where each blob is this frame, worked out once a seat.
    let mut frames: [Option<(Vec3, Quat, Vec3, f32)>; MAX_PLAYERS] = [None; MAX_PLAYERS];
    for (seat, frame) in frames.iter_mut().enumerate().take(seats) {
        let heel = w.heel[seat];
        let left = melt.blob[seat];
        if !w.valley.on || !heel.placed || left <= 0.0 {
            continue;
        }
        let phase = t + seat as f32 * 1.7;
        // A slow bob, a breathing squash and stretch, a little sway, and,
        // going, a tilt back as if it swam -- never nose-down, which was
        // the blobfish.
        let bob = (phase * 1.9).sin() * 0.14;
        let squash = (phase * 3.1).sin() * 0.06;
        let sway = (phase * 2.3).cos() * 0.04;
        let size = BLOB_SIZE * left;
        let body = Vec3::new(1.0 + squash, 1.0 - squash, 1.0 + squash * 0.5) * size;
        let yaw = heel.facing.to_f32_for_render() * std::f32::consts::TAU;
        let tilt = if heel.going { -0.12 } else { 0.0 };
        let turn = Quat::from_rotation_y(-yaw) * Quat::from_rotation_x(tilt + sway);
        let centre = fx3(heel.at) + Vec3::Y * (BLOB_FLOAT + bob);
        *frame = Some((centre, turn, body, size));
    }
    for (piece, mut transform, mut visible) in pieces.iter_mut() {
        let (seat, k) = (piece.0, piece.1);
        let Some((centre, turn, body, size)) = frames[seat] else {
            *visible = Visibility::Hidden;
            continue;
        };
        let phase = t + seat as f32 * 1.7;
        let (at, scale, _) = BLOB[k];
        let mut at = Vec3::from(at) * body;
        let mut scale = Vec3::from(scale) * size;
        if k == 12 || k == 13 {
            // The nubs either side wave, out of step with each other.
            let wave = (phase * 4.0 + k as f32 * 2.1).sin();
            at.y += wave * 0.06 * size;
            scale *= 1.0 + wave * 0.15;
        }
        if (3..=8).contains(&k) {
            // A blink every few seconds.
            let blink = ((phase * 0.45).fract() > 0.97) as u8 as f32;
            scale.y *= 1.0 - blink * 0.85;
        }
        if k == 1 || k == 2 {
            // The tail swishes.
            at.x += (phase * 3.4 - k as f32).sin() * 0.05 * k as f32 * size;
        }
        *transform = Transform {
            translation: centre + turn * at,
            rotation: turn,
            scale: scale.max(Vec3::splat(0.001)),
        };
        *visible = Visibility::Inherited;
    }
    // New puffs: only off a blob that is all blob -- once it has eaten an
    // egg it moves as its creature does, and a creature leaves none.
    for (seat, frame) in frames.iter().enumerate() {
        let frame = frame.filter(|_| melt.blob[seat] >= 1.0);
        let Some((centre, turn, body, size)) = frame else {
            puffs.last[seat] = None;
            continue;
        };
        let tip = centre + turn * (Vec3::from(BLOB[TAIL_TIP].0) * body);
        let moving = puffs.last[seat].is_some_and(|(was, then)| {
            let d = tip.distance(was);
            d < PUFF_JUMP && d > PUFF_SPEED * (t - then).max(1e-3)
        });
        puffs.last[seat] = Some((tip, t));
        if !moving || t - puffs.left_at[seat] < PUFF_EVERY {
            continue;
        }
        puffs.left_at[seat] = t;
        let jitter = Vec3::new(puffs.roll(), puffs.roll(), puffs.roll()) - Vec3::splat(0.5);
        let lump = Vec3::new(puffs.roll(), puffs.roll(), puffs.roll()) * 0.4 + Vec3::splat(0.8);
        let spin = Quat::from_euler(
            EulerRot::YXZ,
            puffs.roll() * std::f32::consts::TAU,
            puffs.roll() * std::f32::consts::TAU,
            0.0,
        );
        let i = puffs.next[seat];
        puffs.next[seat] = (i + 1) % PUFFS;
        puffs.ring[seat][i] = Some(Puff {
            at: tip + jitter * 0.1 * size,
            born: t,
            scale: lump * PUFF_SHARE * size,
            turn: spin,
        });
    }
    for (piece, mut transform, mut visible) in drawn.iter_mut() {
        let (seat, i) = (piece.0, piece.1);
        let live = puffs.ring[seat][i].filter(|p| w.valley.on && t - p.born < PUFF_LIFE);
        let Some(p) = live else {
            puffs.ring[seat][i] = None;
            *visible = Visibility::Hidden;
            continue;
        };
        // Evenly to nothing over its life.
        let share = 1.0 - (t - p.born) / PUFF_LIFE;
        *transform = Transform {
            translation: p.at,
            rotation: p.turn,
            scale: (p.scale * share).max(Vec3::splat(0.001)),
        };
        *visible = Visibility::Inherited;
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
        "B: open your bag to feed it"
    };
    lines.push(format!("Your morph: {what} -- {hint}"));
    if clock < told.until {
        lines.push(told.news.clone());
    }
    *t = Text::new(lines.join("\n"));
}

/// B opens and shuts the bag, in the valley. Opening it frees the cursor,
/// as the Oven does, so the first click lands on an item.
pub fn toggle_bag(
    keys: Res<ButtonInput<KeyCode>>,
    sim: Res<crate::Sim>,
    mut bag: ResMut<Bag>,
    mut focus: ResMut<crate::palette::UiFocus>,
) {
    if !sim.cur.valley.on {
        bag.open = false;
        return;
    }
    if keys.just_pressed(KeyCode::KeyB) && !focus.keyboard {
        bag.open = !bag.open;
        bag.chosen = None;
        if bag.open {
            focus.just_opened = true;
        }
    }
}

/// Why an item cannot be fed now, in words.
fn refusal(r: Refusal) -> String {
    match r {
        Refusal::Empty => "Nothing there.".into(),
        Refusal::Down => "You are down.".into(),
        Refusal::Eating => "It is still eating.".into(),
        Refusal::Chased(s) => format!("The {} wants it back -- beat it first.", species_name(s)),
    }
}

/// The bag's window: what the local seat carries, one button an item;
/// click one, then feed it to the morph.
pub fn draw_bag(mut contexts: EguiContexts, mut bag: ResMut<Bag>, mut sim: ResMut<crate::Sim>) {
    // A feeding waiting on another trip goes as soon as the wire is free.
    if let Some(k) = bag.pending
        && sim.travel == sim::input::Travel::NONE
    {
        sim.travel = sim::input::Travel::feed(k);
        bag.pending = None;
    }
    if !bag.open {
        return;
    }
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    let me = sim.local_player().min(MAX_PLAYERS - 1);
    let w = &sim.cur;
    let m = w.valley.keep.morphs[me];
    let satchel = w.valley.keep.satchel[me];
    let mut open = true;
    let mut feed = None;
    egui::Window::new("Bag")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.set_min_width(300.0);
            let what = if m.of == Morph::UNFORMED {
                "Your morph has not taken a shape yet. Feed it an egg.".to_string()
            } else {
                format!(
                    "Your morph: a {}, generation {}.",
                    species_name(SpeciesId(m.of)),
                    m.generation
                )
            };
            ui.label(what);
            ui.separator();
            if satchel.iter().all(|s| s.is_none()) {
                ui.label("Empty. Eggs lie by their creatures; trophies drop when you beat one.");
            }
            for (k, it) in satchel.iter().enumerate() {
                let Some(it) = it else {
                    continue;
                };
                let picked = bag.chosen == Some(k);
                if ui.selectable_label(picked, item_name(it)).clicked() {
                    bag.chosen = if picked { None } else { Some(k) };
                }
                if picked {
                    ui.indent(k, |ui| {
                        let why = match it {
                            Item::Egg(_) => {
                                "An egg starts your morph over as its creature's young."
                            }
                            Item::Trophy(..) => "A trophy grows your morph a generation.",
                        };
                        ui.label(why);
                        match w.can_feed(me, k) {
                            Ok(_) => {
                                ui.horizontal(|ui| {
                                    if ui.button("Feed to morph").clicked() {
                                        feed = Some(k);
                                    }
                                    if ui.button("Cancel").clicked() {
                                        bag.chosen = None;
                                    }
                                });
                            }
                            Err(r) => {
                                ui.add_enabled(false, egui::Button::new("Feed to morph"));
                                ui.label(refusal(r));
                            }
                        }
                    });
                }
            }
            ui.separator();
            ui.weak("B or Esc to close.");
        });
    if crate::dev_mode()
        && let Some(item) = draw_items(ctx, &mut bag.search, satchel.iter().all(|s| s.is_some()))
        && sim.travel == sim::input::Travel::NONE
    {
        sim.travel = sim::input::Travel::grant(item);
    }
    if let Some(k) = feed {
        bag.chosen = None;
        if sim.travel == sim::input::Travel::NONE {
            sim.travel = sim::input::Travel::feed(k as u8);
        } else {
            bag.pending = Some(k as u8);
        }
    }
    if !open {
        bag.open = false;
    }
}

/// Every item there is: each creature's egg and the trophies it drops.
fn every_item() -> impl Iterator<Item = Item> {
    (0..sim::species::COUNT).flat_map(|id| {
        let s = SpeciesId(id as u8);
        let drops = sim::breed::archetypes::of_species(s)
            .map(sim::breed::archetypes::drops)
            .unwrap_or_default();
        std::iter::once(Item::Egg(s))
            .chain(drops.into_iter().flatten().map(move |k| Item::Trophy(k, s)))
    })
}

/// `--dev`: beside the bag, every item in the game under a search box;
/// "Add" puts one in the bag. Which one was asked for, if any.
fn draw_items(ctx: &egui::Context, search: &mut String, full: bool) -> Option<Item> {
    let mut asked = None;
    egui::Window::new("Items (dev)")
        .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(-18.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_min_width(260.0);
            ui.add(egui::TextEdit::singleline(search).hint_text("Search items"));
            if full {
                ui.weak("The bag is full: feed or drop something first.");
            }
            let wanted = search.to_lowercase();
            egui::ScrollArea::vertical()
                .max_height(420.0)
                .show(ui, |ui| {
                    for item in every_item() {
                        let name = item_name(&item);
                        if !name.to_lowercase().contains(&wanted) {
                            continue;
                        }
                        ui.horizontal(|ui| {
                            if ui.add_enabled(!full, egui::Button::new("Add")).clicked() {
                                asked = Some(item);
                            }
                            ui.label(name);
                        });
                    }
                });
        });
    asked
}
