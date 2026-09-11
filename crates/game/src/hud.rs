//! On-screen state.
//!
//! Health, round wins, and the current action with frames remaining. The frame
//! readout is the part that matters: you cannot judge whether fourteen frames
//! of startup feels right without seeing that it was fourteen.

use bevy::prelude::*;
use sim::state::{Action, Phase};

const P1: Color = Color::srgb(0.29, 0.66, 1.0);
const P2: Color = Color::srgb(1.0, 0.54, 0.30);
const INK: Color = Color::srgb(0.86, 0.90, 0.96);
const DIM: Color = Color::srgb(0.52, 0.58, 0.67);

#[derive(Component)]
pub struct HealthBar(pub usize);

#[derive(Component)]
pub struct StateText(pub usize);

#[derive(Component)]
pub struct RoundText;

#[derive(Component)]
pub struct Banner;

#[derive(Component)]
pub struct SensitivityText;

pub fn setup(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(18.0)),
            ..default()
        })
        .with_children(|root| {
            // Top: health bars either side of the round counter.
            root.spawn(Node {
                width: Val::Percent(100.0),
                column_gap: Val::Px(18.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|top| {
                spawn_health(top, 0, P1);
                top.spawn((
                    Text::new("0 - 0"),
                    TextFont {
                        font_size: 26.0,
                        ..default()
                    },
                    TextColor(INK),
                    RoundText,
                ));
                spawn_health(top, 1, P2);
            });

            // Middle: the round banner, empty while fighting.
            root.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|mid| {
                mid.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 44.0,
                        ..default()
                    },
                    TextColor(INK),
                    Banner,
                ));
            });

            // Bottom: per-player frame data, and the controls.
            root.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                ..default()
            })
            .with_children(|bottom| {
                bottom.spawn((
                    Text::new("free"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(P1),
                    StateText(0),
                ));
                bottom.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    children![
                        (
                            Text::new(""),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(INK),
                            SensitivityText,
                        ),
                        (
                            // Generated from `crates/manual`, which the
                            // `--help` text reads too. The legend used to be a
                            // hand-kept copy of the key handlers and had
                            // already drifted once.
                            Text::new(manual::legend()),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(DIM),
                        ),
                    ],
                ));
                bottom.spawn((
                    Text::new("free"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(P2),
                    StateText(1),
                ));
            });
        });
}

fn spawn_health(parent: &mut ChildSpawnerCommands, who: usize, colour: Color) {
    parent
        .spawn((
            Node {
                width: Val::Percent(40.0),
                height: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.09, 0.11, 0.14)),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(colour),
                HealthBar(who),
            ));
        });
}

/// Bevy needs these disjointness bounds spelled out, and inline they are
/// unreadable. Naming them keeps the signature legible.
// Each of these touches `&mut Text`, so every one has to be provably disjoint
// from the others or Bevy refuses the system at run time. The marker components
// are what makes them disjoint; the `Without` bounds are what proves it.
type StateQuery<'w, 's> = Query<
    'w,
    's,
    (&'static StateText, &'static mut Text),
    (
        Without<RoundText>,
        Without<Banner>,
        Without<SensitivityText>,
    ),
>;
type RoundQuery<'w, 's> =
    Query<'w, 's, &'static mut Text, (With<RoundText>, Without<Banner>, Without<SensitivityText>)>;
type BannerQuery<'w, 's> =
    Query<'w, 's, &'static mut Text, (With<Banner>, Without<RoundText>, Without<SensitivityText>)>;
type SensitivityQuery<'w, 's> =
    Query<'w, 's, &'static mut Text, (With<SensitivityText>, Without<RoundText>, Without<Banner>)>;

pub fn update(
    sim: Res<crate::Sim>,
    settings: Res<crate::settings::Settings>,
    mut bars: Query<(&HealthBar, &mut Node)>,
    mut states: StateQuery,
    mut rounds: RoundQuery,
    mut banner: BannerQuery,
    mut sensitivity: SensitivityQuery,
) {
    // `is_changed` is true on the first run as well as after an edit, so the
    // readout fills itself in without a separate startup path.
    if settings.is_changed() {
        if let Ok(mut t) = sensitivity.single_mut() {
            *t = Text::new(settings.label());
        }
    }

    for (bar, mut node) in bars.iter_mut() {
        let hp = sim.cur.players[bar.0].health.max(0) as f32;
        node.width = Val::Percent(100.0 * hp / sim::state::max_health() as f32);
    }

    for (tag, mut text) in states.iter_mut() {
        let p = &sim.cur.players[tag.0];
        *text = Text::new(format!(
            "{}\n{}\n{}",
            p.class.name(),
            describe(p),
            mechanic(p)
        ));
    }

    if let Ok(mut t) = rounds.single_mut() {
        // The simulation frame rides along with the score so a screenshot says
        // which moment it caught. Two captures of "the same" pose are only
        // comparable if you can see they are the same frame.
        *t = Text::new(format!(
            "{} - {}   f{}",
            sim.cur.players[0].rounds_won, sim.cur.players[1].rounds_won, sim.cur.frame
        ));
    }

    if let Ok(mut t) = banner.single_mut() {
        *t = Text::new(match sim.cur.phase {
            Phase::Fighting => String::new(),
            Phase::RoundOver { winner, .. } if winner == u8::MAX => "double KO".into(),
            Phase::RoundOver { winner, .. } => format!("player {} wins the round", winner + 1),
        });
    }
}

/// The class mechanic in one line -- where the shield is, which form is out,
/// how deep the meter runs. Without it the mechanics are invisible.
fn mechanic(p: &sim::state::Player) -> String {
    use sim::class::alloc_free::Summary;
    match p.mechanic.summary() {
        Summary::Text(t) => t.to_string(),
        Summary::Value(label, v) => format!("{label}: {v}"),
    }
}

/// The action, and how many frames of it remain. `4/3/10` alongside it is the
/// move's startup, active and recovery, so a number can be judged in context.
fn describe(p: &sim::state::Player) -> String {
    let phase_of = |kind: u8, name: &str, left: u16| {
        let m = sim::moves::get(p.class, kind);
        format!(
            "{} {name} {left}f   [{}/{}/{}]  {:+} blk",
            m.name,
            m.startup,
            m.active,
            m.recovery,
            m.on_block()
        )
    };
    match p.action {
        Action::Free => "free".into(),
        Action::Startup { kind, left } => phase_of(kind, "startup", left),
        Action::Active { kind, left } => phase_of(kind, "ACTIVE", left),
        Action::Recovery { kind, left } => phase_of(kind, "recovery", left),
        Action::Guard { held } if held < sim::state::parry_window() => {
            format!("PARRY {held}f")
        }
        Action::Guard { held } => format!("guard {held}f"),
        Action::Dodge { left } => {
            let tag = if p.action.invulnerable() {
                "DODGE i-frames"
            } else {
                "dodge recovery"
            };
            format!("{tag} {left}f")
        }
        Action::BlockStun { left } => format!("blockstun {left}f"),
        Action::HitStun { left } => format!("hitstun {left}f"),
        Action::Stagger { left } => format!("STAGGER {left}f"),
    }
}
