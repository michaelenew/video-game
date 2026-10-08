//! Drawing the small bodies (`sim::critter`), any species, from its table.
//!
//! Nothing here knows which animal it is drawing. A critter is drawn as a few
//! boxes -- a body, a head, four legs, a tail and its eyes -- sized from its
//! kind's own knobs, so **the body you see is the box the hit test uses**: its
//! length, width and height are the same three numbers `Critter::body` reads.
//! A species adds only its paint, in its `Look` (`crate::species`).
//!
//! **Posed from the snapshot, not from the renderer's clock.** The gait, the
//! crouch of a windup, the lunge, the flinch and the fall are all read off the
//! critter's own state, timer and animation clock, which are in the world and
//! roll back with it -- so a rollback re-poses a body exactly rather than
//! popping it. Positions and headings are interpolated between the last two
//! snapshots, the way the fighters' are.
//!
//! The debug overlay (F1) draws each body's hit box from `Critter::body` and
//! each live bite from `Critter::hit_volume`: the shapes the hit test reads,
//! not a second description of them. And the pack's ring, its den, and who is
//! holding a token.

use bevy::prelude::*;
use sim::critter::{Critter, CritterField, MAX_CRITTERS, flag, is, stat_fx};
use sim::fixed::Fx;
use sim::species::{self as kinds, Species};

use crate::species::Paint;

/// Which piece of a critter a mesh is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
    Body,
    Head,
    Leg(u8),
    Tail,
    Eyes,
    /// A horn, left or right: only for a kind whose paint has horns, and only
    /// while its species says that horn is whole.
    Horn(u8),
}

const PIECES: [Piece; 10] = [
    Piece::Body,
    Piece::Head,
    Piece::Leg(0),
    Piece::Leg(1),
    Piece::Leg(2),
    Piece::Leg(3),
    Piece::Tail,
    Piece::Eyes,
    Piece::Horn(0),
    Piece::Horn(1),
];

/// One drawable piece: which critter slot, which piece. A fixed pool spawned
/// once, so nothing is spawned while a fight is running.
#[derive(Component)]
pub struct Bit(pub usize, pub Piece);

/// Materials, made once per species and kind: hide, and eyes.
#[derive(Resource)]
pub struct Coats {
    /// By species id, then by kind: `(hide, eyes)`.
    by: Vec<Vec<(Handle<StandardMaterial>, Handle<StandardMaterial>)>>,
    /// By species id, then by kind: its horns, if it has any.
    horns: Vec<Vec<Option<Handle<StandardMaterial>>>>,
    fallback: (Handle<StandardMaterial>, Handle<StandardMaterial>),
}

impl Coats {
    fn of(
        &self,
        species: kinds::SpeciesId,
        kind: u8,
    ) -> &(Handle<StandardMaterial>, Handle<StandardMaterial>) {
        self.by
            .get(species.0 as usize)
            .and_then(|kinds| kinds.get(kind as usize))
            .unwrap_or(&self.fallback)
    }

    fn horn(&self, species: kinds::SpeciesId, kind: u8) -> Option<&Handle<StandardMaterial>> {
        self.horns
            .get(species.0 as usize)
            .and_then(|kinds| kinds.get(kind as usize))
            .and_then(Option::as_ref)
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

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut by = vec![Vec::new(); kinds::COUNT];
    let mut horns = vec![Vec::new(); kinds::COUNT];
    for sp in kinds::all() {
        let look = crate::species::look(sp.id);
        horns[sp.id.0 as usize] = look
            .critters
            .iter()
            .map(|c| c.horns.map(|paint| material(&mut materials, paint)))
            .collect();
        by[sp.id.0 as usize] = look
            .critters
            .iter()
            .map(|c| {
                (
                    material(&mut materials, c.body),
                    material(&mut materials, c.mark),
                )
            })
            .collect();
    }
    let fallback = (
        material(&mut materials, crate::species::NO_BODY),
        material(&mut materials, crate::species::NO_BODY),
    );
    // A small body is rounder than a big one: a rounded box, not a crate
    // (`docs/design/forms.md`); its box is still what the hit test uses.
    let cube = meshes.add(crate::shapes::soft_box(Vec3::ONE, 0.45, 8, None));
    for slot in 0..MAX_CRITTERS {
        for piece in PIECES {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(fallback.0.clone()),
                Transform::default(),
                Visibility::Hidden,
                Bit(slot, piece),
            ));
        }
    }
    commands.insert_resource(Coats {
        by,
        horns,
        fallback,
    });
}

/// A critter between the last two snapshots: where it is, which way it faces.
#[derive(Clone, Copy)]
struct Shown {
    at: Vec3,
    yaw: f32,
}

/// Interpolated, the way a fighter is: a body that teleported (a spawn into a
/// slot, a respawn) snaps rather than sliding across the arena.
fn shown(prev: &Critter, cur: &Critter, alpha: f32) -> Shown {
    let to = fx3(cur.pos);
    let yaw_of = |c: &Critter| c.yaw as f32 / 65536.0 * std::f32::consts::TAU;
    if !prev.present() || prev.kind != cur.kind || fx3(prev.pos).distance(to) > 3.0 {
        return Shown {
            at: to,
            yaw: yaw_of(cur),
        };
    }
    let from = fx3(prev.pos);
    let mut turn = yaw_of(cur) - yaw_of(prev);
    turn = (turn + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    Shown {
        at: from.lerp(to, alpha),
        yaw: yaw_of(prev) + turn * alpha,
    }
}

/// How the body is held this frame, from its state and clock. Pitch is nose
/// up, roll is onto its side, both in radians; `drop` lowers it.
struct Held {
    pitch: f32,
    roll: f32,
    drop: f32,
    /// How far through a stride the legs are, in turns.
    stride: f32,
    /// How far the tail is raised: up for a token, down otherwise.
    tail: f32,
    /// How much of the body is left: fading out once dead.
    left: f32,
    /// Off the floor: a leap's arc.
    lift: f32,
    /// The head turned sideways, in radians: the maul, the hook's cock.
    head_turn: f32,
    /// The head dropped, in radians: the charge's lock, the guard; negative
    /// is thrown back -- the bellow.
    head_dip: f32,
}

fn held(sp: &Species, c: &Critter, stance: Option<crate::species::Stance>) -> Held {
    use sim::critter::pose;
    let clock = c.clock as f32;
    let speed = Vec2::new(c.vel.x.to_f32_for_render(), c.vel.z.to_f32_for_render()).length();
    let length = stat_fx(sp, c.kind, CritterField::Length).to_f32_for_render();
    let mut h = Held {
        pitch: 0.0,
        roll: 0.0,
        drop: 0.0,
        // A stride a body length long: ground covered, roughly, not time.
        stride: clock * speed / 60.0 / length.max(0.1),
        tail: if c.has(flag::TOKEN) { 1.1 } else { -0.35 },
        left: 1.0,
        lift: 0.0,
        head_turn: 0.0,
        head_dip: 0.0,
    };
    // **The move's stock pose** (`sim::critter::pose`), named by its clip
    // number: a critter has no clips of its own. How far through the windup
    // it is, nought to one, eases each pose in.
    let a = sp.attack(c.act);
    let pose = sp
        .moves
        .get(c.act as usize)
        .map_or(pose::CROUCH, |m| m.clip);
    let through = match c.state {
        is::STARTUP => 1.0 - c.timer as f32 / a.startup.max(1) as f32,
        _ => 1.0,
    };
    let wiggle = (clock * 0.9).sin();
    match (c.state, pose) {
        // The crouch: rump up, head down, and a wiggle.
        (is::STARTUP, pose::CROUCH) => {
            h.pitch = -0.28 * (0.4 + 0.6 * through);
            h.roll = wiggle * 0.12;
            h.stride = 0.0;
        }
        // The scuttle: flat and fast, the legs going.
        (is::STARTUP, pose::SCUTTLE) => {
            h.pitch = -0.08;
            h.drop = 0.18 * length;
            h.stride = clock * 0.35;
        }
        // Gathering for the pile-on: deep on the haunches.
        (is::STARTUP, pose::LEAP) => {
            h.pitch = -0.4 * through;
            h.drop = 0.12 * length * through;
            h.roll = wiggle * 0.08;
        }
        // Rearing to howl, head back.
        (is::STARTUP | is::ACTIVE, pose::REAR) => {
            h.pitch = 0.25 * through;
            h.tail = -0.6;
            h.stride = 0.0;
        }
        // Low and wide, head sideways: the maul.
        (is::STARTUP, pose::MAUL) => {
            h.pitch = -0.2;
            h.drop = 0.15 * length * through;
            h.head_turn = 0.7 * through;
            h.roll = wiggle * 0.05;
        }
        // Nose down at the stone, forepaws going.
        (is::STARTUP | is::ACTIVE, pose::DIG) => {
            h.pitch = -0.5;
            h.roll = (clock * 1.7).sin() * 0.1;
            h.stride = clock * 0.25;
        }
        // Forepaws up an edge.
        (is::STARTUP, pose::CLIMB) => {
            h.pitch = 0.75 * through.max(0.3);
            h.stride = clock * 0.2;
        }
        // The pile-on's leap: up and through the air.
        (is::ACTIVE, pose::LEAP) => {
            let t = 1.0 - c.timer as f32 / a.active.max(1) as f32;
            h.pitch = 0.3 - 0.6 * t;
            h.lift = (t * std::f32::consts::PI).sin() * 0.45 * length;
        }
        (is::ACTIVE, pose::MAUL) => {
            h.pitch = 0.05;
            h.head_turn = 0.7;
        }
        // **The paw**: the forequarter rising and coming down, twice, the
        // head up and following you; then, from the lock, dropped level --
        // the frame the lane goes red. Running, it stays down.
        (is::STARTUP, pose::PAW) => {
            let lock = sim::critter::stat(sp, c.kind, CritterField::Lock).max(1) as f32;
            if (c.timer as f32) <= lock {
                h.head_dip = 0.55;
                h.pitch = -0.05;
                h.drop = 0.05 * length;
            } else {
                let paws = 1.0 - (c.timer as f32 - lock) / (a.startup as f32 - lock).max(1.0);
                h.pitch = 0.22 * (paws * std::f32::consts::TAU).sin().abs();
                h.stride = 0.0;
            }
        }
        (is::ACTIVE, pose::PAW) => {
            h.head_dip = 0.55;
            h.pitch = -0.08;
            h.stride = clock * 0.3;
        }
        // **The hook**: head low and cocked toward the side the horn comes
        // from; then the horns sweep up through.
        (is::STARTUP, pose::HOOK) => {
            let side = if mirrored(sp, c) { -1.0 } else { 1.0 };
            h.head_dip = 0.5 * through;
            h.head_turn = 0.55 * side * through;
            h.drop = 0.06 * length * through;
        }
        (is::ACTIVE, pose::HOOK) => {
            let side = if mirrored(sp, c) { -1.0 } else { 1.0 };
            h.head_dip = -0.35;
            h.head_turn = -0.45 * side;
        }
        // **The shoulder**: leaning away from the side it will throw, then
        // the flank coming across.
        (is::STARTUP, pose::LEAN) => {
            let side = if mirrored(sp, c) { -1.0 } else { 1.0 };
            h.roll = -0.22 * side * through;
            h.stride = 0.0;
        }
        (is::ACTIVE, pose::LEAN) => {
            let side = if mirrored(sp, c) { -1.0 } else { 1.0 };
            h.roll = 0.3 * side;
        }
        // **The guard**: low and square, horns forward.
        (is::STARTUP | is::ACTIVE, pose::BRACE) => {
            h.head_dip = 0.45;
            h.drop = 0.08 * length;
            h.stride = 0.0;
        }
        // **The bellow**: head thrown back, neck up -- the loudest thing in
        // the fight, and the biggest silhouette.
        (is::STARTUP | is::ACTIVE, pose::BELLOW) => {
            h.head_dip = -0.7 * through.max(0.35);
            h.pitch = 0.1 * through;
            h.stride = 0.0;
        }
        // **The cow's kick**: head down and tail up, then the rump up and
        // both hind legs straight back.
        (is::STARTUP, pose::KICK) => {
            h.head_dip = 0.4 * through;
            h.tail = 1.0;
            h.stride = 0.0;
        }
        (is::ACTIVE, pose::KICK) => {
            h.pitch = -0.3;
            h.head_dip = 0.4;
            h.tail = 1.0;
        }
        // **The buck**: head down; then the back itself moves (the surface,
        // below), which is what throws a rider.
        (is::STARTUP | is::ACTIVE, pose::BUCK) => {
            h.head_dip = 0.5;
            h.stride = 0.0;
        }
        // Flat out.
        (is::ACTIVE, pose::GALLOP) => {
            h.pitch = -0.05;
            h.head_dip = 0.15;
            h.stride = clock * 0.28;
        }
        (is::STARTUP, pose::GALLOP) => {
            h.head_dip = 0.2 * through;
        }
        // Any other lunge: stretched out, nose up a little.
        (is::ACTIVE, _) => {
            h.pitch = 0.15;
        }
        (is::STARTUP, _) => {
            h.pitch = -0.28;
            h.roll = wiggle * 0.12;
            h.stride = 0.0;
        }
        (is::RECOVERY, _) => {
            h.pitch = -0.08;
        }
        (is::FLINCH, _) => {
            h.roll = 0.45;
            h.pitch = 0.2;
        }
        // On its side, and fading as the corpse runs out.
        (is::DEAD, _) => {
            let corpse = sim::critter::stat(sp, c.kind, CritterField::Corpse).max(1) as f32;
            h.roll = std::f32::consts::FRAC_PI_2;
            h.stride = 0.0;
            h.tail = 0.0;
            h.left = (c.timer as f32 / corpse * 4.0).clamp(0.0, 1.0);
            h.drop = (1.0 - h.left) * 0.3;
        }
        _ => {}
    }
    // **The back as the ride feels it** (`PackMind::surface`): a cow's buck
    // pitches and lifts the box a rider stands on, and it is drawn moving as
    // the grip test reads it.
    if let Some(decl) = sp.pack {
        let (pitch, heave) = decl.mind.surface(c);
        h.pitch += pitch.to_f32_for_render() * std::f32::consts::TAU;
        h.lift += heave.to_f32_for_render();
    }
    // And whatever its species says about a stance of its own: the stunned
    // bull, head in the dirt.
    if let Some(stance) = stance {
        if let Some(st) = stance(c) {
            h.pitch = st.pitch;
            h.roll = st.roll;
            h.head_dip = st.head_dip;
            h.drop = st.drop * length;
            h.stride = 0.0;
        }
    }
    h
}

/// Is a critter's move thrown to its other side (`PackMind::mirrored`)?
fn mirrored(sp: &Species, c: &Critter) -> bool {
    sp.pack.is_some_and(|decl| decl.mind.mirrored(c))
}

/// Put every piece of every critter where the snapshot says.
pub fn place(
    sim: Res<crate::Sim>,
    coats: Res<Coats>,
    mut bits: Query<(
        &Bit,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let cur = &sim.cur.critters;
    let prev = &sim.prev.critters;
    let alpha = sim.clock.alpha();
    let sp = cur.sp();
    for (bit, mut transform, mut visible, mut material) in bits.iter_mut() {
        let Bit(slot, piece) = *bit;
        let c = &cur[slot];
        if !c.present() || sim.cur.pack.is_none() || (c.state == is::DEAD && c.timer == 0) {
            *visible = Visibility::Hidden;
            continue;
        }
        let show = shown(&prev[slot], c, alpha);
        let look = crate::species::look(cur.species);
        let pose = held(sp, c, look.stance);
        // Sized from `Critter::body`, the one description the hit test and
        // the aiming ray read -- so the Big One reared to howl is drawn the
        // height it is hit at.
        let body = c.body(sp);
        let l = body.half_len.to_f32_for_render() * 2.0;
        let w = body.half_wid.to_f32_for_render() * 2.0;
        let h = body.height.to_f32_for_render();
        // The body's frame: yaw about up (zero looks down +X, turning toward
        // +Z, as the simulation's), then the pose's pitch and roll.
        let frame = Quat::from_rotation_y(-show.yaw)
            * Quat::from_rotation_z(pose.pitch)
            * Quat::from_rotation_x(pose.roll);
        let hip = show.at + Vec3::Y * (h * 0.45 - pose.drop + pose.lift);
        let put = |local: Vec3| hip + frame * local;
        let legs = h * 0.45;
        let (centre, size, turn) = match piece {
            Piece::Body => (
                put(Vec3::new(0.0, h * 0.1, 0.0)),
                Vec3::new(l * 0.78, h * 0.5, w),
                Quat::IDENTITY,
            ),
            Piece::Head => (
                put(Vec3::new(l * 0.38, h * 0.28 - pose.head_dip * h * 0.3, 0.0)),
                Vec3::new(l * 0.24, h * 0.36, w * 0.7),
                Quat::from_rotation_x(pose.head_turn) * Quat::from_rotation_z(-pose.head_dip),
            ),
            Piece::Eyes => (
                put(Vec3::new(l * 0.5, h * 0.36 - pose.head_dip * h * 0.3, 0.0)),
                Vec3::new(l * 0.04, h * 0.08, w * 0.5),
                Quat::from_rotation_x(pose.head_turn) * Quat::from_rotation_z(-pose.head_dip),
            ),
            Piece::Horn(n) => {
                let side = if n == 0 { 1.0 } else { -1.0 };
                let whole = look
                    .horns
                    .is_some_and(|f| f(&sim.cur, slot).is_some_and(|h| h[n as usize]));
                if !whole || coats.horn(cur.species, c.kind).is_none() {
                    *visible = Visibility::Hidden;
                    continue;
                }
                let turn =
                    Quat::from_rotation_x(pose.head_turn) * Quat::from_rotation_z(-pose.head_dip);
                (
                    put(Vec3::new(
                        l * 0.42,
                        h * 0.48 - pose.head_dip * h * 0.3,
                        side * w * 0.55,
                    )),
                    Vec3::new(l * 0.07, h * 0.07, w * 0.75),
                    turn * Quat::from_rotation_x(side * 0.5),
                )
            }
            Piece::Tail => {
                let lift = Quat::from_rotation_z(-pose.tail);
                let root = Vec3::new(-l * 0.39, h * 0.2, 0.0);
                (
                    put(root + lift * Vec3::new(-l * 0.15, 0.0, 0.0)),
                    Vec3::new(l * 0.3, h * 0.08, w * 0.18),
                    lift,
                )
            }
            Piece::Leg(n) => {
                let front = if n < 2 { 1.0 } else { -1.0 };
                let side = if n % 2 == 0 { 1.0 } else { -1.0 };
                // Diagonal pairs swing together: a trot.
                let phase = pose.stride * std::f32::consts::TAU
                    + if n == 1 || n == 2 {
                        std::f32::consts::PI
                    } else {
                        0.0
                    };
                let swing = Quat::from_rotation_z(phase.sin() * 0.45);
                let top = Vec3::new(front * l * 0.28, -h * 0.1, side * w * 0.35);
                (
                    put(top + swing * Vec3::new(0.0, -legs * 0.5, 0.0)),
                    Vec3::new(w * 0.18, legs, w * 0.18),
                    swing,
                )
            }
        };
        *transform = Transform {
            translation: centre,
            rotation: frame * turn,
            scale: size * pose.left.max(0.05),
        };
        *visible = Visibility::Inherited;
        let (hide, eyes) = coats.of(cur.species, c.kind);
        let horn = coats.horn(cur.species, c.kind);
        let wanted = match piece {
            Piece::Eyes => eyes,
            Piece::Tail if c.has(flag::TOKEN) => eyes,
            Piece::Horn(_) => horn.unwrap_or(hide),
            _ => hide,
        };
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
    }
}

/// The overlay: each body's hit box, its live bite, the ring and the den.
/// All read off the simulation's own functions.
pub fn overlay(show: Res<crate::debug::ShowDebug>, sim: Res<crate::Sim>, mut gizmos: Gizmos) {
    if !show.0 {
        return;
    }
    let Some(pack) = sim.cur.pack else { return };
    let sp = pack.sp();
    let flat = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    for (slot, c) in sim
        .cur
        .critters
        .iter()
        .enumerate()
        .filter(|(_, c)| c.alive())
    {
        // The box the hit test reads, corner to corner.
        let body = c.body(sp);
        let (lo, hi) = (body.min(), body.max());
        let corner = |x: Fx, y: Fx, z: Fx| fx3(body.to_world(sim::V3::new(x, y, z)));
        let colour = if c.has(flag::TOKEN) { TOKEN } else { HURT };
        for y in [lo.y, hi.y] {
            let ring = [
                corner(lo.x, y, lo.z),
                corner(hi.x, y, lo.z),
                corner(hi.x, y, hi.z),
                corner(lo.x, y, hi.z),
            ];
            for i in 0..4 {
                gizmos.line(ring[i], ring[(i + 1) % 4], colour);
            }
        }
        for (x, z) in [(lo.x, lo.z), (hi.x, lo.z), (hi.x, hi.z), (lo.x, hi.z)] {
            gizmos.line(corner(x, lo.y, z), corner(x, hi.y, z), colour);
        }
        // The bite out this frame, or the one winding up.
        if let Some(t) = sim::pack::telegraph(Some(&pack), &sim.cur.critters, slot) {
            let col = if t.live { HIT } else { WINDUP };
            let at = fx3(t.anchor);
            for y in [t.low, t.high] {
                gizmos.circle(
                    Isometry3d::new(Vec3::new(at.x, y.to_f32_for_render(), at.z), flat),
                    t.radius.to_f32_for_render(),
                    col,
                );
            }
        }
    }
    // The ring each fighter has been given, and the den.
    for seen in pack.seen.iter().filter(|s| s.alive && s.ring_places > 0) {
        for slot in 0..seen.ring_places {
            let at = fx3(sim::pack::ring_point(&pack, seen, slot));
            gizmos.circle(Isometry3d::new(at + Vec3::Y * 0.02, flat), 0.15, RING);
        }
    }
    gizmos.circle(
        Isometry3d::new(fx3(pack.home) + Vec3::Y * 0.02, flat),
        1.0,
        DEN,
    );
}

fn fx3(v: sim::V3) -> Vec3 {
    Vec3::new(
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    )
}

const HURT: Color = Color::srgb(0.95, 0.95, 0.4);
const TOKEN: Color = Color::srgb(1.0, 0.55, 0.1);
const HIT: Color = Color::srgb(1.0, 0.23, 0.31);
const WINDUP: Color = Color::srgb(1.0, 0.78, 0.25);
const RING: Color = Color::srgb(0.5, 0.8, 1.0);
const DEN: Color = Color::srgb(0.6, 0.45, 0.3);
