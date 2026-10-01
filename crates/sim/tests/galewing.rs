//! The Galewing: the rules `docs/design/creatures/galewing.md` pins, as
//! sentences (§10).

use sim::fixed::Fx;
use sim::monster::Monster;
use sim::species::SpeciesId;
use sim::species::galewing::fight;
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, World};

/// A hunt against it, the second fighter out of it.
fn hunt() -> World {
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::GALEWING);
    w.players[1].health = 0;
    w
}

fn beast(w: &World) -> Monster {
    w.monsters[0].expect("the creature")
}

#[test]
#[ignore]
fn probe_flight() {
    let mut w = hunt();
    for f in 0..3600 {
        w.advance([Input::default(); MAX_PLAYERS]);
        let m = beast(&w);
        if f % 30 == 0 {
            println!(
                "{f:5} pos ({:6.1} {:5.1} {:6.1}) yaw {:5.2} doing {:?} aloft {} wind {:5.1} hp {} player ({:5.1} {:5.1} {:5.1}) php {}",
                m.pos.x.to_f32_for_render(),
                m.pos.y.to_f32_for_render(),
                m.pos.z.to_f32_for_render(),
                m.yaw.to_f32_for_render(),
                m.doing,
                fight::aloft(&m),
                fight::wind(&w.lore).to_f32_for_render(),
                m.health,
                w.players[0].pos.x.to_f32_for_render(),
                w.players[0].pos.y.to_f32_for_render(),
                w.players[0].pos.z.to_f32_for_render(),
                w.players[0].health,
            );
        }
    }
    let _ = Fx::ZERO;
}
