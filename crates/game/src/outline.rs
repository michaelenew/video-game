//! The line round everything, as a pass over the finished picture.
//!
//! ## Why this is not geometry
//!
//! The first version of this drew each thing a second time, swollen a few
//! centimetres and turned inside out, so a rim of its back faces showed round
//! the outside. That is cheap and it works in any renderer, and it has one
//! problem that no amount of tuning fixes: **the width is set in metres and
//! read in pixels.** A face seen nearly edge on projects a fixed world-space
//! offset into a wide band; the same face turned to look straight at you
//! projects it into a hairline. So a single edge of a single box goes from
//! thick to thin along its length, which reads as a mistake rather than as a
//! drawing. Corners are worse: three faces each push out along their own
//! normal and meet in a mitre, which comes to a point and catches the eye at
//! exactly the place a drawn line would be calmest.
//!
//! A line wants to be defined where lines live, which is the screen. So: for
//! each pixel, look at every pixel inside a disc of the line's radius, and if
//! any of them is nearer than this one by more than a margin, this pixel is
//! just outside somebody's silhouette and becomes ink.
//!
//! Both properties fall straight out of saying it that way. The width is the
//! disc's radius **in pixels**, the same everywhere in the frame at any angle
//! and any distance. And the shape of a corner is the shape of the disc, so
//! every join and every end is round, which is what stroking a path with a
//! round cap means and what the mitres were failing to be.
//!
//! The test is one-sided -- *is something nearer* -- so the line is laid
//! outside what it outlines and never eats into it.
//!
//! ## What it costs, and what it still cannot do
//!
//! One full-screen pass over the depth buffer the main pass already wrote, with
//! no prepass of its own; a disc of radius two is thirteen samples a pixel.
//!
//! It finds **silhouettes**, because depth is all it looks at: where one thing
//! stands in front of another. It does not find the crease where two faces of
//! the same box meet, which a normal buffer would give and which would cost a
//! prepass that the browser build is the least likely to agree with. An
//! illustrator outlines silhouettes and shades creases, so this is the half
//! worth having.
//!
//! The rule for what colour a line is lives in `look::edge::Line`; the shader
//! is the same arithmetic in WGSL, down to being measured against the lit
//! colour rather than the albedo -- which it gets for free, because by the time
//! this pass runs the picture is finished.

// `ShaderType` writes a per-field size assertion whose helper nothing calls,
// and the lint lands on the field rather than anywhere it could be allowed
// locally. One small module, so the allow is scoped to it.
#![allow(dead_code)]

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
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::view::ViewTarget;

const SHADER: Handle<Shader> = weak_handle!("2f9b1a64-6c3d-47e1-9d5a-7a1c3e8b4d20");

/// How the line is drawn. Put on the camera.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct Outline {
    /// Half the line's width, in pixels.
    pub radius: f32,
    /// How much nearer another pixel has to be, as a fraction of this one's
    /// distance, before it counts as a different thing.
    ///
    /// Relative rather than absolute: an absolute margin either misses every
    /// silhouette in the distance or turns a floor seen at a shallow angle into
    /// one solid wash of ink.
    pub step: f32,
    /// How much darker the line is than what it outlines, in Oklab lightness.
    pub ink: f32,
    /// What its chroma is multiplied by, and the least it may have.
    pub chroma: f32,
    pub floor: f32,
    /// How far from flat a surface has to bend before the fold between its
    /// faces gets a line of its own.
    ///
    /// Depth alone can find a crease, which is worth knowing because the
    /// alternative is a normal buffer and a second prepass. Across a flat
    /// surface, however steeply it is turned away, distance changes linearly --
    /// the pixel halfway between two others is at the average of their
    /// distances. Where two faces of a box meet it is not.
    pub fold: f32,
    /// Metres per unit of depth ratio: the camera's near plane.
    pub near: f32,
    /// Where the line starts fading out, and where it is gone, in metres.
    ///
    /// Ink does not haze. Everything else in the arena fades into the horizon's
    /// colour as it recedes, and a line that does not is a crisp stroke drawn
    /// across the one part of the picture whose job is to dissolve -- the far
    /// rim of a jump course's drop plane came out as a hard brown horizon. So
    /// it is wound down over the same distance the fog is.
    pub haze: f32,
    pub gone: f32,
    pub pad: Vec3,
}

impl Outline {
    /// The line `look::edge::LINE` describes, in pixels.
    ///
    /// The width is the one number that could not come from `look`: that crate
    /// knows nothing about a window, and a line's width in metres is exactly
    /// the thing this file exists to stop mattering.
    pub fn from(line: look::edge::Line) -> Outline {
        Outline::over(line, &look::skies::of(sim::arena::ArenaId::PROVING_GROUND))
    }

    /// The same, wound down over the distance this arena's air reaches.
    pub fn over(line: look::edge::Line, sky: &look::Sky) -> Outline {
        let (start, end) = sky.fog();
        Outline {
            haze: start,
            gone: end,
            ..Outline::plain(line)
        }
    }

    fn plain(line: look::edge::Line) -> Outline {
        Outline {
            radius: 1.6,
            step: 0.035,
            ink: line.ink + 0.10,
            chroma: 2.6,
            floor: 0.075,
            fold: 0.0025,
            near: 0.1,
            haze: 200.0,
            gone: 420.0,
            pad: Vec3::ZERO,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct OutlineLabel;

pub struct OutlinePlugin;

impl OutlinePlugin {
    /// The same plugin, or nothing at all.
    ///
    /// Not a flag inside the plugin: the pipeline is built when the plugin is
    /// added, whether or not any camera ever asks for the pass, and on a
    /// backend that cannot read a depth texture *building* it is the failure.
    pub fn only_if(self, wanted: bool) -> OutlineMaybe {
        OutlineMaybe(wanted.then_some(self))
    }
}

/// [`OutlinePlugin`], or nothing.
pub struct OutlineMaybe(Option<OutlinePlugin>);

impl Plugin for OutlineMaybe {
    fn build(&self, app: &mut App) {
        if let Some(it) = &self.0 {
            it.build(app);
        }
    }

    fn finish(&self, app: &mut App) {
        if let Some(it) = &self.0 {
            it.finish(app);
        }
    }
}

impl Plugin for OutlinePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<Assets<Shader>>().insert(
            SHADER.id(),
            Shader::from_wgsl(include_str!("outline.wgsl"), "game/src/outline.wgsl"),
        );
        app.add_plugins((
            ExtractComponentPlugin::<Outline>::default(),
            UniformComponentPlugin::<Outline>::default(),
        ));
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_render_graph_node::<ViewNodeRunner<OutlineNode>>(Core3d, OutlineLabel)
            // After tonemapping, so what the line is drawn over is the finished
            // colour -- which is what its own rule is measured against -- and
            // before the user interface, which is a drawing already.
            .add_render_graph_edges(
                Core3d,
                // Before the antialiasing, so the line gets smoothed with
                // everything else rather than arriving as a hard stair.
                (Node3d::Tonemapping, OutlineLabel, Node3d::Fxaa),
            );
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<OutlinePipeline>();
            let device = render_app.world().resource::<RenderDevice>();
            let screen = device.create_sampler(&SamplerDescriptor::default());
            let depth = device.create_sampler(&SamplerDescriptor {
                label: Some("outline depth sampler"),
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                ..default()
            });
            render_app.insert_resource(OutlineSamplers { screen, depth });
        }
    }
}

#[derive(Resource)]
struct OutlinePipeline {
    layout: BindGroupLayout,
    id: CachedRenderPipelineId,
}

/// The two samplers the pass binds: one for the picture and one for the depth,
/// which must not filter.
#[derive(Resource)]
struct OutlineSamplers {
    screen: Sampler,
    depth: Sampler,
}

impl FromWorld for OutlinePipeline {
    fn from_world(world: &mut World) -> Self {
        let layout = world.resource::<RenderDevice>().create_bind_group_layout(
            Some("outline bind group layout"),
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    texture_depth_2d(),
                    uniform_buffer::<Outline>(true),
                    sampler(SamplerBindingType::NonFiltering),
                ),
            ),
        );
        let id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("outline".into()),
                    layout: vec![layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader: SHADER,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
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
                });
        OutlinePipeline { layout, id }
    }
}

#[derive(Default)]
struct OutlineNode;

impl ViewNode for OutlineNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewPrepassTextures,
        &'static DynamicUniformIndex<Outline>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (target, prepass, offset): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<OutlinePipeline>();
        let cache = world.resource::<PipelineCache>();
        let Some(ready) = cache.get_render_pipeline(pipeline.id) else {
            return Ok(());
        };
        let Some(binding) = world
            .resource::<ComponentUniforms<Outline>>()
            .uniforms()
            .binding()
        else {
            return Ok(());
        };

        // The depth **prepass**, not the main pass's own depth buffer: that one
        // is a render attachment and nothing else, and a texture has to be
        // created wanting to be read before it can be. A depth prepass is one
        // extra pass over the geometry with no shading in it, which is the
        // cheapest thing a frame can be asked to draw.
        let Some(depth) = prepass.depth_view() else {
            return Ok(());
        };

        // Ping-pong: read what the frame looks like so far, write the version
        // with the line on it.
        let post = target.post_process_write();
        let bind = render_context.render_device().create_bind_group(
            "outline bind group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                post.source,
                &world.resource::<OutlineSamplers>().screen,
                depth,
                binding.clone(),
                &world.resource::<OutlineSamplers>().depth,
            )),
        );
        let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("outline"),
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
