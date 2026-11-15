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
- browse: context-menu submenus (Play with / Add to group / Play from) are flattened into section headers; menu can overflow the window edge.
- browse: no connection-full warning (needs accounts from the dialogs area); list rows lack the edit-mode eye (tiles have it).
- browse: drag reorder is ▲▼ buttons; sidebar "Hide matching…" prefills the edit filter with the name prefix; filter chips don't wrap.
- browse: group dialog is `BrowseApi.request-dialog("group")` + `group-ctx-*` + `group-submit`; orchestrator must wire it to the dialogs area.
- browse: series page also fetches info on open (egui doesn't); verified only with fake data (no Xtream offline).
- browse: posters look blurry (`images.rs` THUMB=96, not browse-owned). Cmd+Left sidebar focus not verified via automation (Cmd+D verified).

## Comparison
Measured 2026-11-15 on an M-series Mac, macOS, both builds with embedded mpv. The runtime numbers come from a copy of the real DB (290,693 streams, 906 categories, auto-refresh off). The script is `/tmp/cmp/measure.sh` (not committed).

| | egui | Slint |
|---|---|---|
| UI code | 6,673 lines Rust (60 files) | 4,570 Rust (33) + 2,681 `.slint` (38) = 7,251 |
| Clean release build | 83 s | 145 s |
| Incremental debug build, one UI file touched | 1 s | 4 s (`.slint`), 3 s (Rust glue) |
| Crates in the dependency graph | 350 | 497 |
| Release binary, stripped | 20.8 MB | 23.8 MB |
| RSS, idle, after 10 s and after 40 s | 155 MB / 155 MB | 134 MB / 134 MB |
| CPU time, idle, 10 s to 40 s | 0.00 s | 0.10 s |

Notes:
- **Feature coverage:** the Slint port is not at parity yet (see Gaps). In particular, the Slint "items" status counts the whole tab, and egui counts the current category.
- **Text:**
  - Slint shapes Arabic RTL correctly (word order is right). egui renders Arabic words in reversed order, because it has no bidi.
  - Superscript letters (`ᵁᴴᴰ`) render once DejaVu is added to the fontique fallback chain (`slint_ui/fonts.rs`, `unstable-fontique-011` feature); a `.slint` font import alone only registers a named family.
- **Idle cost:** egui is fully idle when nothing changes. The Slint port wakes up a little, probably from the toast timer, which repeats even with no toasts.
- **Scrolling:** smoothness wasn't measured, because synthetic input doesn't reach the app on this machine.
- **Screenshots:** `/tmp/cmp/iptv-player.png` (egui) and `/tmp/cmp/iptv-player-slint.png` (Slint), same DB, same first category.

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

- browse: Slint maps Cmd to `modifiers.control` on macOS; a `meta||control` check covers both.
- browse: a child without x/y inside a Rectangle is centered; PopupWindow x/y can't be set from a function (bind to root properties).
- browse: osascript can't send modifier combos; a small CGEvent swift helper can.

## Ambient S2b
- Done: `images.rs` size classes (`Logo`/`Poster`/`Backdrop`, per-class LRU, dominant colour in `images/fx.rs`), hero (`browse-hero.slint`, `browse/hero.rs`), poster focus scale/ring/shadow, hero collapse on scroll.
- Gaps: Live hero shows a `PLAYING` chip, not a progress bar (no EPG data). Movie resume time is `duration × progress` only when the provider duration is `HH:MM:SS`; otherwise plain "Resume". M3U movies have no info, so the hero uses the blurred poster as backdrop. Hover focus is not cleared if a hovered tile is destroyed by scrolling until the pointer moves. Hover focus applies to tiles only; list rows use the cursor. Hover cross-fade, collapse and tile scale were not exercised interactively (no input synthesis); only static screenshots.
- Deviations: `stream-list.slint` got 3 lines (scroll-y/scroll-room forwarding to `BrowseApi`); `stream_model.rs` got a `set_reset_hook` so the hero rebuilds after `set_rows`.
- Dev experience:
  - A component inside a `VerticalLayout` whose height follows `root.height` creates a binding loop (layoutinfo); place it outside the layout with absolute `y` and offset the layout.
  - `changed` handlers only attach to a component's own properties; forward child state (scroll position) through a global.
  - `ListView.viewport-height` is deprecated for `content-height`; slint-build warnings only show when the build script fails.
  - `transform-scale` + `drop-shadow-*` on a `clip: true` rectangle work, but the ListView clips scaled tiles at its edges; inset tiles by the overflow.
  - A fake Xtream server (python, ~40 lines, generated PNG posters) on 127.0.0.1 plus a sqlite `insert into sources` is the quickest offline smoke seed; `pkill -f` can kill the invoking shell, use `killall`.

## Ambient S2c
Gaps / deviations:
- `PaletteItem` has no `icon` field: the row icon is picked from `kind` in Slint. `PaletteApi.close` is a public function, not a callback.
- Touched outside the owned files, all minimal: `worker/mod.rs` (`Command::Search` / `Event::SearchResults`), `ui/events.rs` (empty arm for the new event), `slint_ui/events.rs`, `state.rs` + `mod.rs` (`palette` field, `palette::connect`), `browse/mod.rs` (`pub use series::open as open_series`), `dialog-host.slint` and `dialog-settings.slint` (appearance dialog + button).
- `Core.open-palette` is handled in `palette::connect`, not `callbacks.rs`. Cmd+K is in `PaletteApi.key` (toggles).
- The overlay component is `CommandPalette`, because `Palette` clashes with the std-widgets global.
- Appearance sliders/switch use the std-widgets colours (blue), not `Theme.accent`. Persistence saves on every change (slider drag writes the JSON per tick).
- Not exercised: keyboard nav and activating results in the running app (no accessibility rights to inject keys); search, rendering and the appearance dialog were verified by screenshot via a temporary env hook (removed).

Dev experience:
- Callbacks invoked from inside `with_app` are silently dropped (`try_borrow_mut`); every action that fires a Slint callback needs a `Timer::single_shot(ZERO)`.
- A user component named `Palette` shadows the `std-widgets` global of the same name with "Cannot access id", an error that points at the wrong file.
- `ListView.viewport-y` is deprecated for `content-y`; the error says so, so the fix was quick.
- Animating a derived `t` (0..1) bound to `open` gives enter and exit animation without state machines, and `Theme.motion` = 0 makes it snap.
- Slint runs no synthetic input without OS accessibility rights, so an env-var smoke hook was the practical way to screenshot overlays.
- `HorizontalLayout` defaults to `cross-axis-alignment: stretch`, so a child with a fixed `height` sits at the TOP of the row (an explicit `y` there is overwritten by the layout). Set `cross-axis-alignment: center` on rows with mixed heights; children without a height then shrink to preferred, so use `cross-axis-self-alignment: stretch` where one must fill.

## Ambient S2a
Gaps: VOD chrome (seek bar thumb/tooltip, remaining time), the "Go live" pill (behind-live >= 3 s) and `reduce-motion` were implemented but not exercised in a smoke run. PiP from fullscreen waits a fixed 900 ms for macOS' fullscreen-exit animation (`// shortcut:` in `controls.rs`). `app.slint` also drops `min-width/min-height` while `PlayerApi.pip` (needed to shrink to 480x270).
- Player: `BorrowedOpenGLTextureBuilder` with `flip_y=false` showed the video upside-down on femtovg; `flip_y=true` is correct (checked against an ffmpeg source frame).
- Player: window APIs beyond `set_fullscreen`/position (`drag_window`, `current_monitor`) need the `unstable-winit-030` Cargo feature plus `WinitWindowAccessor`; `always-on-top`/`no-frame` are plain `Window` properties bindable from `.slint`.
- Player: `states` with `in`/`out` transitions on `x` + `opacity` give the zap carousel slide-in/out with no Rust timers other than the 2 s auto-close; `Theme.motion` = 0 collapses them.
- Player: Slint callbacks must not run inside `with_app`; the PiP/fullscreen window calls are deferred with `Timer::single_shot(ZERO)`.
- Smoke: keyboard tests need a list row clicked first, otherwise the search box keeps focus and the `keys` FocusScope never sees the key.
