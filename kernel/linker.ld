ENTRY(_start)

SECTIONS {
    . = 1M;

    .boot :
    {
        KEEP(*(.multiboot2))
        KEEP(*(.multiboot_header))
    }

    .text : ALIGN(4K)
    {
        KEEP(*(.text._start))
        *(.text .text.*)
    }

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K)
    {
        *(.data .data.*)
    }

    .bss : ALIGN(4K)
    {
        *(COMMON)
        *(.bss .bss.*)
    }

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.note .note.*)
    }
}
