//! **The watercolour**, the Bevy half of `look::wash`.
//!
//! Two things, both reading the one treatment chosen for the run
//! (`--look <name>`, or `?look=` on the web):
//!
//! - the surface half is `paint.wgsl`'s, handed in as shader defs by
//!   [`surface_defs`] when the paint's pipeline is built;
//! - the paper half is two passes over the finished picture, `wash.wgsl`:
//!   **brush** after tonemapping and before the line (the wet pigment), and
//!   **paper** after the line and before the antialiasing (the sheet it
//!   dried on).
//!
//! Both read the depth prepass the outline already asks for, so where there
//! is none -- the browser build, whose WebGL2 cannot sample depth -- there is
//! no treatment either, and the picture is the plain one.
//!
//! Nothing here decides a number. `look::wash` does.

#![allow(dead_code)]

use std::sync::OnceLock;

use bevy::asset::weak_handle;
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
use bevy::core_pipeline::prepass::ViewPrepassTextures;
use bevy::ecs::query::QueryItem;
use bevy::image::BevyDefault;
use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::extract_component::{
    ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
    UniformComponentPlugin,
};
use bevy::render::render_graph::{
    NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_resource::binding_types::{
    sampler, texture_2d, texture_depth_2d, uniform_buffer,
};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::view::ViewTarget;

const SHADER: Handle<Shader> = weak_handle!("b4d1c0e7-3a5f-4e92-8c61-0f7e2d9a5b38");

/// The treatment this run is painted in: `plain` -- the picture as it was
/// before any of this -- unless `--look` names another. The painted looks are
/// opt-in until they are cheap enough to be the default: `brushed` as the
/// default cost the owner the game (fan, a walk at a crawl, a class change
/// that froze), 2026-10-16.
pub fn style() -> look::Wash {
    static STYLE: OnceLock<look::Wash> = OnceLock::new();
    *STYLE.get_or_init(|| {
        crate::platform::value("--look")
            .and_then(look::wash::named)
            .unwrap_or(look::wash::PLAIN)
    })
}

/// Is the picture painted at all? False for `plain`, and then nothing here
/// runs: no passes, no paper, no strokes or bends laid into a mesh.
pub fn painted() -> bool {
    style() != look::wash::PLAIN
}

/// The surface half, as the shader defs `paint.wgsl` reads: thousandths.
pub fn surface_defs() -> Vec<ShaderDefVal> {
    let w = style();
    let milli = |name: &str, v: f32| ShaderDefVal::Int(name.into(), (v * 1000.0).round() as i32);
    vec![
        milli("WASH_BANDS", w.bands),
        milli("WASH_TONES", w.tones),
        milli("WASH_SOFT", w.soft),
        milli("WASH_STROKE", w.stroke),
        milli("WASH_STROKE_LEN", w.stroke_len),
        milli("WASH_SHADE_R", w.shade[0]),
        milli("WASH_SHADE_G", w.shade[1]),
        milli("WASH_SHADE_B", w.shade[2]),
        milli("WASH_LIGHT_R", w.light[0]),
        milli("WASH_LIGHT_G", w.light[1]),
        milli("WASH_LIGHT_B", w.light[2]),
        milli("WASH_MOTTLE", w.mottle),
        milli("WASH_DILUTE", w.dilute),
        milli("WASH_TOP", w.top),
        milli("WASH_GRAIN", w.grain),
        milli("WASH_BROAD", w.broad),
        milli("WASH_BROAD_SIZE", w.broad_size),
        milli("WASH_RICH", w.rich),
        milli("WASH_HOLLOW_R", w.hollow[0]),
        milli("WASH_HOLLOW_G", w.hollow[1]),
        milli("WASH_HOLLOW_B", w.hollow[2]),
        milli("WASH_CREST_R", w.crest[0]),
        milli("WASH_CREST_G", w.crest[1]),
        milli("WASH_CREST_B", w.crest[2]),
    ]
}

/// The air and paper halves' numbers, and where the camera and the sun are
/// this frame. Put on the camera; [`follow`] keeps the last three current.
///
/// Every field a `Vec4`, so the layout is the same on both sides with no
/// padding to get wrong.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct Wash {
    /// brush, drawn, haze, haze_falloff
    pub a: Vec4,
    /// haze tint, haze tint amount
    pub b: Vec4,
    /// near_dark, ao, glow, shafts
    pub c: Vec4,
    /// grade shade, saturation
    pub d: Vec4,
    /// grade light, lift
    pub e: Vec4,
    /// dark, key
    pub f: Vec4,
    /// on paper, pool, bleed, granulate
    pub g: Vec4,
    /// dry, distort, relief, turbulence
    pub h: Vec4,
    /// paper
    pub paper: Vec4,
    /// The camera's height, and the height each of its axes gains per unit
    /// along it: right, up, back.
    pub cam: Vec4,
    /// tan of half the vertical field of view, aspect, near plane, and the
    /// distance past which a pixel is the sky.
    pub lens: Vec4,
    /// The sun on screen: x, y in normalised device coordinates, and how far
    /// it is in front (0 behind).
    pub sun: Vec4,
    /// edges
    pub i: Vec4,
    /// crisp until, painted from (metres), broad strokes, frame
    pub j: Vec4,
    /// The camera's axes in the world: what the sky's strokes are laid on.
    pub right: Vec4,
    pub up: Vec4,
    pub back: Vec4,
}

impl Wash {
    /// A treatment's numbers, or nothing if it is the plain one.
    pub fn of(w: look::Wash) -> Option<Wash> {
        if w == look::wash::PLAIN {
            return None;
        }
        let v3 = |c: [f32; 3], w: f32| Vec4::new(c[0], c[1], c[2], w);
        Some(Wash {
            a: Vec4::new(w.brush, w.drawn, w.haze, w.haze_falloff),
            b: v3(w.haze_tint, w.haze_tint_amount),
            c: Vec4::new(w.near_dark, w.ao, w.glow, w.shafts),
            d: v3(w.grade_shade, w.saturation),
            e: v3(w.grade_light, w.lift),
            f: v3(w.dark, w.key),
            g: Vec4::new(
                if w.on_paper { 1.0 } else { 0.0 },
                w.pool,
                w.bleed,
                w.granulate,
            ),
            h: Vec4::new(w.dry, w.distort, w.relief, w.turbulence),
            paper: v3(w.paper, 1.0),
            cam: Vec4::ZERO,
            lens: Vec4::new(0.5, 1.6, 0.1, 450.0),
            sun: Vec4::ZERO,
            i: Vec4::new(w.edges, 0.0, 0.0, 0.0),
            j: Vec4::new(w.crisp, w.painted_from, w.strokes, w.frame),
            right: Vec4::X,
            up: Vec4::Y,
            back: Vec4::Z,
        })
    }
}

/// Keeps the camera's height, axes and lens, and the sun's place on screen,
/// current in the camera's [`Wash`].
pub fn follow(
    mut cams: Query<(&Camera, &GlobalTransform, &Projection, &mut Wash)>,
    sun: Query<&GlobalTransform, With<crate::arenas::Sun>>,
) {
    for (camera, at, projection, mut wash) in &mut cams {
        let (right, up, back) = (at.right(), at.up(), at.back());
        wash.cam = Vec4::new(at.translation().y, right.y, up.y, back.y);
        wash.right = right.extend(0.0);
        wash.up = up.extend(0.0);
        wash.back = back.extend(0.0);
        if let Projection::Perspective(p) = projection {
            // Past this along a ray is the sky's dome (`sky::RADIUS`, 480 m).
            wash.lens = Vec4::new((p.fov * 0.5).tan(), p.aspect_ratio, p.near, 450.0);
        }
        wash.sun = Vec4::ZERO;
        if let Ok(light) = sun.single() {
            // A directional light shines along its forward axis; the sun is
            // the other way, far off.
            let far = at.translation() - *light.forward() * 10_000.0;
            if let Some(ndc) = camera.world_to_ndc(at, far) {
                let ahead = (-*light.forward()).dot(*at.forward());
                if ahead > 0.0 {
                    wash.sun = Vec4::new(ndc.x, ndc.y, ahead, 0.0);
                }
            }
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct BrushLabel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PaperLabel;

/// The two passes, placed round the line if there is one.
pub struct WashPlugin {
    /// Whether the outline pass is in the graph to be placed round.
    pub outlines: bool,
}

impl Plugin for WashPlugin {
    fn build(&self, app: &mut App) {
        if !painted() {
            return;
        }
        app.world_mut().resource_mut::<Assets<Shader>>().insert(
            SHADER.id(),
            Shader::from_wgsl(include_str!("wash.wgsl"), "game/src/wash.wgsl"),
        );
        app.add_plugins((
            ExtractComponentPlugin::<Wash>::default(),
            UniformComponentPlugin::<Wash>::default(),
        ));
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_render_graph_node::<ViewNodeRunner<WashNode<false>>>(Core3d, BrushLabel)
            .add_render_graph_node::<ViewNodeRunner<WashNode<true>>>(Core3d, PaperLabel);
        if self.outlines {
            render_app.add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    BrushLabel,
                    crate::outline::OutlineLabel,
                    PaperLabel,
                    Node3d::Fxaa,
                ),
            );
        } else {
            render_app.add_render_graph_edges(
                Core3d,
                (Node3d::Tonemapping, BrushLabel, PaperLabel, Node3d::Fxaa),
            );
        }
    }

    fn finish(&self, app: &mut App) {
        if !painted() {
            return;
        }
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<WashPipeline>();
        }
    }
}

#[derive(Resource)]
struct WashPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    /// The paper: MNPR's `rough_default` substrate scan (normal in red and
    /// green, height in blue), tiled across the screen.
    sheet: TextureView,
    sheet_sampler: Sampler,
    /// Depth is read point by point, never filtered.
    depth_sampler: Sampler,
    brush: CachedRenderPipelineId,
    paper: CachedRenderPipelineId,
}

impl FromWorld for WashPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let layout = device.create_bind_group_layout(
            Some("wash bind group layout"),
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<Wash>(true),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    texture_depth_2d(),
                    sampler(SamplerBindingType::NonFiltering),
                ),
            ),
        );
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("wash sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            ..default()
        });
        let sheet = {
            let image = Image::from_buffer(
                include_bytes!("paper.jpg"),
                bevy::image::ImageType::Extension("jpg"),
                bevy::image::CompressedImageFormats::NONE,
                false,
                bevy::image::ImageSampler::Default,
                bevy::asset::RenderAssetUsages::RENDER_WORLD,
            )
            .expect("the paper scan is a jpeg compiled into the game");
            let texture = device.create_texture_with_data(
                world.resource::<RenderQueue>(),
                &image.texture_descriptor,
                bevy::render::render_resource::TextureDataOrder::LayerMajor,
                image.data.as_deref().unwrap_or(&[]),
            );
            texture.create_view(&TextureViewDescriptor::default())
        };
        let sheet_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("paper sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            ..default()
        });
        let depth_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("wash depth sampler"),
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..default()
        });
        let queue = |entry: &'static str| RenderPipelineDescriptor {
            label: Some(format!("wash {entry}").into()),
            layout: vec![layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: SHADER,
                shader_defs: vec![],
                entry_point: entry.into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: default(),
            depth_stencil: None,
            multisample: default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        };
        let (brush, paper) = (queue("brush"), queue("paper"));
        let cache = world.resource_mut::<PipelineCache>();
        WashPipeline {
            brush: cache.queue_render_pipeline(brush),
            paper: cache.queue_render_pipeline(paper),
            layout,
            sampler,
            sheet,
            sheet_sampler,
            depth_sampler,
        }
    }
}

/// One of the two passes: the brush (`false`) or the paper (`true`).
#[derive(Default)]
struct WashNode<const PAPER: bool>;

impl<const PAPER: bool> ViewNode for WashNode<PAPER> {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewPrepassTextures,
        &'static DynamicUniformIndex<Wash>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (target, prepass, offset): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<WashPipeline>();
        let id = if PAPER {
            pipeline.paper
        } else {
            pipeline.brush
        };
        let Some(ready) = world.resource::<PipelineCache>().get_render_pipeline(id) else {
            return Ok(());
        };
        let Some(binding) = world
            .resource::<ComponentUniforms<Wash>>()
            .uniforms()
            .binding()
        else {
            return Ok(());
        };
        let Some(depth) = prepass.depth_view() else {
            return Ok(());
        };
        let post = target.post_process_write();
        let bind = render_context.render_device().create_bind_group(
            "wash bind group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                post.source,
                &pipeline.sampler,
                binding.clone(),
                &pipeline.sheet,
                &pipeline.sheet_sampler,
                depth,
                &pipeline.depth_sampler,
            )),
        );
        let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some(if PAPER { "wash paper" } else { "wash brush" }),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_render_pipeline(ready);
        pass.set_bind_group(0, &bind, &[offset.index()]);
        pass.draw(0..3, 0..1);
        Ok(())
    }
}
