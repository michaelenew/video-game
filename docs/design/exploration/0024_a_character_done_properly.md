# 0024 — One character done properly: the Shadow Reaver

**Thread.** The owner's brief (2026-10-20): *a small sample, one character,
done as well as we can possibly get it -- triple-A grade, stylised but with
the human form plausible and not uncanny, the clothing realistic, the
designs, art, texture, animations and particle effects all to that
standard: a class that could plausibly live in Guild Wars or FFXIV. Start
with whichever character is easiest given every one of its effects has to
meet that bar.* And, partway through: *the painted look is really good -- I
always wished Guild Wars 2 had kept its concept art's style in game -- do it
in something resembling that.*

**The short answer.** The Reaver is now a character rather than a
mannequin: a skinned model on the game's own skeleton, built entirely by
script from a public-domain human body (`art/`), dressed in clothes cut
from that body and draped by cloth simulation, textured by a painting
pipeline in the GW2 concept-art direction, with her shadow and the
Reaping's copies drawn as her own body in ink, her coat on springs and her
cuts leaving a brushed trail. Nothing about the simulation, the animation
system or the rules changed: the model rides the skeleton the game already
solves every frame. It is a first sample, honestly assessed in §6 -- a
credible stylised hero, not yet the bar the brief names, with a list of
what stands between them.

## 1. Why the Reaver

| | Reaver | Champion | Bulwark | Elementalist | Blood mage | Dual mage |
| --- | --- | --- | --- | --- | --- | --- |
| What she holds | two daggers | a shapeshifting liquid weapon | a shield that leaves him | nothing | a scythe | nothing |
| Her effects | **one idea**: shadow (a second body, copies, thrown blades) | liquid metal in three shapes | weight, a moving shield | stone, fire, wind, structures | blood, ground, thralls, hearts | dark and light, wings, disks, curses |

Every effect the Reaver has is the same thing -- *her, made of shadow* --
so one material (ink) and one model cover her shadow body, all five
Reaping copies, and (next) her thrown blades. Every other class has three
or four families of effect, each needing its own art. She also holds the
two simplest props in the game, and her fantasy (hooded, masked,
leather and cloth) hides the two hardest things to make well by script: hair
and the mouth.

## 2. The pipeline, and why it is a script

There will not be a developer for this game forever (CLAUDE.md, the art
direction), so a character that is a *file somebody once made in a
sculpting program* is a character nobody can change. A character that is a
*script* can be read, changed in one place, and rebuilt when the skeleton
changes. `art/README.md` has the commands; the stages:

1. **A body that is plausibly human** -- MakeHuman's base mesh (CC0, about
   13 000 quads of production topology) shaped by its own targets: a young
   woman, muscle 0.72, weight 0.32, its "ideal" proportions (long-legged,
   a smaller head: the heroic figure painted fantasy draws). Plausible
   anatomy is the one thing a script cannot invent, and the reason the
   result is not uncanny: it is a measured human, stylised by proportion.
2. **Fitted to the game's skeleton** (`art/lib/fit.py`). The game's twenty-one
   joints (sixteen at first; §8), at the Reaver's build (`view::skeleton::build_for`), in a bind
   pose that bends the skeleton to the body rather than the body to the
   skeleton; each part of the body carried from MakeHuman's bone onto the
   game's, stretched along it, blended across joints by the weights that
   will move it; MakeHuman's 160 bones folded into the game's; the fingers
   closed round a grip, since the game's hand is one bone.
3. **Dressed** (`art/lib/garment.py`, `art/reaver/outfit.py`). A garment is
   the part of the body it covers, **inflated** -- the surface a fixed
   distance from the skin, made through a distance field -- then trimmed by
   planes, its hem relaxed into a curve, and **bridged over hollows** by a
   morphological closing so leather spans the dip under the bust instead of
   following it. Each is made twice: a dense one with its folds pushed in
   (compression folds at elbows and knees, drape below the belt, gathers at
   the neck) and the game's piece, a tenth of the triangles. The coat tails
   are dropped onto her legs under Blender's cloth simulation; the daggers
   are lofted.
4. **Painted** (`art/lib/paint.py`). The folds of the dense pieces are baked
   onto the game's pieces as a **normal map** (so they catch the game's
   light) and as curvature; occlusion, facing and height are baked too; and
   numpy paints the colour from them the way a hand-painted texture is
   made: shade cooler and violet, edges facing the light caught warm,
   value falling toward the feet, every hem trimmed, stitched and worn,
   brushwork laid in 3D so it does not break at a seam. Brows and a smoky
   smudge round the eyes are painted onto the face from the eyes' places.
5. **Exported by hand** (`art/lib/glb.py`) as one binary glTF whose joints
   are exactly the game's, named as `view::skeleton::Joint::name` names
   them -- plus the coat's eight and two points on each blade.

In the game (`crates/game/src/reaver.rs`): one model a seat for her, one for
her shadow, five for the Reaping's copies, made at startup and hung under the
roots the mannequins already had, so they are placed and shown exactly as
those were. Each frame her joints are set from the skin `apply_poses`
already solved. Her materials are the fighters' skin (`still::SkinMaterial`)
so the still world's shade falls on her; the sash is dyed the seat's colour.

## 3. Her effects

- **Ink** (`ink.rs`, `ink.wgsl`). Her shadow and her copies: her model, her
  costume's own light and shade kept but sunk to violet-black, a rim of her
  violet light at the silhouette, and the side-on edge eaten by rising smoke
  -- the same holes cut in the depth prepass (`ink_prepass.wgsl`), so the
  picture and the depth agree. Masked, not blended: blended, her body showed
  through her clothes. Which body is real is read by substance, not shape.
- **The cut** (`slash.rs`). A ribbon from each blade's root to its tip
  through its last ten simulation frames, painted with a generated dry
  brush whose bristles run out toward the old end. *When* is the
  simulation's word (the end of a wind-up, the move out, the first frames of
  recovery); *where* is the drawn steel. Frames a slow machine never draws
  are filled in, so the ribbon is whole at any frame rate.
- **The coat** (`coat.rs`). Each panel a damped pendulum on two joints,
  driven by how the belt accelerates, the lower half whipping after the
  upper, and **held off her legs**: a knee or an ankle further out along a
  panel than it hangs is a floor under its swing, so a stride lifts the
  cloth instead of passing through it.
- **The blades' edges** glow her violet (an emissive map from the honed
  bevel).
- **Her thrown blades** (`blades.rs`, §8): a swept four-pointed star, a shard
  out of the floor, a crescent for a cut left hanging.
- **A blow landing** (`impact.rs`, §8): a flash, a ring and a spray of shards
  at the blade.

All three that move are renderer-local, which `animation.md` allows of
cosmetic motion: none reads as gameplay and none can pop on a rollback.

## 4. What did not work, and why

| Tried | What happened | Now |
| --- | --- | --- |
| Lifting cloth off the skin along its normals | every ear, toe and finger became a spike | the inflated surface of a distance field |
| Clothes that follow the inflated body exactly | a bodysuit: bust, navel and abdominals in leather | the closing bridges hollows; trousers and sleeves as their own garments |
| The first palette, judged in Blender's studio | pale lavender and tan under the arena's sun -- three times too bright | a third of it, judged in the game (`SHOT_PORTRAIT`) |
| Bevy's default reflectance on cloth | the sky's colour on every dark surface | 0.12 on cloth and skin |
| Ink blended | her body and far arm through her clothes | masked, with the prepass's own discard; then see-through over the transmission copy (§8) |
| The ink's prepass reading the clock at the main pass's binding | silently no pipeline, so the copies had no depth and the haze took their tops | the prepass view's binding (one) |
| Pipeline warm-up priming a skinned mesh without its skin | a GPU validation error that took the game down | the primer carries the skin |
| A mirrored body | the arena is right-handed and a drawn pose is reflected (`view::hand_joint`), so a mirrored import was her mirror image | imported as it is; MakeHuman's `.L` is the game's `.r` |
| A trail sampled per drawn frame | a frozen capture collapsed it to a line; a slow machine drew a dot | per simulation frame, gaps filled |

## 5. What it costs

| | |
| --- | --- |
| Triangles | about 72 500 (body 19 300, costume 51 300, sash 1 900) -- a hero's budget; the mannequin was 28 parts |
| Textures | 2048² colour, normal and metal-roughness for the costume, 2048² for the body, an emissive for the blades |
| The model | about 8.1 MB in the binary (`include_bytes!`), the browser build's too |
| A frame | seven skinned instances a seat, six of them hidden unless a Reaving is out; two ribbon meshes a seat rewritten in place during a cut; eight spring joints. Nothing in `sim` or `view` changed, so the simulation's budgets and determinism are untouched |

## 6. Where it stands against the brief

Honestly: a credible stylised hero at the game's camera distance, with a
plausible body and face, garments that read as made, and a signature effect
that is hers. Not yet Guild Wars 2's or FFXIV's bar. What stands between:

1. **Animation.** The clips are the game's procedural ones, authored for a
   mannequin: no fingers, no breathing, no follow-through in the hood. The
   second pass (§8) added a neck, clavicles and toes and a layer that moves
   them, which is a start, not the bar. A body that looks
   real makes stiff motion more visible, not less. The single biggest gap.
2. **The face at close range.** Plausible, not characterful: no hair, no
   painted detail beyond brows and kohl, no expression (there is no jaw or
   brow joint to give one).
3. **Silhouette details** a costume artist would add: a braid or a fringe
   from the hood, sash tails with a knot, buckles and rivets as geometry, a
   second asymmetry (a knife sheath on the thigh).
4. **Texture density.** One 2048 atlas for fifty pieces is about four
   texels a millimetre short of what a close-up wants; the atlas's islands
   are many and small because the game pieces are decimated triangle soups.
   Cleaner low-poly topology (retopology rather than decimation) would fix
   both.
5. **Her other effects** -- made as blades in the second pass (§8), but
   still single meshes in one material: no trail behind a thrown star, no
   ink smoke off them.
6. **Lighting.** The brushed look is where she reads best; under the plain
   look she is lit by a physically based model the painted world does not
   use. A character shader that shares the painted world's light model
   (tone bands, violet shade, warm light) is the natural next step.

## 7. Open questions

- **ART-1.** Is this the direction -- a character built by script from a
  measured body, painted in the concept-art style -- worth carrying to the
  other five classes, in this order: the Reaver finished (animation, face,
  her other effects), then one class at a time?
- **ART-2.** The animation gap (§6.1) is a question about the skeleton: more
  joints (fingers, a neck, a second spine twist, a jaw) are a change to
  `view::skeleton` that every clip would have to learn. Worth it, or should
  the clips get better on the sixteen first?
- **ART-3.** Her palette is indigo, plum leather and steel with a violet
  light, the team colour only on the sash. Is the sash enough to tell seats
  apart at range, or should the hood's trim carry it too?
- **ART-4.** 7.8 MB a character in the browser build. Six of them is about
  45 MB. Acceptable, or should the models stream?

## 8. The second pass (2026-10-20, the owner's notes)

The owner looked at the first sample and asked for eight things. What each
became:

| Asked | Done |
| --- | --- |
| *The shadow transparent, not opaque -- keep the highlights and colours; her more purple, the shadow more grey* | The ink is now **see-through over a copy of the world** (`ink.rs`). A blended body shows its own insides (§4), so instead the ink asks for Bevy's *view transmission texture* -- a copy of everything solid, taken before anything see-through is drawn -- which puts it in a pass drawn after the solid world *with depth written as usual*. The nearest of her surfaces wins the depth test, and that surface mixes itself with the copy: 58% of her across her middle, all of her at the rim and where the smoke is eating her, so her edge light stays. Her body desaturated toward a smoky grey; the rim her violet gone greyer. Her own costume pushed toward purple (`build.py`'s palette). One cost: the depth the outline and the wash read (the prepass) no longer has her shadow in it, so it carries no ink line. |
| *The left click: lunge all the way out, the stabbing hand all the way forward* | The Reave rewritten as a **lunge**: a gather, then the front foot driven a long stride out, the back leg straight behind, the striking shoulder brought forward by its new clavicle, and the arm on the hit line out to 97% of its reach (`anim::track::full_reach`, which does the same for the Plunge and Gallows). |
| *A purple cylinder round her on the aerial left click*; *a visible hit volume on the jump left click* | Two things. `place_beams` drew a rod for any attack whose hit volume is a line, which the Plunge's and Gallows' are; her blows are drawn by her blades and their trail, so it no longer draws for her (nor the Champion, whose weapon is drawn). And the cylinder itself was **the Blood mage's scythe**: `view::scythe::swing_volume` asked whether a move is a scythe swing of its number alone, and a move's number means something only within its class -- the Plunge's and Gallows' numbers are scythe swings of hers, so the scythe's capsule was drawn round the Reaver in the pools' violet. It now asks of the class first (`tests/presentation.rs`, `only_the_blood_mage_swings_a_scythe`). Found by listing every visible mesh over a metre near her on the frame. |
| *A brief effect on impact, to sell the hit; look for other opportunities* | `impact.rs`: the frame her blow meets somebody -- `hit_used` on a move that is out, with the impact freeze holding her or a creature's health just gone down; `hit_used` alone fired on a whiffed Plunge and drew a disc round her, the very thing being removed -- a small hot flash at the tip of the blade that struck, a ring thrown out square to the blow, and fourteen ink shards flung on along it and falling, for seventeen frames of the simulation's clock (so it holds through the impact freeze). The same, at six-tenths the size, where any of her thrown blades catches somebody new. |
| *The shurikens are still their old models -- everywhere they appear* | `blades.rs`: a four-pointed star with swept, diamond-section points, a raised hub and a faint ring where its points blur, in dark violet steel with her violet at the edges; spinning on the simulation's clock. Every throw and spin of hers draws it: the Shuriken, the Fan, the Tumble, the Whirl and the Rising whirl, the burst when her shadow comes home, the Guillotine's lotus. The Plunge's ring is shards rising out of the floor and the Reaping's cut a crescent, in the same steel made solid (blended, a body-sized shard read as violet glass). |
| *More bones, to improve the animations* | Five joints: a **neck**, a **clavicle** each side, a **toe** each side -- twenty-one in all, sixty-six numbers a pose (`animation.md`). An arm hangs from its clavicle but measures its angles from the chest, so no clip had to change for its arm to point where it did. A **secondary layer** (`view::pose::secondary`) moves the new joints on every drawn pose, so every clip got them at once. The model was rebuilt on the new skeleton. |
| *Her chest: athletic clothes, a sports bra -- not two outlines out to each side* | MakeHuman's breast targets on the body: small, firm, close-set and carried high (`reaver_recipe.py`), so the jerkin closes over one compressed shape. |

What it did not do: the new joints carry no authored motion beyond the
secondary layer and the Reave's shoulder; the rest of her clips still move
them only by rule. A trail behind a thrown star, and the shadow's ink line,
are the next two things the effects want.


## 9. The other four (2026-10-21, the owner's word: "build the other 4 classes in the same triple A style, leaving aside bulwark")

**One builder, five specs.** The Reaver's build was split into what every
character shares and what is hers: `art/lib/character.py` (body, dress, paint,
export, driven by a class's spec) and `art/lib/wardrobe.py` (the tailor: a
garment cut from the inflated body, straps, bands, a cape, a collar, boots,
sleeves, a jerkin, coat tails, hair, eyes, the skin and glove masks, the skin
nothing sees). A class is three files under `art/<class>/`: `recipe.py`
(the body), `outfit.py` (what it wears) and `build.py` (the palette). A fix
to the builder or the wardrobe is a fix to every class -- the first was the
boots, which had left the toes out since the toes became joints.

No design document said what any of the four wears (the research found
fantasy, build, effect colours and what each holds, and nothing on costume),
so each outfit is designed from those:

| Class | Who | Wears | Read at range |
| --- | --- | --- | --- |
| Champion | a duellist of the aristocrats' champions; South Asian, the reference middleweight | a high-collared doublet in deep blue, its skirt in four kicking panels; the right sleeve fitted to the wrist, because the gold coils up that arm and forms the weapon; a bracer on the left; a short cape over the left shoulder in the team's colour; pale breeches, tall riding boots, gloves | a dark square torso, colour on one shoulder, pale legs, the gold |
| Elementalist | a terrain author who kneels and drives her hands into the floor; East Asian, tall and narrow | a slate tunic over a long split rust robe that spreads round her when she kneels; linen-bound forearms and bare hands -- they touch the earth; three river stones on her belt; a stole in the team's colour; hair in a knot | a tall figure in earth colours, a long robe, bare hands |
| Blood mage | a reaper and a priestess of something nobody should worship; Black, heavier, not athletic | a long black coat to a high collar, its skirt to the ankle; an oxblood bodice laced over it; long gloves; tall boots; a sash in the team's colour; an ash-silver bob that falls to the jaw | a tall black shape, a red waist, a pale crown -- and the scythe |
| Dual mage | a vessel for two forces, light in her right hand and dark in her left; white, slight | everything split down the middle -- her right ivory and gold, her left obsidian and violet: a fitted high-necked tunic, flared sleeves, four long skirt panels, soft boots, a gold bracer on the right and black glass on the left; her hair to the jaw, white on the right and black on the left | one figure, two halves, and which hand is which |

**Who they are.** The first build gave every body MakeHuman's even mix of its
three ancestry targets, so the women came out alike; the owner caught it
("you don't have to make every model a white woman or a black man"). Each now
has their own heritage in both the face's shape and the painted skin, and
their own hair: a South Asian Champion, an East Asian Elementalist, a Black
Blood mage, a white Dual mage, a Northern European Reaver -- two of the five
white, the owner's call (2026-10-21).

**Hair**, which the Reaver's hood and mask had let her do without, is a
sculpted mass as a painted figure's is (`wardrobe.hair`): the head inflated a
few millimetres, cut back along a hairline, lifted at the crown, falling out
and down below the ears for a bob, and combed on the high piece with uneven
locks that reach the game as the normal map. The first version read as a
swimming cap; the second, with regular grooves, as cornrows.

**In the game** (`game::models`, which `reaver.rs` became): one model of each
class a seat, shown when the seat plays that class; the Reaver's shadow,
copies, blade trail and hit flash stay hers. What each holds is still the
game's own -- the Champion's gold, the Blood mage's scythe -- drawn on the
skeleton's hands; the scythe is now a turned haft and a curved blade
(`blades::haft`, `blades::scythe_blade`) in place of three boxes.

**What it costs.** Five models of about 7 MB each, embedded: about 37 MB in
the binary and in the browser build's download (ART-4 asks whether they
should stream). Each is 55-70 thousand triangles; one of each class a seat
is made at startup and hidden unless used.

**What is left**, in order: the Champion's gold (still beads and boxes),
the Dual mage's wings and the Elementalist's stones are the game's
primitives; the hair is a mass, good at the camera's distance and a cap
close up; faces have painted brows and lips but no expression; the
Bulwark is the mannequin.

**Chests** (2026-10-21, the owner's word: "fix all of the women's chests ...
and remove the blood mage's breasts"). Fitted tops showed two separate
shapes. The body now presses the bust toward the same body built male
(`mh::flatten_bust`): MakeHuman's own breast bones' weights mark the
region, the blend follows them, and a few rounds of smoothing over it take
out what a flat chest still carries. Half way for the Reaver, the
Elementalist and the Dual mage -- one compressed shape, as under a fitted
top -- and all the way for the Blood mage, who has none. Every top bridges
the chest wider as well (`close`), so the cloth spans it instead of
following the skin.

