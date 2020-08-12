# Bar

![Bar](../_media/bar.png)

The bar is where the most important information about the current state can be found.

You can configure the bar by using the `bar` keyword.

```nog
bar #{
    //configuration
};
```

The bar keyword takes a [map](../scripting/types?id=map) which can contain the following properties:

| Key       | Value  | Description                                                          |
|-----------|--------|----------------------------------------------------------------------|
| height    | Number | The height of the bar                                                |
| font      | String | The font of the bar                                                  |
| font_size | Number | The font size of the bar                                             |
| color     | Number | The base color of the bar of which the other colors get derived from |

It is designed to be completely modular, meaning each "section" you can see in the image at the top of this page is a seperate component (e.g. time).

### Components

These are the components that are provided by default. To use the components you have to import them first.

```nog
import 'nog/components' as C;
```

Every documentation below assumes that the components are imported already as `C`;

#### Time

![TimeComponent](../_media/components/time.png)

Displays the current time.

##### Arguments

| Position | Value  | Description                                                       |
|----------|--------|-------------------------------------------------------------------|
| 1        | String | A chrono pattern that specifices how the time should be displayed |

##### Usage

```nog
let component = C::time("%T");
```

#### Date

#### Current Mode

#### Workspaces

#### Padding