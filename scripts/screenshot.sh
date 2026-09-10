#!/usr/bin/env bash
# Render the prototype headlessly and save a screenshot.
#
#   ./scripts/screenshot.sh [out.png] [seconds]
#
# SHOT_FRAME=N stops the simulation on frame N first, so two runs compare the
# same moment. BAKED_ANIM=0 captures the procedural poses instead of the baked
# ones. DEBUG_OVERLAY=1 draws the frame-data readout.
#
# Useful in a container with no display, and as a way to see what a change did
# without launching the game. Needs Xvfb, ImageMagick and a Vulkan driver:
#
#   apt-get install -y xvfb imagemagick mesa-vulkan-drivers libxkbcommon-x11-0
#
# lavapipe (mesa-vulkan-drivers) is a software rasteriser -- slow, but it draws
# exactly what a real GPU would.
set -euo pipefail
cd "$(dirname "$0")/.."

OUT="${1:-target/shot.png}"
WAIT="${2:-20}"
DISPLAY_NUM="${DISPLAY_NUM:-99}"

cargo build -p game
mkdir -p "$(dirname "$OUT")"

export WGPU_BACKEND=vulkan
Xvfb ":$DISPLAY_NUM" -screen 0 1280x760x24 >/dev/null 2>&1 &
XVFB=$!
trap 'kill $XVFB 2>/dev/null || true' EXIT
sleep 2

DISPLAY=":$DISPLAY_NUM" DEMO="${DEMO:-1}" ./target/debug/game >/tmp/game-shot.log 2>&1 &
GAME=$!
trap 'kill $GAME 2>/dev/null || true; kill $XVFB 2>/dev/null || true' EXIT
sleep "$WAIT"

DISPLAY=":$DISPLAY_NUM" import -window root "$OUT"
echo "$OUT"
