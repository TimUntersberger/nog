execute "modes/resize";
execute "keybindings";
execute "rules";

bar #{
    font: "Cascadia Mono",
    font_size: 17
};

workspace 1 #{
    text: "  "
};

workspace 2 #{
    text: "  "
};

set min_height 200;
set min_width 200;

enable multi_monitor;
enable work_mode;
enable use_border;
enable launch_on_startup;
enable display_app_bar;
enable remove_title_bar;
enable remove_task_bar;