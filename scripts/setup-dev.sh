#!/bin/bash

echo "🦀 Setting up Yomi development environment..."

# Check for Rust installation
if ! command -v rustup &> /dev/null; then
    echo "❌ Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
fi

# Install nightly toolchain
rustup toolchain install nightly
rustup default nightly

# Add required components
rustup component add rust-src
rustup component add llvm-tools-preview
rustup component add rustfmt
rustup component add clippy

# Install cargo tools
cargo install bootimage || true

# Install system packages (Ubuntu/Debian)
if command -v apt &> /dev/null; then
    echo "📦 Installing system packages..."
    sudo apt update
    sudo apt install -y qemu-system-x86 nasm grub-pc-bin xorriso build-essential
fi

echo "✓ Setup complete!"
echo ""
echo "Next steps:"
echo "  cargo xtask build    # Build the kernel"
echo "  cargo xtask run      # Run in QEMU"
