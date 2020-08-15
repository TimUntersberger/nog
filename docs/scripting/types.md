# Types

The language is dynamic in nature, but it has types like javascript.

## Number

Numbers can either be integers or floats. Writing a number with a dot like `1.0` makes it a float.

```nog
100 // i32
100.0 // f64
```

When writing a number `_` can be used to seperate units. This doesn't have an effect on the outcome.

```nog
2300 // 2300
2_300 // 2300
```

### Decimal

```nog
1234562
```

### Octal

```nog
0o07762
```

### Hex

```nog
0xabcdef
```

### Binary

```nog
0b010110012
```

### Functions

| name        | description                                                     |
| ----------- | --------------------------------------------------------------- |
| abs         | returns the absolute value of a number                          |
| sign        | returns -1 if the number is negative, +1 if positive, 0 if zero |
| to_float    | converts an integer into a float                                |
| to_int      | converts a float into an integer                                |
| is_nan      | is not a number                                                 |
| is_finite   | is a finite number                                              |
| is_infinite | is an infinite number                                           |

### Math

* sin
* sinh
* cos
* cosh
* tan
* tanh
* asin
* asinh
* acos
* acosh
* atan
* atanh
* sqrt
* exp (exponential)
* ln
* log10
* log
* floor
* ceiling
* round
* int
* fraction

## String

Strings are written surrounded with `"`.

```nog
"Hello World"
```

### Escape Sequences

* \\\\
* \\t
* \\r
* \\n
* \\"
* \\'

#### Unicode

* \\xXX 
* \\uXXXX 
* \\UXXXXXXXX

### Functions

| name       | description                                           |
|------------|-------------------------------------------------------|
| len        | Number of characters in a string (**NOT BYTES**)      |
| pad        | Pad the string with the specified character           |
| append     | Add the string to the end of the string               |
| clear      | Empty the string                                      |
| truncate   | Cuts off the string at the specified length           |
| contains   | Contains the string                                   |
| index_of   | Returns the index of the string (-1 if not found)     |
| sub_string | Extract a substring                                   |
| crop       | Truncate with a start index                           |
| replace    | Replace a string with another string                  |
| trim       | Remove whitespace at the front and back of the string |

## Array

Arrays are written like in javascript.

```nog
let array = [1, 2, 3, 4, 5];
```

### Functions

| name     | description                                                   |
|----------|---------------------------------------------------------------|
| push     | Appends an element to the end of the array                    |
| append   | Appends an array to the end of the array                      |
| insert   | Inserts an element at the specified index                     |
| pop      | Removes the last element and returns it                       |
| shift    | Removes the first element and returns it                      |
| remove   | Removes the element at an index and returns it                |
| len      | Length of the array                                           |
| pad      | Pads the array with the given element to the specified length |
| clear    | Empty the array                                               |
| truncate | Cuts off the array at the specified length                    |

## Object

Objects can be denoted by using `#{}`.

```nog
let obj = #{
    hello: "world",
};
```

The properties can be accessed via either the dot operator or the bracket syntax.

```nog
obj.hello == "world"
obj["hello"] == "world"
```

### Functions

| name      | description                                       |
|-----------|---------------------------------------------------|
| has       | Does the object contain the key                   |
| len       | Amount of properties                              |
| clear     | Empty the object                                  |
| remove    | Remove a property and return its value            |
| mixin     | Override the properties with a specified object   |
| fill_with | Add all missing properties of an specified object |
| keys      | Array of keys                                     |
| values    | Array of values                                   |