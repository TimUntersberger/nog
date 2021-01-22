# mode

Toggles the work mode.
## Signature

```nogscript
fn mode(mode: String, callback: (bind:) -> Void
```

## Example

```nogscript
 nog.mode("custom", bind => {
   bind("F1", () => print("Hello World"))
 })
```

