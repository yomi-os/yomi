global long_mode_start_trampoline
extern kernel_main

; Trampoline in .boot section (physical address) that jumps to higher half
section .boot
bits 64
long_mode_start_trampoline:
    ; Early diagnostic: Write "64" to VGA at top-left to indicate 64-bit mode reached
    ; This helps debug if kernel_main is never reached
    mov word [0xB8000], 0x0F36  ; '6' - white on black
    mov word [0xB8002], 0x0F34  ; '4' - white on black

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

    ; Early diagnostic: Write "HH" to VGA to indicate higher-half reached
    ; Position: column 2-3 (after "64")
    mov word [0xB8004], 0x0F48  ; 'H' - white on black
    mov word [0xB8006], 0x0F48  ; 'H' - white on black

    ; Call Rust code
    ; rdi = Multiboot2 magic number
    ; rsi = Multiboot2 info address
    call kernel_main

    ; If kernel_main returns, halt
    ; This should never be reached
    mov word [0xB8008], 0x4F21  ; '!' - white on red (error indicator)
.halt:
    hlt
    jmp .halt
