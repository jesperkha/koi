# koi language design document

## Basic info

Koi is a statically typed, compiled language, made as an attempt to make a modern version of C. It keeps all the low level control and improves syntax and code writing ergonomics.

## Language specs

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
    Block comment
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
// main must have this exact function signature
func main() int {
    return 0 // Exit code 0
}
```

### Variables

Variables are declared with the `:=` operator. The type is inferred from the value. Assignment uses the `=` operator. Variables can only be assigned values of its declared type. Constants are declared with the `::` operator. You can also specify the type by passing it bewteen `:` and `=`.

```go
age := 10   // Declare variable age of type int
age = 11    // Assign new value to age

name :: "John" // Declare a constant name with the value John.

smallNumber: u8 = 32
```

### The string type

Just like in C, strings are just arrays of bytes. Koi has a distinct `string` type to make string operations easier. However, they behave just like constant byte arrays under the hood. Any string operations on strings are simply sugar for function calls.

```go
name := "Daniel"
name += " Hoffman"

println(name) // Daniel Hoffman
```

### Subroutines

A subroutine is a function inside another function which can access all local variables inside the scope. They are useful for repeated computation that uses state.

```rs
// Upgrade all members with silver or higher tier whos membership is
// older that minSignupDate.
func upgradeMemberships(users []User, minSignupTime time.Duration) void {
    // Subroutine to check if a user is eligble for upgrade.
    // Only available inside this function.
    // Has access to all variables in the outer scope.
    sub isEligible(user User) bool {
        memberDuration := time.now() - user.signupTime
        return memberDuration > minSignupTime && user.tier > .SILVER
    }

    for user in users {
        if isEligible(user) {
            user.tier += 1
        }
    }
}
```

### Ownership

```go
// New user function returns a user pointer with the ! operator,
// indicating that the caller now owns the memory.
func newUser(id int, name string) *User! {
    user := User{
        .id = id
        .name = name
    }

    // alloc() returns a type of *void!
    return alloc(user)
}

func deleteUser(user *User!) void {
    db.removeUser(user.id)
    free(user)
}

func main() void {
    user := newUser(1, "John")

    // Owned memory must be freed in the scope it is allocated in
    // unless the ownership is passed along somewhere else.

    deleteUser(user!) // Commenting this line out will raise an error as
                      // user must be freed in this scope or handed off.
}
```

```go
func a(user *User) void {
    // ...
}

func b(user *User!) void {
    // ...
}

func handleUser(user *User!) void {
    a(user) // ok
    b(user) // error: b requires ownership of the pointer

    a(user!) // error: a does not accept ownership
    b(user!) // ok

    // error: handleUser owns user and must free it
}
```

```go
func newUser() *User! {
    // ...
}

func deleteUser(user *User!) void {
    // ...
}

func main() void {
    user := newUser() // Owns user

    if user.name == "John" {
        deleteUser(user!)
    }

    println(user.name) // error: user cannot be accessed after ownership
                       // was conditionally passed to deleteUser
}
```
