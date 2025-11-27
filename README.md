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
# Launch debug session (starts QEMU with GDB server)
cargo xtask debug

# Or run QEMU in debug mode (manual)
cargo xtask run --mode debug

# Then connect with GDB (in another terminal)
rust-gdb target/x86_64-unknown-none/debug/yomi-kernel
(gdb) target remote :1234
(gdb) continue
```

## 🧪 Testing

Run integration tests:

```bash
# Run all tests
cargo xtask test

# Run specific test
cargo xtask test --filter heap_allocation
```

For detailed debugging and testing instructions, see [docs/XTASK_MIGRATION.md](docs/XTASK_MIGRATION.md).

## 🤝 Contributing

Contributions are welcome! For details, please refer to [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

Copyright 2025 Yomi OS Development Team

---

**Yomi** - Read, Learn, Secure 🦀
