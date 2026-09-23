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

- **Is `space` plus a weapon a modifier the whole roster should have?** It is the
  Champion's takeoff row since 2026-09-14 and the first time the jump button has modified
  anything. A whole extra row of options per class, or a precedent that should not spread.
- Should shift with no direction and no click do something? A spot dodge in place is the
  obvious candidate and costs one branch.
- Is one airdodge per jump right, or does the air want a double jump as well?
- Is 1.5× the walk the right air speed cap? It is the number holding spacing together and it
  was picked, not derived.
- Does the per-class steering spread (0.9 to 1.7) read as character, or just as some classes
  being worse? A floaty class that also steers well may be strictly better.
- Is a 6-frame default aerial hang "slight"? It is the number most likely to be wrong.
- Does the airdodge want a landing-lag tail, the way platform fighters give it one? Right now
  it ends and you are free.
- **Is 7 m/s the right walk speed?** Everything else is spaced against it. Too
  slow makes the arena feel large and approaches feel committal; too fast makes
  spacing imprecise.
- **Should there be any air control?** Currently none: your jump arc is set at
  takeoff. That is the platform-fighter-adjacent choice and it makes jumps
  readable, but it may feel stiff in 3D.
- **Does the dodge cover the right distance?** It is the only hard evasive
  option, so its distance sets the whole neutral spacing game.

### Attacks

- Is 4.2 the right speed while poking, and should it differ per class? A spear poke and a
  hammer poke arguably should not hinder the same amount.
- **Is 1.4 the right speed while committed, and should it differ per move?** ⚠️ **Newly open,
  2026-09-14.** Committed moves stopped rooting and became a crawl; all sixteen got the same
  20% of the walk, which is a uniform first guess in exactly the way the poke's 60% was.
- Should mobility vary *across* a move's phases — free during startup, rooted through active,
  slowed in recovery? That is a common shape and might read better than a flat rate. It is a
  better question than it was: now that nothing roots for its whole length, rooting *only* the
  active frames would put "you committed to a spot" back where it belongs — the frames where
  the hitbox is actually out — for a handful of frames instead of forty.
- **Is Bash at 4 frames of startup too fast to react to?** Human reaction is
  roughly 15 frames at 60 Hz, so a 4-frame move is unreactable by design. That
  is correct for a poke you are meant to *anticipate*, but it may make neutral
  feel like a coin flip.
- **Is Slam at 14 frames readable enough to punish?** It should be the move you
  learn to dodge on sight.
- **Is the grapple's 20-frame startup too slow to ever land?** It beats guard,
  so it needs to be slow — but if it never connects it is decoration.
- **Is 30 frames the right repeat lockout?** The number is a first guess and
  nothing else. Too short and spamming an auto is still correct; too long and
  the correct play becomes standing still, which is the failure mode
  `feel::the_lockout_always_leaves_something_faster_to_do_than_wait` is watching
  for. Half a second was picked because it is roughly the gap between two
  deliberate button presses, and because at that length it happens to charge
  every auto and no committed heavy.
- **Which abilities want a multiplier, and which way?** The per-move column
  exists and is 100% on all thirty of them. The autos are the obvious
  candidates for more, since they are the ones a player leans on; the moves
  that already cost forty frames are candidates for zero, since the lockout is
  invisible on them anyway and a knob that does nothing is a knob that confuses
  somebody later.
- **Should a reactivation be gated at all?** `Move::reactivate` is zero
  everywhere, so the Reaver can send the shadow and recall it on consecutive
  frames. That may be fine — the send costs 18 frames of its own — or it may
  make the send-recall pair a single fast button rather than two decisions.
- **Where do the three stranded committed moves go?** ⚠️ **Newly open,
  2026-09-16.** Shift stopped modifying clicks, so the Bulwark's Slam, the
  Elementalist's Fissure and the Blood mage's Rend have no input at all. Each
  wants a different answer — the Bulwark has a free right click only while the
  shield is thrown, the Elementalist's is a ground ability that might belong on
  a crouch, and the Blood mage has `M` untouched — and guessing all three at
  once is how a grammar gets worse. Watching which of them is *missed* is the
  cheapest way to find out whether all three should come back.

### The Dual mage's depth curve

⚠️ **All newly open, 2026-09-16.** Depth scales everything she throws now; none
of these numbers has been played.

- **Is four-to-one the right spread?** Half at the centre, double at the edge.
  Too wide and the middle of the bar reads as broken rather than weak; too
  narrow and there is no reason to leave it. Both ends are one knob each
  (`Depth, power at the centre` / `at the edge`).
- **Should size ride the whole curve, or less of it?** `Depth, how much of it
  the size takes` is 100%, so a deep cast is exactly as much bigger as it is
  harder. More damage is worse to be hit by and more radius is harder to *not*
  be hit by, and those may not deserve the same slider.
- **Is a Judgement at full depth too much of a health bar?** The strike alone is
  about 43% of one, and the field can take it past 80% against somebody who
  stands in all of it.
- **Is "the form is the arm you last punched with" findable?** Nothing teaches
  it but the bar's colour. The two Lance clips are authored so the *opponent*
  can read which is coming; whether the player can read their own is a
  different question with a different answer.
- **Does the pull want to be weaker than the shove?** They are within 10% of
  each other. Being dragged is worth more than being shoved at the same speed,
  because you are usually walking into the shove and away from the pull.

### Defence
- **Is the 4-frame parry window findable?** This is the single most important
  open question. Too tight and nobody uses it; too loose and it beats everything.
  Test by setting the dummy to attack (key 3) and trying to parry on reaction.
- **Is block stunlock long enough to make blocking a real cost?**
- **Does crouch see enough use?** It only beats overheads, and there is one.

### The Oven

- A knob appended to the registry lands away from its family, which is fine for the palette
  now but means the *bake order* and the *reading order* diverge over time. Worth revisiting if
  the generated file becomes hard to scan.
- Curves. Several of these want to be splines rather than scalars — knockback that varies with
  damage, acceleration that eases. The store is integers so a curve is a new `Unit` and a new
  widget rather than a rewrite.
- Should baking open a pull request rather than pushing to the branch? Pushing is right for a
  solo session and probably wrong the moment someone else is on the branch.
- The palette has no undo beyond revert-to-baked. A session that goes somewhere and wants to
  come back two steps has to remember the numbers.

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

### 2026-09-11 — the dead stop on every basic attack
**Changed** A poke now slows you to 4.2 rather than rooting you (walk is 7.0, crouch 3.0,
guard 2.0). Committed moves still root — but they bleed the speed off over about four frames
instead of snapping to zero in one.
**Why** Reported as jarring. Every attack, poke included, hit a single line that set
horizontal velocity to zero, so the basic attack you throw constantly snapped you from a full
walk to a dead stop in one frame.
**Verdict** kept. Three options were on the table — much briefer, a slow, or no hindrance —
and the answer that generalises is **hindrance proportional to commitment**. No hindrance
removes the spacing cost of throwing a poke, and spacing is most of neutral. A briefer root
leaves two snaps bracketing the move instead of one. A slow keeps a real cost and is
continuous.

The separable half is worth noting on its own: the *snap* and the *rooting* were two different
complaints wearing one coat. Rooting a committed move is correct design; arriving at rooted in
a single frame is not, and fixing that costs about twenty centimetres of slide and no spacing
at all.

One thing the repository caught rather than me: printing the new speeds in the frame table
used `f32`, and the no-floats guard failed because that binary lives under `crates/sim/src`.
Formatted from the fixed-point raw value with integer arithmetic instead. The rule is worth
more than a convenient `{:.1}`.

### 2026-09-11 — hitbox wireframes that cannot lie
**Changed** The debug overlay (F1) now draws attack volumes as wireframe cylinders while they
are out, and every fighter's hurtbox all the time. The volume comes from a new
`sim::state::hitbox`, which the hit test itself calls.
**Why** Asked for, to see what is happening. The overlay that existed drew a small sphere and
was wrong in three ways at once.
**Verdict** kept. The three ways are the useful part:

It **rebuilt the box from the move table**, so it ignored the Champion's weapon form, which
multiplies reach — it drew a spear as though it were a sword. Now there is one function and
the hit test calls it too, with a test pinning that they agree at the boundary for every
class. An overlay that can drift from the rule it illustrates is worse than no overlay.

It drew a **sphere**, implying you could duck under or jump over an attack. The test compares
flat distance only; height is expressed entirely through `hits_crouching`. A cylinder says
that and a sphere does not.

It drew **only the attack radius**, but the threshold is that plus the defender's body radius
— so every hitbox looked smaller than it acted. Drawing both cylinders makes "do these touch"
mean exactly "does this connect".

One thing learned while checking it: at point-blank a move connects on the very frame its box
appears, so a landed hit is drawn dim for every frame you can see. The bright state is what a
*whiff* looks like. I spent a while comparing two screenshots of the same dim state against
each other before writing a test that settled it in a second — worth remembering that
screenshot forensics is the slow way to answer a question that an assertion answers exactly.

### 2026-09-11 — space stopped being clever
**Changed** **Space always jumps**, a vertical takeoff whatever your feet are doing. **Shift
plus a direction dodges.** Added an airdodge: shift plus a direction while airborne, once per
airtime, which wipes vertical speed rather than adding to it.
**Why** Reported from the sandbox: "I keep expecting to jump and not jumping." Space used to
mean *you move more than you otherwise would* — alone it jumped, with a direction it dodged.
That is a tidy sentence and it is wrong in the hand. You hold a direction almost all the time,
so the jump button mostly did not jump.
**Verdict** kept, and the lesson generalises past this one binding: **a button whose meaning
depends on what you happen to be doing anyway is a button you cannot trust.** "Space means
move more" was elegant as a rule and unusable as a control, and the elegance is what hid it —
it reads as one idea rather than two bindings, so it never got examined as two.

This is also the first settled decision to come back open. Dodge moving onto shift retired
"shift beats WASD when both are held", which was the rule that guaranteed a move-while-casting
option existed. Marked shifted in README §1 rather than quietly rewritten, with the knock-on
questions listed as open rather than answered: how move+attack gets differentiated now, and
aerials as variants of their grounded counterparts. Both are noted, neither is built.

Shift is now overloaded, and what disambiguates it is **a click**: shift with a click is the
stronger version of that attack, shift with only a direction is a dodge. The click is checked
first, so a committed move thrown while walking never comes out as a dodge — which is the same
class of bug as the one being fixed, and worth a test rather than a comment.

### 2026-09-11 — the movement pass
**Changed** A floatier, taller, **variable-height** jump; per-class air stats; Quake-style air
acceleration replacing direct velocity assignment; terminal velocity; and a per-move aerial
hang. Full hops now run 45–74 frames depending on class, against 36 for everyone before.
**Why** Verticality is meant to be part of the positioning game and a third of a second in the
air is a commitment that is over before you have read the situation you jumped into. Air
control was direct assignment, so you could reverse direction at the drop of a hat and a jump
cost you nothing.
**Verdict** kept. Notes worth keeping:

**The air-control formula is the whole thing.** `head_room = air_speed − (velocity · wish)`.
Because the budget is granted against the component of your motion you have *not* spent,
holding forward does nothing and strafing across your motion turns you without costing speed.
Nothing else needed to be added to make air movement expressive — the asymmetry falls out of
one dot product.

**A fixed strafe is a one-shot budget, not a rate.** With the aim held still you get about
0.9 m/s sideways and then nothing, which is roughly seven degrees of turn. The skill is in
turning the camera *while* strafing, which keeps redefining what counts as perpendicular so
the budget refills against the new heading. The first version of the test held the aim still,
measured seven degrees, and reported "there is no air control to express" — the test was
wrong, and finding out why is what made the mechanic legible.

**Capped air speed, unlike Source.** Unbounded gain became the genre there; here a player who
can reach any part of the arena from any other has deleted spacing. Capped at 1.5× the walk.
Whether that is the right number is open.

**Sustain, not cut-on-release**, for variable height. Both give you a height range. A cut makes
the short hop feel like the jump was taken away from you; a sustain makes the tall one feel
earned. Releasing is final, so the height is a decision rather than something to mash for.

One test bug worth remembering: the first "releasing jump is final" test ran for ninety frames
and caught the *next* jump, which a still-held button starts the instant you land. It was
comparing two jumps against one. Fixtures that run past a landing are measuring more than they
think they are.

### 2026-09-11 — the Oven
**Changed** Every tuned number in the game — 304 of them — is now runtime state, editable in a
palette on **F7**, with a bake button that writes `crates/sim/src/tuned.rs` and pushes on the
current branch. The hand-written constants in `tuning.rs`, the per-class air stats and the whole
move table now read from it.
**Why** Feel work is a loop and the loop was only as fast as a rebuild. In practice that meant
changing one number, waiting, and losing the comparison you were trying to make — and a session
of tuning ending as something to remember and retype rather than as a commit.
**Verdict** kept. Things worth recording:

**One representation carried the whole design.** Every knob is an `i32` — raw 16.16 bits for
fixed point, frames for frames, 0/1 for flags — with a `Unit` saying how to read it. That is
what made one store, one widget, one file format and one search index enough for three hundred
parameters. Had they each kept their own type this would have been a UI project.

**The 120 existing tests were the verification.** Converting 308 call sites from constants to
accessors is the kind of change where one mis-wired knob hides for weeks. Because the tests
exercise behaviour rather than values, a knob pointed at the wrong slot fails them immediately.
Every one passed on the first run after the refactor, which is the only reason to believe it.

**`sim` stayed float-free.** The obvious shortcut was an `f32` display helper inside the Oven,
and the no-floats guard caught it. Slider bounds are stored raw and the baked comments are
formatted with integer arithmetic instead; the `f32` conversion lives in the palette. An editor
is not a good enough reason to put the first float in a deterministic simulation.

**Mistuned peers now desync loudly.** Tuning is a rule rather than state, so rollback never
carries it — but two peers tuned differently would diverge silently and look like a netcode bug.
The tuning hash goes into `World::checksum`, which turns that into an error message.

Two small things cost more time than they should have and are worth remembering. `cargo fmt`
re-columnises generated arrays, so the file could never equal its generator until the arrays got
`#[rustfmt::skip]`; and it strips a trailing blank line, which left the file permanently one
byte different. Both were caught by the test that compares the committed file against a fresh
emit — which is exactly the test that catches a stale bake later.

### 2026-09-11 — the Oven kept stealing the mouse
**Changed** While the Oven is open the cursor belongs to you: no click captures it. Clicks
landing on the panel no longer reach the game, and typing in the search box no longer drives
the fighter.
**Why** Reported: every click in the palette re-grabbed the mouse for the camera, so changing a
number meant pressing Escape between each one — the exact opposite of the rapid loop the Oven
exists to provide.
**Verdict** kept. Two things worth recording:

**One bug, three symptoms.** The camera re-grabbing was the visible one. The same missing check
also meant a click on a slider *threw a poke*, and typing in the search box moved the fighter
and attacked — J, K and L are attack keys, so searching for "black spike" would have played a
whole sequence. Game and editor share one window and one keyboard, and something has to say
which of them an input belongs to; nothing did.

**The rule got simpler rather than smarter.** The first fix released the cursor only while the
pointer hovered the panel, which is more flexible and still wrong: the first click after F7 is
spent getting the cursor back, because the pointer is wherever the camera left it. "Open means
the cursor is yours" has nothing to learn and nowhere to oscillate. The keyboard keeps playing,
so you can drag a value and immediately feel it with W and J without closing anything.

### 2026-09-11 — aerials slow the rise instead of deleting it
**Changed** An aerial now damps vertical speed toward zero over its hang window rather than
setting it to zero, and gravity stays off for the whole window. A poke thrown in the air also
shoves you a little in the direction you are holding.
**Why** Reported: the dead stop was jarring, and cancelling the momentum outright is not the
right model. The control over jump height that attacking gives is worth keeping.
**Verdict** kept. The distinction is the useful part: **float and punch are separate knobs and
were fighting over one.** Zeroing the velocity gave punch at the cost of any sense of momentum;
leaving gravity on would have given float at the cost of impact. Damping the speed while holding
gravity off gives both, and they are now tunable independently — `Aerial hang damping` and the
per-move `Aerial hang`, plus `Aerial poke boost`, all in the Oven.

The shove belongs to the poke alone. A committed move that also repositioned you would be
strictly better than a poke, which is the sort of thing that quietly deletes a move from the
game.

**The first bake round-tripped correctly**, and the numbers that came back were a real
retuning: takeoff 8 → 17.7, gravity −24 → −42, air acceleration 11 → 14, air speed cap 1.5 →
2.03. A much snappier, heavier jump. Two things fell out of that worth recording. Appending new
scalars to the end of the enum leaves every existing index untouched, so a baked file survives
the addition with a two-line patch — worth knowing before ever inserting one in the middle. And
`a_jump_lasts_long_enough_to_do_something_with` failed, because the floatiest class went to 94
frames against a ceiling of 90. That is the harness working: a tuning decision, not a bug, so
the bound widened to 110 and the reason is recorded here. The assertion exists to catch a number
wrong by an order of magnitude, not to police taste.

### 2026-09-11 — moving between the Oven and the game
**Changed** F7 hands the cursor back; clicking the arena takes it again even with the Oven open;
Escape returns it to the Oven.
**Why** The previous rule — cursor always free while the Oven is open — fixed the slider problem
but made going back to play require closing the panel.
**Verdict** kept. Worth noting the shape: the first version was too clever (release only while
hovering the panel, which left the first click after F7 spent on getting the pointer back), the
second too blunt (never capture while open, which meant closing the panel to play). What was
missing from both was that *clicking the arena* is a different event from *clicking*. Once the
rule distinguishes those, it is three lines and there is nothing to learn.

### 2026-09-11 — the airtime ceiling came out
**Changed** Deleted the assertion that a full hop lasts between 40 and 110 frames. Four
assertions replaced it, one per thing a jump actually has to be: a short hop clears another
fighter, a full hop reaches platform-fighter heights, the rise gets you above head height within
12 frames, strafing carries you two body widths clear, and airtime exceeds reaction plus the
fastest poke.
**Why** The ceiling had no argument behind it. It caught an order-of-magnitude mistake and
otherwise went off whenever anyone tuned the jump — it fired once already this week on a real
tuning decision, and the response was to widen it, which is what a test with no argument behind
it always gets.
**Verdict** kept, and the general point is worth more than the change: **an assertion that
cannot say why it holds is a speed bump, not a guard.** "Between 40 and 110 frames" survives
exactly as long as nobody tunes anything. "The rise must clear a standing fighter's head inside
twelve frames" keeps meaning the same thing after any retune, because it is phrased in terms of
the fighters rather than the numbers.

The specific mistake in the old one is instructive: it was measuring *total airtime* as a proxy
for vulnerability, when vulnerability while jumping is about the time spent at head height where
you can be hit. Those come apart the moment gravity and takeoff speed both go up, which is
exactly what the last bake did.

Where it stands now, against a 1.8 m fighter:

| | Short hop | Full hop | Clears a head | Strafe across a jump |
| --- | --- | --- | --- | --- |
| Bulwark | 2.4 m (1.4×) | 4.1 m (2.3×) | 8f | 6.4 m |
| Champion | 3.7 m (2.1×) | 6.0 m (3.3×) | 6f | 8.2 m |
| Shadow Reaver | 4.9 m (2.7×) | 7.6 m (4.2×) | 5f | 9.4 m |
| Elementalist | 4.8 m (2.7×) | 7.4 m (4.1×) | 6f | 9.8 m |
| Blood mage | 3.8 m (2.1×) | 6.1 m (3.4×) | 6f | 8.3 m |
| Dual mage | 5.8 m (3.2×) | 8.7 m (4.8×) | 5f | 11.1 m |

### 2026-09-11 — the short hop was not short
**Changed** Added a **release cut**: letting go of jump while rising keeps only a fraction of
the remaining climb. Short hops went from about two thirds of a full hop to about a quarter.
Full hops are byte-identical — 4.1, 6.0, 7.6, 7.4, 6.1, 8.7 m before and after.
**Why** Reported: the minimum felt like half the maximum, and a quarter to a sixth would be
better.
**Verdict** kept, and this is a clean reversal worth recording. The sustain was chosen over a
cut deliberately, and the reasoning is still in the log: *"a cut makes the short hop feel like
the jump was taken away from you, whereas a sustain makes the tall one feel earned."*

That was a claim about the **feel of a small difference** and it was fine as far as it went. It
is simply the wrong tool for **range**. With only a sustain, the short hop is *exactly* the
sustain multiplier of the full one — the multiplier is the ratio, definitionally — so a
multiplier gentle enough to feel good leaves the floor at two thirds of the ceiling. There is no
setting of it that produces a quarter without dragging the ceiling along.

The cut lowers the floor and leaves the ceiling alone, because a full hop never releases while
rising. And because apex goes as velocity squared, the cut has *squared* authority over the
short hop: keeping 0.6 of the rise gives 0.36 of the height. That is why one number moved the
ratio from 0.6 to 0.25 without touching anything else.

Two assertions changed with it, and both were wrong rather than merely inconvenient:

**"A short hop clears another fighter"** is now false by design. A short hop in a platform
fighter is for throwing an aerial, not for crossing over someone — the full hop is for that. It
was replaced with a ratio bound (an eighth to a third) and a floor that an aerial must fit
inside the airtime, which is the thing that actually makes a short hop worth having.

**"A jump can be punished"** was measured on the short hop. A short hop being hard to react to
is *correct*: it is the fast, low-commitment option and platform fighters lean on exactly that.
It now measures the full hop, which is the committed one.

### 2026-09-11 — class pickers on the health bars
**Changed** A click-to-cycle button beside each player's health bar, showing that player's
class. F8 toggles them; on under `--dev`.
**Why** Tab cycled player one only, and switching player two meant a relaunch with `--p2`.
**Verdict** kept. Two notes.

They are the **third** thing sharing the pointer, after the arena and the Oven, and they route
through the same `UiFocus`. That is worth having built properly the first time: the question
"whose click is this" now has one answer rather than one per widget, so adding the next piece of
interface is a line rather than a bug.

And the manual's completeness test caught F8 before the commit — it failed with *"the game
handles keys this test does not know how to spell"*, then again with *"the manual never mentions
them"*. Exactly the two ways that could have gone stale, both caught by machinery rather than by
remembering.

### 2026-09-11 — the classes got the things that make them classes
**Changed** Five pieces, all of them the same piece: the moves that only one class has now do
what the kit documents say they do.

- **Fire pillar** (Elementalist) plants a pillar that outlives the move. It starts narrow and
  short and grows into a **wide base** and a **taller, only slightly wider column** — two
  volumes, two threats.
- **Structures** (Elementalist) are real things standing in the arena with a lifetime, not a
  list of coordinates. A fourth costs the first; the clock costs all of them eventually.
- **Black spike** (Blood mage) leaves a field that **drains and slows** anyone standing in it.
- **Uppercut** (Champion) leaps, and **takes whoever it catches into the air with it**.
- **Grapple** (Bulwark) actually grabs: the victim is pinned at arm's length and goes where
  the Bulwark goes until it ends.

**Why** They were all documented and none of them existed. The specials were, in fact,
*unreachable* — the gate on the special slot asked whether the fighter had a shield in hand,
which is true only of the Bulwark, so five of the six classes had a button that did nothing.
That is the kind of bug that hides behind a plausible-looking condition.

**Verdict** kept, with three notes worth keeping.

**Two volumes is the whole fire pillar.** A single growing cylinder makes "can I walk around
it" and "can I jump over it" the same question, and the answer has to be the same for both.
Splitting them lets the base spread wide enough to be a wall while the column stays narrow
enough that a jump is a real answer — at the cost of climbing high enough that the jump has to
be *timed*. Two numbers, two decisions.

**The slow is what makes the drain a wall.** Damage alone makes a puddle you step out of. The
slow makes leaving cost time, which is what turns it into something you put *between* yourself
and someone else. That was in the class doc as a single clause and it turns out to be the
entire mechanic.

**Growth is computed from age, never accumulated.** A pillar's radius is a function of how many
frames it has existed. An effect that grew by adding to itself each tick would drift every time
a rollback replayed those frames, and it would drift *differently* on each peer — a desync that
only appears under packet loss, which is the worst kind to find.

### 2026-09-11 — the special and the mechanic moved to Q and E
**Changed** The class special was shift + middle click and the class mechanic was middle click.
They are now **`Q`** and **`E`**.
**Why** Reported, and correct. Middle click is a scroll wheel on most hands.
**Verdict** kept, and the reason is bigger than the ergonomics.

Those two inputs *are* the class. Everything that makes a Bulwark not an Elementalist lives
there. Putting them behind the least reachable button, one of them under a modifier, said "these
are the optional ones" — and they played that way. A whole match could go by without either
being pressed, which means a whole match could go by without the class mattering.

The Oven now prints the binding in each move's family header — `Bulwark · Grapple [Q]` — because
the first question anyone asks while tuning a number is which button it belongs to, and
answering it in the header removes a lookup from every pass.

### 2026-09-11 — the special stopped working after about ten seconds
**Reported** Playing Elementalist: cast fire pillar nine or so times in a row and `Q` stops
doing anything. Raise a structure and you get nine more casts.

**Cause** Structures had been given a **lifetime**, and the fire pillar is gated on having one
out. Ten seconds after pressing `E` the structure expired and the class's main button went dead
with no feedback. Nine casts is simply how long ten seconds takes.

**Fix, by subtraction.** Structures were in the shared effects array. They do not belong there:
an effect expires on a clock, a structure is spent by raising a fourth. Two different rules, so
two different homes. Taking them out deleted the lifetime, the reconciliation pass that kept the
mechanic's slots and the array agreeing, and the eviction rule that could quietly eat a
structure when the board filled — about eighty lines of `state.rs`, all of it there to hold up a
decision that should not have been made.

**The rule worth keeping: reuse storage only when the lifetimes match.** "These are both
persistent things in the world" was the wrong axis. The right question was "what makes this go
away", and the two answers were different.

Eviction now reaches **your own** effects only. It could previously drop the oldest of anything,
which meant one fighter could delete the other's drain field by holding a button — not a
decision anybody made, just a consequence of a shared array with one rule.

Two regression tests, both phrased as the thing that broke: a structure survives fifty seconds
of doing nothing and the special still comes out, and twenty casts in a row never cost a
structure.

### Open — the autos are due a pass, as a set
Not a bug list, a rework. Recorded so the next pass starts from the measurement rather than
rediscovering it.

**Ranged autos have a hole in front of them.** A hitbox is a circle centred `reach` ahead, so a
move with more reach than radius cannot connect inside the gap. The Elementalist's Bolt is
reach 4, radius 0.7:

| Gap | Damage |
| --- | --- |
| 8.00 m | 0 |
| 5.67 m | 0 |
| 3.33 m | 45 |
| 1.00 m | 0 — standing on top of them |

Melee moves hide this because their radius covers their own reach. It is a property of the
one-circle hit test, not of any class.

**Deliberately not fixed in isolation**, because the whole set is changing:

- The Elementalist's autos should **push her structures around**, and more generally interact
  with persistent effects. That is a different shape from "a circle at range", so retuning the
  circle now would be work done twice.
- The melee classes' autos need **reshaping**, not just renumbering.
- Everyone's autos need their **timing** tuned.

Whether the hit test stays one circle, becomes a swept capsule from the fighter to the reach
point, or ranged autos become real projectiles is a decision for that pass. All three fix the
table above; they differ in what else they make possible.

> **Update, 2026-09-12: the Elementalist's third of this is done**, see the entry below —
> "aimed through terrain" rather than "push structures around", which turned out to be the
> more general shape and covers the effects interaction too. The hole-in-front-of-the-poke
> table above is untouched: this pass changed what the shot does when something is in the
> way, not the hit-test geometry against a fighter. The melee reshaping and the timing pass
> are both still open.

### 2026-09-11 — the mechanic button fired every frame you held it
**Reported** The Elementalist raises a structure once per frame while `E` is held; it should be
one per press.

**It was every class**, and each was broken in its own way. Measured, holding `E` for thirty
frames:

| Class | What you got |
| --- | --- |
| Bulwark | Shield pinned mid-throw — re-thrown every frame, so it never travelled and never planted |
| Champion | Form cycled thirty times; which one you end on is a function of how long you held |
| Shadow Reaver | Shadow placed and unplaced every frame; released on an even count, so no shadow |
| Elementalist | All three structures spent in three frames, stacked on one spot |

**Fix** The mechanic fires on the **press**. One bool on the fighter — was it down last frame —
and the edge is recomputed from the snapshot, which is what makes it survive rollback. A
renderer-side "just pressed" would report a press on every re-simulated frame and fix nothing.

**No cooldown, and this is the reason.** A cooldown was the obvious next thought and it is the
wrong tool twice over. It would be a second mechanism doing the cap's job — three structures is
already the resource, and a fourth already costs the first. And [the frame](README.md) says
abilities cost **frames**, not cooldowns; if raising a structure should cost commitment, the
honest form is recovery frames on the mechanic, which is one knob and belongs with the abilities
pass rather than bolted on here. The press edge is the whole of the reported bug.

Attack buttons still repeat while held, on purpose: mashing a poke is normal for the genre, and
a move's own recovery frames are the rate limit. The mechanic has none, which is exactly why it
needed the edge.

### 2026-09-11 — structures come up out of the ground
**Changed** A structure climbs out of the floor over about a quarter second instead of appearing
in the air. It is earth.

**How it stays honest under rollback:** the structure carries an `age` that the renderer reads.
Tempting to put the timer in the renderer, since this is pure decoration — but rollback jumps
the world backwards, and a renderer-side timer would disagree with the fighter it belongs to
after every correction.

The risk worth naming is that this is the *second* time structures have carried a counter, and
the first time it was a lifetime that silently killed the class's special. So: it **saturates**,
nothing reads it but the renderer, and there is a test asserting a structure still works after
fifty seconds — the counter cannot become a clock again without that test going red.

### 2026-09-11 — a test that keeps feel numbers in the Oven
**Changed** `crates/sim/tests/knobs.rs`. In the simulation, an `Fx` built from a literal or a
numeric `const` is a tuning value: it goes in the Oven, or in the test's exemption table with a
sentence saying why it is not. A second test asserts every exemption has a real reason.

**Why** The harness only helps with numbers it can see. One written into the code cannot be
tuned in the running game and cannot be baked from it either, so the only way to change it is a
recompile — which is the thing the Oven exists to avoid.

**It found thirty-two**, and one of them was a live bug: `arena.rs` collided against a hardcoded
body radius while the hit test used the Oven's knob, so tuning the body made fighters a
different size to walls than to attacks. The rest were ordinary invisibility — the Champion's
nine form multipliers (most of that class), the Bulwark's shield speed, range, damage and
knockback, the Reaver's leash, all three velocity decays, the Dual mage's entire meter. All now
live and bakeable.

**Verdict** kept, and the lesson is the same one the structure lifetime taught: the dangerous
numbers are not the ones in the table you are looking at, they are the ones written where you
stopped looking.

### 2026-09-11 — the first curve: structures hold, then erupt
**Changed** The structure rise is driven by a **cubic Bézier** rather than a straight ramp. Same
duration; it now stays barely out of the floor for the first half and finishes fast.

**Why** Reported after playing: it should be a short delay with visual feedback, then a fast
eruption. And more than that — this is the **hallmark the Elementalist should have**: long,
telegraphed startups; devastating follow-through; moderate frames afterwards, because the cost
was already paid at the front. Recorded in her kit.

**Verdict** kept. Two notes.

**A duration and a curve are different questions**, and the harness had only been able to ask
the first. The rise was already a knob and already the right length; it still read wrong,
because a quarter second spent sliding steadily and a quarter second spent waiting then bursting
are not the same event. That is the argument for curves in one sentence, and it is why the
original Oven plan said "numbers first, splines later".

**The solver is bisection with a fixed iteration count, deliberately.** Newton converges faster
but its step count depends on the handles, so the answer would vary with the shape — and a value
that depends on how hard it was to compute is not one two peers can agree on. Twenty halvings
puts the bracket below what 16.16 can represent, at the same cost every time.

The curve is four knobs in the Oven for now, not a widget. The data model is the part that had
to be right; dragging two handles is a palette feature and can come later without changing
anything underneath.

### 2026-09-11 — the camera could not aim
**Reported** Impossible to aim at the ground near your own character; very hard to push the
reticle further out than where it starts, because the angle goes shallow. And looking up needs
to reach nearly vertical, because verticality is part of the positioning game.

**Cause, and it was one line's worth of premise.** The aim point was pinned *flat*, at the
fighter's own height, a fixed distance ahead. Pitch moved the eye and never the aim point, so
the middle of the screen stayed in the same place whatever the mouse did — the only thing you
were steering was how obliquely you saw that one spot. Everything else followed from that.

**Fix** Screen centre is now simply the look direction. Then the two ends of the pitch range
get different rigs, blended:

*Down is close.* Looking at the ground means looking at the ground near you. A seven-metre arm
puts the ray on the floor before it reaches the fighter, so steep angles aimed **behind your own
heels**. The arm now shortens and the eye drops toward the fighter's head as pitch goes down;
at the bottom the reticle is just in front of their feet and they have risen to near the centre
of frame. That is the reported "mouse down should bring the aim closer", arrived at from the
geometry rather than bolted on.

*Up runs out of third person.* Walking the arm down behind the fighter reads well for forty
degrees and then stops: the arm is on the floor, the body is in the way, terrain shoves the
view. Past a handover angle the camera climbs into the fighter's eyes, the floor and geometry
clamps fade out, and the body stops being drawn — the Skyrim shape, and it turns out the reason
it works is that it is two answers rather than one stretched too far.

**Verdict** kept. Measured across the range: a shallow look down reaches better than six metres
further out than a steep one, and the steep one lands inside a metre and a half of the fighter.
Both were within centimetres of each other before.

**The reticle had to be rebuilt on the camera's own ray** — turned by however far facing lags
aim — rather than constructed from a distance ahead of the fighter. An independent construction
would have to re-derive the pitch, the eye lift and the shoulder offset, and the first one of
those to change would make the reticle lie. This is the second time that class of bug has come
up today: two things that must agree, each computing its own answer.

**Camera numbers stay out of the Oven, on purpose.** Tuning values are folded into the checksum
so mistuned peers desync loudly. That is right for anything deciding what *happens* and wrong
for anything deciding what you *see*: two people must be able to play each other at different
fields of view. The rig's numbers sit with the other per-player camera settings instead.

### 2026-09-11 — the camera sits back, and panning down no longer zooms
**Reported** Too close at all times; it should not zoom toward the character as you pan down;
neutral should rest ten to twenty degrees below the horizon; about 1.5× the setback with the
same height; the model's feet were being cut off and the periphery was too tight.

**Changed** The rig now orbits a point above the fighter, and that point is what does the work.

- **Arm ~11m**, up from 7. Fighters range from small to large, so seeing enough of the space
  matters as much as precision.
- **Neutral pitch rests 15 degrees below the horizon.** Level is the wrong neutral for a game
  played on the ground: resting slightly down puts the mark where the fight is and leaves the
  whole upward range for verticality, instead of spending part of it getting back to level.
- **Panning down no longer shortens the arm.** The aim comes in because the orbit centre drops
  toward the fighter, which is a different mechanism with none of the cost.

**The geometry is the whole argument.** With the camera on a sphere of radius `d` around a point
`h` above the fighter, pitched down by `θ`, the mark on the ground lands at

    h / tan(θ)

in front of them — `d` cancels. Two things fall out of that, and both are properties worth
having rather than accidents:

**Lowering the orbit centre is the right lever for "bring the aim in".** At full downward pitch
`tan θ` is large and `h` is small, so the mark collapses onto the fighter's feet. The previous
version hauled the camera in instead; it moved the mark, but only by trading away the view, and
it read as the game zooming on you for pressing down.

**Zoom and aim are independent.** A player who pulls the camera back to see more of a large
monster has not also changed where their attacks go. That makes distance an honest comfort
setting rather than a balance decision, which matters for a game meant to cover small enemies
and large ones. Asserted across a five-to-fifteen-metre range.

**Verdict** kept. Three assertions came out of it that did not exist before: the whole fighter
is in frame at rest (the reported chopped-off feet, stated as an angle against the field of
view), looking down never shortens the arm, and zooming does not move the aim.

### 2026-09-11 — Bellator is now Champion
**Changed** The class, its two documents, its Oven families and every reference in code.
`--p1 champion` matches; `--p1 bellator` no longer does.

**Why** Reported: the other classes have simpler, more ordinary names and Bellator did not fit.
Correct — and the interesting part is that the old name was not *wrong*. It is Latin for a
combatant, and the class descends from the old cities' professional duelling champions, so it
was accurate and well sourced. It was still the only Latin word on a roster of plain English
ones, which made it read as belonging to a different game. Accuracy is not the same as fitting.

**Verdict** kept. `Champion` says the same thing in the register the rest of the roster is
written in, and the lore already used the word for exactly these fighters.

Bellator survives in the front matter of [champion.md](champion.md) — which already carried
`formerly: Shifter`, so this is the second rename — and **the roster's naming is now open
across the board**. One name being out of place is a reason to look at all six: four of them
(Bulwark, Elementalist, Blood mage, Dual mage) are descriptions and two (Shadow Reaver,
Champion) are titles, and nobody has decided which register the game is in.

### 2026-09-12 — structures became stones
**Changed** A structure stopped being a marker and became a **solid**. It carries a velocity,
falls, shares the arena with the other stones and with both fighters, and costs something to
stand on top of while it comes up. New module: `crates/sim/src/stones.rs`.

**Why** Three things asked for at once, and they turn out to be one thing: stones that push
each other, stones a fighter can stand on, and a stone that warns you before it hits you. The
first two are the same rule — *whatever is in the way gets moved, along whichever axis is the
shorter way out* — which is the least-penetration rule the arena already used for walls. A
stone is a wall the Elementalist made, so it should behave like one, and `arena::resolve` grew
a body size rather than acquiring a second copy of itself with different numbers in it. The
last time a body size was written twice, attacks and walls disagreed about how wide a fighter
was, and that is the bug the knob test exists to catch.

**The eruption's own climb is what throws things.** The rise curve holds a stone barely out of
the floor for half its rise and then bursts: over the last three frames the top climbs at about
25 m/s, which is faster than anything else on the field moves. A stone or a fighter standing on
that top keeps a fraction of it (`Lift kept`, 0.4) when the burst ends, so a stone raised
underneath another pops it about a metre clear and a fighter standing over one is carried to
about 2.9 m against a 2.2 m full hop. Nothing was invented to make that happen — it is the
number the curve was already producing, handed on instead of discarded.

**Off centre throws it sideways, and that is the only source of horizontal speed today.** A
boulder coming up under the *edge* of another flips it clear rather than balancing it, in
proportion to how far off centre it sits. Without that, "a stone knocked into another" would be
a rule with nothing in the kit able to trigger it — the moves that launch stones properly are
not built. With it, an off-centre raise throws one stone into the next and the knock is
reachable in play.

**Two slows now exist, so a slow had to become a strength rather than a flag.** `Player::slowed`
was a frame count, and the multiplier lived on the drain field that set it. The churn under a
rising stone is a *warning* — slight, 0.8 — and a drain field is a *wall* — 0.45. The strongest
slow on you wins rather than the two compounding: two multiplied slows freeze you, and every
new source would quietly make the last one worse.

**Numbers.** Eruption begins at 0.15 of the rise, which lands on frame 7 of 14 — half telegraph,
half burst, matching how the curve was described when it was added. Damage 40 and a 20-frame
stagger: the stagger is the punishment, the damage is there so ignoring a telegraph is never
free. It catches each fighter once, marked per victim on the stone, because a stone erupts once.

**Verdict** open — played only through the test harness and a headless capture, which shows the
lift and the throw doing what they should. The numbers most likely to be wrong are `Lift kept`
and the eruption damage, and both are in the Oven under a new **Stones** family with the
structure knobs that were scattered through *Effects*.

**Still open.** Raise places a stone 2.5 m *ahead*, so "cast beneath yourself to launch into the
air" from the kit still has no input — the lift works, the targeting for it does not exist.
A stone lifted off centre rides up on the shoulder of the one below rather than sliding off it,
which is the same thing the arena's platforms do and may want revisiting when stones are being
thrown around in earnest.

### 2026-09-12 — the Ridgeback, and measuring a monster fight
**Changed** A creature in the arena, the machinery for standing on one, and 203 new knobs.
`H` in game, `--hunt` on the command line. See [monsters.md](monsters.md).

**Why** The frame has always said "coop against monsters" and there were none. The specific
thing worth finding out is whether a fight against something you can *climb* works at all: the
appeal is obvious and the failure modes are not.

**Verdict** kept, and the interesting part is not the creature.

**A fight report is worth more than a tuning session.** `cargo run -p hunt --bin fight` plays a
scripted hunter and prints the dozen numbers a good fight needs -- how much of what the creature
throws can be answered on sight, how long the openings are, how varied its moves are, how long
anyone stays on its back, and how many hits landed that could not have been read. Every real
problem in the first week came out of that output rather than out of playing it, including one
that playing it could not have found: a fixed-point overflow that made *every buck in the game*
do nothing, silently, because a squared 16.16 value saturates just past 181 and accelerations
run into the hundreds. The symptom was a single number reading wrong — `thrown off: 0`.

**The one measure that matters is "unanswerable hits".** A monster can score well on every other
line and still feel cheap, and when it does it will be because of damage the player had no way
to avoid. Defined as: an attack too fast to answer on sight *and* reaching further than its own
volume extends, which is what happens when the animal walks into you during a startup shorter
than human reaction. It is zero, and it is zero because the measure was written before the
tuning rather than after.

**Grip is one comparison and it replaced a whole category of authoring.** A rider comes off when
the surface under their feet accelerates harder than they can hold on through. Nothing tags a
move "this one throws people": the tail sweep throws whoever is on the tail and does nothing to
someone on the shoulder, the middle of the spine is calm because it is near the axis the shake
turns about, and none of that was written down anywhere. The tuning that followed was about
where `grip` sits relative to the shake at the work spot, and that single number moved the fight
from "the back is a safe room" to "the back is a wager" and back twice.

**Open, and only play can answer them:** whether the ride reads as the fight or as a way to skip
it; whether a person fights the front end enough to break a leg, which the scripted hunter never
does; and whether the creature having a different move set depending on where you stand is the
feature it looks like or a way to switch its moves off.

### 2026-09-12 — the characters get a skeleton
**Changed** The six free-floating boxes became sixteen joints hung off each other: two members
per limb, a spine that bends and a chest that turns on top of it. A pose is now fifty-one
angles and no positions at all. Stride length moved into `tuning.rs` as four constants
(`WALK_STRIDE`, `RUN_STRIDE`, `CROUCH_STRIDE`, `STRAFE_STRIDE`), and the simulation gained six
fields that exist only for the renderer.

**Why** Five clips is the most the old model could carry. Nothing held an elbow to a shoulder,
so every pose re-derived where the hand went, and a pose authored for one body could not play
on another — which meant six classes would have meant six sets of everything. Joint angles are
proportion-free, so one authored clip is now correct on the Bulwark's heavy frame and the Dual
mage's slight one without being re-authored.

The stride constants are not free choices and it is worth writing down why. A leg is 0.87 m
long and a hip is 0.86 m off the floor at contact, so a foot can be at most about 0.34 m ahead
of the hip before the leg runs out. Stride length follows from that, not the other way round:
1.10 m per cycle walking, 2.70 m running, and a quarter off sideways because a leg swung
sideways runs out of *hip* long before one swung forward runs out of leg. The first attempt
used 1.35 m and 2.45 m, picked by eye, and the IK quietly refused to reach on every contact
frame.

**Verdict** kept, and one bug it exposed is worth the entry on its own. The walk cycle's phase
was `distance walked / stride length`. A stride is longer at a sprint than at a walk, so the
moment the speed changed the phase jumped by whole cycles — twenty-nine metres into a match,
turning from a run into a diagonal moved a knee ninety degrees in a single frame. A phase has
to be **integrated, not divided**: the simulation now accumulates `speed / stride` each tick.
The same class of mistake is available anywhere a ratio is used where an integral belongs.

**Open:** whether seven metres per second is the right free movement speed at all. At that
speed a 1.8 m body is sprinting, and the "walk speed" knob is named for something the game
does not have — the only walk in it is the guarding one at two. Nothing is wrong with a game
where neutral is a sprint, but it should be on purpose.

### 2026-09-12 — the Elementalist's auto reads what it is aimed through
**Changed** Bolt (`L`) checks, once, on the frame it fires, what lies between the caster and
her own short hit-range against a fighter — along a line much longer than that range, since
the whole point is finding terrain the poke could never reach on its own. Aimed through a
structure, the structure blocks it and gets kicked forward instead, fast at first and dying off
over the back quarter of its travel; the bolt never reaches past it. Aimed through a fire
pillar, the pillar does not block anything — it charges the same shot, which goes on to hit as
normal, harder. Aimed through neither, the plain poke, unchanged. New module-level functions in
`stones.rs` and `effects.rs` (`first_along_shot`, `first_fire_pillar_along`, `kick`), one shared
geometry helper (`math::ray_hits_flat`), and two new transient flags on `Player`
(`bolt_fire`, `bolt_blocked`) that `resolve_hit` reads and resets every time the move comes out,
so a decision from one shot can never leak into the next.

**Why** Recorded above as open: "the Elementalist's autos should push her structures around,
and more generally interact with persistent effects." Pushing structures around and reading
effects turned out to be the same mechanism once phrased as "what is this shot aimed through" —
a single ray query answers both, ordered by whichever the line reaches first. Structures
physically stop things in this game already; pillars are a hazard you choose to walk into, not
a wall; making the bolt respect that distinction for free was the reason to build the query as a
priority-ordered aim-through check rather than two independent special cases.

**Verdict** open — implemented and covered by tests, not yet played. Three things worth
recording regardless of how it lands:

**A fast solid defeats the ordinary "don't let bodies overlap" rule, and the fix had to move
upstream of it.** The obvious place for "did the kicked stone touch a fighter" was beside the
existing churn/eruption checks in `stones::touch`, which runs *after* both fighters have moved
for the frame. It never fired: `resolve_body` — the rule that keeps a fighter from ever
penetrating a stone — had already pushed the stationary fighter back out to exactly `reach`
every single frame, since a fast-approaching solid and a body-sized keep-out zone produce
exactly that shove. The proximity check was reading a distance that the collision rule had
already restored to the boundary before it ever got a turn. The fix is a second pass inside
`stones::step`, which runs *before* anyone moves, comparing the stone's freshly-computed
position against where the fighter stood at the *start* of the tick — catching the sweep before
the push-out rule gets a chance to keep the two apart. The general lesson: a check phrased as
"is A closer to B than X" is not safe from a rule elsewhere in the same tick whose entire job is
making that never true.

**Damage is speed relative to the target, reusing the number `knock` already computes between
two stones**, rather than the stone's raw speed. A closing-speed formula is the one that already
generalises to "the target is also moving" without a special case for it, and it is also why the
new deceleration is a *ceiling* on the stone's own speed rather than a directly-assigned value:
collisions with other stones on the way should be allowed to cost it further, never hand
speed back.

**The deceleration is keyed to distance travelled, not frames, mirroring the structure rise
curve's own reasoning.** A stone parked against a wall partway through its travel should not
keep coasting at full speed just because the clock is still running, and a straight frame-based
timer would do exactly that. Progress is measured against how far it has actually gone.

**Open questions**, none of them blocking a prototype:

- Whether `bolt_aim_range` — set well beyond the poke's own hit-range on the theory that a
  ranged shot's aim should reach further than its point-blank kill zone — is anywhere close to
  the right number. It is the first knob in this pass nobody has played against yet.
- Whether the kicked stone should still deal its ordinary "stone lands on you" cover-collision
  cost on top of the knock damage, or whether the two should be mutually exclusive. Right now
  both can happen in the same encounter and nothing has tried to break that.
- Melee autos and the timing pass, unchanged from the note above — this closed one third of
  "the autos are due a pass, as a set," not the set.

### 2026-09-12 — abilities go where the crosshair is
**Changed** Area abilities stopped appearing a fixed distance straight ahead and started
landing where the player is pointing. Pitch went on the wire to make that possible, the camera
was rebuilt around it, and the crosshair stopped moving.

**Why** The old rule — spawn at `pos + facing × reach`, flattened to the floor — meant the only
way to place a fire pillar or a stone anywhere was to walk there. For a class whose whole
identity is authoring terrain, that is the wrong verb.

**The rule.** Follow the line the player is looking along, out from the point abilities come
out of, and stop at the first of the terrain or the edge of that ability's reach. One sentence,
and every case falls out of it: a spot inside your reach is placed exactly; a spot past it goes
as far along that line as it can; the sky, for something that comes out of the ground, gives
full reach flat ahead.

**The reach sphere is the interesting half.** Tracing to terrain alone lurches — aim a hair
over the lip of a platform and the hit jumps from two metres to the far wall, so a fraction of
a degree swings the ability across the arena. Stopping at the reach bounds that jump to the
ability's own range, which is the most it could ever have meant. It also gives both halves of
what a player wants at once: aim at the ground to pick a *direction*, or aim at a spot inside
your reach to pick a *place*.

**The ray starts at the fighter, not at the eye**, which is the opposite of what a
third-person shooter does. Two reasons, and the second is the binding one. It makes the aimed
line and the travelled line the same line, so pointing at the floor short of someone gives the
ray that passes through them. And the eye cannot be in the simulation at all: camera distance
is a per-player setting and the follow position is smoothed, so solving the aim from there
would have two peers at different zoom levels placing a pillar in different spots with neither
of them wrong.

**Three passes on the camera, and the first two were wrong.**

*Orbit the cast origin.* If the eye sits exactly on the ability's line then the screen's centre
ray **is** that line, which is exact and needs no machinery. It also puts the camera at chest
height: the horizon climbs to the top of the frame and you cannot see the arena you are
fighting in. Zero parallax is not worth a view from a fighter's sternum.

*Leave the camera on the look axis and move the reticle.* Honest — the mark is drawn where the
ability actually lands, sliding off centre by the parallax. Reported immediately, and correctly:
*"a jumping crosshair would feel really really bad, like the player has no real control."* The
reticle is the one thing on screen a player is deliberately holding still.

*Point the camera at the aim point.* What shipped. The crosshair is pinned to the exact centre
of the screen and the **view** absorbs the parallax instead, as a few degrees of pitch. The eye
is then free to sit where it frames the fight best.

**Which turned out to be directly behind and well above.** Reported: *"the camera appears to be
behind and to the right… it feels quite cramped because the character model is almost right on
the crosshairs no matter where you aim."* Both halves were real and they have different causes.
The over-the-shoulder slide turns the whole view once the camera points at the target, so `W`
stops walking up the screen — it is gone. And the fighter was on the reticle because the eye was
at 1.4 m over their feet: the aim point and the fighter are both on the ground with the fighter
nearer, so how far apart they sit on screen is a function of eye height. Lifted to 4 m, the
fighter's head rests about seven degrees below the crosshair.

**The handover moved to the horizon.** It used to start forty degrees up and finish at the pitch
limit, which left a wide band with the arm dragging along the floor behind the fighter. Now the
climb starts the moment the aim crosses the horizon and is complete half a radian above it, and
the body **fades** rather than popping out at a threshold — one continuous motion, the fighter
rising to the middle of the screen and thinning out as they get there.

**Numbers.** Cast height 1.25 m — chest, not eyes, so the shot does not read as first-person
fire from a third-person body. Orbit lift 1.4 → 4.0. Neutral pitch 0.26 → 0.10, which lands the
resting aim about twelve metres out: the mark sits `cast_height / tan(pitch)` ahead, and the
origin of that ray dropped from a point above the fighter's head to their chest. Raise's reach
2.5 → 4 m, because 2.5 m stopped meaning "where the stone goes" and started meaning "how far you
may aim", and 2.5 m is barely enough room to aim in.

**Verdict** open — played through the harness and four headless captures: resting framing,
aiming down at your own feet, aiming up into the fade, and the reticle pinned through all of
them. The number most likely to be wrong is `sky_full`: a twenty-degree glance upward currently
costs you sight of your own fighter, which may be too eager for a game with this much
verticality.

**What did not change.** Melee swings are still flat, at `pos + facing × reach`. A sword is a
body moving, and pointing the camera at the floor should not put the blade there. Only the moves
that *place* something are aimed.

**Still open.** A stone stands a whole body height and abilities come out of the chest, so from
the ground you are always looking at a stone's *side* and never at its top — stacking by aiming
needs you to be above the cap. That is honest geometry rather than a bug, and it may still want
an answer.

### 2026-09-12 — the camera became a prescription
**Changed** The rig stopped being a set of offsets and became a set of **zones in the vertical
aim angle**, each one stating where the fighter should appear on screen. Every boundary and
every percentage is in the Oven under a new **Camera** family. See
[controls.md](controls.md#the-camera-is-prescribed-zone-by-zone--settled-2026-09-12).

**Why** Reported, and the reason is the valuable part: *"I think this is becoming a problem
where the intent with the camera is not clear between threads so we keep causing reversions."*
Three passes in two days had each moved the camera for a good local reason and undone something
the last one bought. A rig described as "arm length, orbit lift, shoulder offset" cannot be
argued with, because none of those is a thing anybody wants — they are means. Stated as *where
the fighter sits in the frame at each angle*, the intent survives the next change, and a
disagreement is about a number rather than about what the camera is for.

**Closed form, and the geometry hands it over.** Asked for "solved per frame" the first version
did nested bisection, which was rejected on the spot and rightly: *"solved per frame needs to
mean closed form solution, to be clear."* It turns out there is one, and it is pretty. Each
condition is "see these two points a given angle apart", and the set of places from which a
segment subtends a fixed angle is a **circle through its two ends** — the inscribed angle
theorem, the same one that says every angle standing on a diameter is a right angle. Two
conditions, two circles, and the eye is where they cross: subtract them for the radical line,
intersect that with either circle, take the root that is behind the fighter. Thirty-odd
operations, exact, and nothing to bake.

It also fixed a real error. The bisection's inner loop was hunting for the distance that makes
the fighter the right size, and the closed form for that is one line — `R = k·cos ε + body·sin ε`
— which disagreed with the small-angle estimate the rest of the design had been sketched
against by nearly two metres at steep angles.

**The camera left the desync checksum, and that is now safe.** Camera numbers were deliberately
kept out of the Oven when it was built, because everything in it is folded into the checksum and
two people must be able to play each other at different fields of view. They can go in now, in
their own family that `hash` skips, and the reason is this week's other change: **aiming stopped
going through the camera.** The aim is solved from the fighter's own cast origin, so where the
eye sits changes nothing about where an ability lands. A camera knob is a personal setting in a
way it could not have been a month ago.

**What the geometry insists on.** Two things came out of the numbers rather than out of the
spec, and both are worth knowing before tuning.

*The neutral zone is two cameras.* The crosshair's mark on the ground sits `cast height /
tan(pitch)` ahead — about 7 m at −10 and 1.2 m at −45. Holding the fighter at a fixed 5% of the
screen while the mark sweeps in that far swings the eye from 51 degrees of elevation at 6.9 m to
85 degrees at 2.6 m, which is a normal third-person arm at one end and almost directly overhead
at the other. Nothing is wrong; it is what "keep the fighter still while the crosshair comes in"
means. The lever is the zone's steep boundary.

*The last few degrees of the look down are degenerate.* At the bottom of the range the crosshair
is already at the fighter's feet, so "put the feet on the crosshair" is satisfied by any camera
at all — and asking for it *exactly* demands an eye in line with them, which is an eye on the
floor. A least-elevation knob is what stops the rig chasing that, and it only applies in that
zone: at level the eye has to come down almost beside the fighter to keep their head just under
the mark.

**The distance setting survived with a new job.** The rig has no free distance any more — where
the eye goes is decided by where the fighter has to land — so `F5`/`F6` scale how much of the
screen the fighter fills. Pull back, smaller fighter, which is the same wish.

**Verdict** open. The shape is what was asked for and the waypoints are pinned by tests rather
than by screenshots, which is the point of the exercise. The number most likely to want moving
is the neutral zone's steep boundary, for the reason above.

### 2026-09-12 — the camera moves on a fixed sphere

**Changed** the eye rides a sphere of fixed radius centred on the fighter's feet, and the only
thing the rig solves is where on it. The old second condition — "the fighter fills a fixed
share of the screen" — is gone, and with it the `Head, neutral` knob; `Sphere radius (m)` takes
its place and starts at 6. The eye elevation bounds moved to 60/60.

**Why** the previous solve met two conditions at once, so it had to move the camera in and out
to do it: 6.9 m at −10 down to 2.6 m at −45. A camera that dollies while the player is only
steering is the thing that reads as "the camera is doing something". Fixing the radius removes
the freedom that was being spent on it.

**The solve got simpler, not harder.** One unknown and one condition. With the eye at
`R(cos e, sin e)` and the feet at the centre of that circle, asking for the fighter to sit a
given angle below the crosshair is `A·cos(e) + B·sin(e) = C` — the `R²` terms cancel — which is
the standard shape that collapses to a single cosine. Two roots, an `acos` either side of a
lead angle, and the camera is the flatter one that is genuinely behind the fighter. The
two-circle radical-line construction it replaces was correct but was doing twice the work for a
question that only had one unknown in it.

**What the geometry insists on, which fixing the radius did not fix.** The crosshair sits where
the aim ray meets the ground, which walks in from about 7 m ahead at −10 to 1.2 m at −45. The
camera is trying to open a gap between the fighter and that mark, and the aim is closing it. On
a sphere of radius `R` the widest *any* eye sees the pair is `atan(mark / R)`, so:

| Aim | Mark ahead | Most the sphere can open | Where the feet land |
| --- | --- | --- | --- |
| −10 | 7.1 m | over half the screen | 5% |
| −15 | 4.7 m | over half | 6% |
| −20 | 3.4 m | over half | 15% |
| −25 | 2.7 m | 40% | 21% |
| −30 | 2.2 m | 33% | 26% |
| −45 | 1.2 m | 19% | 35% |
| −85 | 0.1 m | 2% | 49% |

Two different limits bite down that list. Above about −20 the sphere could open more than half
a screen, but only from an eye swung past vertical, so what actually stops it is the **eye
ceiling** at 60 degrees. Below that the **sphere itself** is the limit and the ceiling is
irrelevant. Either way the 5% waypoint holds to about −15 at six metres and then the fighter
rides up the screen whatever the rig does. This is not the fixed sphere's fault — the previous version bought the
same waypoint by dollying to 2.6 m, and *no* fixed radius satisfies the zone as written: 5% at
−45 wants 2.5 m, and the fighter filling only a fifth of the screen wants about 9 m. The zone
as prescribed asks for both.

**The floor zone turned out to cost nothing.** Its waypoint is "pan until the camera points at
the fighter's feet", and the camera already points at the crosshair's mark, which is itself
sweeping onto the feet as the player looks down. So the *view* pans onto them with the eye
staying exactly where it is. Setting the elevation floor equal to the ceiling makes the whole
look-down range one unmoving camera, and the fighter walks from 5% to 49% of the screen without
the eye travelling a centimetre.

**That also removed a snap.** With the two bounds apart, the last degree and a half of look-down
used to swing the eye about five metres, chasing the final one percent of "feet exactly on the
crosshair" — a waypoint that is within a hand's breadth of satisfied from anywhere by then.
`the_camera_never_jumps_as_the_aim_sweeps` now sweeps four hundred samples across the whole
range and holds the eye to a fifth of the sphere per degree; the fastest thing left is the
handover into the fighter's head, which is a designed sprint.

**Verdict** open. Measured and looked at, at −10, −27 and −45: low and small, then clear of the
reticle, then mid-screen with the crosshair on the chest, which is what aiming a metre from your
own feet has to look like. The number most likely to want moving is the sphere radius, and the
table above says what it buys.

### 2026-09-12 — the eye has to keep moving, and the floor zone needs its own reason to

**Changed** sphere 6 m → 4 m; eye elevation bounds 60/60 → 40/89; and the floor zone now walks
the eye down toward its floor itself, easing from the framing's own answer at −45 to lying along
the fighter's feet at −85.

**Why** reported: past −45 the camera only panned. It was meant to keep working around the
sphere *as well*, the pan being what the floor zone adds on top. Two separate things were
stopping it, and only one of them was a number.

**The one that was my mistake.** Setting the elevation floor equal to the ceiling pinned the eye
outright. The previous entry argued that was elegant — the floor zone's waypoint is met for free,
because the camera points at a mark that is itself sweeping onto the feet — and the waypoint
*is* met. It is not the whole zone. A camera that stops moving while the player can feel they
are still turning reads as the rig giving up, and no amount of the framing coming out right
makes up for it.

**The one that was geometry, and is the useful finding.** From a sphere of radius `R` the widest
any eye sees the fighter and the crosshair's mark is `atan(mark / R)`, and the mark comes in from
about 7 m ahead at −10 to 1.2 m at −45. Ask for a wider gap than that and the solve **saturates**:
it parks the eye against the top of the sphere, where it answers nothing. Saturation is the
failure mode to watch for, and it is invisible to a framing test — a parked eye that happens to
be in the right place still frames correctly. It is why `the_neutral_zone_works_the_eye_around_
the_sphere` exists alongside the framing test, and why the floor zone needed a mechanism rather
than a number: below −35 at any playable radius, the framing has nothing left to say.

**So the floor zone walks the eye down itself.** It eases from the framing's own answer at the
handover — so there is no seam by construction — to the elevation floor at the bottom of the
range. The pan rides on top: the camera is pointed at the mark, the mark is sweeping onto the
feet, so the view comes round to the fighter while the eye comes down. Two motions, which is
what the zone was always described as.

| Aim | Feet | Eye elevation |
| --- | --- | --- |
| −10 | 5% | 41° |
| −20 | 5% | 58° |
| −27 | 5% | 73° |
| −45 | 22% | 89° |
| −55 | 32% | 77° |
| −65 | 39% | 65° |
| −85 | 48% | 40° |

**Why four metres and not less.** The neutral zone is reachable down to about −28 at 4 m, −45 at
2.5 m, and the whole prescription lands exactly at 2.1 m. But the radius is also how big the
fighter is drawn: at 2.5 m a 1.8 m body fills three-quarters of the frame and the crosshair sits
buried in their chest, which is a worse version of the complaint that started all this. Four
metres puts the feet at 5% and the head at 28% at the aim the rig rests at — which is the 5%-to-
25% framing originally asked for — and holds it over the band actually used for aiming at people
and at ground a few metres ahead.

**Verdict** open. The two tests that matter are the eye ones: at least 25 degrees of sweep around
the sphere across the reachable neutral zone, and at least half a radius of travel across the
floor zone while the feet climb a fifth of the screen. The pinned tuning fails the second with
"the eye only travelled 0.00 m", which is exactly the report.

### 2026-09-12 — the camera is an orbit with a tilt, and nothing is solved

**Changed** the whole rig. The eye no longer satisfies a condition; it is placed by a
subtraction. Each zone names a sphere — centre, radius, tilt — and the eye sits at
`tilt - pitch` around it. Sphere 7 m below the horizon, contracting to 0.2 m at the head above
it. The camera points down the look axis rather than at the aim point.

**Why** three attempts at solving the eye from the framing all ended the same way: the condition
becomes unreachable partway down the range, the solve saturates, and the eye parks against a
limit where it stops answering the mouse. Reported twice as "the camera stops moving", and both
times the fix I reached for was a number.

**The thing I had backwards.** The sphere is centred on the thing being framed, so the line from
the eye to that centre *is* the radius the eye is standing on — whichever way round the sphere
it has walked. Turn the view up off that line by a fixed angle and the centre lands at a fixed
place on the screen, at every eye position, for free. **The framing is a consequence of the tilt,
not a condition on the position.** Which leaves the position free to be the mouse, directly, at
one degree of orbit per degree of mouse, with nothing that can saturate. Every previous version
of this file describing a solve was solving a problem that did not need to exist.

**What it costs, and it is not small.** The camera no longer points at the ability's landing
point — it points down the look axis, which is what the prescription asks for. So the crosshair
and the spot a grounded ability lands on are no longer the same place:

| Aim | Crosshair marks | Ability lands | Apart |
| --- | --- | --- | --- |
| −10 | 18.0 m ahead | 7.1 m | 10.9 m |
| −27 | 6.9 m ahead | 2.5 m | 4.4 m |
| −45 | 4.4 m ahead | 1.2 m | 3.2 m |
| −70 | 1.3 m ahead | 0.5 m | 0.8 m |

The two rays are parallel — same direction, different origin — so the miss is the eye's offset
from the chest, and it only closes where the sphere has contracted onto the head. Closing it
properly means the aim tracing from the *eye* rather than from the chest, which makes the
camera's geometry simulation state: it would go into the checksum, peers would have to agree on
it, and the personal distance setting could not scale it any more. That is an architecture
decision rather than a tuning one, so it is written down here rather than taken.

**Verdict** open, and deliberately not merged on its own. The camera is exactly the prescription
and measures out at every waypoint; the aiming correspondence is the open question.

### 2026-09-12 — the crosshair is the aim, so the eye is in the simulation

**Changed** the aiming ray starts at the eye instead of at the fighter's chest, and
`sim::camera` places the eye, so the camera's geometry is simulation state. Camera knobs are in
`oven::hash` now. Camera distance stopped being a personal setting.

**Why** the previous entry shipped a camera that framed exactly as prescribed and left the
reticle sitting four metres from where a grounded ability actually landed. Both rays had the
right *direction* and different origins, so they never converged — parallel lines do not meet.
The targeting rule had already been written down and settles it: work out what the player is
pointing at, then draw the line from the ability's origin to it.

**Measured, aimed at open ground, reach 20 m:**

| Aim | Crosshair | Lands | Apart |
| --- | --- | --- | --- |
| −10 | 18.0 m | 14.0 m | 4.0 m — the range sphere, correctly |
| −20 | 9.14 m | 9.14 m | **0.00 m** |
| −27 | 6.88 m | 6.88 m | **0.00 m** |
| −45 | 4.42 m | 4.42 m | **0.00 m** |
| −85 | 0.00 m | 0.00 m | **0.00 m** |

Exact wherever the crosshair is inside the ability's reach, and clamped to the reach beyond it,
which is the rule as written.

**What it cost, which is not nothing.** Camera numbers decide where abilities land, so they are
gameplay numbers: hashed, shared, and a peer tuned differently now desyncs loudly instead of
quietly placing things somewhere else. The personal distance setting had to go with it — it
scaled the sphere, the sphere is the eye, and the eye is the aim. Field of view survived only
because the *framing* is measured against a tuned field of view of its own, so a player's choice
changes what is projected and never where the eye is.

**Two things moved that were not asked for, and are worth knowing.** Aiming down brings the
reticle in more slowly than it used to, because the ray starts seven metres behind and above the
fighter rather than at their chest: the reticle reaches their own feet at about −85 rather than
−45. And **a stone can no longer be raised directly underneath another** — pointing at where its
base would be means pointing at the stone, and a stone you point at is a surface you land on
top of. Aimed at its foot the new one comes up against its near face and still shoulders it
aside, which is the interaction the stone physics was built for; it is the dead-centre lift that
is now out of reach.

**Verdict** open. The contract the whole aiming pass exists for is exact again, measured rather
than argued, and the camera keeps the orbit from the previous entry unchanged.

### 2026-09-12 — the Blood mage's kit is its mechanic now

**Changed** the whole of the class's implemented kit, and the first implementation of the
mechanic it has been described by since it was written down.

| Key | Was | Is |
| --- | --- | --- |
| `LMB` | Rend, a melee poke | **Bloodletter** — a blade out and back, cutting on both passes |
| `Shift+LMB` | Black spike | **Rend**, moved down and given committed weight |
| `Q` | Reaper's debt | **Grasp** — four arms out in a cone that converge, rooting on all four |
| `E` | nothing at all | **Black spike**, at 9 m instead of 2.5 and a 30-frame cast instead of 18 |

And every one of the four now has a **health cost** and a **leech percentage** in the move
table: 15/40% for the auto up to 120/30% for the spike, against a thousand-point bar.

**Why** the class was described as "everything costs health and the good outcomes give it
back" and not one line of that existed in the simulation. Its abilities were free, they
returned nothing, and the one thing on the roster that was supposed to be a resource loop was
four ordinary attacks with a red colour scheme.

**Three things were broken rather than missing, and they are worth separating out:**

- **The spike drained nobody in a hunt.** Effects were applied to fighters and the creature
  was not one, so a Blood mage hunting alone put a spike in the ground, drained an empty patch
  of arena and got nothing. Half a kit doing nothing in one of the game's two modes, invisible
  because the versus tests passed. Fixed for every effect, so the fire pillar burns the
  creature too now — it did not before either.
- **Friendly fire was on for hazards.** The same fix opened it: a drain field was about to
  become the one thing in the game that could kill a team-mate. Effects now go through the
  same "is there a creature" condition direct hits already use, rather than a second flag that
  could get out of step with the first.
- **The spike had no spike.** It was drawn as a twelve-centimetre stain on the floor, which is
  a thing you find out about by standing in it. It is a cone standing in a disc now, at
  `spike_height`, and the field is tested as a slab of that height rather than as an
  infinitely tall cylinder — so it can be jumped over, and what you see is what catches you.

**Why the spike moved to `E`.** Shift + click means "the committed version of your attack" on
every class, and the spike is not that — it is a placement. Meanwhile `E` is the class
mechanic and the Blood mage's mechanic is *health*, which is not a thing you press a key to
change, so her `E` did nothing for the whole of a match. This cost a fourth column in the move
table, which five classes leave empty. That is the price and it is worth it: the alternative
was a slot that means one thing on five classes and another on the sixth.

**Why the auto is a returning blade.** It is the archive's "low CD ability", and it is the
simplest possible statement of the class: throw something away, get it back if things go
well. The payment arriving **on the catch** rather than on the cut is what makes an auto
attack a small commitment instead of a free poke — the blade is in the air for forty-eight
frames and the health is not yours until it comes home.

**Why the spike's return is continuous.** The archive pays out when the last tether breaks.
Nobody has built tethers, and a lump sum at the end is an ability you survive a timer to
collect on rather than one you build a fight around. Thirty per cent per drain tick means a
Blood mage standing in a fight is being paid the whole time it is up.

**The one number that had to move twice.** The Grasp's root started at 26 frames against the
arms' own 24 frames of hitstun, which made it invisible — it expired inside the stun that
delivered it. It is 40 now, and `a_root_outlives_the_hitstun_that_delivers_it` pins the
relationship so it cannot silently invert again during tuning.

**Verdict** open. The frame data holds every property in `feel.rs`, including a new one that
says a Blood mage ability thrown perfectly must return more than it cost — which the Grasp
failed at 35% leech and passes at 55%. None of it has been played. The costs in particular
are a guess: the class is downstream of TTK, and what fraction of a health bar a cast should
represent is exactly the question a prototype answers and a document cannot.

### 2026-09-13 — why the drain's healing is invisible, and it is not the bug

**Found** the field's return being reported as missing a second time, after the bug that was
actually causing it had been fixed. The second cause is not a bug at all, and it is worth a
dated entry because it will be reported a third time otherwise.

**Health cannot go over the bar, and the eruption gets there first.** The spike costs sixty,
the eruption returns fifty-nine of it on the frame it lands, and every tick of the field after
that is clamped away. Measured, with a target standing in the field for its whole life:

| Cast at | Eruption returns | Field returns |
| --- | --- | --- |
| 1000 / 1000 | +59 | **+1** |
| 700 / 1000 | +59 | +90 |

So the ability reads as broken in exactly the situation anybody tests it in — the first cast of
a fresh round, at full health — and works from the moment you have taken a hit.

**Which is arguably the class working.** She heals when she is hurt and gains nothing when she
is whole, so the spike is nearly free at the top of the bar and a large swing when she needs
one. That is a good shape and nobody designed it; it fell out of the cost and the leech being
close to equal.

**If it should be felt at full health**, the eruption is what eats the headroom, and the
written design already says what to do: the spike *returns a share of everything it drains*,
and arrival damage is not a drain. Stopping the eruption leeching would leave the room for the
field to fill. It needs a second leech number — one move has one today — and it cuts what the
ability returns overall, so it is a decision rather than a correction. Left to the next tuning
pass.

**Verdict** no change. Written down, and the kit document carries the table.


### 2026-09-13 — a Grasp that closes pulls you in

**Changed** `grab_hold` on Grasp means something now. Catch somebody with every one of the four
arms and they are hauled to the caster's arm's length, held for twenty frames, and rooted for
twenty more after the hands open.

**Why** the knob had been set to 20 in a bake and was doing nothing at all: the arms are an
effect, and the effect delivery path hard-coded `grabs: 0`. It carried damage, stun, blockstun,
knockback and launch, and dropped the grab on the floor. A table that can be edited and is not
read is worse than a table with a gap in it.

**The thing that decided the design.** The obvious implementation — every arm grabs — does not
work, and the reason is worth writing down because it is not a balance argument. The arms do
not all land on the same frame:

```
f38  arms landed: [1, 3]      the bottom pair
f39  arms landed: [0, 1, 2, 3]  the top pair, one frame later
```

A grab drags its victim to the caster. So a grab on the first contact moves them ten metres out
from under the arms still in flight, the top pair miss, and `parts_landed == GRASP_ARMS` is
never true — **the grab would silently delete the root**. The two payoffs go on the same
condition because the first one eats the second otherwise.

That turns out to be the better ability anyway. One or two arms is damage and you stay where you
are; all four and the cone closes, takes you with it, and leaves you in melee range of somebody
whose next swing is worth 1.4× against anything that cannot move. Rend is thirty-six frames end
to end and the window is forty, so exactly one of them fits — which is the shape a read should
pay out in.

**How it is tested.** A sweep rather than a fixture. The interesting positions are a hand's
width apart — the arms converge, so four arms and two arms are about a metre from each other —
and a test that picked one of them would be pinned to today's cone width rather than to the
rule. `only_a_full_grasp_catches_anybody` walks the victim across the cone and asserts the
biconditional at every step: held exactly when all four landed, rooted exactly when all four
landed, and it fails if the sweep never sees both cases.

**Verdict** open. The reach is ten metres, which makes this the longest pull in the game by a
distance; if it is too much, the reach is the first knob and the hold is the second.


### 2026-09-13 — the field was draining the creature into thin air, and the kit hit twice as hard as it meant to

**Changed** two things, and only one of them is tuning.

**The bug.** A Blood mage's black spike, standing in a hunt, took health off the Ridgeback and
returned her none of it. The two field effects called the creature-damage path and **discarded
what it told them**:

```rust
self.gore_the_creature(effect, 0, effect.pos, volume.radius);   // returns the damage dealt
```

Every other way she deals damage pays her: a swing, a blade in the air, an arm of a Grasp, a
field ticking on a *fighter*. Fields on the creature did not, so half her economy was missing
in one of the game's two modes.

Two things hid it, and both are worth naming because they will hide the next one:

- **Versus never sees it.** A field only meets the creature in a hunt, and the versus tests are
  where the drain was proved to work.
- **The eruption pays out on the same cast.** The spike hits once when it arrives, that hit
  leeches correctly, and it lands a few frames before the field's first tick. `assert!(health >
  before)` over a window containing both is satisfied by the wrong one. The test now starts
  measuring *after* the eruption is over, and it fails if the payment is removed — checked by
  removing it.

**The number the bug cost, on the way past.** `hits again every` was set to 1 on the spike
while chasing this, on the reasonable-looking guess that it was what made a persistent drain
persist. It is not: it is the re-hit interval for the **move's own hitbox** during its active
frames, and the spike's window is four frames, so it turned one eruption into four or five. The
field's clock is `effects.damage tick interval` and always was.

**The tuning.** With the eruption multiplied by five and a Grasp at 95 per arm, one cast of the
spike was 64% of a health bar and a Grasp was 38%. Halved, with the health costs halved beside
them so that **every ratio the last pass tuned is untouched**:

| | damage | cost | returned | dmg/cost | back/cost |
| --- | --- | --- | --- | --- | --- |
| Bloodletter | 80 → 40 | 15 → 8 | 32 → 16 | 5.33 → 5.00 | 2.13 → 2.00 |
| Rend | 130 → 65 | 60 → 30 | 65 → 32 | 2.17 → 2.17 | 1.08 → 1.07 |
| Grasp | 380 → 192 | 90 → 45 | 209 → 105 | 4.22 → 4.27 | 2.32 → 2.33 |
| Black spike | 640 → 320 | 120 → 60 | 377 → 188 | 5.33 → 5.33 | 3.14 → 3.13 |

The drift is integer rounding on two costs and nothing else. The spike's eruption absorbs the
re-hit going away: forty dealt five times is one hit of a hundred, halved from two hundred, so
reverting the accident and halving the damage are the same edit. The field's own tick went 22 →
11 with it.

**Two tests restored and one added.** A merge two days ago dropped
`the_blood_mage_pays_for_everything_and_nobody_else_pays_for_anything`,
`a_root_outlives_the_hitstun_that_delivers_it` and the bounds on the disabled multiplier —
which are precisely the assertions that keep a bake like this honest, and they were gone for
the bake that needed them. They are back, and `best_case` understands re-hit now.

The new one is `the_blood_mage_does_not_kill_in_two_buttons`, and it is the one that would have
caught this: **her per-hit numbers say very little about what a cast is worth.** Every ability
she has connects several times — two passes, four arms, twenty ticks — so 95 in the table is
380 in the hand. Nothing in the table looked wrong.

**Verdict** open. The ratios are the ones that were played and liked; only the scale moved.


### 2026-09-12 — the Blood mage's root has something on the other side of it

**Changed** a Blood mage's damage is multiplied by **1.4 against anything that cannot move**.
Disabled means rooted, staggered, held, or a creature on its side. One knob,
`disabled_damage_mul`, under Blood mage.

**Why** the entry above gave the class a root and left it as its own reward. Four arms of a
Grasp is the most expensive thing in the kit, the root is forty frames, and landing it bought
you forty frames of somebody standing still — which is worth something, but not ninety health
and twenty frames of recovery. The archive has the answer and has had it since 2016: *naturally
deals increased damage on disabled enemies*. Until this week the class had no disable of its
own, so the trait would have been a bonus against a team-mate's crowd control. It has one now.

**What counts, and the one exclusion that is the whole definition.** Hitstun is **not** a
disable. It happens on every hit anybody lands, so counting it would make the trait "increased
damage from the second hit onward" — a flat damage bonus in a costume, needing no read at all.
What is on the list is what `ability-spec.md` calls a hard stop, and the design only allows
those behind a hard condition:

| Disabled | Earned by |
| --- | --- |
| Rooted | every arm of a Grasp |
| Staggered | a parry |
| Held | a grab |
| Toppled | breaking the creature's poise, which is what the climb is for |

Blockstun is deliberately absent. They blocked, which was the correct decision, and paying the
attacker for it would make guarding worse than standing still.

**One function, five callers.** A swing, a blade in the air, an arm of a Grasp, a field
ticking, and all of the same against the creature. They were five separate pieces of damage
arithmetic and the multiplier goes through one `preying()` in all of them — a class trait that
applied to three of a class's four abilities would not be a trait, it would be a bug somebody
finds in a match.

**The two relationships that had to be pinned.** The bonus is bounded between 1.2 and 2: below
the floor nobody feels it and the Grasp is a root with no payoff, above the ceiling one read
ends the round. And **the root has to outlast her fastest startup**, or there is nothing she
can land inside it — forty frames against Bloodletter's seven and Rend's fourteen, so both
fit.

**The awkward part, and it is in the test rather than the game.** Comparing damage against a
toppled Ridgeback to damage against a standing one is not a fair comparison by default: a
toppled creature lies lower and pitched, so the same swing lands on a different part, and the
hide's vulnerability differs part to part. The fixture freezes the animal, searches for a spot
where the claw reaches the *same part* whether it is up or down, and stands the fighter there.
Worth writing down because the first two versions measured geometry and reported it as the
rule being broken.

**Verdict** open, like everything else in the class this week. 1.4 is a guess inside a bounded
range; whether the root is long enough to actually use is the play question, and if it is not,
the root's length is the first knob and the bonus is the second.


### 2026-09-12 — the zones hand over instead of swapping

**Changed** each zone's ramp is eased in and out, over a share of its own span given by a new
per-zone percentage. Floor 100, the turn 50, the handover 50; the neutral zone and first person
hold still and have nothing to ease.

**Why** reported: the transitions felt unpolished — the camera suddenly starts behaving
differently. That is exactly right, and the measurement says why. Every version of this rig has
held the eye's **position** together across a boundary. What none of them held together was its
**speed**:

| Boundary | Before | After |
| --- | --- | --- |
| −45 | 0.086 → 0.162 m/deg | 0.135 → 0.162, no step |
| −10 | 0.160 → 0.275 | eases across 0.16 → 0.27 |
| 0 | 0.270 → 0.921 | eases across |
| +10 | 0.900 → **0.000** | eases to a stop |

The +10 one is the worst and the most telling: the eye is sweeping at nearly a metre per degree
and then stops dead, because the handover's contraction hits the end of its ramp. Nothing jumps,
and it still feels like something did.

**Why easing the ramp rather than averaging two zones.** The first attempt was the literal
reading — blend the two zones' cameras across a window straddling the boundary — and it does not
work, for a reason worth writing down. **Outside its own band a zone is frozen at the waypoint it
was heading for**, which is exactly its neighbour's value, so the two zones being blended are
identical on one side of every boundary and the average changes nothing there. Worse, blending a
ramping zone against its own frozen endpoint is *algebraically the same thing* as remapping that
zone's ramp — so the window was already an easing, just one centred on the wrong place: it
reached full weight at the boundary instead of zero, which left the kink exactly where it started.

Put the easing inside the ramp and every boundary is covered by whichever side is actually
moving. The curve is the cubic `u²(2 − u)`: flat where it starts, and arriving at exactly the
gradient of the straight line it rejoins, so the two meet without a corner.

**Found on the way, and fixed:** `the_neutral_zone_holds_the_fighter_low` was already failing on
main. The framing field of view was baked to 57 while the renderer draws at 58, so the feet land
at 16.7% of the screen where the knob asks for 16. That is the design working — the framing has
its own field of view precisely so that a player widening theirs cannot move their aim — but the
test was comparing a framing-space waypoint against a render-space measurement. It converts
between them now, and the mismatch is documented rather than assumed away.

**Verdict** open. `the_eye_never_changes_pace_abruptly_at_a_zone_boundary` is the new guard: it
compares the change in pace at each boundary against an ordinary step of the mouse elsewhere, and
with the easing removed it fails at level with thirteen times the ordinary change.

### 2026-09-12 — the Elementalist's auto became a line

**Changed** the auto from a flat circle at a fixed distance in front of her to a **beam**: an
instant ray from her chest along the crosshair, out to the move's own reach. Its `reach` went
4 m → 9 m and its `radius` 0.7 → 0.35 — longer and thinner, because it is a line now. Hitstun
14 → 0 and knockback 2 → 0. `bolt_aim_range`, `fire_bolt_damage_(x)` and `fire_bolt_knockback_(x)`
are gone; seven knobs for a real fire-bolt projectile replace them.

**Why** the complaint was that aiming up did nothing: *"even when I aim upwards, the auto attack
still just follows along the ground."* It did, and the reason was one function. Every trace the
move used — against a stone, against a fire pillar, against a body — went through a flat,
height-free ray test, so the pitch of the aim was thrown away before anything was compared. The
shot went the same distance along the ground whatever the crosshair said, and the overlay drew
the same upright cylinder a fixed distance ahead, which is what made it look correct and behave
wrongly at the same time.

**The fix is a shape, not a special case.** `math::ray_hits_cylinder` is a real
three-dimensional ray against an upright cylinder with both caps, and a fighter, a stone and each
slab of a fire pillar are all upright cylinders. The aim resolver already traced stones that way;
now everything does, and the flat version is deleted rather than left around to be used again by
mistake. The arena's boxes went the same way into `math::ray_hits_box`, which is also what lets
the beam ask the Ridgeback which *part* a line reaches first.

**What it meets first is the whole move** — a fighter, a structure, or fire — rather than the old
two-way "is it aimed through a structure or a pillar" with the fighter check bolted on separately.
Three outcomes and one comparison is both smaller and the thing a player can actually state.

**The fighter case is the design change worth arguing about.** It does small damage, it takes the
move they were charging, and it hands their frames straight back: no hitstun, no stagger, no
shove. That breaks `landing_a_hit_keeps_the_initiative_or_resets_neutral`, which is a real
property and not one to wave away — so the test now excludes moves that hand out no stun at all
and a new one, `a_move_that_never_stuns_is_the_cheapest_thing_its_class_throws`, states the price:
such a move must be minus on hit and must be the smallest hit in its class. What it buys is an
interrupt, and a move that bought an interrupt *and* damage would beat the moves that pay stun for
theirs. Blockstun stayed at 6: "no stagger" is about landing it, and guard is still worth holding.

**The fire interaction is a projectile now, not a longer instant hit.** It used to be the same
hitscan shot with its damage and knockback multiplied and its range quietly swapped for the
20 m aim range — a poke that could cross the arena instantly, which is not a poke. A pillar
**lights** a bolt instead: 38 m/s, 0.3 m across, 24 m of range, 70 damage, 9 frames of stagger,
and it starts *at the pillar*. Starting it at her hand was the tempting simplification and it
would have made the whole interaction invisible.

**The picture and the rule come from the same place.** `state::hitbox` is a capsule between two
points rather than a sphere at one — a swing is the case where both ends coincide — so the game,
the debug overlay and the browser tool all draw the volume that was tested. The shot is drawn as
the thin cylinder it is, and the body tilts on to the aim through spine, chest, shoulders and
head, which is what the bolt clip's own author's note had been asking for: the off hand was "the
only thing in the pose that says which way the bolt went".

**Also fixed on the way.** `nothing_is_dirty_before_anything_is_touched` was racing the one
mutating Oven test about one run in fifteen under load — a pre-existing flake, reproduced 14 times
in 200 runs on the commit before this work. The readers take the same lock as the writer now.

**Verdict** open — none of the seven new numbers has been played against, and the beam's 9 m is a
guess at "short-to-middle". The two properties worth watching are whether the interrupt is strong
enough to be worth the total lack of pressure, and whether a pillar plus an auto is too easy a
long-range poke for a class that is supposed to want you at terrain range.

**Still open from "the autos are due a pass, as a set":** the melee classes' autos, and the timing
pass across all six. This closed the Elementalist's.

### 2026-09-12 — one aiming model, written down and enforced

**Changed** every decision about where an ability goes now comes from
`crates/sim/src/aim.rs`, through one of two functions. The specification is the new
[aiming.md](aiming.md). `crates/sim/tests/one_aim.rs` fails the build if anything else in the
simulation reaches for a ray-against-shape primitive, for the camera's eye, or for the look
direction.

**Why** aiming had been got wrong three times, and the third time was mine. The mistake has
the same shape every time: someone needs to know where an ability should go, works it out next
to the ability that needs it, and writes a ray from the **chest** along the **look angle**.
That ray is *parallel* to the crosshair's and parallel rays never converge — the reticle sits
on one spot and the ability goes to another, by metres, and the error grows with distance.

It had survived this long because grounded abilities were separately settled onto the floor,
which hides it: a fire pillar traced from the chest and one traced from the eye land in
roughly the same place once both are dropped to the ground. Building something that *flies* on
the same sentence is what exposed it.

**The model.** One raycast, from the camera through the crosshair, ignoring anything behind
the character model. It meets terrain, other players, monsters, structures, and the ability's
own max-range sphere; whatever it reaches first is what the player is pointing at. Then
exactly two kinds of skillshot:

- **grounded** — on the ground, cast exactly there; at the sphere, max range on the ground in
  the mouse's direction; on anything else, the floor beneath it. If it travels, from the
  character to that point.
- **not grounded** — on the ground, that spot raised straight up to the height the shot leaves
  at, so it flies level over the place the crosshair is on; on anything else, the point of
  intersection exactly. Straight line from the caster, and that line is its whole reach.

A melee swing is neither and is named as such, because "a sword is a body moving" is a rule
somebody would otherwise delete by accident.

**Three things fell out of it that are worth recording.**

*Fire is not on the aiming ray's list.* You can see through flame, so a pillar must never
steal the crosshair — but a shot that travels through one still notices it. Separating "where
is the player pointing" from "what is in the way of the thing they threw" is what makes both
true at once, and it is now two functions rather than one confused one.

*A level look does not point at the enemy's chest.* The camera sits above the shoulder, so at
level pitch the reticle is above head height and a shot goes over them. That is correct — it
is how a third-person camera works — but every test fixture in the suite had been written
assuming otherwise, which is a good measure of how thoroughly the old model had leaked. They
aim by sweeping for the angle that puts the crosshair on the thing now, which is what a player
does and is robust against the camera being retuned.

*The Bulwark's thrown shield had the same bug*, quietly, and is fixed by the same change: it
flies at what the crosshair is on rather than along the look angle from the chest.

**Also:** the move table gained a `flies at the crosshair` flag, so every move states which of
the three kinds it is rather than leaving each caller to work it out. `Player` carries one
`aim_path` instead of a target and a direction that could disagree — one object cannot drift
from itself.

**Verdict** open on feel; settled on structure. Nobody has played against the corrected
aim yet. What is worth watching: whether the beam ending *on* what the crosshair found —
rather than always running its full range — reads as the shot being eaten by scenery, and
whether needing to aim down slightly to hit someone at your own height is comfortable or
merely correct.
---

### 2026-09-12 — the Champion's weapon stopped being a number

**Changed** the class rebuilt. The form toggle and its nine reach/damage/recovery
multipliers are deleted; the three weapons are the three mouse buttons, each with its own
moves on the ground, in the air and out of a Rush — ten moves in total. Rush is built, on
`E`. The uppercut moved from `Q` to Rush + hammer. Autos have shaped hitboxes for the first
time.

**Why** the class's whole premise is *choosing a distance*, and nothing about playing it
made you choose one. All three forms threw the same three moves at the same three heights;
the form multiplied the numbers. That is the diagnosis worth keeping, because it is not
about this class:

> A multiplier cannot make two moves feel different, because it does not change what they
> do — it changes how much. A spear with 1.55× reach is a sword that reaches further.

What separates weapons is **shape**. A sword goes across, a hammer goes down, a spear goes
out — three different questions about where the other player is standing, and none of them
is a coefficient. So the hitbox stopped being a disc at arm's length and became a **capsule
that moves**: a line from the hand to the head of the weapon, swept over the active frames.

**Verdict** open, and already load-bearing: two thirds of the tuning below exists because
the capsule made height matter for the first time.

### 2026-09-12 — a sword that sweeps at chest height misses a low ridge

**Changed** the flat sweep is thrown from 0.7 of the cast height and travels 0.03 turns
below level (`champion.sweep_thrown_from`, `champion.sweep_travels_below_level`), rather
than from the chest, dead level.

**Why** the hunt harness caught it before a person could have. With the sweep thrown level
from the chest, the scripted hunter stopped being able to break the Ridgeback's poise at
all — zero topples across six hunts, where the old disc broke it reliably. The ridge is a
35 cm strip lying on the animal's back, and a sword swept at chest height genuinely passes
over it. That is correct physics and it made the climb pointless.

It is also what the animation has always said the move is: `champion.rs` calls Sweep "a low
horizontal cut… the hands stay at hip height". The hitbox was simply not doing what the
clip was doing. This is the first time the two have been able to disagree, because it is
the first time the hitbox has had a height.

**Verdict** kept. Topples are back and they come from the swing rather than from standing
nearby.

### 2026-09-12 — 113 and 120, or: why the poke's damage is 66

**Changed** the Champion's sword from 95 to 66, and the other autos with it — hammer 48,
spear 58.

**Why** worth writing down because the number is not arbitrary and looks it. The
Ridgeback's ridge multiplies damage by 1.75 and it flinches at 120. The old Sweep did 65,
which is 113.75 on the ridge — **just under**. A sword at 95 does 166, which flinches it
every single hit, and a creature you can keep permanently flinched never bucks you off: the
hunter got on once, stayed for 1590 frames, took zero damage, and killed it in 35 seconds.
Six hunts out of six.

One damage number crossing one threshold turned a fight into a treadmill. The fix is
one number, but the lesson is that `flinch_threshold` and `vuln_ridge` between them draw a
line across every class's poke damage, and nothing in the code says so.

**Verdict** kept. 1 hunt won out of 6, topples in most of them, rides ending in bucks.
Worth revisiting as a *creature* number rather than a fighter one — a flinch that could be
sustained indefinitely is the real bug.

### 2026-09-12 — the uppercut as a combo rather than a button

**Changed** Uppercut is no longer the class special thrown from standing (16f startup,
committed, on `Q`). It is Rush + middle click: 8f startup, no slowing of the dash, it holds
its victim for 26 frames and carries them up, and pressing space once during the hold takes
both fighters higher.

**Why** the old one did not feel good and the reason is that it was a *commitment to the
air* rather than a *continuation*. Sixteen frames of telegraph from a standstill, and if it
missed you were airborne, helpless, and had announced it. Nothing about being in the air
was a reward, because you got there by committing to getting there.

Out of a Rush it is the opposite: you are already moving, the dash paid for the approach,
and the only decision left is whether to spend the charge. Then the extra leap makes the
height itself a choice made *after* it connects, which is the one moment in an exchange
when a player has information nobody else has.

**Verdict** open. The leap is the part to watch — it may want to be spendable more than
once, or to cost something.

### 2026-09-12 — the takeoff to flight handover was a cut

**Changed** the last three frames of `jump_takeoff` blend into the flight pose they are
about to become.

**Why** found by the Champion rebuild rather than aimed at. The animation continuity guard
went from 0.428 m in one frame to 0.462 against a 0.45 bar, and the frame it failed on was
an airborne fighter doing nothing at all. The join between the takeoff clip and the
rise/apex/fall selection is a hard switch *inside one shape*, so the crossfade never saw it
and never softened it — and how bad it looked depended on how fast you were travelling when
you left the floor. Fine most of the time, a visible hitch on a jump out of a sprint.

**Verdict** kept. Worst discontinuity across a 600-frame scripted match is back under the
bar, and this one is a real seam rather than a threshold that needed raising.
### 2026-09-12 — every move declares its line of effect

**Changed** the move table gained a `line of effect` field, and three moves changed kind:
Fissure and Judgement to **grounded**, Lance to **skillshot**. `cargo run -p sim --bin
frametable` prints the column.

**Why** the aiming pass left the *kind* inferred: a move was grounded if what it left behind
was grounded, a skillshot if a flag said so, and a swing otherwise. That answered correctly for
the two abilities that plant something and quietly called everything else a swing. Going
through the roster against the kit documents found three that are not:

| Move | Kit says | Was aimed as | Now |
| --- | --- | --- | --- |
| Fissure | "a skillshot that races forward through the ground… spawns a structure at the point of impact" | swing, 7 m reach | grounded |
| Lance | "line skillshot" in both forms | swing, 4 m reach | skillshot |
| Judgement | "a delayed area strike… range medium" | swing, 3 m reach | grounded |

All three were bubbles hung several metres off the body, pointing wherever it happened to be
facing. None of them is *built* yet — Fissure does not travel, Lance is not a line, Judgement
has no delay — so what changed today is only where their volumes appear, which is now the
place the crosshair is on rather than a fixed step ahead.

**The inference is gone rather than fixed.** Deriving the answer from the effect was the sort
of rule that is right until the first ability that does not fit, and then silently wrong.
What survives from it is a *test*: `one_aim.rs` asserts that a move planting something
grounded is aimed at the ground, and that a move throwing something that travels is aimed
through the air. Those directions are always true; the reverse is not, because Fissure plants
a structure, which belongs to the mechanic rather than to the effects array.

**Found and not fixed: Guillotine lotus.** Its range is "at the shadow" — the blades erupt
where the Reaver put the mechanic. That is not any of the three: the player aimed when they
placed the shadow, not when they threw the move. It is currently a swing with a reach of
**zero**, which puts its volume on the caster's own body, so it is wrong however the question
is answered. Recorded in [aiming.md](aiming.md) under Open rather than guessed at.

**Also considered and rejected:** asserting that a swing's reach may not exceed roughly twice
its radius. It catches the "hole in front" the log already records above — a one-circle hit
test with reach past its own radius has a gap nothing can be hit in — but that is a property
of the hit test rather than of the aiming, and the rule as written failed Rend at 2.6 m
against 1.2 m, which is a tuning question and not a miscategorisation. Left alone.

**Verdict** structural, and open on feel. Nobody has thrown the three retargeted moves.

### 2026-09-12 — a swing is aimed too, with a dead zone

**Changed** melee no longer comes out flat. A swing's yaw is still the body's facing; its
**pitch follows the camera outside a dead zone below the horizon**:

```text
   above the horizon      follows exactly
   the first 45° below    stays level -- the standard arc in front of the character
   further down           follows what is left over
```

So −45° is the same swing as 0°, −46° is that swing tilted one degree down. New knob,
`aim.swing_stays_level_to (deg down)`, and a new line of effect, `aim::swing_path`.

**Why** "aim direction for melee matters a lot in the air and on hills" — and it does: a swing
pinned to the horizontal misses things plainly in front of you the moment either fighter
leaves the flat. The reason it had been pinned was the opposite error, and the dead zone is
what answers it: **the camera sits above the shoulder, so looking at somebody standing at your
own height means looking slightly down at them.** A swing that followed the camera exactly
would tilt into the floor in the most common situation in the game. Neither "ignore pitch" nor
"follow pitch" is right; the dead zone is, and it costs one number.

Written as a sum rather than a branch — `max(pitch, 0) + min(pitch + dead, 0)` — so the two
halves cannot disagree about the boundary. There is no step at the edge: a test sweeps a tenth
of a degree at a time across it and fails on any jump over 0.4°.

**Guillotine lotus got a fourth line of effect: at the mechanic.** Its kit entry has always
said "Range: at the shadow", and it was declared a swing with a reach of *zero* — so its
volume came out on the Reaver's own chest and the move did nothing it was written to do. The
player aims it when they *place* the shadow; throwing it only cashes that in, and re-aiming it
at the throw would delete the reason shadow placement is a decision. The volume follows the
shadow live, because the Reaver can recall it while the blades are out.

That makes four lines of effect, all of them functions in `aim.rs`: two that start with the
crosshair's raycast, and two pointed by something the player decided earlier.

**The animation follows.** A swing's tilt arrives at the renderer already dead-zoned, as the
angle the attack actually came out at, so the same pose layer the Elementalist's beam uses now
serves melee. A grounded cast is deliberately *not* tilted — a pillar comes out of the floor
and the caster is gesturing at the place, so looking down to plant one must not double her
over.

**Verdict** open. Nobody has swung at anything on a slope or in the air yet, and 45° is a
guess — one number for a grapple at arm's length and a spear at 1.55× reach, which may well
want to differ.

### 2026-09-12 — the dead zone is a standing rule

Merging the Champion's air game into the dead zone broke the class immediately:
`the_combo_the_class_is_built_around_is_playable` failed, because the aerial spike only
connects at **45° of tilt or more** and the look-down limit is 85°. Subtract a 45° dead zone
and the most tilt a falling Champion can reach is 40. The move became unthrowable — not worse,
unavailable.

The fix is not a smaller number. It is that the dead zone was only ever an answer to a
*grounded* fact: the camera sits above the shoulder, so looking at somebody standing on your
own floor means looking slightly down at them. Off the floor there is no shared floor to
correct for, and the thing under your reticle genuinely is below you. So `aim::swing_path`
takes `grounded`, and airborne swings follow the camera exactly, all the way down.

This is the split main's own swing shapes already make — in the air the Champion's fan is
thrown *around* the aim rather than in front of the body — and it is what the brief asked for:
"aim direction for melee matters a lot in the air and on hills."

**Verdict** open, same as the entry above. 45° on the ground is still a guess, and now the
question of whether a fighter who has just left the ground wants the zone to fade rather than
vanish is a real one. Nobody has played it.

### 2026-09-13 — bodies come off the aiming ray

**The report:** "attacking a large monster is awkward because the crosshairs tend to sit high,
so the attacks aim high if close in."

Measured before changing anything, with a hunter standing five metres from the creature's
centre — which is nose to nose, because the animal is four metres long from centre to snout.
Its **head was the first thing the camera's ray met at every look angle in a sweep from 40°
below the horizon to 40° above**, including aiming squarely at the dirt, at a point roughly
1.1 m in front of her and 3 m up. So every skillshot she threw came out as a stub about a
metre long pointed into the sky, at exactly the range where you cannot miss.

Not a tuning problem. The ray answers **"which place in the world is under the crosshair"**,
and a body is not a place — it is a thing standing in one. Putting bodies on the list made the
aim point jump to the surface of whatever was between the player and the ground, and for a big
animal that surface is metres above the thing they meant to hit. So other fighters and the
creature came off the list. Terrain, structures and the ability's own max-range sphere stay.

Nothing is lost, because *what a shot runs into* was always a separate question, asked along
the ability's own path rather than along the camera's ray. The two were never the same line —
the camera is behind and above — so a body the camera could not see was always still a body
the shot went through. Put the reticle on somebody and the shot still reaches them; what
changed is that the aim point is the ground behind them, so a level look is a level shot.

**What it buys, measured after:** from the same spot, a level look now runs into the
creature's barrel at 2.4 m — the flank, which is what a player standing on the ground is
actually trying to hit. Looking up walks the shot to the neck and then the head; looking down
past 25° puts it on the dirt in front of the animal, which is where it was pointed.

Three test fixtures had quietly depended on the old rule and were saying something they did
not mean:

- `beam.rs` read "the crosshair is on them" off the raycast. It now means what a player means
  by it — the line the reticle picks goes through them.
- `effects.rs` aimed at *half a body height above the arena floor* rather than above the
  target's own feet, which is a different point entirely for somebody standing on a platform.
- The same fixture aimed at where it had just teleported the target, one frame before the
  world dropped them onto the platform under them.

**Verdict** open, and one thing to watch: a fighter standing on open ground is now aimed at
through the floor behind them, so a shot at somebody backed against a wall ends on the wall
rather than on them. Both hit. Nobody has played it.


### 2026-09-13 — the Dual mage got both her arms

**Changed** the kit on the buttons. Left click is the **dark auto**, right click is the
**light auto**, `shift` + left is Lance, `Q` is Judgement, and `E` casts a new move,
**Sweep**. Five moves where there were three, and both mouse buttons are attacks.

Right click was doing nothing on this class. `want_guard` asks for a shield in hand and she
has no shield, so half of the mechanic — *right click moves you lighter* — had no input at
all, and the meter could only be driven one way. `E` was also dead, for the same reason the
Blood mage's was before Black spike: the mechanic is a meter steered by which button attacks,
so there is no state for a key to toggle.

**Why the autos come out of the arms.** The class holds two forces apart, one in each arm,
and the only information the player has about which one they just threw is which arm threw it
and which way the bar moved. Both volumes leaving from the middle of the chest would make
that unreadable. So a move now declares a **hand** next to its shape (`moves::hand`), and
`aim::hand_origin` steps the swing's origin out to that shoulder. Everything else in the game
is `Hand::Centre` and is untouched.

The sides are the **skeleton's**, not the world's, and that is worth writing down because it
looks like a bug until you check: the body is authored `+Z` forward with its left arm at
`-X`, which is a left-handed frame in a right-handed world, so the arm the renderer calls the
left one is drawn on the side a quarter turn *toward* strafe-right. Following the skeleton is
the only choice that matters — what an animation and a hitbox have to agree about is which
arm the player can see swinging — and `view/tests/kinematics.rs` now fails if the two ever
disagree. Left as it is rather than fixed: unmirroring the model means negating the sideways
coordinate of every authored key in every clip, which is a change with no visible payoff and
a large blast radius.

**The wing.** A new hit shape, `Shape::Wing`. It starts a little behind the fist, sweeps
outward — away from the body, on whichever side the arm is — and **grows** to the move's whole
reach across the active window, so the tip travels a spiral rather than an arc. Two and a half
arm lengths at full extension, which `view/tests/kinematics.rs` checks against the Dual mage's
own build rather than against a number typed twice.

It is its own shape rather than a swing with a big arc for two reasons that are the same
reason: a swing's head is at a fixed reach and a swing's inner end is at the shoulder. Both of
those are what make a swing *safe to step inside*, and the wing is meant to be the opposite —
the class is long and thin and loses to anyone who has closed.

Both autos share one `arc`. The **hand supplies its sign** (`aim::Hand::outward`), so the
mirror is structural rather than two knobs somebody has to keep equal and opposite.

**Steering, corrected.** The autos now steer **on contact** and everything else on the press,
which is what the design document has always said and what the implementation did not do —
before this, swinging a poke at thin air walked the bar. And the side comes from the *move*
rather than from the input bits: `shift` + left click has a modifier and a side in it, and
reading the bits meant deciding which won. `moves::dual::side` answers it once. A move with no
side — `Q`, `E` — pushes you further along the path you are already on, and does nothing at
dead centre, which is the rule the document states for scroll click and both-click.

**The animation.** The autos are one punch read twice: `Pose::other_arm` mirrors everything
above the hips and re-plants the feet where they were, so a left jab and a right cross come
off one set of keys. Mirroring the *stance* as well was the first attempt and is wrong — every
clip in that file has to start and end on the idle's own stance, so a mirrored first frame
swaps the character's footing on the frame the punch starts and swaps it back on the frame it
ends.

Sweep took three passes to get through the continuity ceiling, and the third one is the
correct animation rather than a concession: the arms now cross the front **during the active
frames** rather than before them, because that is when the hit volume crosses. The first two
versions had the body arrive early and then wait, which is both a 0.4 m per frame hand and a
lie about where the danger is.

**Verdict** open on everything with a number in it. Nobody has played it. The specific
questions: whether the wing's outward opening reads as a wing or as a wild swing, whether 50°
of arc is enough to feel like it wraps, whether the punch at five frames of startup is too
fast to see which arm it was, and whether Sweep at twelve frames is a real answer to somebody
inside the punches or just a slower one.
### 2026-09-13 — the Shadow Reaver's second body

The class's whole kit was rebuilt around one change, and the change is a deletion: **the
shadow can no longer be absent.**

**Changed**

- `Mechanic::Shadow` went from `Option<V3>` to a body with four states -- attending her,
  going out, waiting, coming home. There is no "nowhere".
- The mechanic became a **move** rather than an instant, in a fourth slot in the table:
  *Send shadow* (on `E` at first; see the entry below),
  8/3/14, reach 9 m, damage 70 on the way home. The shadow flies out in ten frames and stops;
  pressed again it dashes home at 34 m/s through anybody in the way, cutting once and slowing
  them to 0.55x.
- The leash went from 8 m to **12 m**, and had to: the throw reaches 9, so at 8 the shadow
  turned round on the frame it landed. A leash shorter than the throw is not a tuning
  mistake, it is the setup deleting itself.
- **Guillotine lotus** stopped being a disc at the shadow and became six blades that erupt
  along curved paths (4.5 m, 7 frames), hang open for 40, and chase the shadow home over 26,
  dealing 70% of what they dealt going out. 40 damage a blade, which is deliberately small:
  six numbers can land at once.
- **The shadow copies her swings**, 4 frames later, at **25%** of her damage, from wherever
  it stands.
- The forward dodge, thrown with the crosshair on the shadow, became the dash to it: 34 m/s
  constant, invulnerable, and arriving collects the shadow.
- Executioner picked up **right click**, which was dead on a class with no shield. Swapped
  back a day later; see below.

**Why** two reasons, and the second is the one that mattered.

The stated one: the class read as a setup class that spends most of a match with no setup.
`None` meant no swap, no Guillotine, no line -- and the fix the kit document had already
reached for, *the baseline dash creates the shadow*, only papered over it.

The one found while building it: **`Option` was making the code worse in the same shape it
was making the class worse.** Every ability that read the mechanic carried a branch for the
case where the mechanic did not exist, and every one of those branches was a design question
nobody had answered. Deleting the case deleted the branches.

**What the numbers are for.** The 25% is the class in one number: holding the shadow is a
flat 1.25x on everything her body does, and sending it out trades that quarter for a second
threat somewhere she is not. That is a real decision every few seconds, which is what a
mechanic is supposed to be. The 40-per-blade is a guess bounded from above: standing exactly
on the shadow through a whole lotus is six blades out and six back, which is 240 plus 168 of
a thousand, and that is meant to be the execute rather than the opening.

**Two implementation notes worth keeping**, because both were bugs first:

- The blades are tested as **swept lines**, not as points. The eruption crosses 4.5 m in 7
  frames and is fastest on the first of them, so a blade sampled as a ball starts the frame
  at the shadow's feet and ends it a metre past whoever was standing there. The one victim
  the ability is named for was the one it missed.
- The echo is **a move index and an age**, and the shadow's own startup/active/recovery are
  derived from the same table hers come from. A second state machine would have to agree with
  the first, and eventually would not.

**Verdict** open, and there is a lot here to play. Three specific worries:

1. **The lotus dragged home may be too much.** It is two buttons, it covers the length of the
   arena, and it hits everything twice. That is the intended fantasy; whether it is a fair
   one is a question for a person.
2. **The attending shadow may be hard to read.** It stands 0.9 m behind her, which from a
   camera sitting directly behind her is exactly the direction that overlaps. It separates the
   moment she moves or turns, and `reaver.shadow_trails_her_by` is the knob if it does not
   separate enough.
3. **Right click doing two things across the roster** -- guard on four classes, an attack on
   two -- is now a real inconsistency rather than a Champion-shaped exception. It is the
   cheapest of the three to reverse.


### 2026-09-14 — the Reaver's two buttons, swapped

**Changed** right click sends the shadow, `E` throws Executioner. It was the other way round
for a day.

**Why** nothing about the moves, and everything about which hand is doing what. Sending the
shadow is a **placement**: a grounded cast at a patch of floor the player picked, and where
that floor is is the decision the whole class is made of. Executioner is a swing off the
body — yaw from the facing, pitch from the camera — and reads the crosshair as an angle
rather than as a place.

Sentence six of the control grammar is *the mouse means where*. Putting the aimed half on
the mouse and the unaimed half on the key is that sentence applied, and the first
arrangement had it backwards for no reason beyond `E` being the mechanic key everywhere
else.

The specific thing that made the first arrangement feel wrong, and the one worth writing
down: **placing something with the hand that is not holding the mouse means committing to a
spot you are about to stop looking at.** You press `E`, and the pointing you did a frame ago
is already stale, because the mouse never stopped moving. On right click the press and the
aim are the same gesture.

**What it costs.** The Reaver is now the one class where `E` carries something that is not
the mechanic, which makes the grammar's fifth sentence slightly less true than it was. The
honest split is two rules rather than one — *a mechanic is an ability when pressing it is
not free*, and *which button it lands on is the crosshair's question* — and both are now in
controls.md.

It also leaves Deadly mistake with nowhere to go: right click ignores `shift`, and `shift` +
`E` is Executioner. That was true of the previous arrangement too, in mirror image.

**Verdict** open. It is a two-line change and reversible, which is most of why it was worth
trying rather than arguing about.

### 2026-09-13 — the wing became a section of a torus

**Changed** the Dual mage's autos from a line that reached out of the fist and grew, to
**a chunk of a horizontal ring around her that sweeps from behind to in front**. The active
window went 4 → 6 frames and the recovery 11 → 9 to pay for it, so the frame advantage is
unchanged: +1 on hit, −6 on block.

```
before   a capsule from just behind the fist, sweeping ~50 deg outward and
         growing from a third of its reach to all of it
now      a ring centred on her own axis, inner arc 0.26 m and outer 1.65 m,
         appearing 151 deg behind her on the punching arm's side and arriving
         directly in front of the fist six frames later
```

**Why** the first version read as an arm attack with a long arm. The class is a vessel holding
two forces, and what comes out of it should not be shaped like a limb — it should be shaped
like something that was already circling her and got let out. A ring does that and a spoke
does not, and the difference is where the volume *starts*: behind, where the player cannot see
it coming, rather than at the fist where they were already looking.

**The volume out on any one frame is the section's radius**, not a curved shape, and that is
not a compromise: a radius of an annulus is straight, so the straight capsule the hit test
already understands *is* the section at that angle. The ring is what the sweep carves. Six
frames at 30 degrees each, with the reach threshold on top, leaves no angular gap for somebody
to stand in.

**Two of its numbers are now stated against her own body** — the inner arc passes through
where the punching elbow starts, and the outer is two to three times further out than the fist
gets, measured from that elbow. Neither is a thing the simulation can check, so
`view/tests/kinematics.rs` reads them off the baked clip: 0.26 m at the elbow, 0.78 m at full
extension, and the reach of 1.65 m is 2.7× the travel. Retiming the punch or re-authoring the
cock moves the check with it. The two Oven knobs the growing version needed collapsed into one
(`wing_inner`), because a section that does not grow has only an inside.

**The plane is the floor's while she is standing**, which makes this the one attack in the game
that ignores the camera's pitch outright. Airborne it tilts with the aim, which is the split
`swing_base` already makes for a flat sweep thrown off the ground.

**Verdict** open. The specific worries: 151 degrees behind her may be so far back that the
first half of every auto is wasted on empty air, and a ring with a 0.26 m hole is a shape you
can beat by standing *on* the mage, which no other melee move in the game rewards.

### 2026-09-13 — the wing opens, the tip pays, and the bar is a bar

Five things on the Dual mage, from playing her.

**The wing is a real section of a torus now, and it opens.** It used to be the section's
radius sweeping round at a constant width; it is an angle that **grows from nothing to a
hundred and thirteen degrees** over the active window, with the leading edge coming round
toward the front. The volume is `math::Sector` — the first thing in the game that is not a
capsule — and both the hit test and the overlay read it, because a straight line through a
curve either misses the inside of it or claims the outside.

**Why bother**: the sweeping version read as a blade. An opening one reads as a wing, which is
the fantasy — the beings inside her extending the movement past where an arm could take it.
The shape had been carrying that idea and not showing it.

**The last frame is the tip, and it hits 1.75×.** The wing opens to most of its arc and stops
short; the tip covers the rest and arrives straight ahead on the final frame. So the only
thing that reaches the point directly in front of her at full extension *is* the tip. That
makes it a spacing decision rather than a bonus on a frame number — the body of the wing is
for somebody already on top of you, the tip is for somebody who thought they were out of
range. The overlay draws it in its own colour, because a decision you cannot see the result of
is not one anybody learns.

Getting there took one correction worth recording. The first version had the wing reach the
front on the frame *before* the tip, so a body standing there was caught by the wing and the
tipper could never land on anybody standing still. Two frames of the shared `swing_progress`
saturate at the end, which is right for a weapon arriving and wrong for a shape whose last
frame is meant to be a distinct event; the wing counts its own frames now.

**`Q` stopped being gated.** Judgement needed the bar deep before it would come out, which
meant the class special did *nothing at all* for the opening of every match — the player
pressing it could not tell an ability from an empty binding. A special you cannot press is not
a special. What the gate was protecting has to come back as power rather than availability,
and that is the next thing this class needs: **nothing scales with depth yet**, which is the
founding idea of the whole mechanic.

**The bar picks sides differently.** The autos have sides; nothing else does. Landing one sets
which force she is *carrying*, and every other input is made of that force and pushes the bar
that way. The old rule — the button that threw it picks the side — could not answer for `Q`
and `E`, which have no side, and answering "further along the way you were going" made the
keys feel like they were guessing. Autos move five, casts move twelve; the damage-scaled
formula that was there before was a knob nobody could find.

**And there is a real bar to look at.** Two-poled, filled out from the centre, deep thresholds
marked, and its border in the colour of the force she carries — which is deliberately a
*different* question from which side of the bar she is on. She can be deep in the dark and
still light, having just landed one light auto, and until she lands a dark one everything she
casts is light. The number in the mechanic line was unreadable in a fight, and the thing it
was reporting is what the player steers with every click.

**Ascension is a clock.** It had no exit at all: reaching the end of the bar burned her down
to one health and went on burning, with no timer, no stun, no reset, and no signal that
anything had happened. Now driving the bar to either end starts three seconds, drains about
seventy per cent of a health bar across them, pins the meter, and ends by putting her back at
the centre staggered. Everything else the design asks of it — the refund on hitting, the
larger form of every ability, the graduated stun — is still unbuilt. This is the shape, not
the feature.

**Verdict** open on all five. The specific worries: a hundred and thirteen degrees opening in
six frames may be too fast to read as an opening at all; the tip may be so much better than
the body of the wing that spacing for it is the only correct way to throw the move; and an
ungated finisher with no depth scaling makes centre a perfectly good place to stand, which is
the thing the whole class is built to punish.

### 2026-09-13 — the bar did not move, and the reason was a rule that reads well

**Reported from play:** "still not getting any movement of the bar with my attacks on dual
mage." Reproduced in one test — a mage standing where she spawns, eight metres from anybody,
pressing each of her five buttons five times:

```
LMB  dark auto   -> meter 0, carrying none
RMB  light auto  -> meter 0, carrying none
S+L  lance       -> meter 0, carrying none
Q    judgement   -> meter 0, carrying none
E    sweep       -> meter 0, carrying none
```

Every button on the class, and the mechanic never moved. Two rules, each defensible alone,
that multiply to nothing:

- **The autos steered on contact only.** With nothing in reach they steered nothing, which is
  exactly what "a whiff steers nothing" says and exactly what makes it unusable — a player
  with no target has no way to see the mechanic exist, and neither has anybody tuning it.
- **Casts took their direction from the force she was carrying, and she carried none** until
  an auto *landed*. So the casts multiplied by zero as well.

Either one alone would have been survivable. Together they made a class whose entire identity
is a resource into a class with no resource.

**Changed.** Steering happens on the **press**, for everything, autos included. And she is
always carrying one of the two forces — dark to start, which is arbitrary between two
symmetric things and is not nothing.

**What that costs.** "Landing the far-side auto is the fast way back toward centre" is a real
idea: it is what forced this class into melee exactly when it is deepest and most fragile, and
it is gone. The obvious replacement was a **bonus for landing** — the press moves you,
connecting moves you again — which keeps the pull toward melee and still lets a player in an
empty arena see their own mechanic work.

**The lesson worth keeping** is not about this class. It is that a rule which says *nothing
happens unless* needs a second rule saying what happens the rest of the time, and both of
these said "nothing". The test that now guards it does not check a number: it checks that
**every button on the class moves the bar with nothing in range.**

**Verdict** kept, and **the landing bonus is not being built.** Played against, the pull
toward melee is already there without it: she is frail, her reach is short to middling, and
staying in the band where she can trade while watching the bar is enough to manage at once.
A second rule about where the resource moves would be a rule to learn rather than a decision
to make. Steering is one sentence now — throw something, the bar moves — and the difficulty
lives where the player already is, in her body and her spacing.

Depth scaling is still the open hole, and it is a bigger one than this ever was.

### 2026-09-13 — the Grasp is aimed with time, and its catch is a trip

**Changed** three things about Grasp, which together turn it from a ten-metre snare into the
class's one setup tool.

```
before   press Q -> arms fly to the move's reach -> anything all four catch is
         teleported to the caster's arm's length, held 20f, and rooted 40f
now      hold Q up to 30f -> a marker travels 3 m to 10 m in front of you ->
         let go and the arms converge where it was -> anything all four catch
         is bound 10f where it stood, hauled in at 40 m/s, and free the frame
         the hold ends (26f in total)
```

**Why the channel.** The move had one range and it was the wrong one most of the time: at ten
metres it sailed past anybody standing at seven, and there was no way to ask for seven. Range
wants to be a decision, and the only input left to make it with is *how long the button is
down* — every other axis of the control scheme is already spent. Holding a cast button is new
grammar, so it means exactly one thing and the move table carries it: `Channel, longest hold`
and `Channel, reach at no hold` are two more columns beside `startup` and `reach`, not a
Blood-mage special case. A second channelled move inherits the whole mechanism.

**The marker is the far end of `aim_path` and nothing else.** It is re-solved every frame of
the wind-up by the same `sim::aim::skillshot_path` call the released move uses, with the reach
the hold has bought so far. That is deliberate to the point of being the design: the codebase
has three separate incidents of a reticle and an ability disagreeing because two pieces of
arithmetic were kept in step by hand. There is one object here, the renderer reads its `to`,
and there is nothing to keep in step. The one number that does get stored, `Player::channelled`,
is the *solved path's length* rather than the reach that was asked for — so a wall in the way
shortens the marker and the arms identically.

**A channel resolves instead of the countdown, not before it.** The first version ran
`step_channel` and then fell through to the ordinary input handling, which read the same
held button, called `begin_move` again, and charged the caster for the ability once per frame
of the wind-up. `casting_costs_the_blood_mage_health` caught it at 90 health for a 45-health
move.

**Why the haul.** The old grab put its victim at the caster's arm's length on the frame it
landed, which reads as the game moving somebody rather than as an ability landing on them. It
is now a bind — about a sixth of a second where nothing moves at all — and then a trip at
forty metres a second, which crosses nine metres in fourteen frames. The bind is the part that
sells it: a pause before the pull is what makes the pull look like a consequence.

**The root came off entirely, and `Player::rooted` went with it.** The catch used to be a hold
followed by a root, and the root was the longer half. But "the target regains the ability to
move immediately after the grasp ends" is the trade that makes a hard stop fair, and a root on
the end of it is the opposite. Nothing else in the game applied a root — the Bulwark's
soft-control lattice is still unbuilt — so the field, its method, and the three movement gates
that read it are gone. `disabled()` is now `Stagger | Held`, which is a shorter sentence for
the same set.

**The whole catch is now shorter than her most expensive cast takes to come out** (26 frames
against Black spike's 30), and that inverts the old feel test. It used to assert the root
outlasted her *fastest* move's startup, so there was something to land inside it. It now
asserts the hold is shorter than her *dearest* move's startup, so there is nothing she can
start on reaction to it. The ability is for dragging somebody into a spike that is already in
the ground, and the punishment for a miss is that you paid forty-five health and put nobody
anywhere.

**One number is load-bearing in a way that is easy to miss.** The hold has to outlast the
haul: `(26 − 10) frames × 40 m/s ÷ 60 = 10.7 m` against the 9 m a full-range catch has to
cross. If that slips the other way, a long Grasp drops its victim mid-air halfway home, and it
fails silently at short range. `a_grasp_always_finishes_hauling_before_it_lets_go` holds the
four knobs together.

**Verdict** open. The specific worries: half a second of channel may be long enough that a
competent opponent simply walks out of the cone while watching the marker, which would make
the move worse at every range than the fixed ten metres was at one. And the pre-commitment the
design asks for — spike first, then Grasp — depends on the spike being worth casting at nobody
in particular, which is a question about Black spike rather than about this.

### 2026-09-13 — the wing became a blade, and the tip became a ball

**Changed** the Dual mage's autos, on every axis of their shape. The band: `wing_inner` from
0.16 of the reach to **0.75**, and `radius` from 0.45 m to **0.2 m**. The ring: reach from
1.65 m to **2.1 m**, arc from 0.42 turns to **0.22**, and the middle of it moved off her — 0.3
of the reach toward the *other* arm (`wing_offside`, new) and 0.1 of it forward (`wing_ahead`,
new). Where it stops: 0.073 turns off her centre line toward the punching hand
(`wing_finish`, new) instead of dead ahead. And the tip: a **bubble** of `wing_tip_radius`
(new, 0.5 m) at the foremost point of the ring, instead of the last slice of the section.

**Why** the shape was doing three things it was not meant to be doing, and each one had the
same root — it was described as a blade and built as a region.

*It was a filled disc.* The band ran from her own elbow (0.26 m) to full range on every frame.
With the hit test's slack on top — the attack's 0.45 m plus a body's 0.5 m — the hole was
gone entirely: the volume was a solid 150° wedge two metres across, and every part of it hit
for the same amount. Nothing about that is a blade. Three quarters out, and a fifth as thick,
leaves a band that is actually a band: it now comes to 0.91 m off her axis at its nearest,
which is just past where the fist finishes, and a body still has to be roughly where the blade
is rather than merely in the same quadrant as it.

*It was a circle drawn round her feet.* The ring was centred on her, so every point of it was
the same distance away and 150° of it wrapped most of the way around her. Shortening the arc
alone does not fix that — a short arc on a ring you are standing in the middle of is still a
piece of a halo. What fixes it is moving the middle: with it 0.63 m toward her other arm and
0.21 m forward, the blade comes in at 1.5 m beside the punching fist and swings out to 2.1 m
in front, so the distance from her *changes* along the sweep. Larger radius and a narrower
angle then make it shallow rather than round. Those three go together; any one of them on its
own does nothing much.

*Both autos finished in the same place.* The section closed on straight ahead, so the dark and
light punches — which are told apart by which arm threw them, and that is the entire mechanic
— ended on the same point of her sternum. It now finishes in front of its own hand: 0.30 m off
the centre line, where the hand is 0.18 m off it.

*And the "tip" was the widest thing the move had.* It was the last 38° of the section, at full
radius: metres of arc, catching the whole front of her, easier to land than the wing it was
meant to be a reward for. It is a ball at the end of the blade now — one frame, half a metre,
at the one point out in front that the wing deliberately stops short of. The spacing story
comes out clean: inside about 1.8 m the body of the wing catches you in front, past that only
the tip does.

**Everything above is a knob**, which is the other half of this change. Four new ones in the
Oven under *Dual mage* — `wing_offside`, `wing_ahead`, `wing_finish`, `wing_tip_radius` — and
the three that place the ring (reach, arc, radius) were already per-move. Numbers this
interdependent cannot be found by arithmetic; they have to be dragged while the move is in the
air. The one trap is that reach, arc and radius are **two** rows each, one per auto, and
moving one without the other silently unmirrors the class: `crates/sim/tests/dual_mage.rs`
fails if that happens, but it fails at the test rather than in the moment.

**The arena draws the band now**, not just its leading edge. That edge used to be a line from
her elbow to full range, which was a reasonable stand-in for the volume; with the band thin it
is a half-metre stub out at the rim, and the move went nearly invisible outside the debug
overlay. `place_wings` lays twelve radial bars along `hitbox.sector` — the same shape the hit
test reads — so the thing swept past you is the thing that decided whether you were hit.

**Verdict** open, and the numbers above are a starting position rather than an answer. The
specific worries, in order: 0.22 turns opening over six frames may now be too *little* travel
to read as an opening at all; the band may be thin enough that the auto whiffs against a
moving target often enough to make steering the meter frustrating, which would be the one
failure this class cannot absorb; and a ball tip with a body's radius of slack on it may still
be more forgiving than a tip should be.

### 2026-09-13 — Cataclysm stopped inventing hitboxes

**Changed** what Cataclysm does to a structure and to a fire pillar, without touching what
decides *whether* it hits either one.

A destroyed structure used to go up in `blast_cone`: an instant cone test of its own, with two
knobs of its own (`cataclysm_cone_cos`, `cataclysm_blast_radius`) that answered a question
none of the game's other attacks needed answered the same way. It is `crate::debris` now — the
structure breaks into seven pieces, fanned along the beam's own line with `debris_spread`, each
a small thrown projectile with a real flight time. What connects depends on range and on
standing inside the fan, the way a real spray does, rather than on standing inside a bubble
that already existed by the time anyone saw it.

A fire pillar torn loose used to become a `FireTornado` struct in its own array
(`crate::tornado`), with its own pull radius, its own damage, and its own stagger — a second,
nearly-identical description of a fire pillar's hitbox sitting a few files away from the real
one. It is `EffectKind::FireTornado` now, the same `Effect` slot the pillar already occupied,
reusing `pillar_volumes()` and the pillar's own `pillar_damage` tick outright (see
`state::burn_the_pillar`, shared by both). The only new number is `tornado_pull`; the pull
radius is just `pillar_volumes().0.radius`, because asking "how far does its pull reach" and
"how far does its burn reach" as two different knobs was answering the same question twice and
risking two different answers to it.

**Why** — reported after playing it: the tornado felt like a new hazard bolted onto the pillar
rather than the pillar itself let loose, and the blast's instantaneous cone gave a destroyed
structure no visible moment between "there" and "everyone around it is already hit." Both were
symptoms of the same thing, a hitbox invented for the occasion instead of reused from one that
already existed and was already trusted.

**Removed** five scalars (`CataclysmConeCos`, `CataclysmBlastRadius`, `TornadoPullRadius`,
`TornadoDamage`, `TornadoStagger`) and the whole `crate::tornado` module. **Added** eight —
`DebrisSpeed`, `DebrisRange`, `DebrisRadius`, `DebrisSpread`, `DebrisDamage`, `DebrisStagger`,
`DebrisBlockstun`, `DebrisKnockback` — and `crate::debris`, built the same way `crate::bolt`
is: a real velocity, stepped frame by frame, because a thrown thing is not a placed one and
`crate::effects`' rule against a velocity was never about this.

**Verdict** not yet played against another person. The shotgun's spread (`debris_spread`,
1/16 turn either side of the beam) and per-piece damage are first guesses — what "close range
is a wall of debris, far range is a couple of stray pieces" actually feels like at the stick
has not been tested.

**Corrected the same day.** The first version of the fan rotated the beam's own horizontal
bearing and left its pitch untouched — the same mistake `aim.rs` exists to keep out of the
game, a ray built from a fixed axis rather than from the line of effect itself, here reappearing
one level down from where that file already guards. Aimed level it looked right and was
accidental; aimed up or down the pieces all kept the beam's own pitch and only fanned out
sideways, which is a slice of a horizontal plane, not a cone. Fixed by building the fan out of
`crate::math::frame_about` instead — sideways and up **square to the beam itself**, however it
is pitched, the same construction the Grasp's arms already spread around their own line of
effect with — and pinned by `the_debris_cone_stands_in_space_rather_than_lying_flat`, which
aims up and checks that the pieces do not all share the beam's own pitch back.

### 2026-09-13 — the Ridgeback, rebuilt

**Changed** the creature is about a third larger and stands four and a half
metres at the back on long legs; ten welded boxes became an eighteen-bone
skeleton with eighteen parts, animated through the factory; a second weak point
(the nape) and four breakable feet; a stumble between the flinch and the topple;
recent-damage thresholds for crowd control and for interrupts; per-move
lockouts.

**Why** the fight had three problems a player would name in the first minute.
The animal read as fifteen cubes glued together, because it *was* — its legs
never moved. Getting on it was a free action from the tail, so the climb was not
a decision. And the back was where the fight happened: ride share ran to 70% and
the ground game was optional.

**The height is in its legs, not its bulk, and that is the whole design.** A
standing full hop reaches 4.14 m; the back sits at 4.57. So the back is out of
reach, and the *only* part of the animal a fighter on the floor can touch is its
feet. That one geometric fact is what turns the ground phase from a chore into a
route: break a foot and it goes down on a knee for nearly two seconds with its
shoulders at 2.35 m. Making it *longer* instead would have cost the arena more
room than it has; making the legs longer cost nothing and produced the ground
game for free.

**What the harness caught, and what it did not.** `legs broken: 0` sat in the
fight report through three separate changes to the hunter's station before the
cause turned up, and the report could not tell the two possible causes apart.
Adding `damage into feet` and `worst foot` took an afternoon of guessing down to
one run: the bot was hitting nothing at all. **A level shot from somebody on the
floor goes under the belly.** The creature is on stilts and the old station was
three and a half metres out, where there is no creature at that height. A
measurement that cannot distinguish "never tried" from "tried and failed" is
worth about as much as no measurement.

**Two animation bugs that were gameplay bugs.** Both are the same shape: the
grip test reads *acceleration*, so anything that puts a corner in the pose
throws people.

- The baked table stores samples and reads them back with a lerp, so every join
  between samples is a corner. At twelve samples per phase a 34-frame shake had
  three frames between corners and threw braced riders on single frames that had
  nothing to do with how hard the animal was moving. Thirty-two samples — about
  one per frame of the longest phase — is now the rule, and the sample count is
  documented as a balance number rather than a file-size one.
- The shake's last startup key and its first whip key were both authored exactly
  on the phase boundary. Two keys at the same instant with different poses are a
  *step*, and a step is an arbitrarily large acceleration. It threw braced
  riders off the hips before the shake had started. The design document had
  already recorded this lesson once, about the old procedural pose, and it was
  re-learned anyway.

**A third that was neither.** Riders standing where two mountable parts overlap
ping-ponged between them once a frame — each swap moves the body a few
centimetres, which the buck reads as an enormous acceleration. The parts overlap
on purpose (a staircase with gaps is not a staircase), so the fix was in the two
rules that decide what you are standing on: a tread you could *step onto* is a
floor rather than a wall, and the step-up probe looks **upward only**.

**Reverted: the slam as a body slam.** Widening its hit volume to a 5.2 m ring
around the creature so it would reach its own back made standing anywhere near
it lethal — four slams is a dead fighter — and the scripted hunter died at 37
seconds having dealt 900 damage. Put back to a front slam, and the rule it was
trying to buy is now stated the other way round and pinned by a test:
**nothing the creature throws can reach its own back.** A rider is threatened by
the buck and by nothing else. That is a better rule than the one it replaced,
because it is what makes riding a phase with its own vocabulary rather than the
ground game at a different altitude.

**Reverted: the tail sweep as a mount route.** The tail attaches at 3.6 m on a
long-legged animal, so no amount of drooping it during the sweep's recovery
brings its *base* low enough to matter — the sweep's tip goes to the floor and
the part you stand on does not. The claim came out of the design document. The
sweep earns its keep a different way: its hitbox is 1.6 m and jumping it needs a
*held* jump, which is a precision test the tapped hop the bot was doing does not
pass.

**The thresholds.** `strain` is damage taken recently, decaying a couple of per
cent a frame, so a burst fills it and a trickle does not. Above one bar the
creature is susceptible to crowd control, weakened; above a higher one, a hit
breaks it out of what it is doing, live hitbox included. Both bars fall by up to
70% as its health does. The intent is the arc of a hunt — methodical while it is
fresh, frantic once it is not — and the reason it is a threshold rather than an
immunity is that half of every kit is otherwise dead weight in a hunt, and the
two halves of the game stop teaching each other anything.

**Verdict** open, and specifically open on three things. Ride share fell from
around 70% to 17%, which is the change this was most meant to produce, but 17%
may now be too *little* — the climb is expensive and the reward may not be worth
the trip. The tail hop has twenty centimetres of margin beside an animal that is
turning, and it is the only route up that does not have to be earned; it may be
the only one anyone finds, or it may be too hard to find at all. And the scripted
hunter now wins five of six where it used to win about half, which is the bot
getting a second plan rather than the creature getting easier — but a bot that
wins is a worse measuring instrument than one that does not.

### 2026-09-13 — the debris cone starts at the middle, and stops where it should

**Changed** two things about Cataclysm's debris, reported the same day the cone fix landed.

`stones::destroy` used to report a stone's `at` — its base, on the floor — as the point its
debris radiates from. A neutral, level cast throws roughly half its pieces on the downward
side of the cone, and a piece that starts on the floor and immediately points down is inside
the ground on the very first frame it exists rather than a moment later. It now reports the
stone's middle instead: `at.y` plus half of `standing_height()`. Nothing about the cone itself
changed; the point it radiates from just moved to the point that is actually inside the thing
that broke.

**Verified**, not changed: a piece already stopped dead at the first stone, fighter or the
quarry it met, per `debris::step`'s existing match on `Contact`. That was reported back as a
requirement worth pinning rather than a bug, so `debris_shatters_on_the_first_stone_it_hits`
now says so directly — a second, farther stone survives a blast that breaks the near one, and
a fighter standing behind it takes nothing.

### 2026-09-13 — the blade comes home to a person, and the Grasp's marker stops jumping

Two fixes from playing the Blood mage, and they are the same fix twice: an ability was
pointed at a *place* where the player was thinking about a *thing*.

**The Bloodletter now returns to the mage rather than to the spot she threw it from.**

```
before   out along the aim to `reach`, then back down the same line to the
         point it left, whether or not anybody was still standing there
now      out along the aim, unchanged; back to `Effect::home`, which is the
         caster's live ability origin, refreshed every frame
```

**Why** a catch is a thing that happens between two objects. The old return arrived at a patch
of air the mage had walked out of half a second earlier and paid her anyway, which made the
flight home a formality rather than the risk the ability is built on. The outward leg does not
move and must not: the throw was aimed, and re-aiming an ability that is already out is the
mistake `aiming.md` exists to prevent. Only the way back follows anybody.

**The return is a fraction of the way home, not a speed**, and that was a real choice. A chase
at a fixed speed reads more dramatically and could be *outrun* — but then the ability's
failure case would be moving, and moving is the one thing an aggressive sustain class has to
be able to do. The lerp arrives on the frame it is supposed to however far she has run, so the
risk stays where the design put it: survive the flight, do not stand still for it.

**The Grasp is now aimed as a swing rather than as a skillshot**, and its channel went from
half a second to a full one, starting at melee.

```
before   skillshot, 3 m to 10 m over 30f -- marker at the far end of the
         crosshair's ray, so it sat on whatever the ray hit
now      swing, 1.5 m to 10 m over 60f -- a ray off the chest along the facing,
         pitched by the camera through the standing 45-degree dead zone
```

**Why** the marker was showing the arena instead of the cast. A skillshot's far end is
whatever the ray stops against, so looking a few degrees further down moved the marker by
metres, and the depth the hold had bought was invisible underneath the geometry. It also
started at three metres, which meant it *appeared* out in the middle distance rather than
leaving the body — so the one thing the channel is for, watching the range grow, was the one
thing you could not see.

As a swing it has nothing to stop against: its length is the hold and only the hold, which is
what `the_marker_is_as_far_out_as_the_hold_and_nothing_else` pins, sweeping the pitch from
above the horizon to straight down. The dead zone is doing the real work. The camera sits
above the shoulder, so a player looking at somebody at their own height is already looking
slightly down; level through the first 45° means the marker runs flat across the floor at
chest height through the whole range a fight happens in, and the higher neutral camera angle
is *why* the depth reads at all. Past 45° the cast goes into the ground, which is a mistake
the player can watch themselves make.

**`one_aim.rs` had to be loosened, carefully.** It asserted that anything leaving a travelling
effect behind must be a skillshot, on the grounds that a swing "would travel along the body's
flat facing and quietly ignore the crosshair". That was written before swings were pitched at
all. The real rule is that a thrown thing has to be pointed by the player, and there are two
ways: the crosshair picks the point, or the body picks the line and a channel picks the
distance. Swing-without-a-channel is still the bug, and is still an assertion failure.

**A second of standing still is the cost**, and it is bigger than the half second was. The
whole wind-up is a telegraph in plain view of somebody who can walk out of the cone.

**Verdict** open on both. The Grasp worry from the last entry gets sharper rather than
resolved: a full second is a long time to hold still, and if opponents learn to simply back up
while watching the marker the move is worse at every depth than the fixed ten metres was at
one. The blade's worry is the opposite — a return that always lands may have made an auto
attack too safe for a class whose whole economy is supposed to be a gamble.

### 2026-09-13 — your own body dims under the crosshair, it does not vanish

**Changed** the fade on the fighter you are driving. There have always been two reasons to stop
drawing them and the stronger one wins; now only one of the two can reach the whole body.

```
before   crosshair fade and near-eye fade both ran 0 -> 1, and either could
         take the body away on its own
now      the crosshair fade is capped at 60% ("Body dims to"); the near-eye
         fade still runs to 100% and is the only thing that does
```

**Why** it was firing at full strength with the camera nine metres away. Looking down — which is
most of the time, since the floor zone runs to −85° — puts your own body high on the screen and
squarely under the reticle, so the crosshair reason saturated while the arm was at full length.
The test that pins this now reports the old behaviour as *"at −85 deg the eye is 8.30 m out and
the body is gone anyway"*, which is the bug in one line.

**The argument is about spacing, not rendering.** A body that winks out takes its own position
with it, and where you are standing is what every judgement about range is measured from: how
far the other fighter is, whether you are inside your own reach, which way you would dodge. A
dim gets the body out of the way of the shot without costing the player that. When the eye is
genuinely close there was nothing to see anyway, so that case still earns the whole body — and
it is what carries the first-person handover, which is unchanged.

**Verdict** open, and the number is the open part. Sixty per cent is a guess: enough to see the
reticle and whatever is behind the shoulder, not so much that the body stops reading as a body.
If aiming past your own head still feels obstructed the knob goes up, and the thing to watch for
at the top of its range is the fade starting to read as a disappearance again.

### 2026-09-13 — the tornado's pull did not actually pull

**Changed** two things reported the same day: the tornado came out already at full size instead
of growing into it the way its pillar does, and standing inside it did not drag you anywhere.

**The pull was real and did nothing.** `apply_effect`'s `FireTornado` arm was subtracting an
inward acceleration from a caught fighter's velocity every frame it had him, exactly as
described -- and a fighter free to act has his own velocity *set* from the stick every frame in
`step_player`, not accelerated, which overwrites whatever the pull just did before the next
frame's position update ever reads it. The fix is not in the pull, which was always correct: the
first frame the tornado reaches somebody now lands a real stagger -- the same eruption a fire
pillar already throws the instant it is cast, reused rather than invented -- and a stunned
fighter's velocity only *decays* each frame (`step_player`'s `stunned()` branch), which is the
one state the pull can actually accumulate inside. Once the stagger runs out he is free to walk
or jump clear and the pull cannot stop him -- it was never meant to hold him forever, only to
buy itself the window a fighter's own legs would otherwise close instantly.

**The size was frozen because resetting `age` to match it broke the flight.** The first fix
tried was forcing `pillar_volumes` to treat every tornado as fully grown, which read as an
instant pop rather than the pillar it came from continuing to erupt. Reusing `progress()`
instead -- the same curve a standing pillar grows on -- meant `age` had to keep counting from
when the pillar was first planted, not reset to zero at the moment it starts moving. But
`tornado_pos` was built on that same `age` to work out how far it had flown, so a pillar caught
after sitting for sixty frames would report a live position sixty frames' worth of travel away
from where it actually was -- an instant teleport the moment it converted, invisible in the one
early test that lit a tornado at age zero and never noticed. `Effect::banked`, otherwise only
the blade's, now takes a snapshot of `age` at the exact frame a pillar is torn loose, and flight
is measured against `age` minus that snapshot instead of `age` alone -- growth keeps its full
history, flight starts counting from zero.

**Two of the three test fixtures were also part of the bug.** Lighting a tornado exactly where a
fighter stood and checking it pulled him within a handful of frames happened to pass regardless,
because the assertion read a value the very next real frame would have overwritten anyway -- it
never ran long enough to notice the drag did not last. And a fixture that wants to test the pull
and the tick, not a chase, cannot light the tornado freshly small either: freshly lit is also
freshly tiny and already travelling at full speed, so standing exactly on top of one is closer
to a test of whether it outruns you. Both fixtures now light it already well grown, with
`Effect::banked` set so its flight starts from zero regardless of how grown it already is.

**Verdict** open. This is meant to be the class's answer to somebody sitting at mid range: get
caught and the tornado drags you further from the Elementalist as it travels, the opposite of
every other catch in the game, rather than merely burning you in place. Whether the stagger is
long enough to matter and short enough to still feel escapable, and whether the wide base
actually forces the jump-or-air-dodge choice the design wants rather than being outwalked, has
not been played against another person.

### 2026-09-14 — the Reaver's right click was at the mercy of her own kit

**Changed** Send shadow now **cuts short the recovery of any move it is pressed during**, and
the press itself is **remembered for twenty-four frames** rather than read on one frame and
thrown away — cleared if she is hit in the meantime. New knob:
`reaver.send_shadow,_press_stays_live`. Right click also grew a **press edge** on this class,
which it never had.

**Why** Reported: sending the shadow feels clunky to chain, because it has to be timed just
right.

It did, and the arithmetic says how badly. Right click reached the move table only on a frame
she was `Action::Free`, and her four moves are nineteen, twenty-five, thirty and forty-six
frames long. A press landing anywhere inside one of those was **discarded** — there was no
buffer of any kind — so the input was live on roughly one frame in twenty. That is not a hard
input, it is an invisible one.

The fix is two rules, and they are the same rule twice. Every other button in the game is an
attack, and an attack eaten by another move's frames is the game correctly saying *you were
busy*. **Right click is not an attack.** It is where her second body stands, and the line
between the two bodies is the class's escape from both the ordinary limits on where she can be
and the ordinary limits on what she can reach. A mechanic the rest of her own kit can lock her
out of is a mechanic behind a timing test.

**The cancel is the Champion's, arrived at from the other end.** Rush ends a recovery because a
linear class needs a way to stop being linear; this ends one because a positional class needs
its position to be available. Recovery only — a cancel reaching into a startup would let her
take a committed swing back after throwing it, and one reaching into a stun would make the
mechanic an answer to being hit.

**It needs no charge behind it, and that is the interesting part.** Rush costs one, which is
what stops it being free. This costs nothing and is still not free, because Send shadow is
twenty-five frames and the longest recovery it can cancel is Executioner's twenty-six: cutting
one short and spending the rest on the mechanic leaves her busy for about as long either way.
The trade is a recovery for the mechanic, not frames bought back — so a blocked Executioner is
still a punish, and `feel.rs`'s punishability property is untouched rather than quietly dodged.
`reaver.rs` now pins that as a relationship, because it is the thing that would break silently
if Send shadow were ever shortened.

**Twelve frames was the first value, and it was wrong in an instructive way.** With the recovery
cancellable, the only frames left that can eat a press are the startup and active ones of
whatever she is already throwing. Slash is the move she chains from constantly and takes ten
frames to become cancellable, so twelve was derived from it — and the test written alongside was
derived the same way, so it passed.

It silently dropped **`Q` then right click**, which this kit's own document calls the class's
biggest turn. The Guillotine takes sixteen frames to become cancellable, so the press expired one
frame before the lotus could be dragged. Deriving a buffer from the *commonest* input rather
than the *longest wait* is how you get a number that works everywhere except the combo — and
writing the test from the same derivation is how you get one that agrees with you.

Twenty-four comes from the longest wait instead: Executioner's sixteen and four put the first
cancellable frame twenty-two after the move begins. The test that pins it now **presses the
button** on every one of her moves rather than reasoning about phase counts, because how long a
move takes to become cancellable is a fact about the state machine and a formula for it in a test
is a second copy of that machine waiting to disagree. Measured: a single press is now honoured on
**every frame of every move she has** — 23 of 23 across Slash, 34 of 34 across Guillotine, 50 of
50 across Executioner. It was one frame per move cycle.

**Being hit clears it.** That is what lets the memory be this long. The buffer exists so her own
kit cannot eat the mechanic; a stun is not her own kit, it is the opponent's reward. Kept across
one, a press from before the hit would send the second body away on the frame she is most likely
to want it, on an input she gave in a situation that no longer exists.

**A latent bug went with it.** Right click was level-triggered here — `clicked_move` read the
button, not a press — so *holding* it sent the shadow, then recalled it, then sent it again,
every twenty-six frames. That is exactly the bug the 2026-09-11 entry below fixed for `E`, and
it came back the day this class's mechanic moved onto the mouse, because a click had no edge of
its own. It now has one, in the snapshot beside `mechanic_held` so rollback can recompute it.

**Verdict** open, and the number to drag is the buffer. Twenty-four frames is long for an input
buffer — most games sit around a quarter of that — and the justification is that this is not a
general input buffer but a specific claim that *her own kit may not lock her out of her own
mechanic*. If it turns out to be too generous, the thing to watch for is a shadow moving on a
press the player had already given up on, which is worse than dropping one, because where the
shadow stands is the whole class.

What has not been played is whether the cancel changes what she *throws*. The prediction is that
Executioner stops being a move you only use on a read — its tail is the reason it is scary to
commit to, and that tail now has somewhere to go.

### 2026-09-14 — the tornado stopped early, dragged nobody, and once vanished outright

**Changed** three things reported against the fix above, once it actually reached a player: the
tornado still ran out of time far too early depending on when it was caught, standing inside it
still left you behind rather than riding along, and aimed through a fire pillar at a dais corner
it would vanish instead of converting at all.

**Growth and expiry were still the same clock.** The previous fix made `age`/`life` answer "how
grown is it" continuously across the moment of conversion, which was right -- but `step_effects`
was still reading that same pair for "has it burned out." A pillar caught late in its own life,
already most of the way to `pillar_life`'s 180 frames, got almost no time to travel before its
budget ran out; one caught early got to travel for a while but stopped growing and expired at
the same instant, because both questions were being asked of one number. They are different
questions: how grown a tornado is was never in doubt, and how long it gets to keep existing once
it starts moving does not need to be the same budget the stationary pillar was already spending.
A new scalar, `tuning::tornado_travel_life`, answers the second question on its own clock --
frames since `Effect::banked` was stamped, the same reference `tornado_pos` already measures
flight from -- so every tornado gets the same travel window regardless of how much of its
pillar's original life was left when it was cut loose.

**The carry was an acceleration fighting a decay it could never win.** The previous entry got the
*window* right -- a stunned fighter's velocity only decays, rather than being overwritten from
the stick -- but the carry itself steered his velocity toward the tornado's own travelling
velocity by a fixed acceleration each frame, the same shape `air_accelerate` uses for a jump.
Solved for steady state, an acceleration racing a proportional decay converges on a velocity far
short of the thing it is chasing: `stun_decay` at 0.86 loses 14% of whatever speed a fighter has
every single frame, so the carry had to out-accelerate that loss before it could add anything at
all, and never fully did -- caught, a fighter fell further behind the live centre every frame
instead of riding alongside it, the exact bug the pull was supposed to have already fixed. The
fix does not go through velocity at all: the tornado's own per-frame displacement is added
straight to a caught fighter's position, the same way a rising stone carries whoever is standing
on it (`stones::resolve_body`) rather than accelerating them upward to match its speed. Moved
this way a fighter cannot fall behind, because he is not chasing the tornado's velocity, he is
*being* the tornado's velocity for that one frame. The radial pull still runs, now measured
*after* the carry rather than before it -- against the position it just moved him to, not the one
he stood at the top of the frame, which used to read the tornado's own stride as radial distance
to close and yank him forward on top of a ride that was already keeping pace on its own, the
source of a slow drift the carry's own test caught once the acceleration bug above no longer
buried it. What the pull is actually for, once the carry is doing its job, is genuine sideways
drift off the axis -- and the fixture that exercises it now stands a fighter off to one side of
the tornado's line rather than dead on it, because dead on it the carry alone already keeps him
matched and there is nothing left for the pull to close.

**The dais corner sent it through the floor.** `fire_the_cataclysm` was assigning a tornado's
travel direction straight from the beam Cataclysm was aimed along -- `beam.dir()`, whatever pitch
that beam happened to be flying at. Most casts are level, because most of what Cataclysm meets
reads as `Met::Ground` and `aim::skillshot_path` flattens the beam's height for exactly that
reading -- but aimed at the elevated top corner of one of the arena's two low platforms, sight's
ray can miss the platform's box outright (a corner is a knife's-edge for `ray_hits_box`) and
land on the max-range sphere instead, at whatever pitch the crosshair was actually on. `Met::Reach`
does not get flattened, so the beam it produces flies wherever it was pointed, pitch included --
still capable of meeting a fire pillar's column along the way, but handing the tornado a
travelling direction with a real vertical component. `tornado_pos` had never needed to guard
against that: it adds `dir` scaled by distance travelled straight onto `pos` every time it is
asked, with nothing keeping the answer at ground level, so a tornado lit with even a small
downward tilt walked its own centre through the floor within a couple of frames and
`arena::inside` read that as having wandered off the map -- the pillar it came from simply
vanished, having never been seen to move at all. A fire tornado is a ground hazard the same way
the pillar it came from was, so its direction is flattened to the horizontal plane and
renormalized the moment Cataclysm tears one loose, the same trick `bolt::light` already uses for
a shot that might have been aimed with pitch of its own. Reproduces consistently: stand beside
either platform, plant a pillar at its base corner, then aim Cataclysm at the platform-top corner
directly above that one.

**Verdict** open. The travel-time and carry fixes make the tornado read as one continuous move
the way it was always meant to, rather than one whose window and drag depended on when in a
pillar's life it happened to get caught. The dais fix is a straightforward correctness bug once
found, but it was found by a player, not a test -- the four kinds of aiming in `aim.rs` all still
assume a caster on flat, unobstructed ground when it comes to what a *travelling effect* born
from an aimed cast should do with that aim's pitch, and Cataclysm's tornado is the first thing in
the kit that keeps a cast's direction alive after the cast itself is over. Worth watching for the
same shape of bug anywhere else a moving effect inherits a beam's raw direction.

### 2026-09-14 — the lotus was a fountain, not a flower

**Changed** The Guillotine's six blades now leave the shadow's **midriff** rather than its
feet, open in **one horizontal plane** rather than arcing up and back down, and come home on a
**spiral of their own turning against the way they opened** rather than retracing the arm they
came out on. `reaver.lotus,_how_high_they_arc` is now
`reaver.lotus,_height_off_the_shadow's_feet` (1.2 → 0.9); new knob
`reaver.lotus,_turn_coming_home_(turns)` at 0.28, against an outward curl of 0.14.

**Why** Reported: the balls start at the feet of the shadow and jump up in a spiral. They did,
and the arc was doing three things none of which anybody asked for.

**The height was a real hitbox bug wearing an animation's clothes.** `arch(out)` is zero at both
ends, so a blade was at floor level when it left *and* at floor level at full extension, peaking
only in the middle. The volume is a ball of radius 0.45; at the reach where the ability does its
work it was centred on the ground and therefore half buried. The blades now sit at 0.9 m — waist
on a 1.8 m fighter, below the 1.25 m a cast comes out of, because these come out of the shadow's
middle rather than its hands — for the whole of their life.

**And a flower whose height changes while it turns is hard to read.** The thing a player has to
judge is whether they are standing in a plane that is sweeping toward them. A volume at a fixed
height is one you decide about once; one that rises and falls while it rotates has to be
re-read every frame, and in third person at four metres out that decision is not available.

**The return was a rewind, not a closing.** Bearing was `θ₀ + curl · extension` and the return
just ran `extension` backwards, so the blade unwound onto the exact bearing it left on and
retraced its outward arm. That is a hit test problem as much as a look: ground a blade has
already crossed is ground whose occupants have been cut once and have had the whole 40-frame
hold to walk off it, so a retraced return could only catch somebody who stepped back into the
same line. The turn is now its own number and its own direction — out `+curl`, home `−uncurl`
with `uncurl > curl` — so the blade crosses its starting bearing and the way home sweeps floor
the way out never touched.

Mechanically that meant splitting reach and bearing out of a single `extension` parameter, since
extension 0.5 no longer says which way the blade is pointed — it depends on whether you are on
the way out or the way back. `lotus_head` takes an **age** now and derives both from the phase,
which also deleted the `lotus_extension_before` trick of cloning the effect with its clock wound
back a frame.

**Verdict** open, and the number to watch is `uncurl`. At 0.28 each blade sweeps about 100° on
the way home against 50° on the way out, and with six blades 60° apart that means the return
covers the full circle with overlap — so a victim near the shadow can be caught by about 1.7
blades on the way back where they were caught by about one before. Per-blade damage is
deliberately small and the return is already only 70% of the way out, so this is a change in
the right direction rather than obviously too much, but it is the first thing to drag if the
recall reads as a blender. The narrow band that winds past the start *without* full coverage is
0.14–0.167, which is not much room; the honest alternative if it is too strong is fewer blades
rather than a smaller turn.

Nothing here touched `lotus_radius`, the three clocks, or the damage, so the ability's timing
and reach are exactly what they were.

### 2026-09-14 — the repeat lockout

**Changed** a move you have just thrown cannot be thrown again for 30 frames.
Per ability, not global: the rest of the kit is untouched the whole time. A new
`Offence / Repeat lockout` scalar in the Oven carries the shared number, and a
`Repeat lockout (%)` column on every move scales it — 100% on all thirty, so
today the rule is exactly "30 frames, everything".

**Why** frame data prices a move against time, so the cheapest move in a kit is
the correct one to throw most of the time, and six abilities that a player uses
one of is not a kit. Nothing in the game pushed back against repetition except
the creature, which has had a variety penalty on its own move choice since it
was built — the player side had nothing equivalent.

The shape of it was chosen against the no-cooldown decision in
[combat-kernel.md](combat-kernel.md) rather than around it. Two properties do
the work. It only ever holds the ability you just threw, so the question is
never *do I have anything* but *what else have I got*. And the clock starts on
the frame the move **comes out** rather than when its recovery ends, so a move
that already commits you for longer than the lockout never notices it: at 30
frames that is every committed heavy in the game, and what is left charged is
every auto and fast poke. Bash pays 13 idle frames on top of its 17; Grapple's
53 frames of commitment pay nothing. The rule taxes cheapness, which is what
made repetition correct in the first place.

Two abilities needed the rule bent, and the bend is the interesting part. Send
shadow and the Guillotine lotus are *activated twice* — out and home, hung and
dragged — and a lockout armed by the first press is a lockout on the second. So
an ability is not counted as used until it is spent: while any of it is still
out in the world its lockout is parked at full rather than running down, and the
press that spends the second activation is exempt. Which button reactivates
which ability is declared in `moves::reactivates` rather than inferred from what
a move leaves behind — inferring it answers yes for the fire pillar and the
black spike, neither of which can be pressed again at all.

**Verdict** open, and open in a way that needs a person rather than a test. What
the harness can say is that the relationships hold: nothing is ever locked out
for longer than it takes to throw something else, one press can never cost more
than one option, and the Reaver's recall still gets through. What it cannot say
is whether half a second reads as *use your kit* or as *the game just ignored my
click*. The HUD's frame-data line names the locked ability and counts it down,
which is there for exactly that judgement — if the answer turns out to be that
the lock needs to be felt rather than read, that is a sign the number is wrong
rather than a sign the readout needs to be bigger.
### 2026-09-14 — Send shadow: instant, and in exchange it cannot take your frames

**Changed** `send_shadow.startup` to 1 (from the 0 it was baked to, and the 8 before that).
`send_shadow.damage` back to 70 from 0. `hitstun`, `blockstun` and `knockback` stay at 0. New
`Hit::interrupts`, false for the recall and true for everything else. The `reaver_mechanic`
clip's gather is now conditional on there being startup frames to hold one.

**Why** Reported: the shadow feels much better at a frame of startup, otherwise it feels like
input lag — and to compensate it must have no immediate effect on struck enemies, or it becomes
the Melee shine and a more oppressive one, because not even her own abilities gate it. She would
never have to fully commit: hold the shadow for an opportune interrupt in case any of her own
abilities put her in a bad position.

That is exactly right, and the second half was **not** achievable by tuning. `apply_hit` writes
`Action::HitStun` over whatever the victim was doing regardless of the number, so
`HitStun { left: 0 }` is one frame of nothing and a *cancelled attack* — a full interrupt with a
zero on it. Zeroing `hitstun` looked like it removed the interrupt and removed only the stun.
Hence a real flag rather than a number, and it is about frames rather than force: knockback is
still whatever the number says, because moving somebody is not the same as stopping them.

With a lever that works, the damage does not need to be zero. The mechanic's own description of
itself is a second body dashing home through anything in the way, **cutting and slowing it** —
what it gives up is the interrupt.

**One frame rather than none.** Zero is not faster in any way a player can feel and it leaves
the animation nothing to put the release on: `phases()` is `(s-1, s, s+a)`, so a zero startup
collapses the wind-up frame and the strike frame onto frame 0 and the body has to teleport into
the gesture. It was breaking two clip tests for exactly that reason.

The clip's gather is conditional now, which is the honest authoring: a wind-up is frames the
opponent gets to read, and at one frame of startup there are none. **A body that visibly gathers
before a move that cannot be reacted to is the animation lying about the frame data.** Drag the
startup back up in the Oven and the gather comes back with it.

**Verdict** open. The prediction is that this is the shape the mechanic wanted all along — the
press answers instantly, and what it buys is position rather than tempo. The thing to watch is
whether the recall now feels *weightless* going through somebody: a cut with no stun and no
shove is a strange sensation, and if it reads as passing through them rather than through them,
the answer is probably a visual one rather than putting the stun back.

### 2026-09-14 — the blades were beach balls

**Changed** The Guillotine opens **twelve** blades rather than six, each a **disc** rather than
a sphere, at **half the damage**. `lotus,_blade_radius` 0.45 → 0.22, new
`lotus,_blade_half-thickness` at 0.08, `guillotine.damage` 40 → 20. `Effect::struck` widened
from `u32` to `u64`.

**Why** Reported: the hitboxes should be discs or short cylinders, because they are supposed to
be shuriken-like blades — right now they are really big and feel like beach balls.

They were. A sphere of radius 0.45 is wider than a fighter's own body and reached from the shins
to the chest, so six of them was less a flower than a ring of boulders. A blade is now wide in
the plane the flower lies in and barely there across it, which is what a shuriken thrown flat
actually is, and the hit test says so: width measured flat against the swept line, height a thin
slab that has to overlap the body.

**The slab is the interesting half.** The flower is planar and at waist height, so a blade this
thin is something a fighter can **jump** — which the old sphere was not. That is counterplay the
ability's own description always implied and never delivered.

**Twelve at half the damage is close to the same ability, and that is deliberate.** Doubling the
count halves the angular spacing while halving the radius halves each blade's coverage, so what
one victim actually eats barely moves; what changes is that the danger is the shape rather than
any one thing landing. Measured against a standing dummy: 66 damage at a metre out, 46 at two,
13–46 at three, 0–33 at four — and at the rim **where you stand relative to a petal is worth the
whole difference**, because twelve arms at full extension are far enough apart to stand between.

That gap is a feature and the reason the blades are not simply widened to close it: near the
shadow the flower is solid, at the rim it is petals, and reading which you are in is the spatial
decision the class is made of. It is worth watching that it does not read as the ability
*whiffing* rather than as the player having positioned well.

**`Effect::struck` had to grow.** The mask packs one bit per part per victim; at six blades and
three victims that is eighteen bits and a `u32` fitted exactly, at twelve it is thirty-six and
would have truncated in **silence** — a blade sharing a bit with another goes quiet the moment
that one lands. There is now a `const` assertion beside `LOTUS_BLADES` so the next count change
fails to compile rather than failing to cut.

**Verdict** open on both the count and the rim gaps. `lotus,_blade_radius` is the knob if the
petals turn out to be too easy to stand between, and the jump is the one to watch in play: it may
turn out that a flower you can hop is a flower nobody respects.

### 2026-09-14 — half the flower was invisible

**Changed** `EFFECT_PARTS` in the renderer is counted from the widest effect rather than
written down. No tuning moved.

**Why** Reported: only the first six blades have a model, the rest are just red discs.

The renderer spawns a fixed pool of meshes per effect and asks `effect_piece` for each one.
That pool was a literal `6`, with a comment saying "six, which is the Guillotine lotus: one
blade each" — correct on the day it was written and quietly wrong the day the flower grew to
twelve. The back six blades had no mesh to be drawn with. The red discs were the debug
overlay's own outlines, which loop over `LOTUS_BLADES` and so were drawing all twelve
faithfully; with `F1` off there was simply nothing there.

**It is worth being clear about which half was wrong.** The hit test was right the whole time —
twelve blades, cutting. So the ability was doing exactly what the entry above describes and
half of it was doing so invisibly, which is the worst version of this bug: not a thing that
fails to work, a thing that works with nothing on screen to say it is there. Everything in the
previous entry about how the flower *reads* was written against a picture nobody could see.

The rule this belongs under is the standing one about the overlay drawing what the hit test
uses, generalised: **anything that walks the parts of an effect has to get the count from the
effect.** The overlay already did. The renderer had its own copy, and a copy of a count is a
count that goes stale. It is derived now, so a thirteenth blade would widen the pool rather
than fall off the end of it — and the test beside it asserts the pool covers every piece
`effect_piece` will answer for, which is the property rather than the number.

**Verdict** kept, and it means the previous entry's verdict is still genuinely open: the thing
it describes has not actually been seen yet.

### 2026-09-14 — the dash to the shadow only ever went along the floor

**Changed** Three things about `shift` + forward on the Reaver, all of them the same
sentence read properly — *the dash goes to where the second body is*.

1. **It answers in the air.** A forward airdodge pointed at the shadow is the dash. It
   spends the airdodge and keeps the dash's own frames.
2. **It drives all three axes.** The line she flies is the straight line between the two
   bodies; gravity and the arena are off for the crossing, and she arrives on the shadow's
   own spot rather than within half a metre of it.
3. **Only a total obstruction refuses it**, which is a new question in `aim.rs`:
   `aim::clear_between`.

Plus a fourth thing that is new rather than fixed: **the carry**, a window the arrival
opens in which a jump takes the dash's speed up with her. New knob,
`reaver.dash_carry,_the_jump_window`, at ten frames.

**Why** Reported, in three parts, and the third is the one that explains the other two.

**The air.** Being off the floor is the commonest reason she is not standing where she
wants to be. A class whose mobility switched off the moment she jumped had the mobility in
the wrong place — the airborne branch of the dodge simply never asked whether the crosshair
was on the shadow. It asks now. It is the *airdodge*, aimed, so it costs the airdodge: one
commitment per airtime is the rule that keeps a jump from becoming flight, and the dash is
a bigger commitment than the one it replaces rather than a free extra. What it does not
take is the airdodge's shorter sixteen frames, because a dash has somewhere to *be* and a
tail cut short strands her halfway.

**The dais.** `dash_drive` zeroed the vertical and `step_her_dash` measured the gap flat,
so a shadow standing on a platform was, to the dash, a shadow standing on the floor
underneath it. She ran at the side of the thing it was standing on and the arena stopped
her — and then "arrived" was never true, so she never picked the shadow up either. The
fix is to drive the real gap and measure the real gap. It brings two consequences with it
and both are deliberate:

- **The crossing is not resolved against the world.** Whether there was anything in the way
  was decided when the dash began; pushing her out of the geometry halfway along is exactly
  the *partial* obstruction the rule says there is no such thing as. It is what caught her
  feet on the platform's side.
- **She lands on the shadow's spot exactly.** The arrival tolerance is half a metre wide,
  which on flat ground is invisible and on a ledge is the difference between the deck and
  the lip.

**The obstruction rule, and why it is so permissive.** "No unobstructed line from her to
the shadow" is the specification, and the interesting part is what counts as *a line*.
Feet-to-feet alone would have refused every dash onto anything, which is the bug. Both
bodies are upright columns over fixed spots, so every line between them shares one
horizontal footprint and they differ only in how they rise — which makes the four corner
lines the extremes of the family, and one of them getting through enough. At the foot of a
ledge the line from her crown clears the lip, so there is a way up and she takes it.

The cost is that it is very permissive, and in this blockout it refuses nothing at all:
platforms and walls are 1.5 m, a fighter is 1.8, so she can always see over. That is a fact
about the arena rather than a hole in the rule — and a **structure** is exactly a fighter's
height, so an Elementalist's stone raised on the line does refuse it. That turned out to be
the nicest thing in the change: denying the Reaver's line is now something another class
can do on purpose, and it is what the test pins.

**The carry.** The slide after a dash was already there — the dodge's tail decays whatever
speed you are carrying, and a dash arrives at thirty-four metres a second, so she keeps
going for a good four metres. Reported as something that *wants* to be usable. It is now:
a jump pressed inside the window takes the slide up with her, and cuts the dodge's tail
short doing it, so the frames she would have spent being punished are spent in the air.

Two decisions inside that:

- **A press, not a hold**, and only after a dash. Every dodge in the game has a punishable
  tail; a jump out of that one would be a universal escape rather than one class's tech.
- **The window is a timing, not a distance.** What is left of the dodge on arrival already
  *is* a window of this kind, but its length is however much of the dodge the crossing did
  not spend — about a frame at the end of the leash, which is the range the class is built
  around. So arriving tops the dodge up to the window's own length. It never shortens one:
  a short dash keeps the whole tail it has always had, and only the first frames of that
  tail are the window.

**Verdict** open on all four, and the numbers to drag are the carry and the slide inside
it. Ten frames is a guess. The slide is *inherited* rather than chosen — nobody picked four
metres, it is what `defence.dodge_decay` does to thirty-four metres a second — and on a
four-metre-wide dais that is most of the way to the far edge, so arriving on high ground
and immediately sliding off it is the thing to watch for. If it reads badly, the honest fix
is a knob for what fraction of the dash survives the arrival rather than shortening the
window, because the window is what makes the jump findable.

**One thing the dash can now do that it could not:** go over a wall. The blockout's walls are
the same one and a half metres as its platforms, so the crown-line clears them and a shadow
sent over one is a shadow she can follow. Leaving the arena was already possible — a full hop
reaches 3.7 m and the walls are low on purpose, so anyone can jump onto one and step off the
far side — and she is not stuck out there, because the wall is as jumpable from outside as
from in. Filed rather than fixed: if the closed arena is meant to be closed, that is a fact
about the walls rather than about this dash.

One thing deliberately not touched: the air-speed cap still clamps her the moment she
*steers* out of a dash jump, because `clamp_air_speed` runs whenever air acceleration finds
head room and a turn always does. Hold the line you left on and the boost survives in full;
fight it and it collapses to 14 m/s. That is the existing air model rather than anything new
here — the Bulwark's leap has the same edge — and changing it is a decision about every
class at once.

**Found while doing it, and fixed elsewhere the same day:**
`move.shadow_reaver.send_shadow.damage` was **0**, so the recall cut nobody against a kit
document that says it "dashes back through anything between the two of you, damaging and
slowing it". This branch wrote it up rather than guessing at a balance number;
*Send shadow is instant and cannot interrupt*, above, put it back to 70 and explains why it
had been zeroed — it was an attempt to remove the recall's **interrupt** with the only
lever available, and the interrupt turned out not to be reachable from that number at all.
Worth keeping as a pair: the same symptom read as a damage problem from one side and a
frames problem from the other, and only the second reading had a fix in it.


### 2026-09-14 — nothing roots you any more
**Changed** Committed moves keep **20% of walking speed — 1.4 m/s — instead of 0**, across all
sixteen of them. Everything else about a committed move is untouched: for its whole length you
still cannot jump, dodge, guard, or throw anything else. The walk table is now 7.0 free, 4.2
poking, 3.0 crouching, 2.0 guarding, 1.4 committed.

Second half, and the half that does the work: **the hindered speed is now arrived at rather
than assigned.** Starting a move used to set horizontal velocity to the move's speed outright,
which was a one-frame drop of 2.8 m/s for a poke and would have been 5.6 for a heavy. It now
bleeds down to the move's speed over about four frames, on the same ramp that already handled
the root, with the *direction* following the stick from the first frame and only the magnitude
lagging. `tuning::attack_root_decay` is `tuning::hindrance_decay` for that reason; its floor is
the move's own speed while you are steering and zero when you are not, which is why letting go
of the stick mid-move still bleeds you to a stop.

**Why** Reported as jarring, and it is the same report the poke drew on 2026-09-11 — "stopping
dead feels jarring; a big slowdown with no jump and no dodge should play the same and read
better". Worth recording that the earlier entry drew the line in the wrong place. It concluded
that *the snap and the rooting were two different complaints wearing one coat*, fixed the snap
everywhere and kept the rooting on the heavies as correct design. The snap half was right. The
other half was a category error: a forty-frame commitment is the longest the game ever holds
you, so it is the worst place to take the controls away, not the most defensible one.

The thing that made rooting look load-bearing is that it was doing two jobs at once and only
one of them was the standing still. Spatial commitment — you chose this, now live in it — is
`Action::actionable` refusing every other button, and that was never this number. Taking the
feet away on top of it read as the game asserting the commitment a second time, to a player
who had already felt it.

**The spacing arithmetic, because it is the obvious objection.** The longest committed move in
the game is 42 frames. At 1.4 m/s that is 0.98 m of drift, against a body radius of 0.5 and a
Slam that reaches 2.0. Facing still locks on frame one, so the drift is along one committed
line rather than a way to re-aim: creeping forward during startup buys about 0.4 m of reach on
a move whose whiff costs 24 frames of recovery. That is a skill expression at the scale of a
footsie, not a change to whether heavies are punishable. Below the guard walk on purpose — a
committed move should still be the most your feet ever cost you, and guarding is the slowest
thing you can otherwise choose to do.

**Verdict** open — wants a real match. The three things to watch: whether whiff-punishing a
heavy got meaningfully harder (the arithmetic says no, the hands are the judge), whether 1.4
reads as a crawl or as sluggish walking, and whether the four-frame ramp is visible at the
start of a heavy or just felt.

**Left open deliberately.** Every one of the sixteen got the same 20%, and there is no reason
to believe a Cataclysm and a Bash-with-a-hammer want the same number — the per-move column
exists and this is a uniform first guess, exactly as the poke's 60% was. The open question
above it is the older one and is still open: *should mobility vary across a move's phases* —
free during startup, rooted through active, slowed in recovery. Rooting only the active frames
is the version worth trying first, because it would put the "commit to a spot" property back
where it actually belongs without any of the cost above; it was not done here because it is a
new mechanic rather than a number, and this change is a number.

**What the harness now pins.** `feel::a_committed_move_is_a_crawl_and_never_a_stop` bounds the
crawl at both ends: above zero, and below the guard walk. `feel::a_committed_move_takes_the_
jump_and_the_dodge_away` drives real presses through a live `World` and is the one that matters
— it is where commitment lives now, and if it ever passes silently while the crawl is being
retuned, the retuning is free. `feel::hindrance_is_proportional_to_commitment` lost its
`committed.roots()` assertion, which is why `Move::roots()` is gone: a predicate whose only
true case the feel harness forbids is a predicate nobody can use.

### 2026-09-14 — a match opens looking at your own fighter

**Reported** The opening view is wrong. Tilt the starting angle down so it reads as an ordinary
third-person view.

**Changed** The opening pitch is its own Oven knob — *Opening pitch, below level*, **35 degrees**
— where it used to borrow `Zones::neutral_pitch`, the middle of the neutral zone, which works
out at 27.5.

**Why the borrowed number was wrong, and it is not that 27.5 is close to 35.** On this rig the
eye rides further out along the sphere the further below the horizon you look — that is the
same geometry the 2026-09-11 entry above works through, where the mark on the ground lands at
`h / tan θ`. So the opening pitch is not only which way the camera faces. It is *how far back
the camera is*, and therefore whether the first thing anybody sees is their own fighter standing
in an arena or a patch of floor with their own shield lying across the corner of it.

At 27.5 degrees it was the floor. The fighter sat just below the bottom edge, the opponent was
pinned against the HUD at the top, and the Bulwark's shield filled a third of the screen because
the eye was close enough to be inside the arm holding it. Nothing about that reads as a game you
know how to stand up in, which matters more than usual right now: it is the frame somebody
following a link sees before they have pressed anything.

**The middle of the neutral zone was never a claim about where to open.** It is a fact about the
zones — halfway between the boundaries, so there is room to steer either way without crossing
one. That is a good property for a resting angle to have and it says nothing about framing. The
two questions wanted two numbers, so they have two now, and `neutral_pitch` goes back to meaning
only what it says.

**What was looked at.** Rendered stills of the opening frame, Bulwark against Bulwark, at 5.7,
11.5, 17.2, 27.5, 34.4, 43 and 54.4 degrees below level:

- **Above ~20** — no own fighter at all. The eye has come in toward the head and the body is
  below the frame or faded.
- **27.5, where it was** — the floor shot described above.
- **34.4** — your fighter low-centre, the opponent centred on the dais, both arena walls in
  frame. This is the one.
- **43** — works, and is visibly further back and higher. Reads more like a diorama than like
  standing there, and it is two degrees off the floor boundary, so a nudge of a neighbouring
  knob would change which zone a match opens in.
- **54.4** — past `floor_from`, so the view tilts toward your own feet: cropped arena, opponent
  back up against the HUD. Worse than where it started.

35 rather than 34.4 because a knob wants a round number and the exact value inside that zone is
not what carries the reading.

**Two tests, in `view/tests/presentation.rs`** rather than one assertion on the number. The
opening angle sits strictly inside the neutral zone with margin at both ends — past `floor_from`
a match opens pointed at the ground, above `neutral_to` the eye has come in too close to see
yourself. And the property the number exists for: at the opening angle the eye is more than a
metre back from the fighter and the body is not faded. They are in `presentation.rs` and not in
`camera_knobs.rs` beside the other camera tests, because that file's one test deliberately drags
the live knobs to nonsense and back, and tests in a file share a process — read these next to it
and they see a camera mid-retune. That cost twenty minutes and a confusing "the eye opens 0.00 m
from the fighter".

**Verdict** open — chosen from stills, by someone who has not played it. The reading being
claimed here (34.4 "reads like standing there", 43 "reads like a diorama") is exactly the kind
of thing a still cannot settle. It is one slider in the Oven under *camera*, and `?shot_pitch=`
in the browser build will render any angle without a rebuild.

---

### 2026-09-14 — the Champion's chain, and a weapon on the way off the floor

**Changed** the grounded row of the Champion's grid became a **three-hit string** — nine
moves where there were three — and every weapon gained a **takeoff** thrown on the same
press as jump. Ten moves became nineteen. The uppercut moved off Rush + hammer onto the
hammer's takeoff, and Rush + hammer became a new move. Six knobs are new: `Chain survives
for`, `Chain, recovery owed on a swap (%)`, `…on a repeat (%)`, `Takeoff window around a
jump`, `Pole drive, forward boost`, and nothing else in the Oven moved except the
Champion's own rows.

**Why** the report was that the class "feels very linear", which is the same word the
2026-09-12 rebuild used about the mode toggle it removed. That rebuild was right and did
not go far enough: it made *which weapon* a real choice, and then gave you exactly one of
those choices per exchange. One press, one shape, back to neutral. The decision was real
and you got to make it about as often as you got to make a decision about anything.

A string is the cheapest possible fix for that, and the reason is arithmetic rather than
taste. **Three hits with three weapons each is twenty-seven orderings out of nine
animations.** The mid-animation swap in [champion.md](champion.md) was reaching for the
same thing and asking for blended tails to get it; this asks for ordinary clips.

#### What makes the weapon being free the mechanic rather than a flourish

Each weapon owns a different volume — the sword owns width, the hammer owns the line
underneath, the spear owns distance. So a **mixed** string covers three different pieces of
space in three beats and a **pure** one covers the same piece three times. Against somebody
who is moving, that is the difference between the third hit landing and the third hit
whiffing, and **nothing had to be added to make it true**. The shapes were already there.
That is the part worth keeping if any of the numbers below get thrown away.

The frame data does one small thing on top of it. A connected link cuts its own recovery
short so the next one can begin, and **swapping cuts it shorter than repeating** — 30% of
the recovery against 55%, which is three frames against six on the sword and six against
twelve on the hammer. The fiction is that the head re-forms out of the follow-through
rather than being re-chambered; the mechanical intent is that linear play stays completely
viable and mixing pays a little better, which is what
[champion.md](champion.md#identity) has asked for since the class was written down. Set the
two equal and the incentive is off, with nothing else changing. **That is the number most
likely to be wrong**, in either direction.

#### The chain is a hit confirm, and that is the load-bearing decision

The first version cancelled the recovery whenever the next button was pressed, and it took
about ten minutes of looking at the frame table to see what that costs: `on_block` is
measured from the full recovery, so a cancel available on block quietly makes half the kit
plus on block, and `every_attack_is_punishable_on_block` — the one property that keeps
offence honest — would have become a statement about a number nobody pays.

So the cancel is **on hit only**. Blocked, parried and whiffed links pay in full. Three
things fall out, and all three are better than the alternative:

- every number the frame table prints stays true against a defender who did something;
- blocking one hit of a string costs the attacker about thirteen frames across a sword
  chain, which is a window rather than a moral victory;
- whiffing the opener and continuing anyway is a choice you can be punished for.

The string still *continues* on a block or a whiff — it just continues at the printed
speed. That matters for feel: a button that does nothing because the last swing missed
reads as broken, and the fix is to make it slower rather than to make it silent.

#### What a string dies to

Half a second of not swinging, being hit, blocking, dodging, or leaving the floor. **Not a
Rush**, and that is deliberate: the grace window keeps running through the dash, so one
charge buys a reposition in the middle of a string and the third hit can arrive from
somewhere they were not watching. It is the best thing in the change and it was free —
`Action::Free` while grounded is what the clock ticks on, and a dash is exactly that.

#### The knockback rule nobody would think to write down

The first pass kept the hammer's opener at its old knockback of 11 and the chain did not
work at all, for a reason that is obvious once seen: **a hit that shoves them out of range
of the next one has ended the string whether or not the game says so.** Eleven metres a
second carries somebody 1.3 m, and Uproot reaches 1.8 m from a body that is already a
metre away.

So the knockback moved to the finishers — 4 on the hammer's opener, 11 on Earthbreaker —
and the hammer's identity moved with it, from "big knockback and a long stagger" to "a long
stagger, and then a finisher that moves people". That is a better hammer anyway: the payoff
is at the end of the commitment rather than at the start of it.

It is pinned as `the_first_two_hits_leave_somebody_standing_where_the_third_can_reach_them`
in `feel.rs`, and the assertion does the geometric-series arithmetic rather than comparing
the speed to the reach — because the first version of the test compared those two directly,
failed on a perfectly good number, and was measuring the wrong thing.

#### The takeoffs

`space` plus a weapon, on the ground, is that weapon's way off the floor. **This is the
first time the jump button has modified anything anywhere in the grammar**, and it is
flagged in [README](README.md#4--open) as a precedent rather than a Champion detail.

- **Rising cut** (sword) — an angled slash up and forward. The hardest hit of the three and
  nothing else in it: no grab, no shove, and a long fall if it misses.
- **Uppercut** (hammer) — the existing move, re-homed. It launches and holds on, and space
  again takes you both higher.
- **Pole drive** (spear) — the butt of the spear into the floor at your own feet. Least
  damage in the class, most height, and a shove in whatever direction you are holding.

Read from behind they are a diagonal, a column and a vault, which is how you tell which one
somebody threw while all three are in the air.

**The uppercut moving off Rush is the biggest single change here** and it is worth its own
sentence. The class's headline loop started with a resource check: hammer, *do I have the
charge*, uppercut. A loop you often cannot start is not a loop. It now starts with a button
everyone always has, and the charge is free for the reposition — which is the half of the
class Rush was always better at. Rush + hammer became **Rush sweep**, the hammer dragged
along the floor through everyone you run past, which fills the hole and gives the Rush row
three heights rather than two.

**The window is eight frames**, and it exists because "attack as you jump" is one intention
and two buttons. Jump then weapon is the order that works; both at once is the same frame.
Click first throws the grounded move, which is correct — it already did.

#### The animation, which was most of the work

Nine new clips, and the rule they are authored to is the thing to keep:

- **Hit one opens from the guard and comes back to it.** It is the only row that does.
- **Hit two never opens from the guard.** Its first key is the far side of somebody else's
  swing — hands where a cut left them, the hammer head still on the floor, the point still
  out. It has no wind-up, only a continuation, and *that is the whole read*: a body that
  did not come back to guard is a body that is not finished.
- **Hit three commits the feet.** The finishers are the only grounded moves that turn, step
  through or leave the floor behind, because they are the only ones you cannot take back.

That third rule is the one a player will actually use without noticing. Everything else in
the kit puts the feet back where it found them, so feet that move mean *this is the last
one*, from any angle and at any distance.

Four clips failed `baked_motion_is_continuous` on the first bake and all four failed the
same way — a torso or a hand crossing too much ground between two keys — which is the test
doing exactly its job. The fixes were the honest ones: **Crescent** keeps its hands near
the sternum and lets the haft go round, which is what the file's own note about grips says
and what a real swing does; **Earthbreaker** got a key halfway down, because a weight that
size does not go from held to landed in one interval; **Impale** got a key with the rear leg
swinging under the hips, because a flèche is a step rather than a hop; **Rising cut**
stopped trying to move the whole torso through sixty degrees in two frames. All four read
better afterwards, which is the usual result and is why the ceiling is where it is.

**Verdict** open, and unusually so — this is a mechanic rather than a number, and the
numbers under it are all first guesses. A held sword string is 234 damage in eighty frames
and a held hammer string is 212 in a hundred and twenty-one, which is the intended shape
(the hammer buys stagger and a guard break rather than numbers) but is a big gap to have
picked at a desk. Things to watch for, in order of how likely they are to be wrong:

1. **Is three hits too many?** A string you can be interrupted out of twice is a string
   whose third hit rarely happens. If so, the fix is not a fourth cancel rule — it is to
   move damage from the finishers onto the openers, so that being cut off costs less.
2. **Is the swap bonus findable?** Three frames on the sword may be below the threshold at
   which anyone notices, in which case it is either bigger or gone. Gone is a real option:
   the shapes already make mixing correct.
3. **Is the grace window right?** 26 frames is a guess, and it is the number that decides
   whether a string is a rhythm or a thing you mash.
4. **Does the hammer string ever get thrown?** Two seconds is a long time to be committed,
   and Earthbreaker's twenty frames of startup is the most readable telegraph in the game
   on purpose. If it never lands, the answer is probably that the *second* hit needs to be
   plus enough to make the third a true block string, which it currently is not.
5. **Is `space` + weapon a good idea at all?** It is comfortable on a keyboard and it is a
   precedent. Somebody with a controller should hold an opinion before it spreads.

### 2026-09-14 — the Elementalist off the ground

**Changed** three moves, on the three buttons she already had. Left click in the air is the
**Air bolt**, right click is the **Gale**, and `E` is **Landfall**. Four moves became seven.
Eight knobs are new, all under `Elementalist`: `Air bolt speed`, `Gale speed`, `Gale size
leaving her hand (x)`, `Landfall dive speed`, `Landfall stone, how far ahead`, `Landfall stone
rise`, `Landfall eruption, above the floor (turns)` and `Landfall eruption push`. Two of the
Oven's *slider bounds* moved with them and no baked value did: a move's `Reach` now runs to
twenty-eight metres — the width of the arena, past which more range cannot change anything —
and its `Knockback` to thirty, which is what every other knockback knob in the Oven already
runs to and what `Move::Knockback` would have had if it had not quietly inherited the shared
twelve.

**Why** the class had nothing in the air, and the README has had **Aerials** open since the
Champion answered it for itself: *airborne attacks should be variants of their grounded
counterparts rather than a separate move list.* That is a claim about one class until a second
one is built the same way. So: the button is the thing you are throwing and the row is where
your feet are, exactly as on the Champion, and a player who has learnt her standing up has
learnt most of her in the air.

#### The three decisions worth keeping if the numbers get thrown away

**Air, and why it is not earth.** Earth is what she is standing on, and off the floor she is
not standing on it. The two shots are the element she can reach up there; the way back to
earth is to go and hit it, which is Landfall. That also gives the kit's *"ice, air and
lightning are the specialisation axis for later"* its first outing without committing to a
loadout.

**Both shots travel, and nothing she throws standing up does.** Bolt and Cataclysm are instant
lines resolved on the frame they come out. These have a speed, and the reason is the situation
rather than the element: she is falling while she throws them, and an instant hit taken from a
position she cannot hold would be free. A flight time is what makes leaving the floor a trade.

**The Gale is worth what it has become.** It leaves her hand at 30% of its radius and arrives
at full size, and damage and knockback ride the same fraction — 36 at point blank against 120
at the tip. Every other projectile in the game is worth the same wherever it lands. This one
inverts the spacing, which is the same sentence Flame spitter is already written around, and
it hands the opponent an answer this class least wants to give and most deserves to be made to
give: *close*. The stun is deliberately flat, because frame data that changed with distance
would be a move nobody could learn.

#### Landfall, and the first startup in the game that ends on a place

Its row says 26 frames. What it *owes* is a 14-frame hang at the top and then a dive at 26 m/s
until her feet arrive — so from the top of her jump the wind-up is a little over half a
second, and from a short hop it is barely longer than the hang. The telegraph is as long as
the height she chose to open up, which is the property the whole move is built on: going
higher is buying reward with time the opponent gets to use.

**And they can use it, with no rule of its own.** The descent is an ordinary startup, so a hit
knocks her out of it exactly the way a hit knocks anybody out of a wind-up, and the slab she
was about to drive up never appears. That was the requirement and it needed no code — which is
the right outcome, and is worth noting because the obvious implementation (a bespoke
"interruptible" flag on the move) would have been a second way of saying something the engine
already says.

The slab is the payoff and it is **driven rather than raised**: out of the floor in six frames
against fourteen, because the telegraph was the plunge rather than the rise, and **leaning**
forty-five degrees away from her, so whoever is standing over it is thrown up and back instead
of merely staggered in place. Stones grew two fields for it — `rise` and `erupt` — and an
ordinary stone's `erupt` is zero, which is the old behaviour written down rather than implied.

**Verdict** open, and every number under it is a first guess. In order of how likely each is
to be wrong:

1. **Is the plunge's length-by-height a trade or a loophole?** Pressed low it is nearly all
   hang, which is the shortest telegraph and the same reward. If that reads badly the fix is
   probably a floor on the dive rather than a cap on the hang — she should not be able to
   *arrive* faster than a foe can answer, and how high she started should stay the thing that
   decides how long they get.
2. **Is a Gale at her own feet too weak to be worth throwing?** 36 damage is close to
   nothing, and a move that is nearly worthless at the range you are most likely to throw it
   may read as broken rather than as spacing. `Gale size leaving her hand (x)` is the one
   knob for it.
3. **Is a fourth structure's worth of terrain every time she leaves the floor too much?** The
   cap of three is a readability guess and this is a new way to spend it, on a button that
   also does something else.
4. **Does the 45° lean actually clear the space?** 13 m/s along it is a real shove on paper;
   whether it separates her from somebody who has closed, or just pops them into a spot they
   can airdodge out of, is the thing the move exists for and the thing a test cannot answer.
5. **The air row shares the grounded clips.** The Air bolt plays Bolt's flick, the Gale plays
   Cataclysm's throw and Landfall plays Fissure's *both hands driven into the ground* — which
   is very nearly right, and is why this shipped without four new recipes. Four authored
   clips is the obvious next piece of work on the class.

### 2026-09-14 — a shot aimed at the floor goes through whoever is standing on it

**Changed** two things, and the second is the reason for the first.

```
before   skillshot: a ground hit is raised to the CASTER's cast height
now      skillshot: a ground hit is raised to the middle of a fighter STANDING
         on that spot (`aim::standing_middle`)

before   Grasp: a channelled swing, dead-zoned level, 1.5 m to 10 m over 60f
now      Grasp: a channelled skillshot whose line is solved ONCE at the full
         reach, with the hold picking a point along it -- and back to 30f
```

**Why the raise moved.** The two are the same number on flat ground and nothing like it off
it. Standing on one of the low platforms, the caster's cast height is 2.75 m above the arena
floor, so every skillshot aimed at somebody below flew out level and over their head. Landing
one meant aiming at a patch of floor well short of the target, which is not a thing a player
should have to know. Measuring the raise from *the ground the ray met* makes the rule true from
any height, and it is what `controls.md` already said — "the middle of a fighter standing
there". `aiming.md` and `CLAUDE.md` said the other thing. One of the two was wrong and both are
fixed.

It is a small change even on flat ground: shots now end 0.9 m above the spot rather than 1.25,
which is a fighter's middle rather than a fighter's shoulder.

**Why the Grasp went back to being a skillshot, and what is actually different this time.**
This is the third version of its aiming and the first two each fixed the other's bug:

1. **Skillshot solved at the wound-up range.** The raycast's own answer changes as the range
   grows, so the far end walked off the floor and onto a wall and back. Holding the mouse
   perfectly still, the marker jumped about while the player was choosing a depth.
2. **Swing, dead-zoned level.** Nothing to stop against, so the line held still — but a level
   line out of a platform passes clean over anybody on the floor, which is the bug above wearing
   a different hat.
3. **Skillshot, solved once at the full reach, hold picks a point along it.** The line is the
   crosshair's, so it converges from any height; it is solved at a constant range, so it does
   not move while the mouse does not; and the hold slides a marker along it.

The third is not a compromise between the first two — it is the observation that *solving the
line* and *choosing a distance along it* are separate questions, and the first two versions
had them tangled. `a_still_mouse_holds_the_line_and_only_the_marker_moves` measures the
direction frame by frame across a channel and gets zero drift at every pitch.

**The far end of the line is also the furthest the marker can wind**, which falls out for free
and is right: a Grasp fully held at a wall six metres away converges on the wall rather than
three metres inside it. `the_marker_never_reaches_past_what_the_crosshair_is_on` pins it.

> **Wrong, and reversed the next day** — see the entry below. Capping the distance at the
> solved length puts the scenery in charge of *how far*, which is the half of the question the
> hold is supposed to own. The marker winds along the move's own reach and goes through walls.


**Not fixed: melee swings from a platform.** The Champion's spear thrown laterally off a
platform still goes out level and over a target on the floor, for the same reason version 2 of
the Grasp did — the swing dead zone. Left alone deliberately. A swing is not aimed *at*
anything, it is a body moving through an arc, and the dead zone exists because you fight people
at your own height by looking slightly down at them. Making it read a target's height would be
making a swing into a skillshot. The honest version of the complaint is that melee is an
imprecise zone and stepping off a platform is the answer; if it turns out to matter, the knob
is `swing_level_to` and the real fix is probably that the dead zone should shrink with height
above the floor.

**The channel went back to half a second.** It had been doubled to sixty frames while the
marker was still jumping about, on the theory that it was too fast to read; once the line
stopped moving, thirty was legible again and a full second of standing still was simply a long
time. The marker now covers about twenty-eight centimetres a frame, which makes placing a
specific depth a real piece of execution rather than a wait.

**Verdict** open on the Grasp, and this is the version to play rather than the last two. The
worry that survives all three is unchanged in shape but smaller at thirty frames: the wind-up
is a telegraph, and the marker being legible now makes it easier for the other player to read
the depth and step out of it too.

### 2026-09-14 — the crosshair picks the direction, the hold picks the distance, and nothing crosses over

**Changed** one line, and it undoes a rule from the entry above that should never have shipped.

```
before   the marker winds along the SOLVED path, so a ray that stopped on a
         wall six metres out capped the Grasp at six metres
now      the marker winds along the move's own reach, so the same Grasp is
         ten metres and the arms converge four metres behind the wall
```

**Why it was wrong.** The entry above got the split right and then let the two halves leak into
each other in the last paragraph. "Solve the line once, then pick a point along it" was
supposed to separate *which way* from *how far*; capping the distance at the solved length puts
the scenery back in charge of the second question. The result is an ability that quietly
becomes a different ability depending on what happens to be in front of you — a full wind-up
that reaches ten metres in the open and two indoors, with nothing on screen saying why. The
marker was honest about it, which is worse rather than better: it told the player their fully
charged cast was a stub.

**A wall is a thing to punch an ability through.** The arms are a cone that converges; there is
no reason for them to respect a surface on the way. Aim at a wall two metres away, wind to
full, and it is still a ten-metre Grasp. `a_grasp_wound_to_full_range_reaches_full_range_through_a_wall`
pins the extreme version — pointed straight down at the floor underfoot, the ten metres go
*under the arena*, and that is correct, because what the player asked for was a depth.

`the_marker_never_reaches_past_what_the_crosshair_is_on` is deleted and
`the_marker_is_as_far_out_as_the_hold_and_nothing_else` is back, which is the assertion the
first version of the channel had and the right one all along — it was only ever failing because
the *aim* was being solved at the wound-up range. Same assertion, different bug underneath it.

**Verdict** settled, on this part at least. Which way and how far are two questions, and the
only thing that should answer the second is the button.
### 2026-09-14 — the camera climbed with you, so the aim point ran away

**Changed** off the ground, the camera's sphere stays centred **below the fighter's feet, on the
height they left**, rather than riding up with them. Eight knobs are new, all under `Camera`:
`Airborne framing, full at (m)` = 3, `most it takes hold a frame` = 0.10, `most it lets go a
frame` = 0.045, and the four handles of its curve. Nothing already baked moved.

**Why.** The report was that a shot thrown from the air, aimed down, went somewhere nobody
chose. Two separate things were wrong and they compounded, which is why the symptom had no shape
to it. The first was the skillshot lift measuring from the caster rather than from the ground
the ray met — fixed the same day from the other end, by the platform case, and written up in the
entry above as `aim::standing_middle`. This is the second.

The camera's sphere is anchored to the fighter, so a jump takes the eye up with them — and at a
fixed downward pitch the patch of floor under the crosshair is set by how high the eye is. Hold
the mouse perfectly still and climb five metres and the thing you are pointing at races seven
metres away from you. There is nothing to learn there; the aim point is simply not yours.

> You climb the screen toward the crosshair instead of the camera climbing beside you — until
> you are just under it, and from there it follows again.

**How far it may hold is derived rather than tuned**, and that is the part worth keeping if the
numbers get thrown away. It is exactly the rise that carries the fighter from wherever the pitch
zone has put them on screen up to the turn zone's own top waypoint — *the head riding 5% under
the crosshair* — and no further, because past that they would be climbing over the reticle.
Which makes the awkward cases answer themselves: look steeply down and the floor zone has
already carried you to the reticle with your feet on the ground, so there is no room left and
the camera stays glued to you, which is exactly right when what you are looking at is the patch
of floor you are standing over. At a shallow aim it works out at about 2.9 m.

Measured on a full jump at 30° down: the eye moved 0.05 m while she climbed the first 3.0 m, and
the aim point held inside a metre of where it started. Before, the same climb swept it from 0.9 m
to 8.0 m ahead of her.

Two knobs shape the approach. An **S-curve in height**, so a hop barely moves it — at `full at` =
3 m a short hop fits inside the hold entirely and the camera simply does not move. And an
**asymmetric rate limit**: quick to take hold going up, slower to let go coming down, because
terminal velocity is eighteen metres a second and the last few metres of a long fall arrive in a
handful of frames. Setting the two equal turns the asymmetry off.

**The cost, stated plainly:** the framing now has memory, so it is simulation state
(`Player::aloft`), in the rollback snapshot and in the desync checksum. That is the same
argument that pulled the camera's knobs into the checksum in the first place — the eye is where
the aiming ray starts — and it is why this could not be done in `view`.

**Verdict** open; this wants playing, which is what it was built for. Things to watch, in order
of how likely each is to be wrong:

1. **Is 3 m the right height to be fully held?** It is set so that a short hop is entirely
   inside the hold and a full jump (5.7 m at this tuning) spends roughly half its climb there.
   Lower and a hop starts swinging the view; higher and the top of a jump arrives as a lurch.
2. **Does the hold read as the camera lagging rather than as you rising?** That is the failure
   mode of the whole idea, and it is the one a measurement cannot detect.
3. **Is letting go at 0.045 a frame slow enough on a long fall, or is it now visibly behind on
   landing?** At the moment the camera is about 1.4 m low as the feet touch and takes a dozen
   frames to settle.
4. **The Elementalist's Landfall dives at 26 m/s.** The framing unwinds slower than the plunge
   arrives, so the camera is deliberately trailing her all the way down. That may read as
   weight, or it may read as the view failing to keep up with the move.
5. **Does anything want this besides a fighter in the air?** Being knocked off a ledge frames
   the same way as jumping to the same height, which seems right and has not been played.

### 2026-09-14 — the Gale was a wall of air, and it is a frisbee

**Changed** three things about the Elementalist's airborne right click, from playing it. Its
`reach` went 16 m → 32 and `Gale speed` 16 → 24 m/s, both as asked. One knob is new,
`Gale full size after (m)` = 16, and one slider bound moved: a move's `Reach` now runs to 40 m
rather than 28.

**The drawing was wrong, and it was the complaint.** *"The rclick isn't a disk, or maybe it's a
disk that faces me."* It was: the mesh was squashed along its own direction of travel, so the
flat face pointed where the shot was going and the player — standing behind it — saw the full
circle head-on. A wall of air coming at you.

The hit volume was never that. `aim::first_along` swells the victim's standing cylinder by the
shot's girth in **radius** and never in height, so what a Gale occupies is a horizontal disc of
that radius sweeping along its line: the same clearance saves you by stepping aside and does not
save you by standing above it. A frisbee, flying edge-first. So the fix is identity rotation and
a thin flat disc — the drawing catching up with the rule rather than the rule changing.

Worth noting as a near miss of the overlay rule. `state::hitbox` is drawn by the debug overlay
and cannot drift from the hit test; a projectile's own mesh is outside that machinery, and drew
a shape the game does not have for a week with nothing to catch it.
`the_gale_is_a_frisbee_rather_than_a_ball` now pins the volume, which is the half a renderer
cannot lie about.

**Why the growth got a knob of its own.** The disc's size was measured against its `reach`,
which is fine until the range moves — and the range just doubled. Tied together, "fly twice as
far" also means "be half as big everywhere a fighter actually stands": at 20 m the disc would
have dropped from full size to 74% of it, and the damage with it. That is a retune nobody asked
for wearing a range change's clothes. How far a shot goes and how fast it opens are two
decisions and now they are two numbers; the opening distance kept the old reach, so the disc
opens exactly as it did and simply stays open for the second half of a much longer flight.

**Verdict** open. Things to watch:

1. **Is a 32 m disc at 24 m/s still walkable-away-from?** It crosses the arena in a little over
   a second now. The whole design of the move is that you get to decide about it.
2. **Does full-size-for-the-second-half read as flat?** The move's identity is that it is worth
   what it has become, and it now spends half its flight having become all of it. If that reads
   as a projectile with no character, the answer is to stretch the opening rather than to
   shorten the range.
3. **Does a flat disc read at all from behind it?** Edge-on is the thinnest possible silhouette
   from exactly where the player is standing. It may want to be visibly thicker than the volume
   it stands for, which would be the first deliberate lie in any of this and should be argued
   for rather than slid in.

### 2026-09-14 — the frisbee is thrown, not laid flat

**Changed** how the Gale is drawn, twice over, and nothing about what it does. Same speed, same
reach, same girth, same hit test. No knobs moved.

**The complaint after the last entry.** *"Right now it's just a giant orb… it reads more like an
orb or a shield. It's not that — it's offensive."* Two faults, and they were compounding.

**It was a sphere.** The disc shared the bolt's mesh, `Sphere::new(0.5)`, squashed to 12% on one
axis. A sphere has no flat face and no rim: every normal points straight out from the centre, so
there is no edge for the light to break on and no silhouette that changes as it turns. In an
almost-transparent material that is a glowing blob whatever you scale it to, and a blob that is
wider than it is tall is a blob. It has its own mesh now — a real disc (`Cylinder`, 0.18 thick
relative to its radius) with a rim you can see it turn on — and its own pool, because which mesh
a shot is drawn as must not be decided per frame on the rollback path.

**And it was laid flat in the world, not in the throw.** The last entry got "flat" right and
stopped one step short. The rotation was identity, so a Gale thrown *down* at somebody was a
dinner plate sliding horizontally through the air on its way to the floor — face-first again,
just face-down this time instead of face-forward. A thrown frisbee tips with the throw.

So the axis the disc spins about is now **square to the line of effect that aimed it, and in the
vertical plane containing that line**: `n = up − (up·dir)·dir`, normalised — the vertical with
whatever part of it runs along the throw taken out. Level throw, flat disc. Throw angled down at
40°, disc tipped 40° nose-down, edge leading all the way in. That is the whole rule, and it is
the user's, stated geometrically.

**It costs the hit test nothing, which is the part worth checking rather than assuming.** The
disc's sideways half-width is `dir × n`, and both `dir` and `n` lie in the same vertical plane,
so that cross product is horizontal however the throw is pitched. Tipping the disc *inside* the
plane its own path already lies in leaves the horizontal footprint exactly where it was; all
that moves is where the leading and trailing edges sit vertically, by less than the body height
`aim::first_along` already spans. Step aside and you are clear, stand above it and you are
clear — before and after. `the_gale_is_a_frisbee_rather_than_a_ball` measures that directly off
`aim::first_along` and is unchanged.

**The last entry half-predicted this and misread which half.** It asked whether a flat disc reads
at all from behind it, and guessed the answer would be to draw it thicker than its volume — the
first deliberate lie. It did not need one. What it needed was a shape with an edge, and an
orientation that turns as the shot is aimed, so the silhouette is doing work instead of sitting
still. Worth remembering the next time "make it bigger than it is" looks like the fix.

**Verdict** open. Things to watch:

1. **Does a downward Gale still read as reaching the target, or as diving past them?** Nose-down
   is right for a throw at the floor and may be too much for a throw at somebody's chest from
   just above them.
2. **Is 0.18 of the radius the right thickness?** Thin enough to be a disc, thick enough to catch
   light at a glance. The number is the drawing's alone; the volume it stands for is thinner
   still, so this is already a small lie and should not grow without an argument.
3. **Does the rim read from directly behind, where the player usually is?** The throw tips the
   disc away from face-on for anything but a level shot, which should help exactly in the case
   the last entry worried about — and does nothing for a level shot, which is the common one.

---

### 2026-09-15 — the Champion's three autos became three different shapes of pressure

**Changed** the nine grounded chain links, and three pieces of machinery underneath them.

The class had three weapons that differed in *reach and weight* and swung the same three
ways: across, down, and straight out. That is a range band, and range bands are what the
class already had. What it did not have was a reason to reach for one weapon rather than
another when the range was the same — so each weapon has been given a **job** instead, and
the frame data, the shapes and the clips have all moved to serve it.

**The sword is the one you throw while moving.** Two descending diagonals, mirrored — down
from the right shoulder, then down from the left — and then a rising cut up the line the
second one came down. Lowest startup and lowest recovery in the class by some way (6/5/9
against the hammer's 15/12/20), and **each of the three steps forward**: 0.55 m, 0.55 m,
1.2 m. A whole sword string closes 2.3 m of ground, which is the difference between "the
fast option" and "the option that gets you there".

**The spear is neutral.** A one-armed jab off the leading hand with the pommel still at the
hip — the only one-armed attack in the class, and the thing that makes it read as a poke
rather than as a short lunge. Then the odd one: a sixteen-frame wind-up, past reaction,
that dashes 1.8 m behind the point and hands out the biggest on-hit advantage in the kit
(+14, which makes the finisher a true combo). Then a spinning sweep, the shaft round the
whole body at knee height, with real knockback — the class's only answer to somebody who
has already closed, and the only volume in the game that threatens behind you.

**The hammer is weight.** A heavy overhead slam with the longest stagger in the chain
(+11 on hit); a short, mid-weight tear back up out of the floor that shoves rather than
lifts; and a finisher that dashes 1.5 m and **throws them up**. The last of those is the
new decision: **press jump during the wind-up and the knock-up is 1.6× and you leave the
floor on the frame it lands**, so the pair of you arrive in the air together and the air
hammer is waiting. Decline and it is a knock-up and a reset. Paid on contact, not on the
press, so a whiffed finisher leaves you standing in your own recovery — the rule the aerial
fan's shove already follows.

**Three pieces of machinery, and each is true of every ability at once rather than of this
class.**

*A move may carry the body.* `Move::step` is a distance down the facing the move locked,
spent over `tuning::step_lead` frames of run-up and then the frame the hitbox appears on —
so the weight arrives with the weapon. It is **added** to the stick rather than replacing
it, which is what lets a cut thrown while strafing come out diagonally instead of snapping
to the front. The follow-through afterwards is momentum bleeding off through the hindrance
ramp, not the step.

The window was the whole active period first, and the hammer's finisher is what found the
problem: it put its head through the floor on the first active frame and then carried the
body most of another metre past it, feet skating, for the five frames the volume was still
out. Ending the drive on contact is the rule that is right for both a cut that sweeps and a
slam that plants.

*A swing may be rolled off the vertical.* `Plane::Diagonal(Hand)` is the upright plane
tilted by `tuning::cut_roll`, with the hand naming which shoulder the head starts over —
one tuned magnitude and the sign from the hand, exactly the way the Dual mage's two wings
share one span. A cut that is either level or vertical has to choose between owning the
width of the front and owning the height of a body; a diagonal owns both, which is why a
swordsman throws one.

*The jump button can be read inside another move's frames.* One move does it. Whether that
generalises is an open question below.

**Two things this broke, and both were worth breaking.**

`a_weapon_keeps_its_shape_for_the_whole_chain` asserted that a weapon's three links were
all the *same* volume. They are not any more: the sword's cuts mirror each other, which is
the whole of why the string flows, and the spear's finisher spends its length swept flat
instead of thrust. What is actually load-bearing is that **no two weapons ever put out the
same volume**, so the shape on the screen always names the weapon — that is what the test
says now, and it is a stronger claim than the one it replaced.

`the_three_weapons_own_three_different_pieces_of_space` probed the sword at 3.2 m and
expected a miss. It connects now, because the step is real reach. The honest restatement is
two properties rather than one: the spear still reaches into a band the sword cannot touch
(4.15 m against 3.40), **and** a spear poke leaves you standing where you were while a
sword cut does not. Where you end up is half of what a spacing weapon is, and it took a
move that carries the body to make that visible.

**One bug fell out, and it is the reason the next paragraph exists.** The spinning finisher
swept one way in the clip and the other way in the hit volume, and nothing in the game
would have said so — `sim` cannot see an animation and `anim` does not resolve hits. The
cause was a doc comment: `Move::arc` said a positive arc runs "right to left across the
body", which is true of an upright swing and backwards for a flat one, because the body is
authored in a left-handed frame dropped into a right-handed world. The comment is fixed and
so is the arc, but the fix that matters is
`the_champion_swings_the_weapon_the_player_can_see` in `view/tests/kinematics.rs`: it reads
the drawn weapon off the baked clip and the volume off the live simulation and fails if
they disagree about up or about sideways. It found this the first time it ran.

**Verdict** open — nobody has played it. Things to watch:

1. **Is the sword's step too much pressure?** Each link closes ground whether it lands or
   not, so a held sword string walks 2.3 m into somebody. The counter is that you finish
   inside their range having spent your fastest options; whether that is a real cost is
   the first thing to find out.
2. **Is the spear's second hit readable enough at sixteen frames?** It is deliberately past
   reaction, and it closes 1.8 m. If it is not visibly a two-part move — coil, then dash —
   from across the arena, the wind-up is not doing its job and the answer is the clip
   rather than the number.
3. **Does "jump into the finisher" occur to anybody?** There is nothing on the HUD that
   suggests it and no other move in the game reads a button mid-animation. If it has to be
   taught, it may want to be the hammer's rule rather than one move's.
4. **Is +14 on hit too much?** It makes hammer-after-spear-two a guaranteed finisher, which
   is the point, but a guaranteed finisher off a move that also closed two metres may be
   the whole class.
5. **Does the spin's knockback fight the chain?** It is the one link with real knockback
   and it is a finisher, so nothing follows it — but it also ends with you standing where
   the string started, which no other finisher does.

### 2026-09-16 — shift is one verb, and Lance goes to middle click
**Changed** Shift plus a click stopped being an attack input, on every class. The Dual mage's
Lance moved to middle click. The Bulwark's Slam, the Elementalist's Fissure and the Blood
mage's Rend now have no input at all; the Shadow Reaver's Executioner was already on `E` and
lost nothing.

**Why** Shift meant two things and the way to tell them apart was whether a click happened to
be held, so what the key did depended on what the rest of your hand was doing. Every other key
in the game means one thing. It also removed the *Move + heavy attack* collision by removing
one side of it.

The Dual mage is what made it urgent rather than tidy. Her committed cast wanted a two-form
split — light and dark, decided by the force she carries — and it could not have one on shift
plus **left** click, because that input has a side: it pushed her dark whatever she was
holding, and the light form had nowhere to live. Middle click has no side, so by the class's
own rule it pushes her further along her current path, and the form is then free to come from
the arm she last punched with. Two rules stopped fighting.

**Verdict** open. Watch three things. Whether `M` is reliable enough for a cast thrown every
exchange rather than a once-a-match finisher — `U` stands in, and the answer may be that the
stand-in has to be the binding. Whether the three stranded moves are missed at all, which is
useful information either way: a committed move nobody notices the absence of may not have
been earning its slot. And whether "a click still wins when both are held" is the right
precedence now that the click is not what the shift is *for*.

### 2026-09-16 — the Dual mage's depth curve, built at last
**Changed** `state::depth`: one straight line from `depth_floor` (0.5) at the centre of the
bar to `depth_ceiling` (2.0) at either end. Everything she throws is multiplied by a point on
it — damage, knockback and pull, launch, leech, what a field drains, how long it lasts, and how
big what arrives is.

**Why** It is the class's founding idea and none of it existed: a cast at the edge was the same
cast as one at the middle, so "ride as close to the edge as you can" was a number going up on
the HUD and nothing else. Built as one function rather than per ability, because the whole
point is that it is true of everything at once.

**Two exclusions, and they are the interesting part.** Depth does not touch **how far** a move
is thrown or **how fast** it comes out. Spacing and frame data are what two players read each
other with, and a class whose range or startup slid continuously along a bar only one of them
can see is unlearnable from both sides. The first version did scale reach, and it was wrong in
a way that showed up immediately in the harness: a Judgement thrown level from depth landed
twelve metres away instead of six, and the light Lance's burst — the thing you aim *past*
somebody — moved from seven metres to ten and a half depending on a number behind her. So the
bar changes how much it hurts and how big the thing that arrives is; where you can put it is
fixed.

**A cast is worth where you were standing when you pressed the button** (`Player::thrown_at`),
not where its own push has since taken you. Throwing anything moves the bar on the press, and
the finisher moves it by 26 — so read live, a Judgement from dead centre came out at 0.89
rather than 0.5, and the one move that is supposed to be embarrassing at the centre was the
least embarrassing thing there. It would also have meant the bar on the HUD never matched what
the player got.

**The two autos are exempt from the size half**, and only that half. Their reach is pinned to
the punch that throws them — `view/tests/kinematics.rs` checks the blade starts where the fist
stops — and they are thrown every second, so a volume drifting off the animation would make the
steering wheel unreadable.

**Verdict** open. Half at the centre and double at the edge is a four-to-one spread, chosen so
the difference is unmistakable rather than because anything says four. The risk in one
direction is that the middle of the bar reads as broken rather than weak; in the other, that
nothing below the last quarter is worth casting from. Both knobs are in the Oven under
`Dual mage`.

### 2026-09-16 — one auto pulls and the other pushes
**Changed** The dark auto's knockback went negative (a pull, measured along the line between
the two bodies rather than down her facing) and it leeches 12%; the light auto shoves. The
tip of the wing now multiplies the *shove* as well as the damage (`wing_tip_shove`).
`Move::knockback` is signed, which it was not before.

**Why** The two autos are the steering wheel and they were the same punch twice. Giving them
opposite answers to the question of where the two of you end up standing makes which arm you
punch with a **spacing** decision at the same time as a meter one — which is the whole argument
for putting the mechanic on the buttons a player presses constantly. A fragile melee mage stays
attached with the dark hand and buys herself room with the light one, and cannot ask for either
without committing to a side of the bar.

**Numbers.** At the centre it moves somebody about 0.2 m, at the deep threshold 0.6 m, at the
edge 0.7 m — and the tip doubles all of those. Small on purpose: this is a punch, not a
shoulder charge, and the thing it has to do is change the range you are fighting at by about a
step.

**Verdict** open, and the number to watch is the pull. Being dragged is worth more than being
shoved at the same speed, because you are usually walking into the shove and away from the
pull. The two are within 10% of each other today, which may be a mirror that should not be one.

### 2026-09-16 — Lance is two moves, and the wind-ups are the whole point
**Changed** Split Lance into **Light lance** (9/4/18, the line pokes and a burst detonates
where it ran out) and **Dark lance** (16/4/20, no volume of its own; the line catches the first
thing it crosses and drains it until the leash parts). Two clips, authored against each other.

**Why** The kit has specified both forms since it was written and nothing on the class had a
split. What made it buildable was middle click — see the entry above.

**The wind-ups are the ability.** A person standing opposite gets the startup and nothing else
to decide between getting out from under a burst and closing to break a tether, and those are
opposite answers. So the light one rises off the right shoulder, goes early and dives down its
own line; the dark one sinks onto the rear leg, drags the left hand down past the hip and comes
through low with the palm open, and its contact pose leans *away* from the arm because
something on the end of it is about to start pulling. Same input, opposite silhouettes, and
opposite arms — dark lives in the left arm on this class.

**The leash is the one number depth does not touch.** Depth buys a harder drain and a longer
hold; how far she may stray is a rule, and it is the whole ability. A leash that grew with the
bar would hand the deep version the one thing it is meant to pay for, and at the edge it would
reach most of the arena.

**Verdict** open. The tether does not pull, on the grounds that the dark *auto* is what pulls
and two pulls would make one of them redundant — but a leash that never tugs may read as a rope
rather than a tether. And it holds fighters only: it catches a Ridgeback with its first pass
and then parts, because nothing holds a ten-metre animal on a leash, which makes the dark form
strictly the worse of the two in a hunt.

### 2026-09-16 — Sweep goes round past both shoulders
**Changed** The arc from 0.30 turns to 0.58, the active window from 6 frames to 9 and the
recovery from 16 to 14. Light throws them off their feet (the launch is the light form's
alone); dark slows them and heals her per target caught. The clip winds the hands behind the
left shoulder and carries them through behind the right, with a fourth key across the active
window.

**Why** It is the answer to somebody who has already got inside the punches, and the punches
are long and thin and thrown out in front — so the person it exists for is standing beside her
or past her shoulder, and a cut that only covered her front would miss exactly them.

**The frames moved because the geometry did**, and the animation is what found it. A hand at
arm's length sweeping 209° in six frames travels a third of a metre a frame, which
`anim/tests/clips.rs` calls a teleport, correctly. A wider sweep takes longer: nine active
frames is the same motion at a speed a body could produce, and it costs a frame on block, which
suits a panic button.

**Verdict** open. Per-target healing rather than a share of the damage is what makes the answer
to being swarmed the same move as the answer to being cornered; in a duel it is one target and
may read as a flat refund.

### 2026-09-16 — Judgement earns its status through power
**Changed** It leaves a **field** now: a wide, low-damage disc that burns anybody in it and
makes *her* fast while she is in it. Radius, damage and how long the field lasts are all on the
depth curve. Its meter push got a tier of its own — 26, against 5 for an auto and 12 for a
cast. Startup 22, recovery 28, repeat lockout 150%.

**Why** The depth gate came off on 2026-09-13 and took the finisher's status with it. The
class's own principle says the replacement is *power*, and this is that made literal: at the
centre a small radius, a small number and a field that is over before anybody walks through it;
at the edge the biggest thing in the game. The 26 is the other half — a Judgement thrown from
the deep threshold lands her well past it and one cast from the edge, which is what
"a deep finisher nearly throws you over the edge" was always describing.

**Verdict** open, and the number to watch is the total. At full depth the strike alone is
about 43% of a health bar and the field can take it past 80% if somebody stands in all of it.
It is reactable at 22 frames, −14 on block, and thrown from a position that is already burning
her — but "the biggest thing in the game" and "too much of a health bar" are one tuning pass
apart.

### 2026-09-16 — a line skillshot was a bubble on the caster's chest
**Changed** `state::hitbox` reads a skillshot's volume as the whole line from the hand to the
point the crosshair picked, unless the move is one of the two that resolve themselves the
instant they fire and stop at whatever they met.

**Why** Not a feel decision — a bug, found by playing the class rather than by a test. The
capsule was drawn to `beam.at(p.beam_reach)`, and `beam_reach` is set only by the
Elementalist's two instant beams and is zero for everything else. So the Lance, a four-metre
line skillshot, had a hitbox that was a ball at her sternum: it could only hit somebody
standing on top of her, at which range the autos are better in every way.

Worth recording as a feel entry rather than a fix, because of how it hid. Nothing failed. The
move came out, the animation played, the frame table printed a sensible row, and the only
symptom was that the ability felt useless — which is indistinguishable from a tuning problem
until you look at where the volume actually is. The lesson is the debug overlay's own rule
pointed the other way: it draws what the hit test uses, so it would have shown this
immediately, and nobody had looked.

**Verdict** kept, obviously. The same change moved a skillshot's origin to the hand that throws
it, which is what lets the two Lance forms be told apart by the arm as well as the wind-up.

### 2026-09-16 — the whole roster was drawn mirrored
**Changed** Authored poses are reflected into the arena's frame on the way to
being baked (`anim::bake::as_drawn`), and `aim::Hand::outward` flips with them so
the simulation swings from the same side. A positive flat swing arc now sweeps
toward the body's left, and the strafe clips travel to the side they are named
after.

**Why** Not a feel decision — a bug, reported as one. Left click came out of the
Dual mage's right hand.

The cause is one line of arithmetic that was never checked. A pose is written the
way a person describes a body: `+Z` forward, `+Y` up, left arm at `-X`. That is a
**left-handed** frame. The arena is right-handed, and `view::body_turn` embeds one
in the other with a rotation — which cannot change handedness, so `-X` came out on
the body's *right*. Every clip in the game was drawn as its own mirror image:
`shoulder_l` moved the arm on the right, `plant_l` put a foot on the right,
`walk_left` strafed to the right while the body moved left.

**How it survived.** `aim::across` had been deliberately written to follow the
skeleton rather than the world, with a good argument attached: what a hitbox and
an animation have to agree about is which arm the player can see swinging. The
argument is right and it was holding two wrongs in place. The simulation and the
renderer agreed with each other and both were the mirror of the body, and *every*
handedness test in the suite compared the two of them to each other. Nobody had
written the one that checks either against the world's own idea of which side of a
body is its left. Both exist now — `view/tests/kinematics.rs` and
`sim/tests/aiming.rs` — and they are the point of the entry.

It took a class where the arms *are* the mechanic to surface it. On everything
else a mirrored body is just a southpaw and nobody looks twice.

**The reflection is applied at the authoring boundary** rather than by renumbering
the six hundred lateral coordinates the recipes are written in, because those
numbers are not wrong: they say what their authors meant, in the frame they were
told to write in. What was missing was the conversion where the two frames meet.
`Pose::mirrored` was already there and already tested.

**Verified as an exact mirror**, which is the only way to change every animation in
the game without being able to look at them: every joint of every frame of all
eighty clips was solved before and after, and each one is its own reflection to
within a rounding error — 40,496 samples, zero deviation. Nothing is distorted;
each clip is the mirror image it should always have been.

**Verdict** kept. Two things fell out that are worth watching. Every character is
now handed the other way round — a stance mirrored is just the other stance, and
each clip finally matches the prose in its own recipe, but anyone who had learnt
the old silhouettes will notice. And the Bulwark's shield now hangs off the body's
left hand rather than the slot named `HandL`; it was on the right before and
nobody had said so.

### 2026-09-23 — Blood mage M1: grey health and the scythe
**Changed** Lost health on the class turns grey instead of going — every cost and every hit
moves red into a segment that fades at 12 a second and that only a pool will turn back. The
auto became Reaping sweep on left click (8/5/14, 22 damage, 2.8 m, cost 3, a tip at 65% of
the blade worth ×1.5), and Reap took Rend's row on right click (20/4/26, 110, unblockable,
an overhead, cost 55). Both reaches multiply by a line to ×1.5 at a full bar of grey, damage
to ×1.3, and the blade is drawn at that length whether or not she is swinging. The Bloodletter
moved to middle click and leeches nothing. Two clips re-authored: the overhead, and the sweep
low to high off the left hip.

**Why** The proposal's inversion — the correct answer to being wounded is to go in — needs
the wound to be a weapon, and a reach that scales is the only reading of that both players
can see. The fade's first number is the plan's own: a Reap's cost survives one exchange
(its whiff plus a dodge, 72 frames) with more than half left. Faster and the blade never
gets long enough to matter.

**Tried and kept off:** removing the `leech` column outright, which the plan asks for. The
Dual mage's dark auto and her tether heal through it and `dual_mage.rs` pins that, so the
column stays and is zero on every move of hers — `feel.rs` now asserts the zero.

**Verdict** open. Built, unverified: C1's questions (does the blade visibly grow; do you feel
more dangerous after being hit, or just lower; does the sweep's arc read as a scythe) are
the person's. The Reap's fall needed a mid key to stay inside the continuity ceiling: a
straight blend from an arched back to a folded one moved the chest a hand's width in one
frame.

### 2026-09-23 — Blood mage M2: essence pools and the drink
**Changed** Every hit she lands spills a pool under the target with a volume equal to the
damage; radius is 0.24 per root of the volume, pools drain at 10 a second, merge on overlap,
and are capped at four per mage. The effect array grew from eight slots to twelve. A move
landed over a pool drinks its column's share (sweep 35%, Reap 60%, Bloodletter 30% on the way
home, spike the whole pool), converted out of grey and never past it, and the pool loses
exactly what she got; the drink comes before the spill. `cargo run -p sim --bin essence`
prints the pool and drink tables.

**Why** The heal has to be somewhere. A pool is a place, and making the heal a place is what
turns "aggression" from an adjective into a direction on the floor.

**Reverted:** the Bloodletter drinking on its cuts as well as on its crossing. Landed on a
Reap's pool it took three shares in one throw — the out-cut, the return crossing and the
return cut — and came back with 85 against the Reap's 64. It drinks once per pool now, on
the way home only.

**Moved:** the drain from 12 to 10 a second and the sweep's cost from 6 to 3, so that the
relationship *a cast landed over a pool of its own making returns more than it cost* holds
for the sweep. At 12 a second a sweep's smear (22) was down to 14 by the time the next
sweep could land on it, and 30% of 14 was the cost exactly. The relationship is pinned in
`essence.rs`; both numbers are knobs.

**Verdict** open. C2's questions — did you know where to stand, did the heal feel earned or
automatic, does four pools clutter the arena — are the person's.

### 2026-09-23 — Blood mage M3: the blink, the haul, the spike on a pool
**Changed** A dodge with the crosshair on one of her pools puts her in it and spends it, on
the ground and in the air; the airborne one costs the airdodge and lands her. All four Grasp
arms on the creature haul her to the contact point at the reel speed. The Black spike is one
event: on bare floor the move's own disc hits, launches (9 m/s), slows and spills; on a pool
the whole pool erupts at its radius, launching and slowing everything in it and drinking all
of it. The drain field and its three knobs are gone; the spike stands for 20 frames as the
thing you can see. `feel.rs` gains *every class has a move that carries the body*, with the
Dual mage's row pending.

**Why** The three payoffs from one object: the blink is the movement, the haul is the
utility, the eruption is the damage, and none of them is a move added to fill a slot.

**Reverted:** asking `aim::pointing_at` about a pool. With the pool's radius folded into the
slack, the column it tests was six metres tall, and a blink aimed at the sky went through.
`aim::pointing_at_disc` is the same ray against a short cylinder on the pool's own disc, 1.5 m
tall — a fourth function in `aim.rs` rather than an angle worked out beside the dodge.

**Found:** the spill has to read the victim as they stood when the hit landed. Read after
the launch, a spiked victim was airborne and spilled nowhere, which made the one move meant
to seed a pool at range the one move that never did.

**Verdict** open. C3's questions — does the loop occur to you unprompted, is blink-or-drink a
real decision, does the eruption read as the floor coming up — are the person's.

### 2026-09-23 — Blood mage M4: the creature, the hunt, the costs
**Changed** The scripted hunter Reaps on right click; the hunt report gains THE BLOOD (pools
made, health drunk, health drunk while the creature was toppled). Costs to the proposal's
shape: sweep 3, Bloodletter 8, Reap 55, Grasp 60, spike 80.

**Measured** One scripted hunt as the class runs to completion (the hunters went down at 143
s, one topple, 75 rides): 5 pools made, 2417 health drunk, 32 of it while the creature was on
its side. The instrument's creature table: a sweep on a standing Ridgeback leaves 22, on a
toppled one 30, and a Reap on a toppled one 153 — the largest pool in the game.

**Verdict** open. C4 is one hunt and two versus rounds against a Champion who moves and one
who trades, and its questions — do the pools vanish before you can use them, and does that
feel like counterplay or like the class not working; do you out-heal a trader, by how much;
do you bank grey for the reach or heal as soon as you can — are the ones this entry cannot
answer.

### 2026-09-23 — Blood mage: the pool is a figure, a drink is one and done, the scythe stands up
**Changed** Three things from the first look at it, all presentation-and-rule pairs, since
the rule is that what is drawn is what is tested. A pool is a shadowy column the size of the
body it came out of — full at 110 essence, never under a third of a body, shrinking as it
drains and drawn more solid the more is left — instead of a disc 0.24 m per root of essence
across the floor. A drink **spends the pool**: the move's share of what is left comes back,
the rest is lost with it, and the pool is gone. The scythe is drawn as a haft and a flat
blade, the tip exactly at the hit volume's end while she swings and standing upright beside
her at rest; its base reach went from 2.8 m to 2.4 m. And a spike cast on a pool erupts at
0.3 m per root of the pool's essence, 3.4 m tall, at ×1.5 damage, where before it was the
pool's own radius at the spike's own damage — which on a small pool was *smaller* than the
bare spike and looked the same.

**Why** The blood spatters were far too big and spread over the ground; a pool could be hit
over and over for health, which is a heal with no decision in it; the scythe read as a
five-metre pole straight out along the facing; and there was no visible or numerical
difference between a spike on bare floor and one on a pool, which was the coolest idea in
the kit.

**What one-and-done changes underneath:** a sweep repeated on one spot no longer builds a
pool — each sweep drinks the last one's figure and leaves its own. Only the moves that do
not drink (the Grasp's four arms) pile essence on one spot, and the test that said five
hits make one pool now asks the Grasp. The shares stay meaningful the other way round: a
sweep through a Reap's pool takes a third and wastes the rest, which is a reason to save a
big pool for the Reap.

**Not done:** the figure starting as a shadow of the target's own model and coalescing. It
is a column; a fading copy of the target's skeleton is presentation and is the next step.

**Verdict** open. Unplayed since the change.

### 2026-09-23 — Blood mage: costs are a share of red, the scythe collects, the spike chains
**Changed** Four things from the second look. Every `Cost` on the class is now a percentage
of her *current* health rather than a flat number: 1 / 6 / 1 / 7 / 9, which at full health is
10 / 60 / 10 / 70 / 90 and at a fifth of a bar a fifth of that. The scythe collects: on every
active frame a sweep or a Reap drinks any pool of hers the blade passes over, nobody needed
in the way, except a pool younger than the swing itself. A bare spike is just a spike, drawn
without a skirt; an eruption keeps the disc, and now sets off every other pool inside its
radius, each of those setting off what it covers. And the weapon rides the hands the clips
put on it — the pole through the leading hand and past the trailing one, the tip at the hit
volume's end while it is out and at the live reach along the hands otherwise — and stands
at rest, butt on the floor under her left hand and the blade curving forward a little over
her head, drawn broader across the blade the more grey she carries. (It was carried low and
horizontal for an hour, to keep the blade out of the crosshair; that stopped it reading as a
scythe, which matters more.) The
sweep's drink went from 35% to 50% so a full-health sweep, whose cost is the largest it can
be, still returns more than it cost through its own pool.

**Why** At low health a flat cost was the class burning itself to death trying to get back
into the fight, and at high health it was too slow a way to open reach; a percentage does
the right thing at both ends. Swinging the scythe through a pool with nobody in it did
nothing, which is the opposite of what the weapon says. The spike and the eruption looked
the same, and an eruption that stopped at one pool made pool placement not matter. The
drawn scythe sat on a line the arms were not on and was pinned to the world's axes, and
with the blade at chest height it was over the crosshair.

**Watch for** the chain: three pools in a row is 700 essence and a launch across the whole
floor for one press. It is bounded by the pools there are and by one-and-done, but whether
a good Grasp-then-spike is a reward or a round is C3's question now.

**Verdict** open. Unplayed since the change.

### 2026-09-23 — Blood mage: the swing is essence, and right click is a bleed
**Changed** Two things. The sweep's hit volume now grows in **width** with grey as well as
reach — `Scythe width at full grey`, ×2 — and its damage curve went from ×1.3 to ×1.6 at a
full bar. The weapon itself no longer grows at all: it is iron, drawn at the row's reach in
her hands, and the volume is drawn around it as essence — the pools' own material, in the
hit test's exact capsule, more solid the more grey she carries, lingering and fading through
the first eight frames of recovery. And the Reap is gone from right click, replaced by the
**Haemorrhage**: a bolt (radius 0.7 m, 9 m in fourteen frames, 30 damage, 4% of red) spent
on the first body it reaches, which opens a three-second bleed of 8 every twelve frames,
each tick spilling a pool under the victim wherever they are. The pool cap went from four to
eight so the trail can exist. The scythe stands under the right hand now, so the left is
free to throw and cast without the weapon following it.

**Why** A weapon that got physically longer with grey looked like the model changing size
rather than the class's mechanic; the thing that grows is her blood, and it should look like
it. A wider volume is what turns grey into collection — the pools the swing passes near are
drunk — which pays the aggression the class is for. And the Reap was a second swing on a
class whose one swing already scales, while the kit's two payoff tools, the spike and the
Grasp, were both genuinely hard to land with nothing easier beside them. The bleed is the
easier thing, and it is not a payoff itself: it marks. A bleeding fighter is a fighter whose
every spike is an eruption and whose trail is a fuse.

**Measured** (`cargo run -p sim --bin essence`): a sweep's volume at 500 grey is 3.0 m long
and 0.67 m across against 2.4 by 0.45 at none, at ×1.30 damage; the bolt on a standing
dummy bleeds 120 over 15 ticks into one pool, and on a walking one into a trail 1.4 m a
stride, of which the last four strides are alive at any moment.

**Watch for** the trail's short life (a tick's pool drains in under a second) and whether
that reads as a fuse or as nothing; and the bolt landing too easily at 0.7 m, which would
make the marker free.

**Verdict** open. Unplayed since the change.

