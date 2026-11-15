# Parabole

Rust IPTV player with a Slint UI. Design notes: `PLAN.md`.

## Building

```sh
cargo run              # Slint UI with the embedded libmpv player (default)
cargo build --no-default-features  # library and tests only: no Slint, no libmpv
```

The embedded player (`embedded-mpv` feature) links against **libmpv**:

- macOS: `brew install mpv`. `build.rs` adds `/opt/homebrew/lib` and `/usr/local/lib` to the link search path.
- Linux: install `libmpv-dev` (or your distro's equivalent) so the linker finds `libmpv`.
- Windows: put `mpv.lib`/`libmpv-2.dll` where the linker can find them (see the `libmpv2` crate docs).

At runtime a missing or failing libmpv makes the app fall back to external mpv with a toast.
