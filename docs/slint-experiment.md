# Slint UI experiment

Goal: port the egui UI (`src/ui/`, ~6.7k lines) to Slint, to compare the two toolkits. This is an experiment branch (`exp/slint-ui`). It is not meant to replace egui on main.

## Ground rules
- The egui app stays untouched and keeps building: `cargo run` = egui.
- The Slint app is a second binary behind the cargo feature `slint-ui`: `cargo run --features slint-ui --bin iptv-player-slint`.
- Rust glue lives in `src/slint_ui/`, with one concern per file and files under 300 lines (CLAUDE.md rules apply). `.slint` markup lives in `ui-slint/`, one component per file, and `app.slint` composes them.
- Reuse `model.rs`, `db/`, `sources/`, `player/` and `worker/` unchanged where possible. The UI talks to the worker only through `Command`/`Event`, exactly as egui does. If shared code is egui-coupled (e.g. `player/embed` takes `eframe::CreationContext`), split out a toolkit-neutral core and keep the egui path working.
- Visual target: `docs/redesign.html` and the tokens in `src/ui/theme/mod.rs`, mirrored as a Slint `global Theme`.
- Feature parity with the egui UI is the target. Anything skipped goes in the "Gaps" list below.
- Gates: `cargo fmt`, plus `cargo clippy --all-targets -- -D warnings` in all three configs (default, `--no-default-features`, `--features slint-ui`), plus `cargo test`.

## Phases
1. Foundation: deps, binary, theme, worker bridge, image loading, main window (top bar, tabs, sidebar, stream list, status bar, toasts), external-player playback.
2. In parallel:
   - (a) the embedded mpv player pane;
   - (b) the browse views: poster grid, series, details, empty states, menus, edit mode;
   - (c) the dialogs: sources, settings, profiles, groups.
3. Merge, then measure and write the comparison below.

## Gaps
(each agent appends skipped features here)
- Phase 1: edit mode, alternates, search filter chips, series/details views, row context menu, sort selector (saved sort is applied, not changeable), playing-row highlight, window-size/sidebar persistence.
- Phase 1: `UrlReady` (copy URL) ignored: no clipboard crate in deps. `PlayEmbedded` falls back to external mpv with a toast.
- Phase 1: `InfoLoaded`/`SeriesLoaded`/`TestResult`/`AccountChecked` ignored (phase 2).
- Phase 1: persisted state is saved only on window close.
- Dialogs: account panel is one coloured text line (not separate labelled fields); dropdowns use the std-widgets `ComboBox` (platform style, not `dialog-kit` styled); dialog card max body height is fixed at 600px (a binding to window height formed a layout loop).
- Dialogs: saving settings does not call the embedded player's `apply_mpv_args` (player area owns that); group dialog is only reachable via `DialogApi.open-dialog("group")` with the context properties set by browse.
- Dialogs: no automated UI test of clicks/typing (osascript click was not delivered); verified by screenshots of each dialog and unit tests for the form builder.
- Player: mini mode is not always-on-top (needs the `unstable-winit-*` slint feature); the window just shrinks to 480x270 and restores.
- Player: dock side and splitter size are not persisted; status-bar "Stop" (`Core.stop-playback`) only stops external players, the pane has its own stop; mpv args from Settings are read per play, not applied live.
- Player: `slint-ui` needs the default `embedded-mpv` feature (no `--no-default-features --features slint-ui` build). Fullscreen was not screenshotted (macOS Space changes the window id); mini/docked right/bottom/stats were.
- Player: key handling uses a `PlayerApi.hover` flag (pointer over video) in place of egui's hover rect; arrows seek/volume only act while the pointer is over the video or in fullscreen/mini.

## Comparison
(filled in phase 3: lines of UI code, clean build time, incremental build time, binary size, RSS at idle and with 290k streams loaded, idle CPU, scroll smoothness, and the dev-experience notes each agent collects as it goes)

## Dev-experience notes
(each agent appends 2–5 terse bullets: what was easier or harder than egui, and any Slint pitfalls hit)
- Phase 1: a lazy `Model` impl (`StreamModel`) gives virtualised lists for free; rows stay as `StreamRow` and `StreamItem` is built per visible row.
- Phase 1: all UI state lives behind one `thread_local` + `with_app`; callbacks and worker events re-enter it, so `try_borrow_mut` guards against re-entrancy.
- Phase 1: `ListView.viewport-y` is deprecated in 1.18; use `content-y`.
- Phase 1: images decode off-thread into `SharedPixelBuffer` (Send); `slint::Image` is not Send, so it is built on the UI thread.
- Phase 1: setting `HOME` breaks rustup; build first, then run the binary with `HOME=/tmp/...`.
- Dialogs: a `FocusScope` in `DialogHost` is needed for Esc, because the overlay is a sibling of the main `keys` scope and key events never bubble to it; focus is handed back to `keys` via a `closed` callback.
- Dialogs: `slint::include_modules!` only exports structs exported from `app.slint`, so new row structs must be added to its export line.
- Dialogs: a component cannot override a built-in `focus()`; use a differently named public function. `viewport-height` is deprecated for `content-height`.
- Player: mpv renders into its own FBO texture sized to the pane's physical px; `BorrowedOpenGLTextureBuilder` with `flip_y=false` + `BottomLeft` origin shows it upright. Rebuilding the `Image` only on resize is enough, since femtovg re-reads the texture each redraw.
- Player: femtovg keeps GL state, so the mpv render is wrapped in a save/restore of fbo, program, VAO, buffers, texture, viewport and the blend/scissor/depth/stencil/cull toggles.
- Player: `parent` is not allowed on a component's root element in `.slint`; position it at the instantiation site. A `PopupWindow` track menu and `Timer`-based control auto-hide (compare `animation-tick()`) both worked without Rust help.
- Player: `mod core;` next to the `core` crate works if you import with `self::core`; python's `http.server` can't seek, so test media needs `-movflags +faststart`.
