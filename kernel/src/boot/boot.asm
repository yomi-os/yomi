global _start
extern kernel_main

section .bss
align 16
stack_bottom:
    resb 16384  ; 16 KiB stack
stack_top:

section .text._start
bits 32
_start:
    ; Set up stack pointer
    mov esp, stack_top

    ; Save multiboot2 magic and info pointer
    ; EAX contains magic number
    ; EBX contains pointer to multiboot2 info structure
    push ebx    ; Push multiboot2 info pointer (2nd argument)
    push eax    ; Push magic number (1st argument)

    ; Call the kernel main function
    call kernel_main

    ; If kernel_main returns, hang
    cli
.hang:
    hlt
    jmp .hang
