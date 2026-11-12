#!/bin/sh
# Builds dist/IPTV-Player-<version>-x86_64.AppImage with libmpv bundled (linuxdeploy). Linux only.
# Needs: libmpv-dev + eframe dev packages (see ci.yml), curl, and FUSE or APPIMAGE_EXTRACT_AND_RUN=1.
set -eu
cd "$(dirname "$0")/.."
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
ARCH=${ARCH:-x86_64}
export ARCH APPIMAGE_EXTRACT_AND_RUN=1

cargo build --release ${CARGO_FLAGS:-}
rm -rf dist AppDir && mkdir -p dist
TOOLS=$(mktemp -d)
curl -sSfL -o "$TOOLS/linuxdeploy" \
  "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-$ARCH.AppImage"
chmod +x "$TOOLS/linuxdeploy"

# linuxdeploy follows ldd, so libmpv and its dependencies land in AppDir/usr/lib.
# GL/EGL/Vulkan drivers are on linuxdeploy's excludelist and must come from the host.
"$TOOLS/linuxdeploy" --appdir AppDir \
  --executable target/release/iptv-player \
  --desktop-file packaging/iptv-player.desktop \
  --icon-file packaging/icon.png --icon-filename iptv-player \
  --output appimage
mv IPTV_Player-*.AppImage "dist/IPTV-Player-$VERSION-$ARCH.AppImage"
echo "built dist/IPTV-Player-$VERSION-$ARCH.AppImage"
