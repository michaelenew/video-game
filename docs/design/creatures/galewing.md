---
status: built
proposed: 2026-09-30
built: 2026-10-01
tier: 4
---

# Galewing — the fight that happens overhead

A raptor with an eighteen-metre wingspan that spends most of the fight circling
above everyone's reach, comes down only to attack, and can be knocked out of the
air by hitting its wings while it is low. Once it is down you can climb onto it,
and when it takes off again, anybody still on its back goes up with it.

It is the tenth creature in the build order of [the bestiary](../bestiary.md),
and it is late on purpose: it brings **P6, flight and the long ride**, the
largest single piece of new machinery in the cast. Everything else it uses is
the [Ridgeback](../monsters.md)'s: the glance, the scoring, the strain
thresholds, the floor markers, the grip test and the fight report. Where this
document does not say how something works, it works the way monsters.md says.

---

## 1 · What the fight is

**It lives where you can't reach — make it come down, then choose whether to go
up with it.**

The body is eight metres from beak to tail. The wings are eighteen metres tip to
tip, three metres deep where they meet the body and one metre deep at the tips.
It has 9000 health (the Ridgeback has 7000), because a third of this fight is
time in which nobody can touch it.

Its heights are the whole design, so here they are against the jump range. A
full hop apexes at 2.7 m (Bulwark) to 6.0 m (Dual mage), and one of the arena's
1.5 m platforms adds 1.5 to either.

| Where it is | Lowest part of it | Who can reach it |
| --- | --- | --- |
| **Circling** | 10–20 m, cruising at 16 | nobody from the floor; the Dual mage from the tower top (12 + 6.0); of the skillshots, only the Elementalist's air row, thrown from a hop |
| **Hovering** (Downwash) | its feet, 6.8 m | nobody from the floor; the Dual mage, Reaver and Elementalist from a stone |
| **Low pass** (Talon pass) | talons 1.2 m, wing undersides 3.0 m | wings: every class with a hop and a swing, the Bulwark included (2.7 m + Bash's 1.5 m reach) |
| **After a Stoop** | wings spread at 0.4–1.6 m | walk up and swing |
| **Crashed** | back at 2.4 m; wings flat on the floor, 0.3 m at the tip | walk up the wing onto the back, every class |
| **Perched** on the tower | back at 15.4 m | whoever climbed the tower |
| **Grounded for good** (both wings broken) | back at 3.4 m | the Ridgeback's question again: a stone, a platform or a hop |

Its speeds against a walk of 7 m/s: it cruises at 16 m/s, which is a lap of its
fourteen-metre circle every five and a half seconds; it passes low at 18; it
stoops at **30**, faster than a dodge at 17. On the ground it walks at 5 and
hops. Nothing about it is outrun. What you control is **where you are standing
when it decides to come down**, and what you do in the second after it does.

The loop:

1. **Aloft.** It circles out of reach and throws things down at you: a lane of
   blade feathers, a hover that blows everyone outward, a dive. You are not
   fighting it yet. You are choosing where to stand so that the moves it has to
   come low for are the ones it picks.
2. **The openings.** Two moves bring it within reach. The **Stoop** puts it on
   the floor for a second with its wings spread. The **Talon pass** skims it
   along a lane at head height with its wings out over the people beside the
   lane. Hits on the wings while it is low fill its **wing poise**, and every hit
   on a wing fills that wing's **break bar**.
3. **Crash.** Poise full, it tumbles and lands on its breast with both wings flat
   on the floor. The wings are ramps. Four seconds of the fight's big window,
   open to every class, the Bulwark included.
4. **The choice.** At the end of the crash it gathers itself and takes off. Get
   off during the gather and you are back at step 1. Stay on and you go up.
5. **Sky ride.** A climb to twenty metres on a back that heaves with every
   wingbeat, a circuit of the arena, and a barrel roll meant to throw you. The
   wing roots are up here, at double damage and double break. Being thrown at
   twenty metres costs **275** — over a quarter of a fighter. Once a lap it swoops to
   three metres over the far side of the arena, and that is the moment to step
   off.
6. **Grounding it for good.** Break one wing and it cannot climb above twelve
   metres and turns badly toward that side. Break both and it never flies again:
   the rest of the fight is on the ground, against a bird that runs, hops and
   screams. That is its two-broken-forefeet: the whole approach bought once,
   for the rest of the fight.

The **perch** runs beside this loop. Flying costs it wind, and when the wind
runs low it flies to the tower, lands on the top and rests. That is the quiet
window: nothing comes at you, and the tower is climbable. Whoever is on its back
when it leaves the perch is on a sky ride that begins twelve metres up.

## 2 · The moves

Nine moves and the carry that follows one of them, and ten answers, no two the same. First guesses throughout; the frames are
startup / active / recovery at sixty a second.

| Move | Range | What it is | Tell (frames) | The answer |
| --- | --- | --- | --- | --- |
| **Stoop** | long, from altitude | Wings fold, it drops at 30 m/s onto a point and hits the floor there. 220 damage, knockdown | 45 / 6 / 60 on the ground + 30 to lift. The floor marker is a circle 6 m across whose fill grows over the 45 | **Dodge at the hit.** It tracks for 30 frames and locks for the last 15; from the middle of the circle a walk in those 15 frames is nearly two metres short of out. Then **walk up and swing**: 60 frames with its wings on the floor |
| **Talon pass** | mid | It drops out of its circle, lines up along a lane and skims it with talons forward. Catches the first fighter standing in the lane and carries them up | 70 / 80 / 60 — the front crosses 24 m at 18 m/s. The lane (24 m × 3 m) is marked from the first frame, with a front that travels along it at the bird's speed; the talons swing forward 20 frames before they reach you | **Crouch.** Talons clear 1.2 m, a crouched fighter is 1.0 m tall and a standing one 1.8. Beside the lane, the wing passes over you at 3 m: **that** is the swing |
| **Carry** | — | The catch: 90 frames climbing to 12 m, then it lets go. 40 on the grab, 75 for the fall | — | **Hit the legs.** The carried fighter can swing at them; three hits, or 150 damage from anybody, drops them at whatever height it has reached |
| **Downwash** | mid, and the tower | It stops in the air eight metres up and beats down. Everyone within 14 m is pushed straight outward from the point under it at 8 m/s for two seconds. No damage: the damage is the edge, the fall, and the volley that follows | 50 / 120 / 40. Wings stand vertical and it hangs still — the loudest silhouette it has. A ring 28 m across on the floor, with the still eye in its middle | **Get into its lee**: behind a solid (a stone, the tower, a planted shield) the push is zero, and in the eye — 2.5 m under it — the air goes straight down. Crouching only slows the slide to a quarter |
| **Feather volley** | long | Rolled onto its side, it rakes a curtain of blade feathers down a lane from above. 60 a feather, three at most | 36 / 30 / 40. Its upper wing lifts and the feathers stand up along it with a rattle; the lane (22 m × 3 m) is marked through the startup | **Leave the lane sideways.** Each point of the lane is under feathers for 20 frames, longer than a dodge's ten invulnerable ones, so a dodge *through* it fails and a dodge *out* of it works |
| **Screech** | mid, on the ground | Head back, throat swollen, then a cone 70° wide and 11 m long. 40 damage and 80 frames stunned | 32 / 12 / 36. The crest rises and the throat fills; the cone is drawn | **Get out of the cone** — beside it or behind it. The cone is only thrown on the ground: after a Stoop, at the end of a crash, on the perch, and for the rest of the fight once it is grounded |
| **Wing buffet** | close, on the ground | The wing on your side comes up high and back, then sweeps the floor out to 6 m, 0–1.4 m high. 150 damage, knockdown | 22 / 5 / 34. The wing lifts; mirrored to the side you are on, the way the Ridgeback's sweep is | **Jump it** |
| **Wingbeats** | aboard | Climbing, every beat's downstroke heaves the back. No damage: grip | 20-frame upstroke, 8-frame downstroke, every 36 frames | **Brace** through the downstroke |
| **Barrel roll** | aboard | A full roll along its spine. Thrown from altitude, you pay for the height | 40 / 30 / 60. The head dips, one wing tucks, and the horizon on your screen starts to tip at the edges (the camera does not roll; see §6) | **Be on the spine, braced.** The roll throws everybody off the wing roots and nobody braced off the spine |
| **Perch** | the tower | It flies to the tower top and rests | 180 frames of announcement, then 300 of rest | **Climb.** Nothing it does can be answered by the perch; it is the window the tower is for |

**The carry is the one hit that goes on after it has landed**, and it needs the
rule monsters.md gives for the buck: it is paid for, and its answer is an action
rather than an endurance. A carried fighter keeps their swings. Held in talons
the dead zone of a swing does not apply (they are off the floor, see
[aiming.md](../aiming.md) §Swing), so looking up puts the swing into the leg. A
teammate's shield, bolt or spear does the same from below. Freed at four metres
you land for nothing; at ten, for 75.

**The Stoop's lock is the dodge's timing.** It follows you through the first 30
frames of its drop at nine tenths of its free turn, like the Ridgeback's windup,
and locks for the last 15 — the length of a human reaction. A dodge started as
the fill reaches the edge of the circle lands its ten invulnerable frames across
the six active ones. Walking out works only from the edge of the circle, which
is what `the_stoop_is_dodged_not_walked_out_of` pins, on the Ridgeback's model.

**Crouch is new as a creature answer, not as a mechanic.** Fighters' moves
already declare `hits_crouching`, and the Champion's spear is the one that
passes over a crouch. The talons are the first creature move that does. The
margin is twenty centimetres, from 1.0 m to 1.2 m, and it wants to be exactly
that small: a talon that passed over somebody standing half-crouched would make
the crouch a suggestion.

## 3 · A threat at every range

**Close, and only on the ground.** The Wing buffet and the Screech, which cover
the two halves of standing next to it: the buffet the patch beside it, and the
Screech the cone in front. The 60 frames after a Stoop are a walk-up window with
the buffet as its price; the scripted hunter learns to swing twice and jump.

**Mid.** The Talon pass and the Downwash. Both are chosen when a target is 8–20 m
from the point under it, and both are why standing in the open middle of the
arena is not safe.

**Long.** The Stoop and the Feather volley. They are thrown from the top of the
circle at whoever is furthest from it, so the far corner is where it dives.

**Aboard.** The wingbeats, the roll and the height. Nothing it throws can reach
its own back — it holds for this creature exactly as it does for the Ridgeback —
so a rider is threatened by the ride and by nothing else.

**On the tower.** The tower top is twelve metres up and inside Air bolt range of
most of the circle, so it is the obvious place to fight from. It is covered by
the Downwash, which it hovers beside the tower to throw, pushing along the top:
six metres of tower is under a second of slide for anybody standing still, and
a crouched fighter holds only while walking into it, which is not swinging. Blown off the top is a twelve-metre fall: 75.

**The tempting safe spot** is **the foot of the tower**, on the side away from
it. The tower is a solid, so it shades you from the Downwash, and a dive has to
come down past it. It is safe for as long as the circle keeps the bird on the
other side, which is two and a half seconds. The volley is aimed along the
tower's flank when a target stands there (the lane runs down the wall, not into
it), and a bird that has come round the tower dives into the patch it could not
reach before. **The shelf below the cliff** is the other: twelve metres down, out
of every close move's reach — and the volley's lanes run along it, three metres
wide on a ledge four metres wide.

## 4 · The approach and the window

**What you break.** Each wing has a break bar of 1400, filled by every hit on
that wing from anywhere, aloft included. Hits on the **wing roots** — the two
places on the back, just outboard of the spine, where the wings meet the body —
count double toward the bar and deal double damage.

- **One wing broken:** it cannot climb above 12 m, climbs at half the rate, and
  lists toward that side. Its turn toward the broken side is at half rate, so
  its circle becomes an egg and there is a side of the arena it comes to slowly.
  It passes low more often because a low pass is cheaper than climbing. The
  broken wing trails, drawn ragged, and whistles.
- **Both broken:** it lands and never leaves the ground again. It walks at 5 m/s,
  hops 6 m forward at 12 m/s as its gap-closer (the Stoop's marker, the Stoop's
  answer), and keeps the Screech and the buffet. Its back is 3.4 m up: the
  Ridgeback's climb question, asked of the same six classes, and answered by the
  same routes — a stone, a platform, the Dual mage.

**What you earn.** Wing poise, 900, fills from wing hits taken **while its body
is below 6 m** — during a Stoop's recovery or a Talon pass. Up high a bird has
room to recover, so hits aloft break wings but never crash it. Poise
regenerates while it is circling, so a crash is earned in one or two low passes
rather than banked over a fight.

**The big window is the crash:** 240 frames on the floor, wings flat, the back
at 2.4 m and every surface on it walkable. The wing roots are within a hop of
the floor for the four taller classes and a walk up the wing for all six.
Damage taken during a crash is ×1.3 on everything. Then a 45-frame gather — it
pulls its wings in and rears — and it lifts.

**The choice the fight is named for** is the gather. Jump off and you are back
on the floor for nothing. Stay and you are on a sky ride: twenty metres up, a
lap of wingbeats, the wing roots under your feet and a roll that will throw you
if you are on them when it comes. The ride is where a wing gets broken fastest,
and where a fight goes wrong fastest.

**Breaking a wing with somebody aboard** is the ride's jackpot. The bird drops,
and whoever is on it rides the crash down, taking a third of the fall's damage
(`crash_ride_share`) and landing on a crashed bird. A wing broken at twenty
metres is a 90-damage landing on top of the big window.

**The arc**, fresh to desperate. Strain works as it does on the Ridgeback, on the
same scale, with the thresholds falling as health does (`strain_desperation`).
Two things are new aloft. Control that lands while it is **susceptible** clips
it: a root, a slow or a grab forces it out of the circle into a low glide for 90
frames — the Talon pass's path without the talons, a free low pass. A **spike**
while susceptible (the Elementalist's Downdraft, the Champion's Air hammer) with
its body below 6 m crashes it outright. Below 40% health it flies lower (cruise
at 13 rather than 16), rests less (the wind regenerates faster), and scores the
Stoop and the Talon pass higher: it commits harder, which means it comes down
more. Below 15% it will Stoop twice in a row, the second from a lower circle.

## 5 · The brain

The Ridgeback's control algorithm with two changes of kind: **it chooses from a
circle**, and **it keeps a budget**.

**It lines up.** A flyer cannot turn onto you from anywhere. While aloft it only
reaches a decision point when the target lies within `line_up_arc` (60°) of its
heading along the circle. So the moment it can attack you is set by where you
stand against its circle, and a player can see it coming round. Everything
still goes through the glance (every `glance` frames, position and velocity,
projected by `lead`); the Stoop has its own `stoop_lead`, because it is thrown
from fifty metres of flight path away.

**The approach is the startup.** A move from the air commits on its first
startup frame and spends the startup flying the approach: the Talon pass's 70
frames are the drop out of the circle and the line-up. During that time it
tracks the target as the Ridgeback's windup does (`startup_tracking`), but
through the flight controller, so its turn is bounded by how hard it can bank
(`bank_max`, 50°). A fighter who moves across the lane early makes it bank; one
who moves late is in the lane. The hit locks, as everything's does.

**It keeps a budget: wind.** 100 at full. Circling costs nothing. The Stoop
costs 25, the Talon pass 20, the Downwash 30, the volley 10; it regains
`wind_regen` a second while circling. Below `perch_wind` (20) it announces the
perch — breaks the circle, calls, and flies to the tower — and rests for
`perch_rest` (300 frames), which refills it. **A hit on the perched bird takes
frames off the rest** (`perch_rest_per_damage`, one frame per four damage):
the perch is a way aboard, not a free damage window, and a team that unloads on
it has chosen a shorter rest.

```text
U(m) =  range(m)        how well the distance from the point under it suits the move
      + arc(m)          whether the target is inside the line-up arc of its heading
      + altitude(m)     a tent on how far it must climb or sink to throw it
      + lee(m)          Downwash: how exposed the targets are — near an edge, on
                        the tower, not behind a solid; nothing if all are sheltered
      + follow(m)       Feather volley after a Downwash; Wing buffet after a Screech
      + rider(m)        enormous for the roll when somebody is aboard
      + wind(m)         a move it cannot afford scores nothing; low wind favours cheap ones
      + variety(m)      the decaying penalty on the move it just used
      + hurt(m)         wounded birds come down more
      ⟂ place(m)        air moves score nothing grounded; ground moves nothing aloft
      ⟂ cooldown(m)     a move on its lockout scores nothing at all
```

`decisiveness` at 0.7 for a solo hunt. The Talon pass locks out for 360 frames
after a catch, because the bird has to drop what it is carrying before it can
catch again. The Downwash locks out for 480, because a hover every lap would
make the edge of the arena a wall.

**Two set-ups, both pinned by the Ridgeback's arithmetic test**
(`a_set_up_lasts_long_enough_for_what_follows_it`): the Screech stuns for 80,
which outlasts its recovery and a buffet's startup; the Downwash's push ends
with you somewhere it chose, and the volley's startup is short enough to arrive
before you have walked back.

**The flight controller** is the part that is new (P6). Position and velocity in
three dimensions, yaw, pitch and bank; it steers toward a target point under
three limits: `turn_rate_max` scaled by bank, `climb_rate` (5 m/s) and
`sink_rate` (12 m/s, 30 in a stoop). The circle is a target point that walks
round `circle_radius` (14 m) at `cruise_altitude`. Like the Ridgeback's yaw
controller, the acceleration limit is the bird's mass: it overshoots a hard
turn, which is the window a player cutting across its lane gets.

**With a rider** it does nothing else. It climbs (`rider_climb`, to 20 m), flies
one lap, rolls, swoops to 3 m over the far side of the arena for 50 frames, and
climbs again — four metres higher each lap. The ground below gets no attacks
while it carries somebody, which is the coop half of the ride (§8).

## 6 · Reading it

**The sun is overhead in the Cliffs, on purpose.** The bird's shadow falls
straight down, hard-edged and eighteen metres wide, and it is the most important
thing drawn in the fight: a player looking level cannot see a bird sixteen
metres up (the framing field of view is 57°, so level puts the top of the
screen about 28° up), but they can always see the floor. The shadow says where
it is; the markers say what it is doing.

**Every move is drawn on the floor**, from the hit test's own answer
(`Monster::telegraph` posing the bird on its first active frame, as for the
Ridgeback): the Stoop's circle with its growing fill; the Talon lane with a
front travelling along it at the bird's speed; the Downwash's ring and its eye,
and a darker patch behind every solid that is the lee — computed by the same
query that zeroes the push; the volley's lane; the Screech's cone. The lee
patch is the one new thing to draw, and it has to be the push's own answer or
it is an overlay that lies.

**The silhouettes differ from below**, and each changes on the first frame of
its startup: an eighteen-metre cross circling; an eight-metre dart (the Stoop);
a flat line at head height (the pass); wings vertical over a still body (the
Downwash); the bird on its side (the volley). **Sound carries what the screen
does not**: a different cry on the decision frame of every air move, panned to
where it is, and a rising whistle through the Stoop.

**Unanswerable, for this creature,** gains a second clause. Monsters.md counts a
hit whose tell is under reaction with no positional warning. Here a tell can be
long and still unseen: the bird is overhead. So the report counts a hit as
unanswerable if its tell was under fifteen frames with no warning, **or if its
floor marker was never inside the victim's view for fifteen frames** before the
hit. That second count must be zero, and it is the measure of whether the
shadow, the markers and the cries are doing their job.

### The camera, looking up

Four things the follow camera has to do.

- **A creature overhead must not lift the camera.** `view::camera` settles the
  drawn eye onto whatever is under it, and it asks `Monster::top_under`, which
  returns the highest solid top over that spot **at any height**. A bird passing
  over the eye would put the camera on its back. The floor under the eye is the
  highest surface below the fighter's own feet, not the highest there is.
  `a_creature_overhead_does_not_lift_the_camera`.
- **The drawn eye and the aiming eye must stay one point when looking up.**
  This fight is spent looking up, so it leans on that harder than any other.
  ⚠️ *Checked 2026-09-30, and a first draft of this section was wrong.* It
  claimed the simulation's eye sinks about 1.7 m below the floor at 40° up
  while the drawn camera rides the floor, which would be two parallel rays.
  Measured with `sim::camera::eye` on the floor, the aiming eye sits 1.77 m up
  at 10°, 1.67 m at 40° and 1.60 m at 80°: above the floor at every upward
  pitch, so the view's floor clamp never fires against it and the two eyes
  agree. No change is needed. The test stays, as a guard for this fight:
  `the_drawn_eye_is_the_aiming_eye_looking_up`, on the floor and aboard.
- **The camera never follows the bird.** A camera that turned itself toward the
  target would turn the aim with it, and that is the creature reading the mouse
  from the other side. The shadow and the cries are the answer to "where is it".
- **Aboard, the camera carries yaw and nothing else.** `carry_yaw` already turns
  the view with the part underfoot; the bird's bank and pitch are *not* carried.
  A horizon that rolled with a barrel roll would roll the crosshair's meaning
  with it and make half the players ill. The rider's world stays level; the bird
  tips under them.

### Aiming at a flyer

The model holds, and one function in `aim.rs` changes.

**Skillshots hit it.** Aimed at a bird in the sky, the camera's ray meets nothing
but the ability's range sphere, and `skillshot_path` goes from the caster to the
sphere's far crossing — a line that converges with the crosshair's at that
point. Bodies are not on the ray, so the bird does not stop it; the path's own
`first_along` meets the bird on the way. Looking up, the eye sits almost on the
line through the fighter's head (the handover's tilt is small), so short of
where they meet the two lines are under a metre apart, against a bird eight
metres long. What stops the shot is the
honest thing: range. From the floor the Bolt (9 m), Cataclysm (12), the
thrown shield (9), Bloodletter (7), Grasp (12) and the lances (7) reach it on a
pass or a hover and not in its circle; the Elementalist's Air bolt (22) and
Gale (32) reach it in its circle, from the air. The reticle should say so: a
shot whose ray ended on the sphere draws the reticle hollow. That is a view
change read off `Path`, not an aim change.

**Grounded casts cannot reach it, and should not.** Aimed at the sky, a grounded
cast goes to max range on the floor in the mouse's direction. A fire pillar under
the Talon lane is the one grounded cast with a use here, and it is placed by
aiming at the lane, which is on the floor.

**Swings on a banked back are the change.** `aim::swing_path` levels a standing
swing through the first 45° below the horizon, and "the horizon" is the world's.
On a back banked 40° that level runs into one wing and over the other: a rider
looking at the wing root at their own feet is looking, in world terms, well down
one side and up the other. The proposal is that `swing_path` (and `origin`) take
the **up of the surface underfoot** — `+y` on the floor, which leaves every
existing swing bit-identical — and measure the dead zone against the plane that
surface defines. It is one change in `aim.rs`, true of every swing at once, and
the Ridgeback's shake, fifty degrees over at its peak, gets it for free.
`a_swing_on_a_banked_back_is_level_with_the_back`.

## 7 · The classes

**Dual mage — her fight.** Her 6.0 m hop does nothing to a bird at 16 m, but both
bars full is six seconds of wings, a beat on every press of space: she can climb
to it and land on it, and mounting is landing, so she boards in the air as
nobody else can. The wings also make the ride survivable — a beat is an upward
impulse and resets the height a fall is measured from, and her slow fall lands
soft for half damage. The cost is the burn wings are paid for and a back reached
alone, with nobody below to hit the legs. **Identity**: the one creature that
makes not touching the floor the fight rather than a shortcut through it. Whether
it makes the crash pointless for her is §12.

**Elementalist — the one who reaches it up there.** Air bolt (22 m) and Gale
(32 m) are the only things in the game that reach the circle, and they are
airborne: a stone and an Updraft put her four metres up with the bird in range.
Hits aloft break wings and never crash it, so she is the wing-breaker. Her
**Downdraft** is her crash: Updraft over the Talon lane and come down on the
bird — a spike on a susceptible bird below 6 m grounds it. A fire pillar in the
lane burns the talons as they pass. **Identity**, and the Ridgeback's "the
Elementalist cannot win" answered here. The ride is hardest for her: her stones
stay on the floor.

**Champion — the wing-striker.** The pass and the Stoop's recovery are his: the
spear's 3.4 m, pitched up with the camera, hits a wing three metres up, and Rush
cancels a recovery into the walk-up. Aboard he out-damages everyone on the wing
roots. Aloft he has nothing and stands where the lane will be. **Hard in the
waiting, easy in the windows** — identity if the openings come often enough,
which is what the out-of-reach share measures.

**Bulwark — the lowest jump and the only portable wall.** The crash's wing ramps
are what make the big window hers. Her **planted wall is a lee**, so the move
that shoves everyone off the edge is answered for the team by where she stands.
Her **thrown shield** (9 m) hits a wing on a pass or the legs of a bird carrying
somebody. The creature's blows load her weight, so a blocked Stoop pays for a
Slam into its recovery; the talons are a grab and unblockable. **Hard, and the
wall is the identity.**

**Shadow Reaver — the tower is hers.** Send shadow reaches 9 m, so from the floor
it makes the gallery at six metres but not the top at twelve; two dashes beat any
climb. A shadow on the tower copies her swings at the perched bird and at a pass
nearby, and Guillotine lotus erupts there. It is also **her parachute**: from a
ride, a dash to a shadow on the floor ends standing — until the climb passes the
18 m leash and pulls the shadow up after her. At 750 health a thrown ride is half
her life. **Identity**: the most to lose aboard, and the best way off.

**Blood mage — her own way up.** Grasp (12 m) meets the bird as an anchor and
hauls her to it; aimed at the back of a low pass the haul lands her on top, which
is mounting, and aimed at the belly it drops her from wherever she met it. Her
cuts bleed onto the floor whatever the bird was over, so a crash leaves pools, and
a 9 m blink to one is a way off the swoop from further out than a jump carries.
Aloft she has Bloodletter's seven metres at a hover and nothing else. **Middle.**

## 8 · Coop

- **The carry has a partner's answer.** 150 damage to the legs drops a carried
  teammate; solo it is three of your own swings from inside the talons. Two
  hunters is the design's intended shape of this move.
- **One baits, one strikes.** A hunter crouched in the lane draws the pass while
  the other stands beside it for the wing. The bird's targeting is the
  Ridgeback's (`target_switch`), and the lane goes where the target is, so
  choosing who is the target is a thing two players can do on purpose.
- **The ride splits the team.** A bird with a rider attacks nobody on the ground.
  The one below has a lap of free shots at a bird that is climbing — out of
  reach for most classes — and a job: be under the swoop, because the rider will
  be coming down there.
- **The lee is shared.** A planted wall, a stone or the tower shelters whoever is
  behind it.
- **Numbers.** Health ×1.6 (14400). `glance` shorter by a fifth. Wind costs
  unchanged, so the perch comes as often and is the team's regroup. The Talon
  pass lockout drops to 240 frames, because with two targets a catch is less
  than half of the pressure.

## 9 · Measuring it

**The scripted hunter's plan** (P8), which is what a person learns in their first
ten minutes:

1. Stand in the open, more than 16 m from any edge, and never on the tower top.
2. **Stoop:** dodge as the fill reaches the circle's edge, then swing twice at
   the nearest wing and jump. (The buffet is the thing it learns to jump.)
3. **Talon lane:** in it, crouch as the front arrives. Within four metres of it,
   step out beside it and swing up at the wing.
4. **Downwash:** walk to the nearest lee — a stone or the tower — or to the eye
   if it is nearer; crouch if neither is reachable in 50 frames.
5. **Volley:** step out of the lane sideways.
6. **Screech:** circle to the side.
7. **Crash:** walk up a wing, hit a wing root; **plan A** jumps off in the
   gather. **Plan B** rides: stands on the spine, braces on every downstroke,
   steps off at the swoop.
8. **Perch:** climb the tower if the announcement began within 4 m of the stairs.

The harness runs both plans across twelve seeds, and the difference between them
is the measure of whether the ride pays.

**Report lines it adds:**

| Measure | Why |
| --- | --- |
| **Out of reach** share | Frames nothing this class can do from where it stands touches the bird. The fight is designed around it; too much and it is a waiting room |
| **Low passes per minute**, **wing hits per pass** | Whether the openings are openings |
| **Wing damage**, per wing, and **worst wing** | The foot line's equivalent: `wings broken: 0` has two causes, and this tells them apart |
| **Crashes**, and what earned each | poise from a pass, from a Stoop, or a clipped spike |
| **Rides**: count, mean length, and **how each ended** | stepped off at the swoop · thrown · rode a broken wing down · dashed or blinked off |
| **Fall damage**, by cause | talon drop · thrown · blown off the tower · blown off the edge |
| **Carries**, and **freed early** | whether the leg answer is ever used |
| **Unseen tells** | hits whose marker was never on the victim's screen for 15 frames — must be zero |

**Targets.** Tier 4: the scripted hunter wins **about one in six**, the wins in
**5–10 minutes**. Plan B should win more often than plan A and lose faster:
the ride is a wager. Out of reach: **at most a third** of the fight, and falling
as the fight goes on. Of the frames it is in reach, the Ridgeback's four
windows: threatening about four tenths, walk-up about a sixth (less than the
Ridgeback's fifth, since the walk-ups here are the Stoop and the crash and
nothing else), the rest poke and way in. Unanswerable hits and unseen tells:
**zero**.

## 10 · What it needs built

**Depends on:** P1 (species), P2 (the Cliffs arena), **P6** (flight, and the long
ride), P8 (its plan and report lines). Not P3, P4, P5 or P7.

**New and specific to it:**

- **Fall damage from real heights.** Gravity is 42 m/s² and terminal velocity
  18, which is reached after 3.9 m — so every fall above four metres lands at the
  same speed, and damage keyed to landing speed cannot tell the shelf from the
  sky. It is keyed to **height fallen** instead: `fell_from`, the highest point
  since the fighter last stood on something or was last pushed up by an impulse
  (a jump, a wing beat, an Updraft), in the snapshot per fighter. Free below
  `fall_free_height` (9 m, as built in F3b), then `fall_damage_per_metre`
  (25): 75 off the plateau onto the shelf, 75 from the tower top or a talon
  drop, 275 from a ride at 20 m and 375 from its second lap at 24. Halved
  for a landing under `soft_landing_speed` (12 m/s), which only the Dual
  mage's slow fall achieves. It applies in every fight, including the
  Ridgeback's back.
- **The wing parts**: two breakable, mountable parts per wing (root and blade)
  whose flags come from the species table, and the crash pose in which the
  blades become ramps.
- **The carry**: a fighter held at a bone, their swings live, released by
  damage to the legs or by the timer.
- **The Downwash field**: a push outward from a point, zeroed for a fighter when a
  line from the point under the bird to them crosses a solid. That is a
  ray-against-shape question about where something goes, so it is asked through
  `aim::clear_between` rather than beside the move.
- **Wind** and the perch as a scheduled behaviour.
- **The aim change** (`swing_path` and `origin` take the surface's up) and **the
  camera changes** (`top_under` below the feet).

**Snapshot.** About 260 B, as the bestiary estimates: the Ridgeback's ~200 for
the monster, plus 3D velocity (12), pitch and bank (4), the circle's phase and
altitude (4), wind (2), two break bars and poise (6), the carried fighter and
timer (2), the ride cycle's lap and phase (2), and `fell_from` per fighter
(4 each; this one is not the Galewing's, it is everyone's).

**Per-frame cost.** Dominated by the Downwash: one `clear_between` per fighter
per frame for 120 frames, against the arena's solids and stones — a few dozen
segment tests a frame, times eight on a rollback. The flight controller is a
handful of multiplies. The grip test is the Ridgeback's, on fewer parts. The
wings are the widest parts in the cast, so `first_along` against them is the
thing to watch, not the thing that dominates.

**Tests** that pin the rules:
`nothing_on_the_circling_galewing_is_inside_any_standing_jump`,
`the_stoop_is_dodged_not_walked_out_of`,
`a_crouched_fighter_passes_under_the_talons_and_a_standing_one_does_not`,
`hitting_the_legs_drops_the_carried_fighter`,
`the_downwash_does_not_move_a_fighter_behind_a_solid`,
`the_eye_of_the_downwash_is_still`,
`the_feather_rake_outlasts_a_dodge_at_every_point_of_its_lane`,
`wing_hits_aloft_break_wings_and_never_crash_it`,
`both_wings_broken_it_never_leaves_the_ground_again`,
`the_roll_throws_riders_off_the_wing_roots_and_not_braced_riders_off_the_spine`,
`the_swoop_is_low_enough_to_step_off_for_free`,
`fall_damage_reads_the_height_fallen_not_the_speed_landed_at`,
`a_fall_from_a_full_hop_off_a_platform_costs_nothing`,
`a_hit_on_the_perched_galewing_shortens_its_rest`,
`the_perch_is_announced_long_enough_to_climb_the_tower`,
`a_skillshot_aimed_at_the_circling_galewing_hits_it_inside_its_reach`,
`a_swing_on_a_banked_back_is_level_with_the_back`,
`a_creature_overhead_does_not_lift_the_camera`,
`the_drawn_eye_is_the_aiming_eye_looking_up`, and
`what_is_drawn_through_the_windup_is_where_the_hit_lands`, extended to the lee.

**Milestones**, each ending in something checkable:

- **M1 · Fall damage.** `fell_from`, the free height, the halving; the two fall
  tests pass, and `beastcheck` prints the damage for the heights in §1.
- **M2 · The Cliffs.** The arena as P2 data: plateau, shelf, stairs, tower,
  stones. `beastcheck` prints the tower's ledges against every class's hop.
- **M3 · Flight.** The flight controller and the species rig: it circles, stoops
  to a point, passes along a lane, hovers, perches and crashes on command, with
  no hits. `SHOT_MOVE` captures each pose.
- **M4 · The moves.** Hit volumes, floor markers (the lee included), the carry,
  the Downwash field, the brain's scoring. The move tests pass and the report's
  coverage line shows every move used.
- **M5 · Wings.** Break bars, poise, the crash and its ramps, one wing and two.
  The wing tests pass; a scripted crash is walkable by the Bulwark.
- **M6 · The sky ride.** Take-off with riders, the ride cycle, the roll, the
  swoop, riding a broken wing down. The ride tests pass.
- **M7 · Aim and camera.** The `swing_path` change and `top_under`.
  `one_aim.rs`, `determinism.rs` and the Ridgeback's twelve seeds unchanged
  except where the camera moves at steep upward pitch, which is written down.
- **M8 · The hunt.** Plans A and B, the report lines, one tuning pass toward
  tier 4. The report's numbers go into this document, as §9 of monsters.md
  records its own.

## 11 · In the world

**The Cliffs**, on the edge of the world map, reached from the Saddle. The
arena is a plateau **44 × 44 m** of bare rock and short grass. Three sides drop
to **the shelf**, a ledge four metres wide twelve metres below, with stairs back up
at each end of each side — six steps of two metres, a hop each. The fourth side is a rock
face. **The tower** stands off-centre, ten metres from the rock face: a ruined
pillar six metres square, with a gallery at six metres and a flat top at twelve,
reached by a spiral of ledges each 1.5 m above the last — a hop for everybody —
eight of them, about six seconds of climbing. Four standing stones, two to two
and a half metres tall, are scattered across the plateau: the lee. The sun is
straight overhead. Seen from the trail, it is circling the tower.

The 28 m arena is too small for it: an eighteen-metre wingspan on a fourteen-
metre circle needs the room, and the edges are the Downwash's teeth.

**Trophy:** a pinion — the long outer flight feather, hung in Hearth.

**Sidegrade family: the air** — hang time and aerial control. The one proposed
here is a **common modifier, for every class: Pinion.** At the top of a held
jump you hang for twelve extra frames at half gravity; your full hop is 0.4 m
lower. It changes when you are in the air rather than how high, which is a
playstyle — the aerial poke lands later and the landing is easier to read — and
for the Bulwark, whose hop is lowest, it is a real question whether the height
is worth more than the hang. Neither side of it beats the next creature.

## 12 · Open questions

1. **Does the Dual mage's boarding in the air make the crash pointless for her?**
   Six seconds of wings reaches the bird at will. The cost is the burn and a ride
   taken alone. If she never grounds it, the fix is not to take her wings away
   but to make the ride from the air start higher (it climbs from wherever she
   boarded). That is a decision, and it is the Ridgeback's tail question again.
2. **Is a third of the fight out of reach too much?** It is the premise, and it
   is also time in which the Champion and the Bulwark stand still. The levers,
   cheapest first: `wind_regen` (a lower one perches it sooner), the aloft
   `think_frames`, and the cruise altitude.
3. **Crouching under the talons with twenty centimetres to spare** — does it
   read as a skill or as a hitbox trick? Only a person crouching under it knows.
4. **Does the camera's horizon stay level through the roll**, and does that feel
   like riding a bird or like watching one roll under a fixed camera? Rolling the
   camera would roll the aim, so the alternative is not free.
5. **Should the Bulwark's guard cover a volley from above?** Her block covers a
   facing arc; feathers coming down at sixty degrees are in front of her and
   above. Yes makes her the volley's answer as well as the Downwash's; no keeps
   one answer per move.
6. **Is 275 for a throw at twenty metres the right price?** It has to be enough
   that stepping off at the swoop is a decision, and not so much that nobody
   rides. The knob is `fall_damage_per_metre`, and it is shared with every fall
   in the game — fall damage is a global rule (the Ridgeback's 5.5 m back and
   every class's own jumps stay free), and whether it should be global rather
   than the Galewing's is a decision.
7. **Gravity and the grip test.** The ride keeps the Ridgeback's rule — the
   surface's acceleration, with the rider held in the part's frame so a bank
   does not slide anyone. Upside down in the roll, gravity pulls a rider off at
   42 m/s², under a tenth of grip, so leaving it out changes little; putting it
   in would make a steep bank slide you along the wing. Which of those feels like
   a bird is a question for a person on one.

## 13 · Where it landed

Built 2026-10-01: `--hunt galewing`, the Cliffs. The species is
`sim/src/species/galewing/` (the table; `fight.rs` for the lore, the moves'
own hits, the wings, the crash, the carry, the perch, the ride and the signs;
`flight.rs` for the flight controller, the circle, every approach, the beat
and the bank; `mind.rs` for its scoring), the arena `sim/src/arena/galewing.rs`,
the clips `anim/src/beast/galewing/`, the look (the wing roots pale, a cracked
wing rust, a broken one dark: a `Tint`) and the Cliffs' dressing with the sun
overhead in `game`, the plan and its lines `hunt/src/plans/galewing.rs`, and
the rules pinned as sentences in `sim/tests/galewing.rs` and
`view/tests/galewing.rs`. The plan is [plans/galewing.md](../plans/galewing.md);
the passes are in [feel-log.md](../feel-log.md) of 2026-10-01.

Numbers from `cargo run -p hunt --bin fight -- --species galewing --class <c>
--repeats 24` (plan A) and `... --gamble` (plan B, the rider); out of reach
from twelve single runs each, which print the report's lines:

```text
                plan A won   mean     plan B won   mean     out of reach (A / B)   unanswerable, unseen tells
  Champion        4/24      482 s       7/24      491 s          67 / 65 %                  0
  Bulwark         0/24        --        2/24      700 s          55 / 53 %                  0
  Reaver          9/24      646 s      11/24      580 s          55 / 53 %                  0
  Elementalist   17/24      176 s      18/24      230 s           5 /  4 %                  0
  Blood mage      0/24        --        0/24        --           51 / 49 %                  0
  Dual mage       0/24        --        0/24        --           56 / 55 %                  0

  (2026-10-01, every class played; before it the Reaver won 0 and 2, the
   Elementalist 0 and 0)

  windows, every class: threatening 79-80 %, poke 6-7, way in 9, walk up 5
  coop, two Champions: 0/12
```

**2026-10-01: the hunter plays all six classes** (`hunt::class`,
[bestiary.md](../bestiary.md) §8). **The Elementalist** wins 17 and 18: when
the bird is down -- crashed, dwelling after a Stoop -- a pillar goes under it
(286 in plan A's 24) and her bolts at its wings are aimed. **The Reaver** wins 9
and 11 from 0 and 2: her shadow goes to the downed bird's wing (845), a lotus
opens on it and is dragged home through it (799), and she cashes the marks (233)
-- a bird on the floor is a target that stays beside her shadow. **The Blood
mage and the Dual mage still win nothing**: out of reach half the fight, and
the windows when it is not are too few for the Dual mage's bars (goaded up 2671
times at home, back down by the next window) or for pools worth a spike.

**Against the targets.** The Champion is at tier 4: plan A wins a sixth in
eight minutes, and plan B -- the ride -- wins more often (7 against 4) and
loses its rider to throws and the roll. **Zero unanswerable hits and zero
unseen tells** for every class and both plans. Short of the targets:

- **Out of reach is two thirds for the melee classes, not a third** (§12's
  second question, answered "too much" by the numbers). It is counted from
  where the fighter stands -- a grounded bird across the plateau is out of
  reach too -- and the levers §12 names moved it little: a slower wind
  (1.0 a second) perches it more and took ten points off, and lost the
  Champion half his wins; a longer dwell on the floor after a Stoop did the
  same. Left at the first guesses for a person to decide.
- **The windows are four fifths threatening**: the bird is threatening while
  it circles, and it circles most of the fight.
- **The carry and the barrel roll are rarely or never seen**: the scripted
  hunter crouches under every pass, and plan B's rides end in the first lap.
- **The rest of the roster lost** to the scripted hunter until 2026-10-01,
  when it played the Champion's fight with every class. With each class's own
  (above), the Elementalist and the Reaver win; the Blood mage, the Dual mage
  (whose wings the hunt never reaches: ascension is not played) and the
  Bulwark do not. The Elementalist's air plan is the plan's, as before.
- **Coop loses** (0 of 12, against 14 400 health): the second Champion runs
  the same plan beside the first, which the plan was not written for.

**What changed from the sections above.**

- **Falls are the built rule (F3b)**: free to 9 m, 25 a metre. The edge and
  the tower top cost 75, a talon drop 75 (with 40 for the grab), a throw at
  20 m 275, at 24 m 375; the swoop is free. §1, §2 and §10 are rewritten to it.
- **The plateau is a 12 m solid**, the shelf twelve metres down: the floor of
  the game is at zero. Marks spawn on the ground under them.
- **Its circle drifts over its target**, and **it decides once per approach**
  which air move the approach is for: a fixed circle round the tower left a
  hunter inside it never in the line-up arc, and without the one decision the
  long moves always pre-empted the mid ones.
- **It dwells on the floor after a Stoop** (2.5 s, the Screech and the buffet
  its price), **perches with one wing broken**, and **follows the terrain**
  round its circle (the tower and the rock face).
- **A crash cannot follow a crash** before it has been back up to its circle:
  without it a rider on the roots toppled it every time it stood.
- **Low is measured from the plateau**, so passing over the tower is not low;
  **nothing on it is solid through the Stoop's dive**, so the body does not
  barge the fighter out of the circle a frame before the hit.
- **The camera and the aim** (§6): `swing_path` measures its dead zone against
  the surface underfoot (`aim::underfoot_up`; bit-identical on the floor);
  `top_under` asks below the fighter's feet; the drawn eye is the aiming eye
  looking up, pinned on the plateau and aboard. **`aim::on_screen`** is the
  cone of `in_view` alone, for floor markers, which are drawn over everything.
- **Floor markers lie on the floor**: the shared telegraphs were drawn at zero,
  under the plateau; they and the bird's lanes lie on the floor their target
  stands on.
- **Question 5 (the Bulwark's guard against the volley)** is not built: the
  volley is unguardable, one answer per move.

**2026-10-01, later: the Bulwark throws his shield** (`hunt::class`,
`Hands::throw_in`): on a window five to eleven metres off he throws it at the
work, leaps to it as it flies and Slams out of the leap. His row above is
re-run with it; the feel log of the day has the before and after. The thrown
shield strikes no creature, so it is a way in, not a ranged blow
([review.md](../review.md) `CLASS-5`). Here nothing in wins (33 and 22 throws): the windows on the floor are too short for the leap and the Slam.
