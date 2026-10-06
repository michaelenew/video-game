// The line round everything, drawn in screen space.
//
// For each pixel, look at every pixel inside a disc of the line's radius. If
// any of them is nearer than this one by more than a margin, this pixel is just
// outside somebody's silhouette, so it is ink.
//
// Two properties fall out of saying it that way, and they are the two the hull
// could not give. The width is the disc's radius **in pixels**, so it is the
// same on a face seen flat on and a face seen edge on, near and far. And the
// shape of a corner is the shape of the disc, so every join and every end comes
// out round rather than mitred into a spike.
//
// It is one-sided on purpose: the test is "is something nearer", so the line is
// laid *outside* the thing it outlines and never eats into it.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct Outline {
    // The line's half-width, in pixels.
    radius: f32,
    // How much nearer, as a fraction of this pixel's distance, another pixel
    // has to be before it counts as a different thing. Relative rather than
    // absolute, or everything far away would be one solid line.
    step: f32,
    // How much darker the line is than what it outlines, in Oklab lightness.
    ink: f32,
    // What its chroma is multiplied by, and the least it may have. A line is
    // saturated, not dark: see `look::edge::Line::colour`, which this is the
    // shader half of.
    chroma: f32,
    floor: f32,
    // How far from flat a surface has to bend before the fold gets a line.
    fold: f32,
    // Metres per unit of `along`: the camera's near plane, which is what turns
    // a depth ratio into a distance.
    near: f32,
    // Where the line starts fading out and where it is gone, in metres.
    haze: f32,
    gone: f32,
    // How strongly the ink is laid on, 0 to 1.
    weight: f32,
    _pad: vec2<f32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var depth_texture: texture_depth_2d;
// **Sampled, not loaded.** `textureLoad` on a depth texture is the obvious way
// to say "the value at this exact pixel" and WebGL2 cannot do it at all: it
// compiles to GLSL, where a depth texture is only reachable through a sampler.
// Sampling at the middle of the pixel with no filtering is the same answer by
// a route both backends have.
@group(0) @binding(4) var depth_sampler: sampler;
@group(0) @binding(3) var<uniform> settings: Outline;

/// The depth at a pixel, clamped to the picture.
fn depth_at(at: vec2<i32>, size: vec2<i32>) -> f32 {
    let on = clamp(at, vec2<i32>(0), size - vec2<i32>(1));
    let uv = (vec2<f32>(on) + vec2<f32>(0.5)) / vec2<f32>(size);
    return textureSampleLevel(depth_texture, depth_sampler, uv, 0i);
}

// Bevy draws with a reversed, infinite projection: the near plane is depth 1
// and the far distance is 0, so distance along the view goes as one over the
// depth. Only ratios are compared here, so the constant does not matter.
fn along(depth: f32) -> f32 {
    return 1.0 / max(depth, 1e-9);
}

fn oklab(c: vec3<f32>) -> vec3<f32> {
    let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;
    let l_ = sign(l) * pow(abs(l), 1.0 / 3.0);
    let m_ = sign(m) * pow(abs(m), 1.0 / 3.0);
    let s_ = sign(s) * pow(abs(s), 1.0 / 3.0);
    return vec3<f32>(
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    );
}

fn un_oklab(c: vec3<f32>) -> vec3<f32> {
    let l_ = c.x + 0.3963377774 * c.y + 0.2158037573 * c.z;
    let m_ = c.x - 0.1055613458 * c.y - 0.0638541728 * c.z;
    let s_ = c.x - 0.0894841775 * c.y - 1.2914855480 * c.z;
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;
    return vec3<f32>(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    );
}

fn in_gamut(c: vec3<f32>) -> bool {
    return all(c >= vec3<f32>(-0.001)) && all(c <= vec3<f32>(1.001));
}

/// The line's colour over a surface of this colour: a little darker, a lot more
/// saturated, same hue. The Rust side of the same rule is `look::edge::Line`.
///
/// **The hue has to survive, so the chroma gives way.** Most of what you can
/// name in a perceptual space is outside what a monitor can show, and 2.6 times
/// the chroma of a strong blue is a long way outside it. Clamping the channels
/// afterwards looks like it costs nothing and silently rotates the hue -- the
/// first version of this drew a magenta line round a blue box, which is the
/// giveaway. So the chroma is walked down until it fits, exactly as
/// `tint::Lch::rgb` does on the other side.
fn ink_over(lit: vec3<f32>) -> vec3<f32> {
    let lab = oklab(lit);
    let chroma = length(lab.yz);
    let dark = max(lab.x - settings.ink, 0.12);
    if (chroma < 1e-5) {
        return clamp(un_oklab(vec3<f32>(dark, 0.0, 0.0)), vec3<f32>(0.0), vec3<f32>(1.0));
    }
    let hue = lab.yz / chroma;
    let want = max(chroma * settings.chroma, settings.floor);
    var lo = 0.0;
    var hi = want;
    if (in_gamut(un_oklab(vec3<f32>(dark, hue * want)))) {
        lo = want;
    } else {
        for (var i = 0; i < 10; i = i + 1) {
            let mid = 0.5 * (lo + hi);
            if (in_gamut(un_oklab(vec3<f32>(dark, hue * mid)))) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
    }
    return clamp(un_oklab(vec3<f32>(dark, hue * lo)), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let here = vec2<i32>(in.position.xy);
    let size = vec2<i32>(textureDimensions(depth_texture));
    let mine = along(depth_at(here, size));
    let reach = i32(ceil(settings.radius));

    // **A silhouette**: something inside the disc is nearer than this pixel, so
    // this pixel is just outside the edge of it. Tracking which one, so the
    // line can be drawn in the colour of what it outlines rather than in one
    // flat ink.
    var closest = 0.0;
    var found = here;
    let within = settings.radius * settings.radius;
    for (var dy = -reach; dy <= reach; dy = dy + 1) {
        for (var dx = -reach; dx <= reach; dx = dx + 1) {
            let off = vec2<f32>(f32(dx), f32(dy));
            if (dot(off, off) > within) {
                continue;
            }
            let at = clamp(here + vec2<i32>(dx, dy), vec2<i32>(0), size - vec2<i32>(1));
            let step = (mine - along(depth_at(at, size))) / max(mine, 1e-9);
            if (step > closest) {
                closest = step;
                found = at;
            }
        }
    }

    // **A crease**, which depth alone can still find. Across a flat surface,
    // however steeply it is turned away, distance changes linearly: the pixel
    // halfway between two others is at the average of their distances. Where
    // two faces of a box meet it is not, and how far off it is says how sharp
    // the fold is.
    //
    // Checked at the line's own radius rather than at one pixel, which is what
    // keeps the fold's line the same width as the silhouette's: a pixel that
    // close to the crease still has one of its two samples on the far side.
    var fold = 0.0;
    for (var i = 0; i < 4; i = i + 1) {
        let a = f32(i) * 0.7853982;
        let step = vec2<i32>(i32(round(cos(a) * settings.radius)), i32(round(sin(a) * settings.radius)));
        let one = along(depth_at(here + step, size));
        let two = along(depth_at(here - step, size));
        fold = max(fold, abs((one + two) * 0.5 - mine) / max(mine, 1e-9));
    }

    let colour = textureLoad(screen_texture, here, 0);

    // **A line fades where the air takes over.** Ink does not haze: left to
    // itself the pass draws the far rim of a two-kilometre drop plane against
    // the sky as a crisp brown stroke, which is a hard line across the one part
    // of the picture whose whole job is to dissolve. So it is wound down over
    // the same distance the fog is.
    let metres = mine * settings.near;
    let seen = settings.weight * (1.0 - smoothstep(settings.haze, settings.gone, metres));
    if (seen <= 0.001) {
        return colour;
    }

    var ink = colour.rgb;
    if (closest > settings.step) {
        ink = ink_over(textureLoad(screen_texture, found, 0).rgb);
    } else if (fold > settings.fold) {
        ink = ink_over(colour.rgb);
    } else {
        return colour;
    }
    return vec4<f32>(mix(colour.rgb, ink, seen), colour.a);
}
