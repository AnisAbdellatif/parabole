# Pre-push checklist (agents: run this before every `git push` to `main` or a tag)

CI runs macOS, Linux and Windows. Anything you can fail locally, fail locally. Run top to bottom, fix, re-run from the failing step. Do not push on a red step.

## 1. Same commands as CI (`.github/workflows/ci.yml`)
Run all five; the first four run in seconds.
1. `cargo fmt --all --check` (fix with `cargo fmt`, then re-check; it also reorders `mod` lines)
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo clippy --all-targets --no-default-features -- -D warnings`
4. `cargo test`
5. `cargo test --no-default-features`

If `ci.yml` gained or changed a step since this list was written, run that step too.

## 2. Cross-platform risk (cannot be run on the dev Mac; review by reading)
- Every `#[cfg(target_os = ...)]` / `#[cfg(windows)]` block: does the other-platform build still compile (no unused imports/vars, no missing fallback fn)? Windows- and Linux-only clippy warnings have broken CI before.
- Tests touching files, paths or URLs must pass on Windows: no hardcoded `/tmp/...` or `/` separators in expectations (use `cfg(windows)` variants or `Path`), and drop every open DB/file handle before deleting its directory (Windows can't delete open files).
- New dependency: is it gated to the right target in `Cargo.toml`? Is `Cargo.lock` committed?
- New build-time tool or file (build.rs, icons, scripts): does the CI runner have it? Paths exist in git (`git ls-files <path>`), not only on disk?
- Shell scripts in `packaging/`: syntax-check with the shell in the shebang (`head -1`; `/bin/sh` is dash on Linux runners, so no arrays or `[[`): `dash -n <script>` or `bash -n <script>`; executable bit kept (`git ls-files -s packaging/*.sh` shows 100755).

## 3. Repo hygiene
- `git status` clean after the checks (no generated file left uncommitted, e.g. `Cargo.lock`).
- No secrets, credentials or Xtream URLs in the diff (`git diff origin/main | grep -iE "password|token|username="`).
- Workflow files changed? Run `actionlint` if installed; CI has an `actionlint` job.
- Version bump: `Cargo.toml` and `Cargo.lock` agree; packaging scripts and README don't hardcode the old version.

## 4. Release (only when pushing a `v*` tag)
- CI on the commit being tagged is green first. Never tag a red or unverified commit.
- The tag matches `Cargo.toml` version (`v0.1.1` ⇔ `version = "0.1.1"`).
- Packaging inputs exist: `packaging/icons/`, `packaging/icon.ico`, `packaging/entitlements.plist`.

## 5. After pushing
- `gh run list -L 3`, then `gh run watch <id> --exit-status` for the new run. Do not report "done" until every job is green.
- On failure: `gh run view <id> --log-failed`, fix the root cause, add the missed check to this list, push again.
