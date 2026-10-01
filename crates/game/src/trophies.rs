//! Trophies: which creatures this player has beaten, and at which temper
//! (`docs/design/world.md` §4 and §6 W1).
//!
//! **The player's, not the fight's.** Nothing in a hunt depends on it, so it is
//! not in the snapshot; it is kept the way a player's settings are -- a file on
//! the desktop, local storage in the browser -- and *where* is
//! [`crate::platform`]'s question, the same as for [`crate::settings`]. In a
//! hunt with somebody else each machine writes its own: a fight counts for
//! both (world.md §5), and a player may go along to a temper they have not
//! earned.
//!
//! **Keyed by species id**, the one byte the simulation knows a creature by,
//! so a record written by a build with a creature this one lacks keeps that
//! creature's line rather than dropping it. The format is a line per creature:
//!
//! ```text
//! # species id = tempers beaten    (name, for a person reading it)
//! 0 = 0 1     # Ridgeback
//! ```
//!
//! **What it offers.** Temper 0 always; temper N once temper N-1 of the same
//! creature is beaten. `--temper <n>` (or `?temper=`) is the dev's way past
//! that, regardless of progress.

use crate::platform;
use bevy::prelude::Resource;
use sim::World;
use sim::species::{self, SpeciesId};
use sim::temper::TEMPERS;
use std::collections::BTreeMap;

/// How many frames into the round-over pause a won hunt is recorded.
///
/// Not on the frame it is won, because that frame may be a prediction: online,
/// a rollback can take back a killing blow up to `net::MAX_ROLLBACK_FRAMES`
/// (eight) frames later, and a trophy for a kill that never happened is a lie
/// written to disk. Twice that is past any rollback, and far inside the
/// pause, which is two and a half seconds. A number here rather than the net
/// crate's because the browser build does not link `net`.
const SETTLED: u16 = 16;

#[derive(Clone, Debug, Default, PartialEq, Eq, Resource)]
pub struct Trophies {
    /// Species id to a bitmask of tempers beaten.
    beaten: BTreeMap<u8, u8>,
    /// The won hunt already recorded, so a pause is written once.
    recorded: bool,
}

impl Trophies {
    /// The player's record, from wherever this platform keeps it. Empty if
    /// there is none yet.
    pub fn load() -> Trophies {
        platform::load_trophies()
            .map(|text| Trophies::from_text(&text))
            .unwrap_or_default()
    }

    pub fn save(&self) {
        platform::save_trophies(&self.to_text());
    }

    pub fn from_text(text: &str) -> Trophies {
        let mut beaten = BTreeMap::new();
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            let Some((id, tempers)) = line.split_once('=') else {
                continue;
            };
            let Ok(id) = id.trim().parse::<u8>() else {
                continue;
            };
            let mask = tempers
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter_map(|t| t.parse::<u8>().ok())
                .filter(|t| *t < 8)
                .fold(0u8, |m, t| m | 1 << t);
            if mask != 0 {
                *beaten.entry(id).or_insert(0) |= mask;
            }
        }
        Trophies {
            beaten,
            recorded: false,
        }
    }

    pub fn to_text(&self) -> String {
        let mut out =
            String::from("# Trophies: species id = tempers beaten. Written by the game.\n");
        for (id, mask) in &self.beaten {
            let tempers: Vec<String> = (0..8)
                .filter(|t| mask & 1 << t != 0)
                .map(|t: u8| t.to_string())
                .collect();
            let name = species::lookup(SpeciesId(*id)).map_or("not in this build", |s| s.name);
            out.push_str(&format!("{id} = {}    # {name}\n", tempers.join(" ")));
        }
        out
    }

    /// How many trophies there are, every creature and temper counted. A
    /// record only grows, so this changes exactly when it does -- which is
    /// what the HUD asks, rather than cloning the record every frame.
    pub fn count(&self) -> u32 {
        self.beaten.values().map(|m| m.count_ones()).sum()
    }

    /// Has this creature been beaten at this temper?
    pub fn beaten(&self, id: SpeciesId, temper: u8) -> bool {
        temper < 8 && self.beaten.get(&id.0).is_some_and(|m| m & 1 << temper != 0)
    }

    /// Record a win. True if it is new.
    pub fn record(&mut self, id: SpeciesId, temper: u8) -> bool {
        if temper >= 8 || self.beaten(id, temper) {
            return false;
        }
        *self.beaten.entry(id.0).or_insert(0) |= 1 << temper;
        true
    }

    /// Is this temper on offer for this creature? Temper 0 always; above it,
    /// once the one below is beaten.
    pub fn offered(&self, id: SpeciesId, temper: u8) -> bool {
        temper < TEMPERS && (temper == 0 || self.beaten(id, temper - 1))
    }

    /// The temper after `now` that is on offer for this creature, wrapping to
    /// 0 -- what `T` steps to. `any` offers every temper: `--temper`.
    pub fn next_temper(&self, id: SpeciesId, now: u8, any: bool) -> u8 {
        (1..=TEMPERS)
            .map(|step| (now + step) % TEMPERS)
            .find(|t| any || self.offered(id, *t))
            .unwrap_or(0)
    }

    /// **Write the trophy for a won hunt**, once per win, once it is past any
    /// rollback: every creature in the fight, at the fight's temper. Returns
    /// whether anything new was written (and saved).
    pub fn notice(&mut self, w: &World) -> bool {
        let Some((beaten, temper)) = w.hunt_won() else {
            self.recorded = false;
            return false;
        };
        let settled = match w.phase {
            sim::state::Phase::RoundOver { left, .. } => {
                left.saturating_add(SETTLED) < sim::tuning::round_over_frames()
            }
            sim::state::Phase::Fighting => false,
        };
        if self.recorded || !settled {
            return false;
        }
        self.recorded = true;
        let mut new = false;
        for id in beaten.into_iter().flatten() {
            new |= self.record(id, temper);
        }
        if new {
            self.save();
        }
        new
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::Class;
    use sim::state::MAX_PLAYERS;

    #[test]
    fn the_record_round_trips_through_its_text() {
        let mut t = Trophies::default();
        assert!(t.record(SpeciesId::RIDGEBACK, 0));
        assert!(t.record(SpeciesId::RIDGEBACK, 2));
        assert!(t.record(SpeciesId::GNATS, 0));
        assert!(!t.record(SpeciesId::GNATS, 0), "recorded twice");
        let text = t.to_text();
        assert!(text.contains("Ridgeback"), "{text}");
        assert_eq!(Trophies::from_text(&text), t);
    }

    #[test]
    fn a_creature_this_build_lacks_keeps_its_line() {
        // Written by a build with the Siegeshell; read by one without it.
        let text = "10 = 0 1    # Siegeshell\n0 = 0\n";
        let t = Trophies::from_text(text);
        assert!(t.beaten(SpeciesId::SIEGESHELL, 1));
        let again = Trophies::from_text(&t.to_text());
        assert_eq!(again, t);
        assert!(again.to_text().contains("10 = 0 1"));
    }

    #[test]
    fn nonsense_in_the_file_loses_only_itself() {
        let t = Trophies::from_text("garbage\n0 = 0 x 1\n= 3\n300 = 1\n\n# 4 = 0\n");
        assert!(t.beaten(SpeciesId::RIDGEBACK, 0));
        assert!(t.beaten(SpeciesId::RIDGEBACK, 1));
        assert!(!t.beaten(SpeciesId(4), 0));
    }

    #[test]
    fn a_temper_is_offered_once_the_one_below_it_is_beaten() {
        let mut t = Trophies::default();
        let r = SpeciesId::RIDGEBACK;
        assert!(t.offered(r, 0));
        assert!(!t.offered(r, 1));
        assert_eq!(t.next_temper(r, 0, false), 0, "nothing to step to yet");
        assert_eq!(t.next_temper(r, 0, true), 1, "--temper offers them all");
        t.record(r, 0);
        assert!(t.offered(r, 1));
        assert!(!t.offered(r, 2));
        assert_eq!(t.next_temper(r, 0, false), 1);
        assert_eq!(t.next_temper(r, 1, false), 0, "wraps back to as tuned");
        assert!(!t.offered(SpeciesId::GNATS, 1), "trophies are per creature");
        t.record(r, 3);
        assert!(!t.offered(r, 4), "there is no fifth temper");
    }

    #[test]
    fn a_won_hunt_records_a_trophy_once_it_is_past_any_rollback() {
        let mut w = World::hunt([Class::Bulwark; MAX_PLAYERS]).tempered(1);
        let mut t = Trophies::default();
        let idle = [sim::Input::default(); MAX_PLAYERS];
        assert!(!t.notice(&w));
        w.monster_mut().unwrap().health = 0;
        w.advance(idle);
        assert!(w.hunt_won().is_some());
        assert!(
            !t.notice(&w),
            "recorded on a frame a rollback could take back"
        );
        let mut wrote = false;
        for _ in 0..SETTLED + 2 {
            w.advance(idle);
            wrote |= t.notice(&w);
        }
        assert!(wrote, "the won hunt was never recorded");
        assert!(t.beaten(SpeciesId::RIDGEBACK, 1));
        assert!(
            !t.beaten(SpeciesId::RIDGEBACK, 0),
            "recorded at the wrong temper"
        );
        assert!(t.offered(SpeciesId::RIDGEBACK, 2));
        // And once only, however long the pause goes on.
        for _ in 0..30 {
            w.advance(idle);
            assert!(!t.notice(&w));
        }
    }

    #[test]
    fn a_lost_hunt_and_a_versus_round_record_nothing() {
        let idle = [sim::Input::default(); MAX_PLAYERS];
        let mut lost = World::hunt([Class::Bulwark; MAX_PLAYERS]);
        for p in lost.players.iter_mut() {
            p.health = 0;
        }
        let mut versus = World::with_classes([Class::Bulwark; MAX_PLAYERS]);
        versus.players[1].health = 0;
        let mut t = Trophies::default();
        for _ in 0..60 {
            lost.advance(idle);
            versus.advance(idle);
            t.notice(&lost);
            t.notice(&versus);
        }
        assert_eq!(t, Trophies::default());
    }
}
