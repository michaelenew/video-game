use std::mem::size_of;
#[test]
fn sizes() {
    use sim::*;
    println!("World {}", size_of::<World>());
    println!("players {}", size_of::<[sim::state::Player; 2]>());
    println!("Player {}", size_of::<sim::state::Player>());
    println!(
        "effects {}",
        size_of::<[Option<sim::effects::Effect>; sim::effects::MAX_EFFECTS]>()
    );
    println!("bolts {}", size_of::<sim::bolt::Flight>());
    println!("debris {}", size_of::<sim::debris::Shrapnel>());
    println!("gusts {}", size_of::<sim::gust::Flight>());
    println!("herd {}", size_of::<sim::monster::Herd>());
    println!("Monster {}", size_of::<Monster>());
    println!("critters {}", size_of::<sim::critter::Critters>());
    println!("pack {}", size_of::<Option<sim::pack::Pack>>());
    println!("phase {}", size_of::<sim::state::Phase>());
}
