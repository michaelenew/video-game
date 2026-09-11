//! Player settings, persisted between runs.
//!
//! Small on purpose. The format is `key = value`, one per line, with `#`
//! comments -- readable and editable by hand, and parsed without pulling in a
//! configuration crate for four lines of work.
//!
//! Unknown keys are preserved on save rather than dropped. A settings file that
//! silently eats anything it does not recognise is a settings file people learn
//! not to edit.

use bevy::prelude::Resource;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Radians of turn per pixel of mouse movement at `sensitivity = 1.0`.
///
/// Sensitivity is exposed as a plain multiplier rather than in radians because
/// that is the number players actually compare between games and between each
/// other. The unit lives here so there is one place to change it.
pub const RADIANS_PER_PIXEL: f32 = 0.0025;

pub const MIN_SENSITIVITY: f32 = 0.1;
pub const MAX_SENSITIVITY: f32 = 10.0;

/// One notch of adjustment, as a ratio.
///
/// Multiplicative, not additive. Sensitivity is perceived as a ratio -- going
/// from 0.5 to 0.6 is a large change and 5.0 to 5.1 is not one you can feel --
/// so a fixed step would be far too coarse at the bottom and useless at the top.
pub const STEP: f32 = 1.08;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct Settings {
    pub sensitivity: f32,
    /// Keys we did not recognise, kept so saving does not discard them.
    other: BTreeMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            sensitivity: 1.0,
            other: BTreeMap::new(),
        }
    }
}

impl Settings {
    /// Radians per pixel, ready for the mouse-look system.
    pub fn radians_per_pixel(&self) -> f32 {
        self.sensitivity * RADIANS_PER_PIXEL
    }

    /// Step sensitivity up or down by one notch, clamped.
    pub fn nudge(&mut self, up: bool) {
        let factor = if up { STEP } else { 1.0 / STEP };
        self.sensitivity = (self.sensitivity * factor).clamp(MIN_SENSITIVITY, MAX_SENSITIVITY);
    }

    pub fn parse(text: &str) -> Settings {
        let mut s = Settings::default();
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            match key {
                "sensitivity" => {
                    // A corrupt or out-of-range value falls back to the default
                    // rather than refusing to start. Nothing here is worth
                    // failing a launch over.
                    if let Ok(v) = value.parse::<f32>() {
                        if v.is_finite() {
                            s.sensitivity = v.clamp(MIN_SENSITIVITY, MAX_SENSITIVITY);
                        }
                    }
                }
                _ => {
                    s.other.insert(key.to_string(), value.to_string());
                }
            }
        }
        s
    }

    pub fn to_text(&self) -> String {
        let mut out = String::from("# Arena prototype settings.\n");
        out.push_str("# sensitivity: mouse turn rate, 1.0 is the default feel.\n");
        out.push_str(&format!("sensitivity = {:.3}\n", self.sensitivity));
        for (key, value) in &self.other {
            out.push_str(&format!("{key} = {value}\n"));
        }
        out
    }

    /// Load from disk, falling back to defaults if anything at all goes wrong.
    pub fn load() -> Settings {
        path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|t| Settings::parse(&t))
            .unwrap_or_default()
    }

    /// Write to disk. Failure is reported and otherwise ignored -- losing a
    /// sensitivity setting is not a reason to interrupt a match.
    pub fn save(&self) {
        let Some(p) = path() else { return };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(&p, self.to_text()) {
            eprintln!("could not save settings to {}: {e}", p.display());
        }
    }
}

/// Where the settings file lives.
///
/// `ARENA_SETTINGS` overrides it, which is what makes this testable and what
/// lets two people on one machine keep separate settings without a profile
/// system.
fn path() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("ARENA_SETTINGS") {
        return Some(PathBuf::from(explicit));
    }
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config/arena/settings.conf"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_file_reads_back_the_same() {
        let mut s = Settings::default();
        s.nudge(true);
        s.nudge(true);
        let round = Settings::parse(&s.to_text());
        assert!((round.sensitivity - s.sensitivity).abs() < 0.001);
    }

    #[test]
    fn unknown_keys_survive_a_save() {
        // Otherwise editing the file by hand to try something is punished by
        // having it silently deleted.
        let s = Settings::parse("sensitivity = 2.0\nsomething_new = yes\n");
        assert!(s.to_text().contains("something_new = yes"));
    }

    #[test]
    fn nonsense_falls_back_instead_of_failing() {
        for text in ["sensitivity = banana", "sensitivity =", "", "= 3", "####"] {
            let s = Settings::parse(text);
            assert_eq!(s.sensitivity, 1.0, "on input {text:?}");
        }
    }

    #[test]
    fn out_of_range_values_are_clamped_on_the_way_in() {
        assert_eq!(
            Settings::parse("sensitivity = 9999").sensitivity,
            MAX_SENSITIVITY
        );
        assert_eq!(
            Settings::parse("sensitivity = -4").sensitivity,
            MIN_SENSITIVITY
        );
        assert_eq!(Settings::parse("sensitivity = inf").sensitivity, 1.0);
    }

    #[test]
    fn nudging_is_symmetric_and_bounded() {
        let mut s = Settings::default();
        s.nudge(true);
        s.nudge(false);
        assert!((s.sensitivity - 1.0).abs() < 0.0001, "{}", s.sensitivity);

        for _ in 0..200 {
            s.nudge(false);
        }
        assert_eq!(s.sensitivity, MIN_SENSITIVITY);
        for _ in 0..400 {
            s.nudge(true);
        }
        assert_eq!(s.sensitivity, MAX_SENSITIVITY);
    }

    #[test]
    fn comments_are_ignored() {
        let s = Settings::parse("# sensitivity = 5\nsensitivity = 2.0 # trailing\n");
        assert_eq!(s.sensitivity, 2.0);
    }
}
