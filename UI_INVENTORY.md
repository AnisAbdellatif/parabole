# IPTV Player — UI inventory (for redesign)

Desktop IPTV player (macOS / Windows / Linux) built with Rust + egui. It plays live TV channels, movies and series from the user's own IPTV playlists. Everything below exists today; the redesign may regroup, restyle or rename it, but should keep every function.

## 1. Main window layout

```
┌ Top bar ───────────────────────────────────────────────────────────────┐
├ Category sidebar ┬ Content list / series view ┬ Player pane (optional) ┤
│ (left, resizable)│ (center)                   │ (right or bottom dock)  │
├ Status bar ─────────────────────────────────────────────────────────────┤
└ Toasts (bottom-right, transient)                                        ┘
```

## 2. Top bar
| Element | Type | What it does |
|---|---|---|
| Profile picker | dropdown | Switches the viewer profile (e.g. "Me", "Kids"). Each profile has its own favorites, hidden or reordered categories, and history. |
| Profiles… | small button | Opens the Profiles dialog (§9). |
| Source filter | dropdown | "All sources" or one playlist. Filters the sidebar and the list. |
| Live / Movies / Series | 3 tabs | Switches the media type shown everywhere below. |
| Search | text field, hint "Search (Ctrl+F)" | Instant search over channel/movie/series names in the current tab. Results replace the list. Esc clears it. |
| Sources… | button | Opens the Sources dialog (§8). |
| Settings… | button | Opens the Settings dialog (§10). |

## 3. Category sidebar (normal mode)
| Element | Type | What it does |
|---|---|---|
| "Categories" title + **Edit** toggle | heading + toggle | Edit switches the sidebar into edit mode (§4). |
| ★ Favorites | virtual entry | Shows the favorite channels of the current tab, across all sources. |
| ⟲ Recent | virtual entry | Recently watched items, newest first. |
| Pinned categories | list rows | Categories the user pinned, always on top. |
| Category rows | selectable rows | Name + item count badge. When "All sources" is selected and there are several sources, the name is prefixed with the source name. A ☆/★ toggle pins or unpins the category. Hidden categories are not shown. |

Typical scale: 10 to 1,500 categories per source. Names often carry country prefixes like `FR |`, `UK:`, `[AR]`.

## 4. Category sidebar (edit mode)
| Element | Type | What it does |
|---|---|---|
| Hint | text | "Select one source to reorder categories." Shown when "All sources" is active, because order is stored per source. |
| Category rows | draggable rows | ☰ drag handle, eye toggle (show/hide), ☆ pin toggle, name, count. Hidden rows are shown dimmed. |
| Show all / Hide all | buttons | Bulk show or hide every category. |
| Reset order | button | Back to the provider's order. Needs a single source selected. |
| Sort A–Z | button | Alphabetical order, saved as the custom order. Pinned ones stay on top. Needs a single source. |
| Filter field | text field, hint "Filter, e.g. FR \|" | Case-insensitive text match on category names. |
| Hide matching | button | Hides every category whose name contains the filter text. |
| Show only matching | button | Hides everything else. |
| Hide new categories | checkbox | Per source and profile: categories that appear in later refreshes start hidden. |

## 5. Content list (center)
| Element | Type | What it does |
|---|---|---|
| List title | heading | Category name, "★ Favorites", "⟲ Recent", or `Search: "news" (42)`. "Select a category" when nothing is selected. |
| Sort selector | dropdown | Provider order / Number / A–Z. Remembered per category. Hidden for Favorites and Recent. |
| Rows | virtualized list (can hold 10k+ rows) | Channel number, logo (small image from URL), name, ☆/★ favorite toggle. Double-click or Enter plays. Arrow keys and PgUp/PgDn move the selection. |
| Hidden rows (edit mode only) | dimmed rows + eye toggle | Lets the user bring hidden channels back. |
| Unavailable favorites | dimmed row, "(unavailable)" | A favorite the provider removed. Not playable; it can only be removed. It comes back to normal when the provider lists it again. |
| Favorites view | draggable rows | Drag to reorder favorites. Drag turns off above 300 favorites. |
| Row context menu (right-click) | menu | Favorite / Unfavorite · Hide / Unhide · Info… (movies and series) · Play with ▸ (Embedded / mpv / VLC / Custom) · Copy stream URL. |
| Empty state: no sources | heading + button | "Add a source" button opens the Sources dialog. |
| Empty state: empty list | text | "Nothing here." |

Movies and series currently use the same row list; a poster grid would be welcome.

## 6. Series view (replaces the list when a series is opened)
| Element | Type | What it does |
|---|---|---|
| ← Back | button | Returns to the list. |
| Series title | heading | |
| Details block | poster + fields | Same content as the Info window (§7). |
| Season picker | dropdown | Season 1, 2, … |
| Episode list | rows | Episode number + title. Double-click plays. |

## 7. Info window (movies and series)
Floating window titled with the item's name. Shows the poster/cover and the fields Genre, Released, Duration, Rating, Director, Cast and Plot (any of them may be missing). States: "Loading details…", "No details available.". For plain M3U playlists only the name and logo are known.

## 8. Sources dialog (playlists)
**Source list**
| Element | Type | What it does |
|---|---|---|
| Source row | row | Name, kind (xtream / m3u_url / m3u_file), "refreshed 3 h ago", a spinner while refreshing, last error in red. Buttons: Refresh, Edit, Delete (Delete asks "Really delete" / "Cancel"). |
| Account panel (Xtream only) | key/value block | Status, Expires (orange under 7 days, red when expired, "unlimited"), Connections active/max (orange at the limit), Trial, Created. Has a **Check account** button. |
| Refresh all | button | |
| Auto-refresh at startup after N hours | number field | 0 turns it off. Default 24. |
| Empty state | text | "No sources yet." |

**Add / edit form**
| Element | Type | What it does |
|---|---|---|
| Paste anything | multiline field | Accepts an M3U link, an Xtream link (`get.php?username=…`), "server user pass" on one or three lines, or a file path. The detected type appears below it as a hint. |
| Name | text field | |
| URL / file + Choose file… | text field + button | The button opens the OS file picker. |
| Backup URLs | multiline field, hint "Xtream only, one per line" | Tried in order when the main server fails. |
| Username / Password | text fields (password masked) | Xtream only. |
| User agent | text field | Some providers require a specific one. |
| Test | button | Checks the source before saving. The result appears in green ("OK: Active, expires …, 1/2 connections" or "M3U playlist (≈278 entries)") or in red with the error. |
| Add / Save | button | |
| Cancel edit | button | |

## 9. Profiles dialog
List of profiles with Rename (inline field, Save / Cancel), Duplicate (copies favorites, hidden and pinned categories and settings into "<name> copy"), and Delete (confirm with "Really delete" / "Cancel"). Plus creating a new profile.

## 10. Settings dialog
| Element | Type | What it does |
|---|---|---|
| Default player | dropdown | Embedded / mpv / VLC / Custom. |
| Live / Movies / Episodes player | 3 dropdowns | "Default" or a specific player per content type. |
| Live latency | 2 checkboxes | "Low-latency buffering" and "Catch up to the live edge". |
| mpv path / VLC path | text fields | Empty means auto-detect. |
| Detect | button + result text | Shows which player binaries were found. |
| Custom command | text field | A template such as `potplayer {url} /title={title}`. |
| Extra mpv args | text field | |
| Save | button | |

## 11. Embedded player
**Pane:** docked right or bottom and resizable. Header buttons: dock right ⇥ / dock bottom ⇩, close ✕. The video fills the pane.

**Overlay controls** sit on top of the video and fade out after 3 s without mouse movement:
- Play/pause ▶/⏸ and Stop ■.
- Mute 🔊/🔇 and a volume slider (0–130 %).
- Seek slider with elapsed/total time (movies and episodes only).
- Audio ▾ and Subtitles ▾ track menus (track list from the stream; "Off" for subtitles).
- Go live (live only; reloads at the live edge).
- Fullscreen.
- Buffering spinner while the stream loads or stalls.
- Title of the current channel or movie.

**Fullscreen:** the video fills the whole window with the same overlay. Double-click on the video enters it and Esc exits.

**Behaviour the UI should be able to show:** auto-retry after a live stream drops (up to 3 attempts), resume position for movies, and the fallback to external mpv if the embedded player fails (via a toast).

## 12. Status bar and toasts
- **Status bar:** "Playing: <title> in mpv" (or VLC / embedded) with **■ Stop** and **Go live** buttons (Go live for live channels).
- **Toasts:** transient messages at the bottom right, with a ✕ to dismiss. Used for errors such as "player not found", "Could not load details", "Episode lists need an Xtream source", the connection-limit warning, and refresh failures.

## 13. Keyboard shortcuts
| Keys | Action |
|---|---|
| Ctrl/Cmd+F | Focus search; Esc clears it |
| ↑ ↓ PgUp PgDn | Move the selection in the list |
| Enter / double-click | Play the selection |
| PgUp/PgDn (player focused) or Ctrl+↑/↓ | Previous/next channel |
| Backspace or L | Back to the previous channel |
| Ctrl+L | Go live |
| Space | Pause/play |
| F / double-click | Fullscreen; Esc exits |

## 14. Constraints for the designer (egui)
- **Immediate-mode toolkit, no CSS.** Global styling covers colors, corner rounding, spacing, stroke widths, fonts and sizes, and light/dark themes. Gradients, shadows and blur are limited. Animation is limited to fades and simple transitions.
- **Fonts:** any TTF/OTF can be bundled. There is no right-to-left shaping, so Arabic renders unjoined.
- **Big lists:** virtualized lists need a fixed row height. Poster grids are fine if each tile has a fixed size.
- **Images:** logos and posters load from URLs, are often missing or oddly sized, and need a placeholder.
- **Video:** the video is a texture, so anything can be drawn on top of it.
- **Platforms:** desktop with mouse and keyboard; a TV/remote mode may come later. Window size is about 1280×800 by default and can be resized down to about 900×600.
- **Language:** English only for now; labels should leave room for translation.
- **Not built yet, the design may reserve space:** an EPG "now/next" line under channel names, a TV-guide grid, catch-up, recording, and a parental PIN per profile.
