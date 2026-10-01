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

fn show(w: &World, f: u32) {
    let m = beast(w);
    println!(
        "{f:5} pos ({:6.1} {:5.1} {:6.1}) yaw {:5.2} bank {:5.2} doing {:?} aloft {} perched {} wind {:5.1} rest {} ride {:?} p0 ({:5.1} {:5.1} {:5.1}) hp {} mount {}",
        m.pos.x.to_f32_for_render(),
        m.pos.y.to_f32_for_render(),
        m.pos.z.to_f32_for_render(),
        m.yaw.to_f32_for_render(),
        fight::bank(&m).to_f32_for_render(),
        m.doing,
        fight::aloft(&m),
        fight::perched(&m),
        fight::wind(&w.lore).to_f32_for_render(),
        w.lore.word(fight::word::REST),
        fight::ride_state(&w.lore),
        w.players[0].pos.x.to_f32_for_render(),
        w.players[0].pos.y.to_f32_for_render(),
        w.players[0].pos.z.to_f32_for_render(),
        w.players[0].health,
        w.players[0].mount,
    );
}

#[test]
#[ignore]
fn probe_perch() {
    let mut w = hunt();
    // Far away and out of its reach, so it only circles.
    w.advance([Input::default(); MAX_PLAYERS]);
    fight::set_fx(&mut w.lore, fight::word::WIND, Fx::from_int(15));
    for f in 0..900 {
        w.players[0].pos = sim::V3::new(Fx::from_int(-25), Fx::ZERO, Fx::from_int(-25));
        w.advance([Input::default(); MAX_PLAYERS]);
        if f % 20 == 0 {
            show(&w, f);
        }
    }
}

#[test]
#[ignore]
fn probe_crash_ride() {
    use sim::monster::Doing;
    use sim::species::galewing as gw;
    let mut w = hunt();
    let mut f = 0u32;
    // Until a pass or a stoop puts it low.
    loop {
        w.advance([Input::default(); MAX_PLAYERS]);
        f += 1;
        let m = beast(&w);
        if fight::flags(&m) & fight::flag::LOW != 0 && fight::aloft(&m) {
            break;
        }
        assert!(f < 3000);
    }
    show(&w, f);
    let m = w.monsters[0].as_mut().unwrap();
    m.take_hit(gw::BLADE_L, 950);
    println!("after hit: {:?} poise {}", m.doing, m.poise);
    let mut placed = false;
    for g in 0..2400 {
        let m = beast(&w);
        if !placed && fight::crashed(&m) {
            let rig = m.rig();
            let top =
                rig.part_to_world(gw::BACK, sim::V3::new(Fx::ZERO, Fx::from_int(1), Fx::ZERO));
            w.players[0].pos = top;
            w.players[0].vel = sim::V3::ZERO;
            w.players[0].grounded = false;
            placed = true;
            println!("placed at {:?}", top);
        }
        // Brace through everything.
        let input = Input::default().with(Input::CROUCH);
        w.advance([input, Input::default()]);
        if g % 15 == 0 {
            show(&w, f + g);
        }
        if matches!(beast(&w).doing, Doing::Dead) {
            break;
        }
    }
}
