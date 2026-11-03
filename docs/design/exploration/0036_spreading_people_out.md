# 0036 · Spreading people out

The owner, 2026-11-02, after the hill maps were rebuilt bigger
([fight-spaces.md](../fight-spaces.md)):

> Let's figure out ways to spread people out. Other objectives, different
> game modes, obvious points of strength high up and spread out aimed down at
> the control point, different sections of a map with long enough traversal
> to point so a team can push toward the other's spawn. TF2 maps are
> interesting, but we have to figure out how to spread people out in this
> game where there's a lot of melee and ranged means mid range. There was
> some game about chaining elements together to get bigger effects that was
> really beloved. I never played but they seem to have figured something
> out.

This is a working note: where the clumping comes from, what other games do
about it, which of it survives a game whose fighting happens inside 25 m,
and what to build first. Nothing here is built.

## 1 · Why people clump, from first principles

A player goes where their next second is worth most. They stand in the
middle because every second there is worth something, and every second
anywhere else is worth nothing. Bigger maps fix the *room*. They do not fix
the *worth*. Three things set the worth of a place:

1. **Where the score is.** King of the Hill has one place that scores, so
   every route leads to it, and every fight ends up on it.
2. **What fighting from there is like.** A shooter spreads people out by
   *range*. A sniper is worth most far away, so somebody stands far away.
   Here, nearly everything lands inside a reach (25 m), and melee inside
   four metres. So standing back is not "fighting from range". It is "not
   fighting". There is no long-range role to pull anybody out of the
   middle. **Range will not spread this game out. Objectives and terrain
   have to.**
3. **What being together buys.** Two allies on one target is focus fire.
   Nothing in the game today makes standing next to a team-mate cost
   anything. On a hill, a side cannot even hurt its own.

So the levers are: **more than one place that scores** (1); **places worth
holding that are not the score** (1 and 2); and **a cost to clumping** (3).

## 2 · What other games do, and what survives here

| Idea | Where it comes from | How it spreads people | Survives a 25 m game? |
| --- | --- | --- | --- |
| **A moving hill** (the point jumps to a new site every minute or so, announced a few seconds early) | Halo's moving King of the Hill; Call of Duty's Hardpoint | Nobody can camp the point; the team that reads the *next* site and gets there first wins it. The fight moves across the whole map | **Yes, and cheapest of all.** The maps are now big enough to hold four or five sites each |
| **Several points at once** (each one held scores) | Battlefield's Conquest; Overwatch and TF2's Domination maps | Two players cannot hold three places. A 2v2 with three points is a set of 1v1s and rotations | **Yes.** It fits melee: each point is a short-range brawl, and the decision is *which* brawl |
| **A front that moves** (points in a row between the spawns; taking one moves your spawn forward) | TF2's five-point maps and payload; Overwatch's Push | The map is sections. A team pushes toward the other's spawn, and the fight is always at the front, not in one middle | **Yes**, but it is a new kind of map (long and laned), not a revision of an old one |
| **Strongpoints that do something to the point** (a tower that captures, or slows the enemy's capture, from up high) | Many games' "capture from range" beacons; TF2's high ground made mechanical | The point is held from *two* places, the disc and the perch, so a team has to split, and the attackers have a reason to go up | **Yes**, and it is exactly the owner's "points of strength high up aimed down at the control point". Without a mechanic, a high perch in a 25 m game is only a view |
| **Item timing** (big health or power-ups on timers in far corners) | Quake and Unreal arena play | The far corners are worth a trip at known times, and controlling the clock is a skill | **Yes.** It costs nothing in range, and every class has health worth going out of the way for |
| **Long-range classes** | TF2's Sniper and Engineer | People stand far back to do their job | **No.** The roster has no long-range role and is not meant to (README §1: a missing long-range poke is a design choice) |
| **Hurting your own** | Magicka (friendly fire is total); Divinity: Original Sin's surfaces burn friend and foe | Clumped allies set each other alight, so everybody learns to stand apart | **Partly.** Full friendly fire would turn melee chaos into a joke. A *share* of splash on your own side might be right |

## 3 · The element-chaining games

The owner remembers "some game about chaining elements together to get
bigger effects that was really beloved". That is most likely **Magicka**
(2011): you cast by typing up to five of eight elements, and they combine
(water and cold make ice, fire and earth a fire boulder, water and
lightning electrify whoever is wet). Or it is **Divinity: Original Sin 2**,
which is loved for its *surfaces*: rain leaves water on the floor, a
lightning bolt into the water stuns everyone standing in it, fire on oil
explodes, poison on fire explodes. Both are worth a look, for different
reasons.

**What they figured out, as it applies here:**

1. **The ground remembers.** In Original Sin a fight leaves its spells
   lying on the floor as surfaces, and the next spell reads what is there.
   So *where* you fight matters more than *how*, and the map changes as the
   fight goes. This game already has the start of it: the Elementalist's
   stones and fire, the Blood mage's ground, the hazards the creatures lay
   (tar, smoke, ash). What it lacks is the **grammar between them**: one
   class's surface changing what another class's move does.
2. **A combo is a reason to be somewhere else.** If an Elementalist's
   wind blowing *through* a Blood mage's stain spreads it, or a Champion's
   shockwave *into* the Elementalist's fire throws burning stones, then a
   team does best with one player **setting** and another **firing from
   another angle**. That is a pincer, not a clump. The most useful version
   of the idea for this note: **combos that need two angles**.
3. **Clumping is punished by the same rules that reward combos.** Stand on
   the same electrified puddle as your friend and you are both stunned.
   Area effects that hit more the more people stand in them, a share of
   which lands on your own side, are the one lever in §1 that works
   *inside* the 25 m, in the middle of a fight, without a map change.
4. **Few elements, many results.** Magicka's eight elements make hundreds
   of spells because each element is a *property* (wet, cold, burning,
   charged) that others react to, not a spell. A kit built that way stays
   small to learn and deep to master, which is the game's intent already
   (the Elementalist's "small algebra of verbs", exploration
   [0002](0002_elementalist_full_kit.md)).

This is its own thread and the biggest of these ideas. It touches every
kit. It is listed last in §5 for that reason, not because it matters
least.

## 4 · Mid range and melee: what a spread-out fight looks like here

In a shooter, spread out means "far apart and shooting". Here it should mean
**several short fights at once, and choices about which one to join**:

- **Distance between places that score should be more than a fight's
  worth of walking.** A versus fight lasts about a minute and a run is
  7 m/s, so two objectives 60–90 m apart (2.5–3.5 reaches; the biggest maps
  are 190 m long) are ten seconds apart. Ten seconds is long enough that
  rotating is a decision, and short enough that it is a good one.
- **The high ground does something to the point, not only at it.** A
  perch whose holder captures the point slowly (or stops the enemy's
  capture) makes the perch an objective. Holding it is then a fight of its
  own, eight to fifteen metres up, in reach of the point and of the
  players on it.
- **Every objective is a melee room.** A point is a disc four metres
  across. A beacon is a ledge you can be knocked off. A health shrine is in
  a dead end. The brawl is where the class design lives, so the map's job is
  to have *several* brawls, not one long gunfight.

## 5 · What to build, in order

Each is a rule in `sim::versus` and a little data on maps that exist. The
first three need no new maps.

1. **A moving hill** (Hardpoint). Each hill map names four or five sites
   (`Hill`s); the point moves to the next one every 60 s, and the next
   site lights up 10 s early. The scoring is King of the Hill's as it is.
   *Cheapest, and it uses all the new room.* A test checks that every site
   is reached from both starts by the ways the map promises.
2. **Beacons: strongpoints that capture.** Each hill map marks one or two
   perches per side (already there: the Quarry's upper bench, the Delta's
   aqueduct, the Henge's giants). A side alone on its beacon captures the
   point at a third of a body's rate, and stops the enemy's progress
   draining their own. The point stays the main thing; the perch is the
   thing you have to leave the point to take.
3. **Two or three points at once** (Domination). On the same maps: the
   point and the two beacons' bases become three discs, each held one
   scores a tick. Works from 1v1 to 4v4.
4. **Health on the clock.** A shrine in each far corner that heals once a
   minute. One line of data a map, and a reason to be out of reach.
5. **A push map.** A long map, two hundred metres or so (fixed point is
   fine to 181 m between any two things that measure each other, so a run
   of sections, not one open field), three points in a row; taking one
   moves your spawn up. Built from the fight-space rules, one section at a
   time.
6. **A share of splash on your own side, and surfaces that combine across
   kits.** Its own exploration: which properties exist (wet, burning,
   charged, bleeding, cursed), which move leaves which, which reads which.
   Played first as a single cross-class pair (Elementalist fire and the
   Champion's Shockwave, say) to see whether it pulls a team apart the
   way the games above do.

## 6 · Open questions for the owner

- **Which first?** The recommendation is 1 then 2: both are small, both
  run on the fifteen maps that now exist, and between them they test
  whether moving the *score* is enough without moving anybody's *kit*.
- **How many players does a typical game have?** Two points suit 2v2;
  three suit 3v3 and 4v4. The room is up to four now (versus.md §6).
- **Friendly splash at all?** The rule "a side cannot hurt its own" is
  settled for hills today. A share of splash would undo it on purpose.
