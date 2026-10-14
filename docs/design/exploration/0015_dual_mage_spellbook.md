---
status: decided 2026-10-12 by the owner's brief, designed and built by Claude; unplayed
started: 2026-10-12
supersedes: 0010_dual_mage_spells.md (every move, the hex and its two reactions); kits/dual-mage.md §"Every move a spell"
depends: ../dual-mage.md (the two bars, the hill, the tiers and ascension -- unchanged but for the moment of ascending), 0007, 0014
---

# 0015 — The Dual mage's spellbook: on them, on the ground, on herself

## The brief (the owner, 2026-10-12, after playing 0010)

> The three clicks rework hasn't really landed like it needed to. Left and right click are
> extremely basic attacks, it doesn't have interesting tools for mobility. Go look at the shadow
> reaver's and elementalist's replays — see how dynamic those fights are, how mobile they are,
> and how impactful the combination of moves is. The dual mage really needs a pretty major
> rework. Every attack needs to feel like an MMO spell. Dark and light can't be push/pull —
> that reads really awkward — but need to be differentiated from each other and from the middle
> click line of abilities. The `Q` and `E` are still the basic Judgement, and I'd rather they
> were MMO style abilities with interesting effects and a lot of spectacle (still starting small
> and getting bigger along the light/dark track).
>
> The banner here is that every ability needs to feel like an MMO spell that you might find on a
> mage class in Guild Wars or FFXIV, but at the same time it needs to be recast into a melee
> range. It's cast to fight, cast to move, cast to defend; that's the real balance of the dual
> mage, and it's against the need to *slow down* and stop casting to make sure you ascend at
> just the right moment.

## What the replays say

Three tapes of one session, judged with `cargo run -p hunt --bin replay` (which now splits its
table by class — a tape is a session, and a class change travels as input):

| | Time | What carried the fight | Biggest single numbers |
| --- | --- | --- | --- |
| **Shadow Reaver** | 200 s | Whirl 74 thrown, 18 landed — and each one kept cutting 14–21 a tick for seconds while she did something else; Reave 14/4; Plunge 5/3 | Plunge 505, Reave 327 and 420, the burst 45 |
| **Elementalist** | 157 s | Cataclysm 18/6, Landfall 25, Fire carpet 21, pillars that burned on | 238, 227, 210, 190, 175 |
| **Dual mage** | 342 s | Reel 53/8, Flare 62/0, Phase 49/0 — the air row thrown as movement into nothing; Shade bolt 11/0, Sunray 23/0 | 48 (Dawn); everything else 3–9 |

Three things the first two have and the Dual mage did not:

1. **Something of hers stays in the world and keeps working** while she moves — the Whirl's
   blades, a carpet, a pillar, a stone. Every Dual mage spell was one hit and gone.
2. **A build and a cash.** Marks, then a finisher worth four times its base; stones, then a
   Cataclysm. Her hex and its reaction were the same idea at a tenth of the size: one hex, one
   reaction, 30–40 damage.
3. **Moving is attacking.** Gallows, the Plunge and the dash cross the arena *and* hit;
   Landfall arrives *and* plants. Her movement — Flare's kick, Phase, Reel — hit nothing in 164
   casts, and her timeline shows why: jump, Reel, land, jump, Flare, land, burning all the while
   because the air row she was using to move was all dark.

## The reframing: three kinds of target

An MMO mage's spells are told apart first by **what they are cast on**: a curse is cast on a
*target*, a consecration is cast on a *place*, a blink or a barrier is cast on *yourself*.
Those three targeting modes are the most familiar distinction in the genre, they are visibly
different on screen, and they give the three clicks three different questions to answer —
which is exactly the differentiation the brief asks for:

| Click | Force | Cast on | What it leaves | Its school |
| --- | --- | --- | --- | --- |
| **Left** | Dark | **them** | **Curses** that stack on a body, tick, and drain to her | The warlock: hexes, grasping hands, a maw |
| **Right** | Light | **the ground** | **Sigils**: consecrated discs that ward her and burn them | The cleric: smites from the sky, halos, sanctuaries |
| **Middle** | Twilight | **herself** | **Twin stars** that orbit her; her own body, moved | The astral mage: orbs, blinks, the balance itself |

Neither force pushes or pulls anybody. Dark *holds* (a root, a well that pins) and *takes*
(drains); light *strikes* and *protects*.

And the rows are the three verbs of the banner. The Champion's grid already makes the row the
situation — on the floor, in the air, leaving the floor — and each of those situations is
already one of the verbs:

| Row | Verb | Why the row is the verb |
| --- | --- | --- |
| **On foot** | **Cast to fight** | Standing is where a melee mage trades blows |
| **In the air** | **Cast to move** | She is already moving; a spell from the air takes her somewhere |
| **Leaving the floor** (space + click) | **Cast to defend** | Leaving is what defending is in this game — the dodge, the jump out |

## The grid

| | Left: dark — on them | Middle: twilight — on herself | Right: light — on the ground |
| --- | --- | --- | --- |
| **On foot — fight** | **Litany of night**: Umbral lash → Grasping shade → Devour | **Binary**: twin stars orbit her | **Litany of dawn**: Radiant palm → Halo → Smite |
| **In the air — move** | **Shadowstep**: to them | **Phase**: anywhere | **Descent**: down onto the ground as a smite |
| **Leaving the floor — defend** | **Veil**: out of reach as shadow | **Equinox**: level her bars | **Sanctuary**: a sigil at her feet |

| Key | Major | Grows along |
| --- | --- | --- |
| `Q` | **Abyss** — a black sun that pins and curses everything under it, then collapses | Dark |
| `E` | **Judgement** — columns of light, one to five, and a great sigil | Light |

Fifteen moves: the eleven slots the class had, re-cast as the new moves in place, and four
appended (the second and third links of each litany).

## The two things she leaves in the world

### Curses — dark, on a body

Every dark spell that lands **curses** what it hits: a stack of Umbra on the body, up to five,
drawn as dark motes circling it, one per stack.

- **They tick.** Every half second each stack deals `curse, tick per stack` (2, times her
  Dark power), and a tenth of it comes back to her (`curse, drained to her`). A five-stack
  curse is a damage-over-time that is a real share of the fight. The drink is small on
  purpose: at a half it paid for the burn of a fully one-sided mage on its own (see the
  numbers below).
- **They fade one at a time.** One clock for the whole curse: a new stack resets it to
  `curse, one fades every` (1.5 s); when it runs out one stack goes and it starts again. The
  Reaver's marks run on the same shape of clock, so a player who learnt one has learnt both.
- **They are what light cashes.** See the Purge, below.
- **A cursed body is a door.** Shadowstep reaches further for a cursed body than a clean one.

### Sigils — light, on the ground

Smite, Descent, Sanctuary and Judgement **write a sigil** where they land: a disc of
consecrated ground, drawn as a slowly turning ring of light, for `sigil, lasts` (4 s). Two at a
time; a third replaces the oldest.

- **Standing in her own sigil she is warded**: every wound she takes is cut by `sigil, wards
  her by` (a third). This is the "cast to defend" of the right click: she fights *from* her
  ground.
- **Standing in her sigil, they burn**: `sigil, burns` every half second, light.
- **A sigil is somewhere to go.** It is a place she put on the map, and Descent and Sanctuary
  are how she gets to one.

### The Purge — light on a curse

A light spell that lands on a cursed body **purges** it: every stack goes off at once.

| Stacks | 1 | 2 | 3 | 4 | 5 |
| --- | --- | --- | --- | --- | --- |
| Purge, 9 × *n*(1 + 0.3*n*), at power one | 11 | 28 | 51 | 79 | 112 |
| step | | +17 | +23 | +28 | +33 |
| from a full Light bar (× 1.4) | 16 | 40 | 71 | 110 | 157 |

Convex, like the Reaver's cash-in, so the fifth stack is worth the most: building to five and
then purging is the class's big number, and the decision is always "one more dark spell, or
cash now". At three stacks or more a purged fighter is also **staggered** (`purge, staggers
for`). It is multiplied by the power of the light spell that set it off — her Light bar — so a
mage who has fed only her dark hand cashes it for little. **That is the alternation, made into
damage**: the curse is built by one hand and cashed by the other, and both bars rise.

## Left — the litany of night (dark, on them)

The bare left click is a **litany**: three spells in a string, the next one thrown if she
presses again within `litany, window` of the last finishing. The Red Mage's melee combo in
FFXIV is the model: a mage's string in sword range, where each step is a spell.

1. **Umbral lash** — a whip of shadow from her left hand across her front, 3.5 m. Fast and
   light. One stack.
2. **Grasping shade** — hands of shadow come up out of the floor where she points, 5 m out,
   and close on whoever is standing there: a **root** for half a second (they keep their feet
   but cannot walk out) and one stack.
3. **Devour** — a maw of darkness opens where she points, 5 m, and snaps shut: the heavy one,
   **two stacks**, and some of what it deals (`leech` 15%) drained back to her.

**Shadowstep** (in the air: cast to move). She becomes shadow and is **behind the first body on
the crosshair's line** — 8 m for anybody, `shadowstep, reaches a cursed body` (14 m) for a
cursed one — and lashes on arrival: one stack. With nobody on the line she still goes, as far
as `shadowstep, without a body` (5 m) along it, stopped short of walls, and goes on falling as
she was. That step at nothing is **once a trip**, as Phase and the airdodge are — all three
spend the airdodge — and **a body found refunds it** and catches her fall: paid in aim
(0007's refuel rule), so a mage who keeps finding bodies chains Shadowsteps across a fight.
Against a creature it finds the part the crosshair's line reaches with a thin probe carried
along it (`Monster::part_struck`, the test every bolt is put to), so a sac on a spider's back
is found over the flank beneath it, and she arrives up against that part.

**Veil** (space + left: cast to defend). She leaves the floor as shadow: **untouchable** for
`veil, untouchable for` (18 frames — a dodge's worth) as she rises, and whoever is beside her
when she goes is cursed once. A jump out that cannot be hit on the way up.

## Right — the litany of dawn (light, on the ground)

1. **Radiant palm** — a burst of light from her right palm, point-blank, 3 m. Fast. Purges.
2. **Halo** — a ring of light that bursts out round her to 3.5 m: **all round**, the spell for
   being surrounded or closed on. Purges whoever it catches.
3. **Smite** — a column of light out of the sky onto where she points, 6 m. The telegraphed,
   heavy one: it lifts whoever it catches off their feet, purges, and **writes a sigil** there.

**Descent** (in the air: cast to move). She comes down as a pillar of light **onto the ground
the crosshair is on**, inside a cone round straight down (`aim::plunge_dir`, the Reaver's
Plunge cone): a dive she aims. Where she lands is a smite — damage round the landing, purging —
and **a sigil** she is now standing in. The air row's way back to the ground, and onto a place
of her choosing.

**Sanctuary** (space + right: cast to defend). A sigil written **at her feet**, bigger than the
others, and a pulse of light through it as it appears — damage, purging — while she hops up out
of it. She comes down warded.

## Middle — twilight, on herself

**Binary** (on foot: cast to fight). **Twin stars**, one dark and one light, wound round her at
arm's length for `binary, orbits for` (4 s). Each star strikes whatever it passes through — the
same body at most once every `binary, strikes a body every` (three quarters of a second) per
star, for `binary, damage a strike` (8): **the dark star curses, the light star purges**. So with the stars up, standing next to somebody
builds and cashes on its own, and *her own spells still do theirs*. One pair at a time; a second
Binary renews it. The class's Whirl: the thing that keeps working while she casts something
else, and why melee range is where she wants to be.

**Phase** (in the air: cast to move), as built: a blink along the crosshair, 6 m, stopped short
of walls, with a flash of light where she left. It spends the airdodge.

**Equinox** (space + middle: cast to defend — against herself). She **levels her bars**:
`equinox, levels by` (12) moves from the higher to the lower, and she jumps as high as the
lower bar is full, as before. The one spell aimed at her own runaway: it brings a burning gap
back toward the band without throwing anything at anybody, and it is a jump out at the same
time.

## The majors — spectacle that grows along the bar

The majors scale their **size** along a steeper curve than everything else
(`majors, size at empty` 0.4 → `majors, size at full` 1.6, against the depth curve's
0.6 → 1.4), so a major from an empty bar is a modest spell and one from a full bar is the
biggest thing on the screen. Range and frames still never scale: where it lands and how fast it
comes out are what two players read each other with.

- **Abyss** (`Q`, dark). A **black sun** where the crosshair meets the floor. For 1.5 s
  everything under it is **pinned** (rooted), takes a tick, and **gains a curse stack every
  half second** — three for anybody who stays the whole time. When it ends it **collapses**:
  one last blow of dark. It no longer drags anybody anywhere.
- **Judgement** (`E`, light). The delayed strike as before, and now **more columns the fuller
  her Light**: one from empty, up to five from full — the middle one where she pointed and the
  rest round it at `judgement, columns stand apart` (2.5 m, grown on the majors' curve with the
  columns themselves, so a full Judgement is five pillars rather than one blob). Every column
  purges. What is left
  burning is a **great sigil**: it wards her and burns them like any other, and it is the
  biggest one she can write.

**Abyss then Judgement** is the class's spectacle: a black sun pins a group and curses them,
and the columns come down through it and purge every one.

**Both majors are slow and telegraphed**, and that is their price: in a game with no
cooldowns, frames are what a move costs, and alternating the two keeps her level, so it costs
nothing else. The Abyss is 24 frames of startup and the Judgement 32 — both over the fifteen
this game treats as answerable on sight — with a ring on the floor where each will land for the
whole of it (dark for the sun, light for the columns), and each is locked out for its own
`repeat lockout` of 255%. The first pass had them at 14 and 22, and the two in turn on a dummy
were 6.7 health bars in half a round.

## Ascending at the right moment

The bars, the hill, the burn, the tiers and ascension are unchanged — and since every spell she
casts goads a bar, a mage casting freely near the top **will** ascend, wherever she happens to
be. So the arrival is now worth something, and that is what makes the timing a decision:

**Apotheosis.** The frame the wings erupt, a nova of both forces bursts from her
(`apotheosis, reaches` 5 m) at the top of the power curve: damage to everybody in it, and
**every curse in it is purged at full power**. Ascend beside a body carrying five stacks and the
arrival is the biggest blow in her kit. Ascend alone in a corner and it is wings and nothing
else.

That is the banner's last sentence in the mechanics: she fights, moves and defends by casting,
every cast climbs her bars, and the skill is knowing when to **stop** — hold at the brink, let
the calm take a little off, step in beside the cursed body — and then cast the one spell that
tips her over.

## The bars

| Input | Which bar | How far |
| --- | --- | --- |
| Umbral lash, Grasping shade | Dark | `meter_auto_push` (5) |
| Radiant palm, Halo | Light | 5 |
| Devour, Shadowstep, Veil | Dark | `meter_cast_push` (9) |
| Smite, Descent, Sanctuary | Light | 9 |
| Binary, Phase | Both | 5 each |
| Equinox | Both by 5, then levelled by 12 | |
| Abyss / Judgement | Dark / Light | `meter_finisher_push` (20) |

So a litany is two autos and a cast on one bar, and the other litany balances it: dark litany,
light litany, and she is where she started, higher. Curse ticks, sigil burns and the stars'
strikes are not casts and goad nothing.

## Combos

1. **Litany into purge.** Lash, Grasp, Devour — four stacks and a root — then Radiant palm: 86
   at power one, and a stagger to start the next. The bread and butter, and the climb.
2. **Stars up, then fight.** Binary, then anything: the dark star stacks and the light star
   cashes while her litanies do the same.
3. **Step in, step out.** Jump, Shadowstep behind a cursed body (from 14 m), litany of night,
   jump, Descent onto a sigil — or write one with it — and fight from it warded.
4. **The black sun and the columns.** Abyss on a group, Judgement through it.
5. **Hold at the brink.** Both bars in the nineties: Equinox to stay level, Veil out of
   trouble, and the Devour that tips her over next to five stacks: Apotheosis purges at the
   top of the curve.

## The numbers

Tuned against the class's benchmarks in [../dual-mage.md](../dual-mage.md) — the class stated
as player actions and their outcomes, pinned in `crates/sim/tests/dual_mage.rs` — with the
dummy brought into sword range (two metres) where it had stood at five. Half a round on a
target that never moves, healed every frame:

| Script | Dealt | Cost her | Benchmark |
| --- | --- | --- | --- |
| A litany of each hand, over and over | 1.85 bars | nothing | B1: 1 to 2¼ |
| The litany of night and the Abyss, no light | 2.07 bars | 0.57 bars | B2: 2 to 3½, and at least half a bar |
| The Abyss and Judgement in turn | 3.35 bars | nothing | B9 (new): 2 to 3½ |
| Judgement alone | 3.69 bars | 0.67 bars | — |

The moves: lash 10, Grasp 10, Devour 23; palm 12, Halo 12, Smite 26 (and a launch of 12);
Shadowstep's arrival 22; Descent's landing 34; the Veil's puff 8; Sanctuary's pulse 24; the
black sun 5 a tick and 24 as it collapses; Judgement 121 in the middle column. Every one of
them times her power, which is 0.6 from an empty bar to 1.4 from a full one.

## What goes

The Shade bolt, Reel, Nightfall, the Sunray, Flare, Dawn — and **the hex**: Umbra and Radiance
as one-at-a-time marks, the Shatter and the Wither, Binary's both-at-half. The Abyss's drag.
Their effects (`EffectKind::ShadeBolt`, `Flare`, `Sunray`) are retired with them.

## What the build touches

- **Her table grows from eleven rows to fifteen.** The eleven keep their slots and are re-cast
  as the new moves; the four new links are appended at the end of the Oven's move store, where
  the Dual mage already is.
- **The curse replaces the hex on a body**: `Player::hex`/`hex_left` and
  `Monster::hex`/`hex_left` become the stacks (with whose they are) and their clock — the same
  bytes, because the snapshot is at its 4 KiB cap to the byte.
- **Her mechanic carries the rest**: the litany's stage and window, the veil, whether she is
  warded this frame. `Mechanic::Meter` is smaller than the largest variant, so this costs the
  snapshot nothing.
- **New effects**: the Sigil, the twin stars, the Halo, the Grasp's hands, the Devour's maw,
  the Smite's column, Apotheosis; the Abyss pins rather than drags, and Judgement's strike
  comes in columns.
- **Untouchable** becomes one question (`Player::untouchable`): a dodge's opening frames, or a
  Veil.
- The hunting bot and the duel bot learn the litanies and the purge; the pins and the envelope
  fixtures are re-recorded; her tests are rewritten against the new kit.

## Questions for play

| ID | Question | Built as |
| --- | --- | --- |
| **DUAL-5** | Is a litany on a bare click the right home for three spells, or should each be its own button? | A string, the next link within `litany, window` of the last finishing — it advances on the press whether or not it landed |
| **DUAL-6** | Is the five-stack purge (112 at power one, 157 at full Light) the right size — big enough to chase, not a round-ender in versus? | Convex, the Reaver's shape; a stagger from three |
| **DUAL-7** | Does a third off in her own sigil make her too hard to kill on the spot? | A third, two sigils at most, four seconds each |
| **DUAL-8** | Should Shadowstep need a body at all? | No: 5 m along the line without one, and only a body refunds the airdodge |
| **DUAL-9** | Is Apotheosis enough reason to time the ascension, or should ascending be a button? | No button: the arrival bursts, so where she is when it happens is the decision |
| **DUAL-10** | Should the Equinox's levelling be stronger, or cost something? | 12 from the higher to the lower, no cost but the cast |
| **DUAL-11** | Is a step at nothing once a trip right, or should the Shadowstep need a body to go anywhere? | Once a trip; a body refunds it. Her lab envelope went from 21–25 m to 21–31 m |
| **DUAL-12** | Her new moves play her six old clips. Which need clips of their own first? | The litanies' one-armed casts on the old thrusts; the Halo, Sanctuary and Abyss on the sweep |
