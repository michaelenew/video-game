"""The Sleeper, standing and asleep, and the sheet that shows both.

    python3 scripts/sleeper/pose.py [out.png]

The beast and its two poses are `beast.py`'s; this is where the rest of the
pipeline asks for them, built once each and kept: `asleep()` is what
`terrain.py` lays on the map, `standing()` how it will be when it wakes.
"""
import functools
import sys

from beast import ASLEEP, STANDING, build, horns


@functools.cache
def asleep():
    """The beast asleep: vertices, triangles, a colour a triangle."""
    return build(ASLEEP)[:3]


@functools.cache
def standing():
    """The beast standing: vertices, triangles, a colour a triangle."""
    return build(STANDING)[:3]


@functools.cache
def _asleep_body():
    return build(ASLEEP, without=("horn",))


def mesh():
    """The sleeping body without its horns, for the land: vertices and
    triangles. The horns are stone laid on it (`asleep_horns`)."""
    v, t, _, _ = _asleep_body()
    return v, t


def asleep_horns():
    """The sleeping beast's horns as chains of capsules, set down where the
    body (`mesh`) is."""
    return horns(ASLEEP, _asleep_body()[3])


def main():
    from render import sheet, view

    out = sys.argv[1] if len(sys.argv) > 1 else "target/sleeper-pose.png"
    sv, st, sc = standing()
    av, at, ac = asleep()
    size = (520, 400)
    panels = [
        ("Standing: from its left", view(sv, st, (-1, -0.05, 0), colours=sc, size=size)),
        ("Standing: from in front, to its left, low", view(sv, st, (-0.6, 0.2, -0.8), colours=sc, size=size, perspective=1.8)),
        ("Standing: from behind, to its right", view(sv, st, (0.6, -0.15, 0.75), colours=sc, size=size, perspective=1.6)),
        ("Asleep: from above (it faces up the page)", view(av, at, (0.0001, -1, 0), colours=ac, up=(0, 0, 1), size=size)),
        ("Asleep: from in front, to its left, above", view(av, at, (-0.5, -0.6, -0.6), colours=ac, size=size, perspective=1.6)),
        ("Asleep: from behind, to its right, above", view(av, at, (0.6, -0.6, 0.6), colours=ac, size=size, perspective=1.6)),
    ]
    sheet(panels, columns=3, title="The Sleeper, standing and asleep. Dark: hide. Pale: bone -- plates, horns, claws, teeth, club.").save(out)
    print(out)


if __name__ == "__main__":
    main()
