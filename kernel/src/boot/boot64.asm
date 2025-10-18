global long_mode_start_trampoline
extern kernel_main

; Trampoline in .boot section (physical address) that jumps to higher half
section .boot
bits 64
long_mode_start_trampoline:
    ; Load the higher-half address of long_mode_start and jump to it
    mov rax, long_mode_start_higher
    jmp rax

; The actual 64-bit entry point at higher half address
section .text
bits 64
long_mode_start_higher:
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
