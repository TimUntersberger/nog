# General

**Note:** keep in mind that all of the functions/variables in the api documentation are part of the `nog` global.
So to use the below `uv` variable you will have to write `nog.uv`.

## uv

lua bindings for libuv provided by [luv](https://github.com/luvit/luv). You can also require it directly by using `require 'luv'`.

## version

A `string` that contains the nog version you are currently running. If you downloaded a `vX.X.X` release this will contain the version number,
but if you downloaded a release of a branch like `development` this will contain the branch name together with the commit hash.

## runtime_path

A `string` that contains the path to the `runtime` folder.

## config_path

A `string` that contains the path to the `nog` folder.

## config

A `table` that contains the configuration of nog.

## quit()

Exits the nog process.

## fmt_datetime(pattern)

Formats the local time using the provided `pattern` and returns it.

**Arguments**:
* `pattern` a string that follows the [chrono spec](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html)

**Return**: a `string` that contains the local time formatted using the provided `pattern`.

## launch(name)

Runs the executable with `name` in a subprocess.

**Arguments**:
* `name` [string] name of the executable (ex. `notepad.exe`)

## scale_color(color, factor)

Scales the `color` by `factor`. Useful for creating different shades of a color.

**Arguments**:
* `color` [number] the color to be scaled
* `factor` [number] how much the color gets scaled

**Return**: `color` scaled by `factor`.

## toggle_work_mode()

Either leaves or enters [work mode](../getting-started/work_mode.html)

## inspect(value, [options])

The [kikito/inspect.lua](https://github.com/kikito/inspect.lua) function.

**Arguments**:

* `value` the value you want information on
* `options` an optional table which can have the following fields
  * `depth` sets the maximum depth that will be printed out. When the max depth is reached, it will stop parsing tables and just return `{...}`
  * `newline` the string used to add a newline to each level of a table. (Default `\n`)
  * `indent` the string used to add an indent to each level of a table. (Default `  `  - two spaces)
  * `process` a function which allow altering the passed object before transforming it into a string. A typical way to use it would be to remove certain values so that they don't appear at all.

**Return**: a human readable representation of the lua value in a `string`.

## get_focused_win()

**Return**: the window ID of the currently focused window (regardless of whether it's managed by nog or not)

## toggle_view_pinned()
Toggles the visibility of all non-workspace specific pinned programs.
