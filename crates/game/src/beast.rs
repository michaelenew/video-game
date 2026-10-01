//! Drawing the creatures.
//!
//! Any species, from its own table: one box per part, placed from the same rig
//! and the same boxes the simulation collides against. What a species looks
//! like -- its paint, and the decorations only it has -- is declared in its own
//! file under `crate::species`; nothing here knows which animal it is drawing. Nothing here rebuilds the creature's geometry:
//! if you can see a box, that box is what stops you walking through it and what
//! your attacks are hitting.
//!
//! The parts are boxes and the creature is a skeleton, so consecutive segments
//! of a limb open a wedge between them whenever the joint bends. A **sphere at
//! each joint** closes it -- the classic capsule silhouette, built out of the
//! shapes that were already there. It is the one thing drawn that the
//! simulation does not collide against, and it is deliberately *inside* the
//! parts it joins, so it can never make the animal look bigger than it is.
//!
//! The weak points are a different colour from everything else on purpose.
//! They are the only places on the animal worth hitting, and a player should be
//! able to see that from across the arena without being told.
//!
//! Every creature slot in the world (`sim::monster::MAX_MONSTERS`) has its own
//! pool of parts, joints and floor markers, so the Pair are two animals drawn
//! the same way one is.

use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
    NotShadowCaster,
};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, CompareFunction, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use sim::monster::{Doing, MAX_MONSTERS, Monster};
use sim::species::{self as kinds, SpeciesId};

use crate::species::{Look, Paint};

/// One drawable part: which creature slot, which part. A fixed pool, spawned
/// once: the creature comes and goes with the match, and spawning meshes when
/// it does would put allocation on a path the rollback re-runs.
#[derive(Component)]
pub struct Limb(pub usize, pub usize);

/// One drawable joint filler: which creature slot, which bone.
#[derive(Component)]
pub struct Knuckle(pub usize, pub usize);

/// Materials, made once per species. Which one a part wears changes when it
/// breaks.
#[derive(Clone)]
struct Skin {
    armour: Handle<StandardMaterial>,
    weak: Handle<StandardMaterial>,
    limb: Handle<StandardMaterial>,
    broken: Handle<StandardMaterial>,
    /// The hide of every body after the first, in a fight of more than one:
    /// the same paint, darker, so two of a kind can be told apart and named.
    second: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct Hide {
    /// By species id; every registered species has one.
    skins: Vec<Option<Skin>>,
}

impl Hide {
    fn of(&self, id: SpeciesId) -> &Skin {
        self.skins[id.0 as usize]
            .as_ref()
            .or_else(|| self.skins.iter().flatten().next())
            .expect("at least one species is registered")
    }
}

fn material(materials: &mut Assets<StandardMaterial>, paint: Paint) -> Handle<StandardMaterial> {
    let [r, g, b] = paint.rgb;
    let [er, eg, eb] = paint.glow;
    materials.add(StandardMaterial {
        base_color: Color::srgb(r, g, b),
        emissive: LinearRgba::rgb(er, eg, eb),
        perceptual_roughness: paint.roughness,
        ..default()
    })
}

/// The most parts and bones any registered species has: how many of each a
/// slot's pool needs.
fn most() -> (usize, usize) {
    kinds::all().fold((0, 0), |(p, b), s| {
        (p.max(s.parts.len()), b.max(s.bones.len()))
    })
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut skins = vec![None; kinds::COUNT];
    for sp in kinds::all() {
        let look = crate::species::look(sp.id);
        skins[sp.id.0 as usize] = Some(Skin {
            armour: material(&mut materials, look.armour),
            weak: material(&mut materials, look.weak),
            limb: material(&mut materials, look.breakable),
            broken: material(&mut materials, look.broken),
            second: material(&mut materials, darker(look.armour)),
        });
    }
    let hide = Hide { skins };
    let any = hide.of(SpeciesId::RIDGEBACK).armour.clone();
    // A unit cube, scaled per part. The part boxes are axis-aligned in their
    // own bone's frame, so one mesh covers all of them.
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let ball = meshes.add(Sphere::new(0.5).mesh().ico(2).unwrap());
    let (parts, bones) = most();
    for slot in 0..MAX_MONSTERS {
        for index in 0..parts {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(any.clone()),
                Transform::default(),
                Visibility::Hidden,
                Limb(slot, index),
            ));
        }
        for bone in 0..bones {
            commands.spawn((
                Mesh3d(ball.clone()),
                MeshMaterial3d(any.clone()),
                Transform::default(),
                Visibility::Hidden,
                Knuckle(slot, bone),
            ));
        }
    }
    commands.insert_resource(hide);
}

pub fn setup_signs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut marks: ResMut<Assets<MarkMaterial>>,
) {
    make_signs(&mut commands, &mut meshes, &mut materials, &mut marks);
}

/// A hide two shades down, for the second of two bodies.
fn darker(paint: Paint) -> Paint {
    Paint {
        rgb: paint.rgb.map(|c| c * 0.45),
        ..paint
    }
}

/// Put every part where the simulation says it is.
pub fn place(
    sim: Res<crate::Sim>,
    hide: Res<Hide>,
    mut limbs: Query<
        (
            &Limb,
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        Without<Knuckle>,
    >,
    mut knuckles: Query<
        (
            &Knuckle,
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        Without<Limb>,
    >,
) {
    let herd = &sim.cur.monsters;

    for (limb, mut transform, mut visible, mut material) in limbs.iter_mut() {
        let (slot, index) = (limb.0, limb.1);
        let Some(beast) = herd[slot].filter(|b| index < b.sp().parts.len()) else {
            *visible = Visibility::Hidden;
            continue;
        };
        // **What the simulation says can be seen of it** (`World::shown`):
        // the Veilstalker's veil, owned by the snapshot because a decloak is a
        // tell. A part shown at nothing is not drawn; how a part shown at
        // some is drawn -- a shimmer, a dithered silhouette -- is the
        // creature's look to add. Every creature without a veil is shown
        // whole, as it always was.
        if sim.cur.shown(slot, index).raw() <= 0 {
            *visible = Visibility::Hidden;
            continue;
        }
        let rig = beast.rig();
        let shape = beast.sp().shape(index);
        let mid = shape.min.add(shape.max).scale(sim::Fx::ratio(1, 2));
        let size = shape.max.sub(shape.min);
        let frame = rig.of(index);

        // The bone's frame, read off the transform itself rather than
        // reconstructed from angles: step one unit along each axis and
        // subtract. There is no handedness convention here to get wrong.
        let at = |offset: sim::V3| fx3(frame.local_to_world(mid.add(offset)));
        let origin = at(sim::V3::ZERO);
        let x = at(sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO)) - origin;
        let y = at(sim::V3::new(sim::Fx::ZERO, sim::Fx::ONE, sim::Fx::ZERO)) - origin;

        *transform = Transform {
            translation: origin,
            rotation: orient(x, y),
            scale: fx3(size),
        };
        *visible = Visibility::Inherited;

        let wanted = skin(hide.of(beast.species), &beast, slot, index);
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }

    // The joints. Each is sized to the thinner of the parts meeting there, so
    // it disappears inside them and only shows in the wedge a bend opens up.
    for (knuckle, mut transform, mut visible, mut material) in knuckles.iter_mut() {
        let (slot, bone) = (knuckle.0, knuckle.1);
        let Some(beast) = herd[slot] else {
            *visible = Visibility::Hidden;
            continue;
        };
        let look = crate::species::look(beast.species);
        let Some(width) = joint_width(beast.sp(), look, bone) else {
            *visible = Visibility::Hidden;
            continue;
        };
        let rig = beast.rig();
        *transform = Transform {
            translation: fx3(rig.bone[bone].at),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(width),
        };
        *visible = Visibility::Inherited;
        let wanted = hide_of(hide.of(beast.species), slot);
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }
}

/// How wide a filler at this bone should be, or `None` where nothing meets.
///
/// The narrower of the two parts hanging off the joint: a sphere the size of
/// the *wider* one would bulge out of the slimmer segment and read as a bead on
/// a string rather than as an elbow.
fn joint_width(sp: &kinds::Species, look: &Look, bone: usize) -> Option<f32> {
    if bone >= sp.bones.len() || look.no_knuckles.contains(&bone) {
        return None;
    }
    let mut narrowest: Option<sim::Fx> = None;
    for (index, part) in sp.parts.iter().enumerate() {
        if part.shape.bone != bone || !part.shape.solid {
            continue;
        }
        let shape = sp.shape(index);
        let size = shape.max.sub(shape.min);
        let thin = size.y.min(size.z);
        if narrowest.is_none_or(|seen| thin.raw() < seen.raw()) {
            narrowest = Some(thin);
        }
    }
    narrowest.map(|w| w.to_f32_for_render() * 0.95)
}

/// The plain hide of the body in `slot`.
fn hide_of(hide: &Skin, slot: usize) -> &Handle<StandardMaterial> {
    if slot == 0 { &hide.armour } else { &hide.second }
}

fn skin<'a>(
    hide: &'a Skin,
    beast: &Monster,
    slot: usize,
    index: usize,
) -> &'a Handle<StandardMaterial> {
    let sp = beast.sp();
    if sp.is_weak_point(index) {
        &hide.weak
    } else if beast.broken(index) {
        &hide.broken
    } else if sp.parts[index].shape.breakable {
        &hide.limb
    } else {
        hide_of(hide, slot)
    }
}

/// A rotation from two nearly-orthogonal axes.
///
/// Re-orthogonalised rather than trusted: the axes come through the fixed-point
/// sine table and are a fraction of a degree off square, which a quaternion
/// built straight from them turns into a visible shear.
fn orient(x: Vec3, y: Vec3) -> Quat {
    let x = x.normalize_or_zero();
    let y = (y - x * x.dot(y)).normalize_or_zero();
    if x == Vec3::ZERO || y == Vec3::ZERO {
        return Quat::IDENTITY;
    }
    Quat::from_mat3(&Mat3::from_cols(x, y, x.cross(y)))
}

fn fx3(v: sim::V3) -> Vec3 {
    Vec3::new(
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    )
}

// ---------------------------------------------------------------------------
// The overlay
// ---------------------------------------------------------------------------

/// The creature's live attack volume, and where you can stand on it.
///
/// Both come from the simulation's own functions. An overlay that rebuilds the
/// shape it illustrates is worse than none: it is confidently wrong at exactly
/// the moment you are using it to work out why something missed.
pub fn overlay(show: Res<crate::debug::ShowDebug>, sim: Res<crate::Sim>, mut gizmos: Gizmos) {
    if !show.0 {
        return;
    }
    for beast in sim.cur.monsters.iter().flatten() {
        overlay_one(&mut gizmos, beast);
    }
}

fn overlay_one(gizmos: &mut Gizmos, beast: &Monster) {
    let rig = beast.rig();
    let sp = beast.sp();

    // Mountable tops, so it is obvious where the climb goes and where it dead
    // ends. Straight off the rig's own top-face corners.
    for index in 0..sp.parts.len() {
        if !sp.parts[index].shape.mountable {
            continue;
        }
        let quad = rig.top_face(index).map(fx3);
        for i in 0..4 {
            gizmos.line(quad[i], quad[(i + 1) % 4], MOUNTABLE);
        }
    }

    // The attack out this frame -- a vertical cylinder, which is what the hit
    // test compares against. Drawn as two rings and a spine, so the height band
    // is visible: unlike the fighters' own attacks, this one genuinely cares
    // how high you are standing.
    if let Some((anchor, radius, low, high)) = beast.hit_volume() {
        let spent = beast.hit_used;
        let colour = if spent { HIT_SPENT } else { HIT };
        let ring = |gizmos: &mut Gizmos, y: sim::Fx| {
            let centre = fx3(sim::V3::new(anchor.x, y, anchor.z));
            gizmos.circle(
                Isometry3d::new(centre, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                radius.to_f32_for_render(),
                colour,
            );
            centre
        };
        let bottom = ring(gizmos, low);
        let top = ring(gizmos, high);
        gizmos.line(bottom, top, colour);
    }

    // What it is doing, as a line off the nose: long for a committed move,
    // short while it is only looking.
    let head_part = crate::species::look(beast.species).head;
    let head = sp.shape(head_part);
    let nose = fx3(rig.part_to_world(
        head_part,
        sim::V3::new(head.max.x, head.max.y, sim::Fx::ZERO),
    ));
    let ahead = fx3(rig.dir_to_world(sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO)));
    let length = match beast.doing {
        Doing::Startup { .. } => 2.5,
        Doing::Active { .. } => 4.0,
        _ => 1.0,
    };
    gizmos.line(nose, nose + ahead * length, intent_colour(beast));
}

// ---------------------------------------------------------------------------
// Reading it: what is coming, drawn on the floor, and the spikes
// ---------------------------------------------------------------------------

/// One piece of the floor marker under a windup. See [`signs`].
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// The whole footprint of the hit: where it lands.
    Area,
    /// The far end of a lane, for a hit that travels.
    AreaEnd,
    /// The ground a travelling hit crosses.
    Lane,
    /// Growing out of the middle through the windup. Full is now.
    Fill,
    /// The same, down the lane.
    FillLane,
}

/// Everything [`signs`] moves on one floor marker.
type MarkParts = (
    &'static Mark,
    &'static Of,
    &'static mut Transform,
    &'static mut Visibility,
    &'static mut MeshMaterial3d<MarkMaterial>,
);

/// Which creature slot a marker or a spike belongs to -- or, past
/// `MAX_MONSTERS`, which critter slot a marker is drawn for.
#[derive(Component)]
pub struct Of(pub usize);

/// One of a volley's spikes (the Ridgeback's spray): bristling along the tail
/// through the windup, then flying as a volley. See `species::Spikes`.
#[derive(Component)]
pub struct Spike(pub usize);

/// **Drawn over the arena, not under it.** A marker lies on the floor, and a
/// raised platform between it and the camera hid it -- while the hit it marks
/// reaches a fighter standing on top of that platform just the same. So the
/// markers skip the depth test: they read as projected onto whatever you are
/// standing on. They are faint and see-through, so passing over a body or a
/// leg tints it rather than hiding it.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct OnTop {}

impl MaterialExtension for OnTop {
    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_compare = CompareFunction::Always;
            depth.depth_write_enabled = false;
        }
        Ok(())
    }
}

/// A marker's material: a flat tint that ignores depth.
pub type MarkMaterial = ExtendedMaterial<StandardMaterial, OnTop>;

/// Materials for the signs, made once and swapped between rather than edited.
#[derive(Resource)]
pub struct Signs {
    area: Handle<MarkMaterial>,
    fill: Handle<MarkMaterial>,
    live: Handle<MarkMaterial>,
}

fn make_signs(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    marks: &mut Assets<MarkMaterial>,
) {
    let mut tint = |c: Color| {
        marks.add(MarkMaterial {
            base: StandardMaterial {
                base_color: c,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            },
            extension: OnTop {},
        })
    };
    let signs = Signs {
        area: tint(Color::srgba(1.0, 0.72, 0.22, 0.16)),
        fill: tint(Color::srgba(1.0, 0.55, 0.12, 0.42)),
        live: tint(Color::srgba(1.0, 0.16, 0.18, 0.55)),
    };
    // Both shapes lie flat on the floor: built in the XY plane, turned down.
    let disc = meshes.add(Circle::new(1.0));
    let strip = meshes.add(Rectangle::new(1.0, 1.0));
    let spike = meshes.add(Cone::new(SPIKE_RADIUS, 1.0));
    // One volley's worth of paint: the first species that throws one. A second
    // species with spikes of another colour would keep a material per species,
    // the way the hide does.
    let paint = kinds::all()
        .find_map(|s| crate::species::look(s.id).spikes)
        .map(|s| s.paint);
    let bone = paint.map(|p| material(materials, p));
    // A set of markers per creature slot, then one per critter slot: a pack's
    // bites are drawn the same way a creature's are (`SIGN_SLOTS`).
    for slot in 0..SIGN_SLOTS {
        for (mark, mesh, material) in [
            (Mark::Area, &disc, &signs.area),
            (Mark::AreaEnd, &disc, &signs.area),
            (Mark::Lane, &strip, &signs.area),
            (Mark::Fill, &disc, &signs.fill),
            (Mark::FillLane, &strip, &signs.fill),
        ] {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::default(),
                Visibility::Hidden,
                NotShadowCaster,
                mark,
                Of(slot),
            ));
        }
        let Some(bone) = bone.as_ref().filter(|_| slot < MAX_MONSTERS) else {
            continue;
        };
        for i in 0..SPIKES {
            commands.spawn((
                Mesh3d(spike.clone()),
                MeshMaterial3d(bone.clone()),
                Transform::default(),
                Visibility::Hidden,
                Spike(i),
                Of(slot),
            ));
        }
    }
    commands.insert_resource(signs);
}

/// **What is coming, and where it will land**, always drawn.
///
/// Through a windup, the floor under the hit it is winding up: the whole
/// footprint faintly, and a fill growing out of the middle that reaches the
/// edge on the frame the hit comes out. A move that travels -- the charge, the
/// spray -- draws the lane it will cross as well. While the hit is out the
/// marker turns red, and it is gone once it has caught somebody, because the
/// simulation has stopped testing it.
///
/// Every shape comes from `Monster::telegraph`, which asks the hit test's own
/// volume where the hit will be; nothing here knows how big a bite is. That is
/// the overlay's rule applied to what a player sees: a picture of a hit that
/// is not the hit is worse than no picture. The hits are far bigger than the
/// parts that throw them -- the bite's is six metres across behind a head one
/// metre wide -- and before this nothing on the screen said so.
pub fn signs(
    sim: Res<crate::Sim>,
    signs: Res<Signs>,
    mut marks: Query<MarkParts, Without<Spike>>,
    mut spikes: Query<(&Spike, &Of, &mut Transform, &mut Visibility), Without<Mark>>,
) {
    let alive: [Option<Monster>; MAX_MONSTERS] =
        std::array::from_fn(|slot| sim.cur.monsters[slot].filter(|b| b.alive()));
    // The pack's bites after the creatures': **the same markers, from the same
    // kind of answer** -- `Critter::telegraph` asks the critter's own hit
    // volume where its bite will be, as `Monster::telegraph` does. Drawn over
    // everything, as every marker is (`OnTop`), which for a knee-high biter at
    // your heels is the point: the character's own body would hide it.
    let crowd = &sim.cur.critters;
    let coming: [Option<sim::monster::Telegraph>; SIGN_SLOTS] = std::array::from_fn(|slot| {
        if slot >= MAX_MONSTERS {
            let c = crowd.get(slot - MAX_MONSTERS)?;
            if sim.cur.pack.is_none() || !c.alive() {
                return None;
            }
            return c
                .telegraph(crowd.sp())
                .filter(|t| !(t.live && c.has(sim::critter::flag::HIT_USED)));
        }
        let beast = alive[slot];
        beast
            .and_then(|b| b.telegraph())
            .filter(|t| !(t.live && beast.is_some_and(|b| b.hit_used)))
    });

    for (mark, of, mut transform, mut visible, mut material) in marks.iter_mut() {
        let Some(t) = coming[of.0] else {
            *visible = Visibility::Hidden;
            continue;
        };
        let anchor = fx3(t.anchor);
        let at = Vec3::new(anchor.x, FLOOR, anchor.z);
        let along = fx3(t.along).with_y(0.0).normalize_or_zero();
        let r = t.radius.to_f32_for_render();
        let sweep = t.sweep.to_f32_for_render();
        let progress = t.progress.to_f32_for_render().clamp(0.0, 1.0);
        let flat = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        // The strip's long side along the lane.
        let lane_turn = Quat::from_rotation_y(-along.z.atan2(along.x)) * flat;
        let travels = sweep > 0.05;
        let (show, place) = match mark {
            Mark::Area => (
                true,
                Transform {
                    translation: at,
                    rotation: flat,
                    scale: Vec3::new(r, r, 1.0),
                },
            ),
            Mark::AreaEnd => (
                travels,
                Transform {
                    translation: at + along * sweep,
                    rotation: flat,
                    scale: Vec3::new(r, r, 1.0),
                },
            ),
            Mark::Lane => (
                travels,
                Transform {
                    translation: at + along * (sweep * 0.5),
                    rotation: lane_turn,
                    scale: Vec3::new(sweep, r * 2.0, 1.0),
                },
            ),
            Mark::Fill => (
                !t.live,
                Transform {
                    translation: at + Vec3::Y * LIFT,
                    rotation: flat,
                    scale: Vec3::new(r * progress, r * progress, 1.0),
                },
            ),
            Mark::FillLane => (
                !t.live && travels,
                Transform {
                    translation: at + along * (sweep * progress * 0.5) + Vec3::Y * LIFT,
                    rotation: lane_turn,
                    scale: Vec3::new(sweep * progress, r * 2.0, 1.0),
                },
            ),
        };
        if !show {
            *visible = Visibility::Hidden;
            continue;
        }
        *transform = place;
        *visible = Visibility::Inherited;
        let wanted = match (mark, t.live) {
            (_, true) => &signs.live,
            (Mark::Fill | Mark::FillLane, false) => &signs.fill,
            _ => &signs.area,
        };
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }

    // The spikes. Only a move a species' look says throws a volley has any:
    // the Ridgeback's spray.
    for (spike, of, mut transform, mut visible) in spikes.iter_mut() {
        let volley = alive
            .get(of.0)
            .copied()
            .flatten()
            .zip(coming[of.0])
            .and_then(|(beast, t)| {
                crate::species::look(beast.species)
                    .spikes
                    .filter(|s| s.kind == t.kind)
                    .map(|s| (beast, t, s.along))
            });
        let Some((beast, t, along)) = volley else {
            *visible = Visibility::Hidden;
            continue;
        };
        let i = spike.0;
        let u = i as f32 / (SPIKES - 1) as f32;
        if !t.live {
            // **Bristling along the tail through the windup**, growing as it
            // comes over the back -- the tell reads from in front, where the
            // person it is for is standing, as a row of spikes standing up.
            let progress = t.progress.to_f32_for_render().clamp(0.0, 1.0);
            let rig = beast.rig();
            // Half along the middle of the tail, half along its tip, in two
            // rows either side of the spine. Worked out in the simulation's
            // own numbers and only turned into floats to be drawn.
            let (part, k) = if i < SPIKES / 2 {
                (along[0], i)
            } else {
                (along[1], i - SPIKES / 2)
            };
            let shape = beast.sp().shape(part);
            let size = shape.max.sub(shape.min);
            let side = if i % 2 == 0 {
                sim::Fx::ONE
            } else {
                sim::Fx::ONE.neg()
            };
            let spread = sim::Fx::ratio(15 + 70 * k as i32 / (SPIKES as i32 / 2 - 1), 100);
            let base = sim::V3::new(
                shape.min.x.add(size.x.mul(spread)),
                shape.max.y,
                shape
                    .min
                    .z
                    .add(size.z.mul(sim::Fx::ratio(1, 2)))
                    .add(size.z.mul(sim::Fx::ratio(35, 100)).mul(side)),
            );
            let out = base.add(sim::V3::new(
                sim::Fx::ZERO,
                sim::Fx::ONE,
                side.mul(sim::Fx::ratio(35, 100)),
            ));
            let root = fx3(rig.part_to_world(part, base));
            let tip = fx3(rig.part_to_world(part, out));
            let dir = (tip - root).normalize_or(Vec3::Y);
            let length = BRISTLE * (0.25 + 0.75 * progress);
            *transform = Transform {
                translation: root + dir * (length * 0.5),
                rotation: Quat::from_rotation_arc(Vec3::Y, dir),
                scale: Vec3::new(1.0, length, 1.0),
            };
        } else {
            // **The volley**, spread across the width of the hit and up its
            // height, staggered along the flight so it reads as a flight of
            // spikes rather than a wall, each pointing the way it is going.
            let along = fx3(t.along).with_y(0.0).normalize_or_zero();
            let across = Vec3::new(-along.z, 0.0, along.x);
            let centre = fx3(t.anchor);
            let r = t.radius.to_f32_for_render();
            let (lo, hi) = (t.low.to_f32_for_render(), t.high.to_f32_for_render());
            let side = (u * 2.0 - 1.0) * r * 0.9;
            let up = lo + (hi - lo) * (0.2 + 0.6 * ((i * 7 % SPIKES) as f32 / SPIKES as f32));
            let back = ((i * 3 % 5) as f32) * 0.35;
            let point = Vec3::new(centre.x, up, centre.z) + across * side - along * back;
            *transform = Transform {
                translation: point - along * (FLIGHT * 0.5),
                rotation: Quat::from_rotation_arc(Vec3::Y, along),
                scale: Vec3::new(1.4, FLIGHT, 1.4),
            };
        }
        *visible = Visibility::Inherited;
    }
}

/// How many sets of floor markers there are: one per creature slot, then one
/// per critter slot.
const SIGN_SLOTS: usize = MAX_MONSTERS + sim::critter::MAX_CRITTERS;

/// Just off the floor, so the marker is not fighting it for the same depth.
const FLOOR: f32 = 0.03;
/// The fill sits a hair above the footprint for the same reason.
const LIFT: f32 = 0.01;
const SPIKES: usize = 12;
const SPIKE_RADIUS: f32 = 0.13;
/// A spike standing on the tail at the top of the windup, and one in flight.
const BRISTLE: f32 = 1.1;
const FLIGHT: f32 = 1.5;

const MOUNTABLE: Color = Color::srgb(0.38, 0.85, 0.62);
const HIT: Color = Color::srgb(1.0, 0.23, 0.31);
const HIT_SPENT: Color = Color::srgb(0.55, 0.16, 0.20);
const WINDUP: Color = Color::srgb(1.0, 0.78, 0.25);
const DOWN: Color = Color::srgb(0.45, 0.62, 1.0);
const CALM: Color = Color::srgb(0.62, 0.66, 0.72);

fn intent_colour(beast: &Monster) -> Color {
    match beast.doing {
        Doing::Startup { .. } => WINDUP,
        Doing::Active { .. } => HIT,
        Doing::Toppled { .. } | Doing::Flinch { .. } | Doing::Stumble { .. } => DOWN,
        _ => CALM,
    }
}
