# Modules

Every file is its own module. A module has its own scope and only the thing that you want to expose get exposed, except a few exceptions:

* modules you import in the file
* functions

If you want to not expose a function you can use the `private` keyword.

```nog
private fn f(){
  //...
}
```

## Importing

You can import a module by using the `import` keyword.

```nog
import "nog/components" as C;
```

The `import` keyword can be used to import modules of nog and modules you have written yourself.

The path gets either resolved to a module provided by nog or a module relative to `%APPDATA%\nog`.

**Note**: "nog/**" is reserved for nog

```nog
import "my/module" as M; // %APPDATA%\nog\my\module.nog
```

You can also import a module without using the `as` keyword if you don't care about the exported values.

```nog
import "script";
```

**WARNING**: Recursive imports CAN crash the applications very fast.

## Exporting

Exported variables are always readonly.

You can export variables via the `export` keyword.

```nog
let x = 0;
export x;
```

You can also rename the export;

```nog
let x = 0;
export x as y;
```

## Builtin

* [nog/components]()
* [WIP nog/http]()
* [WIP nog/popup]()