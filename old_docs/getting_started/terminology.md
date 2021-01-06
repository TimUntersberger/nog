# Terminology

## Work Mode

Work mode is the normal state of Nog, where every keybinding and anything else works just like expected. Once you leave work mode the application is still running in the background, but every keybinding (except [ToggleWorkMode](configuration/keybindings?id=toggleworkmode)) gets unregistered and you won't even notice that it is still running. This can be useful when you use your machine for both work and casual use, for example when gaming you may not want to have a tiling window manager running or when just browsing the web. With this feature the tiling window manager is only a keypress away instead of having to launch the program again.

## Tile

The window manager only manages tiles not windows. This is only a theoretical difference, in practice there is almost no difference between a window or tile. So if you read something regarding a tile you can just imagine a window.