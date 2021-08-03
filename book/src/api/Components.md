# Components

## Component

A component is a table with the following fields:
* `name`
* `render`
* `on_click` (optional)

## workspaces()

Creates a component that displays the workspaces currently being used on this display. 
The workspace that has focus is highlighted.

**Return**: [Component](#component)

## datetime(pattern)

Creates a component that displays the current datetime formatted with the `pattern`.

**Arguments**:
* `pattern` [string] a pattern for [fmt_datetime](../api/general#fmt-datetime)

**Return**: [Component](#component)

## padding(amount)

Creates a component that displays a space for `amount`.

**Arguments**:
* `amount` [number] amount of spaces

**Return**: [Component](#component)

## active_mode()

Creates a component that displays either nothing or the active mode.

**Return**: [Component](#component)

## current_window(max_width)

Creates a component that displays either nothing or the title of the window that has focus.

**Arguments**:
* `max_width` [number] the maximum width of the component

**Return**: [Component](#component)

## split_direction(values)

Creates a component that displays either the first item of `values` or the last one depending on the current split direction.

**Arguments**:
* `values` [table] must have 2 items where the first one is for vertical and the second one for horizontal

**Return**: [Component](#component)

## fullscreen_indicator(indicator)

Creates a component that displays either nothing or the `indicator` if the workspace is in [fullscreen mode](../getting-started/fullscreen-mode).

**Arguments**:
* `indicator` [string] the text to display

**Return**: [Component](#component)
