# Sourced by guest-e2e.sh when E2E_REAL=1 (uses its helpers). Real content is unknown, so the checks are only
# "not black" and "changes over time". One stream at a time, ~3 min of playback in total (provider connection limit).
lit() { read -r r g b <<<"$(probe cur)"; [ $(((r + g + b) / 3)) -gt 10 ]; }
moving() { # three tries, 1.5 s apart: a still scene may not change in one
    local i
    for i in 1 2 3; do
        probe mv-a >/dev/null; sleep 1.5; probe mv-b >/dev/null; crop_probe mv-a pa; crop_probe mv-b pb
        [ "$(same pa pb)" -gt 0 ] && return 0
    done
    return 1
}
pick() { # name substring or empty: search it (empty clears the search), then play the first row
    key ctrl+f; sleep 0.3; key ctrl+a
    if [ -n "$1" ]; then xdotool type --delay 40 -- "$1"; key Return; sleep 1.5; else key Escape; sleep 0.5; fi
    key Home; key Return
}
click_stop() { read -r vx vy vw vh _ <<<"$(video_rect)"; hover_video; sleep 0.5; xdotool mousemove $((vx + 68)) $((vy + vh - 26)); sleep 0.3; xdotool click 1; sleep 2; }
settled() { lit && moving; }

r_play_live() {
    xdotool mousemove 5 5; pick "${E2E_REAL_LIVE:-}"
    wait_for 60 lit || bad "live video black after 60 s" || return 1
    moving || bad "live frames never change" || return 1
    shot real-play-live
}
r_zap_down() {
    # moving content changes anyway, so a frame diff can't tell the channels apart: give the provider time to connect
    # (it takes 2-4 s) and require a lit, moving picture afterwards, so later scenarios don't race the loading stream
    hover_video; key Next; sleep 0.4; shot real-zap-card; sleep 6
    wait_for 40 settled || bad "no lit, moving picture 40 s after PgDn" || return 1
    shot real-zap-down
}
r_pause() {
    hover_video; key space; sleep 1.5; probe pause-a >/dev/null; sleep 1; probe pause-b >/dev/null
    crop_probe pause-a pa; crop_probe pause-b pb
    [ "$(same pa pb)" -eq 0 ] || bad "video moves while paused" || return 1
    key space; moving || bad "video did not resume" || return 1
}
r_fullscreen() {
    hover_video; MODE=full; key f; wait_for 10 win_size_is eq "$SW" "$SH" || bad "window is ${AW}x${AH}, not ${SW}x${SH}" || return 1
    wait_for 20 lit || bad "fullscreen video black" || return 1
    moving || bad "fullscreen frames never change" || return 1
    shot real-fullscreen
    key Escape; MODE=docked; wait_for 10 not_fullscreen || bad "Esc did not leave fullscreen" || return 1
}
r_vod_seek() {
    click_stop; key ctrl+2; sleep 1; pick "${E2E_REAL_VOD:-}"
    wait_for 60 lit || bad "movie video black after 60 s" || return 1
    hover_video; key space; sleep 1; probe vod-a >/dev/null; key Right; key Right; key Right; sleep 2; probe vod-b >/dev/null
    # movies often open on a still title card, so compare the control strip (time text, seek bar) instead of the picture
    read -r vx vy vw vh _ <<<"$(video_rect)"
    for n in a b; do $IM "$OUT/vod-$n.png" -crop "${vw}x50+${vx}+$((vy + vh - 60))" +repage "$OUT/ctl-$n.png"; done
    [ "$(same ctl-a ctl-b)" -gt 0 ] || bad "seek bar and time unchanged after seeking 30 s" || return 1
    shot real-vod-seek
}
r_stop() {
    hover_video; shot before-stop; crop_probe before-stop pa; click_stop; shot after-stop; crop_probe after-stop pb
    [ "$(same pa pb)" -gt 0 ] || bad "video pane unchanged after clicking Stop" || return 1
    kill -0 "$APP" 2>/dev/null || bad "app died" || return 1
    ! grep -q panicked "$OUT/app.log" || bad "panic in app.log" || return 1
}
run_real() {
    local t0=$SECONDS
    run play-live r_play_live; run zap-down r_zap_down; run pause r_pause; run fullscreen r_fullscreen
    run vod-seek r_vod_seek; run stop r_stop
    echo "real playback total: $((SECONDS - t0)) s"
}
