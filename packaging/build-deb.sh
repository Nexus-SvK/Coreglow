#!/bin/sh
# Build target/debian/ccxt-control_<version>_<arch>.deb with plain dpkg-deb (no cargo-deb needed).
set -eu
cd "$(dirname "$0")/.."

cargo build --release --locked
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
arch=$(dpkg --print-architecture)
pkg=ccxt-control
stage=target/debian/${pkg}_${version}_${arch}
rm -rf "$stage"

install -Dm755 target/release/ccxtd      "$stage/usr/bin/ccxtd"
install -Dm755 target/release/ccxtctl    "$stage/usr/bin/ccxtctl"
install -Dm755 target/release/ccxt-gui   "$stage/usr/bin/ccxt-gui"
install -Dm644 packaging/ccxtd.service   "$stage/usr/lib/systemd/user/ccxtd.service"
install -Dm644 packaging/70-ccxt-control.rules "$stage/usr/lib/udev/rules.d/70-ccxt-control.rules"
install -Dm644 packaging/ccxt-gui.desktop "$stage/usr/share/applications/ccxt-gui.desktop"
install -Dm644 packaging/ccxt-control.svg "$stage/usr/share/icons/hicolor/scalable/apps/ccxt-control.svg"
install -Dm644 assets/config.toml         "$stage/usr/share/ccxt-control/config.toml"
install -Dm644 README.md                  "$stage/usr/share/doc/$pkg/README.md"
install -Dm644 packaging/copyright        "$stage/usr/share/doc/$pkg/copyright"

mkdir -p "$stage/DEBIAN"
size=$(du -ks "$stage/usr" | cut -f1)
cat > "$stage/DEBIAN/control" <<EOF
Package: $pkg
Version: $version
Architecture: $arch
Maintainer: eddy <eddy@localhost>
Installed-Size: $size
Depends: libc6 (>= 2.35), libgcc-s1, libegl1, libgl1, libxkbcommon0, libwayland-client0, libwayland-egl1, libx11-6, libxcursor1, libxi6, libxrandr2, systemd, udev
Section: utils
Priority: optional
Description: Lighting effects, profiles and fan control for the Corsair Commander Core XT
 Drives the Corsair iCUE Commander Core XT on Linux without iCUE:
 lighting effects (solid, breathing, rainbow, waves, temperature colours),
 per-part lighting, fan curves on CPU or case temperature, and profiles.
 .
 Contains ccxtd (user service), ccxtctl (command line) and ccxt-gui (window).
EOF
install -m755 packaging/debian/postinst packaging/debian/prerm packaging/debian/postrm "$stage/DEBIAN/"

dpkg-deb --root-owner-group --build "$stage" "target/debian/" >/dev/null
echo "target/debian/${pkg}_${version}_${arch}.deb"
