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
