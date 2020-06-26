
# wwm
Windows window manager is a tiling window manager for Windows 10 (like i3 for linux)

![Preview](/screen-1.png?raw=true "Preview")

Terminal: Windows Terminal | Colourscheme: Nord

[Demo](https://gfycat.com/glisteningbrighthound)

[Demo with GUI](https://gfycat.com/differentadorablekiskadee)

## Table Of Contents

  * [Motivation](#motivation)
  * [Info](#info)
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
     
## Motivation

In the beginning i always had a Virtual Machine with i3 for my development purposes. Because I am lazy I always dreaded having to start a Virtual Machine just for developing something and evolved my setup into running i3 in WSL and showing it on Windows using an X-Server. This didn't really feel right, so after some time I just decided to live without a TWM on Windows. After seeing FancyZones of Microsoft I saw the potential for a native TWM on Windows 10, but sadly FancyZones is not the Window Manager I want nore want to deal with. That's why decided to create my own TWM and here we are. 

## Info

WWM does currently not handle windows that only hide when "closing" them.

An example is powershell. When you type exit inside the console it doesn't close the window so I don't know how to detect that.
     
## Config

The config lives in `C:\Users\<User>\AppData\Roaming\wwm\config.yaml`.

If you want to see my own config or just want to know what a config looks like: [my config](#example-config).

### Gap

WWM supports two different types of gaps, the outer and the inner gap.

The inner gap gets defined by the `padding` setting and the outer gap is the sum of the `margin` and `padding` setting.

### bar

The `app_bar_height` setting defines the height of the app bar

The `app_bar_font` setting defines the font for the appbar widgets.

The `app_bar_font_size` setting defines the font size for the appbar widgets.

A color has to be a valid hex value (e.g 0x27242c)

The `app_bar_bg` setting defines the background color of the appbar

The `app_bar_workspace_bg` setting defines the background color of a workspace in the background

### Toggles

The `launch_on_startup` tells wwm whether to start automatically on startup.

The `work_mode` setting tells wwm whether to start in work mode.

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

Note: The windows modifier is reserved for windows itself so we can't use it in custom keybindings.

Keybindings can have the following types:

* [Launch](#launch)
* [CloseTile](#closetile)
* [Quit](#quit)
* [Focus](#focus)
* [Split](#split)
* [Swap](#swap)
* [ToggleFloatingMode](#togglefloatingmode)
* [ToggleWorkMode](#toggleworkmode)
* [ChangeWorkspace](#changeworkspace)
* [MoveToWorkspace](#movetoworkspace)

#### ChangeWorkspace

example
```yaml
type: ChangeWorkspace
key: Alt+1
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

#### MoveToWorkspace

example
```yaml
type: MoveToWorkspace
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

A MoveToWorkspace keybinding takes an id, which is the id of the workspace to move the focused tile to.

Workspaces have an upper limit of 10, so if you define a keybinding of type ChangeWorkspace that goes above 10 or below 1 the program *currently* just crashes.

#### Launch

example
```yaml
type: Shell
key: Control+Alt+Enter
cmd: wt.exe
```

A Launch keybinding takes a cmd, which has to be a valid path to an exe file.

If the exe can be found in the path, then the name is enough (e.g. `wt.exe`).

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

#### ToggleWorkMode

example
```yaml
type: ToggleWorkMode
key: Control+Alt+W
```

A ToggleWorkMode keybinding can be seen as "starting" and "stopping" wwm. Wwm is not really stopped it just makes wwm take the least amout of resources while still listening only ToggleWorkMode keybindings.

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
app_bar_font: Cascadia Mono
app_bar_font_size: 17
app_bar_bg: 0x3b4252

work_mode: false
launch_on_startup: true
display_app_bar: true
remove_title_bar: true
remove_task_bar: true

rules:
  - pattern: ^File Explorer$
    manage: false
  - pattern: ^.*- Mozilla Firefox|Mozilla Firefox$
    workspace: 2
    has_custom_titlebar: true
    firefox: true
  - pattern: ^.*- Google Chrome$
    has_custom_titlebar: true
    chromium: true
  - pattern: ^(.*- (Visual Studio Code|Discord)|Spotify Premium|Discord)$
    has_custom_titlebar: true

keybindings:
  - type: Launch
    key: Alt+Enter
    cmd: wt.exe
  - type: Launch
    key: Alt+B
    cmd: C:\\Program Files\\Mozilla Firefox\\firefox.exe

  - type: CloseTile
    key: Alt+Q

  - type: Quit
    key: Alt+X

  - type: Focus
    key: Alt+H
    direction: Left
  - type: Focus
    key: Alt+J
    direction: Down
  - type: Focus
    key: Alt+K
    direction: Up
  - type: Focus
    key: Alt+L
    direction: Right

  - type: Swap
    key: Alt+Control+H
    direction: Left
  - type: Swap
    key: Alt+Control+J
    direction: Down
  - type: Swap
    key: Alt+Control+K
    direction: Up
  - type: Swap
    key: Alt+Control+L
    direction: Right

  - type: Split
    key: Alt+Plus
    direction: Vertical
  - type: Split
    key: Alt+Minus
    direction: Horizontal

  - type: ToggleFloatingMode
    key: Alt+F
  - type: ToggleWorkMode
    key: Alt+Control+W

  - type: MoveToWorkspace
    key: Alt+Control+1
    id: 1
  - type: MoveToWorkspace
    key: Alt+Control+2
    id: 2
  - type: MoveToWorkspace
    key: Alt+Control+3
    id: 3
  - type: MoveToWorkspace
    key: Alt+Control+4
    id: 4

  - type: ChangeWorkspace
    key: Alt+1
    id: 1
  - type: ChangeWorkspace
    key: Alt+2
    id: 2
  - type: ChangeWorkspace
    key: Alt+3
    id: 3
  - type: ChangeWorkspace
    key: Alt+4
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
