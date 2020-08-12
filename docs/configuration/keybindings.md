# Keybindings

Defining a keybinding is very simple using the `bind` keyword.

```nog
bind "<key-combo>" <type>;
```

It takes two arguments, a [key combination]() and a [type]().

## Example

```nog
bind "Alt+H" focus("Left");
```

## Key Combinations

A key combination is written with each part joined by a `+` sign.

```nog
let key_combo = "Alt+Control+A";
```

**Note**: Every key/modifer written below can be used in a combo as written, except ranges (e.g. 0-9).

Because of the limitations of Windows you can only use the following modifiers:

* Alt
* Control
* Shift

And the key can be one of the following:

* A-Z
* 0-9
* Enter
* Plus / Minus
* Left / Up / Right / Down

## Types

A `type` is basically a function that returns some information about how to handle this key combination.

There exist a variety of different types for example changing focus, swapping tiles, ... and so on.

### ChangeWorkspace

Changes the current workspace. 

#### Arguments

| Position | Value  | Description                             |
|----------|--------|-----------------------------------------|
| 1        | Number | Id of the workspace to change to (1-10) |

#### Usage

```nog
bind "<key-combo>" change_workspace(<id>);
```

### MoveToWorkspace

Moves the current tile to a workspace.

#### Arguments

| Position | Value  | Description                                            |
|----------|--------|--------------------------------------------------------|
| 1        | Number | Id of the workspace to move the current tile to (1-10) |

#### Usage

```nog
bind "<key-combo>" move_to_workspace(<id>);
```

### MoveWorkspaceToMonitor

Move the current workspace to a monitor.

#### Arguments

| Position | Value  | Description                                        |
|----------|--------|----------------------------------------------------|
| 1        | Number | Id of the monitor to move the current workspace to |

#### Usage

```nog
bind "<key-combo>" move_workspace_to_monitor(<id>);
```

### MinimizeTile

Unmanages the current tile and minimizes it.

#### Arguments

| Position | Value  | Description                                        |
|----------|--------|----------------------------------------------------|

#### Usage

```nog
bind "<key-combo>" minimize_tile();
```

### CloseTile

Unmanages the current tile and closes it.

#### Arguments

| Position | Value  | Description                                        |
|----------|--------|----------------------------------------------------|

#### Usage

```nog
bind "<key-combo>" close_tile();
```

### Quit

Exits the window manager.

#### Arguments

| Position | Value  | Description                                        |
|----------|--------|----------------------------------------------------|

#### Usage

```nog
bind "<key-combo>" quit();
```

### ToggleFloatingMode

Manages/Unmanages the current window.

#### Arguments

| Position | Value  | Description                                        |
|----------|--------|----------------------------------------------------|

#### Usage

```nog
bind "<key-combo>" toggle_floating_mode();
```

### ToggleWorkMode

Toggles the [work mode]().

#### Arguments

| Position | Value  | Description                                        |
|----------|--------|----------------------------------------------------|

#### Usage

```nog
bind "<key-combo>" toggle_work_mode();
```

### ToggleFullscreen

Toggles fullscreen. Fullscreen means that the current tile takes up the whole space of the workspace. 
You can still use all the other keybindings like changing focus or swapping tiles.

#### Arguments

| Position | Value  | Description                                        |
|----------|--------|----------------------------------------------------|

#### Usage

```nog
bind "<key-combo>" toggle_fullscreen();
```

### Focus

Change the focus to the next tile in a direction.

#### Arguments

| Position | Value  | Description                                                   |
|----------|--------|---------------------------------------------------------------|
| 1        | String | The direction which you want to focus to (Left/Right/Up/Down) |

#### Usage

```nog
bind "<key-combo>" focus("<direction>");
```

### Swap

Swaps the current tile with the next tile in a direction

#### Arguments

| Position | Value  | Description                                                       |
|----------|--------|-------------------------------------------------------------------|
| 1        | String | The direction which you want to swap with (Left/Right/Up/Down) |

#### Usage

```nog
bind "<key-combo>" swap("<direction>");
```

### Split

Changes the orientation of the current tile. Per default a new tile gets opened vertically.

#### Arguments

| Position | Value  | Description                                               |
|----------|--------|-----------------------------------------------------------|
| 1        | String | New orientation of the current tile (Vertical/Horizontal) |

#### Usage

```nog
bind "<key-combo>" split("<orientation>");
```

### IncrementConfig

Increments a config value that takes a number.

#### Arguments

| Position | Value  | Description              |
|----------|--------|--------------------------|
| 1        | String | Name of the config value |
| 2        | Number | Amount                   |

#### Usage

```nog
bind "<key-combo>" increment_config("<key>", <amount>);
```

### DecrementConfig

Decrements a config value that takes a number.

#### Arguments

| Position | Value  | Description              |
|----------|--------|--------------------------|
| 1        | String | Name of the config value |
| 2        | Number | Amount                   |

#### Usage

```nog
bind "<key-combo>" decrement_config("<key>", <amount>);
```

### ToggleConfig

Toggles a config value that takes a boolean.

#### Arguments

| Position | Value  | Description              |
|----------|--------|--------------------------|
| 1        | String | Name of the config value |

#### Usage

```nog
bind "<key-combo>" toggle_config("<key>");
```