.intel_syntax noprefix
.extern main

.section .data

.section .text
.globl _start

_start:
	call main
	mov r12, rax

	mov rax, 60
	mov rdi, r12
	syscall

.section .note.GNU-stack,"",@progbits
