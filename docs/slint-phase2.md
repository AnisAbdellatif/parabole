# Phase 2 spec: three parallel areas

Base: `exp/slint-ui` after the scaffold commit. Each area runs in its own worktree on its own branch (`exp/slint-player`, `exp/slint-browse`, `exp/slint-dialogs`), and the orchestrator merges them.

The design is fixed. Implement it, and when the real code differs, adapt minimally and report what you changed. Port behaviour from the named egui files; they are the source of truth for what each feature does.

## Shared rules (all areas)
- CLAUDE.md applies: files under 300 lines, one concern each, no `unwrap` outside tests, never log URLs.
- **Scaffold (already in place):**
  - Each area has a global (`PlayerApi`, `BrowseApi`, `DialogApi` in `ui-slint/*-api.slint`), a host component (`player-pane.slint`, `browse-host.slint`, `dialog-host.slint`), and a Rust module `src/slint_ui/<area>/mod.rs` with `<Area>Area` state (a field on `SlintApp`), `connect(win)` and `on_event(a, ev) -> Option<Event>`.
  - `events::drain` routes every event to dialogs, then player, then browse, then the core match. Return `None` to consume an event, or `Some(ev)` to pass it on.
  - Keys go `DialogApi.key`, then `PlayerApi.key`, then `BrowseApi.key`, then the core shortcuts in `app.slint`.
  - Delete your area's `#[expect(dead_code)]` line in `state.rs` once you use the field.
- **Ownership: you may only touch:**
  - your own `*-api.slint` and host file;
  - new files named `ui-slint/<area>-*.slint`;
  - `src/slint_ui/<area>/**`;
  - the extra files listed under your area.
- **Shared files you may only append to:** `types.slint` (new structs at the end), `theme.slint` (new tokens), `icons.slint` (new icons), and `events.rs` (only the core-match arms your area replaces, and say so in the report). Nothing else that's shared.
- **Re-entrancy:** `with_app` uses `try_borrow_mut` and returns `None` when re-entered. Never trigger Slint callbacks synchronously from inside `with_app` (e.g. `invoke_*` on a component); defer with `slint::Timer::single_shot(Duration::ZERO, ...)`.
- **The egui app must stay unchanged in behaviour.** Allowed changes to `src/ui/**`: only visibility (`pub(crate)`), or splitting a function into a toolkit-neutral core plus a thin egui wrapper.
- **Loop:** `cargo check --features slint-ui` after each step, commit at the listed milestones, keep reasoning short.
- **Gates before the final commit:**
  - `cargo fmt`;
  - `cargo clippy --all-targets -- -D warnings` in the 3 configs (default, `--no-default-features`, `--features slint-ui`);
  - `cargo test`;
  - smoke-run with `HOME=/tmp/slint-<area>`, building first because setting HOME breaks rustup;
  - a window screenshot (`screencapture -l <windowid>`, not full screen) to `/tmp/slint-<area>.png`.
- Append to `docs/slint-experiment.md` under Gaps and Dev-experience notes, with each bullet prefixed by your area.
- **Final report:** commit SHAs, gate results, screenshot path, deviations, gaps, and the shared-file lines you touched.

---

## Area A: player (`exp/slint-player`)
Extra files you may touch: `src/player/embed/**`, and `src/ui/player/**` (visibility and neutral-core splits only).

**A1. Split the toolkit-neutral core out of `src/player/embed`** (milestone commit: "embed: toolkit-neutral MpvCore").
- New `embed/core.rs`: `pub struct MpvCore { mpv: Box<Mpv>, state: PlayerState, last: Option<PlayRequest> }`.
  - `pub fn new(settings: &PlayerSettings, wakeup: impl Fn() + Send + Sync + 'static) -> Result<Self>`: the current `EmbeddedPlayer::new` minus the video.
  - `state()`, `poll()`, `mpv() -> &Mpv`.
  - Move every `impl EmbeddedPlayer` method in `commands.rs` and `stats.rs` to `impl MpvCore`.
- `EmbeddedPlayer` becomes `{ video: VideoOutput, core: MpvCore }`; keep that field order, because the video must drop before mpv. Add `Deref`/`DerefMut<Target = MpvCore>` so no egui caller changes. `EmbeddedPlayer::new(cc, settings)` = `MpvCore::new(settings, repaint)` + `VideoOutput::new(core.mpv(), ...)`.
- Make these `pub(crate)` for reuse: `render.rs`'s `ProcAddress`, `proc_address`, `create_target`, `allocate_storage`, `alloc_size`.
- The egui app and its tests must stay green, and the existing tests must still pass.

**A2. `src/slint_ui/player/video.rs`, mpv to a GL texture in Slint.**
- At startup, `win.window().set_rendering_notifier(...)`:
  - `RenderingSetup` with `GraphicsAPI::NativeOpenGL { get_proc_address }`: build `glow::Context::from_loader_function` (use `eframe::glow`; no new dep) and the mpv `RenderContext`, mirroring `render.rs` `VideoOutput::new`. Set the update callback to `invoke_from_event_loop` → `window().request_redraw()`.
  - `BeforeRendering`: if mpv needs a frame, render into an FBO sized to the pane's physical size (`alloc_size` + `create_target`, reusing storage like `ensure_target`). Then publish `PlayerApi.frame = unsafe { BorrowedOpenGLTextureBuilder::new_gl_2d_rgba_texture(tex, size) }.build()` with the flip origin mpv needs.
  - **Save and restore the GL framebuffer binding and viewport around the mpv render.** femtovg expects its own state.
  - `RenderingTeardown`: drop the render context, then the GL objects.
- `PlayerApi`: `in property <image> frame`, `in property <length> video-w / video-h` (pane size → physical px for the FBO).
- If GL setup fails, fall back exactly as the egui path does: toast "Embedded player unavailable (...); using mpv" + `Command::LaunchRequest { kind: Mpv }`.

**A3. Session and events.**
- `on_event` consumes `PlayEmbedded(play)` → `MpvCore::play`, and keeps a `Session`. Reuse `crate::ui::player::session` by making it `pub(crate)`. Also handle the stop/failed paths. **Remove the core-match `PlayEmbedded` fallback arm in `events.rs`.**
- The mpv wakeup → `invoke_from_event_loop` → `core.poll()` → handle `Notice`s as `src/ui/player/mod.rs` does: end of file → `session::end_action` (retry, auto-next, stop), save position via `Command::SavePosition` (as `session.rs`/`mod.rs` do), track memory (reuse `track_memory`; split out a ctx-free tick if it takes `egui::Context`), and alternates (`alternate.rs`).
- Mirror the egui player's state into `PlayerApi` properties: `active`, `paused`, `title`, `position`, `duration`, `volume`, `muted`, `buffering`, `is-live`, track lists as `[TrackItem]` (append a struct to `types.slint`), and `error`.

**A4. Markup, ported from `src/ui/player/` `dock.rs`, `pane.rs`, `controls.rs`, `widgets.rs`, `fullscreen.rs`, `mini.rs`, `nerd_stats.rs`, `autonext.rs`, `zap.rs`.**
- `player-pane.slint` is the docked pane: right or bottom (toggle), with a draggable splitter, and width 0 when inactive.
- Inside it: an `Image { source: PlayerApi.frame; image-fit: contain }` plus overlay controls that auto-hide (play/pause, seek bar, volume and mute, tracks menu, stop, fullscreen, mini).
- Fullscreen: `win.window().set_fullscreen(true)` + the pane filling the window.
- Mini player (P), nerd stats overlay (I), the auto-next countdown card, and the digit-zap buffer display.
- Callbacks on `PlayerApi` for every control. Rust handlers live in `src/slint_ui/player/controls.rs`.

**A5. Keys (`PlayerApi.key`), ported from `src/ui/player/keys.rs` and `zap.rs`.**
- Space, arrows (seek and volume), M, F, P, I, digits for zap, channel up/down via `neighbor_row` (read `SlintApp.streams`), zap-back, and Esc (leave fullscreen or mini first).
- Return false when the player isn't active.

**Milestones:** A1 commit, A2 commit (video shows; screenshot while a local file plays, e.g. `tests/fixtures` media or an `ffmpeg -f lavfi` test pattern written to /tmp), A3–A5 commit.

---

## Area B: browse (`exp/slint-browse`)
Extra files you may touch: `sidebar.slint`, `list-header.slint`, `stream-list.slint`, `stream-row.slint`, `logo-tile.slint`, `src/slint_ui/{data,push,stream_model,callbacks}.rs` (core browsing glue), and `Cargo.toml` (only to add `arboard`, which is already in Cargo.lock via eframe).

Port from `src/ui/`:
1. **Sort + search filter chips:** `sort.rs` and `search_filter.rs`. Add a sort selector and filter chips in `list-header.slint`. Remove phase 1's re-implemented sort in `data.rs` in favour of `pub(crate)` reuse of `ui::sort` where it's egui-free.
2. **Row context menu:** `row_menu.rs`, `row_actions.rs`, `menu_data.rs` (`PopupWindow` or `ContextMenuArea`). It covers play with ▸ (player kinds), favorite, hide, copy URL (`Command::CopyUrl` → `UrlReady` → `arboard`; consume `UrlReady` in `on_event`), add to group, and alternates (`player/alternate.rs` logic stays in the player area; only the menu entries live here).
3. **Sidebar groups and category menu:** `sidebar_groups.rs`, `group_actions.rs`, `hide_new.rs`, and `sidebar_rows.rs`. The sidebar context menu covers hide, sort, and "hide matching…".
4. **Edit mode:** `edit_mode.rs`, `edit_toolbar.rs`, `edit_groups.rs`. Drag reorder can be shortcut to move up/down buttons (`// shortcut:` in the doc Gaps) if Slint DnD is impractical.
5. **Poster grid:** `poster_grid.rs` and `tile.rs`, for the Movies/Series tabs. Use a lazy `Model` like `StreamModel`, with rows of N tiles computed from the width.
6. **Series page + details:** `series_view.rs`, `details_view.rs`, `progress_marks.rs`, plus `episodes.rs` usage. Consume `SeriesLoaded` and `InfoLoaded` in `on_event`. Episodes play via `Command::PlayEpisode`.
7. **Empty and error states:** `empty_state.rs`, with all messages and action buttons. For buttons that open dialogs (Add source, Manage sources), call `DialogApi` callbacks named `open-sources()`. **Don't implement them**: declare nothing in `dialog-api.slint`, and instead set a `BrowseApi.request-dialog(string)` callback that the orchestrator wires at merge.
8. **Remaining nav keys (`BrowseApi.key`):** from `nav_keys.rs`. Cmd+D toggles favorite, Cmd+Left focuses the sidebar (Up/Down/Enter in it), and Esc leaves the series page.

**Milestones:** commit after 1–2, after 3–4, and after 5–8.

---

## Area C: dialogs (`exp/slint-dialogs`)
Extra file you may touch: `topbar.slint` (only to add the buttons that open dialogs: sources, settings, profiles).

`DialogApi` holds an `in-out property <string> open` (`""`, `"sources"`, `"settings"`, `"profiles"`, `"group"`), plus each dialog's fields and callbacks. Also add `callback open-dialog(string)` so the orchestrator can wire browse's `request-dialog` to it at merge. `dialog-host.slint` fills the window with a dimmed backdrop and the active dialog centered; clicking the backdrop or pressing Esc closes it (`DialogApi.key` handles Esc and Tab focus only while a dialog is open).

Port from `src/ui/`:
1. **Sources dialog:** `sources/{mod,list,form,account,ua_preset}.rs`.
   - List with edit/delete/refresh, an add/edit form with paste detection (`sources::detect`), and the UA preset dropdown.
   - Account panel: consume `AccountChecked`/`TestResult` in `on_event`.
   - Commands: `AddSource`, `UpdateSource`, `DeleteSource`, `Refresh`, `CheckAccount`, `TestSource`.
   - Pass `SourceChanged` through, because core reloads on it.
   - **Credentials:** password fields use `input-type: password`, and nothing is logged.
2. **Settings:** `settings_view.rs`. Covers player per media kind, mpv args, custom templates, auto-refresh hours and the rest, persisted with the same DB settings fns.
3. **Profiles:** `profiles_view.rs`. Create, rename, duplicate, delete and copy favorites between profiles. After changes, call `a.reload_all()`.
4. **Group dialog:** `group_dialog.rs`. Create/rename a custom group, opened via `open-dialog("group")` with context in `DialogApi` properties.
5. Shared dialog widgets (buttons, text field, dropdown, checkbox styled per `src/ui/kit.rs`/`theme/widgets.rs`) go in `ui-slint/dialog-kit.slint`.

**Milestones:** commit after 1, then after 2–5.
