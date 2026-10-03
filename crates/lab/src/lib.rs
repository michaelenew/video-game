//! The topology lab: change a creature's body at runtime and measure what
//! breaks. `docs/design/exploration/0005_body_plans.md` is the question and the
//! write-up; this is the instrument.
//!
//! A [`Body`] is an owned, editable copy of a species' table -- bones, parts,
//! legs, moves, clips, baked rows, tuning, fight declaration. Edit it, then
//! [`Body::install`] it: `sim::species::lookup` answers with it from then on
//! (the `lab` feature), so every hunt, rig, sheet and report reads the new
//! body through the same code the game does. [`restore`] puts the compiled
//! species back.
//!
//! Floats are fine here: this is a tool, not the simulation.

use std::sync::RwLock;

use sim::beast::{self, Bone, ClipDecl, Leg, MAX_BREAKABLE, Part, Pose, Rig};
use sim::monster::{Doing, Monster};
use sim::oven::{self, MonsterField};
use sim::species::{self, FightDecl, MoveDecl, Species, SpeciesId, Stock};
use sim::{Fx, V3};

pub mod measure;
pub mod roles;

pub fn leak<T>(v: T) -> &'static T {
    Box::leak(Box::new(v))
}

pub fn slice<T>(v: Vec<T>) -> &'static [T] {
    Box::leak(v.into_boxed_slice())
}

pub fn f(x: Fx) -> f32 {
    x.to_f32_for_render()
}

pub fn fx(v: f32) -> Fx {
    Fx::from_raw((v * 65536.0).round() as i32)
}

pub fn v3(x: f32, y: f32, z: f32) -> V3 {
    V3::new(fx(x), fx(y), fx(z))
}

/// Fields in one creature move's row of knobs.
pub fn fields() -> usize {
    oven::MONSTER_FIELDS
}

// ---------------------------------------------------------------------------
// Baked rows, per species id, behind a plain `fn` the species table can hold
// ---------------------------------------------------------------------------

type Table = &'static [&'static [i32]];

static TABLES: RwLock<[Option<Table>; species::COUNT]> = RwLock::new([None; species::COUNT]);

fn table_row(id: usize, r: usize) -> &'static [i32] {
    let t = TABLES.read().unwrap()[id].expect("no lab table installed for this id");
    t[r.min(t.len() - 1)]
}

macro_rules! rows_for {
    ($($n:literal => $name:ident),*) => {
        $(fn $name(r: usize) -> &'static [i32] { table_row($n, r) })*
        fn row_fn(id: SpeciesId) -> fn(usize) -> &'static [i32] {
            match id.0 { $($n => $name,)* _ => panic!("species id out of range") }
        }
    };
}

rows_for!(0 => r0, 1 => r1, 2 => r2, 3 => r3, 4 => r4, 5 => r5, 6 => r6,
          7 => r7, 8 => r8, 9 => r9, 10 => r10, 11 => r11, 12 => r12);

// ---------------------------------------------------------------------------
// A body
// ---------------------------------------------------------------------------

/// An editable copy of a species.
#[derive(Clone)]
pub struct Body {
    pub id: SpeciesId,
    pub name: &'static str,
    pub bones: Vec<Bone>,
    pub mirror: Vec<(usize, usize)>,
    pub neck: Vec<usize>,
    pub follows: Vec<usize>,
    pub parts: Vec<Part>,
    pub legs: Vec<Leg>,
    pub moves: Vec<MoveDecl>,
    pub clips: Vec<ClipDecl>,
    pub stock: Stock,
    pub span: Vec<(u16, u16)>,
    pub rows: Vec<&'static [i32]>,
    /// The Oven's store for it, in its own layout: common, own, a row per
    /// move, then whatever follows (pack, senses, hazards, objectives).
    pub tuned: Vec<i32>,
    pub own: &'static [oven::KnobDecl],
    pub tuned_path: &'static str,
    pub pack: Option<&'static sim::pack::PackDecl>,
    pub fight: FightDecl,
}

impl Body {
    /// A copy of a species as compiled, with its live tuning as the store.
    pub fn of(s: &'static Species) -> Body {
        let n = oven::species_knobs(s).len();
        let tuned: Vec<i32> = (0..n).map(|i| oven::species_raw(s.id, i)).collect();
        Body {
            id: s.id,
            name: s.name,
            bones: s.bones.to_vec(),
            mirror: s.mirror.to_vec(),
            neck: s.neck.to_vec(),
            follows: s.follows.to_vec(),
            parts: s.parts.to_vec(),
            legs: s.legs.to_vec(),
            moves: s.moves.to_vec(),
            clips: s.clips.to_vec(),
            stock: s.stock,
            span: s.span.to_vec(),
            rows: (0..s.rows).map(|r| (s.row)(r)).collect(),
            tuned,
            own: s.own,
            tuned_path: s.tuned_path,
            pack: s.pack,
            fight: *s.fight,
        }
    }

    /// Where move rows end in the store.
    fn moves_end(&self) -> usize {
        species::Common::ALL.len() + self.own.len() + self.moves.len() * fields()
    }

    /// The store index of one field of one move.
    pub fn move_index(&self, slot: usize, field: MonsterField) -> usize {
        species::Common::ALL.len() + self.own.len() + slot * fields() + field as usize
    }

    pub fn move_row(&self, slot: usize) -> Vec<i32> {
        let at = self.move_index(slot, MonsterField::ALL[0]);
        self.tuned[at..at + fields()].to_vec()
    }

    pub fn common(&self, c: species::Common) -> i32 {
        self.tuned[c as usize]
    }

    pub fn set_common(&mut self, c: species::Common, raw: i32) {
        self.tuned[c as usize] = raw;
    }

    /// Append a move: its declaration, its clip (added to this body's clips,
    /// with its rows), and its row of knobs -- inserted after the last move so
    /// that whatever the store holds after the moves keeps its values.
    pub fn add_move(
        &mut self,
        decl: MoveDecl,
        clip: ClipDecl,
        rows: Vec<&'static [i32]>,
        knobs: Vec<i32>,
    ) {
        let first = self.rows.len() as u16;
        let count = rows.len() as u16;
        self.rows.extend(rows);
        self.clips.push(clip);
        self.span.push((first, count));
        let clip_index = self.clips.len() - 1;
        let at = self.moves_end();
        self.tuned.splice(at..at, knobs);
        self.moves.push(MoveDecl {
            clip: clip_index,
            ..decl
        });
    }

    /// The species table this body is, with its rows behind the lab's table.
    pub fn build(&self) -> &'static Species {
        let rows: Table = slice(self.rows.clone());
        TABLES.write().unwrap()[self.id.0 as usize] = Some(rows);
        let parts = slice(self.parts.clone());
        leak(Species {
            id: self.id,
            name: self.name,
            bones: slice(self.bones.clone()),
            mirror: slice(self.mirror.clone()),
            neck: slice(self.neck.clone()),
            follows: slice(self.follows.clone()),
            parts,
            breakable: breakables(parts),
            legs: slice(self.legs.clone()),
            moves: slice(self.moves.clone()),
            clips: slice(self.clips.clone()),
            stock: self.stock,
            span: slice(self.span.clone()),
            rows: self.rows.len(),
            row: row_fn(self.id),
            own: self.own,
            tuned: slice(self.tuned.clone()),
            tuned_path: self.tuned_path,
            pack: self.pack,
            fight: leak(self.fight),
        })
    }

    /// Build it and make it the species for its id, with the Oven reloaded
    /// from its store.
    pub fn install(&self) -> &'static Species {
        let s = self.build();
        species::lab::install(s);
        oven::reset_to_baked();
        s
    }
}

/// Every species back as compiled.
pub fn restore() {
    species::lab::clear();
    oven::reset_to_baked();
}

pub fn breakables(parts: &[Part]) -> ([u8; MAX_BREAKABLE], usize) {
    let mut out = [u8::MAX; MAX_BREAKABLE];
    let mut n = 0;
    for (i, p) in parts.iter().enumerate() {
        if p.shape.breakable && n < MAX_BREAKABLE {
            out[n] = i as u8;
            n += 1;
        }
    }
    (out, n)
}

// ---------------------------------------------------------------------------
// Proportions
// ---------------------------------------------------------------------------

/// Children of a bone, by index.
pub fn children(bones: &[Bone], b: usize) -> Vec<usize> {
    (0..bones.len()).filter(|c| bones[*c].parent == b).collect()
}

fn dominant(v: V3) -> usize {
    let a = [f(v.x).abs(), f(v.y).abs(), f(v.z).abs()];
    if a[0] >= a[1] && a[0] >= a[2] {
        0
    } else if a[1] >= a[2] {
        1
    } else {
        2
    }
}

fn scale_axis(v: V3, axis: usize, k: f32) -> V3 {
    let mut a = [v.x, v.y, v.z];
    a[axis] = fx(f(a[axis]) * k);
    V3::new(a[0], a[1], a[2])
}

/// **Make bone `b`'s segment longer by `k`**: every child sits `k` times as
/// far out along the axes asked for, and every box hanging off `b` stretches
/// with it along the axis its children lie on (its own rest axis for a bone
/// with no children -- a head, a tail tip, a shin with a foot box).
pub fn stretch(body: &mut Body, b: usize, k: f32, axes: [bool; 3]) {
    let kids = children(&body.bones, b);
    let along = match kids.first() {
        Some(c) => dominant(body.bones[*c].rest),
        None => dominant(body.bones[b].rest),
    };
    for c in &kids {
        for (axis, on) in axes.iter().enumerate() {
            if *on {
                body.bones[*c].rest = scale_axis(body.bones[*c].rest, axis, k);
            }
        }
    }
    if axes[along] {
        for p in body.parts.iter_mut().filter(|p| p.shape.bone == b) {
            p.shape.min = scale_axis(p.shape.min, along, k);
            p.shape.max = scale_axis(p.shape.max, along, k);
        }
    }
}

/// The lowest sole in the standing pose, in metres at scale one.
pub fn standing_sole(s: &'static Species) -> f32 {
    let pose = measure::standing(s, 0.0).pose();
    measure::lowest_feet(s, &pose) / f(s.scale())
}

/// Raise or lower the hips so the standing pose's lowest sole is where it was
/// on `before`: what longer legs need if the animal is not to stand in the
/// floor.
pub fn keep_feet_on_floor(body: &mut Body, before: f32) {
    let now = standing_sole(body.build());
    let dy = before - now;
    let root = body
        .bones
        .iter()
        .position(|b| b.parent == beast::NO_PARENT)
        .unwrap();
    let r = body.bones[root].rest;
    body.bones[root].rest = V3::new(r.x, fx(f(r.y) + dy), r.z);
}

/// Put a monster of this species into a move, on the frame its hit first comes
/// out.
pub fn at_contact(id: SpeciesId, kind: u8) -> Monster {
    let mut m = Monster::new(id);
    let a = m.sp().attack(kind);
    m.doing = Doing::Active {
        kind,
        left: a.active.max(1),
    };
    m
}

pub fn rest_pose(s: &Species) -> Pose {
    Pose::rest(s.bones.len())
}

pub fn rig_of(s: &'static Species, pose: &Pose) -> Rig {
    Rig::build(s, V3::ZERO, Fx::ZERO, pose)
}
