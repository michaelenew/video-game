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
        .add_systems(Update, (census, heaviest, quit_after, casters));
        if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render.add_systems(bevy::render::Render, pipelines);
        }
    }
}

/// **Every pipeline the moment it is asked for**: what it is for, and the
/// mesh layout and treatment it was specialised on. A pipeline is compiled
/// the first time something on screen needs it, and a compile is the hitch
/// a player sees as the game seizing for a moment -- so a pipeline that is
/// first asked for minutes into a fight, rather than as the game starts, is
/// one the warm-up missed.
fn pipelines(
    cache: Res<bevy::render::render_resource::PipelineCache>,
    mut seen: Local<usize>,
    mut started: Local<Option<bevy::platform::time::Instant>>,
) {
    use bevy::render::render_resource::PipelineDescriptor;
    let started = *started.get_or_insert_with(bevy::platform::time::Instant::now);
    let all: Vec<_> = cache.pipelines().collect();
    if all.len() == *seen {
        return;
    }
    let at = started.elapsed().as_secs_f32();
    for p in &all[*seen..] {
        let PipelineDescriptor::RenderPipelineDescriptor(d) = &p.descriptor else {
            eprintln!("pipeline {at:.1}s compute");
            continue;
        };
        let label = d.label.as_deref().unwrap_or("?");
        let defs: Vec<String> = d
            .fragment
            .as_ref()
            .map(|f| &f.shader_defs)
            .unwrap_or(&d.vertex.shader_defs)
            .iter()
            .filter_map(|def| match def {
                bevy::render::render_resource::ShaderDefVal::Bool(name, true) => Some(name.clone()),
                _ => None,
            })
            .filter(|n| n.starts_with("VERTEX_") || n.contains("PREPASS") || n.contains("SHADOW"))
            .collect();
        eprintln!("pipeline {at:.1}s {label} [{}]", defs.join(" "));
    }
    *seen = all.len();
}

/// How many meshes there are, and how many the camera can see, every two
/// seconds. Pools are fixed and mostly hidden, so the two numbers are far
/// apart on purpose; what matters is whether the second one is reasonable.
fn census(
    time: Res<Time>,
    mut last: Local<f32>,
    store: Res<DiagnosticsStore>,
    still: Res<crate::still::Maps>,
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
    eprintln!("still world's shadow maps finished: {}", still.finished);
    eprintln!(
        "meshes: {total} pooled, {shown} not hidden, {seen} in view, {cast} of those cast shadows"
    );
}

/// The render settings, overridable from the command line so one build can
/// be measured at several: `--cascades <n>` and `--shadow-reach <m>` for the
/// sun's cascades, `--shadow-map <px>` for their resolution, `--shadow-filter
/// hard|gaussian` for the edge, and `--no-outline`, `--no-fxaa` and
/// `--no-prepass` to take a pass over the finished picture away,
/// `--casters` and `--no-shadow-lookup` for what a baked shadow would save
/// ([`casters`]), `--no-still` to draw the still world's shadow every frame
/// as it was (`crate::still`), and `--no-vsync` so the frame time shows a
/// saving instead of the screen's 16.7 ms. None of
/// them is a setting: the defaults are the game's, and these exist so a
/// change to them is a measurement before it is an argument.
fn experiment(
    mut commands: Commands,
    suns: Query<Entity, With<crate::arenas::Sun>>,
    cameras: Query<Entity, With<crate::MainCamera>>,
    mut windows: Query<&mut Window>,
) {
    if crate::platform::flag("--no-vsync") {
        for mut window in &mut windows {
            window.present_mode = bevy::window::PresentMode::AutoNoVsync;
        }
    }
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
    // measured by its absence: `--no-outline`, `--no-fxaa`, `--no-prepass`,
    // `--no-wash` (a painted look's screen pass).
    for camera in &cameras {
        if crate::platform::flag("--no-outline") {
            commands.entity(camera).remove::<crate::outline::Outline>();
        }
        if crate::platform::flag("--no-fxaa") {
            commands
                .entity(camera)
                .remove::<bevy::core_pipeline::fxaa::Fxaa>();
        }
        if crate::platform::flag("--no-wash") {
            commands.entity(camera).remove::<crate::wash::Wash>();
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
        "--casters",
    ]
    .iter()
    .filter_map(|flag| crate::platform::value(flag).map(|v| format!("{flag} {v}")))
    .chain(
        [
            "--no-outline",
            "--no-fxaa",
            "--no-prepass",
            "--no-wash",
            "--no-vsync",
            "--no-shadow-lookup",
            "--no-still",
        ]
        .into_iter()
        .filter(|flag| crate::platform::flag(flag))
        .map(String::from),
    )
    .collect();
    if !overrides.is_empty() {
        eprintln!("profiling with {}", overrides.join(", "));
    }
}

/// Everything drawn in the paint that still casts a shadow.
type Still = (
    With<Mesh3d>,
    With<MeshMaterial3d<crate::paint::Paint>>,
    Without<bevy::pbr::NotShadowCaster>,
);

/// **Who casts a shadow**, for measuring what a baked shadow would save:
/// `--casters no-giants` takes the grown giants' wood out of the sun's
/// cascades, as a shadow stand-in a few hundred triangles big would;
/// `--casters moving` takes out everything drawn in the paint -- everything
/// that stands still -- leaving the fighters and creatures, which is what a
/// cascade would hold if the still world's shadow were baked. Runs every
/// frame because the stream keeps spawning; what it has marked drops out of
/// its own query.
fn casters(
    mut commands: Commands,
    still: Query<(Entity, Option<&Name>), Still>,
    mut which: Local<Option<Option<String>>>,
) {
    let which = which.get_or_insert_with(|| crate::platform::value("--casters").map(String::from));
    let Some(which) = which.as_deref() else {
        return;
    };
    for (e, name) in &still {
        let giant = name.is_some_and(|n| n.as_str() == "a grown giant");
        if which == "moving" || (which == "no-giants" && giant) {
            commands.entity(e).insert(bevy::pbr::NotShadowCaster);
        }
    }
}

/// **What the triangles are**, twelve seconds in and a minute in: the meshes the
/// camera can see and the meshes that cast shadows, heaviest first, each
/// with how many copies of it there are and how far it is from the camera.
/// A pass's triangle count in the census says *that* it is drawing too much;
/// this says what. It is how the valley's walls were found drawing three
/// million triangles into four shadow cascades.
#[allow(clippy::type_complexity)] // A Bevy query: one component per thing the list reads.
fn heaviest(
    time: Res<Time>,
    mut done: Local<usize>,
    meshes: Res<Assets<Mesh>>,
    all: Query<(
        Entity,
        &Mesh3d,
        &GlobalTransform,
        &ViewVisibility,
        &InheritedVisibility,
        Has<bevy::pbr::NotShadowCaster>,
    )>,
    names: Query<(Option<&Name>, Option<&ChildOf>)>,
    cameras: Query<&GlobalTransform, With<crate::MainCamera>>,
) {
    // Twelve seconds in, and a minute in, when a slow machine has finished
    // loading what is round it.
    let due = [12.0, 60.0];
    if *done >= due.len() || time.elapsed_secs() < due[*done] {
        return;
    }
    *done += 1;
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
    // What a mesh is, as far as anybody named it: its own name or the
    // nearest named thing it hangs off.
    let named = |mut e: Entity| {
        for _ in 0..8 {
            let Ok((name, parent)) = names.get(e) else {
                break;
            };
            if let Some(name) = name {
                return name.as_str().to_string();
            }
            let Some(parent) = parent else {
                break;
            };
            e = parent.parent();
        }
        String::from("?")
    };
    let list = |title: &str, keep: &dyn Fn(bool, bool, bool) -> bool| {
        let mut by: HashMap<AssetId<Mesh>, (usize, usize, f32, Entity)> = HashMap::new();
        let mut total = 0;
        for (e, m, at, seen, shown, no_shadow) in &all {
            if !keep(seen.get(), shown.get(), !no_shadow) {
                continue;
            }
            let n = tris(m);
            let row = by
                .entry(m.0.id())
                .or_insert((n, 0, at.translation().distance(eye), e));
            row.1 += 1;
            total += n;
        }
        let mut rows: Vec<_> = by.into_values().collect();
        rows.sort_by_key(|r| std::cmp::Reverse(r.0 * r.1));
        eprintln!("{title}: {total} triangles in {} meshes", rows.len());
        for (n, copies, far, e) in rows.into_iter().take(20) {
            eprintln!(
                "  {n:>8} triangles x {copies:>4}, {far:>5.0} m away  {}",
                named(e)
            );
        }
    };
    list("in view", &|seen, _, _| seen);
    // Shown, not hidden in a pool: a hidden mesh casts nothing.
    list("casting shadows", &|_, shown, casts| shown && casts);
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
