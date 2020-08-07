
# wwm
Windows window manager is a tiling window manager for Windows 10 (like i3 for linux)

![Preview](/screen-5.png?raw=true "Preview")

Terminal: Windows Terminal | Colourscheme: Nord

[Demo](https://gfycat.com/glisteningbrighthound)

[Demo with GUI](https://gfycat.com/differentadorablekiskadee)

## Table Of Contents
  - [Motivation](#motivation)
  - [Config](#config)
     - [Gap](#gap)
     - [Bar](#bar)
     - [Toggles](#toggles)
     - [Workspaces](#workspaces)
     - [Rules](#rules)
        - [Settings](#settings)
        - [Examples](#examples)
     - [Keybindings](#keybindings)
     - [Modes](#modes)
  - [Screenshots](#screenshots)
  - [Development](#development)
     - [Create installer](#create-installer)
     - [Create TOC](#create-toc)
     
## Motivation

In the beginning i always had a Virtual Machine with i3 for my development purposes. Because I am lazy I always dreaded having to start a Virtual Machine just for developing something and evolved my setup into running i3 in WSL and showing it on Windows using an X-Server. This didn't really feel right, so after some time I just decided to live without a TWM on Windows. After seeing FancyZones of Microsoft I saw the potential for a native TWM on Windows 10, but sadly FancyZones is not the Window Manager I want nore want to deal with. That's why decided to create my own TWM and here we are. 
     
## Config

The config lives in `C:\Users\<User>\AppData\Roaming\wwm\config.rhai`.

If you want to see what a config looks like you can look inside the `example` folder.

### General

You can specifiy the minimum width/height required for wwm to automatically manage a window. This can be useful when you want to avoid managing small popups that have a dynamic title.

The `min_width` setting defines the minimum width.

The `min_height` setting defines the minimum height.

### Gap

WWM supports two different types of gaps, the outer and the inner gap.

The inner gap gets defined by the `padding` setting and the outer gap is the sum of the `margin` and `padding` setting.

### bar

Example
```
bar #{
  height: 20,
  font_size: 17
};
```

The `height` setting defines the height of the app bar

The `date_pattern` setting defines the [chrono](https://docs.rs/chrono/0.4.13/chrono/format/strftime/index.html#specifiers) pattern used by the appbar for the date (NOTE: the pattern has to be inclosed in quotation marks).

The `time_pattern` setting defines the [chrono](https://docs.rs/chrono/0.4.13/chrono/format/strftime/index.html#specifiers) pattern used by the appbar for the time (NOTE: the pattern has to be inclosed in quotation marks).

The `font` setting defines the font for the appbar widgets.

The `font_size` setting defines the font size for the appbar widgets.

A color has to be a valid hex value (e.g 0x27242c)

The `bg` setting defines the background color of the appbar

### Toggles

The `launch_on_startup` tells wwm whether to start automatically on startup.

The `work_mode` setting tells wwm whether to start in work mode.

The `light_theme` setting changes the way wwm generates the colors for the bar.

The `display_app_bar` setting creates a window at the top of the display that shows all currently used workspaces.

The `remove_title_bar` setting removes the windows styles responsible for giving a managed window the titlebar.

The `remove_task_bar` setting hides the taskbar on launch and shows it again when closing the program.

### Workspaces

Example
```
workspace 4 #{
  monitor: 1
};
```

The first thing you have to specifiy is the id of the workspace you want to change. Afterwards every setting is optional.

The `monitor` setting defines the default monitor of the workspace.

### Rules

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
  <summary>workspace_id</summary></br>
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

If you want WWM to just ignore a window you can specify this via a `ignore $pattern` or set the `manage` property of a rule to `false`;

#### Examples

Firefox
```
rule ".*- Mozilla Firefox|Mozilla Firefox" #{
  has_custom_titlebar: true,
  firefox: true
};
```

Google Chrome
```
rule ".*- Google Chrome" #{
  has_custom_titlebar: true,
  chromium: true
};
```

Microsoft Edge (Chromium version)
```
rule ".*- Microsoft Edge" #{
  has_custom_titlebar: true,
  chromium: true
};
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

  * [ChangeWorkspace](#changeworkspace)
  * [MoveToWorkspace](#movetoworkspace)
  * [MoveWorkspaceToMonitor](#moveworkspacetomonitor)
  * [Launch](#launch)
  * [CloseTile](#closetile)
  * [MinimizeTile](#minimizetile)
  * [Quit](#quit)
  * [ToggleFloatingMode](#togglefloatingmode)
  * [ToggleWorkMode](#toggleworkmode)
  * [ToggleFullscreen](#togglefullscreen)
  * [Focus](#focus)
  * [Swap](#swap)
  * [Split](#split)
  * [IncrementConfig](#incrementconfig)
  * [DecrementConfig](#decrementconfig)
  * [ToggleConfig](#toggleconfig)

#### ChangeWorkspace

example
```
bind "Alt+1" change_workspace(1);
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
```
bind "Control+Alt+1" move_to_workspace(1);
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

#### MoveWorkspaceToMonitor

example
```
bind "Control+Alt+1" move_workspace_to_monitor(1);
```

The monitor property can be any valid number, but the range depends on the amount of monitors connected to the computer.

A MoveWorkspaceToMonitor keybinding moves the current workspace to a different monitor

#### Launch

example
```
bind "Alt+Enter" launch("wt.exe");
```

A Launch keybinding takes a cmd, which has to be a valid path to an exe file.

If the exe can be found in the path, then the name is enough (e.g. `wt.exe`).

#### CloseTile

example
```
bind "Alt+Q" close_tile();
```

A CloseTile keybinding closes the currently focused tile and its window.

#### MinimizeTile

example
```
bind "Alt+Q" minimize_tile();
```

A MinimizeTile keybinding closes the currently focused tile and minimizes its window.

#### Quit

example
```
bind "Alt+X" quit();
```

A Quit keybinding closes wwm and unmanages each window.

#### ToggleFloatingMode

example
```
bind "Control+Alt+F" toggle_floating_mode();
```

A ToggleFloatingMode keybinding either manages the currently focused window if it is not already managed or unmanages it.

#### ToggleWorkMode

example
```
bind "Control+Alt+W" toggle_work_mode();
```

A ToggleWorkMode keybinding can be seen as "starting" and "stopping" wwm. Wwm is not really stopped it just makes wwm take the least amout of resources while still listening only ToggleWorkMode keybindings.

#### ToggleFullscreen

example
```
bind "Control+Alt+F" toggle_fullscreen();
```

A ToggleFullscreen keybinding enables/disables fullscreen mode of the current workspace. A workspace in fullscreen mode only shows the focused tile, but you are still able to change the focused tile via Focus/Swap keybindings.


#### Focus

example
```
bind "Alt+H" focus("Left");
```

values
* Left
* Right
* Up
* Down

A Focus keybinding takes a direction, specifying which window gets the focus.

#### Swap

example
```
bind "Alt+Shift+H" swap("Left");
```

values
* Left
* Right
* Up
* Down

A Swap keybinding takes a direction, specifying which window gets swapped with the current one.

#### Split

example
```
bind "Alt+Split" split("Vertical");
```

values
* Horizontal
* Vertical

A Split keybinding takes a direction, the new SplitDirection of the currently focused window. The SplitDirection specifies how a new window gets placed in the grid.

#### IncrementConfig

example
```
bind "Alt+O" increment_config("app_bar_height", 5);
```

A IncrementConfig keybinding can be used to bind a keybinding to increment a config value by a specific amount. Only works on config values that are numeric. Examples include app_bar_height, app_bar_bg, app_bar_font_size, margin, padding.

#### DecrementConfig

example
```
bind "Alt+Shift+O" decrement_config("app_bar_height", 5);
```

A DecrementConfig keybinding can be used to bind a keybinding to decrement a config value by a specific amount. Only works on config values that are numeric. Examples include app_bar_height, app_bar_bg, app_bar_font_size, margin, padding.

#### ToggleConfig

example
```
bind "Alt+I" toggle_config("display_app_bar");
```

A ToggleConfig keybinding can be used to bind a keybinding to toggle a config value. Only works on config values that are boolean. Examples include use_border, light_theme, remove_title_bar, remove_task_bar, display_app_bar.

### Modes

example
```
mode "resize" "Alt+R" {
    bind "H" resize("Left", 2);
    bind "Shift+H" resize("Left", -2);
   
    bind "J" resize("Down", 2);
    bind "Shift+J" resize("Down", -2);

    bind "K" resize("Up", 2);
    bind "Shift+K" resize("Up", -2);

    bind "L" resize("Right", 2);
    bind "Shift+L" resize("Right", -2);
}
```

The example defines a new mode called `resize`. You can enter/leave the mode mode by typing `Alt+R`. When you enter a mode every keybinding that doesn't get defined by the mode itself gets ignored.

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
