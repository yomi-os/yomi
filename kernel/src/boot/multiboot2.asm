section .multiboot2
align 8

multiboot2_header_start:
    ; Multiboot2 magic number
    dd 0xE85250D6

    ; Architecture (0 = i386 protected mode)
    dd 0

    ; Header length
    dd multiboot2_header_end - multiboot2_header_start

    ; Checksum
    dd -(0xE85250D6 + 0 + (multiboot2_header_end - multiboot2_header_start))

    ; --- Tags start ---

    ; Information request tag
    align 8
    dw 1        ; type = 1 (information request)
    dw 0        ; flags = 0
    dd 20       ; size
    dd 3        ; memory map
    dd 6        ; memory info
    dd 8        ; framebuffer info

    ; Framebuffer tag
    align 8
    dw 5        ; type = 5 (framebuffer)
    dw 0        ; flags = 0
    dd 20       ; size
    dd 1024     ; width
    dd 768      ; height
    dd 32       ; depth

    ; Module alignment tag
    align 8
    dw 6        ; type = 6 (module alignment)
    dw 0        ; flags = 0
    dd 8        ; size

    ; End tag
    align 8
    dw 0        ; type = 0 (end)
    dw 0        ; flags = 0
    dd 8        ; size

multiboot2_header_end:
