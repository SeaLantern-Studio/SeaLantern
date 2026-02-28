section .text
    global getregs
    getregs:
        mov [rdi + 0], rax
        mov [rdi + 8], rbx
        mov [rdi + 16], rcx
        mov [rdi + 24], rdx
        mov [rdi + 32], rsi
        mov [rdi + 40], rdi
        mov [rdi + 48], rbp
        mov [rdi + 56], rsp
        mov [rdi + 64], r8
        mov [rdi + 72], r9
        mov [rdi + 80], r10
        mov [rdi + 88], r11
        mov [rdi + 96], r12
        mov [rdi + 104], r13
        mov [rdi + 112], r14
        mov [rdi + 120], r15
        mov rax, [rsp]
        mov [rdi + 128], rax

        pushfq
        pop rax
        mov [rdi + 136], rax

        mov rax, 0
        mov ax, cs
        mov [rdi + 144], rax
        mov ax, ds
        mov [rdi + 152], rax
        mov ax, es
        mov [rdi + 160], rax
        mov ax, fs
        mov [rdi + 168], rax
        mov ax, gs
        mov [rdi + 176], rax
        mov ax, ss
        mov [rdi + 184], rax

        lea rax, [rdi + 192]
        fxsave [rax]

        ret
