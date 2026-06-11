# Todo

## Known bugs

- `x86-64`: Multiple calls in the same expression override return value in `rax`
    - In Fibonacci example: `fib(n - 1) + fib(n - 2)`
    - `rax` is overridden in the second call
- String compare still uses simple pointer comparison
    - Implement string compare instrinsic

## Todos

- Global constants
    - Declare constants in global scope
    - Export and import constants
    - `name := expr`
    - `mut name := expr`
    - `name: type = expr`

## Other

- LSP
    - [Tower LSP](https://github.com/ebkalderon/tower-lsp)
