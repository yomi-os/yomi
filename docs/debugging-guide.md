# Yomi OS Kernel Debugging Guide

This guide provides comprehensive instructions for debugging the Yomi OS kernel using GDB and QEMU.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Prerequisites](#prerequisites)
3. [Debugging Workflow](#debugging-workflow)
4. [GDB Commands Reference](#gdb-commands-reference)
5. [Common Debugging Scenarios](#common-debugging-scenarios)
6. [Advanced Techniques](#advanced-techniques)
7. [Troubleshooting](#troubleshooting)

## Quick Start

The fastest way to start debugging the kernel:

```bash
# Build the kernel and ISO image (if not already built)
cargo build --package yomi-kernel
./scripts/build-iso.sh

# Start debugging session
./scripts/debug.sh
```

This will:
1. Check that the kernel binary and ISO image exist
2. Start QEMU with GDB server enabled (paused at startup)
3. Launch GDB and connect to QEMU
4. Load kernel symbols and debugging helpers

**Note**: Both the kernel binary (for debug symbols) and ISO image (for booting) are required.

## Prerequisites

### Required Tools

```bash
# Install QEMU
sudo apt install qemu-system-x86

# Install GDB (rust-gdb recommended for better Rust support)
sudo apt install gdb

# Install rust-gdb (optional but recommended)
rustup component add rust-src
```

### Build the Kernel with Debug Symbols

Debug symbols are included by default in debug builds:

```bash
# Debug build (includes debug symbols)
cargo build --package yomi-kernel

# Build ISO image (required for booting)
./scripts/build-iso.sh

# Release build (optimized, but can still be debugged)
cargo build --package yomi-kernel --release
```

**Note**:
- Debug builds are located at `target/x86_64-unknown-none/debug/yomi-kernel`
- The kernel uses Multiboot2 format and must be booted via ISO image
- GDB needs the kernel binary for symbols, but QEMU boots from the ISO

## Debugging Workflow

### Method 1: Using debug.sh Script (Recommended)

The `debug.sh` script automates the entire setup:

```bash
./scripts/debug.sh
```

This script:
- Validates the kernel binary exists
- Starts QEMU in debug mode (paused)
- Launches GDB with proper configuration
- Automatically loads `.gdbinit` settings
- Connects to QEMU GDB server

### Method 2: Manual Setup

For more control, you can start QEMU and GDB separately:

**Terminal 1 - Start QEMU with GDB server:**
```bash
./scripts/run-qemu.sh debug

# Or manually:
qemu-system-x86_64 \
    -kernel target/x86_64-unknown-none/debug/yomi-kernel \
    -s \
    -S \
    -serial stdio \
    -display none \
    -no-reboot \
    -m 256M
```

**Terminal 2 - Start GDB:**
```bash
# Using rust-gdb (recommended)
rust-gdb target/x86_64-unknown-none/debug/yomi-kernel

# Or using standard gdb
gdb target/x86_64-unknown-none/debug/yomi-kernel
```

**In GDB:**
```gdb
(gdb) target remote :1234
(gdb) break kernel_main
(gdb) continue
```

## GDB Commands Reference

### Basic Commands

| Command | Description | Example |
|---------|-------------|---------|
| `c` or `continue` | Continue execution | `(gdb) c` |
| `s` or `step` | Step into function | `(gdb) s` |
| `n` or `next` | Step over function | `(gdb) n` |
| `finish` | Run until current function returns | `(gdb) finish` |
| `bt` or `backtrace` | Show call stack | `(gdb) bt` |
| `info registers` | Show all registers | `(gdb) info registers` |
| `info frame` | Show current stack frame | `(gdb) info frame` |
| `quit` | Exit GDB | `(gdb) quit` |

### Breakpoints

```gdb
# Set breakpoint at function
(gdb) break kernel_main
(gdb) break yomi_kernel::boot::multiboot2::parse_boot_info

# Set breakpoint at address
(gdb) break *0xffffffff80001000

# Set breakpoint with condition
(gdb) break kernel_main if some_var == 42

# List all breakpoints
(gdb) info breakpoints

# Delete breakpoint
(gdb) delete 1

# Enable/disable breakpoint
(gdb) disable 1
(gdb) enable 1

# Set temporary breakpoint (deleted after hit)
(gdb) tbreak kernel_main
```

### Watchpoints

```gdb
# Break when memory location is written
(gdb) watch *0xffffffff80100000

# Break when memory location is read
(gdb) rwatch *0xffffffff80100000

# Break when memory location is read or written
(gdb) awatch *0xffffffff80100000

# Watch variable
(gdb) watch my_variable
```

### Memory Inspection

```gdb
# Examine memory at address (hex format, 16 bytes)
(gdb) x/16xb 0xffffffff80001000

# Examine as instructions
(gdb) x/10i $rip

# Examine as 64-bit words
(gdb) x/8xg $rsp

# Print variable
(gdb) print my_variable
(gdb) print *my_pointer

# Print in different formats
(gdb) print/x my_variable  # Hexadecimal
(gdb) print/t my_variable  # Binary
(gdb) print/d my_variable  # Decimal
```

### Custom Commands (from .gdbinit)

These commands are provided by the Yomi OS `.gdbinit` configuration:

| Command | Description | Example |
|---------|-------------|---------|
| `dump-regs` | Show all general purpose, special, and segment registers | `(gdb) dump-regs` |
| `dump-cr` | Show control registers (CR0, CR2, CR3, CR4) | `(gdb) dump-cr` |
| `dump-stack [count]` | Show stack contents | `(gdb) dump-stack 64` |
| `dump-pagetable [addr]` | Dump page table hierarchy | `(gdb) dump-pagetable 0xffffffff80001000` |
| `dump-idt [count]` | Dump Interrupt Descriptor Table | `(gdb) dump-idt 256` |
| `dump-gdt [count]` | Dump Global Descriptor Table | `(gdb) dump-gdt 16` |

### TUI Mode (Text User Interface)

GDB's TUI mode provides a split-screen view with source code:

```gdb
# Enable TUI mode
(gdb) tui enable

# Or use layout commands
(gdb) layout split    # Source + assembly
(gdb) layout regs     # Registers + source
(gdb) layout asm      # Assembly only

# Navigate TUI
Ctrl+X, A            # Toggle TUI mode
Ctrl+X, 2            # Switch layout
Ctrl+L               # Refresh screen
```

## Common Debugging Scenarios

### Debugging Boot Process

```gdb
# Set breakpoint at kernel entry point
(gdb) break _start
(gdb) continue

# Step through boot sequence
(gdb) next
(gdb) step

# Check boot information
(gdb) print boot_info
(gdb) print multiboot_info
```

### Debugging Page Faults

```gdb
# Set breakpoint at page fault handler
(gdb) break page_fault_handler

# When hit, examine CR2 (faulting address)
(gdb) dump-cr
(gdb) print $cr2

# Examine page tables
(gdb) dump-pagetable $cr2

# Check error code
(gdb) print error_code
```

### Debugging Panics

```gdb
# Set breakpoint at panic handler
(gdb) break rust_begin_unwind

# Or break at panic macro
(gdb) break core::panicking::panic

# Examine backtrace
(gdb) backtrace full

# Print panic message
(gdb) print msg
```

### Debugging Interrupts

```gdb
# Examine IDT
(gdb) dump-idt

# Set breakpoint on interrupt handler
(gdb) break timer_interrupt_handler
(gdb) break keyboard_interrupt_handler

# Check interrupt state
(gdb) print $rflags
```

### Debugging Memory Allocation

```gdb
# Break on allocation function
(gdb) break __rust_alloc
(gdb) break __rust_dealloc

# Watch heap pointer
(gdb) watch heap_start

# Examine heap state
(gdb) print allocator
```

## Advanced Techniques

### Scripting GDB

Create a GDB script file (e.g., `debug-script.gdb`):

```gdb
# debug-script.gdb
target remote :1234
break kernel_main
commands
    print "Kernel main reached!"
    backtrace
    continue
end
continue
```

Run GDB with script:
```bash
gdb -x debug-script.gdb target/x86_64-unknown-none/debug/yomi-kernel
```

### Conditional Breakpoints

```gdb
# Break only when condition is true
(gdb) break memory_allocator.rs:42 if size > 1024

# Break after N hits
(gdb) ignore 1 10  # Ignore breakpoint 1 for 10 hits
```

### Logging Output

```gdb
# Log all GDB output to file
(gdb) set logging on
(gdb) set logging file debug.log

# Log only command output
(gdb) set logging redirect on
```

### Examining Rust Data Structures

```gdb
# Print Rust Option
(gdb) print my_option

# Print Rust Result
(gdb) print my_result

# Print Rust Vec
(gdb) print my_vec

# Access Rust struct fields
(gdb) print my_struct.field_name
```

### Remote Debugging

For debugging on a different machine:

**On target machine (running QEMU):**
```bash
qemu-system-x86_64 \
    -kernel kernel \
    -gdb tcp::1234 \
    -S
```

**On development machine:**
```bash
gdb kernel
(gdb) target remote target-ip:1234
```

## Troubleshooting

### GDB Cannot Connect to QEMU

**Problem**: `Connection refused` when connecting to `:1234`

**Solutions**:
1. Verify QEMU is running with `-s` flag
2. Check if port 1234 is in use: `lsof -i :1234`
3. Try specifying full address: `target remote localhost:1234`

### No Debug Symbols

**Problem**: `No debugging symbols found`

**Solutions**:
1. Ensure you're using debug build: `cargo build --package yomi-kernel`
2. Check if binary is stripped: `file target/x86_64-unknown-none/debug/yomi-kernel`
3. Verify symbol file path in `.gdbinit`

### Breakpoint Not Hit

**Problem**: Breakpoint set but never triggered

**Solutions**:
1. Verify function name is correct: `info functions kernel_main`
2. Check if code is actually executed: use `break *address` with exact address
3. Try setting breakpoint after symbols are loaded

### Python Extensions Not Loading

**Problem**: `dump-pagetable` and other Python commands not available

**Solutions**:
1. Check GDB has Python support: `gdb --configuration | grep python`
2. Verify path to `gdb-helpers.py` in `.gdbinit`
3. Check for Python errors when loading: `source scripts/gdb-helpers.py`

### QEMU Freezes

**Problem**: QEMU appears frozen after starting with `-S`

**Solutions**:
1. This is normal! QEMU waits for GDB to connect and issue `continue`
2. Connect GDB and type `c` or `continue`
3. Or remove `-S` flag if you don't want to pause at startup

## Best Practices

1. **Use `rust-gdb`**: Better Rust type visualization than standard GDB
2. **Save your session**: Use GDB's history and logging features
3. **Create custom commands**: Add frequently-used sequences to `.gdbinit`
4. **Use watchpoints**: Great for tracking memory corruption
5. **Leverage Python**: Write custom GDB Python scripts for complex analysis
6. **Document your findings**: Keep notes of debugging sessions

## Additional Resources

- [GDB Documentation](https://sourceware.org/gdb/documentation/)
- [OSDev GDB Wiki](https://wiki.osdev.org/GDB)
- [QEMU GDB Usage](https://qemu.readthedocs.io/en/latest/system/gdb.html)
- [Debugging with GDB (PDF)](https://sourceware.org/gdb/current/onlinedocs/gdb.pdf)
- [Rust GDB Pretty Printers](https://github.com/rust-lang/rust/tree/master/src/etc)

## See Also

- [Build System Documentation](../plan/ja/42-BUILD-SYSTEM.md)
- [Testing Strategy](../plan/ja/43-TESTING-STRATEGY.md)
- [Kernel Design](../plan/ja/10-KERNEL-DESIGN.md)
