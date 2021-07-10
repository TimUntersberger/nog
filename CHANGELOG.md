* 2021/10/07 

Removed `ignore` property of rules in favor of a `action`.

```lua
{
  ignore = false
}

-- is equivalent to

{
  action = "true"
}

-- it is now possible to tell nog to always manage a matched window

{
  action = "manage"
}

-- default value

{
  action = "validate"
}
```
