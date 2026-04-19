# Todo

## Known bugs

- Multiple calls in the same expression override return value in `rax`
    - In Fibonacci example: `fib(n - 1) + fib(n - 2)`
    - `rax` is overridden in the second call
- String compare still uses simple pointer comparison
    - Implement string compare instrinsic

## Todos

- Add remaining primitive types
    - `byte i8...i64 u8...u64 f32 f64`
- Floating point literals
    - Float lits and arithmetic ops
- Assignment with specific type
    - `a: type : value`
- Make variables immutable by default
    - `mut` keyword to mutate variables
    - `mut a := 0`
    - `mut a: int = 0`
- Casting
    - Numeric cast operator `v as type`
    - Bit cast (instrinsic) `bit_cast(v, type)`
- Global constants
    - Declare constants in global scope
    - Export and import constants
    - `name := expr`
    - `mut name := expr`
    - `name: type = expr`

## Other

- LSP
    - [Tower LSP](https://github.com/ebkalderon/tower-lsp)

