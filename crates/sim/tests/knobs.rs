//! Every magnitude in the simulation must be reachable from the Oven.
//!
//! The Oven only helps with numbers it can see. A feel number written straight
//! into the code is invisible to it *and* invisible to the bake, so it cannot be
//! tuned in the running game and it cannot be committed from there either —
//! which means the one place the number can be changed is a recompile, and the
//! whole point of the harness is that changing it should not need one.
//!
//! Worse, it can silently disagree with the knob that is supposed to control it.
//! This test was written after finding exactly that: `arena.rs` collided against
//! a hardcoded body radius while the hit test used the Oven's, so tuning the
//! body made fighters a different size to attacks than to walls.
//!
//! The rule: in the simulation, a `Fx` built from a literal, or a `const` of a
//! numeric type, is a tuning value. It belongs in the Oven, or it belongs in
//! `EXEMPT` below with a sentence saying why it is not tuning.

use std::path::Path;

/// Files that *are* the number machinery, or that carry no magnitudes.
///
/// `input.rs` is the wire format -- bit positions and an angle unit, nothing
/// with a size. `curve.rs`, `fixed.rs` and `math.rs` are arithmetic: the `3` in
/// a cubic Bézier is the definition of a cubic Bézier, and tuning it would not
/// change how anything feels, it would stop the curve being a curve.
const NOT_GAMEPLAY: &[&str] = &[
    "tuning.rs",
    "oven.rs",
    "tuned.rs",
    "fixed.rs",
    "math.rs",
    "curve.rs",
    "input.rs",
];

/// Magnitudes that are deliberately not knobs. The reason is the point: an
/// entry without one is just a way to silence the test.
const EXEMPT: &[(&str, &str)] = &[
    (
        "let knee = |u: Fx| ease.mul(u.mul(u).mul(Fx::from_int(2).sub(u)))",
        "The cubic that leaves flat and arrives at the gradient of the line it \
         joins. Solving for those two conditions is what produces the 2; it is \
         the curve's definition rather than a number with a feel to it.",
    ),
    (
        "V3::new(at.x, at.y.add(t::body_height().div(Fx::from_int(2))), at.z)",
        "Halving a body, not choosing a height. The target is the middle of a \
         fighter standing there, and the middle of anything is half of it.",
    ),
    (
        ".add(self.to.sub(self.from).scale(Fx::ONE.div(Fx::from_int(2))))",
        "The midpoint of a hit volume. Halving a line, not choosing a length: the middle \
         of anything is half of it, and any other number would stop it being the middle.",
    ),
    (
        "let tan_tilt = Fx::ONE.sub(ball.at.mul(Fx::from_int(2))).mul(tan_half)",
        "Turning a screen fraction into a tangent. The 2 is that a fraction is \
         measured from the bottom of the screen while the angle is measured from \
         its middle, which is half of it -- geometry, not a number to tune.",
    ),
    (
        "const SHORTEST_STRIDE: Fx = Fx::ratio(1, 10)",
        "A division guard, not a stride. Far below any value the stride constants \
         can produce; it exists so the phase cannot be divided by nearly zero.",
    ),
    (
        "pub const PARRY_FLOURISH: u16 = 14",
        "How long the parry's celebration animation plays. It is a renderer clock kept \
         in the snapshot so it survives a rollback; it decides nothing about combat, \
         and tuning it would change how long a flourish lasts and nothing else.",
    ),
    (
        "const QUARTER_TURN: Fx = Fx::from_raw(1 << 14)",
        "An angle unit, not a quantity. A quarter of the u16 turn space, exact by construction.",
    ),
    (
        "const SKIN: Fx = Fx::ratio(1, 32)",
        "Collision epsilon: how far a body is held off a surface so it does not re-collide. \
         Numerically motivated, not felt.",
    ),
    (
        "const SAMPLES: i32 = 5",
        "How finely a weapon's line is sampled when asking the creature which part it \
         touched. A discretisation of a continuous test, not a quantity: the spacing it \
         produces is smaller than the thinnest weapon in the game, and raising it would \
         make the approximation slightly better rather than make anything feel different.",
    ),
    (
        "const GROUND_Y: Fx = Fx::ZERO",
        "The floor is the origin. Moving it would move the world, not change how it feels.",
    ),
    (
        "pub const ARENA_HALF: Fx = Fx::from_int(14)",
        "Arena geometry is `const` because it is not part of the rollback snapshot. Making the \
         blockout live means having the Oven rebuild SOLIDS, which is its own piece of work.",
    ),
    (
        "pub const WALL_HEIGHT: Fx = Fx::ratio(3, 2)",
        "Arena geometry, and `const` for the same reason as ARENA_HALF: it builds SOLIDS, which \
         is not part of the snapshot.",
    ),
    (
        "pub const DT: Fx = Fx::ratio(1, TICK_HZ as i32)",
        "Derived from the tick rate. The tick rate is a networking decision, not a feel one.",
    ),
    (
        "const TAIL_PIVOT_X: Fx = Fx::ratio(-24, 10)",
        "Where the tail hinges is one of the creature's proportions, and the proportions are \
         `const` for the same reason the arena's geometry is: a shape rather than a feel number, \
         and not part of the rollback snapshot. The number you would actually reach for -- how \
         big the animal is -- is `monster_scale`, which multiplies all of it and is in the Oven.",
    ),
    (
        "const QUARTER: Fx = Fx::from_raw(1 << 14)",
        "An angle unit, not a quantity: it puts the shake's pitch wobble a quarter turn out of \
         phase with its yaw. Exact by construction in the u16 turn space.",
    ),
    (
        "const HALF_TURN: Fx = Fx::from_raw(1 << 15)",
        "An angle unit, not a quantity. Half of the u16 turn space, exact by construction, used \
         to face a spawning creature at the hunters without arithmetic.",
    ),
    (
        "Fx::ratio(1, 2)",
        "Half of an overlap, given to each of the two bodies. Arithmetic, not a knob: any other \
         value would move the pair's centre of mass.",
    ),
    (
        "Fx::ratio(1, 10))",
        "A division guard on the knock travel distance, not a distance itself: it only keeps a \
         retuned range from reaching zero and dividing by it. `bolt_knock_range` is the knob.",
    ),
    (
        "Fx::ratio(1, 100))",
        "A division guard on the decel span, not a span itself: it only keeps a retuned decel \
         start of exactly 1 from reaching zero and dividing by it.",
    ),
    (
        "crate::math::lerp3(self.pos, apex, p.mul(Fx::from_int(2)))",
        "Out and back. The blade covers its path twice in one lifetime, so each leg is half of \
         it and the progress through a leg is twice the progress through the throw. The 2 is \
         the word `back`; `bloodletter_flight` is the knob that decides how long it takes.",
    ),
    (
        "crate::math::lerp3(apex, self.home, p.sub(half).mul(Fx::from_int(2)))",
        "The same 2, on the return leg. See above.",
    ),
    (
        "let bulge = Fx::from_int(4).mul(p).mul(Fx::ONE.sub(p))",
        "What normalises `p(1 - p)` so its peak is exactly one. Without it the widest the \
         Grasp's arms reach would be a quarter of `grasp_spread`, and the knob would be lying \
         about what it means. Tuning it would not widen the cone, it would break the knob.",
    ),
    (
        "let along = if line.len().raw() > Fx::ratio(1, 10).raw() {",
        "A degeneracy guard on the line a thrown ability travels along, not a distance: it only \
         catches the case where the crosshair resolved onto the caster's own hand, which would \
         leave nothing to normalise. The ability's reach is the knob.",
    ),
    (
        "if flat.flat_len().raw() < Fx::ratio(1, 100).raw() {",
        "A degeneracy guard, not a distance: straight up or down there is no horizontal \
         'sideways' for a cone to open into, and this is how near vertical counts as vertical. \
         Any small value does; nothing about the game feels different for a different one.",
    ),
];

fn sim_sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read sim/src") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if !name.ends_with(".rs") || NOT_GAMEPLAY.contains(&name) {
                continue;
            }
            let shown = path
                .strip_prefix(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap(),
                )
                .unwrap_or(&path)
                .display()
                .to_string();
            out.push((shown, std::fs::read_to_string(&path).expect("read source")));
        }
    }
    out.sort();
    out
}

/// Does this line build a quantity out of a literal?
fn is_a_magnitude(line: &str) -> bool {
    let code = line.split("//").next().unwrap_or(line).trim();
    if code.is_empty() {
        return false;
    }
    // A fixed-point value made from a literal.
    for ctor in ["Fx::ratio(", "Fx::from_int(", "Fx::from_raw("] {
        if let Some(rest) = code.split_once(ctor).map(|(_, r)| r) {
            if rest.starts_with(|c: char| c.is_ascii_digit()) {
                return true;
            }
        }
    }
    // A numeric constant. Counts, indices and tags are not magnitudes, and
    // neither is a `const fn` -- that is a signature, not a value.
    let is_const = (code.starts_with("const ") || code.starts_with("pub const "))
        && !code.contains("const fn");
    let quantity = [": Fx", ": i32", ": u16"]
        .iter()
        .any(|ty| code.contains(ty));
    is_const && quantity
}

#[test]
fn every_magnitude_in_the_simulation_is_reachable_from_the_oven() {
    let mut loose = Vec::new();
    for (file, source) in sim_sources() {
        for (i, line) in source.lines().enumerate() {
            if !is_a_magnitude(line) {
                continue;
            }
            let code = line
                .split("//")
                .next()
                .unwrap_or(line)
                .trim()
                .trim_end_matches(';');
            if EXEMPT.iter().any(|(snippet, _)| code.contains(snippet)) {
                continue;
            }
            loose.push(format!("  {}:{}  {}", file, i + 1, code));
        }
    }
    loose.sort();
    assert!(
        loose.is_empty(),
        "these magnitudes are where the Oven cannot reach them.\n\
         Put each one in the Oven so it can be tuned and baked, or add it to EXEMPT in this file \
         with the reason it is not a tuning value:\n{}",
        loose.join("\n")
    );
}

#[test]
fn every_exemption_gives_a_reason() {
    for (snippet, reason) in EXEMPT {
        assert!(
            reason.len() > 40,
            "{snippet} is exempt without a real reason -- an exemption nobody had to justify is \
             just a way to silence the test"
        );
    }
}
