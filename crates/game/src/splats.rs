//! **The brush as a primitive**: the strokes of the land, placed once per
//! tile and drawn as quads, in place of every pixel searching a hundred
//! strokes ([`docs/design/exploration/0020`](../../../docs/design/exploration/0020_brush_splats.md)).
//!
//! Each tile of land within sight carries a [`Splats`]: one vertex buffer
//! of every stroke the brush lays on it at every length its distance can
//! ask for (`look::strokes::place`, the same strokes `paint.wgsl`'s
//! `strokes_exact` would find), four corners each, in painter's order --
//! coarsest level first, lowest order first -- so the topmost lands last.
//! A pass of its own, after the depth prepass and before the surface pass,
//! draws them all into two screen-sized textures ([`TARGET`], [`TARGET_ODD`])
//! with the original's inner loop as the fragment shader (`splats.wgsl`);
//! the surface shader reads the two at its pixel, adds them, and applies the
//! stroke as it always did. A fragment keeps itself only on its ground (the
//! prepass depth, read both ways: a wide stroke's quad on a slope cuts
//! through the air as much as under the ground) and only where its level
//! is in season at the fragment's own distance (a sixty-metre stroke's far
//! corner may be in season while its near end is three octaves out). That
//! season, `broad_strokes`' blend of the two nearest lengths, is what keeps
//! the brush the same size on the screen.
//!
//! What it costs: placing is hashing, a few milliseconds for the finest
//! tile; drawing is four or five fragments a pixel of one noise lookup
//! each (4.3 million at 1280 x 760, measured), which a GPU makes nothing
//! of and the software renderer, which charges per fragment, 260 ms of.

// `ShaderType` writes a per-field size assertion whose helper nothing calls,
// and the lint lands on the field rather than anywhere it could be allowed
// locally (the same note as `outline.rs`).
#![allow(dead_code)]

use std::collections::HashMap;

use bevy::asset::weak_handle;
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::core_pipeline::prepass::ViewPrepassTextures;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::diagnostic::RecordDiagnostics;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_asset::{RenderAssetUsages, RenderAssets};
use bevy::render::render_graph::{
    NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_resource::binding_types::uniform_buffer;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::view::{ViewUniform, ViewUniformOffset, ViewUniforms};
use bevy::render::{Render, RenderApp, RenderSet};
use bevy::window::PrimaryWindow;
use sim::atlas::{self, Atlas, TILE_M};

use crate::stream::Stream;

/// The screen-sized textures the strokes are drawn into and `paint.wgsl`
/// reads. Two, the even levels' and the odd levels': the two levels in
/// season at a pixel are always one of each, and the original *adds* its
/// two layers where within a layer the topmost stroke wins -- so each
/// layer replaces into its own texture, in painter's order, and the
/// surface shader adds the two. Each texel: the stroke's shade, warmth and
/// third axis, weighted by how far in season, at a quarter plus a half
/// (the weight reaches 1.4); the alpha is 1 where a stroke is.
pub const TARGET: Handle<Image> = weak_handle!("5f1a7b0c-7e1e-4a4e-9c1d-0b2a3f4e5d6c");
pub const TARGET_ODD: Handle<Image> = weak_handle!("5f1a7b0c-7e1e-4a4e-9c1d-0b2a3f4e5d6e");

const SHADER: Handle<Shader> = weak_handle!("5f1a7b0c-7e1e-4a4e-9c1d-0b2a3f4e5d6d");

/// Whether the strokes are drawn as splats: the painted looks, unless
/// `--exact` asks for the brush at the pixel (or the sheets).
pub fn on() -> bool {
    crate::wash::painted()
        && crate::platform::value("--exact").is_none()
        && crate::platform::draws_splats()
}

/// The finest level ever placed: quarter-metre strokes. Finer is what the
/// brush asks for inside two metres of the eye, and four times the strokes
/// a level; the quarter metre reads the same.
const FINEST: i32 = -2;
/// The coarsest: sixty-four-metre strokes, for the far mountains.
const COARSEST: i32 = 8;

/// A tile's strokes, on the GPU.
#[derive(Component, Clone, ExtractComponent)]
pub struct Splats {
    vertices: Buffer,
    indices: Buffer,
    count: u32,
}

/// What the pass needs of the frame: the map origin and the look.
#[derive(Resource, Clone, Copy, Default, ExtractResource, ShaderType)]
struct Frame {
    origin: Vec4,
    params: Vec4,
}

/// Which levels each tile has built, so a tile is rebuilt only when its
/// distance asks for a level it does not have.
#[derive(Resource, Default)]
struct Built(HashMap<(i32, i32), (Entity, i32, i32)>);

pub struct SplatsPlugin;

impl Plugin for SplatsPlugin {
    fn build(&self, app: &mut App) {
        if !on() {
            // The paint binds the targets whether or not anything draws
            // into them: a texel of no stroke each.
            app.add_systems(Startup, placeholders);
            return;
        }
        app.world_mut().resource_mut::<Assets<Shader>>().insert(
            SHADER.id(),
            Shader::from_wgsl(include_str!("splats.wgsl"), "game/src/splats.wgsl"),
        );
        app.add_plugins((
            ExtractComponentPlugin::<Splats>::default(),
            ExtractResourcePlugin::<Frame>::default(),
        ))
        .init_resource::<Frame>()
        .init_resource::<Built>()
        .add_systems(Update, (target, follow.after(crate::stream::stream)));
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<Batch>()
            .init_resource::<FrameBuffer>()
            .add_systems(Render, prepare.in_set(RenderSet::Prepare))
            .add_render_graph_node::<ViewNodeRunner<SplatsNode>>(Core3d, SplatsLabel)
            .add_render_graph_edges(
                Core3d,
                (Node3d::EndPrepasses, SplatsLabel, Node3d::StartMainPass),
            );
    }

    fn finish(&self, app: &mut App) {
        if !on() {
            return;
        }
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<SplatsPipeline>();
        }
    }
}

/// The targets with the splats off: one texel of no stroke each.
fn placeholders(mut images: ResMut<Assets<Image>>) {
    for handle in [TARGET, TARGET_ODD] {
        images.insert(handle.id(), blank(1, 1));
    }
}

fn blank(w: u32, h: u32) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[128, 128, 128, 0],
        TextureFormat::Rgba8Unorm,
        // Kept in the main world too, or it is gone the frame after it is
        // extracted and `target` would remake it every frame (which it did).
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST;
    image
}

/// The targets, the window's size: made, and remade when the window
/// changes. Every paint is touched when they are remade, so their bind
/// groups pick up the new textures.
fn target(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut paints: ResMut<Assets<crate::paint::Paint>>,
    mut remade: Local<bool>,
) {
    // The frame after a remake: a paint's bind group is built the same
    // frame it is marked changed, before the new texture is, so it would
    // keep the old one. Marked a frame later, it gets the new.
    if *remade {
        *remade = false;
        for (_, _) in paints.iter_mut() {}
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let (w, h) = (
        window.physical_width().max(1),
        window.physical_height().max(1),
    );
    if images
        .get(&TARGET)
        .is_some_and(|i| i.width() == w && i.height() == h)
    {
        return;
    }
    for handle in [TARGET, TARGET_ODD] {
        images.insert(handle.id(), blank(w, h));
    }
    *remade = true;
}

/// Place the strokes of every tile of land whose distance asks for a level
/// it has not got, and let the map's origin through to the pass. After
/// `stream::stream`, which owns the tiles.
fn follow(
    mut commands: Commands,
    sim: Res<crate::Sim>,
    st: Res<Stream>,
    mut built: ResMut<Built>,
    mut frame: ResMut<Frame>,
    camera: Query<&Transform, With<crate::MainCamera>>,
    device: Res<RenderDevice>,
) {
    let w = &sim.cur;
    if !w.valley.on {
        built.0.clear();
        return;
    }
    let Ok(eye) = camera.single() else {
        return;
    };
    let o = w.map_origin();
    let origin = Vec3::new(
        o.x.to_f32_for_render(),
        o.y.to_f32_for_render(),
        o.z.to_f32_for_render(),
    );
    let broad_size = crate::wash::style().broad_size;
    frame.origin = origin.extend(0.0);
    frame.params = Vec4::new(broad_size, 0.0, 0.0, 0.0);
    let at = eye.translation + origin;
    let size = TILE_M as f32;
    built.0.retain(|t, _| st.tiles().contains_key(t));
    let atlas = atlas::valley();
    for (&t, &(e, cells, _)) in st.tiles() {
        let (x0, z0) = (t.0 as f32 * size, t.1 as f32 * size);
        // The nearest and farthest points of the tile, flat.
        let dx = (x0 - at.x).max(at.x - x0 - size).max(0.0);
        let dz = (z0 - at.z).max(at.z - z0 - size).max(0.0);
        let near = (dx * dx + dz * dz).sqrt();
        let fx = (at.x - x0).abs().max((at.x - x0 - size).abs());
        let fz = (at.z - z0).abs().max((at.z - z0 - size).abs());
        let far = (fx * fx + fz * fz).sqrt();
        // The levels in season somewhere on the tile, for the kinds of
        // ground on it, and one coarser, so a step back does not rebuild.
        let (size_lo, size_hi) = Splats::sizes(atlas, t);
        let l_min = (broad_size * near * size_lo).max(0.05).log2();
        let l_max = (broad_size * far * size_hi).max(0.05).log2();
        let lo = (l_min.floor() as i32).clamp(FINEST, COARSEST);
        let hi = (l_max.ceil() as i32 + 1).clamp(FINEST, COARSEST);
        let have = built.0.get(&t).copied();
        if have.is_some_and(|(he, hlo, hhi)| he == e && hlo <= lo && hhi >= hi) {
            continue;
        }
        let splats = Splats::place(&device, atlas, t, cells, lo, hi);
        if let Ok(mut ec) = commands.get_entity(e) {
            ec.insert(splats);
            built.0.insert(t, (e, lo, hi));
        }
    }
}

impl Splats {
    /// The smallest and largest stroke size the ground of a tile asks for,
    /// read at sixteen points: which levels can be in season on it.
    fn sizes(atlas: &Atlas, t: (i32, i32)) -> (f32, f32) {
        let size = TILE_M as f32;
        let (x0, z0) = (t.0 as f32 * size, t.1 as f32 * size);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for j in 0..4 {
            for i in 0..4 {
                let x = x0 + (i as f32 + 0.5) * size / 4.0;
                let z = z0 + (j as f32 + 0.5) * size / 4.0;
                let (_, slope) = crate::land::ground_normal(atlas, x, z);
                let (_, kind) = crate::land::stroke_kind_at(atlas, x, z, slope);
                lo = lo.min(kind[1]);
                hi = hi.max(kind[1]);
            }
        }
        (lo, hi)
    }

    /// Every stroke of the tile at levels `lo..=hi`: four corners each,
    /// in painter's order.
    /// `cells` is how finely the tile's mesh is drawn: the strokes lie on
    /// that mesh, read between its vertices, not on the relief it samples,
    /// or far out, where a cell is metres, they would sink under it.
    fn place(
        device: &RenderDevice,
        atlas: &Atlas,
        t: (i32, i32),
        cells: usize,
        lo: i32,
        hi: i32,
    ) -> Splats {
        let size = TILE_M as f32;
        let (x0, z0) = (t.0 as f32 * size, t.1 as f32 * size);
        // The ground, on the mesh's own lattice: height, normal, the way
        // the strokes run, rigidity and size. Asked once a lattice point
        // rather than once a stroke.
        let step = size / cells.max(1) as f32;
        let n = cells.max(1) + 1;
        let lattice: Vec<[f32; 7]> = (0..n * n)
            .map(|k| {
                let (i, j) = (k % n, k / n);
                let x = x0 + i as f32 * step;
                let z = z0 + j as f32 * step;
                let (normal, slope) = crate::land::ground_normal(atlas, x, z);
                let (flow, kind) = crate::land::stroke_kind_at(atlas, x, z, slope);
                let h = atlas.relief_at(fx(x), fx(z)).to_f32_for_render();
                let way = look::strokes::way_on_ground([x, h, z], normal, flow, kind[0]);
                [h, normal[0], normal[2], way[0], way[2], kind[0], kind[1]]
            })
            .collect();
        let read = |x: f32, z: f32| -> ([f32; 5], [f32; 2]) {
            let u = ((x - x0) / step).clamp(0.0, n as f32 - 1.001);
            let v = ((z - z0) / step).clamp(0.0, n as f32 - 1.001);
            let (i, j) = (u as usize, v as usize);
            let (fu, fv) = (u - i as f32, v - j as f32);
            let (a, b, c, d) = (
                lattice[j * n + i],
                lattice[j * n + i + 1],
                lattice[(j + 1) * n + i],
                lattice[(j + 1) * n + i + 1],
            );
            let mut smooth = [0.0; 5];
            for k in 0..5 {
                let top = a[k] + (b[k] - a[k]) * fu;
                let bottom = c[k] + (d[k] - c[k]) * fu;
                smooth[k] = top + (bottom - top) * fv;
            }
            // The kind of ground is one thing or another: the nearest.
            let nearest = match (fu < 0.5, fv < 0.5) {
                (true, true) => a,
                (false, true) => b,
                (true, false) => c,
                (false, false) => d,
            };
            (smooth, [nearest[5], nearest[6]])
        };
        let kind = |x: f32, z: f32| {
            let (s, k) = read(x, z);
            [s[4].atan2(s[3]), k[0], k[1]]
        };
        let mut vertices: Vec<u8> = Vec::new();
        let mut indices: Vec<u8> = Vec::new();
        let mut strokes = 0u32;
        for level in lo..=hi {
            for s in look::strokes::place(level, [x0, z0], [x0 + size, z0 + size], &kind) {
                let (g, _) = read(s.centre[0], s.centre[1]);
                let (h, nx, nz) = (g[0], g[1], g[2]);
                let ny = (1.0 - nx * nx - nz * nz).max(1e-3).sqrt();
                let (sn, cs) = s.angle.sin_cos();
                let base = strokes * 4;
                strokes += 1;
                for (cu, cv) in [(-1.2, -1.8), (1.2, -1.8), (1.2, 1.8), (-1.2, 1.8)] {
                    let dx = cu * s.long * cs - cv * s.wide * sn;
                    let dz = cu * s.long * sn + cv * s.wide * cs;
                    // On the ground's plane at the stroke's middle.
                    let y = h - (nx * dx + nz * dz) / ny;
                    for v in [
                        s.centre[0] + dx,
                        y,
                        s.centre[1] + dz,
                        cu,
                        cv,
                        s.order,
                        level as f32,
                        s.scale,
                        s.loose,
                        s.colour[0],
                        s.colour[1],
                        s.colour[2],
                        s.curve,
                        s.seeds[0],
                        s.seeds[1],
                        s.long,
                        0.0,
                    ] {
                        vertices.extend_from_slice(&v.to_le_bytes());
                    }
                }
                for i in [base, base + 1, base + 2, base, base + 2, base + 3] {
                    indices.extend_from_slice(&i.to_le_bytes());
                }
            }
        }
        let vertices = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("splats vertices"),
            contents: &vertices,
            usage: BufferUsages::VERTEX,
        });
        let count = strokes * 6;
        let indices = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("splats indices"),
            contents: &indices,
            usage: BufferUsages::INDEX,
        });
        Splats {
            vertices,
            indices,
            count,
        }
    }
}

fn fx(v: f32) -> sim::Fx {
    sim::Fx::from_raw((v * 65536.0) as i32)
}

// ---- The render world ----

/// Every tile's splats this frame.
#[derive(Resource, Default)]
struct Batch(Vec<Splats>);

#[derive(Resource, Default)]
struct FrameBuffer(UniformBuffer<Frame>);

fn prepare(
    splats: Query<&Splats>,
    frame: Res<Frame>,
    mut batch: ResMut<Batch>,
    mut buffer: ResMut<FrameBuffer>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    batch.0.clear();
    batch.0.extend(splats.iter().cloned());
    buffer.0.set(*frame);
    buffer.0.write_buffer(&device, &queue);
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct SplatsLabel;

#[derive(Resource)]
struct SplatsPipeline {
    layout: BindGroupLayout,
    id: CachedRenderPipelineId,
}

impl FromWorld for SplatsPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let layout = device.create_bind_group_layout(
            Some("splats bind group layout"),
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<Frame>(false),
                    bevy::render::render_resource::binding_types::texture_depth_2d(),
                ),
            ),
        );
        let descriptor = RenderPipelineDescriptor {
            label: Some("splats".into()),
            layout: vec![layout.clone()],
            vertex: VertexState {
                shader: SHADER,
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![VertexBufferLayout::from_vertex_formats(
                    VertexStepMode::Vertex,
                    [
                        VertexFormat::Float32x3,
                        VertexFormat::Float32x2,
                        VertexFormat::Float32x4,
                        VertexFormat::Float32x4,
                        VertexFormat::Float32x4,
                    ],
                )],
            },
            fragment: Some(FragmentState {
                shader: SHADER,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                // A stroke writes alpha 1 to its own level's texture and 0
                // to the other's: replaced there, left alone here.
                targets: vec![
                    Some(ColorTargetState {
                        format: TextureFormat::Rgba8Unorm,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    }),
                    Some(ColorTargetState {
                        format: TextureFormat::Rgba8Unorm,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    }),
                ],
            }),
            primitive: PrimitiveState {
                cull_mode: None,
                ..default()
            },
            // No depth test here: the fragment shader reads the prepass
            // depth and keeps only what lies *on* the surface, both ways,
            // since a wide stroke's flat quad on a slope cuts through the
            // air in front of it as much as under it.
            depth_stencil: None,
            multisample: default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        };
        let id = world
            .resource_mut::<PipelineCache>()
            .queue_render_pipeline(descriptor);
        SplatsPipeline { layout, id }
    }
}

#[derive(Default)]
struct SplatsNode;

impl ViewNode for SplatsNode {
    type ViewQuery = (&'static ViewUniformOffset, &'static ViewPrepassTextures);

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_offset, prepass): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<SplatsPipeline>();
        let Some(ready) = world
            .resource::<PipelineCache>()
            .get_render_pipeline(pipeline.id)
        else {
            return Ok(());
        };
        let Some(view_binding) = world.resource::<ViewUniforms>().uniforms.binding() else {
            return Ok(());
        };
        let Some(frame_binding) = world.resource::<FrameBuffer>().0.binding() else {
            return Ok(());
        };
        let images = world.resource::<RenderAssets<GpuImage>>();
        let (Some(target), Some(target_odd)) = (images.get(&TARGET), images.get(&TARGET_ODD))
        else {
            return Ok(());
        };
        let Some(depth) = prepass.depth.as_ref() else {
            return Ok(());
        };
        let bind = render_context.render_device().create_bind_group(
            "splats bind group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                view_binding,
                frame_binding,
                &depth.texture.default_view,
            )),
        );
        let batch = world.resource::<Batch>();
        let diagnostics = render_context.diagnostic_recorder();
        let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("splats"),
            color_attachments: &[
                Some(RenderPassColorAttachment {
                    view: &target.texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(LinearRgba::new(0.5, 0.5, 0.5, 0.0).into()),
                        store: StoreOp::Store,
                    },
                }),
                Some(RenderPassColorAttachment {
                    view: &target_odd.texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(LinearRgba::new(0.5, 0.5, 0.5, 0.0).into()),
                        store: StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        let span = diagnostics.pass_span(&mut pass, "splats");
        pass.set_render_pipeline(ready);
        pass.set_bind_group(0, &bind, &[view_offset.offset]);
        for s in &batch.0 {
            pass.set_vertex_buffer(0, s.vertices.slice(..));
            pass.set_index_buffer(s.indices.slice(..), 0, IndexFormat::Uint32);
            pass.draw_indexed(0..s.count, 0, 0..1);
        }
        span.end(&mut pass);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    /// A look at what is placed: a near tile's metre strokes and a far
    /// tile's sixty-four-metre ones.
    #[test]
    fn placed_strokes_are_sane() {
        let atlas = sim::atlas::valley();
        let kind = |x: f32, z: f32| {
            let (n, slope) = crate::land::ground_normal(atlas, x, z);
            let (flow, k) = crate::land::stroke_kind_at(atlas, x, z, slope);
            let way = look::strokes::way_on_ground([x, 0.0, z], n, flow, k[0]);
            [way[2].atan2(way[0]), k[0], k[1]]
        };
        for (level, t) in [(0, (3, 0)), (7, (20, 0))] {
            let size = sim::atlas::TILE_M as f32;
            let (x0, z0) = (t.0 as f32 * size, t.1 as f32 * size);
            let strokes = look::strokes::place(level, [x0, z0], [x0 + size, z0 + size], &kind);
            eprintln!("level {level} tile {t:?}: {} strokes", strokes.len());
            for s in strokes.iter().take(4) {
                eprintln!("  {s:?}");
            }
            assert!(
                strokes
                    .iter()
                    .all(|s| s.colour.iter().all(|c| c.is_finite() && c.abs() <= 1.0))
            );
            assert!(
                strokes
                    .iter()
                    .all(|s| s.long.is_finite() && s.angle.is_finite())
            );
        }
    }
}
