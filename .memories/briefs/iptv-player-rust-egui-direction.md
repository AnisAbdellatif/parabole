---
id: iptv-player-rust-egui-direction
title: IPTV player (Rust+egui) — direction
status: active
created: 2026-11-11T12:26:24.843Z
updated: 2026-11-12T17:47:18.532Z
tags:
  - planning
  - iptv
  - rust
  - egui
---

## Goal
Cross-platform desktop IPTV player, Rust + egui (glow renderer). Inputs: Xtream Codes, M3U/M3U8 URL or file (Stalker later). Players: embedded libmpv, external mpv (IPC reuse), VLC (external only), custom command templates.

## Key decisions (see PLAN.md, CLAUDE.md)
- egui uses the `glow` renderer, because the libmpv render API is OpenGL.
- Provider data is replaced on refresh. The user overlay is keyed by natural keys and survives refreshes.
- Xtream URLs are built at play time. Playlist URLs are untrusted: scheme allowlist, no shell.
- Code rules: one concern per file, files under 300 lines, comments only for a why. The code is maintained by models with small context windows.
- Migrations use PRAGMA user_version (rusqlite_migration is incompatible with rusqlite 0.42).
- Packaging uses shell scripts plus dylibbundler in `packaging/`, not cargo-packager. CI and release live in `.github/workflows`.

## Status
PLAN.md §9 milestones M0–M6 are done on main, plus the QoL and organizing extras. 136 tests pass.

Not yet verified on GitHub: CI, the release workflow, and Windows/Linux/Flatpak packaging. The app icon is a placeholder.

Next: EPG (XMLTV), then recording, Stalker, and the rest of PLAN.md's Later list.
