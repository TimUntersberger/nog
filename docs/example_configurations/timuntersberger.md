# timuntersberger

## config.nog

```nog
import "nog/components" as C;

execute "modes/resize";
execute "keybindings";
execute "rules";

bar #{
    font: "CaskaydiaCove NF",
    font_size: 18,
    components: #{
        left: [C::workspaces()],
        center: [C::time("%T")],
        right: [C::active_mode(), C::padding(5), C::date("%e %b %Y"), C::padding(1)]
    }
};

workspace 1 #{
    text: "  "
};

workspace 2 #{
    text: "  "
};

workspace 3 #{
    text: " 阮 "
};

workspace 4 #{
    text: " ﭮ "
};

set min_height 200;
set min_width 200;

update_channel "test" #{
    branch: "development"
};

set default_update_channel "test";

enable work_mode;
enable use_border;
enable launch_on_startup;
enable display_app_bar;
disable multi_monitor;
enable remove_title_bar;
enable remove_task_bar;
```

## keybindings.nog

```nog
bind "Alt+Enter" launch("wt.exe");
bind "Alt+B" launch("C:\\Program Files\\Mozilla Firefox\\firefox.exe");

bind "Alt+Q" close_tile();
bind "Alt+M" minimize_tile();
bind "Alt+X" quit();

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
```

## rules.nog

```nog
ignore "explorer.exe";
ignore "Taskmgr.exe";
ignore "SnippingTool.exe";

rule "firefox.exe" #{
    has_custom_titlebar: true,
    workspace_id: 2,
    firefox: true
};

rule "Discord.exe" #{
    has_custom_titlebar: true
};

rule "Spotify.exe" #{
    has_custom_titlebar: true
};

rule "chrome.exe" #{
    has_custom_titlebar: true,
    workspace_id: 2,
    chromium: true
};

rule "Code.exe" #{
    has_custom_titlebar: true
};
```

## modes

### resize.nog

```nog
mode "resize" "Alt+R" {
    bind "H" resize("Left", 2);
    bind "Shift+H" resize("Left", -2);
   
    bind "J" resize("Down", 2);
    bind "Shift+J" resize("Down", -2);

    bind "K" resize("Up", 2);
    bind "Shift+K" resize("Up", -2);

    bind "L" resize("Right", 2);
    bind "Shift+L" resize("Right", -2);

    bind "C" reset_column();
    bind "R" reset_row();
}
```