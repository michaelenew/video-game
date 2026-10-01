---
status: built
proposed: 2026-09-30
built: 2026-10-01
tier: 2
---

# Mireback — the floor is the clock

A toad the size of a barn: seven metres to the top of its back, twelve across,
and a walk of two metres a second. It is the slowest thing in the cast and the
first creature that changes the arena rather than standing in it. Everything it
does leaves tar on the floor, and tar is where every other move it has becomes
hard to answer. Fire takes the tar back, and fire is also the only way up it.

Part of the cast planned in [../bestiary.md](../bestiary.md), and the creature
that brings **P4, floor hazards**. The body, the glance, the scoring, the strain
thresholds and the telegraph markers are the Ridgeback's, from
[../monsters.md](../monsters.md), and this document only says where it differs.

---

## 1 · What the fight is

**It does not chase you; it takes the floor away.**

Left alone, the arena is about three fifths tar by the fourth minute, and a
fighter in tar walks at 2.8 m/s and dodges half as far — slower than the moves
it has to get out of. Burning the tar clears the floor, hurts whoever is
standing in it (the Mireback most of all, because it likes to stand in it), and
leaves **slag**: hardened mounds that are the steps up its back. So fire is the
floor, the damage and the climb at once, and every one of those three cuts both
ways.

The body, first guesses for `beastcheck` to print rather than numbers to trust:

| Part | Standing | Why that number |
| --- | --- | --- |
| Footprint | 11 m nose to rump, 12 m across, a squat dome | wider than long, so "beside it" is a long walk round, and its flank is the tempting place (§3) |
| Rim of the back | **5.2 m** — where the flank rounds over into the dome | above every standing hop but the Dual mage's 6.0; inside a hop from one slag mound for everybody but the Bulwark |
| Crown | **7.0 m**, four warts along it | the weak points on top; a walk of about four metres up the dome from the rim |
| Throat sac | 1.2–2.6 m under the chin, hidden until it inflates or belches | the weak point on the ground, in front, and only when it chooses to show it |
| Walk | **2 m/s**, turning at most 50°/s | slower than a fighter wading in tar (2.8), so it never runs you down; it relocates by leaping (the Belly flop) |
| Health | 11 000, first guess | a tier-2 fight is two to four minutes; about four tenths of that should come from fire |

Its **coat** is tar. While the coat is whole, hits on the hide do **×0.6**;
burning cracks it and hits do full damage; the Wallow puts it back. The coat is
one number in the snapshot, drawn as the sheen on its skin.

The jump table this is designed against (full hops from the floor, from
[../feel-log.md](../feel-log.md) 2026-09-17): Bulwark 2.7, Champion 4.0, Blood
mage 4.0, Elementalist 5.0, Reaver 5.1, Dual mage 6.0. The routes to the rim, as
`beastcheck` should print them:

```text
the rim, standing                    5.2 m   a standing jump for Dual mage
  from one slag mound (1.5 m)                 every class but the Bulwark (4.2)
  from two-layer slag (3.0 m)                 every class
  from a loaded planted shield (2.8 m)        Bulwark (5.5)
the rim, pressed flat after a flop   4.6 m   standing jump for Elementalist, Reaver, Dual mage
the rim, winded (sac burst)          3.8 m   standing jump for all but the Bulwark; from slag, her too
the crown's warts                    7.0 m   a walk up the dome from the rim
```

So slag is the route everybody can **build**, and the two sags are routes a few
classes can **earn** in a window. The Elementalist's structure jump and the Dual
mage's wings go anywhere, as on the Ridgeback, and §7 says what that costs them.

The loop:

1. **Hold clean floor.** Stand on it, fight from it, and walk out of the spew's
   marker when it lands on you. Every tar pool is a place a later move becomes
   unanswerable in.
2. **Bring fire to its tar.** Kick a brazier into tar that runs to the toad, or
   make it belch while it is standing in tar, or — on the Elementalist — just
   light it. Tar that touches tar carries fire, so a floor that has filled up
   is also a fuse.
3. **Burn it.** Fire under its body is the biggest damage in the fight and it
   cracks the coat.
4. **Climb the slag.** The burnt tar leaves mounds, often in a ring round where
   it was sitting. Get up, walk to the crown, and burst a wart. Each burst wart
   takes a quarter of its tar for the rest of the fight.
5. **Gut it.** When the coat is cracked it wants to Wallow — roll on its back in
   its own tar to recoat. Set that tar alight while it is belly-up and it is
   **gutted**: the big window.

## 2 · The moves

First guesses, labelled as such. Frames are startup / active / recovery.

| Move | Range | What it is | Tell (frames) | The answer |
| --- | --- | --- | --- | --- |
| **Spew** | long, 8–24 m | Cheeks fill, head tips back, and a glob of tar is lobbed at where you will be. **110** on the glob, and a 3 m pool where it lands | 36 / 3 / 40, the landing marked from the commit | **Turn.** It led you, so the marker is where you were going: change direction and walk out of it. Seven metres a second clears it from anywhere; 2.8 in tar does not |
| **Belly flop** | mid, leaps 4–14 m | A long crouch, a leap, and a crash. **200** within 6 m of where it lands, and a ring of four tar pools at the crash's edge | 50 / 4 / 100 | **Dodge the crash frame.** From the middle of the marker you cannot walk six metres in fifty frames, and from tar you cannot walk two. Then cross the ring for the underside while it heaves itself up |
| **Tongue** | mid, 4–16 m, ±70° ahead | Throat pulses, jaw drops, and the tongue goes out along a line at 80 m/s. Caught, you are pulled in and **swallowed** | 22 / to 12 / 60 on a miss | **Put something solid on the line** — slag, a stone, a planted shield — or cross it; walking away along it keeps you in it. Or let it happen: see Swallowed |
| **Swallowed** | inside | Two seconds in its stomach. Acid does **20 every 10 frames**; your hits on the stomach wall do **×2.5**; then you are spat out for **60** and a knockdown, into a fresh pool | 120 inside, or less (below) | **Hit the stomach.** The one move whose answer is damage. It retches you out early once it has taken 900 |
| **Flint belch** | 10 m around its mouth | The throat glows and swells; a spark in its gullet; **every tar pool within 10 m ignites at once**, and a 3 m cone of flame in front does **120** | 60 / 8 / 50 | **Get off the tar.** Or hit the throat sac hard enough to interrupt it, and it goes off in its own mouth (§4) |
| **Backwash** | behind, a 90° cone to 6 m | The hind feet churn and throw a low sheet of tar backwards. No damage: you are **tarred** for 3 s wherever you stand, and a pool is left behind it | 18 / 10 / 30 | **Jump it.** The sheet is shin-high. Only ever thrown at somebody behind, which is where you know it can come from |
| **Inflate** | close, within 1.5 m of its hide | It swells to half again its girth; a shove of 8 m/s outward and **90** to anyone touching it; then it stays puffed for 90 frames with the throat sac out | 24 / 6 / 90 held + 30 | **Stop your string and back off** during the swell. Thrown when it has taken a burst from close by, so the tell is your own greed. Then hit the sac. Aboard: **brace** |
| **Wallow** | on itself | It rolls over onto its back in tar and grinds, legs flailing (**60** within a metre), and the coat comes back over 150 frames | 30 / 150 / 40 | **Light it, or break it.** Past `wallow_interrupt` it flips back upright, staggered, with the coat half made. Lit, it is gutted. Aboard: **leave** |

Eight rows and eight answers: turn, dodge the crash, block the line, hit from
inside, leave the tar, jump, back off, and burn it.

**Tar**, on a fighter standing in it:

- walk ×0.4 — 2.8 m/s, still faster than the toad, so it never runs you down;
- **dodge distance ×0.5, invulnerable frames unchanged.** The ten frames are the
  answer to *when*; the distance is the answer to *where*. Tar takes only the
  second, so a dodge in tar still survives a hit and no longer gets you out of a
  crash circle;
- jump takeoff ×0.85, so every apex is about seven tenths — enough to leave, not
  enough to climb from;
- a 20-frame tail on leaving, on the `slowed` mechanism that already exists, so
  the slow does not flicker at the pool's edge. **Tarred** (from Backwash, or
  being spat out) is the same slow for 180 frames wherever you stand.

**Fire.** A pool that catches **burns for 4 s** and burns everyone standing in
it, fighter or toad, at **12 every 10 frames** on a fighter. Flame stands 2.5 m
high, so a body above that is not in it. **Fire runs.** A burning pool lights
every tar pool touching it after **20 frames**, so a joined-up field burns
outward at a rate you can watch and outrun. A burnt pool leaves **one slag
mound** at its centre — a solid cylinder a metre in radius and 1.5 m tall — and
the rest of its disc is clean floor. Tar that lands on slag coats it; burn that
and the mound is **3 m**, which is as tall as slag goes.

**The Mireback in its own fire** takes **30 every 10 frames per burning pool
under its footprint, up to three pools** — up to 540 a second — and its coat
loses a quarter a second while it burns. That is the fight's big damage, and the
reason the ring a Belly flop leaves matters: its inner edge is half a metre
inside the toad's footprint. Every flop sits it in a ring of its own fuel.

**Swallowed**, in full. The tongue's grab pulls you to the mouth over 12 frames
and you are then **inside**: a hollow part of the rig, a stomach 2.4 m across,
on the Ridgeback's ride rules — your position in the part's frame is the
authoritative one, which is why the toad turning does not throw you round it.
You cannot dodge or jump. You can attack, and the walls are the stomach, a weak
point with no coat. It ends after 120 frames, or on the frame the stomach has
taken 900, whichever comes first, and it spits you 8 m along its facing. **You
can choose it**: the tongue is 22 frames and a decent player dodges it, so not
dodging is a choice, and it is priced — about 300 of your 1000 against at most
900 of its 11 000, and after a swallow the tongue is locked for 900 frames
(`tongue_full`). Two exceptions make it a read rather than a trade:

- **A Bulwark blocking the tongue gives it the shield instead.** It chokes — a
  60-frame gag, throat sac out — and spits the shield out planted where it lands.
- **A fighter pulled out of burning tar** is a hot mouthful. It retches at 20
  frames, takes 400, and is **winded** (§4). The fighter took the burn to get
  there. It will not throw the tongue at somebody it *saw* standing in fire, so
  this is done by stepping into the fire after the tell begins.

## 3 · A threat at every range

| Range | What it does to you | What that asks |
| --- | --- | --- |
| **Close** (touching to 4 m) | Inflate if you have been hitting it; the Wallow's flailing; Backwash if you are behind | stop a string before the third hit; jump from behind |
| **Mid** (4–16 m) | The Tongue ahead of it, the Belly flop anywhere it can turn to | keep a solid on the tongue's line; dodge the crash |
| **Long** (8–24 m) | The Spew, and the flop's 14 m leap closes the gap in one move | turn out of the marker; do not stand still at range |
| **Aboard** | Inflate throws unbraced riders; the Wallow and the flop's crash throw everybody | brace for the swell; leave before the roll |

**Nothing it throws can reach its own back**, as on the Ridgeback: the glob
leaves the mouth forward and lands at least eight metres off, the tongue is
aimed at the floor ahead, and the belch ignites floor. What riding costs is the
buck, and the coat: an uncracked back is tar, and a rider on it walks at 2.8.

**The tempting safe spot is its flank**, seven or eight metres out. Outside the
Inflate, outside the tongue's cone, inside the spew's minimum range, off the
Backwash's cone, and the toad turns at fifty degrees a second, so it takes the
better part of two seconds to come round. What covers it is **the floor**. The
flop's ring and the wallow lay tar at its sides, the Spew it throws as it comes
round lands there, and the Flint belch is scored on exactly one thing — fighters
on tar within ten metres. The flank is safe until you are standing in tar, and
standing beside the Mireback for long is how you end up in tar.

**Under its chin**, inside the tongue's four metres and the spew's eight, is the
other one, and the Inflate and the belch's cone both cover it.

## 4 · The approach and the window

**What you break, for the rest of the fight: the warts.** Four along the crown,
600 each, ×2 damage. The warts are its tar glands, and each one burst takes a
**quarter of its tar** for good — spew pools a quarter smaller, the flop ring
one pool fewer, the Wallow's coat slower to come back. You can see it: a burst
wart weeps black down the flank and stops. With all four gone it spews no pools
at all. This is contract item 6, and it is the one that fights the clock: a
player who climbs early slows the floor for the rest of the hunt.

**What you earn on the ground: the throat sac**, ×2.5, out only when it shows
it, and it answers differently to the two moves that show it:

- **During Inflate** it fills a poise pool (`sac_poise`). Full, the sac tears
  and the toad is **winded**: it deflates, sags, and the rim comes down to 3.8 m
  for 110 frames — the one route up that needs no slag for most of the roster.
- **During the Flint belch**, a hit that crosses `interrupt_strain` makes it
  **backfire**: the spark goes off in its own mouth, only the tar within **4 m**
  of its chin ignites — which is the tar it is standing in, if there is any —
  and it is staggered for 60 frames. You chose which tar burned.

**The big window: gutted.** Set its tar alight while it is wallowing. It is on
its back, soaked, belly up, and the fire takes it: **240 frames** in which it
flails without threat, the belly is soft (×2, no coat), and its own burn ticks
double. It rolls upright at the end with the coat gone and — because every pool
it was lying in has just burned — **ringed by slag**. So the window also builds
the staircase, and the next thing to do is obvious.

The route to it is the whole fight. Burn it once to crack the coat; that makes
the Wallow the move it wants most; the Wallow needs two pools under it, so it
walks or flops into its tar; and fire has to be *ready* — a brazier with tar
joined to the toad, a pillar, or a belch you can backfire — at that moment. A
Wallow broken by damage instead (`wallow_interrupt`, six tenths of
`interrupt_strain`, because interrupting it is the point of the move) is the
consolation: a 60-frame stagger and half a coat.

**Crowd control** is the Ridgeback's negotiation. It cannot be knocked up — it
is already on the floor — so a knock-up landing while it is susceptible is a
flinch. A root holds it, which on a creature that walks two metres a second
matters mostly for the flop: a rooted toad cannot leap away from a fire.

**The arc.**

- **Fresh.** Coat whole, floor clean, thresholds high. It spews and flops and
  the floor fills; the first fire is a race to a brazier with a fuse long
  enough to reach.
- **Worn.** A wart or two gone, the floor half tar and half slag, and the fight
  is played on the mounds. Thresholds fall with health (`strain_desperation`),
  so a Wallow is easier to break and the belch easier to backfire.
- **Desperate, under three tenths: the tide.** Each Spew throws two globs, and
  it belches whenever anybody is on tar near it, **whether or not it is standing
  in tar itself**. It stops protecting itself, so the end of the fight is a toad
  setting fire to its own floor — frantic, dangerous, and generous.

## 5 · The brain

The Ridgeback's glance, lead, scoring and weighted draw, with four things taken
out and five put in.

**Out:** the gallop, `pursuit_gain` and `closing_appetite`. It does not chase —
that is the one-liner. It walks toward the middle of its own tar (the centroid of
tar pools within 12 m of its target), and relocates by flopping. `rear_pause`
goes too: it turns so slowly that the pause is its turn rate.

**Changed:** `glance` longer (about 30 frames against the Ridgeback's), because
it is a patient animal; `lead` longer, because the Spew and the flop are both
lobbed at where you will be, and the lead *is* those two moves. The skill it
rewards is the Ridgeback's — change direction between glances — with a floor
that makes changing direction slow.

**The set-up is tar.** `combo_appetite` reads a target standing in tar or
tarred as a target that cannot act, and it can: it just cannot leave. Arithmetic
for `a_set_up_lasts_long_enough_for_what_follows_it`: tarred is 180 frames,
which outlasts the Backwash's recovery (30), the beat, and a flop's 50-frame
tell. That is the Ridgeback's sweep-and-bite, in tar.

```text
U(m) =  range(m) + arc(m) + rider(m) + variety(m) + hurt(m)
      + floor(m)   tar-laying moves, by the clean floor within 4 m of the lead point
      + kindle(m)  the belch, by targets on tar within 10 m, less a large
                   penalty for tar under its own footprint (dropped in the tide)
      + coat(m)    the wallow, by (1 − coat) while two or more pools are under it
      + crowd(m)   the inflate, by strain it took from within 4 m in the last second
      + flee(m)    the flop, aimed away, by fire within 6 m of the tar it stands in
      ⟂ cooldown(m)
```

`flee` is the thing that makes the fire a race: it notices burning ground next
to its tar and leaps. It notices it **by glance** — fire is a position it
samples, not an event it hears — so a brazier kicked in the frames after a glance
has up to thirty frames of fuse before the toad knows.

Knobs, as Oven families under the species (P1): `mire_walk`, `mire_turn`,
`tar_slow`, `tar_dodge`, `tar_jump`, `burn_fighter`, `burn_self`,
`burn_self_pools`, `burn_life`, `fire_spread`, `slag_height`, `slag_cap`,
`coat_armour`, `coat_crack`, `wart_health`, `sac_poise`, `wallow_interrupt`,
`gutted_frames`, `swallow_frames`, `swallow_retch`, `tongue_full`,
`brazier_relight`, `tide_health`.

## 6 · Reading it

**The floor is the hazard list, drawn.** Every tar pool, every burning pool and
every slag mound is one entry in `World::hazards` — kind, centre, radius, age,
layer — and the renderer draws the floor decals **only** from that list, at the
radius the hit test reads. The slow, the burn, the fire's spread and slag's
collision all ask the same entries (`hazard::under`, `hazard::solids`). A pool
that looks a metre wider than it slows is the Ridgeback's six-metre bite drawn as
a one-metre head, and `what_is_drawn_on_the_floor_is_the_hazard_list` pins the
view to the list. The debug overlay rings each entry at its radius.

Three looks for three states, all readable from across the arena at a glance:
**tar** is flat, black, with a slow sheen; **burning** is the same disc under
flame 2.5 m tall, with the flame thinning as its four seconds run down; **slag**
is a grey-orange crust mound, cooling to grey. Fire's spread is visible as a
front crossing from one pool into the next, twenty frames a step, which is what
lets a player outrun it.

**Every move marks the floor**, from `Monster::telegraph` as usual: the Spew's
landing circle, the flop's crash circle **and the ring of pools it will leave**,
the tongue's line (stopping at the first solid, because the line is the hit
test's own answer), the Backwash's cone. The **Flint belch marks every pool it
will ignite** — they glow from the inside through its sixty frames — so "get
off the tar" is a question with the answer drawn on it.

**Silhouette.** A squat dome that changes shape a lot for each move, the house
style: the cheeks fill for the Spew; it sinks to the floor for the flop and the
floor ripples; the jaw drops for the tongue; the throat lights orange and grows
for the belch; the body rounds out by half for the Inflate; the hind end lifts
for the Backwash. **Sound**: a croak with the tongue's tell, a wet gulp for the
spew, a rising drone for the belch, a slap for the flop.

**What "unanswerable" means here.** The report's rule — below reaction and with
no positional warning, from where the creature stood when it committed — needs
one addition: tar. A crash that could not be walked out of because you were in
tar *at the commit* was answerable: you chose the tar, and the dodge still
worked. A crash that could not be escaped because a Spew landed tar under you
*after* the flop committed is a combo that was not on the floor when you had to
decide, and counts as unanswerable. The report measures escape from the tar
state at the commit frame.

## 7 · The classes

**Elementalist — easy, and it is identity.** Her fire lights tar: pillar, Cinder
spray, embers, lit shots, the ring of fire off a Downdraft into a burning patch
(which lights six metres of tar at once), and a lit stone. A stone raised
through a tar pool comes up **tarred**, and catches when the fire reaches it,
so her stones in a mire are fuses. Her structure jump goes anywhere, the crown
included. She decides when the floor is cleared, which is what the fight is
about, and the Ridgeback's open line "the Elementalist cannot win" gets the
creature it deserved. The cost is that she carries fire into her own footing
and her pillar is one at a time. If she wins nine in ten, that is tier 1 for
her, and §12 asks whether that is fine.

**Dual mage — tar barely touches her, and that is identity.** Tar is a floor
thing and she is the class least on the floor. Her first hop out of a pool is
seven tenths of her 6.0, and the second jump, the slow fall and the wings are
air and untouched. Flame is 2.5 m tall, so above it she is out of it. Her
single hop reaches the standing rim. Her hole is the other half: **she has no
fire.** She can stay out of the clock and cannot turn it back, so solo she must
play the braziers and the backfire harder than anybody. Her pull and push move
the fight, not the toad.

**Blood mage — the tar escape.** Her blink is not a dodge: it goes to a pool,
not a distance, so tar does not shorten it, and the Mireback bleeds pools under
the struck part's floor projection, usually beside it and often in tar. Blood
and tar are separate lists and do not mix: a blood pool stands on tar, is drunk
the same, and does not burn. So she moves through the mire by its own blood, and
blinking into a pool in tar lands her in tar — a door into the thing she was
leaving. Her 4.0 hop needs a slag mound. Grasp's root on a susceptible toad
stops a flop away from a fire, which is her one lever on the burn. No fire:
braziers and the backfire.

**Bulwark — hard, and it is where the Bulwark question gets an answer.** Tar
takes her 2.7 hop to under 2. But her shield **blocks the Spew** — the glob's
damage is stored as weight like any blocked hit, and the pool lands at the
shield's front edge, a metre ahead of her rather than under her — and a loaded
shield planted beside the toad is 2.8 m of standable wall, from which her hop
reaches the rim. Two-layer slag reaches it too. So she climbs on weight she
blocked off the creature, which is her kit working. Blocking the tongue gives it
the shield (§2), which is her version of choosing to be swallowed. No fire.

**Champion — the hardest, and it is the fight's test of him.** No fire, a 4.0
hop, and his way into the air — the pole vault off a Rush — **needs a runway.**
A Rush started in tar goes half as far, as a dodge does, and a vault off a half
Rush is a hop. So the Champion is the class that most needs clean floor near
the toad, and has nothing of his own to clear it with. Kick braziers, backfire
the belch, and vault from the floor you burned. His three-hit chains are the
string the Inflate punishes, so the flank station is where he learns to stop at
two.

**Shadow Reaver — middling.** The dash to the shadow is travel, not a dodge, so
it crosses tar at full length and is her way out of a crash circle; the shadow
cannot go on the toad (bodies are not on the ray) but it can stand on slag,
which makes a mound a place she can arrive at. Her 5.1 hop reaches the rim from
one mound and the flop's pressed-flat rim from the floor. Her shadow's copied
swings on the flank from out of the Inflate's reach are a real tool. No fire.

## 8 · Coop

Two hunters halve the time to kill and do not halve the clock, so the tar is
laid half again as fast with two (`tar_coop`), or the floor stops mattering.
The flop and the belch pick the point that catches most bodies, so standing
together on tar is punished twice. What changes for the better:

- **A swallowed partner can be cut out.** Hits to the belly from outside while
  somebody is inside count toward the retch, so two players cut the swallow
  short. This also caps the swallow as a strategy: two players alternating
  swallows is the degenerate line, and `tongue_full` (15 s) is what stops it.
- **One climbs, one kindles.** The natural split: somebody holds the brazier
  side and times the fuse, somebody waits on a mound. The Wallow is gutted by the
  one who is not on its back.
- **The Dual mage and the Elementalist** is the easy pair (the climber and the
  fire), and **the Champion and the Bulwark** the hard one.

## 9 · Measuring it

**The scripted hunter's plan** — what a person learns in the first ten minutes:

1. Stay on clean floor. Stand at the flank, seven or eight metres out.
2. Walk out of every Spew marker, across the way you were going.
3. On a flop: dodge the crash frame, then cross the ring to hit the underside
   from the front for the recovery.
4. When the toad stands in tar that is joined to a brazier, go and kick the
   brazier. Get off the fuse's path first.
5. On the belch: leave the tar; if the sac is inside a poke, hit it.
6. On a Wallow: light it if a fuse is ready, otherwise hit it.
7. When a mound beside it is inside your hop to the rim, climb, walk to the
   crown and hit a wart. Brace on the swell; leave on a roll or a crouch.
8. Never be swallowed. A second plan, `swallow_greed`, lets the tongue land
   below a quarter of the toad's health, so the report can say whether the gamble
   pays.

**Report lines it adds** (per species, P8):

- **Tar coverage**, mean and peak, as a share of the arena — the clock;
- **time in tar** and **hits taken in tar** — whether the clock is what kills;
- **burns by source** — brazier, belch, backfire, class fire — and **self-burn
  damage** as a share of all damage dealt;
- **coat uptime**;
- **slag built / climbed from**, and **warts burst**, with the minute each went;
- **swallowed**: count, stomach damage dealt, acid taken;
- **gutted** and **wallows broken**.

**Targets**, tier 2 (bestiary §2): the scripted hunter wins about **two in
three**, won fights in **two to four minutes**.

| Measure | Target | Why |
| --- | --- | --- |
| Threatening / poke / way in / walk up | ~35% / the rest / the rest / ~25% | a slower animal than the Ridgeback, and a tier lower |
| Unanswerable hits | 0 | the contract |
| Peak tar coverage | under 45% in a won fight, near the 60% ceiling in a lost one | the clock should be what the losses are about |
| Self-burn share of damage | 30–45% | fire is the big damage source, not a bonus |
| Warts burst in a won fight | at least one | the climb is being played at all |
| Every move landed at least once, none above half | — | the Ridgeback's "a telegraph that never lands is decoration" |

## 10 · What it needs built

**Shared pieces:** P1 (species — moves, parts, weak-point flags, knob
families), P2 (the arena as data: the mire, with its braziers), **P4 (floor
hazards) — this is the creature that brings it**, and P8 (the plan and the
report lines above).

**New and specific to it:**

- **The hazard list**, P4 as designed: `World::hazards: [Hazard; 16]`, eight
  bytes each — kind and slag layer in one byte, centre as two `i16` in
  millimetres, radius a `u8` in fiftieths of a metre, age or fuse a `u16`, one
  spare byte. Kinds: tar, burning, slag. A new pool landing on tar merges into it
  (areas add, radius capped at 4.5 m); a new pool with the list full merges into
  the nearest tar. Slag is capped at six; a seventh crumbles the oldest. The
  Belly flop **shatters slag** in its crash circle — it takes away your stairs
  as well as your floor.
- **Slag as solids.** Mounds are cylinders in the arena's solid query, the one
  the crosshair's ray, `first_along`, the stones and body resolution already
  read. That is a change to the solid query (P2), **not to `aim.rs`**; no new
  line of effect, and the tongue asks `aim::first_along` for the solid it stops
  on, like any line.
- **Braziers.** Four arena objects with a two-byte state each: standing, tipped
  (with a direction), or relighting. Any hit tips one away from the hitter and
  spills coals in a lane 5 m by 2 m, which ignites any tar in it or burns bare
  floor for 2 s. It relights after 15 s (`brazier_relight`). The Mireback's own
  hits tip them too.
- **The species**: a dome, four short legs, a jaw, a tongue bone, a throat sac
  that scales, a hollow stomach part, four wart parts. The rim is the mountable
  part; the stomach is mountable only by being swallowed.
- **Swallowed** on the ride machinery, one `u8` on the fighter naming the
  swallower and a timer. **It is the one place the creature touches aiming, and
  it needs no change there**: the stomach is a place, so `ground_under` returns
  its floor, the crosshair's ray passes through its wall as through any body,
  and `first_along` stops on the wall. The camera stays outside, behind the
  shoulder as always, and the view draws the belly bulging where each hit lands.
- **Fire sources as a flag.** Every effect that is fire — `FirePillar`,
  `FireTornado`, `Embers`, the fire ring, a lit stone, the coals, the belch —
  declares `ignites` in its kind table, the same way moves declare `aim()`,
  rather than being named in the hazard code. `JudgementField` does not: it is
  light, not fire, and a second class carrying fire is a decision (§12).

**Snapshot:** the monster about 210 B (a smaller rig than the Ridgeback's,
plus coat, four wart healths, sac poise), the hazard list 128 B, braziers 8 B,
swallow state 4 B — **about 350 B**, a little over the bestiary's 330.

**Per-frame cost** is dominated by the hazard list, and it is bounded: sixteen
discs against three bodies (two fighters and the toad's footprint) is 48 cheap
tests; fire's spread is pairs of pools, 256 at worst, run every ten frames and
only for pools that are burning; and six slag cylinders join the solid query.
None of it allocates, and all of it is fixed-size.

**Tests that would pin its rules:**

- `tar_slows_the_walk_and_halves_the_dodge_but_keeps_its_invulnerability`
- `fire_runs_along_tar_that_touches_and_stops_where_it_does_not`
- `a_burnt_pool_leaves_one_slag_mound_at_its_centre`
- `slag_burnt_again_is_a_step_the_bulwark_can_climb_from`
- `the_mireback_burns_in_its_own_tar`
- `the_belch_ignites_every_pool_it_marks_and_nothing_it_does_not`
- `a_belch_broken_through_the_sac_backfires_under_its_own_chin`
- `setting_a_wallow_alight_guts_it`
- `a_swallowed_fighter_is_spat_out_by_time_or_by_damage_whichever_is_first`
- `what_is_drawn_on_the_floor_is_the_hazard_list`
- `the_hazard_list_never_holds_more_than_sixteen`
- `a_burst_wart_takes_a_quarter_of_its_tar`
- `nothing_it_throws_can_reach_its_own_back` (the Ridgeback's, on the species)
- `the_rim_is_out_of_a_standing_hop_for_all_but_the_dual_mage`

**Milestones**, each ending in something checkable:

1. **M1 · The hazard list.** P4 with tar only: the list, the slow and the dodge
   rule, drawn from the list. A debug key drops a pool under the cursor. Ends in
   the first two tests and a screenshot.
2. **M2 · Fire and slag.** Burning, spread, slag as solids, braziers in a mire
   arena (P2). Ends in the fire and slag tests, and a fighter climbing a
   two-layer mound in the sandbox.
3. **M3 · The body.** The species rig, the rim and crown, `beastcheck` printing
   the routes table in §1, the walk, Spew and Backwash. Ends in the rim test and
   `beastcheck` output matching §1 or §1 corrected.
4. **M4 · The big three.** Belly flop with its ring, Tongue and Swallowed,
   Inflate and the sac. Ends in the swallow test and a scripted hunt that uses
   every move.
5. **M5 · Coat, warts, belch, wallow.** Self-burn, the coat, the four warts,
   backfire, gutted, the tide, and the brain's five terms. Ends in the remaining
   tests.
6. **M6 · Measured.** The hunter's plan and the report lines; tune to the tier-2
   targets across twelve seeds. Ends in a report printed beside §9's table, and
   a feel-log entry for what moved.

## 11 · In the world

**The Mire** ([../world.md](../world.md)), tier 2, the lowlands, across the
Crossing from the Dunes. A drowned causeway: the marsh people keep pitch fires
burning in iron braziers along it to keep the thing off the road, which is why
there is fire in a swamp.

**The arena** is 36 × 36 m — bigger than the first arena's 28 because the toad
alone is twelve across and the clock needs floor to eat. Flat peat, knee-deep
reeds drawn and not collided with, 1.5 m banks for walls. Four braziers on 1.5 m
stone plinths at the middle of each bank, standable, so each is also a platform.
The toad starts in the middle on clean floor. The floor's area is 1296 m²; the
tar's ceiling (sixteen pools at 4.5 m, less overlap) is about three fifths of
it, reached at about four minutes if nothing burns.

**Trophy:** a slag-crusted wart, as world.md already proposes.

**Sidegrade**, in the floor family: **Mire-walker** — *floor hazards slow you
half as much, and your dodge is 15% shorter everywhere.* A common-mechanic
modifier in the spirit of [../parked.md](../parked.md): it changes where you can
stand rather than how hard you hit, and it pays for it in the verb every class
leans on. It is worth most against the Mireback's own clock, the Blood mage's
Black spike field and the Elementalist's rough ground, and costs the most
against anything with a crash circle.

## 12 · Open questions

1. **Is a 40% walk mud or glue?** The number is set by the escape arithmetic
   (2.8 m/s is still faster than the toad). Whether it feels like a place you are
   slow in or a place you are stuck in is the difference between a clock and a
   trap, and only somebody wading through it can say.
2. **Is choosing to be swallowed a good gamble or a degenerate one?** At about
   300 health for 900 of its 11 000 it should be a late-fight play. The
   `swallow_greed` plan measures it for the bot; whether it feels like a clever
   read or a cheese is a person's call. Decision wanted: keep it choosable.
3. **Does the Elementalist make it tier 1?** She carries the fight's answer in
   her kit. That may be exactly right — every class has a creature it was built
   for — or it may want the toad's `flee` to favour her fire.
4. **The Dual mage has no fire and no floor.** She ignores the clock and cannot
   push it back. Identity, or a class that plays a different, duller fight here?
5. **A second class with fire.** Should `JudgementField` ignite tar? It burns
   people already; saying yes gives the Dual mage a lever, and makes fire less
   the Elementalist's thing.
6. **Braziers relight in 15 s.** Too long and the non-fire classes are waiting
   on a timer; too short and the clock never runs. The Ridgeback's history says
   the first number will be wrong.
7. **The stomach camera.** The camera stays outside and the belly bulges. Is two
   seconds of watching a bulge fun, or should the view go in?
8. **Is a full floor a loss condition or a slow one?** At the tar ceiling the
   fight is still winnable with fire. Whether that final minute feels desperate
   or merely tiring is the question the whole premise stands on.

## 13 · Where it landed

Built 2026-10-01: `--hunt mireback`, the Mire. The species is
`sim/src/species/mireback/` (the table, `fight.rs` for the floor and the rules,
`mind.rs` for its own scoring), the arena `sim/src/arena/mireback.rs`, the
clips `anim/src/beast/mireback/`, the look and the Mire's dressing in `game`,
the plan and the report lines `hunt/src/plans/mireback.rs`, and the rules
pinned as sentences in `sim/tests/mireback.rs`. The plan is
[plans/mireback.md](../plans/mireback.md); the passes are in
[feel-log.md](../feel-log.md) of 2026-10-01.

Numbers from `cargo run -p hunt --bin fight -- --species mireback --class <c>
--repeats 24`; the scripted hunter plays §9 with a fifteen-frame reaction.

```text
                won    mean    health left (a win)   threat / poke / way in / walk up   unanswerable
  Champion     21/24   120 s        454                  44 / 9 / 14 / 33 %                  1
  Bulwark      18/24   102 s        563                  45 / 8 / 13 / 34 %                  0
  Reaver       22/24    89 s        441                  43 / 8 / 13 / 37 %                  2
  Elementalist 24/24    41 s        653                  34 / 7 / 11 / 48 %                  0
  Blood mage    5/24   149 s        118                  42 / 8 / 13 / 36 %                  3
  Dual mage     0/24     --          --                  39 / 8 / 13 / 40 %                  0

  (2026-10-01, every class played. As first measured the rows read
   15 / 12 / 13 / 24 / 1 / 0 won and 1 / 1 / 1 / 0 / 0 / 0 unanswerable; on main
   the hour before the class layer, after the shared rule and the cast's
   other merges, 21 / 16 / 23 / 24 / 5 / 0 won and 1 / 0 / 1 / 0 / 1 / 2.)

  coop, two Champions 10/12 in 84 s;  temper 3, Champion 6/12 in 115 s
  swallow_greed (--gamble), Champion 7/12 in 113 s -- the same as the plan without it

  landed / thrown, 24 Champion hunts
    Spew 21/307  Belly flop 19/342  Tongue 18/232  Flint belch 2/22  Backwash 0/25
    Inflate 22/113  Wallow 9/64     (Swallow and Gag are never chosen: the tongue starts one)
```

**The Mireback's own lines**, over the same 24 Champion hunts: tar coverage
peaks at 18–45 % in a won fight and 28–52 % in a lost one; self-burn is 25–72
% of all the health it lost in a won fight, about 45 % at the median; at
least one wart burst in 14 of the 15 wins, the first at 0.6–2 minutes; one or
two guttings a fight; zero to three swallows.

**2026-10-01: the hunter plays all six classes** (`hunt::class`,
[bestiary.md](../bestiary.md) §8). **The rows had moved before it**: the
Champion wins 21 of 24 and the Bulwark 16 on main, against the 15 and 12 this
section first recorded, and the unanswerable counts moved with them -- the
Mantis made the shared rule ask whether the hurt hunter was inside the move's
reach, and the cast's other merges landed in between, so which moved them is
not separable here. With the classes played: **the Dual mage still wins
nothing**, and now for a reason of the toad's: her hop is the one that reaches
the rim, she climbs onto it, and her punch passes over the warts from the
crown (one burst in a fourteen-minute hunt, 1058 swings to 48 connecting).
**The Blood mage** wins 5, the same as before, spiking the pools her Grasps
leave under it (19 spikes, 21 Grasps in the 24) and losing her red to the
price; her unanswerable hits rose from one to three -- every one of them, and
the Champion's one and the Reaver's two, the old measure of a flop from fresh
tar (below): with it fixed, zero for every class. **The Reaver** wins
22 in 89 s, her shadow at the flank (95 sent, 37 lotuses). **The
Elementalist** is where she was: every hunt, in 41 s.

**Against the targets.** The Champion wins five in eight, against about two
in three, in two minutes (two to four asked). Zero unanswerable hits except one
in 24 for three classes, each counted by the report's own rule as a flop out
of tar laid after it committed. **They were the measure's, and it is fixed
(2026-10-01, [plans/polish-fights.md](../plans/polish-fights.md))**: the tar
was the crash's own ring of pools -- laid on the crash frame, or merged into an
old pool that grew under the hunter -- or, in coop, old tar a floating Dual
mage came down into during the windup. Neither stopped an escape. The rule
now asks for tar that was not on the floor at the commit and was under them
the frame before the crash, which is §6's sentence; zero since, six classes,
solo and coop. Self-burn is about where §9 wants it, a little high. Warts
are being burst. **What is off:** threatening is 45 % against 35, and walk-up
32 against 25 -- the toad spends its time winding up long moves, and the
openings are long ones; the poke window is small because nearly everything it
throws covers the flank. **The losses are not about the clock**: the floor
never nears the 60 % ceiling, and the hunter dies to the flop and the spew it
did not walk out of. The belch is thrown rarely and lands rarely (the hunter
leaves tar before it, which is the answer); the Backwash hits for nothing by
design (it tars), so the generic "landed" never counts it. **The Elementalist
makes it a tier-1 fight** (§12 question 3): her pillar lights every flop ring
under it, and she wins every hunt in under a minute. **The Dual mage and the
Blood mage** lose nearly all of them -- since 2026-10-01 with their bars and
pools played (above): the Dual mage on the crown, missing the warts; the Blood
mage out-paid by her own costs.

**The body, as `beastcheck --species mireback` prints it**: rim 5.18 m
standing (the Dual mage's 5.97 hop only), the dome 5.78, the crown and its
warts 6.4 (a walk up the dome; §1 said 7.0, but on a toad whose rim is 5.2
the warts at 6.4–7.0 read as "on top" and stay out of every hop); winded,
the left flank 3.73 (all but the Bulwark); through the flop 4.48 (Reaver,
Elementalist, Dual mage); gutted or wallowing, the flank at 1.7–1.8 (everybody).
Slag adds 1.5 m a layer, so one mound puts the rim inside everybody's hop but
the Bulwark's and two put it inside his; a loaded planted shield is his other
way (`tests/mireback.rs`).

**Changed from this document while building**, beyond the numbers:

- **Health 10 000**, from 11 000: won fights ran past three minutes.
- **The Swallow and the Gag are moves the brain never chooses**
  (`MoveDecl::never_chosen`): the tongue landing starts the swallow, a guarded
  tongue the gag. Their frames are still declared, so the table, the clips and
  the frame data have them.
- **Gutted is the stock topple, winded the stock stumble**: the shared strain
  machinery runs them, and `beastcheck` reads their poses.
- **The legs are soft**: a solid forearm made a ledge at hip height that the
  hunter climbed by accident.
- **The brow sits on the root**, not the head, so the rim does not dip every
  time the head moves; the head's moves keep the rim at 5.0 or above.
- **It burns by its footprint**, up to three pools (`burn_self_pools`), not by
  the shared rule's position ([hazards.md](../hazards.md) §8).
- **Fire spreads after its `Spread` frames**, measured shape to shape, so a
  fuse runs at a pace a hunter can see and the coals' lane lights what it
  touches (generic, [hazards.md](../hazards.md) §8).
- **The flop shoves bodies out sideways** (`FightDecl::lands_on_bodies`) and
  **a face pointing at the floor is no surface** (`rolls_over`): the crash
  pressed a fighter through the ground, and a fighter dodging past a wallow
  was mounted on the underside of its flank. Both are opt-in so the
  Ridgeback's pins hold; whether the Ridgeback wants them is a person's call.
- **The braziers are drawn from the snapshot** as the species' marks (iron,
  flame, embers relighting), not a renderer prop; tipped by any fighter's hit
  or the toad's own body.
- **Not built**: the coat's sheen and a swollen sac on the body (the renderer
  has one material per paint, not per creature state); the stomach camera
  stays outside (§12 question 7).

**2026-10-01, polished ([plans/polish-fights.md](../plans/polish-fights.md)):
a pause of 60 frames between moves, not 40, and health 11 000, not 10 000**
(§1's first guess). Threatening was 45 % against 35 because a toad that
decides every two thirds of a second leaves nothing but a poke between its
long tells; at 60 it is 35–41 % for every class (the Champion 38 / 8 / 13 /
41), and the thousand back keeps the fight where it was asked rather than
handing it over: the Champion 18 of 24 (from 21) in 147 s (from 120), the
Bulwark 16, the Reaver 21, the Elementalist 24 in 48 s, the Blood mage 4,
the Dual mage 0; two Champions 12 of 12 in 98 s; zero unanswerable. Walk-up
is 40–52 % against 25 -- the pause is where it went. Tried: 70 and 12 000
(37 % threatening, the Champion 12 of 24 in 190 s, the Blood mage none).

**Questions for a person**, beyond §12: is the Elementalist's one-minute fight
her identity or a hole (`flee` could favour her fire); is 45 % threatening
oppressive in the hands or only in the harness; and does the flop -- the
toad's main damage, nineteen landings in 24 hunts -- read from its ring of
warnings early enough, which only somebody watching it come down can say.

**2026-10-01, later: the Bulwark throws his shield** (`hunt::class`,
`Hands::throw_in`): on a window five to eleven metres off he throws it at the
work, leaps to it as it flies and Slams out of the leap. His row above is
re-run with it; the feel log of the day has the before and after. The thrown
shield struck no creature then ([review.md](../review.md) `CLASS-5`); **later the same day it
did** -- thrown, the first body it meets; recalled, each it passes once. Here it wins him two more (37 throws, every one leapt and slammed).
 Same seeds, after: 18 of 24 in 102 s, bit for bit the same: every throw is leapt to before it reaches the toad.