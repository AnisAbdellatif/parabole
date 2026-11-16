# Football tab — design

Status: implemented on branch `feat/football-tab` (2026-11-16). Research done 2026-11-16.

A tab listing upcoming and live matches from popular competitions, with the TV channels showing each match and their language. Channels are information only: they don't have to exist in the user's sources, the user looks them up.

## 1. Constraints

- Free data only (no paid API plans). Unofficial public JSON endpoints are acceptable; scraping is the last resort.
- These endpoints are undocumented and can change or block at any time. Design for breakage: one provider breaking must cost one small file fix, never a redesign, and must never break the rest of the tab or the app.
- Opt-in: off by default (it calls third-party sites the user didn't add).

## 2. Data sources (probed 2026-11-16)

| Provider | Gives | Notes |
|---|---|---|
| 365scores `webws.365scores.com/web/` | `games/allscores/?appTypeId=5&langId=1&timezoneName=UTC&userCountryId=<id>&sports=1&startDate=dd/mm/yyyy&endDate=dd/mm/yyyy` → matches, status, minute, score. `game/?…&userCountryId=<id>&gameId=<id>` → `game.tvNetworks` **for that one country**. `countries/` → country ids. | Good for DE, FR, ES, IT, PT, BR, MX, SA (beIN). Empty for GB, US, CA, IE, AU in the sample. |
| LiveScore `prod-public-api.livescore.com/v1/api/app/date/soccer/YYYYMMDD/0?MD=1` | One call per day: matches, status, score, and `Events[].Media["112"][]` items with `type == "TV_CHANNEL"`, `eventId` (channel name), `allowedCountries`. | Mostly GB, IE, US, CA. |

Dead or blocked: LiveSoccerTV, Sofascore, wheretowatch.football (403, Cloudflare), FotMob tvlistings (404), TheSportsDB free key (truncated), tvguideapi free key (yesterday only), ESPN (US only).

No source gives a channel's language. It is derived (§5).

## 3. Architecture

Follows the existing layers: providers know nothing about DB or UI; the worker does all network I/O; the UI only sends Commands.

```
src/sports/                      no DB, no UI
  mod.rs          shared HTTP client + get_text()
  lang.rs         languages → countries to query, country → language, channel-name hints
  merge.rs        pairs the same match across providers (kickoff ±15 min + both team names share a word)
  tests_live.rs   #[ignore] checks against the real endpoints
  s365/           mod.rs (URLs, fetch_day, fetch_tv), parse.rs (pure), ids.rs (competition allowlist, country ids)
  livescore/      mod.rs (URL, fetch_day), parse.rs (pure)
src/worker/football/             background thread: refresh stale cache, assemble the list (assemble.rs)
src/worker/football_settings.rs  FootballSettings { enabled, langs }, global setting key "football"
src/db/sports_cache.rs           cache table (migration_7.sql)
src/slint_ui/football/           tab glue (mod.rs) + pure text formatting (rows.rs)
ui-slint/football-view.slint, football-api.slint
```

The popular-competitions allowlist is the 365scores id table in `s365/ids.rs`; LiveScore is only used to add channels to those matches, so it needs no competition table.

`model.rs` gains the shared types:

```rust
pub struct Match {
    pub key: String,            // provider-prefixed id, e.g. "s365:4672273"
    pub competition: Competition,
    pub home: String,
    pub away: String,
    pub kickoff_utc: i64,
    pub status: MatchStatus,    // Upcoming | Live { minute: String, score: (u8, u8) } | Finished { score }
    pub broadcasts: Vec<Broadcast>,
}
pub struct Broadcast {
    pub channel: String,
    pub country: String,        // ISO 3166-1 alpha-2
    pub lang: Option<String>,   // ISO 639-1, None = unknown/multi (always inferred)
    pub provider: String,       // "365scores" | "LiveScore"
}
```

Worker: `Command::Football { langs }` (cached result at once, then stale parts refreshed on a background thread with its own SQLite connection), `Event::Football(FootballUpdate { matches, providers, done })` sent several times per load.

DB: `sports_cache(key, schema, fetched_at, json)` holding parsed results. TTLs: today's lists 60 s, other days 30 min, channels 6 h. The UI ticks every 60 s only while the tab is open and a match is live or about to start.

Request budget: channels fetched only for matches in the next 48 h and only for countries implied by the user's languages (§5). Throttle 1–2 req/s, sequential, with backoff on 429/403.

## 4. Modularity and maintenance

The rules that keep a breakage cheap to fix:

1. **One provider = one directory, nothing leaks out.** All URLs, headers, field names and provider ids live in `sports/<provider>/`. Outside that directory only the neutral types in `model.rs` exist. Fixing a provider never touches merge, cache, worker or UI.
2. **Fetch and parse are separate.** `parse.rs` is a pure `&str -> Result<Vec<…>>` function, so a format change is fixed against a saved response, offline, in a unit test.
3. **Same small surface for every provider.** Each exposes plain functions (no trait, no generics), e.g. `fetch_day(client, date) -> Result<Vec<Match>>` and, if it has channels, `fetch_channels(client, &Match, country) -> Result<Vec<Broadcast>>`. Adding a third provider later = new directory + one line in `sports/mod.rs`. Removing one = delete the directory + that line.
4. **Tolerant parsing.** serde structs with `Option`/`#[serde(default)]`, unknown fields ignored. A malformed item is skipped and counted, not a failed response. A whole response failing to parse → that provider reports an error, the others still render.
5. **Failure isolation and visible health.** `ProviderStatus { name, ok, last_success, error }` is returned with every result. The tab shows cached data plus a quiet footer line like "LiveScore unavailable since 3 h — channels may be incomplete". Never a modal, never a crash.
6. **Data tables, not logic.** Competition allowlist, provider id maps, language→countries and channel-name hints are plain `const` tables in their own files. Most "maintenance" (a league id changed, a new channel name pattern) is a one-line table edit.
7. **Cache is versioned.** `schema_version` per provider in the cache key; bump it when parsing changes so stale rows are ignored.
8. **Fixtures + live smoke test.**
   - `tests/fixtures/sports/<provider>/*.json`: real saved responses (credential-free), used by the parse tests.
   - `scripts/sports-fixtures.sh` re-downloads them, so updating fixtures after a format change is one command.
   - An `#[ignore]` live test (`cargo test sports_live -- --ignored`) hits each real endpoint and checks it still parses into non-empty data. Optional weekly CI job runs it so breakage is noticed before users report it.
9. **Runbook.** This doc keeps a short "when a provider breaks" section (§8) so the fix doesn't need re-research.

Shortcuts allowed by these rules get the project's `// shortcut:` comment (e.g. the team-name matcher).

## 5. Language

Derived (no source gives it); the UI says so in one footnote instead of marking each channel:

1. Channel-name hints first: `(Arabic)`, ` EN`, ` FR`, ` ES`, `Deportes`, `TUDN`, `Telemundo` → es, etc.
2. Else the country's main language (`FR→fr`, `BR→pt`, `SA→ar`). Multi-language countries (CH, BE, CA) → `None` ("multi") unless a hint matches.

The user's language choice also decides which countries are queried (en → GB, IE, US, CA, AU; fr → FR, BE, CH, CA; es → ES, MX, AR, US; ar → SA; de → DE, AT, CH; …). Default: English.

## 6. UI

- New rail button "Football" after Series (`shell-rail.slint`), shortcut `4`, `Core.tab == 3`. Rust state keeps `tab: MediaKind` for media and adds a separate "football screen open" flag, since football isn't a `MediaKind`.
- Views in `ui-slint/football*.slint`, glue in `src/slint_ui/football/`, reusing `stream-list` / `list-header` styling.
- Header (layout after Flashscore): team search; All / Live (n) / Upcoming filter; one day at a time with ‹ Today › (←/→ keys), up to 7 days ahead. Live and search ignore the day; search adds the day to section headers.
- List grouped by competition (header band per competition). Row on two lines: time or red live minute, home over away, live scores stacked, language chips (user's languages first, then `+N`).
- Details panel: channels grouped by language, country in brackets, `~` for derived language, a "Copy channel name" button, and "may be incomplete" note. Later: "Find in my channels" (switch to Live with the search pre-filled).
- Empty/error states: offline, all providers down, nothing scheduled. Always show cached data with "updated N min ago" when available.

## 7. Settings

- Settings dialog gets a "Football" tab with a `DCheck` **"Show Football tab"** (default off) and the language picker (§5), shown only when enabled.
- Stored app-wide (like player settings) in the `settings` table, key `football`, JSON `{enabled, langs}`.
- When off: rail button hidden, shortcut `4` ignored, the palette doesn't list it, and the worker makes **no** sports requests (no polling, no prefetch). Cache rows are kept, so re-enabling shows data immediately.
- Turning it off while the tab is open switches to Live.

## 8. When a provider breaks (runbook)

1. `cargo test sports_live -- --ignored` → shows which provider fails and whether it's HTTP (blocked/moved) or parse (format changed).
2. Format changed: `scripts/sports-fixtures.sh`, fix `sports/<provider>/parse.rs` until the parse tests pass, bump that provider's `schema_version`.
3. Endpoint moved/blocked: fix the URL/headers in `sports/<provider>/mod.rs`. If it's gone for good, delete the provider directory and its line in `sports/mod.rs`. The tab keeps working with the remaining provider.
4. League or country id changed: edit `sports/<provider>/ids.rs`.

## 9. Risks

- Undocumented endpoints: can change or block anytime; probably against the sites' terms. Fine for personal use, a real risk at scale, since every install calls them directly.
- Coverage gaps (e.g. NL, BE, PL, IN had no channels for a Nations League match). The UI says "may be incomplete".
- Store kick-off in UTC, display local. Rights change every season: never hard-code channels.

## 10. Open questions

- MLS / Saudi Pro League / Liga MX are not in the allowlist (one line each in `s365/ids.rs`: 104, 649, 141).
- Whether 365scores accepts several countries per call (would cut requests a lot). Untested.
