mode "resize" "Alt+R" {
    bind "H" resize("Left", 2);
    bind "Shift+H" resize("Left", -2);
   
    bind "J" resize("Down", 2);
    bind "Shift+J" resize("Down", -2);

    bind "K" resize("Up", 2);
    bind "Shift+K" resize("Up", -2);

    bind "L" resize("Right", 2);
    bind "Shift+L" resize("Right", -2);

    bind "A" focus("Left");
    bind "W" focus("Up");
    bind "D" focus("Right");
    bind "S" focus("Down");
}