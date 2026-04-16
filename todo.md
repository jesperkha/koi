# Todo

## Doing

- `feat/global-const`: Global constants
    - Local top-level constants which can be exported and imported in modules

## Known bugs

- Comments mess up line count in lexer (error messages)
- Variable shadowing doesnt work
    - Separate scopes should overlap stack memory when assembling
- Function variable scope does not get dropped after an error is raised inside a function body
    -   1. Declare `x := 0` in function `f`
    -   2. Produce and error `io.println(b)` (not defined)
    -   3. Declare `x := 0` in function `g` below
    - Results in "`b` is not defined" _and_ "`x` is already defined"

## Language features

- For-loop
- Operator assignment (`a += 1`)
- Typedef
- Floats
- Structs
- Slices
- Strings

## Other

- LSP
    - [Tower LSP](https://github.com/ebkalderon/tower-lsp)

