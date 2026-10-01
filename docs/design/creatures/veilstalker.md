---
status: built
proposed: 2026-09-30
built: 2026-10-01
tier: 4
---

# Veilstalker — you never see it; you see what it touches

A lanky, four-metre lizard-cat that is not there. It is drawn as nothing at all
while it is cloaked, and the fight is played by reading the world it moves
through: its footprints in the snow, its breath, the embers it stirs, the
braziers it knocks over, and the paint your own hits leave on its hide. It is
the eighth creature in the build order of [the bestiary](../bestiary.md) and the
second to use P5, on sight rather than sound. It reuses everything
[the Ridgeback](../monsters.md) built — the glance, the scoring, the strain
thresholds, the floor markers, the fight report — and adds one rule that the
whole design stands on:

> **It shows itself before it strikes, every time, where you are looking.**

Everything else in this document is either how that rule is kept, or what the
fight does with the time between strikes.

---

## 1 · What the fight is

**You never see it; you see what it touches — and it always shows itself
before it strikes.**

**The body.** Nose to tail about six and a half metres, three of them tail. It
moves in two postures, and the difference between them is the fight:

- **Slinking**, shoulder at 2.2 m, belly at 1.0 m, head held low at 2.4 m. This
  is how it moves while cloaked. A level swing from a fighter's chest (about
  1.3 m) meets the ribs and the forelegs, so a swing at a place you *think* it
  is lands if you are right. The Ridgeback's lesson — a level shot from the
  floor went under the belly for three passes before anyone noticed — is why the
  belly is set at a metre and not higher.
- **Reared**, head at 4.0 m, forelegs off the floor, tail up as a counterweight.
  This is what it does for most of a decloak. It is the silhouette that says
  *it is about to strike*, and it is tall on purpose: four metres of animal
  standing up out of nothing is visible across the arena.

It is **not climbable** and there is no ride. The creature is small enough to
be hit by aiming rather than by going somewhere on it, and its back is never a
place (§3 has no "aboard").

**Health 4500**, against the Ridgeback's 7000. It is hit less often than the
Ridgeback — most of the fight it cannot be found — and a tier-4 fight is five to
ten minutes, so the total is the number of engagements it takes, not the
number of swings. About two hundred damage a clean engagement puts it at twenty
or so engagements, at fifteen to twenty-five seconds each.

**Speeds against a walk of seven.** It slinks at **6 m/s** while cloaked, and it
**bounds** at **17 m/s**, but a body moving faster than **9 m/s** shimmers
(§6). So it can catch you or it can hide, never both: a fighter walking away at
seven outpaces a cloaked slink, and the only way to close a gap faster is to be
seen doing it. That trade is the reason the shimmer threshold sits between its
two gaits and above a fighter's walk.

**Heights against the jump range.** Nothing here needs a jump to reach — its
torso is inside a standing swing. The heights that matter are *its*: it climbs
the arena's dead trunks, whose tops are at **5 m**, and pounces from them
(§2). Five metres is inside only the Dual mage's hop (6.0 m) and out of the
Bulwark's by more than two, so a trunk top is its perch and not yours, except
for her and for an Elementalist on a stone.

The loop:

1. **Hunt.** It is cloaked and somewhere. You read the floor: fresh prints in
   the snow, a breath of fog, stirred embers on the ash, a ripple in the stream.
   A fast approach shows as a shimmer. A slow one shows only as prints.
2. **The decloak.** It rears out of nothing in front of you — never behind, see
   §6 — for **at least eighteen frames** before any hit, and every decloak marks
   the floor it will strike, the way every Ridgeback windup does.
3. **Answer it.** Each move has its own answer (§2), and the decloak says which
   move it is by its silhouette before the floor marker has filled.
4. **Punish, and paint.** Its recovery is visible, and every hit you land leaves
   a mark on its hide that stays lit for six seconds. Each hit is a lamp for the
   next one.
5. **It leaves.** After two hits taken in one engagement it disengages, bounds
   away, and re-cloaks. The paint fades on the way. The hunt starts again — and
   **it is easier every time**, because a region of hide that has taken enough
   damage stops cloaking for the rest of the fight (§4).
6. **Fire.** Catch it standing in fire and it panics: fully visible, down in
   the snow, for two and a half seconds. That is the big window, and the route to
   it is reading the prints well enough to burn **where it is going**.

## 2 · The moves

Frame counts are **first guesses**, labelled as such; damage is set against a
fighter's thousand, and against the Ridgeback's "a fifth to a quarter a hit" it
is a little lighter per hit, because an invisible animal lands more of them.

| Move | Range | What it is | Tell (frames) | The answer |
| --- | --- | --- | --- | --- |
| **Ambush lunge** | 5–10 m | Rears out of the cloak, gathers, and leaps along the floor at you, jaws first | **20 decloak + 6 gather** = 26 startup, tracking at nine tenths like the Ridgeback's bite; 12 active (the flight; the hit is its last 6 and 2 on landing); **60 recovery**, fully visible. **170** | **Dodge at the lunge** — timed to its arrival, not to the decloak. Sixty visible frames after: the walk-up window |
| **Tail spear** | 3–6 m | Stays low; the tail comes over its back and drives straight forward past its head, a lance 0.6 m wide | **24 decloak**, the tail tip first; the lane is drawn from frame 0 and **locks at frame 12**; 4 active; 40 recovery. **120** | **Walk out of the line.** It is narrow and it stops following you half way through its tell; seven metres a second clears it with room. A dodge through it works too, and costs you the punish |
| **Rake, and vanish** | 0–3 m | Two low swipes of the forelegs at shin height, left then right, then it re-cloaks where it stands | **18 decloak**; 5 active, 8 between, 5 active; 30 recovery then a 30-frame fade, hittable throughout. **65 + 65** | **Jump it.** Both swipes are under a metre. The fade is the punish: it is still there, and a hit in the fade paints it |
| **Pounce** | from a trunk or the wall's ledge, 6–12 m out | Decloaks on its perch, crouches, and drops onto a circle four metres across | **22 decloak on the perch** (the floor circle is drawn from frame 0), then **20 in the air**; 3 active on landing; 50 recovery. **150** and a knockdown | **Leave the circle.** Forty-two frames at a walk is 4.9 m; the circle is 2 m in radius. Positional, not timed |
| **Quill fling** | 12–22 m | Rears, tail snaps over its head, five quills leave in a flat fan 30° wide at chest height, 35 m/s | **22 decloak**, then the flight (20 m is 34 frames); 40 recovery. **30 a quill** | **Get behind a solid** — a stone, a trunk, a platform, a planted shield. The fan's gaps at twenty metres are a gamble, not a plan |
| **Smoke** | on itself | Vents a cloud from the flanks: four metres in radius, grown over 30 frames, standing for 8 s. It goes on hunting inside it | The cloud is the tell: it grows from a point for **30 frames**, and nothing strikes into it for **60 more** | **Leave it, or burn it.** Fire burns a cloud off in 20 frames. Inside it, its decloak is hard to see (it is meant to be), so the answer is not to be in there |
| **Mimic** | 6–10 m, in view | A decloak shimmer and a rear with nothing in it; the real animal stands still elsewhere | **20 frames** of a decloak that is exactly a decloak — except that **it leaves no footprints** | **Do nothing.** Look at the floor under it. A real decloak stamps four fresh prints on its first frame; a mimic stamps none. Dodge it and you have spent your dodge at nothing |
| **Retreat** | — | After two hits in one engagement: a recoil, a bound twelve metres away at 17 m/s, and a 20-frame re-cloak | **12-frame recoil**, visible; the bound shimmers | **Follow the prints** — or **break it**: a burst past `interrupt_strain` during the recoil cancels it (§4). Not an attack; its answer is a chase |

No two rows share an answer (contract item 2), and three — the lane, the
circle, the solid — are answered by where you stand rather than by reaction.

### The subtle parts

**The decloak is the tell, and it is a floor.** Every attack begins with a
decloak of at least eighteen frames, which is three frames over a human
reaction. The Oven has a knob per move (`veil_decloak_<move>`) and
`crates/sim/tests/feel.rs` gains a line that **no knob may set below
`veil_decloak_floor`**, eighteen. A move faster than the floor is not a
tuning question, it is a different creature.

**The silhouette tells the move before the floor does.** Four rears, four
shapes: the lunge rears and crouches back on its haunches; the spear stays low
and the tail rises first; the rake does not rear at all, it spreads its
forelegs; the quills rear highest with the tail over the head. Read from the
first eight frames of the decloak — the reveal ramps up from nothing, so the
shape at frame eight is at about half strength — which leaves ten frames for
the answer. The floor marker is there for the ones the silhouette did not
settle.

**Why the mimic is fair.** It tests the patient read, and that is only a test
if the patient read is *available*. A real decloak always stamps four prints
on its first frame (its feet setting); the mimic stamps none; the real animal
is standing still while the mimic plays, so it leaves none either. Everything
needed is on the floor, in view, for twenty frames. And the mimic is never
followed by a strike inside **60 frames** (`veil_mimic_quiet`), so somebody who
read it right is not punished for having looked down.

**Why the smoke is fair.** Inside its own cloud the decloak is drawn through
the smoke and is hard to see — that is what the cloud is *for*. So the rule is
positional: **it never strikes out of the cloud at somebody outside it, and it
never strikes into it until 90 frames after it began to grow.** Walking out
from the middle is four metres at seven a second, 34 frames. The fight report
counts hits inside smoke as answered by standing (like the Ridgeback's stomp),
and the hunter's plan leaves.

**The re-cloak is visible and hittable.** Every cloak-in takes thirty frames of
fading, and a body that is fading is still a body. The rake's name is a
promise that it *vanishes*, not that the vanish is free.

## 3 · A threat at every range

**Close (0–3 m): the rake.** Fast by this creature's standards (18 frames) and
low, so the answer is a jump rather than a dodge. Standing next to it after a
lunge's recovery is how you get raked.

**Mid (3–10 m): the lunge and the spear.** The two moves it throws most, and
the pair that makes mid range a question: the lunge is dodged at its arrival,
the spear is walked out of before its lock. The rears are different enough to
tell apart in eight frames; an answer thrown to the wrong one is punished by
the other.

**Long (10–22 m): the quills, and the approach.** At range it flings a fan,
answered from behind a solid. It also closes — at 17 m/s, shimmering, which is
the one time it is visible while moving and the reason long range is where you
see it *coming*. A fighter backpedalling at seven from a bounding animal is
caught; one who turns and watches the shimmer is ready.

**Above: the pounce.** Off a trunk top or the wall's ledge onto a circle.
Anyone standing near a trunk is under it.

**The tempting safe spot is a corner, back to the wall.** Nothing can decloak
behind you there, so the whole creature is in front, where you are looking.
What covers it: the **pounce** from the wall's ledge lands in the corner, the
**smoke** dropped on a corner leaves you inside it with a wall at your back
and one way out, and the **quills** do not care where your back is. A corner
beside a brazier is better — fire is close — which is why the braziers stand
out in the open.

**The second tempting spot is a trunk top**, for the classes that can reach
one: nothing on the floor reaches five metres. Covered by the **quills**,
which fly at chest height from a rear of four metres and so reach a body
standing on a trunk, and by the **pounce** from the next trunk.

## 4 · The approach and the window

**What you earn: paint.** A hit leaves a mark on its hide at the point of
contact, in the frame of the part it landed on, so it moves with the body. The
mark is lit for **six seconds** (360 frames, `veil_paint_frames`) and fades
over the last second. Four marks at a time; a fifth replaces the oldest. A
painted animal is not visible, but the paint is — a patch of hide hanging in
the air, rearing when it rears — and one lit mark is enough to follow it. **The
first hit of an engagement is the hard one; the second is aimed at a lamp.**

**What changes for the rest of the fight: mottling.** The hide is four
regions — head and neck, left flank, right flank, tail. Each keeps its own
damage total, and a region that has taken **500** (`veil_mottle_damage`) is
**mottled**: its patch of hide no longer cloaks. It stays drawn at full
strength for the rest of the fight. One mottled region makes it trackable in
the open; three make it a large animal with a few holes in it. **The fight gets
easier the better you did**, and the player can see which region they earned,
and where they hit it. It is the Ridgeback's broken foot, turned to reading.

The Retreat reads it too: a mottled region is a lamp it cannot put out, so a
mottled animal **stalks from further out** (`veil_mottled_reach`, +2 m a
region) and uses the smoke more.

**The window: fire panic.** If any part of it spends **20 frames** in fire —
a fire pillar, burning ground, a cloud of embers, a lit stone, a knocked
brazier's coals — it **panics**: **150 frames**, fully visible, rolling and
thrashing in the snow at a height of a metre and a half, throwing nothing. It
then gets up, fully visible for 30 frames more, and runs. Every class's swings
land on a rolling animal; it is the one time in the fight where the answer is
"everybody, everything". A panic starts a **1200-frame lockout** on the next
one (`veil_panic_lockout`), so fire is a window and not a lock.

**The route to the window is the read.** It sees fire and will not walk into
it. So fire placed where it *is* is avoided, and the panic comes from fire
placed where it is *going* — the head of a line of prints, a knocked brazier
tipped across its path, the cloud of embers it has to cross to reach you.
That is the approach this creature teaches: the prints are a direction, and a
direction is a prediction.

**Every class can make fire.** Four iron braziers stand in the arena, each
tippable once by any hit. A tipped brazier spills burning coals — a burning
floor patch (P4) two metres across that burns for **8 s** before the snow puts
it out. Four braziers is four fire windows for the classes without fire of
their own, and the creature knocks them over too when it bounds through one,
which is both a trace and a waste of a window you were saving.

**Outlines.** Standing in water, smoke that is *not* its own, or fire, any part
of it inside the hazard is drawn as an outline. The stream is the permanent
one: a ripple ring around each leg, and it cannot be crossed unseen — but prints
end at the water, which is how it loses a tracker.

**Strain and crowd control.** The strain thresholds apply unchanged. A lighter
animal sits lower on the scale than the Ridgeback, so `cc_strain` and
`interrupt_strain` are about two thirds of its. Two rules specific to it:

- **A hit during a decloak flinches it** like any startup, and a burst past
  `interrupt_strain` cancels the move and throws it back into full view for its
  recovery. Shooting at a line of prints and catching a decloak is a real play.
- **A burst past `interrupt_strain` in the Retreat's recoil cancels the
  retreat**: it stays, visible, for a 40-frame stagger, and the engagement goes
  on. The retreat's twelve frames are the window, and a class that can put a
  burst into twelve frames can keep it in the fight.

**The arc.** Fresh, it stalks long (three to four seconds between
engagements), throws the lunge and the spear, and leaves after two hits. Below
**60%** it starts to mimic and smoke. Below **30%**, desperate:

- it leaves after **three** hits rather than two, because it is committing
  harder;
- its **cloak frays** — the shimmer threshold falls from 9 m/s to 5, so even a
  slink is a faint shape;
- the strain thresholds fall by `strain_desperation`, as on the Ridgeback;
- the panic lockout halves.

The first half is patient; the last is a frayed, mottled animal fighting
harder and hiding worse, which is what winning looks like.

## 5 · The brain

The Ridgeback's algorithm, with three things added and one gate.

**It glances, and the glance has a head in it.** Every `veil_glance` frames
(10, against the Ridgeback's) it samples each target's position and velocity
**and the direction their camera is looking** — the fighter's look, which is
drawn as the character's head and shoulders turning. It acts on that sample,
which is at least one glance old. It is the same stale information the
Ridgeback aims with; it is not reading the mouse any more than aiming at a
position is reading `W`. `lead` is 8 frames.

**The view gate.** A move scores **nothing** unless the point it will decloak
at is inside the target's view as last glanced — inside a cone of
`veil_view_cone` (40° either side, inside the 44° half-width of the drawn 58°
frame on a 16:9 screen), and not behind a solid. That is `aim::in_view`
(§10), and it is the whole of §6's fairness rule on the creature's side.

```text
U(m) =  range(m)        as the Ridgeback
      + arc(m)          as the Ridgeback, from its own facing
      + variety(m)      as the Ridgeback
      + hurt(m)         as the Ridgeback
      + edge(m)         prefers a decloak near the edge of the target's view
      + bait(m)         the mimic, when it saw the target dodge recently
      + exposure(m)     painted or mottled: the smoke and the retreat score up,
                        engagements score down
      + leave(m)        enormous for the retreat after two hits this engagement
      ⟂ view(m)         outside the glanced view cone, or occluded: nothing
      ⟂ cooldown(m)     as the Ridgeback
      ⟂ fire(m)         a decloak point inside fire: nothing
```

- **`edge`** (`veil_edge_bias`) is why it tends to come from the side of the
  screen. It keeps the rule — it is in view — and it makes the read a skill: a
  player learns to watch the edges of the frame. It is a pattern, and a
  pattern a person can name is the test of a difficulty knob.
- **`bait`** (`veil_bait_appetite`) rises when the last glances saw the target
  moving at a dodge's speed. It reacts to something it saw, fifteen frames old
  or more, which is contract item 5.
- **`leave`** is the rider term's shape reused: once two hits have landed since
  the last cloak, the retreat outscores everything.

**It stalks between engagements.** After a cloak it spends `veil_stalk_frames`
(180–240, drawn from the seed) closing to its chosen range at a slink,
choosing a circle round the target that keeps it near the edge of their
glanced view. It crosses water only when it is being followed (it saw the
target walking toward its last position in two consecutive glances), because
crossing water breaks the trail.

**Decisiveness is lower than the Ridgeback's.** Its moves are fewer at any one
range, so the weighted draw has to spread further for the distribution to be
learnable and the next move not.

**Cooldowns.** Smoke 900 frames, mimic 400, pounce 300 (it has to climb back
up). The lunge and the spear have none past the shared variety penalty; they
are the fight.

## 6 · Reading it

**The one rule: it decloaks where you are looking.** Every attack and every
mimic decloaks inside the target's view as the creature last glanced it. **It
never strikes from off-screen.**

Is an off-screen decloak fair? Only with a cue that reaches a player who is not
looking, and the game has **no sound**. A decloak behind you with nothing to
hear is a hit from nowhere, which is the one thing the contract forbids. So
the creature waits until it is in front — which it can always arrange, by
walking — and the predator's fantasy of "from behind" becomes "from the edge
of the frame", which is scarier than it sounds, because the edge is where a
person is not looking. If audio arrives, an off-screen decloak with a longer
tell (36 frames) and a directional cue is the obvious next step, and it is an
open question (§12).

The gate uses the *glanced* view, so a player who whips the camera away during
a decloak can be hit by one they no longer see. That is the same case as the
Ridgeback's "a hunter walking into a stomp at a full run": the report measures
from what the target was looking at **when it committed**.

**What "unanswerable" means here.** A hit counts as unanswerable if either:

- its tell was under fifteen frames with no positional answer (the standard
  test; nothing in the set should ever trip it), or
- **its decloak was inside the target's view, at commit, for fewer than
  fifteen of its frames** — measured with `aim::in_view` from the target's eye
  and look on each frame of the decloak, against the look at commit.

Both must be zero. Hits inside the creature's own smoke are answered by
position (leave it) and are counted as such, on a line of their own.

**Everything the player reads, drawn from the simulation:**

| Trace | What it says | Where it comes from |
| --- | --- | --- |
| Footprints in snow | Where it walked, 8 s | the footfall ring in the snapshot |
| Footprints in ash, and stirred embers | Where it walked, 4 s, and a glow for 1 s | the same ring, floor material from the arena |
| Breath fog | Where its head is, a puff every 90 frames | derived from the frame count and the head's position; nothing stored |
| Ripple rings | A leg in the stream | derived from the parts and the arena's water |
| Shimmer | A body over 9 m/s | its velocity |
| Paint | Where you hit it, 6 s | the paint ring |
| Mottled hide | Where it has been hurt enough | four bits |
| A knocked brazier | It went through there | the brazier's state |
| The decloak | Which move, and where it will land | the veil state, and `Monster::telegraph` |

**Footprints are the creature's alone.** Fighters leave none. Six hunters'
prints would bury its trail, and the read is only a read if every print on the
floor means the same thing. Its prints are three-toed and dark; nothing else
in the arena is either.

**The simulation owns visibility.** Cloak state — cloaked, shimmering,
decloaking and its frame, shown, re-cloaking, panicked — is in the snapshot,
and so are the paint and the mottled regions. The renderer draws the
creature's parts at the strength one function answers, `Veil::shown(part)`,
which is the largest of: the decloak ramp, the panic, the shimmer, a painted
point on that part, a mottled region, and an outline from a hazard. **The
fight report and the scripted hunter read the same function**, so the hunter
sees exactly what a person sees and "visible" means one thing. That is the
rule that the overlay draws what the hit test uses, applied to sight.

**Being unseen is not being untouchable.** Cloak is never a hit-test fact.
Bodies are not on the crosshair's ray, so a skillshot aimed at a fresh print
goes to the middle of a body standing there (`aim::standing_middle`, about
0.9 m) and `aim::first_along` meets the leg or the belly on the way if it is
there. Swinging at a patch of snow where prints are appearing is a real
attack.

## 7 · The classes

**Shadow Reaver — easy, and identity.** Her shadow out on the field turns its
copy toward the nearest body in reach (`aim::shadow_faces`), and a cloaked
body is a body. So a shadow left out in the snow is a sentry: when it turns,
something is there. Marks the shadow lands are drawn as pips over the victim,
which on this creature are pips hanging over nothing. Two bodies watch twice
the floor. Her hole is her health: 750, and a lunge is 170.

**Elementalist — the class this creature is afraid of.** Fire pillars on the
head of a trail are the panic, a cloud of embers is an outline where it
stands, a lit stone is a door it will not walk through, and her stones are
the solids the quills are answered behind. The worry is that it is too easy
for her, and the lever if it is: fire placed *on* it (not ahead of it) takes
the full twenty frames to panic, and her pillar's startup is long enough that
a slinking animal has walked out of the base by then. Her long startups are
the other half: everything she casts, it sees.

**Blood mage — tracks in blood.** Her hits spill it, and the creature walking
through one of her pools leaves **red prints for four seconds** after — the
pool marks where it was, and the red trail says where it went. Her blink to a
pool puts her on its last position. Grasp hauls a caught body to her, cloaked
or not, and the haul is a hit, so it paints. Medium: her game is close range
and the rake is close range.

**Dual mage — the view from above.** She floats, and prints are easier to read
looking down; a trunk top is inside her hop, which takes the pounce's perch
away from it. Her dark Lance **tethers** what it hits, and a tether drawn to an
invisible animal is a line to it. Her burden is the mimic: a floating fighter
who dodged at nothing is a floating fighter coming down. Easy, and identity —
the height is her thing here as on the Ridgeback.

**Champion — hard, and a question.** Three-hit chains meet a creature that
leaves after two, and chasing prints is not what a sword is for. His answer is
the only one the retreat has: a chained string is the burst that crosses
`interrupt_strain` inside the recoil's twelve frames, so he is the class that
**keeps it in the fight**. Rush closes on a shimmer. Whether a Champion who
cannot land a chain finds the fight dull is an open question.

**Bulwark — hard, and the sharpest case.** A frontal arc against an ambusher.
Two things help. The decloak rule means it is always in front of his camera,
and so, with his camera and his guard facing the same way, always in the arc;
and the parry, the opening frames of the block, is the best answer to the
lunge in the game — a parried lunge staggers it in full view. His planted
shield is a solid, which is his cover against quills and his wall for the
corner. His hole is the pounce from above and the rake, which is low and fast
and wants a jump, and his jump is the lowest. Identity, probably; the pounce
is the thing to watch.

## 8 · Coop

**Two cameras cover more of the circle.** It still decloaks only in its
target's view, but the partner, looking from elsewhere, sees the decloak from
the side, and the side is where the rear's silhouette is clearest. One player
baits — stands in the open on snow — and the other watches the floor around
them. That is the coop game.

**A hit paints for everybody.** That is the mark: one player lands a hit, and
both have a lamp for six seconds. The Reaver's pips, the Blood mage's red
prints and the Dual mage's tether are all shared the same way, because they
are drawn in the world, not on one player's HUD.

**It hunts the one who cannot see it.** With two targets, `edge` and `view`
are scored per target, and a target whose partner is looking at the decloak
point is scored down (`veil_watched_penalty`) — it prefers the hunter who is
alone in their own view.

**Numbers for two:** health ×1.6 (7200), the retreat after two hits *from
either*, and the stalk shortened by a quarter so there are more engagements.
The braziers stay at four; they are a shared resource, and deciding who tips
which is a conversation.

## 9 · Measuring it

**The scripted hunter's plan** is what a person learns in the first ten
minutes. It reads only what is drawn — `Veil::shown`, the footfall ring
filtered to what is on screen through its own `aim::in_view`, the paint and the
hazards — never the creature's position.

1. **Stand in the open, on snow, and watch the floor.** Sweep the camera
   slowly; face any print younger than a second within twelve metres.
2. **Answer the rear, not the shimmer.** On a decloak: if there are no fresh
   prints under it, do nothing (mimic). Otherwise read the silhouette at frame
   eight — lunge: dodge at its arrival; spear: walk sideways; rake: jump;
   pounce circle: walk out; quills: step behind the nearest solid, or dodge if
   none is within three metres.
3. **Punish the recovery**, and unload into the second hit on the paint.
4. **Follow the trail** after a retreat at a walk, facing the newest print.
5. **Leave smoke** at right angles to the way in.
6. **Tip a brazier across the trail** when the newest prints are heading toward
   one and within six metres of it.

**Fight report lines it adds:**

- **Blind hits** — hits whose decloak was in view for fewer than fifteen of its
  frames, at commit. **Must be zero.** (This is its unanswerable line.)
- **Smoke hits** — hits taken inside its smoke; answered by position, counted
  apart.
- **Mimics: dodged at / held** — whether the mimic is readable. If the scripted
  hunter, which reads prints perfectly, dodges any, the mimic stamped a print.
- **Time to find** — frames from a retreat to the next hit on it or its next
  decloak. The hunt's own length.
- **Print lead** — frames from the first print within ten metres of the hunter
  to the decloak. Short means the trail is too faint to use.
- **Paint uptime** and **follow-up rate** — share of frames with a lit mark, and
  share of engagements where the second hit landed.
- **Mottled regions** at the end, and **when** each went.
- **Panics**, and what fire caused each (pillar, cloud, stone, brazier).
- **Decloaks by view position** — centre, middle, edge thirds. If it is all
  edge, `veil_edge_bias` is too high.

**The windows.** "How long until it can move again" is not enough for an
animal that is free the whole time it stalks; the report adds the decloak floor
to `frames_until_free` for this species — the soonest it can *hit* — so a
stalk counts as threatening only once it is within lunge range and in view.

| Measure | Target |
| --- | --- |
| Scripted hunter wins | about one in six (tier 4) |
| Fight length, won | 5–10 min |
| Threatening | ~40% |
| Open to a poke | ~10% |
| Open to a way in | ~20% |
| Safe to walk up | ~30% — higher than the Ridgeback's; its recoveries are long and visible, and the hard part is finding them |
| Blind hits | 0 |
| Every move thrown and landed at least once | yes |

## 10 · What it needs built

**Shared machinery:** P1 (species), **P2** (a new arena with a floor material per
region and water), **P4** (smoke, burning coals; four hazards), **P5** (the
perception filter — here pointed the other way: what the *hunter* can see is
what the species exposes, and the glance samples the look), P8 (a plan and
report lines). Not P3, P6 or P7.

**New and specific to it:**

- **`aim::in_view(who, look, at, scene) -> bool`** — *is that point on this
  player's screen and not behind a solid.* It is built from the eye and the
  look, so it belongs in `aim.rs` beside `pointing_at` and is enforced there by
  `one_aim.rs`; the list of non-line-of-effect functions in CLAUDE.md gains a
  sentence for it. The brain calls it with the glanced look, the report with
  the live one.
- **The veil**: cloak state, `Veil::shown(part)`, the paint ring, the mottle
  bits and region totals.
- **The footfall ring**: 32 prints, stamped by the gait on each foot's contact
  and by every real decloak, read by the renderer as decals.
- **Braziers**: four arena objects, tippable once, that spawn a P4 burning
  patch.
- **Rendering**: a shimmer pass (the creature's mesh drawn as an offset of
  what is behind it), print decals, breath puffs, paint. If a copy of the frame
  is too dear for `view`'s budget, a dithered silhouette at the same strength
  is the fallback.

**Snapshot, about 390 bytes**, a hundred over the bestiary's estimate:

| Piece | Bytes |
| --- | --- |
| Monster (a lighter rig than the Ridgeback's: 12 bones) | ~190 |
| Footfall ring: 32 × 4 (position at 12.5 cm, foot, birth frame mod 8192) | 128 |
| Paint ring: 4 × 4 (part, local point, age) | 16 |
| Veil state, region totals, mottle bits, retreat count | ~16 |
| Hazards: 4 × 8 | 32 |
| Braziers | 1 |

The ring is what grew: sixteen prints at four footfalls a second is four
seconds of trail, and the read wants eight.

**Per-frame cost** is dominated by the hazard overlap (twelve parts against
four hazards for outlines and fire) and, at decision points only, one
`in_view` per target — a cone test and one `clear_between`. Quills are five
`first_along` a frame for under a second. All cheap; none allocates. The
renderer's shimmer pass is the one to watch against `view`'s eighth.

**Tests that pin its rules:**

- `every_attack_decloaks_for_at_least_eighteen_frames_first`
- `it_never_commits_to_a_decloak_outside_the_target_s_glanced_view`
- `a_real_decloak_stamps_its_feet_and_a_mimic_stamps_nothing`
- `no_strike_follows_a_mimic_inside_the_quiet`
- `every_footfall_is_in_the_ring_and_a_rollback_lays_the_same_trail`
- `a_cloaked_body_is_hit_exactly_like_a_visible_one`
- `a_hit_paints_the_hide_where_it_landed_and_the_paint_moves_with_it`
- `a_region_that_has_taken_enough_never_cloaks_again`
- `twenty_frames_in_fire_panics_it_and_it_is_shown_throughout`
- `it_never_strikes_out_of_its_own_smoke_or_into_it_early`
- `after_two_hits_it_leaves_and_a_burst_in_the_recoil_keeps_it`
- `what_is_drawn_is_what_the_hunter_and_the_report_call_visible`
- `no_decloak_knob_may_go_below_the_floor` (in `feel.rs`)

**Milestones** — about six hours of AI implementation in all, each ending in
something checkable:

- **M1 · The body and the veil.** Species entry, rig, slink and bound, cloak
  state, `Veil::shown`, drawn at that strength; the F1 overlay draws it
  whole. *Check:* a screenshot of it cloaked, shimmering and shown; the
  first-frame test for `a_cloaked_body_is_hit_exactly_like_a_visible_one`.
- **M2 · The Ashwood and the trail.** The arena as P2 data (snow, ash, stream,
  trunks, braziers), the footfall ring, prints drawn. *Check:* the rollback
  trail test; a screenshot of a trail across snow and ash.
- **M3 · The strikes.** Lunge, spear, rake, pounce, quills, each with its
  decloak and floor marker from `Monster::telegraph`. *Check:* the decloak
  floor test and a `SHOT_MOVE` screenshot of each.
- **M4 · Paint, mottle, fire.** The paint ring, regions, P4 smoke and coals,
  outlines, panic, braziers. *Check:* the paint, mottle and panic tests.
- **M5 · The brain.** `aim::in_view`, the view gate, edge, bait, exposure,
  leave, stalking, smoke, mimic, retreat. *Check:* the view-gate, mimic and
  retreat tests; `determinism.rs` still green.
- **M6 · The hunter and the report.** The plan above, the new lines, twelve
  seeds. *Check:* blind hits zero; the four windows printed against §9's
  targets; the first tuning pass recorded in the feel log.

## 11 · In the world

**Region: the Ashwood** — a northern birch wood that the charcoal-burners
worked until something started taking them. The arena is their last clearing:
**36 × 36 m**, larger than the current 28 because a hunt needs room for a trail
to be a trail. The floor is snow, with grey **ash** round the old burning pits
(prints last half as long there, and stir embers), and a shallow **stream**
three metres wide across one corner. **Six dead trunks**, 5 m tall with flat
broken tops — its perches, and solids for the quills. **Four iron braziers**
the burners left lit, standing in the open. The wall is **3 m** of stacked
cordwood with a walkable ledge on top, which is the other perch, and which
the Ridgeback's 1.5 m wall would not be.

**Trophy: a mottled pelt** — its hide, with the regions you mottled drawn as the
patches they were, so the trophy of a fight records how that fight went.

**Sidegrade: Veilstep** — a common-mechanic modifier in the spirit of
[parked.md](../parked.md). The first six frames of your dodge are drawn as a
shimmer rather than as your body, so an opponent reading your dodge's direction
sees a smear; the dodge's invulnerability is **two frames shorter**. It changes
how a dodge reads — to the other player, not to you — at the price of the
margin that makes a dodge forgiving. Useful in versus against a reader, and a
liability in a hunt. A playstyle, not power.

## 12 · Open questions

1. **Off-screen decloaks.** The rule is "never", because there is no sound. Is
   "it always comes from in front of your camera" a fair rule that reads as
   eerie, or a rule a player spots in two minutes and then plays around by
   pointing the camera at a wall? If audio comes, should a behind-you decloak
   with a 36-frame tell and a directional hiss replace it?
2. **Glancing the look.** The creature samples which way your camera faces, a
   glance old. It is the character's head turning, and it is stale — but it is
   the nearest thing in the cast to reading the mouse. Keep it, or gate on the
   character's body facing instead (which follows movement, not the camera)?
3. **Does a shimmer at 9 m/s read at all** in a busy arena on a laptop screen?
   Only a person looking at one can say, and the threshold is the whole
   long-range game.
4. **Is the Elementalist's fight too easy?** Fire is her kit and the window is
   fire. The lever is written (§7); whether it is needed is a playing question.
5. **Does the Champion enjoy it?** A creature that leaves after two hits against
   a class built on three is either his best fight (he is the one who can stop
   it leaving) or his dullest.
6. **Should fighters leave footprints?** None is cleaner to read; some would let
   a hunter follow a partner and let the creature's trail be lost in the
   crowd — which could be a mechanic rather than a flaw.
7. **Six seconds of paint, 500 to mottle.** Both decide how fast the fight
   becomes easy, and both are first guesses.

## 13 · Where it landed

Built 2026-10-01: `--hunt veilstalker`, the Ashwood. The species is
`sim/src/species/veilstalker/` (the table and its own knobs; `fight.rs` for
the veil, the glance of the look and the view gate, the footfall and paint
rings, the regions and the mottle, fire and panic, the smoke, the quills, the
mimic's ghost, the retreat, the perches and the leaps -- run in the frame
hook and kept in the hunt's lore; `mind.rs` for appetite and where it walks),
the arena `sim/src/arena/veilstalker.rs`, the clips
`anim/src/beast/veilstalker/`, the look, the veil's materials, the Ashwood's
dressing and the trail, paint, breath, ripples and quills in `game`
(`species/veilstalker.rs`, `arenas/veilstalker.rs`, `beast.rs`, `veil.rs`),
the plan, the view and the report lines `hunt/src/plans/veilstalker.rs`, and
the rules as sentences in `sim/tests/veilstalker.rs`, `sim/tests/feel.rs`
(`no_decloak_knob_may_go_below_the_floor`) and `hunt/tests/veilstalker.rs`.
The plan is [plans/veilstalker.md](../plans/veilstalker.md); the passes are in
[feel-log.md](../feel-log.md) of 2026-10-01.

Numbers from `cargo run -p hunt --bin fight -- --species veilstalker --class
<c> --repeats 24`; the scripted hunter plays §9 with a fifteen-frame reaction,
a camera it sweeps, sight of only what `World::shown`, the prints, the paint,
the apparition and the floor markers put on its screen, and a timed answer
off by a tenth of the wait it judged (below).

```text
                won    mean win   threat / poke / way in / walk up   unanswerable (blind)
  Champion      6/24    336 s         45 /  8 /  6 / 40 %                0
  Bulwark       8/24    463 s         44 /  7 /  7 / 43 %                0
  Reaver        3/24    433 s         46 /  8 /  6 / 40 %                0
  Elementalist  5/24    347 s         24 /  7 /  8 / 62 %                0
  Blood mage    0/24      --          34 / 10 /  9 / 47 %                0
  Dual mage     0/24      --          48 /  9 /  6 / 38 %                0

  coop, two Champions 12/12 in 303 s;  temper 3, Champion 5/12 in 344 s

  landed / thrown, 24 hunts of each class
    Ambush lunge 9/137   Tail spear 4/73   Rake 66/292   Rake, again 46/194
    Pounce 1/108   Quill fling 46/134   (Champion's 24)
    Pounce 39/199 (Elementalist's, on the trunk tops), every strike landed
    in some class's hunts; Smoke, Mimic, Retreat, Climb, Get up do no damage
```

**Its own lines**, over single Champion hunts: blind hits **zero** of every
hit; print lead four to eight seconds; time to find a retreated animal two to
four seconds; paint lit half to two thirds of the fight; the second hit of an
engagement landed one time in three to ten; mimics held, never dodged, a
handful a hunt; decloaks three quarters in the centre third of the screen,
almost never at the edge; one or two retreats broken by a burst in the recoil
in a long hunt; panics rare (the Champion tips a brazier now and then) and no
cloud burnt off -- the harness's Champion carries no fire.

**Against the targets.** **Zero blind hits and zero unanswerable** in all
144 hunts of the six classes, in coop and at temper 3. Twenty-two of the 144
won, about one in six and a half -- tier 4 -- in five and a half to eight
minutes (five to ten asked). Threatening 44-48 % against ~40. **What is
off:**

- **The way in is 6-8 %, against ~20, and walking up ~40 % against ~30.** The
  windows read `frames_until_free` plus the decloak floor (`Tally::until_free`),
  and the animal's long recoveries are visible and *walkable*: the hunter
  is near enough to walk in, so the time that §9 calls the way in reads as
  walk-up. The split between them is the measure's, not the fight's.
- **The lunge and the pounce land rarely** against a hunter that reads every
  silhouette (one in fifteen; one in a hundred against the Champion, who
  walks out of every circle; one in five against the Elementalist on a trunk
  top). They are answered as designed; whether a person answers them as well
  is the playing question.
- **The Blood mage and the Dual mage lose every hunt**, as against every
  creature: the harness does not play their pools or their bars.
- **The Elementalist's fire is not played**: the harness casts no fire at
  the clouds or the hide, so §7's worry (her fight too easy) is untested.

**Changed from this document while building**, beyond the numbers:

- **Health 7 000** (11 200 with two hunters), not 4 500, and **the hits
  heavier**: lunge 260, spear 190, rake 140 + 140, pounce 220, quill 70; a
  region mottles at **800**. At the document's damage the Champion won ten to
  twelve hunts in twelve with half his health left, and fights at 4 500 ran
  short of five minutes: a hunter that reads the floor perfectly is hit only
  when it is slow, so each hit has to count.
- **The stalk is 90-150 frames at 11 m**, and the first one starts when the
  hunt's grace ends rather than under it.
- **Eighteen bones**, the Pair's cat topology, not twelve: bones cost the
  snapshot nothing (§10's "lighter rig").
- **The glance is the facing**: the simulation keeps no input, and a
  fighter's facing is the look's yaw whenever they can act. The eye is built
  by `aim::in_view_from` from `camera::eye_under`, level.
- **The view gate is "plainly in view"**: from where the hunter stood and a
  metre either side (`ViewMargin`), through a few degrees of the look's
  wander (`ViewSweep`), and of where a strike thrown out of a bound will stop
  skidding as well as where it starts. Each closed a way a stale glance
  became a blind hit in the harness.
- **At most three strikes an engagement** (`EngagementStrikes`), not counting
  mimics; hits closer than eight frames are one hit for the retreat
  (`HitGap`); a painted animal close to its hunter while it stalks runs off
  (`ExposedNear`); it will not strike on bare ground (no prints there); it
  walks round a trunk on its way rather than into it (`PostWidth`,
  `PostClear`).
- **The hunter's timing is judged, not known**: a dodge timed to an arrival
  is off by up to a tenth of the frames it judged across, either way, beside
  the fixed three. Without it the harness dodged a lunge on the same frame
  from any distance, which no person does; it is the only change that made
  the lunge land at all.
- **Seams**: `FightDecl::apparition` and `World::apparition` (the mimic's
  ghost, drawn and read like a body), `aim::in_view_from` and `aim::off_look`,
  `Tally::until_free`, and `MonsterField::Cooldown` widened to 900 for the
  smoke; listed in [species.md](../species.md).

**Not built**: the Veilstep sidegrade (parked with the rest of §11's
sidegrades); a foreign smoke cloud outlining it; the water-crossing rule for
prints (ripples are drawn, the trail simply has no prints in water); the
trophy is the hunt's ordinary trophy, not the mottled pelt; the hunter does
not read the breath (it is drawn); the shimmer is a translucent silhouette,
the stated fallback, not a refraction of what is behind it.

