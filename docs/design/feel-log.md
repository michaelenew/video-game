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
- Should mobility vary *across* a move's phases — free during startup, rooted through
  recovery? That is a common shape and might read better than a flat rate.
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

### The Oven

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

It **rebuilt the box from the move table**, so it ignored the Bellator's weapon form, which
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
