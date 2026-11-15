# Pre-push checklist (agents: go through this before every `git push` to `main` or a tag)

CI runs macOS, Linux and Windows. Anything you can fail locally, fail locally. Do not push on a red step.

## 1. Run the script
`./scripts/preflight.sh` (add the tag, e.g. `./scripts/preflight.sh v0.1.2`, when about to tag). Commit first: it requires a clean tree. It runs, in order:
- the five CI cargo commands (fmt, clippy and test, each with default features and `--no-default-features`);
- `scripts/check-packaging.sh` (release scripts: syntax under their shebang shell, shellcheck, executable bits, referenced files exist) and `actionlint`;
- `Cargo.toml` vs `Cargo.lock` version, clean working tree, secrets grep on the diff;
- with a tag: tag == `Cargo.toml` version, HEAD message has no `[skip ci]`, tag not already on origin, green CI on HEAD.

If `ci.yml` gained a step the script doesn't run, add it to the script, not just here.

## 2. What the script cannot check: read the diff for these
Windows and Linux can't be built from the dev Mac (C dependencies such as aws-lc fail to cross-compile), so CI is the first real test of anything platform-specific.
- Every `#[cfg(target_os = ...)]` / `#[cfg(windows)]` block: does the other-platform build still compile (no unused imports/vars, no missing fallback fn)? Windows- and Linux-only clippy warnings have broken CI before.
- Tests touching files, paths or URLs must pass on Windows: no hardcoded `/tmp/...` or `/` separators in expectations (use `cfg(windows)` variants or `Path`), and drop every open DB/file handle before deleting its directory.
- New dependency: gated to the right target in `Cargo.toml`?
- New build-time tool or file (build.rs, icons, scripts): does the CI runner have it, and is it in git (`git ls-files <path>`), not only on disk?
- Release scripts and `release.yml` only run on a tag: `check-packaging.sh` is static only. A new tool installed by a script, a new runner dependency, or a changed `linuxdeploy`/`iconutil` argument needs a read-through against `release.yml`.
- `/bin/sh` is dash on Linux runners: no arrays, `[[`, or `local` surprises in `#!/bin/sh` scripts.

## 3. Pushing and tagging rules
- Never put `[skip ci]` on a commit that will be tagged: it also skips the release run. `ci.yml` already ignores docs-only pushes via `paths-ignore`.
- Tag only a commit whose CI run is green (the script checks). A tag whose release run failed before publishing may be moved (delete the remote tag, retag a fixed commit). Never move a tag once its release is published.
- `TAP_TOKEN` (secret) lets the `tap` job in `release.yml` bump `Casks/parabole.rb` in AnisAbdellatif/homebrew-tap; it needs contents:write there.
- Changing `Cargo.toml` profiles or `Cargo.lock` invalidates the cargo cache: the first CI run after is slow (~2x). That is not a regression; compare the second run.

## 4. After pushing
- Wait for the run to finish, then read the result: `until [ "$(gh run view <id> --json status -q .status)" = completed ]; do sleep 15; done; gh run view <id> --json conclusion,jobs -q '.conclusion,(.jobs[]|[.name,.conclusion]|join(" "))'`. Don't pipe `gh run watch` into `tail`: that hides its exit code.
- Do not report "done" until every job is green. After a release run, also confirm the release has its four assets and the tap got a `parabole <version>` commit.
- On failure: `gh run view <id> --log-failed`, fix the root cause, then add the missed check to `scripts/preflight.sh` (preferred) or this list, and push again.
