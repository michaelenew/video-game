---
status: proposed — nothing here is built
proposed: 2026-10-29
---

# The opening — a prison, a climb, and the first fight

How a new player meets the game: where they wake, how the kit is handed to
them, and the first fight they are built toward. Written from a conversation
with the owner on 2026-10-29. It serves [intent](intent.md), and it is also
the first test of a **combat identity** (§2) that the intent leaves unsaid.

## 0 · Why this exists

The owner, on what is wrong with how the game starts now:

- *"I actually really dislike the current valley design as 'progression'. It's
  boring both visually and mechanically to progress through."* The things that
  hold people are the climbing challenges, and most people try those for a
  short while and then give up, so **a hard jump course is not the hook**.
- *"Walking out of a prison cell means something about your circumstance,
  appearing in a bland village center does not."* The model is the start of
  Dark Souls or Tears of the Kingdom: a place that tells you your situation.
- **There have been explicit requests to introduce the clicks one at a
  time.** A whole kit at once is too much to take in.

And on what the hook has to be. The owner's children's games
(`michaelenew/kids-games`) worked because the first minutes had one clear
goal, a win that was not too hard, and plain feedback that they were doing
well, followed by short bursts of learning, each paid off by something fun:
building the robot, getting a unicorn. In Mecha Forge the reward is **the
power of the upgrade**, and the rule is *the reward is not a medal screen,
it's watching the hit bounce off*.

This game borrows that rhythm but not its difficulty. The draw is Dark
Souls': *"the Asylum Demon isn't a pushover, it's a genuinely challenging
fight and quite scary the first time you encounter it ... you can beat this
thing, ... but the cards were stacked against you from the start and that's
abundantly clear the moment it walks in."* **The cards stacked against you,
but beatable, so it was fair.**

Two things are therefore wanted from the opening at once:

1. **The world reacts, materially, as you get through each challenge.** Things
   break, open and fall. A panel ticking a box is not the reward
   ([tutorials.md](tutorials.md)'s halls stay as reference, not as the hook).
2. **Each new piece of the kit is a jump in power you can feel.** The Mecha
   Forge feeling, made out of moves the game already has.

## 1 · What the opening is for

Finding out what is fun about this game. The owner: *"figuring out 'what's the
walk up to the first fight, and what does that fight look like' I think is
the process of figuring out what's fun about this game."* The Asylum Demon is
the model of a Dark Souls fight in one creature, *big guy smash hard, be good
to smash back*. This game has not found its equivalent yet, its **canonical
fight**, and the first fight is a good place to look for it.

## 2 · The combat identity this tests

What sets this game apart is **movement**: near-total freedom, very large
maneuvers, and a skill in adjusting on the fly and doing it consistently.
That is closer to Mario Odyssey or Sonic Adventure 2 Battle than to Dark
Souls. The fights do not use it yet. In the Ridgeback, the fight that uses
it most, movement is a phase (get up, then ride), not the fabric of the
fight ([monsters.md](monsters.md) §1).

The proposal, for the first fight:

> **The monster owns the floor. You own everything above it.**

- **Staying on the floor is the hard way to fight it.** Its big moves are
  aimed at the ground: sweeps, shockwaves, slams. They can be dodged on the
  floor, and a careful player should be able to win there, slowly.
- **Getting off the floor is how you stop taking hits.** As in a movement
  shooter, you avoid a blow by not being where it lands. This fits what is
  already decided: no cooldowns, *Smash, not Tekken* ([combat
  kernel](combat-kernel.md)).
- **Height is where the damage is.** The soft place opens only from above:
  a dive from a ledge, a perch, its own back. Moving well becomes hitting
  hard.
- **The arena changes under the fight.** It breaks what you stand on, and
  what it breaks becomes a new way up.

That is how a fight can be won by anybody and mastered through movement. A
Mario Odyssey boss can be beaten by jumping its shockwaves; a good player
never touches the floor.

**It is the rule for this fight, not for every fight.** The owner expects
other fights to work this way too, and some not to. That is fine.

The Asylum Demon already makes the point. The fight a player wins **starts
with a dive from a ledge** that takes a large share of its health: where you
stand decides the fight. That is the moment this opening is built around.

## 3 · The kit, handed out one piece at a time

The opening hands the kit over in order, and each piece is a reward. Nothing
new is designed: every move exists, and the whole kit is in hand by the end
of the opening's first stretch. It is also a first, gentle trial of the
progression the owner likes best, *unlocking new parts of the character*
([intent](intent.md) §4), with none of that idea's input-budget worry,
because nothing is added.

**The rule that makes it work: a click arrives as itself, and its takeoff
arrives with the movement.** Most of a class's height comes from its
takeoffs, `space` + a click (the Reaver's Gallows, the Elementalist's Earth
jump, the Champion's Pole drive), and from its own movement tool. A click
handed over early is its tap, its hold and its move in the air; its takeoff
stays locked until the movement unlock (§5, step 6). The dive in the air
(Plunge, Landfall, the air swing) is part of the click, not the takeoff, and
is needed early: it is how you hit from above.

| Step | What is in hand |
| --- | --- |
| The cell and the climb | **The moves every class shares**: walk, run, jump (and holding it), wall slide and wall jump, ledge catch, dodge |
| Partway up | **One click**, its tap, hold and air move, no takeoff |
| The first fight | The same. A small kit, completely learned |
| Its reward | **The class's movement**: every takeoff, and the class's own movement tool |
| After it | **The other clicks and keys**, one per short stretch, each shown by a small fight or obstacle that needs it |

Per class, a first guess:

| Class | First click | The movement unlock |
| --- | --- | --- |
| Champion | Left, the sword | The takeoffs (the Pole drive above all) and the Rush |
| Shadow Reaver | Left, the finishers: Reave, and Plunge from the air | Gallows (the takeoff), the dash and the dash jump |
| Elementalist | **Open** (OPENING-3): her stones are both an attack and a climb | Earth jump and the structure jumps |
| Blood mage | Whatever her first click is when this is built | Her movement tool, whatever it is then |
| Dual mage | The same | The same |

**The Blood mage and the Dual mage are left loose on purpose.** The owner
expects both movement tools to change, so the opening should not be fitted to
what they are now. The test they need is a single one: that with only the
shared moves and one click, the climb (§5) is still a climb for them. The
Dual mage floats by nature (her gravity is ×0.8, and her hangs in the air are
the longest), so she is the one to check first.

**Where the reduced kit applies.** Only in the opening. The tutorial halls
and versus keep the whole kit, and a returning player can skip the opening
(OPENING-5).

## 4 · Why the creature does not have to be enormous

The owner's worry: *"it's possible to cover a lot of height very quickly, and
if the run up the tower needs to be a challenge this thing needs to be many
times the height of the Ridgeback today ... The Ridgeback can be mounted
trivially by every real class with a lot of room left over."*

That is true with the whole kit, and most of that reach is in the class
tools: the stone launch 17–25 m ([courses.md](courses.md) §1a,
[tutorials.md](tutorials.md) §2), the Reaver 50–75 m through the Whirl
(courses.md §0). **The shared moves alone reach much less, and have a
ceiling**:

- A standing full hop peaks at 2.7 m on the heaviest class and 6.0 m on the
  floatiest ([monsters.md](monsters.md) §1); a run and jump goes 7–9 m
  level (courses.md §1a).
- A wall jump rises about 0.6 of the class's takeoff, each one after it in
  the same airtime rises ×0.7 of the last, and the same wall works again
  only from lower down. *"A chimney or a corner can still be climbed a
  little; it cannot be climbed forever"* ([controls.md](controls.md),
  walls).
- Three of the five classes have no answer to a 9 m rise without their tools
  (courses.md §1a). The Ridgeback's back is out of reach from the floor for
  five of the six classes (monsters.md §1).

So **with the kit held back, a creature of about two Ridgebacks**, its soft
place some 10–12 m up, in a shaft where the walls and perches decide how you
go up, **is a real threat on the way up and a real fight at the bottom.**
Its exact size is a number to set by play, not here (OPENING-4).

This also sets how creatures scale. The first is sized against the small kit
and later ones (the Siegeshell) against the whole one. **The Ridgeback
feeling small once your movement is unlocked is the point, not a flaw**:
it is how you feel the power you were given.

## 5 · The opening, step by step

A sketch, to be found by play. The owner is not a storyteller
([intent](intent.md) §5), so the place below is a *circumstance*, not a
story: what is happening to you, with as little said as possible, the way
an Undead Asylum cell says it.

1. **The cell.** You wake at the bottom of a tower prison cut into a cliff, a
   place for keeping things. Something enormous lives in the tower's central
   shaft (*the Warden*, a placeholder name). Before you can do anything, it
   strikes the tower and **your cell wall cracks open**. That is your way
   out, and the first time the world moves because of you.
2. **The climb out.** You climb through the broken tower, one shared move at
   a time: the jump, holding it, the wall slide and wall jump, the ledge
   catch, the dodge. **The margins are wide.** This is
   [intent](intent.md) §3, danger that feels larger than it is, not a jump
   course to grind. A fall stands you a short way back. The Warden is seen
   through gaps and windows the whole way, close, and once or twice it
   reaches in and takes away the way you were about to use.
3. **The first click.** Partway up, the first click is given (by what, and
   how, is OPENING-6), and small things that live in the prison show what it
   does, in two or three hits each.
4. **The first meeting.** The climb comes out onto a gallery over the
   Warden's floor, and it is there. **You can fight it**, and a good player
   might win; the cards are plainly stacked against you, and there is a way
   out up the far wall, as obvious as the Asylum Demon's door.
5. **The return, from above.** The way out leads round and back to a ledge
   over the floor. **The fight opens with a dive onto it** (§6).
6. **The reward: your movement.** You beat it and are given the class's
   movement (§3). **The world shows you at once what it means**: the shaft
   you crawled up, or a gap you could not cross, is one jump now.
7. **The view out.** Out of the top of the tower, the land and something to
   go toward. What that is belongs to a storyteller, and is OPENING-7.
8. **Then the rest of the kit**, a click or a key per short stretch, each one
   paid off.

## 6 · The first fight

**The arena.** The bottom of the shaft: a round floor of broken stone, the
wall rising all round, and things to get up on: **columns, scaffolding,
chains, ledges** at heights the shared moves reach in two or three steps.

**What it does.** It owns the floor. Its moves are aimed at the ground, so
each has a floor answer (a dodge, a step out) and a better answer up off it:

- **A sweep** across the floor at a height you jump.
- **A slam that throws a shockwave** out along the floor, jumped or dodged
  (Mario Odyssey's teaching move).
- **A move that takes the floor away** for a while: a quake that cracks it,
  something that pools on it. Built for this fight: the game's floor
  hazards today are things to step round on the ground, not reasons to
  leave it.

**What it does to you up high.** Being up cannot be free, or the fight is
over at the first perch. **It swats what you are standing on.** A column
struck falls, a scaffold struck breaks. That tells you to move on, and
**the arena changes**: every perch it breaks is one fewer, and the rubble
is a new ramp, often onto it.

**Where it is soft.** Up high: on its back, at the nape, wherever a dive from
a perch lands. A dive from above is the big damage, as the opening dive is.
Its rhythm is therefore:

1. Hold out on the floor.
2. Get up: a column, the scaffolding, the wall.
3. Dive onto the soft place.
4. Move on before it swats where you are.

**The contract.** The [bestiary](bestiary.md)'s, unchanged: every threat
announced before it lands, every opening real, a loss the player's and never
the creature's. *Tough*, in the intent's sense: a fight you do not win the
first time and can see yourself winning.

## 7 · Building it, and the test

**Build the fight first.** Not the tower, not the cell: a rough blockout of
the arena (plain boxes for the columns and ledges) and the creature, and
play it. The tower and the walk up are designed back from the fight once it
works: each step of the climb teaches one thing the fight asks for.

**Nothing in the game today is a starting point for the substance.** The
owner, 2026-10-29: *"Nothing in the game today has the right substance that I
am looking for. Every fight and arena keeps you on the ground almost
entirely."* No existing arena, creature, move set or hazard is to be adapted
into this fight: the arena, the creature's moves and the way it pushes you up
are designed new, from §2 and §6. Tuning an existing fight toward this would
inherit the ground-bound shape that is the problem.

What can be used is plumbing only, the formats things are written in rather
than anything written in them:

- **A creature is a table** ([species.md](species.md)) and **an arena is a
  table** ([arenas.md](arenas.md)): the Warden and its shaft are new entries,
  not changed copies of old ones.
- **Reading what a player just did** off two frames (`tutorial::Seen`,
  [tutorials.md](tutorials.md) §2), to know when a step of the climb or a
  click has been learned.
- **The scripted hunter** ([bestiary.md](bestiary.md) §8), to measure the
  fight once a person has found it good, never before
  ([courses.md](courses.md) §0: *"You build, I play."*).

Everything else is new: the arena and its perches, **perches that break** (a
solid removed, its rubble added), the creature and its moves, and **a kit
with pieces locked**, per player, in the opening only.

**The test, in one question:** *in the blockout, do players leave the floor
because they want to, rather than because they are told to?* If they do,
the identity holds and the rest of the opening follows from the fight. If
they do not, the blockout says what is missing, and much more cheaply than
a valley would.

## 8 · Open

Answer by ID.

- **OPENING-1: is "the monster owns the floor" what people enjoy in the
  movement?** It was agreed for the intro fight. Whether the pull of the
  movement is that, or speed, flow, or style, the blockout should show.
- **OPENING-2: one fight for every class, or one built around a starting
  class?** The draft is one fight, built from the shared moves, so that each
  class's movement is only a faster way up. This keeps [courses.md](courses.md)
  §0's rule that the main route is every class's.
- **OPENING-3: the Elementalist's first click.** Earth is her first click
  and also her climb. A stone with its takeoff locked may still be a step to
  stand on. Hand her fire first, or earth without its takeoff, and check the
  climb.
- **OPENING-4: the Warden's size.** About two Ridgebacks, the soft place
  10–12 m up, is the guess. It is set by play, against the shared moves of
  the five classes that move (`sim::class::MOVERS`).
- **OPENING-5: skipping the opening.** A returning player, a second player
  joining, someone who has played it on another machine. And how the opening
  is played by two people at once.
- **OPENING-6: how a piece of the kit is given.** Found, taken back,
  remembered, woken? It should be the world reacting, not a screen. The
  answer is part of the story.
- **OPENING-7: what is outside the tower, and what the valley becomes.** The
  owner does not want the valley as progression. What replaces it is not
  decided here; this document only covers the start.
