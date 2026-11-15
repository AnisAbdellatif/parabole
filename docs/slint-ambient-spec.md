# "Ambient" redesign spec (Slint only)

The visual target is `docs/slint-redesign-mockup.html`; open it in a browser. The egui app is untouched, and only the Slint UI changes. The Slint UI doesn't have to match egui any more.

The design is fixed. Implement it, and where the real code differs, adapt minimally and report what you changed.

**Rules for everyone:**
- CLAUDE.md applies: files under 300 lines, one concern each, no `unwrap` outside tests, never log URLs.
- Run `cargo check --features slint-ui` after each step.
- Gates before the final commit: `cargo fmt`, clippy `-D warnings` in all 3 configs, and `cargo test`.
- Smoke-run with `HOME=/tmp/slint-<stage>`, building first.
- Take window screenshots with `screencapture -l <id>`, where `<id>` comes from `swift /tmp/cmp/winid.swift <pid>`.
- Re-entrancy: never invoke Slint callbacks from inside `with_app`; defer with `Timer::single_shot(ZERO)`.
- Every animation duration goes through `Theme.motion`, or a multiple of it, so reduced motion turns animation off.

Stages: **S1 (one agent)** lays the foundation everyone builds on. Then **S2a, S2b and S2c run in parallel** on separate branches from S1.

---

## S1: runtime theme, shell and motion (`exp/ambient-s1`)

### S1.1 Runtime theme (`ui-slint/theme.slint`, new `src/slint_ui/appearance.rs`)
- `Theme` gets inputs that Rust sets:
  - `in-out property <color> accent` (default `#5eead4`) and `accent-2` (default `#818cf8`);
  - `in-out property <int> mode` (0 dark, 1 OLED, 2 light);
  - `in-out property <float> density` (0.85 compact, 1.0 cozy, 1.6 TV);
  - `in-out property <float> radius-scale` (default 1.0);
  - `in-out property <float> font-scale` (default 1.0);
  - `in-out property <bool> reduce-motion`.
- Derived values:
  - `out property <duration> motion: reduce-motion ? 0ms : 180ms`.
  - Every colour token becomes an expression on `mode`. For example, `bg: mode == 1 ? #000000 : mode == 2 ? #f5f7fa : #0b0f14`. Pick a coherent light palette with text, borders and surfaces inverted.
  - Every length token becomes `<base> * density`, and every radius token `<base> * radius-scale`.
  - New font-size tokens `fs-xs/sm/md/lg/xl/hero` are `<base> * font-scale * density`. Replace hard-coded `font-size: Npx` across `ui-slint/` with them, using the nearest token.
- Also import `../assets/fonts/DejaVuSans.ttf` so it's registered as a fallback for superscript glyphs like `ᵁᴴᴰ`. **Verify in a screenshot.** If Slint's fallback ignores registered fonts, report it; don't hack around it.
- `appearance.rs`:
  - `#[derive(Serialize, Deserialize, Clone)] pub struct Appearance { accent: u8, mode: u8, density: u8, radius_scale: f32, font_scale: f32, reduce_motion: bool }` with `Default`.
  - `pub const ACCENTS: [(u32, u32); 6]` holds the (accent, accent-2) pairs: teal/indigo, indigo/pink, pink/orange, orange/yellow, lime/teal, and slate/sky.
  - `pub fn apply(&self, win: &AppWindow)`, plus clamping.
  - Add `appearance: Appearance` to `Persisted` in `state.rs` (serde default), and call `apply` in `run()`.
  - Test: `apply` inputs are clamped, and serde round-trips.

### S1.2 Shell layout (`app.slint`, `topbar.slint` → replaced, new `ui-slint/shell-rail.slint`, `ui-slint/shell-header.slint`)
- Layout from left to right: **Rail** (56px collapsed) | **Sidebar** (categories, 220px × density) | **Main** (`ShellHeader` + `BrowseHost`) | `PlayerPane`. `PlayerPaneBottom` and `StatusBar` stay below.
- **Rail:** icon buttons, top to bottom:
  - Search: opens the palette. For now it calls `Core.open-palette()`, a new callback declared in `core.slint`. S1 leaves it unhandled; S2c handles it.
  - Live, Movies, Series: `Core.tab-selected`. The active one gets the accent gradient + glow (`drop-shadow-color: accent.with-alpha(0.35)`).
  - Favorites: `Core.sidebar-selected("fav")`.
  - A spacer.
  - Sources: `DialogApi.open-dialog("sources")`.
  - Profile avatar: a circle with initials that opens `"profiles"`.
  - Settings: opens `"settings"`.
  - On hover the rail expands to 180px *as an overlay* (it doesn't push the layout; draw it above with a shadow). Labels fade in, and both width and opacity animate with `Theme.motion`.
- **ShellHeader**, above the browse area, 52px × density:
  - the search field (move it from the old topbar);
  - the source filter `ComboBox` ("All sources" + each source). It sets `Core.source-index` and calls `Core.source-selected(int)`; the Rust side sets `state.source` and calls `reload_categories`. This closes a gap.
  - the sort selector, moved here from the list header.
- Delete `topbar.slint`. Its features (profile switcher, tabs, dialog buttons) now live in the rail and header.

### S1.3 Sidebar motion (`sidebar.slint`, `push.rs`)
- Selection pill: one `Rectangle` behind the ListView rows at `y: Core.selected-index * row-step + list.viewport-y`, with `animate y, height { duration: Theme.motion; easing: ease-out; }` and a 3px accent bar on its left.
- `push_sidebar` sets `Core.selected-index` (-1 hides the pill). Rows no longer draw their own selected background.
- Section labels (PINNED / MY GROUPS / CATEGORIES) use letter-spacing and `fs-xs`.

### S1.4 Living rows (`stream-row.slint`, `stream_model.rs`, `types.slint`)
- Hover: `animate x` by 3px, `drop-shadow-blur: 18px`, and the background goes to `surface`.
- Append `playing: bool` to `StreamItem`. Add `StreamModel::set_playing(Option<i64>)`, which notifies the old and new rows.
  - The playing id = the id last activated through `stream-activated`, stored in `SlintApp.playing_id`. Set it on `Event::Playing` and clear it on `PlayerStopped`, in `events.rs`.
- A playing row shows an equalizer: 3 bars with heights animated in a loop (`animate height { duration: 600ms; iteration-count: -1; easing: ease-in-out; }`, with staggered target heights driven by a looping `property <bool> phase` toggled by the animation). When `reduce-motion` is on, the bars are static.
- Watch progress: a 60px pill bar (accent when playing).

### S1.5 Toasts and skeleton (`toasts.slint`, new `ui-slint/skeleton.slint`, `data.rs`)
- Each toast slides in from the right: `init => { shown = true; }`, `x: shown ? 0 : 40px`, `opacity`, `easing: ease-out-back`, `duration: Theme.motion * 2`.
- **Idle fix:** start `toast_timer` only when the first toast appears, and stop it once the list is empty.
- Skeleton: while `Core.syncing` and the list is empty, show 8 placeholder rows. Each row is a rounded rectangle with a `@linear-gradient` whose highlight position animates in a loop (shimmer). Show it in `BrowseHost` in place of the empty state.

**S1 milestone commits:** after S1.1, after S1.2–S1.3, and after S1.4–S1.5. Screenshots go to `/tmp/ambient-s1-dark.png`, `-light.png` and `-tv.png`; set the mode via a temporary env hook and remove it before committing.

---

## S2a: immersive player (`exp/ambient-player`, based on S1)
Owns `ui-slint/player-*.slint` and `src/slint_ui/player/**`. In `app.slint` it may only add the two Window bindings below.

1. **Chrome:**
   - Top scrim (a `@linear-gradient` from black at 70% alpha to transparent): LIVE chip (red) or VOD chip, title, and tech info from `MpvCore` stats (resolution and audio channels; see `stats.rs`).
   - Bottom scrim: the seek bar with an accent gradient fill and a white thumb. On hover the thumb grows from 10 to 14px with an accent glow (`drop-shadow`), and a time tooltip follows the pointer.
   - Live streams show "−00:42 behind live" plus a "Go live" pill, using whatever live-latency value the egui player already computes. VOD shows elapsed / remaining time.
   - The chrome fades with `Theme.motion` after 3 s idle.
2. **Zap carousel:** on channel up/down or a digit zap, show a vertical list on the left: the 2 previous channels, the current one (an elevated card with number, logo and name), and the 2 next ones, taken from `SlintApp.streams` rows with logos from `Images`. It slides in from x −220 using `states` with `in`/`out` transitions, and fades out after 2 s. The digit buffer shows in the same card.
3. **Real picture-in-picture:** mini mode now sets `PlayerApi.pip`.
   - `AppWindow` binds `always-on-top: PlayerApi.pip; no-frame: PlayerApi.pip;`.
   - Rust saves the window size and position, then resizes to 480×270 at the bottom-right of the screen. Exiting PiP restores them.
   - In PiP only the video and a minimal hover overlay show: close, return, play/pause. Drag the window via a `TouchArea` that moves the window position.
4. Respect `Theme.reduce-motion` everywhere.

**Milestones:** commit after 1, then after 2–4. Screenshots: `/tmp/ambient-player.png` (chrome visible over a test pattern), `/tmp/ambient-zap.png`, `/tmp/ambient-pip.png`.

---

## S2b: ambient hero and posters (`exp/ambient-hero`, based on S1)
Owns `ui-slint/browse-*.slint`, `browse-host.slint`, `src/slint_ui/browse/**` and `src/slint_ui/images.rs`.

1. **`images.rs` size classes:**
   - `pub enum Size { Logo /*96*/, Poster /*360*/, Backdrop /*480 wide, blurred*/ }`, and `get(url, size)`. The key is `(size, url)`; keep the LRU cap per class or a shared byte budget.
   - Backdrop: decode, resize to 480 wide, apply `image::imageops::blur(sigma 14)`, then darken by 35%.
   - Each decode also computes a dominant colour: the saturation-weighted mean of pixels, skipping near-black and near-white. Expose it as `pub fn color(&self, url) -> Option<slint::Color>`.
   - Tests: the dominant colour of a synthetic two-colour image, and the cache key separation.
   - Posters in the grid and the series page use `Size::Poster`. This fixes the blurry posters.
2. **Hero** (new `ui-slint/browse-hero.slint`, at the top of `BrowseHost`, height 42% of the main area):
   - It shows the *focused* item: the cursor row or the hovered tile.
   - **Live:** a `@radial-gradient` from the logo's dominant colour (falling back to the accent) into `Theme.bg`, plus logo, name, category, a LIVE chip, progress when it's playing, and Watch and Favorite pill buttons.
   - **Movies/Series:** a backdrop `Image` (`Size::Backdrop`). Cross-fade it by keeping two layers and alternating them with `animate opacity`. Add a bottom scrim, title, and a meta line (year · duration · rating) plus a 2-line plot from `InfoLoaded`. Request it via `Command::LoadInfo` after the focus has stayed put for 300 ms, and cache it in `BrowseArea`. Show "Resume 1:12:40" when there's progress, otherwise "Play", plus a Details pill.
   - The hero collapses (animated height → 0) once the list or grid has scrolled more than 40px, and expands at the top.
3. **Poster wall:** tiles focused by hover or cursor scale to 1.06 (`transform-scale`, animated) with a 2px accent ring and a shadow. Resume progress is a 3px accent bar at the bottom edge. Columns stay responsive from the width, as today.

**Milestones:** commit after 1, then after 2–3. Screenshots: `/tmp/ambient-hero-live.png` and `/tmp/ambient-hero-movies.png` (seed a source whose fixture has logo URLs; if none can load offline, use a local `file://` or `http://127.0.0.1` image server under /tmp and kill it afterwards).

---

## S2c: command palette and appearance settings (`exp/ambient-palette`, based on S1)
Owns new `ui-slint/palette*.slint`, `ui-slint/dialog-appearance.slint`, `src/slint_ui/palette/**`, and `src/slint_ui/dialogs/appearance.rs`. In `app.slint` it may only add the `Palette { }` overlay line and put `PaletteApi.key(e)` first in the key chain. In `dialogs/mod.rs` it may add the `"appearance"` arm. In `callbacks.rs` it may handle `Core.open-palette`.

1. **Palette:**
   - Opens with Cmd/Ctrl+K or the rail's Search. It's a centred card (58% width) with a drop shadow and a translucent surface, and it animates in (opacity + y −8px → 0).
   - `PaletteApi`: `open`, `query`, `[PaletteItem { kind: int /*0 channel 1 vod 2 series 3 category 4 action*/, title, subtitle, icon: image }]`, `cursor`, callbacks `edited(string)`, `activate(int)`, `close()`.
   - Rust `palette/mod.rs`:
     - Debounce queries by 100 ms.
     - Sources, in order: streams via `db.search(profile, None, None, q, 20)` across all media kinds; categories whose name matches, current tab first; actions.
     - Actions: Refresh all sources, Switch theme mode, Cycle density, Toggle reduce motion, Toggle player dock, Open sources, Open settings, Open profiles, Open appearance, and Switch to profile <name> for each profile.
   - `palette/fuzzy.rs`: `pub fn score(query, text) -> Option<i32>`. Case-insensitive subsequence matching, with bonuses for word starts and consecutive runs. It ranks categories and actions; streams keep the DB order. Unit tests cover ordering and non-matches.
   - Keys: Up/Down move the cursor and Enter activates. Esc or clicking outside closes. Activating a stream sends `Command::Play`, a category switches tab if needed and selects it, and an action runs.
2. **Appearance dialog** (`DialogApi.open-dialog("appearance")`, also reachable from a Settings-dialog button):
   - 6 accent swatches in a selected ring, from `appearance::ACCENTS`.
   - Segmented controls for mode (Dark / OLED / Light) and density (Compact / Cozy / TV).
   - Sliders for corner radius (0.5–1.5) and font size (0.85–1.3), and a reduce-motion switch.
   - Every change applies live (`Appearance::apply`) and is persisted to `Persisted`, saved immediately.

**Milestones:** commit after 1, then after 2. Screenshots: `/tmp/ambient-palette.png` and `/tmp/ambient-appearance.png`.
