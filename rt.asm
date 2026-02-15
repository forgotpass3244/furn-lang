
extern _PKG_main_main
global _start
_start:
    sub rsp, 32 ; alloc return val
    call _PKG_main_main
    call exit
exit:
    mov rax, 60
    mov rdi, [rsp+8]
    syscall
global _PKG_rt_print
_PKG_rt_print:
    mov rax, 1
    mov rdi, 1
    mov rsi, [rsp+16]
    mov rdx, [rsp+8]
    syscall
    ret 16
