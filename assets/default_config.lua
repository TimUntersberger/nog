local direction_keys = {
  h = "left",
  j = "down",
  k = "up",
  l = "right"
}

local workspaces = { 1, 2, 3, 4, 5, 6, 7, 8, 9 }

nog.config.bar.font = "CaskaydiaCove NF"
nog.config.bar.font_size = 18

nog.config.work_mode = false
nog.config.display_app_bar = true
nog.config.launch_on_startup = true
nog.config.multi_monitor = true
nog.config.remove_task_bar = true

nog.config.workspaces = {
  [1] = {
    text = " Code "
  },
  [2] = {
    text = " Browser "
  }
}

nog.config.rules = {
  ["explorer.exe"] = { 
    ignore = true 
  },
  ["taskmgr.exe"] = { 
    ignore = true 
  },
  ["snippingtool.exe"] = { 
    ignore = true 
  },
  ["firefox.exe"] = {
    has_custom_titlebar = true,
    firefox = true
  },
  ["chrome.exe"] = {
    has_custom_titlebar = true,
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
