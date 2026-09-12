//! `cargo run -p art --example daylight` -- the lighting rig across a day.
//!
//! One column per thing the renderer gets told, one row per sun elevation.
//! The point is that nothing here was chosen: the reddening, the fall in
//! intensity and the sky turning warm at dusk are all consequences of how much
//! air the light crossed.

use art::sky::Sky;

fn main() {
    println!(
        "{:>5}  {:>4}  {:>17}  {:>9}  {:>17}  {:>8}",
        "elev", "air", "sun colour", "sun lux", "sky colour", "sky lux"
    );
    for e in [90.0f32, 60.0, 30.0, 15.0, 8.0, 4.0, 2.0, 0.0, -1.5] {
        let s = Sky::at(e, 40.0);
        println!(
            "{e:>5.1}  {:>4.1}  {:>5.2} {:>5.2} {:>5.2}  {:>9.0}  {:>5.2} {:>5.2} {:>5.2}  {:>8.0}",
            art::sky::air_mass(e),
            s.sun_color[0],
            s.sun_color[1],
            s.sun_color[2],
            s.sun_illuminance,
            s.sky_color[0],
            s.sky_color[1],
            s.sky_color[2],
            s.sky_illuminance,
        );
    }
}
