#!/usr/bin/env bash
# Everything CI would reject, run locally before `git push`. Pass a tag (v0.1.2) when about to tag a release.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
tag=${1:-}
fail=0
step() { echo "== $*"; }
bad() { echo "FAIL: $*"; fail=1; }

step "cargo (same commands as ci.yml)"
cargo fmt --all --check || bad "cargo fmt (run: cargo fmt)"
cargo clippy --all-targets -- -D warnings || bad "clippy"
cargo clippy --all-targets --no-default-features -- -D warnings || bad "clippy --no-default-features"
cargo test || bad "tests"
cargo test --no-default-features || bad "tests --no-default-features"

step "release scripts"
./scripts/check-packaging.sh || bad "packaging checks"
if command -v actionlint >/dev/null; then actionlint || bad "actionlint"; else echo "skip: actionlint missing"; fi

step "version and lockfile"
ver=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
lock=$(awk '/^name = "parabole"$/ {getline; gsub(/[^0-9.]/, ""); print; exit}' Cargo.lock)
[ "$ver" = "$lock" ] || bad "Cargo.toml $ver != Cargo.lock $lock"
[ -z "$(git status --porcelain)" ] || { bad "working tree not clean (commit or discard)"; git status --short; }

step "secrets in the diff vs origin/main"
git fetch -q origin main 2>/dev/null
if git diff origin/main -U0 -- . ':!docs' ':!scripts/preflight.sh' | grep -E '^\+' | grep -iE 'password=|passwd=|token=[A-Za-z0-9]|api[_-]?key=|secret[_-]?key|gh[pousr]_[A-Za-z0-9]{20,}|BEGIN [A-Z ]*PRIVATE KEY|username=[^&]*&password'; then bad "possible secret in diff (review the lines above)"; fi

if [ -n "$tag" ]; then
  step "tag $tag"
  [ "$tag" = "v$ver" ] || bad "tag $tag != Cargo.toml version v$ver"
  git log -1 --format=%B | grep -qiE '\[(skip ci|ci skip|no ci|skip actions)\]' && bad "HEAD message has [skip ci]: GitHub would skip the release run too"
  git ls-remote --exit-code --tags origin "$tag" >/dev/null 2>&1 && bad "tag $tag already exists on origin (never move a tag whose release was published)"
  sha=$(git rev-parse HEAD)
  [ "$(gh run list --workflow ci --commit "$sha" --json conclusion -q '.[0].conclusion' 2>/dev/null)" = success ] || bad "no green CI run on HEAD $sha (push, wait, then tag)"
fi

[ $fail = 0 ] && echo "preflight ok: push, then watch CI to green" || echo "preflight FAILED"
exit $fail
