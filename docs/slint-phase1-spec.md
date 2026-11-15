# Phase 1 spec: Rust glue for the Slint main window

The `.slint` markup (ui-slint/, `Core` global) exists and is committed (1932a0e). This spec covers the Rust side. Implement it file by file, in this order, and run `cargo check --features slint-ui` after each step. Report deviations; don't redesign. Every file stays under 300 lines.

## Step 0: make the markup compile
Get `cargo build --features slint-ui --bin iptv-player-slint` green with every component wired into `app.slint` (still no Rust logic). Commit: "slint: markup compiles".

## Step 1: `src/imgcache/mod.rs`, a toolkit-neutral entry point (small change)
Add one public fn, reusing `store` and `fetch`:
```rust
/// Cached bytes for `url`, downloading and storing them on a miss. Blocking; call off the UI thread.
pub fn get_or_fetch(dir: &Path, client: &reqwest::blocking::Client, url: &str) -> Result<Vec<u8>>
pub use fetch::client as http_client;
```
Also add `pub fn default_dir() -> Option<PathBuf>`, which returns `ProjectDirs(...).cache_dir().join("images")`. Make `boot.rs` `install_image_cache` use it. The egui path must stay unchanged in behaviour. Add one test: `get_or_fetch` returns the cached bytes without a network call when the file exists (use a tempdir and a bogus URL).

## Step 2: `src/slint_ui/state.rs`
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Sel { Favorites, Recent, Continue, Group(i64), Category(SourceId, String) }
impl Sel { pub fn to_id(&self) -> String; pub fn from_id(s: &str) -> Option<Sel> }
// ids: "fav" | "recent" | "continue" | "g:<id>" | "c:<source>:<key>" (key may contain ':', so split once after the source)
#[derive(Serialize, Deserialize, Default)] #[serde(default)]
pub struct Persisted { profile: Option<ProfileId>, tab: MediaKind, source: Option<SourceId>, sel: Option<Sel> }
impl Persisted { pub fn load(path) -> Self /* missing or bad file -> default */; pub fn save(&self, path) -> Result<()> }
pub struct SlintApp {
    pub db: Db, pub worker: worker::Handle, pub win: slint::Weak<AppWindow>,
    pub state: Persisted, pub state_path: PathBuf,
    pub profiles: Vec<Profile>, pub sources: Vec<Source>, pub categories: Vec<Category>, pub groups: Vec<Group>,
    pub streams: Rc<StreamModel>, pub search_text: String, pub search_applied: String,
    pub search_timer: slint::Timer, pub refreshing: HashSet<SourceId>,
    pub toasts: Rc<VecModel<ToastItem>>, pub next_toast: i32, pub images: Rc<Images>,
}
thread_local! { pub static APP: RefCell<Option<SlintApp>> = const { RefCell::new(None) }; }
pub fn with_app<R>(f: impl FnOnce(&mut SlintApp) -> R) -> Option<R>  // borrow_mut; None before init
```
The state file is `<data_dir>/slint-ui.json`, next to `db_path()`. Use the real Db/model types (`grep` for `Category`, `Group`, `Profile`, `StreamRow` in `src/model.rs` and `src/db`). Tests: a `Sel` id round-trip (including a key with ':') and loading a corrupt file falls back to default.

## Step 3: `src/slint_ui/images.rs`, the logo loader
```rust
pub struct Images { lru: RefCell<Lru>, pending: RefCell<HashSet<String>>, failed: RefCell<HashSet<String>>, tx: Sender<String> }
impl Images {
    pub fn new(on_ready: impl Fn(String, SharedPixelBuffer<Rgba8Pixel>) + Send + Sync + 'static) -> Rc<Self>
    /// Cached image or None; on a miss, queues a fetch (once per URL).
    pub fn get(&self, url: &str) -> Option<slint::Image>
    pub fn insert(&self, url: String, buf: SharedPixelBuffer<Rgba8Pixel>)  // UI thread
    pub fn fail(&self, url: String)
}
```
- 4 worker threads share an `Arc<Mutex<Receiver<String>>>`. Each one runs `imgcache::get_or_fetch`, then `image::load_from_memory`, then `thumbnail(96, 96)` to cap memory, then `to_rgba8`, then `SharedPixelBuffer::clone_from_slice`. It calls `on_ready` (which does `slint::invoke_from_event_loop`) or sends a failure the same way.
- Without a cache dir (`default_dir()` is None), every `get` returns None.
- The LRU is a `HashMap<String, (slint::Image, u64 tick)>`, capped at 600 entries; evict the oldest tick. Don't add a crate.
- Accept only http(s) URLs. Never log a URL.
- Test: LRU eviction order.

## Step 4: `src/slint_ui/stream_model.rs`, a lazy list model
```rust
pub struct StreamModel { rows: RefCell<Vec<StreamRow>>, notify: ModelNotify, images: Rc<Images>, show_source: Cell<bool>, source_names: RefCell<HashMap<SourceId, String>> }
impl Model for StreamModel { type Data = StreamItem; row_count; row_data(i) -> builds StreamItem on demand; model_tracker -> &self.notify }
impl StreamModel {
    pub fn set_rows(&self, rows: Vec<StreamRow>, show_source: bool, names: HashMap<..>)  // notify.reset()
    pub fn row(&self, i: usize) -> Option<StreamRow>
    pub fn image_ready(&self, url: &str)  // notify.row_changed for each row whose logo == url
}
```
- `row_data` uses `self.images.get(logo)` to fill `logo` and `has-logo`.
- `initials`: the first letters of the first two words, uppercased.
- `meta` and `progress`: copy the egui stream_row.rs logic (source prefix when `show_source`, watched fraction from the history columns).
- `image_ready` scans `rows`. That's fine: the egui lists hold at most ~tens of thousands of rows, and search is capped at 500.

## Step 5: `src/slint_ui/data.rs`, the reloads
Port `src/ui/data.rs` one-to-one as `impl SlintApp`: `toast`, `reload_all`, `reload_sources`, `reload_categories`, `reload_streams`, plus the sort load. Use the same DB calls and the same limits (RECENT 200, SEARCH 500). Skip edit mode (`edit.on` = false), alternates and the search filter chips; list them in Gaps. Each reload ends by calling the matching `push_*` from step 6.

## Step 6: `src/slint_ui/push.rs`, state to Core properties
- `push_profiles`: names, plus `profile-index`/`profile-name`.
- `push_sidebar`: section headers, the special entries (Favorites, Recent, Continue watching), groups, then categories. Use `kind`/`icon` per types.slint, skip hidden categories, and set `selected-id = sel.to_id()`.
- `push_streams`: model `set_rows`, `list-title`/`list-subtitle`, `empty-text`, and clamp `cursor`.
- `push_status`: `has-sources`, `syncing` (= refreshing non-empty), `status-sources`, `status-items`.

`Core.streams` is set ONCE at startup to `ModelRc::from(app.streams.clone())`, and the same applies to toasts.

## Step 7: `src/slint_ui/callbacks.rs`, one `pub fn connect(win: &AppWindow)`
Every handler is `|args| { with_app(|a| ...); }`.
- `tab-selected(i)`: set the tab (0 Live, 1 Movies, 2 Series), `sel = None`, `reload_categories`.
- `profile-selected(i)`: set the profile, then `reload_all`.
- `search-edited(s)`: store the text and `search_timer.start(SingleShot, 150ms, ...)` → `search_applied = text.trim()`, then `reload_streams`.
- `search-cleared`: clear both strings, then `reload_streams`.
- `sidebar-selected(id)`: `Sel::from_id` → `reload_streams`.
- `stream-activated(i)`: `worker.send(Command::Play { profile, stream_id: row.id, kind: None })`.
- `favorite-toggled(i)`: copy `src/ui/row_actions.rs` (`db.set_favorite`), then `reload_streams`.
- `toast-dismissed(id)`: remove the toast.
- `stop-playback`: `Command::StopPlayer`.

## Step 8: `src/slint_ui/events.rs`, `pub fn drain()` on the UI thread
`with_app`, then `while let Ok(ev) = a.worker.rx.try_recv()`, mirroring `src/ui/events.rs`:
- `SourceChanged` → `reload_sources`.
- `RefreshStarted`/`RefreshDone` → `refreshing` set, a toast on error, `reload_sources`, `push_status`.
- `Playing` → `playing-title`/`playing-kind`.
- `PlayerStopped` → clear them.
- `PlayFailed` and `Error` → toast.
- `UrlReady` → copy to the clipboard via `arboard` only if it's already a dependency; otherwise toast "copied" is skipped (Gap).
- `PlayEmbedded(play)` → for phase 1, the egui fallback: toast "Embedded player not yet in Slint UI; using mpv", then `Command::LaunchRequest { req: play.req, kind: PlayerKind::Mpv, track: play.track, stream_id: play.stream_id }`.
- `InfoLoaded`, `SeriesLoaded`, `TestResult` and `AccountChecked` → ignore for now; phase 2 owns them.

Toasts expire after 4 s via a single repeating `slint::Timer` (stored in `SlintApp`) that drops expired entries. Store the creation `Instant` next to each id in a side Vec.

## Step 9: `src/slint_ui/mod.rs`, `run()`
1. `db_path()` → `install_mpv_script` → `Db::open` → `ensure_default_profile`.
2. `AppWindow::new()`.
3. `worker::spawn(path, Arc::new(|| { let _ = slint::invoke_from_event_loop(events::drain); }))`.
4. `Images::new(on_ready)`, where `on_ready` invokes from the event loop: `with_app` → `images.insert`, then `streams.image_ready`.
5. Build `SlintApp` and put it in `APP`. `Persisted::load`. Set the models on `Core`. `callbacks::connect`. `reload_all`.
6. `worker.send(AutoRefresh(profile))` (as `boot.rs:98`).
7. `win.window().on_close_requested` → save Persisted + `worker.send(Command::Quit)` → `HideWindow`.
8. `win.run()`.

## Step 10: verify and commit
- Gates (see slint-experiment.md): fmt, clippy in all three configs, tests.
- Smoke test with `HOME=/tmp/slint-home`. Add a local M3U fixture as a source via a tiny one-off: `Db::open` + `add_source` in a test-only helper or sqlite3 CLI, whichever is simpler. Never touch the real DB.
- Screenshot to `/tmp/slint-p1.png`.
- Append Gaps and Dev-experience notes to `docs/slint-experiment.md`.
- Commit: "slint: phase 1 main window".
