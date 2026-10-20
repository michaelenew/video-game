//! **The cairns this player has touched**, for fast travel (2026-10-20).
//!
//! The player's, not the fight's, like their trophies ([`crate::trophies`]):
//! nothing in the world depends on which cairns somebody has seen, so it is
//! not in the snapshot, and it is kept where a player's record is kept --
//! a file beside the trophies, local storage in a browser
//! ([`crate::platform`]). The world only hears which cairn was picked, as a
//! trip on the wire (`sim::input::Destination::Cairn`), and checks it is one.
//!
//! **Kept by where a cairn is, not by its number.** A cairn's number is its
//! place in `sim::valley::cairns`, which moves whenever the map gains or loses
//! one; its place on the map does not. A line per cairn, the middle of its top
//! in centimetres of the map:
//!
//! ```text
//! # x y z    (place, for a person reading it)
//! 1000 24042 -6597    # Needle
//! ```
//!
//! In `--dev` every cairn is offered, touched or not.

use crate::platform;
use bevy::prelude::Resource;
use sim::World;
use std::collections::BTreeSet;

/// How near a remembered point has to be to a cairn to be that cairn, in
/// centimetres: a map that moved a cairn a little still knows it.
const SAME: i32 = 150;

#[derive(Clone, Debug, Default, PartialEq, Eq, Resource)]
pub struct Cairns {
    /// The middle of each touched cairn's top, map centimetres.
    seen: BTreeSet<(i32, i32, i32)>,
}

/// One cairn fast travel offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offered {
    /// Its number on the wire: its place in `sim::valley::cairns`.
    pub k: u8,
    /// The place it stands in.
    pub place: &'static str,
    /// How far up it is, over the place's own floor, in metres.
    pub up: i32,
    /// What its button says: the height, and where another in the same
    /// place is as high, which side of the place it is on (and a count,
    /// where that is not enough).
    pub label: String,
}

/// The middle of a cairn's top on the map, in centimetres.
fn key(k: usize) -> Option<(i32, i32, i32)> {
    let atlas = sim::atlas::valley();
    let b = *sim::valley::cairns().get(k)? as usize;
    let top = sim::valley::top_of(atlas.solids.get(b)?);
    let cm = |v: sim::Fx| ((v.raw() as i64 * 100) >> 16) as i32;
    Some((cm(top.x), cm(top.y), cm(top.z)))
}

impl Cairns {
    pub fn load() -> Cairns {
        platform::load_cairns()
            .map(|text| Cairns::from_text(&text))
            .unwrap_or_default()
    }

    pub fn save(&self) {
        platform::save_cairns(&self.to_text());
    }

    pub fn from_text(text: &str) -> Cairns {
        let seen = text
            .lines()
            .filter_map(|line| {
                let mut n = line
                    .split('#')
                    .next()?
                    .split_whitespace()
                    .map(|w| w.parse::<i32>().ok());
                Some((n.next()??, n.next()??, n.next()??))
            })
            .collect();
        Cairns { seen }
    }

    pub fn to_text(&self) -> String {
        let atlas = sim::atlas::valley();
        let mut text = String::from("# x y z, map centimetres: the cairns touched (fast travel)\n");
        for &(x, y, z) in &self.seen {
            let place = atlas
                .place_at(sim::arena::cm(x), sim::arena::cm(z))
                .map_or("?", |p| p.arena.get().name);
            text.push_str(&format!("{x} {y} {z}    # {place}\n"));
        }
        text
    }

    /// Has this player touched the `k`th cairn?
    pub fn has(&self, k: usize) -> bool {
        key(k).is_some_and(|(x, y, z)| {
            self.seen.iter().any(|&(a, b, c)| {
                (a - x).abs() <= SAME && (b - y).abs() <= SAME && (c - z).abs() <= SAME
            })
        })
    }

    /// The world as it stands: the cairn seat `me` last touched is one
    /// they have touched. Saved when it is new; true then.
    pub fn notice(&mut self, w: &World, me: usize) -> bool {
        if !w.valley.on {
            return false;
        }
        let Some(b) = w.valley.cairn.get(me).and_then(|c| c.checked_sub(1)) else {
            return false;
        };
        let Ok(k) = sim::valley::cairns().binary_search(&(b as u32)) else {
            return false;
        };
        if self.has(k) {
            return false;
        }
        let Some(at) = key(k) else {
            return false;
        };
        self.seen.insert(at);
        self.save();
        true
    }

    /// **What fast travel offers**: the cairns touched -- every one, with
    /// `all` -- by place in the map's order, then from the bottom up.
    pub fn offered(&self, all: bool) -> Vec<Offered> {
        let atlas = sim::atlas::valley();
        let mut out: Vec<(usize, Offered, (i64, i64))> = sim::valley::cairns()
            .iter()
            .enumerate()
            .filter(|(k, _)| *k <= u8::MAX as usize && (all || self.has(*k)))
            .filter_map(|(k, &b)| {
                let top = sim::valley::top_of(&atlas.solids[b as usize]);
                // The place it is one of, as `valley::cairn_at` reads it: the
                // place whose box it is, or for one the layout made, the place
                // it stands in. Not simply the ground under it: the Needle's
                // top is over the town.
                let extra = atlas.sources[b as usize].place == sim::atlas::EXTRA;
                let owner = if extra {
                    atlas.place_at(top.x, top.z)
                } else {
                    atlas.places.get(atlas.sources[b as usize].place as usize)
                }?;
                let order = atlas.places.iter().position(|p| p.arena == owner.arena)?;
                // A route's island hanging over a place is the route's: the
                // Sky Road over the town is not a cairn of the town's to a
                // person reading the list.
                let route = if extra { route_of(top) } else { None };
                let mid = (
                    (owner.lo.0.raw() as i64 + owner.hi.0.raw() as i64) / 2,
                    (owner.lo.1.raw() as i64 + owner.hi.1.raw() as i64) / 2,
                );
                let order = match route {
                    Some((_, after)) => atlas.places.iter().position(|p| p.arena == after)?,
                    None => order,
                };
                Some((
                    order,
                    Offered {
                        k: k as u8,
                        place: route.map_or(owner.arena.get().name, |(name, _)| name),
                        up: top.y.sub(owner.at.y).to_int(),
                        label: String::new(),
                    },
                    (top.x.raw() as i64 - mid.0, top.z.raw() as i64 - mid.1),
                ))
            })
            .collect();
        out.sort_by_key(|(order, o, _)| (*order, o.up, o.k));
        // Two at the same height in one place say which side of it they are.
        let same = |a: &Offered, b: &Offered| a.place == b.place && (a.up - b.up).abs() <= 2;
        let mut labels: Vec<String> = out.iter().map(|(_, o, _)| format!("{} m", o.up)).collect();
        for i in 0..out.len() {
            if (0..out.len()).any(|j| j != i && same(&out[i].1, &out[j].1)) {
                labels[i] = format!("{} {}", labels[i], compass(out[i].2));
            }
        }
        // And any still alike -- a few on one level of one side -- count.
        let firsts = labels.clone();
        for i in 0..out.len() {
            let alike: Vec<usize> = (0..out.len())
                .filter(|&j| firsts[j] == firsts[i] && out[j].1.place == out[i].1.place)
                .collect();
            if alike.len() > 1 {
                let n = alike.iter().position(|&j| j == i).unwrap_or(0) + 1;
                labels[i] = format!("{} {n}", firsts[i]);
            }
        }
        out.into_iter()
            .zip(labels)
            .map(|((_, o, _), label)| Offered { label, ..o })
            .collect()
    }
}

/// **The route a cairn the layout made is on**, if it is on one of the
/// High Plane's (`sim::arena::heights`): the Sky Road's islands, or the
/// Hanging Stones' column and span, by where their stones are -- with the
/// place the list puts it after, the one it leaves from.
fn route_of(top: sim::V3) -> Option<(&'static str, sim::arena::ArenaId)> {
    use sim::arena::{ArenaId, heights};
    let at = |s: &heights::Stone| {
        (sim::arena::cm(s.x).sub(top.x)).abs().raw() < sim::Fx::ONE.raw()
            && (sim::arena::cm(s.z).sub(top.z)).abs().raw() < sim::Fx::ONE.raw()
    };
    if heights::sky_road().iter().any(at) {
        Some(("Sky Road", ArenaId::SPIRE))
    } else if heights::LADDER.iter().chain(heights::SPAN.iter()).any(at) {
        Some(("Hanging Stones", ArenaId::BANK))
    } else {
        None
    }
}

/// Which way an offset on the map points, as a compass reads it: `x` east
/// and `z` north, as the places are written (`sim::arena::spire`).
fn compass((x, z): (i64, i64)) -> &'static str {
    const NAMES: [&str; 8] = ["E", "NE", "N", "NW", "W", "SW", "S", "SE"];
    if x == 0 && z == 0 {
        return "";
    }
    // Eighths of a turn from +x toward +z, without a float: by the slope.
    let (ax, az) = (x.abs(), z.abs());
    let diagonal = 5 * ax.min(az) >= 2 * ax.max(az);
    let i = match (x >= 0, z >= 0, diagonal, ax >= az) {
        (true, true, true, _) => 1,
        (false, true, true, _) => 3,
        (false, false, true, _) => 5,
        (true, false, true, _) => 7,
        (true, _, false, true) => 0,
        (false, _, false, true) => 4,
        (_, true, false, false) => 2,
        (_, false, false, false) => 6,
    };
    NAMES[i]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_reads_back_what_it_wrote() {
        let mut c = Cairns::default();
        for k in [0, 3, 7] {
            let at = key(k).expect("a cairn");
            c.seen.insert(at);
        }
        let back = Cairns::from_text(&c.to_text());
        assert_eq!(back, c);
        assert!(back.has(3) && !back.has(1));
    }

    #[test]
    fn a_cairn_moved_a_little_is_still_known() {
        let (x, y, z) = key(5).expect("a cairn");
        let c = Cairns::from_text(&format!("{} {} {}\n", x + 40, y - 30, z + 100));
        assert!(c.has(5));
    }

    #[test]
    fn dev_offers_every_cairn_and_a_fresh_player_none() {
        let all = Cairns::default().offered(true);
        assert_eq!(all.len(), sim::valley::cairns().len().min(256));
        assert!(Cairns::default().offered(false).is_empty());
        // Every name a place's, and the Needle's top among them.
        assert!(all.iter().all(|o| !o.place.is_empty()));
        // No two buttons of one place say the same.
        for a in &all {
            let twins = all
                .iter()
                .filter(|b| b.place == a.place && b.label == a.label)
                .count();
            assert_eq!(twins, 1, "{} has two {:?}", a.place, a.label);
        }
        assert!(all.iter().any(|o| o.place == "Needle" && o.up > 300));
        // And nothing up there under the name of the ground it is over: the
        // Needle's top and the Sky Road's islands hang over the town and the
        // Mouth, but the cairns are theirs.
        let high: Vec<_> = all
            .iter()
            .filter(|o| o.place != "Needle" && o.place != "Sky Road" && o.up > 300)
            .collect();
        assert!(high.is_empty(), "{high:?}");
    }

    #[test]
    fn the_compass_reads_x_east_and_z_north() {
        assert_eq!(compass((10, 0)), "E");
        assert_eq!(compass((0, 10)), "N");
        assert_eq!(compass((-10, -10)), "SW");
        assert_eq!(compass((0, -10)), "S");
        assert_eq!(compass((10, 1)), "E");
    }
}
