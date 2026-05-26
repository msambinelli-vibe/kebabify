# Maintainer: Maycon Sambinelli <msambinelli@gmail.com>
pkgname=kebabify
pkgver=0.1.0
pkgrel=1
pkgdesc='CLI to rename files to kebab-case or snake_case'
arch=('x86_64')
url='https://github.com/your-org/kebabify'
license=('MIT')
depends=('glibc')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/your-org/kebabify/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

check() {
  cd "$pkgname-$pkgver"
  cargo test --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 target/release/kebabify "$pkgdir/usr/bin/kebabify"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE" || true
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md" || true
}
