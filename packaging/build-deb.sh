#!/bin/sh
# Build target/debian/coreglow_<version>_<arch>.deb with plain dpkg-deb (no cargo-deb needed).
set -eu
cd "$(dirname "$0")/.."

cargo build --release --locked
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
arch=$(dpkg --print-architecture)
pkg=coreglow
maintainer="Nexus-SvK <96940759+Nexus-SvK@users.noreply.github.com>"
stage=target/debian/${pkg}_${version}_${arch}
rm -rf "$stage"

install -Dm755 target/release/coreglowd      "$stage/usr/bin/coreglowd"
install -Dm755 target/release/coreglowctl    "$stage/usr/bin/coreglowctl"
install -Dm755 target/release/coreglow   "$stage/usr/bin/coreglow"
install -Dm755 target/release/coreglow-tray "$stage/usr/bin/coreglow-tray"
install -Dm644 packaging/coreglow-tray.desktop "$stage/etc/xdg/autostart/coreglow-tray.desktop"
install -Dm644 packaging/coreglowd.service   "$stage/usr/lib/systemd/user/coreglowd.service"
install -Dm644 packaging/70-coreglow.rules "$stage/usr/lib/udev/rules.d/70-coreglow.rules"
install -Dm644 packaging/coreglow.desktop "$stage/usr/share/applications/coreglow.desktop"
install -Dm644 packaging/coreglow.svg "$stage/usr/share/icons/hicolor/scalable/apps/coreglow.svg"
install -Dm644 packaging/io.github.nexus_svk.Coreglow.metainfo.xml "$stage/usr/share/metainfo/io.github.nexus_svk.Coreglow.metainfo.xml"
install -Dm644 assets/config.toml         "$stage/usr/share/coreglow/config.toml"
install -Dm644 README.md                  "$stage/usr/share/doc/$pkg/README.md"
for m in packaging/man/*.1; do
    install -Dm644 "$m" "$stage/usr/share/man/man1/$(basename "$m")"
    gzip -9n "$stage/usr/share/man/man1/$(basename "$m")"
done
ln -s coreglow.1.gz "$stage/usr/share/man/man1/coreglow-tray.1.gz"
install -Dm644 packaging/copyright        "$stage/usr/share/doc/$pkg/copyright"
# Debian wants a changelog (changelog.gz: a native package); the release notes live in docs/devlog-and-roadmap.md
printf '%s (%s) stable; urgency=medium\n\n  * Release %s: see docs/devlog-and-roadmap.md in the source.\n\n -- %s  %s\n' \
    "$pkg" "$version" "$version" "$maintainer" "$(date -R -d "@${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct 2>/dev/null || date +%s)}")" \
    | gzip -9n > "$stage/usr/share/doc/$pkg/changelog.gz"

# Libraries the binaries link against (dpkg-shlibdeps), plus the ones the window loads at
# run time (OpenGL/EGL, Wayland, X11, xkbcommon) that it can't see.
shlibs=$(
    tmp=$(mktemp -d)
    mkdir "$tmp/debian"
    printf 'Source: %s\n\nPackage: %s\nArchitecture: any\n' "$pkg" "$pkg" > "$tmp/debian/control"
    bins="$PWD/$stage/usr/bin"
    cd "$tmp" && dpkg-shlibdeps -O "$bins/coreglowd" "$bins/coreglowctl" "$bins/coreglow" "$bins/coreglow-tray" 2>/dev/null \
        | sed -n 's/^shlibs:Depends=//p'
    rm -rf "$tmp"
)
runtime="libegl1, libgl1, libxkbcommon0, libwayland-client0, libwayland-egl1, libx11-6, libxcursor1, libxi6, libxrandr2"

mkdir -p "$stage/DEBIAN"
echo /etc/xdg/autostart/coreglow-tray.desktop > "$stage/DEBIAN/conffiles"
size=$(du -ks "$stage/usr" | cut -f1)
cat > "$stage/DEBIAN/control" <<EOF
Package: $pkg
Version: $version
Architecture: $arch
Maintainer: $maintainer
Installed-Size: $size
Depends: ${shlibs:-libc6 (>= 2.35), libgcc-s1}, $runtime, init-system-helpers (>= 1.58), systemd, udev
Replaces: ccxt-control
Conflicts: ccxt-control
Provides: ccxt-control
Section: utils
Priority: optional
Homepage: https://github.com/Nexus-SvK/Coreglow
Description: lighting effects, profiles and fan control for Corsair Commander controllers
 Drives the Corsair iCUE Commander Core XT (and, experimentally, the
 Commander CORE of the ELITE CAPELLIX / ELITE LCD coolers) on Linux
 without iCUE: lighting effects (solid, breathing, rainbow, waves,
 temperature colours), per-part lighting, fan curves on CPU or case
 temperature, profiles and night dimming.
 .
 Contains coreglowd (user service), coreglowctl (command line),
 coreglow (window) and coreglow-tray (tray icon).
EOF
install -m755 packaging/debian/postinst packaging/debian/prerm packaging/debian/postrm "$stage/DEBIAN/"
(cd "$stage" && find usr -type f -print0 | sort -z | xargs -0 md5sum > DEBIAN/md5sums)

dpkg-deb --root-owner-group --build "$stage" "target/debian/" >/dev/null
echo "target/debian/${pkg}_${version}_${arch}.deb"
