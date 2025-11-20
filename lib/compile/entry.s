# Exports
# extern func println(s string)
# extern func write(s string, len int)
# extern func len(s string) int

.intel_syntax noprefix
.extern main

.section .data

	.NEWLINE: .asciz "\n"

.section .text
.globl _start

# rdi = string, rsi = length
.globl write
write:
	mov eax, 1

	mov r9, rdi
	mov r10, rsi

	mov rdi, 1
	mov rsi, r9
	mov rdx, r10
	syscall
	ret


# rdi = string
.globl len
len:
	push rbp
	mov rbp, rsp
	sub rsp, 16

	mov QWORD PTR [rbp-8], rdi
	mov rax, 0

.loop:
	mov r12, QWORD PTR [rbp-8]
	cmp BYTE PTR [r12 + rax], 0
	je .end

	inc rax
	jmp .loop
.end:

	leave
	ret


# rdi = string
.globl println
println:
	push rbp
	mov rbp, rsp
	sub rsp, 16
	mov QWORD PTR [rbp-8], rdi 	# string input

	mov rdi, QWORD PTR [rbp-8]
	call len

	mov rdi, QWORD PTR [rbp-8]
	mov rsi, rax
	call write

	lea rdi, .NEWLINE
	mov rsi, 1
	call write

	leave
	ret


_start:
	call main
	mov r12, rax

	mov rax, 60
	mov rdi, r12
	syscall

.section .note.GNU-stack,"",@progbits
