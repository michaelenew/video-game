//! **Her shadow, in ink** (`docs/design/exploration/0024_a_character_done_properly.md`).
//!
//! The Reaver's second body -- the shadow she sends, the five of her the
//! Reaping throws -- is her own model drawn in a material of its own: the
//! costume's light and shade kept, but sunk to a violet near-black, with a
//! rim of her violet light round its silhouette and the edge it is seen
//! side-on at breaking up into smoke that drifts upward. It is her, made of
//! shadow: the same shape as the real one, so which body is which is read by
//! its substance, not its outline.
//!
//! The smoke moves on the renderer's clock, not the simulation's. That is
//! the one thing here allowed to: it is texture on a surface whose every
//! place and pose comes from the snapshot, so a rollback can never make it
//! pop -- it only keeps drifting.

use bevy::asset::weak_handle;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

const INK_SHADER: Handle<Shader> = weak_handle!("5c1f9e72-4b8a-4d36-9e05-2a7c8d1b6f43");
const INK_PREPASS: Handle<Shader> = weak_handle!("8d2e6a14-7f3b-4c91-a5d8-3e9b1c6f2a07");
const INK_COMMON: Handle<Shader> = weak_handle!("1a7c4e93-2d6f-4b85-9c30-6e1f8a4d5b29");

pub type InkMaterial = ExtendedMaterial<StandardMaterial, Ink>;

/// What the ink adds to the standard material: its colours and how much of
/// it is there.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct Ink {
    #[uniform(100)]
    pub look: InkLook,
}

pub use look::InkLook;

/// The uniform on its own, because `ShaderType`'s derive writes a layout
/// check per field beside it that nothing calls (as `still::Shade`).
mod look {
    #![allow(dead_code)]
    use bevy::prelude::*;
    use bevy::render::render_resource::ShaderType;

    #[derive(ShaderType, Reflect, Debug, Clone, Copy)]
    pub struct InkLook {
        /// The body's colour, and how solid it is (alpha) at full strength.
        pub body: Vec4,
        /// The rim's colour, and how bright it is, in the emissive's units.
        pub glow: Vec4,
        /// How much of her there is, 0 to 1 (a copy fading out); how far
        /// the smoke eats into her silhouette; and how much of the light
        /// behind her comes through where she is thin.
        pub fade: Vec4,
    }
}

impl Ink {
    /// Her shadow: a smoky grey, half there across her middle and solid
    /// at her edges, its rim her violet gone toward grey -- the owner
    /// (2026-10-20): her the purple one, her shadow the grey.
    pub fn shadow() -> Ink {
        Ink {
            look: InkLook {
                body: Vec4::new(0.016, 0.014, 0.020, 0.58),
                glow: Vec4::new(0.50, 0.38, 0.72, 1100.0),
                fade: Vec4::new(1.0, 1.0, 0.55, 0.0),
            },
        }
    }
}

impl MaterialExtension for Ink {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(INK_SHADER)
    }

    /// Her holes in the depth too (`ink_prepass.wgsl`).
    fn prepass_fragment_shader() -> ShaderRef {
        ShaderRef::Handle(INK_PREPASS)
    }
}

/// Ink over a standard material read from the model: its texture is what
/// keeps the costume legible inside the dark.
///
/// **See-through, not blended.** Blended, her body showed through her
/// clothes and her far arm through her near one, because a blended mesh
/// does not write depth. Instead she reads the view's transmission
/// texture -- a copy of everything solid, taken before she is drawn -- so
/// Bevy draws her after it with depth written as usual: the nearest of her
/// surfaces wins, and it mixes itself with the world behind (`ink.wgsl`).
/// The specular transmission is what puts her in that pass; the shader
/// turns its own light back off. Masked, so the smoke's holes cut her
/// shadow on the floor too (`ink_prepass.wgsl`).
pub fn ink(mut base: StandardMaterial, ink: Ink) -> InkMaterial {
    base.alpha_mode = AlphaMode::Mask(0.5);
    base.specular_transmission = 0.01;
    base.double_sided = false;
    InkMaterial {
        base,
        extension: ink,
    }
}

pub struct InkPlugin;

impl Plugin for InkPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
        shaders.insert(
            INK_COMMON.id(),
            Shader::from_wgsl(include_str!("ink_common.wgsl"), "game/src/ink_common.wgsl"),
        );
        shaders.insert(
            INK_SHADER.id(),
            Shader::from_wgsl(include_str!("ink.wgsl"), "game/src/ink.wgsl"),
        );
        shaders.insert(
            INK_PREPASS.id(),
            Shader::from_wgsl(
                include_str!("ink_prepass.wgsl"),
                "game/src/ink_prepass.wgsl",
            ),
        );
        app.add_plugins(MaterialPlugin::<InkMaterial>::default());
    }
}
