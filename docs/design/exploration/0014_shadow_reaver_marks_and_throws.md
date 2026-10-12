---
status: decided 2026-10-12 by the owner's brief, designed and built by Claude; unplayed
started: 2026-10-12
supersedes: 0013_shadow_reaver_dual_blades.md where they differ
depends: 0013_shadow_reaver_dual_blades.md, ../kits/shadow-reaver.md, ../shadow-reaver-v2.md, ../aiming.md
---

# 0014 — The Shadow Reaver, third pass: marks, throws and the shadow's clock

## The brief

From the owner, after playing the second pass (0013): "Much better, but needs some polish
still." In order, the asks:

1. **The finishers cash the marks** — "low damage into a high, super impactful burst", and
   "consuming 4 to 5 stacks should get relatively more damage than going 3 to 4". The marks
   are on one clock: *a shadow blade hits → marks = min(5, marks + 1), clock = 2 s; the clock
   runs out → marks = max(0, marks − 1), clock = 2 s*.
2. **Every shadow blade marks**: the throws, every burst, and so on.
3. **The aerial left click aims**, but never outside a 45° cone round straight down.
4. **The jumping left click works in the air**: the shadow goes out first, she blinks to it,
   then plunges in the same cone.
5. **Every send of the shadow is on a ~3 s clock home**, reset by another send.
6. **The shadow rejoins her whenever she is near it**, not only on the dash; the leash is
   removed.
7. **Collecting the shadow bursts** — the dash, and standing on it.
8. **The aerial right click (Whirl)** reaches as far as the grounded send, and a second one
   resets the spinning blades rather than adding a second set.
9. **The jumping right click** is the aerial right click with a jump.
10. **Middle click is three throws.** On foot, a line skillshot, and the shadow (wherever it
    is) throws one too at the same point; jumping, the Tumble as it is; in the air, a level
    fan that stalls her a moment.
11. **The camera**: off the ground, looking up, back out to third person as far as a
    body-height bubble round the eye allows — with no regression on the ground. See the last
    section.

## The grid

| | Left: finishers (cash the marks) | Middle: throws | Right: the shadow |
| --- | --- | --- | --- |
| **On foot** | **Reave** | **Shuriken** — hers, and the shadow's from wherever it is | **Send** (moves it) |
| **In the air** | **Plunge** — aimed in a 45° cone round down | **Fan** — five level blades, and a stall | **Whirl** — as far as the send; one set |
| **`space` + click** | **Gallows** — shadow up, blink to it, plunge in the cone | **Tumble**, as it was (and the double jump) | **Rising whirl** — a jump and a Whirl |

| Key | Move |
| --- | --- |
| `Q` | Guillotine lotus, as built |
| `E` | Executioner, as built — and it **does not cash** the marks |

Slots kept from 0013 by number where the move survived; the Blade ring, the Blade spin and the
Shadow arc were retired, and their slots carry the Fan, the Rising whirl and the Shuriken.
`moves::Shape::Disc` and the hitbox's height band went with the two spins.

## The marks

Every shadow blade marks a body once a pass — the shadow's copies of her swings, the Whirl
(each body once a Whirl), the Shuriken (hers and the shadow's are two marks), the Fan's and
the Tumble's blades (each blade once), the burst, the lotus. The cap is five, and **one clock
for the whole tally**: a mark resets it to 120 frames (`marks, one fades every`); when it runs
out one mark goes and it starts again.

**Only the finishers cash it** — the Reave, the Plunge and Gallows (`moves::reaver::cashes`).
The multiplier is `1 + 0.25·m + 0.07·m²` (`cash-in, damage per mark` and the new
`cash-in, extra per mark squared`), so each mark is worth more than the last:

| marks | 0 | 1 | 2 | 3 | 4 | 5 |
| --- | --- | --- | --- | --- | --- | --- |
| × | 1.00 | 1.32 | 1.78 | 2.38 | 3.12 | 4.00 |
| step | | +0.32 | +0.46 | +0.60 | +0.74 | +0.88 |

A full tally: Reave 280, Plunge 520, Gallows 680, and the 30-frame stagger as before.
Executioner stays on `E` as the committed overhead that does not need the setup — it does not
cash, so it does not spend a tally a finisher would have paid more for.

## Left — finishers

- **Reave**, unchanged: the short dash and the cut with the sweet spot.
- **Plunge** (in the air). She stops where she is and dives **along the crosshair, held inside
  a cone of 45° round straight down** (`aim::plunge_dir`, `Plunge and Gallows, cone round
  straight down`): looking steeply down she goes where she looks; looking level or up she goes
  45° down in the direction she faces. 26 m/s, 130, and the ring of shadow from the landing as
  before (it does not mark — it is a shove, not a blade).
- **Gallows** (`space` + left, on the floor or in the air). The shadow goes **first**, up and a
  little ahead (`Gallows, the shadow goes up` 3 m, `goes ahead` 1 m, `aim::hung_ahead`); the
  frame it arrives she **blinks to it**, which collects it (so it bursts), hangs a beat
  (`Gallows, hangs after the blink`, 6 frames), and plunges in the same cone as the Plunge.
  170, 32 frames of recovery, −19 on block. From the floor the landing hook would have started
  its active frames before the blink; it waits for the blink now.

## Middle — throws

- **Shuriken** (on foot). 5/3/12, a line skillshot to the crosshair's point, 30, 14 m. **The
  shadow throws one too**, from its middle wherever it stands — at her shoulder or out on the
  field — to the same point. Two blades, two marks.
- **Fan** (in the air). Five blades, level, spread either side of her facing
  (`thrown blades, fan spread`), 22 each, 9 m; and she **stalls** for 16 frames.
- **Tumble** (`space` + middle), as it was: a flipping jump and three blades forward and down,
  and in the air once a trip it is her double jump. A second press in that trip is the Fan.

Thrown blades fly at `thrown blades, fly at` (32 m/s) and cut each body once.

## Right — the shadow

- **The clock home.** Every send — the grounded send, the Whirl, the Rising whirl, Gallows —
  starts `shadow, comes home after a send` (180 frames, three seconds) when the shadow arrives.
  Another send resets it. When it runs out the shadow comes home, cutting on the way as ever.
- **No leash.** `Shadow leash` is gone: nothing pulls the shadow home for distance. The dash is
  still only offered within what a dodge's frames cover at the dash's speed.
- **Collecting.** Standing within `shadow, collected standing within` (1.2 m) of a waiting
  shadow rejoins it, as the dash does. Either way it **bursts**: blades thrown round her, 45, a
  2.5 m thin disc, marking. Its coming home on its own clock is not a collection and does not
  burst.
- **Whirl** (in the air): reach 9 m, as the send. The crosshair's point is its middle, but it
  stops at the first body or creature on the way (`aim::first_along`), so a Whirl aimed at a
  creature's part spins there rather than behind it. **A second Whirl moves the one set of
  blades** and starts its clock again; there is never more than one.
- **Rising whirl** (`space` + right): a jump (`self lift` 12 m/s) and a Whirl.

## What it does to movement

Gallows is now a blink — 3 m up and 1 m ahead, and then a dive she can aim — and the Fan
stalls her in the air. **The lab search finds her going further than after 0013, not less.**
`cargo run --release -p sim --bin envelope -- search --class "Shadow Reaver"`, shared / whole
kit, metres:

| Rise | −8 | −4 | −2 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 8 | 10 | 12 | 16 | 20 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0013 | 29.5/62.7 | 45.5/56.7 | 58.5/60.3 | 55.5/70.3 | 48.3/61.2 | 53.2/63.4 | 66.6/67.4 | 15.9/16.1 | 49.1/62.8 | 16.6/51.1 | 53.8/76.2 | –/– | 57.9/62.6 | –/58.0 | –/– |
| 0014 | 26.5/61.3 | 34.0/42.2 | 53.2/53.6 | 59.8/78.4 | 44.5/50.1 | 45.1/45.1 | 61.7/89.4 | 43.2/44.0 | 47.9/60.6 | 46.6/70.2 | 55.9/78.6 | 52.3/52.6 | –/14.9 | 51.7/61.6 | –/61.2 |

A search is a lower bound, so a cell that fell may be a route this run did not find again; the
ones that rose (lane +4 m, from 16 to 43 m; lane +10 m, from nothing to 52 m) are new routes.
Her fixture lines in `tests/fixtures/envelope.txt` are re-recorded. REAVER-5 (0013) is still
open, still the owner's, and the new levers are Gallows' blink and the Fan's stall.

## Questions for play

| ID | Question | Built as |
| --- | --- | --- |
| **REAVER-10** | Is 4× at five marks "super impactful" without one Gallows ending a round? | 1 + 0.25m + 0.07m²; Gallows 680 full |
| **REAVER-11** | Should Executioner cash too? | No: finishers only |
| **REAVER-12** | Is 1.2 m the right "near" for collecting a standing shadow? | 1.2 m, and not during a dash |
| **REAVER-13** | Should the Plunge's ring mark? | No: it is a shove |

## The camera: a bubble round the eye

The ask: "replace the shrink-the-sphere-as-pitch-goes-up first person with an obstacle-based
bubble", keep the ground as it is, pull back to third person in the air, the crosshair fixed
and on the character, and **no regressions** — "or tell me why you can't".

**What changed.** Looking up, the zones shrink the camera's sphere onto the head (the
handover) because an eye swung down behind a standing fighter would go into the floor. In the
air that floor is somewhere else. So the radius is **let back out toward the full sphere as far
as keeping the eye `Eye bubble` (1.8 m, a body) off the floor under the fighter allows** —
the hard edge — eased in over the first `Eye bubble, soft edge` (2 m) of room above that — the
soft edge (`camera::let_out`). The room is a new snapshot number, `Player::clear`: the feet's
height off the floor under them, nought standing, and allowed to grow at most
`Eye bubble, most it opens a frame` (0.25 m) a frame so stepping off a ledge eases out.

**Why it cannot regress on the ground.** The room is never more than `clear`, which is nought
whenever the feet are on anything, and nought room is the zones' radius exactly
(`tests/eye_bubble.rs`, `standing_the_eye_is_the_eye_it_always_was`). Looking level or down the
zones already keep the full sphere, so height changes nothing there either.

**Why the crosshair cannot move.** Only the distance changes; the tilt — where the sphere's
centre sits on screen — is the zones'. Looking up the zones centre the head, so the crosshair is
on her at every distance (`in_the_air_looking_up_the_camera_is_third_person_and_on_the_head`).

**The lessons kept** (feel log, 2026-09-11 to 2026-09-14):

- *Never solve for the eye* — nothing is solved; the radius is a closed form in the room and
  the elevation, and the mouse still walks the sphere one degree a degree.
- *The eye is the aim* — it is in the simulation, hashed, and the bots' `aim::look_onto` asks
  the same camera (`clear` is passed beside `aloft`).
- *The pole under the camera* — the floor limit does not shorten an arm at a fixed height; it
  holds the eye a bubble off the floor, and the drawn camera's floor clamp and wall pull-in come
  back in proportion as the eye leaves the head (`view::camera`, `sky` times what is still
  inside it).
- *No jumps* — continuous in pitch, in height and on a ledge (`the_eye_moves_continuously_…`).

**What it does not do, plainly.** It sees only the floor under the fighter, not a wall or a
ledge behind them: those are the drawn camera's to pull in from, as in third person anywhere.
And it is not quite as gentle as the ground's fastest moment: a metre or so up, a few degrees
above level, the handover's shrink and the floor limit arrive in the same degree and the eye
moves up to a fifth faster per degree of mouse than the handover itself does on the ground —
continuous, but quicker. A one-metre soft edge made that half again; two is the setting.
