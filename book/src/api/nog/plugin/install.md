# install

Tries to install the plugin from the github repository.

`short_name` should be of the following pattern: “\<username\>/\<repo\>”.

The plugins get installed into the plugins folder which is located next to your config file.

## Signature

```nogscript
fn install(short_name: String) -> Void
```

## Example

```nogscript
 nog.plugin.install("TimUntersberger/counter.nog")
```

