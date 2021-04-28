# Keybindings

Nog keybindings can be of three different kinds:

* Normal (n)
* Work (w)
* Global (g)

`Normal` keybindings are only active while being inside [work mode](/getting-started/work_mode.html) **and** not inside any mode.

`Work` keybindings are only active while being inside [work mode](/getting-started/work_mode.html).

`Global` keybindings are always active as long as nog is running.

Nog comes with 4 different functions which can be used to define a new keybinding:

* `nog.bind(kind, key, cb)`
* `nog.nbind(key, cb)`
* `nog.wbind(key, cb)`
* `nog.gbind(key, cb)`

`nog.bind` expects to receive 3 arguments

* `kind` which has to be either `"n"`, `"w"` or `"g"`
* `key` which has to be a valid [key combination](#key-combos)
* `cb` which has to be a function

`nbind`, `wbind` and `gbind` just call the `bind` function with their kind and pass the given arguments.

```lua
nog.nbind("Alt+F1", function()
  print("Hello World!")
end)
```

**Note** that any keybinding currently active will swallow the keys, meaning that if you bind `1` you won't be able to type `1`.

## Key combos

A key combo is a `string` which can either have just a [key](#key-combos/keys) 
or a list of [modifiers](#key-combos/modifiers) followed by a [key](#key-combos/keys) 
and separated by `+`.

**Examples**

* `1`
* `Alt+Enter`
* `Alt+Control+T`

### Keys

* A - Z
* 0 - 9
* F1 - F12
* ,
* .
* Tab
* Space
* Enter
* Plus
* Minus
* Escape
* Backspace
* Left
* Up
* Right
* Down

### Modifiers

The windows key can't be bound, because windows 10 reserves all win key related keybindings for itself.

* Shift
* Control
* Alt
