# Rules

Rules are used to add special handling to windows that match a regex pattern.

```lua
nog.rules = {
  ["notepad.exe"] = {
    ...
  }
}
```

The above snippet defines a rule for any window that either has a title or executable name matching `notepade.exe`.

A rule can contain the following settings

| Key                       | Value   | Description                                                                   |
|---------------------------|---------|-------------------------------------------------------------------------------|
| has_custom_titlebar       | Boolean | Changes how we align the window (applications like [vscode](https://code.visualstudio.com/) should have this enabled) |
| action                    | String  | Tells nog what to do when the rule matches. Can be either ignore, validate or manage |
| chromium                  | Boolean | Adds chromium specific handling (anything based on chromium like the new microsoft edge should have this enabled) |
| firefox                   | Boolean | Adds firefx specific handling |
| workspace_id              | Number  | Which workspace this window gets moved to |

The default config contains a few useful rules if you want to see them in action.
