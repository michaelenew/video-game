//! Every sound the game makes, on one sheet, without starting the game.
//!
//! ```text
//! cargo run -p sound --example sheet          # target/sound-sheet/
//! ```
//!
//! Writes a WAV per cell and one picture: each cell a spectrogram (time
//! across, pitch up on a log scale, loudness as brightness) with the
//! patch's name under it. The rows are the instruments and the columns walk
//! one parameter each -- a blow by weight, by material, by the size of what
//! it struck; a telegraph by startup; a growl by throat; the rest by what
//! they have. **This is the loop**: listen to the WAVs, look at the sheet,
//! change a number in `patch.rs`, run it again -- the same loop the skies
//! have, for the same reason: there will not be a developer for this game
//! forever, and a sound that needs the game running to be judged is a sound
//! nobody tunes.

use look::sheet::{text, write};
use sound::patch::{
    Crackle, FIGHTER, Growl, Gust, Material, Patch, Ring, Rumble, Strike, Wet, Whoosh,
};
use sound::{spectrum, wav};

const CELL_W: usize = 180;
const CELL_H: usize = 110;
const LABEL_H: usize = 12;
const COLS: usize = 6;

fn strike(weight: f32, sharp: f32, material: Material, size: f32) -> Patch {
    Patch::Strike(Strike {
        weight,
        sharp,
        material,
        size,
    })
}

/// The sheet's rows: a title, and up to `COLS` patches.
fn rows() -> Vec<(&'static str, Vec<Patch>)> {
    vec![
        (
            "a blow on a body, by weight",
            [0.15, 0.3, 0.5, 0.7, 0.85, 1.0]
                .iter()
                .map(|w| strike(*w, 0.3, Material::Flesh, FIGHTER))
                .collect(),
        ),
        (
            "a blow on a body, by edge",
            [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
                .iter()
                .map(|s| strike(0.5, *s, Material::Flesh, FIGHTER))
                .collect(),
        ),
        (
            "what was struck",
            [
                Material::Flesh,
                Material::Hide,
                Material::Plate,
                Material::Shield,
                Material::Stone,
                Material::Wood,
            ]
            .iter()
            .map(|m| strike(0.6, 0.4, *m, FIGHTER))
            .collect(),
        ),
        (
            "hide, by the size of the animal",
            [0.6, 1.2, 1.8, 3.0, 6.0, 12.0]
                .iter()
                .map(|s| strike(0.6, 0.3, Material::Hide, *s))
                .collect(),
        ),
        (
            "a footfall, by floor",
            [
                Material::Earth,
                Material::Grass,
                Material::Rock,
                Material::Sand,
                Material::Snow,
                Material::Water,
            ]
            .iter()
            .map(|m| strike(0.25, 0.15, *m, FIGHTER))
            .collect(),
        ),
        (
            "a footfall, more floors; a landing",
            vec![
                strike(0.25, 0.15, Material::Ash, FIGHTER),
                strike(0.25, 0.15, Material::Peat, FIGHTER),
                strike(0.25, 0.15, Material::Wood, FIGHTER),
                strike(0.25, 0.15, Material::Stone, FIGHTER),
                strike(0.6, 0.1, Material::Earth, FIGHTER),
                strike(1.0, 0.1, Material::Earth, FIGHTER),
            ],
        ),
        (
            "a telegraph, by startup",
            [4u16, 9, 14, 20, 30, 45]
                .iter()
                .map(|f| {
                    Patch::Whoosh(Whoosh {
                        frames: *f,
                        size: FIGHTER,
                        rising: true,
                        weight: 0.6,
                    })
                })
                .collect(),
        ),
        (
            "a swing, by length; a dodge; a thrown shield",
            vec![
                Patch::Whoosh(Whoosh {
                    frames: 6,
                    size: FIGHTER,
                    rising: false,
                    weight: 0.3,
                }),
                Patch::Whoosh(Whoosh {
                    frames: 8,
                    size: FIGHTER,
                    rising: false,
                    weight: 0.6,
                }),
                Patch::Whoosh(Whoosh {
                    frames: 12,
                    size: FIGHTER,
                    rising: false,
                    weight: 1.0,
                }),
                Patch::Whoosh(Whoosh {
                    frames: 14,
                    size: 9.0,
                    rising: false,
                    weight: 1.0,
                }),
                Patch::Whoosh(Whoosh {
                    frames: 10,
                    size: FIGHTER,
                    rising: false,
                    weight: 0.2,
                }),
                Patch::Whoosh(Whoosh {
                    frames: 10,
                    size: 1.0,
                    rising: false,
                    weight: 0.6,
                }),
            ],
        ),
        (
            "a growl, by the animal",
            [0.6, 1.8, 3.0, 5.0, 9.0, 40.0]
                .iter()
                .map(|s| {
                    Patch::Growl(Growl {
                        frames: 30,
                        size: *s,
                    })
                })
                .collect(),
        ),
        (
            "a growl, by startup",
            [8u16, 14, 20, 30, 45, 60]
                .iter()
                .map(|f| {
                    Patch::Growl(Growl {
                        frames: *f,
                        size: 5.0,
                    })
                })
                .collect(),
        ),
        (
            "parry; stone up; stone broken; fire; blood; wind",
            vec![
                Patch::Ring(Ring { pitch: 1760.0 }),
                Patch::Rumble(Rumble {
                    frames: 12,
                    size: 2.0,
                }),
                strike(0.8, 0.3, Material::Stone, 2.0),
                Patch::Crackle(Crackle { seconds: 0.5 }),
                Patch::Wet(Wet { weight: 0.6 }),
                Patch::Gust(Gust {
                    frames: 24,
                    weight: 0.7,
                }),
            ],
        ),
    ]
}

fn main() {
    let out_dir = std::path::Path::new("target/sound-sheet");
    std::fs::create_dir_all(out_dir).expect("target is writable");
    let rows = rows();
    let w = COLS * CELL_W;
    let h = rows.len() * (CELL_H + LABEL_H);
    let mut px = vec![[12u8, 12, 14]; w * h];
    let win = 1024;
    let mut longest = 0.0f32;
    for (r, (title, patches)) in rows.iter().enumerate() {
        let y0 = r * (CELL_H + LABEL_H);
        for (c, patch) in patches.iter().enumerate().take(COLS) {
            let samples = patch.render();
            let seconds = samples.len() as f32 / sound::synth::RATE as f32;
            longest = longest.max(seconds);
            let name = patch.name();
            let file = out_dir.join(format!(
                "{:02}-{:02}-{}.wav",
                r,
                c,
                name.replace([' ', '.'], "_")
            ));
            std::fs::write(&file, wav::encode(&samples)).expect("wav written");
            // The spectrogram: one column of pixels per hop, so every cell
            // is on the same time scale -- a long sound is a wide one.
            let hop = sound::synth::RATE as usize / 400; // 2.5 ms a pixel
            let frames = spectrum::spectrogram(&samples, win, hop);
            let x0 = c * CELL_W;
            for (fx, frame) in frames.iter().enumerate().take(CELL_W) {
                for py in 0..CELL_H {
                    // Log frequency axis, 40 Hz at the bottom to 12 kHz at the top.
                    let u = 1.0 - py as f32 / CELL_H as f32;
                    let hz = 40.0 * (12_000.0f32 / 40.0).powf(u);
                    let bin = (hz * win as f32 / sound::synth::RATE as f32) as usize;
                    let m = frame.get(bin.min(win / 2 - 1)).copied().unwrap_or(0.0);
                    let db = 20.0 * (m.max(1e-6)).log10();
                    // -60 dB black to 0 dB white, through amber.
                    let v = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
                    let colour = [
                        (v.powf(0.7) * 255.0) as u8,
                        (v.powf(1.2) * 200.0) as u8,
                        (v.powf(2.5) * 120.0) as u8,
                    ];
                    px[(y0 + py) * w + x0 + fx] = colour;
                }
            }
            // The seconds, as a tick every 100 ms along the bottom edge.
            let mut t = 0.0;
            while t < seconds {
                let x = x0 + (t * 400.0) as usize;
                if x < x0 + CELL_W {
                    for py in CELL_H - 4..CELL_H {
                        px[(y0 + py) * w + x] = [120, 120, 130];
                    }
                }
                t += 0.1;
            }
            text(&mut px, w, x0 + 2, y0 + CELL_H + 2, &name);
        }
        text(&mut px, w, 2, y0 + 2, title);
    }
    let picture = out_dir.join("sheet.ppm");
    write(picture.to_str().unwrap(), w, h, &px);
    println!(
        "{} rows, {} sounds, the longest {:.2} s. {}",
        rows.len(),
        rows.iter().map(|(_, p)| p.len().min(COLS)).sum::<usize>(),
        longest,
        picture.display()
    );
}
