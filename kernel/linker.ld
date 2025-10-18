ENTRY(_start)

OUTPUT_FORMAT(elf64-x86-64)

/* Higher Half Kernel virtual address */
KERNEL_VIRTUAL_BASE = 0xFFFFFFFF80000000;
KERNEL_PHYSICAL_BASE = 0x100000;

SECTIONS
{
    . = KERNEL_PHYSICAL_BASE;

    __kernel_physical_start = .;

    /* Place Multiboot2 header at the beginning */
    .multiboot2 ALIGN(8) : AT(ADDR(.multiboot2))
    {
        KEEP(*(.multiboot2))
    }

    /* Boot code (32-bit) */
    .boot ALIGN(4K) : AT(ADDR(.boot))
    {
        KEEP(*(.boot))
    }

    /* Map to Higher Half from here */
    . += KERNEL_VIRTUAL_BASE;
    __kernel_virtual_start = .;

    /* Text section */
    .text ALIGN(4K) : AT(ADDR(.text) - KERNEL_VIRTUAL_BASE)
    {
        __text_start = .;
        *(.text .text.*)
        __text_end = .;
    }

    /* Read-only data */
    .rodata ALIGN(4K) : AT(ADDR(.rodata) - KERNEL_VIRTUAL_BASE)
    {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    }

    /* Data section */
    .data ALIGN(4K) : AT(ADDR(.data) - KERNEL_VIRTUAL_BASE)
    {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    /* BSS section */
    .bss ALIGN(4K) : AT(ADDR(.bss) - KERNEL_VIRTUAL_BASE)
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    __kernel_virtual_end = .;
    __kernel_physical_end = . - KERNEL_VIRTUAL_BASE;

    /* Debug information (not placed in memory) */
    .debug_info     0 : { *(.debug_info) }
    .debug_abbrev   0 : { *(.debug_abbrev) }
    .debug_line     0 : { *(.debug_line) }
    .debug_str      0 : { *(.debug_str) }
    .debug_ranges   0 : { *(.debug_ranges) }

    /* Discard unnecessary sections */
    /DISCARD/ :
    {
        *(.eh_frame)
        *(.note .note.*)
    }
}
