# Parabole — conventions

Rust IPTV player. Default UI: Slint (`ui-slint/` + `src/slint_ui/`, bin `parabole`). Design: `PLAN.md` (read only the section you need).

## Layout
- `src/model.rs` — shared plain data types. Changing it affects every module.
- `src/sources/` — detect pasted input, M3U parser, Xtream API, stream URL resolution. No UI, no DB.
- `src/db/` — SQLite storage and queries. No network, no UI.
- `src/player/` — external player launching (mpv / VLC / custom). No DB, no UI.
- `src/worker.rs` — background thread: runs network/DB jobs, talks to UI via Command/Event channels.
- `ui-slint/` + `src/slint_ui/` — Slint views and their Rust glue (feature `slint-ui`, on by default). Notes: `docs/slint-experiment.md`. No UI does network I/O; it sends Commands to the worker.

## Code rules (code must stay workable for models with small context windows)
- One concern per file. Keep files under ~300 lines; split into a submodule directory before they grow past that.
- A reader should understand a file from that file plus `model.rs`. Prefer explicit, boring code over clever generics or macros.
- Public functions get a one-line `///` doc only when the name alone doesn't say it. No other comments, except a one-line *why* where code can't show it.
- Shortcuts with a known limit: `// shortcut: <limit>, <when to upgrade>`.
- Errors: `anyhow::Result`, add `.context(...)` at I/O boundaries. No `unwrap()` outside tests.
- Never log or display Xtream credentials (they are inside stream URLs).
- Playlist content is untrusted input: validate URLs before handing them to a player; never run commands through a shell.
- Tests live next to the code (`#[cfg(test)] mod tests` or a sibling `tests.rs`); fixtures in `tests/fixtures/`.
- Before finishing: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` all clean (also with `--no-default-features` when shared code changed).
