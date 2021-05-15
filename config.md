# Updating Config Files
#### _Tips for converting .ns config files to .lua format_

The new .lua format uses the Lua programming language, so looking up the syntax may be helpful but if you've worked with the .ns config before, you should be able to read/write .lua format pretty easily.


## Variables/Key Objects

_config.ns_

```
var vim_keys = #{
    "H": "Left",
    "J": "Down",
    "K": "Up",
    "L": "Right",
}
```
_init.lua_
```
local vim_keys = {
  h = "left",
  j = "down",
  k = "up",
  l = "right"
}
```

Note the following:
- `local` instead of `var`
- no `#` at the start of the object
- lowercase
- no quotes in the key value (h instead of "h")
- `=` instead of `:`

The `range` function has been removed and instead you can just create a lua table.

_config.ns_
```
nog.bind_arr("Alt", nog.workspace.change, range(workspace_count))

```
_init.lua_
```
local workspaces = { 1, 2, 3, 4, 5, 6, 7, 8, 9 }
nog.nbind_tbl("alt", nog.ws_change, workspaces)
```

## Workspace/Window
In .ns files, functions that were workspace- or window- specific were prefixed with `workspace.` or `window.`. In .lua those same functions are prefixed with ws_ or win_. 

_config.ns_
```
nog.workspace.set_split_direction("Vertical")
nog.bind("Alt+Shift+Q", nog.window.close)
```

_init.lua_
```
nog.ws_set_split_direction("Vertical")
nog.nbind("alt+shift+q", nog.win_close)
```

## Binding 

In .ns files, using a table/object type to bind multiple values at once was done with bind_map. Instead, in .lua bind_tbl should be used.

_config.ns_
```
nog.bind_map("Alt", nog.workspace.focus, vim_keys)
```

_init.lua_
```
nog.nbind_tbl("alt", nog.ws_focus, vim_keys)

```

Also note, binding specifically to Normal, Work or Global uses an `n`, `w`, or `g` prefix:

```
nog.nbind("alt+f", nog.ws_toggle_fullscreen) -- only in the current mode/normal mode
nog.gbind("alt+f", nog.ws_toggle_fullscreen) -- always bound
nog.wbind("alt+f", nog.ws_toggle_fullscreen) -- bound only in work mode
```

## User-defined Functions

Functions in .ns files were declared with `() => { }`. In .lua, they follow this syntax `function() end`

_config.ns_
```
nog.bind("Alt+Shift+Enter", () => nog.launch("notepad.exe"))
```
_init.ns_
```
nog.nbind("Alt+Shift+Enter", function () 
    nog.launch("notepad.exe")
end)
```


## Setting Config Values

_config.ns_
```
nog.config.enable("display_app_bar");
```

_init.lua_
```
nog.config.display_app_bar = true
```
Pretty straightforward for setting an initial value; use an `=` and `true/false` value or a number for an integer type. There no longer are `enable`, `disable`, `increment`, `decrement` or `toggle` functions. Instead, they can be handled with user-defined functions. 

_enable function_
```
nog.nbind("alt+t", function ()
    nog.config.display_app_bar = true
end)
```

_disable function_
```
nog.nbind("alt+t", function ()
    nog.config.display_app_bar = false
end)
```

_toggle function_
```
nog.nbind("alt+t", function ()
    nog.config.display_app_bar = not nog.config.display_app_bar 
end)
```

_increment function_
```
nog.nbind("alt+t", function ()
    nog.config.inner_gap = nog.config.inner_gap + 1
end)
```

_decrement function_
```
nog.nbind("alt+t", function ()
    nog.config.inner_gap = nog.config.inner_gap - 1
end)
```

## Configuring Bar

A few bar config values can now be set on the nog.config object and the syntax for defining components is slightly different.

_config.ns_
```
nog.bar.configure(#{
    height: 16,
    font_size: 20,
    font: "Fixedsys",
    components: #{
        left: [components.workspaces()],
        center: [components.current_window()],
        right: [
            components.active_mode(),
            components.padding(4),
            components.fullscreen_indicator("[]"),
            components.padding(1),
            components.split_direction("|", "-"),
            components.padding(1),
            components.date("%e %b"),
            components.padding(1),
            components.time("%T"),
            components.padding(1)
        ]
    }
})
```

_init.lua_
```
nog.config.bar.height = 16
nog.config.bar.font_size = 20
nog.config.bar.font = "Fixedsys"

nog.config.bar.components = {
    left = { nog.components.workspaces() },
    center = { nog.components.current_window() },
    right = {
        nog.components.active_mode(),
        nog.components.padding(4),
        nog.components.fullscreen_indicator("[]"),
        nog.components.padding(1),
        nog.components.split_direction({ "|", "-" }),
        nog.components.padding(1),
        nog.components.datetime("%e %b %T"),

    }
}
```

Note:
- split_direction takes a table `{ "|", "-" }` instead of two parameters `"|", "-"`
- Date/Time has changed from `date("%e %b")` and `time("%T")` to just `datetime("%e %b %T")`

## Rules

_config.ns_
```
var ignored = [
    "explorer.exe",
    "Taskmgr.exe",
    "SnippingTool.exe",
]
ignored.for_each(nog.rules.ignore);


nog.rules.match("firefox.exe", #{
    has_custom_titlebar: true,
    firefox: true
})
nog.rules.match("chrome.exe", #{
    has_custom_titlebar: true,
    chromium: true
})
```

_init.lua_
```
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
}
```

Note: 
- `=` instead of `:`
- keys are wrapped in `[" "]`

## Modes

_config.ns_
```
mode("move", bind => {
    bind("Alt+M", () => nog.toggle_mode("move"))
    bind("Escape", () => nog.toggle_mode("move"))

    bind("W", () => nog.workspace.focus("Up"))
    bind("A", () => nog.workspace.focus("Left"))
    bind("S", () => nog.workspace.focus("Down"))
    bind("D", () => nog.workspace.focus("Right"))

    bind("H", () => nog.workspace.swap("Left"))
    bind("J", () => nog.workspace.swap("Down"))
    bind("K", () => nog.workspace.swap("Up"))
    bind("L", () => nog.workspace.swap("Right"))
})
nog.bind("Alt+M", () => nog.toggle_mode("move"))

```

_init.lua_
```
nog.mode("move", function()
    nog.nbind("Escape", function()
      nog.toggle_mode("move")
    end)
    nog.nbind("alt+m", function()
      nog.toggle_mode("move")
    end)

    nog.nbind("w", function() nog.ws_focus("up") end)
    nog.nbind("a", function() nog.ws_focus("left") end)
    nog.nbind("s", function() nog.ws_focus("down") end)
    nog.nbind("d", function() nog.ws_focus("right") end)

    nog.nbind("h", function() nog.ws_swap("Left") end)
    nog.nbind("j", function() nog.ws_swap("Down") end)
    nog.nbind("k", function() nog.ws_swap("Up") end)
    nog.nbind("l", function() nog.ws_swap("Right") end)
end)

```

Note:
- `function () end` vs `() =>` syntax



## Having Issues?
Feel free to create an issue or add a comment in [documentation feedback](https://github.com/TimUntersberger/nog/issues/106) if you're running into problems converting to the new format. 

