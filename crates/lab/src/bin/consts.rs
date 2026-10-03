//! The two numbers the fight contract compares against, for scripts/spread.py.
fn main() {
    let c = sim::moves::get(sim::Class::Champion, sim::state::SLOT_COMMITTED);
    let p = sim::moves::get(sim::Class::Champion, sim::state::SLOT_POKE);
    println!("opening_needed={}", c.startup + c.active);
    println!("poke_whiff={}", p.whiff_cost());
}
