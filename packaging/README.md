# Packaging

All scripts run from anywhere; output goes to `dist/` (git-ignored). Version comes from `Cargo.toml`.
Fonts, icons and the mpv Lua script are `include_bytes!`/`include_str!`'d into the binary, so packages ship only the executable plus libmpv and its libraries.

## macOS (.app + .dmg)

```sh
brew install mpv dylibbundler
packaging/macos.sh          # dist/Parabole.app + dist/Parabole-<ver>-macos-<arch>.dmg
packaging/verify-macos.sh   # no /opt/homebrew or /usr/local links left, signature valid
packaging/smoke-macos.sh    # launches the app with a scratch HOME for 6s
```

`dylibbundler` copies libmpv and its dependencies into `Contents/Frameworks` and rewrites their load paths to `@executable_path/../Frameworks`. Without APPLE_* env vars the bundle is ad-hoc signed (Gatekeeper will warn on other Macs).

## Windows (zip)

Needs Rust (MSVC), a Visual Studio developer shell (for `lib.exe`), and `7z`.

```powershell
./packaging/windows.ps1     # dist\Parabole-<ver>-windows-x64.zip (exe + libmpv-2.dll)
```

`packaging/windows-mpv.ps1` downloads the latest shinchiro `mpv-dev-x86_64` build and generates `mpv.lib` from its `.def`. No MSI (skipped: zip is enough for now).

## Linux (AppImage)

```sh
sudo apt-get install libmpv-dev   # plus the windowing dev packages listed in .github/workflows/ci.yml
packaging/linux.sh          # dist/Parabole-<ver>-x86_64.AppImage
```

GL/EGL/Vulkan drivers are intentionally not bundled; they come from the host.

## Flatpak

```sh
flatpak-builder --user --install --force-clean build-dir packaging/flatpak/dev.parabole.app.yml
```

Builds mpv (libass + ffmpeg) from source against `org.freedesktop.Platform//24.08`. Not built in CI. Uses network during the cargo build, so it is not Flathub-ready as is.

## Icon

`packaging/icon.png` is a generated placeholder (`python3 packaging/make_icon.py packaging/icon.png`). Replace it with real artwork; the scripts pick it up.

## CI and releases

- `.github/workflows/ci.yml`: fmt, clippy (default and `--no-default-features`) and tests on macOS, Linux, Windows.
- `.github/workflows/release.yml`: on tag `v*`, builds the three packages and uploads them to a **draft** GitHub release.

Release secrets (all optional; macOS signing/notarization is skipped without them, ad-hoc signed instead):

| Secret | Value |
|---|---|
| `APPLE_CERTIFICATE` | base64 of the Developer ID Application `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | password of that `.p12` |
| `APPLE_ID` | Apple ID email used for notarization |
| `APPLE_TEAM_ID` | 10-character team ID |
| `APPLE_APP_PASSWORD` | app-specific password for the Apple ID |
