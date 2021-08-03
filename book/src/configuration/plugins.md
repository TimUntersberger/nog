# Plugins

The plugins live the in the `plugins` folder located next to the `config` ([ref](../configuration/introduction.md)).

You can install plugins either manually or using the provided API. Manually installing plugins is as 
easy as cloning a valid plugin into the `plugins` folder. This works because nog uses this folder as the plugins index.
If you don't want to install the plugins manually you can use `nog.plug_install`.

```lua
nog.plug_install("GithubUser/NogPlugin")
```

This function will clone the repo if it hasn't been cloned yet.

You can update your plugins by calling `nog.plug_update`. This function will check for any new commits upstream and pull them if there are some.
Uninstalling is as simple as removing its folder or calling `nog.plug_uninstall`.

```lua
nog.plug_uninstall("GithubUser/NogPlugin")
```

If you want to get a list of all installed plugins you can use the `nog.plug_list` functions which will return a list of absolute file paths.

## Writing Plugins

Any repository that contains a top-level `lua` folder inside is a valid nog plugin. 
At startup nog will add the path to the `lua` folder to the `package.path` variable so you can `require` its content.
This also means that you will have to namespace your lua files to not cause conflicts. 
Usually you would have a folder with your plugins name underneath the `lua` folder where all of your source lives.

**Example Structure**

* README.md
* lua
  * cool_plugin
    * init.lua

You can then use the plugin by requiring `cool_plugin`.

```lua
local x = require 'cool_plugin'
```
