# atomic

Wraps the given value in an object.

## Signature

```nogscript
fn atomic(value: Any) -> AtomicValue
```

## Example

```nogscript
let count = atomic(0)

count.value++

print(count)
```

Output

```
AtomicValue {
  value: 1
}
```
