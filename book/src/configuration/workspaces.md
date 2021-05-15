# Workspaces

**Note**: The monitor ids are counted from left to right and from top to bottom

</br>

Nog has `10` different workspaces. Each workspace can be customized.

When customizing a workspace you can change the following settings:

| Key     | Value  | Description                                             |
|---------|--------|---------------------------------------------------------|
| monitor | Number | Id of the monitor this workspace resides on per default |
| text    | String | Text to display instead of the id (can be unicode)      |

## Example

```lua
nog.workspaces = {
  [1] = {
    text = "First Workspace"
  },
  [2] = {
    text = "Second Workspace",
    monitor = 1
  },
}
```
