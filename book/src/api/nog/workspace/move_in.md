# move_in

Moves the current window into the adjacent row/column/window found in the given
direction. If the adjecent item is a row or column, this simply moves the window
to the end of the row or column. If the adjacent item is a window, this introduces
a new column or row container, whichever is the opposite of the current window's
parent, and appends the window and the adjacent window within the new container.
## Signature

```nogscript
fn move_in(direction: "Left") -> Void
```

