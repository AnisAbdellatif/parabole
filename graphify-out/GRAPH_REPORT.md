# Graph Report - parabole  (2026-11-15)

## Corpus Check
- 258 files · ~79,422 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2227 nodes · 4809 edges · 126 communities (106 shown, 20 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 392 edges (avg confidence: 0.73)
- Token cost: 57,137 input · 0 output

## Community Hubs (Navigation)
- Playback Session
- DB Queries
- Slint App Bootstrap
- Browse Area & Sorting
- Xtream Parsing
- Source Parser Tests
- DB Shared Helpers
- Player Area & Zap
- Software Platform & PiP
- Slint Video Surface
- Worker Event Loop
- Release Workflow
- Category Edit Mode
- Ambient Redesign Spec
- UI Icon Set
- VM Guest E2E Script
- Browse Snapshots
- Slint Image Cache
- CI Workflow
- External Player
- Settings & Zap Snapshots
- Player Key Handling
- Slint Groups & APIs
- Row Actions (Slint)
- Stream List Model
- Flatpak Manifest
- Palette Entries
- Sources Module
- Python Tooling Scripts
- DB Overlay Tests
- Worker Commands
- Command Palette
- mpv Core & Events
- Live Low-Latency
- Appearance Settings
- HTTP Client
- Playback Stats
- Stream Failover
- Hero Banner
- MpvCore Controls
- Settings Dialog
- Player Argv
- Worker Jobs & Events
- Poster Grid Model
- Redesign Mockup Layout
- Media Details
- Browse Callbacks
- Sources Dialog
- Packaging & CI Docs
- Slint Experiment Doc
- Series Browse
- Zap Carousel
- DB Alternates Tests
- Source Input Detection
- mpv IPC
- Source Kinds
- Plan: Data Model
- Dialog Area
- Real-Provider Scenarios
- Slint Reloads & Toasts
- Slint Entry & Paths
- Project Conventions Doc
- Binary Locate
- Snapshot Testing
- URL Validation
- Empty States
- Selection Model
- Palette Execution
- Fonts & Licenses
- README Build Notes
- Source Form Callbacks
- Phase 1 Spec
- Phase 2 Spec
- MpvCore
- Profiles
- Plan: Architecture
- UI Inventory
- Project Direction Brief
- Watch Progress
- Menus & Profile Dialog
- Channel Name Normalize
- Row Menus
- UI Review Passes
- Headless Tests Spec
- Slint Test Seed
- VM Host E2E Script
- Resume Position
- Category Order Edits
- Fuzzy Matching
- Next-Steps Agents
- VM Seed Binary
- Backdrop FX
- Worker Tests
- M6 Packaging History
- Slint Experiment Start
- Source Probing
- HTTP Image Loader
- Xtream HTTP Client
- VM E2E Spec
- Plan: Playback
- Browse Callbacks (2)
- Key Decisions (Memory)
- DB Migrations
- M3U Parser Entry
- DB Group Tests
- Slint Push Helpers
- Auto Refresh
- VM E2E History
- Slint Font Fallback
- Process Spawning
- Live Catchup Lua
- DB Group Tests (2)
- Track Prefs & Player Actions
- LRU Cache
- Phase 2 Orchestration
- Slint Redesign History
- Profile Ops
- Linux Packaging
- macOS Packaging
- macOS Smoke Test
- macOS Verify
- Parabole Binary

## God Nodes (most connected - your core abstractions)
1. `SlintApp` - 102 edges
2. `MediaKind` - 93 edges
3. `SourceId` - 92 edges
4. `ProfileId` - 66 edges
5. `Command` - 49 edges
6. `Source` - 48 edges
7. `StreamRow` - 48 edges
8. `Db` - 39 edges
9. `with_app()` - 38 edges
10. `PlayerKind` - 37 edges

## Surprising Connections (you probably didn't know these)
- `MpvCore toolkit-neutral split` --semantically_similar_to--> `MpvCore`  [INFERRED] [semantically similar]
  docs/slint-phase2.md → src/player/embed/core.rs
- `appearance.rs (Appearance struct, ACCENTS)` --semantically_similar_to--> `Appearance`  [INFERRED] [semantically similar]
  docs/slint-ambient-spec.md → src/slint_ui/appearance.rs
- `state.rs (Sel, Persisted, SlintApp, with_app)` --references--> `SlintApp`  [EXTRACTED]
  docs/slint-phase1-spec.md → src/slint_ui/state.rs
- `Living rows and StreamItem.playing` --references--> `StreamModel`  [EXTRACTED]
  docs/slint-ambient-spec.md → src/slint_ui/stream_model.rs
- `stream_model.rs lazy StreamModel` --references--> `StreamModel`  [EXTRACTED]
  docs/slint-phase1-spec.md → src/slint_ui/stream_model.rs

## Import Cycles
- 1-file cycle: `src/slint_ui/push.rs -> src/slint_ui/push.rs`
- 1-file cycle: `src/db/alt_name.rs -> src/db/alt_name.rs`
- 1-file cycle: `src/slint_ui/player/chrome.rs -> src/slint_ui/player/chrome.rs`
- 1-file cycle: `src/slint_ui/player/sync.rs -> src/slint_ui/player/sync.rs`
- 2-file cycle: `src/slint_ui/player/session.rs -> src/slint_ui/player/track_memory.rs -> src/slint_ui/player/session.rs`
- 2-file cycle: `src/slint_ui/player/autonext.rs -> src/slint_ui/player/session.rs -> src/slint_ui/player/autonext.rs`
- 2-file cycle: `src/sources/m3u/entry.rs -> src/sources/m3u/mod.rs -> src/sources/m3u/entry.rs`
- 2-file cycle: `src/sources/xtream/client.rs -> src/sources/xtream/mod.rs -> src/sources/xtream/client.rs`
- 2-file cycle: `src/worker/embed.rs -> src/worker/mod.rs -> src/worker/embed.rs`
- 2-file cycle: `src/slint_ui/browse/grid.rs -> src/slint_ui/stream_model.rs -> src/slint_ui/browse/grid.rs`

## Hyperedges (group relationships)
- **Source ingestion flow (detect, parse/fetch, refresh into overlay-safe tables)** — plan_input_detection, plan_m3u_parser, plan_xtream_client, plan_refresh, plan_sqlite_schema [EXTRACTED 0.85]
- **Slint experiment: start, phase 2, results, ambient, review passes** — _memories_2026_11_15_105711_1eb0_slint_experiment, _memories_2026_11_15_121146_81bd_slint_phase2_scaffold, _memories_2026_11_15_124837_5f1a_slint_vs_egui_measurements, _memories_2026_11_15_144818_86d4_ambient_redesign, _memories_2026_11_15_181950_9a68_ui_review_passes_merged [EXTRACTED 0.85]
- **VM e2e harness and real-source follow-up** — _memories_2026_11_15_163851_6f1d_vm_e2e_harness, _memories_2026_11_15_170726_1476_e2e_real_mode, _memories_2026_11_15_170726_1476_live_catchup_lua_fix, _memories_2026_11_15_163851_6f1d_app_bugs_fixed_by_e2e [EXTRACTED 0.85]
- **Playback backends (embedded mpv, external mpv, VLC, custom)** — plan_embedded_mpv, plan_external_mpv, plan_vlc, plan_custom_player [EXTRACTED 0.95]
- **Persistent app shell (icon rail, category sidebar, status bar)** — tests_fixtures_slint_snapshots_live_dark_icon_rail_navigation, tests_fixtures_slint_snapshots_live_dark_category_sidebar, tests_fixtures_slint_snapshots_live_dark_status_bar [EXTRACTED 0.95]
- **Ambient redesign stages S1 then parallel S2a/S2b/S2c** — docs_slint_ambient_spec_s1, docs_slint_ambient_spec_s2a, docs_slint_ambient_spec_s2b, docs_slint_ambient_spec_s2c [EXTRACTED 1.00]
- **Headless UI testing stack** — docs_slint_headless_tests_spec_prereq_build, docs_slint_headless_tests_spec_shared_seed, docs_slint_headless_tests_spec_behaviour, docs_slint_headless_tests_spec_snapshots, docs_slint_headless_tests_spec_sw_platform [EXTRACTED 1.00]
- **Phase 1 Rust glue pipeline** — docs_slint_phase1_spec_state, docs_slint_phase1_spec_images, docs_slint_phase1_spec_stream_model, docs_slint_phase1_spec_data, docs_slint_phase1_spec_push, docs_slint_phase1_spec_callbacks, docs_slint_phase1_spec_events, docs_slint_phase1_spec_run [EXTRACTED 1.00]
- **Content category navigation icons** — assets_icons_live_live, assets_icons_movie_movie, assets_icons_series_series, assets_icons_recent_recent, assets_icons_star_star [INFERRED 0.75]
- **Window and layout mode controls** — assets_icons_dock_bottom_dock_bottom, assets_icons_dock_right_dock_right, assets_icons_mini_mini, assets_icons_fullscreen_fullscreen, assets_icons_fullscreen_exit_fullscreen_exit [INFERRED 0.75]
- **Player configuration surfaces** — tests_fixtures_slint_snapshots_settings_playback_tab, tests_fixtures_slint_snapshots_settings_players_players_tab, tests_fixtures_slint_snapshots_settings_players_detect_button [INFERRED 0.75]
- **Player transport controls** — assets_icons_play_play, assets_icons_pause_pause, assets_icons_stop_stop, assets_icons_resume_resume [INFERRED 0.85]
- **Modal dialogs over dimmed app shell** — tests_fixtures_slint_snapshots_settings_players_settings_dialog, tests_fixtures_slint_snapshots_sources_sources_dialog, tests_fixtures_slint_snapshots_sources_form_add_source_form [INFERRED 0.85]
- **Orchestrator-designed parallel worktree agent batches** — _memories_2026_11_11_123726_e3bc_parallel_worktree_agents, _memories_2026_11_11_154833_2f33_agent_a_player_fixes_mini, _memories_2026_11_12_172857_6d14_m6_polish_agent, _memories_2026_11_15_121146_81bd_slint_phase2_delegates, _memories_2026_11_15_174542_6e20_ui_improvement_passes [INFERRED 0.85]
- **Modal overlays on dimmed backdrop** — tests_fixtures_slint_snapshots_appearance_settings_dialog, tests_fixtures_slint_snapshots_profiles_profiles_dialog, tests_fixtures_slint_snapshots_palette_chan_command_palette [INFERRED 0.85]
- **Theming system (accent, mode, light/dark)** — tests_fixtures_slint_snapshots_appearance_accent_color_picker, tests_fixtures_slint_snapshots_appearance_theme_mode_dark_oled_light, tests_fixtures_slint_snapshots_live_dark_dark_theme_teal_accent, tests_fixtures_slint_snapshots_live_light_light_theme [INFERRED 0.85]
- **Three channel zapping display modes** — tests_fixtures_slint_snapshots_zap_carousel_zap_carousel_mode, tests_fixtures_slint_snapshots_zap_fullscreen_zap_fullscreen_mode, tests_fixtures_slint_snapshots_zap_mini_zap_mini_mode [INFERRED 0.85]
- **Release builds three platform packages** — packaging_readme_release_workflow, packaging_readme_macos_app_dmg, packaging_readme_windows_zip, packaging_readme_linux_appimage [EXTRACTED 1.00]
- **Main window layout regions** — ui_inventory_top_bar, docs_redesign_sidebar, docs_redesign_content_area, docs_redesign_player_panel, docs_redesign_status_bar [EXTRACTED 1.00]
- **libmpv linking, bundling and runtime fallback** — readme_embedded_mpv, readme_libmpv, packaging_readme_dylibbundler, packaging_readme_windows_mpv_script, readme_external_mpv_fallback [INFERRED 0.85]

## Communities (126 total, 20 thin omitted)

### Community 0 - "Playback Session"
Cohesion: 0.05
Nodes (44): backoff(), MAX_RETRIES, three_retries_with_doubling_delay(), AutoNext, COUNTDOWN, find_next(), behind_s(), tech_line() (+36 more)

### Community 1 - "DB Queries"
Cohesion: 0.07
Nodes (23): Db, Db, stream_row(), Db, Db, Db, Db, Db (+15 more)

### Community 2 - "Slint App Bootstrap"
Cohesion: 0.08
Nodes (30): distance(), ink_rows(), px(), Rect, accessibility(), appearance(), behaviour(), chord() (+22 more)

### Community 3 - "Browse Area & Sorting"
Cohesion: 0.05
Nodes (24): chip(), connect(), entry(), grid_view(), load_sort(), media_from(), media_id(), SlintApp (+16 more)

### Community 4 - "Xtream Parsing"
Cohesion: 0.07
Nodes (33): find(), label(), labels(), next(), ordered(), Episode, Season, SeriesState (+25 more)

### Community 5 - "Source Parser Tests"
Cohesion: 0.07
Nodes (39): ParsedCategory, ParsedStream, scan(), Scanned, classify(), Draft, Pending, set_header() (+31 more)

### Community 6 - "DB Shared Helpers"
Cohesion: 0.07
Nodes (17): NAME_CANDIDATES, STREAM_SELECT, FavRow, Db, rebuild_fts(), Db, NewSource, classify() (+9 more)

### Community 7 - "Player Area & Zap"
Cohesion: 0.06
Nodes (23): Entry, file_name(), MAX_CACHE_BYTES, MAX_IMAGE_BYTES, plan_prune(), prune(), file_name_has_no_path_separators_for_hostile_urls(), file_name_is_stable_hex_and_distinct() (+15 more)

### Community 8 - "Software Platform & PiP"
Cohesion: 0.07
Nodes (16): connect(), enter(), Geometry, MARGIN_X, MARGIN_Y, restore(), save(), SIZE (+8 more)

### Community 9 - "Slint Video Surface"
Cohesion: 0.07
Nodes (9): allocate_storage(), create_target(), Loader, proc_address(), ProcAddress, install(), Saved, TOGGLES (+1 more)

### Community 10 - "Worker Event Loop"
Cohesion: 0.12
Nodes (13): hours(), Worker, EmbedPlay, episode_key(), SeriesCtx, Track, Worker, Worker (+5 more)

### Community 11 - "Release Workflow"
Cohesion: 0.06
Nodes (40): env, APPLE_APP_PASSWORD, APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD, APPLE_ID, APPLE_TEAM_ID, CARGO_TERM_COLOR, GITHUB_TOKEN (+32 more)

### Community 12 - "Category Edit Mode"
Cohesion: 0.11
Nodes (22): connect(), entry(), header(), matches(), menu_entries(), menu_picked(), prefix_of(), row() (+14 more)

### Community 13 - "Ambient Redesign Spec"
Cohesion: 0.10
Nodes (34): Ambient redesign spec (Slint only), Appearance dialog, appearance.rs (Appearance struct, ACCENTS), Player chrome with scrims and seek bar, palette/fuzzy.rs score, Ambient hero (browse-hero.slint), images.rs size classes (Logo, Poster, Backdrop), Living rows and StreamItem.playing (+26 more)

### Community 14 - "UI Icon Set"
Cohesion: 0.08
Nodes (32): audio icon - music note: audio track selection, back icon - left chevron: navigate back, check icon - checkmark: confirm/selected, close icon - X: close/dismiss, dock_bottom icon - panel docked at bottom of window, dock_right icon - panel docked at right of window, eye icon - open eye: show/visible, eye_off icon - struck-through eye: hide/hidden (+24 more)

### Community 15 - "VM Guest E2E Script"
Cohesion: 0.17
Nodes (27): bad(), crop_probe(), DISPLAY, gen(), geom(), hover_video(), is_bright(), key() (+19 more)

### Community 16 - "Browse Snapshots"
Cohesion: 0.10
Nodes (32): Accent color picker (6 swatches), Appearance Settings Snapshot, Density, corner radius, font size and reduce-motion controls, Dimmed backdrop modal overlay pattern, Settings modal dialog (Playback / Players / Appearance tabs), Theme mode selector: Dark / OLED / Light, Category sidebar (Favorites, Recent, My Groups, Categories with counts), Channel list rows with initials avatar and UHD badge (+24 more)

### Community 17 - "Slint Image Cache"
Cohesion: 0.14
Nodes (13): cache_keys_separate_sizes(), decode(), Decoded, Images, is_http(), Key, lru_evicts_least_recently_used(), Size (+5 more)

### Community 18 - "CI Workflow"
Cohesion: 0.07
Nodes (28): runs-on, steps, name, runs-on, steps, strategy, env, CARGO_TERM_COLOR (+20 more)

### Community 19 - "External Player"
Cohesion: 0.13
Nodes (8): exited_child_is_not_running(), ExternalPlayer, mpv_reuse_requires_live_mpv_child(), Running, sleep_args(), spawn_failure_is_reported_without_args(), start_replaces_previous_child(), start_stop_and_is_running()

### Community 20 - "Settings & Zap Snapshots"
Cohesion: 0.12
Nodes (27): Live Latency Checkboxes (low-latency buffering, catch up to live edge), Playback Tab (default/live/movie/episode player selectors), App Shell Layout (nav rail, category sidebar, channel list, status bar), Detect Player Binaries Button, Players Tab (mpv path, VLC path, custom command, extra mpv args), Tabbed Settings Modal (Playback / Players / Appearance), Settings Dialog - Players Tab, Settings Dialog - Playback Tab (+19 more)

### Community 21 - "Player Key Handling"
Cohesion: 0.17
Nodes (19): connect(), handle(), handle_keys(), help(), is(), neighbor_row(), play_embedded(), Pressed (+11 more)

### Community 22 - "Slint Groups & APIs"
Cohesion: 0.15
Nodes (8): connect(), tab_name(), connect(), drain(), kind_name(), set_playing(), with_app(), connect()

### Community 23 - "Row Actions (Slint)"
Cohesion: 0.10
Nodes (18): PlayerKind, Custom, Embedded, Mpv, Vlc, play(), RowAction, Activate (+10 more)

### Community 24 - "Stream List Model"
Cohesion: 0.13
Nodes (3): ImageHook, initials(), StreamModel

### Community 25 - "Flatpak Manifest"
Cohesion: 0.09
Nodes (22): app-id, build-options, append-path, build-args, command, finish-args, modules, build-commands (+14 more)

### Community 26 - "Palette Entries"
Cohesion: 0.11
Nodes (16): Db, Profile, Action, Density, OpenDialog, ReduceMotion, RefreshAll, SwitchProfile (+8 more)

### Community 27 - "Sources Module"
Cohesion: 0.20
Nodes (19): row_to_source(), MediaInfo, Source, check_account(), episode_url(), fetch(), fetch_series(), fetch_vod_info() (+11 more)

### Community 28 - "Python Tooling Scripts"
Cohesion: 0.10
Nodes (3): chunk(), inside_tri(), Handler

### Community 29 - "DB Overlay Tests"
Cohesion: 0.19
Nodes (18): playlist(), setup(), first_import_hides_nothing(), new_categories_hidden_only_for_opted_in_profile(), bulk_hide_applies_all_in_one_go(), delete_source_cascades_overlay_and_search(), failed_import_rolls_back(), names() (+10 more)

### Community 30 - "Worker Commands"
Cohesion: 0.09
Nodes (22): Command, AccountChecked, AddSource, AutoRefresh, CheckAccount, CopyUrl, DeleteSource, Fetched (+14 more)

### Community 31 - "Command Palette"
Cohesion: 0.15
Nodes (9): connect(), DEBOUNCE, item(), kind_of(), MAX_CATEGORIES, on_event(), PaletteArea, SlintApp (+1 more)

### Community 32 - "mpv Core & Events"
Cohesion: 0.17
Nodes (11): apply(), observe_all(), OBSERVED, poll(), Notice, Ended, Loaded, parse_tracks() (+3 more)

### Community 33 - "Live Low-Latency"
Cohesion: 0.15
Nodes (15): file_options(), keys(), live_gets_low_latency_options(), low_latency(), options_string(), options_string_quotes_values(), SCRIPT, SCRIPT_NAME (+7 more)

### Community 34 - "Appearance Settings"
Cohesion: 0.17
Nodes (9): ACCENTS, Appearance, clamps_out_of_range(), DENSITIES, rgb(), commit(), connect(), from_api() (+1 more)

### Community 35 - "HTTP Client"
Cohesion: 0.13
Nodes (8): client(), download(), blocked_hint(), client(), get(), SafeRead, SafeRead<R>, DEFAULT_USER_AGENT

### Community 36 - "Playback Stats"
Cohesion: 0.12
Nodes (6): get(), MpvCore, NONE, num(), Stats, value()

### Community 37 - "Stream Failover"
Cohesion: 0.18
Nodes (13): candidates(), fake(), pick(), pick_index(), pick_index_does_not_wait_for_slow_less_preferred(), pick_index_prefers_configured_order_over_speed(), pick_skips_dead_primary(), PROBE_TIMEOUT (+5 more)

### Community 38 - "Hero Banner"
Cohesion: 0.16
Nodes (11): action(), build(), connect(), fmt_hms(), focus(), HeroState, meta(), push() (+3 more)

### Community 39 - "MpvCore Controls"
Cohesion: 0.13
Nodes (8): origin_header(), file_options(), live_request_gets_latency_options_vod_gets_start(), MpvCore, TrackKind, Audio, Sub, Video

### Community 40 - "Settings Dialog"
Cohesion: 0.19
Nodes (11): SlintApp, connect(), detect_text(), kind_at(), kind_label(), kind_names(), opt_path(), path_text() (+3 more)

### Community 41 - "Player Argv"
Cohesion: 0.25
Nodes (15): Headers, PlayRequest, custom(), custom_does_not_reexpand_and_handles_missing_values(), custom_substitutes_per_token(), custom_title_cannot_add_argv_items(), mpv(), mpv_full_argv() (+7 more)

### Community 42 - "Worker Jobs & Events"
Cohesion: 0.11
Nodes (15): Event, AccountChecked, Error, InfoLoaded, PlayEmbedded, PlayerStopped, PlayFailed, Playing (+7 more)

### Community 44 - "Redesign Mockup Layout"
Cohesion: 0.12
Nodes (16): Category edit mode (hide/show, pin, drag reorder), Content area, Filter + Hide matching categories, Live channel list with favorite stars, Media type tabs (Live/Movies/Series), Pinned categories section, Poster grid for movies/series, Profile selector (Me/Kids) (+8 more)

### Community 45 - "Media Details"
Cohesion: 0.25
Nodes (13): cached(), close(), connect(), fields(), InfoStore, Key, loaded(), Open (+5 more)

### Community 46 - "Browse Callbacks"
Cohesion: 0.20
Nodes (10): add_to_group(), delete_group(), GroupCtx, open_group_dialog(), refresh_groups(), remove_from_group(), request_dialog(), RECENT_LIMIT (+2 more)

### Community 47 - "Sources Dialog"
Cohesion: 0.18
Nodes (5): account_text(), ago(), now(), read_fields(), SlintApp

### Community 48 - "Packaging & CI Docs"
Cohesion: 0.17
Nodes (14): cargo fmt / clippy -D warnings / test (also --no-default-features), CI workflow (fmt, clippy, tests on macOS/Linux/Windows), dylibbundler bundles libmpv into Contents/Frameworks, Flatpak (dev.parabole.app.yml), Placeholder icon (make_icon.py), Linux AppImage (linux.sh), macOS .app + .dmg (macos.sh, verify-macos.sh, smoke-macos.sh), Packaging (+6 more)

### Community 49 - "Slint Experiment Doc"
Cohesion: 0.22
Nodes (15): Slint UI experiment, Accessibility pass (ui/a11y), Ambient redesign (summary), egui vs Slint comparison, Default UI switch (Slint default, parabole-egui), Dev-experience notes and Slint pitfalls, DejaVu fontique fallback (fonts.rs), Gaps list (+7 more)

### Community 50 - "Series Browse"
Cohesion: 0.27
Nodes (13): close(), connect(), episode_id(), loaded(), open(), play_episode(), PROGRESS_EVERY, push() (+5 more)

### Community 51 - "Zap Carousel"
Cohesion: 0.20
Nodes (12): close(), fill_pane(), initials(), item(), OPEN_FOR, PANE_SPAN, pane_window(), set_open() (+4 more)

### Community 52 - "DB Alternates Tests"
Cohesion: 0.19
Nodes (13): id_of(), ignores_hidden_unlinked_and_non_live(), load(), matches_by_epg_id_then_normalized_name(), names(), with_epg(), import_200k_streams(), cat() (+5 more)

### Community 53 - "Source Input Detection"
Cohesion: 0.28
Nodes (13): SourceInput, M3uFile, M3uUrl, Xtream, base_of(), detect(), detect_line(), detect_url() (+5 more)

### Community 54 - "mpv IPC"
Cohesion: 0.22
Nodes (9): live_payload_carries_low_latency_options(), loadfile_payload(), payload_format(), req(), send(), send_reports_rejection_and_dead_socket(), send_skips_events_and_accepts_success(), serve_once() (+1 more)

### Community 55 - "Source Kinds"
Cohesion: 0.18
Nodes (8): SourceKind, M3uFile, M3uUrl, Xtream, builds_m3u_with_host_as_default_name(), default_name(), detect_paste(), Fields

### Community 56 - "Plan: Data Model"
Cohesion: 0.23
Nodes (13): Data model: provider data vs user overlay, Fred TV (open-tv), Add-source dialog and input detection, Lenient Xtream deserializers, M3U/M3U8 streaming parser, Natural keys (category_id, stream_id, tvg-id, group-title), Transactional source refresh, SQLite schema (profiles, sources, categories, streams, FTS) (+5 more)

### Community 57 - "Dialog Area"
Cohesion: 0.20
Nodes (4): connect(), DialogArea, on_event(), SlintApp

### Community 58 - "Real-Provider Scenarios"
Cohesion: 0.24
Nodes (11): click_stop(), lit(), moving(), pick(), r_fullscreen(), r_pause(), r_play_live(), r_stop() (+3 more)

### Community 60 - "Slint Entry & Paths"
Cohesion: 0.18
Nodes (3): build(), run(), install_mpv_script()

### Community 61 - "Project Conventions Doc"
Cohesion: 0.18
Nodes (6): src/db SQLite storage, Module layout, src/model.rs shared data types, Parabole conventions, src/player external player launching (mpv/VLC/custom), src/sources (M3U, Xtream, input detection, URL resolution)

### Community 62 - "Binary Locate"
Cohesion: 0.28
Nodes (10): custom_resolves_first_token(), env_dir(), find_binary(), locate(), mpv_defaults(), path_walk_finds_binary(), scratch(), search_path() (+2 more)

### Community 63 - "Snapshot Testing"
Cohesion: 0.22
Nodes (5): CHANNEL_TOLERANCE, check(), differs(), MAX_DIFF_RATIO, root()

### Community 64 - "URL Validation"
Cohesion: 0.23
Nodes (9): ALLOWED_SCHEMES, clean_opt(), clean_request(), clean_request_cleans_headers(), error_does_not_leak_url(), MAX_HEADER, MAX_TITLE, sanitize() (+1 more)

### Community 65 - "Empty States"
Cohesion: 0.27
Nodes (6): Content, Cta, ClearSearch, ManageSources, Retry, SlintApp

### Community 66 - "Selection Model"
Cohesion: 0.18
Nodes (8): Persisted, Sel, Category, Continue, Favorites, Group, sel_ids_round_trip(), Recent

### Community 68 - "Fonts & Licenses"
Cohesion: 0.18
Nodes (10): DejaVu / Bitstream Vera font license, Arev Fonts Copyright (Tavmjong Bah), Bitstream Vera Fonts Copyright, Inter font license (SIL OFL 1.1), SIL Open Font License 1.1, Later features (EPG, catch-up, recording, Stalker, PIN), M0 spikes (libmpv in egui, font fallback), Milestones M0-M6 and later work (+2 more)

### Community 69 - "README Build Notes"
Cohesion: 0.20
Nodes (10): Slint UI (ui-slint/ + src/slint_ui/, feature slint-ui), src/worker.rs background thread, Main window mockup (1440x900), Docked player panel (right or bottom), Status bar (sources synced, item counts, profile), Toast notification (source updated), Building, embedded-mpv feature (embedded libmpv player) (+2 more)

### Community 70 - "Source Form Callbacks"
Cohesion: 0.21
Nodes (6): connect(), CUSTOM_UA, preset_index(), reset_form(), is_custom(), PRESETS

### Community 71 - "Phase 1 Spec"
Cohesion: 0.33
Nodes (12): Phase 1 spec: Rust glue for Slint main window, callbacks.rs connect, data.rs reloads, events.rs drain, images.rs logo loader (Images, LRU), imgcache get_or_fetch and default_dir, push.rs state to Core properties, mod.rs run() (+4 more)

### Community 72 - "Phase 2 Spec"
Cohesion: 0.38
Nodes (11): Phase 2 spec: three parallel areas, Area A: player (exp/slint-player), Area B: browse (exp/slint-browse), Area C: dialogs (exp/slint-dialogs), Area scaffold (Api global, host, Area state, on_event), dialog-kit.slint shared widgets, DialogApi.open-dialog and BrowseApi.request-dialog, Event routing: dialogs, player, browse, core (+3 more)

### Community 73 - "MpvCore"
Cohesion: 0.23
Nodes (3): MpvCore, INIT_OPTIONS, split_mpv_arg()

### Community 74 - "Profiles"
Cohesion: 0.24
Nodes (5): Db, hide_new(), keys(), setting_key(), Db

### Community 75 - "Plan: Architecture"
Cohesion: 0.20
Nodes (11): Parabole Plan (Rust + egui), Architecture: UI thread + worker thread, Cinebox, Command / Event channels, egui-mpv-glow reference, Hypnotix, IPTVnator, Module layout (src tree) (+3 more)

### Community 76 - "UI Inventory"
Cohesion: 0.18
Nodes (11): Category edit mode (drag, hide, pin, Hide matching), Parabole UI inventory (for redesign), Category sidebar (normal mode), Content list, Embedded player (pane, overlay controls, fullscreen), Info window, Keyboard shortcuts, Main window layout (+3 more)

### Community 77 - "Project Direction Brief"
Cohesion: 0.18
Nodes (7): eframe built with glow and system_fonts, Redesign foundation committed (checkpoint), Redesign foundation commit 9f03c22 (src/ui/theme), Redesign agents A topbar/dialogs, B sidebar/list, C player, IPTV player Rust+egui direction (brief), Next: EPG (XMLTV), recording, Stalker, Key decisions

### Community 78 - "Watch Progress"
Cohesion: 0.20
Nodes (7): Watch, Partial, Unstarted, Watched, continue_lists_partial_movies_newest_first(), movies(), series_continue_is_one_row_per_series_with_episode()

### Community 79 - "Menus & Profile Dialog"
Cohesion: 0.25
Nodes (3): connect(), copy_name(), MOD_LABEL

### Community 80 - "Channel Name Normalize"
Cohesion: 0.27
Nodes (6): fold(), normalize(), QUALITY, strip_country(), Db, Alternate

### Community 81 - "Row Menus"
Cohesion: 0.40
Nodes (7): action(), connect(), entries(), header(), in_group(), item(), separator()

### Community 82 - "UI Review Passes"
Cohesion: 0.24
Nodes (9): Slint UI improvement passes started (checkpoint), Pass ui/browse-density, Pass ui/dialogs, Pass ui/nav-search, Pass ui/player-pane, Four parallel Opus 5.5 UI passes on disjoint files, Slint UI review passes merged (checkpoint), Pass ui/a11y accessibility (+1 more)

### Community 83 - "Headless Tests Spec"
Cohesion: 0.38
Nodes (10): Headless UI tests (behaviour and snapshots), Slint headless UI tests spec, Alignment asserts, Behaviour tests (tests/slint_behaviour.rs), Dev-dependencies (i-slint-backend-testing, software renderer), Done when criteria, Prerequisite refactor: build() split from run(), Shared seed (tests/common/slint_seed.rs) (+2 more)

### Community 85 - "VM Host E2E Script"
Cohesion: 0.38
Nodes (7): build(), fetch(), guest(), e2e.sh script, sync_src(), test_run(), up()

### Community 86 - "Resume Position"
Cohesion: 0.24
Nodes (6): Db, MAX_FRACTION, MIN_FRACTION, record_watch_keeps_saved_position(), resume_start(), save_then_resume()

### Community 87 - "Category Order Edits"
Cohesion: 0.38
Nodes (6): bulk(), connect(), move_item(), set_edit(), SlintApp, start_with_filter()

### Community 88 - "Fuzzy Matching"
Cohesion: 0.27
Nodes (3): greedy(), lower(), score()

### Community 89 - "Next-Steps Agents"
Cohesion: 0.36
Nodes (7): Agent A: FBO leak, mini player, digit zapping, Agent B: refresh guard, mirror probing, UA presets, Agent C: continue watching, auto-next (migration_5), Agent D: custom groups, failover (migration_6), Next-steps batch spawned (checkpoint), Next-steps batch merged (checkpoint), Next-steps batch merged on main eae59ff

### Community 90 - "VM Seed Binary"
Cohesion: 0.39
Nodes (5): main(), seed_local(), seed_real(), wait_refresh(), db_path()

### Community 91 - "Backdrop FX"
Cohesion: 0.36
Nodes (6): backdrop(), backdrop_blurs_an_edge(), backdrop_darkens_and_keeps_size(), BLUR_SIGMA, DARKEN, two_colour()

### Community 92 - "Worker Tests"
Cohesion: 0.50
Nodes (7): add_source_refreshes_and_imports(), bad_source_reports_error_and_play_without_player_fails_cleanly(), m3u_source(), play_failure_is_reported_not_fatal(), second_refresh_of_same_source_is_skipped_until_done(), temp_db(), wait_for()

### Community 93 - "M6 Packaging History"
Cohesion: 0.36
Nodes (7): M6 phases 1+2 agents spawned (checkpoint), Packaging agent: CI, release workflow, packaging scripts, Polish agent: logo disk cache, keyboard nav, empty states, M6 polish + packaging merged (checkpoint), ci.yml and release.yml workflows, imgcache disk-backed BytesLoader, entitlements.plist for LuaJIT under hardened runtime

### Community 94 - "Slint Experiment Start"
Cohesion: 0.29
Nodes (6): Slint UI experiment started (checkpoint), Slint UI experiment (egui to Slint port), Slint experiment complete (checkpoint), MpvCore split from EmbeddedPlayer shared by both UIs, Slint shapes Arabic RTL; egui lacks bidi, Slint vs egui measurements

### Community 95 - "Source Probing"
Cohesion: 0.32
Nodes (3): describe(), SAMPLE_BYTES, test_connection()

### Community 98 - "VM E2E Spec"
Cohesion: 0.46
Nodes (7): VM end-to-end playback tests spec, Xvfb :99 with xfwm4 display, scripts/vm/e2e.sh host script, Environment (UTM Xubuntu guest, no sudo), scripts/vm/guest-e2e.sh guest script, Generated test media (ch1-3, ep1-2) over local HTTP, Scenarios (play-live, zap, pause, fullscreen, mini, vod-seek, stop)

### Community 99 - "Plan: Playback"
Cohesion: 0.36
Nodes (6): Custom player templates (shlex, no shell), Embedded mpv via libmpv render API, External mpv with IPC instance reuse, Playback (PlayerKind, PlayRequest, resolution chain), External VLC launching, Settings dialog

### Community 100 - "Browse Callbacks (2)"
Cohesion: 0.50
Nodes (5): clear_search(), connect(), reset_cursor(), SEARCH_DELAY, select()

### Community 101 - "Key Decisions (Memory)"
Cohesion: 0.33
Nodes (5): POC orchestration started (checkpoint), Skeleton commit 38cccfe with stubbed public APIs, POC merged on main (checkpoint), POC known shortcuts (FTS rebuild, unvirtualized favorites), POC merged: 63 tests, 5.8k lines

### Community 102 - "DB Migrations"
Cohesion: 0.38
Nodes (4): Conflicting migration numbers (checkpoint), Parallel agents created conflicting migration_3.sql, MIGRATIONS, run()

### Community 103 - "M3U Parser Entry"
Cohesion: 0.48
Nodes (4): ParsedPlaylist, parse(), parse_detect(), fetch()

### Community 104 - "DB Group Tests"
Cohesion: 0.29
Nodes (3): account_json_roundtrip(), copy_appends_missing_favorites_in_order_without_duplicates(), settings_global_vs_profile()

### Community 106 - "Auto Refresh"
Cohesion: 0.33
Nodes (4): DEFAULT_HOURS, hours_default_and_roundtrip(), KEY, set_hours()

### Community 107 - "VM E2E History"
Cohesion: 0.47
Nodes (6): App bugs fixed: mpv_args ignored, video repaint on X11, PiP resize, VM e2e harness done (checkpoint), VM e2e harness scripts/vm (9 scenarios), E2E real-source mode (checkpoint), E2E_REAL=1 real-provider mode with redaction, live-catchup.lua fix: skip while paused, 10 s seek cooldown

### Community 110 - "Live Catchup Lua"
Cohesion: 0.60
Nodes (3): on_cache(), set_catching(), update_live()

### Community 111 - "DB Group Tests (2)"
Cohesion: 0.70
Nodes (4): group_items_survive_refresh_and_show_missing(), groups_are_ordered_and_items_append_without_duplicates(), groups_are_per_profile_and_cascade(), setup_two()

### Community 112 - "Track Prefs & Player Actions"
Cohesion: 0.40
Nodes (4): LangKind, Audio, Sub, TrackLangs

### Community 114 - "Phase 2 Orchestration"
Cohesion: 0.83
Nodes (3): Slint phase 1 done, phase 2 running (checkpoint), Phase 2 delegates: player, browse, dialogs, Orchestrator scaffold: per-area Api globals and hosts

### Community 115 - "Slint Redesign History"
Cohesion: 0.67
Nodes (3): Slint Ambient redesign (S1, S2a, S2b, S2c), Slint Ambient redesign merged (checkpoint), Headless Slint behaviour and snapshot tests

## Ambiguous Edges - Review These
- `Left icon rail navigation (Search, Live, Movies, Series, Sources, profile, Settings)` → `Series Episodes Snapshot`  [AMBIGUOUS]
  tests/fixtures/slint-snapshots/series-episodes.png · relation: conceptually_related_to

## Knowledge Gaps
- **153 isolated node(s):** `ClearSearch`, `ManageSources`, `Retry`, `SlintApp`, `Partial` (+148 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 552 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **20 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `MediaKind` connect `DB Queries` to `Xtream Parsing`, `Source Parser Tests`, `DB Shared Helpers`, `Worker Event Loop`, `Player Key Handling`, `Slint Groups & APIs`, `Row Actions (Slint)`, `Palette Entries`, `Sources Module`, `Worker Commands`, `Command Palette`, `Live Low-Latency`, `Hero Banner`, `MpvCore Controls`, `Player Argv`, `Worker Jobs & Events`, `Media Details`, `Browse Callbacks`, `Series Browse`, `Zap Carousel`, `mpv IPC`, `Source Kinds`, `Snapshot Testing`, `Selection Model`, `Palette Execution`, `Watch Progress`, `Row Menus`, `Resume Position`, `Worker Tests`, `Browse Callbacks (2)`, `Track Prefs & Player Actions`?**
  _High betweenness centrality (0.078) - this node is a cross-community bridge._
- **Are the 6 inferred relationships involving `SlintApp` (e.g. with `set_playing()` and `BrowseArea`) actually correct?**
  _`SlintApp` has 6 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ClearSearch`, `ManageSources`, `Retry` to the rest of the system?**
  _153 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Playback Session` be split into smaller, more focused modules?**
  _Cohesion score 0.05030864197530864 - nodes in this community are weakly interconnected._
- **Why does `SlintApp` connect `Browse Callbacks` to `DB Queries`, `Browse Area & Sorting`, `DB Shared Helpers`, `Player Area & Zap`, `Worker Event Loop`, `Slint Image Cache`, `Player Key Handling`, `Slint Groups & APIs`, `Row Actions (Slint)`, `Stream List Model`, `Palette Entries`, `Sources Module`, `Command Palette`, `Appearance Settings`, `Hero Banner`, `Settings Dialog`, `Media Details`, `Series Browse`, `Zap Carousel`, `Dialog Area`, `Selection Model`, `Palette Execution`, `Phase 1 Spec`, `Menus & Profile Dialog`, `Row Menus`, `Browse Callbacks (2)`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **Are the 21 inferred relationships involving `MediaKind` (e.g. with `.categories()` and `.favorites()`) actually correct?**
  _`MediaKind` has 21 INFERRED edges - model-reasoned connections that need verification._

### Low-confidence Hints
_AMBIGUOUS edges — the extractor was unsure. Verify before acting on these._

- **What is the exact relationship between `Left icon rail navigation (Search, Live, Movies, Series, Sources, profile, Settings)` and `Series Episodes Snapshot`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._