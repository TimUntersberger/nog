local Window = {}

local function new_window(instance)
  setmetatable(instance, Window)
  return instance
end

--- Returns the window that has focus AND is managed by nog. Might be nil
function Window.get_focused()
  local win = nog.get_current_win()

  if win then
    return new_window(nog.get_win_info(win))
  end
end

return Window
