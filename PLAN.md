# IPTV Player — Plan (Rust + egui)

Cross-platform (macOS / Windows / Linux) desktop IPTV player. Playback through embedded mpv, VLC, or any external player.
Inputs: Xtream Codes, M3U/M3U8 URL, M3U file. Stalker portals come later.
Main goal: browse, organize and play. The UI must handle big real-world provider lists (100k+ entries, messy data), multiple profiles and playlists, show/hide/reorder categories, and favorites.

EPG, catch-up, recording, metadata and Stalker are planned for later. The schema and module layout leave room for them; none of them get built now.

---

## 1. Prior art: what to take, what to avoid

| Project | Stack / license | Take | Avoid |
|---|---|---|---|
| **Fred TV (open-tv)** github.com/Fredolx/open-tv | Rust + Tauri, GPL-2 | SQLite cache of the whole provider list → instant search. Per-channel HTTP headers parsed from `#EXTVLCOPT`. Tolerant JSON helpers for Xtream (`get_serde_json_string/u64`). Series episodes loaded lazily (`get_series_info` on open). Per-source `use_tvg_id`. "Stream caching" toggle. Hotkeys (Ctrl+A all / Ctrl+S categories / Ctrl+D favorites / Ctrl+F search). Live/VOD/Series can each be toggled off. | It stores `favorite` **on the channel row**, so every refresh puts favorites at risk. Its migration history shows repeated dedupe/unique-index rewrites to cope. → We keep user data in a separate overlay keyed by provider IDs (§4). |
| **IPTVnator** github.com/4gray/iptvnator | Electron/Angular, MIT | Source types M3U + Xtream + Stalker. External MPV/VLC/IINA presets. Favorites and "continue watching" across sources. Command palette (Ctrl/Cmd+K). Recently viewed. | Electron weight. |
| **Hypnotix** (Linux Mint) | Python/GTK + libmpv, GPL-3 | Three top-level tabs: Live / Movies / Series. A free-text "mpv options" field in settings (users need it, e.g. `vo=x11` on Wayland). Ships with a legal default demo provider (Free-TV/IPTV), useful for testing. | — |
| **egui-mpv-glow** github.com/Trained-Monkey-Studios/egui-mpv-glow | Rust, MIT | Working reference for libmpv render API → GL texture → `egui::Image`, with hardware decode and zero CPU copy. Controls can draw on top of the video. Version is locked to the egui version. | Depending on it directly (it lags egui releases). Vendor/adapt its ~few hundred lines instead. |
| **Cinebox** (reddit r/rust, Oct 2026) | Rust + egui glow + libmpv | Confirms that mpv under an egui UI via the glow backend works on Win/Linux. The earlier iced/wgpu attempt had to put a separate mpv window inside the app window, with no overlay. → **egui must use the `glow` renderer, not wgpu.** | — |

License note: Fred TV is GPL-2 and Hypnotix GPL-3, so reuse their *ideas* and do not copy their code unless our project is GPL. libmpv is GPL unless built with `-Dgpl=false`. If we bundle a stock libmpv, **the app license should be GPL-3**. Decide this up front (recommended: GPL-3).

---

## 2. Stack

| Concern | Choice | Why |
|---|---|---|
| UI | `eframe`/`egui` 0.37, **`glow` renderer** | mpv's render API is OpenGL. `wgpu` would mean a separate window and no overlay. |
| Embedded video | `libmpv2` 7.x (render API) | Maintained, current. Pattern taken from egui-mpv-glow. |
| VLC | **External process only** | `vlc-rs` is dead (2018). Embedding libvlc in egui means CPU frame copies through `libvlc_video_set_callbacks`. Not worth it while mpv is embedded. |
| HTTP | `reqwest` 0.13 (rustls, gzip, stream) on a `tokio` runtime in a worker thread | Streaming download of large playlists, concurrent requests, gzip. |
| DB | `rusqlite` 0.42 (`bundled`, FTS5) + `rusqlite_migration` | One file, instant search, survives restarts. Fred TV shows this scales. |
| JSON | `serde` / `serde_json` + custom lenient deserializers | Xtream returns numbers as strings, `""` for null, and so on (§5.2). |
| Images (logos/posters) | `egui_extras::install_image_loaders` (`http` + `image` features) | Built in; loads only visible rows. `shortcut:` no disk cache. Upgrade to a hashed-URL disk cache in M6 if logo reloads hurt. |
| Paths | `directories` 6 | Platform config/data dirs. |
| Drag reorder | `egui_dnd` 0.18 | Category / favorite reordering. |
| Arg splitting | `shlex` | Custom player command templates **without a shell** (§7.4). |
| XML (later, EPG) | `quick-xml` (streaming) + `flate2` | XMLTV is often 100MB+ gzipped. |
| Secrets (optional) | `keyring` 4 | Off by default: credentials also end up in every stream URL, so the keyring adds little. Opt-in setting. |

Single crate (`lib.rs` + `main.rs`). The lib holds everything that is not UI, so it can be tested headless. No workspace until something needs it.

---

## 3. Architecture

```
┌──────────────── UI thread (eframe::App::update) ────────────────┐
│ AppState: profile, current view, cached rows for visible list  │
│ reads: own rusqlite read-connection (WAL), small indexed queries │
│ sends Command ─────────────┐        ┌──── drains Event each frame │
└────────────────────────────┼────────┼─────────────────────────────┘
                             ▼        │ (std::sync::mpsc + ctx.request_repaint())
┌──────────── Worker thread (tokio runtime) ────────────┐
│ network: playlist download, Xtream API, series info   │
│ import: parse → bulk insert in one transaction        │
│ write connection (single writer)                       │
│ player supervisor: spawns/IPCs external players        │
└────────────────────────────────────────────────────────┘
Embedded mpv: libmpv handle owned by UI side; renders inside the egui glow paint callback.
mpv wakeup/update callbacks → ctx.request_repaint().
```

```rust
enum Command { AddSource(SourceInput), Refresh(SourceId), DeleteSource(SourceId),
               LoadSeries(SourceId, String), Play(PlayRequest), StopExternal, /* … */ }
enum Event   { Progress(SourceId, Stage, u64), SourceDone(SourceId, Result<Stats, String>),
               SeriesLoaded(..), PlayerExited(..), Error(String) }
```

Module layout:
```
src/
  main.rs              eframe::run_native (glow), fonts, image loaders
  lib.rs
  app.rs               App, top-level layout, event drain
  model.rs             MediaKind{Live,Movie,Series}, Source, Category, Stream, PlayRequest
  worker.rs            Command/Event loop
  db/mod.rs            open(), migrations, queries
  db/schema.sql
  sources/detect.rs    classify pasted input (§5.4)
  sources/m3u.rs       streaming parser
  sources/xtream.rs    API client, lenient types, URL builders
  player/mod.rs        PlayerKind, dispatch
  player/external.rs   mpv(IPC)/vlc/custom
  player/mpv_embed.rs  libmpv render ctx → egui texture
  ui/{topbar,sidebar,list,series,player_view,source_dialog,settings,profiles}.rs
tests/fixtures/        m3u + xtream json samples (§10)
```

---

## 4. Data model: provider data vs user overlay

**Rule:** a refresh replaces provider tables wholesale. User choices (favorites, hidden, order, history) live in overlay tables keyed by **natural keys** that stay the same across refreshes, never by row ids. Favorites survive refreshes, and a favorite whose channel disappears for a while comes back when the channel returns. The UI shows such favorites greyed out as "missing".

Natural keys:
- Xtream category: `category_id`. Xtream stream: `stream_id` (live/vod) or `series_id`.
- M3U category: the `group-title` string.
- M3U stream: `tvg-id` if present and unique within the source. Otherwise `name + "\x1f" + group`. If still duplicated, append `#n` (occurrence index).
  The URL is not used as a key: many providers put rotating tokens in it.

```sql
-- identity / config
CREATE TABLE profiles (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, color INTEGER, pin_hash TEXT);
CREATE TABLE sources (
  id INTEGER PRIMARY KEY, name TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('xtream','m3u_url','m3u_file')),   -- 'stalker' later
  url TEXT NOT NULL, username TEXT, password TEXT,
  user_agent TEXT, output_format TEXT,            -- xtream: 'ts'|'m3u8', NULL = auto from allowed_output_formats
  epg_url TEXT,                                   -- from #EXTM3U url-tvg or manual; used later
  account_json TEXT,                              -- xtream user_info/server_info snapshot (expiry, max_connections)
  last_refresh INTEGER, last_error TEXT
);
CREATE TABLE profile_sources (profile_id INTEGER REFERENCES profiles ON DELETE CASCADE,
  source_id INTEGER REFERENCES sources ON DELETE CASCADE, position INTEGER NOT NULL, enabled INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (profile_id, source_id));

-- provider data (DELETE + re-INSERT per source on refresh, in one transaction)
CREATE TABLE categories (id INTEGER PRIMARY KEY, source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  media TEXT NOT NULL, ext_key TEXT NOT NULL, name TEXT NOT NULL, provider_pos INTEGER NOT NULL,
  UNIQUE (source_id, media, ext_key));
CREATE TABLE streams (id INTEGER PRIMARY KEY, source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  media TEXT NOT NULL, ext_key TEXT NOT NULL, category_key TEXT, name TEXT NOT NULL,
  num INTEGER, logo TEXT,
  url TEXT,                    -- m3u only; xtream URLs are built at play time from creds (password change ≠ refresh)
  container_ext TEXT,          -- xtream vod
  epg_id TEXT, archive_days INTEGER,       -- stored now, used later (EPG/catch-up)
  headers_json TEXT,           -- {user_agent, referrer, origin} from #EXTVLCOPT / url|headers
  extra_json TEXT,             -- rating, added, plot… (cheap to keep, avoids migrations)
  provider_pos INTEGER NOT NULL,
  UNIQUE (source_id, media, ext_key));
CREATE INDEX streams_cat ON streams(source_id, media, category_key, provider_pos);
CREATE VIRTUAL TABLE streams_fts USING fts5(name, content='streams', content_rowid='id', tokenize='unicode61 remove_diacritics 2');
CREATE TABLE series_cache (source_id INTEGER REFERENCES sources ON DELETE CASCADE, series_key TEXT,
  json TEXT NOT NULL, fetched_at INTEGER, PRIMARY KEY (source_id, series_key));

-- user overlay (never wiped by refresh)
CREATE TABLE category_prefs (profile_id INTEGER REFERENCES profiles ON DELETE CASCADE, source_id INTEGER REFERENCES sources ON DELETE CASCADE,
  media TEXT, cat_key TEXT, hidden INTEGER NOT NULL DEFAULT 0, position INTEGER, fav_position INTEGER,
  PRIMARY KEY (profile_id, source_id, media, cat_key));
CREATE TABLE stream_prefs (profile_id INTEGER REFERENCES profiles ON DELETE CASCADE, source_id INTEGER REFERENCES sources ON DELETE CASCADE,
  media TEXT, stream_key TEXT, hidden INTEGER NOT NULL DEFAULT 0, fav_position INTEGER,
  PRIMARY KEY (profile_id, source_id, media, stream_key));
CREATE TABLE history (profile_id INTEGER REFERENCES profiles ON DELETE CASCADE, source_id INTEGER REFERENCES sources ON DELETE CASCADE,
  media TEXT, stream_key TEXT, watched_at INTEGER NOT NULL, position_s REAL, duration_s REAL,
  PRIMARY KEY (profile_id, source_id, media, stream_key));
CREATE TABLE settings (profile_id INTEGER, key TEXT, value TEXT NOT NULL, PRIMARY KEY (profile_id, key)); -- profile_id NULL = global
```

Category ordering in one query: custom order first, then new or unordered categories in provider order.
```sql
SELECT c.*, p.hidden, p.fav_position FROM categories c
LEFT JOIN category_prefs p ON p.profile_id=?1 AND p.source_id=c.source_id AND p.media=c.media AND p.cat_key=c.ext_key
WHERE c.source_id=?2 AND c.media=?3 AND (?4 OR COALESCE(p.hidden,0)=0)
ORDER BY p.position IS NULL, p.position, c.provider_pos;
```
Reordering rewrites `position` for that (profile, source, media) in one transaction. With at most a few hundred categories, renumbering is trivial and needs no gap-numbering scheme.

`shortcut:` one category per stream (newer Xtream panels send a `category_ids` array; we take the first). Upgrade to a join table if users report channels missing from secondary categories.

Profiles: a profile is a viewer persona. It picks which sources it sees (`profile_sources`) and has its own hidden/order/favorites/history. Example: a "Kids" profile on the same source with adult categories hidden. The first run creates a "Default" profile.

---

## 5. Inputs

### 5.1 M3U / M3U8 parser (`sources/m3u.rs`)
Hand-written, streaming over `BufRead` lines (reqwest byte stream → `tokio_util::io::StreamReader` → lines, or a file). No regex. Bulk insert with prepared statements in one transaction. Target: 100k entries in under 2s.

Must handle:
- UTF-8 BOM, CRLF, blank lines, leading whitespace. Invalid UTF-8 → lossy decode (some lists are Latin-1).
- Header `#EXTM3U url-tvg="…" x-tvg-url="…" tvg-shift=…` → `sources.epg_url` (comma-separated lists allowed).
- `#EXTINF:<dur> attr="v" attr2='v' attr3=v,Display Name`:
  - Attribute scanner with quote awareness. The title starts after the first comma **outside quotes**, because logo URLs and names contain commas.
  - Keys are case-insensitive: `tvg-id tvg-name tvg-logo group-title tvg-chno channel-number tvg-shift radio catchup catchup-days catchup-source timeshift`.
  - Name precedence: display name after the comma, then `tvg-name`. Fred TV prefers `tvg-name`; make it a per-source toggle only if users ask.
  - `group-title` may hold `;`-separated multiple groups → take the first (same shortcut as above).
- `#EXTGRP:Group` (applies when there is no `group-title`).
- `#EXTVLCOPT:http-user-agent=… / http-referrer=… / http-origin=…` → headers_json.
- `#KODIPROP:inputstream.adaptive.license_type=…` → mark the stream DRM (show a lock icon; mpv can't play Widevine).
- URL with pipe headers: `http://x/y.ts|User-Agent=Foo&Referer=http://bar` → split off and merge into headers.
- Entries without a group → "Uncategorized".
- Media classification (a get.php M3U contains VOD too): URL path contains `/movie/` → Movie, `/series/` → Series episode (flat; grouped by group-title), extension `.mp4|.mkv|.avi` → Movie, else Live.
- **An `.m3u8` that is an HLS media/master playlist (`#EXT-X-`) is a single stream, not a channel list.** The detector (§5.4) handles it.

### 5.2 Xtream Codes (`sources/xtream.rs`)
Endpoints (`{base}/player_api.php?username=U&password=P&action=…`):
| Call | Use |
|---|---|
| *(no action)* | `user_info{auth,status,exp_date,max_connections,active_cons,allowed_output_formats}` + `server_info{url,port,https_port,server_protocol,timezone}`. `auth==0` → bad credentials. |
| `get_live_categories` / `get_vod_categories` / `get_series_categories` | categories (`category_id`, `category_name`, `parent_id`) |
| `get_live_streams` / `get_vod_streams` / `get_series` | full lists. Fetch all once (no `category_id` param) and cache, so search works globally. |
| `get_series_info&series_id=` | seasons + episodes, lazy on open, cached in `series_cache` |
| `get_vod_info&vod_id=` | details pane (plot, cast, duration). Lazy, optional. |
| `get_short_epg` / `get_simple_data_table` | later (EPG) |

Stream URLs (built at play time, with user/pass percent-encoded as path segments):
- Live `{base}/live/{u}/{p}/{stream_id}.{ts|m3u8}`. Pick `ts` when allowed, else `m3u8`; per-source override.
- Movie `{base}/movie/{u}/{p}/{stream_id}.{container_extension}`
- Episode `{base}/series/{u}/{p}/{episode_id}.{container_extension}`
- Later, catch-up: `{base}/timeshift/{u}/{p}/{minutes}/{YYYY-MM-DD:HH-MM}/{stream_id}.ts`

`base` = what the user typed (scheme://host:port), normalized: trim trailing `/` and strip `player_api.php`/`get.php`. Don't trust `server_info.url`: it's often an internal hostname.

**Lenient types**: the most common cause of crashes in IPTV apps. Write a small set of deserializers and use them everywhere:
- `de_opt_string`: string | number | null | `false` | `""` → `Option<String>`
- `de_opt_i64`: number | numeric string | `""` | null → `Option<i64>`
- `de_bool01`: 0/1/"0"/"1"/true/false/null
- `backdrop_path`: string | array → `Vec<String>`
- `get_series_info.episodes`: **object keyed by season number OR array of arrays**, and `seasons` may be empty → synthesize seasons from the episode keys (Fred TV does this too: `create_makeshift_season`).
- Unknown fields ignored. Keep each raw item's leftovers in `extra_json` for later features.
Fixture-test each one against real-world samples (§10).

Fetch the 3 lists sequentially, not in parallel: providers rate-limit and ban. Send the source's User-Agent; the default UA is configurable globally, since some panels block reqwest's UA. Timeouts: connect 10s, read 60s (big lists are slow).

Account panel per source: status, expiry date (`exp_date` unix, may be null = unlimited), connections `active/max`. Warn when `active_cons >= max_connections` before playing.

### 5.3 Refresh
- Manual "Refresh" per source and "Refresh all". Optional auto-refresh at startup if older than N hours (default 24, setting).
- Steps: download/parse into memory or temp → in **one transaction**: `DELETE` the source's categories/streams, insert the new ones, rebuild FTS rows for the source → commit. If any step fails, roll back and keep the old data, and store the error in `sources.last_error`.
- Progress events (bytes downloaded, items parsed) → progress bar in the sources list.

### 5.4 Add-source dialog + input detection (`sources/detect.rs`)
One "Paste anything" box plus explicit tabs (Xtream form: server / user / pass; M3U URL; M3U file picker via `rfd`).
Detection order for pasted text:
1. URL containing `get.php` or `player_api.php` with `username` & `password` query params → **Xtream**, base and creds extracted. This is the classic "Xtream code" share format; offer "import as Xtream (recommended: richer data)" with "as M3U" as the alternative.
2. Existing local path → M3U file.
3. Other URL → fetch the first ~64KB: `#EXTM3U` + `#EXTINF` → M3U list. `#EXT-X-` → single HLS stream (add to a "Quick streams" source, or just play it). JSON with `user_info` → Xtream base.
4. Formats like `http://host:port username password` on one line, or as 3 lines → Xtream (users paste these from emails).
Test connection before saving. Show the account info for Xtream and the entry count for M3U.

### 5.5 Later inputs
- **Stalker / Ministra portal** (portal URL + MAC): handshake → token → `get_profile` → `itv.get_genres` / `get_all_channels` → `create_link` per play. Its own module; it plugs into the same tables (`kind='stalker'`, URL resolved at play time like Xtream).
- Single stream URL (already covered by "Quick streams").

---

## 6. UI (egui)

```
┌ TopBar: [Profile ▾] [Source: All ▾]  ( Live | Movies | Series )   [🔍 search…]  [⚙] ┐
├───────────────┬───────────────────────────────────────┬────────────────────────────┤
│ Categories    │ Channel list / poster grid            │ Player (embedded mpv)      │
│ ★ Favorites   │ 101  [logo] Channel name        ☆     │                            │
│ ⟲ Recent      │ 102  [logo] Channel name        ★     │   overlay controls on hover│
│ ── pinned ★ ──│ …  (virtualized: ScrollArea::show_rows)│                            │
│ News      124 │                                       │ Now: Channel name          │
│ Sports     88 │                                       │ [⏮][⏯][⏭] vol ── audio ▾ sub ▾│
│ [✎ Edit]      │                                       │                            │
└───────────────┴───────────────────────────────────────┴────────────────────────────┘
```

- **Top bar:** profile switcher; source filter ("All" merges every enabled source of the profile; categories are prefixed by source when there is more than one); media tabs; global search (Ctrl+F) over the FTS index, 150ms debounce, scoped to the current tab, results grouped by source.
- **Category sidebar** (`SidePanel::left`, resizable):
  - Virtual entries at the top: ★ Favorites (channels), ⟲ Recently watched, then pinned favorite categories, then the rest. Count badges.
  - **Edit mode** (✎ toggle): every row shows a drag handle (`egui_dnd`), an eye toggle (hide/show) and a star (pin). Hidden categories appear dimmed in edit mode only. Toolbar: *Show all / Hide all / Invert*, plus a **"Hide matching…" filter**. Provider lists use country prefixes like `FR |`, `UK:`, `[AR]`, so "keep only categories matching `^(FR|UK)`" turns 1500 categories into 60 in one action. Sort presets: Provider order / A–Z / Reset custom order.
  - Normal mode: hidden categories are not shown at all.
- **List pane:** Live = dense rows (number, logo, name, star, later a now-playing EPG line). Movies = poster grid (`ui.horizontal_wrapped` with only visible rows computed). Series = poster grid → series page (seasons as tabs, episode list, resume markers).
  - Context menu (right click / Menu key): ☆ Favorite · Hide channel · Play with ▸ (Embedded / mpv / VLC / custom…) · Copy stream URL · Info.
  - The Favorites view supports drag reorder.
  - Sorting per category: provider order (default) / number / A–Z.
- **Player pane** (`SidePanel::right` or bottom; layout toggle) and **fullscreen** (F / double-click; Esc exits). Overlay controls fade out after 3s of no mouse movement. Channel up/down (PgUp/PgDn) moves within the current list. `Backspace`/`L` = previous channel (zap back).
  When the player is external, this pane shows "Playing in mpv/VLC" plus a Stop button and the list takes the full width.
- **Keyboard first** (works with remotes / air mice later): arrows / Enter / Esc / Backspace, Ctrl+1/2/3 tabs, Ctrl+D favorite toggle, Ctrl+H hide, Ctrl+K command palette (later).
- **Settings:** default player (+ per media kind override, e.g. Live → embedded, Movies → VLC), mpv extra options (free text, `key=value` lines, à la Hypnotix), paths to mpv/VLC (auto-detected, overridable), custom player templates, default User-Agent, auto-refresh, theme (dark/light/system), UI scale (`ctx.set_zoom_factor`), startup profile.
- **Fonts (verify in M0):** channel names are full of Arabic, Cyrillic, CJK and emoji. egui's default fonts lack CJK/Arabic → bundle Noto Sans + Noto Sans Arabic + a CJK subset as fallbacks, or load system fonts. egui has **no complex-text shaping / RTL**: Arabic will render as disconnected, left-to-right letters. Measure this in the M0 spike. If unacceptable, options are a shaping pre-pass (`rustybuzz` → glyph runs) or accepting it as a known limitation. This is the largest UI-side risk of choosing egui.

---

## 7. Playback

```rust
pub enum PlayerKind { Embedded, Mpv, Vlc, Custom(usize /* template index */) }
pub struct PlayRequest {
    pub url: String, pub title: String, pub media: MediaKind,
    pub user_agent: Option<String>, pub referrer: Option<String>, pub origin: Option<String>,
    pub start_s: Option<f64>,            // resume for VOD
}
```
Resolution chain: per-stream override (later) → per-media-kind setting → default player. If the embedded backend is unavailable (libmpv failed to load), fall back to external mpv, then VLC, and show a toast.

### 7.1 Embedded mpv (`player/mpv_embed.rs`)
- `eframe::NativeOptions { renderer: Renderer::Glow, .. }`. In `App::new`, get `cc.gl` and create the mpv `RenderContext` with the GL `get_proc_address` from eframe's context (pattern from egui-mpv-glow).
- Render mpv into an FBO/texture sized to the player rect (× pixels_per_point) inside an `egui::PaintCallback` (`egui_glow::CallbackFn`). Register it with `painter.register_native_texture` and draw it as an image, so egui can overlay controls on top.
- mpv `set_update_callback` → `ctx.request_repaint()`. While idle, nothing repaints.
- Options set at init (all overridable from settings):
  `hwdec=auto-safe`, `vo=libmpv`, `cache=yes`, `demuxer-max-bytes=64MiB`, `network-timeout=15`,
  `stream-lavf-o=reconnect=1,reconnect_streamed=1,reconnect_delay_max=5` (IPTV TS drops),
  `ytdl=no`, `input-default-bindings=no`, `osc=no`.
- Per play: `user-agent`, `referrer`, `http-header-fields=Origin: …` as **per-file options** (`loadfile url replace -1 user-agent=…`). That avoids races between global option changes and the load.
- Observe properties: `pause`, `time-pos`, `duration`, `volume`, `mute`, `track-list`, `media-title`, `paused-for-cache`, `cache-buffering-state`, `eof-reached`. Show a buffering spinner and audio/sub track menus from `track-list`.
- On live EOF/error: auto-retry 3× with backoff, then show an error and keep the UI usable.
- Write the resume position for VOD to `history` every 10s and on stop.
- Only one stream plays at a time (connection limits). Stop external players before playing embedded, and the reverse.

### 7.2 External mpv
`mpv --force-window=immediate --force-media-title=<title> --user-agent=<ua> --referrer=<ref> --input-ipc-server=<sock> -- <url>`
- **Reuse one instance**: if our mpv is alive, send `{"command":["loadfile",url,"replace",-1,{"user-agent":…}]}` over IPC (Unix socket; `\\.\pipe\…` on Windows). Channel zapping becomes instant and doesn't reopen a window. If the IPC write fails, spawn a new one.
- Detection: `PATH`, then `/opt/homebrew/bin`, `/usr/local/bin`, `/Applications/mpv.app/Contents/MacOS/mpv`, `%ProgramFiles%\mpv`, scoop/choco shims. IINA (`iina --mpv-…`) as a macOS preset.

### 7.3 VLC
`vlc --http-user-agent=<ua> --http-referrer=<ref> --meta-title=<title> --one-instance --no-playlist-enqueue -- <url>` (macOS: `/Applications/VLC.app/Contents/MacOS/VLC`, Windows: `%ProgramFiles%\VideoLAN\VLC\vlc.exe`).
`--one-instance` replaces the current item, which gives zapping without a new window on Win/Linux. macOS VLC ignores it, so the app kills and respawns there.

### 7.4 Custom player
Templates such as `potplayer {url} /title={title}`, with placeholders `{url} {title} {user_agent} {referrer}`. Tokenize the template with `shlex` **first**, then substitute into individual argv items, then `Command::new(argv[0]).args(..)`. **Never run it through a shell.**

**Security (trust boundary): playlist content is untrusted input.**
- Allow only URL schemes `http https rtmp rtmps rtsp udp rtp mms`. Reject `file:`, `javascript:`, empty, and anything starting with `-` (mpv/VLC option injection, e.g. a URL `--script=/tmp/x.lua`). Always pass `--` before the URL where the player supports it.
- Strip control characters from titles before passing them as args.
- Redact `username/password` from logs and error messages (they are in every Xtream URL). "Copy URL" copies the real URL, which is intended.

---

## 8. Profiles & organization: behavior spec
- Favorites are per profile and span sources. The ★ Favorites view merges all enabled sources of the current profile for the current media tab, ordered by `fav_position` (drag to reorder). Favoriting an item appends it at the end.
- Favorite categories = "pinned" categories at the top of the sidebar, also drag-orderable.
- Hide works on categories and on individual channels. "Show hidden" exists only in edit mode, so hidden items can be recovered without separate management screens.
- New categories that appear after a refresh show up after the custom-ordered ones, in provider order, never hidden by default. Option: "hide new categories by default" for users who whitelist (cheap: insert `hidden=1` prefs for unseen keys after refresh).
- Profile switcher: create / rename / delete / duplicate. Duplicating copies the overlay ("make a Kids profile from mine"). Optional PIN per profile is **later** (parental control; store an argon2 hash, and the plan says plainly it is a soft lock, not security).
- Recently watched: the last 50 per media tab. "Continue watching" row for VOD/episodes with position > 5% and < 95%.
- Export/import the overlay as JSON (later; small, and it makes the "new machine" story work).

---

## 9. Milestones

Each milestone ends with something usable.

**M0 — Skeleton & spikes (de-risk first)**
- eframe glow window, theme, zoom, persistence of window/panel sizes (`eframe` `persistence` feature).
- DB open + migrations + default profile. Worker thread + Command/Event plumbing.
- **Spike A:** libmpv render into egui on macOS, Windows and Linux (X11+Wayland) with an overlay button. Go/no-go for embedded on each OS.
- **Spike B:** font fallback with Arabic/CJK/emoji channel names. Decide whether RTL shaping is in scope.

**M1 — Sources**
- Add/edit/delete source dialog with paste detection. M3U URL/file + Xtream.
- Streaming M3U parser, lenient Xtream client, transactional refresh, progress, error display, account panel.
- Fixture tests (§10).

**M2 — Browse**
- Tabs Live/Movies/Series, category sidebar, virtualized list, logos, poster grid, series page (lazy `get_series_info`), FTS search, source filter.

**M3 — Playback (external)**
- mpv (IPC reuse) / VLC / custom templates, detection, headers, URL validation, per-media-kind defaults, "Play with ▸" menu.
- *This is the first build anyone can use day to day.*

**M4 — Organization**
- Favorites (channels + categories) with reordering, hide/show, edit mode with drag reorder, "Hide matching…", sort presets, multiple profiles + switcher + duplicate, recently watched.

**M5 — Embedded mpv**
- Player pane + fullscreen, overlay controls, tracks, volume, channel up/down, zap-back, retry on drop, VOD resume, fallback chain.

**M6 — Polish & packaging**
- Keyboard navigation pass, toasts, empty and error states, logo disk cache, startup auto-refresh.
- Packaging with `cargo-packager` or `cargo-dist`: macOS `.app`/`.dmg` with libmpv dylibs relocated (`dylibbundler`), signed + notarized. Windows MSI/zip with `libmpv-2.dll` (shinchiro builds). Linux AppImage + Flatpak (Flathub's mpv runtime extension, as Fred TV does). CI matrix on GitHub Actions for all three.

**Later (in rough priority order)**
1. EPG: XMLTV (from `#EXTM3U url-tvg`, Xtream `xmltv.php`, or manual), streaming parse into an `epg` table keyed by `epg_id`; now/next line in lists; guide grid.
2. Catch-up/timeshift (Xtream `tv_archive`/`archive_days`, M3U `catchup-source` templates).
3. Recording: mpv `stream-record=<file>` for record-while-watching; scheduled recording via a headless ffmpeg `-c copy` process.
4. Stalker portals. 5. Parental PIN. 6. TMDB metadata. 7. Command palette. 8. Overlay export/import. 9. TV/remote "10-foot" mode. 10. Multi-view.

---

## 10. Testing
- **Parser fixtures** (`tests/fixtures/*.m3u`): BOM+CRLF, commas inside quoted attributes and titles, unquoted/single-quoted attrs, `#EXTGRP`, `#EXTVLCOPT` headers, pipe headers, missing group, duplicate names, HLS media playlist (must be detected as a single stream), Latin-1 bytes, 100k-line generated file (perf assertion: under 2s in release).
- **Xtream fixtures** (`tests/fixtures/xtream/*.json`): numbers as strings, `""`/null/false mixes, `backdrop_path` string vs array, `episodes` as object vs array, empty `seasons`, `auth:0`, `exp_date:null`.
- **Overlay survival:** import → favorite/hide/reorder → refresh with shuffled + partially removed + new items → prefs intact, new categories appended, removed favorites reported missing, then restored when re-added.
- **Detection:** table test of pasted strings → expected `SourceInput`.
- **Player argv:** expected argv per backend, headers present, `-`-prefixed and `file:` URLs rejected, template substitution can't inject extra args.
- **URL builders:** Xtream live/movie/series URLs with special characters in user/pass.
- UI: manual checklist per milestone. egui_kittest snapshot tests only if regressions show up.
- Free, legal test content: github.com/iptv-org/iptv (M3U, ~10k channels) and Free-TV/IPTV. For Xtream, use our fixtures. No public legal Xtream test server is known, so a tiny `axum` stub that serves the fixtures is worth it in M1.

---

## 11. Risks & open questions
1. **egui text shaping (RTL/Arabic):** possibly a dealbreaker for some users. Spike B decides. Fallback idea: keep egui and pre-shape names. Alternative stack if it's unacceptable: Slint (has shaping) with the same core lib, since everything in `lib.rs` is UI-agnostic by design.
2. **Embedded mpv per OS:** macOS OpenGL is deprecated but functional (mpv's own `libmpv` GL path still works). Wayland is fine with glow. If one OS fails Spike A, that OS defaults to external mpv; the rest of the app is unaffected.
3. **libmpv distribution + GPL** (§1). Decide the license before the first public build.
4. **Provider hostility:** UA blocking, rate limiting, connection limits. Mitigated by a configurable UA, sequential fetches, a single active stream, and a connection-count warning.
5. **Memory with huge lists:** only the current category's rows are held in memory (id, name, logo, num, fav flag ≈ 100B/row). Search goes through FTS with `LIMIT 500`.
6. **Open question for you:** is "profiles" meant as *viewer personas over shared sources* (this plan), or *one profile = one provider account*? The schema supports both: a profile with exactly one source is the second case.
