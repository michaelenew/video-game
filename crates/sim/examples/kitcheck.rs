//! **The Blood mage's kit, measured against what was promised.**
//!
//!     cargo run --release -p sim --example kitcheck
//!
//! Every line is one sentence of the agreed design
//! (`docs/design/exploration/0038_blood_mage_the_world_bleeds.md`, and the
//! owner's report from play on 2026-11-06), played in the simulation the way a
//! player plays it -- the second press early, the button held -- and measured.
//! It prints what it measured beside what was promised, and `MISSED` where
//! the two disagree, and exits non-zero if any did.
//!
//! It exists because slice 1 was handed over on green tests that the builder
//! wrote from the code rather than from the promise, and on nobody playing it:
//! the zip that went "a few inches", the jump scythe lower than a jump, every
//! move still costing health. Run it before handing a change to her kit over,
//! and paste its table into the report (`docs/design/delivery.md` D3).

use sim::class::{Class, Mechanic};
use sim::effects::{EffectKind, Head};
use sim::moves::blood::{self as b, keys};
use sim::state::Action;
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn f(v: Fx) -> f32 {
    v.raw() as f32 / 65536.0
}
fn m(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}
fn at(x: f32, z: f32) -> V3 {
    V3::new(m(x), Fx::ZERO, m(z))
}
fn flat(a: V3, b: V3) -> f32 {
    let (dx, dz) = (f(a.x.sub(b.x)), f(a.z.sub(b.z)));
    (dx * dx + dz * dz).sqrt()
}

/// Her at (-6, 8) facing +x, and a Bulwark `apart` metres in front.
fn with_foe(apart: f32) -> World {
    let mut w = World::with_classes([Class::BloodMage, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 8.0);
    w.players[1].pos = at(-6.0 + apart, 8.0);
    w
}
fn step(w: &mut World, bits: u16, pitch: i16) {
    w.advance([
        Input::looking_at(bits, LOOK_RIGHT, pitch),
        Input::looking_at(0, LOOK_LEFT, 0),
    ]);
}
fn run(w: &mut World, frames: u32, bits: u16, pitch: i16) {
    for _ in 0..frames {
        step(w, bits, pitch);
    }
}
fn head(w: &World) -> Option<sim::effects::Effect> {
    w.effects
        .iter()
        .flatten()
        .find(|e| e.kind == EffectKind::ScytheHead)
        .copied()
}
fn plants(w: &World) -> usize {
    match w.players[0].mechanic {
        Mechanic::Blood(g) => g.plants.iter().flatten().count(),
        _ => 0,
    }
}
fn hold() -> u32 {
    t::hold_from() as u32 + 2
}

struct Card {
    missed: usize,
}

impl Card {
    fn line(&mut self, promise: &str, measured: String, kept: bool) {
        if !kept {
            self.missed += 1;
        }
        println!(
            "{:<7} {:<62} {}",
            if kept { "kept" } else { "MISSED" },
            promise,
            measured
        );
    }
}

fn main() {
    let mut card = Card { missed: 0 };
    println!("The Blood mage, against what was promised\n");

    // -- What costs -------------------------------------------------------
    for (name, bits, frames) in [
        ("Sweep", keys::SCYTHE, 3u32),
        ("Cast", keys::SCYTHE, hold() + 4),
        ("Rend", keys::HAND, 3),
        ("Seize", keys::HAND, hold() + 4),
        ("Rising reap", Input::SPACE | keys::SCYTHE, 3),
        ("Rising rend", Input::SPACE | keys::HAND, 3),
    ] {
        let mut w = with_foe(30.0);
        let hp = w.players[0].health;
        for fr in 0..60 {
            step(&mut w, if fr < frames { bits } else { 0 }, -2000);
        }
        let paid = hp - w.players[0].health;
        card.line(
            &format!("{name} costs her nothing"),
            format!("paid {paid}"),
            paid == 0,
        );
    }

    // -- The spike and the plant -------------------------------------------
    {
        let mut w = with_foe(30.0);
        let hp = w.players[0].health;
        step(&mut w, keys::SPIKES, -3000);
        run(&mut w, 40, 0, -3000);
        let price = hp - w.players[0].health;
        card.line(
            "a spike on bare ground raises a plant, paid in her blood",
            format!("{} plant(s), paid {price}", plants(&w)),
            plants(&w) == 1 && price > 0,
        );
        run(&mut w, 540, 0, -3000);
        let standing = plants(&w) == 1;
        card.line(
            "a plant stands at least ten seconds",
            format!("stands {} s", t::plant_stands() as f32 / 60.0),
            standing && t::plant_stands() >= 600,
        );
        card.line(
            "what a plant holds stays grey on her bar while it stands",
            format!("grey {} of {price}", w.players[0].grey),
            w.players[0].grey == price,
        );
    }
    {
        // Damage of a spike under a foe 8 m off, at the pitch that lands it.
        let mut best = 0;
        for pitch in (2000..7000).step_by(250).map(|p| -(p as i16)) {
            let mut w = with_foe(8.0);
            let his = w.players[1].health;
            step(&mut w, keys::SPIKES, pitch);
            run(&mut w, 25, 0, pitch);
            best = best.max(his - w.players[1].health);
        }
        let row = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE).damage;
        card.line(
            "a spike under them does less than the old 100",
            format!("{best} (row {row})"),
            best > 0 && best < 100,
        );
    }

    // -- Up ------------------------------------------------------------------
    let top = |bits: u16| {
        let mut w = with_foe(30.0);
        let mut top = Fx::ZERO;
        for fr in 0..90 {
            step(&mut w, if fr < 12 { bits } else { 0 }, 0);
            top = top.max(w.players[0].pos.y);
        }
        f(top)
    };
    let jump = top(Input::SPACE);
    let reap = top(Input::SPACE | keys::SCYTHE);
    let spike = top(Input::SPACE | keys::SPIKES);
    card.line(
        "the Rising reap takes her far higher than a jump (2x)",
        format!("{reap:.1} m against a jump's {jump:.1} m"),
        reap > jump * 2.0,
    );
    card.line(
        "the spike under her own feet launches her (2x a jump)",
        format!("{spike:.1} m"),
        spike > jump * 2.0,
    );

    // -- The scythe's head ---------------------------------------------------
    {
        let mut w = with_foe(30.0);
        let mut out_at = None;
        for fr in 0..60u32 {
            step(&mut w, keys::SCYTHE, -3000);
            if head(&w).is_some() {
                out_at = Some(fr + 1);
                break;
            }
        }
        let frames = out_at.unwrap_or(999);
        card.line(
            "held, the head leaves her hand within a third of a second",
            format!("{frames} frames from the press"),
            frames <= 20,
        );
    }
    for (when, pressed) in [("before it leaves", 0u32), ("while it flies", 6)] {
        let mut w = with_foe(30.0);
        let from = w.players[0].pos;
        run(&mut w, hold(), keys::SCYTHE, -3000);
        step(&mut w, 0, -3000);
        run(&mut w, pressed, 0, -3000);
        step(&mut w, keys::SCYTHE, -3000);
        let mut bit = None;
        for _ in 0..90 {
            step(&mut w, 0, -3000);
            if let Some(h) = head(&w).filter(|h| h.head_state() != Head::Flying) {
                bit.get_or_insert(h.home);
            }
        }
        let out = bit.map_or(0.0, |b| flat(from, b));
        let gone = flat(from, w.players[0].pos);
        card.line(
            &format!("the second press {when} zips her all the way"),
            format!("{gone:.1} m of {out:.1} m"),
            out > 3.0 && gone >= out - 2.0,
        );
    }
    {
        let mut w = with_foe(8.0);
        run(&mut w, hold(), keys::SCYTHE, 0);
        step(&mut w, 0, 0);
        step(&mut w, keys::SCYTHE, 0);
        run(&mut w, 60, 0, 0);
        let gap = flat(w.players[0].pos, w.players[1].pos);
        let reach = f(sim::moves::get(Class::BloodMage, b::REND).reach);
        card.line(
            "a zip onto a fighter leaves her in the Red Hand's reach",
            format!("{gap:.1} m apart, reach {reach:.1} m"),
            gap < reach,
        );
    }
    {
        let mut w = with_foe(60.0);
        run(&mut w, hold(), keys::SCYTHE, 3000);
        let mut lowest = f32::MAX;
        for _ in 0..150 {
            step(&mut w, 0, 3000);
            if let Some(h) = head(&w).filter(|h| h.head_state() == Head::Home) {
                lowest = lowest.min(f(h.home.y));
            }
        }
        card.line(
            "a head that bit nothing in the air comes home through the air",
            format!("lowest {lowest:.1} m on the way home"),
            lowest > 0.8 && lowest < f32::MAX,
        );
    }

    // -- The Red Hand ---------------------------------------------------------
    {
        let mut w = with_foe(2.2);
        let his = w.players[1].health;
        for fr in 0..70u32 {
            let press = matches!(fr, 0 | 16 | 32);
            step(&mut w, if press { keys::HAND } else { 0 }, 0);
        }
        let dealt = his - w.players[1].health;
        let reach = f(sim::moves::get(Class::BloodMage, b::REND).reach);
        card.line(
            "Rend is big: three claws from 2 m away, 150 or more",
            format!("{dealt} dealt, reach {reach:.1} m"),
            dealt >= 150,
        );
    }
    {
        let mut w = with_foe(1.8);
        let his = w.players[1].health;
        let mut highest = 0.0f32;
        for fr in 0..80u32 {
            step(&mut w, if fr < hold() + 2 { keys::HAND } else { 0 }, 0);
            highest = highest.max(f(w.players[1].pos.y));
            if matches!(w.players[1].action, Action::HitStun { .. }) && highest > 0.0 {
                break;
            }
        }
        run(&mut w, 10, 0, 0);
        let dealt = his - w.players[1].health;
        card.line(
            "Seize lifts them over her and slams them, 100 or more",
            format!("lifted {highest:.1} m, {dealt} dealt"),
            highest > 1.0 && dealt >= 100,
        );
    }

    println!(
        "\n{} promise(s) missed",
        if card.missed == 0 {
            "no".to_string()
        } else {
            card.missed.to_string()
        }
    );
    if card.missed > 0 {
        std::process::exit(1);
    }
}
