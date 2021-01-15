# Modes

A mode in Nog is a self contained state, where all of the keybindings get unregistered until you leave the mode again.

It is important to use the `bind` function passed as an argument instead of the global one when registering mode specific keybindings.

Keybindings that are defined with the `xbind` function won't get unregistered when entering/leaving a mode.

You can define a new mode by calling the [nog.mode]() function.

To enter/leave a mode you have to call the [nog.enter_mode]()/[nog.leave_mode]() function.

# Example

```nogscript
import nog

nog.mode("test", bind => {
  bind("F1", () => print("Hello World!"))
})
```
