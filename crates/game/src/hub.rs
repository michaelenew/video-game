//! The animation hub: every clip in the game, editable while it runs.
//!
//! **F9.** The Oven did this for tuning numbers and the argument is the same
//! one. Animation work is a loop -- move a pose, play it, move it again -- and
//! the loop is only as fast as its slowest step. With the clips compiled in,
//! that step is a re-bake and a rebuild, so in practice you change one pose,
//! wait, and lose the comparison you were trying to make.
//!
//! So the hub keeps the *recipes* in memory, re-runs the solver on every edit,
//! and drives a real fighter in the real arena with the result. What you are
//! looking at while you drag a slider is the thing that ships.
//!
//! ## What a person edits here
//!
//! The kinematics are handled: bones hang off their parents, joints stop where
//! a body stops, and a planted foot is solved for rather than keyed. What is
//! left is the part that is actually authorship:
//!
//! 1. **Poses** -- the handful of shapes a move passes through, in degrees,
//!    joint by joint, with the limits built into the sliders.
//! 2. **Timing** -- where each pose sits on the timeline.
//! 3. **The curve between them** -- a cubic Bézier per gap, dragged by its two
//!    handles. This is the knob that decides whether a move is a steady slide,
//!    a hold and then a burst, or a pull-back before a commitment, and it is
//!    where most of the character of a move lives.
//! 4. **Looseness** -- how far each part of the body trails and rings, in
//!    frames rather than in spring frequencies.
//!
//! Saving regenerates `crates/anim/src/clips/<file>.rs` in the same shape a
//! person would have written by hand, and re-bakes the table the game reads. A
//! session in the hub ends as a reviewable diff.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use anim::bake::{Key, Looseness, Recipe};
use anim::ease::Ease;
use view::clips::{ALL, Clip, Family};
use view::pose::Pose;
use view::skeleton::{GROUPS, Group, JOINTS, Joint};

use crate::palette::UiFocus;

#[derive(Resource)]
pub struct Hub {
    pub open: bool,
    search: String,
    clip: Clip,
    /// A working copy of every recipe in the game, including empty ones for
    /// clips nobody has authored -- so a missing clip can be started here
    /// rather than only in a text editor.
    recipes: Vec<Recipe>,
    /// The selected recipe, solved. Re-run on every edit, which costs about a
    /// tenth of a millisecond and is what makes this worth using.
    baked: Vec<Pose>,
    frame: f32,
    playing: bool,
    /// Playback rate. Quarter speed is where timing problems become visible.
    speed: f32,
    key: usize,
    /// Which fighter wears the preview.
    on: usize,
    /// Hold the selected key rather than playing, for posing.
    posing: bool,
    message: String,
    /// A bake running in the background. Re-baking shells out to a fresh
    /// process -- which is the point, because it proves the file that was just
    /// written actually compiles -- and that takes seconds. Doing it on the
    /// frame would stop the game dead in the middle of the thing you were
    /// watching.
    baking: Option<std::sync::Arc<std::sync::Mutex<Option<String>>>>,
}

impl Default for Hub {
    fn default() -> Self {
        let mut hub = Hub {
            open: false,
            search: String::new(),
            clip: Clip::Idle,
            recipes: load_recipes(),
            baked: Vec::new(),
            frame: 0.0,
            playing: true,
            speed: 1.0,
            key: 0,
            on: 1,
            posing: false,
            message: String::new(),
            baking: None,
        };
        hub.rebake();
        hub
    }
}

/// Every recipe there is, with a blank one stood up for any clip that has none.
///
/// The blank is a single key holding the rest pose, which is exactly what the
/// bake falls back to anyway -- so an unauthored clip opens in the hub showing
/// the truth about itself rather than refusing to open.
fn load_recipes() -> Vec<Recipe> {
    let mut authored = anim::clips::all();
    for clip in ALL.iter().copied() {
        if !authored.iter().any(|r| r.clip == clip) {
            authored.push(Recipe {
                clip,
                keys: vec![Key::at(0, Pose::rest())],
                looseness: Looseness::MARTIAL,
                notes: String::new(),
            });
        }
    }
    authored
}

impl Hub {
    fn index(&self) -> usize {
        self.recipes
            .iter()
            .position(|r| r.clip == self.clip)
            .unwrap_or(0)
    }

    fn recipe(&self) -> &Recipe {
        &self.recipes[self.index()]
    }

    fn recipe_mut(&mut self) -> &mut Recipe {
        let i = self.index();
        &mut self.recipes[i]
    }

    fn rebake(&mut self) {
        let recipe = self.recipe().clone();
        self.baked = anim::bake(&recipe).frames;
        self.key = self.key.min(recipe.keys.len().saturating_sub(1));
    }

    /// The pose the hub wants drawn on a fighter, if it is driving that one.
    pub fn preview_for(&self, owner: usize) -> Option<Pose> {
        if !self.open || owner != self.on {
            return None;
        }
        if self.posing {
            return self.recipe().keys.get(self.key).map(|k| k.pose);
        }
        if self.baked.is_empty() {
            return None;
        }
        let i = (self.frame as usize).min(self.baked.len() - 1);
        Some(self.baked[i])
    }

    fn selected_key(&self) -> Option<&Key> {
        self.recipe().keys.get(self.key)
    }

    /// Move the playhead to the selected key, so posing and scrubbing agree.
    fn snap_to_key(&mut self) {
        if let Some(k) = self.selected_key() {
            self.frame = k.frame as f32;
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

pub fn toggle(keys: Res<ButtonInput<KeyCode>>, mut hub: ResMut<Hub>, mut focus: ResMut<UiFocus>) {
    if keys.just_pressed(KeyCode::F9) {
        hub.open = !hub.open;
        if hub.open {
            focus.just_opened = true;
            hub.rebake();
        } else {
            *focus = UiFocus::default();
        }
    }
}

/// Advance the playhead. Renderer-local on purpose: this is a preview clock and
/// touches nothing the simulation can see.
pub fn advance(time: Res<Time>, mut hub: ResMut<Hub>) {
    if !hub.open || !hub.playing || hub.posing || hub.baked.is_empty() {
        return;
    }
    let length = hub.baked.len() as f32;
    hub.frame += time.delta_secs() * 60.0 * hub.speed;
    if hub.frame >= length {
        hub.frame = if hub.clip.looping() {
            hub.frame % length
        } else {
            // One-shots hold on their last frame for a beat, then start again,
            // because the end of a move is the part you are usually judging.
            if hub.frame > length + 24.0 {
                0.0
            } else {
                length - 0.01
            }
        };
    }
}

pub fn draw(mut contexts: EguiContexts, mut hub: ResMut<Hub>) {
    if !hub.open {
        return;
    }
    let ctx = contexts.ctx_mut().clone();
    let ctx = &ctx;
    let mut dirty = false;

    egui::Window::new("Animation hub")
        .default_width(430.0)
        .default_pos([16.0, 90.0])
        .resizable(true)
        .show(ctx, |ui| {
            dirty |= clip_list(ui, &mut hub);
            ui.separator();
            transport(ui, &mut hub);
            ui.separator();
            dirty |= timeline(ui, &mut hub);
            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(360.0)
                .show(ui, |ui| {
                    dirty |= key_editor(ui, &mut hub);
                    ui.separator();
                    dirty |= looseness_editor(ui, &mut hub);
                });
            ui.separator();
            saving(ui, &mut hub);
        });

    if dirty {
        hub.rebake();
    }
}

// ---------------------------------------------------------------------------
// Panels
// ---------------------------------------------------------------------------

fn clip_list(ui: &mut egui::Ui, hub: &mut Hub) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("clip");
        ui.text_edit_singleline(&mut hub.search);
        if ui.button("clear").clicked() {
            hub.search.clear();
        }
    });

    let needle = hub.search.to_lowercase();
    let missing = anim::clips::missing();
    let mut selected = hub.clip;

    egui::ScrollArea::vertical()
        .id_salt("clips")
        .max_height(150.0)
        .show(ui, |ui| {
            let mut family: Option<Family> = None;
            for clip in ALL.iter().copied() {
                if !needle.is_empty() && !clip.name().contains(&needle) {
                    continue;
                }
                if family != Some(clip.family()) {
                    family = Some(clip.family());
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(clip.family().name())
                            .small()
                            .color(egui::Color32::from_gray(150)),
                    );
                }
                let label = if missing.contains(&clip) {
                    // Said plainly rather than hidden: an empty clip is work
                    // outstanding, and the list is the only place it shows.
                    format!("{}  ·  empty", clip.name())
                } else {
                    format!("{}  ·  {} frames", clip.name(), clip.length())
                };
                if ui.selectable_label(selected == clip, label).clicked() {
                    selected = clip;
                }
            }
        });

    if selected != hub.clip {
        hub.clip = selected;
        hub.frame = 0.0;
        hub.key = 0;
        changed = true;
    }
    ui.label(
        egui::RichText::new(hub.clip.what())
            .small()
            .color(egui::Color32::from_gray(170)),
    );
    changed
}

fn transport(ui: &mut egui::Ui, hub: &mut Hub) {
    let length = hub.baked.len().max(1) as f32;
    ui.horizontal(|ui| {
        if ui
            .button(if hub.playing { "pause" } else { "play" })
            .clicked()
        {
            hub.playing = !hub.playing;
            hub.posing = false;
        }
        ui.label("speed");
        for (label, rate) in [("1x", 1.0), ("1/2", 0.5), ("1/4", 0.25)] {
            if ui.selectable_label(hub.speed == rate, label).clicked() {
                hub.speed = rate;
            }
        }
        ui.checkbox(&mut hub.posing, "hold key")
            .on_hover_text("Freeze on the selected key, for posing.");
    });
    ui.horizontal(|ui| {
        ui.label("on player");
        for owner in 0..sim::state::MAX_PLAYERS {
            if ui
                .selectable_label(hub.on == owner, format!("{}", owner + 1))
                .clicked()
            {
                hub.on = owner;
            }
        }
        ui.add(
            egui::Slider::new(&mut hub.frame, 0.0..=(length - 1.0))
                .text("frame")
                .fixed_decimals(0),
        );
    });
}

/// The timeline: keys as draggable marks, the playhead as a line.
fn timeline(ui: &mut egui::Ui, hub: &mut Hub) -> bool {
    let mut changed = false;
    let length = hub.recipe().clip.length().max(1) as f32;
    let height = 34.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click_and_drag(),
    );
    let painter = ui.painter_at(rect);
    let at = |f: f32| rect.left() + rect.width() * (f / length);
    let frame_at = |x: f32| ((x - rect.left()) / rect.width() * length).clamp(0.0, length - 1.0);

    painter.rect_filled(rect, 3.0, egui::Color32::from_gray(28));

    // Phase boundaries, for anything that animates a move. Authoring a contact
    // pose against a typed-in frame number is how a hitbox and its animation
    // end up one frame apart.
    if let Some((last_startup, first_active, first_recovery)) = hub.clip.phases() {
        for (f, colour) in [
            (
                last_startup as f32 + 1.0,
                egui::Color32::from_rgb(200, 160, 60),
            ),
            (first_active as f32, egui::Color32::from_rgb(220, 90, 80)),
            (first_recovery as f32, egui::Color32::from_rgb(90, 150, 210)),
        ] {
            painter.line_segment(
                [
                    egui::pos2(at(f), rect.top()),
                    egui::pos2(at(f), rect.bottom()),
                ],
                egui::Stroke::new(1.0_f32, colour),
            );
        }
    }

    // Keys.
    let keys: Vec<u16> = hub.recipe().keys.iter().map(|k| k.frame).collect();
    for (i, frame) in keys.iter().enumerate() {
        let x = at(*frame as f32);
        let selected = i == hub.key;
        painter.circle_filled(
            egui::pos2(x, rect.center().y),
            if selected { 6.0 } else { 4.0 },
            if selected {
                egui::Color32::from_rgb(250, 220, 120)
            } else {
                egui::Color32::from_gray(190)
            },
        );
    }

    // Playhead.
    let x = at(hub.frame);
    painter.line_segment(
        [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(120, 230, 200)),
    );

    if let Some(pos) = response.interact_pointer_pos() {
        let f = frame_at(pos.x);
        if response.drag_started() || response.clicked() {
            // Grab the nearest key if the click is close to one, otherwise
            // scrub.
            let nearest = keys
                .iter()
                .enumerate()
                .min_by_key(|(_, k)| ((**k as f32 - f).abs() * 100.0) as i32);
            match nearest {
                Some((i, k)) if (*k as f32 - f).abs() < length * 0.03 => {
                    hub.key = i;
                    hub.posing = true;
                }
                _ => {
                    hub.frame = f;
                    hub.posing = false;
                }
            }
        }
        if response.dragged() && hub.posing {
            let i = hub.key;
            let target = f.round().clamp(0.0, length - 1.0) as u16;
            if hub.recipe().keys[i].frame != target {
                hub.recipe_mut().keys[i].frame = target;
                hub.recipe_mut().keys.sort_by_key(|k| k.frame);
                hub.key = hub
                    .recipe()
                    .keys
                    .iter()
                    .position(|k| k.frame == target)
                    .unwrap_or(i);
                changed = true;
            }
        }
        if hub.posing {
            hub.snap_to_key();
        }
    }

    ui.horizontal(|ui| {
        if ui.button("add key").clicked() {
            let frame = hub.frame.round() as u16;
            let pose = hub
                .baked
                .get(frame as usize)
                .copied()
                .unwrap_or_else(Pose::rest);
            // Taken from the baked frame rather than from the nearest key, so
            // inserting a key never moves anything: what you add is what was
            // already on screen.
            hub.recipe_mut().keys.push(Key::at(frame, pose));
            hub.recipe_mut().keys.sort_by_key(|k| k.frame);
            hub.recipe_mut().keys.dedup_by_key(|k| k.frame);
            hub.key = hub
                .recipe()
                .keys
                .iter()
                .position(|k| k.frame == frame)
                .unwrap_or(0);
            changed = true;
        }
        let removable = hub.recipe().keys.len() > 1;
        if ui
            .add_enabled(removable, egui::Button::new("delete key"))
            .clicked()
        {
            let i = hub.key;
            hub.recipe_mut().keys.remove(i);
            hub.key = i.saturating_sub(1);
            changed = true;
        }
        if ui.button("mirror key").clicked() {
            let i = hub.key;
            let mirrored = hub.recipe().keys[i].pose.mirrored();
            hub.recipe_mut().keys[i].pose = mirrored;
            changed = true;
        }
        if ui.button("copy to end").clicked() {
            // A looping clip almost always wants its last key to be its first.
            let pose = hub.recipe().keys[hub.key].pose;
            let last = hub.recipe().clip.length().saturating_sub(1);
            hub.recipe_mut().keys.retain(|k| k.frame != last);
            hub.recipe_mut().keys.push(Key::at(last, pose));
            hub.recipe_mut().keys.sort_by_key(|k| k.frame);
            changed = true;
        }
    });
    changed
}

fn key_editor(ui: &mut egui::Ui, hub: &mut Hub) -> bool {
    let mut changed = false;
    let Some(key) = hub.selected_key().copied() else {
        return false;
    };
    ui.label(
        egui::RichText::new(format!(
            "key {} of {}  ·  frame {}",
            hub.key + 1,
            hub.recipe().keys.len(),
            key.frame
        ))
        .strong(),
    );

    // -- the curve out of this key ------------------------------------------
    ui.collapsing("timing out of this key", |ui| {
        let mut ease = key.ease;
        ui.horizontal_wrapped(|ui| {
            for (name, preset) in Ease::PRESETS {
                if ui
                    .selectable_label(ease.preset_name() == Some(name), *name)
                    .clicked()
                {
                    ease = *preset;
                }
            }
        });
        if ease_editor(ui, &mut ease) || ease != key.ease {
            let i = hub.key;
            hub.recipe_mut().keys[i].ease = ease;
            changed = true;
        }
        ui.label(
            egui::RichText::new(
                "Left to right is time; bottom to top is how much of the change \
                 has happened. Drag the handles. Below the floor pulls back \
                 before it goes; above the ceiling carries past and returns.",
            )
            .small()
            .color(egui::Color32::from_gray(160)),
        );
    });

    // -- the pose -----------------------------------------------------------
    let mut pose = key.pose;
    ui.collapsing("pose", |ui| {
        ui.horizontal(|ui| {
            ui.label("hips, metres");
            for (i, axis) in ["x", "y", "z"].iter().enumerate() {
                let mut v = pose.channels[i];
                if ui
                    .add(
                        egui::DragValue::new(&mut v)
                            .speed(0.005)
                            .range(-0.6..=0.6)
                            .prefix(format!("{axis} ")),
                    )
                    .changed()
                {
                    pose.channels[i] = v;
                    changed = true;
                }
            }
        });
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("reach — drag the end of a limb, the joints follow")
                .small()
                .color(egui::Color32::from_gray(150)),
        );
        for (label, left, foot) in [
            ("foot L", true, true),
            ("foot R", false, true),
            ("hand L", true, false),
            ("hand R", false, false),
        ] {
            changed |= ik_handle(ui, &mut pose, label, left, foot);
        }

        for group in GROUPS {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(group.name())
                    .small()
                    .color(egui::Color32::from_gray(150)),
            );
            for joint in JOINTS {
                if joint.group() != group {
                    continue;
                }
                changed |= joint_sliders(ui, &mut pose, joint);
            }
        }
    });
    if changed {
        let i = hub.key;
        hub.recipe_mut().keys[i].pose = pose;
        hub.posing = true;
        hub.snap_to_key();
    }
    changed
}

/// Drag the end of a limb around and let the solver work out the joints.
///
/// The reason this is worth a widget of its own: a planted foot given the same
/// target on consecutive keys does not move, however much the hips do. Posing a
/// leg by its hip and knee angles means discovering, one key at a time, that the
/// foot has wandered a centimetre — and a foot that wanders is the single most
/// legible sign of animation done badly.
///
/// The numbers are metres in the character's own space: `z` is forward, `y` is
/// up, and the floor is zero.
fn ik_handle(ui: &mut egui::Ui, pose: &mut Pose, label: &str, left: bool, foot: bool) -> bool {
    let skeleton = view::pose::reference();
    let end = match (foot, left) {
        (true, true) => Joint::FootL,
        (true, false) => Joint::FootR,
        (false, true) => Joint::HandL,
        (false, false) => Joint::HandR,
    };
    let mut at = view::skeleton::solve(skeleton, pose).origin[end.index()];
    let before = at;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add_sized([64.0, 16.0], egui::Label::new(label));
        for (i, axis) in ["x", "y", "z"].iter().enumerate() {
            if ui
                .add(
                    egui::DragValue::new(&mut at[i])
                        .speed(0.005)
                        .range(-1.4..=2.2)
                        .prefix(format!("{axis} ")),
                )
                .changed()
            {
                changed = true;
            }
        }
        if changed {
            let moved = (0..3).map(|i| (at[i] - before[i]).abs()).sum::<f32>();
            if moved > 1e-5 {
                if foot {
                    view::ik::foot_to(pose, skeleton, left, at);
                } else {
                    view::ik::hand_to(pose, skeleton, left, at);
                }
            }
        }
        if foot && ui.small_button("level").clicked() {
            *pose = if left { pose.level_l() } else { pose.level_r() };
            changed = true;
        }
        if foot && ui.small_button("toe").clicked() {
            *pose = if left {
                pose.toe_floor_l()
            } else {
                pose.toe_floor_r()
            };
            changed = true;
        }
    });
    changed
}

/// One joint's sliders, in degrees, bounded by what the joint can do.
///
/// A hinge gets one slider rather than three greyed-out ones, because an elbow
/// that can also be splayed sideways is how a rig starts producing broken
/// dolls -- and an editor that offers the control implies it is meant to be
/// used.
fn joint_sliders(ui: &mut egui::Ui, pose: &mut Pose, joint: Joint) -> bool {
    let skeleton = view::pose::reference();
    let limits = skeleton.bone(joint).limits;
    let mut changed = false;
    let names: [&str; 3] = if joint.is_hinge() {
        ["bend", "", ""]
    } else {
        match joint.group() {
            Group::Legs => ["swing", "out", "roll"],
            Group::Arms => ["swing", "out", "roll"],
            _ => ["lean", "tilt", "turn"],
        }
    };
    ui.horizontal(|ui| {
        ui.add_sized([64.0, 16.0], egui::Label::new(joint.name()));
        let channels = if joint.is_hinge() { 1 } else { 3 };
        for c in 0..channels {
            let (lo, hi) = limits.channel(c);
            let mut v = pose.degrees(joint, c);
            if ui
                .add(
                    egui::DragValue::new(&mut v)
                        .speed(0.5)
                        .range(lo.to_degrees()..=hi.to_degrees())
                        .suffix(format!("° {}", names[c])),
                )
                .changed()
            {
                pose.set_degrees(joint, c, v);
                changed = true;
            }
        }
    });
    changed
}

fn looseness_editor(ui: &mut egui::Ui, hub: &mut Hub) -> bool {
    let mut changed = false;
    ui.collapsing("looseness", |ui| {
        ui.horizontal_wrapped(|ui| {
            let current = hub.recipe().looseness.name();
            for (name, preset) in Looseness::ALL {
                if ui.selectable_label(current == *name, *name).clicked() {
                    hub.recipe_mut().looseness = *preset;
                    changed = true;
                }
            }
        });
        ui.label(
            egui::RichText::new(
                "Lag is how many frames behind the keys a part runs. Ring is how \
                 far it carries past and wobbles: 1 never overshoots. Weight \
                 should read as follow-through, never as delay -- a part that \
                 arrives late is a telegraph nobody can see.",
            )
            .small()
            .color(egui::Color32::from_gray(160)),
        );
        for group in GROUPS {
            let feel = hub.recipe().looseness.group(group);
            let (mut lag, mut ring) = (feel.lag, feel.ring);
            ui.horizontal(|ui| {
                ui.add_sized([52.0, 16.0], egui::Label::new(group.name()));
                let a = ui.add(
                    egui::DragValue::new(&mut lag)
                        .speed(0.05)
                        .range(0.4..=6.0)
                        .prefix("lag "),
                );
                let b = ui.add(
                    egui::DragValue::new(&mut ring)
                        .speed(0.01)
                        .range(0.2..=1.0)
                        .prefix("ring "),
                );
                if a.changed() || b.changed() {
                    *hub.recipe_mut().looseness.group_mut(group) = anim::Feel::new(lag, ring);
                    changed = true;
                }
            });
        }
    });
    changed
}

fn saving(ui: &mut egui::Ui, hub: &mut Hub) {
    ui.horizontal(|ui| {
        let file = hub.clip.file();
        let busy = hub.baking.is_some();
        if ui
            .add_enabled(!busy, egui::Button::new(format!("save {file}.rs and bake")))
            .on_hover_text(
                "Rewrites the recipe file, then re-bakes in a fresh process -- \
                 which is also how you find out the file compiles. A few seconds.",
            )
            .clicked()
        {
            let (message, baking) = save(hub);
            hub.message = message;
            hub.baking = baking;
        }
        if ui.button("reload from disk").clicked() {
            hub.recipes = load_recipes();
            hub.rebake();
            hub.message = "reloaded".into();
        }
    });
    let mut notes = hub.recipe().notes.clone();
    ui.label(
        egui::RichText::new("notes -- why this clip is shaped the way it is")
            .small()
            .color(egui::Color32::from_gray(150)),
    );
    if ui.text_edit_multiline(&mut notes).changed() {
        hub.recipe_mut().notes = notes;
    }
    if !hub.message.is_empty() {
        ui.label(egui::RichText::new(&hub.message).small());
    }
}

/// Collect a finished background bake, if there is one.
pub fn collect_bake(mut hub: ResMut<Hub>) {
    let Some(slot) = hub.baking.clone() else {
        return;
    };
    let done = slot.lock().ok().and_then(|mut m| m.take());
    if let Some(message) = done {
        hub.message = message;
        hub.baking = None;
    }
}

/// Write the selected clip's file back out, then re-bake the whole table.
///
/// The write is immediate; the bake runs in a thread, because it shells out to
/// a fresh `cargo run` and that takes seconds. Running it inline would freeze
/// the arena in the middle of the animation you were judging.
type Pending = std::sync::Arc<std::sync::Mutex<Option<String>>>;

fn save(hub: &Hub) -> (String, Option<Pending>) {
    let file = hub.clip.file();
    let mine: Vec<Recipe> = hub
        .recipes
        .iter()
        .filter(|r| r.clip.file() == file && !r.keys.is_empty())
        .cloned()
        .collect();
    let title = format!("{}.", hub.clip.family().name());
    let source = anim::source::emit_file(file, &title, &mine);

    let root = crate::bake::repo_root();
    let path = root.join(format!("crates/anim/src/clips/{file}.rs"));
    if let Err(e) = std::fs::write(&path, source) {
        return (format!("could not write {}: {e}", path.display()), None);
    }

    let slot: Pending = std::sync::Arc::new(std::sync::Mutex::new(None));
    let done = slot.clone();
    std::thread::spawn(move || {
        let _ = std::process::Command::new("rustfmt")
            .args(["--edition", "2024"])
            .arg(&path)
            .current_dir(&root)
            .output();
        // Baked from the *files*, not from what is in memory, so a save that
        // produced source the compiler rejects says so here rather than at the
        // next build.
        let out = std::process::Command::new("cargo")
            .args(["run", "-q", "-p", "anim", "--bin", "bake"])
            .current_dir(&root)
            .output();
        let message = match out {
            Ok(o) if o.status.success() => {
                "saved and re-baked. Rebuild to see it outside the hub.".to_string()
            }
            Ok(o) => format!(
                "saved, but the bake failed: {}",
                String::from_utf8_lossy(&o.stderr)
                    .lines()
                    .last()
                    .unwrap_or("")
            ),
            Err(e) => format!("saved, but could not run the bake: {e}"),
        };
        if let Ok(mut slot) = done.lock() {
            *slot = Some(message);
        }
    });
    (format!("wrote {file}.rs; baking…"), Some(slot))
}

// ---------------------------------------------------------------------------
// The curve widget
// ---------------------------------------------------------------------------

/// Two draggable handles over a cubic Bézier from (0,0) to (1,1).
///
/// The same four numbers as a CSS easing, and the same four the Oven's shaped
/// curves use. Four because it is the smallest thing that can express
/// hold-then-burst and still be edited by dragging two points.
fn ease_editor(ui: &mut egui::Ui, ease: &mut Ease) -> bool {
    let size = egui::vec2(ui.available_width().min(220.0), 150.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 3.0, egui::Color32::from_gray(26));

    // Room above and below for anticipation and overshoot.
    let (lo, hi) = (-0.45f32, 1.45f32);
    let to_screen = |x: f32, y: f32| {
        egui::pos2(
            rect.left() + rect.width() * x,
            rect.bottom() - rect.height() * (y - lo) / (hi - lo),
        )
    };
    let from_screen = |p: egui::Pos2| {
        (
            ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0),
            lo + (rect.bottom() - p.y) / rect.height() * (hi - lo),
        )
    };

    // The unit box, so "all of it" and "none of it" are visible lines.
    for y in [0.0, 1.0] {
        painter.line_segment(
            [to_screen(0.0, y), to_screen(1.0, y)],
            egui::Stroke::new(1.0_f32, egui::Color32::from_gray(60)),
        );
    }

    let points: Vec<egui::Pos2> = (0..=48)
        .map(|i| {
            let t = i as f32 / 48.0;
            to_screen(t, ease.at(t))
        })
        .collect();
    painter.add(egui::Shape::line(
        points,
        egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(120, 230, 200)),
    ));

    let handles = [(ease.x1, ease.y1), (ease.x2, ease.y2)];
    for (i, (x, y)) in handles.iter().enumerate() {
        let anchor = if i == 0 {
            to_screen(0.0, 0.0)
        } else {
            to_screen(1.0, 1.0)
        };
        let p = to_screen(*x, *y);
        painter.line_segment(
            [anchor, p],
            egui::Stroke::new(1.0_f32, egui::Color32::from_gray(90)),
        );
        painter.circle_filled(p, 5.0, egui::Color32::from_rgb(250, 220, 120));
    }

    let mut changed = false;
    if let Some(pos) = response.interact_pointer_pos() {
        if response.dragged() || response.clicked() {
            let (x, y) = from_screen(pos);
            // Whichever handle is nearer in time; a handle is a point in time
            // first and a shape second.
            let first = (x - ease.x1).abs() <= (x - ease.x2).abs();
            if first {
                ease.x1 = x;
                ease.y1 = y;
            } else {
                ease.x2 = x;
                ease.y2 = y;
            }
            changed = true;
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_clip_can_be_opened_in_the_hub() {
        // Including the ones nobody has authored. An empty clip is work
        // outstanding, and the hub is where it gets done -- refusing to open it
        // would mean the only way to start a clip is a text editor.
        let hub = Hub::default();
        for clip in ALL {
            assert!(
                hub.recipes.iter().any(|r| r.clip == *clip),
                "{} cannot be opened",
                clip.name()
            );
        }
        assert_eq!(hub.recipes.len(), ALL.len(), "a clip is in the list twice");
    }

    #[test]
    fn a_closed_hub_drives_nobody() {
        let mut hub = Hub::default();
        assert!(hub.preview_for(0).is_none());
        assert!(hub.preview_for(1).is_none());
        hub.open = true;
        assert!(hub.preview_for(hub.on).is_some());
        assert!(hub.preview_for(1 - hub.on).is_none());
    }

    #[test]
    fn selecting_a_clip_bakes_it_to_its_own_length() {
        let mut hub = Hub::default();
        for clip in [Clip::Idle, Clip::WalkForward, Clip::BulwarkCommitted] {
            hub.clip = clip;
            hub.rebake();
            assert_eq!(
                hub.baked.len(),
                clip.length() as usize,
                "{} baked wrong",
                clip.name()
            );
        }
    }

    #[test]
    fn editing_a_key_changes_what_is_drawn() {
        // The whole argument for the hub is that the loop closes in a frame.
        // If an edit does not reach the preview, it does not.
        let mut hub = Hub::default();
        hub.open = true;
        hub.clip = Clip::Idle;
        hub.rebake();
        let before = hub.preview_for(hub.on).expect("previewing");
        let i = hub.key;
        hub.recipe_mut().keys[i].pose = Pose::rest().shoulders(90.0, 20.0, 0.0);
        hub.rebake();
        let after = hub.preview_for(hub.on).expect("previewing");
        assert!(
            before.separation(&after) > 0.5,
            "the edit did not reach the preview"
        );
    }

    #[test]
    fn holding_a_key_shows_that_key_rather_than_the_solved_frame() {
        // Posing means seeing the pose you are editing, not the spring's
        // opinion of it two frames later.
        let mut hub = Hub::default();
        hub.open = true;
        hub.clip = Clip::Idle;
        hub.rebake();
        hub.posing = true;
        hub.key = 1;
        let shown = hub.preview_for(hub.on).expect("previewing");
        assert_eq!(shown, hub.recipe().keys[1].pose);
    }
}
