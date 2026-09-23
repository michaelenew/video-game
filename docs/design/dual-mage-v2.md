---
status: proposed — v2 of the mechanic, nothing built
decided: 2026-09-23
supersedes: dual-mage.md (the meter, the depth curve, ascension) once built; the kit in kits/dual-mage.md keeps its moves
sources: dual-mage.md, kits/dual-mage.md, docs/archive/combat-design/statera-skills.md, this design thread
---

# Dual mage — v2: two bars, and the hill between them

**Identity.** Two beings that would each kill her alone, and she is what keeps them apart.
Feed one and it grows and the other starves; the further apart they get the faster they pull
and the more of her they burn through. Feed *both* — goad them into a frenzy together while
holding them level — and she gets what neither could give her alone: wings.

The verb is **goad**. The single bar in [dual-mage.md](dual-mage.md) had nothing to contain:
nothing moved on its own, so balance was free and the centre was only the weak spot. Two bars
with a runaway between them make balance an unstable equilibrium — a ball on the top of a hill
— and containment becomes something the hands do, constantly, rather than a word in the
fantasy paragraph.

**The action plan for building this is [plans/dual-mage-v2.md](plans/dual-mage-v2.md).**

## What was wrong with one bar

The single bar conflated two different things: *how powerful she is* and *how unstable she
is*. Deep meant both. So the only way to be strong was to be one-sided, "riding the edge" was
a position rather than an act, and a player who goaded both beings hard ended up at the
centre, which the old design called weak. If you have driven both forces into a frenzy you
should not be neutral. Two bars separate the two quantities, and the class gets two different
edges to ride instead of one.

## The mechanic

### Two bars

**Dark** and **Light**, each from empty to full, side by side. Nothing is signed; there is no
centre.

- **Goading.** The dark auto raises Dark by a little; the light auto raises Light by a little.
  A cast raises the bar of the force she is *carrying* by more, and the finisher by a lot.
  Which force she carries is still the arm she last punched with — the whole of the
  carried-force rule in [dual-mage.md](dual-mage.md#the-last-auto-is-the-force-you-are-carrying--revised-2026-09-13)
  survives, and so does the two-form Lance.
- **The struggle.** Every frame the two bars are compared. Inside a **band** around equal,
  nothing moves on its own. Outside it, **the higher bar rises and the lower falls**, at a
  rate that grows with how far outside the band they are, up to a cap. That is the hill: a
  small lead is stable, a large one runs away.
- **The burn.** Health drains at a rate that grows with the same excess, capped, and never
  past one health. There is no burn inside the band, however high both bars are.
- **Calm.** Both bars fall slowly on their own, always. A mage who stops goading settles
  toward empty. This is what makes a tier something she *holds* by fighting rather than a
  level she reached.

Three quantities fall out, and each is read by something different:

| Quantity | What reads it |
| --- | --- |
| **The carried bar** | How hard a cast hits and how big it arrives — today's `state::depth`, pointed at one bar instead of at distance from a centre |
| **The lower bar** — *frenzy* | The tiers below: what her body can do |
| **The gap** | The drift and the burn: how fast she is losing it |

So she can be **powerful and stable** — both high, held inside the band, which is hard — or
**powerful and unstable** — one bar high, easy to reach, and burning while the other collapses.
A deep Judgement still nearly throws her over an edge. It throws her *sideways* now, and what
she loses when the low bar collapses is the wings.

### The tiers — what frenzy does to her body

The lower bar gates them, so both beings have to be fed. Each is movement, which is the leg
this class has never had, and each falls out of the mechanic rather than from a new button.

| Lower bar at | She gains |
| --- | --- |
| **half** | **The dodge is a blink.** Shift plus a direction puts her where the dodge would have ended, on its first frame, and she is invulnerable for the dodge's own window. In the air it is the airdodge, and it is spent the same way. A blink stops at the first terrain in its way, asked of `aim::clear_between` |
| **three quarters** | **A second jump**, once per airtime, and a **slower fall**. The first class to answer the README's open double-jump row |
| **both full** | **Ascension.** Wings erupt, and she may jump as often as she likes |

The tiers are on the *lower* bar and not on the sum for one reason: a sum can be reached
one-sided. Requiring both to be high is what makes the climb a rhythm of alternating hands —
dark hand, dark Lance, light hand, light Sweep — and that rhythm uses the two-form kit in both
forms, which nothing in the old design ever forced.

### Ascension

Both bars full. Nothing else triggers it, and the drift never delivers it — the drift pulls
the bars apart, so the only way to the top is to goad both while holding them level, which is
the hardest thing the class can do and is meant to be.

- **Wings erupt** and she may jump without limit: every press of space in the air is a wing
  beat.
- **No dodge.** She flies instead. Loss of control is loss of the option to decline, as the
  original document says.
- **Casts fire at the top of the power curve**, because both bars are at the top.
- **Health is the clock**, as built today, and **landing hits pulls some back** — the refund
  the old design wrote and never built.
- **It ends with both bars empty** and a stagger graduated by how much she landed. The vent.
  Then she climbs again.

So a match has a shape: climb, hold a tier, ride it, ascend, vent, climb. In versus at sixty
seconds that is roughly once per match if she is good, which is what a nova should be.

### On her back

**The two bars are two wings**, dark on the left and light on the right — the same sides as
the arms that goad them — each as long as its bar. Lopsided wings are the gap, readable across
the arena by both players; full span is ascension. The HUD carries the two bars too, but the
wings are the display, and the thing the tiers unlock is the thing everyone is already looking
at. That answers the old open question about colour on the body by making the body the meter.

## What stays

The kit in [kits/dual-mage.md](kits/dual-mage.md) is unchanged in its moves: two autos that
pull and shove, Lance in two forms on middle click, Judgement on `Q`, Sweep on `E`. The
carried force, the three tiers of push, the depth curve's shape, the exemption of the autos'
size from scaling, the rule that range and frame data never scale with a bar — all kept, and
the last one is *stronger* here, because the thing that does scale with the bars is drawn on
her back.

## What goes

- The single signed bar, its centre, and the deep threshold on one side.
- The one-sided burn. Burn is a function of the gap now, and only the gap.
- "The far-side auto is the only way back." It is still the way back, but it is *urgent* now
  rather than optional, because the low bar is falling while she waits.
- "Oscillating at centre is correctly weak." Alternating is how she climbs. What is weak is
  not goading at all.

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

## What it costs to build

- **`Mechanic::Meter` becomes two bars.** `value: i32` becomes `dark` and `light`, `colour`
  stays as the carried force, `ascending` stays. Everything that read `value.abs()` reads the
  carried bar for power and the lower bar for tiers.
- **The struggle is a dozen lines in `step_mechanic`**: gap, band, excess, drift up and down,
  burn, calm. All integers, all knobs.
- **`state::depth` reads one bar** rather than distance from a centre. Its floor and ceiling
  knobs survive unchanged.
- **The blink is the dodge branch** the Reaver already has, with a tier check in place of the
  shadow check and `aim::clear_between` in place of `aim::pointing_at`. No new aiming.
- **The second jump is a jump impulse** allowed once per airtime when the tier is held, reset
  on landing, unlimited while ascending. Space-in-the-air has one existing exception, the
  Champion's uppercut, and this is the second.
- **Two wing meshes** scaled by the two bars in `view`, with a test that the drawn span is the
  bar, the way `kinematics.rs` checks the blade against the fist.
- **The HUD's one bar becomes two.**

**Tests that change.** `dual_mage.rs` and the meter assertions in `feel.rs` are rewritten
against two bars. New relationships worth pinning: *inside the band nothing drifts and
nothing burns*; *outside it the higher bar always rises and the lower always falls*; *the
drift alone can never reach ascension*; *a tier is lost when the lower bar drops below it*;
*ascension ends empty*. And the roster-wide one from the Blood mage proposal: **every class
has a move that carries the body**, which she passes at the first tier.

## Open questions

- **Goading on a whiff.** Autos steer on the press, so a mage could climb to the blink tier in
  an empty arena. Calm decay may be enough to make that slow and fragile; if not, the knob is
  how much of the goad a whiff pays, and it is a knob rather than a rule so it can be played.
- **The band's width against a cast's push.** The intended cadence is: one cast from level
  stays in the band, two do not, a finisher never does. Whether that is right is the first
  thing to play.
- **Should the two hands have two rhythms?** Dark slow and heavy, light fast and sharp, so
  which arm she is carrying is felt in the tempo rather than read off a wing. Frames differ by
  form and never by bar, so the readability rule holds. Not in v2; recorded as the first thing
  to try after it.
- **Both forms at once during ascension.** A Lance that bursts *and* tethers. It is the
  obvious payoff and it is one more thing to build; second pass.
- **Does the calm decay make the plateau too hard to hold in a real exchange?** It is meant
  to be hard. If it is impossible, the decay is the knob, not the band.
- **What the wings do to her hurtbox and her silhouette.** They are drawn, not tested against;
  the question is whether a wingspan reads as a bigger target and lies.
- **The names of the two beings**, still. "Goad" is the verb this thread produced and the
  autos may simply be the goads.
