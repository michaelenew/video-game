//! Scratch: a hunt nobody fights, printed every second.
use sim::species::SpeciesId;
use sim::species::broodmother::{self as bm, fight};
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, World};
fn main() {
    let frames: u32 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(3600);
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::BROODMOTHER);
    w.players[1].health = 0;
    let mut moves = [0u32; 16];
    let mut last = None;
    for f in 0..frames {
        w.advance([Input::default(); MAX_PLAYERS]);
        let m = w.monsters[0].unwrap();
        if let Some(k) = m.doing.attacking() {
            if last != Some(k) && matches!(m.doing, sim::monster::Doing::Startup { .. }) {
                moves[k as usize] += 1;
            }
            last = Some(k);
        } else {
            last = None;
        }
        if f % 60 == 0 {
            let sacs: String = (0..6)
                .map(|i| {
                    let (s, c) = fight::site(&w, i);
                    match s {
                        0 => {
                            if fight::red(&w, i) {
                                'R'
                            } else if c * 2 > fight::ripen(&w) {
                                'a'
                            } else {
                                'p'
                            }
                        }
                        1 => 'H',
                        2 => '_',
                        _ => 'X',
                    }
                })
                .collect();
            println!(
                "t{:4} hp {:5} p0 {:4} brood {} sacs {} doing {:?} pos ({:.1},{:.1}) phase {:?}",
                f / 60,
                m.health,
                w.players[0].health,
                fight::brood(&w),
                sacs,
                m.doing,
                m.pos.x.to_f32_for_render(),
                m.pos.z.to_f32_for_render(),
                w.phase
            );
        }
    }
    for (k, n) in moves.iter().enumerate().take(bm::MOVE_COUNT) {
        if *n > 0 {
            println!("{} x{}", bm::MOVES[k].name, n);
        }
    }
}
