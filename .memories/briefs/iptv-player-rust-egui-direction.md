---
id: iptv-player-rust-egui-direction
title: IPTV player (Rust+egui) — direction
status: active
created: 2026-11-11T12:26:24.843Z
updated: 2026-11-11T12:53:25.263Z
tags:
  - planning
  - iptv
  - rust
  - egui
---

## Goal
Cross-platform desktop IPTV player, Rust + egui (glow renderer). Inputs: Xtream Codes, M3U/M3U8 URL/file (Stalker later). Players: embedded libmpv (later), external mpv (IPC reuse), VLC (external only), custom command templates.

## Key decisions (see PLAN.md, CLAUDE.md)
- egui uses the `glow` renderer (libmpv render API is OpenGL).
- Provider data is replaced on refresh; the user overlay is keyed by natural keys and survives.
- Xtream URLs are built at play time. Playlist URLs are untrusted (scheme allowlist, no shell).
- Code rules: one concern per file, under 300 lines, comments only for a why (small-context models maintain it).
- Migrations use PRAGMA user_version (rusqlite_migration is incompatible with rusqlite 0.42).

## Status
POC on main (b06a73b): sources/db/player/worker/ui done, 63 tests. External players only. Next: real-provider testing with mpv installed, then embedded libmpv spike, EPG.
