section .data
extern _PKG_rt_print
_STR_0: db 104,101,108,108,111,44,32,102,117,114,110,33,10
_STR_13: db 101,120,105,116,105,110,103,46,46,46,33,10
_GLOB_0 equ 67
_GLOB_8 equ OP_0
_GLOB_16 equ _STR_0
_GLOB_24 equ 13
global _PKG_main_main
_PKG_main_main equ _GLOB_32
_GLOB_32 equ OP_2
section .text
    OP_0: mov qword [rsp+8], 67
    OP_1: ret
    OP_2: sub rsp, 8
    OP_3: push qword _STR_0
    OP_4: push qword 13
    OP_5: push _PKG_rt_print
OP_6:
    pop rax
    call rax
    OP_7: push qword _STR_13
    OP_8: push qword 12
    OP_9: push _PKG_rt_print
OP_10:
    pop rax
    call rax
    OP_11: sub rsp, 8
    OP_12: call OP_0
OP_13:
    pop rax
    mov [rsp+0], rax
OP_14:
    pop rax
    mov [rsp+8], rax
    OP_15: ret
