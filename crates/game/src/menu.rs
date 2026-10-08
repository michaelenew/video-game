//! The Esc menu: everything that is not the fight.
//!
//! Escape lets go of the mouse and brings it up; a click in the arena takes
//! the mouse back and puts it away. While it is up:
//!
//! - **Play a friend.** *Create a room* makes one from the fight being
//!   practised and shows its link with a *Copy link* button; *Join* takes a
//!   link a friend sent. While a room is active it is listed instead -- its
//!   name, how meeting is going, the link, *Copy link* and *Leave*. Rooms
//!   are `online.rs`; this only draws them.
//! - **Connection details**, in dev mode (F10): the meeting's own account of
//!   how each meeting point's connection went, for a room that will not form.
//! - **Save replay.** The fight so far -- every frame of it, from its start
//!   -- as a file (a download, in a browser) that
//!   `cargo run -p hunt --bin replay` judges and `--replay` plays back. `Y`
//!   does the same without the menu. See `docs/design/replays.md`.
//! - **Progress** -- the creatures, their trophies and tempers -- on the right,
//!   which is otherwise hidden: it is reference, not something to read
//!   mid-fight (`hud::update_picker` writes it; [`show_progress`] shows it).
//!
//! In a browser the page releases the mouse itself on Escape and the key never
//! reaches the game, so the camera also opens the menu when the page reports
//! the lock gone (`platform::pointer_lock_lost`). Either way, the menu is up
//! exactly when the player has stepped out of the fight.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

/// Whether the menu is up, and what has been typed into it.
#[derive(Resource, Default)]
pub struct Menu {
    pub open: bool,
    /// The join box.
    draft: String,
    /// Why the last join was refused, until the next try.
    refused: Option<String>,
    /// "Copied" on the button for a moment after a copy.
    copied_at: Option<f64>,
    /// The same, for the connection details.
    details_copied_at: Option<f64>,
}

/// The Oven and the hub own the screen while they are open; the menu waits.
fn visible(menu: &Menu, palette: &crate::palette::Palette, hub: &crate::hub::Hub) -> bool {
    menu.open && !palette.open && !hub.open
}

pub fn draw(
    mut contexts: EguiContexts,
    mut menu: ResMut<Menu>,
    mut sim: ResMut<crate::Sim>,
    palette: Res<crate::palette::Palette>,
    hub: Res<crate::hub::Hub>,
    time: Res<Time<Real>>,
) {
    if !visible(&menu, &palette, &hub) {
        return;
    }
    let now = time.elapsed_secs_f64();
    let ctx = contexts.ctx_mut();
    egui::Window::new("Menu")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(18.0, 150.0))
        .resizable(false)
        .collapsible(false)
        .default_width(380.0)
        .show(ctx, |ui| {
            ui.heading("Play a friend");
            ui.add_space(4.0);
            match sim.driver.room().cloned() {
                Some(room) => {
                    ui.label(egui::RichText::new(format!("Room {}", room.name)).strong());
                    if let Some(line) = sim.driver.status() {
                        ui.label(line);
                    }
                    ui.add_space(4.0);
                    ui.label("Send your friend this link. In a browser it opens the game; on a desktop it goes to game --join.");
                    // A `&str` is a read-only text buffer: selectable, not editable.
                    ui.add(
                        egui::TextEdit::singleline(&mut room.link.as_str())
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                    ui.horizontal(|ui| {
                        let copied = menu.copied_at.is_some_and(|t| now - t < 1.5);
                        if ui
                            .button(if copied { "Copied" } else { "Copy link" })
                            .clicked()
                        {
                            ui.ctx().copy_text(room.link.clone());
                            menu.copied_at = Some(now);
                        }
                        if ui.button("Leave").clicked() {
                            crate::online::leave(&mut sim);
                        }
                    });
                }
                None => {
                    if let Some(why) = sim.driver.status() {
                        ui.label(egui::RichText::new(why).weak());
                        ui.add_space(4.0);
                    }
                    if ui.button("Create a room").clicked() {
                        crate::online::create(&mut sim);
                        menu.refused = None;
                    }
                    ui.label(
                        egui::RichText::new(
                            "Makes a link for the fight you are practising: these two classes, this creature, this arena.",
                        )
                        .weak(),
                    );
                    ui.add_space(8.0);
                    ui.label("Or join a friend's room:");
                    let mut join = false;
                    ui.horizontal(|ui| {
                        let field = ui.add(
                            egui::TextEdit::singleline(&mut menu.draft)
                                .hint_text("paste their link")
                                .desired_width(260.0),
                        );
                        join = ui.button("Join").clicked()
                            || field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    });
                    if join {
                        let draft = std::mem::take(&mut menu.draft);
                        menu.refused = crate::online::join(&mut sim, &draft).err();
                        if menu.refused.is_some() {
                            menu.draft = draft;
                        }
                    }
                    if let Some(why) = &menu.refused {
                        ui.colored_label(egui::Color32::from_rgb(255, 120, 120), why);
                    }
                }
            }
            details(ui, &mut menu, &sim, now);
            ui.separator();
            ui.heading("Replay");
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("Save replay").clicked() {
                    crate::save_replay(&mut sim);
                }
                ui.label(
                    egui::RichText::new(format!(
                        "the fight so far: {} frames, {:.0} s",
                        sim.tape_len(),
                        sim.tape_len() as f32 / sim::TICK_HZ as f32
                    ))
                    .weak(),
                );
            });
            if let Some(at) = &sim.saved {
                ui.label(egui::RichText::new(format!("Saved: {at}")).weak());
            }
            ui.label(
                egui::RichText::new(
                    "Y saves too. Send the file to be judged: cargo run -p hunt --bin replay -- <file>.",
                )
                .weak(),
            );
            ui.separator();
            ui.label(egui::RichText::new("Click the arena to play. Esc brings this back.").weak());
        });
}

/// **Connection details**, in dev mode: the meeting's own account of itself
/// (`net::Rendezvous::report`) -- each meeting point's connection step by step,
/// who has been heard from, the direct line -- with a button to copy it all,
/// because the person reading it is usually about to paste it to somebody.
/// Outside dev mode, a room that is meeting or has failed says where they are.
fn details(ui: &mut egui::Ui, menu: &mut Menu, sim: &crate::Sim, now: f64) {
    let report = sim.driver.report();
    if report.is_empty() {
        return;
    }
    ui.add_space(6.0);
    if !crate::dev_mode() {
        ui.label(egui::RichText::new("F10 (dev mode) shows the connection details.").weak());
        return;
    }
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Connection details").strong());
        let copied = menu.details_copied_at.is_some_and(|t| now - t < 1.5);
        if ui
            .button(if copied { "Copied" } else { "Copy details" })
            .clicked()
        {
            let mut text = sim.driver.status().unwrap_or("").to_string();
            for line in &report {
                text.push('\n');
                text.push_str(line);
            }
            ui.ctx().copy_text(text);
            menu.details_copied_at = Some(now);
        }
    });
    egui::ScrollArea::vertical()
        .max_height(280.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for line in &report {
                ui.label(egui::RichText::new(line).monospace().small());
            }
        });
}

/// The progress list on the right is part of the menu: shown while it is up,
/// hidden in the fight. The HUD's online line is the other way round: the
/// menu says the same thing, and the line would sit across the list.
pub fn show_progress(
    menu: Res<Menu>,
    palette: Res<crate::palette::Palette>,
    hub: Res<crate::hub::Hub>,
    mut progress: Query<
        &mut Visibility,
        (
            With<crate::hud::PickerText>,
            Without<crate::hud::OnlineText>,
        ),
    >,
    mut online: Query<&mut Visibility, With<crate::hud::OnlineText>>,
) {
    let up = visible(&menu, &palette, &hub);
    let shown = |yes: bool| {
        if yes {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    for mut v in progress.iter_mut() {
        v.set_if_neq(shown(up));
    }
    for mut v in online.iter_mut() {
        v.set_if_neq(shown(!up));
    }
}
