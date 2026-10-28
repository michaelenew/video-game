//! Write a creature's skeleton and part boxes at rest, for the art build.
//!
//!     cargo run -p anim --bin export_beast -- ridgeback target/art/ridgeback.json
//!
//! A creature drawn from a model (`art/<species>/build.py`) is sculpted over
//! the same rest pose the game's grown body is skinned to: every bone placed
//! by `Rig::build` at rest, at the species' tuned scale, standing at the
//! origin facing `+x`. The model's joints are these bones, so the game poses
//! it with the transforms it already sets for the grown body.
//!
//! Metres, the game's frame (`+x` toward the head, `+y` up, `+z` its right).

use std::fmt::Write as _;

fn f(x: sim::Fx) -> f32 {
    x.to_f32_for_render()
}

fn v3(v: sim::V3) -> String {
    format!("[{:.4},{:.4},{:.4}]", f(v.x), f(v.y), f(v.z))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let name = args.first().map_or("ridgeback", String::as_str);
    let sp = sim::species::named(name).unwrap_or_else(|| {
        eprintln!("no species is called {name}");
        std::process::exit(2);
    });
    let out = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| format!("target/art/{}.json", sp.slug()));
    let rig = sim::beast::Rig::build(
        sp,
        sim::V3::ZERO,
        sim::Fx::ZERO,
        &sim::beast::Pose::rest(sp.bones.len()),
    );
    let mut s = String::new();
    let _ = write!(s, "{{\"species\":\"{}\",\n\"bones\":[", sp.name);
    for (i, b) in sp.bones.iter().enumerate() {
        let p = &rig.bone[i];
        let parent = if b.parent == sim::beast::NO_PARENT {
            -1
        } else {
            b.parent as i64
        };
        let _ = write!(
            s,
            "{}\n{{\"name\":\"{}\",\"parent\":{parent},\"side\":{},\"at\":{},\"x\":{},\"y\":{},\"z\":{}}}",
            if i == 0 { "" } else { "," },
            b.name,
            b.side,
            v3(p.at),
            v3(p.rot.r[0]),
            v3(p.rot.r[1]),
            v3(p.rot.r[2]),
        );
    }
    s.push_str("],\n\"parts\":[");
    for (i, part) in sp.parts.iter().enumerate() {
        let sh = sp.shape(i);
        let _ = write!(
            s,
            "{}\n{{\"name\":\"{}\",\"bone\":{},\"min\":{},\"max\":{},\"weak\":{},\"breakable\":{},\"mountable\":{}}}",
            if i == 0 { "" } else { "," },
            part.name,
            sh.bone,
            v3(sh.min),
            v3(sh.max),
            part.weak,
            sh.breakable,
            sh.mountable,
        );
    }
    s.push_str("]}\n");
    if let Some(dir) = std::path::Path::new(&out).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    std::fs::write(&out, s).unwrap_or_else(|e| panic!("could not write {out}: {e}"));
    println!("wrote {out}");
}
