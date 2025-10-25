# YomiOS Testing Framework

This document describes the testing framework for YomiOS kernel.

## Overview

The testing framework uses QEMU for automated kernel testing with programmatic exit codes.

## Components

### 1. QEMU Exit Device

The framework uses the `isa-debug-exit` device to allow the kernel to exit QEMU with specific exit codes:

- `0x10` (Success): All tests passed
- `0x11` (Failed): At least one test failed

### 2. Test Runner Script

The `scripts/run-qemu.sh` script provides three modes:

```bash
# Normal run mode
./scripts/run-qemu.sh run

# Test mode (with isa-debug-exit device)
./scripts/run-qemu.sh test

# Debug mode (GDB server on port 1234)
./scripts/run-qemu.sh debug
```

### 3. Testing Framework (kernel/src/testing.rs)

The testing framework provides:

- `QemuExitCode` enum for test results
- `exit_qemu()` function to exit QEMU programmatically
- `Testable` trait for test functions
- `test_runner()` to execute all tests
- `test_panic_handler()` for test failures

## Running Tests

### Building and Running

```bash
# 1. Build the kernel
cargo build --manifest-path kernel/Cargo.toml

# 2. Create bootable ISO
./scripts/build-iso.sh

# 3. Run in QEMU (test mode)
./scripts/run-qemu.sh test
```

### Integration Tests

Integration tests are located in `kernel/tests/`:

- `basic_boot.rs`: Tests basic kernel boot and functionality
- `heap_allocation.rs`: Tests heap allocator and memory management

**Note**: Due to Rust limitations with `no_std` targets and `cargo test`, integration tests currently have build issues when using `cargo test`. They are designed to be run manually or through custom build scripts.

## Test Structure

### Writing a Test

```rust
#[test_case]
fn test_example() {
    assert_eq!(1 + 1, 2);
    // Test passes if it doesn't panic
}
```

### Integration Test Template

```rust
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(yomi_kernel::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    yomi_kernel::init();
    test_main();
    loop { unsafe { core::arch::asm!("hlt") } }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    yomi_kernel::testing::test_panic_handler(info)
}

#[test_case]
fn my_test() {
    // Test implementation
}
```

## Current Status

### Working Components

✅ QEMU runner script with test/run/debug modes
✅ Testing framework infrastructure (QemuExitCode, test_runner, etc.)
✅ Integration test structure (basic_boot.rs, heap_allocation.rs)
✅ GitHub Actions CI workflow
✅ ISO build system with GRUB
✅ Serial console enabled in GRUB

### Known Issues

⚠️ **Serial output not appearing**: The kernel boots successfully (GRUB menu appears and kernel is loaded), but serial output from the kernel is not displayed. This is being tracked in [Issue #013](../plan/issues/M1-kernel-boot/013-serial-console-debug.md).

⚠️ **Integration tests cannot be built with `cargo test`**: Due to Rust's limitations with bare-metal targets and duplicate lang items. This is a [known limitation](https://github.com/rust-lang/cargo/issues/7359) of Rust/Cargo.

### Debugging Steps

To debug the serial output issue:

```bash
# 1. Build ISO with serial console
./scripts/build-iso.sh

# 2. Run with QEMU (you should see GRUB menu on serial)
./scripts/run-qemu.sh test

# 3. Check if kernel is loaded (GRUB countdown should complete)
# 4. No kernel serial output appears (issue to investigate)
```

Possible solutions to investigate:
- Add early VGA text mode output before serial initialization
- Verify serial port is correctly initialized (check COM1 base address)
- Add assembly-level debugging output in boot code
- Test with different QEMU versions
- Investigate if paging setup affects serial I/O

## Known Limitations

1. **cargo test issues**: The standard `cargo test` command has issues with bare-metal targets due to duplicate lang items. This is a known Rust limitation.

2. **Workaround**: Tests must be run through custom scripts or manually built and executed in QEMU.

3. **Future improvements**: Consider implementing an `xtask` pattern for better test automation.

## CI/CD

GitHub Actions workflow (`.github/workflows/test.yml`) automates:

- Kernel building
- Code formatting checks (cargo fmt)
- Linting (cargo clippy)
- Test execution in QEMU

## Exit Code Mapping

QEMU's `isa-debug-exit` device maps exit codes as follows:

- Write `0x10` → QEMU exit code `33` (success)
- Write `0x11` → QEMU exit code `35` (failure)

The exit code formula is: `(value << 1) | 1`

## Future Enhancements

- [ ] Fix serial console output from kernel ([Issue #013](../plan/issues/M1-kernel-boot/013-serial-console-debug.md))
- [ ] Implement xtask pattern for test automation
- [ ] Add test coverage reporting
- [ ] Add performance benchmarking
- [ ] Add fuzzing support
- [ ] Improve integration test build process (resolve cargo test limitations)
- [ ] Add early boot debugging (VGA text mode fallback)

## Related Issues

- [#013 Serial Console Output Debugging](../plan/issues/M1-kernel-boot/013-serial-console-debug.md) - Fix kernel serial output (High Priority)
- [#010 QEMU Integration Testing](../plan/issues/M1-kernel-boot/010-qemu-testing.md) - Testing framework (Completed)
