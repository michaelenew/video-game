use art::stone::library;
fn main() {
    println!(
        "{:>10}  {:>14}  {:>10}",
        "rock", "linear albedo", "max chroma"
    );
    for (name, s) in library() {
        let mut total = 0.0;
        let mut n = 0;
        for i in 0..24 {
            for j in 0..24 {
                let p = [i as f32 * 0.05, j as f32 * 0.05, 0.13];
                let c = s.at_scale(p, 0.05).colour;
                total += (c[0] + c[1] + c[2]) / 3.0;
                n += 1;
            }
        }
        println!(
            "{name:>10}  {:>14.3}  {:>10.3}",
            total / n as f32,
            s.max_chroma()
        );
    }
    println!("\nreference: asphalt 0.04-0.12, granite 0.15-0.25, concrete 0.20-0.30");
    println!("the arena's old placeholder stone averaged 0.054");
}
