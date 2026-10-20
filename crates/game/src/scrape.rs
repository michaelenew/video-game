//! **Dust off a wall** (2026-10-20): what a wall slide and a wall jump
//! scrape off the face.
//!
//! On a slide the near hand and foot are pressed to the face and dragging
//! (`view::play::on_the_wall`); a stream of dust comes off both, thicker the
//! faster she goes down, pushed out from the face and hanging where it was
//! left while the body goes on down past it -- which is the picture of
//! friction. A kick off the face is a burst of it where her feet were.
//!
//! The renderer's own, like the blob's puffs (`keep`): dust decides nothing,
//! is never in the snapshot, and a rollback that re-runs a few frames of a
//! slide only lays a few more motes. Its colour is the face's own, ground
//! pale (`look::Palette::dust`); what the face is made of is the simulation's
//! answer (`Terrain::material_of_face`), rock where the wall is the land's.
//! Drawn in the standard material, which `warm` primes as it is made.

use bevy::prelude::*;
use sim::arena::{ArenaId, Material};
use sim::state::MAX_PLAYERS;

/// How long a mote lasts, in seconds, shrinking evenly to nothing.
const LIFE: f32 = 0.6;
/// Seconds between motes off each contact at full pace, and the speed down
/// the wall (m/s) that is full pace: the slide's own cap
/// (`sim::tuning::wall_slide_speed`). Slower, fewer: a slide held nearly
/// still on the face leaves a mote now and then, not a stream.
const EVERY: f32 = 0.04;
fn full() -> f32 {
    sim::tuning::wall_slide_speed().to_f32_for_render().max(0.5)
}
/// Below this speed down the face (m/s) nothing is scraped off at all.
const STILL: f32 = 0.3;
/// How wide a mote is at birth, in metres, and how hard it leaves the face
/// (m/s), and how fast that dies away (a rate a second).
const SIZE: f32 = 0.2;
const PUSH: f32 = 1.2;
const DRAG: f32 = 4.0;
/// How much of its own colour a mote gives off, so it shows on a face in
/// shade.
const GLOW: f32 = 0.45;
/// How many motes a kick throws off the face.
const KICK: usize = 14;
/// Room a seat: a full slide's life of it (the most `Default` gives an
/// array).
const MOTES: usize = 32;
/// How high on the face the hand and the foot are pressed, over her feet:
/// the slide clip's own marks (`anim::clips::wall`).
const HAND: f32 = 1.25;
const FOOT: f32 = 0.5;

#[derive(Clone, Copy)]
struct Mote {
    at: Vec3,
    vel: Vec3,
    born: f32,
    size: f32,
    turn: Quat,
}

/// Every seat's dust: a ring of motes, and what is remembered between
/// pictures to know when to lay the next and when a kick happened.
#[derive(Resource, Default)]
pub struct Dust {
    ring: [[Option<Mote>; MOTES]; MAX_PLAYERS],
    next: [usize; MAX_PLAYERS],
    left_at: [f32; MAX_PLAYERS],
    /// Wall jumps seen, a seat: one more than this is a kick to burst.
    kicks: [u8; MAX_PLAYERS],
    /// The colour each seat's dust is in now, and the face it was for.
    face: [Option<(ArenaId, Material)>; MAX_PLAYERS],
    origin: Option<Vec3>,
    dice: u32,
}

impl Dust {
    /// Nought to one, cheaply.
    fn roll(&mut self) -> f32 {
        self.dice = self
            .dice
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.dice >> 8) as f32 / (1u32 << 24) as f32
    }

    fn lay(&mut self, seat: usize, at: Vec3, out: Vec3, t: f32, push: f32) {
        let spread = Vec3::new(self.roll(), self.roll(), self.roll()) - Vec3::splat(0.5);
        let size = SIZE * (0.6 + 0.8 * self.roll());
        let turn = Quat::from_euler(
            EulerRot::YXZ,
            self.roll() * std::f32::consts::TAU,
            self.roll() * std::f32::consts::TAU,
            0.0,
        );
        let i = self.next[seat];
        self.next[seat] = (i + 1) % MOTES;
        self.ring[seat][i] = Some(Mote {
            at: at + out * 0.08 + spread * 0.1,
            vel: out * push + spread * push * 0.9 + Vec3::Y * 0.25,
            born: t,
            size,
            turn,
        });
    }
}

/// One mote on screen: a seat's, a slot in its ring.
#[derive(Component)]
pub struct MotePiece(pub usize, pub usize);

/// Each seat's dust material, coloured as its face is.
#[derive(Resource)]
pub struct DustLook([Handle<StandardMaterial>; MAX_PLAYERS]);

/// A lump of dust: a low ball, pushed about a little so a cloud of them is
/// not a cloud of marbles.
fn lump() -> Mesh {
    let mut mesh = Sphere::new(0.5).mesh().ico(1).unwrap();
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(ps)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for p in ps.iter_mut() {
            let v = Vec3::from(*p);
            let d = v.normalize_or_zero();
            let bump = (d.x * 3.7 + 0.4).sin() * (d.y * 2.9 + 1.1).sin() + 0.5 * (d.z * 4.1).sin();
            *p = (v * (1.0 + 0.18 * bump)).to_array();
        }
    }
    mesh.compute_smooth_normals();
    mesh
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(lump());
    let looks: [Handle<StandardMaterial>; MAX_PLAYERS] = std::array::from_fn(|_| {
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.78, 0.74),
            perceptual_roughness: 1.0,
            ..default()
        })
    });
    for (seat, look) in looks.iter().enumerate() {
        for i in 0..MOTES {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(look.clone()),
                Transform::default(),
                Visibility::Hidden,
                bevy::pbr::NotShadowCaster,
                MotePiece(seat, i),
            ));
        }
    }
    commands.insert_resource(DustLook(looks));
}

/// Lay new dust off every fighter on a wall, burst it off every kick, and
/// put every mote where its flight has taken it.
pub fn place(
    sim: Res<crate::Sim>,
    time: Res<Time>,
    hands: Res<crate::Hands>,
    looks: Res<DustLook>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dust: ResMut<Dust>,
    mut drawn: Query<(&MotePiece, &mut Transform, &mut Visibility)>,
) {
    let w = &sim.cur;
    let t = time.elapsed_secs();
    // The world moved into the next place's coordinates: the dust, which
    // hangs where it was left, moves with it.
    let o = crate::fx3(w.map_origin());
    if let Some(last) = dust.origin.replace(o)
        && last != o
    {
        let by = last - o;
        for m in dust.ring.iter_mut().flatten().flatten() {
            m.at += by;
        }
    }
    let frame = view::interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    let present = w.present().clamp(1, MAX_PLAYERS);
    for seat in 0..MAX_PLAYERS {
        let p = &w.players[seat];
        // A kick: the count went up since the last picture. Reset on
        // landing, so a count that went down is only remembered.
        let kicked = seat < present && p.wall_jumps > dust.kicks[seat] && p.wall_jumped.is_some();
        dust.kicks[seat] = p.wall_jumps;
        let view = frame.players[seat];
        let on = view.wall.filter(|_| seat < present);
        // Due another mote: off a slide, as often as its pace says.
        let due = on.is_some_and(|wall| {
            let pace = (wall.down / full()).clamp(0.0, 1.0);
            wall.kick.is_none()
                && wall.down >= STILL
                && t - dust.left_at[seat] >= EVERY / pace.max(0.1)
        });
        if !kicked && !due {
            continue;
        }
        // What the face is, for the colour.
        let wall = if kicked { p.wall_jumped } else { p.wall };
        let r = sim::tuning::body_radius();
        let probe = sim::V3::new(
            wall.at
                .x
                .sub(wall.normal.x.mul(r.add(sim::Fx::ratio(1, 10)))),
            wall.at.y.add(sim::Fx::ONE),
            wall.at
                .z
                .sub(wall.normal.z.mul(r.add(sim::Fx::ratio(1, 10)))),
        );
        let face = w
            .terrain()
            .material_of_face(probe, sim::Fx::ratio(1, 5))
            .unwrap_or(Material::Rock);
        if dust.face[seat] != Some((w.arena, face)) {
            dust.face[seat] = Some((w.arena, face));
            let c = look::palette::of(w.arena).dust(face);
            if let Some(m) = materials.get_mut(&looks.0[seat]) {
                m.base_color = Color::srgb(c[0], c[1], c[2]);
                // Lit through as well as lit: a wall slid down is as often
                // the shaded side of something as not, and dust in the shade
                // went the shade's blue and vanished into the face.
                m.emissive = Color::srgb(c[0], c[1], c[2]).to_linear() * GLOW;
            }
        }
        let n = crate::fx3(wall.normal);
        if kicked {
            let feet = crate::fx3(wall.at) - n * r.to_f32_for_render();
            for k in 0..KICK {
                let up = FOOT * (0.4 + 0.25 * (k % 4) as f32);
                dust.lay(seat, feet + Vec3::Y * up, n, t, PUSH * 1.8);
            }
            dust.left_at[seat] = t;
            continue;
        }
        let Some(on) = on else {
            continue;
        };
        let pace = (on.down / full()).clamp(0.0, 1.0);
        dust.left_at[seat] = t;
        // Off the hand pressed to the face -- the drawn hand, so the dust
        // comes off it -- and off the foot under it, on the face.
        let contact = Vec3::from_array(on.contact);
        let hand = hands.0[seat][if on.left { 0 } else { 1 }];
        // While the body is still turning round to the face the hand is
        // not on it yet: off the face's own mark at the hand's height then.
        let hand = if hand.distance(contact + Vec3::Y * HAND) < 0.6 {
            hand
        } else {
            contact + Vec3::Y * HAND
        };
        dust.lay(seat, hand, n, t, PUSH * (0.4 + 0.6 * pace));
        dust.lay(
            seat,
            contact + Vec3::Y * FOOT,
            n,
            t,
            PUSH * (0.4 + 0.6 * pace),
        );
    }
    for (piece, mut transform, mut visible) in drawn.iter_mut() {
        let (seat, i) = (piece.0, piece.1);
        let live = dust.ring[seat][i].filter(|m| t - m.born < LIFE);
        let Some(m) = live else {
            dust.ring[seat][i] = None;
            *visible = Visibility::Hidden;
            continue;
        };
        let age = t - m.born;
        // Out from the face and slowing: where a push with drag on it has
        // got to, worked out from its age rather than stepped.
        let gone = (1.0 - (-age * DRAG).exp()) / DRAG;
        let share = 1.0 - age / LIFE;
        // Puffing up a little as it goes, then shrinking away.
        let size = m.size * (1.0 + 0.8 * age / LIFE) * share;
        *transform = Transform {
            translation: m.at + m.vel * gone,
            rotation: m.turn,
            scale: Vec3::splat(size.max(0.001)),
        };
        *visible = Visibility::Inherited;
    }
}
