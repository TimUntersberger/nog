set min_height 200;
set min_width 200;

enable work_mode;
enable display_app_bar;
enable remove_title_bar;
enable remove_task_bar;

bind "Alt+Q" close_tile();
bind "Alt+M" minimize_tile();

bind "Alt+H" focus("Left");
bind "Alt+J" focus("Down");
bind "Alt+K" focus("Up");
bind "Alt+L" focus("Right");

bind "Alt+Control+H" swap("Left");
bind "Alt+Control+J" swap("Down");
bind "Alt+Control+K" swap("Up");
bind "Alt+Control+L" swap("Right");

bind "Alt+Plus" split("Vertical");
bind "Alt+Minus" split("Horizontal");

bind "Alt+Control+F" toggle_floating_mode();
bind "Alt+Control+W" toggle_work_mode();
bind "Alt+F" toggle_fullscreen();

bind_range 1 10 "Alt+Shift" move_to_workspace;
bind_range 1 10 "Alt+Control" move_workspace_to_monitor;
bind_range 1 10 "Alt" change_workspace;

ignore "File Explorer";
ignore "Task Manager";

rule ".*- Google Chrome" #{
    has_custom_titlebar: true,
    chromium: true,
};

rule ".*- Mozilla Firefox|Mozilla Firefox" #{
    has_custom_titlebar: true,
    firefox: true,
};