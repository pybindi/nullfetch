# Maintainer: pybindi <sungunboom@gmail.com>
pkgname=nullfetch
pkgver=3.0.0
pkgrel=1
pkgdesc="Fast system fetch tool written in Rust"
arch=('x86_64')
url="https://github.com/pybindi/nullfetch"
license=('MIT')
makedepends=('cargo')
source=("nullfetch-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$srcdir/nullfetch-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$srcdir/nullfetch-$pkgver"
  install -Dm755 target/release/nullfetch "$pkgdir/usr/bin/nullfetch"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 nullfetch.1 "$pkgdir/usr/share/man/man1/nullfetch.1"
  install -Dm644 scripts/completions/nullfetch.bash "$pkgdir/usr/share/bash-completion/completions/nullfetch"
  install -Dm644 scripts/completions/nullfetch.zsh "$pkgdir/usr/share/zsh/site-functions/_nullfetch"
  install -Dm644 scripts/completions/nullfetch.fish "$pkgdir/usr/share/fish/vendor_completions.d/nullfetch.fish"
}
