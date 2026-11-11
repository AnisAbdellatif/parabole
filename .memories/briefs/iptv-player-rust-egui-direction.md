---
id: iptv-player-rust-egui-direction
title: IPTV player (Rust+egui) — direction
status: active
created: 2026-11-11T12:26:24.843Z
updated: 2026-11-11T12:26:24.843Z
tags:
  - planning
  - iptv
  - rust
  - egui
---

## Goal
Cross-platform desktop IPTV player, Rust + egui (glow renderer). Inputs: Xtream Codes, M3U/M3U8 URL/file (Stalker later). Players: embedded libmpv, external mpv (IPC reuse), VLC (external only), custom command templates.

## Key decisions (see PLAN.md)
- egui must use `glow` renderer (libmpv render API is OpenGL); wgpu rejected (no overlay).
- VLC external only: vlc-rs dead, libvlc embed = CPU copies.
- Provider data wiped+reinserted per refresh; user overlay (favorites/hidden/order/history) keyed by natural keys (xtream ids, m3u tvg-id or name+group) so it survives refresh. Lesson from Fred TV storing favorite on channel row.
- Xtream stream URLs built at play time, not stored.
- Playlist URLs are untrusted: scheme allowlist, reject leading '-', no shell for custom templates.
- License likely GPL-3 because of bundled libmpv.

## Milestones
M0 skeleton+spikes (libmpv-in-egui per OS, Arabic/CJK fonts/RTL shaping) → M1 sources → M2 browse → M3 external playback → M4 organization/profiles → M5 embedded mpv → M6 packaging. Later: EPG, catch-up, recording, Stalker, PIN.

## Status
Plan written (PLAN.md), no code yet. Open question to user: profiles = personas over shared sources vs one profile per account.
