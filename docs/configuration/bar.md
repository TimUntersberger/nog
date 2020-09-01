# Bar

![Bar](../_media/bar.png)

The bar is where the most important information about the current state can be found.

You can configure the bar by using the `bar` keyword.

```nog
bar #{
    //configuration
};
```

The bar keyword takes an [object](scripting/types?id=object) which can contain the following properties:

| Key        | Value  | Description                                                          |
|------------|--------|----------------------------------------------------------------------|
| height     | Number | The height of the bar                                                |
| font       | String | The font of the bar                                                  |
| font_size  | Number | The font size of the bar                                             |
| color      | Number | The base color of the bar of which the other colors get derived from |
| components | Map    | A map of the used components (More information is below)             |

It is designed to be completely modular, meaning each "section" you can see in the image at the top of this page is a seperate component (e.g. time).

## Example

```nog
bar #{
    font: "CaskaydiaCove NF",
    font_size: 18,
    components: #{
        left: [C::workspaces()],
        center: [C::time("%T")],
        right: [C::active_mode(), C::padding(5), C::date("%e %b %Y"), C::padding(1)]
    }
};
```

## Components

To use the components provided by default you first have to import them.

```nog
import "nog/components" as C;
```

Defining the components used by the bar can be done by setting `components` which is an [object](scripting/types?id=object) that can contain the following properties

| Key    | Value | Description                      |
|--------|-------|----------------------------------|
| left   | Array | Aligned to the left of the bar   |
| center | Array | Aligned to the center of the bar |
| right  | Array | Aligned to the right of the bar  |

Each property takes an [array](scripting/types?id=array) of components.

```nog
#{
    left: [C::workspaces()],
    center: [C::time("%T")],
    right: [C::active_mode(), C::padding(5), C::date("%e %b %Y"), C::padding(1)]
}
```

Below is the documentation for each component, where it is assumed that the components are imported already as `C`;

### Time

![TimeComponent](../_media/components/time.png)

Displays the current time.

#### Arguments

| Position | Value  | Description                                                       |
|----------|--------|-------------------------------------------------------------------|
| 1        | String | A chrono pattern that specifices how the time should be displayed |

#### Usage

```nog
let component = C::time("%T");
```

### Date

![DateComponent](../_media/components/date.png)

Displays the current date.

#### Arguments

| Position | Value  | Description                                                       |
|----------|--------|-------------------------------------------------------------------|
| 1        | String | A chrono pattern that specifices how the date should be displayed |

#### Usage

```nog
let component = C::date("%e %b %Y");
```

### Current Mode

![CurrentModeComponent](../_media/components/current_mode.png)

Displays either the current mode or nothing.

#### Arguments

| Position | Value  | Description                                                       |
|----------|--------|-------------------------------------------------------------------|

#### Usage

```nog
let component = C::current_mode();
```

### Workspaces

![WorkspacesComponent](../_media/components/workspaces.png)

Displays the active workspaces for the monitor this bar resides on.

#### Arguments

| Position | Value  | Description                                                       |
|----------|--------|-------------------------------------------------------------------|

#### Usage

```nog
let component = C::workspaces();
```

### Padding

![PaddingComponent](../_media/components/padding.png)

Displays spaces.

#### Arguments

| Position | Value  | Description                    |
|----------|--------|--------------------------------|
| 1        | Number | The number of displayed spaces |

#### Usage

```nog
let component = C::padding(5);
```

### Current Window

![CurrentWindow](../_media/components/current_window.png)

Displays the title of the focused window in the current workspace.

#### Arguments

| Position | Value  | Description                    |
|----------|--------|--------------------------------|

#### Usage

```nog
let component = C::current_window();
```

### Custom

You can create custom components by using the `create` function provided in the `nog/components` module.

A custom component requires at least two things, a name and a function that tells nog how to render the component. Anything else is optional and can be configured in the optional object.

#### Arguments

| Position | Value    | Description                             |
|----------|----------|-----------------------------------------|
| 1        | String   | Name of the component                   |
| 2        | Function | The function that renders the component |
| 3        | Object   | (Optional) Additional configuration     |

#### Usage

```nog
let component = C::create("My Component", || {
    ["Hello World"]
});
```
