# Keybindings

## bind(mode, key, cb)

Registers a new keybinding.

**Arguments**:
* `mode` [string] one of the following:
  * `n` for normal
  * `w` for work
  * `g` for global
* `key` [string] the key combination that activates this binding
* `cb` [function] the function that gets called on keybinding activation

**See Also**:
* [configuring keybindings](/configuration/keybindings.html)

## nbind(key, cb)

Registers a new keybinding in `normal` mode.

**Arguments**:
* `key` [string] the key combination that activates this binding
* `cb` [function] the function that gets called on keybinding activation

**See Also**:
* [configuring keybindings](/configuration/keybindings.html)

## gbind(key, cb)

Registers a new keybinding in `global` mode.

**Arguments**:
* `key` [string] the key combination that activates this binding
* `cb` [function] the function that gets called on keybinding activation

**See Also**:
* [configuring keybindings](/configuration/keybindings.html)

## wbind(key, cb)

Registers a new keybinding in `work` mode.

**Arguments**:
* `key` [string] the key combination that activates this binding
* `cb` [function] the function that gets called on keybinding activation

**See Also**:
* [configuring keybindings](/configuration/keybindings.html)

## nbind_tbl(modifiers, cb, tbl)

Registers a new keybinding for each pair from the given `tbl` in `normal` mode,
where the key of the pair is prepended with the `modifiers` and used as the key 
and the value gets passed to the callback when activated.

**Arguments**:
* `modifiers` [string] the modifiers which will get prepended when registering the keybindings
* `cb` [function] the function that gets called on keybinding activation
* `tb` [table] the mappgins

## wbind_tbl(modifiers, cb, tbl)

Registers a new keybinding for each pair from the given `tbl` in `work` mode,
where the key of the pair is prepended with the `modifiers` and used as the key 
and the value gets passed to the callback when activated.

**Arguments**:
* `modifiers` [string] the modifiers which will get prepended when registering the keybindings
* `cb` [function] the function that gets called on keybinding activation
* `tb` [table] the mappgins

## gbind_tbl(modifiers, cb, tbl)

Registers a new keybinding for each pair from the given `tbl` in `global` mode,
where the key of the pair is prepended with the `modifiers` and used as the key 
and the value gets passed to the callback when activated.

**Arguments**:
* `modifiers` [string] the modifiers which will get prepended when registering the keybindings
* `cb` [function] the function that gets called on keybinding activation
* `tb` [table] the mappgins

## unbind(key)

Unregisters the keybinding that has the given `key`.

**Arguments**:
* `key` [string] the key combination that activates this binding

**See Also**:
* [configuring keybindings](/configuration/keybindings.html)
