# IR

### Constants

```sh
$0 = i64 123
$1 = f32 1.23
$2 = string "Hello"       # Stack - mutable
$3 = const string "World" # Data section - immutable
```

### Types

```sh
type Person
    string
    i64
end

$0 = Person
$0.0 = const string "John"
$0.1 = i64 33
```

### Functions

```sh
# Declare function signature
extern println(string, string) void

#   $0 - local variable
#   %0 - function param
fn greeting(string) void
    $0 = const string "Hello {}"
    call println($0, %0)
    ret
end

fn main() i64
    $0 = const string "John"
    call greeting($0)

    $0 = 0
    ret $0
end
```
