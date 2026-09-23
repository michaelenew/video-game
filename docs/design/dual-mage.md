---
status: built through the wings, 2026-09-23 — unverified in play; the single bar it replaced is recorded at the end
decided: 2026-09-10
revised: 2026-09-23
formerly: Statera
supersedes: docs/archive/combat-design/statera-skills.md (resource system), docs/archive/combat-design/class-builds.md (Statera section); dual-mage-v2.md is folded in
---

# Dual mage

Formerly **Statera**. Renamed to pair with the Blood mage — two casters named for what they
run on — and because "statera" (scales, balance) described the meter rather than the person.

**Identity.** Two beings that would each kill her alone, and she is what keeps them apart.
Feed one and it grows and the other starves; the further apart they get the faster they pull
and the more of her they burn through. Feed *both* — goad them into a frenzy together while
holding them level — and she gets what neither could give her alone: wings.

The verb is **goad**. This is a containment story, not a channelling story: the human is not a
conduit, they are a vessel under load, and the load is two things that want to be apart.

> **Built 2026-09-23**, all of it below through the wings, in one thread against
> [plans/dual-mage-v2.md](plans/dual-mage-v2.md), and **played once the same day**: the first
> values came back overtuned — too much damage, too little burn, bars that drained too fast,
> spells that fed them too much — and were re-tuned to the **benchmarks** below, which are the
> class stated as player actions and their outcomes. `cargo run -p sim --bin goad` prints them
> and `crates/sim/tests/dual_mage.rs` pins them. The feel log for that date carries both passes
> and the play scripts still open. The single signed bar it replaced is under "Was" at the end,
> and the proposal it was built from is [dual-mage-v2.md](dual-mage-v2.md).

## The mechanic

### Two bars

**Dark** and **Light**, each from empty to full (`tuning::meter_max`, 100), side by side.
Nothing is signed; there is no centre.

- **Goading.** The dark auto raises Dark by a little; the light auto raises Light by a little.
  A cast raises the bar of the force she is *carrying* by more, and the finisher by a lot.
  Which force she carries is still the arm she last punched with — the whole of the
  carried-force rule below survives, and so does the two-form Lance.
- **The struggle.** Every frame the two bars are compared. Inside a **band** around equal,
  nothing moves on its own. Outside it, **the higher bar rises and the lower falls**, at a
  rate that grows with how far outside the band they are, up to a cap. That is the hill: a
  small lead is stable, a large one runs away. The higher bar gains exactly what the lower
  loses, so the drift moves what she has from one being to the other and never makes more of
  it — with no input the two together can only fall.
- **The burn.** Health drains at a rate that grows with the same excess, capped, and never
  past one health. There is no burn inside the band, however high both bars are.
- **Calm.** Both bars fall slowly on their own, always. A mage who stops goading settles
  toward empty. This is what makes a tier something she *holds* by fighting rather than a
  level she reached.

| Input | Which bar | How far |
| --- | --- | --- |
| Dark auto (`L`) | Dark, and she is now dark | `tuning::meter_auto_push` — 5 |
| Light auto (`R`) | Light, and she is now light | 5 |
| The finisher (`Q`) | Whichever force she is carrying | `tuning::meter_finisher_push` — 20 |
| Anything else | Whichever force she is carrying | `tuning::meter_cast_push` — 9 |

All of it **on the press**, whether or not it connects — see "Autos are the steering wheel"
below. The band is measured against the pushes and the climb against the calm, so the four move
together: the first pass was 8, 18 and 38 against a band of 30, and it read as spells that fed
the bars too much.

| The hill | Knob | First value | Chosen against |
| --- | --- | --- | --- |
| Band | `meter_band` | 16 | Benchmark B8: one cast from level stays inside, two leave, the finisher always leaves, and the alternating rhythm — auto and cast on one side, then the other — never leaves. So at least an auto plus a cast and less than two casts: 14 to 18 |
| Drift | `drift_gain` / `drift_cap` | 1 per second per unit outside the band, at most 3 per second | B6: uncorrected, a Judgement from level empties the lower bar in eight to twelve seconds; answered by the far-side hand, it is back inside within an exchange; far-side autos alone hold or claw back. Was 2 and 10, which read as bars draining too fast |
| Burn | `burn_gain` / `burn_cap` | 4 health per second per unit, at most 25 per second | B7: ignoring a runaway for ten seconds costs about a Judgement's worth; fully one-sided for half a round costs half to four fifths of a bar. Was 1.5 and 15, which read as a footnote |
| Calm | `meter_calm` | 2 per second | B4 and B5: the climb and the idle fall below. Was 6 |

Three quantities fall out, and each is read by something different:

| Quantity | What reads it |
| --- | --- |
| **The carried bar** | How hard a cast hits and how big it arrives — `state::depth`, pointed at the bar of the force the move is made of: an auto reads its own, everything else the carried one |
| **The lower bar** — *frenzy* | The tiers below: what her body can do |
| **The gap** | The drift and the burn: how fast she is losing it |

So she can be **powerful and stable** — both high, held inside the band, which is hard — or
**powerful and unstable** — one bar high, easy to reach, and burning while the other collapses.
A deep Judgement still nearly throws her over an edge. It throws her *sideways* now, and what
she loses when the low bar collapses is the wings.

### Benchmarks — the class as player actions and outcomes

Set after the first play, which came back overtuned, and built to. Each is a test in
`crates/sim/tests/dual_mage.rs` named for it, and `cargo run -p sim --bin goad` prints the
numbers. An *exchange* is a Judgement's startup to the end of its recovery, twice: 110 frames.

| | The player does | What happens | Measured |
| --- | --- | --- | --- |
| **B1** | Alternates hands for half a round with everything landing on a target that never moves | One to two and a quarter health bars, the last quarter being the six seconds of ascension the climb reaches near the end. Against a person a third lands, which is one kill a round from the sustained game | 2.09 bars |
| **B2** | One dark auto, then dark casts only, a Judgement every time it is up, on a target standing in all of it | The biggest number the class makes: two to three and a half bars, and it costs her at least half of her own | 2.24 bars dealt, 0.64 lost |
| **B3** | Throws a Judgement from a full bar | At most a quarter of a health bar, strike and field together; from an empty bar still at least eight per cent | 25%; 9% |
| **B4** | Alternates cleanly from empty | The blink in ten to fourteen seconds, the second jump in sixteen to twenty-one, the wings in twenty to twenty-six — once a round, with commitment | 10.9 s, 16.8 s, 21.2 s |
| **B5** | Stops at three quarters | The second jump is gone within an exchange; the blink is kept for at least two and gone inside seven | 6.8 exchanges |
| **B6** | Throws a Judgement from level and does nothing | The lower bar is empty in eight to twelve seconds | 10 s |
| | …and answers it with the other hand: the far-side auto, the far-side cast, an auto or two | Back inside the band within one exchange of the finisher recovering; far-side autos alone hold it or claw it back | passes |
| **B7** | Ignores the runaway for ten seconds | Fifteen to twenty-five per cent of a health bar, about what the Judgement did to them. Fully one-sided for half a round: half to four fifths, and never all of it | 20%; 75% |
| **B8** | One cast from level; two; the finisher; alternating | Inside the band; outside; outside; never outside | passes |

What moved to get there, from the first pass: the pushes 8/18/38 → 5/9/20 and the band 30 →
16; the calm 6 → 2 a second; the drift 2 per unit and 10 at most → 1 and 3; the burn 1.5 per
unit and 15 at most → 4 and 25; the depth curve 0.5–2.0 → 0.6–1.4; and the base numbers, the
autos 52 → 30, Lance 58 → 32 with a burst of 150 → 60, Sweep 95 → 55, Judgement 215 → 140
with a field of 12 → 5 a tick, the tether's drain 9 → 2 a tick, and the dark Sweep's heal 22 →
5 a target. **The target never moves, in the test and in the instrument alike**: the goad's
dummy is held where the autos land, because the light hand's shove otherwise walked it out of
range on the second press and the instrument read half of what the test did. And the heal had
to come down with the damage, or one-sided spam paid its own burn back: the dark Sweep's heal
and the tether's leech against a target standing in everything were most of the burn.

### The depth curve

**Power scales continuously with the bar, and it is one function.** `state::depth` is a
straight line from `tuning::depth_floor` at empty to `tuning::depth_ceiling` at full — six
tenths to fourteen tenths — and everything she throws is multiplied by a point on it: damage,
knockback and pull, launch, leech, what a field drains and how long it lasts, and how big what
arrives is. The spread was half to double in the first pass, and with the old base numbers a
Judgement from a full bar was forty per cent of a health bar.

**Two things it deliberately never touches: how far a move is thrown, and how fast it comes
out.** Spacing and frame data are what two players read each other with, and a class whose
range or startup changed continuously with a bar only one of them can see would be unlearnable
from either side. That rule is *stronger* here than it was on one bar, because the thing that
does scale with the bars is drawn on her back.

**A cast is worth where you were standing when you pressed the button.** Throwing anything
moves a bar on the press, and the finisher moves it a long way — so a cast whose power were
read after its own push would be worth more than the bar said.

**The two autos are exempt from the size half of it**, and only that half. Their reach is
pinned to the punch that throws them — see [kits/dual-mage.md](kits/dual-mage.md).

### The tiers — what frenzy does to her body

The lower bar gates them, so both beings have to be fed. Each is movement, which is the leg
this class never had, and each falls out of the mechanic rather than from a new button. A tier
is held while the lower bar is at or above its threshold and lost the frame it drops below —
including mid-air.

| Lower bar at | She gains | Knobs |
| --- | --- | --- |
| **half** (`tier_blink`, 50) | **The dodge is a blink.** Shift plus a direction puts her where the dodge would have ended, on its first frame, and she is invulnerable for the dodge's own window and then stands through its tail. In the air it is the airdodge, and it is spent the same way. A blink stops against the first terrain or stone in its way — `aim::blink_to` decides, and it is stricter than the Reaver's dash: a low platform her head would clear is still a wall to her feet | — |
| **three quarters** (`tier_jump`, 75) | **A second jump**, once per airtime, and a **slower fall**. The first class to answer the README's open double-jump row. Space in the air has one other exception, the Champion's uppercut; this is the second, and it is per class | `second_jump` 0.8 of the first · `slow_fall` 0.6 of the fall cap |
| **both full** (`tier_wings`, 95) | **Ascension.** Wings erupt, and she may jump as often as she likes | below |

The tiers are on the *lower* bar and not on the sum for one reason: a sum can be reached
one-sided. Requiring both to be high is what makes the climb a rhythm of alternating hands —
dark hand, dark Lance, light hand, light Sweep — and that rhythm uses the two-form kit in both
forms, which nothing in the old design ever forced.

**Why "full" is 95 and not 100.** The calm runs every frame and the two bars are pushed by two
different presses, so the moment one is topped up the other has already lost a little; both can
never be at exactly the top on the same frame. The third tier is the lower bar at 95, which is
what the other bar keeps through one move's worth of calm with room to spare. It is a threshold
on the lower bar like the other two, which is also why it is written beside them.

### Ascension

Both bars full. Nothing else triggers it, and the drift never delivers it — the drift pulls
the bars apart, so the only way to the top is to goad both while holding them level, which is
the hardest thing the class can do and is meant to be.

- **Wings erupt** and she may jump without limit: every press of space in the air is a wing
  beat.
- **No dodge.** She flies instead. Loss of control is loss of the option to decline.
- **Casts fire at the top of the power curve**, because both bars are at the top.
- **Health is the clock** — `ascension_frames` (360, six seconds) of `ascension_drain` (2) a
  frame, about seventy per cent of a health bar, inside the "half to all of it" the original
  design asked for — and **landing hits pulls some back**: `ascension_refund` (40) per hit, the
  refund the old design wrote and never built. It was three seconds, and at three a player had
  time for two or three abilities and it was over before they had noticed it had begun.
- **It ends with both bars empty** and a stagger graduated by how much she landed: the share of
  the drain her hits refunded, read against a ceiling (`ascension_stun`, 40 frames, for landing
  nothing) and a floor (`ascension_stun_floor`, 10, for paying it all back). The vent. Then she
  climbs again.

So a match has a shape: climb, hold a tier, ride it, ascend, vent, climb. In versus at sixty
seconds twenty-one seconds of clean alternating reaches the wings, which against a person is
roughly once per match if she is good, which is what a nova should be.

**Loss of control does not mean loss of input. It means losing the ability to decline.** In a
fighting game, control *is* the option to not commit — to wait, block, dodge, reposition, do
nothing. Remove that, and you have genuine loss of control with every input still mattering
completely. The analogy is a car with the accelerator stuck: you still steer, you cannot stop.

**Counterplay** is what keeps it from degenerating into a race to nova every match. Opponents
beat it by playing evasively — deny the hits and the refund never comes — by CC, because she
cannot dodge out, and by blocking, which eats the damage and denies the refund. The Bulwark is
ascension's hard counter. You win with it by timing it for when your opponent is already
vulnerable, not by reaching it as fast as possible.

### On her back

**The two bars are six wings**, three a side, dark on the left and light on the right — the
same sides as the arms that goad them. A wing does not grow: it **materialises**, whole, and
**a wing is a tier** — a side's first wing arrives when that bar reaches the blink's threshold,
its second at the second jump's, its third at the top the wings read. Those are the three marks
the HUD's track already carries, so a wing appearing on her back and a tick passing on the bar
are one moment, the moment something about what her body can do changes. (The first cut put
them at thirds of the bar, a fourth set of marks nothing else used, and it read as wings
arriving in the middle of nowhere.) So a bar is read as a count, one, two or three, which is
what can be read from across the arena; a wing half a metre long is a line, not a wing.
Lopsided wings are the gap, readable by both players; three and three is a mage about to
ascend, and ascension is all six. The HUD carries the two bars too, growing away from each
other from a shared middle, but the wings are the display, and the thing the tiers unlock is
the thing everyone is already looking at. That answers the old open question about colour on
the body by making the body the meter.

**Three sizes, and the biggest comes first.** From the top of her back down: the smallest,
the biggest, the middle one — the seraph's proportions, with the great wing where the shoulder
blades are. They arrive biggest first: the blink puts the great wing out, the second jump the
lower one, the top the small one above. So a mage at half already has a wing you can see
across the arena, and the last one is a flourish rather than the thing you are waiting for.
The great wing reaches up and out from the shoulder blade, the lower one out with its
primaries hanging, and the small one **floats** above the great wing, off her shoulder and
above her head, bound to nothing — it is the highest presence of that being, not a limb, and a
binding that is not physical is the point of it. None crosses her centre line. Each is a bird's wing, built the way a bird's is: an arm that rises from the shoulder to a
wrist well short of the tip, seven primaries fanning from that wrist like fingers — the
outermost reaching the full span, each next one shorter and hung lower, the last hanging down
— and secondaries hanging from the arm behind them, every feather a rounded slat and the wing
the translucent overlap of them. That bend at the wrist is the whole difference between an
eagle's wing and a butterfly's; the first cut was one sheet with a sawtoothed edge, and it read
as an insect's. Each wing stands in the vertical plane through its own length so that from the
front and from behind, where the two players are, it is seen face-on. The
great wing is 2.3 m, the lower 1.7, the small one 1.1; they are translucent, so they never hide
the fight behind them. `view::wings` decides the count, each wing's root, axes and length and
the silhouette, and `view/tests/wings.rs` holds the count, the order and the sizes to the bar
the way `kinematics.rs` holds the blade to the fist. Looked at in a headless capture
(`SHOT_BARS=70,100 ./scripts/screenshot.sh`) from both seats on 2026-09-23. The wings are drawn
and not tested against.

## The last auto is the force you are carrying — revised 2026-09-13

**The autos have sides. Nothing else does.** Left click is dark and right click is light,
and throwing one sets which of the two forces the mage is *carrying*. Every other input —
the committed cast, the key abilities — is made of that force and goads that bar.

It gives the colour something to *be*. Which force she is carrying is what decides which
form her abilities take — the light/dark split every ability is written with — so it has to be
a thing the player sets deliberately and can read off her own animation.

She is **always carrying one of the two**, dark to begin with: a vessel holding two forces is
holding one of them at any moment, and the version where she carried neither until her first
auto landed meant the first key pressed in a match did nothing. So a cast thrown before any
auto is a *dark* cast, and it moves a bar; there is no neutral start to be stuck at.

Three tiers of push rather than a number per ability. "Stronger abilities push harder" was a
formula over damage, which meant a knob nobody could find and a finisher that pushed about as
hard as a poke; three numbers, all in the Oven, are legible and are what a tuning pass can
actually move.

## Every ability has two forms, and the force she carries picks

**Which form an ability takes is the force she is carrying**, which is set by the last auto
she threw. It is *not* which button threw it. Only the two autos have a side at all.

**Built for the first time on 2026-09-16, on Lance.** Middle click throws one of two moves and
the arm she last punched with decides which: light bursts at the far end of the line, dark
tethers what it hits. They are two entries in the move table rather than one with a flag,
because the thing that has to differ is the **wind-up**. Sweep has a form split too and did not
need a second animation, because its shape is the same either way and only what happens to
whoever it caught changes. The kit is in [kits/dual-mage.md](kits/dual-mage.md).

### One pulls and one pushes — 2026-09-16

**The dark auto drags whoever it catches a short way toward her and returns a trickle of
health. The light auto shoves, and the real shove is out at the tip of the wing.** Same frames,
same shape, mirrored arms, opposite answers to the question of where the two of you end up
standing. That is what makes which arm she punches with a **spacing decision as well as a
bar decision** — a fragile melee mage stays attached to somebody with the dark hand and buys
herself room with the light one, and she cannot ask for either without also feeding a being.

### Autos are the steering wheel

> **"A whiff steers nothing" is suspended, 2026-09-13.** It read well and it was unplayable.
> With the autos steering only on contact, a mage with nothing in reach could press every
> button on the class and watch the bar sit at zero. Steering happens on the **press** now,
> and it stayed that way after play: managing a frail character at short-to-mid range while
> holding the bars is already the challenge.

Scroll click and the keys are neither left nor right, so they cannot pick a being — they feed
whichever one she is already carrying. **That is what makes middle click the right home for a
two-form cast**: on a button with no side the push is settled by the force she carries, and the
*form* comes from the same place.

### Why not a neutral form

An earlier draft gave each ability three states — a neutral behaviour at the old centre, plus
Light and Dark at depth. It broke at zero: a neutral form has no side, so nothing voted and the
meter could not leave the middle. There is no centre now, but the argument still holds for the
start of a match: she is dark until she throws a light auto.

## Coming back

The far-side auto is still the way back, and it is **urgent** now rather than optional,
because the low bar is falling while she waits. Far-side autos alone hold a runaway and claw
it back slowly; the far-side **cast** wins it — a light auto, a light Sweep and an auto or two
bring a Judgement's runaway back inside the band within an exchange — so the way back is a
commitment, and the auto buys the time to make it.
Turning round *requires* getting into auto range, which forces the class into melee exactly
when it is most powerful and most fragile.

## What goes, and what stays

Gone: the single signed bar, its centre, and the deep threshold on one side. The one-sided
burn — burn is a function of the gap now, and only the gap. "Oscillating at centre is correctly
weak" — alternating is how she climbs; what is weak is not goading at all.

Kept: the carried force, the three tiers of push, the depth curve's shape, the exemption of the
autos' size from scaling, the rule that range and frame data never scale with a bar, the moves.

## Why this shape

**It gives the mechanic something to contain.** A single bar sat still. Two bars with a
runaway between them are a hill she is standing on top of, and the constant small correction
— punch with the weaker hand — is the fantasy in the fingers.

**Both edges have a reason.** One-sided is burst and danger. Both-sided is mobility and
difficulty. The old design had one edge and a dead middle.

**The third leg comes from the object.** Blink, double jump and wings are what frenzy does to
her body. The Champion vaults on the spear, the Elementalist rides the stone, the Reaver
crosses to the shadow, and now the Dual mage flies on what she has goaded. No new button, no
new move, and the two-form kit gets used in both forms because the tiers demand it.

## Open questions

- **Are the benchmarks right?** They were set from one play and one complaint — too much
  damage, too little burn, bars too fast, spells feeding too much — and they are the first
  thing the next play should argue with. Each is one row above and one test.
- **The plan's calm criteria were dropped.** It wanted stopping at three quarters to lose the
  blink inside one exchange *and* the climb to reach it inside one; the person asked for slower
  bars, so the calm went down to 2 and the blink now outlives seven exchanges of stillness. If
  a fragile plateau is wanted back with a slow climb, the answer is a calm that scales with the
  bar rather than a flat one, which is a shape change and not a number.
- **The band's width against a cast's push.** One cast from level stays inside, two do not, a
  finisher never does. Whether that cadence is right is still the first thing to play.
- **The finisher every fifty-five frames.** One-sided spam is three Judgements in seven
  seconds on a target that stands still, and it is most of benchmark B2. Its lockout cannot
  grow without breaking the rule that a lockout never outlasts the cheapest other thing in the
  kit. If B2 still reads as too much, the answer is in the finisher's own frames, not the bars.
- **Goading on a whiff.** Autos steer on the press, so a mage can climb to the blink in an
  empty arena — eight seconds of it, the instrument says. Calm may be enough to make that slow
  and fragile; if not, the knob is how much of the goad a whiff pays.
- **The refund counts every connection**, including a tether's ticks and a field's. That is
  the "sustained contact" flavour the old design wanted for the dark side, and it may pay the
  dark form back too well. Whether the refund should count hits or damage is a play question.
- **Should the two hands have two rhythms?** Dark slow and heavy, light fast and sharp, so
  which arm she is carrying is felt in the tempo rather than read off a wing. Frames differ by
  form and never by bar, so the readability rule holds. Recorded as the first thing to try
  after this.
- **Both forms at once during ascension.** A Lance that bursts *and* tethers. Second pass.
- **What the wings do to her hurtbox and her silhouette.** They are drawn, not tested against;
  the question is whether a wingspan reads as a bigger target and lies.
- **The names of the two beings.** "Goad" is the verb this thread produced and the autos may
  simply be the goads.

## Was — the single signed bar, 2026-09-10 to 2026-09-23

Recorded because most of its reasoning survives into the two bars, and because the argument
for replacing it is only legible against it.

**One bar with a centre and two ends**, Dark at one and Light at the other. Power scaled
continuously and symmetrically with distance from the centre — `state::depth` was a line from
the floor at zero to the ceiling at either end — so the centre was the weakest place to stand
and the ends the strongest and most lethal. There were no zones and no thresholds, which was
what removed the dead-zone problem of the three-state design before it. Past a **deep
threshold** on either side (65 of 100) she took a burn that grew with distance from centre and
stopped the moment she came back inside: relief from stopping rather than from a reward, which
was the containment beat. Driven all the way to an end the bar started **ascension** as a
clock and nothing else — three seconds of drain, nothing steering, and a flat stagger on the
way out at the centre.

**What was wrong with it.** The single bar conflated two different things: *how powerful she
is* and *how unstable she is*. Deep meant both. So the only way to be strong was to be
one-sided, "riding the edge" was a position rather than an act, and a player who goaded both
beings hard ended up at the centre, which the design called weak. Nothing moved on its own, so
balance was free and the centre was only the weak spot; the fantasy paragraph said
*containment* and the hands had nothing to contain. Two bars separate the two quantities, give
the mechanic a runaway to hold, and give the class two edges to ride instead of one.

**What it got exactly right**, and what the two bars keep: the carried force set by the last
auto, the three tiers of push, steering on the press, a depth curve that scales force and size
and never range or frame data, the autos' size exempt, and an ascension that is a clock with an
exit — the version before that had no exit at all, and "a cost with no end is not a cost, it is
a broken state you play around".
