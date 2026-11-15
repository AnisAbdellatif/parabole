# VM end-to-end playback tests: spec

Goal: run the real Slint app with embedded mpv video and real input inside the UTM "Xubuntu" VM. Nothing reaches the host's display, focus or input.

## Environment (already provisioned)
- UTM VM "Xubuntu" (aarch64, Ubuntu 26.10, 4 vCPU, 3.4 GB RAM + 2 GB swap, 17 GB free).
- `ssh xubuntu@<ip>`: key auth works. The IP comes from the `/var/db/dhcpd_leases` entry `name=xubuntu-vm` (currently 192.168.64.2).
- No sudo: everything below runs as `xubuntu`. Do not try to sudo. If a package is missing, stop and report its name.
- The Linux build works: `cargo build --features slint-ui --bin iptv-player-slint -j3`, about 20 min cold, with an incremental target already in the guest. Build only that bin: linking two debug bins at once ran out of RAM.
- Installed: rustup stable (`. ~/.cargo/env`), libmpv-dev, mpv, ffmpeg, xvfb, xdotool, imagemagick, x11-utils, mesa (llvmpipe), python3, rsync, DejaVu fonts.
- The repo is copied (not cloned) to `~/iptv-player-slint` with rsync, excluding `target`, `.git` and `.tokenade`.
- Host control: `/Applications/UTM.app/Contents/MacOS/utmctl {start --hide|stop|status} Xubuntu`. `utmctl ip-address` does not work (no guest agent); use the lease file.

## Files
1. `scripts/vm/e2e.sh` (host, POSIX sh, under 120 lines).
   - Subcommands: `up | sync | build | test | fetch | all | down`.
   - `up`: start the VM hidden if it's stopped, then wait (max 120 s) for SSH on the leased IP.
   - `sync`: rsync with the excludes above.
   - `build`: `cargo build --features slint-ui --bin iptv-player-slint -j3`.
   - `test`: runs `scripts/vm/guest-e2e.sh` over SSH.
   - `fetch`: rsyncs `~/e2e-out/` back to host `/tmp/vm-e2e/`.
   - `all` = up, sync, build, test, fetch. Exit code = the test result.
   - `VM_HOST` env overrides IP discovery.
2. `scripts/vm/guest-e2e.sh` (guest, bash, under 200 lines). Runs from the repo dir.
   - **Display:**
     - start `Xvfb :99 -screen 0 1600x1000x24 +extension GLX +render -nolisten tcp`;
     - start a WM so fullscreen and resizes behave: `xfwm4 --display :99 --compositor=off`, or another installed WM;
     - set `DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1`;
     - kill everything on exit (`trap`).
   - **Media:** generate into `~/e2e-media/` once, then reuse, all `-movflags +faststart`, 30 s, 640x360, 25 fps, h264 + aac:
     - `ch1.mp4`: `testsrc2` (moving pattern);
     - `ch2.mp4`: solid blue `color=c=0x0000ff`;
     - `ch3.mp4`: solid green `color=c=0x00ff00`;
     - `ep1.mp4` and `ep2.mp4`: 6 s each, solid red and solid yellow.
     Serve with `python3 -m http.server 8765 --bind 127.0.0.1` from that dir.
   - **Data:**
     - `HOME=$(mktemp -d)`, so the run never uses the guest user's real config;
     - an M3U **file** source at `$HOME/e2e.m3u` with category "E2E Live" (ch1–ch3, `tvg-chno` 1–3) and category "E2E Movies" (one movie → ch1.mp4);
     - add the source through the app's normal startup path if the DB API is reachable, else via a tiny `--features slint-ui` example binary or `sqlite3`. Choose the simplest; `src/db` may be used from a Rust example under `examples/` only if needed;
     - Settings: embedded player on (check how `PlayerSettings` picks embedded vs external, and seed it).
   - **App:** run `target/debug/iptv-player-slint` in the background, logging to `~/e2e-out/app.log`. Find its window with `xdotool search --sync --pid`.
   - **Helpers** (bash functions):
     - `shot NAME`: `import -window root ~/e2e-out/NAME.png`;
     - `region_mean X Y W H`: prints mean R G B (0–255) with `convert … -crop … -resize 1x1 txt:-`;
     - `video_rect`: the video area in the docked pane or fullscreen. Find it once from a screenshot, by the bounding box of the non-UI pixels, or by fixed geometry derived from the window size. Document the choice.
     - `wait_for CMD TIMEOUT`: retry until success.
   - **Scenarios**, each printing `PASS name` or `FAIL name: reason`. Keep going after a failure and exit non-zero if any failed. Prefer keyboard input to coordinates: arrows/Enter in the list; `F`, `Space`, PgUp/PgDn, Esc in the player (see `src/slint_ui/player/keys.rs`). Use `xdotool key --window` / `windowfocus` on the app window inside :99 only.
     1. `play-live`: select ch1 and press Enter. Within 20 s the video rect is non-black (mean > 30), and two shots 1 s apart differ (`compare -metric AE`).
     2. `zap-down`: PgDn (or Cmd/Ctrl+Down as the code expects). Within 10 s the video rect mean is blue-dominant (B > 150, R and G < 80). Also take a shot of the zap card within 1 s.
     3. `zap-up`: PgUp back to ch1, so the video is no longer blue.
     4. `pause`: Space; two shots 1 s apart are identical in the video rect. Space again resumes.
     5. `fullscreen`: F; the app window geometry (`xdotool getwindowgeometry`) is the screen size, and the video rect is scaled up and non-black. Esc or F exits. Take a shot with the zap carousel open in fullscreen (PgDn), so the neighbour cards are visible.
     6. `mini`: the mini/PiP button or its key; the window is at most 500x300 and the video is non-black. Restore it.
     7. `vod-seek`: open the movie and play it; the seek bar is visible (shot). Press Right (seek +10 s or whatever the key does); the frame changes. ch1 has a moving pattern, so that's enough.
     8. `stop`: stop; the video pane closes or turns black, and the app doesn't crash (process alive, no panic in `app.log`).
   - At the end: grep `app.log` for `panicked` and `ERROR`, list the shots, then print the `PASS`/`FAIL` summary.
3. `docs/slint-experiment.md`: add a "VM end-to-end tests" section covering how to run them (`scripts/vm/e2e.sh all`), what they cover, their limits (Linux, software GL, no macOS PiP/Spaces) and run time.

## Rules
- Never run anything on the host display: no osascript, CGEvent or screencapture, and no app launch on the host. All UI runs inside the guest on `:99`.
- Never log or print stream URLs beyond the local 127.0.0.1 test ones. Don't touch the host user DB.
- Shell scripts: `set -eu` (bash: `set -euo pipefail`), quote variables, no `eval`.
- When a scenario fails, investigate the root cause in the app before relaxing a threshold. A real app bug found here gets fixed, as a separate commit, with the root cause in the message. A Linux-only build fix gets the smallest `cfg` change.
