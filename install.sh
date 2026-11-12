#!/bin/sh
# Install nullfetch into ~/.local (or $PREFIX).
set -eu

cd "$(dirname "$0")"

prefix=${PREFIX:-$HOME/.local}
bindir="$prefix/bin"

install -d "$bindir"
install -m755 nullfetch.py "$bindir/nullfetch"

echo "installed: $bindir/nullfetch"
case ":$PATH:" in
*":$bindir:"*) ;;
*) echo "note: add $bindir to your PATH" ;;
esac
