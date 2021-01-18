# bind_arr

Defines a new keybinding for each item in the array, where the key is the modifier + index and
the keybinding calls the provided callback with the item.
## Signature

```nogscript
fn bind_arr(arr: Any[], callback: (), key_combo: String) -> Void
```

## Example

```nogscript
 nog.bind_arr("Alt", nog.workspace.change, range(10))
```

