#!/bin/sh
# Install nullfetch into ~/.local (or $PREFIX).
set -eu

cd "$(dirname "$0")"

prefix=${PREFIX:-$HOME/.local}
bindir="$prefix/bin"

install -d "$bindir" "$prefix/share/nullfetch"
install -m755 nullfetch.py "$bindir/nullfetch"
install -m644 logos.py "$prefix/share/nullfetch/logos.py"
./scripts/install-completions.sh

echo "installed: $bindir/nullfetch"
case ":$PATH:" in
*":$bindir:"*) ;;
*) echo "note: add $bindir to your PATH" ;;
esac
