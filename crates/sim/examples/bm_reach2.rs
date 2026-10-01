use sim::state::{MAX_PLAYERS, hitbox};
use sim::{Class, Input, World};
fn f(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let class = match args[1].as_str() {
        "bulwark" => Class::Bulwark,
        "champion" => Class::Champion,
        "reaver" => Class::ShadowReaver,
        "blood" => Class::BloodMage,
        "dual" => Class::DualMage,
        _ => Class::Elementalist,
    };
    let button = if args[2] == "L" {
        Input::LEFT
    } else if args[2] == "R" {
        Input::RIGHT
    } else {
        Input::SPECIAL
    };
    let pitch: i16 = args[3].parse().unwrap();
    let jump = args.get(4).is_some();
    let mut w = World::with_classes([class; MAX_PLAYERS]);
    w.players[1].pos.x = sim::Fx::from_int(-12);
    for _ in 0..60 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let base = w.players[0].pos;
    let mut pressed = false;
    for t in 0..200 {
        let mut i = Input {
            pitch,
            ..Input::default()
        };
        if jump && t < 30 {
            i = i.with(Input::SPACE);
        }
        let at_apex = jump && t > 3 && w.players[0].vel.y.raw() <= 0 && !pressed;
        if (!jump && t == 5) || at_apex {
            i = i.with(button);
            pressed = true;
        }
        w.advance([i, Input::default()]);
        if let Some(hb) = hitbox(&w.players[0]) {
            let feet = w.players[0].pos.y.sub(base.y);
            println!(
                "t{t:3} feet {:.2} from ({:.2},{:.2},{:.2}) to ({:.2},{:.2},{:.2}) r {:.2}",
                f(feet),
                f(hb.from.x.sub(base.x)),
                f(hb.from.y.sub(base.y)),
                f(hb.from.z.sub(base.z)),
                f(hb.to.x.sub(base.x)),
                f(hb.to.y.sub(base.y)),
                f(hb.to.z.sub(base.z)),
                f(hb.radius)
            );
        }
    }
}
