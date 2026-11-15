# Graph Report - parabole  (2026-11-15)

## Corpus Check
- 312 files · ~98,313 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 67 file(s) not represented in the graph (top: .slint 55, .ttf 4, (none) 2)

## Summary
- 2871 nodes · 6609 edges · 162 communities (139 shown, 23 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 471 edges (avg confidence: 0.72)
- Token cost: 400,392 input · 0 output

## Community Hubs (Navigation)
- Slint App Bootstrap
- Category Edit Mode
- Track Prefs & Player Actions
- DB Queries
- DB Shared Helpers
- egui Autonext & Lifecycle
- egui Theme Colors
- HTTP Image Loader
- Worker Event Loop
- Release Workflow
- egui Layout Primitives
- Slint Groups & APIs
- Ambient Redesign Spec
- egui App Root
- Slint Playback Controls
- egui Icons
- UI Icon Set
- VM Guest E2E Script
- Browse Snapshots
- Slint Video Surface
- UI State Persistence
- Slint Image Cache
- mpv Core & Events
- CI Workflow
- Command Palette
- Profiles
- HTTP Client
- egui App Actions
- Settings & Zap Snapshots
- egui Sidebar Rows
- Worker Commands
- Sources Dialog
- Browse Callbacks
- Playback Stats
- Stream List Model
- Player Key Handling
- Flatpak Manifest
- Sources Module
- Source Input Detection
- Xtream Parsing
- Python Tooling Scripts
- Stream Rows & Sort
- libmpv Render
- Stream Failover
- Slint Browse Area
- DB Overlay Tests
- M3U Parsed Entries
- Empty States
- Channel Zapping
- Appearance Settings
- Media Details
- Menus & Profile Dialog
- Hero Banner
- Play Session
- egui Buttons & Forms
- Plan: Data Model
- External Player
- Poster Grid Model
- egui Fonts
- Series Browse
- Settings Dialog
- Xtream Tests
- egui Reload & Toasts
- egui Windows
- Embedded Player
- Row Actions (Slint)
- mpv IPC
- Slint Test Seed
- Zap Carousel
- Player Argv
- egui Center Pane
- Browse Filters & Sort
- M3U Tests
- Slint Experiment Doc
- Player Settings
- Account Info
- egui Sort Mode
- egui Video & Nerd Stats
- egui Poster Grid
- Picture-in-Picture
- Live Low-Latency
- egui Settings View
- Real-Provider Scenarios
- DB Alternates Tests
- EmbeddedPlayer API
- egui UI Kit
- Logo Tiles
- Binary Locate
- Snapshot Testing
- egui Mini Player
- URL Validation
- RowAction Variants
- Key Decisions (Memory)
- Palette Execution
- Phase 1 Spec
- Phase 2 Spec
- egui Series View
- Source Linking
- DB Group Tests
- GL Render Target
- egui Sidebar
- Slint Redesign History
- Watch Progress
- UI Inventory
- Seasons & Episodes
- Image Store Tests
- Ink Rendering
- egui Profiles Window
- UI Review Passes
- Fonts & Licenses
- Redesign Mockup
- Headless Tests Spec
- Packaging Docs
- Plan: Architecture
- VM Host E2E Script
- Resume Position
- Player Kind Status
- Fuzzy Matching
- egui Autonext Overlay
- Next-Steps Agents
- Image Cache Pruning
- Source Probing
- VM Seed Binary
- Xtream Conversion
- Backdrop FX
- MpvCore Controls
- Selection Model
- egui Top Bar
- egui Navigation Keys
- Progress Marks
- Worker Tests
- M6 Packaging History
- VM E2E Spec
- Plan: Playback
- egui Series Playback
- M3U Parser Entry
- Project Conventions
- Channel Name Normalize
- Source Kinds
- Project Direction Brief
- VM E2E History
- Slint Font Fallback
- egui Details View
- egui Style
- Live Catchup Lua
- LRU Cache
- Slint Experiment Start
- Phase 2 Orchestration
- DB Migrations
- Retry Backoff
- Linux Packaging
- Inventory Views
- macOS Packaging
- macOS Smoke Test
- macOS Verify
- Theme Install
- Parabole Binary

## God Nodes (most connected - your core abstractions)
1. `SlintApp` - 140 edges
2. `MediaKind` - 131 edges
3. `SourceId` - 120 edges
4. `StreamRow` - 77 edges
5. `ProfileId` - 73 edges
6. `App` - 69 edges
7. `Command` - 67 edges
8. `with_app()` - 60 edges
9. `Source` - 54 edges
10. `PlayerKind` - 52 edges

## Surprising Connections (you probably didn't know these)
- `MpvCore toolkit-neutral split` --semantically_similar_to--> `MpvCore`  [INFERRED] [semantically similar]
  docs/slint-phase2.md → src/player/embed/core.rs
- `appearance.rs (Appearance struct, ACCENTS)` --semantically_similar_to--> `Appearance`  [INFERRED] [semantically similar]
  docs/slint-ambient-spec.md → src/slint_ui/appearance.rs
- `images.rs size classes (Logo, Poster, Backdrop)` --references--> `Images`  [EXTRACTED]
  docs/slint-ambient-spec.md → src/slint_ui/images.rs
- `images.rs logo loader (Images, LRU)` --references--> `Images`  [EXTRACTED]
  docs/slint-phase1-spec.md → src/slint_ui/images.rs
- `state.rs (Sel, Persisted, SlintApp, with_app)` --references--> `SlintApp`  [EXTRACTED]
  docs/slint-phase1-spec.md → src/slint_ui/state.rs

## Import Cycles
- 1-file cycle: `src/db/alt_name.rs -> src/db/alt_name.rs`
- 1-file cycle: `src/slint_ui/push.rs -> src/slint_ui/push.rs`
- 1-file cycle: `src/player/embed/render.rs -> src/player/embed/render.rs`
- 1-file cycle: `src/ui/player/controls.rs -> src/ui/player/controls.rs`
- 1-file cycle: `src/slint_ui/player/chrome.rs -> src/slint_ui/player/chrome.rs`
- 2-file cycle: `src/ui/sidebar_groups.rs -> src/ui/sidebar_rows.rs -> src/ui/sidebar_groups.rs`
- 2-file cycle: `src/sources/m3u/entry.rs -> src/sources/m3u/mod.rs -> src/sources/m3u/entry.rs`
- 2-file cycle: `src/sources/xtream/client.rs -> src/sources/xtream/mod.rs -> src/sources/xtream/client.rs`
- 2-file cycle: `src/ui/player/autonext.rs -> src/ui/player/session.rs -> src/ui/player/autonext.rs`
- 2-file cycle: `src/ui/player/session.rs -> src/ui/player/track_memory.rs -> src/ui/player/session.rs`
- 2-file cycle: `src/ui/boot.rs -> src/ui/mod.rs -> src/ui/boot.rs`
- 2-file cycle: `src/worker/embed.rs -> src/worker/mod.rs -> src/worker/embed.rs`
- 2-file cycle: `src/slint_ui/browse/grid.rs -> src/slint_ui/stream_model.rs -> src/slint_ui/browse/grid.rs`
- 2-file cycle: `src/player/locate.rs -> src/player/mod.rs -> src/player/locate.rs`
- 2-file cycle: `src/player/mod.rs -> src/player/settings.rs -> src/player/mod.rs`
- 3-file cycle: `src/ui/boot.rs -> src/ui/player/mod.rs -> src/ui/mod.rs -> src/ui/boot.rs`
- 3-file cycle: `src/player/locate.rs -> src/player/settings.rs -> src/player/mod.rs -> src/player/locate.rs`
- 5-file cycle: `src/ui/boot.rs -> src/ui/player/mod.rs -> src/ui/player/session.rs -> src/ui/player/autonext.rs -> src/ui/mod.rs -> src/ui/boot.rs`

## Hyperedges (group relationships)
- **Orchestrator-designed parallel worktree agent batches** — _memories_2026_11_11_123726_e3bc_parallel_worktree_agents, _memories_2026_11_11_154833_2f33_agent_a_player_fixes_mini, _memories_2026_11_12_172857_6d14_m6_polish_agent, _memories_2026_11_15_121146_81bd_slint_phase2_delegates, _memories_2026_11_15_174542_6e20_ui_improvement_passes [INFERRED 0.85]
- **Slint experiment: start, phase 2, results, ambient, review passes** — _memories_2026_11_15_105711_1eb0_slint_experiment, _memories_2026_11_15_121146_81bd_slint_phase2_scaffold, _memories_2026_11_15_124837_5f1a_slint_vs_egui_measurements, _memories_2026_11_15_144818_86d4_ambient_redesign, _memories_2026_11_15_181950_9a68_ui_review_passes_merged [EXTRACTED 0.85]
- **VM e2e harness and real-source follow-up** — _memories_2026_11_15_163851_6f1d_vm_e2e_harness, _memories_2026_11_15_170726_1476_e2e_real_mode, _memories_2026_11_15_170726_1476_live_catchup_lua_fix, _memories_2026_11_15_163851_6f1d_app_bugs_fixed_by_e2e [EXTRACTED 0.85]
- **Playback backends (embedded mpv, external mpv, VLC, custom)** — plan_embedded_mpv, plan_external_mpv, plan_vlc, plan_custom_player [EXTRACTED 0.95]
- **Source ingestion flow (detect, parse/fetch, refresh into overlay-safe tables)** — plan_input_detection, plan_m3u_parser, plan_xtream_client, plan_refresh, plan_sqlite_schema [EXTRACTED 0.85]
- **Platform packaging targets** — packaging_readme_macos, packaging_readme_windows, packaging_readme_linux, packaging_readme_flatpak [EXTRACTED 0.95]
- **Ambient redesign stages S1 then parallel S2a/S2b/S2c** — docs_slint_ambient_spec_s1, docs_slint_ambient_spec_s2a, docs_slint_ambient_spec_s2b, docs_slint_ambient_spec_s2c [EXTRACTED 1.00]
- **Phase 1 Rust glue pipeline** — docs_slint_phase1_spec_state, docs_slint_phase1_spec_images, docs_slint_phase1_spec_stream_model, docs_slint_phase1_spec_data, docs_slint_phase1_spec_push, docs_slint_phase1_spec_callbacks, docs_slint_phase1_spec_events, docs_slint_phase1_spec_run [EXTRACTED 1.00]
- **Headless UI testing stack** — docs_slint_headless_tests_spec_prereq_build, docs_slint_headless_tests_spec_shared_seed, docs_slint_headless_tests_spec_behaviour, docs_slint_headless_tests_spec_snapshots, docs_slint_headless_tests_spec_sw_platform [EXTRACTED 1.00]
- **Player transport controls** — assets_icons_play_play, assets_icons_pause_pause, assets_icons_stop_stop, assets_icons_resume_resume [INFERRED 0.85]
- **Content category navigation icons** — assets_icons_live_live, assets_icons_movie_movie, assets_icons_series_series, assets_icons_recent_recent, assets_icons_star_star [INFERRED 0.75]
- **Window and layout mode controls** — assets_icons_dock_bottom_dock_bottom, assets_icons_dock_right_dock_right, assets_icons_mini_mini, assets_icons_fullscreen_fullscreen, assets_icons_fullscreen_exit_fullscreen_exit [INFERRED 0.75]
- **Persistent app shell (icon rail, category sidebar, status bar)** — tests_fixtures_slint_snapshots_live_dark_icon_rail_navigation, tests_fixtures_slint_snapshots_live_dark_category_sidebar, tests_fixtures_slint_snapshots_live_dark_status_bar [EXTRACTED 0.95]
- **Modal overlays on dimmed backdrop** — tests_fixtures_slint_snapshots_appearance_settings_dialog, tests_fixtures_slint_snapshots_profiles_profiles_dialog, tests_fixtures_slint_snapshots_palette_chan_command_palette [INFERRED 0.85]
- **Theming system (accent, mode, light/dark)** — tests_fixtures_slint_snapshots_appearance_accent_color_picker, tests_fixtures_slint_snapshots_appearance_theme_mode_dark_oled_light, tests_fixtures_slint_snapshots_live_dark_dark_theme_teal_accent, tests_fixtures_slint_snapshots_live_light_light_theme [INFERRED 0.85]
- **Three channel zapping display modes** — tests_fixtures_slint_snapshots_zap_carousel_zap_carousel_mode, tests_fixtures_slint_snapshots_zap_fullscreen_zap_fullscreen_mode, tests_fixtures_slint_snapshots_zap_mini_zap_mini_mode [INFERRED 0.85]
- **Modal dialogs over dimmed app shell** — tests_fixtures_slint_snapshots_settings_players_settings_dialog, tests_fixtures_slint_snapshots_sources_sources_dialog, tests_fixtures_slint_snapshots_sources_form_add_source_form [INFERRED 0.85]
- **Player configuration surfaces** — tests_fixtures_slint_snapshots_settings_playback_tab, tests_fixtures_slint_snapshots_settings_players_players_tab, tests_fixtures_slint_snapshots_settings_players_detect_button [INFERRED 0.75]

## Communities (162 total, 23 thin omitted)

### Community 0 - "Slint App Bootstrap"
Cohesion: 0.05
Nodes (34): FRAME, Job, LOGICAL, Proxy, Queue, SCALE, SwPlatform, WAIT (+26 more)

### Community 1 - "Category Edit Mode"
Cohesion: 0.05
Nodes (31): bulk(), connect(), move_item(), set_edit(), SlintApp, start_with_filter(), Content, Cta (+23 more)

### Community 2 - "Track Prefs & Player Actions"
Cohesion: 0.05
Nodes (44): langs_roundtrip_without_touching_favorites(), LangKind, Audio, Sub, TrackLangs, TrackInfo, TrackKind, Audio (+36 more)

### Community 3 - "DB Queries"
Cohesion: 0.06
Nodes (24): Db, Db, Db, Db, Db, fraction(), Db, CategoryRow (+16 more)

### Community 4 - "DB Shared Helpers"
Cohesion: 0.06
Nodes (17): NAME_CANDIDATES, STREAM_SELECT, FavRow, Db, rebuild_fts(), Db, fts_query(), COLS (+9 more)

### Community 5 - "egui Autonext & Lifecycle"
Cohesion: 0.08
Nodes (10): App, App, App, App, App, App, separator_and_hint(), App (+2 more)

### Community 6 - "egui Theme Colors"
Cohesion: 0.07
Nodes (49): ACCENT, BADGE_TEXT, BG, BORDER, CARD, CARD_BORDER, CARD_BORDER_HOVER, CHANNEL_ROW_H (+41 more)

### Community 7 - "HTTP Image Loader"
Cohesion: 0.07
Nodes (16): DiskCacheLoader, install(), is_http(), Job, Shared, State, Failed, Pending (+8 more)

### Community 8 - "Worker Event Loop"
Cohesion: 0.08
Nodes (23): on_event(), hours(), Worker, Worker, Event, AccountChecked, Error, InfoLoaded (+15 more)

### Community 9 - "Release Workflow"
Cohesion: 0.06
Nodes (40): env, APPLE_APP_PASSWORD, APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD, APPLE_ID, APPLE_TEAM_ID, CARGO_TERM_COLOR, GITHUB_TOKEN (+32 more)

### Community 10 - "egui Layout Primitives"
Cohesion: 0.09
Nodes (8): Item, Item, ROW_H, Fav, GRID_PAGE, LIST_GAP, MAX_DRAG_FAVORITES, PAGE

### Community 11 - "Slint Groups & APIs"
Cohesion: 0.09
Nodes (17): connect(), connect(), detect_paste(), CUSTOM_UA, drain(), kind_name(), set_playing(), build() (+9 more)

### Community 12 - "Ambient Redesign Spec"
Cohesion: 0.10
Nodes (34): Ambient redesign spec (Slint only), Appearance dialog, appearance.rs (Appearance struct, ACCENTS), Player chrome with scrims and seek bar, palette/fuzzy.rs score, Ambient hero (browse-hero.slint), images.rs size classes (Logo, Poster, Backdrop), Living rows and StreamItem.playing (+26 more)

### Community 13 - "egui App Root"
Cohesion: 0.08
Nodes (10): Db, Alternate, StreamGroup, install_image_cache(), run(), GroupDialog, App, Nav (+2 more)

### Community 14 - "Slint Playback Controls"
Cohesion: 0.12
Nodes (29): connect(), FULLSCREEN_EXIT, leave_special_modes(), select(), set_fullscreen(), set_mini(), toggle_fullscreen(), toggle_mini() (+21 more)

### Community 15 - "egui Icons"
Cohesion: 0.07
Nodes (29): Icon, Audio, Back, Check, Close, DockBottom, DockRight, Eye (+21 more)

### Community 16 - "UI Icon Set"
Cohesion: 0.08
Nodes (32): audio icon - music note: audio track selection, back icon - left chevron: navigate back, check icon - checkmark: confirm/selected, close icon - X: close/dismiss, dock_bottom icon - panel docked at bottom of window, dock_right icon - panel docked at right of window, eye icon - open eye: show/visible, eye_off icon - struck-through eye: hide/hidden (+24 more)

### Community 17 - "VM Guest E2E Script"
Cohesion: 0.17
Nodes (27): bad(), crop_probe(), DISPLAY, gen(), geom(), hover_video(), is_bright(), key() (+19 more)

### Community 18 - "Browse Snapshots"
Cohesion: 0.10
Nodes (32): Accent color picker (6 swatches), Appearance Settings Snapshot, Density, corner radius, font size and reduce-motion controls, Dimmed backdrop modal overlay pattern, Settings modal dialog (Playback / Players / Appearance tabs), Theme mode selector: Dark / OLED / Light, Category sidebar (Favorites, Recent, My Groups, Categories with counts), Channel list rows with initials avatar and UHD badge (+24 more)

### Community 19 - "Slint Video Surface"
Cohesion: 0.10
Nodes (4): install(), Saved, TOGGLES, Video

### Community 20 - "UI State Persistence"
Cohesion: 0.08
Nodes (15): RECENT_LIMIT, SEARCH_LIMIT, Persisted, RECENT_LIMIT, SEARCH_LIMIT, EditState, Persisted, Sel (+7 more)

### Community 21 - "Slint Image Cache"
Cohesion: 0.14
Nodes (13): cache_keys_separate_sizes(), decode(), Decoded, Images, is_http(), Key, lru_evicts_least_recently_used(), Size (+5 more)

### Community 22 - "mpv Core & Events"
Cohesion: 0.12
Nodes (12): MpvCore, apply(), observe_all(), OBSERVED, poll(), split_mpv_arg(), Notice, Ended (+4 more)

### Community 23 - "CI Workflow"
Cohesion: 0.07
Nodes (28): runs-on, steps, name, runs-on, steps, strategy, env, CARGO_TERM_COLOR (+20 more)

### Community 24 - "Command Palette"
Cohesion: 0.10
Nodes (18): Action, Density, OpenDialog, ReduceMotion, RefreshAll, SwitchProfile, ThemeMode, ToggleDock (+10 more)

### Community 25 - "Profiles"
Cohesion: 0.12
Nodes (12): Db, Db, hide_new(), keys(), setting_key(), Db, Db, Profile (+4 more)

### Community 26 - "HTTP Client"
Cohesion: 0.10
Nodes (11): client(), download(), get_or_fetch(), MAX_IMAGE_BYTES, blocked_hint(), client(), get(), SafeRead (+3 more)

### Community 27 - "egui App Actions"
Cohesion: 0.15
Nodes (6): App, App, App, App, App, App

### Community 28 - "Settings & Zap Snapshots"
Cohesion: 0.12
Nodes (27): Live Latency Checkboxes (low-latency buffering, catch up to live edge), Playback Tab (default/live/movie/episode player selectors), App Shell Layout (nav rail, category sidebar, channel list, status bar), Detect Player Binaries Button, Players Tab (mpv path, VLC path, custom command, extra mpv args), Tabbed Settings Modal (Playback / Players / Appearance), Settings Dialog - Players Tab, Settings Dialog - Playback Tab (+19 more)

### Community 29 - "egui Sidebar Rows"
Cohesion: 0.19
Nodes (14): group_row(), groups_header(), App, cat_row(), entry_row(), header_row(), Line, Cat (+6 more)

### Community 30 - "Worker Commands"
Cohesion: 0.08
Nodes (22): Command, AccountChecked, AddSource, AutoRefresh, CheckAccount, CopyUrl, DeleteSource, Fetched (+14 more)

### Community 31 - "Sources Dialog"
Cohesion: 0.12
Nodes (11): account_text(), ago(), now(), preset_index(), read_fields(), reset_form(), SlintApp, is_custom() (+3 more)

### Community 32 - "Browse Callbacks"
Cohesion: 0.13
Nodes (16): connect(), tab_name(), add_to_group(), delete_group(), GroupCtx, open_group_dialog(), refresh_groups(), remove_from_group() (+8 more)

### Community 33 - "Playback Stats"
Cohesion: 0.10
Nodes (7): MpvCore, get(), MpvCore, NONE, num(), Stats, value()

### Community 34 - "Stream List Model"
Cohesion: 0.13
Nodes (3): ImageHook, initials(), StreamModel

### Community 35 - "Player Key Handling"
Cohesion: 0.18
Nodes (19): connect(), handle(), handle_keys(), help(), is(), neighbor_row(), play_embedded(), Pressed (+11 more)

### Community 36 - "Flatpak Manifest"
Cohesion: 0.09
Nodes (22): app-id, build-options, append-path, build-args, command, finish-args, modules, build-commands (+14 more)

### Community 37 - "Sources Module"
Cohesion: 0.18
Nodes (19): row_to_source(), Source, check_account(), episode_url(), fetch(), fetch_series(), fetch_vod_info(), resolve_url() (+11 more)

### Community 38 - "Source Input Detection"
Cohesion: 0.18
Nodes (16): SourceInput, M3uFile, M3uUrl, Xtream, builds_m3u_with_host_as_default_name(), default_name(), Fields, base_of() (+8 more)

### Community 39 - "Xtream Parsing"
Cohesion: 0.20
Nodes (12): JSON, null_expiry_is_unlimited(), parses_and_classifies(), SOON_SECS, bool01(), opt_i64(), opt_string(), string_list() (+4 more)

### Community 40 - "Python Tooling Scripts"
Cohesion: 0.10
Nodes (3): chunk(), inside_tri(), Handler

### Community 41 - "Stream Rows & Sort"
Cohesion: 0.15
Nodes (10): Db, stream_row(), classify(), SCAN_LIMIT, STARTED, WATCHED, Db, EpisodeRef (+2 more)

### Community 42 - "libmpv Render"
Cohesion: 0.15
Nodes (7): alloc_size(), Loader, proc_address(), ProcAddress, round_up(), Shared, VideoOutput

### Community 43 - "Stream Failover"
Cohesion: 0.17
Nodes (13): candidates(), fake(), pick(), pick_index(), pick_index_does_not_wait_for_slow_less_preferred(), pick_index_prefers_configured_order_over_speed(), pick_skips_dead_primary(), PROBE_TIMEOUT (+5 more)

### Community 44 - "Slint Browse Area"
Cohesion: 0.13
Nodes (6): activate(), BrowseArea, close_series(), connect(), copy_url(), on_event()

### Community 45 - "DB Overlay Tests"
Cohesion: 0.20
Nodes (17): playlist(), setup(), first_import_hides_nothing(), new_categories_hidden_only_for_opted_in_profile(), bulk_hide_applies_all_in_one_go(), delete_source_cascades_overlay_and_search(), failed_import_rolls_back(), names() (+9 more)

### Community 46 - "M3U Parsed Entries"
Cohesion: 0.16
Nodes (12): Headers, ParsedCategory, ParsedStream, scan(), Scanned, classify(), Draft, Pending (+4 more)

### Community 47 - "Empty States"
Cohesion: 0.18
Nodes (9): App, Content, Cta, ClearSearch, ManageSources, Retry, message(), tab_name() (+1 more)

### Community 48 - "Channel Zapping"
Cohesion: 0.16
Nodes (10): App, buffer_caps_digits(), buffer_fires_after_idle_or_enter(), DIGIT_KEYS, IDLE, MAX_DIGITS, number_lookup_by_num_or_position(), row() (+2 more)

### Community 49 - "Appearance Settings"
Cohesion: 0.17
Nodes (9): ACCENTS, Appearance, clamps_out_of_range(), DENSITIES, rgb(), commit(), connect(), from_api() (+1 more)

### Community 50 - "Media Details"
Cohesion: 0.22
Nodes (15): MediaInfo, cached(), close(), connect(), fields(), InfoStore, Key, loaded() (+7 more)

### Community 51 - "Menus & Profile Dialog"
Cohesion: 0.18
Nodes (9): action(), connect(), entries(), header(), in_group(), item(), separator(), connect() (+1 more)

### Community 52 - "Hero Banner"
Cohesion: 0.16
Nodes (11): action(), build(), connect(), fmt_hms(), focus(), HeroState, meta(), push() (+3 more)

### Community 53 - "Play Session"
Cohesion: 0.22
Nodes (8): PlayRequest, SAVE_EVERY, STABLE_AFTER_S, EmbedPlay, SeriesCtx, Track, Worker, Worker

### Community 54 - "egui Buttons & Forms"
Cohesion: 0.21
Nodes (6): default_name(), source_form(), SourcesDialog, icon_button(), primary_button(), secondary_button()

### Community 55 - "Plan: Data Model"
Cohesion: 0.17
Nodes (17): Data model: provider data vs user overlay, Fred TV (open-tv), Add-source dialog and input detection, Lenient Xtream deserializers, M3U/M3U8 streaming parser, Natural keys (category_id, stream_id, tvg-id, group-title), Profiles (viewer personas), Profiles and organization behavior spec (+9 more)

### Community 56 - "External Player"
Cohesion: 0.23
Nodes (8): exited_child_is_not_running(), ExternalPlayer, mpv_reuse_requires_live_mpv_child(), Running, sleep_args(), spawn_failure_is_reported_without_args(), start_replaces_previous_child(), start_stop_and_is_running()

### Community 58 - "egui Fonts"
Cohesion: 0.21
Nodes (9): add(), install(), medium(), MEDIUM, medium_family(), regular(), SEMIBOLD, semibold() (+1 more)

### Community 59 - "Series Browse"
Cohesion: 0.26
Nodes (14): close(), connect(), episode_id(), loaded(), open(), play_episode(), PROGRESS_EVERY, push() (+6 more)

### Community 60 - "Settings Dialog"
Cohesion: 0.21
Nodes (11): SlintApp, connect(), detect_text(), kind_at(), kind_label(), kind_names(), opt_path(), path_text() (+3 more)

### Community 61 - "Xtream Tests"
Cohesion: 0.21
Nodes (14): account_checks(), list(), live_conversion_with_type_chaos(), fixture!, playlist(), series_info_array_and_empty(), series_info_object_keyed(), series_info_section() (+6 more)

### Community 62 - "egui Reload & Toasts"
Cohesion: 0.21
Nodes (4): App, App, App, App

### Community 63 - "egui Windows"
Cohesion: 0.12
Nodes (3): App, App, App

### Community 65 - "Row Actions (Slint)"
Cohesion: 0.14
Nodes (13): play(), RowAction, Activate, AddToGroup, CopyUrl, Info, NewGroup, Play (+5 more)

### Community 66 - "mpv IPC"
Cohesion: 0.20
Nodes (9): live_payload_carries_low_latency_options(), loadfile_payload(), payload_format(), req(), send(), send_reports_rejection_and_dead_socket(), send_skips_events_and_accepts_success(), serve_once() (+1 more)

### Community 67 - "Slint Test Seed"
Cohesion: 0.14
Nodes (3): spawn(), terminate(), FIXTURE

### Community 68 - "Zap Carousel"
Cohesion: 0.20
Nodes (12): close(), fill_pane(), initials(), item(), OPEN_FOR, PANE_SPAN, pane_window(), set_open() (+4 more)

### Community 69 - "Player Argv"
Cohesion: 0.25
Nodes (13): custom(), custom_does_not_reexpand_and_handles_missing_values(), custom_substitutes_per_token(), custom_title_cannot_add_argv_items(), mpv(), mpv_full_argv(), mpv_live_adds_low_latency_vod_does_not(), mpv_minimal_ends_with_separator_and_url() (+5 more)

### Community 70 - "egui Center Pane"
Cohesion: 0.22
Nodes (5): App, App, chip(), Actions, App

### Community 71 - "Browse Filters & Sort"
Cohesion: 0.24
Nodes (9): chip(), connect(), entry(), grid_view(), load_sort(), media_from(), media_id(), SlintApp (+1 more)

### Community 72 - "M3U Tests"
Cohesion: 0.29
Nodes (13): bom_crlf_header_and_counts(), by_url(), categories_per_media_in_first_appearance_order(), commas_in_attrs_and_title(), duplicate_tvg_id_falls_back_to_name_key(), extgrp_missing_group_and_duplicate_names(), hundred_thousand_entries(), invalid_utf8_is_lossy() (+5 more)

### Community 73 - "Slint Experiment Doc"
Cohesion: 0.24
Nodes (14): Slint UI experiment, Accessibility pass (ui/a11y), Ambient redesign (summary), egui vs Slint comparison, Default UI switch (Slint default, parabole-egui), Dev-experience notes and Slint pitfalls, DejaVu fontique fallback (fonts.rs), Gaps list (+6 more)

### Community 74 - "Player Settings"
Cohesion: 0.21
Nodes (8): origin_header(), file_options(), live_request_gets_latency_options_vod_gets_start(), PlayerSettings, KEY, load(), roundtrip_and_default(), save()

### Community 75 - "Account Info"
Cohesion: 0.16
Nodes (8): AccountInfo, Expiry, Expired, Later, Soon, Unlimited, format_date(), panel()

### Community 76 - "egui Sort Mode"
Cohesion: 0.20
Nodes (7): App, order(), SortMode, .ALL, Name, Number, Provider

### Community 77 - "egui Video & Nerd Stats"
Cohesion: 0.18
Nodes (7): COL_GAP, LINE_H, PAD, paint(), TOP, FADE_AFTER_S, FADE_FOR_S

### Community 78 - "egui Poster Grid"
Cohesion: 0.31
Nodes (7): card_height(), paint_scrim(), paint_title(), RowCtx, show_card(), show_row(), TITLE_H

### Community 79 - "Picture-in-Picture"
Cohesion: 0.24
Nodes (8): connect(), enter(), Geometry, MARGIN_X, MARGIN_Y, restore(), save(), SIZE

### Community 80 - "Live Low-Latency"
Cohesion: 0.21
Nodes (10): file_options(), keys(), live_gets_low_latency_options(), low_latency(), options_string(), options_string_quotes_values(), SCRIPT, SCRIPT_NAME (+2 more)

### Community 81 - "egui Settings View"
Cohesion: 0.21
Nodes (7): App, detect_text(), kind_label(), kind_row(), opt_path(), path_text(), SettingsDialog

### Community 82 - "Real-Provider Scenarios"
Cohesion: 0.24
Nodes (11): click_stop(), lit(), moving(), pick(), r_fullscreen(), r_pause(), r_play_live(), r_stop() (+3 more)

### Community 83 - "DB Alternates Tests"
Cohesion: 0.23
Nodes (11): id_of(), ignores_hidden_unlinked_and_non_live(), load(), matches_by_epg_id_then_normalized_name(), with_epg(), import_200k_streams(), cat(), migrations_are_idempotent_on_reopen() (+3 more)

### Community 85 - "egui UI Kit"
Cohesion: 0.32
Nodes (6): icon_btn(), paint_badge(), paint_text(), pill(), pill_width(), text_width()

### Community 86 - "Logo Tiles"
Cohesion: 0.27
Nodes (7): Fit, Contain, Cover, initials(), is_http(), logo_tile(), paint_image()

### Community 87 - "Binary Locate"
Cohesion: 0.28
Nodes (10): custom_resolves_first_token(), env_dir(), find_binary(), locate(), mpv_defaults(), path_walk_finds_binary(), scratch(), search_path() (+2 more)

### Community 88 - "Snapshot Testing"
Cohesion: 0.22
Nodes (5): CHANNEL_TOLERANCE, check(), differs(), MAX_DIFF_RATIO, root()

### Community 89 - "egui Mini Player"
Cohesion: 0.22
Nodes (5): MINI_SIZE, meta_line(), paint_bg(), ROW_H, show()

### Community 90 - "URL Validation"
Cohesion: 0.23
Nodes (9): ALLOWED_SCHEMES, clean_opt(), clean_request(), clean_request_cleans_headers(), error_does_not_leak_url(), MAX_HEADER, MAX_TITLE, sanitize() (+1 more)

### Community 91 - "RowAction Variants"
Cohesion: 0.15
Nodes (13): RowAction, Activate, AddToGroup, CopyUrl, Info, LoadAlternates, NewGroup, Play (+5 more)

### Community 92 - "Key Decisions (Memory)"
Cohesion: 0.21
Nodes (8): POC orchestration started (checkpoint), Skeleton commit 38cccfe with stubbed public APIs, POC merged on main (checkpoint), POC known shortcuts (FTS rebuild, unvirtualized favorites), POC merged: 63 tests, 5.8k lines, Conflicting migration numbers (checkpoint), Parallel agents created conflicting migration_3.sql, Key decisions

### Community 94 - "Phase 1 Spec"
Cohesion: 0.33
Nodes (12): Phase 1 spec: Rust glue for Slint main window, callbacks.rs connect, data.rs reloads, events.rs drain, images.rs logo loader (Images, LRU), imgcache get_or_fetch and default_dir, push.rs state to Core properties, mod.rs run() (+4 more)

### Community 95 - "Phase 2 Spec"
Cohesion: 0.38
Nodes (11): Phase 2 spec: three parallel areas, Area A: player (exp/slint-player), Area B: browse (exp/slint-browse), Area C: dialogs (exp/slint-dialogs), Area scaffold (Api global, host, Area state, on_event), dialog-kit.slint shared widgets, DialogApi.open-dialog and BrowseApi.request-dialog, Event routing: dialogs, player, browse, core (+3 more)

### Community 96 - "egui Series View"
Cohesion: 0.23
Nodes (7): back_pill(), EPISODE_H, episode_row(), EpisodeEvent, Play, ToggleWatched, PROGRESS_EVERY

### Community 97 - "Source Linking"
Cohesion: 0.20
Nodes (4): NewSource, Db, join_mirrors(), link()

### Community 98 - "DB Group Tests"
Cohesion: 0.21
Nodes (7): account_json_roundtrip(), copy_appends_missing_favorites_in_order_without_duplicates(), group_items_survive_refresh_and_show_missing(), groups_are_ordered_and_items_append_without_duplicates(), groups_are_per_profile_and_cascade(), setup_two(), settings_global_vs_profile()

### Community 99 - "GL Render Target"
Cohesion: 0.26
Nodes (3): allocate_storage(), create_target(), Target

### Community 100 - "egui Sidebar"
Cohesion: 0.23
Nodes (5): Action, Hidden, Pinned, App, App

### Community 101 - "Slint Redesign History"
Cohesion: 0.22
Nodes (9): Redesign foundation committed (checkpoint), Redesign foundation commit 9f03c22 (src/ui/theme), Redesign agents A topbar/dialogs, B sidebar/list, C player, Slint experiment complete (checkpoint), Slint shapes Arabic RTL; egui lacks bidi, Slint vs egui measurements, Slint Ambient redesign (S1, S2a, S2b, S2c), Slint Ambient redesign merged (checkpoint) (+1 more)

### Community 102 - "Watch Progress"
Cohesion: 0.20
Nodes (7): Watch, Partial, Unstarted, Watched, continue_lists_partial_movies_newest_first(), movies(), series_continue_is_one_row_per_series_with_episode()

### Community 103 - "UI Inventory"
Cohesion: 0.20
Nodes (10): Category edit mode (drag, hide, pin, Hide matching), Hypnotix, UI design (egui layout, sidebar, list, player pane), Parabole UI inventory (for redesign), Category sidebar (normal mode), Embedded player (pane, overlay controls, fullscreen), Keyboard shortcuts, Main window layout (+2 more)

### Community 104 - "Seasons & Episodes"
Cohesion: 0.40
Nodes (8): find(), label(), labels(), next(), ordered(), Episode, Season, SeriesState

### Community 105 - "Image Store Tests"
Cohesion: 0.27
Nodes (8): file_name(), file_name_has_no_path_separators_for_hostile_urls(), file_name_is_stable_hex_and_distinct(), prune_removes_oldest_files_from_disk(), read_cached_deletes_oversized_files(), scratch(), write_atomic_round_trips_and_leaves_no_tmp(), write_atomic()

### Community 106 - "Ink Rendering"
Cohesion: 0.29
Nodes (4): distance(), ink_rows(), px(), Rect

### Community 107 - "egui Profiles Window"
Cohesion: 0.27
Nodes (7): App, Op, CopyFavorites, Create, Delete, Duplicate, Rename

### Community 108 - "UI Review Passes"
Cohesion: 0.24
Nodes (9): Slint UI improvement passes started (checkpoint), Pass ui/browse-density, Pass ui/dialogs, Pass ui/nav-search, Pass ui/player-pane, Four parallel Opus 5.5 UI passes on disjoint files, Slint UI review passes merged (checkpoint), Pass ui/a11y accessibility (+1 more)

### Community 109 - "Fonts & Licenses"
Cohesion: 0.22
Nodes (8): DejaVu / Bitstream Vera font license, Arev Fonts Copyright (Tavmjong Bah), Bitstream Vera Fonts Copyright, Inter font license (SIL OFL 1.1), SIL Open Font License 1.1, M0 spikes (libmpv in egui, font fallback), Risks and open questions, Slint as alternative UI stack

### Community 110 - "Redesign Mockup"
Cohesion: 0.29
Nodes (9): Redesign mockup (main window HTML), Category sidebar (virtual rows, pinned, categories), Content area (live list and poster grid), Category edit mode (hide, pin, reorder, hide matching), Player panel (docked right), Status bar, Toast notification, Top bar (profile, source, tabs, search) (+1 more)

### Community 111 - "Headless Tests Spec"
Cohesion: 0.38
Nodes (10): Headless UI tests (behaviour and snapshots), Slint headless UI tests spec, Alignment asserts, Behaviour tests (tests/slint_behaviour.rs), Dev-dependencies (i-slint-backend-testing, software renderer), Done when criteria, Prerequisite refactor: build() split from run(), Shared seed (tests/common/slint_seed.rs) (+2 more)

### Community 112 - "Packaging Docs"
Cohesion: 0.24
Nodes (10): Packaging README, CI and release workflows, Flatpak packaging, Placeholder icon, Linux AppImage packaging, macOS packaging (.app + .dmg, dylibbundler), Apple signing release secrets, Windows packaging (zip + libmpv-2.dll) (+2 more)

### Community 113 - "Plan: Architecture"
Cohesion: 0.22
Nodes (9): Parabole Plan (Rust + egui), Architecture: UI thread + worker thread, Cinebox, Command / Event channels, egui-mpv-glow reference, IPTVnator, Module layout (src tree), Prior art survey (Fred TV, IPTVnator, Hypnotix, egui-mpv-glow, Cinebox) (+1 more)

### Community 114 - "VM Host E2E Script"
Cohesion: 0.38
Nodes (7): build(), fetch(), guest(), e2e.sh script, sync_src(), test_run(), up()

### Community 115 - "Resume Position"
Cohesion: 0.24
Nodes (6): Db, MAX_FRACTION, MIN_FRACTION, record_watch_keeps_saved_position(), resume_start(), save_then_resume()

### Community 116 - "Player Kind Status"
Cohesion: 0.22
Nodes (6): PlayerKind, Custom, Embedded, Mpv, Vlc, kind_name()

### Community 117 - "Fuzzy Matching"
Cohesion: 0.27
Nodes (3): greedy(), lower(), score()

### Community 118 - "egui Autonext Overlay"
Cohesion: 0.31
Nodes (7): AutoNext, Choice, Cancel, PlayNow, COUNTDOWN, find_next(), paint()

### Community 119 - "Next-Steps Agents"
Cohesion: 0.36
Nodes (7): Agent A: FBO leak, mini player, digit zapping, Agent B: refresh guard, mirror probing, UA presets, Agent C: continue watching, auto-next (migration_5), Agent D: custom groups, failover (migration_6), Next-steps batch spawned (checkpoint), Next-steps batch merged (checkpoint), Next-steps batch merged on main eae59ff

### Community 120 - "Image Cache Pruning"
Cohesion: 0.28
Nodes (4): Entry, MAX_CACHE_BYTES, plan_prune(), prune()

### Community 121 - "Source Probing"
Cohesion: 0.28
Nodes (3): describe(), SAMPLE_BYTES, test_connection()

### Community 122 - "VM Seed Binary"
Cohesion: 0.39
Nodes (6): main(), seed_local(), seed_real(), wait_refresh(), App, db_path()

### Community 123 - "Xtream Conversion"
Cohesion: 0.36
Nodes (6): account_json(), actions(), check_account(), convert_media(), convert_stream(), UNCATEGORIZED_KEY

### Community 124 - "Backdrop FX"
Cohesion: 0.36
Nodes (6): backdrop(), backdrop_blurs_an_edge(), backdrop_darkens_and_keeps_size(), BLUR_SIGMA, DARKEN, two_colour()

### Community 126 - "Selection Model"
Cohesion: 0.28
Nodes (7): Sel, Category, Continue, Favorites, Group, sel_ids_round_trip(), Recent

### Community 129 - "Progress Marks"
Cohesion: 0.33
Nodes (4): BAR_H, CHECK, paint_bar(), paint_check()

### Community 130 - "Worker Tests"
Cohesion: 0.50
Nodes (7): add_source_refreshes_and_imports(), bad_source_reports_error_and_play_without_player_fails_cleanly(), m3u_source(), play_failure_is_reported_not_fatal(), second_refresh_of_same_source_is_skipped_until_done(), temp_db(), wait_for()

### Community 131 - "M6 Packaging History"
Cohesion: 0.36
Nodes (7): M6 phases 1+2 agents spawned (checkpoint), Packaging agent: CI, release workflow, packaging scripts, Polish agent: logo disk cache, keyboard nav, empty states, M6 polish + packaging merged (checkpoint), ci.yml and release.yml workflows, imgcache disk-backed BytesLoader, entitlements.plist for LuaJIT under hardened runtime

### Community 132 - "VM E2E Spec"
Cohesion: 0.46
Nodes (7): VM end-to-end playback tests spec, Xvfb :99 with xfwm4 display, scripts/vm/e2e.sh host script, Environment (UTM Xubuntu guest, no sudo), scripts/vm/guest-e2e.sh guest script, Generated test media (ch1-3, ep1-2) over local HTTP, Scenarios (play-live, zap, pause, fullscreen, mini, vod-seek, stop)

### Community 133 - "Plan: Playback"
Cohesion: 0.36
Nodes (6): Custom player templates (shlex, no shell), Embedded mpv via libmpv render API, External mpv with IPC instance reuse, Playback (PlayerKind, PlayRequest, resolution chain), External VLC launching, Settings dialog

### Community 135 - "M3U Parser Entry"
Cohesion: 0.48
Nodes (4): ParsedPlaylist, parse(), parse_detect(), hls_detection()

### Community 136 - "Project Conventions"
Cohesion: 0.38
Nodes (5): Parabole conventions, Module layout, Building and libmpv requirements, Missing libmpv falls back to external mpv, Parabole README

### Community 137 - "Channel Name Normalize"
Cohesion: 0.38
Nodes (4): fold(), normalize(), QUALITY, strip_country()

### Community 138 - "Source Kinds"
Cohesion: 0.33
Nodes (4): SourceKind, M3uFile, M3uUrl, Xtream

### Community 139 - "Project Direction Brief"
Cohesion: 0.33
Nodes (3): eframe built with glow and system_fonts, IPTV player Rust+egui direction (brief), Next: EPG (XMLTV), recording, Stalker

### Community 140 - "VM E2E History"
Cohesion: 0.47
Nodes (6): App bugs fixed: mpv_args ignored, video repaint on X11, PiP resize, VM e2e harness done (checkpoint), VM e2e harness scripts/vm (9 scenarios), E2E real-source mode (checkpoint), E2E_REAL=1 real-provider mode with redaction, live-catchup.lua fix: skip while paused, 10 s seek cooldown

### Community 142 - "egui Details View"
Cohesion: 0.53
Nodes (3): InfoStore, Key, Open

### Community 144 - "Live Catchup Lua"
Cohesion: 0.60
Nodes (3): on_cache(), set_catching(), update_live()

### Community 146 - "Slint Experiment Start"
Cohesion: 0.50
Nodes (3): Slint UI experiment started (checkpoint), Slint UI experiment (egui to Slint port), MpvCore split from EmbeddedPlayer shared by both UIs

### Community 147 - "Phase 2 Orchestration"
Cohesion: 0.83
Nodes (3): Slint phase 1 done, phase 2 running (checkpoint), Phase 2 delegates: player, browse, dialogs, Orchestrator scaffold: per-area Api globals and hosts

### Community 149 - "Retry Backoff"
Cohesion: 0.67
Nodes (3): backoff(), MAX_RETRIES, three_retries_with_doubling_delay()

### Community 151 - "Inventory Views"
Cohesion: 1.00
Nodes (3): Content list, Info window, Series view

## Ambiguous Edges - Review These
- `Left icon rail navigation (Search, Live, Movies, Series, Sources, profile, Settings)` → `Series Episodes Snapshot`  [AMBIGUOUS]
  tests/fixtures/slint-snapshots/series-episodes.png · relation: conceptually_related_to

## Knowledge Gaps
- **213 isolated node(s):** `parabole`, `APPIMAGE_EXTRACT_AND_RUN`, `DISPLAY`, `LIBGL_ALWAYS_SOFTWARE`, `Db` (+208 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 709 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **23 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `MediaKind` connect `DB Queries` to `Category Edit Mode`, `Track Prefs & Player Actions`, `Worker Tests`, `DB Shared Helpers`, `egui Autonext & Lifecycle`, `HTTP Image Loader`, `Worker Event Loop`, `Source Kinds`, `egui App Root`, `egui Details View`, `Slint Playback Controls`, `UI State Persistence`, `Command Palette`, `egui Sidebar Rows`, `Worker Commands`, `Browse Callbacks`, `Player Key Handling`, `Sources Module`, `Stream Rows & Sort`, `M3U Parsed Entries`, `Empty States`, `Channel Zapping`, `Media Details`, `Menus & Profile Dialog`, `Hero Banner`, `Play Session`, `Series Browse`, `Xtream Tests`, `egui Reload & Toasts`, `Row Actions (Slint)`, `mpv IPC`, `Zap Carousel`, `Player Argv`, `Browse Filters & Sort`, `M3U Tests`, `Player Settings`, `Live Low-Latency`, `Snapshot Testing`, `Palette Execution`, `egui Series View`, `Watch Progress`, `Resume Position`, `Player Kind Status`, `Xtream Conversion`?**
  _High betweenness centrality (0.111) - this node is a cross-community bridge._
- **Are the 6 inferred relationships involving `SlintApp` (e.g. with `set_playing()` and `BrowseArea`) actually correct?**
  _`SlintApp` has 6 INFERRED edges - model-reasoned connections that need verification._
- **What connects `parabole`, `APPIMAGE_EXTRACT_AND_RUN`, `DISPLAY` to the rest of the system?**
  _213 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Slint App Bootstrap` be split into smaller, more focused modules?**
  _Cohesion score 0.05396825396825397 - nodes in this community are weakly interconnected._
- **Why does `SourceId` connect `DB Queries` to `Category Edit Mode`, `DB Shared Helpers`, `egui Series Playback`, `Worker Event Loop`, `egui App Root`, `Slint Playback Controls`, `egui Details View`, `UI State Persistence`, `Profiles`, `egui App Actions`, `Worker Commands`, `Browse Callbacks`, `Stream List Model`, `Sources Module`, `Source Input Detection`, `Stream Rows & Sort`, `DB Overlay Tests`, `Media Details`, `Play Session`, `egui Buttons & Forms`, `Series Browse`, `egui Reload & Toasts`, `Account Info`, `DB Alternates Tests`, `egui Series View`, `Source Linking`, `DB Group Tests`, `Seasons & Episodes`, `Resume Position`, `Selection Model`, `egui Top Bar`?**
  _High betweenness centrality (0.092) - this node is a cross-community bridge._
- **Are the 22 inferred relationships involving `MediaKind` (e.g. with `.categories()` and `.favorites()`) actually correct?**
  _`MediaKind` has 22 INFERRED edges - model-reasoned connections that need verification._

### Low-confidence Hints
_AMBIGUOUS edges — the extractor was unsure. Verify before acting on these._

- **What is the exact relationship between `Left icon rail navigation (Search, Live, Movies, Series, Sources, profile, Settings)` and `Series Episodes Snapshot`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._