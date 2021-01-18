# mode

Toggles the work mode.
## Signature

```nogscript
fn mode(callback: (bind:, mode: String) -> Void
```

## Example

```nogscript
 nog.mode("custom", bind => {
   bind("F1", () => print("Hello World"))
 })
```

