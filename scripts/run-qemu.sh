#!/bin/bash
# QEMU runner script for YomiOS kernel testing

set -e

# Check if ISO image or kernel binary should be used
ISO_IMAGE="${ISO_IMAGE:-yomios.iso}"
KERNEL_BIN="${KERNEL_BIN:-}"
MODE="${1:-run}"

# Remove the mode argument from positional parameters
if [ $# -gt 0 ]; then
    shift
fi

# Determine boot method
USE_ISO=false
if [ -f "$ISO_IMAGE" ] && [ -z "$KERNEL_BIN" ]; then
    USE_ISO=true
    BOOT_IMAGE="$ISO_IMAGE"
elif [ -n "$KERNEL_BIN" ] && [ -f "$KERNEL_BIN" ]; then
    USE_ISO=false
    BOOT_IMAGE="$KERNEL_BIN"
elif [ -f "$ISO_IMAGE" ]; then
    USE_ISO=true
    BOOT_IMAGE="$ISO_IMAGE"
else
    echo "Error: Neither ISO image nor kernel binary found"
    echo "ISO: $ISO_IMAGE"
    echo "Kernel: $KERNEL_BIN"
    echo ""
    echo "Build ISO with: ./scripts/build-iso.sh"
    echo "Or build kernel with: cargo build --package yomi-kernel"
    exit 1
fi

# QEMU base command
QEMU_CMD="qemu-system-x86_64"

# Common QEMU options
QEMU_OPTS=(
    -serial stdio
    -no-reboot
    -m 256M
)

# Add boot option based on method
if [ "$USE_ISO" = true ]; then
    echo "Booting from ISO: $BOOT_IMAGE"
    QEMU_OPTS+=(-cdrom "$BOOT_IMAGE")
else
    echo "Booting from kernel: $BOOT_IMAGE"
    QEMU_OPTS+=(-kernel "$BOOT_IMAGE")
fi

case "$MODE" in
    test)
        # Test mode: add isa-debug-exit device for programmatic exit
        QEMU_OPTS+=(
            -device isa-debug-exit,iobase=0xf4,iosize=0x04
            -no-shutdown
        )
        ;;
    debug)
        # Debug mode: enable GDB server
        QEMU_OPTS+=(
            -s
            -S
        )
        echo "QEMU waiting for GDB connection on port 1234..."
        ;;
    run)
        # Normal run mode
        ;;
    *)
        echo "Usage: $0 [run|test|debug]"
        exit 1
        ;;
esac

# Run QEMU
exec "$QEMU_CMD" "${QEMU_OPTS[@]}" "$@"
