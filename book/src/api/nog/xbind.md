# xbind

Defines a new keybinding that calls the callback when the given key combo is pressed.

This keybinding won't get unregisterd when leaving work mode or entering a custom mode.

## Signature

```nogscript
fn xbind(keycombo: String, callback: () -> Void)
```

## Example

```nogscript
nog.xbind("F1", () => print("Hello World"))
```
