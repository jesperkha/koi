# External symbols

Symbols from linked libraries can be declared as `extern` functions.

Koi follows the System V ABI and can seamlessly work with C code. Your program is also linked with `libc` by default:

```
extern func puts(s string)

func main() int {
    puts("Hello from libc!")
    return 0
}
```

Sometimes C libraries use strange or inconsistent naming conventions. The `alias` modifier will rename the symbol to whatever you want. It is purely semantic and does not change the compiled code in any way.

```
@alias stringCompare
extern func strcmp(a string, b string) int

func equal(a string, b string) bool {
    return stringCompare(a, b) == 0
}
```

External symbols can also be re-exported. Aliasing works with exports as well. The standard library `io.println` function is actually declared as:

```
@alias println
pub extern func puts(s string)
```
