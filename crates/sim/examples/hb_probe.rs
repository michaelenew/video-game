// Scratch probe for the Hornback while building: not committed.
use sim::critter::is;
use sim::fixed::Fx;
use sim::species::SpeciesId;
use sim::species::hornback as h;
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};

fn main() {
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::HORNBACK);
    w.players[1].health = 0;
    for _ in 0..3 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let mut p = w.pack.unwrap();
    p.mood = sim::pack::mood::HUNTING;
    p.grace = 0;
    w.pack = Some(p);
    w.advance([Input::default(); MAX_PLAYERS]);
    let mut p = w.pack.unwrap();
    p.grace = 0;
    w.pack = Some(p);
    let b = (0..10).find(|i| w.critters[*i].kind == h::BULL).unwrap();
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    w.critters[b].pos = V3::new(Fx::from_int(-3), Fx::ZERO, Fx::ZERO);
    w.critters[b].yaw = 0x8000;
    let sp = w.critters.sp();
    w.critters[b].state = is::STARTUP;
    w.critters[b].act = h::HOOK;
    w.critters[b].timer = sp.attack(h::HOOK).startup;
    for f in 0..140 {
        w.advance([Input::default(); MAX_PLAYERS]);
        let c = w.critters[b];
        println!(
            "f{f} vy {:.2} fr {} p {:?} y {:.2} g {} hp {} | bull s{} a{} t{}",
            w.players[0].vel.y.to_f32_for_render(),
            w.players[0].frozen,
            w.players[0].action,
            w.players[0].pos.y.to_f32_for_render(),
            w.players[0].grounded,
            w.players[0].health,
            c.state,
            c.act,
            c.timer
        );
    }
}
