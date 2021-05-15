local direction_keys = {
  h = "left",
  j = "down",
  k = "up",
  l = "right"
}

-- Nog has 10 workspaces
local workspaces = { 1, 2, 3, 4, 5, 6, 7, 8, 9 }

nog.config.bar.font_size = 18

nog.config.work_mode = false
-- draws the app bar while nog is in work mode
nog.config.display_app_bar = true
nog.config.launch_on_startup = true
nog.config.multi_monitor = true
-- hides the task bar while nog is in work mode
nog.config.remove_task_bar = true

-- We customize the first two workspaces with a custom display text
nog.config.workspaces = {
  [1] = {
    text = " Code "
  },
  [2] = {
    text = " Browser "
  }
}

nog.config.rules = {
  -- we want to ignore explorer.exe, because the user experience is better when in floating mode IMO
  ["explorer.exe"] = { 
    ignore = true 
  },
  -- same thing here
  ["taskmgr.exe"] = { 
    ignore = true 
  },
  -- same thing here
  ["snippingtool.exe"] = { 
    ignore = true 
  },
  ["firefox.exe"] = {
    has_custom_titlebar = true,
    -- Adds special handling for firefox
    firefox = true
  },
  ["chrome.exe"] = {
    has_custom_titlebar = true,
    -- Adds special handling for chrome
    chromium = true
  },
  ["discord.exe"] = {
    has_custom_titlebar = true
  },
  ["spotify.exe"] = {
    has_custom_titlebar = true
  },
  ["code.exe"] = {
    has_custom_titlebar = true
  },
}

nog.nbind("alt+i", nog.win_ignore)
nog.nbind("alt+q", nog.win_close)
nog.nbind("alt+m", nog.win_minimize)
nog.nbind("alt+x", nog.quit)

-- nog.nbind_tbl will map each key to its value so writing the nog.nbind_tbl line is equal to the following
-- nog.nbind("alt+l", function() nog.ws_focus("Right") end)
-- nog.nbind("alt+k", function() nog.ws_focus("Up") end)
-- nog.nbind("alt+j", function() nog.ws_focus("Down") end)
-- nog.nbind("alt+h", function() nog.ws_focus("Left") end)
nog.nbind_tbl("alt", nog.ws_focus, direction_keys)
nog.nbind_tbl("alt+control", nog.ws_swap, direction_keys)

nog.nbind("alt+plus", function()
  nog.ws_set_split_direction("Vertical")
end)
nog.nbind("alt+minus", function()
  nog.ws_set_split_direction("Horizontal")
end)

nog.nbind("alt+control+f", nog.win_toggle_floating)
nog.gbind("alt+control+w", nog.toggle_work_mode)
nog.nbind("alt+f", nog.ws_toggle_fullscreen)

nog.nbind_tbl("alt+shift", nog.win_move_to_workspace, workspaces)
nog.nbind_tbl("alt+control", nog.ws_move_to_monitor, workspaces)
nog.nbind_tbl("alt", nog.ws_change, workspaces)
