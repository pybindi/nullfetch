#!/bin/sh
set -eu

cd "$(dirname "$0")/.."
prefix=${PREFIX:-$HOME/.local}
install -d "$prefix/share/bash-completion/completions" \
    "$prefix/share/zsh/site-functions" \
    "$prefix/share/fish/vendor_completions.d"
install -m644 scripts/completions/nullfetch.bash \
    "$prefix/share/bash-completion/completions/nullfetch"
install -m644 scripts/completions/_nullfetch \
    "$prefix/share/zsh/site-functions/_nullfetch"
install -m644 scripts/completions/nullfetch.fish \
    "$prefix/share/fish/vendor_completions.d/nullfetch.fish"
echo "installed shell completions under: $prefix/share"
