#!/usr/bin/env bash
# Static checks for the release scripts, which CI only runs on a tag. Run locally and in CI.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
fail=0
bad() { echo "FAIL: $*"; fail=1; }

for f in packaging/*.sh scripts/*.sh; do
  shell=$(head -1 "$f" | sed -E 's|^#! ?(/usr/bin/env )?||; s|^.*/||')
  case $shell in
    sh) if command -v dash >/dev/null; then dash -n "$f" || bad "$f: not valid POSIX sh (dash)"; else echo "skip: dash missing, $f"; fi ;;
    bash) bash -n "$f" || bad "$f: bash syntax" ;;
    *) bad "$f: unknown shebang shell '$shell'" ;;
  esac
  [ "$(git ls-files -s "$f" | cut -c1-6)" = 100755 ] || bad "$f: not executable in git (git update-index --chmod=+x)"
  if command -v shellcheck >/dev/null; then shellcheck -S warning "$f" || bad "$f: shellcheck"; else echo "skip: shellcheck missing"; fi
done

# every packaging/ path the scripts and workflows mention must exist in git
grep -rhoE 'packaging/[A-Za-z0-9_./-]+' packaging/*.sh packaging/flatpak .github/workflows 2>/dev/null | sort -u | while read -r p; do
  p=${p%.}
  case $p in *'*'*|*/) continue ;; esac
  [ -e "$p" ] || { echo "FAIL: referenced but missing: $p"; exit 1; }
done || fail=1
for g in packaging/icons/linux packaging/icons/macos.iconset; do [ -n "$(ls "$g" 2>/dev/null)" ] || bad "$g is empty"; done

if command -v pwsh >/dev/null; then
  for f in packaging/*.ps1; do
    pwsh -NoProfile -Command "\$e=\$null; [void][System.Management.Automation.Language.Parser]::ParseFile('$f',[ref]\$null,[ref]\$e); if(\$e){\$e; exit 1}" || bad "$f: PowerShell parse"
  done
else echo "skip: pwsh missing"; fi

[ $fail = 0 ] && echo "packaging checks ok"
exit $fail
