//! The colours of an arena's surfaces, and the one bright colour that ties
//! them together.
//!
//! ## The problem with being accurate
//!
//! Ten materials used to name ten colours, once, for the whole game: grass was
//! `[0.20, 0.30, 0.16]` everywhere, stone `[0.30, 0.34, 0.40]` everywhere. Both
//! are roughly right about what grass and stone look like, and together they
//! make a picture that looks like nothing in particular -- a set of plausible
//! objects that happen to share a frame. Pushing *further* toward accuracy
//! makes it worse rather than better: at a certain point the eye starts reading
//! the picture as a photograph of something, notices everything that is not
//! quite a photograph, and the whole thing lands in the uncanny valley.
//!
//! ## What a painter does instead
//!
//! Two things, and neither of them is accurate.
//!
//! **Everything is pulled toward the colour of the light.** A green field at
//! sunset is not green-plus-a-sunset, it is a *warm* green, because the only
//! light landing on it is warm. Rotating every surface's hue partway toward the
//! light's own hue is what makes a set of unrelated objects read as a set of
//! objects **in one place**, and it is the single cheapest thing you can do to
//! a palette. Here it is [`Palette::unify`].
//!
//! **One colour is chosen to be wrong.** A bright hue, deliberately far from
//! the light, laid along edges and on the crests of things. It does not
//! describe anything -- nothing in the world is that colour -- and that is what
//! makes it read as a decision rather than as a rendering. Because it appears
//! on *every* object, unrelated objects end up sharing a colour, which is the
//! same unifying trick from the other end. Here it is [`Palette::accent`], and
//! what puts it on an edge is `crate::edge`.
//!
//! ## Why it is derived
//!
//! An arena already names the colour of its air, in `crate::skies`. The light
//! in an arena is the same light that makes its sky, so the palette follows
//! from the sky and nothing has to be chosen twice: a new arena gets a sky, and
//! gets a palette with it. [`Palette::under`] is that derivation, and the loop
//! is the one in [`crate`]: derive, look, tweak, fold back.

use crate::Sky;
use crate::tint::{self, Lch};
use sim::arena::{ArenaId, Material};

/// How an arena's surfaces are coloured.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// The colour of the light here. Everything is pulled partway toward it.
    pub light: [f32; 3],
    /// The bright, deliberately wrong hue. Saturated, for small marks that are
    /// meant to be noticed.
    pub accent: [f32; 3],
    /// The same hue as a **pastel**, and the one that actually gets used on
    /// most surfaces.
    ///
    /// The structure this whole module is after: a rich, fairly flat base, and
    /// then pastels and tints laid over it to bring it to life. Those are two
    /// different jobs and they want two different colours. A saturated accent
    /// run along every edge in the arena is not a highlight, it is a second
    /// base colour fighting the first; a pale one reads as light falling on the
    /// thing, which is what a highlight is.
    pub sheen: [f32; 3],
    /// **What fills a shadow**, which is not black.
    ///
    /// Nothing in daylight is ever lit by one light. The sun is one; the whole
    /// sky is the other, and the sky is a different colour, so the shadowed
    /// side of a rock is not a darker version of its lit side -- it is a
    /// *bluer* one. That is why a shadow painted as plain darkness looks dead
    /// and a shadow painted in the complement looks like a photograph of a real
    /// afternoon.
    ///
    /// So this is the sky's own colour overhead, which is exactly what is
    /// shining into every shadow in the arena, and the game hands it straight
    /// to the ambient light. Under a warm dawn the shadows come out violet;
    /// under a blue midday, a deeper blue. Nothing had to be chosen.
    pub shade: [f32; 3],
    /// How far a surface is turned toward the light's hue, 0 to 1. Zero is ten
    /// materials that have never met; one is a monochrome.
    pub unify: f32,
}

/// How far toward the light, when nothing says otherwise.
///
/// Enough to make a frame hang together and not enough to stop grass being
/// green -- and grass has to stay green, because a player reads the floor at a
/// glance to know what they are standing on.
///
/// It came down from a quarter once the shadows were a colour. A hue rotation
/// and a complementary fill light do the same job from opposite ends, and
/// paying for both leaves nothing: the Gulf's islands went through a rotation
/// toward a peach dawn and then under a violet sky, and arrived as sage.
pub const UNIFY: f32 = 0.15;

/// The lightness every surface is mapped into.
///
/// **Wide, because it is a guard rail and not a decision.** It was a narrow
/// remap when these colours were three floats written by hand and somebody had
/// to stop them being silly. They come from a balanced palette now, where how
/// light a shade is *is* the choice being made, and squeezing snow and sand
/// into the same narrow band threw that choice away and made them the same
/// colour. All this does now is catch something absurd.
///
/// Two tweaks folded back into this one constant, and the second is the more
/// useful. Wide, not narrow: the first version squeezed all ten materials into
/// a third of the scale and gave ten shades of the same thing. And **lower than
/// it looks**: these are albedo, and the light adds roughly a third of the
/// scale on top of them, so a band that looks right on a sheet of raw swatches
/// is a band that arrives on screen past the tonemapper's knee with its
/// differences flattened out. Pastel is what the player sees, which is this
/// band *plus the sun* -- see [`lit`], which is what the sheet draws.
///
/// The floor of this range is the number that does the most work. Under the old
/// palette half the materials sat below 0.45, which is where a colour stops
/// being a colour and starts being a dark shape -- and a picture made of dark
/// shapes under a bright sky reads as a silhouette test, not as a place.
pub const BAND: (f32, f32) = (0.30, 0.90);

/// The most colour a surface may have.
///
/// A single number, and it used to be tied to the accent's own chroma on the
/// reasoning that the one colour meant to be noticed should be the most
/// saturated thing in frame. That rule cost more than it was worth twice: an
/// arena whose accent landed on a hue the palette keeps quiet -- the teals and
/// cyans are a third less colourful than the greens -- pulled every surface in
/// that arena down with it, and the Gulf's grass islands came out as mint.
///
/// What makes an accent read is that it is a *different hue*, laid where
/// nothing else is: along an edge, on a crest. A designer would put a vivid
/// green field under a teal rim without hesitating. So the ceiling is just a
/// ceiling now, and the accent keeps its job by placement.
///
/// Pastel is high lightness **with colour still in it**, so there is a floor as
/// well as a ceiling. The ceiling keeps a bright surface from competing with
/// the accent, which has to be the loudest thing in the frame or it is not an
/// accent.
pub const CHROMA: (f32, f32) = (0.060, 0.225);

impl Palette {
    /// The palette under a sky.
    ///
    /// The light is the air's own colour. The accent is **0.42 of a turn** from
    /// it: not the exact opposite, which pairs with anything and therefore says
    /// nothing, but far enough that it can never be mistaken for more of the
    /// same light. Over a warm dawn that lands in the violets; over a blue
    /// midday, in the corals.
    pub fn under(sky: &Sky) -> Palette {
        // The *hue* of the light is what unifies a palette; its lightness is
        // the sky's business. A cave's sky is nearly black on purpose, and
        // taken literally it would give the cave no light to be under at all,
        // so the lightness is lifted to somewhere a surface could live.
        let light = Lch::of(sky.resolved().horizon);
        let light = Lch {
            l: light.l.max(0.56),
            c: light.c.max(0.02),
            ..light
        };
        // **A whole family, chosen by hue.** The accent wants to be far round
        // the wheel from the light, and rather than computing a colour at that
        // bearing, this takes the nearest family of the borrowed palette and
        // uses it: the accent is its mid shade and the pastel is its pale one.
        //
        // That is better than arithmetic twice over. The two are a pair
        // somebody designed to go together, rather than one colour and a
        // washed-out copy of it. And it cannot land on a colour nobody
        // approved -- an accent worked out from an angle is only as good as
        // what sRGB happens to hold at that angle, and the hues where it holds
        // least are exactly the ones that came out muddy.
        let family = crate::swatch::nearest(light.h + 0.42);
        let accent = family.at(crate::swatch::S500);
        Palette {
            light: light.rgb(),
            accent,
            sheen: family.at(crate::swatch::S100),
            shade: {
                // The sky overhead, made usable as a light: its hue and a good
                // deal of its colour, at a lightness an ambient term wants.
                // **Light, despite being the shadow colour.** This is an
                // irradiance, not a shadow: the engine multiplies it by the
                // ambient brightness, so its lightness is how much sky gets in
                // rather than how dark the shadow ends up. Taken literally from
                // the zenith it came out near black, and every shaded face in
                // the arena went to navy -- rich, and unreadable.
                let z = Lch::of(sky.resolved().zenith);
                Lch::new(0.66, z.c.clamp(0.075, 0.115), z.h).rgb()
            },
            unify: UNIFY,
        }
    }

    /// **Any colour, brought into this arena's scheme.**
    ///
    /// Three steps, in order: the colour itself says *what the thing is*, the
    /// band says *how bright everything here is*, and the pull toward the light
    /// says *where it is*. The order matters -- turn first and the greens have
    /// gone before you ever cap them.
    ///
    /// This takes a colour rather than a material on purpose. An arena's props
    /// name their own colours (a cairn, moss, an egg, a banner), and so will
    /// whatever somebody adds next; putting all of them through here means a
    /// colour written anywhere joins the scheme without anybody remembering to
    /// make it. A dressing that has to be re-tuned per arena is the long-term
    /// effort this whole crate exists to avoid.
    pub fn surface(&self, rgb: [f32; 3]) -> [f32; 3] {
        let own = Lch::of(rgb);
        // Held, not remapped. The colour arrives from a palette somebody has
        // already balanced (`crate::swatch`), so its lightness and its chroma
        // *are the decision* -- a grey family is grey on purpose and a green
        // family is loud on purpose. Stretching every material to one chroma
        // ceiling and one lightness band threw all of that away and produced
        // colours nobody picked: bare earth came out as olive that way, which
        // is the whole reason there is a borrowed palette now. These two lines
        // are a guard rail, and on a well-chosen swatch they do nothing.
        let held = Lch {
            l: own.l.clamp(BAND.0, BAND.1),
            c: own.c.min(CHROMA.1),
            h: own.h,
        };
        glaze(held, Lch::of(self.light), self.unify).rgb()
    }

    /// What a material looks like in this arena.
    pub fn of(&self, material: Material) -> [f32; 3] {
        self.surface(local(material))
    }

    /// A colour with the pastel laid over it, `amount` of the way: what an edge
    /// or a crest gets.
    ///
    /// Blended in Oklab rather than added, unlike the sky's glow: this is paint
    /// on a surface, not light arriving at the eye, and paint replaces what was
    /// under it.
    pub fn accented(&self, base: [f32; 3], amount: f32) -> [f32; 3] {
        tint::mix(base, self.sheen, amount.clamp(0.0, 1.0))
    }
}

/// A surface with a little of the light's own colour laid over it, keeping its
/// value.
///
/// **A glaze, not a rotation.** Turning a hue toward the light's hue walks the
/// colour along the wheel through every hue in between, and the hues in between
/// belong to nobody: a warm brown turned a seventh of the way toward a pale
/// cyan sky goes through olive, which is how the proving ground's floor came
/// out looking like something off a hospital wall. A glaze is what a painter
/// actually does -- a thin wash of one colour over everything -- and it cannot
/// invent a hue that is not already at one end of it.
///
/// **Across the wheel, not around it, and at the surface's own strength.** Two
/// ways of getting this wrong, both tried:
///
/// Turning the hue *angularly* toward the light walks the colour through every
/// hue in between, and the hues in between belong to nobody -- a warm brown
/// turned a seventh of the way toward a pale cyan sky goes through olive, which
/// is how the proving ground's floor came out looking like something off a
/// hospital wall. Moving in a straight line across the colour circle instead
/// goes from brown toward *less brown*, never through green.
///
/// Mixing with the light's actual colour washes everything out, because a sky
/// is nearly white and fifteen percent of nearly white takes a surprising
/// amount of the colour out of anything. A painter does not glaze with white;
/// a glaze is a thin layer of a *saturated* transparent pigment. So the target
/// is the light's hue carrying the surface's own chroma, and what moves is only
/// which way round the circle the colour points.
///
/// The value is put back untouched either way: lightness here is the palette's
/// decision about how dark a thing is, and a glaze is only allowed to change
/// what colour it is.
fn glaze(own: Lch, light: Lch, amount: f32) -> Lch {
    use std::f32::consts::TAU;
    let t = amount.clamp(0.0, 1.0);
    let at = |h: f32| (own.c * (h * TAU).cos(), own.c * (h * TAU).sin());
    let (ax, ay) = at(own.h);
    let (bx, by) = at(light.h);
    let (x, y) = (ax + (bx - ax) * t, ay + (by - ay) * t);
    Lch {
        l: own.l,
        c: (x * x + y * y).sqrt(),
        h: (y.atan2(x) / TAU).rem_euclid(1.0),
    }
}
/// **What a surface of this colour will actually look like on screen**, lit by
/// a sun, facing it.
///
/// A palette is albedo -- *what fraction of the light this bounces* -- and
/// nobody ever sees albedo. A sheet of swatches that shows albedo is a sheet of
/// numbers that are not the picture, and tuning against it produces exactly the
/// mistake it produced here: a set of colours that look bright and distinct on
/// the sheet, and come out on screen as a white wash with the differences
/// crushed out of them.
///
/// Crushed is the word. The renderer multiplies albedo by the light and then
/// **tonemaps**, which is a curve that flattens out at the top so a bright
/// picture does not clip. Two albedos a step apart low down stay a step apart;
/// two albedos a step apart high up arrive almost on top of each other. Half a
/// palette living above the knee is half a palette the player cannot tell
/// apart, however carefully it was chosen.
///
/// So: `x -> E·x / (1 + E·x)` in linear light, which is the shape of the curve,
/// with `E` measured off a screenshot of a lit floor rather than guessed --
/// 4.8, fitted across two materials in the proving ground. It is not the
/// renderer's exact curve and does not need to be; it has the knee in the right
/// place, which is the thing a palette has to be judged against.
pub fn lit(albedo: [f32; 3]) -> [f32; 3] {
    const E: f32 = 4.8;
    let a = tint::linear(albedo);
    tint::srgb([
        E * a[0] / (1.0 + E * a[0]),
        E * a[1] / (1.0 + E * a[1]),
        E * a[2] / (1.0 + E * a[2]),
    ])
}

/// What a material is made of, as a swatch from the borrowed palette.
///
/// One line each, and each line is a decision somebody can disagree with in a
/// word: *rock should be warmer than that*, *sand is too yellow*. That is the
/// point of naming a family and a shade rather than writing three floats --
/// three floats are not a decision anybody can review.
///
/// **The shades sit lower than they look like they should**, mostly 500 to 800.
/// A swatch card is unlit and a game is not: the sun and the tonemap carry a
/// surface a long way up from its albedo (see [`lit`]), by roughly two shades.
/// Picked at the value they are wanted on screen, a set of 300s and 400s comes
/// out as a wash of pastels with no weight anywhere -- which is what happened,
/// and is why this paragraph exists. Snow is the pale exception and is meant
/// to be.
///
/// What each one is doing:
///
/// - **Ground** is bare packed earth and the single largest area in most
///   arenas, so it is the one worth getting right. It is `amber` deep enough
///   to be soil rather than gold -- a sunlit warm earth with real colour in it.
///   The obvious choice was a family actually called *brown*, and measuring it
///   is what ruled it out: every UI palette's brown is deliberately muted, at
///   about a fifth of the colour of its oranges, because a brown in an
///   interface is a background. Laid over a whole floor it is a car park.
/// - **Rock** is cool where the ground is warm, which is what makes a boulder
///   read as a different substance from the dirt it is sitting on.
/// - **Stone** is worked stone -- walls, platforms, the things somebody built.
///   Cooler and bluer again, because separating built from found at a glance
///   is a thing a player does while moving.
/// - **Grass** is `green` rather than `lime` or `emerald`: lime is a spring
///   yellow-green that goes acid under a warm sun, emerald is nearly teal.
/// - **Sand** is `amber` light enough to read as sun-bleached.
/// - **Peat** is the darkest thing here, and the one place a muted brown is
///   right: wet ground has had the colour soaked out of it.
/// - **Water** is `cyan`, not `sky`: a lake is greener than the air above it.
/// - **Wood** is `orange` deep, which is redder than the ground it is lying on
///   -- timber against dirt, and far enough round the wheel to tell apart.
/// - **Ash** is a cool dark grey, and **snow** the one swatch above the band.
pub fn local(material: Material) -> [f32; 3] {
    use crate::swatch::{self, S300, S500, S600, S700, S800};
    match material {
        Material::Ground => swatch::AMBER.at(S700),
        Material::Rock => swatch::BLUE_GREY.at(S600),
        Material::Stone => swatch::SLATE.at(S500),
        Material::Ash => swatch::SLATE.at(S700),
        Material::Grass => swatch::GREEN.at(S700),
        Material::Sand => swatch::AMBER.at(S500),
        Material::Peat => swatch::BROWN.at(S800),
        Material::Wood => swatch::ORANGE.at(S800),
        Material::Water => swatch::CYAN.at(S600),
        Material::Snow => swatch::SLATE.at(S300),
    }
}

/// Every material, for anything that wants to walk them.
pub const EVERY: [Material; 10] = [
    Material::Ground,
    Material::Rock,
    Material::Stone,
    Material::Ash,
    Material::Grass,
    Material::Sand,
    Material::Peat,
    Material::Wood,
    Material::Water,
    Material::Snow,
];
/// The palette for an arena. Derived from its sky, so an arena that has one has
/// both.
pub fn of(id: ArenaId) -> Palette {
    Palette::under(&crate::skies::of(id))
}
impl Palette {
    /// **A way on**, drawn as a light where a seam of the valley is: the
    /// arena's own pastel when the way is open, and a cold grey of the same
    /// lightness when a dark waystone holds it. Derived, so a new place has
    /// its beacons without anybody choosing them.
    pub fn beacon(&self, lit: bool) -> [f32; 3] {
        if lit {
            self.sheen
        } else {
            let l = Lch::of(self.sheen);
            Lch {
                l: l.l * 0.8,
                c: 0.01,
                h: Lch::of(self.shade).h,
            }
            .rgb()
        }
    }

    /// **The face of a cliff** whose top is `top`: rock under a soft top --
    /// turf, snow, sand on a terrace is a skin on stone -- and the top's own
    /// stuff a step darker on bare rock or on something `thin`, a hedge, which
    /// is what it is all the way down.
    pub fn cliff_face(&self, top: Material, thin: bool) -> [f32; 3] {
        if top == Material::Rock || thin {
            let c = Lch::of(self.of(top));
            Lch { l: c.l * 0.88, ..c }.rgb()
        } else {
            self.of(Material::Rock)
        }
    }

    /// A vine on a face: the arena's grass, a step darker, because it hangs
    /// in the face's shadow.
    pub fn vine(&self) -> [f32; 3] {
        let g = Lch::of(self.of(Material::Grass));
        Lch { l: g.l * 0.72, ..g }.rgb()
    }

    /// An updraft: the light itself, which is all rising air is.
    pub fn draft(&self) -> [f32; 3] {
        self.surface([0.94, 0.96, 1.0])
    }
    /// **The land's colour** at a point (`sim::valley::land`): what its
    /// ground is made of, turned to bare rock as it steepens past
    /// `steepest` (slope as rise over run), and laid with old snow high up a
    /// mountainside -- `rise` metres over the floor it rises from, snow
    /// starting at forty and whole by sixty. A mountain reads as a mountain
    /// because its faces are rock and its tops are white; nobody had to paint
    /// either.
    pub fn land(&self, ground: Material, slope: f32, steepest: f32, rise: f32) -> [f32; 3] {
        let base = self.of(ground);
        let rock = self.cliff_face(Material::Rock, false);
        let bare = ((slope - steepest * 0.7) / (steepest * 0.6)).clamp(0.0, 1.0);
        let c = tint::mix(base, rock, bare * bare * (3.0 - 2.0 * bare));
        let snow = ((rise - 40.0) / 20.0).clamp(0.0, 1.0);
        // Snow lies thinner on what is steeper.
        let lies = snow * (1.0 - 0.6 * bare);
        tint::mix(c, self.of(Material::Snow), lies)
    }

    /// **A tree's leaves**: the place's grass, deeper and darker for a pine,
    /// a step lighter and warmer for a broadleaf -- so a wood is the same
    /// green as the meadow it stands in, and a pine wood reads darker than a
    /// copse from across the valley.
    pub fn foliage(&self, pine: bool) -> [f32; 3] {
        let g = Lch::of(self.of(Material::Grass));
        if pine {
            Lch {
                l: g.l * 0.62,
                c: g.c * 1.1,
                h: g.h + 0.02,
            }
            .rgb()
        } else {
            Lch {
                l: g.l * 0.85,
                c: g.c * 1.2,
                h: g.h - 0.03,
            }
            .rgb()
        }
    }

    /// **A trodden road**: earth, the same brown everywhere, balanced into
    /// this palette the way a prop's colour is ([`Palette::surface`]). A
    /// road read as earth under every sky; the ground's own colour turned it
    /// purple under a lilac one.
    pub fn trodden(&self) -> [f32; 3] {
        self.surface([0.58, 0.47, 0.36])
    }

    /// A tree's trunk: the place's timber, darker and greyer -- bark is
    /// weathered wood, and a red trunk reads as a painted post.
    pub fn bark(&self) -> [f32; 3] {
        let w = Lch::of(self.of(Material::Wood));
        Lch {
            l: w.l * 0.62,
            c: w.c * 0.55,
            ..w
        }
        .rgb()
    }

    /// **A crag**: a pillar of bare rock standing on its own, a step lighter
    /// than the rock of a cliff -- it stands out in the light on every side,
    /// where a cliff's face is in its own shadow, and drawn as dark as a
    /// cliff it read as a monolith.
    pub fn crag(&self) -> [f32; 3] {
        let r = Lch::of(self.of(Material::Rock));
        Lch {
            l: (r.l * 1.22).min(0.85),
            ..r
        }
        .rgb()
    }

    /// **Mortar**, between dressed stones: the stone, a good step darker and
    /// greyer -- a joint is a shadow with a little lime in it.
    pub fn mortar(&self) -> [f32; 3] {
        let s = Lch::of(self.of(Material::Stone));
        Lch {
            l: s.l * 0.62,
            c: s.c * 0.5,
            ..s
        }
        .rgb()
    }

    /// **Basalt** (`sim::arena::waterfall`): the place's rock, darker, greyer
    /// and a touch cold -- old lava, which is near black wet and slate grey
    /// dry, and reads as a different stone from the cliffs round it.
    pub fn basalt(&self) -> [f32; 3] {
        let r = Lch::of(self.of(Material::Rock));
        Lch {
            l: r.l * 0.7,
            c: r.c * 0.45,
            h: 0.66,
        }
        .rgb()
    }

    /// **The top of a basalt column**: worn paler than its sides where
    /// nothing grows, and wearing what the box's top is made of where
    /// something does -- moss on a ledge, snow on a cairn.
    pub fn basalt_top(&self, top: Material) -> [f32; 3] {
        let worn = {
            let b = Lch::of(self.basalt());
            Lch {
                l: (b.l * 1.35).min(0.8),
                ..b
            }
            .rgb()
        };
        match top {
            // Moss in patches on worn stone, not a lawn.
            Material::Grass => tint::mix(worn, self.moss(), 0.45),
            Material::Snow => self.of(Material::Snow),
            _ => worn,
        }
    }

    /// **Falling water**: the place's water, paler and whiter -- a fall is
    /// mostly air and foam, and reads lighter than the pool it lands in.
    pub fn falling_water(&self) -> [f32; 3] {
        tint::mix(self.of(Material::Water), [0.94, 0.97, 1.0], 0.55)
    }

    /// **Foam and spray**, where a fall lands: nearly white.
    pub fn foam(&self) -> [f32; 3] {
        tint::mix(self.of(Material::Water), [0.97, 0.98, 1.0], 0.85)
    }

    /// **Moss**, on the tops of old stone and the north of a trunk: the
    /// place's grass, darker and a touch yellower.
    pub fn moss(&self) -> [f32; 3] {
        let g = Lch::of(self.of(Material::Grass));
        Lch {
            l: g.l * 0.78,
            c: g.c * 1.05,
            h: g.h + 0.02,
        }
        .rgb()
    }

    /// **Soil**, where turf is cut -- the lip of a floating island, a bank's
    /// face: the trodden road's earth, a shade darker.
    pub fn soil(&self) -> [f32; 3] {
        let t = Lch::of(self.trodden());
        Lch { l: t.l * 0.72, ..t }.rgb()
    }

    /// **A roof**: fired tile, warm, balanced into the palette like a prop.
    /// Every house in a town wears the same family of reds, so a roof line
    /// reads as a town from across a valley.
    pub fn roof(&self) -> [f32; 3] {
        self.surface([0.62, 0.30, 0.22])
    }

    /// **Plaster**, a house's walls between its timbers: warm off-white.
    pub fn plaster(&self) -> [f32; 3] {
        self.surface([0.86, 0.80, 0.68])
    }

    /// **A house's framing**: the place's timber, dark, so the frame reads
    /// against the plaster.
    pub fn framing(&self) -> [f32; 3] {
        let w = Lch::of(self.of(Material::Wood));
        Lch { l: w.l * 0.55, ..w }.rgb()
    }

    /// **Cloth**, a banner or an awning: the place's accent at full voice.
    pub fn cloth(&self) -> [f32; 3] {
        let a = Lch::of(self.sheen);
        Lch {
            l: (a.l * 0.8).min(0.7),
            c: (a.c * 1.6).max(0.12),
            ..a
        }
        .rgb()
    }

    /// **Flowers** in a meadow: small and few, the warm complement of the
    /// grass, so a patch of them is a point of colour rather than a stain.
    pub fn bloom(&self, k: u32) -> [f32; 3] {
        match k % 3 {
            0 => self.surface([0.92, 0.82, 0.35]),
            1 => self.surface([0.88, 0.50, 0.62]),
            _ => self.surface([0.95, 0.94, 0.90]),
        }
    }
}

/// **A far ridge**, for the lookout's view of the valley: the ground's
/// colour fading into the horizon's with distance, `t` from near (0) to far
/// (1) -- aerial perspective, as one mix.
pub fn far_ridge(horizon: [f32; 3], ground: [f32; 3], t: f32) -> [f32; 3] {
    tint::mix(ground, horizon, 0.35 + 0.6 * t.clamp(0.0, 1.0))
}

/// How much lighter or darker the floor is for standing `height` metres above
/// or below its plain: the relief's shading, as a factor on the floor's colour.
///
/// The floor's rises and dips (`sim::arena::relief`) are a few tens of
/// centimetres over several metres, which tilts the ground a handful of
/// degrees; under a sun sixty degrees up, that is a change in lighting of a
/// few percent, and a hill that is only a few percent is a hill nobody sees.
/// What makes a swell read on real ground is not the sun but the sky: a
/// crown sees the whole dome and a hollow sees less of it, and dust, water
/// and growth all collect downhill, so a dip is darker than the plain for
/// three reasons at once. This is that, as one number per metre: a sixth
/// lighter at half a metre up and a sixth darker at half a metre down,
/// clamped so a tall rise never bleaches.
///
/// Multiplied into the vertex colour, so it goes through `lit` with
/// everything else and is judged on the sheet the same way.
pub fn relief_shade(height: f32) -> f32 {
    (1.0 + 0.33 * height).clamp(0.72, 1.28)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVERY: [Material; 10] = [
        Material::Ground,
        Material::Stone,
        Material::Grass,
        Material::Rock,
        Material::Sand,
        Material::Snow,
        Material::Ash,
        Material::Peat,
        Material::Water,
        Material::Wood,
    ];

    #[test]
    fn nothing_anywhere_comes_out_as_mud_or_as_neon() {
        // The two failures a generated palette has: a surface so dark it reads
        // as a hole, and one so loud it fights the accent. Checked across every
        // arena, because the derivation is what will colour the next one.
        for arena in sim::arena::all() {
            let p = of(arena.id);
            for m in EVERY {
                let c = Lch::of(p.of(m));
                let (name, m) = (arena.name, format!("{m:?}"));
                assert!(c.l >= BAND.0 - 0.01, "{name}/{m}: dark ({:.2})", c.l);
                assert!(c.l <= BAND.1 + 0.01, "{name}/{m}: blown out ({:.2})", c.l);
                assert!(c.c <= CHROMA.1 + 0.005, "{name}/{m}: neon ({:.3})", c.c);
            }
        }
    }

    #[test]
    fn a_player_can_still_tell_the_floor_apart() {
        // Unification is a hue rotation toward one hue, so enough of it turns
        // ten materials into one. This is the line past which the trick costs
        // the game something: a fighter reads what they are standing on at a
        // glance and must not have to look twice.
        for arena in sim::arena::all() {
            let p = of(arena.id);
            for (i, a) in EVERY.iter().enumerate() {
                for b in &EVERY[i + 1..] {
                    let (x, y) = (Lch::of(p.of(*a)), Lch::of(p.of(*b)));
                    let turn = (x.h - y.h).abs().min(1.0 - (x.h - y.h).abs());
                    let apart = (x.l - y.l).abs() + (x.c - y.c).abs() * 2.0 + turn;
                    assert!(
                        apart > 0.035,
                        "{}: {a:?} and {b:?} have become the same surface",
                        arena.name
                    );
                }
            }
        }
    }

    #[test]
    fn the_accent_is_never_just_more_of_the_light() {
        // What makes an accent read as a decision is that it is nowhere near
        // the light. If the derivation ever lands it next door, every edge in
        // the arena goes quietly beige and the whole technique is invisible.
        for arena in sim::arena::all() {
            let p = of(arena.id);
            let (a, l) = (Lch::of(p.accent), Lch::of(p.light));
            let apart = (a.h - l.h).abs();
            let turn = apart.min(1.0 - apart);
            assert!(
                turn > 0.3,
                "{}: the accent is {turn:.2} of a turn from the light",
                arena.name
            );
            // Louder than the arena *typically* is, rather than louder than
            // everything in it. Insisting it beat the most saturated surface
            // meant a quiet accent hue dragged the whole palette down to stay
            // under it; what an accent actually needs is to be a different
            // colour, somewhere nothing else is.
            let usual: f32 =
                EVERY.iter().map(|m| Lch::of(p.of(*m)).c).sum::<f32>() / EVERY.len() as f32;
            assert!(
                a.c > usual,
                "{}: the accent ({:.3}) is duller than an average surface ({usual:.3})",
                arena.name,
                a.c
            );
        }
    }

    #[test]
    fn the_pastel_is_pale_and_the_accent_is_not() {
        // The two jobs, and the reason there are two colours. A highlight is
        // light and soft; a mark meant to be noticed is neither. Run the
        // saturated one along every edge in the arena and it stops being an
        // accent and becomes a second base colour fighting the first.
        for arena in sim::arena::all() {
            let p = of(arena.id);
            let (sheen, accent) = (Lch::of(p.sheen), Lch::of(p.accent));
            let name = arena.name;
            assert!(
                sheen.l > BAND.1,
                "{name}: the pastel is not lighter than a surface"
            );
            // Measured across the whole wheel the worst case is 0.43, in the
            // yellows, where even a pale shade keeps a lot of colour because a
            // desaturated yellow stops looking like yellow at all.
            assert!(
                sheen.c < accent.c * 0.45,
                "{name}: the pastel is not soft ({:.3} against {:.3})",
                sheen.c,
                accent.c
            );
            // Loose on purpose. A hand-tuned ramp **drifts in hue** from its
            // light end to its dark one -- a designer warms the pale shades and
            // cools the deep ones so neither looks washed out, and across these
            // the drift reaches a thirteenth of a turn. That is a property of a
            // palette somebody balanced, not an error: the two still read as
            // the same colour, which is all this is checking. The old limit
            // here was written when both were computed from one angle, and
            // arithmetic has no reason to drift.
            let turn = (sheen.h - accent.h).abs();
            assert!(
                turn.min(1.0 - turn) < 0.09,
                "{name}: the pastel and the accent are not the same colour"
            );
        }
    }

    #[test]
    fn a_shadow_is_filled_with_sky_and_not_with_darkness() {
        // What makes a shadow look like an afternoon rather than like a hole.
        // It has to be a *colour*, and it has to be a different one from the
        // light, or every shadow in the game is just the base colour turned
        // down.
        for arena in sim::arena::all() {
            // The cave has no sky to fill anything, and the lab's sky is flat
            // on purpose -- it is where jumps are measured, and a coloured
            // shadow there is a colour somebody ends up measuring.
            if arena.id == ArenaId::BROODMOTHER || arena.id == ArenaId::LAB {
                continue;
            }
            let p = of(arena.id);
            let (shade, light) = (Lch::of(p.shade), Lch::of(p.light));
            let name = arena.name;
            assert!(shade.c > 0.05, "{name}: the shadow fill went grey");
            let turn = (shade.h - light.h).abs();
            assert!(
                turn.min(1.0 - turn) > 0.02,
                "{name}: the shadow is the same colour as the light"
            );
        }
    }

    #[test]
    fn an_arena_colours_its_own_grass() {
        // The point of deriving the palette from the sky rather than naming ten
        // colours once: the same material belongs to the place it is in. Not so
        // far that it stops being grass -- that is the test above -- but far
        // enough to see.
        let meadow = of(ArenaId::HORNBACK).of(Material::Grass);
        let dusk = of(ArenaId::PAIR).of(Material::Grass);
        let apart: f32 = meadow.iter().zip(dusk).map(|(a, b)| (a - b).abs()).sum();
        assert!(
            apart > 0.03,
            "grass is the same everywhere: {meadow:?} {dusk:?}"
        );
    }
}
