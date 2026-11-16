#!/usr/bin/env bash
# Refreshes tests/fixtures/sports/ from the live provider endpoints.
# Usage: scripts/sports-fixtures.sh [YYYY-MM-DD] [S365_GAME_ID] [S365_COUNTRY_ID]
set -euo pipefail

UA="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0 Safari/537.36"
date_iso="${1:-$(date -u +%F)}"
game_id="${2:-}"
country_id="${3:-4}"

case "$date_iso" in
  [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]) ;;
  *) echo "bad date: $date_iso" >&2; exit 2 ;;
esac
case "$country_id" in *[!0-9]*|"") echo "bad country id" >&2; exit 2 ;; esac
case "$game_id" in *[!0-9]*) echo "bad game id" >&2; exit 2 ;; esac

y="${date_iso%%-*}"
rest="${date_iso#*-}"
m="${rest%%-*}"
d="${rest#*-}"

out="$(cd "$(dirname "$0")/.." && pwd)/tests/fixtures/sports"
mkdir -p "$out"
fetch() { curl -fsSL -A "$UA" -o "$2" "$1"; }

s365="https://webws.365scores.com/web/"
common="appTypeId=5&langId=1&timezoneName=UTC"
fetch "${s365}games/allscores/?${common}&userCountryId=1&sports=1&startDate=$d/$m/$y&endDate=$d/$m/$y" "$out/s365_day.json"

if [ -z "$game_id" ]; then
  game_id="$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["games"][0]["id"])' "$out/s365_day.json")"
fi
fetch "${s365}game/?${common}&userCountryId=$country_id&gameId=$game_id" "$out/s365_tv.json"
fetch "https://prod-public-api.livescore.com/v1/api/app/date/soccer/$y$m$d/0?MD=1" "$out/livescore_day.json"

sdb="https://www.thesportsdb.com/api/v1/json/123/"
fetch "${sdb}eventsday.php?d=2026-11-22&s=Motorsport" "$out/sportsdb_day.json"
sleep 3
fetch "${sdb}lookuptv.php?id=2408216" "$out/sportsdb_tv.json"

for f in s365_day s365_tv livescore_day sportsdb_day sportsdb_tv; do
  printf '%s.json %s bytes\n' "$f" "$(wc -c < "$out/$f.json" | tr -d ' ')"
done
