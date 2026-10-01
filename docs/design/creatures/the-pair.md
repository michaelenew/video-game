---
status: built
proposed: 2026-09-30
built: 2026-10-01
tier: 3
---

# The Pair — two cats, one of them always behind you

Two big cats, bonded, that hunt as one animal with two bodies. Either of them
alone is a fair fight a decent player wins. Together they are built around one
fact about a third-person camera: it shows you about ninety degrees of the
world, and there are two of them. One holds your attention in front; the other
goes where you are not looking. The fight teaches the thing the camera makes
hard — **know where the other one is before you commit to this one** — and it
pays you for learning it with the one big opening in the set: the move where
they both jump at you at once.

This is creature six in [the bestiary](../bestiary.md), the first fight with two
full monsters in the snapshot, and the test that the Ridgeback's rig
([monsters.md](../monsters.md) §6) really is data. Everything here reuses the
Ridgeback's glance, scoring, strain, telegraph markers and fight report, and
says so where it does.

---

## 1 · What the fight is

**Either one alone is fair; together, one is always behind you.**

Each cat is **1.8 m at the shoulder, 4 m nose to tail**, runs at **12 m/s**
against a fighter's walk of 7 and dodge of 17, and has **2200 health** — 4400
between them, against the Ridgeback's 7000. The numbers are chosen against the
camera and the fighter rather than against the Ridgeback:

- **1.8 m at the shoulder** puts the back at a fighter's head height and the
  head at about 1.5 m, so everything on the animal is reachable by a level
  swing and by every class's hop (2.7 m on the Bulwark). There is no climb.
  Nothing is out of reach; the difficulty is entirely in *when* you can reach
  it without being hit from somewhere you are not looking.
- **4 m long** is small enough that both cats fit in the camera's view at
  about eight metres and large enough that one at your shoulder fills a third
  of it. The camera's vertical field is 58°, about 90° horizontally at 16:9,
  so a cat more than ~45° off your look direction is off the screen. The whole
  fight lives on that number.
- **12 m/s** is faster than a walk and slower than a dodge. You cannot walk
  away from either cat, and you can dodge out from between them — once.
- **2200 health each** is about three Ridgeback feet. A decent player's damage
  into a target that is only ever safe to hit for short windows puts a won
  fight at three to five minutes, inside tier 3's three-to-six.

They are not identical, and must not look it: one is **dun** and one is
**dark**, so two players can say "the dark one's behind you" and be
understood. Their numbers are the same.

**Two roles, one of each at a time:**

- The **Holder** stays in front of you, five to six metres off, and mirrors
  your sideways movement. It stands where you are going, not where you are,
  which is what "blocks your retreat" means in practice: backing off walks you
  into its lead. It feints.
- The **Striker** circles to the side of you **opposite the Holder** and
  pounces when you commit.

**"Behind you" is defined by position, never by your look.** The Striker aims
for the far side of you from its partner. That is behind you only if you are
looking at the Holder — which is what a person does, and which is exactly the
habit the fight is teaching them out of. The cats never read where your camera
points (§5, "It does not read your look").

They **swap roles** every seven to ten seconds, and the swap is visible: the two
cross paths, and for about forty frames neither is attacking.

So the loop is:

1. **Read the pair, not the cat.** Turn the camera until both are on screen, or
   until you know exactly where the one you cannot see went. Most of the fight
   is spent strafing so that the two of them end up on the same side of you.
2. **Split them.** A pillar, a stone, a planted shield or a second player
   between you and one cat means it cannot see you (§5), and a cat that cannot
   see you is not a Striker. One cat at a time is the fair fight.
3. **Punish the one in front only while you know where the other is.** The
   Holder's recoveries are long enough to hit. Whether *you* can afford to hit
   it is a question about the other one.
4. **Bait the twin pounce.** Standing between them is the thing the whole
   fight tells you not to do, and it is the only way to make them throw the
   move that knocks them both flat.
5. **Kill one, and then fight the other square.** The survivor enrages, stops
   hunting as a pair, and comes straight at you. The fight turns from a
   positioning puzzle into a reaction duel.

## 2 · The moves

Every cat has all nine; which ones it throws depends on its role (§5). Frame
numbers and damage are **first guesses**, sized against the Ridgeback's (a bite
is 30 / 4 / 90 for 190) and against a dodge of 22 frames with 10 invulnerable.

| Move | Range | What it is | Tell (frames) | The answer |
| --- | --- | --- | --- | --- |
| **Pounce** | 5–9 m | Haunches drop, tail flicks, and it leaps 8 m in an arc, landing forepaws first and **skidding three metres on** with the claws out. 24 / 20 (live on the last 6 of the descent and the 8 of the skid) / 50. 150 | 24 — the haunches drop at frame 0, **the tail flicks at frame 10** | **Dodge toward it, under the arc.** Its belly is two metres up at the top. You come out behind it, and its recovery faces the wrong way. Dodge away and the skid catches the end of your dodge |
| **Rake** | 0–3 m | A forepaw swipe, then the paw **stays cocked** for a varying beat and comes down again. 16 / 4, then a hold of 6–30, then 15 / 4 / 40. 70 + 90 | 16 for the first; for the second, **15 after the cocked paw starts to drop** | **Take the first, then wait.** The delay is the read, as with Nargacuga: a paw still up means it is not finished. A counter thrown into the hold eats the second hit |
| **Swat** | 0–2.5 m, 60° in front | A short cuff off the shoulder. 12 / 3 / 20. 50 and a 3 m shove | 12 — **below reaction**, on purpose | **Don't stand at its nose.** Answered by where you are, like the Ridgeback's stomp: the face belongs to the Holder, and the space in front of it is not yours |
| **Feint** | 5–9 m | The Holder coils exactly as for a pounce, and does not go. 24 of coil, then 10 of standing up. No damage | 24, **with no tail flick at frame 10** | **Do nothing.** The tail is the tell. A dodge here is the thing the Striker is waiting to see (Ambush) |
| **Ambush** | 4–10 m, from behind | The Striker's lunge: belly to the floor, then flat along the ground at 25 m/s in a lane 1.4 m wide, aimed where its last look says you are *going*. 20 / 10 / 45. 170 and a knockdown | 20 — the **lane is drawn under you**, from behind, into the screen | **Step out of the lane sideways.** Walking at right angles clears 1.4 m in the time left after a reaction. A second dodge along the first one's line is what its lead expects |
| **Tail trip** | 1.5–4 m, flanks and behind | A low sweep of the tail round the hips, to the side you are on (the Ridgeback's mirrored clip rule). 18 / 6 / 30. 40, and **tripped for 70 frames** | 18 — the tail drops flat and the hips swing away first | **Jump it.** Tripped lasts long enough for its partner's rake to arrive, and the partner knows (it presses a set-up, §5) |
| **Perch, then Dive** | 6–14 m | It leaps onto anything standable with a top at least 1.5 m wide within 10 m — a platform, the arena wall, an Elementalist stone — and dives from the lip. Dive 30 / 16 / 60. 160 | 30 — head down over the lip, shoulders rolled forward | **Get under the lip.** Inside three metres of the thing it is on, the dive cannot angle down steeply enough to reach you, and it has to come down the slow way |
| **Twin pounce** | both, from opposite sides | Both roar, both coil, and both leap at the same predicted point. 40 / 18 / 50 each. 180 each. **If they meet in the middle, both are dazed for 90 frames** | 40 — the only move both throw at once, and the loudest thing in the fight | **Dodge late** — after they have left the ground. Before that, their windups follow you and they meet wherever you went. After it, they cannot turn, and they land on each other |
| **Interpose** | its mate, below 30% | The healthy cat runs to the line between you and its wounded mate and turns to face you, shouldering whatever is in the way. 15 / the run / 30. 60 and a shove | 15 — ears flat, a snarl, head low toward the line | **Hit it first.** It runs a straight line to a place you can predict. Anything already swinging on that line catches it, and the run is a commitment: a hit on it during the run interrupts it whatever its strain |

Nine answers, no two the same: dodge toward, wait, stand elsewhere, stand
still, step sideways, jump, go close, dodge late, hit first.

**The feint and the ambush are one move made by two animals.** The Holder's
feint is only ever thrown while the Striker is behind you and free, so the feint
is also a message: *the other one is where you are not looking, and it is
watching you.* The tail flick at frame 10 is the only difference between a
pounce and a feint for the first fourteen frames, and it is drawn big — the
whole tail snaps up and over, not a twitch at the tip — because it is the one
thing a new player has to learn to see.

**The rake's hold is drawn from the snapshot's seed at the moment the second
hit is committed**, not at the first hit, so it cannot be learned from the
first. Its range, 6 to 30 frames, is wide enough that timing it from memory
fails about half the time and narrow enough that the paw is never up so long
that walking away is the answer.

**Why the pounce is dodged toward.** Away is what everybody does first, and the
skid exists to punish it: the landing is a 1.6 m circle, and the three metres
after it are live. Toward takes you under a body whose belly is two metres up at
the top of the arc — a dodge's invulnerable frames are not even needed if the
timing is right, which is the point. The Ridgeback's charge taught that a move
too fast to outrun and too wide to walk out of wants a dodge *through* it; the
pounce is the version with height.

**The twin pounce is the only time two hits land on the same frame.** Every
other pair of moves is kept at least `stagger_gap` frames apart (§5), because
two separately telegraphed hits landing together can each be answerable and
still be unanswerable as a pair. The twin pounce is exempt because it is
telegraphed *as one move*: one sound, one silhouette (two cats low and still on
opposite sides of you), one answer.

## 3 · A threat at every range

**Close (0–3 m).** Rake and swat in front, tail trip at the flanks and behind.
The Holder's face is the most dangerous metre in the fight, and its flank is
the safest place to stand *if you know where the Striker is*.

**Mid (3–9 m).** Pounce, feint, ambush. This is where the Holder keeps you, and
the range the fight is mostly fought at: close enough that both cats can reach
you in one move, far enough that you can see one of them coming.

**Long (9 m and beyond).** Nobody walks away from a 12 m/s cat, and backing off
is walking into the Holder's lead. Past nine metres the Striker perches if
anything is in reach and dives from 6–14 m; if nothing is, both close at a
run, and the Holder arrives first because it was aiming where you were going.

**Aboard.** None. The cats are not climbable, and a fighter who lands on a back
is **shed**: the top faces resolve as a slope, not a floor (§10). No riding was
a decision in the seed, and it is kept — a 1.8 m back is a hop, and a ride
reachable by a hop is the Ridgeback's old tail, which was a free route for most
of the roster.

**The tempting safe spot is a corner**, back to the arena wall, where nothing
can get behind you. What covers it is the **Perch**: the wall is 1.5 m tall and
a metre thick, standable, and a cat walks along the top of it and dives on you
from behind — out of the corner is the only direction that is not the wall. The
second temptation is **the top of a platform**, which the cats reach in one
leap. And the Bulwark's planted wall, which is too narrow to perch on (1.1 m at
most), covers one side of him and invites the twin pounce on the other two.

## 4 · The approach and the window

**The approach is awareness, spent as position.** Everything you earn in this
fight you earn by knowing where the second cat is:

- **A split.** A cat that cannot see you (§5) cannot strike, the partner
  without a Striker does not feint, and what you are fighting is one fair cat.
  A split lasts until the blind one walks round whatever is in the way — a few
  seconds, if you made it with a stone or a pillar, and for as long as it holds
  if it is a second player (§8).
- **A scarred eye — the thing that changes it for the rest of the fight.** The
  head is 1.3× damage and sits in the Holder's face. **350 damage into one
  cat's head scars the eye on the side the scarring hit came from**, for good.
  That cat no longer samples you anywhere in a 140° arc on its blind side
  (`blind_arc`), and keeps acting on the last thing it saw. Its lead runs on,
  so a player who was moving on its blind side and changes direction there has
  done something real — the Ridgeback's "change direction between its glances",
  made spatial and permanent. The scarred cat learns a habit to go with it: it
  turns toward its blind side to bring you into the good eye, which is a thing
  a player can see and use. One eye per cat; a blind animal is a fight you win
  by standing still.
- **The bond (below 30%).** The healthy cat interposes (§2). That sounds like
  protection, and it is the opposite: hurting the wounded one *pulls the
  healthy one into your cone along a line you chose*. Hitting a cat that is
  guarding, the one in front, is the safest hit in the fight — its partner is
  behind it, where you can see it.

**The big window is the twin pounce's crash.** Two cats on their sides for 90
frames, touching, a metre from where you were standing — the longest opening in
the fight, and the only one where a sweep, a whirl or a lit stone hits both.
It is thrown only when you are between them (the pincer, §5) and both are free,
so you earn it by doing the thing the fight has spent three minutes teaching you
not to do, and then being calm enough to dodge after they jump rather than when
they roar. The hardest-looking move in the set is the big opening, which is the
Ridgeback's slam lesson — the answer to the scariest move is also the
invitation.

**The arc, fresh to desperate:**

| Stage | What changes | Why |
| --- | --- | --- |
| Fresh | Clean roles, a swap every 7–10 s, one feint in four coils | The pattern has to be learnable before it is pressed |
| Strained (either below 60%) | Swaps every 5–7 s; feints one in three; `strain_desperation` lowers both cats' thresholds as usual | The same machinery as the Ridgeback: ordinary combos start to interrupt |
| Bonded (one below 30%) | The wounded one hangs back and throws only pounces and dives; the healthy one interposes | A cat protects its mate, and the protection is the exploit |
| First death | The survivor **howls over the body for 75 frames**, threatening nothing | The pause between the two fights. It is an opening, and it is also time to find it on the screen |
| Enraged | 15 m/s; recoveries ×0.7; the beat between moves halved; no roles, no feint, no ambush, no twin, no interpose; a missed pounce can chain straight into a second from the skid (20-frame tell) | All in, all in front. **Tells do not shorten** — no enraged tell goes below 15 frames, and the swat stays positional |

**Who to kill first is a decision the fight asks and does not answer.** The
survivor enrages at whatever health it has, so killing the scarred one first
leaves a sighted enraged cat, and bringing both low together and killing them
close together makes the enrage short. The fight report records the gap
between the deaths (§9) so it can be seen whether players find this.

**Strain.** Each cat has its own `strain`, and both sit lower on the scale than
the Ridgeback's (bestiary §1.8: small creatures sit lower): a knock-up trips a
cat after about a third of the recent damage it takes to trip the Ridgeback, and
a grab roots it. A tripped Striker is a split you bought with crowd control.

## 5 · The brain

Each cat is a full `Monster` with its own `Brain`: its own glance, its own
target, its own cooldowns, its own scoring. What is new sits beside them in a
small **pair brain** that owns the roles, the swap clock, the pincer and the
bond. Nothing in the pair brain sees anything; it reads the two cats' own
samples.

### Each cat glances for itself, and can be denied the glance

The Ridgeback's glance (every `glance_frames`, position and velocity, led by
`lead`) is unchanged. **P5's perception filter** gives it two conditions:

1. **Sight.** The sample is taken only if `aim::clear_between(head, target,
   scene)` is true — the existing question "is there any straight line from
   this body to that one", which the Reaver's dodge already asks. No new ray
   code, and nothing in `monster.rs` that reaches for a ray primitive:
   perception is a line of effect for the brain, and it lives with the others.
2. **The scar.** A target inside the scarred cat's `blind_arc` is not sampled.

A glance that fails keeps the old sample, and the old sample keeps being led.
**The two cats do not share what they see.** A Striker that cannot see you
cannot ambush you, even if its partner is looking straight at you — which is
what makes splitting them a real thing to do rather than a flavour line.

### It does not read your buttons, your look, or your dodge

The seed's hardest rule, and it is made airtight structurally rather than by
promise:

- **The brain's only input is `Quarry`** — position, velocity, alive, aboard,
  stunned — taken at a glance. `Monster::think` never receives an `Input`, a
  `Player`, or a facing. That is a function signature, and
  `the_brain_decides_the_same_whatever_buttons_were_pressed` pins it: two
  worlds identical in every `Quarry` sample and different in every input make
  the same choices.
- **"It saw you dodge" is only a velocity.** Nothing marks a dodge. A sample
  whose flat speed is above `seen_fast_speed` (10 m/s — above any walk, below a
  dodge) raises the Ambush's appetite (`fast_appetite`), exactly as
  `closing_appetite` raises the Ridgeback's forward moves. The Champion's Rush
  and the Reaver's dash read the same way, because they look the same.
- **Where it aims is the ordinary lead.** `seen_pos + seen_vel · lead`, with
  the horizon the Ambush's own startup. A fast sample leads far; that is the
  whole of "timing the pounce to the end of your dodge". It is the Ridgeback's
  aiming model applied to a fast target, not a new one.
- **The hit is never sooner than 15 frames after the sample it rests on.** The
  Ambush's startup is 20; the sample is at least a frame old when the decision
  is taken. `every_hit_lands_at_least_fifteen_frames_after_the_glance_that_chose_it`
  holds this for every move of every species, which makes it a contract test
  rather than a Pair test.
- **A dodge it did not see did not happen.** A glance that falls outside a
  22-frame dodge misses it — at the default `glance_frames` of 7 that is
  rare, and it is the reason the blinks (§7) beat the ambush: a blink has no
  frames of motion to see.

### Movement

- **Holder:** goes to `seen + seen_vel · prowl_lead`, held at `hold_distance`
  (5.5 m) — where you are going, not where you are.
- **Striker:** goes to `target + unit(target − holder) · strike_distance`
  (6 m), the far side from its partner — "behind you" by position, never by
  your camera (§1). It orbits the long way round if the short way crosses in
  front of you.
- **The swap:** every `swap_frames` (420–600, drawn), the cat nearer the line
  of your travel becomes the Holder. Both move for `swap_frames_crossing` (40)
  without attacking.
- **Regroup:** further apart than `bond_leash` (14 m), or without sight of each
  other, both drop their roles, the nearer one fights you alone as a solo cat,
  and the far one runs to rejoin.

### Scoring

```text
U(m) =  range(m) + arc(m) + variety(m) + hurt(m)      as the Ridgeback
      + role(m)     the role's moves only: Holder — rake, swat, feint, pounce;
                    Striker — ambush, tail trip, pounce, perch
      + fast(m)     `fast_appetite`, the Ambush only, on a sample above
                    `seen_fast_speed`
      + pincer(m)   the twin pounce, when both are free and the angle between
                    them seen from the target is above `pincer_angle` (140°)
      + perch(m)    a perchable top within `perch_reach` and the target beyond 8 m
      + bond(m)     interpose, when the mate is below `bond_health` (30%) and
                    you are within 10 m of it
      ⟂ cooldown(m)
      ⟂ partner(m)  nothing whose hit would land within `stagger_gap` (20)
                    frames of the partner's live hit, twin pounce excepted
```

`feint_share` sets how often the Holder's coil is a feint (one in four, rising
to one in three when strained), and it is only offered while the Striker is
behind the target and free. `decisiveness` is shared with the Ridgeback's
meaning; it rises when enraged, because an animal with nothing left to protect
throws what it wants.

**It presses a set-up across the pair.** The Ridgeback raises its appetite for
forward moves on a stunned target (`combo_appetite`). Here the *partner's* set-up
counts: a trip from the Striker raises the Holder's rake. The arithmetic test is
the Ridgeback's: `a_set_up_lasts_long_enough_for_what_follows_it` gets the cross
pair (a 70-frame trip outlasts the rest of the trip, the beat, and a 16-frame
rake arriving from 3 m).

## 6 · Reading it

**Every windup marks the floor it will hit, under the target** —
`Monster::telegraph`, unchanged. That rule turns out to be what makes this
fight fair: **the marker is drawn where the hit lands, and the hit lands on
you, so the marker is on your screen even when the cat is not.** The Ambush's
lane runs from behind the camera into the frame; the pounce's circle and its
skid lie under your feet; the dive's circle is where you stand.

**The silhouettes.** The Holder is square to you and low, tail high. The
Striker lopes, head low, tail straight out. The pounce and the feint share a
coil and differ at frame 10 by the tail. The rake's cocked paw is held at the
height of the cat's head, so it reads over the body. The twin pounce is two
cats flat and still on opposite sides of you, which nothing else looks like.

**Sound.** The design wants a snarl on every Striker windup and a roar on the
twin pounce. **The game has no audio yet** — there is no audio crate in the
workspace — so until there is, those are specified and not relied on.

### The camera: what is drawn to help, and what is not

**Decision: no persistent off-screen marker.** An arrow at the screen's edge
that always points at the cat you cannot see answers the fight's question for
you, and the question is the fight. "One is always behind you" becomes "one is
always on the radar", the camera stops mattering, and the lesson — turn, strafe
so they end up on one side, know before you commit — is replaced by glancing at
a HUD. The contract does not ask that you can see the animal; it asks that you
can answer the *hit* (§1.4), and the floor marker already answers that.

**What is drawn: a windup glint.** During the startup of any move committed by
a cat that is outside the target's view, a short arc glows at the screen edge
nearest it, and goes the frame the hit is out. It says *something is coming
from there*, never *something is there*: a cat circling behind you shows
nothing, and the glint stays off until it commits. It is the snarl, drawn,
standing in for audio that does not exist yet; it carries no information a
person with sound would not have; and when audio arrives it becomes an
accessibility option rather than the default. The glint is read from
`Monster::telegraph` like the floor marker is, so it cannot drift from the
move.

**What "unanswerable" means here.** A hit is unanswerable if its tell was under
15 frames and nothing positional warned you — the Ridgeback's definition —
**or if its floor marker was on the victim's screen for fewer than 15 frames
before the hit**. The second clause is new and is the one that matters: a
marker under your feet while you are looking at the sky is not a warning. The
report counts both (§9), and both must be zero.

## 7 · The classes

**Shadow Reaver.** Two bodies against two bodies. Her shadow out on the field is
a second presence the Striker has to walk past, its copied swing turns to the
nearest body in reach (`aim::shadow_faces`), and it marks from there — so a
shadow left on the Striker's circling route harasses it while she works the
Holder, and the dash to the shadow is her way of changing which cat is in front.
The cost is that the dash is fast motion, so the Striker reads it as a dodge,
and at 750 health an ambush is nearly a quarter of her. **Identity**: the pair
is the fight her mechanic was designed for, and it is hard in the way she is
meant to be hard.

**Elementalist.** The lesson's own tool. Three stones make a wall the Striker
cannot see through or walk through, and a stone raised in its circling path is
a split she built. But the stones are standable and 1.5 m across, so a cat
perches on them and dives from them, and a stone behind her back is a corner
with a lip on it. Lit stones burn a percher. Her long startups against a 12 m/s
target miss the cats themselves; they do not miss the interpose run, which is a
straight line she can see coming. After the Ridgeback, where she cannot win,
this is the creature she should have **easiest** of the six if the stones work
as §5 says — and whether it is too easy is a question for §12.

**Blood mage.** Her blink to a pool is instant — no frames of motion — so the
Striker **never sees her dodge** and the feint-and-ambush trap does not close
on her. That is a real hole in the Pair's premise for one class, and it is
kept as **identity**: it costs a pool, which means she has cut somebody there,
which means she was in close. Grasp pulls: pulling the wounded cat away from
its guard at the bond, or pulling the Striker into her cone, is the class's
answer to "split them". Her sweep at full grey is wide enough to take both cats
after a twin-pounce crash, which is her big payday. Hard for her: grey health
wants constant aggression, and two cats punish tunnel vision.

**Dual mage.** Her float is the overview. At 6.0 m apex she sees both cats from
above on one screen, which no other class can, and nothing the cats throw
reaches above about 3.5 m except a pounce off a perch. The cats wait under
her, and she comes down. Her half-tier blink is invisible to the ambush the way
the Blood mage's is. **Identity, bounded by the perch**: the air is her safe
place in every fight, and here it is the high ground the cats also want.

**Champion.** The lesson written against him. A three-hit string is sixty-odd
frames of looking at one cat, which is exactly "committing to one while you
can't see the other". His Rush cancels a recovery, which is the escape when the
lane appears — and is fast motion, which the Striker reads. The spear's reach
lets him poke from where both cats are on screen; the whirl hits all round, and
takes both cats at a crash. **Hard**, and the good kind: he wins by cutting
strings short, which is a skill the versus game wants too.

**Bulwark.** The planted wall is the lesson made literal — plant it behind you
and the Striker must come round, where you can see it. The held shield covers
one arc and the pair is built to be on two sides, so turtling invites the twin
pounce, which he can parry on one side only. The wall is too narrow to perch on
(1.1 m against the perch's 1.5 m), which is deliberate: the one class with a
portable wall should be able to rely on it. A parried pounce staggers the cat
like a parried fighter. His 2.7 m hop reaches the platforms. **Easy to survive,
slow to win** — identity.

## 8 · Coop

**Each takes one** is the obvious plan, and the pair is built to resist it
without making it wrong. A cat's attention now moves to whoever hurt it most in
the last two seconds (`hit_draws`), which the Ridgeback does not have, so two
hunters each working their own cat keep them split. The pair brain pulls the
other way: it assigns both cats to **the hunter standing furthest from their
partner** (the Holder on them, the Striker on their back). Stop hitting your
cat and it goes to your partner.

**The coop lesson is back to back.** Two hunters standing together cover each
other's backs, and there is no "behind" for the Striker to reach. The pair's
answer is the twin pounce aimed at the midpoint between two hunters close
together — dodged late by both, it crashes, and the crash is two people's
window.

**Numbers for two:** 3300 health each; `glance_frames` from 7 to 6; the swap
clock shortened by a fifth; `stagger_gap` stays at 20 per *hunter* (a cat may
hit one hunter while its partner hits the other). The bond is unchanged: it
already reads well with two, because one hunter can hurt the wounded cat while
the other stands where the guardian will arrive.

## 9 · Measuring it

**The scripted hunter must see what a person sees.** Today it knows where the
creature is at all times. Against the Pair that is a radar, and its win rate
would say nothing. P8 for this species adds a **view**: the bot has a camera
yaw it turns at a person's rate, and it perceives a cat's windup only if the cat
is inside its 90° view or the move's floor marker is. Without this the Pair is
unmeasured.

**The plan — what a person learns in the first ten minutes:**

1. Strafe until both cats are in view, and keep them there; turn toward the
   last place the missing one was seen.
2. Never dodge a coil without a tail flick; after a feint, look behind.
3. Poke the Holder only in a recovery, and only while the Striker is in view.
4. On an ambush lane, step sideways; on a pounce, dodge toward.
5. On the twin pounce, stand still through the roar and dodge after the jump;
   unload into the crash.
6. At the bond, hit the wounded one and swing on the interpose line.
7. After the first kill, back off through the howl and fight the survivor face
   on.

**Lines it adds to the fight report:**

| Line | Why |
| --- | --- |
| **Both in view** — share of frames both cats were in the hunter's view | Whether the fight is being played as the lesson says |
| **Hits from off-screen** and **markers seen ≥ 15f** | Must be 100%: the second clause of "unanswerable" |
| **Feints bitten** — dodges during a feint's coil | Whether the tail is readable |
| **Ambush after a feint**, landed / thrown | Whether the trap works, and whether it is escapable |
| **Twin pounces** thrown / crashed / landed | The window's frequency and whether late is learnable |
| **Split time** — frames either cat lacked sight of the target | Whether splitting is happening at all |
| **Blind-side hits** and **scars** | Whether the scar changes anything |
| **Interposes punished** | Whether the bond's exploit is being found |
| **Death gap** and **enraged time** | Who is killed first and whether the order mattered |

**Target numbers.** Tier 3: the scripted hunter wins **about one in three**,
won fights **3–6 minutes**. The four windows, measured with `frames_until_free`
taken as the **smaller of the two cats'** (the pair is free when either is):
threatening ~45%, poke ~20%, way in ~20%, walk up ~15%. *(Changed 2026-10-01,
§13: each cat's own window, with the min-of-two kept as its own line,
"threatening, together".)* Higher threatening and
lower walk-up than the Ridgeback's 40/20 because there are two animals; the
crash is most of the walk-up share. Unanswerable hits: zero. Every move used.
No move landing more than two times in three or fewer than one in ten.

## 10 · What it needs built

**Depends on:** P1 (species), P5 (perception filter), P8 (harness per species,
with a view). P2 (arenas as data) for the Den (§11), but not to start: the
current arena's walls and two 1.5 m platforms are enough to build and measure
the whole fight. Not P3 — the cats are full monsters, not critters — and not
P4, P6 or P7.

**What P1's "the rig is data" has to mean for this to work.** The cat is the
Ridgeback's eighteen bones with the same `PARENTS` and `SIDE`, re-proportioned:
new `REST` offsets, new `SHAPES`, new `LEGS`, its own clips. If any code outside
the species table has to change to make a smaller animal stand, walk, collide
and be hit correctly, that is a place the rig was not data. The known suspects:

- **Collision constants sized for a 13 m animal** — skins, the staircase
  overlap, `surface_within`'s tolerances. Anything in metres that is not a
  species number is a bug this creature finds.
- **Mountability as a species flag, and a new part flag `sheds`**, so a top
  face resolves as a slope and nobody stands on a cat. The tread-becomes-floor
  rule in `Rig::resolve` has to read the flag rather than assume.
- **Clips per species.** *(Built with P1, 2026-10-01: one table per species,
  `species/<species>/baked.rs`.)* `beast_baked.rs` is one table; it becomes one per
  species (or a species offset into one), baked by `bake_beast` for both. The
  32-sample phases stay: a cat's shortest phase is 3 frames and its longest
  under 60.
- **Gait from leg length.** The stride accumulator's cadence reads the species'
  leg length, or the cat skates.
- **Special cases by name** (`SHAKE`, `SWEEP`, `RIDGE || NAPE`) become move and
  part flags — P1 already lists them. The cats add `mirrors_to_target_side` to
  the tail trip and `weak_point` to nothing.
- **`beastcheck` per species**, printing the cat's heights against the hop
  spread the way it does the Ridgeback's.

**Two monsters in `World` — the first time.** `World` holds one
`monster: Option<Monster>` today, with a comment that says an array of one is a
promise the code has not earned. This creature earns it. Proposed:

```rust
pub monsters: [Option<Monster>; MAX_MONSTERS],  // MAX_MONSTERS = 2
pub pair: PairBrain,                            // roles, swap clock, pincer, bond; zeroed outside a pair fight
```

- Every one of the ~140 `.monster` sites becomes a loop, or goes through a
  `World::monster()` accessor returning slot 0 for code that is genuinely
  single-creature (the ride). The Ridgeback must come out bit-identical.
- A fighter's `mount` is a part index today. It packs the slot in the top three
  bits (`slot << 5 | part`, `PARTS` is 18), so the ride is not tied to slot 0
  forever even though the cats cannot be ridden.
- **Monster against monster.** Two flat circles, resolved like two fighters, and
  no friendly fire — except the twin pounce, whose two landing volumes are
  tested against each other and put both cats in `Doing::Toppled { left: 90 }`,
  reusing the topple rather than adding a state.
- The fixed array costs its second slot in every fight, Ridgeback included:
  about 200 bytes that are always there. The alternative, an enum of fight
  shapes, is smaller and is the right thing once P3's critters want the same
  room; for two, an array is honest and cheap.

**Snapshot.** Two `Monster`s at ~200 B each; per cat a role, a scar side, the
rake's hold and an enraged flag (~4 B); a `PairBrain` of ~12 B (swap clock,
twin cooldown, bond flags, the first death's frame). **About 430 B**, against
the bestiary's estimate of 400, taking the snapshot from ~1.8 to ~2.2 KiB.
`size_of` replaces this when it is built.

**Per-frame cost** is dominated by **posing two rigs** (two 18-bone `Mat3`
chains), and during windups posing each again for `Monster::telegraph` — up to
four poses a frame — then the fighters against 36 part boxes. Perception adds
at most two `clear_between` calls per glance, one every seven frames. About
twice the Ridgeback's monster cost; `budget.rs`'s one-advance and rollback
budgets are the check, and at the Ridgeback's measured margin it should pass
without work.

**Aiming.** No change to `aim.rs` is needed, and it is worth saying why. The
cats are fighter-sized: a ground-aimed skillshot goes to the middle of a
fighter standing there, 0.9 m up, which is the cat's flank; a level swing from
the chest meets its body; a leaping cat is behind the crosshair's ray, the ray
goes through it to the sky or the wall, and the straight line from the caster
meets the cat on the way (`aim::first_along`). The Ridgeback's "a level shot
goes under the belly" cannot happen here. The one new use is perception, which
calls the existing `aim::clear_between`.

**Tests that pin its rules:**

- `the_brain_decides_the_same_whatever_buttons_were_pressed`
- `every_hit_lands_at_least_fifteen_frames_after_the_glance_that_chose_it`
- `a_cat_that_cannot_see_you_does_not_sample_you`
- `a_scarred_cat_does_not_see_you_on_its_blind_side`
- `the_feint_and_the_pounce_differ_by_frame_ten`
- `the_pounce_is_dodged_toward_not_away`
- `the_ambush_is_stepped_out_of_sideways`
- `the_rake_hold_is_drawn_when_the_second_hit_commits`
- `the_two_never_land_within_the_gap_except_the_twin_pounce`
- `a_late_dodge_crashes_the_twin_pounce_and_an_early_one_does_not`
- `hurting_the_wounded_cat_brings_the_other_between`
- `the_survivor_enrages_and_no_tell_goes_below_fifteen_frames`
- `nobody_stands_on_a_cat`
- `every_windup_marker_is_on_the_targets_screen_for_fifteen_frames` (in `view`)
- `the_ridgeback_is_bit_identical_with_two_slots`

**Milestones.** Each ends in something checkable.

1. **M1 · Two slots.** *(The two slots and the mount packing were built with
   P1, 2026-10-01 — see [species.md](../species.md) §3; `PairBrain` is still
   this creature's.)* `monsters: [Option<Monster>; 2]`, `PairBrain` zeroed,
   the mount packing. Check: `cargo test --workspace`, and the fight report's
   twelve Ridgeback seeds print the same numbers as before.
2. **M2 · The cat as data.** A species entry on the Ridgeback's topology with
   new `REST`, `SHAPES`, `LEGS`, the `sheds` flag and idle, walk and bound
   clips. Check: `beastcheck` prints the cat's heights; a headless screenshot
   of it standing and walking; `nobody_stands_on_a_cat`.
3. **M3 · One cat.** Pounce, rake, swat, tail trip, perch and dive, with
   telegraphs; the solo brain. Check: the per-move answer tests, and the fight
   report against one cat (it should be tier 1–2 alone — "either one alone is
   fair").
4. **M4 · The pair.** Roles, the swap, the feint, the ambush, `stagger_gap`,
   the twin pounce and its crash. Check: the contract tests above, and a
   `SHOT_MOVE=twin` screenshot of both markers.
5. **M5 · What changes it.** Perception through P5, the scar, the bond and
   interpose, the death and the enrage. Check: their tests; a scripted scar in
   the report.
6. **M6 · Measured.** The hunter's view, the plan in §9, the new report lines,
   and a first tuning pass toward tier 3. Check: the twelve-seed report, with
   unanswerable at zero.

## 11 · In the world

**Region: the scrub highlands** — dry grass, rock outcrops, the country a pair
of big cats hunts over. (Proposed for [world.md](../world.md), which is being
written alongside this.)

**The arena: the Den.** 30 × 30 m, 1.5 m walls, the two 1.5 m platforms of the
current arena, and **three standing stones**, 3.5 m tall and 1.2 m across — too
narrow to perch on, tall enough to break sight for anything behind them. They
are what splitting the pair uses when a class brings no walls of its own, and
they are placed so that no corner is more than 8 m from one. Needs P2; until
then the fight runs in the current arena, which has the walls and platforms
and lacks only the stones.

**The trophy:** the scarred eye's side is remembered. A pair beaten after
scarring an eye leaves a **torn ear** on the hunter's banner on that side —
cosmetic, and a record of how the fight was won.

**The sidegrade: Cat's step** — a common-mechanic modifier, in the spirit of
[parked.md](../parked.md). **Your dodge may be turned once, at the end of its
invulnerable frames, to any direction, at the cost of a fifth of its distance
and four frames more recovery.** It is what this fight teaches — change
direction after the dodge, where the thing that saw you start it expects you to
end — made into a playstyle. It is an enabling stat (it changes what you can
dodge through and where you end up), and it is paid for in exactly the
currencies the dodge is measured in, so it is never simply better.

## 12 · Open questions

1. **Is the windup glint a crutch after all?** It appears only during a tell,
   but a player who learns that "no glint means nothing is coming from behind"
   has been handed some of the answer. Play it with and without; if the fight
   is fair without it, it should be an option from the start.
2. **Is the tail flick readable at fight speed?** Frame 10 of 24 is 14 frames
   before the leap — barely a reaction. If feints are bitten more than half the
   time after ten minutes, the flick moves to frame 6.
3. **Does the Blood mage's blink break the fight?** It makes the ambush trap
   unable to close on her. Kept as identity; a person should say whether she
   plays the Pair or plays around it.
4. **Is the Elementalist too comfortable?** Three stones as a wall may be a
   split she holds forever. The perch is meant to answer it; whether it does is
   a play question.
5. **Enrage by kill order.** Should the survivor's enrage scale with how long
   the pair fought together, or with the survivor's health? As written it is
   flat, and "kill both close together" is the strategy. Is that the strategy
   you want to exist?
6. **Coop's `hit_draws`.** It is new, it is a threat model, and the Ridgeback
   does not have it. It makes "each takes one" possible. Should the Ridgeback
   get it too, or is it a Pair thing?
7. **Should the two cats be told apart by more than colour?** Dun and dark is
   enough for a call-out. If they should *play* differently too — one heavier,
   one faster — the roles get muddier and the fight gets another layer; the
   seed says bonded equals, and this keeps it.

## 13 · Where it landed

Built 2026-10-01: `--hunt pair`, the Den. The species is
`sim/src/species/pair/` (the table; `fight.rs` for the pair brain -- roles,
the swap, the feint's licence, the twin pounce and its crash, the stagger,
the bond, the howl and the enrage, sight and the scar, the leaps -- run in the
species' `frame` hook and kept in the hunt's lore; `mind.rs` for each cat's
own scoring and where it walks), the arena `sim/src/arena/pair.rs`, the clips
`anim/src/beast/pair/`, the look, the Den's dressing and the windup glint in
`game` (`species/pair.rs`, `arenas/pair.rs`, `glint.rs`), the plan, the view
and the report lines `hunt/src/plans/pair.rs`, and the rules pinned as
sentences in `sim/tests/pair.rs` -- all fifteen of §10's, the view's one as
the report's marker clause (below) rather than a test in `view`. The plan is
[plans/the-pair.md](../plans/the-pair.md); the passes are in
[feel-log.md](../feel-log.md) of 2026-10-01.

Numbers from `cargo run -p hunt --bin fight -- --species pair --class <c>`
over 24 seeds; the scripted hunter plays §9 with a fifteen-frame reaction and
**a view**: a camera yaw it turns at three quarters of a turn a second, and it
knows a cat only while the cat or its marker is on that screen.

```text
                won    mean win   threat / poke / way in / walk up   both in view   unanswerable
  Champion     16/24    159 s         78 /  7 /  7 /  8 %                75 %             0
  Bulwark      12/24    186 s         81 /  6 /  6 /  7 %                74 %             0
  Reaver        4/24    218 s         80 /  6 /  7 /  7 %                74 %             0
  Elementalist  4/24    259 s         81 /  6 /  7 /  6 %                72 %             0
  Blood mage    0/24      --          80 /  6 /  7 /  7 %                75 %             0
  Dual mage     0/24      --          80 /  7 /  7 /  6 %                77 %             0

  coop, two Champions 11/12 in 149 s;  temper 3, Champion 9/12 in 159 s

  landed / thrown, 24 Champion hunts
    Pounce 91/723   Rake 29/387   Rake, again 18/345   Swat 5/25
    Ambush 42/374   Tail trip 19/345   Dive 14/47   Twin pounce 2/126
    Interpose 1/39  Feint 86, Perch 101, Drop 55, Cocked paw 11 (no damage)
```

**Its own lines**, over the Champion's hunts: markers on screen at least
fifteen frames before every hit that has a marker to see, in every hunt of
every class (the second clause of unanswerable, §6); one hit in ten from a cat
that was off screen as it began -- answered by its marker; both cats in view
three quarters of the fight; a cat without sight of its target a third of it;
two or three twin pounces a hunt, nearly all crashed; one or two scars a hunt,
and a scarred cat struck from its blind side six to eighteen times; the death
gap six to nine seconds when the hunter wins.

**Against the targets.** **Zero unanswerable hits**, by both clauses, in all
144 hunts of the six classes, coop and temper 3. The four classes the harness
plays win 36 of 96, three in eight, against "about one in three", in two and
a half to four and a half minutes (three to six asked). Every move is used.
**What is off:**

- **Threatening was 78–81 %, against 45**, with the pair free when either
  cat is, as §9 said to measure it. **The measure was wrong, and changed
  (2026-10-01, [plans/polish-fights.md](../plans/polish-fights.md))**: the
  four windows are the rhythm of openings on *a body* -- how often the one in
  front of you can be punished -- and the harness won two hunts in three
  through openings the min-of-two said were not there. Each cat's own window
  now, each frame counted once for each cat alive: **threatening 55–58 %,
  poke 7–8, way in 11, walk up 23–26** across the six classes. The cover the
  second cat gives is the fight's lesson, and it is measured where it is
  taught: *both in view*, the off-screen hits, and a new report line,
  **threatening, together** (either cat able to answer), still about four
  fifths. The creature was left alone: the remaining ten points over 45 are a
  prowling cat close enough to pounce, which is a threat, and fewer of them
  would make a fight the harness already wins two in three easier still.
- **Several moves land under one in ten**: the rake (one in thirteen), the
  second rake, the tail trip, the twin pounce, the interpose. The harness
  answers every marker it sees at a fifteen-frame reaction, perfectly; the
  twin pounce is dodged late every time, which is the lesson, so it crashes
  rather than lands.
- **Feints are rare**: three or four a hunt, and almost never bitten. The
  harness reads the tail perfectly, so whether the flick is readable at
  speed (§12 question 2) is still a person's question.
- **The Blood mage and the Dual mage lose every hunt**, as against every
  creature: the scripted hunter does not play their pools or their bars, a
  limit of the harness, not a finding about the cats.
- **Temper 3 barely moves it**: the glance tightens (`temper::glance`
  reaches the cats through `FightDecl::glance`), but a cat already glancing
  every seven frames has little left to gain.

**The body, as `beastcheck --species pair` prints it**: nose to tail 4.18 m,
shoulders 1.78 m, back 1.53 m, head 1.62 m; every class hops over a standing
cat, and nobody stands on one (`sheds`, `nobody_stands_on_a_cat`).

**Changed from this document while building**, beyond the numbers:

- **Health 2 400 a cat** (3 600 with two hunters); at 2 200 the Champion and
  the Bulwark won eleven in twelve in two minutes, at 3 000 one in five.
- **The pounce is 30 / 14 / 50 for 110, landing at 1.4 m radius**, not 24 /
  20 for 150 at 1.6: at 24 frames a fifteen-frame reaction to the frame-ten
  flick left no dodge that came out under the arc. The flick is at frame 13.
  The mark follows you until startup frame 12 and then is fixed, so the body
  lands where the circle is.
- **The dive is 30 / 6 / 60 and leaves the lip at frame 22; the twin pounce
  is 46 / 6 / 50 for 140, leaving at 40, the mark following until 28.** Both
  at 1.4 m: a sixteen- or eighteen-frame live landing could not be cleared by
  a dodge with ten invulnerable frames.
- **The ambush's lane runs 2.5 m past where it saw you** and is laid at the
  commit, the cat flat along it from the first frame; a lane that ended at
  your feet, or swung on as the tell went, was a marker that arrived late.
  **120 damage**, not 170.
- **The interpose stops short of you** by its own reach: the run hits what
  walks into its line, not somebody standing still on it.
- **Sight is from the head or the crown** (`perception::in_sight_over_cover`):
  a 1.5 m platform between a cat and a fighter hid each from the other at the
  middle's height, and the cats lost the hunter behind every platform.
- **Nothing that hurts is thrown at a memory**: a cat that cannot see its
  target does not attack it, and a move's range must hold where it last saw
  you as well as where it leads you to. Without both, a stale sample put an
  ambush or a swat on a fighter standing inside its tell.
- **The swat is thrown at 1.0–2.6 m from planted feet**: a 12-frame tell
  under a reaction must only reach what was already in reach.
- **A feint needs its mate round the target by 0.17 turns and on its feet**
  (prowling or already coming), not *behind and free*: as written the feint
  happened twice a hunt.
- **A lobbed leap lands on the top under its aim** (`FightDecl::lob_height`)
  and a mark is pushed out of any solid taller than the target's footing: a
  circle drawn inside a platform was a marker nobody could stand in.
- **A leaping cat has a hurtbox and no body** (`Presence::passable`): a dodge
  under the arc has to go through it.
- **The Den's stones** stand at (9.5, 9.5), (−9.5, −9.5) and (0, 10): with
  three stones and four corners "no corner more than 8 m from one" cannot be
  had, and the open floor south of the platforms is kept for the fight.
- **The swap, the stagger and the bond are as written**; the twin pounce's
  crash is a topple of `CrashFrames` (90) on the last live frame if neither
  cat has hit and they are within `CrashReach`.
- **Not built**: the torn-ear trophy (a hunt won records its trophy as every
  creature's does; the banner is not drawn yet), the snarl and the roar (no
  audio), Cat's step.

