//! The Pair: the rules `docs/design/creatures/the-pair.md` pins, as sentences.

use sim::monster::Doing;
use sim::species::SpeciesId;
use sim::species::pair::{self, fight};
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, World};

fn hunt() -> World {
    World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::PAIR)
}

#[test]
fn smoke_two_cats_hunt_a_standing_fighter() {
    let mut w = hunt();
    w.players[1].health = 0;
    assert!(w.monsters[0].is_some() && w.monsters[1].is_some());
    let mut last = [None; 2];
    for f in 0..2400 {
        w.advance([Input::default(); MAX_PLAYERS]);
        for s in 0..2 {
            let m = w.monsters[s].unwrap();
            let k = m.doing.attacking();
            if k != last[s] {
                if let Some(k) = k {
                    println!(
                        "{f:5} cat{s} {:<12} role {} at ({:.1},{:.1},{:.1}) hunter hp {}",
                        pair::MOVES[k as usize].name,
                        fight::role(&m),
                        m.pos.x.to_f32_for_render(),
                        m.pos.y.to_f32_for_render(),
                        m.pos.z.to_f32_for_render(),
                        w.players[0].health
                    );
                }
                last[s] = k;
            }
        }
        if f % 200 == 0 {
            for s in 0..2 {
                let m = w.monsters[s].unwrap();
                println!("   {f} cat{s} {:?} pos ({:.1},{:.1},{:.1}) speed {:.1} flags {:x} seen ({:.1},{:.1}) rooted {} grace {} st {:b}",
                    m.doing, m.pos.x.to_f32_for_render(), m.pos.y.to_f32_for_render(), m.pos.z.to_f32_for_render(),
                    m.speed.to_f32_for_render(), m.own[0], m.brain.seen.x.to_f32_for_render(), m.brain.seen.z.to_f32_for_render(), m.rooted, m.brain.grace,
                    fight::state_of(&w.lore, s));
            }
        }
        if w.players[0].health <= 0 {
            println!("hunter down at {f}");
            break;
        }
    }
    let _ = Doing::Prowl;
}
