local function create_bind_tbl_fn(mode)
  return function(modifier, cb, tbl)
    for key, val in pairs(tbl) do
      local key = string.format("%s+%s", modifier, key)
      nog[mode .. "bind"](key, function()
        cb(val)
      end)
    end
  end
end

local function create_bind_fn(mode)
  return function(key, cb)
    nog.bind(mode, key, cb)
  end
end

nog.nbind = create_bind_fn("n")
nog.nbind_tbl = create_bind_tbl_fn("n")

nog.gbind = create_bind_fn("g")
nog.gbind_tbl = create_bind_tbl_fn("g")

nog.wbind = create_bind_fn("w")
nog.wbind_tbl = create_bind_tbl_fn("w")

nog.components = {}
nog.components.workspaces = function()
  return {
    name = "Workspaces",
    render = function(display_id)
      local c = nog.config
      local ws_ids = nog.get_active_ws_of_display(display_id)
      local result = {}
      local factor

      for _, ws_id in ipairs(ws_ids) do
        if c.light_theme then
          factor = nog.is_ws_focused(ws_id) and 0.75 or 0.9
        else
          factor = nog.is_ws_focused(ws_id) and 2.0 or 1.5
        end

        table.insert(result, {
          text = nog.get_ws_text(ws_id),
          value = ws_id,
          bg = c.bar.color
        })
      end

      return result
    end,
    on_click = function(display_id, payload)
      nog.ws_change(payload)
    end
  }
end

nog.components.datetime = function(format)
  return {
    name = "Datetime",
    render = function()
      return {{
        text = nog.fmt_datetime(format),
      }}
    end
  }
end

nog.components.padding = function(amount)
  return {
    name = "Padding",
    render = function()
      return {{
        text = string.rep(" ", amount),
      }}
    end
  }
end

nog.components.active_mode = function()
  return {
    name = "ActiveMode",
    render = function()
      local mode = nog.get_kb_mode()
      if mode ~= nil then
        mode = mode .. " is active"
      end
      return {{
        text = mode or "",
      }}
    end
  }
end

nog.components.current_window = function()
  return {
    name = "CurrentWindow",
    render = function()
      local win_id = nog.get_current_win()
      local title = nog.get_win_title(win_id)

      return {{
        text = title,
      }}
    end
  }
end

nog.components.split_direction = function()
  return {
    name = "SplitDirection",
    render = function()
      local ws_id = nog.get_current_ws()
      local info = nog.get_ws_info(ws_id)

      return {{
        text = info.split_direction,
      }}
    end
  }
end
