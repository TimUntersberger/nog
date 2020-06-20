
# wwm
Windows window manager

![Preview](/preview.png?raw=true "Preview")

Terminal: Windows Terminal | Colourscheme: Nord

[Demo](https://giant.gfycat.com/ThriftyMassiveJabiru.webm)

## Config

The config lives in `C:\Users\<User>\AppData\Roaming\wwm\config.yaml`

wwm itself doesn't have any default keybindings defined, so you have to define them yourself. Every setting besides keybindings has a default value which can be seen in the example config.

### Toggles

The `display_app_bar` setting creates a window at the top of the display that shows all currently used workspaces.

The `remove_title_bar` setting removes the windows styles responsible for giving a managed window the titlebar.

The `remove_task_bar` setting hides the taskbar on launch and shows it again when closing the program.

### Rules

**[WARNING]: Rules are still WIP so the name of a setting can change at any time**

Because Windows can sometimes have some really annoying applications that introduce various edge cases I decided to support rules.

A rule basically just changes the way a window gets managed by wwm.

wwm knows whether to apply the rule based on a regex that has to be provided.

Example
```yaml
pattern: ^.*- Mozilla Firefox$
has_custom_titlebar: true
x: -6
width: 12
```

This example rule tells wwm that firefox will have a custom titlebar. Some applications like firefox and chrome don't use the native titlebar to minimize wasted space.  

The problem is that sometimes the application then behaves weirdly when trying to move/resize it, that's why I also defined an x and width value. These just manipulate where the window gets placed and its size.

Right below the values settings can you find some rules that I found while developing the application that were helpful.

#### Settings

<details>
  <summary>pattern</summary></br>
  A regex that tells wwm which window this rule applies to.
</br></br></details>

<details>
  <summary>has_custom_titlebar</summary></br>
  A boolean that tells wwm how to remove the "titlebar"
</br></br></details>

<details>
  <summary>workspace [TODO]</summary></br>
  An integer between 1 and 10 that tells wwm in which workspace to put the window.
</br></br></details>

<details>
  <summary>manage [TODO]</summary></br>
  A boolean that tells wwm whether to manage the matched window. </br>
  This overrides the logic that normally decides whether to manage the window.
</br></br></details>

<details>
  <summary>x</summary></br>
  An integer that gets added to the calculated x position of the window
</br></br></details>

<details>
  <summary>width</summary></br>
  An integer that gets added to the calculated width of the window
</br></br></details>

#### Examples

Firefox
```yaml
pattern: ^.*- Mozilla Firefox$
has_custom_titlebar: true
x: -6
width: 12
```

Chrome
```yaml
pattern: ^.*- Google Chrome$
has_custom_titlebar: true
x: -8
width: 16
```


### Keybindings

Each keybinding has to have a type, key and maybe additional settings which can be looked up for each type specifically.

Keybindings can have the following types:

* Shell
* CloseTile
* Quit
* Focus
* Split
* ToggleFloatingMode
* ChangeWorkspace

#### ChangeWorkspace

example
```yaml
type: ChangeWorkspace
key: Control+Alt+1
id: 1
```

values
* 1
* 2
* 3
* 4
* 5
* 6
* 7
* 8
* 9
* 10

A ChangeWorkspace keybinding takes an id, which is the id of the workspace to change to.

Workspaces have an upper limit of 10, so if you define a keybinding of type ChangeWorkspace that goes above 10 or below 1 the program *currently* just crashes.

#### Shell

example
```yaml
type: Shell
key: Control+Alt+Enter
cmd: start firefox
```

A Shell keybinding takes a cmd, which has to be a valid cmd statement. So if it is possible to execute the statement in the cmd terminal then it is also possible to run it using the Shell keybindng.

#### CloseTile

example
```yaml
type: Shell
key: Control+Alt+Q
```

A CloseTile keybinding closes the currently focused tile and its window.

#### Quit

example
```yaml
type: Shell
key: Control+Alt+X
```

A Quit keybinding closes wwm and unmanages each window.

#### ToggleFloatingMode

example
```yaml
type: Shell
key: Control+Alt+F
```

A ToggleFloatingMode keybinding either manages the currently focused window if it is not already managed or unmanages it.

#### Focus

example
```yaml
type: Shell
key: Control+Alt+H
direction: Left
```

values
* Left
* Right
* Up
* Down

A Focus keybinding takes a direction, specifying which window gets the focus.

#### Split

example
```yaml
type: Shell
key: Control+Alt+Minus
direction: Horizontal
```

values
* Horizontal
* Vertical

A Split keybinding takes a direction, the new SplitDirection of the currently focused window. The SplitDirection specifies how a new window gets placed in the grid.

### Example Config
```yaml
display_app_bar: true
remove_title_bar: true
remove_task_bar: true

rules:
  - pattern: ^.*- Mozilla Firefox$
    has_custom_titlebar: true
    x: -6
    width: 12
  - pattern: ^.*- Google Chrome$
    has_custom_titlebar: true
    x: -8
    width: 16

keybindings:
  - type: Shell
    key: Control+Alt+Enter
    cmd: wt
  - type: Shell
    key: Control+Alt+B
    cmd: start firefox

  - type: CloseTile
    key: Control+Alt+Q

  - type: Quit
    key: Control+Alt+X

  - type: Focus
    key: Control+Alt+H
    direction: Left
  - type: Focus
    key: Control+Alt+J
    direction: Down
  - type: Focus
    key: Control+Alt+K
    direction: Up
  - type: Focus
    key: Control+Alt+L
    direction: Right

  - type: Split
    key: Control+Alt+Plus
    direction: Vertical
  - type: Split
    key: Control+Alt+Minus
    direction: Horizontal

  - type: ToggleFloatingMode
    key: Control+Alt+F

  - type: ChangeWorkspace
    key: Control+Alt+1
    id: 1
  - type: ChangeWorkspace
    key: Control+Alt+2
    id: 2
  - type: ChangeWorkspace
    key: Control+Alt+3
    id: 3
  - type: ChangeWorkspace
    key: Control+Alt+4
    id: 4
```

## Development

### Create installer

```
cargo build --release
./rcedit ./target/release/wwm.exe --set-icon ./logo.ico
cargo wix init --force
cargo wix --bin-path "C:\Program Files (x86)\WiX Toolset v3.11\bin" --no-build
```
