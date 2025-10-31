# xtask Build System Migration

## Overview

Successfully migrated from Bash-based automation scripts to Rust-based xtask build system. The xtask pattern provides better maintainability, type safety, cross-platform support, and improved error handling.

## What Changed

### Before (Bash Scripts)
- `scripts/run-qemu.sh` - QEMU runner with 3 modes
- `scripts/build-iso.sh` - ISO image creation
- `scripts/run-tests.sh` - Integration test runner
- `scripts/debug.sh` - GDB debug launcher

### After (Rust xtask)
- `cargo xtask build` - Build kernel
- `cargo xtask iso` - Create bootable ISO
- `cargo xtask run` - Run in QEMU (normal mode)
- `cargo xtask run --mode test` - Run in test mode
- `cargo xtask run --mode debug` - Run in debug mode with GDB server
- `cargo xtask test` - Run integration tests
- `cargo xtask debug` - Launch full GDB debug session
- `cargo xtask clean` - Clean build artifacts

## New Commands

### Build Commands

```bash
# Build kernel (debug)
cargo xtask build

# Build kernel (release)
cargo xtask build --release

# Create bootable ISO image
cargo xtask iso

# Create release ISO
cargo xtask iso --release

# Clean build artifacts
cargo xtask clean
```

### Run Commands

```bash
# Run kernel in QEMU (normal mode)
cargo xtask run

# Run in test mode (with isa-debug-exit device)
cargo xtask run --mode test

# Run with GDB server (waits for GDB connection)
cargo xtask run --mode debug

# Run release build
cargo xtask run --release
```

### Test Commands

```bash
# Run all integration tests
cargo xtask test

# Run tests matching a filter
cargo xtask test --filter heap

# Run specific test
cargo xtask test --filter basic_boot
```

### Debug Commands

```bash
# Launch kernel with GDB (starts QEMU + provides GDB instructions)
cargo xtask debug

# Debug release build
cargo xtask debug --release
```

## Architecture

### Module Structure

```
xtask/
├── Cargo.toml          # Dependencies: clap, anyhow, colored
└── src/
    ├── main.rs         # CLI argument parsing with clap
    ├── build.rs        # Kernel build logic
    ├── iso.rs          # ISO creation with GRUB
    ├── qemu.rs         # QEMU execution (3 modes)
    ├── test.rs         # Integration test runner
    ├── debug.rs        # GDB debug session launcher
    └── util.rs         # Common utilities
```

### Key Features

1. **Type Safety**: Rust's type system catches errors at compile time
2. **Better Error Messages**: Clear, colored output with helpful hints
3. **Cross-Platform**: Works on Linux, macOS, Windows (with appropriate tools)
4. **Maintainable**: Modular code structure, easy to extend
5. **Integrated**: Proper workspace member, uses cargo's build system

## Technical Details

### Workspace Integration

Added to workspace in `Cargo.toml`:
```toml
[workspace]
members = ["kernel", "xtask"]
```

Added cargo alias in `.cargo/config.toml`:
```toml
[alias]
xtask = "run --package xtask --"
```

### Target Configuration

Removed global target override to allow xtask to build for host:
- Kernel builds explicitly specify `--target x86_64-unknown-none`
- xtask builds for host (normal Rust std binary)
- Used `-Z build-std` flags for kernel bare-metal compilation

### QEMU Modes

1. **Run Mode** (normal):
   - Standard QEMU execution
   - Serial output to stdio
   - Interactive kernel

2. **Test Mode**:
   - Adds `isa-debug-exit` device for programmatic exit
   - Exit code 33 = success (0x10 << 1 | 1)
   - Non-interactive, suitable for CI/CD

3. **Debug Mode**:
   - GDB server on port 1234
   - Pauses at startup (`-S` flag)
   - Wait for GDB connection

### ISO Creation Process

1. Build kernel binary
2. Create ISO directory structure (`build/iso/boot/grub/`)
3. Copy kernel to `boot/kernel.bin`
4. Generate GRUB configuration with serial console support
5. Run `grub-mkrescue` to create bootable ISO

### Test Execution

1. Discover test files in `kernel/tests/`
2. Build each test binary with appropriate linker flags
3. Run in QEMU with test mode
4. Parse exit codes (33 = pass, other = fail)
5. Report results with colored output

## Migration Notes

### For Developers

**Old workflow:**
```bash
./scripts/build-iso.sh
./scripts/run-qemu.sh run
./scripts/run-tests.sh
```

**New workflow:**
```bash
cargo xtask iso
cargo xtask run
cargo xtask test
```

### Backward Compatibility

The old Bash scripts are still present for reference and backward compatibility during the transition period. They will be marked as deprecated in a future release.

### CI/CD Updates

Update your CI/CD pipelines to use xtask commands:

```yaml
# Old
- run: ./scripts/build-iso.sh
- run: ./scripts/run-tests.sh

# New
- run: cargo xtask iso
- run: cargo xtask test
```

## Benefits

### 1. Developer Experience
- Single command interface: `cargo xtask <command>`
- Better error messages with colors and context
- Auto-completion support (via clap)
- Consistent interface across commands

### 2. Maintainability
- Type-safe code instead of shell scripts
- Modular architecture (separate file per concern)
- Easy to extend with new commands
- Unit testable functions

### 3. Cross-Platform
- Rust runs on Windows/macOS/Linux
- Can detect platform-specific requirements
- Graceful error messages with installation hints

### 4. Error Handling
- Proper error propagation with `anyhow`
- Context for each error
- Helpful suggestions on failure

## Future Enhancements

Possible future additions to xtask:

- [ ] Watch mode for development (`cargo xtask watch`)
- [ ] Parallel test execution
- [ ] Code coverage integration
- [ ] Performance profiling mode
- [ ] Automatic dependency checking
- [ ] Release automation

## Troubleshooting

### "cargo: command not found"
Install Rust: https://rustup.rs/

### "qemu-system-x86_64: command not found"
```bash
sudo apt install qemu-system-x86
```

### "grub-mkrescue: command not found"
```bash
sudo apt install grub-pc-bin xorriso
```

### Build fails with "can't find crate for std"
Make sure you're using nightly Rust:
```bash
rustup toolchain install nightly
rustup default nightly
rustup component add rust-src
```

## References

- [xtask Pattern](https://github.com/matklad/cargo-xtask)
- [Issue #29](https://github.com/yomi-os/yomi/issues/29)
- [Cargo Book - Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [clap Documentation](https://docs.rs/clap)

## Related Documentation

- `docs/42-BUILD-SYSTEM.md` - Build system design
- `docs/43-TESTING-STRATEGY.md` - Testing strategy
- `README.md` - Main project documentation