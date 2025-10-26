# GDB Configuration for Yomi OS Kernel Debugging
# This file is automatically loaded by GDB when debugging the kernel

# Connect to QEMU GDB server
target remote :1234

# Load kernel symbols
symbol-file target/x86_64-unknown-none/debug/yomi-kernel

# Set architecture to x86-64
set architecture i386:x86-64

# Disable pagination for better automation support
set pagination off

# Enable pretty printing for Rust types
set print pretty on
set print array on
set print array-indexes on

# TUI mode settings (uncomment to enable split-screen view)
# layout split
# layout regs

# Common breakpoints for kernel debugging
# Note: Uncomment the breakpoints you want to use
# break kernel_main
# break _start

# Panic handler breakpoint (useful for debugging crashes)
# break rust_begin_unwind

# Custom commands
define hook-stop
    # Print current instruction pointer and stack info on each stop
    printf "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
    printf "RIP: %p | RSP: %p | RBP: %p\n", $rip, $rsp, $rbp
    printf "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
    # Show next 10 instructions
    x/10i $rip
    printf "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
end

# Helper commands
define dump-regs
    printf "General Purpose Registers:\n"
    info registers rax rbx rcx rdx rsi rdi rbp rsp
    printf "\nSpecial Registers:\n"
    info registers rip rflags
    printf "\nSegment Registers:\n"
    info registers cs ss ds es fs gs
end

define dump-cr
    printf "Control Registers:\n"
    info registers cr0 cr2 cr3 cr4
end

define dump-stack
    if $argc == 0
        x/32xg $rsp
    else
        x/$arg0xg $rsp
    end
end

document dump-regs
Dump all general purpose, special, and segment registers
Usage: dump-regs
end

document dump-cr
Dump control registers (CR0, CR2, CR3, CR4)
Usage: dump-cr
end

document dump-stack
Dump stack contents
Usage: dump-stack [count]
  count: Number of 8-byte words to display (default: 32)
end

# Python helper scripts (if available)
# Load Python extensions for advanced kernel debugging
source scripts/gdb-helpers.py

# Auto-continue on first connection (optional)
# Uncomment the following line to automatically continue execution
# continue

echo \n
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n
echo   Yomi OS Kernel Debugger\n
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n
echo   Connected to QEMU on port 1234\n
echo   Symbol file loaded: target/x86_64-unknown-none/debug/yomi-kernel\n
echo \n
echo   Custom commands:\n
echo     dump-regs        - Show all registers\n
echo     dump-cr          - Show control registers\n
echo     dump-stack       - Show stack contents\n
echo     dump-pagetable   - Dump page table hierarchy\n
echo     dump-idt         - Dump Interrupt Descriptor Table\n
echo     dump-gdt         - Dump Global Descriptor Table\n
echo \n
echo   Ready to debug! Use 'c' to continue or set breakpoints.\n
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n
echo \n
