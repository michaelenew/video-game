//! Where the desktop and the browser differ.
//!
//! Five things, and only five. Three of them are here: **how a run is
//! configured**, **where its settings are kept**, and **what a crash looks
//! like**. [`crate::online`] owns the fourth, whether there is a peer. The
//! fifth — whether there is a checkout to commit a tuning session to — belongs
//! to the two files that want one, [`crate::bake`] and [`crate::hub`], which
//! carry their own browser half and are the two exemptions in
//! `crates/game/tests/one_platform.rs`.
//!
//! That test is the rule: nothing outside those four files may reach the host,
//! so everything else in the crate is written once and compiled twice.
//!
//! **The query string is the command line.** A browser has no argv, and a URL
//! is the only thing you can hand somebody, so `index.html?p1=champion&dev`
//! starts the run that `game --p1 champion --dev` starts. Both go through
//! [`Options`], and `the_same_run_spelled_two_ways` below is the test that
//! keeps the two spellings equal.
//!
//! Names are matched loosely — case is ignored, and `-` and `_` are the same
//! letter — so `--shot-frame`, `SHOT_FRAME=` and `?shot_frame=` are one name
//! rather than three. Not tidiness: the flags are lower case and the
//! environment variables upper case for no reason anyone remembers, and
//! somebody editing a URL should not have to know which of the two a knob
//! happened to be.

use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// The parser both spellings go through
// ---------------------------------------------------------------------------

/// The flags and values a run was started with.
///
/// A `Vec` and a linear scan: this is built once at startup and read a dozen
/// times, so the frame budget has no opinion about it and neither should we.
#[derive(Debug, Default, PartialEq)]
pub struct Options {
    pairs: Vec<(String, Option<String>)>,
}

// Four of the pieces below live on one platform and are compiled into both.
// `from_args` and `is_name` are the desktop's; `from_query` and `decode` are
// the browser's. Each is built everywhere because the tests at the bottom of
// this file run on the host and never on wasm, and a `cfg` would compile the
// subject of the test out from under it. Hence `allow(dead_code)`: not code
// nobody calls, code the other platform calls.

/// A word that names a flag rather than giving a value for one.
///
/// Dashes and then a letter. The letter is what keeps `--shot-pitch -0.3` from
/// reading its own value as a second flag, which is the only way a single-dash
/// name can go wrong here.
#[allow(dead_code)]
fn is_name(word: &str) -> bool {
    let rest = word.trim_start_matches('-');
    rest.len() < word.len() && rest.starts_with(|c: char| c.is_ascii_alphabetic())
}

/// Two names are the same name if they differ only in case, in which of the
/// two word separators they use, or in whether they are written with the
/// leading dashes of a flag.
///
/// The dashes matter for one reason: it lets a call site ask for `"--p1"`,
/// spelled exactly the way the manual says to type it, which is what keeps
/// `crates/manual/tests/complete.rs` able to find every flag the game reads.
fn same(a: &str, b: &str) -> bool {
    let key = |s: &str| {
        s.trim_start_matches('-')
            .chars()
            .map(|c| {
                if c == '-' {
                    '_'
                } else {
                    c.to_ascii_lowercase()
                }
            })
            .collect::<String>()
    };
    key(a) == key(b)
}

impl Options {
    /// From a command line: `--name`, or `--name value` when the next word is
    /// not itself a flag. `-h` counts as a name too.
    ///
    /// Pass the arguments without the program name.
    #[allow(dead_code)]
    pub fn from_args<I: IntoIterator<Item = String>>(args: I) -> Options {
        let args: Vec<String> = args.into_iter().collect();
        let mut pairs = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if !is_name(arg) {
                continue;
            }
            let value = args.get(i + 1).filter(|next| !is_name(next)).cloned();
            pairs.push((arg.trim_start_matches('-').to_string(), value));
        }
        Options { pairs }
    }

    /// From a URL query string: `p1=champion&dev&oven_search=jump%20height`,
    /// with or without the leading `?`.
    #[allow(dead_code)]
    pub fn from_query(query: &str) -> Options {
        let pairs = query
            .trim_start_matches('?')
            .split('&')
            .filter(|part| !part.is_empty())
            .map(|part| match part.split_once('=') {
                Some((name, value)) => (decode(name), Some(decode(value))),
                None => (decode(part), None),
            })
            .collect();
        Options { pairs }
    }

    /// The value given for a name, if it was given one.
    pub fn value(&self, name: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(key, _)| same(key, name))
            .and_then(|(_, value)| value.as_deref())
    }

    /// Is this switch on?
    ///
    /// Present and not switched off. `--dev`, `?dev` and `?dev=1` are on;
    /// `?dev=0` is off. The off spellings exist because a URL gets edited by
    /// hand, often by somebody who did not write it, and changing a `1` to a
    /// `0` is easier to do correctly than deleting a parameter out of the
    /// middle of a query string.
    pub fn flag(&self, name: &str) -> bool {
        match self.pairs.iter().find(|(key, _)| same(key, name)) {
            None => false,
            Some((_, None)) => true,
            Some((_, Some(value))) => !matches!(
                value.to_ascii_lowercase().as_str(),
                "0" | "false" | "no" | "off"
            ),
        }
    }
}

/// Percent-decoding, plus `+` for a space.
///
/// Fifteen lines rather than a dependency, for the same reason `settings.rs`
/// parses its own four keys. A malformed escape is left as written: a search
/// box that came back with a literal `%zz` in it is a great deal easier to
/// understand than one that silently came back empty.
#[allow(dead_code)]
fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

// ---------------------------------------------------------------------------
// What the rest of the crate calls
// ---------------------------------------------------------------------------

/// How this run was configured: argv on the desktop, the query string in the
/// browser.
pub fn options() -> &'static Options {
    static OPTIONS: OnceLock<Options> = OnceLock::new();
    OPTIONS.get_or_init(read_options)
}

/// Is this switch on for this run?
pub fn flag(name: &str) -> bool {
    options().flag(name)
}

/// The value this run gave for a name, if any.
pub fn value(name: &str) -> Option<&'static str> {
    options().value(name)
}

/// A knob that is spelled as an environment variable on the desktop.
///
/// The process environment wins where there is one, and the command line or
/// the query string answers otherwise — so `SHOT_FRAME=200 game`,
/// `game --shot-frame 200` and `?shot_frame=200` all mean the same thing.
pub fn env(key: &str) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(from_environment) = std::env::var(key) {
        return Some(from_environment);
    }
    value(key).map(str::to_string)
}

/// `env`, parsed. `None` if it is absent or is not a number.
pub fn env_parsed<T: std::str::FromStr>(key: &str) -> Option<T> {
    env(key)?.parse().ok()
}

// ---------------------------------------------------------------------------
// The desktop
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
mod host {
    use std::path::PathBuf;

    pub fn read_options() -> super::Options {
        super::Options::from_args(std::env::args().skip(1))
    }

    /// Where the settings file lives.
    ///
    /// `ARENA_SETTINGS` overrides it, which is what makes this testable and
    /// what lets two people on one machine keep separate settings without a
    /// profile system.
    fn path() -> Option<PathBuf> {
        if let Some(explicit) = super::env("ARENA_SETTINGS") {
            return Some(PathBuf::from(explicit));
        }
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".config/arena/settings.conf"))
    }

    pub fn load_settings() -> Option<String> {
        std::fs::read_to_string(path()?).ok()
    }

    pub fn save_settings(text: &str) {
        let Some(p) = path() else { return };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(&p, text) {
            eprintln!("could not save settings to {}: {e}", p.display());
        }
    }

    /// Nothing to install: a panic already prints to the terminal the game was
    /// started from.
    pub fn report_panics() {}
}

// ---------------------------------------------------------------------------
// The browser
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod host {
    /// One key, so a second prototype on the same origin does not overwrite
    /// this one's settings.
    const STORAGE_KEY: &str = "arena.settings";

    pub fn read_options() -> super::Options {
        let query = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .unwrap_or_default();
        super::Options::from_query(&query)
    }

    /// `localStorage`, holding exactly the text the desktop writes to a file.
    ///
    /// Same format on purpose: a settings file and a browser's local storage
    /// are two places to keep four lines, not two ideas about what a setting
    /// is. It is also per-origin and per-browser, so a shared link cannot
    /// carry one player's sensitivity to another.
    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    pub fn load_settings() -> Option<String> {
        storage()?.get_item(STORAGE_KEY).ok()?
    }

    pub fn save_settings(text: &str) {
        if let Some(store) = storage() {
            // Storage can be full or switched off entirely. Losing a
            // sensitivity setting is not a reason to interrupt a match.
            let _ = store.set_item(STORAGE_KEY, text);
        }
    }

    /// Put the panic message where the player can read it.
    ///
    /// Without this a panic in the browser is a blank canvas and
    /// `RuntimeError: unreachable` in a console nobody opened. Somebody who was
    /// sent a link and hit a driver bug should be able to say *what* broke
    /// without being talked through the developer tools.
    pub fn report_panics() {
        std::panic::set_hook(Box::new(|info| {
            let message = format!("{info}");
            web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&message));
            let element = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("fatal"));
            if let Some(element) = element {
                element.set_text_content(Some(&message));
                let _ = element.remove_attribute("hidden");
            }
        }));
    }
}

use host::read_options;
pub use host::{load_settings, report_panics, save_settings};

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_run_spelled_two_ways() {
        // The whole point of the file. If these ever disagree, a link somebody
        // was sent starts a different game from the one that was tested.
        let command_line = Options::from_args(
            ["--p1", "champion", "--p2", "reaver", "--dev", "--hunt"].map(String::from),
        );
        let url = Options::from_query("?p1=champion&p2=reaver&dev&hunt");

        for name in ["--p1", "--p2", "p1", "p2"] {
            assert_eq!(command_line.value(name), url.value(name), "value {name}");
        }
        for name in ["--dev", "--hunt", "dev", "hunt", "oven", "nothing_at_all"] {
            assert_eq!(command_line.flag(name), url.flag(name), "flag {name}");
        }
    }

    #[test]
    fn a_flag_with_a_value_after_it_still_reads_as_on() {
        // `--dev --p1 champion` must not swallow `--p1` as dev's value, and
        // `?dev=1` must not read as off.
        let args = Options::from_args(["--dev", "--p1", "champion"].map(String::from));
        assert!(args.flag("dev"));
        assert_eq!(args.value("dev"), None);
        assert_eq!(args.value("p1"), Some("champion"));
        assert!(Options::from_query("dev=1&p1=champion").flag("dev"));
    }

    #[test]
    fn a_switch_can_be_switched_off_in_a_url() {
        for off in ["dev=0", "dev=false", "dev=no", "dev=OFF"] {
            assert!(!Options::from_query(off).flag("dev"), "{off}");
        }
        for on in ["dev", "dev=1", "dev=yes", "dev=please"] {
            assert!(Options::from_query(on).flag("dev"), "{on}");
        }
    }

    #[test]
    fn case_and_the_two_word_separators_are_the_same_name() {
        let url = Options::from_query("shot_frame=200");
        assert_eq!(url.value("SHOT_FRAME"), Some("200"));
        assert_eq!(url.value("shot-frame"), Some("200"));
        let args = Options::from_args(["--shot-frame", "200"].map(String::from));
        assert_eq!(args.value("SHOT_FRAME"), Some("200"));
    }

    #[test]
    fn a_value_survives_being_url_encoded() {
        let url = Options::from_query("?oven_search=jump%20height&note=a%2Bb+c");
        assert_eq!(url.value("oven_search"), Some("jump height"));
        assert_eq!(url.value("note"), Some("a+b c"));
    }

    #[test]
    fn nonsense_in_a_url_does_not_lose_the_rest_of_it() {
        // A URL arrives however somebody retyped it. Whatever is legible
        // should still be read.
        let url = Options::from_query("?&&p1=champion&%zz=1&trailing%");
        assert_eq!(url.value("p1"), Some("champion"));
        assert!(url.flag("trailing%"));
    }

    #[test]
    fn a_negative_number_is_a_value_and_not_a_flag() {
        // `SHOT_PITCH` is an angle and angles go below the horizon.
        let args = Options::from_args(["--shot-pitch", "-0.3", "-h"].map(String::from));
        assert_eq!(args.value("SHOT_PITCH"), Some("-0.3"));
        assert!(args.flag("-h"));
    }

    #[test]
    fn an_absent_name_is_absent_in_both() {
        assert_eq!(Options::default().value("p1"), None);
        assert!(!Options::default().flag("dev"));
        assert_eq!(Options::from_query("").value("p1"), None);
        assert_eq!(Options::from_args(Vec::<String>::new()).value("p1"), None);
    }
}
