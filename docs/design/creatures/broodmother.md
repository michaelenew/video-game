---
status: built
proposed: 2026-09-30
built: 2026-10-01
tier: 3
---

# Broodmother — a clock with legs

A spider the height of a house, carrying her young on her back in six sacs that
ripen while you watch. The first creature that makes more of the fight as it
goes, and the one the cast gives the lesson **target priority**: not "which
move do I answer" but "which of the three things in front of me am I hitting".

Part of the cast planned in [../bestiary.md](../bestiary.md); the body, the
glance, the scoring and the strain thresholds are the Ridgeback's, from
[../monsters.md](../monsters.md). Her brood are gnawers
([gnawers.md](gnawers.md)): the same critter, the same pack brain, attack tokens
and morale, with the Broodmother standing where the gnawers' leader stands.
Anything that doc says about how a gnawer bites, circles and takes turns is true
of a broodling, and this one does not repeat it.

---

## 1 · What the fight is

**Every second you spend on the little ones, she makes more; every second you
ignore them, they eat you.**

She is three things at once, and the fight is the order you hit them in:

- **the brood** — broodlings, knee-high gnawers, at most **eight** alive;
- **the sacs** — six, on her abdomen, each on a visible clock that ends in two
  more broodlings;
- **her** — 6000 health, eight legs, and every move in §2.

The body, first guesses to be printed by `beastcheck` rather than trusted:

| Part | Standing | Why that number |
| --- | --- | --- |
| Leg span | 9 m, eight legs, feet 4–4.5 m from her centre | wider than the Ridgeback's footprint is long, so "beside her" is somewhere with legs in it |
| Thorax | top at 3.2 m, head and fangs at 1.4 m | the front of her is at a fighter's height: that is where the melee threat is |
| Pedicel (waist) | 2.8 m, between thorax and abdomen | a weak point only when she is low (§4) |
| Abdomen | 5 m long, 4.4 m wide; underside at 2.2 m, crown at **6.0 m** | carried high on purpose: the sacs have to be above a hop, and the space under it is the tempting spot §3 covers |
| Sacs | 1.2 m across; fore pair at 4.6 m, mid pair at 5.2, aft pair at 5.7 | see below |
| Legs (breakable) | the shin, floor to 2.0 m, of each of the four middle legs | the only part of a leg a fighter on the floor can reach and not be stood on |

Against the jump range: full hops run 2.7 m (Bulwark) to 6.0 m (Dual mage),
with the Elementalist at 5.0 and the middle classes around four. **The rule
the geometry must satisfy is the fight's premise**, so it is written as a rule
and printed, not argued:

1. Standing, **no sac is inside a hop-and-swing for the Bulwark or the
   Champion.**
2. The **fore pair** is inside one at the apex of a full hop for the classes
   that hop four metres or more — a sixty-frame commitment in the air, beside
   her legs, which is a real price rather than a route.
3. The **aft pair** is for the Dual mage's float and for ranged shots only.
4. Through the **Sac slam's** recovery every living sac is inside every
   class's reach, standing: fore 1.4 m, mid 2.0 m, aft 2.5 m.

She walks at **6 m/s** — slower than a fighter's 7, because she is not the
thing that chases you; the brood are. She turns fast (eight legs do not need to
come about the way a quadruped does), which is why §5 slows her *decisions*
rather than her yaw.

### The sac clock

Each sac **swells for 30 seconds** (`sac_ripen`) through three colours:
pale (the first half), amber (to 85%), and **red and twitching** for the last
four and a half seconds (270 frames) — the burst's tell. Then it bursts: two
broodlings drop to the floor under it.

**A sac that bursts grows back; a sac that is popped does not.** The site of a
burst sac is empty for six seconds and then lays again, on the same clock. A
site whose sac was popped is a scar for the rest of the fight. That is the whole
design in one rule: ignoring the sacs is an infinite supply of brood, and
dealing with them is permanent. The clocks start staggered (0, 15, 30, 50, 65,
80% ripe) so a burst comes about every five seconds from a fresh mother, and
the player's first ten seconds show them a sac already going red.

**The cap holds the clock.** With eight brood alive, a red sac that reaches its
burst does not burst: it **holds**, red and pulsing faster, until there is
room. A mother covered in held sacs is a mother whose brood you are losing to,
and it is drawn that way on purpose.

A sac has 150 health — a three-hit Champion chain, two or three ranged pokes.
Popping one kills the brood inside, deals **300** to her and adds **300 to her
strain** (§4), which on its own is a burst.

### The loop

1. **Thin the brood.** Keep it to two or three. Never chase one: they come to
   you, and the one you walked to is the one you were not watching her from.
2. **Read the sacs.** The reddest is the priority. A held sac is a warning that
   you are losing the count.
3. **Wait for the screech.** It calls every broodling to her, which is the
   breather — and it is always followed by the slam.
4. **Take the slam's window.** Out from under the abdomen before it lands,
   back in as it crashes, pop what you can reach — while the brood that the
   screech just gathered round her come for you (Brood guard).
5. **Break legs when you can.** Two on one side and she lists, and that side's
   sacs sit lower for the rest of the fight.
6. **Pop the last sac.** She collapses for four seconds, the brood scatter, and
   what gets up is a faster, simpler animal with nothing left to protect.

## 2 · The moves

Nine rows: six are hers, one is the clock, two are the brood's. First guesses
for frames and damage, labelled as such; the Oven gets a knob for each.

| Move | Range | What it is | Tell (frames) | The answer |
| --- | --- | --- | --- | --- |
| **Leg stab** | close, all round | One leg rises past the knee and drives down onto a disc 0.9 m across, 1–4 m out from the body. In a **flurry** of up to three, 12 frames apart | 20 each, the raised leg | **Step off the lit disc.** Positional: do not stand at a foot, and do not dodge — a dodge along her flank rolls onto the next leg's disc |
| **Fang lunge** | close, in front | Thorax drops, fangs spread, the front of her lunges 5 m | 28, and the windup follows you | **Dodge through to her side at the snap.** Ninety frames of recovery after: the front is the one place a lunge leaves open |
| **Sac slam** | close, under and beside the abdomen | The abdomen rises, holds at the top, and crashes to the floor | 45, the hold at the top | **Get out from under the abdomen before it lands, then turn back in** — the crash is the fight's window (§4) |
| **Screech** | the whole cave | Thorax rears, fangs up, a scream. No damage. Every broodling breaks off and runs to her | 50 | **Stop fighting the brood and go to the edge of the abdomen's shadow** on the side of the reddest sac. The slam is next, every time |
| **Web shot** | mid to long, **behind her** | The abdomen tips up and the spinnerets fire a glob that lands where you will be; it leaves a sticky patch | 30, the abdomen tipping | **Put a solid between you and the spinnerets** — a pillar, a stone, a shield. Caught, **cut yourself free**: every attack you throw takes 20 frames off the root |
| **Web line** | long, across the cave | She fires a line to a wall anchor and reels herself across at 18 m/s, body first; two strands stay behind | 36, she turns her back to the anchor and a glint marks it | **Out of her lane as she crosses** — and afterwards, **crouch-walk or jump the strands**; running through one trips you |
| **Hatch** | under a red sac | The clock ending: the sac splits, two broodlings drop, a wet splash 1.5 m across | 270, the red twitching | **Pop it before it goes, or be elsewhere when it does** |
| **Brood guard** | pack, 8 m from her | When a sac is hit, every broodling within 8 m takes its attacker as its target, and the attacker's token cap rises by one for three seconds | the hit you just landed | **Clear the ring round her before you reach for a sac**, or pop it in one burst from range so there is no second hit to guard |
| **Broodling bite** | close, pack | A gnawer's bite, at the brood's numbers (see [gnawers.md](gnawers.md)) | 18, the crouch before the leap | **Keep them in front of you and count the tokens** — two may bite at once, four on a rooted target |

First-guess numbers:

| Move | Startup / active / recovery | Damage | Reach |
| --- | --- | --- | --- |
| Leg stab | 20 / 3 / 28 (flurry: +12 per extra leg) | 110 | disc 0.9 m |
| Fang lunge | 28 / 4 / 90 | 180 | 5 m, 2 m wide |
| Sac slam | 45 / 5 / 70, then 20 to lift | 220 | 3.5 m round the abdomen's centre |
| Screech | 50 / 40 / 10, then the slam | 0 | the cave |
| Web shot | 30 / flight at 20 m/s / 40 | 60 + root 90 | glob 0.8 m; patch 1.5 m, 8 s |
| Web line | 36 / reel (to 26 m) / 50 | 160, knocked down | lane 2.4 m wide |
| Hatch | 270 tell | 40 | 1.5 m |
| Broodling bite | 18 / 3 / 30 | 45 | per gnawers.md |

**Hits do a sixth to a fifth of a fighter**, a little under the Ridgeback's,
because here the damage comes from more directions at once.

### The subtle parts

**The web shot comes out of her back.** The spinnerets are at the tip of the
abdomen, so her ranged move is aimed *behind* her, and her melee moves in front.
That is what stops "get behind the big animal" from being the answer it was on
the Ridgeback: behind her is where the glob comes from. The windup is the
abdomen tipping toward you — a silhouette change you see from any angle — and the
lane is drawn on the floor from the spinnerets to the landing (§6).

**A patch is two things.** The glob's hit roots for 90 frames; the patch it
leaves is a P4 floor hazard that slows anything in it to a crouch walk (3 m/s)
for eight seconds and roots nobody. Stepping in a patch is a tax, not a
sentence. Fire burns a patch away at once — the Elementalist's pillar, a lit
stone, cinders — which is the one ground effect in the fight a class can clear.

**The root is a set-up, and she presses it.** A rooted fighter is the brood's
favourite target: its token cap goes from two to four, and she scores the fang
lunge and a second web shot higher against it (`combo_appetite`, as the
Ridgeback). Pinned by arithmetic in `a_web_root_outlasts_the_brood_arriving`:
ninety frames is enough for a broodling eight metres away to arrive and bite
once, not twice. Cutting free (each attack −20 frames) is the answer that is
not a dodge, and it is what makes a caught fighter still a player.

**The strands are the Ridgeback's tail sweep, laid flat and left behind.** Two
per crossing, 0.4 m off the floor, between the wall she left and the wall she
reached, for twenty seconds, at most four at once (the oldest goes). Crossing
one faster than a crouch walk — a run, a dodge, a Rush — trips you: 30 damage,
60 frames of stagger. A crouch walk steps over; a jump clears it. Any hit cuts
one. The answer is a *different* posture, not a different timing, which is
the point of having it.

**The screech always leads to the slam, and the slam does not always need the
screech.** The chain is declared in the move table (`follows: SCREECH`), so a
screech is a hundred-frame tell for the only window in the fight — long enough
that the player's job is positioning, not reaction. She also slams bare, on
the ordinary 45-frame tell, when somebody is standing under the abdomen: that is
what the slam is *for* on her side of the fight.

## 3 · A threat at every range

| Range | What reaches you | What it asks |
| --- | --- | --- |
| **Close, in front** | fang lunge, leg stabs on the fore legs | the side of her, never the face |
| **Close, beside** | leg stabs, the brood she has gathered | stand *between* feet, not at one; read the raised leg |
| **Close, under the abdomen** | the bare slam, stabs driven inward, hatch splash | don't |
| **Mid, 5–12 m** | the brood (they close faster than a walk), web shot | keep the count low and a solid near |
| **Long, 12 m and out** | web shot, web line, and every broodling she has not spent | there is no standing off: the brood come to a backpedaller, and the line crosses the cave in a second and a half |

**The tempting safe spot is under the abdomen.** It is 2.2 m of headroom, out of
every leg's disc, out of the fang lunge's cone, directly beneath six things you
want to pop — and from there a jumping swing reaches the underside, which is
soft. Three things cover it: the **bare slam** (she scores it enormous for a
fighter under her, and it is 220 with a 45-frame tell); the **inward stab**, the
middle legs driven in under the body; and the **hatch**, which drops brood on
whoever is beneath a red sac. The second tempting spot is **far away with a bow**,
and the brood and the web line are what cover it: she does not need to reach
you when the eight things she made can.

**The one comfortable place** is beside the abdomen between the third and fourth
leg, outside every disc and the lunge, at the edge of the slam — the Ridgeback's
"beside a hind leg". It is where the scripted hunter stands, and it is only
comfortable while the brood count is low.

## 4 · The approach and the window

**What you break, and what it changes.**

- **A sac popped** is gone for good: fewer brood forever, 300 off her, 300 of
  strain. Each pop also flinches her for 30 frames — and fires the Brood guard.
- **Two legs broken on one side** (any two of that side's four middle-leg shins,
  500 health each) and **she lists**. Permanently: that side sits 1.2 m lower,
  so its fore sac is at 3.4 m and inside every hop but the Bulwark's, standing.
  The slam comes down **lopsided**, onto that side first, which puts its sacs on
  the floor and the other side's at 2.5–3.0 m. She turns worse toward the list,
  and she cannot stab with a broken leg. This is the Ridgeback's two broken
  forefeet: a permanent route for most of the roster, bought with the ground game
  — and the Bulwark's route to the sacs outside the slam.

**The windows.**

| Window | Cause | Length | What it gives |
| --- | --- | --- | --- |
| Flinch | a sac popped, or a hit past `flinch_threshold` | 30 f | a second pop, if you are already there |
| Slam recovery | every slam | 70 f, then 20 lifting | every living sac inside every class's reach |
| Stumble | a leg breaking, or control while susceptible | 100 f | the thorax to 1.8 m and the pedicel exposed |
| **Collapse** | **the last sac popped** | **240 f** | she drops flat, pedicel at 1.2 m for ×2 damage; the brood **break** (they scatter to the walls for the whole collapse — the gnawers' morale rule, with her as the leader) |

**Strain works as it does on the Ridgeback**, with one addition: a sac popped is
300 strain on its own, so popping two in one slam window buys the interrupt
(`interrupt_strain`) and cancels whatever she starts next.

**The arc.**

1. **Fresh.** Six sacs, staggered clocks, a burst every five seconds, the brood
   the main threat. The player learns the count.
2. **Pressed.** Strain thresholds fall with her health (`strain_desperation`,
   unchanged). Sacs popped slow the clock down; sacs ignored speed the fight up.
3. **The clutch — desperation with sacs left.** Below **30% health with any sac
   still alive**, she ripens every living sac at once: they all go red over six
   seconds. It is the punishment for having ignored them, and it is a race you
   can see — pop them or meet a wave of up to twelve against a cap of eight,
   the rest held. A player who popped all six never sees it.
4. **Enraged — desperation without them.** Once all six are popped, the
   collapse, and then an animal with nothing to protect: no brood, no screech,
   no hatch. Walk 6 → 9 m/s, flurries of up to five stabs, fang lunge 28 → 22
   frames, web lines twice as often, `think_frames` two thirds. The slam stays,
   now a straight crash, and its recovery exposes the **pedicel** at 1.5 m —
   the enraged fight's window. **Losing the brood is her desperation**: the
   fight gets faster and simpler at once, which is the reward for having played
   the first half properly.

## 5 · The brain

The Ridgeback's control algorithm, whole: glance, lead, the scored draw with
`decisiveness`, variety, cooldowns, the windup that follows you, the hit that
does not. What differs:

- **She does not have to face you to threaten you.** Moves carry an arc like the
  Ridgeback's, but hers point both ways: the lunge forward, the web shot back,
  the stabs all round. She turns to *choose which end you get*, not to reach you.
- **She glances more often and decides less often.** `glance` 8 (the Ridgeback's
  is longer): eight eyes. `think_frames` longer than the Ridgeback's, because
  the brood fill the beat — a mother who threw moves at the Ridgeback's rate on
  top of eight biters would be unreadable.
- **She has two kinds of target.** Her own moves score against fighters. The
  screech and the clutch score against *her own situation*: how far her brood are
  from her, and how threatened her sacs are.

```text
U(m) =  range(m) + arc(m) + variety(m) + hurt(m)     as the Ridgeback
      + under(m)        enormous for the bare slam while anyone is under the abdomen
      + rooted(m)       combo_appetite: the lunge and a second web shot vs a rooted target
      + aloft(m)        air_appetite: the web shot vs an airborne target (it webs her down)
      + recall(m)       screech only: Σ brood distance from her / cap, + sac_threat
      ⟂ cooldown(m)     screech_lockout 600 f; the slam only from a screech or under(m)
```

`sac_threat` is the number of fighters within reach of a living sac, counting
the fore pair for anybody who can hop to it. It raises the screech's appetite,
which sounds backwards — the screech leads to the slam, which lowers the sacs —
and is the design: a mother whose sacs are being threatened calls her brood
home, and the brood arrive around the sacs just as they come within reach. The
window and its guard are the same event.

The **pack brain** is the gnawers', with two changes. She is the leader, so the
brood's morale does not break while she stands; it breaks for the collapse and
never recovers after it, because there is no brood after it. And the token cap
per target is two, four on a rooted target, plus one for three seconds on
whoever the Brood guard has named.

Knobs, each a family in the Oven under the species (P1): `sac_ripen`,
`sac_regrow`, `brood_cap`, `brood_guard_radius`, `rooted_tokens`,
`screech_lockout`, `clutch_health`, `enrage_speed`, `strand_life`, `list_drop`.

## 6 · Reading it

**The tells, in silhouette.** Abdomen *up and held*: the slam. Abdomen *tipped
toward you*: the web shot. Thorax *rearing*: the screech. Thorax *dropping*: the
lunge. *Her back to a wall*: the web line. One leg *high*: a stab. Six
different outlines for six moves, which a spider's body does unusually well —
most of her poses change the shape against the cave's light, not just the pose
of a limb.

**On the floor**, from `Monster::telegraph` as the Ridgeback: every stab's
disc; the lunge's lane; the slam's footprint, growing through the hold; the web
shot's lane from spinnerets to landing, **and the landing disc**; the web line's
lane from her to the anchor. Two new ones: **a ring under each red sac** where
its brood will land, and **the strands**, which are thin and must be drawn with
a glint and a shadow line on the floor, because a tripwire nobody can see is the
unanswerable hit in its purest form.

**The sacs are the fight's HUD.** Colour and swell are the clock; a held sac
pulses; a popped one is a torn husk that stays. The sacs glow, and the cave's
light must not *depend* on them — a floor of ambient light that does not change
when all six are gone, or the enraged fight is fought in the dark.

**Sound.** Each sac has a heartbeat that quickens as it ripens, and a held sac's
is continuous. The screech is the loudest sound in the fight and it means one
thing. A glob fired from behind her is heard before it is seen.

**What "unanswerable" means here.** A broodling that bites from behind the camera
is answerable if its bite had its 18-frame tell and it held a token — the
gnawers' rules. A web shot from spinnerets off-screen is answerable only because
its lane and landing were drawn on the floor under your feet. A strand is
answerable only if it was drawn. The fight report counts each of these under
the move that did it, and the target stays zero.

## 7 · The classes

**Shadow Reaver — the fastest way into the window.** An eighteen-metre throw
and a dash means she can be beside the slam's footprint on the frame it lands
from anywhere in the cave, and a sac popped with a full tally on the mother is a
burst on top of a burst. Her standing sacs are the fore pair at the top of her
hop and no more. The hole is her copy: `aim::shadow_faces` turns the shadow's
swing to **the nearest body in reach**, and with eight broodlings about, that is
never the mother. Out on the field her shadow is a brood clearer, whether she
wants one or not. Identity, probably — it makes her shadow the thing that
thins the count while she goes for the sacs — but see §12.

**Elementalist — this is her fight.** Her auto is a skillshot, and a skillshot
at a sac standing is the fight's one free pop; stones are standable, and a stone
raised beside the abdomen is a step to the aft pair for anybody; fire burns the
web patches and the strands; Cinder spray's cloud is a brood clearer. Her long
startups are the risk: with four broodlings on her there is no time to raise
anything. The class the Ridgeback could not be won with should win here more
often than anyone, and the harness should say so.

**Blood mage — the brood feed her.** Every broodling she cuts spills a pool, and
the brood are the densest supply of pools in the cast: the thing that threatens
everyone else is her ammunition. The Reaping sweep is a ring clearer, the blink
to a pool by the slam footprint is her way into the window, and grey health means
a bite from behind is recoverable if she drinks fast. Hard on the sacs outside
the window, like everyone at her hop. Easy, by identity.

**Dual mage — floats to the sacs, and pays for it.** Her 6.0 m float reaches
every sac, standing, which is the Ridgeback's question again: she can decline
the window. The answer is on the creature's side rather than hers: an airborne
target raises the web shot's appetite (`aloft(m)`), and a glob that catches a
fighter in the air **webs her down** — she falls, and is rooted where she lands,
with the brood's token cap at four. Popping from the air is allowed and costs a
read. Her pull and push autos also herd broodlings, which the rest of the roster
cannot.

**Champion — the brood killer.** Whirl, the spear's third hit, is the only
volume in the game that threatens behind you and it clears the ring; Rush crosses
into the window and cancels a recovery to get out of it. The sacs are his
problem: standing, nothing is in his reach, and whether a Pole drive or an
Uppercut's rise reaches the fore pair is for `beastcheck` to print, not for this
paragraph to promise. He is the class that most needs the broken legs.

**Bulwark — the hardest, and the one with a job.** The lowest hop reaches no sac
standing, and the fore pair only after he has broken two legs on that side. But
a planted shield is a solid that stops a web glob and funnels the brood into one
facing, and his block is the one defence that handles two broodlings biting from
the same side. Solo, the sacs are a hole; in coop he holds the brood while
somebody else pops, which is identity. Whether his shield throw can pop a sac
depends on whether it travels a skillshot's line; if it does, it is his one
standing answer.

## 8 · Coop

- **The cap stays eight**; the clock runs at 0.8 of its solo length
  (`sac_ripen_coop`), because two players thin brood twice as fast.
- **Tokens are per target**, so two fighters face up to four biters, not two.
- **Brood guard names one attacker**, so the other is free for three seconds:
  one pops, one peels, and the fight is about the division of labour.
- **The screech pulls brood off both** at once — the one breather the pair
  share, and the one moment both should be at the abdomen's edge.
- **Web shot on one, lunge on the other** is her best coop play: she roots
  the one further away and turns her front on the nearer.
- **A stone for a partner.** The Elementalist raising a step beside the abdomen
  for a Champion is the cast's clearest example of a coop combo, and it should
  be tested for.

## 9 · Measuring it

**The scripted hunter's plan** — what a person learns in the first ten minutes:

1. Stand between her third and fourth leg, never at a foot, never under her.
2. Kill a broodling that comes to you; never walk to one. Hold the count at three
   or fewer.
3. Watch the sacs; know which is reddest.
4. On the screech, stop, go to the abdomen's edge on the reddest sac's side, wait
   out the crash, step in, pop the lowest-health sac first, step out before the
   lift.
5. After a web shot's windup, a pillar between you; caught, attack to cut free.
6. Strands: crouch.
7. Enraged: punish the slam's recovery at the pedicel and nothing else.

**The dilemma is measured with ablations.** The fight's sentence is a claim that
both extremes lose, so the harness plays **three plans** on the same seeds: the
balanced one above, **brood only** (never touch a sac or her until the brood are
gone), and **mother only** (ignore brood and sacs). Target: the balanced plan
wins clearly more than either; mother-only loses *fast* (the brood eat it); brood-
only loses *slow* (the clock never ends). If either ablation wins as often as the
balanced plan, the fight has no dilemma and the sentence is false.

**New report lines:**

| Line | Why | Target |
| --- | --- | --- |
| **Time on target**: brood / sacs / mother / nothing | the skill itself: what the hunter's attacks were aimed at, frame by frame | brood 30–45%, sacs 15–25%, mother the rest |
| **Damage on target**, same split | whether the time converted | sacs popped count, not sac damage |
| **Brood alive**: mean, peak, frames at cap | whether the count was held | mean 2–4, at cap under 10% |
| **Sacs**: popped / burst / held-frames | the clock's side of the fight | 4+ popped in a won fight |
| **Pops per slam window** | whether the window is usable | about one; two is a good player |
| **Guarded hits** | bites taken within three seconds of hitting a sac | present, not dominant |
| **Rooted, then**: bites and lunges on a rooted target | whether the set-up works | lands, and is not a death sentence |
| **Strand trips** | whether the strands are seen | low after the first minute |
| **Time to enrage**, and **clutches** | the arc | enrage in most wins; the clutch rarely in a win |

**The four windows** count the brood: the fight is *threatening* in a frame
when either she or any broodling holding a token could land before a reaction.
Targets as the Ridgeback's — about four tenths threatening, a fifth walk-up —
with the walk-up share mostly in slam recoveries and the collapse.

**Tier 3:** the balanced plan wins about one in three, won fights of 3–6 minutes
(her 6000, less 300 a pop, at the rate the window allows). **Unanswerable
hits: zero.**

## 10 · What it needs built

**Depends on:** P1 (species — her rig, legs, sacs, moves), P2 (the cave, with a
ceiling), P3 (critters and the pack brain — **built 2026-10-01**: a body that
brings a pack owns it, `OwnerTokens` shares the budget, `pack::spawn` fills a
freed slot, morale is optional; see [critters.md](../critters.md) §6), P4 (web patches),
P8 (the harness plan and the per-species report). It is the bestiary's seventh
build for that reason: nothing here is new machinery except what is below.

**New and hers:**

- **A monster that owns a pack.** The spawner: a sac bursting writes two
  critters into the pack's slots. The pack's leader slot points at a monster
  rather than a critter.
- **Sac sites** — six, each a clock, a health, a state (swelling, held, empty,
  scarred). A sac is a part box on the abdomen with a `weak_point` flag and a
  `destructible` flag; P1's move and part flags should carry it without a name.
- **Line hazards.** P4 is circles; a strand is a segment. A second small list,
  or a shape tag on P4's entries — P4's call. Trip rule reads speed, not a flag.
- **A move chain** (`follows`), declared in the move table, as `Move::aim()` is.
- **The list.** Per-side leg damage lowering that side of the rig; the
  Ridgeback's broken-foot pitch and roll, generalised to eight legs.
- **A ceiling.** The cave's vault is a solid the crosshair's ray meets (P2
  must allow a ceiling), and the camera must stay under it.

**Aiming.** No new line of effect. A ranged shot at a sac is a skillshot: the
camera ray passes *through* the sac (it is part of a body) to the vault behind
it, and the straight line from the caster to that point runs into the sac by
`aim::first_along`. The parallax between the camera's ray and the caster's line
is largest at the sac when the point behind it is far away, which is an argument
for the ceiling: a vault six metres above the sacs keeps the miss inside a sac's
radius at every range the arena allows. That is a claim, so it is a test —
`a_skillshot_on_the_crosshair_meets_the_sac_under_it` — and if it fails the fix
belongs in `aim.rs`, not beside the sac (see §12).

**Snapshot.** The monster ~200 B (more bones than the Ridgeback, same shape);
pack brain and eight critters ~280 B; six web patches ~48 B; six sac sites ~24 B;
four strands ~48 B; list and enrage state ~8 B. **About 610 B**, against the
bestiary's 550 — the strands are the difference. The snapshot is fine with it.

**Per-frame cost.** Dominated by the rig: eight legs of three bones each plus
thorax, abdomen, pedicel and head is about 30 bones and 30 part boxes against the
Ridgeback's 18, posed every frame and resolved against every fighter. Second,
eight critters against each other and the fighters (64 cheap pairs) and against
six patches and four segments. No per-frame ray casts beyond the fighters' own.
Pinned under the existing `budget.rs` with a Broodmother scene added.

**Tests** (`crates/sim/tests/broodmother.rs`):

- `no_sac_is_inside_a_hop_and_a_swing_for_the_bulwark_or_the_champion_while_she_stands`
- `the_slam_brings_every_living_sac_inside_every_classs_reach`
- `a_burst_sac_grows_back_and_a_popped_one_never_does`
- `the_brood_never_exceed_the_cap_and_a_ripe_sac_holds_while_it_is_full`
- `a_screech_is_always_followed_by_a_slam`
- `a_web_root_outlasts_the_brood_arriving`
- `a_strand_trips_a_run_and_not_a_crouch`
- `two_broken_legs_on_a_side_lower_that_sides_sacs_for_good`
- `popping_the_last_sac_collapses_her_and_she_never_spawns_again`
- `a_skillshot_on_the_crosshair_meets_the_sac_under_it`
- `what_is_drawn_under_a_red_sac_is_where_its_brood_land`
- `the_broodmother_fight_fits_the_snapshot_and_does_not_allocate`

**Milestones** (a few hours of AI time in all):

- **M1 · The body and the cave.** Species entry, eight-legged rig, sacs as weak
  parts, the cave with its vault. *Check:* `beastcheck --species broodmother`
  prints every height in §1 against every class's hop, and the two geometry
  tests pass.
- **M2 · The clock.** Sac sites, ripen, hold, hatch, pop, regrow, the cap; the
  brood as gnawers under her. *Check:* the clock, cap and pop tests; a hunt where
  nobody touches her ends with eight brood and six held sacs.
- **M3 · Her close moves.** Leg stab, fang lunge, the slam, the screech and the
  chain, each with its floor marker. *Check:* `SHOT_MOVE` screenshots of each
  marker; `a_screech_is_always_followed_by_a_slam`.
- **M4 · The web.** Web shot and patches (P4), web line and strands, the root and
  cutting free, fire burning web. *Check:* the root and strand tests.
- **M5 · The arc.** Brood guard, legs and the list, the clutch, the collapse,
  enrage; the brain's new terms. *Check:* the list and collapse tests; a scripted
  run reaches enrage.
- **M6 · Measured.** The three plans, the report lines of §9, twelve seeds each.
  *Check:* the numbers are in this document, and the balanced plan beats both
  ablations — or the document says why not.

## 11 · In the world

**The Hollows** ([../world.md](../world.md)), tier 3, off Pinewood.

**The arena** is a cavern about 30 × 26 m, oval, with a vault **12 m** at the
centre and 8 m at the walls. Not low: a third-person camera under a low ceiling
is pulled in over the shoulder, and a knee-high broodling beneath your own
character is already the hardest thing in the cast to see (bestiary §2, "size").
High enough for the camera and the Dual mage's float, low enough to be a
backdrop behind every sac — the aiming argument in §10. The walls are rock
that rises into the vault, with old web anchors she uses for the line. **Two
rock pillars**, two metres thick, stand at mid-cave: the solids the web shot is
answered with, and the one place a line cannot cross. A ring of **1.5 m shelves**
along the back wall gives the middle classes the fore pair from a shelf, if they
can fight her over to one. Light is a pale ambient from cracks in the vault;
the sacs glow on top of it.

**Trophy:** a sac husk, per [../world.md](../world.md).

**Sidegrade — the priority family.** *Shadow Reaver, mechanic modifier:*
**Chosen prey.** The shadow's copy turns to the last body *she* hit, rather than
the nearest body in reach — and its reach is a metre shorter. A playstyle change
(the shadow follows your choice of target instead of the crowd's), paid for with
a copy that misses more often, and exactly the thing this fight teaches her to
want. In versus, where there is one body to hit, it is almost only the cost, which
is how a sidegrade should look from the other mode.

## 12 · Open questions

1. **Should a sac be on the crosshair's ray?** Bodies are not, and the rule was
   paid for (CLAUDE.md, aiming). A sac is a target rather than a body, and it is
   small — the creature filling the screen was the problem, not a 1.2 m sac.
   If the parallax test fails, the choice is a declared `on_the_ray` flag for
   weak points in `aim.rs`, or a bigger sac. That is a designer's decision.
2. **Is the burst sac growing back too cruel for a first hunt?** It is the whole
   premise, but a new player who does not understand the colours meets an
   endless supply. The cap and the held sac are the safety; whether they read is
   a person's question.
3. **Does the Dual mage's float need more than the web-down?** Same question as
   the Ridgeback's tail, one fight on.
4. **Is the screech → slam chain too predictable?** A hundred frames' notice makes
   the window a positioning test. A person may find that it reads as a script.
   The knob is whether the bare slam also comes from `sac_threat`.
5. **Is the enraged fight easier or harder than the one before it?** It is meant
   to be faster and simpler, a victory lap with teeth. If it kills more hunters
   than the brood did, the speed-up is too much.
6. **Should the slam crush her own brood?** It would reward baiting it near them
   and would read as an animal, not a machine. It also makes the screech pull them
   to exactly where it will crush them, which is either funny or a bug.
7. **Is the Reaver's copy turning to broodlings identity or a hole?** The
   sidegrade in §11 is one answer; changing `aim::shadow_faces` to prefer the
   creature over critters is another, and it would be true of every pack fight.

## 13 · Where it landed

Built 2026-10-01: `--hunt broodmother`, the Hollows. The species is
`sim/src/species/broodmother/` (the table; `fight.rs` for the sac clocks, the
pops, the guard, the web, the strands and the arc; `mind.rs` for her own
scoring and the brood's pack mind over the gnawers'; `legs.rs` for the eight
planted feet, the stab's foot and the list), the arena
`sim/src/arena/broodmother.rs`, the clips `anim/src/beast/broodmother/`, the
look (sacs tinted on their clocks) and the cave's dressing in `game`, the plan
and its lines `hunt/src/plans/broodmother.rs` with its bin `hunt --bin brood`,
the reach probe `sim/src/reachcheck.rs`, and the rules pinned as sentences in
`sim/tests/broodmother.rs`. The plan is
[plans/broodmother.md](../plans/broodmother.md); the passes are in
[feel-log.md](../feel-log.md) of 2026-10-01.

Numbers from `cargo run -p hunt --bin brood -- --all --repeats 24 --balanced`
(the balanced plan; the scripted hunter plays §9 with a fifteen-frame
reaction):

```text
                won    mean    health left (a win)   threat / poke / way in / walk up   unanswerable   pops / slam windows
  Champion      7/24   143 s        308                  44 /  9 / 14 / 33 %                 2              30 / 113
  Bulwark       0/24     --          --                  47 /  9 / 13 / 31 %                 0               5 / 139
  Reaver        0/24     --          --                  47 /  9 / 13 / 31 %                 0              10 / 112
  Elementalist  1/24   142 s        310                  48 /  7 / 12 / 33 %                 1              81 / 117
  Blood mage    0/24     --          --                  48 /  8 / 13 / 31 %                 0               0 / 173
  Dual mage    10/24   152 s        201                  47 /  9 / 14 / 31 %                 3              48 / 182

  the three Champion plans, 24 each: balanced 7 won in 143 s; brood only 1 in 176 s; mother only 0
  (2026-10-01, every class played; before it every class but the Champion
   won 0 of 24 on every plan.) The other plans, by class, 24 each:
    Elementalist  mother only 22 in 83 s; brood only 0
    Dual mage     mother only  5 in 126 s; brood only 0
    the rest      0 on both
  coop, two Champions 12/12 in 66 s
```

**2026-10-01: the hunter plays all six classes** (`hunt::class`,
[bestiary.md](../bestiary.md) §8). **The Dual mage** wins 10 of 24 on the
balanced plan, more than the Champion: her autos clear broodlings with both
hands and her bars climb on them, and she pops a sac in a quarter of the slam
windows (48 of 182). **The Elementalist** pops more sacs than anybody (81 in
117 windows, her bolts at them from the edge of the footprint) and wins one
balanced -- but **22 of 24 mother only**, in 83 s: a pillar under the mother,
with the brood left to bite. For her the fight's sentence does not hold, and
§1's dilemma is the creature's to fix (a mother who leaves fire, or brood that
come for whoever burns her) or a person's to accept. **The Reaver, the
Bulwark and the Blood mage** still win nothing on any plan.

**Against the targets.** Balanced wins about a third for the Champion, and
beats both one-sided plans by a distance -- the fight is the division of
attention it was designed as. Threatening 44 %, as asked. Short of the target:
won fights last two and a half minutes, not three to six; the walk-up share is
a third, not a fifth (her stride is long and the hunter keeps a post beside a
leg, §3); pops are a quarter per slam window, not one. **Unanswerable hits are
not zero**: one or two in 24 for four classes, every one a broodling's
hamstring or dart whose crouch began off the hunter's screen while the lane it
draws had been under them for 7 to 14 frames, short of the report's fifteen
(the gnawers' own rule, as the Gnawers have it). The rest of the roster lost
until 2026-10-01, when the hunter did not play their classes; it does now
(above), and the Dual mage wins, the Elementalist wins by ignoring the brood,
and the Reaver, the Bulwark and the Blood mage do not. Not yet played: the
Elementalist's free pop from a stone, and the Reaver's throw into the window.

**What changed from the sections above.**

- **A sac's health is 250 at x2** (`SacHealth`, `VulnSac`), which a Champion
  tears with two hits of her chain, where §1 asked 150 and a three-hit chain:
  three never fitted the window with the walk in. Her hide is x0.45 and her
  legs x0.7 (`VulnHide`, `VulnLeg`) for a longer fight.
- **A pop in the slam does not flinch her out of it**: the window runs its
  length, and the strain past the bar is kept for the next move she starts
  (§4's "two in one window buys the interrupt").
- **No broodling starts a windup while she lies in the slam**, as through the
  screech: the fighter in the window has the camera on a sac, and a crouch
  begun then was a bite from off the screen. The guard's bill is paid as she
  lifts. The guard no longer retargets a broodling already committed.
- **With her dead, the brood die with her**, so the hunt ends on the mother.
- **§12's first question is answered by a test, and `aim.rs` is unchanged**:
  `a_skillshot_on_the_crosshair_meets_the_sac_under_it`. Bodies stay off the
  ray; the ray finds what is behind the sac, and the straight line from the
  caster to that point still meets the sac -- with the vault behind it or
  without. Question 6 (should the slam crush her brood) is not built: it does
  not.
- **Signs, not marks**, for everything on her floor -- the glob's lane, the
  web line's lane and the ring at its anchor, the slam's footprint drawn faint
  through the screech, a red sac's landing ring, the strands -- so the report's
  "marked" rule counts them. Marks were not folded into signs; that is a
  change to every creature and was not cheap.
- **Generic seams it added**, each off for every other creature:
  `MoveDecl::then` (the screech's declared chain to the slam, run by the shared
  tick), `MoveDecl::unanimated` (the gnawer moves at the head of her table,
  which her body never plays), `FightDecl::repose` (a last word on the pose:
  her planted feet, the stabbing foot, the list), and in `game` a species
  `Tint` beside the look (parts painted from the fight's state).
- **Measured, not argued**: `beastcheck --species broodmother` prints each sac's
  and shin's lowest corner standing and in the move that brings it lowest; the
  reach probe (`sim::reachcheck`) is what §1's three rules are tested with.
