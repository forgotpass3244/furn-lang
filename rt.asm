
extern main
global _start
_start:
    sub rsp, 8 ; alloc return val
    call [main]
    call exit
exit:
    mov rax, 60
    mov rdi, [rsp+8]
    syscall

