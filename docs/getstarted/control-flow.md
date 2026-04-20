# Control flow

## If statements

If statements are the same as any other language. You can chain an arbitrary amount of `else if`s after eachother.

```
n := 10

if n > 5 {
   io.println("Greater than 5!")
} else if n == 3 {
   io.println("3!")
} else {
   io.println("Uncool number...")
}
```

## While loop

Loop while the expression is true. Use `break` and `continue` to break out of the loop or continue to the next iteration.

```
i := 0
while true {
    i = i + 1

    if i % 2 == 0 {
        io.println("Even")
        continue
    }

    if i > 20 {
        break
    }

    io.println("Odd")
}
```

## For loop

The for loop is similar to a C for loop in that it is simplt syntactic sugar for a while loop. It has three parts:

1. A initializer statement. Run once before the loop.
2. A boolean condition. The loop runs while this is true.
3. A post condition statement. Runs after each iteration.

```
sum := 0

for i := 0; i < 10; i += 1 {
    sum += i
}
```

The two statements can be **any valid statement**. This means you can do some pretty interesting stuff:

```
import std.io

func main() int {
    for io.println("Hello"); true; return 0 {
        io.println("World")
    }
    
    return 0
}
```

```
$ koi run
Hello
World
```
