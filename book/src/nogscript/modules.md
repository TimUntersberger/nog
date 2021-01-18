# modules

## Import

Modules can be imported either be import by using the `import` keyword

```nogscript
// builtin module
import nog
// local module
import keybindings
// plugin
import counter
```

or using the `require` function

```nogscript
// builtin module
var nog = require("nog")
// local module
var keybindings = require("keybindings")
// plugin
var counter = require("counter")
```

The file will be looked for in these directories, following the same order:

* BUILTIN_MODULES
* %APPDATA%/nog
* %APPDATA%/nog/plugins

## Export

Every file is automatically a module, but to expose something to users of a module you have to use the `export` keyword.

```nogscript
var x = 1

// You can export later on in the file
export x

// but you can also export immediately
export var y = 1
```
