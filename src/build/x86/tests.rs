use crate::{
    build::x86::{File, assemble},
    util::{compare_string_lines_or_panic, emit_string, must},
};

fn assemble_src(src: &str) -> File {
    let unit = must(emit_string(src));
    assemble(unit)
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
