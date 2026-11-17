-- Keeps live streams near the live edge. Options: --script-opts=live-catchup-<key>=<value>; the app sets live-catchup-live=yes per live file
-- seek_margin must equal LIVE_EDGE_MARGIN_S and seek_margin + 0.5 must equal GO_LIVE_MIN_S in src/player/live.rs
local opts = {
    enabled = true,
    start_above = 1.5, stop_below = 1.0, speed = 1.1, -- catch up at `speed` from start_above until cache < stop_below
    fast_above = 3, fast_speed = 1.25,                -- faster tier while the backlog is above fast_above
    seek_above = 8, seek_margin = 0.7, seek_cooldown = 10,
    burst_after = 2, burst_window = 6, burst_above = 3, -- skip the provider's connect burst once, 2-6 s after load
}
require("mp.options").read_options(opts, "live-catchup")

local live, catching, last_seek, loaded_at = false, nil, -math.huge, nil
-- per-file thresholds derived from live-catchup-ok (seconds behind that still count as live)
local th = { start = opts.start_above, stop = opts.stop_below, fast = opts.fast_above, burst = opts.burst_above, seek = opts.seek_above }

local function set_catching(speed)
    if speed == catching then return end
    catching = speed
    mp.set_property_number("speed", speed or 1)
    mp.msg.info(speed and string.format("live-catchup: speeding up to %.2fx", speed) or "live-catchup: back at 1.0x")
end

-- seeking inside the demuxer cache is instant: no reconnect, no new burst
local function seek_to_edge(cache)
    local edge = mp.get_property_number("demuxer-cache-time")
    if not edge then return false end
    last_seek = mp.get_time()
    mp.msg.info(string.format("live-catchup: %.1fs behind, seeking to edge", cache))
    mp.commandv("seek", edge - opts.seek_margin, "absolute+exact")
    set_catching(nil)
    return true
end

-- mpv reports a (growing) duration for live HLS, so the app flags live files per file instead
local function update_live()
    local flags = mp.get_property_native("options/script-opts") or {}
    live = opts.enabled and flags["live-catchup-live"] == "yes"
    local ok = tonumber(flags["live-catchup-ok"]) or 3
    th = { start = ok, stop = math.max(ok - 0.5, 0.5), fast = ok + 1.5, burst = ok, seek = math.max(opts.seek_above, ok + 5) }
    loaded_at = live and mp.get_time() or nil
    if not live then set_catching(nil) end
end

local function on_cache(_, cache)
    -- paused: the cache keeps growing, and seeking now would blank the paused frame; resume re-checks below
    if not live or not cache or mp.get_property_bool("pause") then return end
    if loaded_at then
        local since = mp.get_time() - loaded_at
        if since > opts.burst_window then
            loaded_at = nil
        elseif since >= opts.burst_after and cache > th.burst then
            loaded_at = nil
            if seek_to_edge(cache) then return end
        end
    end
    -- a decoder (or provider burst) that can't keep up refills the cache at once: seeking every second leaves only black frames
    if cache > th.seek and mp.get_time() - last_seek >= opts.seek_cooldown and seek_to_edge(cache) then
        return
    end
    if cache > th.fast then
        set_catching(opts.fast_speed)
    elseif catching then
        if cache < th.stop then
            set_catching(nil)
        elseif catching == opts.fast_speed then
            set_catching(opts.speed)
        end
    elseif cache > th.start then
        set_catching(opts.speed)
    end
end

-- Go live: seek to the edge when enough is buffered, else reconnect (replays the entry with its loadfile options)
mp.register_script_message("live-catchup-go-live", function()
    if not live then return end
    local cache = mp.get_property_number("demuxer-cache-duration")
    if cache and cache > opts.seek_margin + 0.5 then
        mp.set_property_bool("pause", false) -- a paused backlog is the common case: resume at the edge, don't reconnect
        if seek_to_edge(cache) then return end
    end
    mp.commandv("playlist-play-index", "current")
end)

mp.register_event("start-file", function() catching, last_seek, loaded_at = nil, -math.huge, nil end)
mp.register_event("file-loaded", update_live)
mp.observe_property("demuxer-cache-duration", "number", on_cache)
mp.observe_property("pause", "bool", function(_, paused)
    if paused == false then on_cache(nil, mp.get_property_number("demuxer-cache-duration")) end
end)
