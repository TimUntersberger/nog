# uninstall

Tries to uninstall the plugin.

`short_name` should be of the following pattern: “\<username\>/\<repo\>”.

## Signature

```nogscript
fn uninstall(short_name: String) -> Void
```

## Example

```nogscript
 nog.plugin.uninstall("TimUntersberger/counter.nog")
```

