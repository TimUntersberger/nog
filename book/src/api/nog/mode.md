# mode

Defines a new mode and passes a custom bind function to the passed callback.
Make sure to call this bind function if you want mode specific keybindings.

## Signature

```nogscript
fn mode(name: String, callback: (bind: (keycombo: String, callback: () -> Void) -> Void) -> Void)
```

## Example

```nogscript
nog.mode("custom", bind => {
  bind("F1", () => print("Hello World"))
})
```
