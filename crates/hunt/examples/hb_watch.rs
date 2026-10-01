// Scratch: watch a Hornback hunt frame by frame. Not committed.
use hunt::{Hunter, plans};
use sim::species::SpeciesId;
use sim::species::hornback as h;
use sim::state::{MAX_PLAYERS, Phase};
use sim::{Class, Input, World};

fn main() {
    let seed: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(11);
    let class = match std::env::args().nth(2).as_deref() {
        Some("ele") => Class::Elementalist,
        Some("bul") => Class::Bulwark,
        Some("rea") => Class::ShadowReaver,
        Some("blo") => Class::BloodMage,
        Some("dua") => Class::DualMage,
        _ => Class::Champion,
    };
    let every: u32 = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let card = plans::card(SpeciesId::HORNBACK).unwrap();
    let mut w = World::hunt_of([class; MAX_PLAYERS], SpeciesId::HORNBACK);
    if let Some(p) = w.pack.as_mut() {
        p.rng = seed | 1;
    }
    w.players[1].health = 0;
    let mut bot = Hunter::of(card, 0, seed, hunt::jump_apex(class));
    while w.frame < 20000 && matches!(w.phase, Phase::Fighting) {
        bot.watch(&w);
        let i = bot.act(&w);
        let before = w.clone();
        w.advance([i, Input::default()]);
        let b = (0..10).find(|k| w.critters[*k].kind == h::BULL).unwrap();
        let c = w.critters[b];
        let p = &w.players[0];
        let lost = before.players[0].health - p.health;
        let started = c.state == 2 && before.critters[b].state != 2;
        if w.frame % every == 0
            || lost > 0
            || started
            || (h::stunned(&c) && !h::stunned(&before.critters[b]))
        {
            let pk = w.pack.unwrap();
            let (end, run, solid) = h::charge_lane(&pk);
            println!(
                "f{} {:?} p({:.1},{:.1}) hp{} {:?} | bull({:.1},{:.1}) hp{} s{} a{} t{} {}{} | lane {:.1} solid {} end({:.1},{:.1}) herd {:?}{}",
                w.frame,
                bot.intent(),
                p.pos.x.to_f32_for_render(),
                p.pos.z.to_f32_for_render(),
                p.health,
                p.action,
                c.pos.x.to_f32_for_render(),
                c.pos.z.to_f32_for_render(),
                c.health,
                c.state,
                c.act,
                c.timer,
                if h::stunned(&c) { "STUN " } else { "" },
                if started { "START" } else { "" },
                run.to_f32_for_render(),
                solid,
                end.x.to_f32_for_render(),
                end.z.to_f32_for_render(),
                h::herd_state(&pk),
                if lost > 0 {
                    format!(" LOST {lost}")
                } else {
                    String::new()
                }
            );
        }
    }
    println!("end {:?} frame {}", w.phase, w.frame);
}
