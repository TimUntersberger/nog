# bind_arr

Defines a new keybinding for each item in the array, where the key is the modifier + index and
the keybinding calls the provided callback with the item.

`always_active` is optional and defaults to false.
This flag tells nog to never unregister the keybinding as long as the program is running.
## Signature

```nogscript
fn bind_arr(always_active: Boolean?, arr: Any[], callback: (), modifier: String) -> Void
```

## Example

```nogscript
 nog.bind_arr("Alt", nog.workspace.change, range(10))
```

