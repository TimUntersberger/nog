# Settings

When Nog gets started for the first time it creates a new folder at `%APPDATA%/nog`.

This folder contains all of the machine specific configurations/plugins.

At startup the program looks for a `config.ns` file in the root of this folder and executes it if it can find one.

The config uses a custom language called `Nogscript` which is explained in more detail in the [scripting]() section.

Below you can find a list of settings you can change and their description:

| Key                       | Value   | Description                                                                   |
|---------------------------|---------|-------------------------------------------------------------------------------|
| min_height                | Number  | The minimum height a window has to have so that it gets managed automatically |
| min_width                 | Number  | The minimum width a window has to have so that it gets managed automatically  |
| inner_gap                 | Number  | The gap between each tile                                                     |
| outer_gap                 | Number  | The margin between workspace and the display                                  |
| launch_on_startup         | Boolean | Start when you start your computer                                            |
| multi_monitor             | Boolean | Use all monitors                                                              |
| work_mode                 | Boolean | Start in [work mode]()                |
| use_border                | Boolean | Force managed windows to draw a border. (This can help clarity)               |
| light_theme               | Boolean | Changes how the bar colors get generated to fit light colors                  |
| display_app_bar           | Boolean | Enable the bar                                                                |
| remove_title_bar          | Boolean | Remove the titlebar of managed windows                                        |
| remove_task_bar           | Boolean | Remove the taskbar while the program is running                               |
| ignore_fullscreen_actions | Boolean | Ignore grid-modifying keybindings (swap, focus, move, etc) while fullscreened |

All of these settings can be modified by using any of the functions in the [nog.config]() module.

## Examples

```nogscript
import nog.config

config.set("min_height", 200)
config.set("min_width", 200)
config.enable("light_theme")
```
