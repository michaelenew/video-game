// Scratch: watch a Hornback crossing. Not committed.
use hunt::{Hunter, plans};
use sim::arena::ArenaId;
use sim::species::SpeciesId;
use sim::species::hornback as h;
use sim::state::{MAX_PLAYERS, Phase};
use sim::{Class, Input, World};

fn main() {
    let seed: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x2545_F491);
    let class = match std::env::args().nth(2).as_deref() {
        Some("ele") => Class::Elementalist,
        Some("bul") => Class::Bulwark,
        _ => Class::Champion,
    };
    let every: u32 = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let card = plans::card(SpeciesId::HORNBACK).unwrap();
    let mut w = World::hunt_in(
        [class; MAX_PLAYERS],
        [Some(SpeciesId::HORNBACK), None],
        ArenaId::HORNBACK_CROSSING,
    );
    if let Some(p) = w.pack.as_mut() {
        p.rng = seed | 1;
    }
    w.players[1].health = 0;
    let mut bot = Hunter::of(card, 0, seed, hunt::jump_apex(class));
    let f = |x: sim::Fx| x.to_f32_for_render();
    while w.frame < 20000 && matches!(w.phase, Phase::Fighting) {
        bot.watch(&w);
        let i = bot.act(&w);
        let before = w.clone();
        w.advance([i, Input::default()]);
        let b = (0..10).find(|k| w.critters[*k].kind == h::BULL).unwrap();
        let c = w.critters[b];
        let p = &w.players[0];
        let cart = h::rules::the_cart(&w);
        let cb = h::rules::the_cart(&before);
        let hit = match (cart, cb) {
            (Some(a), Some(b)) => a.state.taken - b.state.taken,
            (None, Some(b)) => b.state.health,
            _ => 0,
        };
        let started = c.state == 2 && before.critters[b].state != 2;
        if w.frame % every == 0 || hit > 0 || started {
            let pk = w.pack.unwrap();
            println!(
                "f{} {:?} p({:.1},{:.1}) | bull({:.1},{:.1}) s{} a{} q{:?} mood{} herd{:?} | cart {:?} hp {:?} took {} | wave {:?} home({:.1},{:.1})",
                w.frame,
                bot.intent(),
                f(p.pos.x),
                f(p.pos.z),
                f(c.pos.x),
                f(c.pos.z),
                c.state,
                c.act,
                h::quarry(&pk, &w.terrain()).map(|q| f(q.x)),
                pk.mood,
                h::herd_state(&pk),
                cart.map(|o| f(o.at.x)),
                cart.map(|o| o.state.health),
                hit,
                h::rules::wave_lane(&w).map(|l| f(l.0.x)),
                f(pk.home.x),
                f(pk.home.z)
            );
        }
    }
}
