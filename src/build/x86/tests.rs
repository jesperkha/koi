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
    mov eax, 0
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
    sub rsp, 16
    mov DWORD PTR [rbp-4], edi
    mov QWORD PTR [rbp-12], rsi
    mov edi, 1
    lea rsi, [rip + .D0]
    call f
    mov DWORD PTR [rbp-16], eax
    mov eax, DWORD PTR [rbp-16]
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
    mov DWORD PTR [rbp-5], 1
    mov DWORD PTR [rbp-5], 0
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
fn test_binary_add() {
    compare(
        r#"
func f(a int, b int) int {
    return a + b
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
    mov DWORD PTR [rbp-4], edi
    mov DWORD PTR [rbp-8], esi
    mov eax, DWORD PTR [rbp-4]
    mov r10d, DWORD PTR [rbp-8]
    add eax, r10d
    mov eax, eax
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_binary_sub() {
    compare(
        r#"
func f(a int, b int) int {
    return a - b
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
    mov DWORD PTR [rbp-4], edi
    mov DWORD PTR [rbp-8], esi
    mov eax, DWORD PTR [rbp-4]
    mov r10d, DWORD PTR [rbp-8]
    sub eax, r10d
    mov eax, eax
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_binary_mul() {
    compare(
        r#"
func f(a int, b int) int {
    return a * b
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
    mov DWORD PTR [rbp-4], edi
    mov DWORD PTR [rbp-8], esi
    mov eax, DWORD PTR [rbp-4]
    mov r10d, DWORD PTR [rbp-8]
    imul eax, r10d
    mov eax, eax
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_binary_div() {
    compare(
        r#"
func f(a int, b int) int {
    return a / b
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
    mov DWORD PTR [rbp-4], edi
    mov DWORD PTR [rbp-8], esi
    mov eax, DWORD PTR [rbp-4]
    mov r10d, DWORD PTR [rbp-8]
    cdq
    idiv r10d
    mov eax, eax
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_binary_eq() {
    compare(
        r#"
func f(a int, b int) bool {
    return a == b
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
    mov DWORD PTR [rbp-4], edi
    mov DWORD PTR [rbp-8], esi
    mov eax, DWORD PTR [rbp-4]
    mov r10d, DWORD PTR [rbp-8]
    cmp eax, r10d
    sete al
    mov al, al
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_binary_lt() {
    compare(
        r#"
func f(a int, b int) bool {
    return a < b
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
    mov DWORD PTR [rbp-4], edi
    mov DWORD PTR [rbp-8], esi
    mov eax, DWORD PTR [rbp-4]
    mov r10d, DWORD PTR [rbp-8]
    cmp eax, r10d
    setl al
    mov al, al
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_binary_and() {
    compare(
        r#"
func f(a bool, b bool) bool {
    return a && b
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
    mov BYTE PTR [rbp-1], dil
    mov BYTE PTR [rbp-2], sil
    mov al, BYTE PTR [rbp-1]
    mov r10b, BYTE PTR [rbp-2]
    and al, r10b
    mov al, al
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_binary_or() {
    compare(
        r#"
func f(a bool, b bool) bool {
    return a || b
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
    mov BYTE PTR [rbp-1], dil
    mov BYTE PTR [rbp-2], sil
    mov al, BYTE PTR [rbp-1]
    mov r10b, BYTE PTR [rbp-2]
    or al, r10b
    mov al, al
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_unary_neg() {
    compare(
        r#"
func f(a int) int {
    return -a
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
    mov DWORD PTR [rbp-4], edi
    mov eax, DWORD PTR [rbp-4]
    neg eax
    mov eax, eax
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_unary_not() {
    compare(
        r#"
func f(a bool) bool {
    return !a
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
    mov BYTE PTR [rbp-1], dil
    mov al, BYTE PTR [rbp-1]
    xor al, 1
    mov al, al
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
    mov DWORD PTR [rbp-4], edi
    mov BYTE PTR [rbp-5], sil
    mov QWORD PTR [rbp-13], rdx
    mov DWORD PTR [rbp-4], 0
    mov BYTE PTR [rbp-5], 0
    mov DWORD PTR [rbp-17], 123
    mov eax, 0
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_if_else() {
    compare(
        r#"
func f(a bool) int {
    if a {
        return 0
    } else {
        return 1
    }
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
    mov BYTE PTR [rbp-1], dil
    cmp BYTE PTR [rbp-1], 0
    jz .Lf_cond_0
    mov eax, 0
    leave
    ret
    jmp .Lf_cond_end_0
    .Lf_cond_0:
    mov eax, 1
    leave
    ret
    .Lf_cond_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_if_elseif_else() {
    compare(
        r#"
func f(a bool, b bool) int {
    if a {
        return 0
    } else if b {
        return 1
    } else {
        return 2
    }
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
    mov BYTE PTR [rbp-1], dil
    mov BYTE PTR [rbp-2], sil
    cmp BYTE PTR [rbp-1], 0
    jz .Lf_cond_0
    mov eax, 0
    leave
    ret
    jmp .Lf_cond_end_0
    .Lf_cond_0:
    cmp BYTE PTR [rbp-2], 0
    jz .Lf_cond_1
    mov eax, 1
    leave
    ret
    jmp .Lf_cond_end_0
    .Lf_cond_1:
    mov eax, 2
    leave
    ret
    .Lf_cond_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_if_elseif_else_computed() {
    compare(
        r#"
func f(a int, b int) int {
    if a > 0 {
        return 1
    } else if a < b {
        return 2
    } else {
        return 3
    }
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
    mov DWORD PTR [rbp-4], edi
    mov DWORD PTR [rbp-8], esi
    mov eax, DWORD PTR [rbp-4]
    mov r10d, 0
    cmp eax, r10d
    setg al
    cmp al, 0
    jz .Lf_cond_0
    mov eax, 1
    leave
    ret
    jmp .Lf_cond_end_0
    .Lf_cond_0:
    mov eax, DWORD PTR [rbp-4]
    mov r10d, DWORD PTR [rbp-8]
    cmp eax, r10d
    setl al
    cmp al, 0
    jz .Lf_cond_1
    mov eax, 2
    leave
    ret
    jmp .Lf_cond_end_0
    .Lf_cond_1:
    mov eax, 3
    leave
    ret
    .Lf_cond_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_if_elseif_no_else() {
    compare(
        r#"
func f(a bool, b bool) {
    if a {
        return
    } else if b {
        return
    }
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
    mov BYTE PTR [rbp-1], dil
    mov BYTE PTR [rbp-2], sil
    cmp BYTE PTR [rbp-1], 0
    jz .Lf_cond_0
    leave
    ret
    jmp .Lf_cond_end_0
    .Lf_cond_0:
    cmp BYTE PTR [rbp-2], 0
    jz .Lf_cond_end_0
    leave
    ret
    jmp .Lf_cond_end_0
    .Lf_cond_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_if_else_nested_in_then() {
    compare(
        r#"
func f(a bool, b bool) int {
    if a {
        if b {
            return 0
        } else {
            return 1
        }
    } else {
        return 2
    }
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
    mov BYTE PTR [rbp-1], dil
    mov BYTE PTR [rbp-2], sil
    cmp BYTE PTR [rbp-1], 0
    jz .Lf_cond_0
    cmp BYTE PTR [rbp-2], 0
    jz .Lf_cond_1
    mov eax, 0
    leave
    ret
    jmp .Lf_cond_end_1
    .Lf_cond_1:
    mov eax, 1
    leave
    ret
    .Lf_cond_end_1:
    jmp .Lf_cond_end_0
    .Lf_cond_0:
    mov eax, 2
    leave
    ret
    .Lf_cond_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_if_else_nested_in_else() {
    compare(
        r#"
func f(a bool, b bool) int {
    if a {
        return 0
    } else {
        if b {
            return 1
        } else {
            return 2
        }
    }
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
    mov BYTE PTR [rbp-1], dil
    mov BYTE PTR [rbp-2], sil
    cmp BYTE PTR [rbp-1], 0
    jz .Lf_cond_0
    mov eax, 0
    leave
    ret
    jmp .Lf_cond_end_0
    .Lf_cond_0:
    cmp BYTE PTR [rbp-2], 0
    jz .Lf_cond_1
    mov eax, 1
    leave
    ret
    jmp .Lf_cond_end_1
    .Lf_cond_1:
    mov eax, 2
    leave
    ret
    .Lf_cond_end_1:
    .Lf_cond_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_while_simple() {
    compare(
        r#"
func f(a bool) {
    while a {
    }
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
    mov BYTE PTR [rbp-1], dil
    .Lf_loop_0:
    cmp BYTE PTR [rbp-1], 0
    jz .Lf_loop_end_0
    jmp .Lf_loop_0
    .Lf_loop_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_while_with_body() {
    compare(
        r#"
func f(a bool) {
    while a {
        a = false
    }
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
    mov BYTE PTR [rbp-1], dil
    .Lf_loop_0:
    cmp BYTE PTR [rbp-1], 0
    jz .Lf_loop_end_0
    mov BYTE PTR [rbp-1], 0
    jmp .Lf_loop_0
    .Lf_loop_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_while_computed_condition() {
    // Condition binary expression is re-evaluated at the top of every iteration
    compare(
        r#"
func f(a int, b int) {
    while a < b {
        a = a + 1
    }
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
    mov DWORD PTR [rbp-4], edi
    mov DWORD PTR [rbp-8], esi
    .Lf_loop_0:
    mov eax, DWORD PTR [rbp-4]
    mov r10d, DWORD PTR [rbp-8]
    cmp eax, r10d
    setl al
    cmp al, 0
    jz .Lf_loop_end_0
    mov eax, DWORD PTR [rbp-4]
    mov r10d, 1
    add eax, r10d
    mov DWORD PTR [rbp-4], eax
    jmp .Lf_loop_0
    .Lf_loop_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_while_break() {
    compare(
        r#"
func f(a bool) {
    while a {
        break
    }
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
    mov BYTE PTR [rbp-1], dil
    .Lf_loop_0:
    cmp BYTE PTR [rbp-1], 0
    jz .Lf_loop_end_0
    jmp .Lf_loop_end_0
    jmp .Lf_loop_0
    .Lf_loop_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_while_continue() {
    compare(
        r#"
func f(a bool) {
    while a {
        continue
    }
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
    mov BYTE PTR [rbp-1], dil
    .Lf_loop_0:
    cmp BYTE PTR [rbp-1], 0
    jz .Lf_loop_end_0
    jmp .Lf_loop_0
    jmp .Lf_loop_0
    .Lf_loop_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_while_break_continue_nested() {
    // break exits inner loop; continue restarts outer loop
    compare(
        r#"
func f(a bool, b bool) {
    while a {
        while b {
            break
        }
        continue
    }
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
    mov BYTE PTR [rbp-1], dil
    mov BYTE PTR [rbp-2], sil
    .Lf_loop_0:
    cmp BYTE PTR [rbp-1], 0
    jz .Lf_loop_end_0
    .Lf_loop_1:
    cmp BYTE PTR [rbp-2], 0
    jz .Lf_loop_end_1
    jmp .Lf_loop_end_1
    jmp .Lf_loop_1
    .Lf_loop_end_1:
    jmp .Lf_loop_0
    jmp .Lf_loop_0
    .Lf_loop_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}

#[test]
fn test_while_nested() {
    // Each while claims its own cond/end label pair via next_cond_label /
    // next_end_label (both increment), so nested loops get distinct labels
    // and there are no duplicate-label collisions.
    compare(
        r#"
func f(a bool, b bool) {
    while a {
        while b {
        }
    }
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
    mov BYTE PTR [rbp-1], dil
    mov BYTE PTR [rbp-2], sil
    .Lf_loop_0:
    cmp BYTE PTR [rbp-1], 0
    jz .Lf_loop_end_0
    .Lf_loop_1:
    cmp BYTE PTR [rbp-2], 0
    jz .Lf_loop_end_1
    jmp .Lf_loop_1
    .Lf_loop_end_1:
    jmp .Lf_loop_0
    .Lf_loop_end_0:
    leave
    ret

.section .note.GNU-stack,"",@progbits
        "#,
    );
}
