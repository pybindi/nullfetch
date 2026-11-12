#!/bin/sh
# Build a .deb package from the current source tree.
set -eu

cd "$(dirname "$0")/.."

version=$(sed -n 's/^VERSION = "\(.*\)"$/\1/p' nullfetch.py)
if [ -z "$version" ]; then
    echo "cannot read VERSION from nullfetch.py" >&2
    exit 1
fi

root=$(mktemp -d)
trap 'rm -rf "$root"' EXIT

pkg="$root/nullfetch"
mkdir -p "$pkg/DEBIAN" "$pkg/usr/bin" "$pkg/usr/share/doc/nullfetch" \
    "$pkg/usr/share/nullfetch" "$pkg/usr/share/man/man1"

install -Dm755 nullfetch.py "$pkg/usr/bin/nullfetch"
install -Dm644 logos.py "$pkg/usr/share/nullfetch/logos.py"
install -Dm644 README.md "$pkg/usr/share/doc/nullfetch/README.md"
install -Dm644 debian/copyright "$pkg/usr/share/doc/nullfetch/copyright"
gzip -9n -c nullfetch.1 >"$pkg/usr/share/man/man1/nullfetch.1.gz"

size=$(du -sk "$pkg" | cut -f1)

cat >"$pkg/DEBIAN/control" <<EOF
Package: nullfetch
Version: $version
Section: utils
Priority: optional
Architecture: all
Depends: python3
Recommends: pciutils
Installed-Size: $size
Maintainer: pybindi <sungunboom@gmail.com>
Homepage: https://github.com/pybindi/nullfetch
Description: tiny system fetch for Linux written in Python
 nullfetch prints system information next to an ASCII logo using only
 the Python standard library.
EOF

mkdir -p dist
dpkg-deb --build --root-owner-group "$pkg" "dist/nullfetch_${version}_all.deb"
echo "dist/nullfetch_${version}_all.deb"
