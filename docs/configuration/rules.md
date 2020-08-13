# Rules

Rules are used to define specific properties on a window like for example whether the window uses the native titlebar or a custom one.

You can define a new rule by using the `rule` keyword.

```nog
rule "<pattern>" #{
    //flags
};
```

**Note**: The pattern can either match the process name (e.g. firefox.exe) or a window title

This keyword requires two arguments. The first one has to be a String which contains a [Regex]() which is used to know when to apply this rule. The second argument is a [map]() which can contain the following properties

| Key                 | Value   | Description                                           |
|---------------------|---------|-------------------------------------------------------|
| has_custom_titlebar | Boolean | Uses a custom titelbar                                |
| workspace_id        | Number  | To which workspace the window gets moved when managed |
| manage              | Boolean | Ignore this window                                    |
| firefox             | Boolean | Needs firefox specific handling                       |
| chromium            | Boolean | Needs chromium specific handling                      |

Thankfully there are currently only a few applications that need a lot of specific changes which are already included. You only need to tell Nog which window belongs to this application. There currently exist two flags that are basically a collection of different flags:

* firefox
* chromium

**Note**: You also have to set chromium to true if you use a browser that uses chromium as their base, like the new `Microsoft Edge`

## Examples

### Firefox

```nog
rule ".*- Mozilla Firefox|Mozilla Firefox" #{
  has_custom_titlebar: true,
  firefox: true
};
```

### Google Chrome

```nog
rule ".*- Google Chrome" #{
  has_custom_titlebar: true,
  chromium: true
};
```

### Visual Studio Code

```nog
rule ".*- Visual Studio Code" #{
    has_custom_titlebar: true
};
```

## Shortcuts

This section is a list of shortcuts for common patterns.

### Ignore

Ignore this window

#### Usage

```nog
ignore "<pattern>";
```

#### Rule equivalent

```nog
rule "<pattern>" #{
    manage: false
};
```