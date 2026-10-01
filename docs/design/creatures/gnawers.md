---
status: built
proposed: 2026-09-30
built: 2026-10-01
tier: 1
---

# Gnawers — a pack that takes turns

> **Built 2026-10-01.** `--hunt gnawers` (`?hunt=gnawers`), in the Commons.
> The species is `crates/sim/src/species/gnawers/` -- its table, its mind
> (`mind.rs`) and its rules over the world (`rules.rs`) -- on the critter
> machinery of [critters.md](../critters.md); its hunter is
> `crates/hunt/src/plans/gnawers.rs`; its rules are pinned in
> `crates/sim/tests/gnawers.rs`. What the harness says, what the build
> changed in this document, and what only a person can answer are §13. The
> numbers in the tables below are the ones built; where the build moved one,
> §13 says from what and why. The plan it was built by is
> [plans/gnawers.md](../plans/gnawers.md).

Six knee-high biters and the one they follow. The first fight that is not one
body, the first that needs the small-body form and the pack brain from
[the bestiary](../bestiary.md) §3 P3, and the first creature short enough to
walk under the aiming model. That last is the most important thing in this
document, and §1a is about it. The voice, the machinery and most of the
vocabulary are the Ridgeback's; read [monsters.md](../monsters.md) first.

---

## 1 · What the fight is

**Nothing in the pack can hurt you much; the pack can.**

A gnawer is a badger-shaped rat-dog **0.6 m at the crown and 1.2 m nose to
tail**, with 160 health (120 proposed; §13). The pack is **six of them and the
Big One**, the leader: 1.1 m at the shoulder, 2 m long, 500 health (360
proposed). (Five to seven is the
range a tempered hunt or a region varies; six is the number everything below
is tuned for.)

Sizes, against what a fighter is:

- A fighter is 1.8 m tall and casts from 1.25 m. A gnawer's crown is **below a
  fighter's hip**, and the Big One's is below the chest. Every class's lowest
  hop (2.7 m, the Bulwark) clears both, which matters for exactly one move.
- They **dart at 9 m/s against a walk of 7**. You cannot outrun them, and
  running is showing them your back, which is what one of their moves is for.
  You out-position them. A dodge at 17 m/s is faster than a gnawer for its
  length and no longer.
- One bite is **45**, under a twentieth of a fighter. The Ridgeback's is 190.
  The number the fight is about is how many land in the same second.

The loop:

1. **Find a back.** The pack circles at 5 m and only two may bite at once (the
   attack tokens); the other four go round to your back. A wall, an arena
   platform's side, a raised stone or a planted shield behind you makes the
   ring a half-ring, and a half-ring is in front of you.
2. **Read the crouch.** A gnawer holding a token crouches and wiggles before it
   lunges. Any hit during the crouch knocks it out of the lunge; they are light.
   That is the fight's rhythm: two crouches at a time, each one a small
   opening if you swing first.
3. **Don't get slowed.** A hamstring from behind slows you, and a slowed
   fighter is what the whole pack is waiting for: every gnawer in range leaps,
   tokens or not. That is where the damage comes from.
4. **Kill one; take the window.** A death makes the others flinch back eight
   metres for a second. The Big One flinches three. That second is the way to
   the leader.
5. **Break it.** Kill the Big One and the pack routs for good. Kill three and
   it routs to its den, and comes back in ten seconds if you let it.

### 1a · Small bodies break the aim

The Ridgeback found that **a level shot from a fighter on the floor goes under
the belly** (monsters.md §8). The gnawer is the mirror image: **a level shot
goes over its back**, and aiming at it does not fix that, because of how the
aim was built.

- **Skillshots.** Bodies are not on the crosshair's ray, so the crosshair on a
  gnawer is the crosshair on the floor behind it, and a ground hit is raised to
  `aim::standing_middle` — **0.9 m, the middle of a fighter**. The shot's line
  runs from the caster's chest at 1.25 m down to 0.9 m and is then over. A
  gnawer's crown is 0.6 m, so the shot touches it only if its radius is at
  least 0.3 m, and only at the far end of the line. First-order arithmetic from
  `tuned.rs`: the Elementalist's bolt (0.35) clears the crown by −5 cm, which
  is to say it grazes when perfectly centred; the fire bolt and air bolt (0.3)
  are exactly tangent and miss. **Her auto cannot hit a gnawer.** From above it
  is worse, not better: the line from a high caster ends 0.9 m above the floor
  it was aimed at, 0.3 m over the back.
- **Swings.** Standing, the first 45° below the horizon is a dead zone and the
  swing stays level. Pointing the crosshair at a gnawer a few metres off is
  twenty-odd degrees down — inside the dead zone — so the swing is level from
  the chest. Flat cuts are fine: they start at `sweep_height` (0.7 of chest,
  0.875 m) and dip, so the sword's tip is near 0.5 m at full reach. Thrusts are
  not: the spear's jab is a 0.28 m ball on a level line at 1.25 m, and its
  lowest point is **0.97 m, 37 cm over the gnawer**. To tilt a swing down onto
  one you have to look more than 45° down, which puts the crosshair on your own
  feet.

These numbers are prose, and geometry arguments conducted in prose go wrong.
They are here to show the size of the problem, and §10 M2 builds the instrument
that replaces them.

**The fix belongs in `aim.rs`, and it is one idea: "there" has a height, and
the height is whatever stands there.** Today "the middle of a fighter standing
on it" is a constant because the only things that stand are fighters.

```text
aim::stands_at(ray, world) -> Fx
    the crown height of the LAST body the crosshair's ray passed through
    on its way to the floor; t::body_height() if it passed through none

aim::standing_middle(ground, height)      ground.y + height / 2
aim::skillshot_path    ground hit: standing_middle(ground, stands_at(..))
aim::swing_path        standing, inside the dead zone: tilted down by the
                       angle that takes the hand's line from the middle of a
                       fighter at `reach` to the middle of what stands there;
                       zero when that is a fighter
```

Three properties, each load-bearing:

- **Bodies are still not on the ray.** The ray goes through the gnawer to the
  floor behind it, exactly as it goes through a fighter; nothing stops it
  short, and the creature-filling-the-screen bug CLAUDE.md describes cannot come
  back. The only thing a body now contributes is *how tall "there" is*. The
  line from the chest to a point 0.3 m up behind the gnawer passes through the
  gnawer on its way, and `aim::first_along` does the rest, as it does for a
  fighter.
- **Where only fighters stand, every answer is bit-identical.** `stands_at`
  returns `body_height`, the tilt is zero, and the Ridgeback, versus, the
  sparring bot and every existing test come out the same. It reads a
  fighter's *standing* height, not its crouched one, so crouch stays the flag it
  is today (`hits_crouching`) — see §12.
- **The swing reads the crosshair, not the pack.** A swing that looked for the
  nearest short body in reach and ducked for it would be aim assist. This one
  dips because the player pointed at something short. Point over the gnawer at
  the Big One's head and it swings level.

The alternative that was considered and is rejected: a `hits_low` flag per move,
beside `hits_crouching`. It is a workaround written as a property. The overlay
would draw the spear passing over the gnawer while the hit test hit it, which is
the one thing [CLAUDE.md](../../../CLAUDE.md) says an overlay may never do.

**Built 2026-10-01, ahead of this creature, with P3** ([critters.md](../critters.md),
[aiming.md](../aiming.md) §"Small bodies"). `stands_at`, the skillshot to the
body's middle, and the standing swing's dip are in `aim.rs`, bit-identical
where only fighters stand. Changed while building, by the instrument: the swing
meets a short body at **the share of its height a level swing meets a fighter
at** (`aim::stoop`), not its middle -- the middle tipped it steeper than it meets
a fighter for nothing -- and the Dual mage's wing, a flat disc and a thrust each
read the same drop. The instrument is `cargo run -p sim --bin critcheck`, generic
rather than `gnawcheck`, with a fighter standing at the same distance as the
control. On a 0.6 m body every move touches it wherever it touches a fighter,
except **lunges that carry the fighter through it** (the Skewer and the Dual
mage's dark auto at 1–2 m) and **the Guillotine**, whose blades are flat at 0.9 m
(critters.md §7 -- a person's call). The Elementalist's bolt and her air shots
touch it at every distance; her auto is no longer the sharpest form of the hole.

## 2 · The moves

Frame numbers and damage are **first guesses**, to be moved by the fight
report. Every tell is at or above twenty frames.

| Move | Who, range | What it is | Tell (f) | The answer |
| --- | --- | --- | --- | --- |
| **Dart-bite** | any gnawer with a token, 5 m out | The pack hands it a token: tail up, it comes in -- round into the front arc if it was at a flank -- and at 2.1 m it goes belly to the floor, rump up, a wiggle; then a lunge along a locked line. 28 / 12 / 30, **45** | 28 | **Hit it in the crouch.** Any hit knocks it out of the lunge |
| **Hamstring** | a gnawer in your rear third | Handed a token from behind: it comes in to 2.5 m behind your heels, then a low scuttle and a latch on the calf. 20 / 6 / 24, **25** and a **40% slow for 120 f**; latched, 5 every 10 f until shaken. Turn to face it before it begins and it gives up | 20, behind you | **Keep your back to something solid.** Caught, any dodge sheds the latch |
| **Pile-on** | every gnawer within 8 m of a slowed or staggered fighter | Tokens suspended. Called on the glance, every leaper locks to that one spot; the ring tightens to 2.5 m and they leap in sequence, one every 4 f. 30(+4 each) / 8 / 50, **40** each, and three landed is a knockdown. A death in the heap does not call it off | 30 | **Dodge out of the ring** when the crouches come. The slow does not slow a dodge |
| **Scatter** | every gnawer within 8 m of one that dies | Not an attack. A yelp, and they flinch 8 m away for **60 f**; the Big One steps back only 3 m over it | — | **Take the window.** Go for the leader while the ring is open |
| **Gnaw** | up to three gnawers, at a stone you stand on | They dig at its foot. 3 diggers bring it down in 120 f, 2 in 180. The stone falls, you land among them staggered for 40 f | 20, then 120+ | **Come down on the diggers.** They have their backs to you |
| **Scramble** | a gnawer with a token, beside a platform you stand on | 30 f with its forepaws on the edge, then up | 30 | **Hit it while it clings.** Any hit knocks it off |
| **Maul** | the Big One, 2.2 m, from in front | A heavier lunge at the legs; the volume stops at 1.0 m. 30 / 5 / 60, **110**. No token: it is the Big One's answer to somebody who has come to it | 30 | **Jump it.** Every class's hop clears a metre |
| **Howl** | the Big One, from behind the ring (5–13 m) | Rears to 1.8 m and howls. Rally: **three tokens for 300 f**, and the scatter ends. 40 / 1 / 20; lockout 900 f | 40 | **Hit the leader.** Anything that lands cancels it and flinches it 30 f |

Eight rows, eight answers: hit first, cover your back, dodge out, press,
punish the diggers, knock off the climber, jump, and target priority.

**Why the dart-bite's tell is 24 and not the seed's 18.** The answer is a hit
during the crouch, and the crouch has to be reacted to and then swung into:
fifteen frames of reaction plus a six-frame sword leaves a three-frame margin
at 24 and none at 18. At 18 the answer is pre-emption only, and at tier 1 the
answer should be available on sight. Classes whose fastest poke is slower than
nine frames answer it by pre-empting — swinging at a gnawer as it takes the
token and before it crouches — which is a thing a person learns. `frametable`
says which those are.

**The crouch tracks; the lunge does not.** The crouch keeps turning toward the
glance's lead at full rate until 8 f before the lunge (`dart_lock`), then locks.
So a sidestep in the last eight frames makes it miss, and a sidestep earlier
does not; but the table's answer is the hit, because a pack that is dodged is a
pack still standing.

**The hamstring's tell is behind you, and still on screen.** The camera sits
behind the shoulder, so "behind the fighter" is the floor between the fighter's
heels and the camera. The launch range is 2.5 m so the whole marker lies in the
floor the camera shows below the character (`hamstring_reach`, pinned against
the camera by a test in §10). And it is heard: the gnawer that takes a
hamstring chitters from its side of the stereo field on the frame it commits.

**The pile-on is sequenced so the dodge answers it.** Leaping together, six
bodies arrive on one frame and a dodge either clears all of them or none. Four
frames apart, the dodge's ten invulnerable frames take the first two and its
2.8 m of travel takes you out of the rest's lane; the heap they land in is the
best sweep target in the fight — fifty frames of recovery, all of them in one
place. It is the pack's biggest threat and its biggest opening at once.

**The gnaw is slow on purpose.** It exists to cover the one place in the arena
they cannot otherwise reach (§3), not to be a threat by itself. The stone
cracks visibly and its top darkens as it goes.

## 3 · A threat at every range

**Close (0–3 m).** Two dart-bites at a time, the Big One's maul if it has come
in, and a hamstring whenever your back is open. Nothing here is big; everything
here is frequent.

**Mid (3–8 m).** The ring. Standing at 5 m is where a gnawer waits for a token,
and a token taken at 5 m is a crouch followed by a 4 m lunge that closes the
rest. Mid is also where the Big One hangs back (`leader_hangback`, 8 m behind
the ring from the target) and howls.

**Long (8 m and out).** They close at 9 m/s, spread rather than in a line — a
skillshot lane at long range catches one, not the pack. And a fighter moving
away from them shows them a back: the ring re-forms behind a retreating fighter
first, so walking away is how you are hamstrung, and a hamstrung fighter is how
a pile-on starts. Retreating is the long-range threat.

**The tempting safe spots, and what covers each:**

| Spot | Why it tempts | What covers it |
| --- | --- | --- |
| An arena platform (1.5 m) | Out of reach of a 0.6 m animal | They **scramble** up it: 30 f with their forepaws on the edge. Any hit knocks one off, so it is a defensible position, not a safe one |
| A raised stone (1.8 m) | Too tall to scramble | **Gnaw.** Stand there and it falls |
| A corner | Two walls; no back to reach | It is the intended answer, and it does not win: the Big One stays 8 m out, the scatter sends the rest out of reach, and the howl gives three tokens against a fighter who has nowhere to dodge to |
| The air | The Dual mage can stay up | **Treed**: see §5. They spread under her so whatever she drops catches one |

## 4 · The approach and the window

**What you earn.** Each kill is a scatter, and a scatter is the window. Sixty
frames with the ring pushed 8 m out and the Big One only 3 m back — close, alone,
and a second from its pack returning. A dash, a Rush, a blink or a shadow
crossing reaches it in that time; walking does not. That is the route to the
leader, and the leader is the fight.

**The big window** is the Big One knocked down: a hit landed during its howl,
followed by a burst past its `interrupt_strain`, puts it in a 90 f stumble
where its pack is still scattered and nothing covers it. It is the gnawers'
topple.

**What changes for the rest of the fight.** The pack's size, and you can see it.
Six gnawers ring you with a body every sixty degrees and four of them can be
behind you. **At three left the ring has gaps a fighter can keep in front**:
three bodies spaced round 5 m are ten metres of arc apart, and the pack brain
stops trying to fill a circle and forms a line (§5). The fight changes shape,
visibly, because of kills you made.

**Morale.** The pack routs when the Big One dies or half the gnawers are dead
(`rout_share`, 1/2).

- **The Big One dead**: the rout is final. The survivors run for the den and
  the hunt is won when the last is in it or dead. This is the trophy.
- **Half dead, the Big One alive**: they run for the den, **regroup after 600 f**
  (`regroup_frames`) if no fighter is within 10 m of its mouth, and come back
  behind a howl, free. The rout count is counted again against who came back.
  Routing gnawers do not fight; they are 9 m/s and you are 7, so chasing them
  is a ranged or a dashing class's business, and standing at the den to stop
  the clock is anybody's.

**From fresh to desperate.** The pack does not have strain thresholds that fall
with health: a gnawer is interruptible by anything at any time, which is what
"they're light" means (bestiary §1.8: small creatures sit lower on the same
scale). The Big One does, at `interrupt_strain` 150, falling to 90 as it is hurt
(`strain_desperation`). Desperation is the pack's instead: **below half the Big
One's health it howls on cooldown**, and a howled pack of three is as dangerous
as a quiet pack of six. The fight goes from many-small to few-and-fast.

## 5 · The brain

What differs from the Ridgeback's control algorithm. The pack brain (P3) is one
object; the critters are its hands.

**One glance for the pack.** Every `pack_glance` frames (12) the pack samples
each target's position, velocity and **facing** — facing is visible, so it is
something it *saw* — and every critter acts on that one sample, projected by
`pack_lead`. The skill the Ridgeback's glance teaches carries over, and grows: a
direction change between glances fools the whole ring at once.

**Slots, not chasing.** At each glance the pack assigns every gnawer without a
token to a slot on a ring of `ring_radius` (5 m) round the lead point. Slots
are scored by angle from the target's seen facing, biased to the rear
(`rear_bias`); a slot inside a solid, or with a solid between it and the target,
scores nothing — which is what makes a wall at your back work. With three or
fewer gnawers the ring becomes an arc in front of the target
(`line_below`), because three cannot surround anybody and pretending otherwise
leaves them strung out to be picked off.

**Tokens.** `pack_tokens` (2) may be held at once. A token is handed at a
decision point to the best-scoring gnawer and returned when its move ends,
and a returned token rests `token_rest` frames (30) before it can be handed
again. The rest is where the walk-up window lives. A howl raises the count to
`howl_tokens` (3) for `howl_frames` (300). The pile-on suspends tokens entirely.

Each gnawer scores its own moves with the Ridgeback's terms and four more:

```text
U(m) =  range(m) + arc(m) + variety(m)            as the Ridgeback
      + rear(m)         hamstring only: the target's rear third, not ours
      + setup(m)        enormous for the pile-on when the target is slowed
                        or staggered, and nothing otherwise
      ⟂ token(m)        a bite or a hamstring without a token scores nothing
      ⟂ fear(m)         scattered or routed: attacks score nothing at all
```

**Individuals are noisy; the pack is not.** A gnawer's `decisiveness` is low
(0.5), so which one bites is a draw. The ring and the tokens are rules, so how
many bite is not. The distribution a player learns is the pack's.

**The Big One** runs the same scoring with its own moves and one standing
order: keep `leader_hangback` behind the ring. It howls when the pack has lost
a gnawer in the last 300 f, or its target is set up, or it is below half.

**Treed.** A target whose feet are above `leap_reach` (2 m — what a pile-on
leap reaches) for more than 90 f is out of reach. The ring widens to 7 m
around the glance's projected landing point and the gnawers stop closing, so
nothing is bunched under an area attack. They wait; waiting costs them nothing.

**Thinking is staggered.** Gnawer *i* decides on frames where `frame % 4 == i %
4`, so a frame never holds seven decisions. Deterministic, and cheap.

**Turning is not the fight here.** A gnawer turns quickly; the Ridgeback's
turn-rate controller is kept but its limits are loose. The fight is *where they
are*, not which way they face, and a player's advantage is the ring's discipline
being predictable.

Knobs, per species family in the Oven: `pack_glance`, `pack_lead`,
`ring_radius`, `rear_bias`, `line_below`, `pack_tokens`, `token_rest`,
`howl_tokens`, `howl_frames`, `dart_reach`, `dart_lock`, `hamstring_reach`,
`hamstring_slow`, `pileon_radius`, `pileon_stagger`, `leap_reach`,
`scatter_distance`, `scatter_frames`, `rout_share`, `regroup_frames`,
`leader_hangback`, `gnaw_frames`.

## 6 · Reading it

**The camera problem is the reverse of the Ridgeback's.** A knee-high body at
your feet is hidden by your own character. Two rules:

- **Markers are drawn over the character,** as the Ridgeback's are drawn over
  the arena. A dart-bite's lane, a hamstring's latch point and a pile-on's
  tightening ring all fill toward the hit frame, and all come from
  `Monster::telegraph`'s critter equivalent asking the hit test where the hit is.
- **Who holds a token is drawn.** A gnawer with a token carries its tail up and
  its eyes catch the light. The two raised tails are the most important thing on
  the screen: they say which two can bite you, and a third tail is a howl you
  missed.

**Silhouettes.** The crouch is rump-up and wiggling, readable at knee height
from any angle. The Big One rears to **1.8 m to howl — a fighter's height** —
so the one moment it must be hit is the one moment it is the shape everything
in the game already aims at. The maul is low and wide, head sideways.

**Sound.** A chitter from the side a hamstring commits on; a yelp for the
scatter; the howl, which is long and which you hear from anywhere; a scrabble
for the scramble; grinding for the gnaw.

**Unanswerable, for this creature**, is two things the report counts: a hit
from a move whose tell was under reaction (none are), and **a hit from a gnawer
that committed while it was outside the camera's view** — off the side of the
screen, or hidden by the fighter's own model with no marker drawn. The second is
the new one, and it is what a pack does to a camera.

## 7 · The classes

**Champion.** Easy, once §1a lands. The sword's flat cuts reach a gnawer today;
the spear's jab and skewer pass 37 cm over it until the swing dips, which is
the exact case the change is for. The spear's third hit, Whirl, is the shaft
round the whole body at 3 m — the best anti-pack move in the roster and a reason
to finish the spear chain rather than stop at the jab. Rush cancels a recovery,
which makes it the Champion's pile-on escape and its route to the Big One in a
scatter. Identity: the class that likes a crowd.

**Shadow Reaver.** Good, in a particular way. Her shadow's copy turns to the
nearest body in reach (`aim::shadow_faces`), and in a pack there is always one,
so the copy is always cutting. But the tally is wasted on 120-health bodies:
marks die with what carries them. The right play is to **mark the Big One** —
send the shadow past the ring to where it hangs back — and cash in on it in the
scatter window with the dash. Her Slash is 1.2 m wide and reaches low. Identity:
she is the class the leader-first lesson was written for.

**Elementalist.** Hard today; easy after §1a. Her auto is the one that cannot
touch a gnawer (the bolt's 0.35 m grazes the crown only when centred), which is
the sharpest form of the aiming hole. Her kit is otherwise ideal: **three stones
make a horseshoe with one mouth**, the pack comes in by one lane, and the
fire pillar (2 m) and Quake (2.5 m) empty it. The gnaw is what stops the
horseshoe being a room: stand on a stone and they dig; stand in the horseshoe
and you are fine. Identity, provided the bolt can hit.

**Blood mage.** Probably easiest. Every cut spills a pool, a pack is many cuts,
and pools are her movement (the blink) and her heal. The Reaping sweep is flat,
2.4 m, low — built for this. **Grasp hauls the Big One out from behind its
ring**, which is target priority as a single button. The risk is that she has it
too easy; the report's hits-taken line will say. A pool under a dead gnawer is a
small figure — pool size follows essence, not the body — so she is not blinking
into the middle of the ring by accident.

**Dual mage.** She can decline it, like the Ridgeback's ground game, and here
it costs her. At both bars full she is on wings and above `leap_reach`, and the
pack **trees**: spreads to 7 m under her predicted landing so her Judgement
field (3.6 m) catches one or two. The bars calm toward empty always, so the
wings end, and she lands into a ring that has been waiting. Identity: the air
is hers; the pack's answer to it is patience rather than reach. Below the wings
she is a melee mage whose autos are punches from the shoulders — thrusts, and
the case §1a's swing dip exists for. The light auto's shove and backstep are
the best spacing tool anyone has against a crouch.

**Bulwark.** Hardest, and the most interesting. A guard that covers a facing arc
and turns slowly is exactly what a ring that goes to your back is built to beat.
His answers are all his identity: **plant the shield behind him** and it is the
wall — a structure (bulwark-v2) the ring's slot scoring treats as solid — at the
cost of fighting without a guard; or hold guard, take the dart-bites on the
shield, and **spend the weight on Slam** (1.4 m shake) when the heap lands. His
2.7 m hop clears the maul by more than a metre and a half. Bash's 0.9 m radius
reaches low. Identity, and a hard one: the class that must choose between its
shield in front and its shield behind.

## 8 · Coop

Two fighters back to back have no back, which is the coop answer and should be
allowed to work. So coop does not add tokens per fighter; it adds bodies.

- **Nine gnawers and the Big One, three tokens.** Ten critters is the P3 cap.
- **The pack splits pairs.** Slot scoring adds a term for the gap between the
  two targets: the wider it is, the more the ring favours the rear of the one
  further from help. Two fighters apart are two fighters with backs.
- **The pile-on goes to whoever is slowed**, and the partner breaks it: every
  leaper is a light body in the air, and any hit knocks one out. Saving a
  partner is a sweep into a heap.
- **The Big One hangs back from both**, 8 m from the nearer. Coop's route to it
  is one fighter holding the ring while the other goes.

## 9 · Measuring it

**The scripted hunter's plan** — what a person learns in the first ten minutes:

1. In the grace moment, walk to the nearest wall, stone or platform side and
   put your back to it. (The Elementalist raises her horseshoe; the Bulwark
   plants.)
2. Turn to face the centroid of the gnawers it can see; never let the facing
   drift more than 60° from it.
3. Swing at any gnawer that raises its tail within reach, before it crouches if
   the class is slow, in the crouch if it is fast.
4. Slowed: dodge out of the ring, away from the centroid -- when the pile-on's
   crouches come, or at once if one is on your heels (§13: thrown at the slow
   itself, the dodge is spent before the pile-on is called).
5. A gnawer dies: dash, Rush, blink or cross to the Big One and unload for the
   rest of the scatter; then go back to a wall.
6. Howl: hit the Big One with whatever reaches it.
7. Never cross open floor with the pack behind.

**Report lines it adds** (from the species table, P8):

| Line | Why |
| --- | --- |
| **Swings over** | Swings whose volume passed within 0.5 m above a gnawer's crown without touching it. The aiming bug detector, and this fight's `damage into feet`: zero once §1a lands |
| **Tokens live** | Share of time with the full token count held |
| **Behind you** | Share of time a gnawer was in the rear third within 3 m |
| **Hamstrings landed / thrown** and **pile-ons started / escaped** | Whether the back lesson and the slow lesson are being taught |
| **Crouches interrupted** | Dart-bites stopped by a hit, against those thrown |
| **Howls cancelled / completed** | Target priority |
| **Scatter windows used** | Windows in which the hunter hit the Big One |
| **Leader dead at**, **routs**, **regroups** | The arc of the fight |
| **Hidden commits** | Hits from a gnawer outside the camera's view when it committed — the second kind of unanswerable |

**Targets.** Tier 1: the scripted hunter wins **about nine in ten**, in **one to
two minutes**, with a third to a half of its health left in a win. Unanswerable
hits: **zero**, both kinds.

| Window | Asked for | Why it differs from the Ridgeback |
| --- | --- | --- |
| Threatening | ~30% | Tier 1: less of the fight is dangerous |
| Open to a poke | ~25% | A crouch is a poke's window, many times a minute |
| Open to a way in | ~20% | The scatter windows |
| Safe to walk up | ~25% | The token rest |

For a pack, "how long until it can move again" is the pack's: the soonest any
token holder, or anybody in a pile-on, could land. That is one new function
beside `frames_until_free`, and the four bands are computed from it unchanged.

## 10 · What it needs built

**Depends on:** P1 (species), P3 (critters and the pack brain — **built
2026-10-01 ahead of this creature**: [critters.md](../critters.md) §6 is the
recipe, and its last list says which of the below are machinery already), P8
(the harness per species). P2 only for the den: the current
28 × 28 m arena works with a den mouth at one wall's midpoint. Nothing from P4–P7.

**New and specific:** the aim change of §1a; `stands_at` reading critter
capsules; the gnaw against stones (a damage path into a stone that exists today
only for Cataclysm); the scramble onto a platform edge; morale and the den.

**The critter.** One capsule lying along its yaw: radius 0.3 m, segment 0.6 m,
centre 0.3 m up (the Big One: 0.55, 0.9, 0.55). No skeleton in the simulation:
nothing stands on a gnawer. Fighters pass through critters, and critters keep a
metre from each other; seven bodies that could wall a fighter in would be an
unanswerable trap.

**Snapshot.** Per critter, 32 B: position 12, horizontal velocity 8, yaw 2,
health 2, state 1, timer 2, animation clock 2, slot/token/role 1, target 1,
flags 1. Eight critters 256 B; the pack brain about 72 B (the glance's sample
per target 48, token rests 4, morale and rout timer 3, howl timer and lockout 4,
leader index 1, seed 4, spare). **About 330 B**, against the bestiary's 300; coop
with ten critters about 390 B. Dead critters keep their slot (and their clock,
so a corpse fading out does not pop on rollback), which is also what lets the
Broodmother spawn into a freed one.

**Per-frame cost.** Dominated by hit tests: every live volume (two fighters'
hitboxes and up to twelve effects each) against eight capsules, about two
hundred capsule tests a frame, times eight on a rollback. Separation is 28
pairs; slot assignment is 64 slot-by-gnawer scores, once per glance. No critter
raycasts. `stands_at` is one ray against eight capsules, once per cast. Well
inside a thirty-second of a frame.

**For the brood and the parasites.** The Broodmother's brood and the
Siegeshell's parasites are gnawers, so the design has to allow:

- **Morale is optional** (`rout_share` off): a brood does not rout, it is a clock.
- **The leader is optional**, and tokens can be **shared with a monster**: the
  Broodmother's own moves take one of the pack's tokens, or two bodies bite
  while she slams.
- **Spawning mid-fight into a freed slot**, from a point (a sac), without
  allocating.
- **Standing on a moving creature.** Parasites live on the Siegeshell's back, so
  a critter carries an optional mount part and a part-local position, with the
  same rule riders follow: mounted, the local position is authoritative.
  This is the largest demand, and it is why it is not built here; the gnawer's
  critter must only leave room for it (one flag bit and a part index).
- **`stands_at` must see them too**, which it does for free because it lives in
  `aim.rs`: that is the point of changing the model rather than a move.

**Tests** that pin the rules:

- `where_only_fighters_stand_every_aim_is_unchanged`
- `a_skillshot_aimed_through_a_gnawer_lands_on_the_gnawer`
- `a_standing_swing_pointed_at_a_gnawer_dips_to_meet_it`
- `a_swing_pointed_over_a_gnawer_stays_level`
- `every_class_can_touch_a_gnawer_with_its_auto`
- `only_the_token_holders_bite`
- `a_hit_in_the_crouch_knocks_the_gnawer_out_of_its_lunge`
- `a_hamstring_is_only_thrown_at_a_back`
- `the_hamstring_marker_is_inside_the_floor_the_camera_shows`
- `a_slowed_fighter_draws_every_gnawer_in_range`
- `a_dodge_taken_on_the_slow_clears_the_pile_on`
- `a_death_scatters_the_rest_for_the_window_it_promises`
- `any_hit_cancels_the_howl`
- `the_maul_is_cleared_by_every_class_hop`
- `a_wall_at_your_back_empties_the_slots_behind_you`
- `the_pack_breaks_at_half_or_at_the_leader`
- `a_routed_pack_left_alone_comes_back_and_a_watched_one_does_not`
- `a_stone_you_stand_on_falls_to_three_diggers`
- `a_pack_frame_does_not_allocate` and `a_full_pack_fits_the_snapshot`

**Milestones** — a few hours of AI time in all, each ending in something
checkable:

- **M1 · Critters and a ring.** The critter form, the pack brain with glance
  and slots, no attacks. Checkable: a sandbox hunt shows six gnawers circling at
  5 m and filling the rear first; the size and allocation tests pass;
  `determinism.rs` passes.
- **M2 · The aim change and the instrument.** `aim::stands_at`, the two
  callers, and `cargo run -p sim --bin gnawcheck`, which poses a gnawer and the
  Big One at 1, 2, 3 m and at the ring, puts the crosshair's ray through each,
  runs every class's every move through `state::hitbox` against the capsule,
  and prints, the way `beastcheck` does:

  ```text
  gnawer crown 0.6 m, the Big One 1.1 m; standing, crosshair on the middle

  Champion   spear      touches at 1 2 3 m          (was: over by 0.37 m)
  ...
  every class's auto touches a gnawer at 2 m: yes
  ```

  Checkable: every existing test unchanged, and every auto touches.
- **M3 · Bites and tokens.** Dart-bite, hamstring, tokens, the telegraph markers
  and the raised tails. Checkable: the token and crouch tests, a screenshot of
  two marked lanes.
- **M4 · The pack's behaviours.** Pile-on, scatter, morale, the den and the
  regroup, treeing. Checkable: their tests, and a hunt that routs.
- **M5 · The Big One.** Maul and howl, the hang-back, the stumble window.
  Checkable: the howl and maul tests.
- **M6 · Covering the safe spots, and measuring.** Scramble and gnaw; the
  hunter's plan and the report lines. Checkable: twelve hunts against a
  Champion inside tier 1's band, zero unanswerable, zero swings over.

## 11 · In the world

**The Commons, the Gnawer den.** The first fight place past Hearth: a grazed
meadow with a bank along one side and the den's mouth dug into it — a dark hole
the pack runs for when it breaks. Open floor in the middle, which is the lesson
(nothing to put your back to where you walk in), and at the edges the things
that answer it: the bank itself, a fallen trunk 1.5 m high they can scramble,
and two standing boulders. From the trail you see the pack's idle life: gnawers
worrying at a carcass, the Big One on the bank watching.

*Built 2026-10-01* (`crates/sim/src/arena/gnawers.rs`, dressed in
`crates/game/src/arenas/gnawers.rs`): 36 by 30 metres of grass; the bank 3 m
high along the north, the den a notch in its middle three metres wide; low
walls round the other three sides and a six-metre hedge behind them and the
bank; the trunk (1.5 m, scrambled) west of the middle, two boulders (3.5 m)
east; the carcass and the den's dark drawn as dressing. The hunters walk in
from the south, the pack musters at the den. The Big One on the bank and the
pack at the carcass before the hunt (the idle life) is not built: the pack
hunts from the first frame.

**Trophy:** the Big One's skull, as [world.md](../world.md) already lists.
Recorded on a win like any creature's (`World::hunt_won` names the pack), and
its tempers are the pack's glance, lead and cadence
(`a_won_hunt_is_the_gnawers_trophy_at_its_temper`).

**Sidegrade family: crowds — hits that spread.** *Parked with every sidegrade
(world.md §0); not built.* One, for the class it is most obvious on:

- **Blood mage, auto modifier — "Splash".** A Reaping sweep that lands on two
  or more bodies spills **one** pool between them, pooling all the essence,
  instead of a pool under each; and the tip bonus is lost. Fewer, larger pools
  in the middle of a crowd: better blinks and a better drink where the fight is,
  worse coverage everywhere else, and less damage at the point. A playstyle,
  not a number.

## 12 · Open questions

1. **Should crouch use `stands_at` too?** A crouching fighter is 0.99 m. Read
   honestly, the model says a crouch lowers "there", and the spear would dip for
   it — which is a versus change `hits_crouching` exists to decide by
   property. This document keeps crouch out; a designer should choose.
2. **Are two tokens fair or polite?** Two biters at once may read as the pack
   waiting its turn. The knob is `pack_tokens`, and the question is whether
   three with a longer `token_rest` feels more like animals.
3. **Is the leader-dead rout too cheap a win?** A Reaver who marks the Big One
   in the grace moment and cashes on the first scatter might end it in twenty
   seconds. If so, the Big One's hang-back grows, not its health.
4. **The Dual mage treed.** Is a pack waiting under her identity or a stalemate
   that wastes a person's time? The lever is how long the wings last, which is
   hers, not the pack's.
5. **Is 24 f of crouch too readable for a pack?** With two at once it may be
   right; with a howled three it may be too little. Only play says.
6. **Can fighters walk through gnawers?** It is chosen so seven bodies cannot
   wall anybody in. Whether it reads as ghosts is a person's question; a slow
   through a gnawer rather than a pass-through is the alternative.
7. **Corpses.** They stay until their slot is needed. Whether the count of
   bodies on the floor is the pack count a person reads, or clutter, is visual.

## 13 · Where it landed

Numbers from `cargo run -p hunt --bin fight -- --species gnawers --class <c>
--repeats 24`, after the passes in [feel-log.md](../feel-log.md) of
2026-10-01. The scripted hunter plays §9 with a fifteen-frame reaction; the
report's pack lines are [critters.md](../critters.md) §5's and its own are
§9's (`plans::Card::tally`).

```text
                won    mean    health left   threat / poke / way in / walk up   unanswerable
  Champion     24/24    35 s    812 of 1000      27 / 43 / 13 / 17 %                  0
  Reaver       24/24    33 s    599 of  750      25 / 45 / 11 / 20 %                  0
  Elementalist 24/24    24 s    889 of 1000      23 / 52 / 12 / 13 %                  0
  Blood mage   21/24    57 s    272 of 1000      26 / 46 /  8 / 20 %                  0
  Bulwark      24/24    35 s   1084 of 1250      26 / 45 / 11 / 18 %                  0
  Dual mage    20/24    87 s    312 of 1000      28 / 49 / 12 / 11 %                  0

  (2026-10-01, every class played: before it, the Reaver 627 left, the
   Elementalist 45 s and 585, the Blood mage 24/24 and 308, the Dual mage
   2/24 in 111 s and 24)

  coop, two Champions 12/12 in 22 s; two Bulwarks 12/12 in 23 s
  temper 3, Champion  12/12 in 33 s, 755 left, threatening 39 %

  landed / thrown, 24 Champion hunts
    Dart-bite 47/275   Hamstring 15/29   Pile-on 5/29   Maul 15/198   Howl 43
    (Blood mage: Dart-bite 41/525, Hamstring 43/65, Pile-on 43/132, Maul 97/373)
```

**2026-10-01: the hunter plays all six classes** (`hunt::class`,
[bestiary.md](../bestiary.md) §8), so the rows above are each class's own fight
rather than the Champion's pressed by everybody. **The Dual mage** wins 20 of 24
from 2: both hands now, the bars kept level and goaded up between bites (344
goads in the 24), and the punch turned so the line from the shoulder it leaves
meets the body. **The Elementalist**
wins in 24 s keeping nine tenths of her health: bolts aimed down at the
bodies, a pillar on the ring (36 over the 24 hunts) -- the fight's easiest for
any class. **The Blood mage** loses three: the scythe the plan pressed is
unchanged, and the Haemorrhage it now throws from range (55 in the 24) costs
her red. **The Reaver** sends her shadow into the ring (61) and is as quick as
the Champion. The Champion's and the Bulwark's hunts did not move.

**Against the targets.** Zero unanswerable hits of both kinds, and zero hidden
commits, for every class -- held by rule since 2026-10-01 rather than by luck
of the plan: the same gnawer, borrowed by the Broodmother and the Siegeshell,
bit from behind a pillar and from below a hunter's screen, so every pack now
lands a bite begun off your screen only through a marker under you for a
reaction ([critters.md](../critters.md) §2). The numbers here did not move a
frame. In coop the report used to count three in twelve Dual mage pairs: a
pile-on that swung to the other hunter at the next glance (a committed body
keeps its target now), and a scramble that reached nobody. The windows are near what §9 asks (30/25/20/25):
threatening on the mark, poke high and the two long windows short, because a
free token reads as a poke's window. Won nine in ten or better for five
classes. **Short and easy**: 33 to 57 seconds against one to two minutes, and
the Champion and the Bulwark keep four fifths of their health against a third
to a half. The scripted hunter is good at the one thing the fight asks --
it sees a crouch fifteen frames late and has a six-frame answer in reach --
and a person reading two raised tails and a Big One at once will be slower;
whether the fight is too easy for a person is the first question below.

**What the harness caught**, in the order it was found:

- **A ring at five metres bites nobody.** The generic pack throws a token move
  at its range from where a body stands; a dart's reach was under the ring's
  radius, and a fighter who stood still was never bitten. The pack now hands
  out a token first and the body **closes** to its windup (`mind::close_in`):
  the tail goes up and it comes in, which is also the warning a slow class
  answers by swinging before the crouch.
- **A crouch out of reach is no answer.** Crouching at 2.8 m, outside a
  sword's reach, the crouch could not be hit; at 2.1 m the Champion's sword
  reaches it. And it slid the last metre in on its momentum, so the body the
  hunter saw crouch fifteen frames before was not where its swing went: the
  crouch stops dead now.
- **Twenty-four frames was a coin toss.** The tell's arithmetic (§2) is fifteen
  of reaction and a six-frame sword; the Champion's sword starts high and is at
  knee height on its *second* active frame, and a person (or the bot) does not
  press on the first frame they could. At 24 the Champion stopped none of
  twenty-four crouches in a hunt; at 28 it stops about half (six of fifteen in
  the default hunt).
- **A lunge crawled.** A critter's `Advance` was reached through its walking
  acceleration, so a twelve-frame dart covered a third of its distance and an
  eight-frame leap almost none (`pack::drive`; every pack's lunge is at speed
  from its first frame now).
- **Darts from the side were off the screen.** The report's hidden commits
  (A5) were half of all bites: the dart is thrown only from the front arc now
  (`FrontArc`, ±37°), a body at a flank comes round into it to crouch, and a
  crouch is measured to where the glance saw the fighter rather than its lead
  (a dodge projects the lead three metres ahead). The maul is front-only too.
- **`aim::in_view` was blind with your back to a wall**: the eye it places is
  nine metres behind, through the wall. It asks the cone of the eye and the
  line of sight of the character now.
- **A hunter jumping a maul against the south wall went over it** and fought
  from outside the arena. A six-metre hedge rings the Commons.
- **A rout was a free win.** Routed gnawers do not fight, so a hunter who
  followed them to the den killed them there. A rout cornered at the den
  (`CorneredAt`, 4 m) turns and fights; one watched from further off stays
  out, as §4 says.
- **The pile-on was always dodged and never landed**, and then always landed.
  It leaps at one spot now -- where the glance put the fighter when it was
  called (`mind::pile_on`) -- so the answer is a dodge when the crouches come,
  and the heap lands where you were. The hunter's plan dodges then (§9 step 4).
  A death in the heap no longer calls it off (`CritterMove::committed`): that
  is what makes the heap a sweep target.
- **The Big One walked into the hunter and died**: it scattered *toward* the
  fighter and howled at their feet. It steps back three metres over a scatter
  now, howls only from behind the ring, and comes in only for somebody set up
  or when the pack is too few to hide behind.
- **The leader-dead rout ended most hunts at twenty seconds**, so the bodies
  are tougher than proposed (160 and 500); the token rest is 60 frames, so the
  walk-up window exists.

**Changed from this document while building**, beyond the numbers: the dart
and the hamstring are handed out by the pack and close in before their
windups (above); the scramble is a move with a token (at most two cling at
once); the maul needs no token and is thrown from in front; the den's mouth is
a notch open to the sky (a lintel hid the fight there from the camera); a rout
cornered at the den turns. Nothing in `aim.rs` changed for this creature but
`in_view`; §1a is what the foundation built.

**What the scripted hunter does not exercise.** It never stands on a stone or a
platform, so the gnaw and the scramble are pinned by tests
(`a_stone_you_stand_on_falls_to_three_diggers`,
`a_gnawer_scrambles_up_to_a_fighter_on_the_trunk_and_a_hit_knocks_it_off`)
and not by the report. It rarely reaches the Big One in a scatter (windows used
0 of 3 in a typical hunt) and never in a howl, so the stumble -- the big window
-- is pinned by `a_burst_past_its_strain_knocks_the_big_one_down` and is
otherwise unmeasured. It does not use the Champion's Rush, and the Reaver's
shadow crossing and the Blood mage's blink only now and then (one dash and one
blink in 24 hunts), which are what §4 says reach the leader in a window.
*Changed 2026-10-01:* it plays the Dual mage's bars now, and she wins 20.

**2026-10-01, polished ([plans/polish-fights.md](../plans/polish-fights.md)):
the fight is still short, and nothing was changed.** Every class played, the
won fights run 24 s (the Elementalist) to 87 s (the Dual mage), the Champion
35. Health is not the lever: gnawers at 260 and the Big One at 800 bought
the Champion 14 seconds and took the Blood mage from 21 wins to 6 and the
Dual mage from 20 to 6; the Big One alone at 1000, eleven seconds and the
Blood mage to 3; 200 and 650, five seconds and the Blood mage to 18. The
pack is short of danger rather than of health -- one hit taken in a typical
Champion hunt, two bodies winding up at once 3 % of it -- so the levers left
are the ones this section already names (the front arc, the tokens, the
crouch), and they are a person's to pull with the fight in their hands.

### Still open -- for a person

- **Is it too easy?** The scripted hunter keeps most of its health. The
  levers, cheapest first: `FrontArc` (wider means more darts, and more of them
  from the edge of the screen), the token count (open question 2: three with a
  ninety-frame rest was tried and moved the windows toward §9's but made no
  hunt harder), and the crouch's 28 frames.
- **Is 28 frames too readable for a pack?** (Question 5.) It was 24 and the
  answer was a coin toss for the class the document wrote it for.
- **Does the tail coming in read?** The dart now has a visible approach before
  its crouch. That is a warning §2 did not have.
- **The rout cornered at the den** is a decision the build made; a person
  should say whether being fought at the mouth reads as cornered animals or as
  the rout not working.
- **The hedge** makes the Commons a box from some angles; arenas.md says tall
  walls read badly. Six metres is what nobody hops.
- **The Dual mage** wins 20 of 24 now (2026-10-01), in 87 s, the slowest
  class; her dark auto also runs through a gnawer inside two metres
  (`critcheck`), the lunge-through question critters.md §7 leaves to a person.
- **Is the Elementalist's 24 s with nine tenths of her health the fight
  working?** (2026-10-01) Her bolts reach the ring from outside it.

*Changed from the seed:* the dart-bite's tell is 24 f rather than 18 (so it is
answerable on sight); the pile-on leaps in sequence rather than at once (so the
dodge answers it); the Big One's death makes the rout final; and two moves were
added — the maul answered by a jump, and the gnaw that covers a stone's top.
