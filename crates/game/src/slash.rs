//! **The arc her blades cut** (`docs/design/exploration/0024_a_character_done_properly.md`).
//!
//! A ribbon swept through where the Reaver's dagger has just been: from
//! the blade's root to its tip, at every simulation frame of the last
//! [`LIFE`] frames of a cut and where it is drawn now, bright at the newest
//! edge and at the tip, gone at the oldest. The blade's root and tip are two points the model
//! carries under each hand (`blade.l.tip` and so on, written by
//! `art/reaver/outfit.py`), so the ribbon leaves exactly the steel that is
//! drawn, whatever the pose.
//!
//! When it is drawn is the simulation's word: while a move is out and for
//! the first frames of its recovery, as the Champion's trail
//! (`view::arms::trail`), and its age is counted in the simulation's
//! frames, so a capture stopped on a frame (`SHOT_FRAME`) holds the cut it
//! stopped in. Where the steel was is the picture's: the drawn blade is
//! sampled once a simulation frame, and the newest edge is wherever it is
//! drawn now. That keeps a memory the snapshot does not have, which is
//! allowed of something that only decorates (`docs/design/animation.md`):
//! a rollback moves the newest edge and the rest catches up within [`LIFE`].
//!
//! A fixed mesh a hand, written in place: nothing is allocated in a fight.

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::NoFrustumCulling;
use sim::state::{Action, MAX_PLAYERS};

/// How many samples a ribbon keeps, and how long one lasts, in simulation
/// frames: a sixth of a second.
pub const SAMPLES: usize = 16;
pub const LIFE: f32 = 10.0;

/// How bright the ribbon is at its newest edge, by the tip and by the root:
/// her violet, hotter toward the point.
const TIP: [f32; 4] = [0.95, 0.55, 1.6, 1.0];
const ROOT: [f32; 4] = [0.35, 0.12, 0.8, 1.0];

/// A ribbon: which seat and which hand (0 left, 1 right).
#[derive(Component)]
pub struct Ribbon {
    pub owner: usize,
    pub hand: usize,
}

/// A hand's samples, newest first: tip, root, and the simulation frame
/// each was taken on. The first is where the blade is drawn now.
#[derive(Clone, Copy)]
struct Trace {
    tip: [Vec3; SAMPLES],
    root: [Vec3; SAMPLES],
    at: [f32; SAMPLES],
    count: usize,
    /// The frame the last kept sample (the one after the live edge) is of.
    last: Option<u32>,
}

impl Default for Trace {
    fn default() -> Self {
        Trace {
            tip: [Vec3::ZERO; SAMPLES],
            root: [Vec3::ZERO; SAMPLES],
            at: [0.0; SAMPLES],
            count: 0,
            last: None,
        }
    }
}

impl Trace {
    /// The blade as drawn now, on `frame` (with `alpha` of the next).
    fn see(&mut self, tip: Vec3, root: Vec3, frame: u32, alpha: f32) {
        // More than a metre and a half a frame is not a cut: the world
        // moved into another place's coordinates, or she blinked. Start
        // again. (Her fastest dash is about half a metre a frame.)
        let frames = self
            .last
            .map_or(1, |last| frame.saturating_sub(last).max(1));
        if self.count > 0 && self.tip[0].distance(tip) > 1.5 * frames as f32 {
            self.count = 0;
            self.last = None;
        }
        // A new simulation frame: the live edge so far is kept as a sample,
        // and any frames that were never drawn (a slow machine draws one
        // picture for several) are filled in on the way to where it is now.
        if let (true, Some(last)) = (self.count > 0, self.last) {
            if last != frame {
                let (was_tip, was_root, was_at) = (self.tip[0], self.root[0], self.at[0]);
                let skipped = frame.saturating_sub(last).min(SAMPLES as u32);
                for k in 0..skipped {
                    let t = (k + 1) as f32 / (skipped + 1) as f32;
                    self.tip.copy_within(0..SAMPLES - 1, 1);
                    self.root.copy_within(0..SAMPLES - 1, 1);
                    self.at.copy_within(0..SAMPLES - 1, 1);
                    self.tip[0] = was_tip.lerp(tip, t);
                    self.root[0] = was_root.lerp(root, t);
                    self.at[0] = was_at + (frame as f32 + alpha - was_at) * t;
                    self.count = (self.count + 1).min(SAMPLES);
                }
                self.last = Some(frame);
            }
        }
        self.tip[0] = tip;
        self.root[0] = root;
        self.at[0] = frame as f32 + alpha;
        self.count = self.count.max(1);
        if self.last.is_none() {
            self.last = Some(frame);
        }
    }

    fn forget(&mut self, now: f32) {
        while self.count > 0 && now - self.at[self.count - 1] > LIFE {
            self.count -= 1;
        }
        if self.count == 0 {
            self.last = None;
        }
    }
}

/// Every hand's trace.
#[derive(Resource, Default)]
pub struct Traces([[Trace; 2]; MAX_PLAYERS]);

/// The stroke a cut is painted with: a flat brush dragged along the arc,
/// its bristles each their own weight, breaking into dry streaks toward the
/// old end. Across is `v` (the tip at 0), along is `u` (the newest edge at
/// 0). Made once, from a hash, so nothing is read from disk.
fn brush() -> Image {
    const W: usize = 128;
    const H: usize = 32;
    let hash = |n: u32| -> f32 {
        let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
        x ^= x >> 15;
        x = x.wrapping_mul(0x2C1B_3C6D);
        x ^= x >> 12;
        (x & 0xFFFF) as f32 / 65535.0
    };
    let mut data = vec![0u8; W * H * 4];
    for y in 0..H {
        // Each bristle's weight, and where along the stroke it runs dry.
        let weight = 0.45 + 0.55 * hash(y as u32 * 7 + 1);
        let dry = 0.35 + 0.6 * hash(y as u32 * 13 + 5);
        let v = y as f32 / (H - 1) as f32;
        // Heavier toward the tip's side, feathered at both edges.
        let across = (1.0 - v).powf(0.7) * (v * 8.0).min(1.0).min((1.0 - v) * 14.0 + 0.2);
        for x in 0..W {
            let u = x as f32 / (W - 1) as f32;
            let flicker = 0.75 + 0.25 * hash((y * 131 + x / 6) as u32);
            let alive = ((dry - u) / 0.12).clamp(0.0, 1.0);
            let a = (weight * across * flicker * alive).clamp(0.0, 1.0);
            let i = (y * W + x) * 4;
            let c = (255.0 * a) as u8;
            data[i..i + 4].copy_from_slice(&[c, c, c, c]);
        }
    }
    Image::new(
        Extent3d {
            width: W as u32,
            height: H as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// The ribbons' mesh and material, one a hand, made at startup.
pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(images.add(brush())),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        double_sided: true,
        cull_mode: None,
        ..default()
    });
    for owner in 0..MAX_PLAYERS {
        for hand in 0..2 {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; SAMPLES * 2]);
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                vec![[0.0f32, 1.0, 0.0]; SAMPLES * 2],
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32; 4]; SAMPLES * 2]);
            let uv: Vec<[f32; 2]> = (0..SAMPLES * 2)
                .map(|i| [(i / 2) as f32 / (SAMPLES - 1) as f32, (i % 2) as f32])
                .collect();
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
            let mut indices = Vec::with_capacity((SAMPLES - 1) * 6);
            for i in 0..(SAMPLES as u32 - 1) {
                let (a, b, c, d) = (2 * i, 2 * i + 1, 2 * i + 2, 2 * i + 3);
                indices.extend_from_slice(&[a, b, c, b, d, c]);
            }
            mesh.insert_indices(Indices::U32(indices));
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material.clone()),
                Transform::default(),
                Visibility::Hidden,
                NotShadowCaster,
                NoFrustumCulling,
                Ribbon { owner, hand },
            ));
        }
    }
}

/// Whether a fighter's blades are cutting: the last frames of a wind-up,
/// where the swing comes down; a move out; or the first frames of its
/// recovery.
fn cutting(p: &sim::state::Player) -> bool {
    match p.action {
        Action::Startup { left, .. } => left <= 3,
        Action::Active { .. } => true,
        Action::Recovery { kind, left } => {
            let m = sim::moves::get(p.class, kind);
            m.recovery.saturating_sub(left) < view::arms::TRAIL_GHOST
        }
        _ => false,
    }
}

/// Sample every blade and redraw every ribbon. After the transforms are
/// propagated, so the blades are where this frame drew them.
pub fn draw(
    sim: Res<crate::Sim>,
    rigs: Res<crate::reaver::Rigs>,
    points: Query<&GlobalTransform>,
    mut traces: ResMut<Traces>,
    mut ribbons: Query<(&Ribbon, &Mesh3d, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let alpha = sim.clock.alpha();
    let frame = sim.cur.frame;
    let now = frame as f32 + alpha;
    for owner in 0..MAX_PLAYERS {
        let p = &sim.cur.players[owner];
        let blades = rigs.blades[owner].filter(|_| crate::reaver::drawn(p.class));
        for hand in 0..2 {
            let trace = &mut traces.0[owner][hand];
            if let (Some(blades), true) = (blades, cutting(p)) {
                let [tip, root] = blades[hand];
                if let (Ok(tip), Ok(root)) = (points.get(tip), points.get(root)) {
                    trace.see(tip.translation(), root.translation(), frame, alpha);
                }
            }
            trace.forget(now);
        }
    }
    for (ribbon, mesh, mut seen) in ribbons.iter_mut() {
        let trace = &traces.0[ribbon.owner][ribbon.hand];
        let want = if trace.count >= 2 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *seen != want {
            *seen = want;
        }
        if trace.count < 2 {
            continue;
        }
        let Some(mesh) = meshes.get_mut(&mesh.0) else {
            continue;
        };
        if let Some(VertexAttributeValues::Float32x3(at)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            for i in 0..SAMPLES {
                // Past the samples kept, the ribbon folds onto its oldest
                // edge: zero width, nothing drawn.
                let k = i.min(trace.count - 1);
                at[2 * i] = trace.tip[k].to_array();
                at[2 * i + 1] = trace.root[k].to_array();
            }
        }
        if let Some(VertexAttributeValues::Float32x4(colour)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
        {
            for i in 0..SAMPLES {
                let k = i.min(trace.count - 1);
                let age = ((now - trace.at[k]) / LIFE).clamp(0.0, 1.0);
                let left = if i < trace.count {
                    (1.0 - age) * (1.0 - age)
                } else {
                    0.0
                };
                colour[2 * i] = TIP.map(|c| c * left);
                colour[2 * i + 1] = ROOT.map(|c| c * left);
            }
        }
    }
}
