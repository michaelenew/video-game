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
            // `OVEN=1` starts it open, so a headless capture can see it, and
            // `--dev` starts it open because that is the working mode.
            open: crate::dev_mode() || crate::platform::env("OVEN").as_deref() == Some("1"),
            // Presetting the search lets a headless capture show real rows.
            search: crate::platform::env("OVEN_SEARCH").unwrap_or_default(),
            message: String::new(),
            note: String::new(),
        }
    }
}

/// **Knobs moved in a match, on their way to the world.** A knob set straight
/// into the Oven changes the rules on this machine alone, and the next frame
/// is a different game from the other machine's -- a desync. So in a match
/// (and in any world that already carries a match's edits) the palette asks
/// for the change instead: it rides on the next tick's input
/// (`sim::input::Tune`), every machine takes it on the same frame, and a
/// rollback puts it back where it was (`sim::oven::Edits`). Training keeps
/// the direct path, which has no limit on how many knobs move.
#[derive(Default)]
pub struct Moved {
    /// Not yet sent, oldest first, one per knob: a slider dragged across a
    /// dozen values in a tick sends only where it stopped.
    waiting: Vec<(u16, i32)>,
    /// Sent, and not yet in the world: input delay holds an edit a few
    /// frames, and the slider shows where it was let go meanwhile.
    sent: Vec<(u16, i32, u32)>,
}

impl Moved {
    /// Ask for `cell` to be `raw`.
    fn ask(&mut self, cell: u16, raw: i32) {
        self.waiting.retain(|(c, _)| *c != cell);
        self.waiting.push((cell, raw));
    }

    /// What `cell` has been asked to be, if it is on its way.
    fn asked(&self, cell: u16) -> Option<i32> {
        self.waiting
            .iter()
            .rev()
            .find(|(c, _)| *c == cell)
            .map(|(_, raw)| *raw)
            .or_else(|| {
                self.sent
                    .iter()
                    .rev()
                    .find(|(c, _, _)| *c == cell)
                    .map(|(_, raw, _)| *raw)
            })
    }

    /// The edit the next tick's input carries, if any. Taken off the queue
    /// by [`Moved::went`] once a frame has really been made with it: a tick
    /// the line held back sends nothing.
    pub fn next(&self) -> sim::input::Tune {
        self.waiting
            .first()
            .map_or(sim::input::Tune::NONE, |(c, raw)| {
                sim::input::Tune::set(*c, *raw)
            })
    }

    /// A frame was made, at `frame`, with [`Moved::next`] on it.
    pub fn went(&mut self, frame: u32) {
        if !self.waiting.is_empty() {
            let (c, raw) = self.waiting.remove(0);
            self.sent.push((c, raw, frame));
        }
    }

    /// The world as it stands: an edit it holds has arrived, and one sent a
    /// second ago and still not there was refused (a match full of edits).
    pub fn settle(&mut self, w: &sim::World) {
        self.sent.retain(|(c, raw, at)| {
            w.edits.get(*c) != Some(*raw) && w.frame.saturating_sub(*at) < sim::TICK_HZ
        });
    }
}

/// How the palette changes a knob: straight into the Oven in training, or
/// asked for on the wire in a match ([`Moved`]).
struct Setter<'a> {
    moved: Option<&'a mut Moved>,
    edits: sim::oven::Edits,
}

impl Setter<'_> {
    fn raw(&self, knob: Knob) -> i32 {
        match &self.moved {
            Some(m) => m.asked(knob.cell()).unwrap_or_else(|| knob.raw()),
            None => knob.raw(),
        }
    }

    fn set(&mut self, knob: Knob, raw: i32) {
        match &mut self.moved {
            Some(m) => m.ask(knob.cell(), raw),
            None => knob.set_raw(raw),
        }
    }

    /// Whether this knob can move here: in a match, while it has room for
    /// another, or this one has moved already.
    fn movable(&self, knob: Knob) -> bool {
        self.moved.is_none() || !self.edits.is_full() || self.edits.get(knob.cell()).is_some()
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
pub fn sample_focus(
    mut contexts: EguiContexts,
    palette: Res<Palette>,
    // The hub shares this window and these devices, so it shares the answer to
    // who owns them. One place decides, or a click lands in two places at once.
    hub: Res<crate::hub::Hub>,
    // And the Esc menu, whose join box takes typing and whose buttons take
    // clicks that must not also be a punch or a grab of the mouse.
    menu: Res<crate::menu::Menu>,
    // And the bag, whose items take clicks.
    bag: Res<crate::keep::Bag>,
    mut focus: ResMut<UiFocus>,
) {
    if !palette.open && !hub.open && !menu.open && !bag.open {
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

pub fn draw(mut contexts: EguiContexts, mut palette: ResMut<Palette>, mut sim: ResMut<crate::Sim>) {
    if !palette.open {
        return;
    }
    let sim = &mut *sim;
    let in_match = sim.driver.online() || !sim.cur.edits.is_empty();
    let edits = sim.cur.edits;
    let mut setter = Setter {
        moved: in_match.then_some(&mut sim.tunes),
        edits,
    };
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
                                    knob_row(ui, knob, &mut setter);
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
                // Every knob back at once is more edits than a match holds:
                // one at a time there, with each knob's own ↺.
                if ui
                    .add_enabled(dirty && !in_match, egui::Button::new("Revert"))
                    .on_hover_text(if in_match {
                        "In a match, one knob at a time: each one's ↺"
                    } else {
                        "Back to the values committed in tuned.rs"
                    })
                    .clicked()
                {
                    oven::reset_to_baked();
                    message = "reverted to baked".into();
                }
            });
            if in_match {
                ui.label(
                    egui::RichText::new(format!(
                        "In a match: a knob moved moves on every machine. {} of {} moved{}",
                        edits.iter().count(),
                        sim::oven::MATCH_EDITS,
                        if edits.is_full() {
                            " -- the most a match holds; those can still move"
                        } else {
                            ""
                        }
                    ))
                    .small()
                    .weak(),
                );
            }
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
fn knob_row(ui: &mut egui::Ui, knob: Knob, setter: &mut Setter) {
    let unit = knob.unit();
    let (lo, hi) = knob.range();
    let raw = setter.raw(knob);
    let mut value = to_display(unit, raw);
    let movable = setter.movable(knob);

    ui.horizontal(|ui| {
        // A dot marks a value that differs from the baked one, so a session's
        // changes are findable after the fact without remembering them.
        let dirty = raw != knob.baked_raw();
        let mark = if dirty { "●" } else { " " };
        ui.label(egui::RichText::new(mark).small());
        ui.label(knob.label()).on_hover_text(knob.id());
    });

    ui.horizontal(|ui| {
        ui.add_enabled_ui(movable, |ui| {
            if unit == Unit::Flag {
                let mut on = raw != 0;
                if ui.checkbox(&mut on, "").changed() {
                    setter.set(knob, on as i32);
                }
            } else {
                let range = to_display(unit, lo)..=to_display(unit, hi);
                let slider = egui::Slider::new(&mut value, range)
                    .fixed_decimals(decimals(unit))
                    .clamping(egui::SliderClamping::Never);
                if ui.add(slider).changed() {
                    setter.set(knob, from_display(unit, value));
                }
            }
            if raw != knob.baked_raw()
                && ui
                    .small_button("↺")
                    .on_hover_text("Reset this one")
                    .clicked()
            {
                setter.set(knob, knob.baked_raw());
            }
        });
    });
}
