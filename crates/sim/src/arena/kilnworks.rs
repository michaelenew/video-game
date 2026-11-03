//! **The Kilnworks** (2026-11-03, `crate::versus::Kind::Front`,
//! `docs/design/versus.md` §3a): the first five-point map. A stub until it
//! is built: five points in a row on flat ground.

#![allow(unused_imports)]

use super::hills::{round, slab, turned, whole};
use super::rim::Rim;
use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns, cm};
use crate::drift::Drift;
use crate::math::V3;
use crate::valley::Vent;
use crate::versus::{Hill, Kind, Line, Stage, Way};

use Material::{Ash, Grass, Ground, Peat, Rock, Sand, Snow, Stone, Water, Wood};

pub static ARENA: Arena = Arena {
    id: ArenaId::KILNWORKS,
    name: "Kilnworks",
    creature: None,
    bounds: Bounds::cm((-11500, 11500), (-3500, 3500)),
    floor: Ground,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-10800, 0, (0, 0)), Mark::cm(10800, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    rim: Some(Rim::new((-11500, 11500), (-3500, 3500), [0, 0, 0, 0], 1500)),
};

pub static LINE: Line = Line {
    points: [
        Hill::cm(-9000, 0, 0, 400, 50, 1000),
        Hill::cm(-4500, 0, 0, 400, 50, 1000),
        Hill::cm(0, 0, 0, 400, 50, 1000),
        Hill::cm(4500, 0, 0, 400, 50, 1000),
        Hill::cm(9000, 0, 0, 400, 50, 1000),
    ],
    spawns: [
        Mark::cm(-10800, 0, (0, 0)),
        Mark::cm(-6300, 0, (0, 0)),
        Mark::cm(-1800, 0, (0, 0)),
        Mark::cm(2700, 0, (9000, 0)),
        Mark::cm(7200, 0, (9000, 0)),
    ],
};

pub static STAGE: Stage = Stage {
    arena: ArenaId::KILNWORKS,
    kind: Kind::Front,
    theme: "A brick kiln town in ruins: five points from yard to loading dock",
    pit: None,
    hill: None,
    front: Some(&LINE),
    vents: &[],
    drifts: &[],
    ways: &[],
};

const SOLIDS: [Solid; 3] = whole(
    [Solid::cm([-200, 0, 2500], [200, 300, 2900], Stone)],
    [Solid::cm([-2500, 0, 2500], [-2100, 300, 2900], Stone)],
);
