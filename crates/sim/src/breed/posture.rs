//! Posture: how a plan *stands*. Repair makes a plan legal; this makes it
//! balance, with every foot on the ground, and look like an animal.
//!
//! One principle does the work: **a body at rest is balanced when its
//! centre of mass sits over its feet.** The parts have masses (a box's
//! volume times a density the mass slider sets), the grounded limbs have
//! feet, and when the mass is not over the feet the body has a handful of
//! ways to put it there -- the ones a real animal uses:
//!
//! - **shift** the limbs along the body (bring the hind limbs forward
//!   under the weight), keeping a natural spacing between pairs;
//! - **reach**: lean the limbs so the feet land ahead of or behind the
//!   hips;
//! - **pitch** the torso, which moves the front mass back over the hind
//!   feet (or the reverse);
//! - put the **wings down** and walk on them, a pterosaur, when the weight
//!   is ahead of the only legs;
//! - **drag**: with nothing behind the fore limbs, the rear rests on the
//!   ground and lengthens into a tail that trails.
//!
//! Which mix a creature uses is rolled once from its genome's fingerprint,
//! so the same creature always stands the same way and two creatures with
//! the same problem may solve it differently. The continuous ones are
//! applied together, each a random share of the correction, capped by
//! what the body allows; the binary ones are rolled where they apply.
//!
//! Then the **vertical**: a root height and a pitch at which every
//! grounded limb reaches the floor with a bend between a crouch and nearly
//! straight -- a longer hind pair bends its knees more, which is a hopper,
//! and the tendency is recorded for the move set. A head is carried above
//! the back unless the head kind says low. Everything here is a pure
//! function of the genome; nothing is kept.

use super::build::{Proportions, proportions};
use super::{Build, Genome, HeadKind, Rng, Role, STATIONS, Stance, Station, fx};
use crate::fixed::{Fx, cos_turns, sin_turns};

/// How one limb pair stands.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct StationPose {
    /// Where it attaches along the torso's axis, metres from the root
    /// (forward positive). Starts at the station's natural place.
    pub attach: Fx,
    /// How far ahead of the hip the foot lands, metres (negative is
    /// behind). Zero stands straight under.
    pub reach: Fx,
    /// Hip to foot, as a share of the limb's standing length: near one is
    /// straight, a half is a crouch. Zero for a limb that does not stand.
    pub extension: Fx,
    /// It carries weight on the ground.
    pub grounded: bool,
    /// A wing that walks on its wrist.
    pub wing_walks: bool,
    /// It could not reach the ground at any height the rest allowed: it
    /// hangs. Never true of an archetype.
    pub dangles: bool,
}

/// What the balance used, for the tests and the sheet.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Applied {
    pub shift: Fx,
    pub reach: Fx,
    pub pitch: Fx,
    pub wing_walks: bool,
    pub drags: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Posture {
    /// The torso's pitch in turns, nose up positive, after balance.
    pub pitch: Fx,
    /// The root's height off the floor, metres.
    pub root: Fx,
    pub stations: [StationPose; STATIONS],
    /// The rear rests on the ground and trails.
    pub drags: bool,
    /// How much longer the rear is for it, metres.
    pub drag_len: Fx,
    /// Hind limbs longer than fore on a level body: a hopper, nought to
    /// one.
    pub hop: Fx,
    /// Turns the neck is raised above the torso's line.
    pub neck_rise: Fx,
    /// Where the mass sits along the body, world x, metres from the root.
    pub com: Fx,
    /// The feet's span, world x: the support.
    pub support: (Fx, Fx),
    pub balanced: bool,
    pub applied: Applied,
}

// ---------------------------------------------------------------------------
// Masses
// ---------------------------------------------------------------------------

/// Limbs attach this far down the flank, as a share of the torso's
/// half-height: at the bottom of the body, where every creature in the
/// tables has them.
pub const ATTACH_DOWN: Fx = fx(3, 4);

/// The band of the support the mass must sit inside: a fifth in from each
/// end.
const BAND: Fx = fx(1, 5);

/// A part with mass, at a point along the torso's axis (x) and above it (y).
#[derive(Clone, Copy)]
struct Mass {
    x: Fx,
    y: Fx,
    m: Fx,
}

const MAX_MASSES: usize = 24;

struct Masses {
    items: [Mass; MAX_MASSES],
    count: usize,
}

impl Masses {
    fn push(&mut self, x: Fx, y: Fx, m: Fx) {
        if self.count < MAX_MASSES {
            self.items[self.count] = Mass { x, y, m };
            self.count += 1;
        }
    }

    /// The centre of mass's world x, with the axis pitched by `pitch`.
    fn com(&self, pitch: Fx) -> Fx {
        let (c, s) = (cos_turns(pitch), sin_turns(pitch));
        let mut sum = Fx::ZERO;
        let mut total = Fx::ZERO;
        for it in &self.items[..self.count] {
            // A point (x along the axis, y above it) in the world after the
            // pitch: x cos - y sin.
            let wx = it.x.mul(c).sub(it.y.mul(s));
            sum = sum.add(wx.mul(it.m));
            total = total.add(it.m);
        }
        if total == Fx::ZERO {
            Fx::ZERO
        } else {
            sum.div(total)
        }
    }
}

fn masses(g: &Genome, p: &Proportions) -> Masses {
    let mut out = Masses {
        items: [Mass {
            x: Fx::ZERO,
            y: Fx::ZERO,
            m: Fx::ZERO,
        }; MAX_MASSES],
        count: 0,
    };
    let rho = fx(1, 2).add(g.sliders.mass);
    let t = p.torso;
    let third = t.mul(fx(1, 3));
    let slab = third
        .mul(p.half_h)
        .mul(p.half_w)
        .mul(Fx::from_int(4))
        .mul(rho);
    // The torso: three boxes from the root forward.
    out.push(Fx::ZERO, Fx::ZERO, slab);
    out.push(third, Fx::ZERO, slab);
    out.push(third.mul(Fx::from_int(2)), Fx::ZERO, slab);
    if p.thorax {
        out.push(t, p.half_h.mul(fx(1, 2)), slab.mul(fx(3, 2)));
    }
    if p.abdomen {
        out.push(
            t.mul(fx(-3, 2)),
            p.half_h.mul(fx(1, 2)),
            // Big, but hollow: a spider's abdomen is lighter than its size.
            slab.mul(Fx::from_int(2)),
        );
    }
    if p.shell {
        out.push(third, p.half_h, slab.mul(Fx::from_int(2)));
    }
    // The head, out on its neck.
    let hh = p.head_half;
    let head_m = hh.mul(hh).mul(hh).mul(Fx::from_int(6)).mul(rho);
    out.push(
        p.front.add(p.neck_len).add(hh),
        p.neck_len.mul(fx(1, 4)),
        head_m,
    );
    out.push(
        p.front.add(p.neck_len.mul(fx(1, 2))),
        p.neck_len.mul(fx(1, 8)),
        head_m.mul(fx(1, 3)),
    );
    // The tail, trailing level.
    if p.tail_len > Fx::ZERO {
        let th = p.half_h.mul(fx(1, 2));
        let seg = p
            .tail_len
            .mul(fx(1, 3))
            .mul(th)
            .mul(th)
            .mul(Fx::from_int(4))
            .mul(rho);
        for k in 1..=3 {
            out.push(p.tail_len.neg().mul(Fx::ratio(2 * k - 1, 6)), Fx::ZERO, seg);
        }
    }
    // The limbs, hanging from where they attach; wings are light and
    // swept back.
    for (st, pair) in g.pairs() {
        let i = st.index();
        let l = p.limb[i];
        let girth = p.girth[i];
        if pair.role == Role::Wing {
            let chord = t.mul(fx(2, 5));
            let m = l
                .mul(chord)
                .mul(girth)
                .mul(Fx::from_int(2))
                .mul(rho)
                .mul(fx(3, 10));
            out.push(p.attach[i].sub(l.mul(fx(3, 20))), p.half_h, m);
        } else {
            let m = l.mul(girth).mul(girth).mul(Fx::from_int(8)).mul(rho);
            out.push(p.attach[i], p.half_h.neg().mul(ATTACH_DOWN), m);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The pass
// ---------------------------------------------------------------------------

/// A limb is **splayed** -- out to the side, then down -- when it is a
/// spindle or a stub, or on any low-slung body (the Siegeshell's pillars
/// go out eleven metres before they come down).
pub fn splayed(stance: Stance, build: Build) -> bool {
    stance == Stance::Lowslung || matches!(build, Build::Spindle | Build::Stub)
}

/// A limb's standing length: a column or a sprung leg stands on its whole
/// length, a splayed one on its lower half, a wing walking on its first
/// two segments.
pub fn standing_length(stance: Stance, build: Build, wing_walks: bool, l: Fx) -> Fx {
    if wing_walks {
        return l.mul(fx(2, 3));
    }
    if splayed(stance, build) {
        l.mul(fx(1, 2))
    } else {
        l
    }
}

/// How straight a build likes to stand: hip to foot as a share of its
/// standing length. A spindle stands deep in its splay.
fn natural_extension(stance: Stance, build: Build, wing_walks: bool) -> Fx {
    if wing_walks {
        return fx(4, 5);
    }
    match build {
        Build::Column => {
            if splayed(stance, build) {
                fx(4, 5)
            } else {
                fx(19, 20)
            }
        }
        Build::Sprung => fx(4, 5),
        Build::Raptorial => fx(7, 10),
        Build::Spindle => fx(1, 2),
        Build::Stub => fx(7, 10),
        Build::Wing => fx(4, 5),
    }
}

/// The most the balance pitches the torso either way, in turns: a level
/// or upright body a good deal, a low-slung one hardly at all.
fn pitch_cap(stance: Stance) -> Fx {
    match stance {
        Stance::Lowslung => fx(1, 50),
        _ => fx(3, 25),
    }
}

/// The most a limb reaches ahead or behind its hip, as a share of its
/// standing length.
const REACH_CAP: Fx = fx(2, 5);
/// The least a limb bends and the most, as hip-to-foot over standing
/// length.
const EXT_MAX: Fx = fx(97, 100);
const EXT_MIN: Fx = fx(7, 20);

struct Solver<'a> {
    g: &'a Genome,
    p: Proportions,
    masses: Masses,
    rng: Rng,
    pose: Posture,
}

impl Solver<'_> {
    fn grounded(&self) -> impl Iterator<Item = (usize, &StationPose)> {
        self.pose
            .stations
            .iter()
            .enumerate()
            .filter(|(_, s)| s.grounded)
    }

    /// A foot's world x: the hip's, plus the reach.
    fn foot_x(&self, i: usize) -> Fx {
        let s = &self.pose.stations[i];
        s.attach.mul(cos_turns(self.pose.pitch)).add(s.reach)
    }

    /// The feet's span, with a foot's own length either end, and the rear
    /// contact if it drags.
    fn support(&self) -> (Fx, Fx) {
        let mut lo = Fx::from_int(1_000);
        let mut hi = Fx::from_int(-1_000);
        for (i, _) in self.grounded() {
            let fl = self.p.limb[i].mul(fx(3, 20));
            let x = self.foot_x(i);
            lo = lo.min(x.sub(fl));
            hi = hi.max(x.add(fl));
        }
        if self.pose.drags {
            let rear = self.p.rear.sub(self.pose.drag_len);
            lo = lo.min(rear.mul(cos_turns(self.pose.pitch)));
        }
        if lo > hi {
            (Fx::ZERO, Fx::ZERO)
        } else {
            (lo, hi)
        }
    }

    /// How far the mass is outside the support's inner band: positive is
    /// ahead of it, zero is balanced.
    fn error(&self) -> Fx {
        let com = self.masses.com(self.pose.pitch);
        let (lo, hi) = self.support();
        let band = hi.sub(lo).mul(BAND);
        let (lo, hi) = (lo.add(band), hi.sub(band));
        // Within a hundredth of the torso is balanced: the arithmetic is
        // fixed point and the feet are not points.
        let slack = self.p.torso.mul(fx(1, 100));
        if com > hi.add(slack) {
            com.sub(hi)
        } else if com < lo.sub(slack) {
            com.sub(lo)
        } else {
            Fx::ZERO
        }
    }

    /// The least space between two neighbouring pairs along the body.
    fn spacing(&self, a: usize, b: usize) -> Fx {
        self.p.girth[a]
            .add(self.p.girth[b])
            .mul(Fx::from_int(2))
            .max(self.p.torso.mul(fx(3, 25)))
    }

    /// Shift every grounded pair along the body by up to `want`, keeping
    /// their order, their spacing, and the torso under them. Returns what
    /// it managed.
    fn shift(&mut self, want: Fx) -> Fx {
        let indices: [usize; STATIONS] = [0, 1, 2, 3];
        // The furthest any pair may go, forward (positive) or back.
        let mut room = want.abs();
        for i in indices {
            if !self.pose.stations[i].grounded {
                continue;
            }
            let at = self.pose.stations[i].attach;
            let limit = if want > Fx::ZERO {
                self.p.front.sub(at)
            } else {
                at.sub(self.p.rear_attach)
            };
            room = room.min(limit.max(Fx::ZERO));
        }
        let shift = if want > Fx::ZERO { room } else { room.neg() };
        for i in indices {
            if self.pose.stations[i].grounded {
                self.pose.stations[i].attach = self.pose.stations[i].attach.add(shift);
            }
        }
        // Keep neighbours apart: walk from the hind end forward.
        for k in (1..STATIONS).rev() {
            if let (true, true) = (
                self.pose.stations[k].grounded,
                self.pose.stations[k - 1].grounded,
            ) {
                let gap = self.spacing(k, k - 1);
                let behind = self.pose.stations[k].attach;
                if self.pose.stations[k - 1].attach < behind.add(gap) {
                    self.pose.stations[k - 1].attach = behind.add(gap);
                }
            }
        }
        shift
    }

    /// Lean every grounded limb so its foot lands up to `want` further
    /// forward (or back), within each limb's cap.
    fn reach(&mut self, want: Fx) -> Fx {
        let mut room = want.abs();
        for (i, s) in self.grounded() {
            let pair = self.g.limbs[i].unwrap();
            let standing = standing_length(self.g.stance, pair.build, s.wing_walks, self.p.limb[i]);
            let cap = standing.mul(REACH_CAP);
            let left = if want > Fx::ZERO {
                cap.sub(s.reach)
            } else {
                cap.add(s.reach)
            };
            room = room.min(left.max(Fx::ZERO));
        }
        let step = if want > Fx::ZERO { room } else { room.neg() };
        for s in self.pose.stations.iter_mut() {
            if s.grounded {
                s.reach = s.reach.add(step);
            }
        }
        step
    }

    /// Pitch the torso, a hundredth of a turn at a time, as long as it
    /// brings the mass toward the support, up to `want` of correction and
    /// the cap either side of where the sliders put it.
    fn pitch(&mut self, want: Fx, base: Fx) -> Fx {
        let step = fx(1, 100);
        let mut done = Fx::ZERO;
        let mut e = self.error();
        for _ in 0..12 {
            if done.abs() >= want.abs() || e == Fx::ZERO {
                break;
            }
            // Try nose up; keep it if the error shrank, else try nose down.
            let mut improved = false;
            for dir in [step, step.neg()] {
                let next = self.pose.pitch.add(dir);
                if next.sub(base).abs() > pitch_cap(self.g.stance) {
                    continue;
                }
                let before = self.pose.pitch;
                self.pose.pitch = next;
                let e2 = self.error();
                if e2.abs() < e.abs() {
                    done = done.add(e.sub(e2));
                    e = e2;
                    improved = true;
                    break;
                }
                self.pose.pitch = before;
            }
            if !improved {
                break;
            }
        }
        done
    }

    /// Balance: the binary strategies where they apply, then the
    /// continuous ones in a rolled mix, a few rounds.
    fn balance(&mut self) {
        let base = self.pose.pitch;
        let e0 = self.error();
        let wing = self
            .g
            .pairs()
            .find(|(_, p)| p.role == Role::Wing)
            .map(|(st, _)| st.index());
        let has_hind = self.pose.stations[2].grounded || self.pose.stations[3].grounded;
        let has_fore = self.pose.stations[0].grounded || self.pose.stations[1].grounded;

        // Nothing behind the legs to hold the rear up: it drags.
        if !has_hind && has_fore && self.g.stance != Stance::Limbless {
            self.pose.drags = true;
            self.pose.drag_len = self.p.torso.mul(fx(2, 5));
            self.pose.applied.drags = true;
        }
        // Weight ahead of the legs and wings to put down: half the time
        // the wings walk.
        if e0 > Fx::ZERO {
            if let Some(w) = wing {
                if !self.pose.stations[w].grounded && self.rng.chance(fx(1, 2)) {
                    self.pose.stations[w].grounded = true;
                    self.pose.stations[w].wing_walks = true;
                    self.pose.stations[w].reach = self.p.limb[w].mul(fx(1, 5));
                    self.pose.applied.wing_walks = true;
                }
            }
        }

        for _ in 0..6 {
            let e = self.error();
            if e == Fx::ZERO {
                break;
            }
            // A rolled mix: each strategy a share of the correction, and
            // any of them may sit this one out.
            let mut ws = [Fx::ZERO; 3];
            for w in ws.iter_mut() {
                *w = if self.rng.chance(fx(1, 3)) {
                    Fx::ZERO
                } else {
                    self.rng.unit()
                };
            }
            if ws.iter().all(|w| *w == Fx::ZERO) {
                ws[self.rng.below(3)] = Fx::ONE;
            }
            let total = ws[0].add(ws[1]).add(ws[2]).max(fx(1, 100));
            let shares = [
                e.mul(ws[0]).div(total),
                e.mul(ws[1]).div(total),
                e.mul(ws[2]).div(total),
            ];
            // The support moves toward the mass: shift and reach go the
            // way the mass is; the pitch finds its own direction.
            let did_shift = self.shift(shares[0]);
            let did_reach = self.reach(shares[1]);
            let did_pitch = self.pitch(shares[2], base);
            self.pose.applied.shift = self.pose.applied.shift.add(did_shift);
            self.pose.applied.reach = self.pose.applied.reach.add(did_reach);
            self.pose.applied.pitch = self.pose.applied.pitch.add(did_pitch);
            if did_shift == Fx::ZERO && did_reach == Fx::ZERO && did_pitch == Fx::ZERO {
                // Everything is at its cap. The wings, if any, come down
                // even if the roll said not to; then there is nothing more.
                if let Some(w) = wing {
                    if !self.pose.stations[w].grounded && self.error() > Fx::ZERO {
                        self.pose.stations[w].grounded = true;
                        self.pose.stations[w].wing_walks = true;
                        self.pose.stations[w].reach = self.p.limb[w].mul(fx(1, 5));
                        self.pose.applied.wing_walks = true;
                        continue;
                    }
                }
                break;
            }
        }
        self.force();
    }

    /// Whatever is left, the limbs take regardless of their caps: a body
    /// stands before it looks right. A shift along a pitched body moves
    /// the feet by its cosine, so the step is scaled up for that.
    ///
    /// On a body pitched near the vertical a shift along it is mostly a
    /// change of height, so the step is not scaled past double, and no
    /// limb leaves the torso: a plan that still does not balance after
    /// this is reported as such rather than stood on hips under the floor.
    fn force(&mut self) {
        for _ in 0..3 {
            let e = self.error();
            if e == Fx::ZERO {
                break;
            }
            let c = cos_turns(self.pose.pitch).max(fx(1, 2));
            let step = e.div(c);
            let mut moved = Fx::ZERO;
            for s in self.pose.stations.iter_mut() {
                if s.grounded {
                    let to = s.attach.add(step).clamp(self.p.rear_attach, self.p.front);
                    moved = moved.add(to.sub(s.attach));
                    s.attach = to;
                }
            }
            if moved == Fx::ZERO {
                break;
            }
            self.pose.applied.shift = self.pose.applied.shift.add(step);
        }
    }

    /// Hip height at a station: the root's, plus the pitch's lift along
    /// the body, less where the limb hangs on the flank.
    fn hip_height(&self, i: usize, root: Fx, pitch: Fx) -> Fx {
        root.add(self.pose.stations[i].attach.mul(sin_turns(pitch)))
            .sub(self.p.half_h.mul(ATTACH_DOWN))
    }

    /// The root heights at which every grounded limb reaches the floor at
    /// this pitch, and the one they would like.
    fn vertical_at(&self, pitch: Fx) -> (Fx, Fx, Fx) {
        let mut lo = self.p.half_h; // the belly stays off the floor
        let mut hi = Fx::from_int(10_000);
        let mut want = Fx::ZERO;
        let mut n = 0;
        for (i, s) in self.grounded() {
            let pair = self.g.limbs[i].unwrap();
            let standing = standing_length(self.g.stance, pair.build, s.wing_walks, self.p.limb[i]);
            let lift = s.attach.mul(sin_turns(pitch));
            let hang = self.p.half_h.mul(ATTACH_DOWN);
            // hip = root + lift - hang must be within [EXT_MIN, EXT_MAX] of standing
            lo = lo.max(standing.mul(EXT_MIN).sub(lift).add(hang));
            hi = hi.min(standing.mul(EXT_MAX).sub(lift).add(hang));
            want = want.add(
                standing
                    .mul(natural_extension(self.g.stance, pair.build, s.wing_walks))
                    .sub(lift)
                    .add(hang),
            );
            n += 1;
        }
        if n > 0 {
            want = want.div(Fx::from_int(n));
        } else {
            want = self.p.half_h;
        }
        (lo, hi, want)
    }

    /// Choose the root height and, if the limbs cannot all reach at this
    /// pitch, the pitch too.
    fn vertical(&mut self) {
        if self.g.stance == Stance::Limbless {
            self.pose.root = self.p.half_w;
            return;
        }
        let base = self.pose.pitch;
        let (mut lo, mut hi, mut want) = self.vertical_at(base);
        if lo > hi {
            // Search nearby pitches for one where they all reach.
            let step = fx(1, 100);
            let mut best = (lo.sub(hi), base);
            let mut found = None;
            for k in 1..=12 {
                for dir in [1, -1] {
                    let p = base.add(step.mul(Fx::from_int(k * dir)));
                    let (l, h, _) = self.vertical_at(p);
                    if l <= h {
                        found = Some(p);
                        break;
                    }
                    if l.sub(h) < best.0 {
                        best = (l.sub(h), p);
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            let p = found.unwrap_or(best.1);
            self.pose.applied.pitch = self.pose.applied.pitch.add(p.sub(self.pose.pitch));
            self.pose.pitch = p;
            let v = self.vertical_at(p);
            lo = v.0;
            hi = v.1;
            want = v.2;
        }
        let root = if lo <= hi {
            want.clamp(lo, hi)
        } else {
            // Still impossible: as low as the shortest limb needs, the
            // long ones folding deeper than they like -- a foot on the
            // floor before a comfortable knee. Only a belly on the floor
            // leaves a limb hanging.
            hi.max(self.p.half_h)
        };
        self.pose.root = root;
    }

    /// With the root and the pitch settled, each limb's bend and how far
    /// its foot may reach -- after everything that moves a hip.
    fn feet(&mut self) {
        let root = self.pose.root;
        let pitch = self.pose.pitch;
        for i in 0..STATIONS {
            if !self.pose.stations[i].grounded {
                continue;
            }
            let pair = self.g.limbs[i].unwrap();
            let standing = standing_length(
                self.g.stance,
                pair.build,
                self.pose.stations[i].wing_walks,
                self.p.limb[i],
            );
            let hip = self.hip_height(i, root, pitch);
            let ext = if standing > Fx::ZERO {
                hip.div(standing)
            } else {
                Fx::ONE
            };
            let s = &mut self.pose.stations[i];
            s.dangles = ext > Fx::ONE;
            s.extension = ext.clamp(fx(1, 5), Fx::ONE);
            // The reach cannot take the foot further than the limb goes.
            let max_reach = standing
                .mul(standing)
                .sub(hip.mul(hip))
                .max(Fx::ZERO)
                .sqrt();
            s.reach = s.reach.clamp(max_reach.neg(), max_reach);
        }
    }

    /// A hopper: hind limbs longer than the fore on a level body.
    fn hop(&mut self) {
        let fore = self.p.limb[0].max(self.p.limb[1]);
        let hind = self.p.limb[2].max(self.p.limb[3]);
        let fore_grounded = self.pose.stations[0].grounded || self.pose.stations[1].grounded;
        let hind_grounded = self.pose.stations[2].grounded || self.pose.stations[3].grounded;
        if self.g.stance == Stance::Level && fore_grounded && hind_grounded && fore > Fx::ZERO {
            self.pose.hop = hind.sub(fore).div(fore).clamp(Fx::ZERO, Fx::ONE);
        }
    }

    /// Carry the head above the back: raise the neck until the head's
    /// centre clears the torso's top line, unless the head is a grazer's.
    fn head(&mut self) {
        let base = match self.g.head {
            HeadKind::Raised => fx(1, 10),
            HeadKind::Low => fx(-3, 25),
            HeadKind::Forward | HeadKind::Merged => Fx::ZERO,
        };
        let mut rise = base;
        if matches!(self.g.head, HeadKind::Forward | HeadKind::Raised) {
            let arm = self.p.neck_len.add(self.p.head_half);
            for _ in 0..25 {
                let lift = arm.mul(sin_turns(self.pose.pitch.add(rise)));
                let front_top = self.p.half_h.mul(fx(3, 5));
                if lift >= front_top || rise >= fx(1, 4) {
                    break;
                }
                rise = rise.add(fx(1, 100));
            }
        }
        self.pose.neck_rise = rise;
    }
}

/// How the genome stands. Deterministic: the same genome always gets the
/// same posture.
pub fn posture(g: &Genome) -> Posture {
    let p = proportions(g);
    let mut stations = [StationPose::default(); STATIONS];
    for st in Station::ALL {
        let i = st.index();
        if let Some(pair) = g.limbs[i] {
            stations[i] = StationPose {
                attach: p.attach[i],
                reach: Fx::ZERO,
                extension: Fx::ZERO,
                grounded: pair.role.grounded(),
                wing_walks: false,
                dangles: false,
            };
        }
    }
    let masses = masses(g, &p);
    let mut solver = Solver {
        g,
        p,
        masses,
        rng: Rng::new(g.fingerprint()),
        pose: Posture {
            // The slider's pitch, a quarter turn at one -- but a low-slung
            // body lies flat whatever its slider says: its belly is on
            // the floor and its legs are short.
            pitch: if g.stance == Stance::Lowslung {
                g.sliders.pitch.mul(fx(1, 4)).min(fx(1, 50))
            } else {
                g.sliders.pitch.mul(fx(1, 4))
            },
            root: Fx::ZERO,
            stations,
            drags: false,
            drag_len: Fx::ZERO,
            hop: Fx::ZERO,
            neck_rise: Fx::ZERO,
            com: Fx::ZERO,
            support: (Fx::ZERO, Fx::ZERO),
            balanced: false,
            applied: Applied::default(),
        },
    };
    if g.stance != Stance::Limbless && g.pair_count() > 0 {
        solver.balance();
    }
    solver.vertical();
    // The vertical may have pitched it to get every foot down; the feet
    // follow the weight once more.
    solver.feet();
    if g.stance != Stance::Limbless && g.pair_count() > 0 {
        // Clamping the reach can move the feet off the weight again: the
        // last word is the limbs', and then the feet are settled on it.
        solver.force();
        solver.feet();
    }
    solver.hop();
    solver.head();
    solver.pose.com = solver.masses.com(solver.pose.pitch);
    solver.pose.support = solver.support();
    solver.pose.balanced = g.stance == Stance::Limbless || solver.error() == Fx::ZERO;
    solver.pose
}
