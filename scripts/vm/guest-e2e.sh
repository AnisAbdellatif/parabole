#!/bin/bash
# Guest side of the VM e2e tests: real Slint app + embedded libmpv on Xvfb :99. Run from the repo dir.
set -euo pipefail
REPO=$PWD
MEDIA=$HOME/e2e-media-v3
OUT=$HOME/e2e-out
PORT=8765
# E2E_DISPLAY=:99 (default): private Xvfb + xfwm4. E2E_DISPLAY=:0: the guest's own desktop, watchable in the UTM window.
DISP=${E2E_DISPLAY:-:99}
export DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1
IM=convert; command -v magick >/dev/null && IM=magick
REAL=${E2E_REAL:-} # 1 = play the guest's own source (~/.config/iptv-e2e/source.url) instead of the local test media
rm -rf "$OUT"; mkdir -p "$OUT" "$MEDIA"
PIDS=()
HLS=
SCRATCH=
cleanup() { # always runs; in real mode the redaction runs before anything can fetch $OUT
    kill "${PIDS[@]}" 2>/dev/null || true
    [ -z "$HLS" ] || rm -rf -- "$HLS"
    [ -z "$SCRATCH" ] || rm -rf -- "$SCRATCH"
    if [ -n "$REAL" ]; then printf '%s\n%s\n%s\n' "$RU" "$RP" "$RHOST" | python3 -I "$REPO/scripts/vm/redact.py" "$OUT" || exit 1; fi
}
trap cleanup EXIT
RU= RP= RHOST=
if [ -n "$REAL" ]; then # credentials live only in these shell variables (builtins only: never in argv or env of a child)
    set +x
    URLFILE=$HOME/.config/iptv-e2e/source.url
    [ -r "$URLFILE" ] || { echo "FAIL setup: $URLFILE missing"; exit 1; }
    u=$(<"$URLFILE"); u=${u%%[[:space:]]*}
    RHOST=${u#*://}; RHOST=${RHOST%%/*}
    IFS='&' read -ra kv <<<"${u#*\?}"
    for kvp in "${kv[@]}"; do case $kvp in username=*) RU=${kvp#username=} ;; password=*) RP=${kvp#password=} ;; esac; done
    unset u kv kvp
    [ -n "$RU" ] && [ -n "$RP" ] && [ -n "$RHOST" ] || { echo "FAIL setup: url file has no username/password"; exit 1; }
fi

# --- display (only what is started here goes into PIDS; the :0 desktop and its xfwm4 are never touched) ----
if [ "$DISP" = :99 ]; then
    Xvfb :99 -screen 0 1600x1000x24 +extension GLX +render -nolisten tcp >"$OUT/xvfb.log" 2>&1 & PIDS+=($!)
    for _ in $(seq 40); do xdpyinfo >/dev/null 2>&1 && break; sleep 0.25; done
    xfwm4 --display :99 --compositor=off >"$OUT/wm.log" 2>&1 & PIDS+=($!)
else
    UID_=$(id -u); found=
    for xa in "${XAUTHORITY:-}" "$HOME/.Xauthority" "/run/user/$UID_/gdm/Xauthority" "/run/user/$UID_/.mutter-Xwaylandauth."* \
        "/run/user/$UID_/xauth_"* "/run/lightdm/$USER/xauthority" ""; do
        [ -z "$xa" ] || [ -r "$xa" ] || continue
        if XAUTHORITY=$xa xdpyinfo -display "$DISP" >/dev/null 2>&1; then found=1; [ -z "$xa" ] || export XAUTHORITY=$xa; break; fi
    done
    [ -n "$found" ] || { echo "FAIL setup: cannot open $DISP (no usable Xauthority; is xubuntu logged in on seat0?)" >&2; exit 1; }
fi
read -r SW SH <<<"$(xdpyinfo | awk '/dimensions:/{split($2,a,"x"); print a[1], a[2]}')"
echo "display $DISP ${SW}x${SH}"

# --- media. Live = HLS from one `ffmpeg -re` per channel (like real IPTV, no seeking), served by plain http.server;
# VOD (movie, episodes) = cached faststart mp4 served with Range support by range_server.py ---
S=size=640x360:rate=25
FS=(-movflags +faststart)
gen() { # file lavfi-source seconds
    [ -s "$MEDIA/$1" ] || ffmpeg -loglevel error -y -f lavfi -i "$2" -f lavfi -i "sine=frequency=440:duration=$3" \
        -t "$3" -r 25 -c:v libx264 -pix_fmt yuv420p -c:a aac "${FS[@]}" -f mp4 "$MEDIA/$1"
}
if [ -z "$REAL" ]; then
gen ch1.mp4 "testsrc2=$S" 120; gen ep1.mp4 "color=c=0xff0000:$S" 6; gen ep2.mp4 "color=c=0xffff00:$S" 6
python3 "$REPO/scripts/vm/range_server.py" "$MEDIA" $PORT >"$OUT/http.log" 2>&1 & PIDS+=($!)
HLS=$(mktemp -d)
H=size=320x180:rate=25
n=0
for src in "testsrc2=$H" "color=c=0x0000ff:$H" "color=c=0x00ff00:$H"; do
    n=$((n + 1)); mkdir "$HLS/ch$n"
    ffmpeg -loglevel error -re -f lavfi -i "$src" -re -f lavfi -i "sine=frequency=440" -c:v libx264 -preset ultrafast \
        -tune zerolatency -g 25 -pix_fmt yuv420p -c:a aac -f hls -hls_time 1 -hls_list_size 6 \
        -hls_flags delete_segments+omit_endlist "$HLS/ch$n/index.m3u8" >"$OUT/ffmpeg-ch$n.log" 2>&1 & PIDS+=($!)
done
python3 -m http.server $((PORT + 1)) --bind 127.0.0.1 --directory "$HLS" >"$OUT/hls-http.log" 2>&1 & PIDS+=($!)
for n in 1 2 3; do
    for _ in $(seq 60); do [ "$(grep -c '\.ts' "$HLS/ch$n/index.m3u8" 2>/dev/null || true)" -ge 3 ] && continue 2; sleep 0.5; done
    echo "FAIL setup: ch$n HLS playlist never reached 3 segments"; exit 1
done
fi

# --- data: scratch HOME, M3U file source, embedded player on ---------------
unset XDG_DATA_HOME XDG_CACHE_HOME XDG_CONFIG_HOME
export HOME; HOME=$(mktemp -d); SCRATCH=$HOME
if [ -n "$REAL" ]; then
    "$REPO/target/debug/examples/vm_seed" --real "${URLFILE}"
else
{
    echo '#EXTM3U'
    for n in 1 2 3; do
        echo "#EXTINF:-1 tvg-id=\"e2e$n\" tvg-chno=\"$n\" group-title=\"E2E Live\",E2E Channel $n"
        echo "http://127.0.0.1:$((PORT + 1))/ch$n/index.m3u8"
    done
    echo "#EXTINF:-1 group-title=\"E2E Movies\",E2E Movie"
    echo "http://127.0.0.1:$PORT/ch1.mp4"
} >"$HOME/e2e.m3u"
"$REPO/target/debug/examples/vm_seed" "$HOME/e2e.m3u" "$OUT/mpv.log"
fi

# --- app ----------------------------------------------------------------
"$REPO/target/debug/iptv-player" >"$OUT/app.log" 2>&1 & APP=$!; PIDS+=($APP)
WIN=
for _ in $(seq 60); do
    WIN=$(xdotool search --onlyvisible --pid "$APP" 2>/dev/null | head -1 || true)
    [ -n "$WIN" ] && break; sleep 0.5
done
[ -n "$WIN" ] || { echo "FAIL setup: app window never appeared"; [ -n "$REAL" ] || tail -20 "$OUT/app.log"; exit 1; }
xdotool windowfocus "$WIN" || true
sleep 3
if [ -n "$REAL" ]; then # the app's startup refresh fetches the new source; vm_seed prints only a fixed status word
    echo "real source: $("$REPO/target/debug/examples/vm_seed" --wait 300 || true)"
    sleep 5
fi

# --- helpers --------------------------------------------------------------
shot() { import -display "$DISP" -window root "$OUT/$1.png"; }
region_mean() { # file X Y W H -> "R G B" (0-255)
    $IM "$1" -crop "${4}x${5}+$2+$3" +repage -resize '1x1!' \
        -format '%[fx:round(255*r)] %[fx:round(255*g)] %[fx:round(255*b)]' info:
}
geom() { # sets AX AY AW AH: the app window's client area on the root window
    read -r AX AY AW AH <<<"$(xwininfo -id "$WIN" | awk '/Absolute upper-left X/{x=$NF} /Absolute upper-left Y/{y=$NF} /Width:/{w=$NF} /Height:/{h=$NF} END{print x, y, w, h}')"
}
# video_rect: docked => fixed layout from app.slint/player-pane.slint (pane 420 px at the window's right edge,
# 16 px padding, 16:9 video at the top); fullscreen and mini => the whole window. The returned probe box is the
# middle band of that rect, clear of the top chrome and the bottom control scrim.
MODE=docked # docked | full | mini, set by the scenarios (a window that is as wide as the screen is not necessarily fullscreen)
video_rect() { # echoes "VX VY VW VH PX PY PW PH"
    geom
    if [ "$MODE" != docked ]; then VX=$AX VY=$AY VW=$AW VH=$AH
    else VX=$((AX + AW - 420 + 16)) VY=$((AY + 16)) VW=388 VH=218; fi
    echo "$VX $VY $VW $VH $((VX + VW / 5)) $((VY + VH * 3 / 10)) $((VW * 3 / 5)) $((VH / 4))"
}
probe() { # NAME -> "R G B" of the probe box in a fresh shot
    read -r _ _ _ _ px py pw ph <<<"$(video_rect)"; shot "$1"; region_mean "$OUT/$1.png" "$px" "$py" "$pw" "$ph"
}
crop_probe() { read -r _ _ _ _ px py pw ph <<<"$(video_rect)"; $IM "$OUT/$1.png" -crop "${pw}x${ph}+$px+$py" +repage "$OUT/$2.png"; }
same() { compare -metric AE -fuzz 3% "$OUT/$1.png" "$OUT/$2.png" null: 2>&1 | grep -o '^[0-9]*'; }
wait_for() { local t=$1; shift; local end=$((SECONDS + t)); until "$@"; do [ $SECONDS -lt $end ] || return 1; sleep 0.5; done; }
key() { xdotool key --clearmodifiers "$@"; }
hover_video() { read -r vx vy vw vh _ <<<"$(video_rect)"; xdotool mousemove $((vx + vw / 2 - 20)) $((vy + vh / 2)); sleep 0.2
    xdotool mousemove $((vx + vw / 2)) $((vy + vh / 2)); sleep 0.3; }
is_bright() { read -r r g b <<<"$(probe cur)"; [ $(((r + g + b) / 3)) -gt 30 ]; }
is_blue() { read -r r g b <<<"$(probe cur)"; [ "$b" -gt 150 ] && [ "$r" -lt 80 ] && [ "$g" -lt 80 ]; }
not_blue() { read -r r g b <<<"$(probe cur)"; ! { [ "$b" -gt 150 ] && [ "$r" -lt 80 ] && [ "$g" -lt 80 ]; } && [ $(((r + g + b) / 3)) -gt 30 ]; }
not_fullscreen() { geom; [ "$AW" -lt "$SW" ] || [ "$AH" -lt "$SH" ]; }
win_size_is() { geom; [ "$AW" -$1 "$2" ] && [ "$AH" -$1 "$3" ]; }
bad() { WHY=$1; return 1; }

# --- scenarios --------------------------------------------------------------
s_play_live() {
    xdotool mousemove 5 5; key Home; key Return; sleep 1
    wait_for 20 is_bright || bad "video rect still black after 20 s: $(probe cur)" || return 1
    hover_video; probe live-a >/dev/null; sleep 1; probe live-b >/dev/null
    crop_probe live-a pa; crop_probe live-b pb
    [ "$(same pa pb)" -gt 0 ] || bad "frames 1 s apart are identical (not moving)" || return 1
    shot play-live
}
s_zap_down() {
    hover_video; key Next; sleep 0.4; shot zap-card
    wait_for 10 is_blue || bad "never blue after PgDn: $(probe cur)" || return 1
    shot zap-down
}
s_zap_up() {
    hover_video; key Prior
    wait_for 10 not_blue || bad "still blue/black after PgUp: $(probe cur)" || return 1
    shot zap-up
}
s_pause() {
    hover_video; key space; sleep 1.5; probe pause-a >/dev/null; sleep 1; probe pause-b >/dev/null
    crop_probe pause-a pa; crop_probe pause-b pb
    [ "$(same pa pb)" -eq 0 ] || bad "video moves while paused" || return 1
    is_bright || bad "paused frame is black" || return 1
    key space; sleep 1; probe res-a >/dev/null; sleep 1; probe res-b >/dev/null
    crop_probe res-a pa; crop_probe res-b pb
    [ "$(same pa pb)" -gt 0 ] || bad "video did not resume" || return 1
}
s_fullscreen() {
    hover_video; MODE=full; key f; wait_for 10 win_size_is eq "$SW" "$SH" || bad "window is ${AW}x${AH}, not ${SW}x${SH}" || return 1
    for i in 1 2 3 4; do sleep 1.5; shot "full-t$i"; done
    wait_for 10 is_bright || bad "fullscreen video black" || return 1
    read -r _ _ _ _ <<<"$(probe full-main)"
    local bb; bb=$($IM "$OUT/full-main.png" -fuzz 8% -trim -format '%w %h' info:)
    read -r w h <<<"$bb"; [ "$w" -ge $((SW * 9 / 10)) ] && [ "$h" -ge $((SH * 8 / 10)) ] || bad "video not scaled up (non-black box ${w}x${h})" || return 1
    key Next; sleep 0.4; shot fullscreen-carousel
    wait_for 10 is_blue || bad "zap in fullscreen did not reach ch2" || return 1
    key Prior; wait_for 10 not_blue || bad "zap back to ch1 failed" || return 1
    key Escape; MODE=docked; wait_for 10 not_fullscreen || bad "Esc did not leave fullscreen (${AW}x${AH})" || return 1
}
s_mini() {
    hover_video; MODE=mini; key p; wait_for 10 win_size_is le 500 300 || bad "window ${AW}x${AH} after P, expected <=500x300" || return 1
    sleep 1; wait_for 10 is_bright || bad "mini video black" || return 1
    shot mini
    key Escape; MODE=docked; wait_for 10 win_size_is ge 800 500 || bad "Esc did not restore (${AW}x${AH})" || return 1
}
s_vod_seek() {
    key ctrl+2; sleep 1; key Home; key Return; sleep 1
    wait_for 20 is_bright || bad "movie video black" || return 1
    hover_video; sleep 1; shot vod-seekbar
    key space; sleep 1; probe vod-a >/dev/null; key Right; sleep 1.5; probe vod-b >/dev/null
    crop_probe vod-a pa; crop_probe vod-b pb
    [ "$(same pa pb)" -gt 0 ] || bad "frame unchanged after Right while paused" || return 1
    shot vod-after-seek
}
s_stop() {
    read -r vx vy vw vh _ <<<"$(video_rect)"
    hover_video; sleep 0.5; shot before-stop; crop_probe before-stop pa
    xdotool mousemove $((vx + 68)) $((vy + vh - 26)); sleep 0.3; xdotool click 1; sleep 2
    shot after-stop; crop_probe after-stop pb
    [ "$(same pa pb)" -gt 0 ] || bad "video pane unchanged after clicking Stop" || return 1
    kill -0 "$APP" 2>/dev/null || bad "app died" || return 1
    ! grep -q panicked "$OUT/app.log" || bad "panic in app.log" || return 1
}

s_mpv_args() { [ -s "$OUT/mpv.log" ] || bad "mpv.log missing: stored mpv_args were not applied"; }
RESULTS=(); FAILED=0
run() { # scenario-name function
    local t0=$SECONDS; WHY=
    if "$2"; then RESULTS+=("PASS $1 ($((SECONDS - t0))s)")
    else RESULTS+=("FAIL $1: $WHY ($((SECONDS - t0))s)"); FAILED=1; shot "fail-$1" || true; fi
    echo "${RESULTS[-1]}"
}
shot start
if [ -n "$REAL" ]; then
. "$REPO/scripts/vm/real-scenarios.sh"
run_real
else
run play-live s_play_live; run zap-down s_zap_down; run zap-up s_zap_up; run pause s_pause
run fullscreen s_fullscreen; run mini s_mini; run vod-seek s_vod_seek; run stop s_stop; run mpv-args s_mpv_args
fi

echo "--- app.log: panicked/ERROR lines"
if [ -n "$REAL" ]; then echo "$(grep -cE 'panicked|ERROR' "$OUT/app.log" || true) (count only: lines can hold URLs)"
else grep -E 'panicked|ERROR' "$OUT/app.log" | head -10 || true; fi
echo "--- shots"; ls "$OUT"/*.png
echo "--- summary"; printf '%s\n' "${RESULTS[@]}"
exit "$FAILED"
