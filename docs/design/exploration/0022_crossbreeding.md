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

### Wings by degrees, and a spider's form (fourth pass)

The owner's read of the posed sheets: wings jumped from big flying wings to
walking wings and back, which reads as a creature losing something; the
Broodmother's base form was wrong; and the Ridgeback-to-Mantis lineage went
strange in the middle.

- **The middle of Ridgeback × Mantis** was two bugs. A leg rebuilt as
  another kind of leg (column to sprung) was halved by the rule meant for
  a leg becoming a wing, so a ten-metre body found itself on stubs and
  lay on its belly; the rule now applies only to a limb becoming a wing.
  And balancing on its hind legs alone, the pitch search was free to drop
  the nose; it may now drop it a hair at most -- an animal rears to get
  its weight back, it does not bow.
- **How far a wing leans is a number** (`StationPose::wing_lean`), not a
  coin rolled per creature -- the coin re-rolled every generation, which
  is what flickered. It is how far the weight sits ahead of the hind
  feet against how far those legs could comfortably come under it: held
  and tight, the wrist lowered, touching (`WING_PLANT`), carrying more and
  planted further ahead. A lineage's wings come down and go back up a
  little at a time (`tests/posture.rs`, `wings_lean_by_degrees`).
- **Wings are folded at rest and walking, for everybody, and spread only
  in flight** (`build::body_flying`, the `_flight` sheets): airborne for a
  creature that flies, flared on the ground for one whose wings cannot
  lift it. Flight is a capability, read from the lift margin, and never
  tied to whether the wings also walk -- a pterosaur walks on its wings
  and flies on them, and its flight sheet shows the wings it flies on. A
  folded wing is drawn as one: the humerus back along the flank, the
  forearm forward to a wrist near the shoulder, the hand folded back to
  the flank and tucked to about the tail's tip, a band of feathers or
  membrane along it as large as the wing has grown -- a stub folds
  small, a full wing folds full. A wing is a fifth humerus, three tenths
  forearm and half hand, as a bird's is, solved as a two-bone reach for
  held and walking wings alike, so the two are one geometry.
- **The Broodmother** stands as a spider: her cephalothorax is narrow
  (the toad's width rule no longer applies to a body on spindles), her
  eight legs radiate in a fan from it -- the fore pair reaching forward,
  the hind pair back -- with the knees high above the body and the feet
  far out, and her abdomen hangs from the waist at the haunch's rear,
  tilted up and sized as a sac, rather than floating half a torso behind.
- **BREED-14 · A level body rears a sixteenth of a turn at most.** With
  the nose kept up, a level biped balancing on its hind legs would rear
  to the stance's whole pitch cap; a bird standing tall is about a
  sixteenth.

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

## 10 · Motion: how it walks, how a move looks, and the contract measured (2026-10-18)

The owner's ask: *"real creatures that can engage in real fights, be
readable and learnable, and be fun and challenging to fight."* Where an
egg slots into the game is left open; this is the core underneath it. Four
modules, all in `sim::breed`, all deterministic and float-free like the
rest:

- **`drive.rs` -- one frame of motion.** Everything that bends a body for
  one frame: how high the root rides and how far it has travelled, the
  torso's pitch, the neck's pitch, turn and reach, the jaw, the tail's
  swing and curl, a limbless body's wave and rise, the abdomen's tilt, a
  swell (an inflation, a bristle), each foot's place in its stride, a
  striking arm cocked or thrust, the wings folded, flared or aloft and
  their beat. The builder (`build::body_driven`) reads only this, so
  standing, walking, every move and flight are the same body under
  different drives -- and a move's hit can be read off the very bones the
  drive swung. The old rest, walk and flight poses are now three drives.
- **`locomotion.rs` -- the gait.** The **Froude number** (speed squared
  over gravity times hip height: a speed with the body's size taken out)
  picks the gait, because animals of every size change gait at about the
  same values of it -- walk below a half, trot to about two and a half,
  gallop or bound above (Alexander and Jayes, 1983). Stride follows
  Alexander's law, about `2.3 h Fr^0.3`, held to what the shortest leg can
  cover while its foot is down -- past that the body takes more, quicker
  steps rather than let a planted foot slide. The **order the feet fall in
  is the leg count's**, from the animals that have it: a lateral-sequence
  walk and diagonal trot on four legs, a rotary gallop or a bound on sprung
  legs, a back-to-front wave and a tripod on six, a wave and alternating
  tetrapod on eight, a slither with none. A planted foot stays where it was
  put while the body passes over it; the body bobs twice a cycle in a
  walk, rocks nose to tail in a gallop; the head is held steady against
  the rock; the tail swings against the body. A big body takes long slow
  strides and a small one short quick ones at the same Froude number,
  which is what makes a giant read as heavy.
- **`strike.rs` -- how a move looks.** Every move is three poses -- rest,
  **cocked** and **struck** -- and its frames (`MoveSpec`) say how long
  between them. The **windup** goes from rest to cocked fast at first and
  settling, so the silhouette has changed most of the way by a third of
  it: the tell arrives early, not at the last frame. The **strike**
  accelerates from cocked to struck (with an arc for a leap). The
  **recovery** holds the struck pose for its first third -- the punish
  window -- then eases back, and a body that travelled stays where it got
  to. Each move names its strikers (head and jaw for a bite, one forefoot
  for a stomp, the tail for a sweep), and **what it hits is those parts
  where they are on each strike frame**, strung as spheres down each box;
  an area move hits round where they come down, a thrown one flies from
  them out to its reach. Strikes are **aimed**: a bite brings the head to
  chest height on the ground ahead, a tail sweep its tail to a metre, a
  talon pass its feet to half a metre, by a few rounds of build and
  correct (the head first, the body sinking and pitching when the neck
  runs out) -- so a giant's bite comes down to you instead of passing over
  your head. Every move carries the **answer** the bestiary asks for (§1.2:
  sidestep, dodge, jump, crouch, back off, flank, leave the lane, get
  behind a solid, hit it first, leave its back, hold).
- **`verdict.rs` -- the contract, measured.** Nothing a move declares is
  trusted. Its **lane** (front, side, rear, all round, aboard, or *above*:
  never low enough to touch somebody standing) and **range** (close to
  3 m, mid to 10, long beyond) are read off where its hit actually goes;
  its tell is how far its silhouette moves and how early. Then: a damaging
  move is **unanswerable** if its tell is under the fifteen frames of
  `HUMAN_REACTION_FRAMES` or its silhouette changes too little (an eighth
  of the body) or too late (half by a third of the windup) -- *unless its
  answer is where you stand* (back off, flank, leave the lane, leave its
  back, hold), which is §1.4's exemption exactly. It counts **questions**
  (answer, lane, range: two moves are the same question only if all three
  match), **lookalikes** (two moves whose cocked silhouettes are within a
  tenth of the body: same windup, different answer is the cruellest kind
  of unlearnable), **safe** moves (no punish window of eight frames), and
  which **ranges** the kit threatens.

### What the measure found, and what changed because of it

Run on the cast as first written, it found exactly the things that make a
fight unfair or unreadable, and every one was a real fix:

| Found | Fix |
| --- | --- |
| Every gnawer, Pair and Veilstalker bite, and the Galewing's peck, had a tell of 11-14 frames | A move answered by reacting is never faster than a reaction: startup floored at `HUMAN_REACTION_FRAMES` unless its answer is positional (the Mantis's scythe stays fast -- you answer it by standing out of its reach) |
| Recovery as short as 4 frames: nothing to punish | Recovery at least 24 frames, a third of it held still |
| The Ridgeback's bite, charge and tail sweep, and the Galewing's talon pass, passed over a standing head | Strikes aimed at the ground ahead |
| Bites barely moved the silhouette (a few percent of the body) | Bites draw back and up, the jaw gaping; a snake coils into an S |
| Mireback's inflate and spew, Hornback's gore and charge, Siegeshell's shrug and shed, the Sandmaw's bite and rise-bite all cocked alike | Each given its own silhouette: spew leans in low, inflate rises; gore sinks with the head hooked aside, charge stands tall pawing with the tail up; shed rises with the plates lifting, shrug crouches |
| A big body given a scorpion's sting arced it over its back nine metres up | Past six metres long, a sting stabs down behind |
| The four-legged walk and the spider's wave put their feet down in reverse order | Fixed; the footfall diagrams on the gait sheets are how it was seen |

As it stands: **every one of the eleven has no unanswerable move, no safe
move, no two windups alike, every hit able to reach somebody standing, a
threat up close, and three questions or more.** All but the two pack
creatures (the Pair, the gnawers) threaten close, mid and long; a pack's
long range is its other members. `tests/motion.rs` holds that, and holds it
for **seven lineages bred thirty generations each** -- drift, a Mantis's
claw on a Ridgeback every generation, feathers every generation, a fang, a
horn on a spider -- so whatever a body becomes, its moves can still be read
and answered. It also holds the gaits' order, that a planted foot does not
slide, and that a big body walks with a slower cadence.

`cargo run -p sim --bin breedcheck -- moves [name]` prints every move
against the contract; `-- gaits` every gait by speed. The sheets are
`scripts/breed-motion.py`: `gallery/breed/motion/moves_<name>.png` is every
move from rest through the tell, the strike (red spheres: what it hits;
orange rings through the windup: where it will land, the floor marker
drawn from the hit itself) and the held punish window to recovered;
`gaits_<name>.png` is every gait's cycle with its footfall diagram.

### The opens, closed (2026-10-18, later)

What §10 first left open, built:

- **BREED-15 · Facing and turning -- built.** A drive has a **yaw**: the
  whole body turned about the point it stands on, its travel going the way
  it faces. A tail sweep, a tail trip and a lash now turn the body with
  the tail to wind it and spin it through. `strike::aimed` turns a move to
  face a bearing -- as far as a windup can turn, an eighth -- and a thrown
  move comes down to chest height **at the distance it was aimed** and on
  to the floor past it, so a web spat at somebody close no longer passes
  four metres over them. The aim is decided when the move starts and
  never again: a fighter who steps off the line after the tell is missed
  (tested: `a_move_turns_to_where_it_was_aimed`).
- **BREED-16 · Moves in time -- built** as `actor.rs`. An actor has a
  place, a facing and an act (standing, going at a gait, or a move of its
  kit). Walking carries it at its gait's speed and turns it no faster than
  its size allows (the Oven's `BreedTurn`, over the root of its length: a
  giant turns three times slower than a gnawer, which is what makes
  getting to its side an answer). A move plays from where it stood and,
  when it ends, **the actor is where the move carried it, facing the way
  it ended** -- a charge leaves the Ridgeback forty metres on. The body is
  cached and rebuilt only when the drive changes; a still body is built
  once. Nothing allocates. Still not simulated: collision with terrain and
  with other bodies, which is the `World`'s job when this goes in it.
- **BREED-17 · A brain -- built** as `brain.rs`. It **glances** every few
  frames (the Oven's `BreedGlance`) and decides on the newest glance at
  least `HUMAN_REACTION_FRAMES` old, carried forward along the velocity it
  saw -- so it reacts to what it saw a reaction ago, never to a button.
  It throws **only moves that can land**: the target in the move's lane
  and between the near and far reach the verdict measured. It picks by
  appetite, never the same move twice running when there is another,
  closes in when it has moves that reach less far (more often the more
  aggressive it is), turns to face, circles when too close, and waits a
  beat between moves that aggression shortens. Its four numbers are Oven
  knobs (`Breeding` family).
- **BREED-18 · Range gaps -- filled.** `strike::kit` measures the kit and,
  when nothing threatens mid and long, adds a **rush** (a new move: head
  down, weight back, a run along a line of three body lengths and never
  less than twelve metres, answered by leaving the lane); when nothing is
  close, a shoulder (a stomp for a big body). Every cast member and every
  bred generation tested now threatens close, mid and long.
- **The stale lineage sheets -- redrawn**, so the `_posed` ones show the
  gait each body now chooses.

**The contract, played.** `tests/duel.rs` puts each creature's actor and
brain against a fighter for thirty seconds, twice. Standing still in front
of it, the fighter is hit -- 51 times across the cast by moves answered by
reacting. Answering every move a reaction (15 frames) after its tell
starts, with the game's own dodge (its i-frames and speed), jump (its
takeoff speed and gravity) and crouch (its hurtbox), timed as a player who
has learned the move, the fighter is hit by **none** of them -- for the
cast and for three lineages grafted thirty generations. It found four
things the measure alone had not, all fixed:

| Found by playing it | Fix |
| --- | --- |
| A stomp's shockwave was a ball 4.5 m tall: no jump clears it | Shockwaves are discs half a metre high along the floor (`strike::shockwave`) |
| Thrown shots moved two metres a frame and passed between frames | A shot is tested along its whole path each frame |
| A 15-frame jump-answered tell gives a jump no time to rise | A move's tell is a reaction **plus the time its answer takes** (`strike::lead`: a jump's rise to a metre, from the game's own jump numbers) |
| A trip swept from hip height down, catching a jumper; a crouch-answered talon pass came in at half a metre | Low sweeps are low from the windup (half the tell); a crouch-answered move passes between a crouched head and a standing one (`strike::crouch_line`) |

### The owner's opens, with a recommendation each

These are decisions about the game rather than the creatures, so they
are recommendations, not code; each says what it would take.

- **BREED-8 · Pace.** Recommended: **one egg is one generation, and an
  egg takes up to three trophies before it hatches**, applied in the order
  given. The "medium" lineages (a trophy every fourth generation) are then
  a player who feeds an egg one trophy; the "high" ones a player who feeds
  three. Hatching in a day of play, breeding at full size a few days
  after, puts a visibly different creature about a week in -- the drift
  sheets' pace. It costs a field on the egg and nothing in the breeder.
- **BREED-9 · What a built body looks like -- half closed.** How it moves
  is answered: procedural gaits and moves from the same body, with no
  baked clips (§10). How it is *drawn* waits on where it slots in: the
  renderer draws species by their tables, and a built body would be drawn
  part by part from its boxes' tags (`torso`, `limb`, `feathered`...) by
  the same forms the town's props use.
- **BREED-10 · Food by tier.** Recommended: **keep tier as the one
  currency**, but take it from the plan's size *or* its reaction,
  whichever is higher -- a quick duellist is as hard to keep as a big
  hill. One line in `rear.rs`.
- **BREED-11 · The hatchling in a fight.** Recommended: **it fights only
  where there is a free creature slot** -- a hunt of one creature, never
  the Pair -- and when it falls it is carried home rather than lost, so
  raising one is never a loss to a bad fight. It would use exactly this
  actor and brain, with its target the hunted creature.

## 11 · Burrowing, as a trait that passes on (2026-10-18)

The owner: *"The sandworm doesn't have a place in the system without it."*
Before this, burrowing was a flag read off one fact -- a limbless stance --
and nothing used it: the breach-dive sank the body for a pose and that was
all. Worse, it could not really be inherited. A trophy can flip an egg's
stance to limbless, but a body with legs on a limbless plan is repaired to
low-slung (`plan::repair`) with its legs kept, so a legged line became a
burrower only if mutation stripped every pair, which almost never happens.

**The trait.** A new limb build, **`Build::Spade`**: short, broad, clawed,
turned out to the side -- a mole's forelimbs. A body **burrows** when it is
limbless (the worm's way) or when it stands on a pair of spades at its
front half, is no bigger than three fifths on the size slider, is not
upright, and its legs can lift it (`plan::digs`). The plan pushes back the
way it does for every build: spades want a small, solid, level body, so a
big one on spades strains, and past three fifths it keeps its spades as
claws (it rakes with them) and does not go under. The Ridgeback on spades
rakes; the Hornback on spades burrows.

**How it passes.**
- **By trophy:** a trophy from a burrower turns the egg's front walking pair
  to spades, at the major rate times the trophy's say on limbs and plan
  together. A Sandmaw's plates on a Hornback, a trophy a generation: 29 of
  40 lineages dig within thirty generations, every one of them on legs
  (`tests/burrow.rs`).
- **By breeding:** spades are a limb build like any other, so a pair goes
  over from the digging parent about half the time.
- **By mutation:** a build can flip to spades at the major rate, and a body
  can still become a worm by losing every pair.

**How it fights.** Two new moves for anything that burrows, and an
underground state in the actor:
- **Burrow** -- nose down, forefeet scrabbling, then under. Answered by
  hitting it first: it is open while it digs. It ends *under*; it does not
  climb back out.
- **Under** (`actor::Act::Under`) -- travelling below the floor at half its
  top speed, steered toward where it believes you are. **It cannot be
  hit**, and so it is never there long: it has to come up within the
  Oven's `BreedUnderMost` (four seconds), wherever it is. The ground churns
  over it (`Actor::mark`), so you always know where it is.
- **Erupt** -- the windup starts under the floor and travels to where it was
  aimed; the ground there churns as **the mark** (orange on the sheets)
  for the whole windup, which is the tell drawn on the floor (§1.4). Then a
  column the width of the mark (a quarter of its length, never more than
  three metres) and too tall to jump bursts up, and the body after it,
  front first, open through a full punish window. **Answered by stepping
  off the mark**, which is a new answer: its windup is a reaction plus the
  time it takes to run out of the widest mark (`strike::lead`), and the aim
  locks when the windup starts, so moving after you see it always works.
- The breach-dive stays: it is the Sandmaw's own short dive through the
  sand, and a different question.

**The brain** treats going under as one of a burrower's ways to close in:
weighed among its moves whenever you are more than two mark-widths away,
and its fastest way to turn round when you are at its side. Under, it
steers toward its belief of you (a reaction old, as always) and comes up
once you are in reach -- or when its time is up.

**Tested by playing it** (`tests/duel.rs`): the Sandmaw goes under and
comes up under a fighter who stands still, and is never exposed while
under nor under longer than allowed; a fighter who steps off the mark a
reaction after it appears is never hit by an eruption, over five fights. A
Hornback on spades does the same, and is answered the same way. The kit
measure holds for a Hornback grafted thirty generations with a Sandmaw's
plates. On the sheets: `moves_sandmaw.png` and `moves_hornback_spades.png`
(the mole); parts below the floor are not drawn, and the churning ground
over them is the brown ring.

## 12 · In the world, v1: eggs, trophies, and the morph (2026-10-18)

The owner: *"you find an egg (every enemy has one) and need to beat the enemy
to stop it pursuing you to get the egg back ... rather than a breeding
program you feed the egg to a 'morph' and it resets into the baby version,
and as you feed it trophies the animal grows a 'generation'. I have friends
playing this in short bursts, so I want the reward to be immediate ... each
trophy makes a clear difference."*

**The loop** (`sim::keep`, `tests/keep.rs`):
- **Every creature has an egg.** When its hunt begins, its egg lies on the
  floor behind it, from where you came in -- you go past it to get it
  (`KeepEggBehind`, 6 m). Walking into it picks it up (`KeepPickup`, 1.2 m);
  there is no button, and nothing to learn.
- **Carry its egg and it does not stop.** It is roused at once, held by
  nothing, on the one carrying it; and the hunt cannot be walked away from
  -- it ends when the creature is beaten. Fall, and the egg drops where you
  fell; lose the hunt, and it took its egg back. Since 2026-10-19 it follows
  the carrier out of its range and across the whole map
  ([0023](0023_roaming.md)).
- **Beaten, it drops trophies**: two of what its parts are
  (`archetypes::drops`, `KeepTrophies`), a different two each time, picked
  up the same way.
- **Each player has a morph at heel**, a floating blob until it eats an
  egg. It follows past three metres (`KeepFollow`), hurrying when it is far
  behind, and is put back beside you after a seam or a fall. **In a fight
  it keeps back** ten metres (`KeepFollowFight`), on your far side from the
  nearest creature, out from under your swings (2026-10-19: at three metres
  it was "annoyingly right in the way").
- **Feed it from the bag** (B, 2026-10-19; it was a crouch beside it,
  which fed whatever came first). The bag is a window of what you carry:
  click an item, then "Feed to morph". An egg starts it over as that
  creature's young -- not while the egg's creature still hunts you for it,
  and the bag says so (`World::refusal`, `Refusal`). A trophy is a
  generation. The bag holds as much as you pick up.
- **With `--dev`, any item** (2026-10-19): a second window beside the bag
  lists every egg and every trophy a creature drops, under a search box;
  "Add" puts one in your bag (`Travel::grant`: an egg carried, a trophy
  handed over through `World::picked` like one picked up).

**One step at a time, in an order you can learn** (`cross::feed`,
`tests/morph.rs`; rewritten 2026-10-19). The first version was a graft at
high rates with dice on every part, and in play it read as noise: fed the
same thing twice the morph grew and then shrank, and its plan wandered.
The owner: *"needs to be much more consistent and a much more intelligible
evolution path."* So a feeding now rolls nothing:
- **One part of the donor goes over** -- of the parts they differ in, the
  one the trophy pulls hardest on (a feather the limbs, a horn or a tooth
  the head, a plate the hide), front to back, ties to the plan first. The
  next feeding of the same takes the next one. A step the repair would
  undo (a limbless stance on a legged body) is passed over for the next.
- **From the body as it is now.** A feeding reads the morph's genome as
  it stands after every feeding before it, and nothing else: no ancestor,
  and no shape it should be. Every body is as natural as any other. The
  first version broke that without meaning to: after a feeding it settled
  the sliders toward what the body plan "wants" (`plan::settle`, a table
  of what each stance and build should be -- right for a lineage, wrong
  here), so every feeding was pulled back toward that table as well as on
  toward the donor, which was the grow-shrink-grow. A morph's feeding now
  only repairs (it has to stand), and never settles
  (`cross::stand`; `tests/morph.rs`,
  `a_feeding_starts_from_the_body_as_it_is_now`).
- **Its proportions move a fixed share** of the way from where they are to
  the donor's (the pull, the inertia, `MorphSliderGain`), so fed the same
  thing they only close on it
  (`fed_the_same_thing_its_size_only_closes_on_the_donors`). When size is
  the step, it goes half the way, all of it when close.
- **No mutation.** A morph changes by what it eats.

Six Galewing beaks walk a Hornback: head raised, beak and crest, front
limbs to wings, back limbs sprung (it flies), tail level, feathers -- the
same six, in that order, every time. "Something you can see" is still one
definition, `morph::changes`, which writes the line the game shows. The
morph grows as it eats: hatched at two fifths of its size, more each
trophy. (The three chance knobs the dice used are gone.)

**Where it lives: as little as possible in the world** (2026-10-19). The
owner: *"inventory doesn't need to be shared game state ... using an item
is something we can treat like a move / input. It just happens when the
user clicks the key, and then both players agree that the item was used
rather than needing to agree that the item was always there ... I want to
avoid bloat in the state the players need to agree on."* So the world
keeps only what the fight reads:
- **What lies on the floor** (`World::loose`), in the place's coordinates.
- **Which eggs each player carries** (`Keep::eggs`, a bit a creature, the
  journey's): a creature hunts whoever carries its egg, so both machines
  have to agree who.
- **What each player picked up this frame** (`World::picked`), emptied
  every frame: how a game hears of a trophy for its bag.

Everything else is each game's own. **The bag and every seat's morph**
(`game::bag`) are kept from what the world says happened: a trophy in
`picked` goes in this player's bag, and a feeding -- an input naming the
item, `Travel::feed(item)`, not a slot, since the other machine does not
know your bag -- is eaten by that seat's morph on every machine, so every
game draws every morph the same. **A rollback is a correction**: the game
sees every frame it advances, predicted or not, so what happened is kept by
frame number, the last word on a frame standing (as the replay's tape keeps
inputs), and settled only once it is older than a rollback can reach
(`game/src/bag.rs` tests a pickup undone by a rollback). **Where each morph
stands** (`game::heel`) is worked out every frame from where the fighters
are, by the same rules and the same Oven knobs; nothing in a fight reads it.
**It keeps to your side, further the bigger it is** (2026-10-19: grown to
a creature's size, about 12 m, it stood between the camera and you): its
body's own reach plus the follow distance out, on whichever side of you it
is, and in a fight back and to that side. **The bag's "Copy feeding
history"** puts everything your morph has eaten on the clipboard, one item
a line: a feeding rolls no dice, so that list rebuilds it exactly, which is
how a morph that looks wrong is sent to be looked at.

That took the keep from about 640 bytes of the snapshot to 16 for the eggs
and 8 for `picked` (the world: 20,876 bytes to 20,340). What it gives up,
for now: a machine cannot check a trophy fed was ever carried -- a feeding
is taken at its word, as the owner chose; one day verifying offline play
would sign the bag, outside the world's checksum. And both games build the
morphs from the feedings they have seen, so they agree from the start of a
session; a morph carried in from another session would need sending.

**Bodies that stand clear of themselves** (2026-10-19, `sim::breed::build`,
`tests/limbs.rs`). Two things the owner saw on fed morphs:
- **An abdomen in the ground.** It trailed at the torso's own pitch, so an
  upright torso drove it down -- the Mantis's own, 0.8 m under. It points
  down a little at most now, whatever the torso does, and further up if
  its underside would still touch the floor standing.
- **Legs passing through each other.** Every pair bends in the body's own
  plane, so a long leg bent deep swung its knee into the next pair, and a
  stride swung a fore foot back through the hind one. Each pair's reach
  front to back is worked out from how it stands plus the furthest a
  stride swings a foot; where two pairs' reaches overlap, one of them --
  the longer limb, or the hinder -- holds its knee out from the body and
  steps outside the other's line, as a walking wing's elbow does (the
  owner: *"moving limbs towards wings does this by default and it looks
  great"*). Of six trophies of each creature on each other's egg, 56 of
  121 crossed standing and 74 walking; 14 and 26 do now. What is left is
  mostly pairs closer together than they are thick (the Siegeshell's
  front two) and spider legs fanning across another kind of leg.

**A stance comes over in thirds** (2026-10-19). Size only closes on the
donor's, but the stance is one major that flipped at once: a body that took
the Broodmother's sprawl and was then fed the Ridgeback stood its legs
under it in one feeding, and across it went from about 14.5 m to 10.5 m
while its length kept growing -- "it got small again". The morph now
remembers the stance it came from (`Morph::stance_was`) and how many
feedings it has stood the new one (`settled`, of `Morph::SETTLING`, three);
until it is all the way over, the game draws each part between the body as
it stood and as it stands, at the same point of its stride, a third more a
feeding (`Morph::stance_blend`, `game::keep::Drawn`). A part the old body
had none of grows in. It is the morph's alone -- a lineage's crossing has
nobody watching -- and, the morph being each game's own, it costs the
shared world nothing.

**On screen** (`game::keep`): an egg is big, pale and glowing in its
creature's colour, bobbing and turning under a column of light you can see
across a range; a trophy is the same, small and gold. The morph is the
builder's own body, in its egg-creature's colours, walking at the gait its
body chooses. **Unformed, it is a blob** (2026-10-19; the owner: *"like
Morph from Treasure Planet ... a floating blob that doesn't look like
anything"*): glossy pink, two big eyes and a mouth, floating at shoulder
height, wobbling; fed an egg it melts into the creature, the blob shrinking
away as the body grows in where it hung. The first blob read as a
blobfish -- squat, tipped nose-down, its face slid to the bottom -- so it
is round and upright, about a guinea pig's size (a quarter of a metre;
it was 0.85 m and far too big), its face high: big eyes close together with a
highlight in each, a small smile, pink cheeks, nubs, and a tapering tail.
**Moving, it leaves a trail of puffs** off that tail, like a cartoon's
propulsion: a quarter of its size, a little lumpy, left where they were
made (shifted with the world at a crossing) and shrinking evenly to
nothing in a second. Only the blob: once fed, it moves as its creature
does, and leaves none. A few lines at the bottom left say what you carry, what your
morph is, what the last feeding changed, and -- when it is so -- that the
creature wants its egg back.

**Not yet**: the morph does not fight (a hatchling in a fight is BREED-11);
morphs are not drawn in arenas outside the valley; eggs and trophies are
the valley's only.
