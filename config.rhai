bind "Alt+Enter" launch("wt.exe");
bind "Alt+B" launch("C:\\Program Files\\Mozilla Firefox\\firefox.exe");

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