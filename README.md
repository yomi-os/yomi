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

## 🤝 Contributing

Contributions are welcome! For details, please refer to [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

MIT License

---

**Yomi** - Read, Learn, Secure 🦀
