---
status: living document
opened: 2026-09-10
---

# Feel log

Tuning is going to be fragmented over a long time. Without a record, the same
number gets tried twice and the reason a value settled where it did is lost —
and "why is dodge 22 frames" becomes unanswerable six months later.

**This file is the memory.** The numbers live in `crates/sim/src/tuning.rs`; the
*reasoning* lives here.

## How to use it

One entry per experiment. Write the entry when you make the change, and fill in
the verdict when you have played it. An entry with no verdict is an open
experiment, and that is fine — it is still better than an unrecorded change.

```
### YYYY-MM-DD — short name
**Changed** what moved, from what to what
**Why** the hypothesis
**Verdict** kept / reverted / partial — and what it actually felt like
```

Reverted experiments are the most valuable entries in the file. Keep them.

## What is testable and what is not

"Feels good" cannot be a test. But the *properties that must hold for the design
to work* can be, and they are, in `crates/sim/tests/feel.rs`:

- Every attack must be punishable on block, or blocking is pointless.
- The parry window must be shorter than the fastest attack's startup, or parry
  is a mash rather than a read.
- Dodge invulnerability must be shorter than the dodge, or it is never punished.
- A committed move must be committed — recovery long enough that whiffing it
  actually costs something.

Those tests pin *relationships*, not values. Numbers should move freely; if one
of these fails, either it is a bug or a design decision changed and the document
needs updating too.

`cargo run -p sim --bin frametable` prints the derived table — on-block and
on-hit advantage for every move — which is what tells you at a glance whether a
move is safe.

---

## Open questions

Things the design cannot answer and only play can. Move these into dated entries
as they get tested.

### Movement
- **Is 7 m/s the right walk speed?** Everything else is spaced against it. Too
  slow makes the arena feel large and approaches feel committal; too fast makes
  spacing imprecise.
- **Should there be any air control?** Currently none: your jump arc is set at
  takeoff. That is the platform-fighter-adjacent choice and it makes jumps
  readable, but it may feel stiff in 3D.
- **Does the dodge cover the right distance?** It is the only hard evasive
  option, so its distance sets the whole neutral spacing game.

### Attacks
- **Is Bash at 4 frames of startup too fast to react to?** Human reaction is
  roughly 15 frames at 60 Hz, so a 4-frame move is unreactable by design. That
  is correct for a poke you are meant to *anticipate*, but it may make neutral
  feel like a coin flip.
- **Is Slam at 14 frames readable enough to punish?** It should be the move you
  learn to dodge on sight.
- **Is the grapple's 20-frame startup too slow to ever land?** It beats guard,
  so it needs to be slow — but if it never connects it is decoration.

### Defence
- **Is the 4-frame parry window findable?** This is the single most important
  open question. Too tight and nobody uses it; too loose and it beats everything.
  Test by setting the dummy to attack (key 3) and trying to parry on reaction.
- **Is block stunlock long enough to make blocking a real cost?**
- **Does crouch see enough use?** It only beats overheads, and there is one.

### Animation

- Does the baked overhead read as *heavy*, or just as *slow*? The lag budget
  says it arrives on time; whether it lands with weight is a human judgement.
- Roll and recoil are placeholders. The roll in particular has no sense of
  the body compressing and unfolding.
- Should hitstun drive the recoil clip's playback rate, or is a fixed-length
  clip clearer to read? Fixed length is easier to reason about in frames.

### Camera

- Is the mouse sensitivity in the right range? It is currently one number with no in-game
  way to change it, which is not good enough for more than one person to play.
- Does the camera distance work at both melee and full-screen range, or does it want to
  pull in when close the way the old rig pulled back when far?
- Is the shoulder offset on the correct side, and should it swap when you put your back
  to a wall?
- Does a locked facing during a move read as commitment or as the game ignoring you?
- Does the camera-relative dodge do the right thing when you dodge *toward* the camera?
- Is 1.0 the right default sensitivity, and is an 8% notch the right step?
- Is 58° the right default field of view? It is the number most likely to differ between
  people, which is why it is a setting.
- Does the camera want to pull *in* at melee range and back out at distance, rather than
  holding one distance? The old auto-framing rig did that for free and it has not been
  missed yet — but nobody has played a real match.
- Does the crosshair sliding off centre during a committed move read as useful information,
  or just as the reticle glitching? The alternative is to freeze it in place and only dim it.
- Should the crosshair show reach — whether the thing under it is actually within range of
  the move you are about to throw?
- **Does the auto-frame pull in too aggressively when fighters close?**
- **Is the perpendicular angle right, or should it favour player one's side?**
- **Does occlusion pull-in read as a camera bug when it fires?**

### Rounds
- **Is 1000 health with these damage numbers close to the 60-second target?**
  Roughly six Slams or seventeen Bashes. Needs a real match to check.

---

## Entries

### 2026-09-10 — opening numbers
**Changed** everything, from nothing. First playable values.
**Why** They needed to be *something* to be judged. Chosen so the relationships
in `tests/feel.rs` hold, not because any individual number is believed.
**Verdict** open — awaiting first real match.

### 2026-09-10 — baked animation, first pass
**Changed** Added an offline animation factory (`crates/anim`) and a baked
clip table for poke, overhead, guard-in, roll and recoil. Toggle with **F2**;
`BAKED_ANIM=0` starts with the old procedural poses.
**Why** Hand-keyed poses read as a slideshow. The parts that make motion look
alive — the arm trailing the shoulder, the overshoot at the end of a swing —
are exactly the parts that are miserable to key by hand, and easy for a
spring-damper to generate. Baking offline keeps `pose = f(state)`, so none of
this touches rollback.
**Verdict** kept. Reads clearly better than the procedural poses on the poke
and the overhead. Roll and recoil are adequate, not good.

### 2026-09-10 — "heavy" was making moves invisible
**Changed** `Looseness` presets no longer take a spring frequency. Each part is
now described by **lag in frames** and **ring**, and the frequency is derived.
HEAVY went from roughly six frames of lag on the torso to 2.4.
**Why** The first pass expressed weight as a low spring frequency. A spring
chasing a moving target settles into a lag of about `2·ring/frequency`, so a
low frequency does not mean *heavy*, it means *late*. The Bulwark's slam had a
14-frame startup and its silhouette had barely moved by frame 5 — the opponent
was supposed to be reading that windup and deciding whether to block, and there
was nothing on screen to read.
**Verdict** kept, and this is the useful lesson: weight should read as
follow-through and settle, never as delay. A heavy part still has to *arrive on
time*, it just carries past the mark and takes longer to stop wobbling. Pinned
by `the_lag_knob_means_what_it_says` and `heavy_parts_arrive_late_but_not_absent`.

### 2026-09-10 — the demo replay was not frame-exact
**Changed** Scripted and dummy inputs are now sampled once per simulation tick
instead of once per rendered frame.
**Why** One rendered frame can cover several simulation ticks. Reusing a single
input sample across all of them smeared the demo script's four-frame presses
into whatever the frame rate happened to be, so two runs of the same script
diverged. Two screenshots of "the same moment" were two different moments.
**Verdict** kept. `SHOT_FRAME=N` now stops the simulation on an exact frame and
the frame number is drawn next to the score, so a capture says which moment it
caught. Comparisons across a long, fragmented experiment are only worth
anything if they are reproducible.

### 2026-09-11 — the camera became the aim
**Changed** Deleted the auto-framing camera. Third-person, behind the fighter, mouse-look,
and the player owns it. Movement is camera-relative — `W` is away from the camera, not along
a world axis — and attacks go where the camera points.
**Why** The old rig framed both fighters in profile and swung on its own. It was explicitly
a *consequence* of a control scheme in which nothing needed mouse-look, and aimed attacks
break that premise. It was also 1v1-only: framing "both fighters" means nothing in coop.
**Verdict** kept. Not judged in a real match yet; the open questions below are the ones a
match has to answer.

### 2026-09-11 — a centred camera hides your opponent
**Changed** The eye sits about a metre to one side. The aim point does not move with it.
**Why** First pass put the camera directly behind the fighter. At melee range — which is
most of this game — your own body sits exactly between the camera and the person you are
fighting. Raising the camera does not fix it: a body is wider than a sightline. This is
why over-the-shoulder cameras exist.
**Verdict** kept. Worth noting the shape of the fix: the *eye* moved and the aim point
stayed on the look axis, so the crosshair is still straight ahead of the fighter and
nothing was traded away for the visibility. The test had to be rewritten to assert the aim
point rather than the camera's own axis — the old test only passed because the two used to
coincide.

### 2026-09-11 — facing locks when a move starts
**Changed** During startup, active and recovery frames, the mouse moves the camera but not
the fighter. Guard turns at a limited rate rather than snapping.
**Why** With facing tied to the mouse, an uncommitted facing means dragging a live hitbox
around during its active frames — every whiff becomes rescuable by turning after the fact,
and whiff punishment is most of the game. The guard case is the same argument: an arc you
can flip instantly is a bubble with extra steps.
**Verdict** kept, and this is the one most likely to need tuning. Whether a locked facing
feels *committed* or just *unresponsive* is exactly the sort of thing that cannot be
reasoned out. The guard turn rate in particular is a single number pulled out of the air.

### 2026-09-11 — the arena was too dark to read from the new angle
**Changed** Added an unshadowed fill light from the opposite side, roughly doubled ambient,
lightened the stone.
**Why** Not a taste change. The old camera looked down into the arena from outside it; this
one looks along the floor at the *inside* faces of the walls, which the key light never
reaches. They rendered as flat black and the fight appeared to happen in front of a void.
**Verdict** kept, but this is a stopgap — the lighting was built for a camera that no longer
exists and deserves a proper pass.

### 2026-09-11 — sensitivity, and a crosshair that does not lie
**Changed** Mouse sensitivity is adjustable in-game on `-` / `=`, shown on screen, and saved
to `~/.config/arena/settings.conf`. Added a crosshair.
**Why** Sensitivity was one hardcoded number, which does not survive two people sharing a
keyboard. The steps are multiplicative because sensitivity is felt as a ratio: 0.5 to 0.6 is
a large change and 5.0 to 5.1 is not one you can feel.
**Verdict** kept. The interesting half is the crosshair. The obvious build is a cross painted
at screen centre — but facing locks when a move starts and lags while guarding, so for a real
fraction of every match the camera points somewhere the attack will not go. A centred reticle
would be confidently wrong exactly when it matters. So it is projected from where the fighter
is actually pointed: still in the middle while facing tracks aim, sliding off when it does
not, dimmed while committed. Whether the slide reads as *information* or as *the crosshair
being broken* is a question for a real match.

### 2026-09-11 — the pole under the camera
**Changed** Looking up now lowers the camera to the ground and lets it ride along at full
distance, instead of hauling it in toward the fighter's head at a fixed height.
**Why** Reported as "it feels like there's a five foot pole under the camera", which was an
exact description of the bug. The floor clamp compared the camera arm against the ground
*without counting the eye lift*, so it fired about three metres early — a ten-degree glance
upward already triggered it — and it resolved the imaginary collision by shortening the arm.
The eye height was therefore pinned at exactly `floor + lift` while the camera slid in toward
the head. A pole, with the camera on it.
**Verdict** kept. The general lesson is the one worth keeping: **how you refuse an illegal
camera position is itself a feel decision.** Shortening the arm and clamping the height both
keep the camera above the floor, and they feel nothing alike. At a 23-degree look up the
camera now stays at 92% of its distance instead of 36%.

Two things fell out of it. Fixing the clamp let the camera reach far enough back to discover
that the shoulder offset was being applied *after* the occlusion check — so the camera could
be slid sideways into a platform that had just been cleared. That had been wrong since the
shoulder was added and was only hidden by the broken clamp keeping the arm too short to
reach anything. And at full pitch the camera now sits on the ground close behind the fighter,
who fills a good deal of the frame; whether the 57-degree pitch limit is too generous is a
question for a match.

### 2026-09-11 — it felt cramped
**Changed** Vertical field of view 45° → 58°, camera distance 6.2 m → 7.0 m. Both are now
settings, on `F3`/`F4` and `F5`/`F6`, saved like sensitivity.
**Why** Reported as cramped. The field of view was Bevy's default 45°, which is a portrait
lens pointed at an arena you are supposed to be moving around inside — it had never been set
deliberately at all.
**Verdict** kept. Worth recording what the first attempt got wrong: 62° at 7.6 m opened it up
so far that all four walls were in frame and the fighters read as small. Both knobs shrink the
subject, so they multiply rather than add, and reaching for both at full strength overshoots.
The pair that landed gives roughly two thirds the subject size of before, not half.

The more useful outcome is that this is the third camera-feel pass in a row, so field of view
and distance became settings rather than constants. They are the first numbers anyone reaches
for when a camera feels wrong, and a value you have to rebuild to try is a value that gets
tried once. Candidate framings can now be rendered straight from the settings file without
touching the code, which is how the two above were compared.
