# Maintainer: pybindi <sungunboom@gmail.com>
pkgname=nullfetch
pkgver=1.2.3
pkgrel=1
pkgdesc="A tiny system fetch for Linux — pure Python, zero dependencies"
arch=('any')
url="https://github.com/pybindi/nullfetch"
license=('MIT')
depends=('python')
optdepends=('pciutils: GPU detection with lspci')
source=("$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('8cac8dd5d25b737c9563a41555b14b5ed6b2d159a6482045a4c192af95eb8ee4')

package() {
    cd "nullfetch-$pkgver"
    install -Dm755 nullfetch.py "$pkgdir/usr/bin/nullfetch"
    install -Dm644 logos.py "$pkgdir/usr/share/doc/$pkgname/logos.py"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 nullfetch.1 "$pkgdir/usr/share/man/man1/nullfetch.1"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
