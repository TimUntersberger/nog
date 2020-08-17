# Functions

You can define a new function `add` in the following way

```nog
fn add(x, y){
  return x + y;
}
```

## Implicit return

Because the last expression in a function gets automatically returned you can also write it like this

```nog
fn add(x, y) {
  x + y;
}
```

## No external scope

One thing that is important to know, is that you can't access the external scope inside a function

```nog
let x = 10;
fn add(y) {
  x + y; //syntax error
}
```

## Pass by value

Every parameter passed to a function is passed by value, meaning you can't mutate any arguments.

```nog
fn add(x, y) {
  x += y;
}

let x = 1;
let y = 1;

add(x, y);

x == 1;
```

## Methods

You can also use functions as methods. When calling functions with `variable.function()` the `this` gets bound to the `variable` and you can mutate `this`.

```nog
fn add(y) {
  this += y;
}
let x = 1;
let y = 1;
x.add(y);
x == 2;
```

## Overloading

You can overload functions by arity.

```nog
fn add(y){
  this += y;
}
fn add(y, z){
  this += y + z;
}

2.add(2); // works
2.add(2, 2); // works

function add(y){
  // does nothing
}

// the original add function is now replaced with the new one

2.add(2); // works, but does nothing
2.add(2, 2); // works
```

## Function Pointers

It is possible to store a function pointer in a variable.

```nog
fn add(x, y){
  x + y;
}

let f = Fn("add");
```

You can then call the function like this.

```nog
f.call(2, 2) == 4;
```

**Note**: You can only create a function pointer to a global function, meaning a function pointer to a function imported from a module is not possible.

If you want to bind the this with a function pointer you can do the following.

```nog
fn add(y) {
  this += y;
}

let adder = Fn("add");

2.call(adder, 2) == 4;
```

## Anonymous functions

You can also define anonymous functions.

```nog
let add = |x, y| {
  x + y;
}

add(2, 2);
```