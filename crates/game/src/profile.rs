//! A frame-cost readout, on request: `--profile`.
//!
//! Every two seconds, to the log: the frame time, how many entities there
//! are and how many of the meshes are actually on screen, and -- where the
//! driver can time its own work -- what each render pass cost the GPU. It is
//! what turns "the game feels heavy" into a list of passes with numbers
//! beside them, and it is off unless asked for because the readout itself
//! is not free.

use std::collections::HashMap;

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
        .add_systems(Update, (census, heaviest, quit_after));
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

/// **What the triangles are**, once, twelve seconds in: the meshes the
/// camera can see and the meshes that cast shadows, heaviest first, each
/// with how many copies of it there are and how far it is from the camera.
/// A pass's triangle count in the census says *that* it is drawing too much;
/// this says what. It is how the valley's walls were found drawing three
/// million triangles into four shadow cascades.
fn heaviest(
    time: Res<Time>,
    mut done: Local<bool>,
    meshes: Res<Assets<Mesh>>,
    all: Query<(
        &Mesh3d,
        &GlobalTransform,
        &ViewVisibility,
        Has<bevy::pbr::NotShadowCaster>,
    )>,
    cameras: Query<&GlobalTransform, With<crate::MainCamera>>,
) {
    if *done || time.elapsed_secs() < 12.0 {
        return;
    }
    *done = true;
    let eye = cameras
        .single()
        .map(|c| c.translation())
        .unwrap_or_default();
    let tris = |m: &Mesh3d| {
        meshes
            .get(&m.0)
            .and_then(|m| m.indices())
            .map_or(0, |i| i.len() / 3)
    };
    let list = |title: &str, keep: &dyn Fn(bool, bool) -> bool| {
        let mut by: HashMap<AssetId<Mesh>, (usize, usize, f32)> = HashMap::new();
        let mut total = 0;
        for (m, at, seen, no_shadow) in &all {
            if !keep(seen.get(), !no_shadow) {
                continue;
            }
            let n = tris(m);
            let row = by
                .entry(m.0.id())
                .or_insert((n, 0, at.translation().distance(eye)));
            row.1 += 1;
            total += n;
        }
        let mut rows: Vec<_> = by.into_values().collect();
        rows.sort_by_key(|r| std::cmp::Reverse(r.0 * r.1));
        eprintln!("{title}: {total} triangles in {} meshes", rows.len());
        for (n, copies, far) in rows.into_iter().take(20) {
            eprintln!("  {n:>8} triangles x {copies:>4}, {far:>5.0} m away");
        }
    };
    list("in view", &|seen, _| seen);
    list("casting shadows", &|_, casts| casts);
}

/// `--quit-after <seconds>`: leave cleanly after that long, so a build with
/// `bevy/trace_chrome` gets to write its trace (it is written on the way
/// out, and a killed game never takes that way).
fn quit_after(time: Res<Time>, mut exit: EventWriter<AppExit>) {
    let Some(after) = crate::platform::value("--quit-after").and_then(|v| v.parse::<f32>().ok())
    else {
        return;
    };
    if time.elapsed_secs() > after {
        exit.write(AppExit::Success);
    }
}
