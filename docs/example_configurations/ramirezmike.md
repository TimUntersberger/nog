# ramirezmike

## config.nog

```nog
import "modes/resize";
import "modes/move";
import "nog/components" as C;

enable work_mode;
enable multi_monitor;
disable launch_on_startup;
enable display_app_bar;
enable remove_title_bar;
enable remove_task_bar;

// Following prevents nog from auto-managing smaller windows
set min_width 2000;
set min_height 1200;

// Prevents grid-modifying while fullscreened
set ignore_fullscreen_actions true;

bar #{
    height: 16,
    font_size: 20,
    font: "Fixedsys",
    components: #{
        left: [C::workspaces()],
        // shows focused window title in center of bar
        center: [C::current_window()], 
        right: [C::active_mode(), C::padding(5), 
                // show current split direction
                C::split_direction("-", "+"), 
                // show day month and time
                C::date("%e %b"), C::padding(1), C::time("%T"), 
                C::padding(1)]
    }
};

bind "Alt+Shift+Q" close_tile();
bind "Alt+Shift+M" minimize_tile();

bind "Alt+X" quit();

bind "Alt+W" focus("Up");
bind "Alt+A" focus("Left");
bind "Alt+S" focus("Down");
bind "Alt+D" focus("Right");

bind "Alt+Plus" split("Vertical");
bind "Alt+Minus" split("Horizontal");

bind "Alt+Shift+F" toggle_floating_mode();
bind "Alt+Control+W" toggle_work_mode();
bind "Alt+F" toggle_fullscreen();

// Bindings to toggle elements like appbar/taskbar
bind "Alt+I" toggle_config("display_app_bar");
bind "Alt+Shift+I" toggle_config("remove_task_bar");

bind_range 1 10 "Alt+Shift" move_to_workspace;
bind_range 1 10 "Alt+Control" move_workspace_to_monitor;
bind_range 1 10 "Alt" change_workspace;

bind "Alt+Shift+V" launch("C:\\Program Files (x86)\\Vim\\vim82\\gvim.exe");
bind "Alt+Shift+Enter" launch("C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe");
bind "Alt+Enter" launch("C:\\Program Files\\Git\\git-bash.exe");

ignore "File Explorer";
ignore "Task Manager";
ignore "Snipping Tool";

rule ".*- Google Chrome" #{
    has_custom_titlebar: true,
    chromium: true,
};

rule ".*- Mozilla Firefox|Mozilla Firefox" #{
    has_custom_titlebar: true,
    firefox: true,
};
```

## modes

### resize.nog

```nog
mode "resize" "Alt+R" {
    bind "W" focus("Up");
    bind "A" focus("Left");
    bind "S" focus("Down");
    bind "D" focus("Right");

    bind "Shift+H" resize("Left", -2);
    bind "Shift+J" resize("Down", -2);
    bind "Shift+K" resize("Up", -2);
    bind "Shift+L" resize("Right", -2);

    bind "H" resize("Left", 2);
    bind "J" resize("Down", 2);
    bind "K" resize("Up", 2);
    bind "L" resize("Right", 2);

    bind "R" reset_row();
    bind "C" reset_column();

    bind "P" increment_config("outer_gap", 2);
    bind "O" decrement_config("outer_gap", 2);
    bind "Shift+P" increment_config("inner_gap", 2);
    bind "Shift+O" decrement_config("inner_gap", 2);
}
```

### move.nog
```nog
mode "move" "Alt+M" {
    bind "W" focus("Up");
    bind "A" focus("Left");
    bind "S" focus("Down");
    bind "D" focus("Right");

    bind "H" swap("Left");
    bind "J" swap("Down");
    bind "K" swap("Up");
    bind "L" swap("Right");

    bind "R" swap_columns_and_rows();

    bind "Alt+H" move_in("Left");
    bind "Alt+J" move_in("Down");
    bind "Alt+K" move_in("Up");
    bind "Alt+L" move_in("Right");

    bind "Alt+Shift+H" move_out("Left");
    bind "Alt+Shift+J" move_out("Down");
    bind "Alt+Shift+K" move_out("Up");
    bind "Alt+Shift+L" move_out("Right");
}
```
