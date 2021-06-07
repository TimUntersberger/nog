# Workspace

## get_active_ws_of_display(display_id)

Returns the id of the currently focused workspace of the display with `display_id`.

**Arguments**:
* `display_id` [number] id of display

**Return**: [number] id of workspace

## is_ws_focused(ws_id)

Checks if the workspace with the `ws_id` is focused.

**Arguments**:
* `ws_id` [number] id of workspace

**Return**: [boolean] whether the workspace is focused

## get_ws_info(ws_id)

Returns a table with various information about the workspace with the `ws_id`.

**Arguments**:
* `ws_id` [number] id of workspace

**Return**: [table] information about workspace
* `id` id of workspace
* `is_fullscreen` whether the workspace is in fullscreen mode
* `is_empty` whether the workspace is empty
* `split_direction` in which direction a new window gets managed (`"Vertical"` or `"Horizontal"`)
* `windows` a list of window ids that are inside the workspace

## get_current_ws()

Returns the id of the currently focused workspace.

**Return**: [number] id of workspace

## get_ws_text(ws_id)

Returns the display text of the workspace with the `ws_id`.

**Arguments**:
* `ws_id` [number] id of workspace

**Return**: [string] display text of workspace

## ws_toggle_fullscreen()

Toggles the [fullscreen mode](/getting-started/fullscreen_mode.html) of the current workspace.

## ws_reset_row()

Resets any resizing done on the current row.

## ws_reset_col()

Resets any resizing done on the current column.

## ws_move_to_monitor(display_id)

Moves the current workspace to the display with the `display_id`.

**Arguments**:
* `display_id` [number] id of display

## ws_replace(ws_id)

Empties the workspace with the `ws_id` and moves the content of the current workspace into the given one.

**Arguments**:
* `ws_id` [number] id of workspace

## ws_change(ws_id)

Changes the focus from the current workspace to the workspace with the `ws_id`.

**Arguments**:
* `ws_id` [number] id of workspace

## ws_focus(direction)

Changes the focus from the current window to the next window in the `direction`.

**Arguments**:
* `direction` [string] has to be one of the following:
  * `Left`
  * `Up`
  * `Right`
  * `Down`

## ws_resize(direction)

Resizes either the row or the column by the given `amount` depending on the given `direction`.

Left | Right -> Column

Up | Down -> Row

**Arguments**:
* `direction` [string] has to be one of the following:
  * `Left`
  * `Up`
  * `Right`
  * `Down`
* `amount` [number] by how much to resize

## ws_swap(direction)

Swaps the position of the current window with the next window in the `direction`.

**Arguments**:
* `direction` [string] has to be one of the following:
  * `Left`
  * `Up`
  * `Right`
  * `Down`

## ws_swap_columns_and_rows()

Turns all columns in the current workspace into rows and all rows into columns

## ws_set_split_direction(direction)

Sets the split direction of the current workspace.

**Arguments**:
* `direction` [string] has to be one of the following:
  * `Vertical`
  * `Horizontal`

## ws_move_in(direction)

Moves the current window into the adjacent row/column/window found in the given `direction`.

* If the adjecent item is a row or column, this simply moves the window to the end of the row or column. 
* If the adjacent item is a window, this introduces a new column or row container, 
whichever is the opposite of the current window's parent, and appends the window and the adjacent window within the new container.

**Arguments**:
* `direction` [string] has to be one of the following:
  * `Left`
  * `Up`
  * `Right`
  * `Down`

## ws_move_out(direction)

Moves the current window out of a row/column in the given `direction`. 
The behavior of this movement is essentially moving the current window so that it is a sibling of its parent and introducing a new parent node that is the opposite type of the previous parent if necessary.

**Arguments**:
* `direction` [string] has to be one of the following:
  * `Left`
  * `Up`
  * `Right`
  * `Down`
