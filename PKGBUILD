# maintainer: sharkthakftw @ https://github.com/sharkthakftw
pkgname=wikid
pkgver=2.3.0
pkgrel=1
pkgdesc="feature-rich terminal wikipedia client"
arch=('x86_64' 'aarch64')
url="https://github.com/sharkthakftw/wikid"
license=('MIT')
depends=('gcc-libs')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
