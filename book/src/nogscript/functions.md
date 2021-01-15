# Functions

Nogscript supports both normal functions and anonymous arrow functions.

```nogscript
fn f() {
}
```

```nogscript
var f = () => {}
```

Functions capture their environment.

```nogscript
var count = 0;

// This function captures count
var f = () => {
  print(count)
}

f() // prints 0
```

It is important to know that only some variables are passed by reference.

Primitives like `Number` and `Boolean` are passed by value which can cause some unwanted behaviour.

```nogscript
var count = 0;

// This function captures a copy of count not a reference
var inc = by => {
  count += by
}

// increments a copy of count
inc(1)
// increments a copy of count
inc(1)

// Count is still 0
print(count)
```

Objects are captured by reference so you can use a simple trick to have a reference to count.

```nogscript
var count = #{ value: 0 };

// This function captures a ref to count
var inc = by => {
  count.value += by
}

// increments the original count
inc(1)
// increments the original count
inc(1)

// Count is 2
print(count)
```

To make it easier for you to use this pattern we provide an `atomic` function in the prelude that basically does the same thing for you.

```nogscript
var count = atomic(0);

/// ... same as previous example
```
