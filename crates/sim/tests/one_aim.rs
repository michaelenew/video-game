//! There is one aiming model, and one place that knows it.
//!
//! Aiming has been got wrong three times, the same way each time: someone
//! needed to know where an ability should go, wrote the intersection next to
//! the ability that needed it, and got a ray that is *parallel* to the
//! crosshair's rather than the crosshair's. Parallel rays never meet, so the
//! reticle sat on one thing and the ability went past it — by more the further
//! away it was.
//!
//! The fix that sticks is structural. **`crate::math` owns ray-against-shape
//! arithmetic. `crate::aim` owns every decision about where an ability goes.
//! Nothing else in the simulation may do either.** This test is what makes that
//! true rather than merely written down: it reads the simulation's own source
//! and fails if the primitives, the camera's eye, or the look direction are
//! reached for anywhere else.
//!
//! If you are here because this test failed: you almost certainly want
//! [`sim::aim::grounded_path`] or [`sim::aim::skillshot_path`], and if neither
//! fits, change them — a change there is true of every ability at once, which
//! is the whole point.

use std::path::{Path, PathBuf};

/// The primitives, and the two inputs the aiming ray is built from.
///
/// The eye and the look direction are on the list because they are how the
/// mistake is made: a module that has both can write the parallel ray in two
/// lines and will look perfectly reasonable doing it.
const RESERVED: &[(&str, &str)] = &[
    ("ray_hits_cylinder(", "ray against a shape"),
    ("ray_hits_box(", "ray against a shape"),
    ("camera::eye(", "where the aiming ray starts"),
    (".look_dir()", "where the aiming ray points"),
];

/// Files allowed to use them, and why.
///
/// An entry without a reason is just a way to silence the test.
const ALLOWED: &[(&str, &str)] = &[
    (
        "aim.rs",
        "The aiming model itself. This is the file every other one is being \
         pointed at.",
    ),
    (
        "math.rs",
        "Defines the primitives. Pure arithmetic over fixed point, with no \
         opinion about what is being aimed.",
    ),
    (
        "input.rs",
        "Defines `look_dir` as the wire format's own accessor: two integer \
         angles turned into a unit vector, which is a conversion rather than a \
         decision about where anything goes.",
    ),
    (
        "camera.rs",
        "Defines `eye`. Where the camera sits is the camera's business; what to \
         do with it is not.",
    ),
    (
        "monster.rs",
        "The creature's parts are boxes authored in its own body space, behind \
         an articulation only this file knows how to undo. `part_struck_along` \
         is that geometry, and `aim` calls it -- the creature is a thing you \
         can aim at, not a second aiming model.",
    ),
];

fn sim_sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read sim/src") {
            let path: PathBuf = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            if !name.ends_with(".rs") {
                continue;
            }
            out.push((name, std::fs::read_to_string(&path).expect("read source")));
        }
    }
    out.sort();
    out
}

#[test]
fn only_the_aim_module_decides_where_an_ability_goes() {
    let allowed: Vec<&str> = ALLOWED.iter().map(|(f, _)| *f).collect();
    let mut loose = Vec::new();
    for (file, source) in sim_sources() {
        if allowed.contains(&file.as_str()) {
            continue;
        }
        for (i, line) in source.lines().enumerate() {
            // Prose is allowed to name them; that is how the rule gets read.
            let code = line.split("//").next().unwrap_or(line);
            for (needle, what) in RESERVED {
                if code.contains(needle) {
                    loose.push(format!(
                        "  crates/sim/src/{file}:{}  {needle}  ({what})",
                        i + 1
                    ));
                }
            }
        }
    }
    assert!(
        loose.is_empty(),
        "these reach past `crate::aim` and decide for themselves where something \
         goes.\nThat is how the crosshair and the ability came to disagree, three \
         times. Use `aim::grounded_path` or `aim::skillshot_path`; if neither \
         fits, change them, and the change is then true of every ability at \
         once. If the file genuinely owns a shape of its own, add it to ALLOWED \
         in this test with the reason:\n{}",
        loose.join("\n")
    );
}

#[test]
fn every_exemption_gives_a_reason() {
    for (file, reason) in ALLOWED {
        assert!(
            reason.len() > 40,
            "{file} is allowed without a real reason -- an exemption nobody had \
             to justify is just a way to silence the test"
        );
    }
}

#[test]
fn every_move_says_which_line_of_effect_it_uses() {
    // The matrix is meant to be exhaustive: no move without an answer, and no
    // fifth kind. A caller that had to guess is a caller that would eventually
    // guess differently from the last one.
    use sim::aim::Kind;
    use sim::class::ALL_CLASSES;

    let mut kinds = Vec::new();
    for class in ALL_CLASSES {
        for slot in 0..sim::moves::SLOTS {
            let m = sim::moves::get(class, slot as u8);
            kinds.push(m.aim());
            // The two directions that can be checked from the move's own data.
            // A move may be grounded without leaving anything behind -- Fissure
            // races along the floor and plants a structure, which is the
            // mechanic's business rather than the effects array's -- so this
            // only runs the way round that is always true.
            if let Some(leaves) = sim::effects::EffectKind::from_code(m.effect) {
                if leaves.grounded() {
                    assert_eq!(
                        m.aim(),
                        Kind::Grounded,
                        "{} {} plants something on the floor but is not aimed at the floor",
                        class.name(),
                        m.name
                    );
                }
                // A thing that flies through the air has to be pointed by the
                // player, and there are exactly two ways to do that. The
                // crosshair picks the point it flies to -- that is a skillshot
                // -- or the body picks the line and a **channel** picks the
                // distance along it, which is the Grasp. Anything else means a
                // thrown ability that goes somewhere nobody chose, which is the
                // bug this whole file exists to stop coming back.
                if leaves.travels() {
                    let pointed = match m.aim() {
                        Kind::Skillshot => true,
                        // Swing aiming without a channel is the bug in its
                        // original form: the flat facing, and no answer at all
                        // to how far. With one, the hold is the answer and the
                        // dead-zoned pitch is what makes a depth readable
                        // against the floor -- see the Grasp in
                        // `docs/design/kits/blood-mage.md`.
                        Kind::Swing => m.channels(),
                        Kind::Grounded | Kind::AtTheMechanic => false,
                    };
                    assert!(
                        pointed,
                        "{} {} throws something that travels but is aimed as {:?}, \
                         which does not let the player say where it goes",
                        class.name(),
                        m.name,
                        m.aim()
                    );
                }
            }
        }
    }
    for wanted in [
        Kind::Grounded,
        Kind::Skillshot,
        Kind::Swing,
        Kind::AtTheMechanic,
    ] {
        assert!(
            kinds.contains(&wanted),
            "no move in the roster is aimed `{}` any more, so that line of effect \
             is untested by everything else in the suite",
            wanted.name()
        );
    }
}

#[test]
fn a_move_aimed_at_the_mechanic_needs_one_to_be_out() {
    // The one kind that can have nowhere to go. `mechanic_path` falls back to
    // the caster's own feet, which is a volume on your own body -- fine as a
    // last resort, useless as a design. The move table's gate is what stops it
    // happening: you cannot throw the move without the mechanic placed.
    use sim::aim::Kind;
    use sim::class::ALL_CLASSES;

    for class in ALL_CLASSES {
        for slot in 0..sim::moves::SLOTS {
            let m = sim::moves::get(class, slot as u8);
            if m.aim() != Kind::AtTheMechanic {
                continue;
            }
            assert!(
                m.needs_mechanic,
                "{} {} erupts at the mechanic but can be thrown without one out",
                class.name(),
                m.name
            );
        }
    }
}
