# Maintainer: pybindi <sungunboom@gmail.com>
pkgname=nullfetch
pkgver=1.1.0
pkgrel=1
pkgdesc="A tiny system fetch for Linux — pure Python, zero dependencies"
arch=('any')
url="https://github.com/pybindi/nullfetch"
license=('MIT')
depends=('python')
optdepends=('pciutils: GPU detection with lspci')
source=("$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('e3ea50b4cbc1c4891ac9c17ed343bf724b1fe7e7764d06e554171bebfe826f84')

package() {
    cd "nullfetch-$pkgver"
    install -Dm755 nullfetch.py "$pkgdir/usr/bin/nullfetch"
    install -Dm644 logos.py "$pkgdir/usr/share/doc/$pkgname/logos.py"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
