global _start
extern kernel_main

section .bss
align 16
stack_bottom:
    resb 16384  ; 16KB stack
stack_top:

section .boot
bits 32
_start:
    ; Set up stack pointer
    mov esp, stack_top

    ; Save Multiboot2 magic and info address
    mov edi, eax    ; Magic number
    mov esi, ebx    ; Multiboot2 info address

    ; Check CPUID support
    call check_cpuid
    test eax, eax
    jz .no_cpuid

    ; Check Long mode support
    call check_long_mode
    test eax, eax
    jz .no_long_mode

    ; Set up paging
    call setup_page_tables
    call enable_paging

    ; Load GDT
    lgdt [gdt64.pointer]

    ; Jump to 64-bit mode
    jmp gdt64.code:long_mode_start

.no_cpuid:
    mov al, "C"
    jmp error

.no_long_mode:
    mov al, "L"
    jmp error

error:
    ; Display error message to VGA buffer
    mov dword [0xb8000], 0x4f524f45  ; "ER" (white on red)
    mov dword [0xb8004], 0x4f3a4f52  ; "R:" (white on red)
    mov dword [0xb8008], 0x4f204f20  ; "  " (white on red)
    mov byte  [0xb800a], al          ; Error code
    hlt

; Check CPUID support
check_cpuid:
    pushfd
    pop eax
    mov ecx, eax
    xor eax, 1 << 21
    push eax
    popfd
    pushfd
    pop eax
    push ecx
    popfd
    xor eax, ecx
    ret

; Check Long mode support
check_long_mode:
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode

    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29
    jz .no_long_mode

    mov eax, 1
    ret

.no_long_mode:
    xor eax, eax
    ret

; Set up page tables
setup_page_tables:
    ; P4 table: Set up identity mapping (P4[0])
    mov eax, p3_table
    or eax, 0b11  ; present + writable
    mov [p4_table], eax

    ; P4 table: Set up higher half mapping (P4[511])
    mov eax, p3_table
    or eax, 0b11
    mov [p4_table + 511 * 8], eax

    ; P3 table: Set pointer to P2 table
    mov eax, p2_table
    or eax, 0b11
    mov [p3_table], eax

    ; P2 table: Map 512 2MB pages (total 1GB)
    mov ecx, 0
.map_p2_table:
    mov eax, 0x200000   ; 2MB
    mul ecx
    or eax, 0b10000011  ; present + writable + huge page
    mov [p2_table + ecx * 8], eax

    inc ecx
    cmp ecx, 512
    jne .map_p2_table

    ret

; Enable paging
enable_paging:
    ; Load P4 table into CR3
    mov eax, p4_table
    mov cr3, eax

    ; Enable PAE
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; Enable Long mode
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; Enable paging
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096

section .rodata
gdt64:
    dq 0  ; null descriptor
.code: equ $ - gdt64
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53)  ; code segment
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

; 64-bit mode entry point (defined in boot64.asm)
extern long_mode_start
