# The Koi programming language

## About Koi

Koi is a statically typed, compiled language, made as an attempt to make a modern version of C. It keeps all the low level control and improves syntax and code writing ergonomics.

## Table of contents

## Basic semantics, syntax, and more

### Hello world

```go
package main

func main() int {
    println("Hello, world!")
    return 0
}
```

### Comments

```go
// Single line comment

/*
    Multi-line comment
*/
```

### Packages

Before you can write any code you must declare what package the file belongs to. A package declares a translation unit in compilation. Because of this, packages can be compiled seperately and used as dynamic/static libraries.

```go
package main // Main package. Contains entry point.
```

```go
// A user package. All files in the same directory with the same
// package declaration will be part of the same translation unit.
package user
```

### Main function

You must declare a main function in the main package. This is the programs entry point. The returned integer is the programs exit code.

```go
package main

// main must have this exact function signature
func main() int {
    return 0 // Exit code 0
}
```

### Functions

Functions are declared similar to Go, using the `func` keyword. The return type is specified after the parameter list. Functions that do not return a value must still declare that with the `void` return type.

```go
// Return sum of two integers.
func add(a int, b int) int {
    return a + b
}

// Say hello. Has no return value.
func sayHello() void {
    println("Hello!")
}
```

### No semicolons

Koi does not use semicolons and is therefore whitespace sensitive, to an extent. Statements end with a newline or a right brace `}`.

```go
// Valid
func double(n int) int { return n * 2 }

// Error
func addOne(n int) int { n += 1 return n }
//                             ^ expected end of statement
```

### Strings

Strings literals are static arrays of bytes. They are enclosed in double quotes `"`. Character (byte) literals are written with single quotes `'`. Special characters are escaped with a backslash `\`.

```go
"Hello"

"a" // String
'a' // Byte

'\n' // newline

len("Bob") // 3
```

### Numbers and booleans

Integer literals default to a 32-bit signed integer `i32`. Number literals with a decimal point default to a 32-bit float `f32`. Boolean values are either `true` or `false`. They are their own type and cannot be compared with numbers.

```go
2   // i32
2.0 // f32

true == 1 // error: mismatched types in comparison
```

### Variables and constants

Variables are declared with the `:=` operator. The type is inferred from the value. Assignment uses the `=` operator. Variables can only be assigned values of its declared type. Constants are declared with the `::` operator. You can specify the type by passing it before either operator.

```go
age := 10      // Declare variable age of type int
name :: "John" // Declare a constant name with the value John.

age = 11        // Assign new value to age
name = "David"  // error: cannot assign to constant

number int := 32       // Specify type in declaration
name const string :: "John"  // Same but constant
```

Constant strings and arrays are put in the data section during compilation.

```go
animal := "Cat" // Allcated on the stack during runtime
animal :: "Dog" // Statically stored in data section of binary
```

### Arrays

```rs
// The type is inferred from the first element
fruits := {"Banana", "Apple", "Orange"} // type is []string

// Specify type of array two ways
numbers []u8 := {1, 2, 3}
numbers := {1 as u8, 2, 3}

len(numbers) // 3
```

### Structs

```go
struct Person {
    name string
    age  int
}

func f() {
    john := Person{name: "John", age: 32}
    println(john.name) // John
}
```

### Struct methods

```go
struct Dog {
    name string

    func bark() {
        // self is a reference to this Dog instance
        // and is available in all struct methods
        println("woof woof my name is {}", self.name)
    }
}

func f() {
    buddy := Dog{name: "Buddy"}
    buddy.bark() // woof woof my name is Buddy
}
```

```go
struct Account {
    holder  string
    balance f64
    debt    f64

    // Use the 'meta' keyword to make a method globally accessible through the
    // Account type. The 'self' keyword is not available in meta methods.
    meta func new(name string) Account {
        return Account{
            holder: name,
            balance: 0,
            debt: 0,
        }
    }
}

func f() {
    acc := Account.new("James")
    acc.new() // error: 'new' is a meta method on 'Account' and
              // is not available to 'Account' instances
}
```

### Tuples

```go
tuple Podium {
    string
    string
    string
}

func f() {
    p := Podium("John", "Mary", "Bob")

    println(p.0) // John
    println(p.1) // Mary
    println(p.2) // Bob
}
```

### Interfaces

```go
interface Named {
    name() string
}

struct Person is Named {
    name string

    func name() string {
        return self.name
    }
}

// Using multiple interfaces
struct File is Writer, Reader, Closer {
    ...
}
```

### Error interface

```go
// Special builtin interface
interface Error {
    error() string
}
```

```go
tuple SyntaxError is Error {
    string
    int

    func error() string {
        return fmt("syntax error: {}, line {}", self.0, self.1)
    }
}

func f() error {
    throw SyntaxError("missing semicolon", 21)
}
```

```go
// Builtin Err type
tuple Err is Error {
    string

    func error() string {
        return self.0
    }
}
```

### Throw and catch

```go
// This function either returns a float or throws an error
func divide(a float, b float) float | error {
    if b == 0 {
        throw Err("cannot divide by 0")
    }
    return a / b
}

// Example 1: Catching errors
func example1() {
    // An error is raised here and we print it out
    // result with be given a default value (0 in this case as it is default for int)
    result := divide(3, 0) catch err {
        println("oops, got error: {}", err)
    }
}

// Example 2: Re-throwing errors
func example2() error {
    // ? operator just throws the error again
    result1 := divide(1, 0)?

    // Is the same as this
    result2 := divide(1, 0) catch err {
        throw err
    }
}

// Example 3: Errors must be handled
func example3() {
    result := divide(8, 2) // error: error must be handled
}
```

### Ownership

Koi uses ownership rules to improve code readability and maintance when dealing with pointers and memory. There are only a few rules that govern ownership:

1. A pointer must have an owner.
2. You must pass on the ownership of a pointer in the same scope you acquired it.
3. You cannot assign to a variable owning a pointer before giving up ownership.

Koi uses the `!` symbol to denote ownership of a pointer. Note that **this is not a type**, but rather an indicator saying "you now own this pointer".

Simplest example:

```go
func example() void {
    // Allocate number on the heap.
    // number now owns the pointer to that memory
    number: *int = alloc(int, 1);
    //          ^ note that there is no ! here because, again, *its not a type*

    println("My favorite number is {}", *number);

    // Using ! to pass along ownership of number
    free(number!)

    *number += 1 // This will raise a compilation error as number
                 // is used after ownership is passed on
}
```

Explanation of rule 3:

```go
func example() void {
    mem := alloc([32]byte) // mem owns the pointer returned

    // Error. Cannot assign to a variable before passing on ownership
    mem = alloc([64]byte)

    // Ok
    mem = realloc(mem!, [64]byte)

    // Error. Ownership of mem was never passed on
}
```

### Pointer lifetimes

Ownership can be artificially created and destroyed with the respective `own()` and `end()` builtin functions. They are both 'magic' functions as they are technically disallowed by the compiler. They serve no purpose other than to open and close the ownership loop (mostly used when creating custom allocators).

```go
// Returns the same pointer, but now owned.
func own(ptr *void) *void!

// Takes final ownership of a pointer and does nothing.
func end(ptr *void!)
```

`alloc()` and `free()` are implemented using them:

```go
func alloc(t type) *void! {
    // ...acquire memory using syscalls etc...
    ptr := ...

    return own(ptr)
}

// Takes final ownership of a pointer and does nothing.
func free(ptr *void!) {
    // ...mark memory as freed and do other stuff...

    end(ptr)
}
```
