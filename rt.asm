
extern PKG_main_main
global _start
_start:
    push qword 0 ; alloc return val (default to 0)
    call PKG_main_main
    mov rax, 60
    pop rdi
    syscall
global PKG_Rt_print
PKG_Rt_print:
    mov rax, 1
    mov rdi, 1
    mov rsi, [rsp+16]
    mov rdx, [rsp+8]
    syscall
    ret 16
global PKG_Rt_print_char
PKG_Rt_print_char:
    mov rax, 1
    mov rdi, 1
    lea rsi, [rsp+8]
    mov rdx, 1
    syscall
    ret 8
global PKG_Rt_print_digit
PKG_Rt_print_digit:
    mov rax, [rsp+8]
    add rax, 48
    push rax
    call PKG_Rt_print_char
    ret 8
global PKG_Rt_print_u64
PKG_Rt_print_u64:
    mov rax, [rsp+8] ; get arg
    
    sub rsp, 24             ; reserve stack space (more than enough)
    lea rcx, [rsp + 23]     ; point to end of buffer

    mov rbx, 10

    test rax, rax
    jnz .convert

    ; handle zero explicitly
    mov byte [rcx-1], '0'
    lea rsi, [rcx-1]
    mov rdx, 2              ; "0\n"
    jmp .write

.convert:
    xor rdx, rdx
.loop:
    div rbx                 ; RAX /= 10
    add dl, '0'
    dec rcx
    mov [rcx], dl
    xor rdx, rdx
    test rax, rax
    jnz .loop

    lea rsi, [rcx]
    mov rdx, rsp
    add rdx, 24
    sub rdx, rcx            ; length

.write:
    mov rax, 1              ; sys_write
    mov rdi, 1              ; stdout
    syscall

    add rsp, 24
    ret 8

