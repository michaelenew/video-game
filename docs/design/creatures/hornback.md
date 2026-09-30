---
status: proposed
proposed: 2026-09-30
tier: 1
---

# Hornback herd — the herd is the terrain, the bull is the fight

Eight grazing cows and one bull on an open meadow with rocks in it. Walk past at a
distance and nothing happens: the herd grazes, the bull lifts its head and watches you
go. Come close, or hit one, and the herd bunches behind the bull, the bull puts itself
between you and them, and the fight is on. This is the first creature in the cast that
is not simply an enemy. It is an animal defending something, and a player can decline
the fight by keeping their distance. That is the ecology [world.md](../world.md) is
built on.

It is the cast's lesson in **using the arena**: solids, lanes and high ground. The
bull's biggest move ends in a stun when it runs into something solid, so the fight is
about where you stand relative to the rocks. The herd's stampede is a moving wall that
splits around those same rocks. It is tier 1, the second fight a new player should
meet after the [Gnawers](gnawers.md), and it sits in the [bestiary](../bestiary.md)
§2 cast. Everything here assumes the machinery of the Ridgeback in
[monsters.md](../monsters.md) and the shared pieces P1–P8 in
[bestiary.md](../bestiary.md) §3.

---

## 1 · What the fight is

**The herd is the terrain; the bull is the fight — and the terrain is the weapon you
turn on it.**

The **cows** stand 2.2 m at the back, are 3.5 m long and 1.2 m wide. A standing full
hop apexes at 2.68 m on the Bulwark, so **every class can get onto a cow's back**,
with half a metre to spare for the lowest jumper. Grazing, a cow walks at 1.5 m/s.
Bolting or stampeding, it runs at **10 m/s**. That is faster than your walk of 7, so
you cannot outrun a stampede. It is slower than a dodge's 17, so you can cross in
front of one if you commit. There are eight of them.

The **bull** stands 3.2 m at the shoulder and is 4.5 m long and 1.8 m wide. Its horns
spread 2.4 m tip to tip. Its shoulder is above a standing hop for the Bulwark and
inside one for the other five. That does not matter, because **the bull is not a
surface**: nothing stands on it (§10 says why). It walks at 5 m/s and trots at
**11 m/s**, which catches a walking fighter. Its charge runs at **18 m/s**, just
faster than a dodge. It has **3500 health** (first guess) against the Ridgeback's
7000, and its hits do a tenth to a fifth of a fighter's 1000, where the Ridgeback's
do a fifth to a quarter. That is what tier 1 means.

Its one large, earned window is the **stun**. If the bull's charge meets a solid
(a boulder, the bank's face, an Elementalist's stone, a planted Bulwark shield), it
stands dazed for **120 frames**, head down and horns in the dirt. Nothing it does
can hurt you during that time, and its head is low enough to hit.

The loop:

1. **Approach.** It is not hostile yet. Choose where to start: near a boulder, not in
   open grass. The fight begins when you come within 12 m of any animal or hit one.
2. **Stand in front of something solid.** The charge wants a target 6–14 m away, and
   it runs 12 m straight. With a rock 2–4 m behind you, the lane it draws on the
   floor ends in that rock.
3. **Wait for the head to drop, then step out.** It paws twice. The windup follows
   you until the second paw lands. After that it cannot turn. Leave the lane after
   the lock and before it arrives; leave before the lock and it follows you.
4. **Spend the stun on the head.** For 120 frames the horns are at knee height and
   the Guard cannot come up. Damage to the head goes into the **horns** as well as
   into the bull. Enough of it breaks one horn, and that is permanent (§4).
5. **Handle the herd.** At any range the bull can **Bellow** and send the herd down
   a lane through you. Get onto high ground, into the lee of a solid, or onto a cow.
   A ridden cow carries you home, and home is **behind the bull**.
6. **Kill the bull.** The herd flees the arena through the ford, and the fight is
   over.

Each boulder breaks after two charges, so the rocks are spent. The fight you have in
minute two is played on a field you changed in minute one.

## 2 · The moves

Frames at 60 per second. All numbers are **first guesses**, to be moved by the fight
report rather than by argument.

| Move | Range | What it is | Tell (frames) | The answer |
| --- | --- | --- | --- | --- |
| **Paw & charge** | long, 6–14 m ahead | Paws twice, then drops its head and runs 12 m straight at 18 m/s. Cannot turn once running | 36: two paws of 18. It tracks through the first paw, and the yaw **locks on the second** | **Stand in front of a solid, and step out of the lane after the lock.** Into a solid it is stunned for 120 |
| **Hook** | close, 0–3 m, front 120° | Head drops and cocks to one side, then the horns sweep up through you | 22, head low and cocked toward the side the horn comes from | **Dodge through it toward its flank**, the side the head is *not* cocked to |
| **Trample** | close, on a fighter lying down | Rears on its hind legs and stomps where you lie | 30 at least, and never sooner than 16 after you can move again | **Leave the spot the moment you can act.** Do not swing, block or get up in place |
| **Shoulder** | close, 2.5 m to the flank you are on | Leans away from you, then throws its whole flank sideways | 18, the lean away | **Hit it first.** Any hit during the lean flinches it out of the move |
| **Guard** | close, front 120° | Braces its horns forward, low and square, for up to 90 frames | 8 to brace, and it only braces against a swing it *saw coming* | **Go round it**, or break it: `Q`, Earthbreaker, a loaded Slam |
| **Bellow** | any, lane 8 m wide along its facing | Head up, neck swells, a long call. The herd stampedes down a lane through you | 50, the loudest thing in the fight | **Get above it or behind something**: high ground, the lee of a solid, or a cow's back |
| **Cow kick** | 2 m directly behind any cow, 40° wedge | Head down, tail flicks, both hind legs straight back | 20 | **Pass a cow at its side, never behind it.** Positional: be off its tail line |
| **Buck** | aboard a cow | Head drops, rump kicks up. Throws a rider who is not braced | 20, the head going down | **Crouch to hold the first; jump before the second** |

Startup / active / recovery and damage (first guesses):

| Move | Startup | Active | Recovery | Damage | On hit |
| --- | --- | --- | --- | --- | --- |
| Paw & charge | 36 | up to 40 (12 m) | 50 on a miss; **120 stunned** into a solid | 180 | knockdown, 40 |
| Hook | 22 | 5 | 40 | 110 | launched ~3 m; lands as a 30-frame knockdown |
| Trample | 30+ (see below) | 4 | 45 | 150 | — |
| Shoulder | 18 | 4 | 30 | 60 | shoved 3 m |
| Guard | 8 | held ≤ 90 | 20 | 0 | attacker bounces: recoil 14, pushed 1.5 m |
| Bellow | 50 | the stampede, ~3–4 s | 60, head high | 70 per cow | the first cow knocks down, 40 |
| Cow kick | 20 | 4 | 30 | 80 | knockback 3 m |
| Buck | 20 | 24 | — | 30, the fall | thrown off the side |

**The charge is walked out of, and only after the lock.** From 12 m away the run
takes 40 frames. A walk of 7 m/s covers 4.7 m in that time, and the lane is 2 m wide.
That means a player who waits for the head to drop has time to spare. This is on
purpose: the charge is a tier-1 lesson in **timing a commitment**, not a reflex check.
The failure it teaches is leaving *early*. During the paws the windup turns toward
where you will be (at `startup_tracking`, as on the Ridgeback), so a player who
sidesteps on the first paw is simply charged in their new spot. The dodge also works,
as it works everywhere, but a dodge through the charge wastes it. The bull skids 50
frames and turns, and nothing is stunned. The rock is the reward, and the dodge is
only the escape.

**The lane is the whole run, drawn at the first paw.** The bull does not decide how
far to run. It runs 12 m (16 when desperate, §4) or until something stops it. So the
marker is exact. It draws where a solid will end the run, and it draws the lane
cut short there, in a different colour, so a player can see the rock will do its job.

**The Trample never lands on somebody who could not have moved.** Its startup is
`max(30, stagger_left + 16)`. It rears through whatever is left of your knockdown and
comes down no sooner than 16 frames after you can act. The Ridgeback's set-ups are
true follow-ups, because it is a tier-3 animal. This one is a tier-1 teacher, and what
it teaches is that the ground is not a place to stay. The answer is movement, not a
dodge. Dodging out of a knockdown works too, but walking is enough, and a swing thrown
from the floor roots you under the hooves.

**The Guard is a reaction to something it saw, and that is the lesson.** The bull
glances every 14 frames. It braces when its last glance saw a fighter in front of it,
within 4 m, **in the startup of an attack**. That is a pose it can see, not a button
it can read. A fast poke's startup is usually over before the next glance sees it, so
it lands. A slow heavy swing thrown into its face is seen and bounces off. A frontal
hit on the Guard does no damage, and the bull slides back half a metre, which is a
blocker's pushback. The attacker recoils for 14 frames, and after a bounce the bull
wants a Hook (`combo_appetite`). This is what the Bulwark does to you in versus.
Guard breakers work as they do in versus: `Q` specials, the Champion's Earthbreaker,
and a Bulwark Slam carrying weight. Any of them breaks the Guard into a 40-frame
stagger. A hit from more than 60° off the bull's nose ignores the Guard entirely.
**Go round** means the flank, and the flank is where the Shoulder is.

**The Shoulder is the one move answered by hitting it.** It is the one move where a
flinch always wins. It is declared `interruptible` in the move table, so any hit
landing in its 18-frame lean cancels it into an 18-frame flinch, whatever the strain.
It is the fight's introduction to the interrupt that §4 of monsters.md makes a
negotiation on big animals. The tell is short enough (18 frames) that it will
sometimes land. The answer that always works is a poke already in flight, which is
the rhythm the flank teaches.

**The stampede only hurts in a drawn lane.** Outside a Bellow, cows *push* and never
damage. A cow walking home, a cow milling, and a cow bolting in alarm all shove a
fighter aside like any body. Inside the lane, the first cow to touch you knocks you
down for 40 frames. **Cows do not tread on the fallen.** A cow steers over or around
a body lying in the lane, so a stampede does at most 70. What makes the knockdown
dangerous is that the bull may be near enough to Trample.

## 3 · A threat at every range

**Close, 0–3 m.** In front: the Hook, and the Guard against anything slow. At the
flank: the Shoulder. Behind the bull: **the herd**. When it is home, the herd grazes
6–10 m behind the bull, and the ground there is full of cow tails. On the floor after
any knockdown: the Trample.

**Mid, 3–14 m.** The charge, which is chosen for 6–14 m and is the fight's centre of
gravity. The trot, at 11 m/s, closes on a walker at 4 m/s net. Standing at a range
where nothing reaches you gets you charged.

**Long, past 14 m.** The Bellow reaches the whole arena. The lane runs to the arena's
edge. Past 20 m the bull trots to close, and against a target moving away it adds the
target's speed to its own (`pursuit_gain`, as the Ridgeback does), up to its trot. A
ranged class backpedalling across open grass is given a stampede.

**Aboard a cow.** The buck is the ride's price. The bull is the other threat. **The
Hook reaches a rider.** A rider's body spans 2.2 to 4.0 m, and the Hook's volume rises
to 3.5 m inside its 3 m reach. A cow carrying you home past the bull's nose is a cow
carrying you into a Hook. Ride home, and jump off before you pass in front of it.

**The tempting safe spots, and what covers each:**

- **A boulder's top** (2.0 m). It is out of the stampede, since the herd splits
  around a boulder. The Hook reaches it. A charge into the boulder you stand on shakes
  you off: 40 damage and a 20-frame stagger, landing beside a bull that is stunned
  for only 60 frames, because a charge into an occupied rock is not a clean hit. It
  also cracks the rock. Two charges and there is no rock.
- **The bank** (1.5 m, along one side). It is out of the stampede. The bull will not
  charge at a target above it, because the lane would end in the bank's face, which
  is the stun. Instead it walks up the ramp at either end and charges *along* the
  bank, which is 4 m deep and has nowhere to step out to. The bank is the answer to
  one Bellow, not a place to live.
- **Inside the herd.** Cows push you around, and every cow behind you is a kick. The
  bull's charge runs straight through the herd, because **cows part for the bull**:
  they steer out of its lane, so the herd is never a solid for the bull.
- **Directly behind the bull.** This is covered by the herd while the herd is home.
  After a Bellow the herd is away for three or four seconds, and **the bull's rear is
  open**. That is a real opening, and it is what the Bellow costs the bull.

## 4 · The approach and the window

**What you earn.** The stun, 120 frames, every time a charge meets a solid. It is the
big window and the Ridgeback's topple in miniature: all angles, no Guard, no moves,
head at knee height. It takes about 40 frames to reach the head from where you
stepped out of the lane, which leaves about 80 frames of damage. That is a full
three-hit chain and a half for a Champion.

**What you break, and what that changes for the rest of the fight:**

- **The horns.** Each horn has 500 health (first guess), and only damage to the head
  counts. The head is behind the Guard and three metres up except while stunned, so
  in practice the horns break in the stun windows. The horn that breaks is the one on
  the side the damage landed. A broken horn **opens its half of the Guard**: frontal
  hits on that side land for full damage. It also makes the Hook to that side a
  shove with no launch, so no Trample follows it. Both horns broken, and the bull has
  no Guard at all. The player can see it: the horn is gone, and the Guard pose
  visibly covers one side.
- **The rocks.** A boulder cracks on the first charge and shatters on the second,
  leaving rubble 0.4 m high that is not a solid. There are four boulders, so eight
  full stuns are built into the arena. Stones and shields are the rest.
- **The herd.** A cow that takes 120 damage (`cow_nerve`) bolts out through the
  ford and does not come back. Every cow gone makes the stampede thinner: three
  abreast at eight cows, two below six, one below four. The rear also becomes less
  covered, and there is one fewer ride. It costs time spent not hitting the bull, and
  it costs kicks. Whether it should cost anything in the world is §12.

**It learns a wall once.** After a stun the bull is **wary** for 600 frames
(`wary_frames`). During that time it scores the charge at nothing when its lane
passes within 2 m of **the solid it just hit** (its index is one byte in the
snapshot). The same rock twice in a row is a mistake it makes once. The player moves
to another rock, or raises one, and that movement is the fight. Without this, a
tier-1 fight would be one boulder and a metronome.

**The arc, fresh to desperate.** Strain works as it does on the Ridgeback, at critter
scale: `cc_strain` 150 and `interrupt_strain` 300 at full health, both falling with
`strain_desperation`.

- **Fresh, above half.** It charges from 6–14 m with the full two-paw tell and
  bellows only when the herd is home.
- **Below half.** The run grows to **16 m** (`desperate_run`). A missed charge can
  chain: it skids, turns, and charges again with one paw, an 18-frame tell. That is
  still above reaction, and the lane is still drawn. Stepping out of the first lane
  into open grass is now a mistake. Step out toward the next rock.
- **Below a quarter.** The herd rallies. It stops grazing 8 m behind the bull and
  mills around it at 4–6 m (`rally_radius`), so the flanks are kick country as well
  as the rear. The fight narrows to the front, the charge and the rocks you have left.
  That is the right ending for a fight about rocks: it asks what you spent.

## 5 · The brain

The Ridgeback's control algorithm with three changes. The herd adds a second,
simpler brain (the P3 pack brain) that the bull steers.

**It is not hostile until alarmed.** Before alarm there is no target. The herd
grazes, and the bull faces the nearest fighter inside 20 m (`watch_radius`). That
turn of the head is the only tell that you are being watched. Alarm is any fighter
within 12 m of any animal (`alarm_radius`), or any hit on any animal. A fighter who
backs out past 25 m before alarm is never fought. After alarm the first 120 frames
(`hunt_grace`) are spent interposing, with no moves thrown. If every fighter stays
past 25 m for 600 frames (`calm_frames`), the herd calms and the fight is abandoned
rather than lost.

**It interposes rather than approaching.** Its movement goal is the point on the
line from the herd's centre to its target, `guard_standoff` (7 m, first guess) short
of the target. The bull stands between you and the herd and turns to face you. That
standoff is the middle of the charge's range, so the bull's resting position is a
threat. It trots to close only past 20 m.

**Glance and lead.** `glance` 14 frames and `lead` half the Ridgeback's. That makes
it easier to walk around than the Ridgeback, which is the tier. Cutting across its
nose between glances is still the skill the pair teaches.

**Scoring.** At a decision point, every move is scored:

```text
U(m) =  range(m)          tent on the move's window, as the Ridgeback
      + arc(m)            the move's cone: front for Hook and Guard, flank for Shoulder
      + variety(m)        decaying penalty on the last move
      + hurt(m)           wounded animals commit harder
      + downed(m)         Trample only: large when the target is knocked down within
                          4 m, and exactly zero otherwise; it is only ever a follow-up
      + seen_swing(m)     Guard only: large when the last glance saw a fighter in front,
                          inside 4 m, in an attack's startup; zero otherwise
      + herd_home(m)      Bellow only: zero unless the herd is home and not running;
                          plus `bellow_appetite` when the target has no solid within 4 m
      ⟂ wary(m)           charge only: nothing at all if its lane passes the solid it
                          last hit, while wary
      ⟂ cooldown(m)       a move on its lockout scores nothing
```

`herd_home` makes the Bellow a reply to a player standing in the open. A player who
stays near rocks gets charged. A player who leaves the rocks gets stampeded. That
split is the lesson of the fight, and one scoring term carries it.

**Decisiveness** 0.7, a little lower than the Ridgeback's, so its choices are less
predictable. The tier-1 part is in the tells, not in predictability.

**Cooldowns.** The Bellow is locked for 600 frames after the herd returns home, so a
stampede is an event and not the weather. The charge is locked for 90 frames after a
stun ends, so the bull's first act after a window is a turn and a walk, not a paw.
The Guard is locked for 120 frames after it is broken.

**The herd's brain** is five states on the pack brain: *grazing*, *alarmed* (bunch
up and drift to home behind the bull), *stampede* (run the lane), *returning* (wheel
at the lane's end and come home along the arena's edge), and *rallied* (below a
quarter, §4). A cow's kick is its own decision, not the pack's. A cow kicks when a
fighter has stood in its rear wedge for a whole glance (a cow glances every 20
frames, staggered across the herd). You can walk behind a cow. You cannot stop there.

**Flocking.** Every cow steers by the sum of four terms, clamped to its state's top
speed:

```text
v_want = separation   away from every cow within 2.5 m, by overlap
       + cohesion     toward the herd's centre, weak while grazing, strong running
       + goal         grazing: a slow wander, seeded from the snapshot
                      running: toward the lane's far end
                      returning: toward home, along the edge
       + avoid        away from solids, fighters (outside a stampede) and the bull's lane
```

Alignment is left out on purpose. A running herd gets its alignment from sharing one
goal, and that saves a pass.

## 6 · Reading it

**Every windup marks the floor**, from `Monster::telegraph`'s successor for critters
(§10), which poses the animal on its first active frame and asks its hit volume:

- **Paw & charge:** a 2 m lane, 12 m long (16 desperate), filling through the two
  paws, turning red on the head-drop. Where it ends in a solid, the stub past the
  solid is not drawn, and the solid gets a ring in the "this will stop it" colour.
- **Hook:** a 3 m, 120° arc, brighter on the side the horn is coming from.
- **Trample:** a 2 m disc on the fallen fighter, which fills to land 16 frames after
  they can move.
- **Shoulder:** a 2.5 m rectangle along the flank it will throw.
- **Guard:** drawn in the *no* colour, not the *hit* colour. A frontal arc that says
  "hits here bounce". With one horn gone it covers half the arc.
- **Bellow:** the lane, 8 m wide, from the herd's far end to the arena's edge. **The
  lee of every solid in it is drawn clear.** That is both the answer and the
  split. It is honest, because the herd is held to it (§10).
- **Cow kick:** a 2 m, 40° wedge behind the cow. It is drawn faintly all the time
  while you stand inside it, and fully through the 20-frame tell.

**Silhouette.** The bull's head is its tell. It is up and turned toward you when
watching, low and square in Guard, dropped and cocked for a Hook, dropped and level
for the charge, and thrown back for the Bellow. The paws are the big motion, and they
are **heavy**: the whole forequarter rises. A charge begins with the head going down,
which is the frame the lane goes red.

**Sound.** The paw is a thud and a scrape, twice. It is audible off-screen, which is
what makes a charge from behind the camera readable. The Bellow is a long call over a
second that starts quietly and swells. It is the one sound a new player should learn
in their first minute. The stampede is a rolling rumble whose pan says where the lane
is.

**What "unanswerable" means here.** A hit is unanswerable if its tell was under 15
frames *and* no marker had been on the fighter's spot for 15 frames when it landed.
Cows cannot score one outside a stampede, because they do no damage outside one. The
risk is a lane that comes from behind the camera. The Bellow's 50 frames and its
sound are the answer, and the report's `unanswerable` line is the check.

## 7 · The classes

**Shadow Reaver.** The shadow is her answer to the Bellow. Place it out of the lane,
dash to it, and the stampede is somebody else's problem. Her forward dodge to a
shadow placed behind the bull puts her into the open rear a Bellow leaves. The
charge is harder for her. She has to stand in front of a rock like everybody, and at
750 health a charge is nearly a quarter of her. **The dash is refused across a
solid**, so a shadow placed on the far side of the boulder she is baiting with cannot
be reached. She has to place it to the side. That is identity: two bodies means two
positions to think about. Her copy's marks land well in the stun.

**Elementalist.** She has it easiest, and that is the point: she *makes* the solids.
A stone is 1.8 m tall and 1.4 m across. It stops a charge like a boulder and
collapses doing it, which spends a slot. The play that belongs to her alone is
**raising a stone into a charge that is already running**. The run is 40 frames and
a stone rises in 14, so a Raise six metres in front of her after the head drops is a
wall the bull cannot turn from. Two limits keep it a skill and not a lock: she has to
be in the lane to do it, and the bull is wary of the stone for 600 frames. That
matters less for her, because the stone is gone. A **lit stone** burns the bull when
it hits one. A stone raised in a Bellow lane is a lee the herd splits around. This is
the first creature she should beat reliably, and it is the plan the harness should
learn for her (§9). **Question**: can a stone be raised *under* the bull? §12.

**Blood mage.** She is middle. She has no solids and no vertical answer, so she lives
by the boulders. Her blink to a pool is her way out of a lane, and her pools fall
where she cuts. So cut the bull at the edge of where the stampede will run, and the
pool is on the safe side. **Grasp on the bull** binds it when it is susceptible and
does nothing when it is not. It is a 3.2 m animal and is not hauled. **Grasp on a
cow** hauls the cow, which moves a body, and a moving body is a ride. Cutting cows
makes pools and drives them off (§4), so she can drink off the herd at the cost of
thinning it. That is a real choice, and it is hers.

**Dual mage.** She floats over the stampede. With a second jump and wings, a Bellow
barely concerns her. She is also the one class that could stand on the bull if the
bull were a surface, which is one reason it is not. The charge is her real test.
Floating over it makes it miss and costs the stun, so she has to learn to stay on the
ground in front of a rock. The fight's lesson reaches her, so it is identity, not a
hole. The thing she declines is the herd, and the herd is tier-1 terrain.

**Champion.** Easy, and a good teacher. Rush closes to the head after a stun in a few
frames, and three hits deep is what the window is sized for. **Pole vault** clears a
stampede lane, and it is his vertical answer. He has no `Q`, so his guard breaker is
Earthbreaker, the third hit of a chain. From the front the first two hits bounce, so
**he has to go round**, which is exactly the lesson. Hammer's Uproot launches cows.
Nothing in the fight asks him to, and he will.

**Bulwark.** This fight is a mirror for him, and he has it easy on the charge and hard
on the herd. **A planted shield is a solid**, so it stuns the bull like a boulder and
**loads the shield with the charge's 180**, which is the heaviest deposit in the game.
A **parry** of the charge on a held shield staggers the bull for 60 frames, half a
stun. A **block** does not stun the bull, and pushes him back 4 m with a long
stunlock, because the lesson is to plant. The Guard is his own block used against
him. The herd is his hard part. At 2.68 m his hop clears a cow's back by less than
half a metre, so riding is a timing test for him, and the bank and boulders are the
only high ground he reaches. That is identity: the wall lives on the floor.

## 8 · Coop

With two hunters, the fight becomes **bait and punish**. The bull targets one hunter
(`target_switch` 0.6 as on the Ridgeback). The other stands near the bull's flank, or
places a stone or a shield for the charge to meet. The window is shared, so two
hunters fill 120 frames with twice the damage. Numbers for two: bull health ×1.6,
Guard reaction unchanged, and the Bellow scored on *either* hunter standing in the
open. The lane goes through the target but is 8 m wide, so a partner standing near
the target shares it.

What changes in kind: a **ride home** is now a flank attack. One hunter holds the
bull's attention at the front while the other catches a returning cow and is carried
to its rear. The herd's return path becomes a route. The rally below a quarter is
tuned for one. With two hunters, kick coverage of the flanks may need to be wider,
and only a coop report can say.

## 9 · Measuring it

**The scripted hunter's plan** is what a person learns in their first ten minutes:

1. Do not stand in open grass. Keep the nearest solid 2–4 m behind you, on the line
   from the bull through you.
2. Watch for the second paw. Walk out of the lane on the head-drop, toward the next
   solid.
3. In a stun, go to the head and hit it. Leave with 20 frames to spare.
4. On a Bellow, go to the nearest lee or onto the bank, whichever is closer across
   the lane.
5. Never stand still inside a cow's rear wedge.
6. Poke fast from the front and swing heavy from the flank. Dodge the Hook toward
   the flank.
7. After a knockdown, walk. Do not swing.

Riding is plan v2. After a Bellow, catch a returning cow and jump off behind the
bull. It is in the report as an option the hunter takes one time in three, so the
ride's numbers exist.

**Fight report lines it adds** (P8):

- `charges: thrown / into a solid / dodged through / walked out of / landed`
- `stun frames, and damage dealt in them`: whether the window is used
- `horns broken`, `boulders cracked / shattered`, `cows driven off`
- `bellows: thrown / cows that touched the hunter / hunter in a lee, aloft, riding`
- `guard: braced / bounced / broken`
- `kicks: thrown / landed`
- `rides: count / mean length / ended by buck or by jump / ended in a Hook`
- `herd cost`: mean and worst microseconds a frame for the pack brain and flock

**Targets** (tier 1, from [bestiary](../bestiary.md) §2): the scripted hunter wins
**about nine in ten**, and a won fight lasts **60–120 s** solo. Four windows:
threatening **~30%**, open to a poke and open to a way in share the rest, and safe to
walk up **~30%**. A tier-1 animal should spend more of its life open than the
Ridgeback's fifth. **At least one charge in three into a solid** for the scripted
hunter, or the fight's idea is not landing. **Unanswerable hits: zero.** Landed per
move: the Hook and charge well under half, the kick rare, the Trample under one in
five.

## 10 · What it needs built

**Depends on P1** (species), **P2** (arenas as data: a bigger meadow with rocks),
**P3** (critters and the pack brain), **P8** (a plan and report per species), and
**P7** for the cart variant only (§11).

**The bull is a critter with extra state, not a light monster.** A monster's
skeleton earns its bytes when its parts are geometry people stand on and break. The
bull's are not. Nothing stands on it. Its one breakable part, the horns, is two health
counters and an arc test against its facing, and its Guard is also an arc. A full
`Monster` is about 200 bytes and a rig resolve every frame, for a body that one
yawed box describes. The cost of that choice is honest: **the bull can never be
ridden**. A bull ride would be a monster, which is §12.

**What is new and specific to it:**

1. **A critter's hurt volume is a yawed box, floor to back**, not a capsule. P3's
   capsule fits a gnawer and does not fit a 4.5 m animal. A box from the floor fills
   the gap under the belly, because a level shot under a quadruped is the Ridgeback's
   lesson and nobody is meant to shoot between a cow's legs. `aim::first_along` gains
   critters as things a shot's path can run into. That is a change to *what* it
   meets, not a new line of effect, and it lives in `aim.rs` and `math.rs` like the
   rest. **The crosshair ray is unchanged**: bodies are not on it, and close up the
   bull fills the screen exactly the way the rule was written for.
2. **A critter can carry a ride.** The ride machinery reads a part's frame. A cow
   supplies one mountable box whose pose is its position, yaw, and a pitch and heave
   from its clip clock, which is in the snapshot. The buck is a clip that pitches
   that box hard, and **the grip test throws the rider**, so the buck is
   acceleration, as monsters.md §3 requires. The first buck, at `ride_patience` 120
   frames, is sized between the grip of 450 m/s² and the braced 900. The second,
   60 frames later, is above both. The fighter's mount reference gains a critter
   variant.
3. **The charge stops on solids.** This is a swept test of the bull's box against the
   arena's solids, stones and planted shields, only while the charge is active. The
   arena's edge is not a solid for this. The bull pulls up short of it, as the
   Ridgeback does, because an edge that stunned would make the edge the whole fight.
4. **Solids with a health.** P2's arena solids gain a crack count, one byte each,
   in the snapshot.
5. **The lane is a hard constraint on the herd, and the lee is part of it.** A cow in
   a stampede is clamped inside the 8 m lane, and every solid in the lane casts a lee
   (its circle extended 4 m downstream) that cows steer out of. That is what makes the
   drawn lane and the drawn lees the hit test's own answer, not a prediction of
   emergent behaviour.
6. **A creature Guard.** This is the defence path run in reverse. The bull is the
   blocker, and the attacker takes a recoil. It reuses `apply_hit`'s `blocked`
   decision, with guard breakers read from the same flag versus uses.

**Snapshot estimate.** Nine critters at 32 B is 288. The bull's extra state is about
40 B: horns 2×2, stun, wary solid, lane origin and direction, guard timer, charge run
left. The pack brain with the herd's state, lane, home point and timers is about
32 B. Four boulder crack counts. The rider's critter index is one byte on the fighter.
**About 365 B**, against bestiary §4's estimate of 350.

**The per-frame cost, bounded.** Flocking is neighbour queries, and with a herd
capped at eight there is no grid. A grid for eight bodies costs more than the pairs.
Per frame:

- separation and cohesion: 28 cow pairs, each a squared distance and a compare;
- solids: 8 cows × at most 13 solids (4 boulders, the bank, 3 stones for each of two
  players, 2 shields) = 104 circle tests, plus the lee test on the same pairs while
  stampeding;
- fighters: 8 cows × 2 = 16 capsule pushes and kick-wedge tests;
- the bull's lane-clear check: 13 tests, **once a glance**, not once a frame.

That is about 160 fixed-point tests, with **no ray casts at all** and no allocation.
The herd's arrays are fixed at `[Critter; 9]`. Estimated at a few microseconds a
frame, and ×8 for a rollback burst that is still under 2% of one `advance`'s 520 µs.
The number is bounded by constants (herd ≤ 8, solids ≤ 13), not by what happens in
the fight. `budget.rs` gets a stampede frame to hold it there.

**Tests that would pin its rules:**

- `the_herd_is_not_hostile_until_somebody_comes_close_or_hits_one`
- `the_charge_follows_you_until_the_second_paw_and_not_after`
- `a_charge_that_meets_a_solid_stuns_for_the_whole_window`
- `a_charge_pulls_up_short_of_the_arena_edge`
- `a_boulder_takes_two_charges_and_the_second_breaks_it`
- `a_stone_and_a_planted_shield_stop_the_bull_like_a_boulder`
- `the_bull_does_not_charge_the_solid_it_last_hit_while_wary`
- `the_trample_lands_no_sooner_than_a_reaction_after_you_can_move`
- `a_frontal_hit_on_the_guard_bounces_and_a_flank_hit_lands`
- `a_broken_horn_opens_its_side_of_the_guard`
- `any_hit_in_the_shoulders_lean_interrupts_it`
- `a_cow_kick_only_reaches_behind_a_cow`
- `cows_do_no_damage_outside_a_stampede`
- `no_cow_leaves_the_lane_it_was_called_down`
- `no_cow_enters_the_lee_of_a_solid_in_its_lane`
- `what_is_drawn_for_a_stampede_is_where_the_herd_can_be`
- `every_class_can_jump_onto_a_cow` (printed by `beastcheck`, as the Ridgeback's routes are)
- `a_braced_rider_holds_the_first_buck_and_not_the_second`
- `the_herd_leaves_when_the_bull_dies`
- `a_stampede_frame_fits_the_advance_budget`

**Milestones** (each ending in something checkable):

- **M1 · The meadow and the grazing herd.** P2's meadow as data (48 × 40 m, four
  boulders, the bank, the ford). Nine critters that graze and flock, and alarm that
  triggers by radius and by a hit. *Check:* the alarm test. A screenshot of the herd
  grazing. `determinism.rs` green.
- **M2 · The stampede.** Lane, clamp, lee, the return home, the knockdown and the
  "no treading on the fallen" rule. Floor markers. *Check:* the lane and lee tests,
  a `SHOT_MOVE=bellow` screenshot, and the budget test.
- **M3 · Paw & charge.** Tracking, lock, the swept stop on solids, the stun, cracks,
  wary. *Check:* the charge tests, and a `SHOT_MOVE=charge` screenshot showing a lane
  ending in a boulder.
- **M4 · The close game.** Hook, Trample, Shoulder, Guard with horns, cow kicks,
  strain at critter scale. *Check:* the guard, horn, trample and kick tests.
- **M5 · Riding a cow.** The mountable box, the buck clip, grip. *Check:* the buck
  test, and `beastcheck` printing a cow's back against every class's hop.
- **M6 · The brain and the report.** Scoring, interpose, desperation, the hunter's
  plan, the report lines. *Check:* twelve hunts against the tier-1 targets in §9.
- **M7 · The cart.** P7 and the defend variant (§11). *Check:* its own twelve runs.

## 11 · In the world

**Region.** The low meadows: grass, scattered granite, a river crossed at a ford.
The Hornback herd migrates across it, and it is the first region past the starting
ground.

**The arena.** A meadow **48 × 40 m** (P2). Its edge is a slope and thicket the bull
pulls up short of, not a wall. There are **four boulders**, each 2.5 m across and
2.0 m tall, so every class can stand on them and none is a stun if the fight has
worn it down to rubble. There is **one bank** along the north side, 1.5 m high and
4 m deep, with ramps at both ends. There is a **ford** in the east edge where the
herd leaves. The herd grazes in the south half, and a fighter arrives from the west.
The ground between you and the herd is open. The fighter's first decision is
therefore which rock to walk toward.

**The defend variant: the crossing.** A cart with 1500 health (P7) travels a road
across a longer meadow (80 × 36 m) at 2 m/s. It moves only while a hunter is within
6 m of it, so escorting it is also steering it. The herd **migrates across the
road** in waves on a fixed schedule. Each wave is a stampede lane drawn on the road
ahead 5 s before it runs. The bull, alarmed, **treats the cart as a threat**: when
the cart is nearer the herd than any hunter, the bull targets the cart. It paws and
charges it, 300 to the cart and stunned, because the cart is a solid. The hunters'
jobs are to stop the cart short of a crossing, which means leaving it and so
stopping it; to hold the bull's attention by hitting it (a hit raises its interest
past `target_switch`); and to put solids between the cart and the lanes. An
Elementalist's stone in front of the cart is a lee the herd splits around. **Win**:
the cart reaches the far side. **Lose**: it breaks. Killing the bull ends the herd's
aggression and is allowed, but it is not required. The harness learns to defend
with one plan: stand between the bull and the cart, stop the cart before every drawn
lane, and bait charges away from the road.

**What beating it earns.**

- **Trophy: the horn.** The one you broke, with the break showing, or both if you
  broke both. If you killed it with its horns whole, you get the whole horned skull,
  which is the harder trophy and says so.
- **Sidegrade: Hornback hide**, a common-mechanic modifier on the dodge in the
  spirit of [parked.md](../parked.md). **The dodge is a barge**: 2 of its 10
  invulnerable frames are gone, and a dodge that passes through a body shoves that
  body 2 m aside with no damage. It changes what the dodge is for, from escaping to
  making room. It is worse at the thing a dodge usually does, so it is a sidegrade,
  not power.
- **The crossing** earns the carter's key, which is passage to the next region along
  the road. In the world the migration is also a timetable: the road is closed while
  the herd is crossing.

## 12 · Open questions

1. **Should the arena's edge stun?** The seed said a wall would. This design says no,
   because an edge that stuns makes the edge the whole fight, and the boulders being
   spent is the fight's arc. If the meadow feels too open, the lever is more boulders,
   not a stunning edge.
2. **Can the Elementalist raise a stone under the bull?** A stone raised under a body
   pushes it up. Lifting a 3.2 m bull onto a 1.8 m stone would be spectacular and a
   lock. The proposal is that the bull is too heavy, and a Raise under it is
   refused. Or does it tip over, as a small topple, as her reward?
3. **Is the charge too easy to walk out of?** 4.7 m of walk against a 2 m lane is
   generous on purpose for tier 1. A person will say whether it reads as a lesson or
   a formality. The lever is the run's length against the tell (`charge_speed`,
   `paw_frames`).
4. **Should driving cows off cost anything?** In the fight it thins the stampede and
   the rides. In the world, a herd you thinned could stay thinner next time you
   pass. That is ecology, and also a permanent consequence in a game that has avoided
   them.
5. **Should the bull be rideable?** The critter decision rules it out. A bull ride,
   braced against its charge into a rock, is a strong image. If wanted, the bull
   becomes a light monster (P1, about 200 B) and the ride is the Ridgeback's.
6. **Does the rally below a quarter make the ending a squeeze or a wall?** Only play
   says. The alternative is that the herd leaves the bull at a quarter, and the last
   quarter is one animal alone.
7. **Is a declinable fight right for a tier-1 teacher?** A new player who walks
   around it learns nothing about solids. Should the road, or the region's gate, go
   through the herd's grazing ground?
8. **Two seconds of ride.** `ride_patience` 120 frames carries you about 20 m, which
   is home from most of the arena. Whether that is a trick or a toy is a person's
   question.
