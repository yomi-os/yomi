global long_mode_start
extern kernel_main

section .text
bits 64
long_mode_start:
    ; Clear segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; Call Rust code
    ; rdi = Multiboot2 magic number
    ; rsi = Multiboot2 info address
    call kernel_main

    ; If kernel_main returns, halt
.halt:
    hlt
    jmp .halt
