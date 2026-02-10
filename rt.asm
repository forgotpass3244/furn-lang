
extern _PKG_main_main
global _start
_start:
    sub rsp, 8 ; alloc return val
    call _PKG_main_main
    call exit
exit:
    mov rax, 60
    mov rdi, [rsp+8]
    syscall
