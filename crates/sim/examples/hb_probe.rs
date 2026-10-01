// Scratch probe for the Hornback while building: not committed.
use sim::species::SpeciesId;
use sim::species::hornback as h;
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, World};

fn main() {
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::HORNBACK);
    w.players[1].health = 0;
    let steps: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(600);
    let walk: bool = std::env::args().nth(2).is_some();
    for f in 0..steps {
        let mut inp = [Input::default(); MAX_PLAYERS];
        if walk {
            // walk east (aim 0 = +x?), W
            inp[0] = Input::aimed(Input::W, 0);
        }
        w.advance(inp);
        if f % 30 == 0 {
            let p = w.pack.unwrap();
            let c = &w.critters;
            print!(
                "f{} mood {} herd {:?} p0 ({:.1},{:.1}) hp {} |",
                w.frame,
                p.mood,
                h::herd_state(&p),
                w.players[0].pos.x.to_f32_for_render(),
                w.players[0].pos.z.to_f32_for_render(),
                w.players[0].health
            );
            for (i, k) in c.iter().enumerate().filter(|(_, k)| k.present()) {
                print!(
                    " {}:{}{} s{} a{} ({:.1},{:.1})",
                    i,
                    if k.kind == 1 { "B" } else { "c" },
                    k.health,
                    k.state,
                    k.act,
                    k.pos.x.to_f32_for_render(),
                    k.pos.z.to_f32_for_render()
                );
            }
            println!();
        }
    }
}
