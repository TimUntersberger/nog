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

## unbind(key)

Unregisters the keybinding that has the given `key`.

**Arguments**:
* `key` [string] the key combination that activates this binding

**See Also**:
* [configuring keybindings](/configuration/keybindings.html)
