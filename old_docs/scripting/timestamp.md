# Timestamps

If you want to time something you can use the `timestamp()` function.

This function returns a `timestamp` that can be used to measure the elapsed time.

```nog
let now = timestamp();
expensive_operation();
print("operation took " + now.elapsed + "ms");
```