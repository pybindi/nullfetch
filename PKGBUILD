# Maintainer: pybindi <sungunboom@gmail.com>
pkgname=nullfetch
pkgver=1.7.0
pkgrel=1
pkgdesc="A tiny system fetch for Linux — pure Python, zero dependencies"
arch=('any')
url="https://github.com/pybindi/nullfetch"
license=('MIT')
depends=('python')
optdepends=('pciutils: GPU detection with lspci')
source=("$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('ff3bac988f9734723b026c5599197bf357c7505e287a4fcf605fc59980e3327f')

package() {
    cd "nullfetch-$pkgver"
    install -Dm755 nullfetch.py "$pkgdir/usr/bin/nullfetch"
    install -Dm644 logos.py "$pkgdir/usr/share/nullfetch/logos.py"
    install -Dm644 scripts/completions/nullfetch.bash "$pkgdir/usr/share/bash-completion/completions/nullfetch"
    install -Dm644 scripts/completions/_nullfetch "$pkgdir/usr/share/zsh/site-functions/_nullfetch"
    install -Dm644 scripts/completions/nullfetch.fish "$pkgdir/usr/share/fish/vendor_completions.d/nullfetch.fish"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 nullfetch.1 "$pkgdir/usr/share/man/man1/nullfetch.1"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
