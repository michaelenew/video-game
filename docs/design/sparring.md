---
status: built 2026-09-26, unplayed; measured kits, five levels and the menu's Difficulty 2026-10-20
---

# The sparring bot

Somebody to fight when there is nobody to fight. Player two, on keys **5**, **6**
and **7** (easy, normal, hard), in the Esc menu's **Training dummy** section --
*Does: Sparring bot*, and a **Difficulty** row of all five levels, each with a
line saying what it is like -- or `--bot <level>` / `?bot=<level>` to start
against one. It lives in `crates/hunt/src/duel.rs`, beside the creature hunter,
and `cargo run -p hunt --bin duel` plays it against itself.

The brief was three things: **reasonable decisions, some unpredictability, and
imperfect reflexes.** Each of them is a mechanism you can point at.

## Imperfect reflexes: it sees late, and its hands are human

**It sees through a delay line.** Everything it knows about the other fighter
comes out of a ring of remembered frames, read some number of frames back. That
number *wanders* between two bounds — a frame up or down, now and then — so it
is sharp for a while and then slow for a while, which is what attention does. A
move whose startup is shorter than the delay cannot be answered on sight, and
it will not be. `tests/duel.rs::it_sees_late` holds that it never sees the
present at any level.

| Level | Sees | Notices a move | Takes an opening | Aim error | Timing slop |
| --- | --- | --- | --- | --- | --- |
| Beginner | 30–45 frames late | 30% | 15% | ±18° | ±9 frames |
| Easy | 24–36 | 45% | 30% | ±11° | ±6 |
| Normal | 18–27 | 70% | 60% | ±5° | ±4 |
| Hard | 15–21 | 88% | 85% | ±2.5° | ±2 |
| Expert | 15–18 | 95% | 95% | ±1.5° | ±1 |

Beginner never leads a moving target; Expert nearly always does, and thinks
every two to five frames. `the_levels_climb` holds that each level is at least
as good as the one below in every number, and that none is under the floor.

Fifteen frames is the floor for a reason: it is about as fast as a person
reacts to something they are already waiting for. Below it the bot would be
answering moves nobody can, and losing to it would teach nothing.

**Its hands are imperfect.** The mouse turns at a finite rate, so a bot facing
the wrong way has to come round before it swings — and if it runs out of
patience it swings anyway. Every shot gets an aim error drawn fresh, and only
some shots lead a moving target. Every timed press — a dodge, a jump, a duck —
lands a few frames early or late.

**It aims the way a player does.** Where to put the mouse for a skillshot is
`aim::look_onto`: the camera's ray run backwards, so the bot's crosshair and
the game's ray are one line. See [aiming.md](aiming.md). It aims at the floor
under you, because a skillshot that meets the floor is raised to the middle of
whoever stands there.

**It knows its own kit by pressing buttons.** Which button throws which move
depends on the class and on grammar that has moved three times this month
([controls.md](controls.md)). Rather than keep a second copy of that grammar to
go stale, the bot does what a player does when they pick a class up: it presses
each button in an empty arena and watches what comes out (`duel::Kit::learn`).

**And then it tries each one on somebody** (`duel::Kit::measure`, 2026-10-20,
printed by `duel --kits`): every button thrown the way the bot throws it, at a
fighter standing 1.5, 3, 6, 10 and 15 m away, and the health it took counted.
That -- not the move table's damage -- is what it weighs its moves by, and how
far a move hurt is part of its reach. The table's number is the swing's own, and
three kinds of move hurt by something else: a thrown thing (the Shuriken, the
Thorn lash) has none in the table and was never thrown; **the Elementalist's
two pushes have the stone's damage in the table and go straight through a
body**, so she threw them all fight at people they could not touch; and the
Dual mage's clicks start as their long hold and become the auto on the
release, so the kit had her autos down as the long holds. A move that measured
at nothing is not thrown at you, not strung into and not dodged, unless its
class has a reason for it. Measured once a class for the whole program -- a
few thousand frames, tens of milliseconds -- so a class switched mid-fight does
not hitch the game.

## Reasonable decisions: plans, openings, answers

**It commits to a plan for a while**, as a person does, and re-decides when it
runs out or when it gets hit:

| Plan | What it does |
| --- | --- |
| Press | Walks in and swings -- to the end of **its own** close reach, if that outreaches yours (the Dual mage's ball, sweet at six metres, against a sword), rather than on into yours. |
| Footsies | Dances in and out at the edge of your reach, swinging at whatever steps into its own. |
| Circle | Strafes round you at a distance. |
| Retreat | Backs off — when hurt, or when well ahead. |
| Zone | Keeps its distance and throws its longest-reaching skillshot. Only if it has one. |
| DashIn | Walks to dodging distance, dodges in, swings out of it. |
| JumpIn | Jumps at you and swings on the way down. |
| Bait | Walks into your reach and straight back out. |
| Wait | Stands and watches. |

**It takes openings it can see.** A recovery, the vulnerable tail of a dodge, a
stagger: if it can reach you with something quick enough to land inside what is
left of it — worked out in the present, not in what it saw — it goes, most of
the time.

**It answers moves it sees coming**, if it notices them, and if they are pointed
at it and long enough to reach. The answer is chosen by weight, not rule: dodge
sideways or away, duck an overhead, guard if it has a shield and the move is
blockable, walk out if there is time, jump, swing first if its move is out
before yours — or freeze, which is what people do most. Bolts in flight get a
sidestep, sometimes.

**Strings are practised, not reacted to.** Some of the time a second press is
already on its way when the first swing comes out, whether or not it hit — which
is a Champion chain if it did and a mashed button if it did not.

## Unpredictability: a personality, a mood, and a memory

**Personality.** Five traits — aggression, patience, caution, jumpiness and
zoning — drawn from the seed, 20 to 80 each, and drifting by up to fifteen every
round. In game the seed is the clock, so pressing 5, 6 or 7 again is a new
opponent.

**Mood.** Landing a hit makes it confident; taking one makes it careful — unless
it is the aggressive sort, in which case it gets angry. Mood decays back to
nothing and feeds straight into how it picks plans and how readily it swings.

**Memory of itself.** It remembers its last four plans and marks down any it
has just been doing, so it does not repeat itself into being read.

**Memory of you.** It counts how often it sees you start moves. Against
somebody who throws a lot it baits and plays footsies; against somebody who
throws nothing it presses.

## Its class

Since the second day it plays its class, not just its buttons —
`crates/hunt/src/duel/mechanics.rs`. Two hooks per class: one that may start a
**gesture** each frame (an aimed press of `E`, of right click, of `Q` or of a
dodge, made with the same slow mouse and the same aim error as an attack), and
one that weighs its moves by the state of the mechanic.

| Class | What it does with it |
| --- | --- |
| Champion | **Chains on a hit it felt** — its own swing connecting is proprioception, not sight, so it holds the next weapon through the recovery and the link comes out when the window opens. **Rush-cancels a whiff or a block** out of the recovery, away or past, never toward a wall. Rushes in from mid range and swings out of it — with the sword or the hammer, and looking level, because the spear looking down out of a Rush is the Pole vault. |
| Bulwark | **Guards at the edge of your reach** to load the shield, and weighs Slam by how full it is: seven times as likely at nine tenths. Grapples you when you guard. **Throws** from five to nine metres, sometimes **leaps after it**, and **recalls** it the moment you are standing on the way home, or before long anyway, since he cannot guard without it. |
| Shadow Reaver | **Sends the shadow** at you (never as a poke). With it beside you: **the lotus**, then a recall that drags the blades through you; or, with two marks on you, **dashes to it and slashes** out of the carry. Recalls a shadow left somewhere useless. Swings harder for her own body with three marks up. |
| Elementalist | Her buttons since 2026-10-09: left click is earth (Raise, Landfall in the air), middle the pillar, `Q` and `E` the pushes -- the bot pressed `E` for Raise until 2026-10-20. **Raises a stone under your feet**, led by the fourteen frames it takes to rise. **Raises one in the lane and pushes it into you** -- the Bolt mostly, Cataclysm sometimes -- and pushes whenever a stone of hers is on the line, or shoots the Bolt through her own fire, which comes out of it a fire bolt (what the creature hunter does with a pillar under the work: without it her hunts do half the damage); never otherwise, since a push goes through a body. Puts one **in front of her** when you are on top of her and she is backing off. **Rides one up** out of trouble, jump held, from at least eight metres inside the walls (a stone's jump took her over them from four). Landfall from above you. The pillar is most of her damage. `the_elementalist_pushes_stones`. |
| Blood mage | **Blinks to a pool beside you and sweeps** (the scythe, right click), or to one far from you when she is low. Weighs Black spike (middle click) by whether a pool is at your feet, the scythe by her grey, and her costly moves down when her red is short. |
| Dual mage | **Keeps her bars level.** A move that would push the gap past the band — where she burns — is never thrown; one on the low side is three times as likely; lopsided, she walks in whatever her plan, because the autos that mend it are thrown up close. **No ascension from behind** — it costs a great deal of health. The **second jump** off the tier, and all-in while the wings are out. **Both majors, back to back**, with her lower bar past the jump tier and you inside Judgement's reach: the Abyss spends the dark bar and Judgement the light, so the pair leaves her level where either alone would burn her. |

The creature hunter plays its class the same way since 2026-10-01
(`crates/hunt/src/class.rs`, [bestiary.md](bestiary.md) §8): it learns its
buttons with `duel::Kit::learn` and keeps the Dual mage's bars with the rule
above (`duel::dual_bias`), and refuses her ascension besides -- a hunt cannot
pay for six seconds of wings.

## Measured

`cargo run -p hunt --bin duel` plays every pairing for three minutes;
`--level hard --against easy` sets the two sides; `--trace` prints the fight
twice a second; the last column counts deliberate uses of the mechanic. On the
second day:

- Every class fights its mirror: both sides throw 75–200 moves in three minutes,
  land 30–120 of them and finish rounds. `tests/duel.rs::every_class_fights`
  holds it, and that nobody spends ten seconds outside the walls.
- Every class uses its mechanic at least ten times in three minutes —
  the Reaver over a hundred, the Blood mage least, about fifteen, because her
  pools have to be in the right place. `every_class_uses_its_mechanic`.
- The Dual mage stays inside her band four fifths of the time and gets high
  enough to blink. `the_dual_mage_keeps_her_balance`.
- Hard beats easy across every pairing, and easy still wins some.
  `harder_is_better`. No plan takes half its time. `it_mixes_it_up`.
- **The uneven pairings are lopsided**: the Reaver and the Elementalist win
  nearly everything against the Blood mage and the Dual mage, and the Reaver
  beats the Bulwark and the Champion. The Reaver jumped once she started
  using her shadow. That is either the bot playing some classes better than
  others, or those classes being stronger; the mirrors say it is not simply
  broken, and which it is needs a person.

On 2026-10-20, every pairing at normal, three minutes, seed 1 -- the
Elementalist's rounds against each class (hers–theirs), before and after the
measured kit, her buttons, the majors and pressing to your own reach:

| Against | before | after |
| --- | --- | --- |
| Bulwark | 1–4 | 7–0 |
| Champion | 4–1 | 7–2 |
| Shadow Reaver | 2–8 | 7–4 |
| Blood mage | 0–14 | 7–4 |
| Dual mage | 2–2 | 10–0 |
| Her throws that land | about 1 in 8 | about 1 in 2 |

From losing most of her fights she now wins most of them, nearly all of it
with the pillar (565 health measured on a standing body, the most of any
button in the game) and stones pushed into people. Whether that is the bot or
the pillar is for a person to say.

The Dual mage still loses nearly every pairing. She lands plenty, but her
autos are fourteen a hit against swords of sixty and seventy, and her bars
seldom get deep enough for her spells to grow: that is the class's numbers at
low bars as much as the bot, and is for a person to judge. Over six seeds the
levels climb by a step each: hard takes 59% of rounds off normal, normal 62%
off easy, and expert 54% off hard -- the least, since both are at the reaction
floor and what is left is how it thinks, which no level changes.

## Found by building it

**The arena walls are lower than a jump.** They are a metre and a half, and the
shortest full hop in the roster is 2.7 m. The first long run spent two minutes
of a three-minute fight with one bot walking round the *outside* of the arena
after a jump beside the wall carried it over. A person can do the same. The bot
now keeps its jumps four metres inside the walls, and hops back over if it is
ever on the wrong side — but whether the arena should let anyone out is an open
question for [README.md](README.md) §4, not something a bot should decide.

## Not built

- **The creature.** In a hunt, player two sits out as the dummy always has; the
  hunter (`hunt::Hunter`) is the bot for that fight.
- **The deeper tech.** No double structure jump, no throw-leap-Slam as one
  deliberate string, no Earthbreaker leap, no Grasp onto a pool, no drinking
  on purpose. Each is a thing a strong player of that class does and none is
  needed to make it that class.
- **Aerials beyond the jump-in.** It swings in the air only on the way down from
  a jump it chose.
- **Online.** It drives player two in local play and the browser. It could
  drive a peer — it produces inputs, and inputs are what cross the wire — but
  nothing is plumbed.
