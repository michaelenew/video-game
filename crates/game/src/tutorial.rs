//! **The tutorial halls, on the screen** (`sim::tutorial`): the room you are
//! standing in, what it is for, and a checklist of what to do in it -- a box
//! ticked the first time, a count beside it every time after -- with a word
//! on whether the last thing you did landed.
//!
//! Everything it says is read off the simulation: which room is where you
//! stand, and whether a task is done is the tutorial's own question of two
//! frames, asked once a tick ([`observe`]). What this module adds is the
//! panel, the dummy that swings on a count in a room that wants one, and
//! keeping you in the hall of the class you are playing -- Tab to another
//! class walks you into its hall.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use sim::tutorial::{self, Dummy, Progress, Tutorial, Word};
use sim::{Class, World};

/// What this player has done in the halls, and a hall asked for that waits
/// on a class change to arrive first.
#[derive(Resource, Default)]
pub struct Coach {
    pub progress: Progress,
    /// The menu asked for this class's hall: the class first, then the trip.
    pub want: Option<Class>,
    /// The newest frame tallied: online a rollback makes a frame again, and
    /// it is counted the first time (as it is sounded).
    through: Option<u32>,
}

/// One tick of the halls: what this machine's fighter, in seat `me`, did --
/// their own tally, whoever else is in the hall (2026-10-25). Each frame
/// once, the first time it is made.
pub fn observe(coach: &mut Coach, before: &World, after: &World, me: usize) {
    if coach
        .through
        .is_some_and(|f| after.frame <= f && after.frame + 600 > f)
    {
        return;
    }
    coach.through = Some(after.frame);
    coach.progress.observe(before, after, me);
}

/// **Keep the hall and the class together**: playing a class in another
/// class's hall walks you into your own, and a hall the menu asked for
/// follows the class change it waited on.
pub fn steer(sim: &mut crate::Sim, coach: &mut Coach) {
    if sim.travel != sim::input::Travel::NONE {
        return;
    }
    let me = sim.local_player();
    let class = sim.cur.players[me].class;
    if let Some(want) = coach.want {
        if class == want {
            coach.want = None;
            if let Some(t) = tutorial::for_class(want) {
                sim.travel = sim::input::Travel::arena(t.arena);
            }
        }
        return;
    }
    if let Some((here, _)) = sim.cur.hall_of(me)
        && here.class != class
        && let Some(t) = tutorial::for_class(class)
    {
        sim.travel = sim::input::Travel::arena(t.arena);
    }
}

/// **The Esc menu's tutorials**: a hall for every class on offer.
pub fn menu(ui: &mut egui::Ui, sim: &mut crate::Sim, coach: &mut Coach) {
    ui.heading("Tutorials");
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        for class in sim::class::ROTATION {
            let Some(t) = tutorial::for_class(class) else {
                continue;
            };
            let me = sim.local_player();
            let here = sim.cur.hall_of(me).is_some_and(|(u, _)| u.arena == t.arena);
            let (done, of) = coach.progress.rooms_done(t, false);
            let label = if of > 0 && done == of {
                format!("{} ✔", class.name())
            } else {
                class.name().to_string()
            };
            if ui.selectable_label(here, label).clicked() && !here {
                if sim.cur.players[me].class == class {
                    sim.travel = sim::input::Travel::arena(t.arena);
                } else {
                    sim.travel = sim::input::Travel::class(me, class);
                    coach.want = Some(class);
                }
            }
        }
    });
    ui.label(
        egui::RichText::new(
            "A hall for each class: what it is, every click, and past the basics the advanced rooms.",
        )
        .weak(),
    );
}

const GOOD: egui::Color32 = egui::Color32::from_rgb(110, 200, 120);
const WARN: egui::Color32 = egui::Color32::from_rgb(235, 170, 80);
const ADVANCED: egui::Color32 = egui::Color32::from_rgb(200, 150, 255);
const BOX: egui::Color32 = egui::Color32::from_rgb(70, 74, 82);

/// The checkbox: an empty square, or a filled one with a tick in it.
fn checkbox(ui: &mut egui::Ui, done: bool, fresh: bool) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
    let p = ui.painter();
    let fill = if done {
        GOOD
    } else {
        egui::Color32::TRANSPARENT
    };
    p.rect_filled(rect, 4.0, fill);
    let edge = if fresh {
        egui::Color32::WHITE
    } else if done {
        GOOD
    } else {
        BOX
    };
    p.rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(2.0_f32, edge),
        egui::StrokeKind::Inside,
    );
    if done {
        let c = rect.center();
        let stroke = egui::Stroke::new(3.0_f32, egui::Color32::from_rgb(20, 40, 24));
        p.line_segment(
            [c + egui::vec2(-6.0, 0.0), c + egui::vec2(-2.0, 5.0)],
            stroke,
        );
        p.line_segment(
            [c + egui::vec2(-2.0, 5.0), c + egui::vec2(6.0, -5.0)],
            stroke,
        );
    }
}

/// The count of times past the first: a badge that says it was done again.
fn again(ui: &mut egui::Ui, times: u16, fresh: bool) {
    if times < 2 {
        return;
    }
    let colour = if fresh { egui::Color32::WHITE } else { GOOD };
    ui.label(
        egui::RichText::new(format!("×{times}"))
            .strong()
            .size(16.0)
            .color(colour),
    );
}

/// **The panel**: drawn while you are in a hall.
pub fn draw(
    mut contexts: EguiContexts,
    sim: Res<crate::Sim>,
    coach: Res<Coach>,
    menu: Res<crate::menu::Menu>,
) {
    let Some((t, local)) = sim.cur.hall_of(sim.local_player()) else {
        return;
    };
    if menu.open {
        return;
    }
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    let w = &sim.cur;
    let k = tutorial::room_at(t, local.x);
    egui::Window::new("tutorial")
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-18.0, 150.0))
        .resizable(false)
        .default_width(380.0)
        .max_width(380.0)
        .show(ctx, |ui| panel(ui, t, k, w, &coach.progress));
}

fn panel(ui: &mut egui::Ui, t: &Tutorial, k: usize, w: &World, progress: &Progress) {
    let room = &t.rooms[k];
    let (basic, basics) = progress.rooms_done(t, false);
    let (adv, advs) = progress.rooms_done(t, true);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{} tutorial", t.class.name()))
                .small()
                .weak(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "room {} of {} · basics {basic}/{basics} · advanced {adv}/{advs}",
                    k + 1,
                    t.rooms.len()
                ))
                .small()
                .weak(),
            );
        });
    });
    if room.advanced {
        ui.label(
            egui::RichText::new("ADVANCED TECHNIQUE")
                .small()
                .strong()
                .color(ADVANCED),
        );
    }
    ui.heading(room.title);
    ui.add_space(2.0);
    if k == 0 {
        for (head, line) in [
            ("What it is", t.what),
            ("How it plays", t.plays),
            ("How it wins", t.wins),
        ] {
            ui.label(egui::RichText::new(head).strong());
            ui.label(line);
            ui.add_space(2.0);
        }
        ui.separator();
    }
    ui.label(room.intro);
    ui.add_space(6.0);

    let said = progress.saying(w.frame);
    let fresh = |task: usize| {
        matches!(said, Some(Word::Done { room: r, task: x, .. })
            if r as usize == k && x as usize == task)
    };
    for (i, task) in room.tasks.iter().enumerate() {
        let times = progress.times(t.class, k, i);
        ui.horizontal(|ui| {
            checkbox(ui, times > 0, fresh(i));
            ui.vertical(|ui| {
                let what = egui::RichText::new(task.what).strong();
                ui.label(if times > 0 { what.color(GOOD) } else { what });
                ui.label(egui::RichText::new(task.keys).weak());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                again(ui, times, fresh(i));
            });
        });
        ui.add_space(2.0);
    }

    // What the last thing you did came to.
    match said {
        Some(Word::Done {
            room: r,
            task,
            times,
        }) if r as usize == k => {
            let what = room.tasks.get(task as usize).map_or("", |x| x.what);
            let line = match (
                times,
                room.tasks.get(task as usize).is_some_and(|x| x.lands),
            ) {
                (1, true) => format!("✔ Hit! {what}: done"),
                (1, false) => format!("✔ {what}: done"),
                (n, true) => format!("✔ Hit again: {what} ×{n}"),
                (n, false) => format!("✔ Again: {what} ×{n}"),
            };
            ui.label(egui::RichText::new(line).strong().color(GOOD));
        }
        Some(Word::Missed { room: r, task }) if r as usize == k => {
            let what = room.tasks.get(task as usize).map_or("", |x| x.what);
            ui.label(
                egui::RichText::new(format!("✘ {what}: that missed the dummy -- try again"))
                    .strong()
                    .color(WARN),
            );
        }
        _ => {}
    }
    if room.dummy == Dummy::Swings {
        // In tenths: the clash starts at 0.2 (`sim::tutorial::dummy_input`),
        // and a count in whole seconds cannot show it.
        let left = tutorial::SWING_EVERY - w.frame % tutorial::SWING_EVERY;
        let line = format!(
            "The dummy thrusts in {:.1} s",
            left as f32 / sim::TICK_HZ as f32
        );
        ui.label(egui::RichText::new(line).strong().size(16.0).color(WARN));
    }
    ui.add_space(4.0);
    if progress.room_done(t, k) {
        let line = if k + 1 == t.rooms.len() {
            "Every room in the hall is done."
        } else if !room.advanced && t.rooms[k + 1].advanced {
            "Room done. That is the basics -- the advanced rooms are ahead."
        } else {
            "Room done -- practise more, or walk on to the next room."
        };
        ui.label(egui::RichText::new(line).color(GOOD));
    }
    ui.label(
        egui::RichText::new("Tab: another class's hall · Esc: the menu")
            .small()
            .weak(),
    );
}
