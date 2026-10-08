//! Every arena's sky, in one table.
//!
//! One file, one line per arena, because that is what makes the next arena
//! cheap. Scattered across the fourteen files that build the arenas, a sky is
//! fourteen places to look and twenty-three colours nobody can compare; here
//! they are a list you can read top to bottom and a sheet you can render in one
//! command (`cargo run -p look --example skies`).
//!
//! **Most of these are one colour.** [`Sky::over`] does the rest, and what it
//! does is described in [`crate::sky`]: deeper overhead, paler below. An arena
//! that wants something the derivation cannot reach -- a cave, or a course
//! above the clouds -- writes it out, and whatever that turns out to teach gets
//! folded back into the derivation so the next arena starts closer.
//!
//! **Bright and high-chroma, not grey.** Pastel is high lightness with real
//! colour still in it; high lightness with the colour taken out is fog, and a
//! grey horizon reads as a fog bank somebody forgot to clear rather than as
//! sky. So every horizon here names a hue and means it.

use crate::Sky;
use sim::arena::ArenaId;

/// The sky over an arena. Every id has one: an arena nobody has given a sky
/// gets a plain bright day rather than a black void, which is the failure that
/// is easy to miss and ugly to ship.
pub fn of(id: ArenaId) -> Sky {
    match id {
        // --- dev arenas ---------------------------------------------------
        //
        // These are where everything else is judged, so they are a clean,
        // bright, unremarkable day. A dev arena with a mood would quietly tint
        // every decision made in it.
        ArenaId::PROVING_GROUND => Sky::over([0.78, 0.87, 0.95]),
        ArenaId::RANGE => Sky::over([0.76, 0.86, 0.95]),
        // Deliberately the flattest sky in the game: the lab is for reading
        // numbers off a jump, and a gradient behind the subject is a gradient
        // you end up measuring.
        ArenaId::LAB => Sky {
            zenith: [0.70, 0.74, 0.78],
            ground: [0.82, 0.84, 0.86],
            ..Sky::over([0.80, 0.83, 0.86])
        },
        ArenaId::BENCH => Sky::over([0.80, 0.86, 0.92]),

        // --- the creature arenas ------------------------------------------
        //
        // One colour apiece, chosen for the hour and the place. The haze below
        // and the deep overhead follow from it.

        // The commons, early and overcast: light the colour of a cloud with the
        // faintest lilac in it, which is what an overcast actually is.
        ArenaId::GNAWERS => Sky::over([0.84, 0.85, 0.92]),
        // Open meadow at midday.
        ArenaId::HORNBACK => Sky::over([0.80, 0.90, 0.93]),
        // The same country an hour later and dustier, so a touch warmer.
        ArenaId::HORNBACK_CROSSING => Sky::over([0.85, 0.89, 0.88]),
        // The mire. Green, but a *bright* green -- sun through leaves over
        // standing water, not the brown it used to be. The reach is short
        // because the air here is thick, and that is the whole character of the
        // place: you cannot see what is coming.
        ArenaId::MIREBACK => Sky {
            reach: 260.0,
            ..Sky::over([0.80, 0.91, 0.82])
        },
        // The pan, at the hottest part of the day: bleached cream-yellow, and
        // the sun's glow is the loudest in the game because there is nothing
        // between you and it.
        ArenaId::SANDMAW => Sky {
            glow: Some([0.95, 0.72, 0.30]),
            reach: 700.0,
            ..Sky::over([0.98, 0.92, 0.76])
        },
        // The den at dusk. The warmest sky here, and the one furthest from a
        // real one: a dusk this pink is a painting, which is the point.
        ArenaId::PAIR => Sky {
            glow: Some([0.90, 0.38, 0.30]),
            reach: 420.0,
            ..Sky::over([0.98, 0.76, 0.72])
        },
        // The hollows. Underground, so the gradient runs the other way: there
        // is no sky, and what little light there is comes off the walls. Not
        // black, though -- a violet cast, so the dark reads as a lit cave
        // rather than as a rendering failure, and a short reach so the far wall
        // of a cave is lost in it.
        ArenaId::BROODMOTHER => Sky {
            zenith: [0.03, 0.03, 0.05],
            horizon: [0.11, 0.09, 0.16],
            ground: [0.06, 0.05, 0.09],
            glow: None,
            reach: 110.0,
        },
        // Ashwood under snow: almost white, and cold -- the blue has to stay in
        // it or the snow reads as paper.
        ArenaId::VEILSTALKER => Sky {
            reach: 340.0,
            ..Sky::over([0.86, 0.91, 0.98])
        },
        // The shrine, late afternoon: pale apricot, the hour everything is lit
        // from the side.
        ArenaId::MANTIS => Sky {
            glow: Some([0.88, 0.52, 0.22]),
            ..Sky::over([0.97, 0.87, 0.76])
        },
        // The cliffs, and the highest open sky in the game. Thin air, so the
        // most saturated blue and the longest reach of any arena: you are meant
        // to look out and see a long way.
        ArenaId::GALEWING => Sky {
            reach: 1200.0,
            ..Sky::over([0.70, 0.85, 0.98])
        },
        // The last valley, at the end of the day: rose and gold over dust.
        ArenaId::SIEGESHELL => Sky {
            glow: Some([0.86, 0.44, 0.26]),
            reach: 600.0,
            ..Sky::over([0.96, 0.84, 0.80])
        },

        // --- the valley ------------------------------------------------------
        //
        // One day, climbing: a warm morning in the town, the river's fresh
        // light at the Mouth, noon on the Shelves, the wood's green afternoon,
        // and the thinnest, bluest air of the game on the Saddle. The reach
        // grows with the height, so the higher you are the further you see.
        ArenaId::HEARTH => Sky {
            reach: 500.0,
            ..Sky::over([0.93, 0.88, 0.80])
        },
        ArenaId::RING => Sky::over([0.94, 0.87, 0.79]),
        ArenaId::MOUTH => Sky {
            reach: 520.0,
            ..Sky::over([0.80, 0.89, 0.93])
        },
        ArenaId::BANK => Sky {
            reach: 560.0,
            ..Sky::over([0.84, 0.90, 0.89])
        },
        ArenaId::SHELVES => Sky {
            reach: 720.0,
            ..Sky::over([0.76, 0.86, 0.96])
        },
        ArenaId::PINEWOOD => Sky {
            reach: 600.0,
            ..Sky::over([0.82, 0.90, 0.85])
        },
        ArenaId::SADDLE => Sky {
            reach: 1100.0,
            ..Sky::over([0.73, 0.84, 0.98])
        },
        ArenaId::HIGHLANDS => Sky::over([0.82, 0.86, 0.92]),

        // --- the jump courses ---------------------------------------------
        //
        // Nine siblings, one sky, nine turns of the wheel. See `ALOFT`.
        ArenaId::CLIMB_STAIR => aloft(-0.12),
        ArenaId::CLIMB_CAUSEWAY => aloft(-0.09),
        ArenaId::CLIMB_SPIRAL => aloft(-0.06),
        ArenaId::CLIMB_FALLS => aloft(-0.03),
        ArenaId::CLIMB_SPIRE => aloft(0.00),
        ArenaId::CLIMB_GULF => aloft(0.03),
        ArenaId::CLIMB_SLALOM => aloft(0.06),
        ArenaId::CLIMB_FORK => aloft(0.09),
        ArenaId::CLIMB_REACH => aloft(0.12),

        _ => Sky::over([0.78, 0.87, 0.95]),
    }
}

/// The jump courses' sky: dawn, **above the cloud deck**.
///
/// This one is written out rather than derived, because it is the sky that has
/// to do a job. The complaint it answers is that a course sixty metres up does
/// not feel sixty metres up, and three things in here are that feeling:
///
/// **What is below is bright, not dark.** Looking down off an island you see
/// the top of a cloud layer, which is nearly as pale as the sky. The derivation
/// would have given a dim version of the horizon -- fine for an arena with
/// ground under it, and for a course it throws away the one view that says how
/// high you are.
///
/// **The gradient is long.** Deep periwinkle overhead to warm peach at eye
/// level is most of the way across the wheel, so there is a visible horizon
/// line from anywhere on the course. A horizon line is the only thing on screen
/// that says which way is level.
///
/// **The reach is short for the size of the course.** 520 m against courses
/// that run longer than that, so the far end of a causeway is visibly hazed
/// while the near end is crisp. That is the depth cue doing the work; without
/// it a two-hundred-metre gap reads as a short one.
pub const ALOFT: Sky = Sky {
    zenith: [0.28, 0.33, 0.74],
    horizon: [0.99, 0.82, 0.74],
    ground: [0.73, 0.76, 0.93],
    glow: Some([0.92, 0.42, 0.20]),
    reach: 520.0,
};

/// `ALOFT`, turned round the wheel.
///
/// Nine courses, nine dawns of the same dawn. Turning the whole scheme together
/// keeps what makes it read as height -- the long gradient, the bright deck
/// below, the low glow -- and changes only the key, so the courses are
/// distinguishable at a glance without any of them being a different idea.
///
/// **A band, not a sweep.** The first version walked a half turn across the
/// nine, which put four of them in the greens: a green dawn is not a mood, it
/// is a bug that happens to compile. A twelfth of a turn either side of the
/// dawn covers peach, rose, pink and lilac -- plainly nine different skies, all
/// of them a sky somebody has stood under.
fn aloft(turns: f32) -> Sky {
    ALOFT.turned(turns)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tint::Lch;

    #[test]
    fn every_registered_arena_has_a_sky_worth_looking_at() {
        for arena in sim::arena::all() {
            let sky = of(arena.id).resolved();
            let name = arena.name;
            for part in [sky.zenith, sky.horizon, sky.ground] {
                assert!(
                    part.iter()
                        .all(|c| c.is_finite() && (0.0..=1.0).contains(c)),
                    "{name}: {part:?} is not a colour"
                );
            }
            assert!(sky.reach > 50.0, "{name}: you cannot see {} m", sky.reach);
        }
    }

    #[test]
    fn the_open_air_arenas_are_bright_and_not_grey() {
        // The two halves of pastel. Light enough to read as daylight, and with
        // enough colour left in it to read as sky rather than as fog -- the
        // failure that produced an all-brown wash the first time round.
        for arena in sim::arena::all() {
            if arena.id == ArenaId::BROODMOTHER {
                continue; // A cave. The one place dark is the right answer.
            }
            let h = Lch::of(of(arena.id).resolved().horizon);
            let name = arena.name;
            assert!(h.l > 0.80, "{name}: the horizon is dim ({:.2})", h.l);
            assert!(h.c > 0.010, "{name}: the horizon is grey ({:.3})", h.c);
        }
    }

    #[test]
    fn a_course_sees_its_own_floor_hazed() {
        // The height cue, as a number: the far end of a course has to be well
        // into the fog or the drop is not in the picture.
        let sky = of(ArenaId::CLIMB_GULF);
        let (start, _) = sky.fog();
        assert!(
            start < 160.0,
            "haze starts {start} m out, past most of a course"
        );
        // And what is below is *pale*: brighter than the deep overhead, because
        // it is cloud, not earth.
        let sky = sky.resolved();
        assert!(
            Lch::of(sky.ground).l > Lch::of(sky.zenith).l + 0.3,
            "the deck below a course is not reading as cloud"
        );
    }

    #[test]
    fn the_nine_courses_are_siblings_and_not_twins() {
        // Distinguishable, but plainly the same hour of the same world: the
        // lightness structure is shared, only the key moves.
        let ids = [
            ArenaId::CLIMB_STAIR,
            ArenaId::CLIMB_CAUSEWAY,
            ArenaId::CLIMB_SPIRAL,
            ArenaId::CLIMB_FALLS,
            ArenaId::CLIMB_SPIRE,
            ArenaId::CLIMB_GULF,
            ArenaId::CLIMB_SLALOM,
            ArenaId::CLIMB_FORK,
            ArenaId::CLIMB_REACH,
        ];
        let base = Lch::of(ALOFT.horizon);
        for id in ids {
            let h = Lch::of(of(id).resolved().horizon);
            assert!(
                (h.l - base.l).abs() < 0.08,
                "a course's horizon drifted out of the family: {:.2} vs {:.2}",
                h.l,
                base.l
            );
        }
        for (a, b) in ids.iter().zip(ids.iter().skip(1)) {
            let (x, y) = (of(*a).resolved().zenith, of(*b).resolved().zenith);
            let apart: f32 = x.iter().zip(y).map(|(p, q)| (p - q).abs()).sum();
            assert!(apart > 0.04, "two courses got the same sky: {x:?} {y:?}");
        }
    }
}
