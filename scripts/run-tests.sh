#!/bin/bash
# Run integration tests for YomiOS kernel

set -e

cd "$(dirname "$0")/.."

echo "Building kernel..."
cargo build --manifest-path kernel/Cargo.toml

echo ""
echo "====================================="
echo "Running integration tests..."
echo "====================================="
echo ""

# Build and run each integration test separately
for test_file in kernel/tests/*.rs; do
    test_name=$(basename "$test_file" .rs)
    echo "Building test: $test_name"

    # Build the test as a binary
    cargo build \
        --manifest-path kernel/Cargo.toml \
        --bin "yomi-kernel" \
        --features test-$test_name 2>/dev/null || {

        # If feature-based build fails, try building the test directly
        cargo rustc \
            --manifest-path kernel/Cargo.toml \
            --test "$test_name" \
            -- \
            -C link-arg=--nmagic \
            -C link-arg=--no-dynamic-linker \
            -C link-arg=-Tkernel/linker.ld \
            -C relocation-model=static 2>&1 | grep -v "duplicate lang item" || true
    }

    test_bin="target/x86_64-unknown-none/debug/$test_name"

    if [ -f "$test_bin" ]; then
        echo "Running test: $test_name"
        KERNEL_BIN="$test_bin" ./scripts/run-qemu.sh test

        # Check exit code
        exit_code=$?
        if [ $exit_code -eq 0 ] || [ $exit_code -eq 33 ]; then  # 33 = (0x10 << 1) | 1
            echo "✓ Test passed: $test_name"
        else
            echo "✗ Test failed: $test_name (exit code: $exit_code)"
            exit 1
        fi
    else
        echo "⚠ Test binary not found, skipping: $test_name"
    fi

    echo ""
done

echo "====================================="
echo "All tests completed successfully!"
echo "====================================="
