local direction_keys = {
  H = "Left",
  J = "Down",
  K = "Up",
  L = "Right"
}

local workspaces = { 1, 2, 3, 4, 5, 6, 7, 8, 9 }

nog.config.bar.font = "CaskaydiaCove NF"
nog.config.bar.font_size = 18

nog.config.work_mode = false
nog.config.display_app_bar = true
nog.config.launch_on_startup = true
nog.config.multi_monitor = true
nog.config.remove_task_bar = true

--nog.config.workspaces = {
--  1 = {
--    text = "  "
--  },
--  2 = {
--    text = "  ",
--    monitor = 1
--  },
--  3 = {
--    text = " 阮 "
--  },
--  4 = {
--    text = " ﭮ "
--  },
--}

--nog.config.rules = {
--  "explorer.exe" = { 
--    ignore = true 
--  },
--  "Taskmgr.exe" = { 
--    ignore = true 
--  },
--  "SnippingTool.exe" = { 
--    ignore = true 
--  },
--  "firefox.exe" = {
--    has_custom_titlebar = true,
--    workspace_id = 2,
--    firefox = true
--  },
--  "chrome.exe" = {
--    has_custom_titlebar = true,
--    workspace_id = 2,
--    chromium = true
--  },
--  "Discord.exe" = {
--    has_custom_titlebar = true
--  },
--  "Spotify.exe" = {
--    has_custom_titlebar = true
--  },
--  "Code.exe" = {
--    has_custom_titlebar = true
--  },
--}

nog.wbind("F1", function()
  nog.launch("notepad.exe")
end)

nog.nbind("Alt+I", nog.win_ignore)
nog.nbind("Alt+Q", nog.win_close)
nog.nbind("Alt+M", nog.win_minimize)
nog.nbind("Alt+X", nog.quit)

nog.nbind("Alt+R", function() 
  nog.toggle_mode("resize")
end)

nog.nbind_tbl("Alt", nog.ws_focus, direction_keys)
nog.nbind_tbl("Alt+Control", nog.ws_swap, direction_keys)

 -- Moved this from window to workspace, because the split direction is workspace scoped and not window scoped.
 nog.nbind("Alt+Plus", function()
   nog.ws_set_split_direction("Vertical")
 end)
 nog.nbind("Alt+Minus", function()
   nog.ws_set_split_direction("Horizontal")
 end)

 nog.nbind("Alt+Control+F", nog.win_toggle_floating)
 nog.gbind("Alt+Control+W", nog.toggle_work_mode, "global")
 nog.nbind("Alt+F", nog.ws_toggle_fullscreen)

 nog.nbind_tbl("Alt+Shift", nog.win_move_to_workspace, workspaces)
 nog.nbind_tbl("Alt+Control", nog.ws_move_to_monitor, workspaces)
 nog.nbind_tbl("Alt", nog.ws_change, workspaces)
