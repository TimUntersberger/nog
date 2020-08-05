bind "Alt+Enter" to launch("wt.exe");
bind "Alt+B" to launch("C:\\Program Files\\Mozilla Firefox\\firefox.exe");

bind "Alt+Q" to close_tile();
bind "Alt+M" to minimize_tile();

bind "Alt+H" to focus("Left");
bind "Alt+J" to focus("Down");
bind "Alt+K" to focus("Up");
bind "Alt+L" to focus("Right");

bind "Alt+Control+H" to swap("Left");
bind "Alt+Control+J" to swap("Down");
bind "Alt+Control+K" to swap("Up");
bind "Alt+Control+L" to swap("Right");

bind "Alt+Plus" to split("Vertical");
bind "Alt+Minus" to split("Horizontal");

bind "Alt+Control+F" to toggle_floating_mode();
bind "Alt+Control+W" to toggle_work_mode();
bind "Alt+F" to toggle_fullscreen();

for i in range(1, 10) {
    let key = "Alt+Shift" + i;
    bind key to move_to_workspace(i);
}

for i in range(1, 10) {
    let key = "Alt+Control" + i;
    bind key to move_workspace_to_monitor(i);
}

for i in range(1, 10) {
    let key = "Alt+" + i;
    bind key to change_workspace(i);
}