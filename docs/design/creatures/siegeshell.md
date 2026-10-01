---
status: proposed
proposed: 2026-09-30
tier: 5
---

# Siegeshell — a hill that walks to the wall

The last fight in the world and the biggest body in it: a crab-shelled, turtle-headed
colossus twenty-four metres to the top of its crown, forty long, on six legs like
towers. It is not hunting anybody. It is walking down a valley toward a town wall,
and the whole fight is the distance left. This document is one entry in the cast
planned in [../bestiary.md](../bestiary.md), and it leans on every piece of the
Ridgeback's machinery in [../monsters.md](../monsters.md) — the skeleton, the ride,
the grip test, the glance, the telegraph markers, the fight report — at a scale none
of it was written for. It is last in the build order because it is the fight that
will find every assumption the other ten did not.

The parasites that live on its shell are [gnawers](gnawers.md), the same critters and
the same pack brain, with one new trick: they can hold on to a moving part.

---

## 1 · What the fight is

**It isn't fighting you; it is walking to the wall, and you are the only thing that
can stop it.**

Everything it does to a fighter is a side effect of walking, or of shaking off what
is on it. You cannot kill it from the ground. Its body takes a twentieth of any hit
(`shell_armour`), and there is no health bar. It dies when the three **anchors** on
its crown are broken, and an anchor can only be reached by climbing. So the ground
game exists to open a way up, the climb exists to reach the crown, and the clock
runs the whole time.

The sizes, measured against the jump (full hop 2.7 m on the Bulwark to 6.0 m on the
Dual mage):

| Where | Height standing | Why that height |
| --- | --- | --- |
| Ankle joint (the breakable box) | 0.5–3.5 m | A level swing from a chest at 1.2 m hits it. The Ridgeback's feet taught that a level shot goes under anything on stilts |
| Belly (the plastron) | 9 m | Out of everybody's reach, and a roof over the ground between the legs |
| Shell rim | **14 m** | Out of reach for all six classes standing. Twice the Ridgeback's back |
| Flank plates, the shell plateau | 16, 18, 20 m | The staircase from the rim to the crown, 2 m a tread |
| Crown, where the anchors stand | 22 m, the anchors 2.5 m tall on it | 24 m to the top, the number the seed asked for |
| **Rim on a stumbling side** | **6 m** | The Dual mage floats to it; everyone else climbs the leg (§4) |
| Knee of a broken leg, stumbling | 2.4 m | Inside the Bulwark's 2.68 m hop by a quarter-metre. The climb is designed to his hop, deliberately |

It is 24 m across the shell and plants its feet 28 m apart. **The valley is 50 m
wide, not the seed's 30**: at 30 there was a metre between its feet and the valley
wall, so there was no flank, and the flank is where the ground game is played.
The valley is **240 m long** and ends in the town wall.

**It walks at 0.8 m/s, not the seed's 1.2**, in a tripod gait: three feet land
together, then the other three, **a beat every three seconds** (180 frames). At
1.2 m/s it arrived in two and a half minutes, and three anchors each worth a climb
cannot be broken in that, even by two. At 0.8 it arrives in five minutes untouched,
and every halt, stumble and broken ankle you buy pushes that out. A fighter walks at
7 m/s, nine times its pace: you can always catch up, and that is not the problem. The
problem is that catching up does not stop it.

The loop:

1. **Read the beat.** Every tripod landing throws a shockwave ring around each of
   the three feet. Near the legs, the ground game is a rhythm: jump on the beat,
   swing between them.
2. **Break two ankles on one side.** That side stumbles for twelve seconds, the rim
   comes down to 6 m and the broken legs' thighs lie as a stair from the floor to it.
3. **Climb the moving landscape.** Rim, flank, plateau, crown: steam vents on
   timers, gnawers in the crevices, and a shell that shrugs off whatever stands on
   its rim.
4. **Break an anchor.** It halts for twenty seconds and kneels, and its phase
   changes. Get down, or stay up for the next one.
5. **Open the crown.** Two ankles breaking on one side *while somebody is standing
   at an anchor* opens that anchor's root for the stumble's length. That is the big
   window, and it takes two people to make (§4, §8).
6. **At the wall**, it plants and charges the **Siege beam** through its anchors.
   Break one in the twenty-second tell, or the wall is breached.

### The fail state

**The hunt is lost when the wall is breached twice.** The first beam brings down the
gate and the hunt carries on, with the gate's rubble in the valley's last forty
metres as new solids. The second takes the district behind it, and that is the end.
For a rudimentary world this is one number the P7 wall carries (`wall_breaches`,
starting at two). Having the town keep the damage between hunts — a district lost
for good — is a world decision, not a fight decision, and is §12's first question.

---

## 2 · The moves

Every move is either the walk itself or the shell getting rid of something. Frames
and damage are **first guesses**, to be replaced by the harness.

| Move | Range | What it is | Tell (frames) | The answer |
| --- | --- | --- | --- | --- |
| **Footfall** | close, around each foot | A tripod lands. Under a pad: crushed. Out to 6 m: a knee-high shockwave ring | The lift, 90; the landing spot drawn 60 before. Ring out 12, 0.6 m tall. **110** + a 40-frame knockdown; the pad 300 | **Jump the ring, on the beat** |
| **Stamp** | close, off the beat | One foot lifted high and held over a fighter who is under where it would land | Startup 70 (tracks at `stamp_tracking`), active 4, recovery 60 planted. **260** | **Dodge out of the pad at the drop** |
| **Ankle drag** | close, clinging | A planted foot is dragged inward along the ground, toward the belly, in a 7 m arc | Startup 50 (the knee cocks inward, dirt heaps on the marker), active 18, recovery 40. **150** + stagger 60 | **Go round the outside of the leg** |
| **Shed** | mid, 4–14 m outside the feet | The rim on one side scrapes and rattles, and six to ten shell plates fall onto marked circles 2.5 m across | Startup 60 (the plates stand up like a skirt), falling 40, recovery 90. **140** each | **Go in, under the rim**: the shell overhangs its own feet |
| **Plough** | long, ahead, 15–45 m | The head drops and scours the valley floor, throwing a lane of rubble down the valley at 25 m/s | Startup 80, active 30, recovery 120. Lane 8 m wide. **120** + knockdown | **Get behind a solid**: a boulder, a stone, a planted shield, a Bulwark's guard |
| **Vent** (P4 hazard) | aboard, on the grates | A grate on the shell blows a column of steam 2 m across and 4 m tall | Hiss and wisps 60, blast 90, on a 360-frame cycle per grate. **40** a tick every 10 + a push up of 6 m/s | **Step off the grate** |
| **Shrug** | aboard, rim and flanks | The shell rolls 18° away from one side and back | Startup 45 (that rim lifts, the plates grind), active 30 | **Brace** (crouch). Rim and flanks throw loose riders; the crown is calm |
| **Shiver** | aboard, the crown | The plates round the anchors lift like hackles and rattle | Startup 40 (a rising hum), whip 36, lockout 300 | **Jump it**, late in the tell. It exceeds a brace, like the Ridgeback's shake |
| **Siege beam** | the wall | From the siege line, the anchors pour light through the head into the wall | **1200** (twenty seconds) at the wall; beam 180 | **Break an anchor before it fires** |

Nine rows, nine answers. The two that share a floor — Footfall and Stamp, both a foot
coming down — are deliberately a jump and a dodge, and they are told apart by **when**
rather than by what: the footfall is always on the beat and the stamp is never
(§5, `beat(m)`). A fighter who learns the beat has the footfall for free and is
listening for the foot that lifts when nothing else does.

**What is subtle.**

- **The ring and the pad are one move with a positional half.** Nobody should be under
  a pad: its landing spot is drawn from the lift, ninety frames before the hit, and
  the pad's 300 is the price of not reading the floor for a second and a half. The
  ring is the half a person answers. It sweeps from the pad's edge at 2 m out to 6 m
  in twelve frames, so it passes any one spot in two or three, and a hop spends forty
  frames above 0.6 m. Jumping it is easy alone and the point is that it is never alone:
  it is on a beat, and everything else you are doing has to fit between beats.
- **The stamp's recovery is the ankle's window.** Sixty frames with that foot planted
  and nothing else it can do with the leg is the longest walk-up the ground game
  offers. Bait a stamp, dodge it, and unload into the ankle it left behind. A stamp is
  on a 600-frame lockout per leg, so it cannot be farmed.
- **The drag goes inward, always.** A clinging fighter is usually on the outside of a
  leg, away from the belly, because that is where the Shed does not reach from above
  and the Plough does not reach at all. The drag punishes staying there too long and
  the answer is to be on the *far* side of the leg from the arc — round the outside.
  Staying in close by the leg is fine; staying in close *for four seconds* is what it
  triggers on (`cling_frames`).
- **The Plough is blockable.** It is a volley of projectiles, so it stops on solids and
  on a guard. That is what makes it the one long-range move whose answer is the
  arena rather than your feet, and it is the Bulwark's and the Elementalist's moment.
- **Nothing it throws reaches its own shell**, as on the Ridgeback, with two
  exceptions that are not throws: the vents are hazards in the part's own frame, and
  the gnawers are bodies. `nothing_it_throws_can_reach_its_own_shell` pins the first
  half; §3 is the second.

---

## 3 · A threat at every range

**Close — at the legs.** The footfall rings on the beat, the stamp off it, the drag on
whoever clings. Six legs, three landing at a time, so standing between two legs of a
tripod puts you inside two rings at once — they meet, and the jump is the same jump.
Between a landing leg and a lifted one is the good place: one ring, and a lifted foot
you can see coming.

**Mid — the flank, 4–14 m out from the feet.** The Shed, on the side the target is
on. It is the move that stops the flank being where you stand and watch: the answer
is to go in, toward the legs, where the beat is. So the mid range pushes you close.

**Long — ahead of it.** The Plough, 15 to 45 m down the lane it is walking. Standing
in front of it and shooting is the most natural thing to do in a fight that is a
walk toward you, and it is the thing the head is aimed at.

**Long — behind it.** Nothing it *throws*. Behind it at range is not a place you fight
from; it is a place you lose from, because nothing you can do there stops it walking.
The gnawers make that explicit: a fighter more than 25 m behind the tail for more than
five seconds (`straggler_frames`) is what the pack comes down for.

**Aboard.** Vents on the plateau, Shrug on the rim and flanks, Shiver at the crown,
the gnawers everywhere, and the fall under all of it. Where you stand is a gradient, as
on the Ridgeback:

| Standing on | Walk | Shrug | Shiver | Stumble (that side) | Kneel (anchor broken) |
| --- | --- | --- | --- | --- | --- |
| Rim, the low side | calm | throws loose, brace holds | — | **throws anyway** | calm |
| Flank plates | calm | throws loose, brace holds | — | throws loose, brace holds | calm |
| Plateau (the vents) | calm | calm | — | calm | calm |
| Crown (the anchors) | calm | calm | **throws anyway** | throws loose, **brace holds** | calm |

The walk never throws anybody. Its bob on the beat peaks at a third of grip
(`the_walk_never_throws_a_rider`), because a colossus whose every step bucked would make
the ride a room nobody could stand in. The kneel after an anchor breaks is slow on
purpose — ninety frames to settle — so the person who broke the anchor stays on it.

**The tempting safe spot: under the belly, between the middle legs.** Thirteen metres
from every foot, so outside every ring; under nine metres of shell, so out of every
Shed; behind the head, so out of the Plough. It was a safe spot in the first sketch of
this design, for the same reason the Ridgeback's tail root was. What covers it is the
population: **the plastron is the gnawers' roost**, and anybody under it for more than
three seconds (`roost_frames`) is dropped on by the pack, which fights by the pack's
own rules — only two attack at once, count them, kill the leader
([gnawers.md](gnawers.md)). It is a place you can go, and it is a fight when you get
there.

**The second one: standing on a lifted foot.** A pad in the air is a platform nobody
aimed; a fighter on top of one is carried up and put down again. It is not mountable
(`foot_pads_are_not_footing`): the pads are solid, you slide off. Otherwise the stamp
is a lift.

---

## 4 · The approach and the window

**The ankles.** Six of them, 1200 each (first guess). Each break is permanent:

- the leg **drags** for the rest of the fight, and the walk is 8% slower per broken
  ankle (`limp_per_ankle`), so breaking legs buys clock directly, and you can see it
  in the stride;
- that side of the shell sits a metre lower per broken ankle — after two, the rim on
  that side is at 11 m for good, which is still out of every standing hop but is a
  shorter fall;
- a broken ankle **buckles** instead of breaking again: 400 damage into a broken ankle
  (`buckle_health`) folds it once more, and a buckle on a side with two broken ankles is
  another stumble. So the first stumble on a side costs 2400 and every one after it
  costs 400. That is the fight's arc on the ground: the legs are a wall at the start
  and a lever by the end.

**The stumble.** Twelve seconds (720 frames). The side drops over forty frames, the
rim to 6 m, the two broken legs' thighs splay out and lie as a stair: treads at 2.4,
3.6, 4.8 and the rim at 6.0. Every gap is 1.2 m, inside every class's hop with a metre
and a half to spare; the first tread is inside the Bulwark's by a quarter-metre. The
stumble **halts the walk**, which is its other half: twelve seconds of clock back.

**The crown.** Three anchors, 2400 each, in a triangle round the crown's peak. They
take full damage and are the only thing on the body that does. Breaking one:

1. **halts it for twenty seconds** and it **kneels**, the whole shell settling so that
   the rim is at 6 m on both sides — a second way up for whoever is still on the floor;
2. **cancels a Siege beam** that is charging;
3. **changes its phase**:

| Anchors broken | Phase | What changes |
| --- | --- | --- |
| 0 | **Walking** | The beat every 180 frames. Vents slow (a 360-frame cycle). The gnawers roost and drop only on stragglers and the belly |
| 1 | **Roused** | Shed and Plough come off their long lockouts. Vents quicken to 270. The pack wakes: it hunts riders on the shell as well as the ground |
| 2 | **Hurried** | It walks 25% faster and the beat comes every 144 frames. Shiver's lockout drops to 200. The last anchor's grates vent together |
| 3 | — | It dies where it stands, the shell settling into the valley as a hill |

**Hurried is the trade.** Breaking the second anchor early buys twenty seconds and then
takes back more than that over what is left of the valley. Whether to break it before
the siege line or at the wall, in a beam's tell, is a real decision, and it is the one
this design most wants a pair of players to argue about.

**The big window: the Opening.** A stumble that begins while a fighter is standing
within 3 m of an anchor **opens that anchor**: its plates part and its root is exposed
for the stumble's twelve seconds, at `opening_damage` ×2.5. An open anchor dies in
about eight seconds of steady swinging against about twenty closed and interrupted by a
Shiver every six. Nothing else in the fight is as fast, and it takes somebody at the
legs and somebody at the crown at the same moment. The Opening is to this fight what
the topple is to the Ridgeback's: the thing the approach is for.

**From fresh to desperate.** Strain works as on the Ridgeback: recent damage to an
ankle buys susceptibility, then an interrupt. The thresholds fall with **anchors
broken**, not health, because it has none. Fresh, nothing short of a real burst
interrupts a stamp; hurried, an ordinary chain on an ankle does. What control does:

- a **slow** landing while susceptible slows the walk at a quarter strength
  (`slow_share`). A slow on a colossus is clock, and it is priced like it;
- a **knock-up** while susceptible counts as a buckle on the ankle it hit;
- a **grab** does nothing to the body and everything to a gnawer.

---

## 5 · The brain

Most of it is not a brain. **The walk is a gait with a heading, and every footfall is
the gait.** What is left to decide is which foot does something other than walk, and
what the shell does about riders. Two channels, each with its own `Doing`:

- **Legs.** Footfall (not chosen: it is the gait), Stamp, Ankle drag.
- **Body.** Shed, Plough, Shrug, Shiver, the Siege beam.

A body move may not start inside `channel_gap` frames of a leg move's active window,
and vice versa, so two hits never arrive from two channels on one frame. The walk goes
on under body moves; under leg moves the gait pauses on that leg only.

**Steering.** No pursuit, no target heading. It follows the valley's centre line to the
siege line 20 m from the wall (`path_pull`), turning only to stay in the valley. The
Ridgeback's yaw controller is reused with a turn rate that is barely there, because a
body that does not turn is part of the premise: you cannot make it face you.

**Glancing.** Every `glance` frames (40, slower than the Ridgeback's) it samples every
fighter and every rider. No `lead` for the legs: a foot tracks through its startup and
locks on the drop, which is enough. The Plough leads, as the Ridgeback's charge does,
because it is the one move thrown at somebody moving across open ground.

**Scoring**, per channel, at each decision point:

```text
U(m) =  range(m)        as the Ridgeback's
      + beat(m)         zero on the half-cycle after a tripod lands; −∞ on the half before.
                        Off-beat moves never share a beat with the footfall
      + cling(m)        grows with how long one fighter has been within 3 m of one ankle
      + region(m)       for the body moves: which part a rider is standing on
      + variety(m)      as the Ridgeback's
      + phase(m)        a per-phase table: 0 locks Shed and Plough behind long lockouts
      ⟂ cooldown(m)     a move on its lockout scores nothing
      ⟂ siege           at the siege line, the beam is the only body move until it fires
```

`decisiveness` is high, 0.85: this is a machine walking, and the thing to learn is its
rhythm, not its moods. `think_frames` does not apply to the legs — the beat is the
think timer — and is 50 for the body.

**Targeting** is per channel. Each leg considers the fighters near *it*, so a pair
split between two legs is two separate problems for the creature and none for the
players. The body considers riders by region and ignores the floor except for the
Plough and the Shed. It never keeps a target in the Ridgeback's sense
(`target_switch`), because it is not after anybody.

**The gnawers have their own brain** — the pack brain, attack tokens, the leader — and
the Siegeshell's only say in it is when they come down (`roost_frames`,
`straggler_frames`) and how many the shell holds: eight, and it refills one every
twenty seconds from crevices on the plateau (`brood_frames`). A dead leader breaks the
pack for thirty seconds and then a new one crawls out.

---

## 6 · Reading it

**The beat is the tell.** A tripod lifts, hangs, lands, and the next lifts. It is heard
before it is seen: a creak of the lift, a pause, the boom. The sound is on the
simulation's clock (the stride accumulator, in the snapshot), so a rollback never
double-plays a footfall.

**The floor carries almost everything**, because at this size the body is out of frame
most of the time:

- every lifted foot's landing spot, as a pad circle from the lift, and its 6 m ring
  faintly outside it, filling toward the landing as the Ridgeback's markers do;
- a **stamp** pad drawn in the stamp's colour rather than the beat's, and moving while
  it tracks;
- the **drag**'s arc on the ground, from the outside of the leg sweeping in;
- the **Shed**'s circles on the flank, the **Plough**'s lane down the valley;
- the vent grates, drawn on the shell as plates of a different colour that glow through
  the hiss;
- the **Siege beam**: a line from the crown to the wall, filling over its twenty
  seconds, and the wall itself cracking in the same fill.

All of it comes from `Monster::telegraph` and `Monster::hit_volume`, as the contract
requires. **The distance is drawn in the world**: standing stones every 40 m down the
valley, so the clock is read off the landscape rather than off a HUD.

**Silhouette.** A domed shell with a raised crown and three anchor crystals glowing on
it; six columnar legs; a low heavy head. Broken ankles leak and drag; a leg that has
buckled is visibly shorter. An open anchor's plates stand apart and its root glows.

**The camera, and what changes.** The follow camera is a 9.2 m sphere at a 57° framing
field of view, and a fighter at a leg sees about ten metres of height: the ankle, the
shin, nothing above. That is deliberate, and nothing in the aim changes for it:

- **The aiming ray does not change on the ground.** Bodies are not on it, so a shot
  aimed at the floor behind an ankle crosses the ankle and `aim::first_along` meets it;
  aimed up at the crown, it goes to the sky, and the shot's own path stops on the rim's
  armour. The crown is not hittable from the floor, by design, and the rule already
  says so.
- **The drawn camera must not be pulled in by the creature.** The occlusion pull-in in
  `view::camera` reads the arena. It must go on reading only the arena: a leg the size of
  a tower swinging through the boom every three seconds would yank the view on the beat.
  Parts inside the boom are drawn cut away instead. That is `view` only, and the aim is
  unaffected.
- **The look-up limit is 85°**, enough to see the crown from under the rim. No change.
- **Aboard is where the aim breaks, and `aim.rs` has to change.** On the shell the rider's
  floor is a creature part, and parts are not on the ray. A grounded ability aimed at the
  plateau goes through it and is cast on the valley floor 20 m below; a Reaver's shadow
  sent to the rim from the floor goes through the rim. The proposal: **the top face of a
  mountable part, met from above, is a place** — terrain, for the ray's purposes — and a
  grounded ability's "max range on the ground" is on the footing under the caster when
  they are mounted. Met from below it is still a body, which keeps what the 2026-09-13
  change was for: a crosshair under the rim, on the creature's chest, still goes through
  it. This is a change to `aim.rs`, true of every creature at once, **and it changes the
  Ridgeback** (a skillshot at its back from a platform), so it lands on its own after P1
  with the Ridgeback's twelve seeds re-run and the difference read, never inside P1's
  bit-identical step.
- **Riders carry yaw, not pitch.** The shell pitches 15° in a stumble and rolls 18° in a
  shrug. The camera does not tilt with it — a horizon that rolls under your hands makes
  people sick — and the swing's dead zone reads the true horizon, which on a 15° slope is
  close enough.

**Unanswerable, for this creature**, means three specific things the report must count
and the design must drive to zero: a stamp thrown while its pad was out of the
fighter's view *and* off the drawn floor (it is always on the floor, so this is a
marker bug); a ring meeting a fighter who is helpless after a throw (rings ignore the
throw's helpless frames — `a_thrown_rider_is_not_caught_by_the_beat`); and two channels'
hits in one window (`channel_gap`).

---

## 7 · The classes

**Champion — has it easy on the ground, ordinary above.** The ankles are his fight:
three weapons into a static box, a Rush to cross between legs inside a beat, and the
pole vault as a way onto the first tread without waiting for the stair. Aboard he is a
melee fighter on a moving floor against a pack, which is what he is anywhere. He is the
natural legs half of a pair. Identity.

**Bulwark — the question, again, answered two ways.** The stumble's stair was designed
to his hop: 2.4 m first tread, 1.2 m risers. So he climbs every route anybody climbs,
which the Ridgeback never managed. And the Plough is his: a held guard stops the
rubble, a planted wall stops it for whoever stands behind it, and in a pair that is a
lane of the valley made safe. The proposal on top: **a thrown shield that strikes a
mountable part plants on it**, in the part's frame, and leaping to it is his climb —
throw at the rim, leap, arrive with the shield in hand on the edge. It needs P1 to let
a planted shield live on a part, which is the same change the Blood mage's pools need.
Hard for him: the Shiver, because his jump is the lowest and the late-jump read has the
least margin. Identity, if the shield plant is built; a hole on the shell if it is not.

**Elementalist — a hole, or the sharpest identity in the cast.** Her structure jump
off one stone is 25.9 m, which clears the crown from the floor. Stones cannot be
raised on the shell (`stones_rise_from_the_ground_only`: they climb out of earth), so
she goes up in one leap from where the creature *was*, and has nothing to build with
once she is there. Aboard she is thin. On the ground she is the Plough's other answer
— a stone is a solid — and her fire pillars light the flank. Unchecked, a perfectly
timed double (44.8 m) is a flight over the body. The proposal keeps the leap and prices
it: fall damage from the apex (P6) means a jump that overshoots the crown is a fall of
twenty metres onto it. Whether she should be able to reach the crown from the floor at
all is §12.

**Blood mage — the most interesting climb.** Her blink goes to any pool of hers the
crosshair is on with a clear line. Pools spill where she cuts somebody — and a gnawer is
somebody. A Haemorrhage bolt into a gnawer on the rim spills a pool under it every tick,
wherever it runs, and it runs home up the shell. She climbs by bleeding a parasite and
following its trail. That needs pools anchored to a part (the same change as the
Bulwark's shield) and the top-face aim rule, since the crosshair has to be able to rest
on a pool on the shell. Hard for her: the ankles, where her scythe's reach is wasted on a
box that does not move. Identity.

**Dual mage — she reaches the rim early, and that is her identity here.** Her 6.0 m
float reaches a stumbling rim from the floor. At three quarters on the lower bar her
second jump reaches the stair's top tread from anywhere nearby; with wings, every press
of space is a beat and she can climb to the standing rim at 14 m without anybody
breaking anything. The cost is real: wings need both bars full and she is burning while
she holds them, the gnawers on the rim are the targets she needs to keep them, and a
wing beat resets her fall, so she is also the one class the fall does not punish. Solo,
she is the one class whose plan can be *mostly the crown*; she still needs the legs for
the Opening, and without it an anchor is twenty seconds of swinging on a shivering crown.
Identity, and the harness should say whether it is too much of one.

**Shadow Reaver — hard on the climb, excellent above.** Her shadow goes where the
crosshair is, nine metres out: with the top-face rule, that is the stumbling rim at
6 m, and a dash up to it. The standing rim is out of a throw's range from the floor. On
the shell her two bodies are the right shape for a forty-metre landscape: the shadow at
an anchor while she handles the pack on the plateau, the copy marking the anchor from
the field, the dash to cash the marks. The shadow has to ride the part it waits on, or
the walk leaves it behind at 0.8 m/s — the same part-frame anchoring as the pools and
the shield, a third time. Hard for her: 750 health and a 24 m fall. Identity.

---

## 8 · Coop

**This is the fight built for two, and the numbers are set for two.** Solo uses the same
numbers, which is why a decent player alone mostly loses.

The design is two roles that need each other on a count:

- **Legs.** Breaks ankles on the beat. Watches the crown. Holds the second ankle on a side
  at a sliver until the climber calls, then breaks it — that stumble is the Opening.
- **Crown.** Climbs on the first stumble, crosses the plateau between vents, reaches an
  anchor and calls. Braces for the stumble (the crown's lurch is under a braced grip),
  unloads into the open root, jumps the Shiver.

The roles swap on the kneel: an anchor broken puts the rim at 6 m on both sides for
twenty seconds, which is the legs player's way up and the crown player's way down.

What changes with two, beyond the roles:

- **The pack splits.** Attack tokens are shared, so two targets halve what reaches each.
- **The clock does not.** Two people do not make it walk slower; they make each halt
  earlier. That is the point of setting the clock for two.
- **Stumbles throw the rim.** A legs player who breaks without looking throws the crown
  player's approach off the low rim. The call is not optional.
- **The Plough protects.** One player behind the other's shield or stone is the lane
  made safe.

**Solo** is possible, and the route is the kneels: break two ankles, climb in the
stumble, reach an anchor closed, break it in the Shiver's gaps, and use each kneel to
come down and start again. No Openings. The scripted hunter should win about one in
twenty that way.

---

## 9 · Measuring it

**The scripted hunters' plan (P8)** — two of them for the coop target, one for the solo,
each a decent player with a fixed fifteen-frame reaction and the Ridgeback harness's
slop. What a person learns in their first ten minutes:

1. **Legs:** stand 4 m from an ankle, on the outside, beside a leg that has just landed.
   Jump every ring. Swing between beats. On the drag's tell, step round the outside. On
   a stamp, dodge at the drop and unload into the planted ankle. Move to the next leg on
   the same side after a break. Stop one hit short of the second break and wait for
   the call.
2. **Crown:** wait at the knee of the side being broken. Climb on the stumble. Walk the
   treads, stepping off grates through the hiss. Brace on the Shrug's tell. At the
   crown, stand at the nearest anchor and call; swing; jump the Shiver in frames 25–39 of
   its tell.
3. **Both:** never stand under the belly; never trail behind; at the siege line, all
   hands on the anchor with the least health left.

The call is a shared flag between the two scripted hunters in `crates/hunt`, not
simulation state.

**Report lines it adds:**

```text
  distance left at each anchor     148 m / 61 m / at the wall
  arrived at the siege line        6:42         beams fired / cancelled   1 / 2
  wall breaches                    1 of 2
  beats: jumped / caught           212 / 19     stamps landed / thrown   3 / 14
  stumbles                         5 (2 buckles)  openings earned / used   2 / 2
  climbs: started / reached crown  7 / 4        time aboard   38%
  thrown / fell                    9 / 3        fall damage taken   611
  gnawers killed                   23           leader kills   2
  windows, ground   threatening / poke / way in / walk up   44 / 14 / 17 / 25 %
  windows, aboard   threatening / poke / way in / walk up   38 / 16 / 20 / 26 %
```

(The numbers are illustrative; they are the shape of the report, not a prediction.)

**Targets.** Tier 5 in [../bestiary.md](../bestiary.md) §2: a scripted hunter alone
wins about **one in twenty**; a won fight lasts **8–20 minutes**. For the pair, about
**one in three** — this fight's version of the Ridgeback's four in twelve, because it is
tuned for two the way the Ridgeback is tuned for one. The four windows: about **four
tenths** threatening and **a fifth** walk-up, **per region**, because a fight that is safe
on the ground and a meat grinder on the shell averages to the right numbers and is two
wrong fights. **Unanswerable hits: zero.**

The clock is the length lever and the budget checks it: untouched it arrives in five
minutes; a pair that stumbles it five times, breaks two anchors on the way and limps it
on three broken ankles arrives in about eight; the siege's two beams give
roughly two more. A won fight lands at nine to thirteen minutes, inside the band.

---

## 10 · What it needs built

**Depends on:** P1 (a bigger rig: the species table must carry a skeleton of 26 bones
rather than 18, and breakables as their own list rather than health on every part), P2
(the valley: 50 × 240 m, boulders, standing stones, the siege line), P3 (gnawers, and a
critter that can hold a part — **built 2026-10-01**: a critter's perch on a part is
authoritative and carried by the pose, `pack::perch_on` stands one there, and one
falling lands on any mountable top; see [critters.md](../critters.md)), P4 (vents, in a part's frame), P6 (the long ride and
**fall damage**), P7 (the wall), P8 (a coop plan, and per-region windows).

**New and specific to it:**

- **The rig.** Root, fore shell, aft shell, crown, neck, head (6); six legs of hip,
  thigh, shin, ankle (24, of which the hips are fixed on the shell bones — call it 20
  that move). About 26 bones and 40 parts: 6 rim segments, 6 flank treads, 3 plateau
  plates, the crown, 3 anchors, 6 thighs (footing only when stumbling), 6 ankles, 6
  pads, the head and the plastron.
- **The gait emits moves.** A footfall is not chosen; the stride accumulator crossing a
  landing phase *is* the move's active frame, and the telegraph reads the gait.
- **Two `Doing`s**, legs and body, with `channel_gap` between them.
- **Things that live in a part's frame.** Hazards (vents), critters (gnawers), and
  proposed: the Bulwark's planted shield, the Blood mage's pools, the Reaver's waiting
  shadow. One mechanism, used five times: a part index and a local position, converted
  to the world through the pose every frame.
- **Fall damage.** As decided for the whole cast ([hazards.md](../hazards.md) §4): free
  up to 9 m (the Ridgeback's back stays free), 25 a metre past it, measured from the last
  thing stood on, halved for a slow landing. The rim is 125, the crown 325. This belongs
  to every creature, and is P6's.
- **The top-face aim rule** in `aim.rs` (§6), landed on its own.
- **The objective**: the siege line, the beam's charge, `wall_breaches`.

**Snapshot, itemised** (bytes, estimates to be replaced by `size_of`):

| Piece | Bytes |
| --- | --- |
| Monster core: position, yaw, rate, speed, strain, stride, beat, slowed, hit flag | 44 |
| Two `Doing`s (legs, body) | 24 |
| Brain: glance samples for 2 fighters, variety, 9 cooldowns, rng, think | 64 |
| Breakables: 6 ankles and 3 anchors, i16 health; 6 buckle meters | 30 |
| Phase, halt timer, stumble side and timer, opening anchor | 8 |
| Siege: beam charge, beam state, `wall_breaches` (P7) | 6 |
| Gnawer pack brain | 40 |
| 8 gnawers at 32 B, plus a mount byte each | 264 |
| 8 vents (P4, part index + local position + phase) | 80 |
| 12 falling things: shed plates and plough rubble | 96 |
| Per fighter: fall-start height, and a part-frame anchor for a shield, pool or shadow | 2 × 12 |
| **Total** | **~680** |

Under the ~700 the bestiary allotted, with about 20 to spare. The falling-things list is
the one to watch: a Shed and a Plough overlapping would want more than twelve, and the
brain's `channel_gap` is also what keeps that list short.

**Per-frame cost.** Dominated by **resolving bodies against parts**: two fighters and
eight gnawers against forty boxes is 400 box tests a frame, 3200 in a rollback burst.
Parts are grouped (fore, aft, crown, six legs) with a bounding box per group, and a body
tests only the groups it is inside, which brings it to about 80. Posing 26 bones is
cheap next to that. Aim rays meet the top faces only on a cast. `crates/sim/tests/budget.rs`
should run a Siegeshell hunt at the siege line with the pack full, which is its worst frame.

**Tests that would pin its rules:**

- `every_ring_lands_on_the_beat_and_every_stamp_off_it`
- `the_walk_never_throws_a_rider`
- `two_broken_ankles_on_one_side_bring_the_rim_within_every_hop_of_the_stair`
- `the_stumble_stair_is_climbable_by_the_bulwark`
- `a_broken_ankle_buckles_for_a_third_of_what_broke_it`
- `an_anchor_breaking_halts_it_and_cancels_the_beam`
- `a_stumble_with_somebody_at_an_anchor_opens_it`
- `the_crown_cannot_be_hit_from_the_floor`
- `nothing_it_throws_can_reach_its_own_shell`
- `the_plough_stops_on_a_solid`
- `a_thrown_rider_is_not_caught_by_the_beat`
- `a_vent_moves_with_the_plate_it_is_on`
- `a_top_face_met_from_above_is_a_place_and_from_below_is_not`
- `a_fall_from_the_crown_costs_what_the_table_says`
- `the_wall_falls_on_the_second_breach`
- `the_siegeshell_fits_in_the_snapshot` (with the pack and every list full)

**Milestones** (a few hours of AI time in all, after P1–P8 exist):

1. **M1 — the body walks.** The species entry, the rig and the valley. It walks the
   valley on the tripod gait and stops at the siege line. Checkable: `beastcheck` prints
   every height in §1 and every tread, and a screenshot of it in the valley.
2. **M2 — the beat.** Footfall rings and pads from the gait, the markers, the sound on
   the stride clock. Checkable: the first two tests above, and a hunt where a fighter
   who jumps on the beat takes nothing.
3. **M3 — the legs.** Ankles, breaks, limps, buckles, the stumble and its stair; Stamp and
   Drag. Checkable: the stumble tests, and the harness's legs plan breaking a side.
4. **M4 — the shell.** Part-frame hazards and critters: vents, gnawers roosting and
   dropping, Shrug and Shiver, fall damage. Checkable: the vent and fall tests, and the
   crown plan reaching an anchor.
5. **M5 — the crown and the clock.** Anchors, halts, phases, the Opening, the Siege beam,
   the wall. Checkable: the anchor and wall tests; a full solo hunt runs to an end.
6. **M6 — the aim rule.** The top-face change in `aim.rs`, with the Ridgeback's twelve
   seeds re-run and the difference written down. Checkable: its test, and the Ridgeback
   report beside the old one.
7. **M7 — two hunters.** The coop plan, the call, per-region windows. Checkable: the
   report in §9, and the numbers set against the tier-5 targets.

---

## 11 · In the world

**The Last Valley**, the end of the world's road: a long cut between cliffs that
narrows toward a walled town. The floor is packed earth with boulders every thirty
metres or so (the Plough's cover, and the only solids the valley has), and standing
stones every forty that are the clock. The wall at the end is the town's, and after
the first beam, a gate's rubble. The creature's path is worn into the floor as a pair
of long ruts, so from the first frame you can see where it is going.

**The trophy:** a shard of an anchor, set into the town wall above the gate — the gate
the hunt did or did not lose. It is the world's record that the last fight was won.

**The sidegrade: Keel** — a common-mechanic modifier in the sense of
[../parked.md](../parked.md). Bracing grips at four times rather than twice, and a
braced rider walks at full rider speed instead of a crouch walk; in exchange, your
full hop is 15% lower everywhere. It makes you better at the brace answer and worse at
the jump answer — on every creature you can ride, and in versus, where a lower hop is a
real cost. A playstyle, not power: it moves you from one answer to the other.

---

## 12 · Open questions

1. **The fail state.** The hunt is lost on the second breach, and nothing carries over.
   Should the town keep the damage — a district lost until the Siegeshell is beaten —
   which is more world and more weight for a first failure?
2. **The Elementalist's structure jump clears the crown from the floor.** Keep it and
   let fall damage price the overshoot, or cap what a stone launch can carry near a
   creature? It is her identity or the fight's hole, and it is a decision.
3. **The Dual mage's wings reach the standing rim.** Is "she does not need the stumble"
   her identity here, as the float was on the Ridgeback, or does the harness show her
   solo win rate is several times anybody else's?
4. **The top-face aim rule changes the Ridgeback.** It is the right rule for a shell you
   stand on; it also means a skillshot aimed at the Ridgeback's back from a platform
   lands on the back. Is that acceptable, or does the rule need to be riders-only?
5. **Is 0.8 m/s visibly walking?** A forty-metre body at under a metre a second may read
   as standing still, and the ride "at scale" is about the landscape moving. The lever
   is stride length against beat.
6. **The beat is a jump every three seconds for ten minutes.** Is that a rhythm or a
   chore? The answer decides whether the Walking phase's beat stays at 180 frames or
   the rings shrink to the tripod's own legs.
7. **One in twenty solo.** The bestiary asks whether tier 5 should be soloable at all.
   This fight's answer is "yes, by the kneels, rarely" — is that a floor or a wall for a
   person?
8. **Three classes want things to live on a part** (a shield, a pool, a shadow). Build
   the one mechanism and give it to all three, or give none of them and accept that the
   Bulwark and the Blood mage climb by the stair like everybody else?
