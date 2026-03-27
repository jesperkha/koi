use crate::{
    build::x86::{File, assemble},
    config::Config,
    util::{compare_string_lines_or_panic, emit_string, must},
};

fn assemble_src(src: &str) -> File {
    let unit = must(emit_string(src));
    assemble(unit, &Config::test())
}

fn compare(src: &str, expect: &str) {
    compare_string_lines_or_panic(assemble_src(src).to_string(), expect.into());
}

#[test]
fn test_return_0() {
    compare(
        r#"
func main() int {
    return 0
}
        "#,
        r#"
.intel_syntax noprefix
.section .data

.section .text

.globl main
main:
    push rbp
    mov rbp, rsp
    mov rax, 0
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_string_return() {
    compare(
        r#"
func f() string {
    return "Hello"
}
        "#,
        r#"
.intel_syntax noprefix
.section .data

.D0: .asciz "Hello"

.section .text

f:
    push rbp
    mov rbp, rsp
    lea rax, [rip + .D0]
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_function_call() {
    compare(
        r#"
func f(a int, b string) int {
    n := f(1, "Hello")
    return n
}
        "#,
        r#"
.intel_syntax noprefix
.section .data

.D0: .asciz "Hello"
.section .text

f:
    push rbp
    mov rbp, rsp
    sub rsp, 32
    mov QWORD PTR [rbp-8], rdi
    mov QWORD PTR [rbp-16], rsi
    mov rdi, 1
    lea rsi, [rip + .D0]
    call f
    mov QWORD PTR [rbp-24], rax
    mov rax, QWORD PTR [rbp-24]
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_assignment() {
    compare(
        r#"
func f() {
    a := true
    a = false
    b := 1
    b = 0
}
        "#,
        r#"
.intel_syntax noprefix
.section .data

.section .text

f:
    push rbp
    mov rbp, rsp
    sub rsp, 16
    mov BYTE PTR [rbp-1], 1
    mov BYTE PTR [rbp-1], 0
    mov QWORD PTR [rbp-9], 1
    mov QWORD PTR [rbp-9], 0
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_string_assignment() {
    compare(
        r#"
func f() {
    s := "Hello" 
    s = "World"
    x := s
    s = x
}
        "#,
        r#"
.intel_syntax noprefix
.section .data

.D0: .asciz "Hello"
.D1: .asciz "World"
.section .text

f:
    push rbp
    mov rbp, rsp
    sub rsp, 16
    lea rax, [rip + .D0]
    mov QWORD PTR [rbp-8], rax
    lea rax, [rip + .D1]
    mov QWORD PTR [rbp-8], rax
    mov rax, QWORD PTR [rbp-8]
    mov QWORD PTR [rbp-16], rax
    mov rax, QWORD PTR [rbp-16]
    mov QWORD PTR [rbp-8], rax
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_param_alloc() {
    compare(
        r#"
func f(a int, b bool, c string) int {
    a = 0
    b = false
    d := 123
    return 0
}
        "#,
        r#"
.intel_syntax noprefix
.section .data

.section .text

f:
    push rbp
    mov rbp, rsp
    sub rsp, 32
    mov QWORD PTR [rbp-8], rdi
    mov BYTE PTR [rbp-9], sil
    mov QWORD PTR [rbp-17], rdx
    mov QWORD PTR [rbp-8], 0
    mov BYTE PTR [rbp-9], 0
    mov QWORD PTR [rbp-25], 123
    mov rax, 0
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}
