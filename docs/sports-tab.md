# Sports tab (supersedes the UI/worker parts of `football-tab.md`)

The Football tab becomes a **Sports** tab. Each sport is a small, self-contained module; adding one is one file plus one line in a registry.

## 1. What the user sees
- Rail button "Sports" (same slot, `Ctrl/⌘+4`).
- Top row: sport switcher (`Segmented`: Football · Basketball · Tennis · F1 · WWE …), only the sports enabled in Settings, in registry order.
- Below: the current Football layout, unchanged and shared by every sport: search, All / Live / Upcoming, day picker, competition bands, two-line rows, channels panel.
- Single-event sports (a race, a wrestling show, a fight card) show one line: the event name (no score column).
- Filter, day, search and selection are remembered per sport for the session.

## 2. Data sources (all free)
| Sport | Schedule + live | TV channels | Notes |
|---|---|---|---|
| Football | 365scores sport 1, LiveScore soccer | 365scores per country, LiveScore | as today |
| Basketball (NBA, EuroLeague) | 365scores sport 2 | 365scores | verified: NBA game → US channels |
| Ice hockey (NHL) | 365scores sport 4 | 365scores | |
| American football (NFL) | 365scores sport 6 | 365scores | |
| Tennis (ATP/WTA/Slams) | 365scores sport 3 | 365scores | tournaments change weekly: filter by tour category, not competition id (to verify) |
| F1 / MotoGP | TheSportsDB `eventsday.php?s=Motorsport`, leagues 4370 / 4407 | TheSportsDB `lookuptv.php` | free TV data is sparse (often 0–2 channels) |
| WWE / UFC | TheSportsDB `eventsday.php?s=Fighting`, leagues 4444 / 4443 | TheSportsDB `lookuptv.php` | same; free key `123`, ~30 req/min |

The worker loads **only the selected sport**, so extra sports cost nothing until opened.

## 3. Modularity
```
src/sports/
  mod.rs            client, get_text (unchanged)
  lang.rs merge.rs  unchanged
  registry.rs       Sport struct + SPORTS table (the only list of sports)
  catalog/          one file per sport: football.rs, basketball.rs, tennis.rs, f1.rs, wwe.rs …
  s365/             provider, now parameterised by 365scores sport id + competition filter
  livescore/        provider, parameterised by its sport path ("soccer", "basketball", …)
  sportsdb/         new provider: TheSportsDB day list + TV lookup
```
```rust
// registry.rs
pub type FetchDay = fn(&Client, NaiveDate) -> Result<Vec<Match>>;
pub type FetchTv = fn(&Client, &Match, &str /*country*/) -> Result<Vec<Broadcast>>;
pub struct DaySource { pub provider: &'static str, pub schema: i64, pub fetch: FetchDay }
pub struct Sport {
    pub id: &'static str,          // "basketball": settings + cache-key prefix
    pub name: &'static str,        // "Basketball"
    pub days: &'static [DaySource],// first is the base list, the rest are merged in (merge::attach)
    pub tv: Option<FetchTv>,       // per-country channels, fetched for events within 48 h
}
pub const SPORTS: &[&Sport] = &[&football::SPORT, &basketball::SPORT, …];
```
A sport file holds its allowlist and thin wrappers, e.g. `fn day(c, d) { s365::fetch_day(c, 2, COMPETITIONS, d) }`.
A broken provider only breaks the sports that list it; its status shows in the header as today.

## 4. Model changes (`src/model.rs`)
- `Match` gains `sport: String`.
- Single events: `away` is empty and `home` holds the event name (documented on the field).
- `MatchStatus::Live { clock, home, away }`: scores become `String` (tennis "6-4 3-2", basketball "98").
- Cache keys gain the sport prefix: `basketball:365scores:day:2026-11-16`; TV keys stay per provider game id. Bump schemas.

## 5. Worker, settings, UI
- `Command::Sports { sport, langs }` → `Event::Sports(SportsUpdate { sport, … })`; one load at a time, the UI re-requests on `done` if the user switched sport meanwhile.
- Setting key `sports`: `{ enabled, langs, sports: ["football", …] }`; read the old `football` key once if `sports` is missing.
- Settings → "Sports" page: show tab, language checkboxes (as today), plus a checkbox per sport.
- Rename `football` → `sports` in `src/slint_ui/`, `src/worker/`, `ui-slint/` (`SportsApi`, `SportsView`, `Core.sports-enabled`).

## 6. Steps (each compiles and passes tests on its own)
1. Model + registry + parameterised s365/LiveScore; football moves to `catalog/football.rs`. No UI change.
2. Worker + settings generalised (`Command::Sports`, migration of the old key).
3. UI rename + sport switcher + single-event rows; snapshot updated.
4. Add 365scores sports (basketball, hockey, NFL), then tennis (category filter).
5. Add `sportsdb` provider, then F1 and WWE (and UFC).
