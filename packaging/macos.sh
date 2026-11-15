#!/bin/sh
# Builds dist/Parabole.app and dist/Parabole-<version>-macos-<arch>.dmg. Needs: brew mpv dylibbundler.
# Signs with Developer ID + notarizes when APPLE_* env vars are set, else ad-hoc signs.
set -eu
cd "$(dirname "$0")/.."
NAME="Parabole"
BIN=parabole
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
APP="dist/$NAME.app"
DMG="dist/Parabole-$VERSION-macos-$(uname -m).dmg"

cargo build --release ${CARGO_FLAGS:-}
rm -rf dist && mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Frameworks" "$APP/Contents/Resources"
cp "target/release/$BIN" "$APP/Contents/MacOS/$BIN"

iconutil -c icns packaging/icons/macos.iconset -o "$APP/Contents/Resources/icon.icns"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>$NAME</string>
<key>CFBundleDisplayName</key><string>$NAME</string>
<key>CFBundleIdentifier</key><string>dev.parabole.app</string>
<key>CFBundleExecutable</key><string>$BIN</string>
<key>CFBundleIconFile</key><string>icon</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>$VERSION</string>
<key>CFBundleVersion</key><string>$VERSION</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>LSApplicationCategoryType</key><string>public.app-category.video</string>
</dict></plist>
PLIST

# No libmpv link (--no-default-features build) means nothing to bundle.
if otool -L "$APP/Contents/MacOS/$BIN" | grep -q libmpv; then
  SEARCH=""
  for d in /opt/homebrew/lib /usr/local/lib; do [ -d "$d" ] && SEARCH="$SEARCH -s $d"; done
  # shellcheck disable=SC2086
  dylibbundler -od -b -x "$APP/Contents/MacOS/$BIN" -d "$APP/Contents/Frameworks" -p @executable_path/../Frameworks $SEARCH
  # dylibbundler leaves duplicate LC_RPATH entries, which dyld rejects; deps already use absolute @executable_path.
  for f in "$APP"/Contents/Frameworks/*.dylib; do
    for rp in $(otool -l "$f" | awk '/LC_RPATH/{getline; getline; print $2}'); do
      install_name_tool -delete_rpath "$rp" "$f"
    done
  done
fi

if [ -n "${APPLE_CERTIFICATE:-}" ]; then
  KC="$RUNNER_TEMP/build.keychain"
  KCPASS=$(uuidgen)
  echo "$APPLE_CERTIFICATE" | base64 --decode > "$RUNNER_TEMP/cert.p12"
  security create-keychain -p "$KCPASS" "$KC"
  security set-keychain-settings -lut 3600 "$KC"
  security unlock-keychain -p "$KCPASS" "$KC"
  security import "$RUNNER_TEMP/cert.p12" -k "$KC" -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
  security set-key-partition-list -S apple-tool:,apple: -s -k "$KCPASS" "$KC" >/dev/null
  security list-keychains -d user -s "$KC" login.keychain
  IDENTITY=$(security find-identity -v -p codesigning "$KC" | sed -n 's/.*"\(Developer ID Application.*\)"/\1/p' | head -1)
  codesign --force --deep --options runtime --timestamp --entitlements packaging/entitlements.plist --sign "$IDENTITY" "$APP"
else
  codesign --force --deep --sign - "$APP"
fi

STAGE=$(mktemp -d)
cp -R "$APP" "$STAGE/" && ln -s /Applications "$STAGE/Applications"
hdiutil create -volname "$NAME" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null

if [ -n "${APPLE_CERTIFICATE:-}" ] && [ -n "${APPLE_ID:-}" ]; then
  codesign --force --sign "$IDENTITY" --timestamp "$DMG"
  xcrun notarytool submit "$DMG" --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_APP_PASSWORD" --wait
  xcrun stapler staple "$DMG"
fi
echo "built $DMG"
