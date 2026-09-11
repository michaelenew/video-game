#!/usr/bin/env bash
# Print every command, key, flag and environment variable this project has.
#
# A thin wrapper: the content lives in crates/manual, which the game's --help
# and the on-screen legend also read, so there is only one copy to go stale.
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo run -q -p manual
