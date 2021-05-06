# Plugin

## plug_install(id)

Clones the github repository with the given `id` into the plugins folder. If a folder with the same name already exists, this function is noop.

**Arguments**:
* `id` [string] id of repo (ex. `TimUntersberger/nog`)

## plug_install(id)

Removes the cloned repo locally.

**Arguments**:
* `id` [string] id of repo (ex. `TimUntersberger/nog`)

## plug_update()

Checks each folder inside the plugins folder for new commits upstream and if some exist it pulls them.

## plug_list()

Returns a list of installed plugins.

**Return**: a list of absolute paths
