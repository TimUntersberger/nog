# Workspaces

Nog has a total of 10 different workspaces and you can customize each workspace however you want seperately.

To start customizing a workspace you can use the `workspace` keyword.

```nog
workspace <id> #{
    //settings
}
```

This keyword requires two arguments. The first one is the id of the workspace you want to customize and the second argument is a [map]() which can contain the following properties

**Note**: The monitor ids are counted from left to right and from top to bottom

| Key     | Value  | Description                                             |
|---------|--------|---------------------------------------------------------|
| monitor | Number | Id of the monitor this workspace resides on per default |
| text    | String | Text to display instead of the id (can be unicode)      |

## Example

```nog
workspace 1 #{
    monitor: 1,
    text: " code "
};
```