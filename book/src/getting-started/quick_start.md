# Quick Start

After [installing](installation.html) nog I would recommend you to do the following steps:

1. [configure](/configuration/introduction.html) nog to fit your needs
2. Execute `start-process $("$env:APPDATA\nog\bin\nog.exe")` to start nog
2. Press `Ctrl+Alt+W` to enter [work mode](work_mode.html)

When nog is started for the first time it creates an `init.lua` file in the config folder 
which contains the following configuration.

```lua
{{#include ../../../assets/default_config.lua}}
```
