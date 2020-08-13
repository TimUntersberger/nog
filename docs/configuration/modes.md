# Modes

A mode is a self contained state of Nog, which means that every keybinding that is defined outside of the current mode gets deactivated.
This can be useful when you want to reuse key combinations that have multiple meanings based on context (e.g. resizing and focusing).

You can define a custom mode using the `mode` keyword.

```nog
mode "<name>" "<key-combo>" {
    //keybindings
}
```

This keyword requires three arguments. 

1. Name of the new mode
2. Key combination used to toggle this mode
3. A code block containing a list of statements

## Example

You want to use `Alt` + `H`/`J`/`K`/`L` for focusing and resizing. These are already used for focusing when not in a mode, so now you want to define a `resize` mode that uses these to resize the tile in a direction.

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
}
```