-- Keeps live streams near the live edge. Options: --script-opts=live-catchup-<key>=<value>; the app sets live-catchup-live=yes per live file
local opts = { enabled = true, start_above = 4, stop_below = 2, speed = 1.07, seek_above = 20, seek_margin = 1 }
require("mp.options").read_options(opts, "live-catchup")

local live, catching = false, false

local function set_catching(on)
    if on == catching then return end
    catching = on
    mp.set_property_number("speed", on and opts.speed or 1)
    mp.msg.info(on and "live-catchup: speeding up" or "live-catchup: back at 1.0x")
end

-- mpv reports a (growing) duration for live HLS, so the app flags live files per file instead
local function update_live()
    local flags = mp.get_property_native("options/script-opts") or {}
    live = opts.enabled and flags["live-catchup-live"] == "yes"
    if not live then set_catching(false) end
end

local function on_cache(_, cache)
    if not live or not cache then return end
    if cache > opts.seek_above then
        local edge = mp.get_property_number("demuxer-cache-time")
        if edge then
            mp.msg.info(string.format("live-catchup: %.1fs behind, seeking to edge", cache))
            mp.commandv("seek", edge - opts.seek_margin, "absolute+exact")
        end
        set_catching(false)
    elseif cache > opts.start_above then
        set_catching(true)
    elseif cache < opts.stop_below then
        set_catching(false)
    end
end

mp.register_event("start-file", function() catching = false end)
mp.register_event("file-loaded", update_live)
mp.observe_property("demuxer-cache-duration", "number", on_cache)
