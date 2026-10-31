#!/bin/sh
# Build target/debian/coreglow_<version>_<arch>.deb with plain dpkg-deb (no cargo-deb needed).
set -eu
cd "$(dirname "$0")/.."

cargo build --release --locked
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
arch=$(dpkg --print-architecture)
pkg=coreglow
stage=target/debian/${pkg}_${version}_${arch}
rm -rf "$stage"

install -Dm755 target/release/coreglowd      "$stage/usr/bin/coreglowd"
install -Dm755 target/release/coreglowctl    "$stage/usr/bin/coreglowctl"
install -Dm755 target/release/coreglow   "$stage/usr/bin/coreglow"
install -Dm644 packaging/coreglowd.service   "$stage/usr/lib/systemd/user/coreglowd.service"
install -Dm644 packaging/70-coreglow.rules "$stage/usr/lib/udev/rules.d/70-coreglow.rules"
install -Dm644 packaging/coreglow.desktop "$stage/usr/share/applications/coreglow.desktop"
install -Dm644 packaging/coreglow.svg "$stage/usr/share/icons/hicolor/scalable/apps/coreglow.svg"
install -Dm644 assets/config.toml         "$stage/usr/share/coreglow/config.toml"
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
Replaces: ccxt-control
Conflicts: ccxt-control
Provides: ccxt-control
Section: utils
Priority: optional
Homepage: https://github.com/Nexus-SvK/Coreglow
Description: Coreglow: lighting effects, profiles and fan control for the Corsair Commander Core XT
 Drives the Corsair iCUE Commander Core XT on Linux without iCUE:
 lighting effects (solid, breathing, rainbow, waves, temperature colours),
 per-part lighting, fan curves on CPU or case temperature, and profiles.
 .
 Contains coreglowd (user service), coreglowctl (command line) and coreglow (window).
EOF
install -m755 packaging/debian/postinst packaging/debian/prerm packaging/debian/postrm "$stage/DEBIAN/"

dpkg-deb --root-owner-group --build "$stage" "target/debian/" >/dev/null
echo "target/debian/${pkg}_${version}_${arch}.deb"
