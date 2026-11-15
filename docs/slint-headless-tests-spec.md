# Slint headless UI tests — spec

Goal: UI tests that never open a window, take focus or send OS input. Two layers:
- **Behaviour** (`i-slint-backend-testing`): find elements, click, type, check state. No pixels.
- **Snapshots** (software renderer): render screens to PNG and compare with goldens. Catches layout and alignment bugs.

No osascript, CGEvent or `screencapture` anywhere in this work.

## 0. Prerequisite refactor: `src/slint_ui/mod.rs`
Split `run()`:
- `pub fn build(db_path: &Path) -> Result<AppWindow>`: everything `run()` does now, from `install_mpv_script` through `wiring::connect` and the `with_app(reload_all…)` block, plus `on_close_requested`. It does not call `fonts::install_fallback()` or `win.run()`.
- `pub fn run() -> Result<()>`: `fonts::install_fallback(); let win = build(&db_path()?)?; win.run()`.
- `fonts::install_fallback` becomes `pub` (tests call it).
- `player::connect` / `video.rs`: when `set_rendering_notifier` returns `Err` (the software renderer and the testing backend have no OpenGL), skip the embedded video and continue. Never fail `build`. The `on_fail` path already exists; reuse it.
- `AppWindow` and the globals (`Core`, `PlayerApi`, `PaletteApi`, `DialogApi`, `BrowseApi`, `Theme`) are already `pub` via `include_modules!`. Don't add any other public API unless a test can't work without it. If one is needed, put it in `pub mod testkit` (`#[doc(hidden)]`, one file, under 80 lines).

## 1. Cargo.toml
```toml
[dev-dependencies]
i-slint-backend-testing = "=1.18.1"   # shortcut: unstable API pinned to slint's exact version, bump together
slint = { version = "1.18", default-features = false, features = ["renderer-software", "software-renderer-systemfonts"] }
```
- The slint dev-dep only adds features to the existing dependency during tests.
- Check the real feature names in `~/.cargo/registry/src/*/slint-1.18.1/Cargo.toml`. Check whether `i-slint-backend-testing` needs a feature for `ElementHandle` or for the event-loop init. Read its `src/lib.rs` docs in the registry; don't guess.
- `png` encoding: use the existing `image` dep (features `png`).

## 2. Shared seed: `tests/common/slint_seed.rs`
- `pub fn seed(dir: &Path) -> PathBuf` creates a scratch DB at `dir/library.db` with 1 profile and 1 M3U source whose URL is `http://127.0.0.1:9/none.m3u`, so a refresh fails fast with no network.
- It loads `tests/fixtures/slint-ui.m3u`, a new fixture with ~3 categories × 6 live channels plus 1 VOD category × 6 movies. Include one name with `ᵁᴴᴰ` and one Arabic name. Leave out logo and poster URLs, so nothing is fetched or cached.
- Insert it with the existing `Db` / m3u parse APIs; look at how `src/worker/tests.rs` and `src/db/tests/` seed data. Auto-refresh off.
- `pub fn isolate_home(dir: &Path)`: at the start of each test binary (before any thread), set `HOME` to `dir` (`unsafe { std::env::set_var }` in edition 2024, with a one-line why comment). Image cache and config paths then never touch the real user dirs.
- Use `std::env::temp_dir().join(format!("iptv-slint-{name}-{pid}"))`, removed at the end. No new tempfile dep.

## 3. Behaviour tests: `tests/slint_behaviour.rs`
- Start with `#![cfg(feature = "slint-ui")]`.
- Use ONE `#[test]` function. The backend is initialised once per process and Slint objects are `!Send`, so it runs the cases below in sequence and names each one in its assert messages.
- Call `i_slint_backend_testing::init_integration_test_with_system_time()`, then `build(seeded_path)`, then `win.show()`.
- Run the cases inside `slint::spawn_local(async { …; slint::quit_event_loop() })` + `slint::run_event_loop()`, so worker events (`invoke_from_event_loop`) and the deferred `Timer::single_shot(ZERO)` callbacks run.
- Write a small `async fn settle(ms)` built on a `slint::Timer` future, plus `wait_until(cond, timeout)`, which polls every 10 ms and fails with a message after 3 s.

Cases (find elements with `ElementHandle::find_by_element_id(&win, "Component::id")`; add `id`s in `.slint` where missing):
1. Startup: `Core.streams` row count > 0 after `wait_until`, and the first category is selected.
2. Sidebar: clicking the 2nd category row changes the selection and the stream rows.
3. Palette:
   - Cmd+K opens it (dispatch the key through the window or the key `FocusScope`, as the app's key chain expects).
   - Type "chan"; results arrive from the worker.
   - Down + Enter activates a result, the palette closes, and the selection or playing state changes as `palette` defines.
   - Esc closes it.
4. Appearance:
   - Open the dialog (via `DialogApi`, or the Settings button by id).
   - Click the 2nd accent swatch, and `Theme.accent` changes.
   - Toggle reduce-motion, and `Theme.motion` becomes 0.
   - The setting is persisted to the state file in the scratch dir.
5. Dialog focus: open Settings, press Esc, the dialog closes, and keys reach the main key scope again (Cmd+K opens the palette).
6. Profile switch via palette: add a 2nd profile in the seed. The "Switch to profile <name>" action changes the active profile.

Playback needs mpv, so it is out of scope. Note it in the gaps.

## 4. Snapshot tests: `tests/slint_snapshots.rs` + `tests/common/sw_platform.rs`
- `#![cfg(feature = "slint-ui")]`, one `#[test]` function.
- **`sw_platform.rs`** (under 150 lines), a custom `slint::platform::Platform`:
  - `create_window_adapter` returns one `MinimalSoftwareWindow` (`RepaintBufferType::NewBuffer`), sized 1280×800 logical at scale factor 2.0 (sharp crops).
  - `duration_since_start` returns a manually advanced clock (`Cell<Duration>`), so animations are deterministic.
  - `new_event_loop_proxy` returns a proxy that pushes closures into an `Arc<Mutex<VecDeque<…>>>`. `invoke_from_event_loop` works from the worker and image threads.
  - `pump(&self, rounds)`: run queued closures, advance the clock 16 ms, `slint::platform::update_timers_and_animations()`, repeat. Use `pump_until(cond)` with a 3 s wall-clock timeout for worker results.
  - `render() -> image::RgbaImage`: `window.draw_if_needed` into a `SharedPixelBuffer<Rgb8Pixel>` (or `PremultipliedRgbaColor`), then convert.
- Before `build`: `isolate_home`, `slint::platform::set_platform`, `fonts::install_fallback()`. Force reduce-motion on in the seeded appearance state, so no mid-animation frames.
- Screens, each one a PNG in `tests/fixtures/slint-snapshots/<name>.png`:
  `live-dark`, `live-light`, `movies-wall`, `palette-chan`, `appearance`, `settings`, `sources`, `profiles`, `player-chrome`, `zap-carousel`, `toast`.
  - Open overlays and dialogs through the globals (`DialogApi.invoke_open_dialog(...)`, `PaletteApi`/`Core` callbacks) or dispatched key events.
  - `player-chrome` / `zap-carousel`: set `PlayerApi` properties directly (title, tag, tech, is-live, zap items) with no mpv. The video area is black.
- **Compare:** a pixel differs if any channel differs by more than 8. Fail if more than 0.3% of pixels differ.
  - On failure, write `target/slint-snapshots/<name>.actual.png` and `<name>.diff.png` (differing pixels in red), and include those paths in the panic message.
  - `UPDATE_SNAPSHOTS=1` rewrites the goldens.
  - A missing golden is written and the test fails once with "new snapshot".
- **Alignment asserts** (cheap, no goldens needed): for the player controls row and one sidebar row, find the bounding rows of non-background pixels of each control in a crop. Assert their vertical centres are within 1 physical px. Keep it simple: column bands at known x ranges from the layout.

## 5. Done when
- `cargo test --features slint-ui` runs both binaries headless: no window appears, no focus change. Time it and report.
- Goldens are committed. Review each PNG visually (Read the file) and report anything misaligned or clipped. Do not "fix" UI in this task; list it.
- Gates: `cargo fmt`; `cargo clippy --all-targets -- -D warnings` in all three configs; `cargo test` (default) and `cargo test --features slint-ui`.
- `docs/slint-experiment.md`:
  - Replace the "no automated UI test" gap lines with a "Headless UI tests" section: how to run, how to update goldens, limits (the software renderer differs from femtovg; no mpv video).
  - Add 2–5 dev-experience bullets.
