# Classes

Explaining classes in nogscript is easier by just showing an example.

```nogscript
class Point {
  var x = 0
  var y = 0

  var info // = null

  fn inc_both() {
    this.x++
    this.y++
  }

  static fn new(x, y) {
    return Point {
      x: x,
      y: y
    }
  }
}

var point

point = Point.new(1, 1) // call static function
point = Point {} // uses defaults
point = Point { x: 1 } // override only x

point.inc_both() // call method
```
