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
sudo apt-get install libmpv-dev imagemagick   # plus the windowing dev packages listed in .github/workflows/ci.yml
packaging/linux.sh          # dist/Parabole-<ver>-x86_64.AppImage
```

GL/EGL/Vulkan drivers are intentionally not bundled; they come from the host.

## Flatpak

```sh
flatpak-builder --user --install --force-clean build-dir packaging/flatpak/dev.parabole.app.yml
```

Builds mpv (libass + ffmpeg) from source against `org.freedesktop.Platform//24.08`. Not built in CI. Uses network during the cargo build, so it is not Flathub-ready as is.

## Icon

`packaging/icon.svg` is the only source. `packaging/make_icons.sh` (needs inkscape) regenerates every derived file, which are committed: `packaging/icons/macos.iconset` (16–1024px with 1x/2x, art inset like other macOS icons; `macos.sh` builds the .icns), `packaging/icons/linux/` (16–512px; AppImage and Flatpak, plus the svg as the Flatpak scalable icon), `packaging/icons/windows/` + `packaging/icon.ico` (16–256px, embedded in the exe by `build.rs`), `packaging/icon.png` (1024px) and `assets/icon.png` (256px window icon).
