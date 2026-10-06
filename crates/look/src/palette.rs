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
    /// The most chroma a surface in *this* arena may have.
    ///
    /// Not the constant: sRGB cannot hold the same chroma at every hue, so an
    /// accent asked for a strong violet comes back strong and one asked for a
    /// strong teal comes back weaker, and in that arena a rich floor would
    /// out-shout the one colour that is supposed to be the loudest thing in
    /// frame. Keeping the ceiling a fixed way under whatever the accent
    /// actually got makes the relationship true by construction rather than by
    /// being lucky with the hue.
    pub ceiling: f32,
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
/// Mid, not pale. A base layer that is already nearly white has nowhere to put
/// a highlight: the pastels laid on it do not read, and the shadow side has
/// only grey to fall into. Rich down here, light up there.
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
pub const BAND: (f32, f32) = (0.38, 0.70);

/// How much colour a surface may have: a floor and a ceiling.
///
/// The ceiling came up a long way once there were proper shadows to put under
/// these. A desaturated base plus a pastel on top is two weak colours; a rich
/// base with a pastel on top is a picture. The ceiling is still under the
/// accent's own chroma, because the one colour that is meant to be noticed has
/// to be the most saturated thing in the frame.
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
        // **As loud as that hue gets**, at whatever lightness holds the most
        // colour: see `tint::loudest`. A fixed lightness gives a vivid violet
        // and a muddy teal, and since the surface ceiling is kept under the
        // accent, a muddy teal takes the whole arena down with it -- which is
        // how the Gulf's emerald islands arrived as sage.
        let accent = tint::loudest(light.h + 0.42).rgb();
        Palette {
            light: light.rgb(),
            // Bright and strongly coloured, and at a lightness of its own:
            // an accent that matches the surfaces it sits on disappears.
            accent,
            // Lighter and much softer: a highlight, not a second base colour.
            // Taken off what the accent *became* rather than off a constant,
            // for the same reason the ceiling is: a hue sRGB cannot hold
            // strongly would otherwise get a pastel as loud as its own accent.
            sheen: Lch {
                l: 0.88,
                c: Lch::of(accent).c * 0.38,
                h: Lch::of(accent).h,
            }
            .rgb(),
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
            ceiling: CHROMA.1.min(Lch::of(accent).c * 0.82),
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
        Lch {
            l: BAND.0 + (BAND.1 - BAND.0) * share(own.l),
            c: CHROMA.0 + (self.ceiling - CHROMA.0) * vividness(own.c),
            h: own.h,
        }
        .toward_hue(Lch::of(self.light).h, self.unify)
        .rgb()
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

/// Where a colour's chroma sits in the range the materials span, 0 to 1.
///
/// The same trick as [`share`], and for a better reason. Clamping chroma into
/// the range leaves everything that was already inside it exactly where it was,
/// so a palette written by hand is only as rich as whoever wrote the triples
/// happened to make it -- which turned out to be a floor at a third of the
/// chroma it was allowed, and a picture that looked washed out while the
/// numbers all said it should not be.
///
/// Stretching instead puts stone near the bottom of the arena's range and grass
/// near the top, whatever was typed. Richness becomes a property of the
/// derivation rather than of somebody's eye for an sRGB triple.
fn vividness(c: f32) -> f32 {
    const DULLEST: f32 = 0.012;
    const RICHEST: f32 = 0.100;
    ((c - DULLEST) / (RICHEST - DULLEST)).clamp(0.0, 1.0)
}

/// Where a lightness sits in the range the materials span, 0 to 1.
///
/// The materials keep their *order* -- peat stays the darkest thing and snow
/// the lightest -- while the range they occupy is squeezed into [`BAND`]. Order
/// is what a player reads; the absolute values are ours to choose.
fn share(l: f32) -> f32 {
    const DARKEST: f32 = 0.55;
    const LIGHTEST: f32 = 0.96;
    ((l - DARKEST) / (LIGHTEST - DARKEST)).clamp(0.0, 1.0)
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

/// A material's own colour, before any arena has had an opinion about it.
///
/// Written **rich** rather than accurate, because accurate is where the uncanny
/// valley is and pale is where nothing has anywhere to go. These say *grass*,
/// *sand*, *snow* at a glance -- the only thing they have to do, since a floor
/// is read at a glance and never looked at -- and they say it in a colour
/// strong enough to carry a pastel highlight and a coloured shadow on top of
/// it. Grass here is an emerald, not a sage.
pub fn local(material: Material) -> [f32; 3] {
    match material {
        Material::Ground => [0.58, 0.48, 0.32],
        Material::Stone => [0.58, 0.63, 0.76],
        Material::Grass => [0.24, 0.68, 0.46],
        Material::Rock => [0.62, 0.56, 0.50],
        Material::Sand => [0.90, 0.76, 0.46],
        Material::Snow => [0.88, 0.93, 0.99],
        Material::Ash => [0.48, 0.48, 0.58],
        Material::Peat => [0.42, 0.30, 0.26],
        Material::Water => [0.14, 0.62, 0.72],
        Material::Wood => [0.68, 0.38, 0.20],
    }
}

/// The palette for an arena. Derived from its sky, so an arena that has one has
/// both.
pub fn of(id: ArenaId) -> Palette {
    Palette::under(&crate::skies::of(id))
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
                assert!(c.c <= p.ceiling + 0.005, "{name}/{m}: neon ({:.3})", c.c);
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
            // Against the surfaces this arena actually has, not against the
            // ceiling: sRGB cannot hold the same chroma at every hue, so an
            // accent asked for 0.19 comes back as whatever its hue allows, and
            // what matters is only that it still wins.
            let loudest = EVERY
                .iter()
                .map(|m| Lch::of(p.of(*m)).c)
                .fold(0.0f32, f32::max);
            assert!(
                a.c > loudest,
                "{}: the accent ({:.3}) is duller than a surface ({loudest:.3})",
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
            assert!(sheen.c < accent.c * 0.6, "{name}: the pastel is not soft");
            let turn = (sheen.h - accent.h).abs();
            assert!(
                turn.min(1.0 - turn) < 0.03,
                "{name}: they are not the same hue"
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
