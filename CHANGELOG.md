## 2021/10/07 

* Removed `ignore` property of rules in favor of a `action`.

```lua
{
  ignore = true
}

-- is equivalent to

{
  action = "ignore"
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
