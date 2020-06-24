
# wwm
Windows window manager

![Preview](/screen-1.png?raw=true "Preview")

Terminal: Windows Terminal | Colourscheme: Nord

[Demo](https://giant.gfycat.com/ThriftyMassiveJabiru.webm)

## Table Of Contents

  * [Config](#config)
     * [Gap](#gap)
     * [Bar](#bar)
     * [Toggles](#toggles)
     * [Rules](#rules)
        * [Settings](#settings)
        * [Examples](#examples)
     * [Keybindings](#keybindings)
        * [ChangeWorkspace](#changeworkspace)
        * [Shell](#shell)
        * [CloseTile](#closetile)
        * [Quit](#quit)
        * [ToggleFloatingMode](#togglefloatingmode)
        * [Focus](#focus)
        * [Swap](#swap)
        * [Split](#split)
     * [Example Config](#example-config)
  * [Screenshots](#screenshots)
  * [Development](#development)
     * [Create installer](#create-installer)
     * [Create TOC](#create-toc)

## Info

WWM does currently not handle windows that only hide when "closing" them.

An example is powershell. When you type exit inside the console it doesn't close the window so I don't know how to detect that.
     
## Config

The config lives in `C:\Users\<User>\AppData\Roaming\wwm\config.yaml`.

If you want to see my own config or just want to know what a config looks like: [my config](#example-config).

### Gap

**[INFO]: There currently exists a problem with the padding. Sometimes the inner gap may vary slightly, because of rounding issues. I currently have no idea how to fix this**

The `margin` setting defines the size of the gap around the grid

The `padding` setting defines the size of the gap inside the grid

### bar

The `app_bar_height` setting defines the height of the app bar

The `app_bar_font` setting defines the font for the appbar widgets.

The `app_bar_font_size` setting defines the font size for the appbar widgets.

A color has to be a valid hex value (e.g 0x27242c)

The `app_bar_bg` setting defines the background color of the appbar

The `app_bar_workspace_bg` setting defines the background color of a workspace in the background

### Toggles

The `display_app_bar` setting creates a window at the top of the display that shows all currently used workspaces.

The `remove_title_bar` setting removes the windows styles responsible for giving a managed window the titlebar.

The `remove_task_bar` setting hides the taskbar on launch and shows it again when closing the program.

### Rules

**[WARNING]: Rules are still WIP so the name of a setting can change at any time**

Because Windows can sometimes have some really annoying applications that introduce various edge cases I decided to implement a way to configure specific windows.

A rule basically just changes the way a window gets managed by wwm.

WWM knows whether to apply the rule based on a regex that has to be provided.

#### Settings

<details>
  <summary>pattern</summary></br>
  A regex that tells wwm which window this rule applies to.
</br></br></details>

<details>
  <summary>has_custom_titlebar</summary></br>
  A boolean that tells wwm whether the window need special handling
</br></br></details>

<details>
  <summary>workspace</summary></br>
  An integer between 1 and 10 that tells wwm in which workspace to put the window.
</br></br></details>

<details>
  <summary>manage</summary></br>
  A boolean that tells wwm whether to manage the matched window. </br>
  This overrides the logic that normally decides whether to manage the window.
</br></br></details>

<details>
  <summary>firefox</summary></br>
  A boolean that tells wwm whether the matched window is based on firefox </br>
  Firefox doesn't follow the windows 10 standard and kind of does it's own thing. (I know it is stupid)
  WWM will do some specific things for firefox when this is enabled.
  Firefox is one of two programs that I found while developing WWM, where the UI just did whatever it wanted.
</br></br></details>

<details>
  <summary>chromium</summary></br>
  A boolean that tells wwm whether the matched window is based on chromium </br>
  The same thing as with firefox.
</br></br></details>

#### Examples

Firefox
```yaml
pattern: ^.*- Mozilla Firefox|Mozilla Firefox$
has_custom_titlebar: true
firefox: true
```

Google Chrome
```yaml
pattern: ^.*- Google Chrome$
has_custom_titlebar: true
chromium: true
```

Microsoft Edge (Chromium version)
```yaml
pattern: ^.*- Microsoft Edge$
has_custom_titlebar: true
chromium: true
```

### Keybindings

Each keybinding has to have a type, key and maybe additional settings which can be looked up for each type specifically.

Keybindings can have the following keys:

* Enter
* Plus
* Minus
* A
* B
* C
* D
* E
* F
* G
* H
* I
* J
* K
* L
* M
* N
* O
* P
* Q
* R
* S
* T
* U
* V
* W
* X
* Y
* Z
* Left
* Up
* Right 
* Down
* 0
* 1
* 2
* 3
* 4
* 5
* 6
* 7
* 8
* 9

Keybindings can have the following modifiers:

* Alt
* Control
* Shift

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
type: CloseTile
key: Control+Alt+Q
```

A CloseTile keybinding closes the currently focused tile and its window.

#### Quit

example
```yaml
type: Quit
key: Control+Alt+X
```

A Quit keybinding closes wwm and unmanages each window.

#### ToggleFloatingMode

example
```yaml
type: ToggleFloatingMode
key: Control+Alt+F
```

A ToggleFloatingMode keybinding either manages the currently focused window if it is not already managed or unmanages it.

#### Focus

example
```yaml
type: Focus
key: Alt+H
direction: Left
```

values
* Left
* Right
* Up
* Down

A Focus keybinding takes a direction, specifying which window gets the focus.

#### Swap

example
```yaml
type: Swap
key: Control+Alt+H
direction: Left
```

values
* Left
* Right
* Up
* Down

A Swap keybinding takes a direction, specifying which window gets swapped with the current one.

#### Split

example
```yaml
type: Split
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

## Screenshots

### default

![Screenshot 4](/screen-4.png?raw=true "Screenshot 4")

### display_app_bar

![Screenshot 3](/screen-3.png?raw=true "Screenshot 2")

### display_app_bar + remove_title_bar

![Screenshot 2](/screen-2.png?raw=true "Screenshot 3")

### display_app_bar + remove_title_bar + remove_task_bar

![Screenshot 1](/screen-1.png?raw=true "Screenshot 1")

## Development

### Create installer

```
cargo build --release
./rcedit ./target/release/wwm.exe --set-icon ./logo.ico
cargo wix init --force
cargo wix --bin-path "C:\Program Files (x86)\WiX Toolset v3.11\bin" --no-build
```
