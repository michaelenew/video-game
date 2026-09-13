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

/// One notch of sensitivity, as a ratio.
///
/// Multiplicative, not additive. Sensitivity is perceived as a ratio -- going
/// from 0.5 to 0.6 is a large change and 5.0 to 5.1 is not one you can feel --
/// so a fixed step would be far too coarse at the bottom and useless at the top.
pub const STEP: f32 = 1.08;

/// Vertical field of view, in degrees.
///
/// Degrees because that is the unit every other game's settings screen uses, so
/// it is the number a player can carry between them.
pub const MIN_FOV: f32 = 40.0;
pub const MAX_FOV: f32 = 100.0;
const FOV_STEP: f32 = 2.0;

/// Which number a key press is reaching for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Knob {
    Sensitivity,
    Fov,
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct Settings {
    pub sensitivity: f32,
    /// Vertical field of view, in degrees.
    pub fov: f32,
    /// Keys we did not recognise, kept so saving does not discard them.
    other: BTreeMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            sensitivity: 1.0,
            // Wider and further back than a default perspective camera. A
            // 45-degree view from six metres reads as cramped in an arena you
            // are meant to be moving around inside.
            fov: 58.0,
            other: BTreeMap::new(),
        }
    }
}

impl Settings {
    /// Radians per pixel, ready for the mouse-look system.
    pub fn radians_per_pixel(&self) -> f32 {
        self.sensitivity * RADIANS_PER_PIXEL
    }

    /// Field of view in radians, which is what the renderer wants.
    pub fn fov_radians(&self) -> f32 {
        self.fov.to_radians()
    }

    /// Step one setting up or down by a notch, clamped.
    ///
    /// Sensitivity moves by a ratio and the other two by a fixed amount,
    /// because that is how each is perceived: doubling a sensitivity feels like
    /// a consistent change at any value, whereas two degrees of view is two
    /// degrees of view whether you are at 45 or at 90.
    pub fn nudge(&mut self, knob: Knob, up: bool) {
        let sign = if up { 1.0 } else { -1.0 };
        match knob {
            Knob::Sensitivity => {
                let factor = if up { STEP } else { 1.0 / STEP };
                self.sensitivity =
                    (self.sensitivity * factor).clamp(MIN_SENSITIVITY, MAX_SENSITIVITY);
            }
            Knob::Fov => {
                self.fov = (self.fov + sign * FOV_STEP).clamp(MIN_FOV, MAX_FOV);
            }
        }
    }

    /// One line for the heads-up display.
    pub fn label(&self) -> String {
        format!("mouse {:.2}   fov {:.0}", self.sensitivity, self.fov)
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
            // A corrupt or out-of-range value falls back to the default rather
            // than refusing to start. Nothing here is worth failing a launch
            // over.
            let number = value.parse::<f32>().ok().filter(|v| v.is_finite());
            match (key, number) {
                ("sensitivity", Some(v)) => {
                    s.sensitivity = v.clamp(MIN_SENSITIVITY, MAX_SENSITIVITY)
                }
                ("fov", Some(v)) => s.fov = v.clamp(MIN_FOV, MAX_FOV),
                ("sensitivity" | "fov", None) => {}
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
        out.push_str("# fov: vertical field of view, degrees.\n");
        out.push_str(&format!("sensitivity = {:.3}\n", self.sensitivity));
        out.push_str(&format!("fov = {:.1}\n", self.fov));
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
        s.nudge(Knob::Sensitivity, true);
        s.nudge(Knob::Sensitivity, true);
        s.nudge(Knob::Fov, true);
        let round = Settings::parse(&s.to_text());
        assert!((round.sensitivity - s.sensitivity).abs() < 0.001);
        assert!((round.fov - s.fov).abs() < 0.05);
    }

    #[test]
    fn unknown_keys_survive_a_save() {
        // Otherwise editing the file by hand to try something is punished by
        // having it silently deleted.
        let s = Settings::parse("sensitivity = 2.0\nsomething_new = yes\n");
        assert!(s.to_text().contains("something_new = yes"));
    }

    #[test]
    fn a_known_key_with_a_bad_value_is_not_mistaken_for_an_unknown_one() {
        // It must fall back to the default, not get copied through to the
        // output as an opaque leftover -- which would write the file twice.
        let s = Settings::parse("fov = banana\n");
        assert_eq!(s.fov, Settings::default().fov);
        assert_eq!(s.to_text().matches("fov =").count(), 1);
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
        assert_eq!(Settings::parse("fov = 300").fov, MAX_FOV);
        // `camera_distance` is gone -- it moved the eye, and the eye is where
        // the aiming ray starts -- but an old config file still has the line in
        // it and must not be rejected for that.
        assert_eq!(
            Settings::parse("camera_distance = 3").fov,
            MAX_FOV.min(58.0)
        );
    }

    #[test]
    fn nudging_is_symmetric_and_bounded() {
        let mut s = Settings::default();
        for knob in [Knob::Sensitivity, Knob::Fov] {
            let before = Settings::default();
            s.nudge(knob, true);
            s.nudge(knob, false);
            assert!(
                (s.sensitivity - before.sensitivity).abs() < 0.0001
                    && (s.fov - before.fov).abs() < 0.0001,
                "{knob:?} did not come back"
            );
        }

        for _ in 0..400 {
            s.nudge(Knob::Sensitivity, false);
            s.nudge(Knob::Fov, false);
        }
        assert_eq!(s.sensitivity, MIN_SENSITIVITY);
        assert_eq!(s.fov, MIN_FOV);

        for _ in 0..400 {
            s.nudge(Knob::Sensitivity, true);
            s.nudge(Knob::Fov, true);
        }
        assert_eq!(s.sensitivity, MAX_SENSITIVITY);
        assert_eq!(s.fov, MAX_FOV);
    }

    #[test]
    fn comments_are_ignored() {
        let s = Settings::parse("# sensitivity = 5\nsensitivity = 2.0 # trailing\n");
        assert_eq!(s.sensitivity, 2.0);
    }
}
