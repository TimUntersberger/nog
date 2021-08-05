# Window

## get_win_title(win_id)

Returns the title of window with the given `win_id`

**Arguments**:
* `win_id` [number] id of window

**Return**: [string] window title

## get_current_win()

Returns the id of the window that has currently focus and is managed by nog.

**Return**: [number] window id


## get_focused_win_of_display(display_id)

Returns the id of the window on the given display that is focused and is managed by nog.

**Arguments**:
* `display_id` [number] id of display

**Return**: [number] window id

## win_minimize()

Minimizes the currently focused window and unmanages it.

## win_ignore()

Unmanages the currently focused window and adds a rule that prevents this window from being managed while nog is running.

## win_close()

Closes the currently focused window.

## win_toggle_floating()

 Toggles [floating mode](../getting-started/floating_mode.html) of the currently focused window.

## win_move_to_ws(ws_id)

Moves the currently focused window to the workspace with the provided `ws_id`.

**Arguments**:
* `ws_id` [number] id of workspace

## win_toggle_pin(win_id)

Pins or unpins the given window. This is a no-op if the window is currently tile-managed by nog.

**Arguments**:
* `win_id` [number] id of window

## win_toggle_ws_pin(win_id)

Pins or unpins the given window to the current workspace. This is a no-op if the window is currently tile-managed by nog.
Note: when using multiple monitors only one workspace is focused. Ensure you're pinning the window to the correct workspace.

**Arguments**:
* `win_id` [number] id of window

## win_is_pinned(win_id)

Returns whether the given window is pinned or not.

**Arguments**:
* `win_id` [number] id of window

**Return**: [bool] true if pinned, otherwise false

## win_hide_title_bar(win_id) 
Hides the title bar on the given window.

**Arguments**:
* `win_id` [number] id of window

## win_show_title_bar(win_id) 
Shows the title bar on the given window.

**Arguments**:
* `win_id` [number] id of window

## win_hide_border(win_id) 
Hides the border on the given window.

**Arguments**:
* `win_id` [number] id of window

## win_show_border(win_id) 
Shows the border on the given window.

**Arguments**:
* `win_id` [number] id of window
