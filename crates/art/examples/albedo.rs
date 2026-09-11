//! `cargo run -p art --example albedo` -- what each material reflects, against
//! what real surfaces reflect.
//!
//! Worth having because it caught a bug that looked like the opposite of what
//! it was. The first capture with generated materials came out near-white and
//! the obvious reading was "the materials are too bright". They were not: the
//! floor was at 0.06 reflectance, which is asphalt. The *lighting* had been
//! raised, long before, to make placeholder colours at 0.02 reflectance read
//! -- darker than anything real -- and that debt came due the moment the
//! albedo became physical.
use art::materials;
fn main() {
    println!("material     linear albedo (mean of R,G,B) across the ramp");
    for m in materials::FIXED {
        let mut lo = 1.0f32;
        let mut hi = 0.0f32;
        let mut sum = 0.0f32;
        for k in 0..=16 {
            let c = m.surface.ramp.at(k as f32 / 16.0);
            let mean = (c[0] + c[1] + c[2]) / 3.0;
            lo = lo.min(mean);
            hi = hi.max(mean);
            sum += mean;
        }
        println!(
            "{:>9}   {lo:.3} .. {hi:.3}   mean {:.3}",
            m.name,
            sum / 17.0
        );
    }
    println!("\nfor reference: asphalt 0.04-0.12, granite 0.15-0.25, concrete 0.20-0.30,");
    println!("skin 0.20-0.45, fresh snow 0.80");
}
