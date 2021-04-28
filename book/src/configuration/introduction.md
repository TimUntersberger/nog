# Introduction

The configuration lives in `%APPDATA%/nog/config`.
At startup the program looks for a `init.lua` file in the root of this folder and executes it.

Nog comes with a preconfigured `init.lua` which aims to show you how you can build your own config.
We try to explain everything as good as possible with comments in our config so you shouldn't have to look anything up.
If you want to know what keybindings are available to you by default please look at the `init.lua`.

The configuration itself is written in lua and tries to have a similar style to neovim. 
Any lua code executed by nog has access to the global [nog]() variable which contains a bunch of functions to make it easy to control nog programmatically.

Nog also includes [luv](https://github.com/luvit/luv) and [inspect](https://github.com/kikito/inspect.lua) to make development easier for the user.

If you ever want to look at the structure of a lua table you can easily print a very nice overview using the following statement

```lua
print(nog.inspect(tbl))
```
