ENTRY(_start)

OUTPUT_FORMAT(elf64-x86-64)

SECTIONS {
    . = 1M;

    __kernel_physical_start = .;

    .multiboot2 ALIGN(8) : AT(ADDR(.multiboot2))
    {
        KEEP(*(.multiboot2))
    }

    .boot ALIGN(4K) : AT(ADDR(.boot))
    {
        KEEP(*(.boot))
        KEEP(*(.text._start))
    }

    .text : ALIGN(4K)
    {
        __text_start = .;
        KEEP(*(.text._start))
        *(.text .text.*)
        __text_end = .;
    }

    .rodata : ALIGN(4K)
    {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    }

    .data : ALIGN(4K)
    {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    .bss : ALIGN(4K)
    {
        __bss_start = .;
        *(COMMON)
        *(.bss .bss.*)
        __bss_end = .;
    }

    __kernel_physical_end = .;

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.note .note.*)
    }
}
