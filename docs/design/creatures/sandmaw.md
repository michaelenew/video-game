---
status: built
proposed: 2026-09-30
built: 2026-10-01
tier: 2
---

# Sandmaw — the worm that hunts by ear

A worm eleven metres long that lives under a sand floor, perceives the arena
only as vibration, and comes up where it last heard something. The second
creature built on perception rather than reach, after the Ridgeback taught what
a monster must be. It is one entry in [the bestiary](../bestiary.md) and it
owes every rule in that document's §1; where this says "as the Ridgeback", the
rule is written out in [monsters.md](../monsters.md).

---

## 1 · What the fight is

**It is only there when it chooses to be — so make it choose.**

Most of the time the Sandmaw is under the sand and has **no hurtbox at all**.
What you see is its wake: a raised line of sand with a dorsal fin cutting
through it, moving at 9 m/s — faster than a walk of 7, so it cannot be
outrun for long, and slower than a dodge of 17, so it can be side-stepped.
Nothing you do to the fin does anything to the worm.

It does not see. It **hears**: every footfall at a walk, every dodge, every
landing, every hit that lands on sand, every stone raised and shield planted
puts a noise into the arena, with a place and a loudness, and the worm acts on
the noises it heard. A fighter who stands still, or crouch-walks at 3 m/s,
makes no noise at all, and outside the **feel radius** — 6 m from its head,
where it feels weight through the sand directly — is not there as far as it
is concerned.

So the body the worm attacks is never you; it is **the place you were loud**.
That is the whole fight, and it points two ways. Loud and then still is how
you get bitten. Loud *on purpose, somewhere near you but not on you*, is how
you make it come up where you can hit it.

The numbers, against the jump range of 2.7 m (Bulwark) to 6.0 m (Dual mage):

| | | Why |
| --- | --- | --- |
| Length | 11 m | Long enough that when it stands 6 m out of the sand, 5 m of it — and the tail — are still under, behind it |
| Thickness | 2.4 m | Lying beached on its side its back is at **2.2 m**, inside every class's hop with half a metre to spare |
| Mouth | a ring of teeth, open 4 m across | The swallow's tell is the mouth itself, and 4 m is legible from anywhere in the arena |
| Standing height (after a rise-bite) | 6 m of body out of the sand, leaning toward where it bit | The soft throat is the leaning face, from 1 m to 5 m up |
| Buried swim | 9 m/s, turn rate 120°/s | Catches a walker in the end; a dodge breaks contact |
| Health | 5200 (first guess) | See §9: fewer windows than the Ridgeback's, so less to chew through |

The arena (P2) is the **Pan**: 36 × 36 m of sand inside a rock rim 1.5 m
high, with three rock islands, each about 5 m across and 0.5 m high, set in a
triangle 14 m apart. The worm **cannot pass under rock**, the rim included. The
islands are where the bites cannot reach — which is why §3 spends a paragraph
on them.

The loop:

1. **Find the fin.** It opens buried, circling, for the Ridgeback's three
   seconds of `hunt_grace`. You learn where it is before it learns where you
   are.
2. **Choose what it hears.** Go quiet — stand, or crouch-walk — to be lost.
   Make a noise where you want it to surface: a landing, a hit into the sand, a
   stone, a planted shield.
3. **Get off the marker.** The rise-bite comes up under the noise after thirty
   frames of drawn warning. If the noise was you standing still, move.
4. **Punish the stand.** For seventy frames it stands six metres out of the
   sand with its throat open to you and its lowest vents behind it. This is the
   ordinary window, and it is most of the damage.
5. **Beach it.** Break it past `interrupt_strain` while it stands, or make it
   come up into rock — an Elementalist stone, or the lane of a breach that ends
   on one. It falls over and lies writhing on the sand for three seconds,
   2.2 m high: the back is climbable by everybody.
6. **Ride the writhe to the vents.** The spiracles along its back take double
   damage and are exposed only while it is out of the sand. When it dives it
   throws you. Go quiet, and do it again.

## 2 · The moves

Seven moves: three from below, three standing, one to go back down. All
numbers are first guesses; the first harness run will move them.

| Move | Range | What it is | Tell (frames) | The answer |
| --- | --- | --- | --- | --- |
| **Rise-bite** | any — at a noise | From under the sand, straight up through a circle **3 m across** centred on the noise it heard; launches whoever is in it. Then it stands for the window | Wake converges on the spot; then **30f** marker on the floor under the noise. 6 active, **70 recovery** (standing). 200 damage, launch 4 m | **Leave the marker.** Walk off it — it is under where you were, not where you are, and it does not follow |
| **Breach-dive** | long — at a trail | Surfaces ahead of a *moving* noise source and arcs over like a dolphin through a lane 10 m long and 2.6 m wide, re-entering at the far end | **36f**: sand lifts along the whole lane, drawn. 24 active (the arc), 40 recovery buried and deaf. 160 damage, knockdown | **Sidestep the lane** — a change of direction, which is also what beats the lead it placed the lane with |
| **Undertow** | close — at something it *felt* | A sinkhole 6 m across (a P4 hazard) under a quiet body inside the feel radius. Pulls toward the centre at 3 m/s for 120f, and **everything touching the sand in it moves at half speed**, dodges included. Rise-bite at the centre at the end | **20f** of sand sinking in a ring before the pull; the centre's bite marker for the last 30f. 0 damage itself | **Jump out of it.** The air is not sand: airborne fighters are not pulled and keep their takeoff speed, and a hop at a walk carries 5–6 m |
| **Sand spit** | mid — standing | A cone of wet sand from the mouth, 8 m long and 60° wide | **24f**: the head draws back, throat swells. 10 active, 40 recovery. 40 damage, **slow to half for 120f**. Never touches the camera | **Get behind a solid.** Rock, a stone, a planted shield or a raised one stop it; it is a spray, not a lane |
| **Tail lash** | close, behind — standing | The tail breaks the sand behind the standing column and sweeps a half-circle 3–7 m from the hole at 1.3–1.9 m up | **20f**: the sand humps and the tail tip comes out. 6 active, 50 recovery. 150 damage, stagger 60f. `hits_crouching` off | **Crouch under it.** It is aimed at a standing chest, and it is the one thing on the fight that a crouch answers |
| **Swallow-grab** | close, in front — standing | The head comes down, the tooth ring opens to 4 m, and it takes whoever is in the mouth under the sand | **24f** of the mouth opening. 6 active. Holds up to 120f: 60 on the bite, then 20 a gulp. See below | **Hit it first.** Any hit into the open mouth during those 24 frames gags it — a 40f flinch, the tooth ring's damage ×1.5 |
| **Sound** | its own back, and 3 m round the hole | The dive: the spiracles shut with a hiss, the body shudders, and it plunges | **40f** of the vents closing. 20 active. Riders thrown (45); anyone within 3 m of the hole knocked down (80) | **Leave its back** — step off before the plunge, not during it |

**No two answers are the same**: leave a marker, sidestep a lane, jump, get
behind something, crouch, hit it first, get off. Only the tail lash and the
sand spit could be answered by the dodge, and neither is *best* answered by it,
because the dodge is loud (§5) — a fighter who dodges the spit has told the
worm where they are.

**The rise-bite does not lead.** Every other creature aims at where you will
be. This one aims at where the noise *was*, and the marker is under it. It
means a fighter who keeps walking is never rise-bitten — they are thirty frames
and the glance's age beyond it before the sand opens. What the rise-bite
punishes is being loud and then **staying**: a landing, a heavy hit's
recovery, a stone you are standing beside, a dodge that ended where the last
one would have. The runner is the breach's business, and the breach leads.

**Its marker always contains the noise.** It cannot rise closer than its own
radius (1.2 m) to rock, so a noise at an island's edge is bitten at the nearest
point it fits — and since the circle's radius is 1.5 m, the noise is still
inside the circle. The marker is not always *centred* on the noise; it always
*covers* it. That is the fairness rule, and a test pins it.

**The swallow, alone.** Grabbed, you are dragged under and held in the throat,
drawn as the camera held just above the sand where you went down with the
throat's glow around you — the camera is never put underground. You take 60
on the bite and 20 each gulp. The throat **gulps every 30 frames**, a visible
contraction and a sound, and each gulp opens a 10-frame window: a press of
`Q` inside it is a guard break from the inside, and you are spat out beside the
hole and it flinches, surfaced. One press per gulp is read; a press outside a
window is spent and that gulp is gone, so pressing repeatedly is worse than
pressing once. Taken on the first gulp, the swallow costs 80; missed
throughout, 220 and you are spat out at 120f. **A teammate who hits the head
frees you at once**, and that is the coop answer (§8). The special is the key
because it is already each class's guard breaker; the move itself is not
thrown, so a Reaver's lotus and a Bulwark's grapple escape identically.

**The breach is a body in the air.** For its 24 active frames the arc has a
hurtbox, and a hit past `interrupt_strain` in that window drops it across its
own lane, beached. So does a lane that ends on rock: it arcs over and comes down
on a stone it did not hear. Both are rare and both are earned.

## 3 · A threat at every range

**Close — inside the feel radius, 6 m.** Here it does not need you to be loud.
A quiet body within 6 m of its head is felt, and gets the **undertow**. Standing
still beside the fin is the first thing a person tries, and the undertow is why
it does not work. Surfaced and close, in front is the **swallow** and behind is
the **tail lash**.

**Mid — 6 to 14 m.** The rise-bite at any noise you make, and the **spit** from
a standing worm, which reaches 8 m.

**Long — beyond 14 m.** The **breach** along a runner's trail, and the
rise-bite at anything loud enough: a landing is heard at 18 m, a raised stone at
30. A still sniper far away is unheard — which is the tempting spot below.

**Aboard — beached.** The writhe is a buck, judged by the Ridgeback's grip rule
(acceleration in the part's frame, crouch braces). It is gentle at the middle
of the back and violent at the head and tail. The **sound** ends every ride.

**The tempting safe spots, and what covers each:**

- **Standing still at a distance.** When it has heard nothing for
  `silence_patience` (4 s), it stops circling its last noise and **searches**:
  a widening spiral out from there at a swim. At 9 m/s the spiral passes within
  6 m of anybody in the Pan inside about eight seconds. Quiet buys time, not
  safety, and the wake coming toward you is the warning to move — quietly, or
  to make the noise you want it to follow.
- **A rock island.** Nothing from below reaches rock. So the worm, having heard
  you step onto it — footfalls on rock ring, and carry half as far again as on
  sand — circles the island, and after `island_patience` (5 s) **surfaces at
  the edge nearest your last noise and spits**. An island is 5 m across; the
  spit is 8 m. Standing behind the island's single boulder stops it; standing
  behind the worm puts you in the tail lash. The island is somewhere to breathe,
  and every six seconds it costs 40 and a slow — and a slowed fighter crossing
  sand is a slow noise.
- **Behind the standing column.** The lowest spiracles face backward at
  1.2–2 m, reachable by a level swing, and that is where the tail comes out.
  This is the station the fight is played from, exactly as beside a hind leg is
  the Ridgeback's: the lash's 20-frame tell is long enough to see over a poke
  already thrown, and a crouch answers it. Directly in front of it is the
  throat, and the swallow.
- **The air.** Airborne fighters make no noise until they land, and are not
  felt — but see the Dual mage in §7: *felt* means within 1 m of the sand, not
  standing on it.

## 4 · The approach and the window

**What you earn.** The rise-bite's **70-frame stand** is the ordinary window:
the throat (×1.5) from the front, the low vents (×2.0) from behind. A person
learns to earn it on purpose within a few minutes: make a noise, step off it,
and be two metres from the marker's edge when the sand opens.

**The big window is the beach.** 180 frames on its side, the back at 2.2 m,
six spiracles along it at ×2.0 and nothing it throws reaching its own back.
Three routes, all earned:

1. **Rise into rock.** A stone that arrives inside the rise-bite's marker
   during its 30-frame tell — the Elementalist's, raised into the circle — or a
   breach that ends on one.
2. **Break it while it stands.** A burst past `interrupt_strain` while
   surfaced, including mid-breach.
3. **Gag it into the ground.** A swallow's gag while strain is past `cc_strain`
   beaches it rather than flinching it. The answer to its most dangerous close
   move is also a door.

**What you break.** The **tooth ring**, 800 health, is reachable only when the
mouth is down: through the swallow's startup, the gag, and on a beached worm at
the head. Broken, **the rise-bite's circle shrinks from 3 m across to 2 m for
the rest of the fight**, and the marker shrinks with it — the players can see
that they made its main move easier to leave. Broken teeth also cannot hold: the
swallow's hold ends at the first gulp. The swallow is the move you hit into to
break the teeth, so the more of it you answer, the less of it there is.

**Crowd control.** Buried, it has no body and takes none. Surfaced, the
Ridgeback's thresholds hold: past `cc_strain` a root **keeps it up** (each root
pushes the sound back 60f, once per stand), a knock-up while past `cc_strain`
beaches it, and a slow slows its buried swim for the slow's length once it
dives.

**The arc.** `strain_desperation` lowers both thresholds as it is worn down,
as the Ridgeback's does. Two more steps:

- **Below half health, hungry.** It hears a quarter further
  (`hunger_hearing`) and its searches start after 2.5 s instead of 4. The Pan
  gets smaller.
- **Below a quarter, frantic.** It cuts its own stand short — **40 frames**
  rather than 70 — to dive after the *landing* of whoever it just launched,
  which is a noise like any other. The launched fighter's answer is the same as
  everyone's: the landing is loud, so get off where you landed. The window is
  shorter and comes more often; the fight goes from methodical to frantic
  without any rule changing.

## 5 · The brain

Everything in [monsters.md §7](../monsters.md) holds — it commits, it
telegraphs, it scores and draws — except what it glances at.

**It glances at noises, not bodies (P5).** The perception filter for this
species returns no bodies at all. Instead, fighters and the world push
**noises** into a ring of the eight most recent: a place, a loudness (the
distance it carries, in metres), who made it, and when. Every `glance` frames
(12) the worm takes the noises that reached its head since the last glance —
distance under loudness — and attends to the loudest, by loudness minus
distance. That is its whole picture of the world until the next glance.

| Noise | Loudness | Where |
| --- | --- | --- |
| Standing, crouch-walking | 0 | — |
| Footfall at a walk | 12 m | each stride, at the foot |
| Footfall on rock | 18 m | each stride |
| Landing | 10 m + 3 m per metre fallen, to 24 | where you land |
| Dodge | 20 m | at its start and its end |
| A hit landing on sand, or a guard breaker thrown | 24 m | at the impact — the end of the path `aim.rs` computed, never a new calculation |
| Champion's Rush | 24 m | every 2 m along it |
| Stone raised | 30 m | at the stone |
| Shield planted | 28 m, and rings for 1 s per unit of weight | at the shield |
| Quake, Tremor | the whole Pan | at the patch, every glance through the wind-up |
| **Felt** | — | any body within `feel_radius` (6 m) of the head and within 1 m of the sand |

**The lead is per move.** The rise-bite and the undertow have none
(`bite_lead` = 0): they go where the noise was, which is the fairness rule. The
breach leads along a **trail**: two noises from the same source within 40
frames give it a velocity, and the lane is laid along `trail_lead` (20 frames)
of it. That is the one prediction it makes, and it is only possible because you
kept making noise.

**The score:**

```text
U(m) =  heard(m)       whether the attended noise suits the move: a point for
                       the rise-bite, a trail for the breach, a felt body for
                       the undertow
      + range(m)       tent on head-to-noise distance, as the Ridgeback's
      + posture(m)     buried moves only buried, standing moves only standing
      + island(m)      large for surface-and-spit once island_patience has run
                       out on a noise last heard on rock
      + variety(m)     as the Ridgeback's
      + hurt(m)        as the Ridgeback's
      ⟂ cooldown(m)    as the Ridgeback's
```

`decisiveness` is higher than the Ridgeback's (0.8 against its value) because
the choice is mostly made for it by what it heard; variety comes from the
player's noises, not from its dice.

**It has no idea where you are between noises.** Buried with nothing attended,
it swims to the last noise and circles it (`circle_radius`, 5 m); after
`silence_patience` it searches. It never takes a body's position from anything
but a noise or the feel radius, and `crates/sim/tests/sandmaw.rs` fails if it
does.

**It stays up only so long.** Standing, it throws standing moves until
`surface_max` (240f) runs out or it has been struck past `interrupt_strain`,
then sounds. While standing it still hears, so a loud fighter in front of it is
the swallow's and a loud one behind it is the lash's.

**Deaf after a breach.** Re-entering, it hears nothing for its 40-frame
recovery (`breach_deafness`), because sand around it is loud. This is the moment
to go quiet or to reposition, and the wake visibly slows to show it.

New knobs, per species in the Oven (P1): `feel_radius`, `feel_height`,
`bite_lead`, `trail_lead`, `circle_radius`, `silence_patience`,
`island_patience`, `surface_max`, `breach_deafness`, `hunger_hearing`, and one
loudness per noise kind.

## 6 · Reading it

**Everything it knows is drawn.** The noise ring is in the snapshot, so the
renderer draws each noise the worm **heard** as a brief ring in the sand where it
was made — the player learns what is loud by seeing what it heard, and the
drawing is the rule's own data, as the overlay rule asks. Noises it did not
hear are not drawn.

**The wake and the feel.** The fin and the raised sand are always drawn. Around
the head's position under the sand a faint disc 6 m across shows the feel
radius, the same shape the simulation uses; inside it the sand at your feet
trembles. The wake **turns toward** whatever it attended at its glance, and a
turning wake is the first tell of every move from below.

**Floor markers**, from `Monster::telegraph` as the Ridgeback's: the rise-bite's
circle, the breach's lane, the sinkhole's ring and its centre bite, the spit's
cone, the lash's half-annulus, the sound's 3 m ring. Drawn over the sand, never
under it.

**Silhouette, standing.** The column leans toward where it bit, so the throat
faces the fight and the vents face away. The mouth opening to 4 m for the
swallow is the loudest shape in the fight. The spiracles are dark slits along
the back that **flare open while it is up and clamp shut 40 frames before it
sounds** — the ride's one tell, visible from the back you are standing on.

**Sound.** A low thrum under everything that rises in pitch as the wake closes
on you; a dry crack when it attends to a noise (the glance, audible); the
gulp. The worm's own sounds are the only music.

**The spit never blinds.** It is a cone on the floor and a spray in the air
that is drawn thin at the camera and never covers the screen. A creature that
takes away the picture has taken away the only thing the player plays from.

**What "unanswerable" means here.** A hit is unanswerable if its tell was under
15 frames and nothing positional warned of it, as for every creature — and,
for this one, **a rise-bite or undertow whose marker did not cover a noise the
victim made, or a point where the victim was felt.** That count must be zero.

## 7 · The classes

**Shadow Reaver — easy, and it is her identity.** The shadow has no weight: it
makes no noise and is never felt, and its copied swings into sand are silent.
Her body is loud and she can leave it: make the noise, send the shadow two
metres off the coming marker, dash. No class gets off a rise-bite marker
quieter or later. The hard part is that her tally builds on copies thrown from
the field, and a buried worm offers no body to copy onto — she earns the tally
only in the stand and the beach.

**Elementalist — the bait, and the beach.** Her stones are the loudest single
noise and the only thing that beaches the worm on purpose: raise a stone into a
rise-bite's marker during its 30-frame tell and it comes up into rock.
**Quake is the best bait in the game**: a patch that is loud for its whole
wind-up and leaves a stone at its centre, so a worm that commits to the noise
can be standing up through the stone when it arrives. Whether it does depends
on Quake's wind-up against the glance plus the 30-frame tell, which
`cargo run -p sim --bin elemental` will print; if it lands every time, that is
a tuning bug, not a feature. Tremor is the same noise under her own feet, and
standing on its stone she is on rock. The risk is that she has it too easy;
the cap of three stones and the spit that covers every stone she stands on are
the cost.

**Blood mage — close and quiet.** Her scythe wants her within reach, which is
the feel radius, so she fights the undertow more than anyone. Her blink is
silent on departure and a small splash on arrival (8 m), which makes it the
quietest movement on the roster that covers ground. Blood on sand soaks in —
pools drain at twice the rate on the Pan — so she must drink from the stand and
the beach rather than stockpile. The creature bleeds when struck standing; a
pool beside a beached worm is a door onto its back.

**Dual mage — she cannot bait with her feet, and is not ignored.** Floating
makes no footfalls, so she is the only class who moves quietly at full speed.
She cannot bait it with her body the way a walker can; she baits it the way a
caster does, with hits that land on sand. She is **not** unperceived:
`feel_height` (1 m) means a float close to the sand is felt inside 6 m, exactly
as standing is. Only real air — the second jump, the wings — is unfelt. Is it
fair? She is the class the worm loses most easily, and she pays for that with
bar depth she must climb for; that is her identity, not a hole, **if** the
harness shows she cannot win alone without ever touching sand. If she can, the
feel height goes up.

**Champion — loud, and must learn to stop.** Every weapon lands heavy and the
Rush lays a trail every two metres, which is exactly what the breach leads
along. His lesson is the fight's: he is the best bait on the roster and the most
often bitten. Rush cancels recovery, so it is how he leaves a marker after a
hammer hit — noisily, into a breach if he goes straight. The spear's reach hits
the low vents from outside the lash's inner edge.

**Bulwark — the decoy, and the one who struggles to jump out.** A planted
shield **rings**: it is a noise that repeats for a second per unit of weight,
so he can plant it, walk away quietly and let the worm come up beside a wall of
metal. A rise-bite does not beach on a shield; it throws the shield clear, 3 m,
still planted. The shield in hand blocks the spit and the lash from the front,
and a parry on the swallow's lunge is a hit into the mouth. His hole is the
undertow: the lowest hop on the roster has the least airtime, carrying about
4.7 m from a walk — enough from the centre of a 3 m-radius hole, not from a
standing start near it. A hole or the class? A person has to say; the knob is
the undertow's radius.

## 8 · Coop

**Two players split the verbs.** One is loud, one is quiet: the loud one baits
and leaves the marker, the quiet one stands at the vents when it rises. The
worm attends the loudest noise, so the pair chooses which of them it hunts —
the fight turns into a conversation about who makes noise next.

- **The swallow rescue.** A teammate's hit on the head frees at once. Solo, the
  swallow is a gulp timing; in coop it is a call for help, and the most
  valuable thing to be standing near.
- **The undertow catches two.** A quiet pair standing together inside the feel
  radius both get it. Coop spreads out.
- **A beached worm with two riders** and one on the floor: the vents are
  shared, and the sound throws both.
- **Numbers.** Health ×1.7 (8840). The noise ring stays at eight: two players
  fill it twice as fast, which makes it forget sooner, which is right. The
  glance stays at 12. `surface_max` stays — two people make its stands worth
  more, not longer.

## 9 · Measuring it

**The scripted hunter's plan** — what a person learns in their first ten
minutes:

1. Stand still until the wake is inside 10 m and heading at you.
2. Make one deliberate noise — a jump landing, a hit into the sand — then walk
   (not dodge) 2 m off where it was made.
3. When the marker shows, step to 1–2 m past its edge, facing it. When it
   stands: in front, hit the throat until the swallow's mouth opens, then hit
   the mouth; behind, hit the low vents and crouch on the lash's tell.
4. On a breach lane: sidestep. On a sinkhole: jump outward at once.
5. Beached: climb, go to the middle of the back, hit vents, crouch when the
   writhe is violent, step off when the vents clamp.
6. On an island: wait for the spit's tell and stand behind the boulder, then
   come off onto sand on the side away from the worm.
7. Escape a swallow on the first gulp, with the slop the harness gives every
   timing.

**New fight report lines** (P8):

| Line | Why | Target |
| --- | --- | --- |
| **Bitten while quiet** — split *felt* and *unfelt* | Any bite on a hunter who made no noise for 60f. Felt is the undertow working; unfelt is a bug | felt: a few a fight; **unfelt: zero** |
| **Marker covered the noise** | Every rise-bite and undertow, checked against the noise it acted on | 100% |
| **Unperceived share** | Frames the hunter was neither heard nor felt | about a third |
| **Heard, per kind** | Which noises it acts on | no kind above half |
| **Stands, beaches, and by which route** | Whether the window and the big window are being earned, and how | a beach a minute; every route seen across twelve hunts |
| **Island time** and **spits taken there** | Whether islands are a room or a breath | under a fifth of the fight on rock |
| **Swallows**, **escaped on gulp n** | Whether the solo escape can be done by a person with a quarter-second reaction | most escapes on the first or second gulp |
| **Tooth ring broken, at what minute** | Whether the permanent change happens in a won fight | in most wins |

**The four windows** need one change. For the Ridgeback, *free* meant "could
start a move". For this worm, free and deaf to you are not the same threat, so
the harness counts a frame as **threatening** only if the worm could start a
move **and** has perceived the hunter in the last glance. Frames where it could
move but has not are their own band, *unperceived*, above.

| Window | Asked for |
| --- | --- |
| Threatening | ~35% |
| Open to a poke | the rest |
| Open to a way in | the rest |
| Safe to walk up | ~20% — the stands and the beaches |
| Unperceived (new) | ~30%, overlapping the above |

**Tier 2 targets**: the scripted hunter wins about two in three, in 2–4 minutes;
unanswerable hits zero; bitten-while-quiet-and-unfelt zero.

## 10 · What it needs built

**Shared machinery:** P1 (species), P2 (the Pan: sand and rock as floor
materials, three islands, and "can this pass under here" as an arena query), P4
(the sinkhole — one hazard, plus the spit's slow if slows are hazards), P5 (the
perception filter: this species perceives noises and the feel radius, and no
bodies), P8 (a plan and report lines).

**New, and only this creature's:**

- **Noise emission.** A small set of hooks where fighters and the world already
  do the loud thing — stride, dodge start and end, landing, hit landing,
  stone raised, shield planted, Rush step, Quake glance — each pushing one entry
  into the ring. The ring is fixed-size and overwrites the oldest. Positions of
  ranged impacts come from the path `aim.rs` already built for the hit.
- **A buried body.** A part flag `buried` meaning "no hurtbox, not solid, not
  mountable". Buried is most of the fight, so this is the most-read flag.
- **A worm spine.** Ten segments that follow the head (follow-the-leader),
  depth per segment, so the wake, the standing column and the beached body are
  one chain. Stored as positions, not as clips, because where it is is gameplay.
  Standing and beached poses are clips on the Ridgeback's factory.
- **The swallow's hold**: a held fighter, a gulp clock, one read per gulp.

**No aim.rs change.** The standing column and the beached body are bodies, not
on the crosshair's ray, and `aim::first_along` finds them along a shot's path
exactly as it finds the Ridgeback. The throat 1–5 m up is reached by a swing's
exact pitch above the horizon, and by a skillshot whose ray meets the rim or the
range sphere behind it. The one thing aim-shaped here — where a ranged hit's
noise is — reads the path's end point that `aim.rs` already returns.

**Snapshot:** monster core ~120 B, spine 10 × 5 B = 50 B, noise ring 8 × 12 B =
96 B, one hazard 8 B, swallow and tooth state 8 B — about **280–300 B**, as the
bestiary's budget guessed.

**Per-frame cost** is dominated by the spine: ten capsules against every
fighter body and live hitbox, and against rock for the head. Noise hearing is
eight distance compares a glance. A rollback of eight frames is under a
thousand capsule tests. No allocation: the ring and the spine are arrays.

**Tests** (`crates/sim/tests/sandmaw.rs`):

- `a_buried_sandmaw_has_no_hurtbox`
- `a_fighter_standing_still_outside_the_feel_radius_is_never_perceived`
- `a_crouch_walk_makes_no_noise_and_a_walk_does`
- `it_never_takes_a_position_from_a_body_it_did_not_hear_or_feel`
- `the_rise_bite_marker_always_covers_the_noise_it_acted_on`
- `a_fighter_who_walks_off_at_once_clears_the_rise_bite`
- `a_spit_slowed_fighter_can_still_walk_off_the_marker`
- `nothing_passes_under_rock`
- `an_airborne_fighter_is_not_pulled_by_the_undertow`
- `a_hop_from_the_undertow_centre_clears_its_edge_on_every_class`
- `a_crouching_fighter_ducks_the_tail_lash`
- `the_swallow_is_escaped_on_a_gulp_and_extra_presses_cost_the_gulp`
- `a_teammate_hitting_the_head_frees_the_swallowed`
- `a_rise_into_a_stone_beaches_it`
- `the_beached_back_is_inside_every_class_hop`
- `a_broken_tooth_ring_shrinks_the_rise_bite_for_the_rest_of_the_fight`
- `the_spit_slows_and_never_covers_the_camera`
- `the_noise_ring_is_bounded_and_does_not_allocate`

**Milestones**, each a few hours at most and ending in something checkable:

- **M1 · The Pan.** Sand and rock regions, three islands, the rim, the
  "under rock" query. *Check:* `nothing_passes_under_rock`; a screenshot.
- **M2 · Noise.** The emission hooks and the ring; the perception filter; noises
  drawn in the sand. *Check:* the three perception tests, and `determinism.rs`.
- **M3 · Buried, and the rise-bite.** The spine, the wake, the fin, the feel
  disc, the rise-bite and its stand, the throat and low vents. *Check:* the
  marker tests; `SHOT_MOVE=rise_bite` screenshot.
- **M4 · The rest of the moves.** Breach with its trail lead, undertow on P4,
  spit, lash, swallow and its gulp, sound. *Check:* each move's test; the
  frametable prints seven moves with their answers.
- **M5 · The beach and the ride.** Three routes to it, the beached body as
  mountable parts at 2.2 m, spiracles, the writhe as a buck, the tooth ring.
  *Check:* `the_beached_back_is_inside_every_class_hop` and the tooth-ring
  test; `beastcheck` prints the beached back against every hop.
- **M6 · The hunter and the report.** The plan above, the new lines, the
  unperceived band. *Check:* twelve hunts, unfelt quiet bites zero, and the
  numbers written into §9 under "Where it landed".

## 11 · In the world

**Region:** a salt-and-sand basin at the dry end of the world — proposed to
[world.md](../world.md) as **the Glass Flats**, where the sand has been fused
to glass in places by old heat, and the glass is the rock it cannot pass under.

**The arena:** the Pan as in §1, its rim a ring of glassy rock, three glass
islands with one boulder each (the spit's cover), bleached bones half sunk in
the sand as the only landmarks. At dusk the sand is pale and the wake reads
dark; the art needs no new asset beyond the worm, the islands and the ripple.

**Trophy:** the **tooth ring**, if it was broken in the fight — hung as a
circle of teeth over the hub's door. A trophy that says how you won it, not
only that you did.

**Sidegrade** (a common-mechanic modifier, in the spirit of
[parked.md](../parked.md)): **Sand-step** — the dodge covers a fifth less
ground and ends three frames sooner, and it makes no noise in a hunt. A shorter,
quicker dodge that is better at leaving a marker and worse at crossing a lane;
in versus it is a spacing tool traded for reach. Enabling, not power.

## 12 · Open questions

1. **Is standing still too strong?** The search spiral and the feel radius are
   the cover. Whether a person reads the wake coming and feels hunted, or feels
   they can wait it out, is the whole premise, and the knobs are
   `silence_patience` and `feel_radius`.
2. **Is walking the right thing to be loud?** Footfalls at a walk mean ordinary
   movement is noise and the only quiet movement is 3 m/s. That might be the
   fight, or it might be a slog. The alternative is that only a sprint is loud,
   which the game does not have.
3. **The Dual mage.** `feel_height` at 1 m makes her float felt close in and her
   air unfelt. Is she the class the worm loses — her identity — or a class for
   whom the fight is not happening?
4. **The Bulwark's undertow.** His hop carries about 4.7 m. Is that a hole to
   close by shrinking the sinkhole, or the class being bad at one thing?
5. **The Elementalist's beach.** If stone-into-marker works every time, the
   fight is a three-stone script for her. Is a beach every 20 seconds her
   identity or a solved puzzle?
6. **Seeing what it heard.** Drawing each heard noise teaches the rules
   quickly; it also makes the worm a readout. Keep the rings, or only the
   crack of its glance?
7. **Is the swallow's `Q`-on-a-gulp readable?** It is the one timed press in
   any creature fight. The alternative — a fixed cost, no escape — is simpler
   and has no skill in it.

## 13 · Where it landed

Built 2026-10-01: `--hunt sandmaw`, the Pan. The species is
`sim/src/species/sandmaw/` (the table; `fight.rs` for the postures, the
senses, the sinkhole, the swallow and the beach; `mind.rs` for its own
scoring and where it swims), the arena `sim/src/arena/sandmaw.rs`, the clips
`anim/src/beast/sandmaw/`, the look and the Pan's dressing in `game`, the plan
and the report lines `hunt/src/plans/sandmaw.rs`, and the rules pinned as
sentences in `sim/tests/sandmaw.rs`. The plan is
[plans/sandmaw.md](../plans/sandmaw.md); the passes are in
[feel-log.md](../feel-log.md) of 2026-10-01.

Numbers from `cargo run -p hunt --bin fight -- --species sandmaw --class <c>
--repeats 24`; the scripted hunter plays §9 with a fifteen-frame reaction.

```text
                won    mean    health left (a win)   threat / poke / way in / walk up   unanswerable
  Champion     18/24   191 s        461                  65 / 12 / 15 /  8 %                 0
  Bulwark      24/24   214 s        665                  64 / 12 / 16 /  7 %                 0
  Reaver       10/24   223 s        329                  67 / 11 / 15 /  8 %                 0
  Elementalist 11/24   338 s        252                  49 /  7 /  9 / 35 %                 0
  Blood mage    0/24     --          --                  66 / 11 / 15 /  8 %                 0
  Dual mage     3/24   209 s        326                  61 / 12 / 18 /  8 %                 0

  (2026-10-01, every class played; on main the hour before it, 18 / 24 / 8 /
   4 / 0 / 0 won -- the Champion's and the Bulwark's rows had moved since this
   section was first written; the class layer moved only the Bulwark's health,
   622 a win before it)

  coop, two Champions 12/12 in 95 s;  temper 3, Champion 6/12 in 207 s

  landed / thrown, 24 Champion hunts
    Rise-bite 39/717  Breach-dive 2/53  Undertow 0/157  Sand spit 60/199
    Tail lash 7/114   Swallow-grab 8/309  Sound 46/684  Dive 0/35
    (the undertow's damage is the rise at its middle, counted as a rise-bite;
     Swallowed and Dive are never chosen: the grab and the beach start them)
```

**Its own lines**, over the same 24 Champion hunts: bitten while quiet 0 felt
/ 0 unfelt; the marker covered the noise 717 of 717; unperceived 24 % of the
fight; what it acted on, felt 83 %, footfalls 14 %, landings 2 %, rock 1 %;
717 stands and 35 beaches (6 broken, 29 knocked down by the Champion's rush,
none by stone, lane or gag); 1 % of the fight on rock; 8 swallows, all out on
the first gulp; the tooth ring broken in 2 of the 19 wins. The perceived
threatening band is 49 %, unperceived 18 %. The Elementalist's 24: 268
beaches, all but one by a stone in the circle.

**2026-10-01: the hunter plays all six classes** (`hunt::class`,
[bestiary.md](../bestiary.md) §8). **The Elementalist** wins 11 from 4: the
plan's stone beaches the worm as before, and now a pillar goes under the
beached back (138 in the 24) and her bolts at the throat are aimed at it --
slow, at five and a half minutes, but won. It was 20 before main's aim A3
merged in: the beached back seen from above became a place, so a bolt aimed
at it from a stone stopped on the hide's top; the layer now pushes such a
spot out past the edge (`Hands::spot`), which won back ten of the nineteen
the merge first cost. Whether the rest is the aim or the plan is open.
**The Reaver** sends her shadow to the throat on the way in to a stand (427)
and opens a lotus on it (196): 10 from 8.
**The Dual mage** wins 3, her finishers into the stand (711). **The Blood mage
still wins nothing**: she blinks in and out of a stand on her pools (166), but
the pools a scythe leaves on a worm this size are never worth a spike, and her
damage is a third of a sword's against 13 000. Zero unanswerable for every
class, still.

**Against the targets.** Zero unanswerable hits and zero quiet-unfelt bites in
every class, every hunt: the contract holds. The Champion wins four in five
(two in three asked) in a little over three minutes (two to four asked).
**What is off:** threatening is 49–66 % against 35, walk-up 7 % against 20, and
beaches come about once in two minutes, not one a minute. The stand is the
window, and the stand counts as *threatening* -- it can act at once -- so the
walk-up band is only the beached worm and the rise's recovery. "Felt" is 83 %
of what it acts on: the hunter stands inside the feel radius at a stand far
more than §9's plan says, because the stand is where the damage is. The tooth
ring is rarely broken: the hunter hits the throat and the spiracles, not the
mouth, except through a gag. **The Reaver** loses to the spit (half of its
spits land): a dodge that has to cover six active frames from fifteen frames
behind is the hardest timing in her kit. **The Elementalist** beaches it at
every rise with a stone, and since 2026-10-01 burns the beached back and
wins most hunts, slowly. **The Dual mage and the Blood mage** lose nearly all
of them with their bars and pools played (above): the worm's health against
their damage, a finding for their kits rather than the worm.

**The body, as `beastcheck --species sandmaw` prints it**: nose to tail
12.5 m (eleven of body and the lips); buried, every surface under the floor;
beached, its lowest surface at the floor and its back, through the whole
writhe, inside every class's hop
(`the_beached_back_is_inside_every_class_hop`). Standing, the column is a
wall (`Steepest`, cos 0.7): nobody stands on it, and nobody mounts a worm that
is not beached.

**Changed from this document while building**, beyond the numbers:

- **The undertow is 2.4 m in radius**, from 3 (4.8 m across, from 6), since
  the Hornback fixed the floor's lift (2026-10-01). Until then every falling
  fighter in a fight with anything on the floor fell at almost nothing a frame,
  so a hop floated out of the sinkhole on any class. Without the float a hop
  leaves the sand at the half-speed walk it took off from (3.5 m/s), and the
  Bulwark's carries 2.5 m from the centre, not §7's 4.7; the Champion's 3.2, the
  Dual mage's 4.4. §7 names the radius as the lever for him, and 2.4 is the
  largest that keeps **jumping the way out for every class**
  (`a_hop_from_the_undertow_centre_clears_its_edge_on_every_class`). The
  sinkhole's "jump from it" is a slow (0 to 1), not a boost, so it cannot be
  the lever. Re-run at 2.4 with the fix, 12 hunts each: the Champion 9/12 in
  190 s (65 / 12 / 15 / 8 %), the Bulwark 12/12 in 215 s, zero unanswerable.

- **Health 13 000**, from 5 200: a stand is eight seconds of a free throat,
  and the scripted hunter at 9 000 and 10 000 won in two minutes and a half.
- **The spit is 6 active**, from 10: the dodge's ten invulnerable frames
  could not cover ten active frames with any slop, so a spit with no rock near
  was a hit nobody could answer by the dodge §2 offers.
- **The swallow's tell is 30 frames**, from 24, and **a blow inside the open
  ring gags it** (`MouthOpen`, 2 m half across), not only one on the teeth: the
  neck arcs over a fighter in front and the throat meets a swing before the
  ring does. At 24 frames a person with a quarter-second reaction had two
  frames to start a swing in.
- **The Bulwark's raised shield stops the spit** (it is a solid he carries);
  §7 already said so.
- **Struck past `interrupt_strain` in a stand beaches it**, rather than
  sending it under (§5): one rule for the stand, the window that pays.
- **The rise travels round rock under the sand** and is aimed at the nearest
  sand to the noise it can fit (`fit`), 1.2 m off any rock; a noise on an
  island is answered by surfacing at its edge to spit (`IslandPatience`).
- **The swallow is 60 on the bite, 20 a gulp at 30, 60 and 90 frames, and 80
  when it spits you out at 120** -- the only reading of §2 that makes both of
  its totals true.
- **Strain bleeds at 1 % a frame** and **the interrupt is 700**, from the
  shared 2 % and 900: at those, three swings into a stand never added up.
- **Not built**: the sand-step dodge (§11, parked), and a separate camera for
  the swallowed fighter (the Mireback's stomach is the same).

**Questions for a person**, beyond §12: is a stand that counts as threatening
the right reading of §9's band (the worm can act at once, but every move it
has from a stand is a 20–30-frame tell); is the stone beach on every rise the
Elementalist's identity or a solved puzzle (§12 question 5 -- the harness
says it is every rise); and does the swallow's mouth, coming down 4 m across,
read as the thing to hit, which only somebody facing it can say.
