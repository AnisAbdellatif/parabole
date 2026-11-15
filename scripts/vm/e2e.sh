#!/bin/sh
# Host driver for the VM end-to-end tests. Usage: e2e.sh up|sync|build|test|fetch|all|down
# Everything runs in the UTM "Xubuntu" guest on Xvfb :99; nothing touches the host display.
set -eu
cd "$(dirname "$0")/../.."
UTMCTL=/Applications/UTM.app/Contents/MacOS/utmctl
VM=Xubuntu
GUEST_DIR=iptv-player-slint
OUT=/tmp/vm-e2e
SSH_OPTS="-o BatchMode=yes -o ConnectTimeout=5 -o StrictHostKeyChecking=accept-new"

lease_ip() {
    # several stale entries can share the name: take the newest lease (equal-length 0x hex compares as text)
    awk -F= '/^\{/{ip="";n="";l=""} /name=/{n=$2} /ip_address=/{ip=$2} /lease=/{l=$2} /^\}/{if(n=="xubuntu-vm"&&l>best){best=l;last=ip}} END{print last}' /var/db/dhcpd_leases
}
host() { echo "${VM_HOST:-$(lease_ip)}"; }
# shellcheck disable=SC2086
guest() { ssh $SSH_OPTS "xubuntu@$(host)" "$@"; }

up() {
    case "$("$UTMCTL" status "$VM" 2>/dev/null)" in
        started) ;;
        *) "$UTMCTL" start --hide "$VM" ;;
    esac
    i=0
    until [ -n "$(host)" ] && guest true 2>/dev/null; do
        i=$((i + 1))
        [ "$i" -le 60 ] || { echo "ssh to guest not up after 120 s" >&2; exit 1; }
        sleep 2
    done
}

sync_src() {
    # no --delete: the guest's target/ and e2e media must survive
    rsync -az --exclude target --exclude .git --exclude .tokenade \
        -e "ssh $SSH_OPTS" ./ "xubuntu@$(host):$GUEST_DIR/"
}

build() {
    guest ". ~/.cargo/env && cd $GUEST_DIR && cargo build --features slint-ui --bin iptv-player-slint -j3 && cargo build --features slint-ui --example vm_seed -j3"
}

test_run() {
    case "${E2E_DISPLAY:-:99}" in :[0-9]*) ;; *) echo "E2E_DISPLAY must look like :0 or :99" >&2; exit 2 ;; esac
    case "${E2E_REAL:-}" in ""|1) ;; *) echo "E2E_REAL must be 1 or unset" >&2; exit 2 ;; esac
    for v in "${E2E_REAL_LIVE:-}" "${E2E_REAL_VOD:-}"; do
        case "$v" in *[!A-Za-z0-9\ ._-]*) echo "E2E_REAL_LIVE/E2E_REAL_VOD: letters, digits, space . _ - only" >&2; exit 2 ;; esac
    done
    guest "cd $GUEST_DIR && E2E_DISPLAY='${E2E_DISPLAY:-:99}' E2E_REAL='${E2E_REAL:-}' E2E_REAL_LIVE='${E2E_REAL_LIVE:-}' E2E_REAL_VOD='${E2E_REAL_VOD:-}' bash scripts/vm/guest-e2e.sh"
}

fetch() {
    mkdir -p "$OUT"
    rsync -az --delete -e "ssh $SSH_OPTS" "xubuntu@$(host):e2e-out/" "$OUT/"
    echo "screenshots and logs in $OUT"
}

case "${1:-}" in
    up) up ;;
    sync) sync_src ;;
    build) build ;;
    test) test_run ;;
    fetch) fetch ;;
    all)
        up
        sync_src
        build
        rc=0
        test_run || rc=$?
        fetch
        exit "$rc"
        ;;
    down) "$UTMCTL" stop "$VM" ;;
    *) echo "usage: $0 up|sync|build|test|fetch|all|down" >&2; exit 2 ;;
esac
