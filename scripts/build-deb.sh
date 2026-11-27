#!/bin/sh
set -eu

version=$(grep '^version' Cargo.toml | head -n1 | sed 's/.*"\(.*\)".*/\1/')
if [ -z "$version" ]; then echo "cannot read version" >&2; exit 1; fi

arch=$(dpkg --print-architecture 2>/dev/null || echo amd64)
pkg="build/nullfetch_${version}_${arch}"
rm -rf "$pkg"
mkdir -p "$pkg/DEBIAN" "$pkg/usr/bin" "$pkg/usr/share/doc/nullfetch" "$pkg/usr/share/man/man1" \
  "$pkg/usr/share/bash-completion/completions" "$pkg/usr/share/zsh/vendor-completions" "$pkg/usr/share/fish/vendor_completions.d"

cargo build --release --locked
install -m755 target/release/nullfetch "$pkg/usr/bin/nullfetch"
install -m644 README.md "$pkg/usr/share/doc/nullfetch/README.md"
install -m644 nullfetch.1 "$pkg/usr/share/man/man1/nullfetch.1"
install -m644 scripts/completions/nullfetch.bash "$pkg/usr/share/bash-completion/completions/nullfetch"
install -m644 scripts/completions/nullfetch.zsh "$pkg/usr/share/zsh/vendor-completions/_nullfetch"
install -m644 scripts/completions/nullfetch.fish "$pkg/usr/share/fish/vendor_completions.d/nullfetch.fish"

cat > "$pkg/DEBIAN/control" <<EOF
Package: nullfetch
Version: $version
Section: utils
Priority: optional
Architecture: $arch
Maintainer: pybindi <sungunboom@gmail.com>
Description: fast system fetch tool
 A fast system information tool for Linux terminals, written in Rust.
EOF

mkdir -p dist
dpkg-deb --build --root-owner-group "$pkg" "dist/nullfetch_${version}_${arch}.deb"
