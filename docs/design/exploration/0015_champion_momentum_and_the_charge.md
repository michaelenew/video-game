---
status: decided 2026-10-12 by the owner over three rounds and built the same day; a fourth round, from play, built 2026-10-13 -- ../kits/champion.md §"The rework"
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

Two rounds of the owner's answers. The first, to CHAMP-1 to CHAMP-5:

> I don't think beat and hold are mutually exclusive, but the hold needs to do something more
> than just extra damage if it's added. […] the fully charged versions of the attacks get an
> extra effect. Momentum is a meter for now and a glow on the weapon. Charge should reduce
> incoming damage up to a threshold. I don't want a big long term damage soak ability though.
> The last moments of the charge are what can eat a hit entirely, everything else is damage
> reduction, and enough damage while charging can stop the charge. [Grand finisher without
> momentum:] No. [The Clash on fighters:] Yes.

The second, on the first write-up of the hold:

> Whirl […] isn't a finisher [if it pulls them in]. Whirl should be a big knockback.
>
> Charged attacks shouldn't be limited to just beat hits. The charged attack breaks tempo but
> may save you. In smash if you realize you've started a smash too late, the answer is often to
> hold it, either so your recovery frames happen while your opponent is not able to capitalize
> or to wait for your opponent to make a mistake in their spacing. And sometimes you can't hit
> them and can't approach, and a charged attack becomes a reasonable deterrent against certain
> neutral plays. […] it doesn't compete with the beat because it's in a totally different
> space. It's still a good idea that the charged attacks vary with whether it's the first,
> second, or third hit though.
>
> I wonder though if the grand finishers aren't supposed to be the same as the fully charged
> attacks. That's a nice natural reward - you don't have to charge anymore to get your grand
> finisher - but leaves the spectacle and the impact available for times when you can't keep
> up a barrage of hits (like the galewing) or for lower skill player fights that will involve
> less contact. […] the stronger is the third. That way the core of the class is still "decide
> mid swing what your next move is", with the third hit being strongest but nothing being
> completely locked behind the 3 hit combo. Plus I do think it will read as too complex if
> there are multiple big hit kinds gated behind separate conditions.
>
> I do want the charge effects to be genuinely net new moves, bigger, faster, stronger, and
> more spectacular than what's existed before.

The third round, building it:

> [Heaven-piercer's spear coming home on its own] is antithetical to the class. The spear comes
> back immediately. The class isn't actually 3 different weapons, it's supposed to be 1 weapon
> that shape shifts. The rush should be the second part of the move, not a choice later.
>
> [On a downed creature, every third hit is a full charge] — not accurate. Every full charge on
> a third hit is a grand finisher. An uncharged third hit is a third hit.
>
> Good with the move proposals, build it.

So, in one line: **there is one kind of big hit — the full charge — and two doors to it.**
Hold for it, or earn it with momentum and it comes out on a tap. A full charge on a third hit is
a grand finisher; an uncharged third hit is a third hit. Six of them, one per weapon per place in the string, and every one a new
move rather than an old one with a bigger number.

To keep two things called "charge" apart, the dash is called by its in-game name, **Rush**,
throughout; **charge** means only a held attack.

## The fourth round, 2026-10-13, from play

> When full, the next tap comes out as a full charge and empties the meter -> What does this
> mean? [...] the momentum consumption feels like it should be more of a [Final Fantasy limit
> break -style] attack that's intentionally chosen. A bar that buffs you the fuller it is with
> a big move that drains it. [...] Momentum doesn't build against the hornback. [...] I don't
> think the clash with a rush is working against the hornback charge [...] build the clash
> against every attack in the game, and ideally in a way that doesn't require special logic on
> every new move. [...] threading them into combat right now still feels repetitive because
> the pattern boils down to "click a lot to get momentum full, then click some more to make the
> most of momentum". [...] I'm wondering if the control scheme needs to become "tap to combo,
> hold to finish". The player can choose any moment to hold the mouse button, but the charge
> duration becomes more like the length of the windup frames of the tap version of the move,
> and holding is simply the indicator that you'd like to move to the finisher version of the
> move. [...] It also gives it some ranged poke with a hold on lclick. It also brings rush
> functionality onto the rclick hold [...] (DON'T remove rush from e for now) [...] the third
> click hold doesn't need to have extreme frame data. [...] the hold attacks are the [move
> that spends the meter], dealing more damage and consuming the full meter (I don't want to
> disincentivize using the charge attacks at partial momentum) and getting some extra cool
> effect if you get it going. [...] I'd want [them] to not really ever be the optimal choice for
> agility sections [...] so [...] extra damage, more and more obvious stagger, and bigger /
> flashier effects but using the same movement and player animations as the ordinary charged
> attacks [...] These aren't really "limit breaks" btw, so don't call them that anywhere.

What changed, and it supersedes the sections below where they differ:

- **Tap to combo, hold to finish.** A link's button held for `Hold, a finisher after` (12
  frames from the press -- about a wind-up; a press a little long on the sword's six is still
  the sword) is that weapon's **finisher**, at any place in the string: the first two links'
  are the Crescent, the Shockwave and the Comet, the third's the grand three. There is no part
  charge. A finisher **ends the string**. The finishers' own wind-ups are short (2-4 frames),
  because holding through the tapped move's wind-up was the commitment; their recoveries are
  the price.
- **The Comet is a Rush on the spear**: it goes where the stick is held, along the crosshair
  with the stick let go. Rush stays on `E`.
- **Momentum** buffs every hit he lands by its share of 20 % (a full meter), and **every
  finisher spends all of it**, whatever is in it: it is worth its share of 75 % more damage and
  size, its share of 40 frames more stagger, and a full meter spent knocks a fighter down and a
  creature off its feet. Nothing about where it goes or how it moves changes, so it is never the
  answer to a jump. The tap at a full meter, the faster holds and the tiers' buffs are gone; the
  tiers are only the colours now.
- **Momentum builds on everything**: a link landing on a critter -- the Hornback's bull, a
  gnawer -- confirms the string and feeds the meter, as on a creature or a fighter.
- **The Clash is where every blow lands** (`state::apply_hit`): a blow meeting his heavy one is
  spent on it, and whoever threw it is knocked out of it afterwards by where they are
  (`World::settle_the_clashes`) -- a creature mid-swing that reaches him flinches, a critter
  mid-attack that reaches him is knocked out of it, a fighter within 6 m is staggered. No attack
  needs a line of its own; the Hornback's charge, every species' own blows and every critter's
  are covered by construction.

## The ideas

Roughly in order of how much they fix for what they cost. They stack into one loop
(§"The loop"), but each stands alone.

### C1 · Rush — fast, braced, and an opener

The owner's first ask, and the smallest change with the largest effect.

- **Faster.** 19 → ~32 m/s, 20 → ~12 frames: about the same 6.5 m, arriving in a fifth of a
  second instead of a third. Fast enough that it reads as a lunge rather than a step.
- **The shoulder.** Running into a body is itself a hit: little damage, a long freeze (8–10
  frames, a hammer's worth of stick) and a short stagger. **It counts as hit one of the
  string**, so a Rush that connects opens the chain — the core combo starts from the move that
  crosses the gap rather than after it.
- **Braced, not armoured** (decided). Three bands over the Rush's twelve-odd frames:
  1. **Most of the run — damage reduced.** A hit costs him a share of its damage (half, as a
     first guess) and does not stop him.
  2. **The last few frames — the shoulder down.** One hit here is **eaten whole**: no damage,
     no stop. A timing, not a buff — Rush so that their swing arrives as you do.
  3. **Too much, and it breaks.** Damage taken during one Rush (after the reduction) is
     summed; past a threshold it ends in an ordinary hitstun. A creature's big bite stops him;
     a poke does not.

  Nothing lasts past the Rush, so there is no soak to sit in. It stays punishable on block like
  everything else (`feel.rs` asks it). Knobs: `Rush, damage taken (%)`, `…, frames at the end
  that eat a hit`, `…, damage that breaks it`.
- **Refund.** A **landed third hit refills the Rush.** The loop's hinge: finish, and the way out
  (or back in) is already there.
- **The stab carries.** Rush stab stops dead and landed none of seventeen. On contact it should
  drive *through* — pin and push them along the rest of the run (Monster Hunter's lance
  charge), with the stop and the big hit at the end.

### C2 · The Beat — a link pressed on the freeze is a true link

Every connected hit already freezes both bodies (sword 3f, spear 4–8f, hammer 8–13f), and a
press during the freeze is already delivered on the frame after. So the beat exists; nothing
rewards finding it.

**Press the next link during the freeze of the last one** and it is a *true link*: a few more
frames of freeze on the next hit, a bit more damage, a distinct sound and a flash on the weapon,
and more momentum (C4). Press late, it is an ordinary link, as now.

The window *is* the weapon's mass: the hammer's 13-frame freeze is an easy beat to find, the
sword's 3 frames a hard one. Heavy is forgiving, fast is precise. And it is the DMC lesson:
**the same number of presses, played in time, hits harder.**

The Beat lives inside a string that is landing. The Hold (C3) lives everywhere else. They do
not compete.

### C3 · The Hold — a charged attack, from anywhere

Smash Bros.' charged smash attack, for the reasons the owner gives. **Any grounded weapon press
can be held** — an opener in neutral, the second hit, the third — and the move stops at the top
of its wind-up and charges. Release to swing. Hold to the cap (~1.5 s) and it swings on its own.

What a hold is *for*, none of which is damage:

- **Saving a press you regret.** Started a swing too late, or into somebody who has stepped
  back? Hold it. Your recovery now happens when you choose, not when they are ready to punish
  it, and a held swing is a threat they have to respect while you wait.
- **Waiting for their mistake.** A full charge aimed at the gap they have to cross is a wall
  they must walk around or bait out. It is the answer to "I can't hit them and can't approach".
- **Breaking tempo.** A held hit arrives late, and a late hit is the oldest mix-up in melee. The
  Mantis parries the second hit when it comes on time; held half a second, it does not. While a
  link is held the string's grace window waits for it.

The rules: he can **turn** while holding but not walk. **Rush cancels a hold** (the universal
cancel, as now), which makes a held swing a feint as well as a threat. Hit out of the hold, the
charge is lost. A **part charge** is the ordinary move, harder and further. A **full charge** is
a different move altogether — the six below.

### C4 · Momentum — charge banked by fighting

The Aran counter, built to be the Reaver's opposite. Her tally lives **on the target** and is
cashed by leaving and arriving; his lives **on himself** and is lost by leaving.

- Every connected hit adds to it; a hit with a **different weapon** from the last adds more; a
  true link (C2) adds more again.
- It **survives between strings** for about two seconds without a landed hit, then drains.
  Taking a hit costs a share of it.
- **Shown twice** (decided): a **meter on the HUD** with the tiers marked, and a **glow on the
  weapon** in his hands that climbs from ember to white.

**What it buys is charge.** Momentum is a full charge you have already paid for in hits:

1. *Heated* — holds charge faster.
2. *Blazing* — faster again.
3. *Champion* — **the next press comes out fully charged on a tap**, and spends the meter.

So the six full charges have two doors, and they suit two different fights. A player in a
fight with lots of contact (the Mantis, another fighter) earns them with momentum, and they come
out at full speed in the middle of a string — "you don't have to charge anymore". A player in a
fight with little contact (the Galewing overhead, a newer player's fight) holds for them, and
the spectacle is still there. Nothing is locked behind three hits in a row.

Momentum is gameplay, so it lives in the `World` snapshot and is hashed. It holds no position,
so `World::shift` does not need it.

### C5 · The full charges — six new moves

One per weapon for the first and second hits, one per weapon for the third, **and the third is
the strongest**. The core of the class is unchanged — decide mid-swing what the next hit is — and
the place in the string now also decides *how big* a full charge is.

Every one is **new**: its own volume, its own clip, its own trail and sound, and several are new
kinds of thing in the game. Each is bigger, faster on release and more spectacular than anything
in the kit today, and each has a long impact freeze, a camera punch, and something it leaves in
the world for a moment afterwards. The shapes keep the class's rule that **the shape names the
weapon**: the sword owns a line ahead, the hammer the ground around him, the spear distance.

| | Full charge, hit 1 or 2 | Full charge, hit 3 — the strongest |
| --- | --- | --- |
| **Sword** | **Crescent.** The cut leaves the blade: a wide crescent of light flies ~10 m along the aim, cutting everything it passes. The sword's answer to the kiting the owner describes, and a neutral deterrent down a line | **Thousand Cuts.** A drawn-sword dash *through* them, ~7 m in a flash, and he is past. A beat later the cuts appear in the air along the line he took and land, five of them, then a last one that throws them up. The samurai who walks through a line of people and they fall over behind him, finally literal |
| **Hammer** | **Shockwave.** The head driven into the floor and a ring of broken ground bursting out ~5 m round him: everything standing in it is thrown up, and a guard that blocks the head itself is crushed. The deterrent all round, where the sword's is a line | **Meteor.** He leaps — high, ~5 m — and comes down where the crosshair is (`aim::grounded_path`) as a falling star. A crater, a wide ring that launches, a scorched floor for a few seconds, and against a creature a guaranteed stumble. The hit that turns a fight |
| **Spear** | **Comet.** The spear levelled and him behind it, ~8 m along the aim in an instant. Everything on the line is pierced; a fighter is caught on the point and carried to the end, and **slammed into a wall** if there is one (a wall-splat: a long stagger) | **Heaven-piercer.** The spear *thrown*: a javelin that crosses ~25 m, pierces everything, and **pins** whoever it ends in, with the biggest knockback in the class — and he goes after it at once: the throw's second half is a rush along its line to where it stopped, arriving with a hit as he tears it out. One weapon that changes shape, never two weapons and a missing third |

The ordinary third hits are unchanged and stay what they are: **Upcut** pops them up,
**Earthbreaker** breaks guard and launches, and **Whirl is a big knockback** — the ring cleared,
two and a half metres, the furthest shove of any link (the owner: "Whirl should be a big
knockback").

**New machinery**, and the owner has asked for it rather than around it: a travelling cut (the
Crescent), hits that land after he has left (Thousand Cuts), a ring burst off the floor
(Shockwave), a leap to an aimed point (Meteor; the Elementalist's Landfall is the nearest thing
built), a carry and a wall-splat (Comet), and a thrown weapon that stays in the world and is
gone from his hands (Heaven-piercer). Every one goes through `aim.rs` for where it goes, the
Oven for its numbers and `state::hitbox` for its volume, like everything else.

### C6 · Break — the stun game against monsters

The owner's sharpest complaint: no stun or combo game against creatures. The creature model
already has the ladder — flinch, stumble, topple ([monsters.md §5](../monsters.md)) — the
Champion just does not climb it.

**Impact.** Champion blows feed a creature's strain *beyond their damage*: hammer hits and third
hits count double, true links half again, full charges a great deal. He is the class that puts
monsters on the floor, the way Monster Hunter's hammer is. Other classes keep strain as damage.
A creature on the floor is the time to hold for a grand finisher.

### C7 · The Clash — meeting the swing

Monster Hunter's offset attack. **A full charge, a hammer third hit, or the last frames of a
Rush, whose live frames meet a creature's live attack on the part that is swinging**, cancel that
attack: the creature is interrupted (the one thing besides a topple that may already cancel a
live hitbox), both freeze for a long beat, the sound is the loudest the game makes, and the
Champion takes nothing.

**On fighters too** (decided). Two live attacks meeting, at least one of them a Champion's
clashing move: **the heavier wins** and the lighter is cancelled into a short stagger — heavier
meaning the longer impact freeze, the number that already says how much a weapon weighs.
**Equal, both recoil**: two Champions' full charges meeting bounce both of them apart, neutral
again, with the biggest freeze in the game. A Clash is not a block, so it moves no on-block
number; it is a trade the heavier blow wins outright, which is the fantasy and the thing to watch
in tuning — a Clash too easy to land is a parry with no risk.

It is the outplay the class is missing against a creature: the alternative to dodging the
scythe is **hitting the scythe**, and a held charge aimed at where the scythe will be is exactly
how a player does it on purpose.

## The loop

> **Rush in** — braced, the shoulder is hit one → **two links on the beat** → **a third hit** —
> refunds the Rush, banks momentum → **Rush out, through, or back in** → again. Momentum climbs
> across loops until a tap is a full charge. When they will not let you in, **hold**: a full
> charge is a wall, a feint, or a hit that turns the fight. Against a creature every loop fills
> its break; when it falls, hold for the grand finisher; when it swings at you, **Clash**.

About one press every third of a second in a string, fewer in neutral; the skill has moved from
*which button* to *when*, and to *how long*.

## Suggested order to build

1. **C1, Rush.** Speed and refund are knobs; the shoulder and the bracing are small. Play it.
2. **C2, the Beat.** Small, and it changes every hit the class lands.
3. **C3, the Hold** — the timing alone, with part charges only: the wind-up held, the grace
   window waiting, Rush cancelling it. Play it before any full charge exists.
4. **C5, the full charges**, one weapon at a time, each with its clip, trail and sound. The
   hammer first (Shockwave, Meteor — the ring and the leap are the closest to things built), the
   spear (Comet, Heaven-piercer), the sword last (the Crescent and the delayed cuts are the
   newest machinery).
5. **C4, Momentum**, the meter and the glow, once there are full charges for it to buy.
6. **C6, Break.**
7. **C7, the Clash**, creatures and fighters.

Replaying the owner's Mantis tape after each step is the measurement: the third-hits-landed
column above is the number to move.

## Open questions

Answered: CHAMP-1 (both, in different spaces), CHAMP-2 (a meter and the glow), CHAMP-3 (Rush
braced in three bands), CHAMP-4 (no — and then superseded: a full charge is the grand finisher,
and momentum is one of its doors), CHAMP-5 (yes, heavier wins). CHAMP-7 (the wave as the only
new machinery) is answered by C5: every full charge is new.

- **CHAMP-6** — answered from play (2026-10-13): aerials hold, and aerials and takeoffs count in
  the string (ground, jump, third), with the third hit thrown from the air. See the fifth round.
- **CHAMP-8** — answered: the spear comes back at once; the rush to it is the throw's own second
  half.
- **CHAMP-9** — answered: no free full charges on a downed creature; a full charge on a third
  hit is a grand finisher, and an uncharged third hit is a third hit.
- **CHAMP-10** — answered: the six as proposed; build it.
- **CHAMP-11** (from the build). The full charges are movement too: Heaven-piercer's run after
  its spear is 25 m, the Comet 8 m (gone since the fifth round) and Thousand Cuts 7 m, and a string advances on the throw,
  whiffed or not. The lab search's whole-kit reach went from 55.8 to 85.3 m at lane 0, and back to 68.1 m without the Comet (the fifth round). Should
  they cross the gaps the movement sections were built against?

## The fifth round (2026-10-13, from play)

Asked: holds on the aerials (the sword's Crescent off a jump without losing speed, the hammer
held in the air to slam back down); the spear's first finisher as a sweep that repels, not a
dash ("you can just keep dashing everywhere while dealing damage"); aerials and jumping attacks
in the three-hit count, in rhythm, and a third hit from the air; fast clicks read as holds; the
Rush and the spear's dash going where the stick was rather than the mouse; and a third hit held
at a full meter draining it during the wind-up.

Built:

- **The string in the air.** An aerial or a takeoff is the next hit of the string, wherever the
  last ones were thrown; a jump does not end it; with two hits behind him an aerial is the third
  -- it ends the string, refunds the Rush, and held is the grand finisher. Throwing the ground's
  third links from the air was tried and reverted: the hunt bots struck a creature's back a
  quarter less often. (`moves::champion::in_string`, `Fury::place`.)
- **Aerials hold** to their weapon's finisher for that place. From the air the Crescent keeps
  the jump's speed, the Repel is thrown where he is, and the Shockwave and Meteor slam him down
  the crosshair (inside a cone round straight down) and break the ground where he lands.
- **The Repel** replaces the Comet: the spear swung flat a whole turn round him, little damage,
  a long shove.
- **A hold is one press, held**: one release and that hit is a tap. The bug was a second click
  arriving at the top of the wind-up; on the hammer one frame of it was a finisher.
- **The meter waits** while he winds up a hold.
- **The Rush goes toward the crosshair**, flat along the floor from his feet.
