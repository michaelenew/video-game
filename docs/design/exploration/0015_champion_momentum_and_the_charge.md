---
status: open — CHAMP-1 to CHAMP-5 answered 2026-10-12; nothing built
started: 2026-10-12
depends: ../kits/champion.md, ../champion.md, ../monsters.md, ../creatures/mantis.md, 0014_shadow_reaver_marks_and_throws.md
---

# 0015 — The Champion: momentum, the charge, and something to break

## The brief

From the owner, with three replays (a Champion run of 28 minutes ending in five attempts at
the Mantis, and a Reaver and an Elementalist run ending at the same Mantis):

> Compared to the other classes, the champion just has a really linear playstyle and not a lot
> of options for outplay. It's kind of the same attack over and over, and the monster fights
> especially feel really bad because there's no semblance of a stun/combo game. […] there's no
> core loop. It's kind of just stay at the same range, kite, dodge, attack when you see an
> opening. […] the charge needs to be a little faster and more impactful. And there needs to be
> a bigger and better feeling reward for landing his core combo and basic cycle. […] I'm aiming
> for fewer inputs per second, but I want a similar sort of core feel as the others.

Inspirations named: Dark Souls (skilful melee), MapleStory's Aran (combo factory, big payoffs),
Monster Hunter (spectacle and impact).

## What the replays say

`cargo run -p hunt --bin replay -- <file>`, the Champion's 22 minutes as the Champion (the tape
also holds short Blood mage and Dual mage stretches, which the replay now tables apart — see
"The replay tool" below):

| Move | Thrown | Landed |  | Move | Thrown | Landed |
| --- | --- | --- | --- | --- | --- | --- |
| Sword | 91 | 32 | | Upcut (finisher) | 30 | **3** |
| Backcut | 54 | 22 | | Whirl (finisher) | 0 | — |
| Spear | 52 | 6 | | Earthbreaker (finisher) | 0 | — |
| Hammer | 9 | 2 | | Skewer | 5 | **0** |
| Rush slash | 10 | **10** | | Rush stab | 17 | **0** |
| Air sword | 60 | 14 | | Uppercut | 7 | **0** |

Airborne 23 % of the time as the Champion; 57 % as the Reaver and 46 % as the Elementalist on
the other tapes. The Reaver's finishers (Reave, Plunge) landed 7 of 19; the Champion's 3 of 30.

Four things follow, and the first is most of it.

1. **The reward lives at hit three, and hit three never arrives.** Three finishers landed out of
   thirty thrown, all of them the sword's; the hammer was opened nine times and Earthbreaker
   never thrown. The chain is a hit confirm by design — a blocked, parried or whiffed link
   pays its whole recovery — and against a creature that guards (the Mantis was built as the
   Champion's duel and parries the second hit as a habit) the string ends at hit one or two.
   Everything the kit offers as a payoff — the launch, the spike, the guard break, the ring
   clear — sits behind a door the player almost never opens. What is left is sword, backcut,
   reset: "the same attack over and over" is an accurate description of the moves that land.
2. **The charge is a dodge with a hitbox.** Rush is 19 m/s for 20 frames; the ordinary dodge
   every class has is 17 m/s for 22. The Reaver's dash to her shadow is 50 m/s. The one Rush
   move that never stops (the slash) landed every time it was thrown; the one that stops dead
   (the stab) never did.
3. **Nothing carries from one exchange to the next.** Every string starts at zero. The Reaver
   and the Elementalist both bank something between openings — marks on a two-second clock,
   stones on the field — so a short exchange is an *investment*. The Champion's short exchange
   is just a short exchange, so the right play is the one the owner described: hold range,
   wait, poke.
4. **Against a creature his verbs translate to damage.** A launch, a knock-up, a stagger and a
   guard break are things a *fighter* does; a creature reads them as strain, and strain is
   damage taken lately ([monsters.md §4](../monsters.md)). The Champion's 60–100 a hit is not
   a burst, so his control vocabulary goes quiet in exactly the fights the owner says feel worst.

## What the reference games do, as mechanisms

Not what they look like — what each one *does* that the Champion lacks.

| Game | Mechanism | What it teaches here |
| --- | --- | --- |
| **MapleStory, Aran** | A combo counter on the character that rises with every hit, survives a short gap, and at thresholds makes every skill stronger; finishers spend it | Reward that **accumulates across exchanges** instead of resetting per string |
| **Devil May Cry, Nero's Exceed** | Pressing in time with the sword's swing charges the next attack; the timing is a beat, not a reaction | **Skill in timing, not in input count** — the same presses, played well, hit harder |
| **Monster Hunter, Great Sword** | Hold to charge through three levels; the "tackle" carries the charge through an incoming hit | Few inputs, huge payoff, the skill is the read |
| **Monster Hunter Rise / Wilds, offsets and power clashes** | A heavy swing timed into the monster's own attack cancels it and knocks it down | The **clash**: meeting the monster head-on *is* the outplay |
| **Monster Hunter, hammer KO** | Blunt hits to the head fill a hidden stun meter; full, the monster falls over | A melee class whose job is **putting the monster on the floor** |
| **Elden Ring, stance break** | Heavy and jumping attacks fill a hidden posture bar; broken, the boss kneels and a critical hit plays | The payoff for pressure is a **cinematic hit**, not a bigger number |
| **Street Fighter 6, Drive Impact / Drive Rush** | An armoured lunge that absorbs a hit and crumples on a counter; a dash cancel off a normal | The charge **eats a hit and wins the trade** |
| **Lost Ark, counters and stagger** | A boss's telegraphed move can be hit with a counter skill for a stun | A clear, *seen* window where the monster becomes punishable |

The common thread: in every one of them the reward is reached **by doing the core thing
repeatedly and well**, not by completing a specific sequence that the opponent can break.

## The replay tool

Found while reading these tapes: `hunt::replay::Hands` counted moves by slot number and named
them all by the **last** class played, so a tape with a class change folded every class's moves
into one table under the wrong names (the Elementalist tape's "Landfall 161" was mostly a Dual
mage's Reel). Fixed alongside this note: one table per class played, a blow landed counted to the
class that threw it, and `a_change_of_class_gets_its_own_table` in `hunt/tests/replay.rs`.

## Decided, 2026-10-12

The owner's answers to CHAMP-1 to CHAMP-5:

> I don't think beat and hold are mutually exclusive, but the hold needs to do something more
> than just extra damage if it's added. It might provide some more variety and a variation in
> fight tempo and a way to make the most of a mistimed input. […] the fully charged versions of
> the attacks get an extra effect. Momentum is a meter for now and a glow on the weapon. Charge
> should reduce incoming damage up to a threshold. I don't want a big long term damage soak
> ability though. The last moments of the charge are what can eat a hit entirely, everything
> else is damage reduction, and enough damage while charging can stop the charge. [Grand
> finisher without momentum:] No. [The Clash on fighters:] Yes.

What that changes below: **C1**'s armour is rewritten as three bands; the **Hold** (once C7) is no longer the
Beat's alternative but its other half, for every weapon, with an extra effect on each full
charge; **C3** is a HUD meter and the weapon's glow; **C4** is reached only through momentum;
**C6** works on fighters, with a rule for who wins.

## The ideas

Roughly in order of how much they fix for what they cost. They are designed to stack
into one loop (§"The loop"), but each stands alone.

### C1 · The Charge — Rush made fast, braced and an opener

The owner's first ask, and the smallest change with the largest effect.

- **Faster.** 19 → ~32 m/s, 20 → ~12 frames: about the same 6.5 m, arriving in a fifth of a
  second instead of a third. Fast enough that it reads as a lunge rather than a step.
- **The shoulder.** Running into a body is itself a hit: little damage, a long freeze (8–10
  frames, a hammer's worth of stick) and a short stagger. **It counts as hit one of the
  string**, so a charge that connects opens the chain — the Champion's core combo now starts
  from the move that crosses the gap rather than after it.
- **Braced, not armoured** (decided). Three bands over the charge's twelve-odd frames:
  1. **Most of the run — damage reduced.** A hit costs him a share of its damage (half, as a
     first guess) and does not stop him.
  2. **The last few frames — the shoulder down.** One hit here is **eaten whole**: no damage,
     no stop. This is the clash in its simplest form, and it is a timing, not a buff — charge
     so that their swing arrives as you do.
  3. **Too much, and it breaks.** Damage taken during one charge (after the reduction) is
     summed; past a threshold the charge ends in an ordinary hitstun. A creature's big bite
     stops him; a poke does not.

  Nothing lasts past the charge, so there is no soak to sit in: twelve frames, once per charge.
  It stays punishable on block like everything else (`feel.rs` asks it). Knobs: `Charge, damage
  taken (%)`, `…, frames at the end that eat a hit`, `…, damage that breaks it`.
- **Refund.** A **landed finisher refills the charge.** This is the loop's hinge: finish, and
  the way out (or back in) is already there.
- **The stab carries.** Rush stab stops dead and landed none of seventeen. On contact it should
  drive *through* — pin and push them along the rest of the charge (Monster Hunter's lance
  charge), with the stop and the big hit at the end.

Knobs throughout; the speed and the refund are a morning's work.

### C2 · The Beat — a link pressed on the freeze is a true link

Every connected hit already freezes both bodies (sword 3f, spear 4–8f, hammer 8–13f), and a
press during the freeze is already delivered on the frame after. So the beat exists; nothing
rewards finding it.

**Press the next link during the freeze of the last one** and it is a *true link*: a few more
frames of freeze on the next hit, a bit more damage, a distinct sound and a flash on the
weapon, and more momentum (C3). Press late, it is an ordinary link, as now.

What falls out for free: the window *is* the weapon's mass. The hammer's 13-frame freeze is an
easy beat to find; the sword's 3 frames is a hard one. Heavy is forgiving, fast is precise —
the risk ladder nobody had to design. And it is the DMC lesson exactly: **the same number of
presses, played in time, hits harder.** It asks nothing more of the hands per second.

### C2b · The Hold — the slow path, for every weapon (decided: beside the Beat)

The Beat is the fast way through a string; the Hold is the slow one, and the point of having
both is that **no press is wasted**. Hold a weapon button and its move stops at the top of the
wind-up and charges, in two steps — part and **full** — shown on the weapon (the head glows,
the trail thickens). Release to swing. Hold too long and it swings on its own.

**One rule separates them:** the Beat is decided by when you *press*, the Hold by when you
*let go*. Pressed in the freeze and released inside it, the link is a true link. Still held
when the freeze ends, it is a charge, and the beat is forgone.

What that buys, in the owner's three words:

- **A mistimed input is recovered.** Pressed late for the beat, keep holding: the time you were
  going to lose is charge time instead. Pressed early, the same.
- **Tempo.** A held link is a **delayed** link, and a delay is the oldest mix-up in melee
  (Souls' delayed swings, a fighting game's frame trap). The Mantis parries the second hit
  *when it comes*; a second hit that comes half a second late is a different question. While
  a link is held, the string's grace window waits for it, up to the hold's cap.
- **Variety.** Every full charge does something its quick version does not — not more damage
  (a little, but that is not the point), a different **effect**:

| | Full charge, hits 1–2 | Full charge, the finisher |
| --- | --- | --- |
| **Sword** | **The wave.** The cut leaves the blade as a crescent and travels ~6 m along the aim (a skillshot, `aim::skillshot_path`). The sword's answer to the kiting the owner describes | **Upcut** carries him up with them — an air door that does not cost the takeoff |
| **Hammer** | **Guard crush.** Blocked, it breaks the guard (the block's stun becomes a stagger), and it adds a large share to a creature's strain (C5) | **Earthbreaker** leaves a ring on the floor that trips whoever stands in it, as wide as the crater |
| **Spear** | **The lunge.** The step becomes ~4 m and the point pierces: everything on the line is hit, and a fighter is pinned and carried to the end of it | **Whirl** turns twice and **pulls inward** instead of shoving out — it gathers them for the next string instead of ending it |

A full charge that lands counts for momentum as a true link does. A charge is still a wind-up:
hit out of it and it is lost, and the full charge's extra frames are frames to be read and
punished in — that is the price of the effect.

### C3 · Momentum — reward that survives between strings

The Aran counter, built to be the Reaver's opposite. Her tally lives **on the target** and is
cashed by leaving and arriving; his lives **on himself** and is lost by leaving.

- Every connected hit adds to it; a hit with a **different weapon** from the last adds more
  (the chain's existing incentive to mix, made visible); a true link (C2) adds more again.
- It **survives between strings** for about two seconds without a landed hit, then drains.
  Taking a hit costs a share of it. Walking away to reset costs all of it, slowly.
- **Shown twice** (decided): a **meter on the HUD** with the tiers marked on it, and a **glow on
  the weapon** in his hands that climbs from ember to white.
- **Three tiers:**
  1. *Heated* — more damage, brighter trails.
  2. *Blazing* — finishers gain armour (they can't be stuffed by a parry), the charge recharges
     faster.
  3. *Champion* — **the next finisher is a Grand finisher** (C4) and spends the lot.

This is what makes the class stop being "kite, wait, poke". Every short exchange that lands a
single hit is now an investment, and backing off has a price, so the correct play is to stay in
— which is what a champion is. It also puts the finisher's reward back within reach: the big
payoff is gated on **hits landed in total**, not on three in a row that a guarding creature
will always break.

Momentum is gameplay, so it lives in the `World` snapshot and is hashed. It holds no position,
so `World::shift` does not need it.

### C4 · Grand finishers — the spectacle

One per weapon, each the weapon's finisher made enormous. What it is matters less than three
properties: a long freeze on both bodies (20+ frames), a camera punch, and a mark on the world
that lingers (a crater, a scar, debris).

- **Sword — Thousand Cuts.** The upcut becomes a rising flurry: he follows them up and the
  blade lands four or five times on the way.
- **Hammer — Meteor.** Earthbreaker becomes a leap and a crater: a ring that lifts everything
  standing in it. Against a creature, a guaranteed stumble (C5).
- **Spear — Heaven-piercer.** The whirl ends with the spear thrown through the target and
  pinned in the floor behind it; he charges to it to pull it out (a free C1 to wherever it
  landed).

**Momentum's top tier is the only way in** (decided): a perfect string does not earn one on
its own, so the Grand finisher stays the reward for a fight played well rather than for one
string played well.

This is the place to spend art and sound, because it is the moment the player is working
toward. Aran's lesson: the payoff has to look like the payoff.

### C5 · Break and Ruin — the stun game against monsters

The owner's sharpest complaint: no stun or combo game against creatures. The creature model
already has the ladder — flinch, stumble, topple ([monsters.md §5](../monsters.md)) — the
Champion just doesn't climb it.

- **Impact.** Champion blows feed a creature's strain *beyond their damage*: hammer hits and
  finishers count double, true links half again, Grand finishers a lot. He is the class that
  puts monsters on the floor, the way Monster Hunter's hammer is. Other classes keep strain as
  damage.
- **Ruin.** When a creature is stumbled or toppled, the Champion's next finisher *aimed at its
  weak point* becomes a **Ruin** — a fixed, cinematic heavy hit with a long freeze (Elden
  Ring's critical on a broken stance). A clear reward for the clear moment.

### C6 · The Clash — meeting the monster's swing

Monster Hunter's offset attack. **A hammer finisher, a Grand finisher, a full Hold or the braced charge
whose live frames meet a creature's live attack on the part that is swinging** cancels that
attack: the creature is interrupted (the one thing besides a topple that may already cancel a
live hitbox), both freeze for a long beat, the sound is the loudest the game makes, and the
Champion takes nothing.

**On fighters too** (decided). Two live attacks meeting, at least one of them a Champion's
clashing move (a hammer finisher, a Grand finisher, a full Hold, the charge's last frames):
**the heavier wins** and the lighter is cancelled into a short stagger — heavier meaning the
longer impact freeze, the number that already says how much a weapon weighs. **Equal, both
recoil**: two Champions' hammers meeting bounce both of them apart, neutral again, with the
biggest freeze in the game. A Clash is not a block, so it moves no on-block number; it is a
trade the heavier blow wins outright, which is the fantasy and is also the thing to watch in
tuning — a Clash too easy to land is a parry with no risk.

It is the outplay the class is missing against a creature: the alternative to dodging the
scythe is **hitting the scythe**. It needs the creature's tell to be readable (they all are,
by design) and it pays the player for reading it rather than for running from it. The Mantis
counters off a parry; the Clash is the Champion's counter to its counter.

## The loop

What the kit plays like with C1–C6 together, in a sentence:

> **Charge in** — braced, the shoulder is hit one → **two links, on the beat or held** →
> **finisher** — refunds the charge, banks momentum → **charge out, through, or back in** →
> again. Momentum
> climbs across loops; at the top the finisher is **grand**. Against a creature every loop
> also fills its break; when it falls, **Ruin**; when it swings at you, **Clash**.

About one press every third of a second, much as now; the skill has moved from *which button*
to *when*, which is what the owner asked for.

## Suggested order to build

1. **C1, the Charge.** Speed and refund are knobs; the shoulder and armour are small. Play it.
2. **C2, the Beat.** Small, and it changes every hit the class lands.
3. **C2b, the Hold**, the timing first (the wind-up held, the grace window waiting), then the
   six full-charge effects one at a time, the sword's wave last (it is new machinery).
4. **C5's impact.** One multiplier where Champion blows meet strain. Re-run the Mantis harness.
5. **C3, Momentum**, with a plain HUD meter and the weapon glow.
6. **C4, Grand finishers.** The expensive one: clips, trails, sound.
7. **C6, the Clash**, creatures and fighters, and **C5's Ruin.**

Replaying the owner's Mantis tape after each step is the measurement: the finishers-landed
column above is the number to move.

## Open questions

- **CHAMP-1** — answered: both. The Beat is the press, the Hold is the release (C2b).
- **CHAMP-2** — answered: a HUD meter and the weapon's glow.
- **CHAMP-3** — answered: reduced through the run, one hit eaten in its last frames, broken by
  enough damage (C1).
- **CHAMP-4** — answered: no; a Grand finisher needs momentum's top tier.
- **CHAMP-5** — answered: yes; the heavier blow wins, equals both recoil (C6).
- **CHAMP-6.** The air share (23 %) is the other half of "less dynamic". Out of scope here, but
  a charge that works in the air (an air dash with the shoulder) is the obvious next door.
- **CHAMP-7.** The six full-charge effects in C2b are first proposals. The sword's wave is the
  only one that adds a new kind of thing (a travelling cut) and the one most worth questioning:
  it gives the closing weapon a ranged answer, which is what the kiting problem asks for, but it
  also overlaps the spear's job of owning distance.
