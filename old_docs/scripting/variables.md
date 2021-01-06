# Variables

You can declare a variable by either using the `let` or `const` keyword.

```nog
let var1 = 1;
const var2 = 2;
```

A `let` variable is mutable and a `const` variable is immutable.

Mutating a `const` variables results in an error.

## Naming

Variables follow the C naming rules:

* Must contain only ASCII letters, digits and underscores.
* Cannot start with a digit
* If the first character is an underscore the next character has to be a letter
* They cannot have the same name as an existing keyword