---
status: direction agreed 2026-11-04 ("this is definitely the right vein. Build it like you've described"); slice 1 built, unplayed
started: 2026-11-03
supersedes (when built): the Thorn lash, the Bramble, the vines, Thornfall, the Reap and the Harvest as bound, the Stake, the Upheaval, the teleport to a stain
keeps: grey and its fade, every cost being a stain, her ground, the swim, hearts and thralls (for now), Coagulate on `Q`, the Sweep and its string, the spike's telegraph
depends: ../kits/blood-mage.md, 0021 (territory), 0037 (the Dual mage on the razor's edge, on its own branch), the profiles below
---

# 0038 · The Blood mage: the world bleeds

## The brief (the owner, 2026-11-03)

> I need to finalize the blood mage's v1 kit. It's still a little clunky and the best strategy
> is still to spam spikes at the enemy's feet. The bloody ground mechanic works well, the
> hearts work fine. What needs an overhaul are the left and right clicks. They both feel
> unimpactful [...] We also need to overhaul all the aerials [...] why does my jump-lclick
> blink me in the direction I'm aiming when my lclick is an area dot spell? [...] as the game
> has drifted more towards hypermobility as a necessary component I want to make sure it also
> has tools that are interesting and fun to use for jump courses.

## What the finished classes share (profiled 2026-11-03)

The Reaver, the Elementalist and the Champion each have something that goes **up**, **along**,
**down** and **holds still** in the air; each piece spends a **different budget** (a takeoff a
trip, the airdodge, shrinking hangs, the class's object), so the order is free; the class's
**object exists in the air** (the shadow anywhere, a stone under her feet, the spear as a
pole); an air move is **the ground move in the air**; and distance is **paid in timing** (the
dash jump's last six frames, two stones two or three frames apart).

The Blood mage had none of it. Her object lived only on the floor, so every door brought her
down. In the movement search's fixtures her own kit adds **5%** to the longest gap she
crosses, against the Champion's 60% and the Elementalist's 30%. Spiking their feet won because
it did 100 against the Sweep's 45, landed exactly where the crosshair met the floor, and came
out 2.2 m wide in 16 frames -- about how far a walking fighter goes in that time.

## How the direction was found

The owner's tests, in the order they were applied:

1. **A move simply is, and how you use it is the intent.** The shadow, the stone and the spear
   are each movement, offence and defence by where and when. Thornfall and the old send-and-zip
   were added because they helped, and read as "why does it do that?"
2. **Health is not the fuel for movement.** "lclick then dash to the bloodstain to recover the
   lost health, and missing that can permanently lower your pool."
3. **The vanishing point.** The Dual mage clicked when its magic was aimed *at her* (0037): one
   fact, and every move follows from it. "Health cost can mean anything." What is the
   *inescapable* consequence of being a blood mage?
4. **Why blood, and not paint?** Blood that sets into any shape rules nothing out.

The answers, kept:

- **What only blood does: where it falls, something grows.** Cadmus sowed dragon's teeth and
  armed men came up; anemones from Adonis; the mandrake under the gallows; poppies in
  Flanders. The kit was three themes stapled together -- a reaper (the scythe, the Harvest), a
  gardener (thorns, bramble, vines) and a blood mage (stains, bleeds, hearts) -- and **sow,
  grow, reap** makes them one cycle.
- **The law (the owner, 2026-11-04): "it is not possible to change without bleeding.
  Everything they change leaves its pooled essence to be manipulated."** Whatever she cuts,
  strikes, moves or breaks bleeds: a fighter, a creature, the floor, a cliff. This is what
  frees her movement: there is always something to cut, so mobility is never gated behind
  hurting an enemy or herself.
- **The owner's Bloodwright** (a 5e class of theirs) showed what the game's version lacked:
  one currency, chosen in the moment, paid and paid off together. Its Rite of the Red Hand --
  "your blood hardens into armor and lashes out at those who harm you. You dance with death" --
  is this class's paradigm.
- **A brawler, not a summoner** (the owner): "a farmer sits behind tending plants." She fights
  inside her patch, because the fight is what waters it: every drop spilled there, hers
  included, feeds what grows. "Fight when there's blood in the streets, even if it's your own."
- **The heal stays a refund.** Lifesteal is among the strongest tools in any versus game. Every
  cast still costs her health and turns it grey; what she grew holds it; reaping gives back
  what she put in, never more than her grey. Unretrieved, it withers; until it is retrieved,
  her bar is short.

## The kit

### Right: the scythe -- reap, sow and movement in one button (the owner's design)

- **Tapped: the Sweep**, strung to three as today. It cuts what it touches: a body, and the
  floor in front of her, which bleeds a semicircle -- a pool to raise.
- **Held: the cast.** The head flies out on its chain along the crosshair (a skillshot) and
  **embeds** in whatever it meets.
  - A **fighter** bleeds (today's Haemorrhage bleed), is **slowed**, and is **leashed**: the
    chain's length is fixed when it bites, and neither of them can get further apart than
    that. A leash, not a winch -- neither drags the other.
  - A **creature, a wall, the floor** bleeds where it bit: a pool.
  - **Left alone, it wrenches free** after `Cast, holds for`. A fighter gets a short stagger and
    a shove toward her, not all the way. The head **drags home** along the floor, **reaping
    everything on its line**: her stains are drunk (the refund), her plants burst.
  - **Pressed again while it holds: the zip.** She is hauled to the head, never the other way.
    She keeps `Zip, keeps of its speed` on arrival, with a jump window on the last frames as the
    Reaver's dash jump has, so it starts a chain rather than ending one. Onto a creature, onto a
    ledge, onto a slowed fighter.
- **The same in the air**, exactly. Space + right is the Rising reap: the cut she leaves the
  floor through, which bleeds under her.

### Left: the Red Hand -- what she does when she gets there

Her other hand, gloved in her own blood clotted hard. A scythe is a long weapon: a body inside
its reach is too close to sweep. The hand is what she fights with inside it.

- **Tapped: Rend.** A fast string of claw strikes, two or three, about 1.5 m, about 5 frames
  out: the fastest thing she has, so she wins the trade at arm's length. Every hit pools.
- **Held: Seize.** A grab. She catches hold and **feeds them to her garden**: near a plant she
  throws them onto it (impaled, for the plant's size, and the plant drinks); with none near
  she slams them into the floor, which bleeds in a ring round them. A grab beats a block and
  loses to a strike, which is what makes her a brawler: a player who blocks her scythe and her
  claws is thrown into the thorns.
- **The same in the air**: the string, and an air seize that drives them down onto what is
  below. **On a creature**, the seize holds on and she rides it.

### Middle: the spike -- grow

- **Raised only from a pool.** On bare ground nothing happens, because nothing there bled.
  That ends spiking their feet: something has to have changed there first.
- **Small grows fast, big grows slow.** A footprint or a cut in stone rises in about 8 frames,
  a fighter's blood in 16 (just past reaction time), a creature's in 20.
- It launches whatever is over it -- **her too**: the scythe into the floor, then the spike, is
  her way up.
- It does **less damage than today** and **stands** afterwards as a plant: solid, cover, a
  foothold, and on its own it **lashes** anybody within about 2 m (light damage, a slow), so
  nobody wants to fight beside one.
- **Held to its end (later): Bloom**, every plant in reach acting at once.

### Reap

The scythe through a standing plant breaks it, and its thorns fly the way the blade was going,
a cone about 8 m, worth what the plant held. Raise is slow, readable and hits where they stand;
reap is fast and hits along lines away from her plants. Together they are a mix-up: step off
the pool to dodge the raise, and you have stepped where the stalk is aimed.

### The garden (slice 2)

One plant that matures by **drinking blood spilled within a few metres of it** -- any blood,
hers included -- so tending is fighting beside it:

| Stage | Reached by | On its own | Reaped |
| --- | --- | --- | --- |
| Spike | raised from a pool | stands, lashes at ~2 m | a small cone |
| Bramble | a spike that has drunk enough | a spreading patch of thorns that slows | a bigger burst |
| Heart-tree | a bramble that has drunk a lot; one at a time | today's heart: holds her ground, sends thralls, a canopy to stand on | everything it held |

Unfed, a plant drains, and the grey in it with it. Her ground is the roots: what grows spreads
it, and it joins as today.

### The loop (the owner's words: "the red hand paradigm")

**Cast the head -> raise the pool its bite left -> zip in -> rend or seize -> keep them in
close, so every drop spilled feeds the patch -> reap.**

In the air: launch them off a spike, cast up at them, zip, air seize, drive them down onto
the plant.

## What is cut, and why

| Cut | Why |
| --- | --- |
| Thornfall (space + left) | the blink the owner named: a dot spell's takeoff should not throw her along the aim |
| The Thorn lash and the Bramble on left | a bleed wants distance; a brawler wants to be in it. The bleed lives on in the cast's bite |
| The vines | plants, as a separate cast; the garden grows them instead |
| The Reap and the Harvest as bound | the hold is the cast now. A finisher on the scythe comes back if the slice wants one |
| The Stake and the Upheaval | the spike in the air and from the floor is the same raise |
| The teleport to a stain on `shift` | "it happens when I don't want to dash at all"; the zip replaces it, and you can see where it goes |

## Slices

1. **The loop, end to end** (built 2026-11-04, unplayed; [kits/blood-mage.md](../kits/blood-mage.md) §"The world bleeds"): the cast with the leash, the wrench and drag home,
   and the zip; the Sweep wounding the floor; the Red Hand's Rend and Seize; the spike only from
   pools, standing as a plant that lashes, launches her, and is reaped by the scythe; the
   bindings above. Tests for each rule; the frametable; the movement search re-run for her.
2. **The garden's stages** and the hearts folded into the heart-tree.
3. **`Q` and `E`** (the owner: later), Bloom, polish and animation.

## Round three: the owner's tweaks, 2026-11-07 -- the promise card

The owner, after playing the rebuild: "This is better. A few tweaks." Every line below is a
promise with the number it must meet, written before the code (delivery.md D7), and each is a
line of `cargo run --release -p sim --example kitcheck`.

**The scythe head is a physical thing** ("it feels like a magic skillshot")

| Promise | Measured by |
| --- | --- |
| The head leaves her hand as the blade itself: the scythe she holds has no blade while it is out | drawn: filmed |
| It flies under gravity: thrown at the crosshair, it arcs, and a throw at nothing comes down short of where a straight line would go | the head's height drops over its flight; a level throw at nothing lands on the floor |
| Aimed at something within reach, it still meets the crosshair: the throw's speed is solved for the arc, so grabbing people stays easy | a Bulwark 10 m off, crosshair on his middle: bitten |
| The chain goes slack when the head falls without being pulled in, and taut when it holds or is pulled | drawn: filmed (sag from the chain's length against the gap) |
| Called home, it is reeled: it starts slow and speeds up, not a constant glide | the head's speed rises over the reel |
| The zip is a pull, not a glide: she is accelerated along the chain, keeps her momentum and falls under gravity as she goes | her speed rises over the pull; a zip up to a high bite arcs |

**Left click while the head is out: the chain in her other hand** (the owner's answers, 2026-11-07)

| Promise | Measured by |
| --- | --- |
| A tap of left pulls her to what the head bit | she ends beside the bitten fighter |
| Left held to its end reels them in to her | a fighter 10 m off ends within 2 m of her |
| Left let go early while the head is in them rips it out: a hit, a bleed and a stagger | damage dealt, stagger, the head comes home |
| Right pressed while the head is out calls it home | the head starts back on the press |
| **The shrike**: a body reeled or knocked into one of her plants is impaled on it -- held on its thorns a moment, bleeding, and the plant drinks | the reeled fighter held at the plant |

**The Red Hand**

| Promise | Measured by |
| --- | --- |
| Three taps are three different blows: a quick swipe, a cross-body backhand, and a heavy two-handed rake that knocks them down | three different rows, each bigger than the last |
| Held, the claw charges: the longer the hold, the further out the swipe lands and the bigger it is | the swipe's distance and size rise with the hold |
| Over-holding is punished: the swipe lands at the held distance, so held too long it lands past them | a fighter 3 m off is missed by a full hold |
| The charged claw slows what it hits | slowed after the hit |
| Its claw marks are longer and more solid than the tap's | drawn: filmed |
| Raking one of her plants feeds it: bigger, and its lash longer and harder, for a while | the plant's size and lash reach rise |
| Space and left held is a charge-jump: the longer the hold, the further she leaps, low, with the claw riding in front of her the whole way (built: "higher" went over the heads of what was in her path, and the owner's ask was "further") | the leap's distance rises with the hold; a fighter in her path is hit mid-leap |

**Plants**

| Promise | Measured by |
| --- | --- |
| Jumping off a plant does not burst it; only the scythe through it does | a jump off a plant's top leaves it standing |
| Plants look grown, not turned: a gnarled stem, branches and thorns, no cylinder | drawn: filmed |

## Open

- Does a refund capped at grey, with enemy blood growing plants but never healing her, keep
  the risk the owner wants, or should enemy blood refund a little?
- Should Seize be unblockable, as the old Grasp was, or lose to a timed dodge too?
- What a spike does on its own, and to what range.
- Can anybody break a plant, or only a heavy hit?
