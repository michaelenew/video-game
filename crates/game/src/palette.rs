//! The Oven's palette: every tuned number in the game, editable while it runs.
//!
//! **F7.** Floats over the arena rather than inside it, so the fight stays
//! visible while you tune — the whole point is watching the change, and a panel
//! that covers what you are judging defeats itself.
//!
//! Three hundred numbers need two ways in. **Families** group them the way you
//! think about them (Movement, Air, Defence, one per move), and **search**
//! finds one when you already know its name. Either alone would be unusable at
//! this size.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use sim::oven::{self, Knob, Unit};

use crate::bake;

/// Whether the palette is currently taking the mouse or the keyboard.
///
/// The game and the editor share one window and one set of input devices, so
/// something has to say which of them a click belongs to. egui already tracks
/// it; this copies the answer somewhere the game systems can read.
///
/// Sampled a frame behind, because the palette draws after the tick. That is
/// fine and standard: a pointer that entered the panel this frame is still over
/// it next frame.
#[derive(Resource, Default)]
pub struct UiFocus {
    /// The pointer is over the palette, or dragging one of its widgets.
    pub pointer: bool,
    /// A text field has focus — the search box or the bake note.
    pub keyboard: bool,
    /// F7 was pressed this frame and the panel came up. Consumed by the camera,
    /// which hands the cursor back so the first click lands on a widget.
    pub just_opened: bool,
}

#[derive(Resource)]
pub struct Palette {
    pub open: bool,
    search: String,
    message: String,
    note: String,
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            // `OVEN=1` starts it open, so a headless capture can see it.
            open: std::env::var("OVEN").is_ok_and(|v| v == "1"),
            // Presetting the search lets a headless capture show real rows.
            search: std::env::var("OVEN_SEARCH").unwrap_or_default(),
            message: String::new(),
            note: String::new(),
        }
    }
}

/// Raw store units are integers; a person edits decimals. The conversion lives
/// here, on the editor side, so `sim` stays free of floating point.
fn to_display(unit: Unit, raw: i32) -> f32 {
    match unit {
        Unit::Fixed => raw as f32 / 65536.0,
        _ => raw as f32,
    }
}

fn from_display(unit: Unit, shown: f32) -> i32 {
    match unit {
        Unit::Fixed => (shown * 65536.0).round() as i32,
        Unit::Flag => (shown >= 0.5) as i32,
        _ => shown.round() as i32,
    }
}

fn decimals(unit: Unit) -> usize {
    match unit {
        Unit::Fixed => 3,
        _ => 0,
    }
}

pub fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut palette: ResMut<Palette>,
    mut focus: ResMut<UiFocus>,
) {
    if keys.just_pressed(KeyCode::F7) {
        palette.open = !palette.open;
        if palette.open {
            focus.just_opened = true;
        } else {
            // Closing it hands the mouse straight back to the game rather than
            // leaving a stale claim on it.
            *focus = UiFocus::default();
        }
    }
}

/// Ask egui what it is currently claiming.
pub fn sample_focus(mut contexts: EguiContexts, palette: Res<Palette>, mut focus: ResMut<UiFocus>) {
    if !palette.open {
        *focus = UiFocus::default();
        return;
    }
    let ctx = contexts.ctx_mut();
    // `just_opened` is set by `toggle` and cleared by the camera that consumes
    // it, so it is left alone here.
    // `wants_pointer_input` alone is not enough: it is false while the pointer
    // merely hovers the panel without a button down, which is exactly when a
    // click is about to land on a slider.
    focus.pointer = ctx.wants_pointer_input() || ctx.is_pointer_over_area();
    focus.keyboard = ctx.wants_keyboard_input();
}

pub fn draw(mut contexts: EguiContexts, mut palette: ResMut<Palette>) {
    if !palette.open {
        return;
    }
    let ctx = contexts.ctx_mut().clone();
    let ctx = &ctx;

    let knobs = oven::all_knobs();
    let needle = palette.search.to_lowercase();
    let matching: Vec<Knob> = knobs
        .into_iter()
        .filter(|k| {
            needle.is_empty()
                || k.id().contains(&needle)
                || k.label().to_lowercase().contains(&needle)
                || k.family().to_lowercase().contains(&needle)
        })
        .collect();

    let dirty = oven::is_dirty();
    let mut search = palette.search.clone();
    let mut note = palette.note.clone();
    let mut message = palette.message.clone();

    egui::Window::new("Oven")
        .default_width(380.0)
        .default_pos([16.0, 90.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("search");
                ui.text_edit_singleline(&mut search);
                if ui.button("clear").clicked() {
                    search.clear();
                }
            });
            ui.label(
                egui::RichText::new(format!(
                    "{} of {} shown",
                    matching.len(),
                    oven::all_knobs().len()
                ))
                .small()
                .weak(),
            );
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(420.0)
                .show(ui, |ui| {
                    // One header per family. Grouping is `oven::grouped`, which
                    // gathers a family wherever its members sit in the registry
                    // rather than assuming they are adjacent -- appending a knob
                    // is what keeps baked indices stable, so they usually are
                    // not.
                    for (family, members) in oven::grouped(&matching) {
                        egui::CollapsingHeader::new(&family)
                            .id_salt(&family)
                            // Searching means you already know what you want, so
                            // results come open rather than making you click in.
                            .default_open(!needle.is_empty())
                            .show(ui, |ui| {
                                for knob in members {
                                    knob_row(ui, knob);
                                }
                            });
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("note");
                ui.text_edit_singleline(&mut note);
            });
            ui.horizontal(|ui| {
                let bake_label = if dirty { "Bake ●" } else { "Bake" };
                if ui
                    .add_enabled(dirty, egui::Button::new(bake_label))
                    .on_hover_text("Write tuned.rs, commit and push on the current branch")
                    .clicked()
                {
                    let what = if note.trim().is_empty() {
                        "from the palette".to_string()
                    } else {
                        note.trim().to_string()
                    };
                    message = match bake::bake(&what) {
                        bake::Outcome::Ok(m) => m,
                        bake::Outcome::Failed(m) => format!("FAILED: {m}"),
                    };
                }
                if ui
                    .add_enabled(dirty, egui::Button::new("Revert"))
                    .on_hover_text("Back to the values committed in tuned.rs")
                    .clicked()
                {
                    oven::reset_to_baked();
                    message = "reverted to baked".into();
                }
            });
            if !message.is_empty() {
                ui.label(egui::RichText::new(&message).small());
            }
        });

    palette.search = search;
    palette.note = note;
    palette.message = message;
}

/// One editable value: a slider, its number, and whether it has drifted from
/// what is committed.
fn knob_row(ui: &mut egui::Ui, knob: Knob) {
    let unit = knob.unit();
    let (lo, hi) = knob.range();
    let mut value = to_display(unit, knob.raw());

    ui.horizontal(|ui| {
        // A dot marks a value that differs from the baked one, so a session's
        // changes are findable after the fact without remembering them.
        let mark = if knob.is_dirty() { "●" } else { " " };
        ui.label(egui::RichText::new(mark).small());
        ui.label(knob.label()).on_hover_text(knob.id());
    });

    ui.horizontal(|ui| {
        if unit == Unit::Flag {
            let mut on = knob.raw() != 0;
            if ui.checkbox(&mut on, "").changed() {
                knob.set_raw(on as i32);
            }
        } else {
            let range = to_display(unit, lo)..=to_display(unit, hi);
            let slider = egui::Slider::new(&mut value, range)
                .fixed_decimals(decimals(unit))
                .clamping(egui::SliderClamping::Never);
            if ui.add(slider).changed() {
                knob.set_raw(from_display(unit, value));
            }
        }
        if knob.is_dirty()
            && ui
                .small_button("↺")
                .on_hover_text("Reset this one")
                .clicked()
        {
            knob.set_raw(knob.baked_raw());
        }
    });
}
