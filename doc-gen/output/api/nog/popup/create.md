# create

Creates a new popup with given settings.

```
type PopupSettings = #{
  text?: String | String[],
  padding?: Number
}
```
## Signature

```nogscript
fn create(settings: PopupSettings) -> Void
```

## Example

```nogscript
 nog.popup.create(#{ text: "Hello World" })
```

