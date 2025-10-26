#!/usr/bin/env python3
"""
GDB Helper Functions for Yomi OS Kernel Debugging

This module provides custom GDB commands for inspecting kernel-specific
data structures and state.

Usage:
    Load this file in GDB with:
        source scripts/gdb-helpers.py

    Or add to .gdbinit:
        source scripts/gdb-helpers.py
"""

import gdb
import struct


class YomiGdbCommand(gdb.Command):
    """Base class for Yomi OS GDB commands"""

    def __init__(self, name, command_class=gdb.COMMAND_USER):
        super().__init__(name, command_class)

    def read_u64(self, address):
        """Read 64-bit unsigned integer from memory"""
        try:
            return int(gdb.parse_and_eval(f"*(unsigned long long *){address:#x}"))
        except gdb.MemoryError:
            return None

    def read_u32(self, address):
        """Read 32-bit unsigned integer from memory"""
        try:
            return int(gdb.parse_and_eval(f"*(unsigned int *){address:#x}"))
        except gdb.MemoryError:
            return None

    def read_bytes(self, address, count):
        """Read bytes from memory"""
        try:
            inferior = gdb.selected_inferior()
            return inferior.read_memory(address, count)
        except gdb.MemoryError:
            return None


class DumpPageTable(YomiGdbCommand):
    """Dump x86_64 page table entries

    Usage: dump-pagetable [address]
        address: Virtual address to translate (optional, uses current CR3 if not specified)

    This command walks the 4-level page table hierarchy and displays:
    - PML4 (Page Map Level 4)
    - PDPT (Page Directory Pointer Table)
    - PD (Page Directory)
    - PT (Page Table)
    """

    def __init__(self):
        super().__init__("dump-pagetable", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        args = gdb.string_to_argv(arg)

        # Get CR3 register value (physical address of PML4)
        try:
            cr3_value = int(gdb.parse_and_eval("$cr3"))
        except gdb.error:
            print("Error: Cannot read CR3 register")
            return

        # Mask off lower 12 bits to get page table base
        pml4_base = cr3_value & ~0xFFF

        print("=" * 80)
        print(f"Page Table Walk (CR3: {cr3_value:#018x})")
        print(f"PML4 Base: {pml4_base:#018x}")
        print("=" * 80)

        if len(args) > 0:
            try:
                virt_addr = int(args[0], 0)
                self.walk_page_table(pml4_base, virt_addr)
            except ValueError:
                print(f"Error: Invalid address '{args[0]}'")
        else:
            # Dump first few PML4 entries
            self.dump_pml4_entries(pml4_base, 16)

    def walk_page_table(self, pml4_base, virt_addr):
        """Walk page table for a specific virtual address"""
        print(f"\nTranslating virtual address: {virt_addr:#018x}")
        print("-" * 80)

        # Extract page table indices from virtual address
        pml4_index = (virt_addr >> 39) & 0x1FF
        pdpt_index = (virt_addr >> 30) & 0x1FF
        pd_index = (virt_addr >> 21) & 0x1FF
        pt_index = (virt_addr >> 12) & 0x1FF
        offset = virt_addr & 0xFFF

        print(f"Indices: PML4[{pml4_index}] -> PDPT[{pdpt_index}] -> "
              f"PD[{pd_index}] -> PT[{pt_index}] + {offset:#x}")
        print()

        # Read PML4 entry
        pml4_entry_addr = pml4_base + (pml4_index * 8)
        pml4_entry = self.read_u64(pml4_entry_addr)

        if pml4_entry is None:
            print("Error: Cannot read PML4 entry")
            return

        print(f"PML4[{pml4_index}] @ {pml4_entry_addr:#018x}: {pml4_entry:#018x}")
        self.print_page_entry_flags(pml4_entry)

        if not (pml4_entry & 0x1):
            print("  → Page not present")
            return

        # Read PDPT entry
        pdpt_base = pml4_entry & ~0xFFF
        pdpt_entry_addr = pdpt_base + (pdpt_index * 8)
        pdpt_entry = self.read_u64(pdpt_entry_addr)

        if pdpt_entry is None:
            print("Error: Cannot read PDPT entry")
            return

        print(f"PDPT[{pdpt_index}] @ {pdpt_entry_addr:#018x}: {pdpt_entry:#018x}")
        self.print_page_entry_flags(pdpt_entry)

        if not (pdpt_entry & 0x1):
            print("  → Page not present")
            return

        # Check for 1GB page
        if pdpt_entry & 0x80:
            phys_addr = (pdpt_entry & ~0x3FFFFFFF) | (virt_addr & 0x3FFFFFFF)
            print(f"  → 1GB page, physical address: {phys_addr:#018x}")
            return

        # Read PD entry
        pd_base = pdpt_entry & ~0xFFF
        pd_entry_addr = pd_base + (pd_index * 8)
        pd_entry = self.read_u64(pd_entry_addr)

        if pd_entry is None:
            print("Error: Cannot read PD entry")
            return

        print(f"PD[{pd_index}] @ {pd_entry_addr:#018x}: {pd_entry:#018x}")
        self.print_page_entry_flags(pd_entry)

        if not (pd_entry & 0x1):
            print("  → Page not present")
            return

        # Check for 2MB page
        if pd_entry & 0x80:
            phys_addr = (pd_entry & ~0x1FFFFF) | (virt_addr & 0x1FFFFF)
            print(f"  → 2MB page, physical address: {phys_addr:#018x}")
            return

        # Read PT entry
        pt_base = pd_entry & ~0xFFF
        pt_entry_addr = pt_base + (pt_index * 8)
        pt_entry = self.read_u64(pt_entry_addr)

        if pt_entry is None:
            print("Error: Cannot read PT entry")
            return

        print(f"PT[{pt_index}] @ {pt_entry_addr:#018x}: {pt_entry:#018x}")
        self.print_page_entry_flags(pt_entry)

        if not (pt_entry & 0x1):
            print("  → Page not present")
            return

        # Calculate final physical address
        phys_addr = (pt_entry & ~0xFFF) | offset
        print(f"  → 4KB page, physical address: {phys_addr:#018x}")

    def dump_pml4_entries(self, pml4_base, count):
        """Dump first N PML4 entries"""
        print(f"\nFirst {count} PML4 entries:")
        print("-" * 80)

        for i in range(count):
            entry_addr = pml4_base + (i * 8)
            entry = self.read_u64(entry_addr)

            if entry is None:
                print(f"PML4[{i:3d}]: Error reading memory")
                continue

            if entry & 0x1:  # Present bit
                print(f"PML4[{i:3d}] @ {entry_addr:#018x}: {entry:#018x} "
                      f"[P:{(entry & 0x1) != 0} W:{(entry & 0x2) != 0} "
                      f"U:{(entry & 0x4) != 0} NX:{(entry & 0x8000000000000000) != 0}]")

    @staticmethod
    def print_page_entry_flags(entry):
        """Print page table entry flags"""
        flags = []
        if entry & 0x1:
            flags.append("P")  # Present
        if entry & 0x2:
            flags.append("W")  # Writable
        if entry & 0x4:
            flags.append("U")  # User
        if entry & 0x8:
            flags.append("PWT")  # Write-Through
        if entry & 0x10:
            flags.append("PCD")  # Cache Disable
        if entry & 0x20:
            flags.append("A")  # Accessed
        if entry & 0x40:
            flags.append("D")  # Dirty
        if entry & 0x80:
            flags.append("PS")  # Page Size
        if entry & 0x100:
            flags.append("G")  # Global
        if entry & 0x8000000000000000:
            flags.append("NX")  # No Execute

        print(f"  Flags: [{' | '.join(flags) if flags else 'None'}]")


class DumpIDT(YomiGdbCommand):
    """Dump Interrupt Descriptor Table (IDT)

    Usage: dump-idt [count]
        count: Number of IDT entries to display (default: 256)

    This command displays the IDT entries, showing:
    - Entry number
    - Handler address
    - Segment selector
    - Gate type
    - DPL (Descriptor Privilege Level)
    - Present flag
    """

    def __init__(self):
        super().__init__("dump-idt", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        args = gdb.string_to_argv(arg)
        count = int(args[0]) if len(args) > 0 else 256

        # Get IDTR (IDT register)
        try:
            # Read IDTR - this is a 10-byte structure (2 bytes limit + 8 bytes base)
            idtr_output = gdb.execute("info registers idtr", to_string=True)

            # Parse IDTR output
            # Format: idtr           {base=0x..., limit=0x...}
            import re
            match = re.search(r'base=(0x[0-9a-fA-F]+).*limit=(0x[0-9a-fA-F]+)', idtr_output)

            if not match:
                print("Error: Cannot parse IDTR")
                print(f"IDTR output: {idtr_output}")
                return

            idt_base = int(match.group(1), 16)
            idt_limit = int(match.group(2), 16)

        except gdb.error as e:
            print(f"Error reading IDTR: {e}")
            return

        max_entries = min(count, (idt_limit + 1) // 16)

        print("=" * 80)
        print(f"Interrupt Descriptor Table (IDT)")
        print(f"Base: {idt_base:#018x}, Limit: {idt_limit:#06x}")
        print(f"Max entries: {max_entries}")
        print("=" * 80)

        for i in range(max_entries):
            self.print_idt_entry(idt_base, i)

    def print_idt_entry(self, idt_base, index):
        """Print a single IDT entry"""
        entry_addr = idt_base + (index * 16)  # Each IDT entry is 16 bytes

        # Read 16 bytes for IDT entry
        entry_bytes = self.read_bytes(entry_addr, 16)

        if entry_bytes is None:
            print(f"IDT[{index:3d}]: Error reading memory")
            return

        # Parse IDT entry structure
        offset_low = struct.unpack("<H", entry_bytes[0:2])[0]
        selector = struct.unpack("<H", entry_bytes[2:4])[0]
        ist = entry_bytes[4] & 0x7
        type_attr = entry_bytes[5]
        offset_mid = struct.unpack("<H", entry_bytes[6:8])[0]
        offset_high = struct.unpack("<I", entry_bytes[8:12])[0]

        # Construct full offset
        offset = (offset_high << 32) | (offset_mid << 16) | offset_low

        # Parse type and attributes
        present = (type_attr & 0x80) != 0
        dpl = (type_attr >> 5) & 0x3
        gate_type = type_attr & 0xF

        gate_type_names = {
            0xE: "Interrupt",
            0xF: "Trap",
        }
        gate_type_name = gate_type_names.get(gate_type, f"Unknown({gate_type:#x})")

        if present and offset != 0:
            print(f"IDT[{index:3d}]: {offset:#018x} "
                  f"(Selector: {selector:#06x}, Type: {gate_type_name}, "
                  f"DPL: {dpl}, IST: {ist}, P: {present})")


class DumpGDT(YomiGdbCommand):
    """Dump Global Descriptor Table (GDT)

    Usage: dump-gdt [count]
        count: Number of GDT entries to display (default: 16)
    """

    def __init__(self):
        super().__init__("dump-gdt", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        args = gdb.string_to_argv(arg)
        count = int(args[0]) if len(args) > 0 else 16

        # Get GDTR
        try:
            gdtr_output = gdb.execute("info registers gdtr", to_string=True)

            import re
            match = re.search(r'base=(0x[0-9a-fA-F]+).*limit=(0x[0-9a-fA-F]+)', gdtr_output)

            if not match:
                print("Error: Cannot parse GDTR")
                return

            gdt_base = int(match.group(1), 16)
            gdt_limit = int(match.group(2), 16)

        except gdb.error as e:
            print(f"Error reading GDTR: {e}")
            return

        max_entries = min(count, (gdt_limit + 1) // 8)

        print("=" * 80)
        print(f"Global Descriptor Table (GDT)")
        print(f"Base: {gdt_base:#018x}, Limit: {gdt_limit:#06x}")
        print(f"Max entries: {max_entries}")
        print("=" * 80)

        for i in range(max_entries):
            self.print_gdt_entry(gdt_base, i)

    def print_gdt_entry(self, gdt_base, index):
        """Print a single GDT entry"""
        entry_addr = gdt_base + (index * 8)
        entry = self.read_u64(entry_addr)

        if entry is None:
            print(f"GDT[{index:3d}]: Error reading memory")
            return

        if entry == 0:
            if index == 0:
                print(f"GDT[{index:3d}]: NULL descriptor")
            return

        # Parse descriptor
        present = (entry >> 47) & 0x1
        dpl = (entry >> 45) & 0x3
        desc_type = (entry >> 44) & 0x1
        segment_type = (entry >> 40) & 0xF

        if present:
            print(f"GDT[{index:3d}]: {entry:#018x} "
                  f"(DPL: {dpl}, Type: {segment_type:#x}, P: {present != 0})")


# Register all commands
DumpPageTable()
DumpIDT()
DumpGDT()

print("[GDB] Yomi OS kernel debugging helpers loaded")
print("[GDB] Available commands:")
print("  dump-pagetable [address] - Dump page table hierarchy")
print("  dump-idt [count]         - Dump Interrupt Descriptor Table")
print("  dump-gdt [count]         - Dump Global Descriptor Table")
