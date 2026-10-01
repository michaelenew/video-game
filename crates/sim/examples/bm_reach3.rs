use sim::class::ALL_CLASSES;
use sim::monster::Doing;
use sim::species::SpeciesId;
use sim::species::broodmother::{self as bm, fight};
fn main() {
    let a = bm::SPECIES.attack(bm::SLAM);
    let states = [
        ("standing", Doing::Prowl),
        (
            "slam",
            Doing::Recovery {
                kind: bm::SLAM,
                left: a.recovery - 30,
            },
        ),
    ];
    for (label, doing) in states {
        println!("== {label}");
        for site in [0usize, 2, 4] {
            let part = bm::sac_part(site);
            let mut line = format!("  {:16}", bm::PARTS[part].name);
            for class in ALL_CLASSES {
                const ONLY: [fn(&mut sim::Monster); 6] = [
                    |m| fight::only_sac(m, 0),
                    |m| fight::only_sac(m, 1),
                    |m| fight::only_sac(m, 2),
                    |m| fight::only_sac(m, 3),
                    |m| fight::only_sac(m, 4),
                    |m| fight::only_sac(m, 5),
                ];
                let r = sim::reachcheck::reach_with(
                    SpeciesId::BROODMOTHER,
                    doing,
                    class,
                    part,
                    ONLY[site],
                );
                line += &format!(
                    " {:?}:{}{}",
                    class,
                    if r.standing { "S" } else { "-" },
                    if r.hop { "H" } else { "-" }
                );
                if let Some(p) = r.hop_from {
                    line += &format!(
                        "({:.1},{:.1})",
                        p.x.to_f32_for_render(),
                        p.z.to_f32_for_render()
                    );
                }
            }
            println!("{line}");
        }
    }
}
