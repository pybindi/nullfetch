# Maintainer: pybindi <322216534+pybindi@users.noreply.github.com>
pkgname=nullfetch-git
pkgver=r6.433f82f
pkgrel=1
pkgdesc="A tiny system fetch for Linux — pure Python, zero dependencies"
arch=('any')
url="https://github.com/pybindi/nullfetch"
license=('MIT')
depends=('python')
makedepends=('git')
optdepends=('pciutils: GPU detection with lspci')
source=("git+$url.git")
sha256sums=('SKIP')

pkgver() {
    cd "$srcdir/nullfetch"
    printf 'r%s.%s' "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}

package() {
    install -Dm755 nullfetch.py "$pkgdir/usr/bin/nullfetch"
    install -Dm644 logos.py "$pkgdir/usr/share/doc/$pkgname/logos.py"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
