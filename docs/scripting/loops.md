# Loops

## while

Just a simple while loop like in C.

```nog
let i = 0;
let len = 10;
while i < len {
    print(i);
    i += 1;
}
```

## loop

Syntactic sugar for infinite loops.

```nog
loop {
    print("I am infinite");
}
```

This could also be written as

```nog
while true {
    print("I am infinite");
}
```

## for

A basic `for-in` loop.

```nog
let str = "Hello world";

for c in str {
    print(c);
}
```

The for loop can iterate through strings and arrays.

For convenience there exists a `range` function.

```nog
for i in range(0, 50) { // range(inclusive, exclusive)
    print(i);
}
```