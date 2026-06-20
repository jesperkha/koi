# Structs

Structs let you group related values together under a named type.

## Declaring a struct

Fields are listed one per line inside curly braces. Each field has a name and a type.

```
struct Point {
    x int
    y int
}
```

Structs can be exported from a module with `pub`:

```
pub struct Point {
    x int
    y int
}
```

## Creating instances

Use a struct literal to create an instance. All fields must be provided.

```
p := Point{x: 3, y: 5}
```

Fields can be listed in any order, and multi-line literals are allowed:

```
p := Point{
    x: 3,
    y: 5,
}
```

## Accessing fields

Use `.` to read a field from a struct value:

```
func distance_x(a Point, b Point) int {
    return b.x - a.x
}
```

## Structs as parameters and return values

Structs are passed and returned by value:

```
struct Rect {
    w int
    h int
}

func area(r Rect) int {
    return r.w * r.h
}

func main() int {
    r := Rect{w: 4, h: 6}
    return area(r)
}
```

## Structural compatibility

Two structs with the same field names and types are structurally compatible and can be used interchangeably:

```
struct Foo {
    n int
}

struct Bar {
    n int
}

func takes_foo(f Foo) int {
    return f.n
}

func main() int {
    b := Bar{n: 42}
    return takes_foo(b) // ok: Bar and Foo have the same fields
}
```

## Imported struct types

When creating a struct literal from an imported module, prefix the type name with the module name:

```
import shapes

func main() int {
    p := shapes.Point{x: 1, y: 2}
    return p.x
}
```
