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

## Comparison
(filled in phase 3: lines of UI code, clean build time, incremental build time, binary size, RSS at idle and with 290k streams loaded, idle CPU, scroll smoothness, and the dev-experience notes each agent collects as it goes)

## Dev-experience notes
(each agent appends 2–5 terse bullets: what was easier or harder than egui, and any Slint pitfalls hit)
