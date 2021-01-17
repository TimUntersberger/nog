# bind

Defines a new keybinding that calls the callback when the given key combo is pressed.

## Signature

```nogscript
fn bind(keycombo: String, callback: () -> Void)
```

## Example

```nogscript
nog.bind("F1", () => print("Hello World"))
```
