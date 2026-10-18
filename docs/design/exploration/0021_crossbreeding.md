---
status: the builder, the crossing and the rearing built 2026-10-18, as `sim::breed`; nothing in the world yet
started: 2026-10-18
---

# Crossbreeding — a creature as a genome

> **The owner's brief, 2026-10-18.** Monsters drop trophies (feathers,
> beaks, horns). Eggs are found and taken; an egg is modified as it grows by
> giving it trophies, which move some of the egg's properties toward the
> creature the trophy was won from. Body plan elements are *majors* (limb
> count, torso position, what the limbs are for) and move slowly; what is
> at the end of a limb, horns, the move set's parts, the sliders are
> *minors* and move more easily. Mutations are normally distributed on the
> sliders; on majors they are rare. It should be possible to give a
> Ridgeback flight, but not quickly: its front limbs have to become wings,
> its bones lighten, the wings grow, over generations. Making a creature
> bigger is breeding the biggest of its kind. The egg is incubated, the
> hatchling fed and follows you, grows, rebels at full size, and is tamed
> by beating it. The core is a deterministic creature builder, whose litmus
> test is rebuilding the current cast.

This note is what was built against that brief, what it decided, what it
measured, and the questions left for the owner.

## 1 · What a creature is now

A creature is a [`Genome`](../../../crates/sim/src/breed/mod.rs): a handful of
discrete choices and a row of numbers. Everything else -- the skeleton in
metres, the boxes, which limbs are legs, what it can do, what its moves cost
and deal -- is **derived**, deterministically, by the builder
([`build.rs`](../../../crates/sim/src/breed/build.rs)). Two machines with the
same genome build the same animal; a rollback rebuilds it bit for bit.

Three kinds of gene, moving at three speeds:

| Kind | Genes | How it moves |
| --- | --- | --- |
| **Major** -- the body plan | the **stance** (level on legs; upright on the hind pair with the fore free; low-slung and splayed; limbless); **where the head is** (forward, raised, low, merged into the body); the **segments** (a thorax, an abdomen, a tail and how it is carried); up to four **limb pairs**, each at a *station* along the body (fore, mid-fore, mid-hind, hind), each with a **role** (walk, strike, both, wing) and a **build** (column, sprung, raptorial, spindle, stub, wing) | A trophy flips one with a chance of **one in eight** at full pull; a mutation with **one in four hundred**. A limb pair the egg lacks is added at one in sixteen; a trophy never removes one |
| **Minor** -- grafts onto any plan | the **jaw** (grinder, teeth, fangs, beak, mandibles, maw -- which also sets the diet), the **crown** (horns, crest, feelers), the **tail tip** (club, blade, sting), the **hide** (skin, fur, plates, shell, chitin, feathers, slick), what is on the **back** (a ridge of spines, sacs, a burden), and each limb's **tip** (pad, hoof, claw, talon, blade, feather) | Flips to the donor's at **two in five** at full pull; mutates at one in two hundred |
| **Slider** -- the magnitudes, nought to one | **size**, **mass** (bone density at a size), **pitch** (how upright the torso is), **speed**, **reaction**, **aggression**, **tempo** (quick moves over heavy ones), **guard**; and per limb pair its **length** (as a share of the body) and **girth** | Every crossing moves each a random share toward the donor, scaled by the trophy's pull and the slider's **inertia**: size, mass and pitch a third, speed and reaction two thirds, the disposition fully. A limb's proportions are bone and take the heavy inertia. Mutation is a normal nudge, standard deviation three hundredths times the inertia |

The one that needed working out was the owner's own doubt in the brief:
*"a ridgeback torso, though it does not ordinarily connect to a thorax,
actually does have a preference on the kind of thorax it connects to."* It
does, and so does every major. That is §3.

## 2 · The builder

The genome goes in; a body and a move set come out
([`build::body`](../../../crates/sim/src/breed/build.rs),
[`build::moves`](../../../crates/sim/src/breed/build.rs)). The body is in the
same terms the species tables are written in -- `beast::Bone`s at rest
offsets in metres, `beast::Part` boxes, `beast::Leg`s -- so a built genome
can be laid beside a hand-written creature, and `tests/breed.rs` does:

- **Length** is `0.4 m × 2^(6.5 × size)`: a gnawer at a quarter of the way
  is a metre, the Ridgeback at 0.78 is thirteen, the Siegeshell at 0.94
  twenty-eight. Doubling along the slider rather than adding, because a
  tenth of the slider should mean the same thing to a rat as to a hill.
- **Hips** come off the longest grounded pair: a pillar stands nearly
  straight (nineteen twentieths of its length), a sprung leg bent, a
  spindle mostly sideways, and a low-slung body sits on its splayed legs
  plus half its own depth. Every creature with a table lands within a
  quarter of its table's hip height; the Ridgeback within two percent (4.86 m
  against 4.95), the cats within a hand.
- **Width** is a share of length that grows with mass, four times as wide
  low-slung (the Mireback is wider than it is long).
- **The torso** is a chain of three along the pitch direction (a quarter
  turn at pitch one), a thorax ahead and an abdomen behind if the plan has
  them; the head on a two-bone neck pointed up or down by its kind, or
  merged into the body; a three-bone tail carried level, drooping or
  raised; a limbless body is eleven segments in a line, five ahead of the
  root and five behind, the Sandmaw's column.
- **Limbs** hang from the segment their station names, half their length
  per bone, down for a column or a sprung leg, out then down for a spindle
  or a stub, up and forward for a raptorial pair, out for a wing -- which
  is half again as long as a leg of the same share, because the Galewing's
  wings reach nine metres on a body of five and a half.

The **move set** is what the plan can do, each move's numbers read off the
sliders. A jaw bites (a beak pecks, a maw also swallows, and a maw on a
low-slung body has a tongue); horns gore; a level body on walking legs that
is quick enough charges if it has horns or weight; a tail with a club sweeps,
with a blade spears, with a sting stings, and a small level body's plain
tail trips; a column pair that strikes stomps at the fore and kicks behind,
a sprung pair rakes, a raptorial pair scythes, lunges and parries, a spindle
stabs; a wing that lifts stoops, buffets, blasts a downwash, flings feathers
if feathered, makes talon passes if there are talons behind -- and a wing
that cannot lift only flares. A sprung body pounces and leaps; a level heavy
body rears and slams; a low-slung one belly-flops (or slams its sacs if it
has an abdomen full of them); a limbless one rises, breaches, spits and
lashes; anything big enough to be stood on shakes; anything with a body puts
its shoulder in. A ridge of spines sprays spikes (on a column body) or
flings quills (on a sprung one); sacs on an abdomen shoot web and screech,
on a body inflate; a burden shrugs, a shell sheds, a slick hide spews and
wallows; a guard above two fifths guards.

Each move's startup is its base, stretched by mass and size and trimmed by
tempo; damage grows with the square of size and with mass; appetite is the
aggression, bent toward the quick moves by a high tempo and the heavy ones by
a low one. The Ridgeback's bite starts in 25 frames for 163, a gnawer's in
11 for 30; the Siegeshell's plough in 94 for 451.

## 3 · Cohesion: every major wants something of the rest

This is the part the brief left open, and the decision that makes a cross a
creature rather than a pile of parts. Every major is also a **preference**
about the body it is in ([`plan::wants`](../../../crates/sim/src/breed/plan.rs)):

- A **level** torso wants a level pitch and, weakly, middling mass (cats and
  birds are level and light, the Ridgeback level and stone). An **upright**
  one wants pitch four fifths, light bones, a quick tempo. **Low-slung**
  wants weight and no hurry. The torso's say is the strongest -- weight one.
- A **column** limb wants a heavy, level, unhurried body; a **sprung** one a
  light, quick one; a **raptorial** pair an upright, light, quick body that
  guards; a **wing** a light, quick body. A striking pair wants aggression.
  A limb's say is half the torso's. The torso also says what it wants of a
  limb at each station -- heavy walking pillars a third of the body long on
  a level body, long thin fore limbs and short hind ones on an upright one,
  short splayed stubs low down (unless they are spindles, which reach out
  and come back down, so a spider's long legs sit it just as low) -- and
  nothing at all about a wing's length, which is the wing's own business.
- The **head** wants a little: raised wants a little pitch, low wants
  weight. A **hide** wants a little: feathers want light bones, plates and
  shell heavy slow ones, fur speed. Horns want aggression; a grinder wants
  calm.

After every crossing the sliders **settle** half the way toward the weighted
mean of everything the plan wants (a limb's proportions a sixth of the way),
times each slider's inertia. So the Ridgeback given a Mantis's arms: the arms
want the torso upright (weight a half), the torso wants level (weight one);
pitch settles a little up. The torso wants the arms heavy and walking, the
arms want to be long, thin and striking; they come out between, nearer the
torso. The brief's example -- *mid-long limbs, on the ground, for locomotion
but used in attacks, proportioned more like a mantis, with the torso tilted
a bit* -- is what `breedcheck -- mantis` prints.

Then the plan is **repaired** to the nearest one that stands up
([`plan::repair`](../../../crates/sim/src/breed/plan.rs)), each rule changing
the least it can, and every repair is named (`Fix`) so the game can later
say what the egg could not be: limbs on a limbless plan make it low-slung,
no limbs make it limbless; a wing behind the shoulders walks; a wing's role
and build agree, and a limb that has just become a wing is a stub of one; a
raptorial limb does not carry weight alone; an upright body with nothing
under its hips goes level or low; a level body on one pair that is not its
hind pair goes low; a tail tip needs a tail; a maw needs a body big enough to
swallow; a merged head does not stand upright. Two thousand random genomes
all stand after one repair, and a second repair changes none of them.

The **assessment** ([`plan::assess`](../../../crates/sim/src/breed/plan.rs))
is what the game will read off an egg: what it can do (fly, or has wings
that cannot lift it; charge; pounce; guard; burrow; be stood on; swallow), a
**plausibility** from nought to one (one less a penalty for every slider off
what the plan wants and every strain the plan carries), and a **tier** from
one to five, the bestiary's: mostly size, growing with its square, then
reaction and guard -- which is what makes a tier-four duellist of a
four-metre insect -- with a step for flight and for a maw.

**Flight** is a margin: the wing's length (as a share of the body) times one
and a half, plus lightness times four fifths, less size times three fifths,
less nine tenths. The Galewing clears it by about 0.9. A Ridgeback whose fore pair
has just become a wing -- a stub, half the length its leg was, on bones
still at 0.75 -- is at about minus a half, and has to lighten to under a
half and grow the wing past two thirds of its body before the margin turns.

## 3b · Posture: a legal plan also stands (2026-10-18, second pass)

The first sheets showed it: a Ridgeback whose fore pair had become wings
kept its hind pair where it was and stood with its weight out in front of
its only legs. Repair makes a plan *legal*; nothing made it *stand*.
[`posture.rs`](../../../crates/sim/src/breed/posture.rs) does, between the
repair and the builder, from one principle: **a body at rest is balanced
when its centre of mass sits over its feet.** Every part has a mass (its
box's volume times a density the mass slider sets -- a spider's abdomen
counts light for its size, a wing lighter still), every grounded pair has
feet, and when the mass is outside the feet's span the pass has the ways a
real animal has of putting it there:

- **shift** the limbs along the body (bring the hind pair forward under
  the weight), keeping their order and a natural spacing between pairs;
- **reach**: lean the limbs so the feet land ahead of or behind the hips,
  up to two fifths of the limb;
- **pitch** the torso up to a tenth of a turn, which moves the front mass
  back over the hind feet (a low-slung body hardly pitches at all);
- put the **wings down** and walk on the wrist, a pterosaur, when the
  weight is ahead of the legs -- rolled, so some such creatures do and
  some do not, and forced when nothing else can balance it;
- **drag**: with fore limbs and nothing behind them, the rear rests on
  the floor and lengthens into a rump that trails.

Which mix a creature uses is rolled once from its genome's fingerprint:
each continuous strategy takes a random share of the correction, and any
of them may sit a creature out, so two creatures with the same problem may
solve it differently and the same creature always stands the same way.
What a roll cannot reach, the limbs take regardless, within the torso.

Then the **vertical**: the root height, and the pitch if need be, at
which every grounded limb reaches the floor with a bend between a crouch
and nearly straight. Each build has a standing length (a column or a
sprung leg its whole length, a splayed one -- a spindle, a stub, anything
on a low-slung body -- its lower half, a walking wing its first two
segments) and a bend it likes (a pillar near straight, a spindle deep in
its splay); limbs attach at the bottom of the flank, where every creature
in the tables has them. A longer hind pair bends its knees more: that is a
**hopper**, recorded for the move set. A head is carried above the back
unless the head kind says low. The builder then places two-segment limbs
with the joint off the line -- elbow back on a fore limb, knee forward on
a hind one -- splayed limbs out then down, wings as three broad thin planes
swept back and rising with the chord narrowing to the tip, a walking wing
with its hand folded up along the body, and a dragging rump down to the
floor.

`tests/posture.rs`: every archetype stands as written (the bird's legs
come forward under its weight, nothing else moves), the same genome always
stands the same way, heads are above backs, the winged Ridgeback stands by
at least three different mixes across forty near-identical creatures with
some of them walking on their wings, fore limbs alone drag a rump that
touches the floor, long hind limbs make a hopper with its hind knees bent
more than its fore, the Galewing never walks on its wings, and two
thousand random repaired plans all stand with their feet on the floor (a
belly on the floor with legs too short to reach is the one case a limb
hangs, and it is flagged).

- **BREED-12 · A strategy's caps are the body's, not the stance's taste.**
  A level body may pitch a tenth of a turn and lean its legs two fifths of
  their length because that is what the limbs allow, not because it looks
  right; what looks right is the mean want the settle pulls toward. If a
  posture reads as strained it is the plausibility score's business to
  say so, which it does not yet.

### Legs against weight, wings that read, and standing easy (third pass)

Three more things the posed sheets asked for:

- **Leg strength** ([`plan::leg_strength`](../../../crates/sim/src/breed/plan.rs)).
  Bone carries load by its cross-section and weight grows with volume, so
  the rule is area against mass: the grounded pairs' cross-sections over
  the body's length cubed times its density, against the Ridgeback's
  ratio. The cast comes out between 0.9 (the Mantis) and 9 (the cats);
  a gnawer grown to a spider's size on a gnawer's legs is under a half.
  Under one, the settle thickens the legs by the root of what is missing
  (the torso's say, so a body that outgrows its legs grows them back over
  generations), the plausibility falls by three tenths of the shortfall,
  and under a half the creature is **weak-legged**: it cannot charge or
  pounce, and its posture rests its belly on the floor with the legs out
  as paddles -- which is its own support, so it is balanced by it.
- **Wings** are drawn by what they are made of (`build::WingLook`, from
  the hide): *feathered* keeps its chord to the tip and fans primaries
  behind it; a *membrane* tapers to a point along a leading-edge finger;
  a *furred* wing is a thick fringed plane, one plane per segment along
  its bone. A wing that walks is drawn as a wing's bone,
  thin, and its wrist lands clear ahead of the hind feet by a pair's
  spacing -- a wing is not a leg among the legs. The raptorial arm is a
  mantis's Z: up and forward from the shoulder, the blade folded back
  down under it with the point ahead; walking, the leading arm
  half-extends.
- **Standing easy.** Nothing stands straight: a pillar at nine tenths,
  a sprung leg at three quarters. And each creature has its own slack,
  rolled once from its genome: one side's feet a sixteenth of a limb
  ahead, the head turned a hair that way, the tail curving the other.
  Walking adds the stride, the lift, the bob, the turned head, the
  swaying tail, the beat or the fold.

- **Every part is a box in its bone's frame** (`Body::axes`,
  `build::frame`), as the species tables' parts are: authored along the
  bone's direction with (along, across, up) extents, reaching half a girth
  past either joint so a limb, a tail or a wing is one connected thing and
  not three sticks. The first sheets drew every box square to the world --
  a bent segment as its bounding block, a swept wing as a staircase of
  slabs -- which is a renderer's shortcut the game never takes: it draws a
  part through its rotated bone. `breedcheck -- boxes` prints every part
  as its centre, its axis and its extents, and the sheet draws that.

- **BREED-13 · The band is capped.** The mass has to sit a fifth of the
  stance in from either end, but never more than a fifth of the torso:
  a wide stance is all the steadier, and a wrist put down ahead should not
  count against the body behind.

## 4 · Crossing

[`cross.rs`](../../../crates/sim/src/breed/cross.rs). Three operations, each
ending in a settle and a repair:

- **Breeding** two parents: every major from one or the other, a limb pair
  present in only one inherited half the time, each slider somewhere
  between the two, then a mutation.
- **Grafting** a trophy onto an egg. A trophy is its donor's genome and a
  **pull** per channel: a *feather* pulls fully on the limbs, half on the
  hide, a third on the plan and bulk, a fifth on the temper; a *horn* fully
  on the head and half on bulk and temper; a *fang*, *beak* or *tooth* on
  the head and the temper; a *claw*, *hoof* or *talon* four fifths on the
  limbs; a *plate*, *shell*, *pelt* or *chitin* on the hide and bulk; a
  *quill* or *tail tip* on the tail; a *sac* half on the plan. Each major
  flips at the major rate times the pull, each minor at the minor rate, each
  slider moves a random share (mostly small: the square of a uniform roll)
  times the pull times its inertia. A limb pair the egg has at the donor's
  station is **blended by a rolled proportion** -- ten percent Mantis,
  ninety Ridgeback -- its role and build going over only at the major rate,
  and a limb of another kind taking a quarter of the pull (a feather does
  little to a leg's proportions until the leg is a wing). A limb rebuilt as
  another kind comes in at half its length.
- **Mutation**, alone: the normal nudge on every slider, the rare minor and
  rarer major flips, a limb budding at an empty station (copying a
  neighbour) or, rarest, one lost.

What a creature **drops** is read off its genome
([`archetypes::drops`](../../../crates/sim/src/breed/archetypes.rs)): fangs
drop a fang, a beak a beak, feathers a feather, plates a plate, a ridge a
quill, a tipped tail its tip, a clawed limb a claw. A creature in the world
is its species' archetype **varied** by a seed
([`archetypes::vary`](../../../crates/sim/src/breed/archetypes.rs)): sliders
nudged by a twentieth times their inertia, majors never -- so a herd has a
biggest bull, and the biggest is worth breeding.

## 5 · Rearing

[`rear.rs`](../../../crates/sim/src/breed/rear.rs): a `Ward` is a genome and
a stage. An **egg** gathers warmth, a thousand frames to hatch. A
**hatchling** is the adult at two fifths of its size, growing to full on
what it eats and what it fights. A grazer eats grass and greens, a carnivore
meat, an omnivore (mandibles, a maw) anything; a meal grows it a twelfth,
scaled by the food's tier over the creature's own, so a tier-five hill fed
grass grows a fifth as fast as one fed the Needle's greens. A fight won
against its own tier grows it a twentieth, against a harder one twice that
and raises its **vigor**, which adds up to a tenth to the size it will
reach. At full size it **rebels**; subduing it makes it **tame**. None of
this touches the `World` yet; it is the state machine the world will
drive.

## 6 · The litmus test, and the generations

`cargo test -p sim --test breed` (twenty-two tests) and
`cargo run -p sim --bin breedcheck` (`cast`, `feathers`, `mantis`, `drift`,
`herd`).

**The cast rebuilds.** All eleven archetypes are legal plans their own repair
leaves alone, within a sixth of what their own plan wants on every slider.
Every creature with a table builds the table's number of legs (four, four,
two, eight, six, four, four, four, none) and hips within a quarter of the
table's; every kind of move the table has comes out of the builder (the
Ridgeback's bite, stomp, tail sweep, charge, rear-and-slam, shake, back kick
and spike spray; the Mantis's scythe, lunge, guard, parry, leap and flare;
the Galewing's stoop, talon pass, downwash, feather volley, screech, buffet
and hop; the Broodmother's leg stab, bite, sac slam, screech and web; the
Siegeshell's stamp, shed, plough and shrug; the cats' pounce, rake, ambush,
tail trip and leap, the Veilstalker's tail spear and quills; the Mireback's
spew, flop, tongue, swallow, inflate and wallow; the Sandmaw's rise-bite,
breach, spit, lash and swallow; the bull's charge, hook, trample, guard and
kick); only the Galewing flies; the tiers come out in the bestiary's order
(gnawer 1, bull 2, Ridgeback 3 or 4, Mantis 4, Siegeshell 5).

**Generations.** Measured over seeds:

- *Two of a kind breed true*: twenty lineages of the Pair, twenty
  generations each, keep their stance and limb count in all but a handful
  of four hundred births and their body sliders within a quarter.
- *The biggest breed bigger*: a herd of eight varied Ridgebacks, the two
  largest bred each generation, grows its mean size slider by more than
  0.04 in fifteen; `breedcheck -- herd` shows it reaching the slider's end,
  a thirty-six-metre animal, in twenty -- selection on a hundredth of
  mutation a generation is strong, and the cap is the only thing stopping
  it (BREED-4).
- *A cross lands between its parents*: Ridgeback × Mantis, thirty-two
  seeds, every child's majors from one parent, its sliders within a fifth
  of the parents' range, standing up.
- *One Mantis claw* on a Ridgeback: the stance holds in more than seven of
  eight grafts, the pitch comes up a little, the fore pair does not shrink.
  *Forty claws*: at least half the lineages turn the fore pair raptorial,
  and the torso follows it up part of the way -- a mean pitch between a
  tenth and six tenths.
- *A Ridgeback given feathers*, twenty-four lineages of eighty: it never
  flies after one feather; wings arrive on average after more than three
  (the major rate), and flight more than two generations after the wings,
  once the bones have lightened and the wing has grown; most lineages have
  wings within eighty generations and some fly; the body stands up every
  generation and plausibility falls under 0.45 in fewer than one generation
  in twenty.
- *A graft never breaks the body*: every trophy kind from five donors onto
  five eggs (the worm and the spider among both), thirty grafts each, and
  every result stands.
- *Drift alone*: every creature, two hundred generations of mutation, four
  seeds: a major moves about one generation in forty, plausibility never
  under 0.4, and a lineage whose plan held keeps its mass and pitch within
  three tenths -- size walks further, because nothing but selection holds
  it, and nobody is selecting.

What the harness prints for one feather lineage is in §8.

## 7 · Decided while building, for the owner

- **BREED-1 · Stance is a major; pitch is a slider.** The brief asks how far
  a horizontal Ridgeback *stands up* toward a Mantis -- a number -- and also
  for the upright plan to be a major. Both: the **stance** (level, upright,
  low-slung, limbless) is the discrete major that decides what the plan
  wants and what stands up; the **pitch** is the heavy slider that says how
  far the torso has actually tilted, and the stance pulls it. A level body
  can lean to a third before anything flips.
- **BREED-2 · Limbs blend by proportion; their kind flips at the major
  rate.** The brief's second thought was right: a pair at a matched station
  blends its sliders by a rolled proportion, mostly small; its role and
  build go over as rarely as any other major. The alternative -- role going
  over when the proportion passes a half -- made wings arrive in a third of
  all grafts.
- **BREED-3 · A torso has a say about limbs it does not have.** Its say is
  written per station and role, so a Ridgeback torso that gains a fore
  striking pair wants it heavy and a third of the body long. A thorax or
  abdomen that appears is wanted by nothing yet; the first use of that will
  be the Mantis's thorax on a Ridgeback, and it is a line in `limb_want`'s
  neighbour when it comes.
- **BREED-4 · Nothing holds size but selection.** Mass and pitch are pulled
  by the plan; size is pulled by nothing, so a lineage nobody selects walks
  a hundredth a generation. This is what makes *breeding the biggest* work,
  and it means a wild population's size needs a reason to stay put -- its
  range's food, say -- when the world runs this.
- **BREED-5 · The rates are a table, not knobs yet.** `cross::Rates` and
  the rearing's growth per meal are tuning values and belong in the Oven;
  until an egg is in the game there is no feel to tune them against, so
  they are a `const` judged by the generation tests, and `tests/knobs.rs`
  lists the breed files as creature data with that reason.
- **BREED-6 · A pack creature's genome is one of its animals.** The gnawers'
  genome is a gnawer, the Hornback's the bull; a hatched gnawer egg is one
  gnawer. Whether an egg of a pack species should hatch a pack is open.
- **BREED-7 · A spawned creature is the archetype varied, majors never.**
  Variety in the wild is sliders only (a twentieth, times inertia): the
  owner's *"creatures that spawn in the world need to be recategorized this
  way and given some variety"*, read as *the same plan at different sizes
  and tempers*. If the wild should also throw the odd two-tailed cat, that
  is `vary` rolling `mutate` once.

## 8 · One lineage, as printed

`cargo run -p sim --bin breedcheck -- feathers 9`: the bones lighten from the
first feather (its pull on bulk), the wings arrive at the ninth as stubs a
quarter of the body long, and two generations of growing them later it
flies. Across eight seeds the wings came between the second and the ninth
feather and flight two or three after -- *multiple generations*, as the
brief asks, and the pace is `Rates::graft_major` (BREED-5) if that is too
quick:

```text
generation   1: Level fore=Both/Column pitch 0.00 mass 0.75 size 0.78 margin -1.00 tier 4 plausibility 0.94 [charges, mountable]
generation   2: Level fore=Both/Column pitch 0.01 mass 0.71 size 0.78 margin -1.00 tier 4 plausibility 0.96 [charges, mountable]  <- a major moved
generation   3: Level fore=Both/Column pitch 0.00 mass 0.69 size 0.75 margin -1.00 tier 4 plausibility 0.97 [charges, mountable]
generation   4: Level fore=Both/Column pitch 0.01 mass 0.64 size 0.76 margin -1.00 tier 4 plausibility 0.93 [charges, mountable]
generation   5: Level fore=Both/Column pitch 0.01 mass 0.64 size 0.75 margin -1.00 tier 3 plausibility 0.95 [charges, mountable]
generation   6: Level fore=Both/Column pitch 0.01 mass 0.64 size 0.75 margin -1.00 tier 3 plausibility 0.96 [charges, mountable]
generation   7: Level fore=Both/Column pitch 0.02 mass 0.60 size 0.75 margin -1.00 tier 3 plausibility 0.96 [charges, mountable]
generation   8: Level fore=Both/Column pitch 0.02 mass 0.60 size 0.73 margin -1.00 tier 3 plausibility 0.97 [charges, mountable]
generation   9: Level fore=Wing/Wing pitch 0.01 mass 0.54 size 0.72 wing 0.27 margin -0.55 tier 3 plausibility 0.46 [vestigial wings, mountable]  <- a major moved
generation  10: Level fore=Wing/Wing pitch 0.02 mass 0.52 size 0.69 wing 0.39 margin -0.34 tier 3 plausibility 0.49 [vestigial wings, mountable]
generation  11: Level fore=Wing/Wing pitch 0.02 mass 0.50 size 0.67 wing 0.68 margin 0.12 tier 4 plausibility 0.70 [flies, mountable]
generation  12: Level fore=Wing/Wing pitch 0.01 mass 0.48 size 0.66 wing 0.79 margin 0.30 tier 3 plausibility 0.74 [flies, mountable]
generation  13: Level fore=Wing/Wing pitch 0.02 mass 0.45 size 0.68 wing 0.88 margin 0.46 tier 3 plausibility 0.77 [flies, mountable]
generation  14: Level fore=Wing/Wing pitch 0.03 mass 0.44 size 0.68 wing 0.91 margin 0.51 tier 4 plausibility 0.76 [flies, mountable]
generation  15: Level fore=Wing/Wing pitch 0.02 mass 0.44 size 0.66 wing 0.92 margin 0.53 tier 3 plausibility 0.76 [flies, mountable]
...
```

And the brief's own example, `breedcheck -- mantis 7`, trimmed to where a
major moved and a few milestones: eighteen claws of a Ridgeback leaning
slightly, lightening, its fore pair creeping longer; then the arms go over,
the torso follows them up over the next six generations to two thirds
upright, and what is left is a duellist with a Ridgeback's tail:

```text
generation   1: Level fore=Both/Column pitch 0.02 mass 0.79 size 0.77 margin -1.00 tier 4 plausibility 0.93 [charges, mountable]
generation   4: Level fore=Both/Column pitch 0.08 mass 0.73 size 0.77 margin -1.00 tier 4 plausibility 0.87 [charges, mountable]  <- a major moved
generation   5: Level fore=Both/Column pitch 0.10 mass 0.70 size 0.75 margin -1.00 tier 4 plausibility 0.82 [charges, pounces, mountable]  <- a major moved
generation   8: Level fore=Both/Column pitch 0.07 mass 0.61 size 0.72 margin -1.00 tier 4 plausibility 0.91 [charges, pounces, mountable]
generation  13: Level fore=Both/Column pitch 0.11 mass 0.54 size 0.70 margin -1.00 tier 4 plausibility 0.92 [charges, pounces, mountable]
generation  15: Level fore=Strike/Raptorial pitch 0.16 mass 0.50 size 0.69 margin -1.00 tier 4 plausibility 0.81 [charges, pounces, guards, mountable]  <- a major moved
generation  16: Upright fore=Strike/Raptorial pitch 0.27 mass 0.47 size 0.68 margin -1.00 tier 4 plausibility 0.75 [pounces, guards]  <- a major moved
generation  25: Upright fore=Strike/Raptorial pitch 0.64 mass 0.33 size 0.63 margin -1.00 tier 4 plausibility 0.95 [pounces, guards]
generation  26: Upright fore=Strike/Raptorial pitch 0.65 mass 0.32 size 0.64 margin -1.00 tier 4 plausibility 0.93 [pounces, guards]  <- a major moved
generation  33: Upright fore=Strike/Raptorial pitch 0.70 mass 0.32 size 0.65 margin -1.00 tier 4 plausibility 0.97 [pounces, guards]
generation  35: Upright fore=Strike/Raptorial pitch 0.70 mass 0.31 size 0.64 margin -1.00 tier 4 plausibility 0.94 [pounces, guards]  <- a major moved
generation  39: Upright fore=Strike/Raptorial pitch 0.68 mass 0.32 size 0.61 margin -1.00 tier 4 plausibility 0.95 [pounces, guards]
   final: Upright Forward thorax=false abdomen=true tail=Level | Fore:Strike/Raptorial/Blade 0.56x0.20, MidFore:Walk/Sprung/Claw 0.30x0.26, Hind:Walk/Sprung/Claw 0.35x0.26 | M
   moves: bite, tail sweep, scythe, lunge, parry, pounce, leap, wing flare, shoulder, quill fling, guard
```

## 9 · Open

- **BREED-8 · Where do eggs come from, and how many trophies is a
  generation?** The lineage tests treat one graft as one generation. An egg
  that takes a trophy a day, hatches in a week and breeds at full size is a
  very different pace from one that takes twenty before hatching.
- **BREED-9 · What does a built body look like?** The builder makes bones
  and boxes; the renderer draws species by their `Look`. A crossbred body
  needs a look derived the way the body is -- a form per build and hide --
  and clips: the baked tables are per species, so a built body needs
  procedural gaits, or the nearest archetype's clips retargeted.
- **BREED-10 · Is tier the right currency for food?** A tier-five hill
  needs the Needle's greens. A tier-four duellist is tier four for its
  reaction, not its size; should it need rare food too?
- **BREED-11 · The hatchling in a fight.** It follows, helps, can die. It
  is a creature slot; the world has two. A hunt with a hatchling and the
  Pair is three.
