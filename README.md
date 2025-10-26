# Yomi

**Next-Generation Security OS**

A dedicated OS for security research and penetration testing, integrating Redox's safety with Linux's practicality.

## 🚀 Quick Start

### Requirements

- Rust (nightly)
- QEMU
- GRUB tools

### Setup

```bash
# Install dependencies
./scripts/setup-dev.sh

# Build
cargo xtask build

# Run in QEMU
cargo xtask run
```

## 🎯 Features

- **Safety**: 100% Rust with memory-safe design
- **Microkernel**: Isolated architecture based on the principle of least privilege
- **eBPF Drivers**: Statically verified safe driver framework
- **Security Tool Integration**: OS-level security APIs

## 🐛 Debugging

Debug the kernel with GDB:

```bash
# Build kernel and ISO (if not already built)
cargo build --package yomi-kernel
./scripts/build-iso.sh

# Quick start debugging
./scripts/debug.sh

# Or manually start QEMU in debug mode
./scripts/run-qemu.sh debug

# Then connect with GDB (in another terminal)
rust-gdb target/x86_64-unknown-none/debug/yomi-kernel
```

For detailed debugging instructions, see the [Debugging Guide](docs/debugging-guide.md).

## 🤝 Contributing

Contributions are welcome! For details, please refer to [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

Copyright 2025 Yomi OS Development Team

---

**Yomi** - Read, Learn, Secure 🦀
