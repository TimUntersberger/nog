# Modes

Modes in nog are an easy way to condionally reuse keybindings for different actions.

Defining a new mode is very easy

```lua
nog.mode("my mode", function()
  nog.nbind("L", function()
    print("L")
  end)
  nog.nbind("Escape", function()
    nog.toggle_mode("my mode")
  end)
end)
```

It is important to not forget a binding which will leave the mode else you will probably be stuck inside the mode until you restart nog.

Now to toggle a mode all you have to do is call `nog.toggle_mode`

```lua
nog.toggle_mode("my mode")
```

When entering a mode nog will unbind all [normal](../configuration/keybindings.html) keybindings and execute the provided `cb`.
Leaving a mode will cause nog to unbind all normal keybindings again and afterwards rebind all normal keybindings that were defined previously.
