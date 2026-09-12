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

/// A click-to-cycle class picker, one per player.
///
/// Sits beside that player's health bar because that is where you are already
/// looking to know who is who. The button *is* the class name, so it labels
/// itself and there is nothing to read off elsewhere.
#[derive(Component)]
pub struct ClassButton(pub usize);

/// Whether the pickers are drawn. **F8**, and on by default under `--dev`.
#[derive(Resource)]
pub struct ShowClassButtons(pub bool);

impl Default for ShowClassButtons {
    fn default() -> Self {
        ShowClassButtons(crate::dev_mode())
    }
}

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
                spawn_class_button(top, 0, P1);
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
                spawn_class_button(top, 1, P2);
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

fn spawn_class_button(parent: &mut ChildSpawnerCommands, who: usize, colour: Color) {
    parent.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor(colour),
        BorderRadius::all(Val::Px(3.0)),
        BackgroundColor(Color::srgba(0.09, 0.11, 0.14, 0.85)),
        ClassButton(who),
        children![(
            Text::new("—"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(colour),
            ClassLabel(who),
        )],
    ));
}

fn spawn_health(parent: &mut ChildSpawnerCommands, who: usize, colour: Color) {
    parent
        .spawn((
            Node {
                width: Val::Percent(34.0),
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
// Every one of these touches `&mut Text`, so each must be provably disjoint
// from the rest or Bevy refuses the system at run time. The marker components
// make them disjoint; the `Without` bounds prove it. Adding a fifth means
// adding it to the other four -- which is the cost of the pattern, and the
// reason the compiler cannot catch a miss here.
type StateQuery<'w, 's> = Query<
    'w,
    's,
    (&'static StateText, &'static mut Text),
    (
        Without<RoundText>,
        Without<Banner>,
        Without<SensitivityText>,
        Without<ClassLabel>,
    ),
>;
type RoundQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<RoundText>,
        Without<Banner>,
        Without<SensitivityText>,
        Without<ClassLabel>,
    ),
>;
type BannerQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<Banner>,
        Without<RoundText>,
        Without<SensitivityText>,
        Without<ClassLabel>,
    ),
>;
type SensitivityQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<SensitivityText>,
        Without<RoundText>,
        Without<Banner>,
        Without<ClassLabel>,
    ),
>;

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
        // The launch multiplier sits where Smash puts its percent: beside the
        // fighter, because it is the number that decides whether the next hit
        // is a shove or a kill, and neither player can read the system without
        // being able to see it.
        *text = Text::new(format!(
            "{}   launch x{:.1}\n{}\n{}",
            p.class.name(),
            sim::state::launch_scale(p).to_f32_for_render(),
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
///
/// A contact freeze is labelled rather than left to be inferred. Frozen frames
/// look exactly like a dropped frame or a hung game from the outside, and the
/// label is the difference between "the hit landed" and "something broke".
fn describe(p: &sim::state::Player) -> String {
    let state = describe_action(p);
    if p.hitlag > 0 {
        return format!("FREEZE {}f  ({state})", p.hitlag);
    }
    state
}

fn describe_action(p: &sim::state::Player) -> String {
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
        Action::Held { left } => format!("HELD {left}f"),
    }
}

/// Advance one player's class, leaving the other alone.
///
/// Separated from the click handling so the rule can be tested without a
/// window, a cursor, or a rendered frame.
pub fn cycle_class(mut classes: [sim::Class; 2], who: usize) -> [sim::Class; 2] {
    let all = sim::class::ALL_CLASSES;
    let next = (classes[who] as usize + 1) % all.len();
    classes[who] = all[next];
    classes
}

type ButtonQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static ClassButton,
        &'static Children,
    ),
    Changed<Interaction>,
>;

/// Click a picker to cycle that player's class.
///
/// Restarts the match, exactly as Tab does, because a class change mid-round
/// would leave the mechanic holding somebody else's state — a shield in flight
/// belonging to a class that no longer has one.
pub fn class_buttons(
    mut sim: ResMut<crate::Sim>,
    show: Res<ShowClassButtons>,
    buttons: ButtonQuery,
) {
    if !show.0 {
        return;
    }
    for (interaction, button, _) in buttons.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let classes = cycle_class(
            [sim.cur.players[0].class, sim.cur.players[1].class],
            button.0,
        );
        let w = sim::World::with_classes(classes);
        sim.prev = w.clone();
        sim.cur = w;
    }
}

/// Marker for the text inside a picker, so its label can be kept current.
#[derive(Component)]
pub struct ClassLabel(pub usize);

type LabelQuery<'w, 's> = Query<
    'w,
    's,
    (&'static ClassLabel, &'static mut Text),
    (
        Without<RoundText>,
        Without<Banner>,
        Without<SensitivityText>,
        Without<StateText>,
    ),
>;

/// Show or hide the pickers, and keep their labels showing the current class.
pub fn update_class_buttons(
    sim: Res<crate::Sim>,
    show: Res<ShowClassButtons>,
    mut buttons: Query<(&mut Node, &mut BackgroundColor, &Interaction), With<ClassButton>>,
    mut labels: LabelQuery,
) {
    for (mut node, mut bg, interaction) in buttons.iter_mut() {
        node.display = if show.0 { Display::Flex } else { Display::None };
        // A hovered picker lightens, so it reads as something you can press
        // rather than as another readout.
        bg.0 = match interaction {
            Interaction::Hovered | Interaction::Pressed => Color::srgba(0.20, 0.24, 0.30, 0.95),
            Interaction::None => Color::srgba(0.09, 0.11, 0.14, 0.85),
        };
    }
    for (label, mut text) in labels.iter_mut() {
        let name = sim.cur.players[label.0].class.name();
        if text.0 != name {
            *text = Text::new(name);
        }
    }
}

/// Whether the pointer is over a class picker.
///
/// Folded into the same `UiFocus` the Oven uses. Without it, clicking a picker
/// would also throw a poke and re-capture the mouse for the camera — the game
/// and its interface share one pointer, and something has to say which of them
/// a click belongs to.
pub fn sample_button_focus(
    show: Res<ShowClassButtons>,
    buttons: Query<&Interaction, With<ClassButton>>,
    mut focus: ResMut<crate::palette::UiFocus>,
) {
    if !show.0 {
        return;
    }
    if buttons
        .iter()
        .any(|i| matches!(i, Interaction::Hovered | Interaction::Pressed))
    {
        focus.pointer = true;
    }
}

pub fn toggle_class_buttons(keys: Res<ButtonInput<KeyCode>>, mut show: ResMut<ShowClassButtons>) {
    if keys.just_pressed(KeyCode::F8) {
        show.0 = !show.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::class::{ALL_CLASSES, Class};

    #[test]
    fn a_picker_moves_only_its_own_player() {
        // Two pickers sharing one array is the obvious place to get an index
        // wrong, and the symptom would be changing the wrong fighter's class
        // mid-session -- confusing rather than obviously broken.
        let start = [Class::Bulwark, Class::Elementalist];
        let after = cycle_class(start, 0);
        assert_ne!(after[0], start[0], "the picker did not advance its player");
        assert_eq!(after[1], start[1], "it changed the other player too");

        let after = cycle_class(start, 1);
        assert_eq!(after[0], start[0]);
        assert_ne!(after[1], start[1]);
    }

    #[test]
    fn cycling_reaches_every_class_and_comes_home() {
        let start = [Class::Bulwark, Class::Bulwark];
        let mut seen = vec![start[0]];
        let mut classes = start;
        for _ in 1..ALL_CLASSES.len() {
            classes = cycle_class(classes, 0);
            seen.push(classes[0]);
        }
        for class in ALL_CLASSES {
            assert!(
                seen.contains(&class),
                "{class:?} is unreachable by clicking"
            );
        }
        classes = cycle_class(classes, 0);
        assert_eq!(classes[0], start[0], "the cycle does not wrap");
    }
}
