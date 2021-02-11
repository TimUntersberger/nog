# move_out

Swaps the position of the current window with the next window in the given direction

Moves the current window out of a row/column in the given direction. The behavior
of this movement is essentially moving the current window so that it is a sibling
of its parent and introducing a new parent node that is the opposite type of the
previous parent if necessary.
## Signature

```nogscript
fn move_out(direction: "Left") -> Void
```

