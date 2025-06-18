PLUGIN_NAME = 'test_plugin'
PLUGIN_AUTHOR = 'freemorger'
PLUGIN_VERSION = "v1.1"
PLUGIN_DESC = "Test plugin with counter and time"

ctr = 0

function onInit()
    frevi_stat_msg("Test Plugin loaded")
    frevi_reg_com("!testplug", test_counter)
    frevi_reg_com("!testplug_time", show_time)
end

function test_counter()
    ctr = ctr + 1
    frevi_stat_msg("Counter: " .. ctr)
end

function show_time()
    local time = os.date("%Y-%m-%d %H:%M:%S")
    frevi_stat_msg("Current time: " .. time)
end
