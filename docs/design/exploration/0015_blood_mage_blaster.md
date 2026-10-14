---
status: built 2026-10-12 along §0 and §6, unplayed; what is in the game is ../kits/blood-mage.md §"The blaster"
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
three references reduced to what is borrowed, §3 the five rules underneath everything, §4 the two keys (Frenzy and the Hemoplague), §5 the seven
combos, and §6 the grid that falls out of them.

## 0 · The owner's answers, 2026-10-12

Folded into the sections below; recorded here so the revision can be read against the first draft.

- **The stored charge stays** (BLAST-1).
- **Everything collects.** Walking, the dodge-teleport and every attack absorb stains for health,
  and an attack that absorbs one also hits harder. Getting health back is far too hard today and
  this is to be **too strong first**, tuned down later (BLAST-2).
- **Stain size is not the problem; small damage is.** Keep how a stain's size follows its essence.
  Give her more moves that hit hard enough to leave a stain you *plan around*: a small stain is a
  chore, a big one is an objective (BLAST-3).
- **Hemoplague** is the **big slow cloud over an area**: high damage, but over time, so the play is
  finding a way to keep them inside it.
- **Frenzy** is **"I stop moving, and everything else bends around me."** Stains rush to her; an
  area round her hurts, swallows stains and **explodes** them. It cannot last: no healing from
  stains, grey drains fast, and every point of damage she has taken or takes before it ends is
  **permanent**. To keep it going she **teleports along the trails** her bleeding enemies leave.
- **Every attack deals damage, and every hit on a creature leaves a stain.** No exceptions.

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
- **Why hitting a creature left nothing to drink.** A probe over the same replay found every one
  of her 80 stains placed correctly on the floor — none buried, none lost — but **small**: the
  median stain held **8 essence** (a bleed tick) and 44 of the 80 held under 20, which at
  10 a second is gone in under two seconds. 68 of the 80 were born on a frame she hit the
  creature. So the creature *does* bleed; it bleeds too little, too briefly to be seen. The
  fix is damage per hit and the drain, not where the stain goes.
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

## 3 · Five rules underneath everything

These are what the moves in §6 are built from. Each is one sentence the opponent can learn.

### R1 · Charge on the floor, fire from anywhere

**Every click on foot is held to charge, through three tiers.** A tap fires tier 1 at once. Holding
reaches tier 2 after half a second and tier 3 after one more half-second. **Each tier is paid in red
health as she holds**, the way the Blood nova is paid now, so a tier-3 spell is a real cut.

While charging she **walks at a crawl** and cannot jump, dodge or attack. **Jumping or dodging out
of a charge stores it** in her **vessel**: one slot, holding one spell at the tier it reached.
Pressing that click again releases it — on the floor, in the air, at the end of a teleport — as the
on-foot spell at the stored tier. Being hit while charging **loses** the charge. A charge already
stored is safe.

The vessel is visible — her off hand glows the tier's colour — and it **bleeds**: about 1% of red
a second while it holds a spell, into grey. Carrying a stored tier-3 spell is a threat both
players can see, with a clock on it.

> Why this and not the plain RO cast: a fixed cast bar in a game with dodges and jumps is a stand
> -still that loses to anyone who walks at you. That is "spam spike and hope". A charge that can be
> banked on the dodge means the charge and the movement **take turns** instead of excluding each
> other, and the combos in §5 are the orders you put them in.

### R2 · Every hit bleeds, and hits big enough to matter

"Pool" becomes **stain**, the owner's word. The size rule is kept as built — a stain's size follows
its essence, and its essence is the damage that made it. What changes is how much damage there is.

- **Every move deals damage**, the movement included: the chain on its catch, the wings on their
  burst, the teleport on arrival, Frenzy's area, the Hemoplague's cloud. Nothing in the grid is
  "movement, no hitbox" any more.
- **Every hit on a creature leaves a stain**, sized by the **move's own damage** rather than by
  what the creature's armour let through. A hit on a hard plate still opens it.
- **Fewer, bigger stains.** Every on-foot move's tier 1 deals 60 or more, so its stain is an
  objective. The bleed ticks less often and harder (25 every half second instead of 8 every
  fifth), so a trail is a line of stains worth teleporting along, not a line of specks. Ticks on
  the same body merge into its newest stain while it stands still.
- **Drain halves** to 5 a second. A tier-1 spike's stain lasts twenty seconds; a tick's, five.

### R3 · Everything collects

The Demon Avenger's orbs, made generous. A stain is absorbed — essence turned from grey back to
red — by **any of three things**:

| How | Heals | And |
| --- | --- | --- |
| **Walking** into it | 100% of its essence | — |
| **The dodge-teleport** to it | 100% | the arrival is a burst (R5) |
| **An attack** that lands over it, or whose volume passes over it | 100% | **the attack deals extra damage: +50% of the essence**, on top of its own |

The Black spike on a stain still **erupts** it (and chains) instead — the one way to spend a stain
on damage rather than health, and the biggest hit she has. So every stain is still a decision:
**walk it, hit through it, or blow it up.**

**The ceiling stays, the clock slows.** A heal still only turns grey back to red — that is what
keeps her from healing past where she stood a moment ago, and it is what Frenzy's price is made of
(R4). But grey fades at **3 a second instead of 12**, so most of what she lost in an exchange is
still there to win back when the exchange ends. These numbers are deliberately too strong; they
are where tuning starts, not where it ends.

Her own blood still never heals her (the wings' and the vessel's), so casting cannot refund itself.

### R4 · Grey speeds the charge

Grey still lengthens the scythe. It now also **speeds the charge**: up to ×1.5 faster at a full bar
of grey. A wounded Blood mage casts faster — Vladimir's and the Demon Avenger's "lower health,
higher threat", made into the thing a blaster most wants. Drinking slows her back down. The edge
to ride, unchanged in spirit: *how much of my bar am I willing to leave open.*

### R5 · The dodge is a teleport when it has somewhere to go

`shift` with the crosshair on a stain within **20 m**: she is **there**, instantly — the built
blink, at twice the reach and against stains that are now big enough to aim at. She absorbs it
(R3) and arrives in a **burst**: 40 to everyone within 2 m, spilling them. With no stain under the
crosshair it is **Sanguine flow**: she melts into blood and slides 6 m along the floor,
invulnerable, absorbing any stain she crosses.

## 4 · Frenzy and the Hemoplague

The two keys, defined in full because the first draft did not.

### Frenzy — `Q` · *I stop moving; everything bends around me*

*The Demon Avenger's Demon Frenzy and Vladimir's Hemoplague-into-pool all-in, as one window.*

Press `Q` (needs at least 150 grey: she has to be wounded to frenzy). For **three seconds**:

- **She is planted.** She cannot walk, jump or charge. She can attack, and she can teleport.
- **The area.** Everything within **6 m** of her takes 20 every sixth of a second (120 a second)
  and **starts bleeding**, so whoever runs leaves a trail.
- **The pull.** Every stain of hers within **15 m** slides toward her **fast** (12 m/s). A stain
  that reaches the edge of the area **explodes**: its essence ×2 as damage in a 2.5 m burst where it
  is, spilling whoever it hits (which makes new stains, which are pulled in, which explode).
- **The price.**
  - **No stain heals her** while it lasts. Walking, teleporting and attacking through stains turn
    them into explosions instead.
  - **Grey drains at 60 a second** — five times the old fade, twenty times the new one.
  - **Everything is permanent.** The moment Frenzy starts, all her grey is locked out of reach,
    and every hit she takes until it ends goes straight to gone. What she lost to get here, she
    will not get back.
- **Keeping it going: the trail.** A teleport to a stain **extends Frenzy by one second** and
  brings the area with her; she arrives already exploding. The stains her area's bleed lays behind
  fleeing enemies are the road. Up to eight seconds in all. When it ends she is free to heal again
  — from whatever grey she has left, which is very little.

Frenzy is a commitment of the whole bar: three to eight seconds where standing near her is ruinous,
paid for in health she can never recover. Its counterplay is the same as its power: **don't
bleed**, or, bleeding, don't run in a straight line.

### Hemoplague — `E` · *the big slow cloud*

*Vladimir's Hemoplague, RO's Storm Gust and Quagmire.* A grounded cast at the crosshair, **22 m**,
with a long telegraph (40 frames — the ground darkens where it will be). Then a **cloud 5 m across
and 4 m tall** for **five seconds**:

- **Damage over time.** 30 every half second to everyone inside — 300 for staying the whole time,
  the biggest number in the kit, and none of it up front.
- **It drinks into itself.** Every hit inside the cloud, hers or the cloud's, spills into **one
  stain at the cloud's centre** instead of under the body. When the cloud ends, that stain is
  usually the largest thing on the floor: a 300-essence objective in the middle of where the fight
  was.
- **It does nothing to keep them there.** No slow, no root. That is the whole skill of the move:
  the rest of the kit has to hold them in it (C5).
- 15% of red. Long recovery. One at a time.

## 5 · The combos, and the MMO spell each one is

Each is a short chain of moves from §6, each move useful on its own. The names in the headings are
the spell the chain adds up to.

### C1 · *Meteor Storm* — Upheaval → Stake → Black spike

**Upheaval** (space + middle) erupts a geyser under the target from range and launches them; she
takes a small hop with the cast. At the top of their arc she throws the **Stake** (middle, in the
air) down through them: it **pins** them in the air and drives into the floor beneath, leaving a
stain right under the pinned body. She lands and taps **Black spike** on that stain: the eruption
comes up 3.4 m tall into a body that cannot move.

Three spells, three jobs — launch, hold, hit — and the stain from the second is the place for the
third. 0009's *Hanging*, rebuilt so that every piece reaches 15 m or more.

### C2 · *Tides of Blood + Sanguine Pool* — charge, flow, release

Hold right click: **Tides** charges. Someone runs at her to stop it. She **dodges**: the charge goes
into the vessel and she becomes **Sanguine flow**, sliding through them and out the other side,
drinking the stains on the way. She surfaces behind them and **releases the stored Tides** — a
tier-3 ring of scythes from where she now stands.

Vladimir's signature play, and the brief in one combo: charge, move, fire, never two at once.

### C3 · *Storm Gust* — Haemorrhage → Black spike on the trail

**Haemorrhage** from the air opens a bleed; a bleeding target walks a trail of stains. A **Black
spike** on its oldest end erupts and chains along it, toward wherever they have got to.

### C4 · *Get over here* — Reaper's chain → Tides

**Reaper's chain** (right, in the air) thrown at a fighter yanks them to her. She lands on a stain
— absorbing it — and a **tapped Tides** goes off with them inside it, hitting through the stain
under her for the extra damage. Thrown at a wall, a ledge or the floor instead, the chain pulls
**her** — the same move as the escape.

### C5 · *Hemoplague* — the cloud, and four ways to keep them in it

**Hemoplague** on them. Then hold them in it, any of:
- **Stake** pins anyone she has knocked into the air above it;
- **Reaper's chain** yanks a runner back to her while she stands in the cloud's middle;
- **Black spike**'s slow, or its tier-3 field over the same ground;
- **Tides** from inside the cloud, which pushes outward — so she stands at its edge and pushes
  *in*.

When it ends, the stain at the centre is the reward: walk into it for the heal, or put a spike in
it for the biggest eruption the game has.

### C6 · *Spellbook* — charge behind cover, wings, release

Out of sight behind a wall or a stone: hold left click to tier 3 (**Transfusion** becomes a
piercing lance). **Blood wings** (space + left) — the charge goes into the vessel as she jumps — and
she bursts over the wall. In the air she releases the lance at somebody who thought they were safe.
The stain the wings left at her takeoff is her way back: one teleport to it, out of reach.

### C7 · *The red hunt* — Haemorrhage, Frenzy, the trail

Haemorrhage the one who will run. Hemoplague or a tier-3 spike on the rest, so the floor is wet.
**Frenzy**: the stains rush in and explode; everyone near bleeds; the runner's trail stretches
away. Teleport down the trail, stain to stain, each one a second more and an explosion where she
lands, until she catches them or the frenzy burns out and leaves her with nothing to heal from.

**In a hunt** the same chains read: C1's launch and pin hold a creature's weak point in reach; C3
marks a creature and chains along its trail (`Monster::bleeding` already exists); a creature that
cannot leave a Hemoplague is the best target it has; and every hit on a creature now leaves a stain
worth walking to.

## 6 · The grid

The columns keep 0009's meaning: **left is my blood**, **middle is your blood**, **right is the
scythe**. On the floor she charges, in the air she fires quickly, and space + a click is a
takeoff. Every move deals damage; the **stain** column is what a clean hit leaves.

| | Left: my blood | Middle: your blood | Right: the scythe |
| --- | --- | --- | --- |
| **On foot** (hold: 3 tiers) | **Transfusion** | **Black spike** | **Reaping sweep / Tides** |
| **In the air** | **Haemorrhage** | **Stake** | **Reaper's chain** |
| **Leaving the floor** (space + click) | **Blood wings** | **Upheaval** | **Reaper's leap** |

| Key | Move |
| --- | --- |
| `Q` | **Frenzy** (§4) |
| `E` | **Hemoplague** (§4) |
| `shift` + direction | The **teleport** to a stain under the crosshair, else **Sanguine flow** (R5) |

Numbers are first guesses for the Oven, there so the shape is unambiguous. A full bar is 1000; a
second is 60 frames.

| Move | Reach | Damage | Stain | Cost (% of red) |
| --- | --- | --- | --- | --- |
| Transfusion T1 / T2 / T3 | 20 m | 60 / 110 pierce / 200 + knockback | 60 / 110 each / 200 streak | 1 / 3 / 6 |
| Black spike T1 / T2 / T3 | 22 m | 100 / 3 × 80 / 3 waves × 80 | 100 / 80 each / 240 | 6 / 9 / 14 |
| Reaping sweep (tap) | 2.4 m, grows with grey | 45 | 45 | 1 |
| Tides T1 / T2 / T3 | 4 / 6 / 8 m ring | 60 / 100 / 150 | each body hit | 2 / 5 / 9 |
| Haemorrhage | 18 m | 40, then 25 every 0.5 s for 4 s | 40, then 25 a tick | 4 |
| Stake | 18 m | 90 in 2.5 m; pins in the air | 90 | 4 |
| Reaper's chain | 18 m | 40 on a body | 40 | 2 |
| Blood wings | 8 m burst | 40 in 2 m where she left | 40 (theirs) + hers | 2 |
| Upheaval | 15 m | 80, launches | 80 | 4 |
| Reaper's leap | 10 m | 90 in 2.5 m on landing | 90 | 3 |
| Teleport / flow | 20 m / 6 m | 40 in 2 m on arrival | 40 | — |
| Frenzy | 6 m around her | 120 a second, + exploding stains | the bleed's | grey (§4) |
| Hemoplague | 22 m, 5 m cloud | 300 over 5 s | one stain at the centre | 15 |

### On foot — the charges

**Transfusion** (left, held). *Vladimir's Q, RO's bolts and Jupiter Thunder.* A bolt along the
crosshair. T1 a fast bolt; T2 a heavy bolt that **pierces** everyone on the line; T3 the **lance**,
a long knockback whose victim leaves a stain **streak** along the slide. Recovery grows by tier:
T3 is punishable on a whiff by everybody.

**Black spike** (middle, held). *The template; RO's Heaven's Drive and Lord of Vermilion.* T1 is
today's spike — 30 frames of telegraph, launch, slow; on a stain the eruption and the chain, as
built. T2 three spikes in a 3 m triangle, staggered 6 frames. T3 a **field** 6 m across, three
waves half a second apart; on a stain, every wave erupts it again until it is gone. The answer to
"spam spike": a tap is the poke, the damage is in a second or more of being hittable.

**Reaping sweep / Tides** (right). *Vladimir's Tides of Blood.* **Tap**: the sweep as built, the
close-range "get off me", growing with grey. **Hold**: **Tides**, the scythe whirled overhead and
released as a ring of blood that pushes everything out. The Blood nova folds into this.

### In the air — quick and committed

Every aerial gives her a short **hang** (fall speed capped through the startup), so from the top of
a jump she casts down onto the floor. Aerials do not charge; a stored charge fires instead when its
own click is pressed.

**Haemorrhage** (left, air). As built, at 18 m with a 0.9 m bolt, and the slower, heavier bleed
(R2) so its trail is a road.

**Stake** (middle, air). *RO's Meteor Storm, one meteor.* A long black spike thrown down along the
crosshair. Through somebody **in the air** it **pins** them and lands in the floor beneath; on the
floor it hits a 2.5 m area. Replaces the Nail, which only did anything to a body already in the air.

**Reaper's chain** (right, air). *The Hook, fixed.* The scythe thrown on a chain to **exactly where
the crosshair is** — the skillshot's end point: floor, wall, ledge, stone or body. On a surface it
pulls her there, keeping her momentum. On a fighter it yanks **them** to her. On a creature it
pulls her to it. On nothing (the sky) it still pulls her 5 m along the aim, so it never comes back
empty. The Hook today only tests a thin line, so looking at open floor — the commonest thing to aim
at from the air — caught nothing; aiming at the crosshair's own point fixes that.

### Leaving the floor — the takeoffs

**Blood wings** (space + left). *The Demon Avenger's wings; the Blood jet, fixed.* A **tap** bursts
her 8 m along the aim, any direction including up, hurting whoever is beside her as she goes and
leaving a stain of her own blood at the takeoff — a teleport target, never a heal.

**Upheaval** (space + middle). *Heaven's Drive, one column.* A geyser at the crosshair that
launches; on a stain it is wider and launches higher. She hops as she casts, already in the air for
the Stake (C1). Replaces Marionette, which launched at sword range.

**Reaper's leap** (space + right). *A demon's slam.* A leap to a point up to 10 m along the
crosshair, the scythe brought down on landing; every stain within 3 m is absorbed and adds its
extra damage to the slam. Replaces Harvest.

## 7 · What goes, what stays

| Built (0009) | Becomes |
| --- | --- |
| Blood nova (left, held) | Folded into **Tides** (right, held) |
| Grasp (middle, held) | Its haul goes to **Reaper's chain** on a fighter; cut |
| Reaping sweep (right) | Kept as the tap of right click; 22 → 45 |
| Haemorrhage (left, air) | Kept; longer range, a heavier, slower bleed |
| Nail (middle, air) | **Stake**: area on the floor, pin kept |
| Hook (right, air) | **Reaper's chain**: to the crosshair's point, never empty, deals damage |
| Blood jet (space + left, held) | **Blood wings**: tapped, 8 m, deals damage |
| Marionette (space + middle) | **Upheaval**: a ranged launch |
| Harvest (space + right) | **Reaper's leap**: lands, slams and collects |
| Bloodletter (`Q`) | Cut (it was in no combo); `Q` is **Frenzy** |
| Black spike (`E`) | Moves to middle click, held, three tiers; `E` is **Hemoplague** |
| Blink (`shift` at a pool) | The **teleport**, 20 m, collects, bursts on arrival; else **Sanguine flow** |

**Kept unchanged:** grey health, as the ceiling on a heal; ×1.4 against the disabled; a cut never
kills her; her own blood never heals her; the size rule of a stain; the eruption and its chain; the
scythe drawn at the size it hits at.

## 8 · Movement, compared

| | Before | After |
| --- | --- | --- |
| On the floor | A dodge; a blink only to her own narrow pools | A 20 m teleport to any stain; else a 6 m slide that collects |
| Up | A plain 4 m jump; a held jet | Blood wings, 8 m any direction, tapped |
| Across a gap / onto a ledge | The Hook, air only, only if a thin line meets something | Reaper's chain to wherever the crosshair is, 18 m, never empty |
| Into a fight | — | Reaper's leap, 10 m, lands with a slam |
| Inside Frenzy | — | Teleport along trails, a second more of Frenzy each |

Every one of them **ends a charge** (or banks it). She is the most mobile she has been, and every
metre of it is a moment she is not charging.

## 9 · What it costs to build

Nothing in the aiming model changes; every line of effect is one of the five in `aim.rs`:
Transfusion, Haemorrhage, Stake, Reaper's chain and Blood wings are **skillshots**; Black spike,
Upheaval, Reaper's leap and Hemoplague are **grounded**; the sweep and Tides are **swings**; the
teleport asks `aim::pointing_at_disc` and the flow `aim::blink_to`, as the blink and the Dual
mage's blink already do. Frenzy's area and pull are effect-against-effect distances, not aiming.

- **Charge tiers and the vessel.** The Blood nova's held-and-paid channel, generalised to three
  moves with tier thresholds; `Player` gains the vessel (a move and a tier: two bytes).
- **Collection.** Walking: a disc-against-feet test in `step_effects`. Attacks: `drink_over`
  already runs on every hit; it gains the extra damage and runs for every move. The teleport: the
  blink's own pool, absorbed instead of spent.
- **Creature stains by the move's damage.** The `spill` after a creature hit takes the move's
  damage, not `dealt`.
- **Frenzy.** A clock and a locked-grey flag on `Player`; the pull and the explosions in
  `step_effects`, over the pools already in the effect array.
- **Hemoplague.** A cloud effect with a tick, and one stain it owns at its centre.
- **New clips:** the Tides whirl, the lance, the Stake's throw, the wings, the leap's slam, the
  flow, Frenzy's stance. The spike's, the sweep's and the Haemorrhage's are built.
- **The move table** grows by appending, never renumbering, so no knob moves.

**Tests to add or change.** `feel.rs`: *every tier-3 charge is punishable on a whiff by every
class*; *a stored charge does what the same charge released at once does*; *every Blood mage move
deals damage*; *every hit she lands on a creature leaves a stain of at least its move's damage*;
*no stain heals her during Frenzy, and nothing she took during it comes back*. `grey.rs` gains
Frenzy's lock. `blood_mage_clicks.rs`: one test per new move and C1–C7 end to end. And `--bin
essence` re-keyed so it measures the moves it names.

## 10 · Questions still open

- **BLAST-4 · Black spike off `E`.** It becomes the middle click, held. Or it stays on `E` as well,
  and the Hemoplague moves to `R` or `F`.
- **BLAST-5 · Grasp cut.** Its haul lives on in the chain. Keep the held, aimed-by-time Grasp
  anywhere?
- **BLAST-6 · Ranges.** 18–22 m against today's 9 — guesses at a blaster's range; the first play
  says whether the arenas are wide enough.
- **BLAST-7 · Frenzy's entry price.** 150 grey to start it, so she has to be hurt. Or free, and the
  locked grey is the whole price.
- **BLAST-8 · Hemoplague's stain at the centre.** One big stain for the whole cloud, or a stain
  under each body as usual. The first makes the end of the cloud an objective; the second keeps
  the rule simpler.
