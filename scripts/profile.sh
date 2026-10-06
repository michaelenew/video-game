#!/usr/bin/env bash
# Measure a frame, headlessly: run the release build under Xvfb and lavapipe
# with `--profile` on, for a while, and keep what it printed.
#
#   ./scripts/profile.sh [log] [seconds] [game args...]
#   ./scripts/profile.sh target/profile.log 40 --hunt ridgeback
#   ./scripts/profile.sh target/c2.log 40 --hunt ridgeback --cascades 2 --shadow-reach 60
#
# Every two seconds the log gets the frame time, the entity count, what is on
# screen, and -- lavapipe can time its own passes -- what each render pass cost
# and how many triangles and pixels it drew. Compare the last few readings of
# two runs; the first ten seconds are the fight starting up.
#
# A software rasteriser, so the GPU numbers are slow in absolute terms and the
# shares between passes are what to read. The CPU side of a frame is best read
# off a chrome trace instead:
#
#   cargo build --release -p game --features bevy/trace_chrome,bevy/bevy_log --target-dir target/trace
#
# which writes trace-<time>.json in the working directory when the game exits.
# DEMO=1 (the default here) has the scripted hunter play, so a fight happens
# with nobody at the keyboard. Needs what `./scripts/setup-tools.sh shot`
# installs.
set -uo pipefail
cd "$(dirname "$0")/.."

LOG="${1:-target/profile.log}"
WAIT="${2:-40}"
shift 2 2>/dev/null || shift $#
DISPLAY_NUM="${DISPLAY_NUM:-99}"

cargo build --release -p game --bin game
mkdir -p "$(dirname "$LOG")"

export WGPU_BACKEND=vulkan
Xvfb ":$DISPLAY_NUM" -screen 0 1280x760x24 >/dev/null 2>&1 &
XVFB=$!
trap 'kill $XVFB 2>/dev/null || true' EXIT
sleep 2

DISPLAY=":$DISPLAY_NUM" DEMO="${DEMO:-1}" timeout "$WAIT" ./target/release/game --profile "$@" >"$LOG" 2>&1
echo "$LOG: $(grep -c '^frame_time' "$LOG") readings"
tail -n 60 "$LOG"
