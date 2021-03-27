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
