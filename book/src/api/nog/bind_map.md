# bind_map

Defines a new keybinding for each key in the map, where the key has the provided modifier
prepended and the keybinding calls the provided callback with its value.

`always_active` is optional and defaults to false.
This flag tells nog to never unregister the keybinding as long as the program is running.
## Signature

```nogscript
fn bind_map(always_active: Boolean?, callback: (), map: Map<String,, modifier: String) -> Void
```

## Example

```nogscript
 nog.bind_map("Alt", nog.workspace.focus, #{
   "H": "Left",
   "J": "Down",
   "K": "Up",
   "L": "Right"
 })
```

