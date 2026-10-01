//! Scratch: a skillshot with the crosshair on each sac, from range.
use sim::monster::Doing;
use sim::reachcheck::*;
use sim::species::SpeciesId;
use sim::species::broodmother::{self as bm, fight};
use sim::{Class, Fx, Input, V3};

fn main() {
    let class = Class::Elementalist;
    let a = bm::SPECIES.attack(bm::SLAM);
    for (label, doing) in [
        ("standing", Doing::Prowl),
        (
            "slam",
            Doing::Recovery {
                kind: bm::SLAM,
                left: a.recovery - 30,
            },
        ),
    ] {
        for site in [0usize, 2, 4] {
            let mut line = format!("{label:9} sac {site}:");
            for dist in [6, 9, 12] {
                for bearing in [0i32, 8192, 16384, 24576, 32768, 49152] {
                    const ONLY: [fn(&mut sim::Monster); 6] = [
                        |m| fight::only_sac(m, 0),
                        |m| fight::only_sac(m, 1),
                        |m| fight::only_sac(m, 2),
                        |m| fight::only_sac(m, 3),
                        |m| fight::only_sac(m, 4),
                        |m| fight::only_sac(m, 5),
                    ];
                    let mut base = stage(SpeciesId::BROODMOTHER, class);
                    if std::env::args().nth(1).is_some() {
                        base.arena = sim::arena::ArenaId::PROVING_GROUND;
                    }
                    let at = centre(&base);
                    pin_with(&mut base, doing, at, ONLY[site]);
                    let target = middle(&base, bm::sac_part(site));
                    let dir = V3::from_turns(Fx::from_raw(bearing));
                    let from =
                        V3::new(target.x, Fx::ZERO, target.z).add(dir.scale(Fx::from_int(dist)));
                    let mut hit = false;
                    for button in [Input::LEFT, Input::RIGHT] {
                        let mut w = base.clone();
                        w.players[0].pos = from;
                        let slot = bm::SPECIES.break_slot(bm::sac_part(site)).unwrap();
                        let before = w.monsters[0].unwrap().breaks[slot];
                        for fr in 0..90 {
                            pin_with(&mut w, doing, at, ONLY[site]);
                            let p = &w.players[0];
                            let d = target.sub(p.pos);
                            let aim = (sim::math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16;
                            let pitch = sim::aim::look_onto_closely(p.pos, aim, p.aloft, target);
                            let bits = if fr == 3 { button } else { 0 };
                            w.advance([Input::looking_at(bits, aim, pitch), Input::default()]);
                            if w.monsters[0].unwrap().breaks[slot] < before {
                                hit = true;
                                break;
                            }
                        }
                        if hit {
                            break;
                        }
                    }
                    line += if hit { " Y" } else { " ." };
                }
                line += " |";
            }
            println!("{line}");
        }
    }
}
