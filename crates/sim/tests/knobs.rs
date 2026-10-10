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

/// **Level data**: files that say where things are and what the ground is
/// shaped like, the way an arena's table of boxes does -- the valley's land
/// (how wide a floor, how steep a mountainside, how loud the noise on it) and
/// its layout (where a crag stands, how far apart the trees). They are judged
/// by looking at them, on the plan sheet (`cargo run -p look --example
/// atlas`) and in the game, not by how a fight feels; the numbers that are
/// about feel -- how steep a body walks, how fast it slides -- are in the
/// Oven. A fight's rim (how steep its bank, how far its noise grows) and the
/// town's tables (how far apart the merlons, how high the wall) are the same
/// kind of thing.
const LEVEL_DATA: &[&str] = &["land.rs", "layout.rs", "rim.rs", "hearth.rs"];

/// Magnitudes that are deliberately not knobs. The reason is the point: an
/// entry without one is just a way to silence the test.
const EXEMPT: &[(&str, &str)] = &[
    (
        "let third = Fx::ratio(1, 3)",
        "Presentation: where a Fire carpet's drawn flame is widest along its trip. The hit \
         test is the capsule, and no flame is drawn wider than it.",
    ),
    (
        "let thins_by = Fx::ratio(3, 4)",
        "Presentation: how far a drawn carpet flame thins by the end of its trip.",
    ),
    (
        "let half = Fx::ratio(1, 2)",
        "Presentation: how small a drawn carpet flame starts, against the strip that burns.",
    ),
    (
        "const CARPET_FLOW: u16 = 30",
        "Presentation: how fast the drawn flames run along a Fire carpet, like the bead \
         counts beside it. The carpet burns the same whatever this is.",
    ),
    (
        "pub const EARTH: u16 = Input::LEFT",
        "A binding, not a magnitude: which button is the Elementalist's earth click.",
    ),
    (
        "pub const FIRE: u16 = Input::MIDDLE",
        "A binding, not a magnitude: which button is her fire click.",
    ),
    (
        "pub const WIND: u16 = Input::RIGHT",
        "A binding, not a magnitude: which button is her wind click.",
    ),
    (
        "pub const STRONG_PUSH: u16 = Input::MECHANIC",
        "A binding, not a magnitude: which button is her strong push.",
    ),
    (
        "pub const WEAK_PUSH: u16 = Input::SPECIAL",
        "A binding, not a magnitude: which button is her weak push.",
    ),
    (
        "pub const DARK: u16 = Input::LEFT",
        "A binding, not a magnitude: which button is the Dual mage's dark click.",
    ),
    (
        "pub const TWILIGHT: u16 = Input::MIDDLE",
        "A binding, not a magnitude: which button is the Dual mage's twilight click.",
    ),
    (
        "pub const LIGHT: u16 = Input::RIGHT",
        "A binding, not a magnitude: which button is the Dual mage's light click.",
    ),
    (
        "pub const DARK_MAJOR: u16 = Input::SPECIAL",
        "A binding, not a magnitude: which button is the Dual mage's dark major.",
    ),
    (
        "pub const LIGHT_MAJOR: u16 = Input::MECHANIC",
        "A binding, not a magnitude: which button is the Dual mage's light major.",
    ),
    (
        "pub const BLADE: u16 = Input::LEFT",
        "A binding, not a magnitude: which button is the Shadow Reaver's blade click.",
    ),
    (
        "pub const EXECUTION: u16 = Input::MIDDLE",
        "A binding, not a magnitude: which button is the Shadow Reaver's execution click.",
    ),
    (
        "pub const SHADOW: u16 = Input::RIGHT",
        "A binding, not a magnitude: which button is the Shadow Reaver's shadow click.",
    ),
    (
        "pub const LOTUS: u16 = Input::SPECIAL",
        "A binding, not a magnitude: which button is the Shadow Reaver's lotus key.",
    ),
    (
        "pub const MISTAKE: u16 = Input::MECHANIC",
        "A binding, not a magnitude: which button is the Shadow Reaver's Deadly mistake key.",
    ),
    (
        "pub const STRIKE: u16 = Input::LEFT",
        "A binding, not a magnitude: which button is the Bulwark's strike click.",
    ),
    (
        "pub const WEIGHT: u16 = Input::MIDDLE",
        "A binding, not a magnitude: which button is the Bulwark's weight click.",
    ),
    (
        "pub const GUARD: u16 = Input::RIGHT",
        "A binding, not a magnitude: which button is the Bulwark's guard click.",
    ),
    (
        "pub const MY_BLOOD: u16 = Input::LEFT",
        "A binding, not a magnitude: which button is the Blood mage's my-blood click.",
    ),
    (
        "pub const YOUR_BLOOD: u16 = Input::MIDDLE",
        "A binding, not a magnitude: which button is the Blood mage's your-blood click.",
    ),
    (
        "pub const SCYTHE: u16 = Input::RIGHT",
        "A binding, not a magnitude: which button is the Blood mage's scythe click.",
    ),
    (
        "pub const BLOODLETTER: u16 = Input::SPECIAL",
        "A binding, not a magnitude: which button is the Blood mage's Bloodletter.",
    ),
    (
        "pub const SPIKE: u16 = Input::MECHANIC",
        "A binding, not a magnitude: which button is the Blood mage's Black spike.",
    ),
    (
        "r.run().add(Fx::from_int(16)).mul(Fx::from_int(2))",
        "How far past an arena's bounds the aiming ray walks the floor to reach its rim's \
         bank: the bank and the foothills past it, both sides. A reach, not a tuning value.",
    ),
    (
        "rise.raw() <= steepest.mul(run).add(Fx::ratio(1, 50)).raw()",
        "Rounding slack in the steepness test: two samples of the land a step apart differ \
         by a few thousandths along a contour. The steepness itself is the knob.",
    ),
    (
        "V3::new(p.pos.x, Fx::from_int(1000), p.pos.z)",
        "Asking for the floor from above everything: a height over any place, not a height \
         anything is at.",
    ),
    (
        "V3::new(mark.at.x, Fx::from_int(1000), mark.at.z)",
        "The same: the floor under a mark, asked from above everything.",
    ),
    (
        "V3::new(m.at.x, Fx::from_int(1000), m.at.z)",
        "The same: the floor under a mark, asked from above everything.",
    ),
    (
        "pub const HALF: i32 = 400",
        "Level data, like an arena's table: how wide a crag is.",
    ),
    (
        "pub const RISE: i32 = 220",
        "Level data: how far a crag's ledges are apart, sized to the plainest jump \
         (`tests/valley.rs` holds every class to it).",
    ),
    (
        "pub const OUT: i32 = 160",
        "Level data: how far a crag's ledge stands out of its face.",
    ),
    (
        "pub const MAX_SPAN_M: i32 = 16_000",
        "What 16.16 can hold, not a size anything is: the widest a map can be with \
         every place still able to see every other.",
    ),
    (
        "const RESOLVE_REACH: Fx = Fx::from_int(2)",
        "How far round a body the map's tiles are read for the resolve: a search radius, \
         not a size. It has to exceed the furthest one box's push can carry a body in a \
         frame, and any larger value gives the same answer more slowly.",
    ),
    (
        "const CEILING_REACH: Fx = Fx::from_int(16)",
        "How far round a point the map is searched for a sloped ceiling: a search radius. \
         Past it the slope has carried any ceiling above the eye already; the slope is \
         the knob.",
    ),
    (
        "pub const EXTRA: u16 = u16::MAX",
        "A marker meaning 'no place', not a quantity.",
    ),
    (
        "pub const TILE_M: i32 = 16",
        "The side of a map tile: how the world is indexed, not anything in it. Any size \
         gives the same answers; this one only decides how fast they come.",
    ),
    (
        "pub const APRON: i32 = 600",
        "Level design, like an arena table's boxes: how much ground the valley's layout \
         puts round a room. Where things are, not how anything feels.",
    ),
    (
        "const CLIFF_OVER: i32 = 400",
        "Level design: how high the cliff round a room stands over its tallest box.",
    ),
    (
        "const DOOR: i32 = 600",
        "Level design: how wide a doorway into a room is cut.",
    ),
    (
        "const DOOR_HIGH: i32 = 600",
        "Level design: how high every doorway is cut over its floor.",
    ),
    (
        "const CAP: i32 = 300",
        "Level design: how far either side of a joint the caps of two passages are cut. \
         It has to take in the caps' thickness and nothing else.",
    ),
    (
        "const VOID: i32 = -10_000",
        "Where there is no place: a floor far enough under the map that a body there has \
         fallen out of it. Not a height anything is at.",
    ),
    (
        "let across = t::structure_radius().mul(Fx::from_int(2))",
        "A stone's diameter, for the broken ground a pressed stone leaves: twice its \
         radius, because that is what a diameter is. The radius is the knob.",
    ),
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
        "ground.y.add(height.div(Fx::from_int(2))),",
        "The same halving, in `aim::standing_middle`: where a shot aimed at a \
         patch of floor actually goes. Not a height with a feel to it -- it is \
         the middle of whatever is standing on that patch, and the middle of \
         anything is half of it.",
    ),
    (
        "let cm = |v: Fx| v.mul(Fx::from_int(100)).to_int().clamp(-32000, 32000) as i16",
        "Metres to centimetres, for the two bytes a critter keeps per axis of its perch \
         on a creature (`critter::Critter::perch`). A unit conversion, not a size: the \
         100 is what a centimetre is, and the clamp is what an i16 holds.",
    ),
    (
        ".add(self.to.sub(self.from).scale(Fx::ONE.div(Fx::from_int(2))))",
        "The midpoint of a hit volume. Halving a line, not choosing a length: the middle \
         of anything is half of it, and any other number would stop it being the middle.",
    ),
    (
        ".sub(ball.at.mul(Fx::from_int(2)))",
        "Turning a screen fraction into a tangent. The 2 is that a fraction is \
         measured from the bottom of the screen while the angle is measured from \
         its middle, which is half of it -- geometry, not a number to tune.",
    ),
    (
        "let screen = standing.radius.mul(tan_half_fov()).mul(Fx::from_int(2))",
        "The same 2, used the other way round: the height of the screen at the \
         sphere's surface is twice the half-height a tangent gives, because a \
         screen has two halves. It turns a share of the screen into metres so \
         the camera can hold still for exactly the rise that carries a fighter \
         up to the crosshair and no further.",
    ),
    (
        ".div(self.sp().brake().mul(Fx::from_int(2)))",
        "The distance to stop from a speed at a constant deceleration is the \
         speed squared over twice the deceleration. The 2 is that formula -- \
         kinematics, not a number with a feel to it. The feel is the braking \
         rate, and that is in the Oven.",
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
        "pub const DT: Fx = Fx::ratio(1, TICK_HZ as i32)",
        "Derived from the tick rate. The tick rate is a networking decision, not a feel one.",
    ),
    (
        "const SMALLEST: Fx = Fx::from_raw(1)",
        "The smallest number 16.16 can hold. It guards two divisions from a retuned speed of \
         exactly zero; it is not a speed, and no value other than the representation's own floor \
         would be correct.",
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
            if !name.ends_with(".rs") || NOT_GAMEPLAY.contains(&name) || LEVEL_DATA.contains(&name)
            {
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
