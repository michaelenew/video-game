//! Scratch: how high and how far each class's attacks reach, standing and at a hop's apex.
use sim::state::{MAX_PLAYERS, hitbox};
use sim::{Class, Input, World};

fn f(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}

fn probe(class: Class, button: u16, pitch: i16, jump: bool) -> (f32, f32, f32) {
    let mut w = World::with_classes([class; MAX_PLAYERS]);
    w.players[1].pos.x = sim::Fx::from_int(-12);
    for _ in 0..60 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let floor = w.players[0].pos.y;
    let mut top = f32::MIN;
    let mut far = 0.0f32;
    let mut apex = 0.0f32;
    let mut pressed = false;
    let base = w.players[0].pos;
    for t in 0..200 {
        let mut i = Input {
            pitch,
            ..Input::default()
        };
        i.aim = 0;
        if jump && t < 30 {
            i = i.with(Input::SPACE);
        }
        let at_apex = jump && t > 3 && w.players[0].vel.y.raw() <= 0 && !pressed;
        if (!jump && t == 5) || at_apex {
            i = i.with(button);
            pressed = true;
        }
        w.advance([i, Input::default()]);
        apex = apex.max(f(w.players[0].pos.y.sub(floor)));
        if let Some(hb) = hitbox(&w.players[0]) {
            let hi = f(hb.from.y.max(hb.to.y)) + f(hb.radius) - f(floor);
            top = top.max(hi);
            let dx = f(hb.to.x.sub(base.x))
                .abs()
                .max(f(hb.from.x.sub(base.x)).abs())
                + f(hb.radius);
            far = far.max(dx);
        }
    }
    (top, far, apex)
}

fn main() {
    for class in [
        Class::Bulwark,
        Class::Champion,
        Class::ShadowReaver,
        Class::Elementalist,
        Class::BloodMage,
        Class::DualMage,
    ] {
        for (name, b) in [("L", Input::LEFT), ("R", Input::RIGHT)] {
            for pitch in [0i16, 4096, 8192, 12000] {
                let s = probe(class, b, pitch, false);
                let j = probe(class, b, pitch, true);
                println!(
                    "{class:?} {name} pitch {pitch:5}: stand top {:.2} far {:.2} | hop apex {:.2} top {:.2} far {:.2}",
                    s.0, s.1, j.2, j.0, j.1
                );
            }
        }
    }
}
