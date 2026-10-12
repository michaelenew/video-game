---
status: proposed 2026-10-12, for the owner's review; nothing built
started: 2026-10-12
supersedes: 0009 where they differ, once decided
depends: ../kits/blood-mage.md, ../blood-mage.md, 0009_blood_mage_on_three_clicks.md
---

# 0015 — The Blood mage as the blaster

The owner's verdict on the three-click build (0009), after playing it: **it has not landed.** The
abilities are awkward and short, the pools are too small to use, the hook does not catch
cleanly, and the best plan is "spam Black spike and hope they don't attack you". Her movement in
and out of a fight is far behind every other class's. Black spike is the one spell that feels
right: a wind-up, decent damage, a big pool as the reward, and the pool grows.

The brief:

- **This is the blaster class.** It needs **range**, and every one of the nine click moves has to
  feel like an ability you would find in an MMO: **charges, heavy attacks, big areas, big damage,
  high commitment.** The models are the mage line in *Ragnarok Online* (RebirthRO), *Vladimir* in
  *League of Legends*, and the *Demon Avenger* in *MapleStory*.
- **She is not immobile.** She needs movement to buy time for her charges and to go and collect
  her bloodstains — **but she cannot do both at once.**
- **This is a platform-fighter-like game first.** The class needs core intended **combos that
  compose** into the effects of MMO spells — each MMO spell broken into separate abilities that
  stand on their own, and the combination more interesting and more impactful than the spell.

Method: as 0009 decided, **combos first, then the pieces.** §1 is what the replay says, §2 the
three references reduced to what is borrowed, §3 the four rules underneath everything, §4 the six
combos, and §5 the grid that falls out of them.

## 1 · What the replay says

`ffb325a2-blood-mage-fight.replay`, the last ten minutes of one tape (the Saddle and the Shrine,
against the Mantis), counted per class:

| Move | Thrown | Landed |
| --- | --- | --- |
| Black spike | 28 | 0 |
| Reaping sweep | 23 | 6 |
| Haemorrhage | 17 | 7 |
| Blood nova | 16 | 0 |
| Hook | 12 | — (no hit to count; but no follow-up either) |
| Bloodletter | 5 | 0 |
| Grasp | 3 | 0 |
| Nail | 2 | 1 |
| Blood jet, Harvest | 1 each | 0 |
| Marionette | 0 | — |

Caveat: the counter credits a hit to whichever move is in its active frames when the hit lands,
so a hit from something already out in the world, like the spike's eruption, may be under-counted.
Even so, the shape is plain. **Two moves did the fighting** — the spike and the Haemorrhage, the
two with range — and the sweep was the panic button. Six of the nine click moves were tried once
or never. The takeoff row went unused: the class's air game never started.

Measured on the build (`cargo run -p sim --bin frametable`, `--bin essence`):

- **Range.** Spike and Haemorrhage 9 m, Bloodletter 7 m, sweep 2.4 m. The Grasp reaches 12 m,
  but only after half a second of standing still.
- **Pools.** A pool is a **column 0.18–0.50 m wide** — narrower than a body. It drains in one to
  nine seconds. Something that small is hard to see from 9 m and harder to put a spell through.
- **Movement.** A plain jump (4.0 m, the same as the Champion's; the Reaver and the Elementalist
  get 5, the Dual mage 6). A blink that only goes to a pool of hers — which, see above, are
  hard to aim at. A Hook that only works in the air and catches only what a thin line meets. A
  Blood jet you have to hold.

(Side finding: `--bin essence` measures the wrong moves since the three-click re-keying — it
reports the Bloodletter dealing 160 and the Haemorrhage's bolt 0. Worth fixing on its own.)

## 2 · What each reference contributes

Only what is borrowed, and why it fits a platform fighter.

**Ragnarok Online's mage line — the cast bar.** Wizards and Warlocks are defined by long casts that
turn into huge areas: *Storm Gust*, *Lord of Vermilion*, *Meteor Storm*, *Heaven's Drive*, *Jupiter
Thunder*'s knockback. The skill is **finding the time and the place to cast**, while your partner,
your positioning or a *Safety Wall* buys it for you. A Warlock can also preload spells into a book
and release them later (*Reading Spellbook* / *Release*). Borrowed: **charge tiers** — the longer
you hold, the bigger the spell — and **a stored charge**.

**Vladimir — health is the resource, and the pool is the escape.** *Transfusion*: a targeted bolt,
stronger every third cast. *Tides of Blood*: hold to charge, pay health as you hold, release a nova
around you. *Sanguine Pool*: melt into a pool on the floor, untargetable, move and slow what you
pass through. *Hemoplague*: curse an area, amplify damage, then everything marked bursts. Borrowed:
**paying health to charge**, **the pool as movement**, **mark-then-detonate**.

**Demon Avenger — orbs you go and pick up.** No mana: health is the resource. Attacks drop blood
orbs that she **runs through to collect**, which is what turns her movement into her sustain.
*Demon Frenzy* is a toggle: lose health each second, hit harder. Borrowed: **collection by
movement**, and **a frenzy that trades health for doing two things at once.**

**Platform fighters — the stored charge.** Samus's charge shot in *Smash*: charge, cancel the
charge with a shield or a roll and it is **kept**, then fire it later from anywhere. That one rule
is what turns "a cast bar" into "a cast bar you can move between the two halves of", which is the
exact shape the brief asks for: she can charge, and she can move, and not both at once — but she
can do one, then the other, then fire.

## 3 · Four rules underneath everything

These are what the moves in §5 are built from. Each is one sentence the opponent can learn.

### R1 · Charge on the floor, fire from anywhere

**Every click on foot is held to charge, through three tiers.** A tap fires tier 1 at once. Holding
reaches tier 2 after half a second and tier 3 after one more half-second. **Each tier is paid in red
health as she holds**, the way the Blood nova is paid now, so a tier-3 spell is a real cut.

While charging she **walks at a crawl** and cannot jump, dodge or attack. **Jumping or dodging out
of a charge stores it** in her **vessel**: one slot, holding one spell at the tier it reached.
Pressing that click again releases it — on the floor, in the air, at the end of a dash — as the
on-foot spell at the stored tier. Being hit while charging **loses** the charge. A charge already
stored is safe.

The vessel is visible — her off hand glows the tier's colour — and it **bleeds**: about 1% of red
a second while it holds a spell, into grey. Carrying a stored tier-3 spell is a threat both
players can see, with a clock on it.

> Why this and not the plain RO cast: a fixed cast bar in a game with dodges and jumps is a stand
> -still that loses to anyone who walks at you. That is "spam spike and hope". A charge that can be
> banked on the dodge means the charge and the movement **take turns** instead of excluding each
> other, and the combos in §4 are the orders you put them in.

### R2 · Stains are big, last, and are collected by moving

"Pool" becomes **stain** — the name question 0009 left open, answered by the owner's own word.

- **Size.** A stain is a **disc on the floor** again, not a column: radius `0.25 m × √essence`,
  never under **1.0 m** and never over **4.0 m**. A tier-1 spike's stain (100) has a 2.5 m
  radius; a sweep's (22) is 1.2 m; a tier-3 field's is the full 4 m.
  > History: on 2026-09-23 a disc of 0.24 m per root was judged "too big, and it spread" and became
  > the column. That was when she was a melee class fighting inside 2.4 m. **At 15–20 m of range a
  > stain has to be readable and reachable from across the arena**, and the floor of 1 m is what
  > makes a sweep's stain worth going to.
- **Life.** Drain drops from 10 a second to **6**, so a tier-1 spike's stain lasts about sixteen
  seconds instead of nine.
- **Collected by moving.** The owner's "collect her bloodstains" is the Demon Avenger's orbs.
  **Her blood-form dodge (Sanguine flow) and her landing leap absorb every stain they cross**, a
  share of each turning grey back to red. Walking over a stain does *not* — the collecting is a
  move, so it is a decision, and it is a move she cannot make while charging.
- **Still fuel.** A spell landed **on** a stain still erupts it (the Black spike as now, chaining)
  — so every stain is still a choice: **drink it, or blow it up.** That replaces "a heal or a door"
  — because now going *to* it is how she drinks it — and the reason to still have a choice is the
  same.
- Her own blood still never heals her. Grey still fades; grey is still the ceiling on every heal.

### R3 · Grey speeds the charge

Grey still lengthens the scythe. It now also **speeds the charge**: up to ×1.5 faster at a full bar
of grey. A wounded Blood mage casts faster — Vladimir's and the Demon Avenger's "lower health,
higher threat", made into the thing a blaster most wants. Drinking a stain slows her back down.
That is the edge to ride, unchanged in spirit: *how much of my bar am I willing to leave open.*

### R4 · Frenzy breaks the rule, for a price

`Q` is **Frenzy** (Demon Avenger's *Demon Frenzy*), a toggle. While it is on:

- she pays about **2% of red a second**, into grey (which speeds her charges, R3);
- **stains within 5 m drift toward her** and are absorbed when they reach her — so she **can**
  charge and collect at once.

It is the brief's "can't do both" with a price on breaking it, and the price makes her more
dangerous and more killable at the same time. Off by default; it ends on its own at one health.

## 4 · The six combos, and the MMO spell each one is

Each is a short chain of moves from §5, each move useful on its own. The names in the headings are
the spell the chain adds up to.

### C1 · *Meteor Storm* — Upheaval → Stake → Black spike

**Upheaval** (space + middle) erupts a geyser under the target from range and launches them; she
takes a small hop with the cast. At the top of their arc she throws the **Stake** (middle, in the
air) down through them: it **pins** them in the air and drives into the floor beneath, leaving a
stain right under the pinned body. She lands and taps **Black spike** on that stain: the eruption
comes up 3.4 m tall into a body that cannot move.

Three spells, three different jobs — launch, hold, hit — and the stain from the second is the
place for the third. 0009's *Hanging* combo, rebuilt so that **every piece reaches 15 m or more**
instead of needing Marionette at sword range.

### C2 · *Tides of Blood + Sanguine Pool* — charge, flow, release

Hold right click: **Tides** charges. Someone runs at her to stop it. She **dodges**: the charge goes
into the vessel and she becomes **Sanguine flow**, sliding through them (slowing them) and out the
other side, drinking the stains on the way. She surfaces behind them and **releases the stored
Tides** — a tier-3 ring of scythes from where she now stands.

This is Vladimir's signature play, and it is the brief in one combo: charge, move, fire, never two
at once.

### C3 · *Storm Gust* — Haemorrhage → Black spike on the trail

The best-landing move in the replay, kept. **Haemorrhage** from the air (where she hangs briefly
while casting) opens a bleed; a bleeding target walks a trail of stains. Now that stains are
1 m-plus discs, the trail is a **continuous red path**. A **Black spike** on its oldest end erupts
and chains along it, toward wherever they have got to. A tier-2 spike (three spikes in a triangle)
covers enough to catch the chain even if they turn.

A damage-over-time area that follows its victim, made of a mark and a detonation.

### C4 · *Get over here* — Reaper's chain → Tides

**Reaper's chain** (right, in the air) thrown at a fighter yanks them to her. She lands on her own
stain (a Blood wings takeoff leaves one) and a **tapped Tides** goes off with them inside it. Thrown
at a wall, a ledge or the floor instead, the chain pulls **her** — the same move as the escape.

The Grasp's job (haul them to my stain) with the Grasp's half-second of standing still removed.

### C5 · *Hemoplague* — mark, flood, burst, harvest

**Hemoplague** (`E`): a slow curse ball to 20 m that marks everyone in a 4 m area. For four seconds
the marked take more from her; then each **bursts**, leaving a big stain under every marked body.
Into the marked group (or the marked creature) goes a **tier-3 Black spike** or a stored Tides. Then
**Reaper's leap** into the middle of the stains: the landing slam absorbs every stain within 3 m.

The MMO "ultimate rotation" — amplify, nuke, heal off the wreckage — as four buttons and a jump.

### C6 · *Spellbook* — charge behind cover, wings, release

Out of sight behind a wall or a stone: hold left click to tier 3 (**Transfusion** becomes a
piercing lance). **Blood wings** (space + left) — the charge goes into the vessel as she jumps — and
she bursts over the wall. In the air she releases the lance at somebody who thought they were safe.
The stain the wings left at her takeoff is her way back: one **Sanguine flow** to it, out of reach.

RO's preload-and-release, and the counterplay is visible: the glowing hand, and the bleed it costs.

**In a hunt** the same chains read: C1's launch and pin are creatures' weak points held in reach;
C3 marks a creature and chains along its trail (`Monster::bleeding` already exists); C5's burst puts
a 4 m stain under a toppled creature, and the leap into it is a heal and a way onto its back.

## 5 · The grid

The columns keep 0009's meaning: **left is my blood**, **middle is your blood**, **right is the
scythe**. Every row is a different situation: on the floor she charges, in the air she fires
quickly, and space + a click is a takeoff.

| | Left: my blood | Middle: your blood | Right: the scythe |
| --- | --- | --- | --- |
| **On foot** (hold: 3 tiers) | **Transfusion** | **Black spike** | **Reaping sweep / Tides** |
| **In the air** | **Haemorrhage** | **Stake** | **Reaper's chain** |
| **Leaving the floor** (space + click) | **Blood wings** | **Upheaval** | **Reaper's leap** |

| Key | Move |
| --- | --- |
| `Q` | **Frenzy** (toggle) |
| `E` | **Hemoplague** |
| `shift` + direction | **Sanguine flow** — with the crosshair on a stain, to the stain |

Numbers are first guesses for the Oven, there so the shape is unambiguous. A full bar is 1000; a
second is 60 frames. "Tier" times are from the press.

### On foot — the charges

**Transfusion** (left, held). *Vladimir's Q, RO's bolts and Jupiter Thunder.* A bolt along the
crosshair, **20 m**.
- **T1 (tap):** fast bolt, 60 damage, a stain under the hit.
- **T2 (0.5 s):** a heavy bolt, 110, **pierces** — everyone on the line is hit and spilled.
- **T3 (1.0 s):** the **lance**, 200 and a long knockback (8 m); the body slides and leaves a stain
  **streak** along the slide.
- Costs 1 / 3 / 6% of red. Recovery grows by tier: T3 is punishable on a whiff by everybody.

**Black spike** (middle, held). *The template; RO's Heaven's Drive and Lord of Vermilion.* Grounded,
**22 m** (up from 9).
- **T1 (tap):** today's spike — 30 frames of telegraph, 100, launch, slow, a 2.5 m stain. On a
  stain: the eruption and the chain, as built.
- **T2:** three spikes in a 3 m triangle, staggered 6 frames apart, 80 each.
- **T3:** a **field**: 6 m across, three waves of spikes rising half a second apart, 80 a wave,
  a 4 m stain at the centre. On a stain, every wave erupts it again until it is gone.
- Costs 6 / 9 / 14%. The answer to "spam spike": T1 alone is a poke; the damage lives in T2 and T3,
  which are a second and more of being hittable.

**Reaping sweep / Tides** (right). *Vladimir's Tides of Blood.* **Tap** is the sweep, as built —
the close-range "get off me", growing with grey. **Hold** is **Tides**: the scythe whirled overhead
while charging, released as a ring of blood around her.
- **T1:** 4 m ring, 50, pushes back.
- **T2:** 6 m ring, 90.
- **T3:** 8 m ring, 140, and a ring of stains where it reached — she is standing in the middle of
  her own buffet.
- Costs 2 / 5 / 9%. **Blood nova is folded into this** and its left-click slot freed.

### In the air — quick and committed

Every aerial gives her a short **hang** (fall speed capped through the startup), so from the top of
a jump she casts down onto the floor — the blaster's high ground. Aerials do not charge; a stored
charge in the vessel fires instead when its own click is pressed.

**Haemorrhage** (left, air). As built, at **18 m** and with a 0.9 m bolt. The bleed's ticks spill
1 m discs, so the trail reads as a path.

**Stake** (middle, air). *RO's Meteor Storm, one meteor.* A long black spike thrown down along the
crosshair, **18 m**. Through somebody **in the air** it **pins** them (the Nail's pin) and lands in
the floor beneath; on the floor it hits a **2.5 m area** where it lands — 90, a stain. Replaces the
Nail, which was a thin line that only did anything to a body already in the air.

**Reaper's chain** (right, air). *The Hook, fixed.* The scythe thrown on a chain to **exactly where
the crosshair is** — the skillshot's end point, 18 m: the floor, a wall, a ledge, a stone, a body.
- **On a surface:** pulls her there, fast, and she arrives with her momentum (a pull, not a
  placement), so she can chain into a jump.
- **On a fighter:** yanks **them** to her (the Grasp's haul).
- **On a creature:** pulls her to it.
- **On nothing** (the sky): still pulls her 5 m along the aim. **It never whiffs into nothing.**
  The chain is drawn out and back on every throw.

Why the Hook "does not register cleanly": it only tests a thin line through bodies and boxes, and
looking at open floor — the commonest thing to aim at in the air — catches nothing and does nothing.
Aiming at the crosshair's own point fixes both.

### Leaving the floor — the takeoffs

**Blood wings** (space + left). *The Demon Avenger's wings; the Blood jet, fixed.* A **tap** bursts
her 8 m along the aim (any direction, including up), and leaves a stain of her own blood where she
took off — a door back for Sanguine flow. No hold: movement should not charge. 2% of red.

**Upheaval** (space + middle). *Heaven's Drive, one column.* A grounded geyser at the crosshair,
**15 m**, 16 frames: 70, a big launch, and a stain. **On a stain** the geyser is wider and launches
higher. She hops a body's height as she casts, so she is already in the air for the Stake (C1).
Replaces Marionette, which launched at sword range and was never thrown.

**Reaper's leap** (space + right). *A demon's slam.* A leap to a grounded point up to **10 m** away
along the crosshair, the scythe brought down on landing: 80 in 2.5 m, and **every stain within 3 m
is absorbed**. The heal-and-reposition move, and the end of C5. Replaces Harvest, whose spin
passed over everything.

### Keys

**Sanguine flow** (`shift`). *Vladimir's Sanguine Pool.* Her dodge: she melts into a stain and
**flows 6 m along the floor** (twice a dodge's distance), invulnerable while flowing, slowing any
body she passes under, absorbing every stain she crosses, and stopping short of the first thing her
feet meet (`aim::blink_to`). **With the crosshair on a stain within 20 m** she flows **to** it —
the blink, now a fast slide along the floor rather than a jump cut, so it can be seen and
punished at the end. In the air it is the air dodge, and it lands her.

**Frenzy** (`Q`). R4.

**Hemoplague** (`E`). *Vladimir's ultimate.* A slow ball (24 frames, 20 m, a 4 m area): marks
everyone it covers for 4 s, during which they take ×1.25 from her; at the end each takes 120 and
leaves a 4 m stain. 15% of red. It is the slowest, most telegraphed thing she has, and the biggest
reward.

## 6 · What goes, what stays

| Built (0009) | Becomes |
| --- | --- |
| Blood nova (left, held) | Folded into **Tides** (right, held) |
| Grasp (middle, held) | Its haul goes to **Reaper's chain** on a fighter; cut |
| Reaping sweep (right) | Kept as the tap of right click |
| Haemorrhage (left, air) | Kept, longer range, bigger trail |
| Nail (middle, air) | **Stake**: area on the floor, pin kept |
| Hook (right, air) | **Reaper's chain**: to the crosshair's point, never empty |
| Blood jet (space + left, held) | **Blood wings**: tapped, 8 m |
| Marionette (space + middle) | **Upheaval**: a ranged launch |
| Harvest (space + right) | **Reaper's leap**: lands and collects |
| Bloodletter (`Q`) | Cut (it was in no combo); `Q` is **Frenzy** |
| Black spike (`E`) | Moves to middle click, held, three tiers; `E` is **Hemoplague** |
| Blink (`shift` at a pool) | **Sanguine flow**: a dodge everywhere, to a stain when aimed at one |

**Kept unchanged:** grey health and its fade; ×1.4 against the disabled; a cut never kills her;
her own blood never heals her; the eruption and its chain; the bleed; the scythe drawn at the size
it hits at.

## 7 · Movement, compared

| | Before | After |
| --- | --- | --- |
| On the floor | A dodge; a blink only to her own narrow pools | Sanguine flow, 6 m anywhere, 20 m to any stain, collects on the way |
| Up | A plain 4 m jump; a held jet | Blood wings, 8 m any direction, tapped |
| Across a gap / onto a ledge | The Hook, air only, only if a thin line meets something | Reaper's chain to wherever the crosshair is, 18 m, never empty |
| Into a fight | — | Reaper's leap, 10 m, lands with a slam |

Four ways to move, every one of which **ends a charge** (or banks it). That is the brief's
trade-off written into the inputs: she is the most mobile she has ever been, and every metre of it
is a moment she is not charging.

## 8 · What it costs to build

Nothing in the aiming model changes; every line of effect is one of the five in `aim.rs`:
Transfusion, Haemorrhage, Stake, Reaper's chain, Blood wings and Hemoplague are **skillshots**;
Black spike, Upheaval and Reaper's leap are **grounded**; the sweep and Tides are **swings**;
Sanguine flow asks `aim::pointing_at_disc` and `aim::blink_to`, as the blink and the Dual mage's
blink already do.

- **Charge tiers and the vessel.** The Blood nova's held-and-paid channel, generalised to three
  moves with tier thresholds; `Player` gains the vessel (a move and a tier: two bytes; the 4 KiB
  cap has room). Tier thresholds and costs are Oven knobs.
- **Stains as discs.** The pool effect's radius changes from a column to `0.25 √essence` clamped
  to [1, 4] m; the drink-by-moving check is a disc-against-path test in `step_effects`.
- **Frenzy.** A flag, a drain, and a drift in `step_effects`.
- **Hemoplague.** A mark with a clock on fighters and creatures (`Monster::bleeding` is the
  precedent).
- **New clips:** the Tides whirl, the lance, the Stake's throw, the wings, the leap's slam, the
  flow (a body melting into the floor and re-forming). The spike's, the sweep's and the
  Haemorrhage's are built.
- **The move table** grows by appending (Tides, Transfusion, Stake, chain, wings, Upheaval, leap,
  Hemoplague, Frenzy), never renumbering, so no knob moves.

**Tests to add or change.** `feel.rs`: *every tier-3 charge is punishable on a whiff by every
class*; *a charge stored and released does what the same charge released at once does*; *no
collection move can be performed while charging, except under Frenzy*. `blood_mage_clicks.rs`: one
test per new move and C1–C6 end to end. Retire the Grasp, Marionette, Harvest, Bloodletter and Nail
tests with the moves.

## 9 · Questions for the owner

- **BLAST-1 · The vessel.** One slot, filled by dodging or jumping out of a charge (the Samus
  rule), bleeding about 1% a second. Or none: a charge simply lost on a dodge, which is purer RO and
  harsher.
- **BLAST-2 · Collecting.** Collected only by moves (flow, leap, Frenzy's drift), not by walking.
  Walking would be friendlier and remove the "can't do both" from collecting.
- **BLAST-3 · Stain size.** 1–4 m, a disc. Bigger is easier to use and noisier on the floor; the
  cap of eight stays.
- **BLAST-4 · Black spike off `E`.** It becomes the middle click, held. Or it stays on `E` too, as a
  fixed tier-1 spike for players who want it without the hold.
- **BLAST-5 · Grasp cut.** Its haul lives on in the chain. If the held, aimed-by-time Grasp is worth
  keeping as its own thing, it is the candidate for `Q` instead of Frenzy.
- **BLAST-6 · Ranges.** 18–22 m against today's 9. These are guesses at "the blaster's range"; the
  first play will say whether the arenas are wide enough for them to matter.
