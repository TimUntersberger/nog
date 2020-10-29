# Exec

The `exec` keyword makes it very easy to automate a workflow.

Let's say you want have a keybinding that launches firefox twice.

```nog
bind "<keybinding>" callback(|| {
  exec launch("firefox.exe"); //execute the launch command
})
```

As you probably already noticed, the `exec` keyword takes a keybinding type and does whatever happens when you use a keybinding that is bound to this type.
