#!/bin/sh
# Install nullfetch into ~/.local (or $PREFIX).
set -eu

cd "$(dirname "$0")"

prefix=${PREFIX:-$HOME/.local}
bindir="$prefix/bin"

install -d "$bindir" "$prefix/share/nullfetch"
if command -v cargo >/dev/null 2>&1 || [ -x "$HOME/.cargo/bin/cargo" ]; then
  CARGO="${CARGO:-$HOME/.cargo/bin/cargo}"
  command -v cargo >/dev/null 2>&1 && CARGO=cargo
  RUSTFLAGS="${RUSTFLAGS:--C link-arg=-L/usr/lib}" "$CARGO" build --release
  install -m755 target/release/nullfetch "$bindir/nullfetch"
else
  install -m755 nullfetch.py "$bindir/nullfetch"
fi
install -m644 logos.py "$prefix/share/nullfetch/logos.py"
install -m644 nullfetch_image.py "$prefix/share/nullfetch/nullfetch_image.py"
if [ -f nullfetch.bat ]; then
  install -m644 nullfetch.bat "$prefix/share/nullfetch/nullfetch.bat"
fi
./scripts/install-completions.sh

echo "installed: $bindir/nullfetch"
case ":$PATH:" in
*":$bindir:"*) ;;
*) echo "note: add $bindir to your PATH" ;;
esac
