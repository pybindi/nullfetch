#!/bin/sh
set -eu

cd "$(dirname "$0")"
prefix=${PREFIX:-$HOME/.local}
bindir="$prefix/bin"

if ! command -v cargo >/dev/null 2>&1 && [ ! -x "$HOME/.cargo/bin/cargo" ]; then
  echo "cargo not found. Install Rust: https://rustup.rs/" >&2
  exit 1
fi

CARGO="${CARGO:-cargo}"
if ! command -v cargo >/dev/null 2>&1; then CARGO="$HOME/.cargo/bin/cargo"; fi

RUSTFLAGS="${RUSTFLAGS:--C link-arg=-L/usr/lib}" "$CARGO" build --release
install -d "$bindir"
install -m755 target/release/nullfetch "$bindir/nullfetch"
./scripts/install-completions.sh

echo "installed: $bindir/nullfetch"
case ":$PATH:" in
  *":$bindir:"*) ;;
  *) echo "note: add $bindir to your PATH" ;;
esac
