# Bar

The bar at the top of the screen when having `display_app_bar` enabled can display a lot of useful information.

It is possible to change the following settings

| Key                       | Value   | Description                                                                   |
|---------------------------|---------|-------------------------------------------------------------------------------|
| font       | String | The font of the bar |
| font_size                    | Number | The font size of the bar|
| color                  | Number | The color of the bar |
| components                   | Table | The component layout of the bar |

```lua
nog.config.bar.font_size = 20
```

## Components

Components are the building blocks of the bar. The `components` table has the following properties:

* `left` contains a list of components which are aligned to the left side of the bar
* `center` contains a list of components which are aligned to the center of the bar
* `right` contains a list of components which are aligned to the right side of the bar

```lua
nog.config.bar.components = {
  left = {},
  center = { nog.components.workspaces() },
  right = {}
}
```

The above snippet changes the layout so that the left side and right side are empty and the center part gets replaced by the workspaces overview.

### Builtin

All of the builtin components can be found under `nog.components`.

* datetime
* workspaces
* current_window
* padding
* fullscreen_indicator
* split_direction
* current_mode

### Custom

A bar component is a table which has to have a `name` and `render` field.

The `name` can be any `string` of your chosing and is only used for debugging purposes.

`render` is the important part. It has to be a function which returns a list of components texts.
A component text is a `table` with a required `text` field and optional `fg` and `bg` fields 
which take a `number` and change the colors of the component. 
The function also receives the current display id as argument 
so you can know which display the component is currently being rendered on.

It is common practice to define a component as a function to make it easy to add customization options later on.

```lua
local hello = function(name)
  return {
    name = "my_component",
    render = function()
      return {{ text = "Hello " .. name .. "!" }}
    end
  }
end

nog.config.bar.components = {
  left = {},
  center = { hello("User" },
  right = {}
}
```
