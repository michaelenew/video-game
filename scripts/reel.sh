#!/usr/bin/env bash
# Film the game headlessly: one run, `frames` pictures `interval` seconds
# apart, then a GIF and a contact sheet. How every character look was judged
# (docs/design/cast-look.md, "The judging kit").
#
#   ./scripts/reel.sh <name> <display> <frames> <interval> [VAR=value ...] -- [game args ...]
#
#   ./scripts/reel.sh champ_spin 91 24 1.0 DEMO=0 SHOT_PORTRAIT=0,0,2.6,1.0 SHOT_SPIN=15 \
#       -- --p1 champion --painted-cast
#
# Writes target/reel/<name>/fNN.png, target/reel/<name>.gif and
# target/reel/<name>_sheet.png. WARM (seconds before the first picture,
# default 60: lavapipe is slow to draw the first frames) and DELAY (GIF
# hundredths a frame, default 25) are read from the environment. Builds
# nothing: run `cargo build -p game` first. Needs `./scripts/setup-tools.sh
# shot`. Use a different display number for each run going at once.
set -euo pipefail
cd "$(dirname "$0")/.."
name=$1; disp=$2; frames=$3; gap=$4; shift 4
envs=(); while [ "$#" -gt 0 ] && [ "$1" != "--" ]; do envs+=("$1"); shift; done
[ "$#" -gt 0 ] && shift
mkdir -p "target/reel/$name"; rm -f "target/reel/$name"/*.png
Xvfb ":$disp" -screen 0 1280x760x24 >/dev/null 2>&1 & X=$!
trap 'kill $X 2>/dev/null || true' EXIT
sleep 2
env DISPLAY=":$disp" WGPU_BACKEND=vulkan "${envs[@]}" ./target/debug/game "$@" \
    >"target/reel/$name.log" 2>&1 & G=$!
trap 'kill $G $X 2>/dev/null || true' EXIT
sleep "${WARM:-60}"
for i in $(seq -w 1 "$frames"); do
  DISPLAY=":$disp" import -window root -crop 1280x760+0+0 "target/reel/$name/f$i.png"
  sleep "$gap"
done
kill $G $X 2>/dev/null || true; wait $G 2>/dev/null || true
convert -delay "${DELAY:-25}" -loop 0 "target/reel/$name"/f*.png -resize 640x "target/reel/$name.gif"
montage "target/reel/$name"/f*.png -tile 4x -geometry 480x285+2+2 "target/reel/${name}_sheet.png" 2>/dev/null || true
echo "$name done"
