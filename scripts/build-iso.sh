#!/bin/bash
set -e

# Build the kernel
echo "Building kernel..."
cargo build --package yomi-kernel

# Create ISO directory structure
ISO_DIR="build/iso"
GRUB_DIR="${ISO_DIR}/boot/grub"

mkdir -p "${GRUB_DIR}"

# Copy kernel
echo "Copying kernel..."
cp target/x86_64-unknown-none/debug/yomi-kernel "${ISO_DIR}/boot/kernel.bin"

# Create grub.cfg
echo "Creating GRUB configuration..."
cat > "${GRUB_DIR}/grub.cfg" << EOF
set timeout=5
set default=0

# Enable serial console
serial --unit=0 --speed=115200
terminal_input console serial
terminal_output console serial

menuentry "YomiOS" {
    multiboot2 /boot/kernel.bin
    boot
}
EOF

# Create ISO using grub-mkrescue
echo "Creating ISO image..."
grub-mkrescue -o yomios.iso "${ISO_DIR}"

echo "ISO image created: yomios.iso"
