#!/bin/bash
# Double-click to install what the game needs, fetch it, build it and play.
# A .command file is what macOS opens in Terminal and runs on a double-click.
#
# Installs, only if missing:
#   - Apple's Command Line Tools: the compiler, the linker and git. macOS
#     shows its own install window; this waits for it to finish.
#   - Rust (rustup, cargo), which builds the game. Into your home folder; no
#     password needed.
# Then clones the game into ~/video-game (or pulls the latest if it is already
# there) and runs it with `cargo run --release -p game`. Arguments are passed
# to the game.
#
# Run it again any time: what is installed is skipped, and only what changed
# since last time is rebuilt. Once the game is fetched, this hands over to the
# launcher that came with it, so the newest launcher always does the rest and
# this file never needs downloading again. If this file sits inside a checkout (as it does
# in the repository's scripts folder), it plays that checkout instead and
# leaves its git state alone.
#
# Without double-clicking, from Terminal:
#   bash -c "$(curl -fsSL https://github.com/michaelenew/video-game/releases/download/launcher/play-macos.command)"

set -u

REPO=https://github.com/michaelenew/video-game.git

say() { printf '\n\033[36m==> %s\033[0m\n' "$1"; }
fail() {
    printf '\n\033[31mFAILED: %s\033[0m\n' "$1"
    printf 'Press Return to close.'
    read -r _
    exit 1
}

# --- 1. Apple's Command Line Tools ---------------------------------------------
# `git` and `cc` exist on a fresh Mac as stubs that only offer this install,
# so ask xcode-select whether the real tools are there rather than the PATH.
tools_ready() { xcode-select -p > /dev/null 2>&1 && xcrun --find clang > /dev/null 2>&1; }

say "Checking for Apple's Command Line Tools (compiler, linker, git)"
if tools_ready; then
    echo '    already installed'
else
    echo '    A window will ask to install them: click Install, and agree to the licence.'
    echo '    It is a large download, often ten minutes or more. This waits for it.'
    xcode-select --install > /dev/null 2>&1
    waited=0
    until tools_ready; do
        sleep 5
        waited=$((waited + 5))
        if [ $waited -ge 3600 ]; then
            fail "the Command Line Tools still are not installed after an hour. Install them with 'xcode-select --install' and run this again."
        fi
    done
    echo '    installed'
fi

# --- 2. Rust ---------------------------------------------------------------------
CARGO_ENV="${CARGO_HOME:-$HOME/.cargo}/env"
[ -f "$CARGO_ENV" ] && . "$CARGO_ENV"

# Asked of cargo, not rustup: Homebrew's rustup puts rustup on the PATH and
# not cargo, and cargo is what is run.
say 'Checking for Rust'
if command -v cargo > /dev/null 2>&1; then
    echo '    already installed'
else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain stable \
        || fail 'the Rust installer did not finish. Check the internet connection and run this again.'
    . "$CARGO_ENV"
    command -v cargo > /dev/null 2>&1 || fail 'Rust was installed but cargo cannot be found. Open a new Terminal window and run this again.'
    echo '    installed'
fi

# --- 3. The game -----------------------------------------------------------------
LAUNCHER_DIR="$(cd "$(dirname "$0")" && pwd)"
CHECKOUT="$(dirname "$LAUNCHER_DIR")"
if [ -f "$CHECKOUT/Cargo.toml" ] && [ -d "$CHECKOUT/crates/game" ]; then
    say "Playing the checkout this launcher is in: $CHECKOUT"
else
    CHECKOUT="$HOME/video-game"
    if [ -d "$CHECKOUT/.git" ]; then
        say "Fetching the latest version into $CHECKOUT"
        if ! git -C "$CHECKOUT" pull --ff-only; then
            printf '\033[33m    could not update (files changed by hand?). Playing the version already here.\033[0m\n'
        fi
    else
        say "Downloading the game into $CHECKOUT"
        git clone "$REPO" "$CHECKOUT" \
            || fail 'git clone did not work. Check the internet connection and run this again.'
    fi
    # The copy in the checkout is the newest launcher: whatever it installs or
    # does differently happens on this run. It is inside a checkout, so it
    # plays rather than handing over again; the flag makes sure of it.
    FRESH="$CHECKOUT/scripts/play-macos.command"
    if [ -f "$FRESH" ] && [ -z "${LAUNCHER_HANDED_OFF:-}" ]; then
        say 'Handing over to the launcher that came with this version'
        export LAUNCHER_HANDED_OFF=1
        exec bash "$FRESH" "$@"
    fi
fi

say 'Building and starting the game'
echo '    The first build takes a long while - often 10 to 30 minutes, and several GB of disk.'
echo '    Later runs only rebuild what changed. The game opens in its own window when it is ready.'
cd "$CHECKOUT" || fail "cannot open $CHECKOUT"
cargo run --release -p game -- "$@" || fail 'the game did not build or stopped with an error. The messages above say what.'
