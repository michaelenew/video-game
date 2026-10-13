---
status: proposed 2026-10-13, for the owner's review; nothing built
started: 2026-10-13
supersedes: 0015 where they differ, once decided
depends: 0015_blood_mage_blaster.md, ../kits/blood-mage.md
---

# 0016 — The Blood mage: stains, thralls and the Well

The owner's verdict on the blaster (0015), after a Ridgeback fight and a Mireback:

- **What works is the stains and the detonation chain.** Getting the combo rolling "feels very
  blood mage, and it's unique". That is the class. Everything below is built to serve it.
- **The snowball is too hard, and the start is too slow.** She drops to 20% before the first few
  hits land. Then, with a critical mass of stains under the target, she spams attacks into it,
  and because **every eruption leaves its own stain**, the same spot gets *stronger* the more she
  hits it rather than running out.
- **The moves are not synergistic.** The right choice is always "the most damage right now". A
  charge is rarely worth it: an uncharged aerial plus a detonation beats it. The spells are too
  alike, and too much of the damage is locked behind stains.
- **Missing feels awful**, and nothing sets up an area while the target is out of range.
- **The scythe is never worth using, and the hook must go.** Frenzy does not work: against a
  monster she just gets trampled.
- **Space + left only works from the floor.** The jump moves have to work in the air.
- **Some stains do not collect.** "There is no such thing as a special bloodstain."

And the direction:

- **More of her kit is about the stains.** A **carpet** that lays stains as it spreads, slows
  what is inside and grows the longer she holds. Every move that costs health leaves a stain.
- **Summons from stains.** Right click: tap directs them, hold summons them out of the stains
  round her. A thrall is as big and as strong as its stain, drains exactly as its stain would
  have, dies to one hit, and turns back into a stain when it does, less the blow. Jump + right
  click is an upward scythe sweep whose cone speeds her thralls along and collects stains.
- **The point of the class is building to one big spell.** "Your reward for playing well is that
  you can stop for a moment and whatever you want to happen just happens." Bigger, easier to
  hit, more damage, gated behind skilful play, and more than one of them. The owner's three:
  a field of black branches that pins and damages, a blood golem, a vast cone.

This note makes those one design, then works out what the thralls open up.

## 1 · The loop

She is a **blood economy** with three places blood can be, and every move moves it between them:

```
        her health  ──cast──▶  a stain on the floor  ──summon──▶  a thrall
             ▲                  │       ▲                          │
             │ drink            │       │ a thrall killed          │ its bites
             │                  ▼       │ or run dry               ▼
        overflow ──▶ the Well   erupt ──┘ (nothing left)     a stain under its prey
                         │
                         ▼
                    a finisher
```

1. **Seed.** Every cast leaves what it cost as a stain where it lands. The carpet seeds an area.
   Hits on bodies spill them, as now.
2. **Shape.** Gather stains together (Coagulate, `Q`), animate them (summon), send the thralls
   where she wants them (tap right click).
3. **Cash.** Drink a stain to close a wound, erupt it for damage, or let a thrall bite with it.
   Drinking past a full bar fills **the Well**.
4. **Unleash.** With the Well full, one of the three held clicks goes past its last tier into a
   **finisher**. Then she goes back to seeding.

The snowball is still there, but it now builds toward the finisher instead of feeding itself.

## 2 · The stain rules, made uniform

The owner's rule: **no special stains.** Every stain, whoever's blood and whatever made it, has
the same properties:

| Rule | Now | Proposed |
| --- | --- | --- |
| Whose blood | Hers is a door, never a heal; theirs heals | **All the same.** Hers is what a cast cost, so drinking it is getting the cost back |
| Walking | Drinks only what her grey can hold; at full health it leaves the stain | **Always drinks.** What her grey cannot hold goes to **the Well** |
| The scythe | Only stains older than the swing, never her own | **Every stain it passes**, hers too |
| A cast's cost | Paid and gone | **Left as a stain where the cast lands** (a bolt where it stops, a spike where it comes up, a self-cast at her feet), holding what was paid |
| A hit through a stain | +50% of its essence, and the stain is drunk | +25%, and the stain is drunk |
| An eruption | Damage by the stain's size, **and the blow spills a new stain** | Damage by the stain's size, and **spills nothing** (Frenzy's fix, applied to every eruption) |
| The drain | 5 a second | Unchanged; thralls drain the same |

The three that matter most:

- **A cast's cost becomes a stain where the cast lands.** A miss is a setup. A spike thrown at a
  target out of reach leaves a stain there, waiting. A cast is a loan: she gets it back by going
  there. That is what lets her start the snowball without landing a string of hits first, and it
  means she does not fall to 20% before anything starts. The **send** (the dodge's glob) then
  costs the stain it leaves too: free blood from nowhere would be a free heal.
- **Eruptions spill nothing.** Hitting the same spot over and over now *runs out*, because each
  eruption spends what was there instead of laying more. The detonation chain stays — a spike on
  a field of stains still sets them all off — but it is a cash-out, not a perpetual engine. Half
  of what an eruption spends goes into the Well, so the chain is how she builds toward the
  finisher.
- **Walking always collects.** Overflow goes to the Well, so walking through stains at full
  health is not wasting them; it is charging the finisher. The cases where a stain "does not
  collect" today are exactly the special cases this removes: her own blood, walking at full
  health, and the scythe skipping young stains and her own.

## 3 · The grid

The rule that makes it read: **a tap is the quick thing, a hold is the field**, on every click.
Left and right switch to the held move once the button is held past the tap's startup (not a
charge, the way the owner described right click). Middle keeps its tiers.

| | Left: my blood | Middle: your blood | Right: the thralls |
| --- | --- | --- | --- |
| **On foot, tap** | Haemorrhage (bolt + bleed, now on the floor too) | Black spike, tier 1 | **Command**: the scythe swung, the thralls sent to the crosshair |
| **On foot, hold** | **Carpet of blood**: spreads while held | Black spike, tiers 2–3 | **Summon**: stains round her rise as thralls |
| **In the air** | Haemorrhage | Stake (kept as is: "the only move that feels good") | Command / summon, the same tap and hold |
| **Space + click** (floor **or air**, once per jump) | Blood wings | Upheaval | **Rising reap**: an upward cone that hurls the thralls forward and drinks the stains it crosses |
| **Past the last hold, Well full** | **Thorn garden** | **Exsanguinate** (the cone) | **Blood golem** |

| Key | Move |
| --- | --- |
| `Q` | **Coagulate**: her stains within 12 m slide together to the crosshair's spot and merge |
| `E` | Hemoplague (kept) |
| `shift` | Send and zip (kept; the send now costs the stain it leaves) |

**Cut:** Reaper's chain, Tides, Reaper's leap, Frenzy, and the Transfusion as its own move (its
job, the quick bolt, is the Haemorrhage tapped on the floor). The sweep survives inside Command.

**The jump row works in the air.** Space + a click is a takeoff from the floor, as now, and
**once per jump in the air** as well — a second burst of wings, an Upheaval thrown down from
above, a Rising reap mid-jump. Refreshed on landing, like the airdodge.

## 4 · The new moves

Numbers are first guesses, there so the shape is clear.

**Carpet of blood** (left, held). Aimed with the floor rule: at her feet if she looks down, at
the crosshair's floor out to 15 m otherwise. A disc of blood spreads from that point while she
holds: 1.5 m at the switch, growing to 6 m over two seconds. It **slows** everything inside
(creatures too) and ticks a small amount of damage. Her health pays for it as she holds, and that
is where the stains come from: **when she lets go, the carpet breaks into stains** spread across
its area, holding what she paid plus what its ticks spilled. It is the seed move. It never
erupts, so it adds no burst of its own.

**Command** (right, tap). The scythe swung across her front, the sweep's volume and its
collection, and every thrall of hers is sent to the crosshair's spot on the floor. There they bite
whatever is within a couple of metres; with nothing to bite they wait. This is the sweep made
worth pressing: the swing is incidental and the order is the point.

**Summon** (right, held past Command's startup). A slow two-handed raising, about two-thirds of a
second, interruptible. Every stain within 6 m of her **rises as a thrall** of the same essence,
in the same place.

**A thrall:**
- **Is a walking stain.** Same essence, same drain, same slot in the effect table. A summon turns
  a stain into a thrall in place, and a dead thrall turns back into a stain in place.
- **Size and strength are its essence.** Bigger ones bite harder and move slower.
- **Dies to any one hit**, and leaves a stain of what it had, **less the blow's damage**. Killing
  thralls is worth something to an opponent, and the blood is still there for her.
- **Bites spend it.** Each bite deals damage from its essence and costs it essence; half of what
  a bite costs spills as a stain under its prey. A thrall turns its blood into damage plus blood
  under the target, and loses some each time. It runs out, so the loop decays.
- **At most six**, and they share the effect table with the stains (the cap of eight already
  shared).

**Rising reap** (space + right, floor or air). The scythe swept upward, sending a cone-shaped
shockwave forward about 8 m. Enemies in it take a light hit and a small lift. **Thralls in it
are hurled forward** along the cone, biting whatever they pass through. **Stains in it are
drunk.** It is the slingshot and the vacuum in one swing.

**Coagulate** (`Q`). Every stain of hers within 12 m slides to the crosshair's spot on the floor
over half a second and merges into one. Many small stains become one big one, which makes one
big thrall, one big eruption, or one big drink. It is also the **answer to "they walked off the
blood"**: drag it back under them. The stains do not hurt anything on the way.

## 5 · The Well and the finishers

**The Well** fills from two things only:
- **overflow**: what she drinks that her grey cannot hold;
- **half of what an eruption spends.**

Full at 400 essence, shown as a bar beside her health. It does not drain, and it empties on
death. Nothing else fills it, so filling it means drinking at full health and detonating fields,
which is playing her game well.

**With the Well full, holding a click past its last stage goes into that click's finisher.** It
costs the whole Well and a long, readable wind-up (about a second). Each finisher is the big form
of what its click already does, so choosing one is reading the situation, not picking the
biggest number:

| Finisher | Click | What it does | For |
| --- | --- | --- | --- |
| **Thorn garden** | Left (the carpet's big form) | Black branches erupt **from every stain of hers within 12 m of the crosshair**, rooting and damaging everything in reach for three seconds. More stains, bigger garden | Pinning a group, or something fast, where you have already bled |
| **Exsanguinate** | Middle (the spike's big form) | A vast cone in front of her, about 20 m long and 70° wide: one instant, enormous hit to everything in it. Every stain inside adds to the blow and is spent | A big target in front of her, now |
| **Blood golem** | Right (the summon's big form) | Every thrall and stain within 8 m fuses into one golem, sized by the total. It **takes several hits** (the one exception to one hit), punches hard, draws a creature's attention, and drains like everything else | A long fight, a tank in a hunt, a second body in a duel |

The owner's "stop for a moment and whatever you want to happen just happens": each is big, easy
to land (a cone, an area from her stains, a body that walks), and gated behind a full Well.

## 6 · The combos

Each is a short chain of the moves above; each piece is useful alone.

1. **Lay the field** (the owner's). Carpet on her own feet → let go → Summon → four to six
   thralls at once → Command at the target. Setup without a single hit landed.
2. **Zip and raise.** Haemorrhage the target from range; its bleed drips a trail. Send and zip
   onto the trail beside it → Summon → the trail rises round the target. Thralls start in bite
   range.
3. **The wall.** Something rushes her. Summon what is around her and Command them to her own
   feet. They have to be cut through, one hit each, and every one cut leaves a stain at her
   feet. Black spike on the pile.
4. **Slingshot.** Thralls gathered behind her → Rising reap forward → they fly in a cone through
   the target, biting as they pass; the stains in the cone are drunk on the way.
5. **Gather and raise.** Carpet a wide area → Coagulate it to one spot → Summon one huge thrall,
   or spike the one huge stain for a single big eruption. Many small into one big is a choice
   about the kind of damage she wants.
6. **Drag it back.** They stepped off the blood. Coagulate it back under them → tier-2 spike as
   it arrives.
7. **Pen and feed.** Hemoplague on the target → Command the thralls into the cloud to hold it
   there (they are bodies in its way, and they bite) → the cloud's centre stain → Summon again
   from it.
8. **The build to a finisher.** Lay the field, feed it, spike the field (the eruption's spend
   fills the Well), walk through what is left at full health (overflow fills the rest), then
   **Thorn garden** on everything still standing in her blood.
9. **Stake from the jump** (kept, the owner's favourite). Upheaval → Stake → spike under the
   pinned body. Now with the jump row in the air: jump, Upheaval thrown down from above, Stake.

## 7 · What else this changes about how she plays

- **The thralls buy the time.** 0015's brief, "mobility to buy time for her charges", is now
  answered by bodies rather than legs: thralls pressure and block while she holds a tier-3 spike
  or a finisher. Charging becomes worth it because she is not the only thing in the fight.
- **Missing is setting up.** Every cast's cost lands where it was aimed, so throwing at a target
  that is out of range is not wasted. Opening a fight is seeding, not trading health for hope.
- **She fights the floor, not just the target.** Where the blood is decides everything: summon
  from it, erupt it, drink it, drag it. The counterplay is clear: kill the thralls (a reward),
  walk off the blood, and punish her during the summon and the finisher wind-ups.
- **Damage is no longer one dimension.** Burst (spike, Stake, Exsanguinate), sustained (thralls,
  Hemoplague), control (carpet slow, Thorn garden root, Stake pin), and economy (drink, Coagulate,
  Well). "The most damage right now" stops being the only question.
- **In a hunt**, thralls and the golem are something the creature can hit that is not her, which
  is the owner's "hitting a thing that's not me on her own terms". That needs the creature to
  notice them (below).
- **In a duel**, thralls are zone control; the opponent's best answer is to cut through them,
  which is also what she wants them to waste time on.

## 8 · What it costs to build

In order, each stage playable on its own:

1. **Uniform stains**: costs leave stains at the landing point; walking always drinks, with
   overflow into the Well; the scythe drinks everything; eruptions spill nothing; +25% through a
   stain; the send costs its stain. Plus: the jump row in the air (once per jump), the hook,
   Tides, the leap and Frenzy cut, the Haemorrhage tapped on the floor. Mostly rules on code that
   exists.
2. **Carpet and Coagulate.** The carpet is a growing effect (the Hemoplague's cloud is close),
   and Coagulate moves stains (Frenzy's pull already does).
3. **Thralls**: a new effect kind that *is* a stain on legs (same slot, same essence field),
   Command, Summon, the Rising reap. Thralls need: steering toward a point, a bite, and being hit
   by fighters' and creatures' attacks, which means putting them on the hit paths the critters
   already use.
4. **The Well and the three finishers.** The Well is one number on the player (two bytes; the
   4 KiB snapshot has about 16 to spare). The golem is a thrall with health.
5. **Creatures notice thralls**: the creature's brain targets the nearest body including thralls
   and the golem. The biggest single piece of work in this list, and the one that makes the golem
   a tank.

## 9 · Questions for the owner

- **THRALL-1 · What thralls do on arrival.** Proposed: bite, spending essence per bite, half of it
  spilled under the prey. Alternatives: latch on and bleed the target; or burst on contact.
- **THRALL-2 · Coagulate on `Q`.** Or `Q` holds something else and gathering is a property of
  Command (thralls carry stains with them).
- **THRALL-3 · The Well's fill.** Overflow plus half of eruptions. Should thralls' bites fill it
  too, or would that make the summon the only way to play?
- **THRALL-4 · The finisher input.** Proposed: hold the click past its last stage with a full
  Well. The alternative is one key (`Q`) that fires the finisher of the last click she held.
- **THRALL-5 · The Haemorrhage tapped on the floor.** It replaces the Transfusion, so the floor
  keeps a quick ranged bolt. Or the Transfusion's tiers stay on left and the carpet goes
  elsewhere.
- **THRALL-6 · The send costs its stain.** Needed so the door is not a free heal under the new
  rules. About 3% of red.
