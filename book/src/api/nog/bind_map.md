# bind_map

Defines a new keybinding for each key in the map, where the key has the provided modifier prepended and the keybinding calls the provided callback with its value.

## Signature

```nogscript
fn bind_map(modifier: String, callback: () -> Void, map: Map<String, Any>)
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
