//! **Trails**: the arc a blade leaves, the streak behind a dash, drawn as a
//! ribbon and painted by their own shader.
//!
//! A trail is **the path the drawn thing took**: every picture, something
//! that moves says where it is drawn now -- an [`Emit`]: the inner and outer
//! edge of a band, how bright, what colour, how far back it reaches -- and
//! that is added to a short memory of where it has been
//! (`view::trail::Path`); the trail is that memory joined by a curve
//! (`view::trail::ribbon`). Its newest end is the thing on every picture,
//! through a wind-up, a cut and a follow-through alike; a trail taken from a
//! hit volume instead was somewhere the weapon was not, since the volume is
//! only out for the cut and the weapon goes on moving after it.
//!
//! The emitters are the systems that run before [`place`] in the same chain:
//! [`emit_champions`] for the Champion's three weapons, and one beside each
//! other thing that leaves a trail. Each names its trail with a [`key`], so
//! a trail keeps its path from picture to picture and a new one starts
//! clean. There is a fixed pool of [`SLOTS`] strips, spawned once.
//!
//! A strip is one mesh, a point on the inner edge and one on the outer per
//! column. Each vertex carries how long ago the thing was there (`uv.x`, 1
//! now and 0 as far back as it reaches), which edge it is (`uv.y`), the tint
//! and how bright it was there (the colour's alpha). `swing.wgsl` turns
//! those into the look.

use bevy::asset::weak_handle;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey, NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::mesh::{
    Indices, MeshVertexBufferLayoutRef, PrimitiveTopology, VertexAttributeValues,
};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};
use bevy::render::view::NoFrustumCulling;
use view::trail::{Path, RIBBON};

use crate::Sim;

const SHADER: Handle<Shader> = weak_handle!("4d9b2f71-6a3e-4c58-9e17-b2c8d5a0f6e3");

/// The arc's material. Everything it varies by is in the vertices, so one
/// serves every fighter and every weapon.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct SwingMaterial {}

impl Material for SwingMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    /// Seen from either side: a cut across the camera shows its back.
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        // `SHOT_SWING=uv`: the strip solid, red along the arc and green
        // across it, to see where it is and which way it runs.
        // `SHOT_SWING=alpha`: the strip solid, as bright as it is opaque.
        let def = match crate::platform::env("SHOT_SWING").as_deref() {
            Some("uv") => Some("SWING_UV"),
            Some("alpha") => Some("SWING_ALPHA"),
            _ => None,
        };
        if let (Some(def), Some(fragment)) = (def, descriptor.fragment.as_mut()) {
            fragment.shader_defs.push(def.into());
        }
        Ok(())
    }
}

pub struct SwingPlugin;

impl Plugin for SwingPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<Assets<Shader>>().insert(
            SHADER.id(),
            Shader::from_wgsl(include_str!("swing.wgsl"), "game/src/swing.wgsl"),
        );
        app.add_plugins(MaterialPlugin::<SwingMaterial>::default())
            .init_resource::<Slots>()
            .init_resource::<Emits>()
            .add_systems(Startup, spawn);
    }
}

/// How many trails can be drawn at once.
pub const SLOTS: usize = 64;

/// How far a thing can move between two pictures and still be the same thing.
const JUMP: f32 = 4.0;

/// One strip of the pool: its own mesh, rewritten in place each picture.
#[derive(Component)]
pub struct Strip {
    slot: usize,
    mesh: Handle<Mesh>,
}

/// **Where a moving thing is drawn this picture**, for the trail behind it.
#[derive(Clone, Copy, Debug)]
pub struct Emit {
    /// Which trail: the same thing says the same key every picture.
    pub key: u32,
    pub inner: Vec3,
    pub outer: Vec3,
    /// How bright the trail is here, 0 to 1: nothing through a wind-up,
    /// all of it while the thing strikes, fading after.
    pub glow: f32,
    /// In linear light.
    pub tint: [f32; 3],
    /// How many frames back the trail reaches.
    pub window: f32,
    pub style: Style,
}

/// How a trail is painted (`swing.wgsl`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// **What an edge leaves**: a crescent, full at the weapon and drawn to a
    /// point behind it, a hot line along the outer edge (the one the tip
    /// drew), streaks fraying at the tail.
    Blade,
    /// **What a body in flight leaves**: the same streaks and taper, but hot
    /// down the middle and soft at both edges -- a ball, a thrown blade, a
    /// body dashing. The band is laid across the line of flight, square to
    /// the eye ([`across`]).
    Streak,
}

impl Style {
    fn code(self) -> f32 {
        match self {
            Style::Blade => 0.0,
            Style::Streak => 1.0,
        }
    }
}

/// The two edges of a [`Style::Streak`]'s band round `at`, `half` either
/// side: square to the way it is going and to the eye, so the band is seen
/// face on whichever way it flies. `going` need not be unit; with no way to
/// tell, the band stands up.
pub fn across(at: Vec3, going: Vec3, eye: Vec3, half: f32) -> (Vec3, Vec3) {
    let side = going.cross(eye - at).try_normalize().unwrap_or(Vec3::Y);
    (at - side * half, at + side * half)
}

/// The name of a trail: what it follows (`source`, one per kind of thing),
/// whose (`index`, a fighter's or a creature's slot) and which of its parts.
pub const fn key(source: u8, index: usize, part: u8) -> u32 {
    ((source as u32) << 24) | ((index as u32 & 0xffff) << 8) | part as u32
}

/// The kinds of thing that leave trails, for [`key`].
pub mod source {
    pub const CHAMPION: u8 = 1;
    /// A piece of an effect (`place_effects`), by its slot.
    pub const EFFECT: u8 = 2;
    /// A piece of a Dual mage's spell (`place_spells`), by her slot.
    pub const SPELL: u8 = 3;
    /// The Blood mage's scythe.
    pub const SCYTHE: u8 = 4;
    /// A swing with no drawn weapon, from its volume.
    pub const VOLUME: u8 = 5;
    /// A spike of a creature's volley (`beast::signs`), by its slot.
    pub const VOLLEY: u8 = 6;
}

/// A colour as a trail's vertices carry it.
pub fn linear(c: Color) -> [f32; 3] {
    let c = c.to_linear();
    [c.red, c.green, c.blue]
}

/// This picture's emits, gathered by the emitters and spent by [`place`].
#[derive(Resource)]
pub struct Emits {
    list: [Option<Emit>; SLOTS],
    count: usize,
}

impl Default for Emits {
    fn default() -> Self {
        Emits {
            list: [None; SLOTS],
            count: 0,
        }
    }
}

impl Emits {
    /// Add one. Past [`SLOTS`] it is dropped: a trail missing for a picture
    /// starts again clean, which is a flicker, not a fault.
    pub fn add(&mut self, e: Emit) {
        if self.count < SLOTS {
            self.list[self.count] = Some(e);
            self.count += 1;
        }
    }
}

/// What a strip is drawing: where the thing it follows has been, the frame
/// the path's clock counts from (so its times stay small and exact however
/// long the game has run), and the last thing it said -- what the trail is
/// painted with, and how long it lingers once the thing says no more. The
/// renderer's own, like a cross-fade: see `view::trail::Path`.
#[derive(Clone, Copy)]
pub struct Held {
    path: Path,
    base: u32,
    last: Emit,
}

#[derive(Resource)]
pub struct Slots([Option<Held>; SLOTS]);

impl Default for Slots {
    fn default() -> Self {
        Slots([None; SLOTS])
    }
}

/// The Champion's weapons' tints, so the three arcs are told apart as well as
/// the three silhouettes: the sword's icy blue, white at its edge, the
/// hammer's a hot ember, the spear's a thin blue streak -- and the string's
/// third hit and the finishers in gold, whatever threw them, the colour of
/// the tempo bar's window into it (2026-10-20, from play).
fn champion_tint(cut: &view::arms::Cut) -> [f32; 3] {
    if cut.place >= 2 {
        return linear(Color::srgb(1.0, 0.78, 0.25));
    }
    linear(match cut.which {
        0 => Color::srgb(0.55, 0.80, 1.0),
        1 => Color::srgb(1.0, 0.55, 0.20),
        _ => Color::srgb(0.45, 0.80, 1.0),
    })
}

/// The strip, every column at the origin: two vertices a column, inner then
/// outer, and the two triangles between each column and the next.
fn strip() -> Mesh {
    let vertices = RIBBON * 2;
    let mut indices = Vec::with_capacity((RIBBON - 1) * 6);
    for k in 0..RIBBON as u32 - 1 {
        let (a, b, c, d) = (2 * k, 2 * k + 1, 2 * k + 2, 2 * k + 3);
        indices.extend_from_slice(&[a, b, c, b, d, c]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; vertices])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32; 2]; vertices])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, vec![[0.0f32; 2]; vertices])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32; 4]; vertices])
    .with_inserted_indices(Indices::U32(indices))
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SwingMaterial>>,
) {
    let material = materials.add(SwingMaterial {});
    for slot in 0..SLOTS {
        let mesh = meshes.add(strip());
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Visibility::Hidden,
            // The strip is rewritten each picture and its bounds are not, so
            // the box it was made with would cull it.
            NoFrustumCulling,
            NotShadowCaster,
            NotShadowReceiver,
            Strip { slot, mesh },
        ));
    }
}

/// **The Champion's weapons**: where the weapon is drawn, through her cut.
pub fn emit_champions(
    sim: Res<Sim>,
    hands: Res<crate::Hands>,
    torsos: Res<crate::Torsos>,
    mut emits: ResMut<Emits>,
) {
    let alpha = sim.clock.alpha();
    for owner in 0..sim::state::MAX_PLAYERS {
        let Some(cut) = view::arms::cut(&sim.cur.players[owner], alpha) else {
            continue;
        };
        let Some(arms) = crate::drawn_arms(&sim, &hands, &torsos, owner) else {
            continue;
        };
        let s = view::arms::sample(&arms, &cut, 0.0);
        emits.add(Emit {
            key: key(source::CHAMPION, owner, 0),
            inner: s.inner.into(),
            outer: s.outer.into(),
            glow: s.glow,
            tint: champion_tint(&cut),
            window: view::arms::WINDOW,
            style: Style::Blade,
        });
    }
}

/// **The Blood mage's scythe**: the blade as it is drawn, out to as far as
/// her grey makes the swing reach -- so what the essence buys is seen -- and
/// brighter the more grey she carries.
pub fn emit_scythes(sim: Res<Sim>, hands: Res<crate::Hands>, mut emits: ResMut<Emits>) {
    let alpha = sim.clock.alpha();
    for owner in 0..sim::state::MAX_PLAYERS {
        let p = &sim.cur.players[owner];
        let Some((kind, glow)) = view::trail::strike(p, alpha) else {
            continue;
        };
        if !sim::moves::blood::scythe(kind) {
            continue;
        }
        let Some(scythe) = crate::drawn_scythe(&sim, &hands, owner) else {
            continue;
        };
        let (grip, tip) = (Vec3::from(scythe.grip), Vec3::from(scythe.tip));
        let outer = grip + (tip - grip) * view::scythe::reach_share(p);
        let grey = p.grey_share().to_f32_for_render();
        emits.add(Emit {
            key: key(source::SCYTHE, owner, 0),
            inner: grip.lerp(tip, 0.45),
            outer,
            glow: glow * (0.45 + 0.55 * grey),
            // Her essence: the pools' magenta-crimson.
            tint: linear(Color::srgb(0.80, 0.12, 0.48)),
            window: SWING_WINDOW,
            style: Style::Blade,
        });
    }
}

/// How many frames back a swing's trail reaches, for the swings that are
/// not the Champion's.
const SWING_WINDOW: f32 = 6.0;

/// **A swing nobody holds a weapon for** -- the Shadow Reaver's Plunge and
/// Gallows are her arm and her shadow: the volume's line, its outer part, and
/// between frames turned about its hub from where it was on the frame before,
/// so it sweeps. The Champion's and the Blood mage's are drawn off their
/// weapons ([`emit_champions`], [`emit_scythes`]).
pub fn emit_volume_swings(sim: Res<Sim>, mut emits: ResMut<Emits>) {
    let alpha = sim.clock.alpha();
    let same_frame = sim.prev.map_origin() == sim.cur.map_origin();
    for owner in 0..sim::state::MAX_PLAYERS {
        let p = &sim.cur.players[owner];
        if matches!(p.class, sim::Class::Champion | sim::Class::BloodMage) {
            continue;
        }
        let Some((kind, glow)) = view::trail::strike(p, alpha) else {
            continue;
        };
        if !matches!(
            sim::moves::get(p.class, kind).shape,
            sim::moves::Shape::Swing(_)
        ) {
            continue;
        }
        let line = |p: &sim::state::Player| {
            sim::state::hitbox(p).map(|hb| (crate::fx3(hb.from), crate::fx3(hb.to)))
        };
        let Some((hub, to)) = line(p) else {
            continue;
        };
        let to = match same_frame.then(|| line(&sim.prev.players[owner])).flatten() {
            Some((_, then)) => Vec3::from(view::math::turn_about(
                then.into(),
                to.into(),
                hub.into(),
                alpha,
            )),
            None => to,
        };
        emits.add(Emit {
            key: key(source::VOLUME, owner, 0),
            inner: hub.lerp(to, 0.4),
            outer: to,
            glow,
            tint: linear(class_tint(p.class)),
            window: SWING_WINDOW,
            style: Style::Blade,
        });
    }
}

/// The colour of a swing drawn off its volume, by who threw it: the
/// Reaver's dusk violet, her shadow's.
fn class_tint(class: sim::Class) -> Color {
    match class {
        sim::Class::ShadowReaver => Color::srgb(0.48, 0.40, 0.90),
        sim::Class::Bulwark => Color::srgb(1.0, 0.80, 0.45),
        _ => Color::srgb(0.85, 0.90, 1.0),
    }
}

/// **Lay every trail behind what it follows**, this picture: each emit is
/// added to the path of the strip that follows its key (a free one, if none
/// does yet), and the strip is that path. A strip whose thing said nothing
/// this picture -- a thrown blade gone, a swing over -- keeps its path and
/// wears away behind where the thing last was, and is let go once it has.
pub fn place(
    sim: Res<Sim>,
    mut emits: ResMut<Emits>,
    mut slots: ResMut<Slots>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut strips: Query<(&Strip, &mut Transform, &mut Visibility)>,
) {
    let alpha = sim.clock.alpha();
    let frame = sim.cur.frame;
    let count = std::mem::take(&mut emits.count);
    let list = &mut emits.list[..count];
    // Which strips were told where their thing is this picture.
    let mut told = [false; SLOTS];
    for (slot, told) in slots.0.iter_mut().zip(told.iter_mut()) {
        let Some(held) = slot.as_mut() else { continue };
        let key = held.last.key;
        if let Some(e) = list
            .iter_mut()
            .find(|e| e.is_some_and(|e| e.key == key))
            .and_then(Option::take)
        {
            held.last = e;
            *told = true;
        }
    }
    // A new trail takes a free strip, and starts clean.
    for e in list.iter_mut().filter_map(Option::take) {
        if let Some(i) = (0..SLOTS).find(|&i| slots.0[i].is_none()) {
            slots.0[i] = Some(Held {
                path: Path::default(),
                base: frame,
                last: e,
            });
            told[i] = true;
        }
    }
    for (strip, mut tf, mut vis) in strips.iter_mut() {
        let slot = &mut slots.0[strip.slot];
        let Some(held) = slot.as_mut() else {
            *vis = Visibility::Hidden;
            continue;
        };
        if frame < held.base {
            // Stepped back past where it began: it has not happened yet.
            *slot = None;
            *vis = Visibility::Hidden;
            continue;
        }
        let e = held.last;
        let now = (frame - held.base) as f32 + alpha;
        if told[strip.slot] {
            // A jump is a different thing under the same name (a slot reused):
            // its trail starts again rather than drawing a line between them.
            if held
                .path
                .samples()
                .last()
                .is_some_and(|s| Vec3::from(s.outer).distance(e.outer) > JUMP)
            {
                held.path.clear();
            }
            held.path.push(
                view::trail::Sample {
                    inner: e.inner.into(),
                    outer: e.outer.into(),
                    at: now,
                    glow: e.glow,
                },
                e.window,
            );
        } else if held
            .path
            .samples()
            .last()
            .is_none_or(|s| now - s.at >= e.window)
        {
            // Worn all the way away.
            *slot = None;
            *vis = Visibility::Hidden;
            continue;
        }
        let r = view::trail::ribbon(&held.path, now, e.window);
        if r.count < 2 {
            *vis = Visibility::Hidden;
            continue;
        }
        let Some(mesh) = meshes.get_mut(&strip.mesh) else {
            continue;
        };
        // Placed at its middle, so it is sorted among the other see-through
        // things by where it is rather than by the origin.
        let middle = r.outer[..r.count]
            .iter()
            .fold(Vec3::ZERO, |sum, p| sum + Vec3::from(*p))
            / r.count as f32;
        tf.translation = middle;
        let head = r.count - 1;
        if let Some(VertexAttributeValues::Float32x3(at)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            for k in 0..RIBBON {
                // Past the head, every column folds onto it: no area, no pixel.
                let c = k.min(head);
                at[2 * k] = (Vec3::from(r.inner[c]) - middle).into();
                at[2 * k + 1] = (Vec3::from(r.outer[c]) - middle).into();
            }
        }
        if let Some(VertexAttributeValues::Float32x2(uv)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            for k in 0..RIBBON {
                let life = r.life[k.min(head)];
                uv[2 * k] = [life, 0.0];
                uv[2 * k + 1] = [life, 1.0];
            }
        }
        if let Some(VertexAttributeValues::Float32x2(style)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_UV_1)
        {
            style.fill([e.style.code(), 0.0]);
        }
        if let Some(VertexAttributeValues::Float32x4(colour)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
        {
            let [red, green, blue] = e.tint;
            for k in 0..RIBBON {
                let glow = r.glow[k.min(head)];
                colour[2 * k] = [red, green, blue, glow];
                colour[2 * k + 1] = [red, green, blue, glow];
            }
        }
        *vis = Visibility::Inherited;
    }
}
