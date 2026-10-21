#!/usr/bin/env bash
# Run when a Claude Code session opens. In a cloud session the container is
# fresh, and the desktop build will not compile without ALSA's headers
# (`alsa-sys`, for the game's sound), so install them before anything else
# runs. `scripts/setup-tools.sh` is the one place the install is written and
# skips in under a second when it is already there. Never fails the session:
# a failed install shows as a build error naming `alsa-sys`, and the log
# says why.
set -uo pipefail

[ "${CLAUDE_CODE_REMOTE:-}" = "true" ] || exit 0

cd "${CLAUDE_PROJECT_DIR:-$(dirname "$0")/../..}"
mkdir -p target
./scripts/setup-tools.sh desktop > target/setup-tools-desktop.log 2>&1 \
  || echo "setup-tools.sh desktop failed; see target/setup-tools-desktop.log" >&2
exit 0
