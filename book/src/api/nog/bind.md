# bind

Defines a new keybinding that calls the callback when the given key combo is pressed.

`always_active` is optional and defaults to false.
This flag tells nog to never unregister the keybinding as long as the program is running.
## Signature

```nogscript
fn bind(always_active: Boolean?, callback: (), key_combo: String) -> Void
```

## Example

```nogscript
 nog.bind("F1", () => print("Hello World"))
```

