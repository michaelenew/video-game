#!/usr/bin/env bash
# Install the tools that are not on every machine, once, and only the ones
# asked for.
#
#   ./scripts/setup-tools.sh web     # the browser build: wasm-bindgen-cli
#   ./scripts/setup-tools.sh shot    # headless screenshots: Xvfb, a software
#                                    # Vulkan driver, ImageMagick
#   ./scripts/setup-tools.sh browser # loading the built page: Playwright and
#                                    # its Chromium, for scripts/web-smoke.sh
#   ./scripts/setup-tools.sh all
#
# Idempotent: a tool already there at the right version is skipped in under a
# second, so it is safe to run at the start of every session and every CI job.
# It is the one place the install steps are written down -- the Pages workflow
# calls it, a cloud environment's setup script calls it, and a person on a new
# machine calls it -- so they cannot drift apart.
#
# Made to be started in the background and checked later:
#
#   ./scripts/setup-tools.sh web > target/setup-tools.log 2>&1 &
#   ...                                  # do the work that does not need it
#   wait; tail -n 3 target/setup-tools.log   # before the step that does
#
# Every line it prints says what it did or why it could not, and it exits
# non-zero if anything it was asked for is still missing at the end, so a
# caller can tell "installed" from "gave up" without reading the log.
#
# What each target needs and why:
#
# web   `wasm-bindgen` at exactly the version of the `wasm-bindgen` crate in
#       Cargo.lock; `crates/web/build-game.sh` refuses any other. It is a cargo
#       install, about ninety seconds cold. The wasm target itself comes from
#       rust-toolchain.toml and needs nothing here.
#
# shot  `scripts/screenshot.sh` renders the desktop build under Xvfb with the
#       lavapipe software rasteriser and grabs the window with ImageMagick.
#       Debian packages, so this half only works where apt-get exists and this
#       is root (or sudo works); elsewhere it prints the package list and
#       leaves the rest to you.
#
# browser
#       `scripts/web-smoke.sh` loads the built page in headless Chromium
#       through Playwright, installed globally with npm so one copy serves
#       every checkout. The browser download is the slow part, a couple of
#       hundred megabytes; a machine that already has one (the cloud
#       containers do, at PLAYWRIGHT_BROWSERS_PATH) is skipped.
set -euo pipefail
cd "$(dirname "$0")/.."

status=0

# --- the browser build -------------------------------------------------------
web() {
  local want have
  want=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/[",]/, "", $3); print $3; exit }' Cargo.lock)
  have=$(wasm-bindgen --version 2>/dev/null | awk '{print $2}' || true)
  if [ "$have" = "$want" ]; then
    echo "web: wasm-bindgen $want already installed"
    return
  fi
  echo "web: installing wasm-bindgen-cli $want (have ${have:-none}); this is a cargo build of about a minute and a half"
  if cargo install wasm-bindgen-cli --version "$want" --locked --force >/dev/null 2>&1; then
    echo "web: wasm-bindgen $want installed"
  else
    echo "web: FAILED: cargo install wasm-bindgen-cli --version $want --locked" >&2
    status=1
  fi
}

# --- headless screenshots ------------------------------------------------------
shot() {
  local pkgs="xvfb imagemagick mesa-vulkan-drivers libxkbcommon-x11-0"
  local missing=""
  for p in $pkgs; do
    dpkg -s "$p" >/dev/null 2>&1 || missing="$missing $p"
  done
  if [ -z "$missing" ]; then
    echo "shot: Xvfb, lavapipe and ImageMagick already installed"
    return
  fi
  if ! command -v apt-get >/dev/null 2>&1; then
    echo "shot: no apt-get here; install the equivalents of:$missing" >&2
    status=1
    return
  fi
  local sudo=""
  if [ "$(id -u)" != 0 ]; then
    if command -v sudo >/dev/null 2>&1; then sudo="sudo"; else
      echo "shot: not root and no sudo; run: apt-get install -y$missing" >&2
      status=1
      return
    fi
  fi
  echo "shot: installing$missing"
  if $sudo apt-get install -y --no-install-recommends $missing >/dev/null 2>&1 \
     || { $sudo apt-get update >/dev/null 2>&1 && $sudo apt-get install -y --no-install-recommends $missing >/dev/null 2>&1; }; then
    echo "shot: installed$missing"
  else
    echo "shot: FAILED: apt-get install -y$missing" >&2
    status=1
  fi
}

# --- the page in a browser -----------------------------------------------------
browser() {
  if ! command -v node >/dev/null 2>&1 || ! command -v npm >/dev/null 2>&1; then
    echo "browser: FAILED: node and npm are needed first" >&2
    status=1
    return
  fi
  local root
  root=$(npm root -g)
  if [ ! -d "$root/playwright" ]; then
    echo "browser: installing playwright globally"
    if ! npm install -g playwright >/dev/null 2>&1; then
      echo "browser: FAILED: npm install -g playwright" >&2
      status=1
      return
    fi
  fi
  # Ask Playwright itself whether its Chromium is where it expects; the path
  # depends on its version and on PLAYWRIGHT_BROWSERS_PATH, so nothing here
  # guesses it.
  if node -e "
      const { createRequire } = require('node:module');
      const { chromium } = createRequire('$root/')('playwright');
      require('node:fs').accessSync(chromium.executablePath());
    " >/dev/null 2>&1; then
    echo "browser: playwright and its chromium already installed"
    return
  fi
  echo "browser: downloading playwright's chromium"
  if npx --prefix "$root/.." playwright install chromium >/dev/null 2>&1 \
     || node "$root/playwright/cli.js" install chromium >/dev/null 2>&1; then
    echo "browser: chromium installed"
  else
    echo "browser: FAILED: npx playwright install chromium" >&2
    status=1
  fi
}

if [ $# -eq 0 ]; then
  sed -n '2,10p' "$0" | sed 's/^# \{0,1\}//'
  exit 2
fi
for target in "$@"; do
  case "$target" in
    web) web ;;
    shot) shot ;;
    browser) browser ;;
    all) web; shot; browser ;;
    *) echo "unknown target '$target': web, shot, browser or all" >&2; exit 2 ;;
  esac
done
exit $status
