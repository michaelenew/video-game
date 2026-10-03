//! **What each class can actually do with its feet**: the movement envelope,
//! played rather than read off the prose. The instrument the jump courses were
//! built against (`docs/design/courses.md`); `cargo run -p sim --bin envelope`
//! prints it.
//!
//! Everything here is a scripted input into a [`World`] standing in the lab
//! (`crate::arena::lab`), the same way `reachcheck` and `critcheck` play a
//! hunt: a pilot that sends buttons and a look every frame, and the fight's own
//! answer for where the body went. Three kinds of number come out:
//!
//! - **The jump**, on the flat: a full hop's apex and airtime, and how far a
//!   running jump carries.
//! - **Gap and rise**, from a real edge: a running jump off the end of the
//!   lab's 20 m runway, recorded all the way down, then replayed against a
//!   landing ledge at every gap and rise through `Arena::resolve` -- the
//!   simulation's own rule for whether a body lands on a top or meets a face.
//!   Plain, and with the airdodge thrown at its best frame.
//! - **Up onto a ledge, with the class's tools**: the lab's eight ledges, 2 to
//!   20 m tall; a pilot per tool and per class, scanned over its timing, from
//!   a standing start at each distance. A tool that cannot be played cheaply
//!   says so rather than guessing (`Tool::note`).
//!
//! Integer arithmetic only: this lives under `crates/sim/src`, and the
//! no-floats guard covers it. It builds worlds and keeps trajectories, so it
//! allocates; it is a tool, not a frame.

use crate::arena::{self, Arena, ArenaId, Bounds, Material, Solid, cm};
use crate::class::{Class, Ghost, Mechanic};
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::{self, V3};
use crate::state::{Action, World};

/// Metres, from a whole number of them.
fn m(v: i32) -> Fx {
    Fx::from_int(v)
}

/// Centimetres from metres, for printing without a float.
pub fn to_cm(v: Fx) -> i32 {
    ((v.raw() as i64 * 100 + (1 << 15)) >> 16) as i32
}

/// A length as "12.3", from metres.
pub fn metres(v: Fx) -> String {
    let c = to_cm(v);
    let sign = if c < 0 { "-" } else { "" };
    let c = c.abs();
    format!("{sign}{}.{}", c / 100, (c % 100) / 10)
}

// ---------------------------------------------------------------------------
// The lab
// ---------------------------------------------------------------------------

/// A fighter of `class` alone in the lab: the second fighter in the far
/// corner, out of everybody's way.
pub fn lab(class: Class) -> World {
    let mut w = World::versus_in([class, Class::Bulwark], ArenaId::LAB);
    w.players[1].health = i32::MAX / 2;
    w
}

/// Put fighter one at `at`, standing still, facing `+x`.
pub fn stand(w: &mut World, at: V3) {
    let p = &mut w.players[0];
    p.pos = at;
    p.vel = V3::ZERO;
    p.facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    p.grounded = true;
    if let Mechanic::Shadow(_) = p.mechanic {
        p.mechanic = Mechanic::Shadow(crate::class::Shadow::attending(p.pos, p.facing));
    }
}

/// One frame: fighter one sends `input`, the other nothing.
pub fn tick(w: &mut World, input: Input) {
    w.advance([input, Input::default()]);
}

/// The yaw and pitch that put fighter one's crosshair on `at`: the raycast
/// run backwards, `aim::look_onto_closely`.
pub fn look_at(w: &World, at: V3) -> (u16, i16) {
    let p = &w.players[0];
    let d = at.sub(p.pos);
    let yaw = (math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16;
    (yaw, crate::aim::look_onto_closely(p.pos, yaw, p.aloft, at))
}

/// Buttons, looking at a point.
pub fn toward(w: &World, bits: u16, at: V3) -> Input {
    let (yaw, pitch) = look_at(w, at);
    Input::looking_at(bits, yaw, pitch)
}

/// Buttons, looking along `+x`, level.
pub fn ahead(bits: u16) -> Input {
    Input::aimed(bits, 0)
}

/// Fighter one's flat speed, horizontal.
fn flat_speed(w: &World) -> Fx {
    let v = w.players[0].vel;
    V3::new(v.x, Fx::ZERO, v.z).flat_len()
}

// ---------------------------------------------------------------------------
// The jump, on the flat
// ---------------------------------------------------------------------------

/// A hop from standing, space held for `hold` frames: its apex and how many
/// frames until the feet are down again.
pub fn hop(class: Class, hold: u32) -> (Fx, u32) {
    let mut w = lab(class);
    let (x, z) = arena::lab::LANE;
    stand(&mut w, V3::new(cm(x), Fx::ZERO, cm(z)));
    let mut apex = Fx::ZERO;
    for f in 0..400u32 {
        let bits = if f < hold { Input::SPACE } else { 0 };
        tick(&mut w, ahead(bits));
        let p = &w.players[0];
        apex = apex.max(p.pos.y);
        if f > 1 && p.grounded {
            return (apex, f);
        }
    }
    (apex, 400)
}

/// A run along the floor: metres a second at full stride.
pub fn run_speed(class: Class) -> Fx {
    let mut w = lab(class);
    let (x, z) = arena::lab::LANE;
    stand(&mut w, V3::new(cm(x), Fx::ZERO, cm(z)));
    for _ in 0..30 {
        tick(&mut w, ahead(Input::W));
    }
    flat_speed(&w)
}

// ---------------------------------------------------------------------------
// Off the runway: the trajectory every gap is read from
// ---------------------------------------------------------------------------

/// What a jump off the runway does in the air, beyond holding forward and
/// holding space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extra {
    Nothing,
    /// The airdodge, forward, this many frames after takeoff.
    Airdodge(u32),
    /// The Dual mage's second jump at this many frames, her bars set to the
    /// tier that grants it.
    SecondJump(u32),
    /// The second jump, then the airborne blink a number of frames after it.
    SecondJumpBlink(u32, u32),
}

/// A trajectory: where the feet were each frame after takeoff, relative to
/// the edge's top -- `x` out past the edge, `y` up from its top.
pub type Path = Vec<V3>;

/// Run off the end of the lab's runway holding forward, press space on the
/// last frame before the edge and hold it, do `extra`, and record the feet
/// until they are 20 m down (or 400 frames have passed).
pub fn off_the_edge(class: Class, extra: Extra) -> Path {
    let mut w = lab(class);
    let (edge, top, z) = arena::lab::RUNWAY;
    let (edge, top, z) = (cm(edge), cm(top), cm(z));
    stand(&mut w, V3::new(edge.sub(m(30)), top, z));
    let tier = matches!(extra, Extra::SecondJump(_) | Extra::SecondJumpBlink(..));
    let mut path = Vec::new();
    let mut jumped_on: Option<u32> = None;
    for f in 0..900u32 {
        if tier {
            set_bars(&mut w, m(80));
        }
        let p = w.players[0];
        let step = p.vel.x.mul(crate::DT);
        if jumped_on.is_none() && p.pos.x.add(step).raw() >= edge.raw() {
            jumped_on = Some(f);
        }
        let mut bits = Input::W;
        if let Some(j) = jumped_on {
            bits |= air_buttons(extra, f - j);
        }
        tick(&mut w, ahead(bits));
        if let Some(j) = jumped_on {
            if f > j {
                let at = w.players[0].pos;
                path.push(V3::new(at.x.sub(edge), at.y.sub(top), Fx::ZERO));
                if at.y.raw() < top.sub(m(20)).raw() || f - j > 400 {
                    break;
                }
            }
        }
    }
    path
}

/// What is held in the air, `after` frames from the takeoff, for `extra`.
/// Space is held for the full hop throughout; a second jump lets go of it
/// at the end of the first's sustain and presses it again.
fn air_buttons(extra: Extra, after: u32) -> u16 {
    let sustain = crate::tuning::jump_hold_frames() as u32;
    match extra {
        Extra::Nothing => Input::SPACE,
        Extra::Airdodge(k) => Input::SPACE | if after == k { Input::SHIFT } else { 0 },
        Extra::SecondJump(k) | Extra::SecondJumpBlink(k, _) => {
            let space = if after <= sustain || (after >= k && after < k + sustain) {
                Input::SPACE
            } else {
                0
            };
            let blink = match extra {
                Extra::SecondJumpBlink(_, b) if after == k + b => Input::SHIFT,
                _ => 0,
            };
            space | blink
        }
    }
}

/// The Dual mage's two bars, both at `level` -- the tier a measurement asks
/// for, set rather than earned (see [`shadowbox`] for what earning it costs).
pub fn set_bars(w: &mut World, level: Fx) {
    if let Mechanic::Meter {
        dark,
        light,
        ascending,
        ..
    } = &mut w.players[0].mechanic
    {
        if *ascending == 0 {
            *dark = level;
            *light = level;
        }
    }
}

/// How far out a path comes back down to the height it left from: the
/// running long jump, edge to feet.
pub fn level_distance(path: &Path) -> Fx {
    let mut last = Fx::ZERO;
    for (i, at) in path.iter().enumerate() {
        if i > 2 && at.y.raw() <= 0 {
            return at.x;
        }
        last = at.x;
    }
    last
}

/// The highest a path went above the edge it left.
pub fn path_apex(path: &Path) -> Fx {
    path.iter().map(|a| a.y).fold(Fx::ZERO, Fx::max)
}

/// **Does this path land on a ledge `gap` out from the edge, its top `rise`
/// above the edge's?** Replayed through the simulation's own resolve, against
/// a bench of one solid, until the first frame it touches anything: on its
/// top is a landing, on its face is a miss. The path is the body's own; what
/// it would have done *after* touching is not asked, because the first touch
/// decides.
pub fn lands(path: &Path, gap: Fx, rise: Fx) -> bool {
    // Lifted 30 m so the bench stands on the floor whatever the rise.
    let lift = m(30);
    let ledge = Solid {
        min: V3::new(gap, Fx::ZERO, m(-50)),
        max: V3::new(gap.add(m(50)), lift.add(rise), m(50)),
        material: Material::Rock,
    };
    let bench = bench_arena(ledge);
    let mut grounded = false;
    let mut prev = V3::ZERO;
    for (i, at) in path.iter().enumerate() {
        let pos = V3::new(at.x, at.y.add(lift), Fx::ZERO);
        let vel = if i == 0 {
            V3::ZERO
        } else {
            pos.sub(prev).div_dt()
        };
        prev = pos;
        let r = bench.resolve(pos, vel, grounded);
        if r.wall {
            return false;
        }
        if r.grounded && r.pos.y.raw() > 0 {
            return true;
        }
        grounded = r.grounded;
    }
    false
}

trait PerFrame {
    fn div_dt(self) -> V3;
}

impl PerFrame for V3 {
    fn div_dt(self) -> V3 {
        let hz = Fx::from_int(crate::TICK_HZ as i32);
        V3::new(self.x.mul(hz), self.y.mul(hz), self.z.mul(hz))
    }
}

/// A one-ledge arena, for [`lands`]. Leaked: a tool asks a few thousand of
/// these and exits.
fn bench_arena(ledge: Solid) -> &'static Arena {
    let solids: &'static [Solid] = Box::leak(Box::new([ledge]));
    Box::leak(Box::new(Arena {
        id: ArenaId::LAB,
        name: "bench",
        creature: None,
        bounds: Bounds::cm((-100000, 100000), (-100000, 100000)),
        floor: Material::Ground,
        regions: &[],
        solids,
        spawns: arena::lab::ARENA.spawns,
        sites: &[],
    }))
}

/// The widest gap, in centimetres to a tenth of a metre, a set of paths lands
/// across at this rise -- or `None` if none of them lands at the smallest.
pub fn widest(paths: &[Path], rise: Fx) -> Option<Fx> {
    let tenth = cm(10);
    let mut gap = tenth;
    let mut best = None;
    // Out to 40 m, which nothing on foot crosses.
    while gap.raw() < m(40).raw() {
        if paths.iter().any(|p| lands(p, gap, rise)) {
            best = Some(gap);
        }
        gap = gap.add(tenth);
    }
    best
}

/// Every airdodge timing worth trying: one every three frames through the
/// airtime.
///
/// The plain jump is among them, so the best of the set is the best a
/// player can do with or without it.
pub fn airdodge_paths(class: Class) -> Vec<Path> {
    let mut out: Vec<Path> = (1..60u32)
        .step_by(3)
        .map(|k| off_the_edge(class, Extra::Airdodge(k)))
        .collect();
    out.push(off_the_edge(class, Extra::Nothing));
    out
}

// ---------------------------------------------------------------------------
// Up onto a ledge, with the class's own tools
// ---------------------------------------------------------------------------

/// One of the lab's ledges: its height, the middle of its row across z, and
/// its near face along x.
#[derive(Clone, Copy, Debug)]
pub struct Ledge {
    pub height: Fx,
    pub z: Fx,
    pub face: Fx,
}

pub fn ledge(i: usize) -> Ledge {
    let (z, h) = arena::lab::LEDGES[i];
    Ledge {
        height: cm(h),
        z: cm(z),
        face: cm(arena::lab::LEDGE_FACE.0),
    }
}

/// How many ledges the lab has.
pub fn ledges() -> usize {
    arena::lab::LEDGES.len()
}

/// A point on the ledge's top, `into` metres past its face.
pub fn on_top(l: Ledge, into: Fx) -> V3 {
    V3::new(l.face.add(into), l.height, l.z)
}

/// **One attempt**: fighter one of `class` standing still `back` metres from
/// the ledge's face, `prepare` done to the world, then `pilot` choosing her
/// input every frame for `frames` frames. True the first frame she is
/// standing on its top.
pub fn onto(
    class: Class,
    l: Ledge,
    back: Fx,
    frames: u32,
    prepare: impl FnOnce(&mut World),
    mut pilot: impl FnMut(u32, &World) -> Input,
) -> bool {
    let mut w = lab(class);
    stand(&mut w, V3::new(l.face.sub(back), Fx::ZERO, l.z));
    prepare(&mut w);
    for f in 0..frames {
        let input = pilot(f, &w);
        tick(&mut w, input);
        let p = &w.players[0];
        if p.grounded
            && p.pos.x.raw() > l.face.raw()
            && p.pos.y.sub(l.height).abs().raw() < cm(5).raw()
        {
            return true;
        }
    }
    false
}

/// How far back [`reach_table`] looks, in metres.
pub const fn scan_metres() -> i32 {
    30
}

/// A tool: what it is called, and a pilot for it -- `try_it(class, ledge,
/// back, setting)` -- scanned over `settings`.
pub struct Tool {
    pub class: Class,
    pub name: &'static str,
    /// What it is, in a line, and what the scan covers.
    pub note: &'static str,
    pub settings: u32,
    pub try_it: fn(Ledge, Fx, u32) -> bool,
}

/// **The furthest back she can stand from a ledge of each height and still
/// end up on it** with this tool, in half metres from half a metre out to 30:
/// `None` where nothing worked from anywhere.
pub fn reach_table(tool: &Tool) -> Vec<Option<Fx>> {
    let half = cm(50);
    let far = m(scan_metres());
    (0..ledges())
        .map(|i| {
            let l = ledge(i);
            let mut best = None;
            let mut misses = 0;
            let mut back = half;
            while back.raw() <= far.raw() && misses < 6 {
                if (0..tool.settings).any(|s| (tool.try_it)(l, back, s)) {
                    best = Some(back);
                    misses = 0;
                } else if best.is_some() {
                    misses += 1;
                }
                back = back.add(half);
            }
            best
        })
        .collect()
}

/// A run-up and a held jump toward the ledge, space pressed `back` from its
/// face after a five-metre approach, and `air` done in the air: the plain
/// jump, as a pilot, to read the other tools against.
fn running_jump(l: Ledge, back: Fx, class: Class, air: impl Fn(u32) -> u16 + Copy) -> bool {
    let mut jumped: Option<u32> = None;
    onto(
        class,
        l,
        back.add(m(5)),
        240,
        |_| {},
        move |f, w| {
            let p = &w.players[0];
            let mut bits = Input::W;
            if jumped.is_none() && p.pos.x.raw() >= l.face.sub(back).raw() {
                jumped = Some(f);
            }
            if let Some(j) = jumped {
                bits |= Input::SPACE | air(f - j);
            }
            ahead(bits)
        },
    )
}

/// Every tool there is, class by class.
pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            class: Class::ShadowReaver,
            name: "Shadow + dash",
            note: "send the shadow onto the top (right click, crosshair on it), then the \
                   dash to it (shift + forward, crosshair on the shadow); from the floor, or \
                   thrown from the top of a hop at 10, 20 or 30 frames",
            settings: 4,
            try_it: |l, back, s| reaver_shadow(l, back, s),
        },
        Tool {
            class: Class::Elementalist,
            name: "Stone jump",
            note: "one stone raised under her own feet, space held from the frame that \
                   rides the eruption (scanned 0-24), forward held once she is above the top",
            settings: 25,
            try_it: |l, back, s| stone_jump(l, back, 1, 0, s),
        },
        Tool {
            class: Class::Elementalist,
            name: "Double stone jump",
            note: "two stones under her feet 2 frames apart, space from the frame that \
                   rides both (scanned 0-24), forward once above the top",
            settings: 25,
            try_it: |l, back, s| stone_jump(l, back, 2, 2, s),
        },
        Tool {
            class: Class::Elementalist,
            name: "Updraft",
            note: "F on the spot and space held through the column (scanned 0-20 frames \
                   after), forward once she is above the top",
            settings: 7,
            try_it: |l, back, s| updraft(l, back, s * 3),
        },
        Tool {
            class: Class::Elementalist,
            name: "Stone stair",
            note: "a stone raised at the foot of the face, walked to and hopped on, then \
                   a held jump to the top: forward and space held throughout",
            settings: 1,
            try_it: |l, back, _| stone_stair(l, back),
        },
        Tool {
            class: Class::DualMage,
            name: "Second jump",
            note: "her bars held at 80 (the second tier, set rather than earned): a \
                   running jump and the second one pressed at 10-40 frames",
            settings: 7,
            try_it: |l, back, s| {
                running_jump_with(l, back, Class::DualMage, true, move |after| {
                    let k = 10 + s * 5;
                    if after + 1 == k {
                        0
                    } else if after >= k && after < k + 20 {
                        Input::SPACE
                    } else if after > 22 && after < k {
                        0
                    } else {
                        Input::SPACE
                    }
                })
            },
        },
        Tool {
            class: Class::Champion,
            name: "Rush + pole vault",
            note: "Rush toward the ledge (E) and the spear planted in the floor (right \
                   click, an eighth of a turn down) 1 to 15 frames into it",
            settings: 8,
            try_it: |l, back, s| pole_vault(l, back, 1 + s * 2),
        },
        Tool {
            class: Class::Champion,
            name: "Takeoff",
            note: "a running jump with a weapon pressed the frame after the feet leave \
                   (the takeoffs: sword, hammer, spear)",
            settings: 3,
            try_it: |l, back, s| {
                let button = [Input::LEFT, Input::MIDDLE, Input::RIGHT][s as usize];
                running_jump(l, back, Class::Champion, move |after| {
                    if after == 1 || after == 2 { button } else { 0 }
                })
            },
        },
        Tool {
            class: Class::BloodMage,
            name: "Grasp haul",
            note: "Q held 0-30 frames with the crosshair a metre onto the top, from the \
                   floor or thrown from a held jump; arms that meet only scenery haul her \
                   to them (the `Grasp hauls` flag, on)",
            settings: 14,
            try_it: |l, back, s| grasp_haul(l, back, (s % 7) * 5, s >= 7),
        },
    ]
}

/// [`running_jump`], optionally with the Dual mage's bars held at the
/// second tier every frame.
fn running_jump_with(
    l: Ledge,
    back: Fx,
    class: Class,
    tier: bool,
    space: impl Fn(u32) -> u16 + Copy,
) -> bool {
    let mut jumped: Option<u32> = None;
    let mut w = lab(class);
    stand(&mut w, V3::new(l.face.sub(back.add(m(5))), Fx::ZERO, l.z));
    for f in 0..260u32 {
        if tier {
            set_bars(&mut w, m(80));
        }
        let p = &w.players[0];
        let mut bits = Input::W;
        if jumped.is_none() && p.pos.x.raw() >= l.face.sub(back).raw() {
            jumped = Some(f);
        }
        if let Some(j) = jumped {
            bits |= space(f - j);
        }
        tick(&mut w, ahead(bits));
        let p = &w.players[0];
        if p.grounded
            && p.pos.x.raw() > l.face.raw()
            && p.pos.y.sub(l.height).abs().raw() < cm(5).raw()
        {
            return true;
        }
    }
    false
}

/// The Reaver: the shadow onto the top, and the dash to it. `setting` 0 is
/// from the floor; 1 to 3 throw it from a hop, 10, 20 or 30 frames up.
fn reaver_shadow(l: Ledge, back: Fx, setting: u32) -> bool {
    let target = on_top(l, m(1));
    let throw_at = if setting == 0 { 0 } else { setting * 10 };
    let mut sent = false;
    let mut dashed = false;
    onto(
        Class::ShadowReaver,
        l,
        back,
        200,
        |_| {},
        move |f, w| {
            let p = &w.players[0];
            let shadow = crate::shadow::of(p);
            let mut bits = if setting > 0 && f < throw_at + 2 {
                Input::SPACE
            } else {
                0
            };
            if !sent && f >= throw_at {
                if f > throw_at {
                    sent = true;
                }
                return toward(w, bits | Input::RIGHT, target);
            }
            if let Some(s) = shadow {
                if matches!(s.doing, Ghost::Waiting) && !dashed {
                    if matches!(p.action, Action::Dodge { .. }) {
                        dashed = true;
                    }
                    bits |= Input::SHIFT | Input::W;
                    return toward(w, bits, s.pos);
                }
            }
            toward(w, bits, target)
        },
    )
}

/// The Elementalist's structure jump: `count` stones under her own feet,
/// `gap` frames apart, space held from `jump_at`, forward once she is above
/// the top.
fn stone_jump(l: Ledge, back: Fx, count: u32, gap: u32, jump_at: u32) -> bool {
    let mut raised = 0u32;
    let mut feet = None;
    onto(
        Class::Elementalist,
        l,
        back,
        300,
        |_| {},
        move |f, w| {
            let p = &w.players[0];
            let here = *feet.get_or_insert(V3::new(p.pos.x, Fx::ZERO, p.pos.z));
            let mut bits = 0;
            if raised < count && f == raised * gap {
                bits |= Input::MECHANIC;
                raised += 1;
            }
            if f >= jump_at {
                bits |= Input::SPACE;
            }
            if p.pos.y.raw() > l.height.add(m(1)).raw() {
                bits |= Input::W;
                return ahead(bits);
            }
            if bits & Input::MECHANIC != 0 {
                return toward(w, bits, here);
            }
            ahead(bits)
        },
    )
}

/// The Updraft: F on the spot, space from `jump_at`, forward once above.
fn updraft(l: Ledge, back: Fx, jump_at: u32) -> bool {
    onto(
        Class::Elementalist,
        l,
        back,
        300,
        |_| {},
        move |f, w| {
            let p = &w.players[0];
            let mut bits = if f < 2 { Input::KEY_F } else { 0 };
            if f >= jump_at {
                bits |= Input::SPACE;
            }
            if p.pos.y.raw() > l.height.add(m(1)).raw() {
                bits |= Input::W;
            }
            ahead(bits)
        },
    )
}

/// A stone at the foot of the face, then forward and space held: up onto the
/// stone, and up off it.
fn stone_stair(l: Ledge, back: Fx) -> bool {
    let foot = V3::new(l.face.sub(m(1)), Fx::ZERO, l.z);
    onto(
        Class::Elementalist,
        l,
        back,
        400,
        |_| {},
        move |f, w| {
            if f < 2 {
                return toward(w, Input::MECHANIC, foot);
            }
            // Let it stand fully out of the floor before walking at it.
            if f < 2 + crate::tuning::structure_rise() as u32 + 4 {
                return ahead(0);
            }
            ahead(Input::W | Input::SPACE)
        },
    )
}

/// The Champion: Rush at the ledge and plant the spear `vault_at` frames in.
fn pole_vault(l: Ledge, back: Fx, vault_at: u32) -> bool {
    let down = -(1 << 13);
    onto(
        Class::Champion,
        l,
        back,
        200,
        |_| {},
        move |f, _| {
            let bits = if f == 0 {
                Input::MECHANIC
            } else if f >= vault_at && f < vault_at + 2 {
                Input::RIGHT | Input::W
            } else {
                Input::W
            };
            Input::looking_at(bits, 0, down)
        },
    )
}

/// The Blood mage: Q held `hold` frames with the crosshair a metre onto the
/// top, from the floor or from a held jump (`hop`), then let go and let the
/// arms do it.
fn grasp_haul(l: Ledge, back: Fx, hold: u32, hop: bool) -> bool {
    let top = on_top(l, m(1));
    onto(
        Class::BloodMage,
        l,
        back,
        260,
        |_| {},
        move |f, w| {
            let mut bits = if (2..=2 + hold).contains(&f) {
                Input::SPECIAL
            } else {
                0
            };
            if hop && f < 40 {
                bits |= Input::SPACE;
            }
            if f > hold + 80 {
                return ahead(Input::W);
            }
            toward(w, bits, top)
        },
    )
}

// ---------------------------------------------------------------------------
// The tools on the flat: single numbers
// ---------------------------------------------------------------------------

/// The Reaver's chain on the flat: how far the shadow goes when sent at
/// nothing (the range, along the floor), and how far the dash jump out of
/// the dash to it carries past it -- and how high.
pub fn reaver_chain() -> (Fx, Fx, Fx) {
    let mut w = lab(Class::ShadowReaver);
    let (x, z) = arena::lab::LANE;
    let start = V3::new(cm(x), Fx::ZERO, cm(z));
    stand(&mut w, start);
    // Level: the ray meets the range sphere, and max range on the ground in
    // the mouse's direction is where it goes.
    for f in 0..20u32 {
        let bits = if f < 2 { Input::RIGHT } else { 0 };
        tick(&mut w, ahead(bits));
    }
    let shadow = crate::shadow::of(&w.players[0]).map_or(start, |s| s.pos);
    let range = shadow.sub(start).flat_len();
    let mut carried = false;
    let mut from = shadow;
    let mut apex = Fx::ZERO;
    for _ in 0..240u32 {
        let p = w.players[0];
        let carrying = crate::shadow::carrying_a_dash(&p);
        let bits = if carrying {
            if !carried {
                from = p.pos;
            }
            carried = true;
            Input::SPACE | Input::W
        } else if carried {
            Input::SPACE | Input::W
        } else {
            Input::SHIFT | Input::W
        };
        let look = toward(&w, bits, shadow);
        tick(&mut w, if carried { ahead(bits) } else { look });
        let p = &w.players[0];
        if carried {
            apex = apex.max(p.pos.y);
            if p.grounded && p.pos.sub(from).flat_len().raw() > m(1).raw() {
                break;
            }
        }
    }
    let jump = w.players[0].pos.sub(from).flat_len();
    (range, jump, apex)
}

/// The Elementalist's structure jumps straight up, on the flat: the best
/// apex over every jump timing, for one stone and for two (at the best gap,
/// 0 to 6 frames), and the Updraft's.
pub fn stone_apexes() -> (Fx, Fx, Fx, Fx) {
    let up = |count: u32, gap: u32, jump_at: u32, f_key: bool| -> Fx {
        let mut w = lab(Class::Elementalist);
        let (x, z) = arena::lab::LANE;
        let here = V3::new(cm(x), Fx::ZERO, cm(z));
        stand(&mut w, here);
        let mut raised = 0;
        let mut apex = Fx::ZERO;
        for f in 0..240u32 {
            let mut bits = 0;
            if !f_key && raised < count && f == raised * gap {
                bits |= Input::MECHANIC;
                raised += 1;
            }
            if f_key && f < 2 {
                bits |= Input::KEY_F;
            }
            if f >= jump_at {
                bits |= Input::SPACE;
            }
            let input = toward(&w, bits, here);
            tick(&mut w, input);
            apex = apex.max(w.players[0].pos.y);
        }
        apex
    };
    let single = (0..30).map(|j| up(1, 0, j, false)).fold(Fx::ZERO, Fx::max);
    let double = (0..7u32)
        .flat_map(|g| (0..30).map(move |j| (g, j)))
        .map(|(g, j)| up(2, g, j, false))
        .fold(Fx::ZERO, Fx::max);
    let draft = (0..30).map(|j| up(0, 0, j, true)).fold(Fx::ZERO, Fx::max);
    let stone = crate::tuning::structure_height();
    (single, double, draft, stone)
}

/// The Dual mage, earning her tiers with nothing to hit: autos alternated on
/// the spot, each pressed as soon as she is free. The frames to the lower
/// bar reaching each of 50, 75 and 95 -- `None` if it never does in 30 s.
pub fn shadowbox() -> [Option<u32>; 3] {
    let mut w = lab(Class::DualMage);
    let (x, z) = arena::lab::LANE;
    stand(&mut w, V3::new(cm(x), Fx::ZERO, cm(z)));
    let marks = [
        crate::dual::Tier::Blink,
        crate::dual::Tier::Jump,
        crate::dual::Tier::Wings,
    ];
    let mut out = [None; 3];
    let mut left = true;
    for f in 0..1800u32 {
        let p = &w.players[0];
        let mut bits = 0;
        if p.action.actionable() {
            bits = if left { Input::LEFT } else { Input::RIGHT };
            left = !left;
        }
        tick(&mut w, ahead(bits));
        let tier = crate::dual::tier(&w.players[0]);
        for (i, t) in marks.iter().enumerate() {
            if out[i].is_none() && (tier >= *t || crate::dual::ascending(&w.players[0])) {
                out[i] = Some(f);
            }
        }
        if out[2].is_some() {
            break;
        }
    }
    out
}

/// The Dual mage ascending: bars set to the top, so the ascension starts, and
/// space pressed every `beat` frames with forward held. The highest she gets
/// and how far she goes before her feet are down again.
pub fn wings(beat: u32) -> (Fx, Fx) {
    let mut w = lab(Class::DualMage);
    let (x, z) = arena::lab::LANE;
    let start = V3::new(cm(x), Fx::ZERO, cm(z));
    stand(&mut w, start);
    set_bars(&mut w, Fx::from_int(crate::tuning::meter_max()));
    let mut apex = Fx::ZERO;
    for f in 0..900u32 {
        let bits = Input::W | if f % beat < 2 { Input::SPACE } else { 0 };
        tick(&mut w, ahead(bits));
        let p = &w.players[0];
        apex = apex.max(p.pos.y);
        if f > 10 && p.grounded {
            break;
        }
    }
    (apex, w.players[0].pos.sub(start).flat_len())
}

/// The Champion's Rush on the flat: how far one charge carries.
pub fn rush() -> Fx {
    let mut w = lab(Class::Champion);
    let (x, z) = arena::lab::LANE;
    let start = V3::new(cm(x), Fx::ZERO, cm(z));
    stand(&mut w, start);
    for f in 0..60u32 {
        let bits = if f == 0 { Input::MECHANIC } else { 0 };
        tick(&mut w, ahead(bits));
    }
    w.players[0].pos.sub(start).flat_len()
}

/// The Champion's pole vault on the flat, at its best timing: apex and how
/// far it carries.
pub fn vault() -> (Fx, Fx) {
    let down = -(1 << 13);
    let mut best = (Fx::ZERO, Fx::ZERO);
    for at in 1..16u32 {
        let mut w = lab(Class::Champion);
        let (x, z) = arena::lab::LANE;
        let start = V3::new(cm(x), Fx::ZERO, cm(z));
        stand(&mut w, start);
        let mut apex = Fx::ZERO;
        let mut left = false;
        for f in 0..200u32 {
            let bits = if f == 0 {
                Input::MECHANIC
            } else if f >= at && f < at + 2 {
                Input::RIGHT | Input::W
            } else {
                Input::W
            };
            tick(&mut w, Input::looking_at(bits, 0, down));
            let p = &w.players[0];
            apex = apex.max(p.pos.y);
            left |= !p.grounded;
            if left && p.grounded {
                break;
            }
        }
        if apex.raw() > best.0.raw() {
            best = (apex, w.players[0].pos.sub(start).flat_len());
        }
    }
    best
}

/// The flat distance a Blood mage blink would cover, if the flag that makes
/// her dodge a blink were on (it is off), and whether it is.
pub fn blood_blink() -> (bool, Fx) {
    (crate::tuning::dodge_blinks(), crate::tuning::blink_range())
}

/// The Dual mage's blink: how far, standing and airborne.
pub fn dual_blink() -> (Fx, Fx) {
    (
        crate::dual::dodge_travel(true),
        crate::dual::dodge_travel(false),
    )
}
