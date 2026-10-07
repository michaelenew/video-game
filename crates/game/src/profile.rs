//! A frame-cost readout, on request: `--profile`.
//!
//! Every two seconds, to the log: the frame time, how many entities there
//! are and how many of the meshes are actually on screen, and -- where the
//! driver can time its own work -- what each render pass cost the GPU. It is
//! what turns "the game feels heavy" into a list of passes with numbers
//! beside them, and it is off unless asked for because the readout itself
//! is not free.

use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy::render::diagnostic::RenderDiagnosticsPlugin;
use bevy::render::view::ViewVisibility;

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        if !crate::platform::flag("--profile") {
            return;
        }
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin,
            RenderDiagnosticsPlugin,
        ))
        .add_systems(Startup, experiment.after(crate::setup))
        .add_systems(Update, census);
    }
}

/// How many meshes there are, and how many the camera can see, every two
/// seconds. Pools are fixed and mostly hidden, so the two numbers are far
/// apart on purpose; what matters is whether the second one is reasonable.
fn census(
    time: Res<Time>,
    mut last: Local<f32>,
    store: Res<DiagnosticsStore>,
    meshes: Query<(&ViewVisibility, &Visibility), With<Mesh3d>>,
    casters: Query<&ViewVisibility, (With<Mesh3d>, Without<bevy::pbr::NotShadowCaster>)>,
) {
    if time.elapsed_secs() - *last < 2.0 {
        return;
    }
    *last = time.elapsed_secs();
    let total = meshes.iter().count();
    let shown = meshes
        .iter()
        .filter(|(_, v)| **v != Visibility::Hidden)
        .count();
    let seen = meshes.iter().filter(|(v, _)| v.get()).count();
    let cast = casters.iter().filter(|v| v.get()).count();
    // Printed rather than logged: the crate has no log plugin, on purpose.
    eprintln!("--- {:.1}s", time.elapsed_secs());
    let mut rows: Vec<String> = store
        .iter()
        .filter_map(|d| {
            let avg = d.average()?;
            Some(format!("{:<60} {:>10.3} {}", d.path(), avg, d.suffix))
        })
        .collect();
    rows.sort();
    for row in rows {
        eprintln!("{row}");
    }
    eprintln!(
        "meshes: {total} pooled, {shown} not hidden, {seen} in view, {cast} of those cast shadows"
    );
}

/// The render settings, overridable from the command line so one build can
/// be measured at several: `--cascades <n>` and `--shadow-reach <m>` for the
/// sun's cascades, `--shadow-map <px>` for their resolution, `--shadow-filter
/// hard|gaussian` for the edge, and `--no-outline`, `--no-fxaa` and
/// `--no-prepass` to take a pass over the finished picture away. None of
/// them is a setting: the defaults are the game's, and these exist so a
/// change to them is a measurement before it is an argument.
fn experiment(
    mut commands: Commands,
    suns: Query<Entity, With<crate::arenas::Sun>>,
    cameras: Query<Entity, With<crate::MainCamera>>,
) {
    use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, ShadowFilteringMethod};
    let cascades: Option<usize> = crate::platform::value("--cascades").and_then(|v| v.parse().ok());
    let reach: Option<f32> = crate::platform::value("--shadow-reach").and_then(|v| v.parse().ok());
    if cascades.is_some() || reach.is_some() {
        let config = CascadeShadowConfigBuilder {
            num_cascades: cascades.unwrap_or(4),
            maximum_distance: reach.unwrap_or(150.0),
            ..default()
        }
        .build();
        for sun in &suns {
            commands.entity(sun).insert(config.clone());
        }
    }
    if let Some(size) = crate::platform::value("--shadow-map").and_then(|v| v.parse().ok()) {
        commands.insert_resource(DirectionalLightShadowMap { size });
    }
    if let Some(filter) = crate::platform::value("--shadow-filter") {
        let method = match filter {
            "hard" => ShadowFilteringMethod::Hardware2x2,
            _ => ShadowFilteringMethod::Gaussian,
        };
        for camera in &cameras {
            commands.entity(camera).insert(method);
        }
    }
    // The passes over the finished picture, one at a time, so each can be
    // measured by its absence: `--no-outline`, `--no-fxaa`, `--no-prepass`.
    for camera in &cameras {
        if crate::platform::flag("--no-outline") {
            commands.entity(camera).remove::<crate::outline::Outline>();
        }
        if crate::platform::flag("--no-fxaa") {
            commands
                .entity(camera)
                .remove::<bevy::core_pipeline::fxaa::Fxaa>();
        }
        if crate::platform::flag("--no-prepass") {
            commands.entity(camera).remove::<(
                crate::outline::Outline,
                bevy::core_pipeline::prepass::DepthPrepass,
            )>();
        }
    }
    let overrides: Vec<String> = [
        "--cascades",
        "--shadow-reach",
        "--shadow-map",
        "--shadow-filter",
    ]
    .iter()
    .filter_map(|flag| crate::platform::value(flag).map(|v| format!("{flag} {v}")))
    .chain(
        ["--no-outline", "--no-fxaa", "--no-prepass"]
            .into_iter()
            .filter(|flag| crate::platform::flag(flag))
            .map(String::from),
    )
    .collect();
    if !overrides.is_empty() {
        eprintln!("profiling with {}", overrides.join(", "));
    }
}
