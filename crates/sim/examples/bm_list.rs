use sim::class::ALL_CLASSES;
use sim::monster::Doing;
use sim::species::SpeciesId;
use sim::species::broodmother::{self as bm, fight};
fn listed(m: &mut sim::Monster) {
    fight::only_sac(m, 1);
    for leg in [3usize, 5] {
        let s = bm::SPECIES.break_slot(bm::shin_part(leg)).unwrap();
        m.breaks[s] = 0;
    }
}
fn main() {
    let mut line = String::from("listed fore sac right:");
    for class in ALL_CLASSES {
        let r = sim::reachcheck::reach_with(
            SpeciesId::BROODMOTHER,
            Doing::Prowl,
            class,
            bm::sac_part(1),
            listed,
        );
        line += &format!(
            " {:?}:{}{}",
            class,
            if r.standing { "S" } else { "-" },
            if r.hop { "H" } else { "-" }
        );
    }
    println!("{line}");
}
