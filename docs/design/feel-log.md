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
| Bellator | 3.7 m (2.1×) | 6.0 m (3.3×) | 6f | 8.2 m |
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
- **Uppercut** (Bellator) leaps, and **takes whoever it catches into the air with it**.
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

### 2026-09-11 — the mechanic button fired every frame you held it
**Reported** The Elementalist raises a structure once per frame while `E` is held; it should be
one per press.

**It was every class**, and each was broken in its own way. Measured, holding `E` for thirty
frames:

| Class | What you got |
| --- | --- |
| Bulwark | Shield pinned mid-throw — re-thrown every frame, so it never travelled and never planted |
| Bellator | Form cycled thirty times; which one you end on is a function of how long you held |
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
different size to walls than to attacks. The rest were ordinary invisibility — the Bellator's
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
