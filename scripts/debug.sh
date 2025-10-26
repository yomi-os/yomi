#!/bin/bash
# GDB debugging launcher for Yomi OS kernel
# This script starts QEMU with GDB server and launches GDB with proper configuration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
KERNEL_BIN="${KERNEL_BIN:-target/x86_64-unknown-none/debug/yomi-kernel}"
ISO_IMAGE="${ISO_IMAGE:-yomios.iso}"
GDB_PORT="${GDB_PORT:-1234}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
QEMU_LOG="${QEMU_LOG:-/tmp/yomi-qemu-debug.log}"

# Function to print colored messages
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to cleanup on exit
cleanup() {
    if [ -n "$QEMU_PID" ] && kill -0 "$QEMU_PID" 2>/dev/null; then
        print_info "Stopping QEMU (PID: $QEMU_PID)..."
        kill "$QEMU_PID" 2>/dev/null || true
        wait "$QEMU_PID" 2>/dev/null || true
    fi

    # Clean up any stray QEMU processes
    pkill -f "qemu-system-x86_64.*$KERNEL_BIN" 2>/dev/null || true

    print_success "Cleanup completed"
}

# Register cleanup function
trap cleanup EXIT INT TERM

# Print banner
echo -e "${BLUE}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "   Yomi OS Kernel Debugger"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${NC}"

# Check if kernel binary exists (needed for GDB symbols)
if [ ! -f "$KERNEL_BIN" ]; then
    print_error "Kernel binary not found at: $KERNEL_BIN"
    echo ""
    print_info "Build the kernel with:"
    echo "  cargo build --package yomi-kernel"
    echo ""
    exit 1
fi

print_success "Kernel binary found: $KERNEL_BIN"

# Check for debug symbols
if ! file "$KERNEL_BIN" | grep -q "not stripped"; then
    print_warning "Kernel binary appears to be stripped"
    print_warning "Debug symbols may be missing. Consider using debug build."
fi

# Check if ISO image exists (needed for booting)
if [ ! -f "$ISO_IMAGE" ]; then
    print_error "ISO image not found at: $ISO_IMAGE"
    echo ""
    print_info "Build the ISO with:"
    echo "  ./scripts/build-iso.sh"
    echo ""
    exit 1
fi

print_success "ISO image found: $ISO_IMAGE"

# Check if QEMU is installed
if ! command -v qemu-system-x86_64 &> /dev/null; then
    print_error "qemu-system-x86_64 not found"
    print_info "Install QEMU with: sudo apt install qemu-system-x86"
    exit 1
fi

# Check if GDB is installed (prefer rust-gdb)
GDB_CMD=""
if command -v rust-gdb &> /dev/null; then
    GDB_CMD="rust-gdb"
    print_success "Using rust-gdb for better Rust type visualization"
elif command -v gdb &> /dev/null; then
    GDB_CMD="gdb"
    print_warning "rust-gdb not found, using standard gdb"
    print_info "Install rust-gdb for better Rust debugging experience"
else
    print_error "GDB not found"
    print_info "Install GDB with: sudo apt install gdb"
    exit 1
fi

# Check if port is already in use
if lsof -i ":$GDB_PORT" &> /dev/null; then
    print_error "Port $GDB_PORT is already in use"
    print_info "Stop the process using the port or change GDB_PORT environment variable"
    exit 1
fi

# Start QEMU with GDB stub in background
print_info "Starting QEMU with GDB server on port $GDB_PORT..."
print_info "Booting from ISO: $ISO_IMAGE"

qemu-system-x86_64 \
    -cdrom "$ISO_IMAGE" \
    -s \
    -S \
    -serial stdio \
    -display none \
    -no-reboot \
    -m 256M \
    > "$QEMU_LOG" 2>&1 &

QEMU_PID=$!

# Verify QEMU started successfully
sleep 0.5
if ! kill -0 "$QEMU_PID" 2>/dev/null; then
    print_error "QEMU failed to start"
    print_info "Check log file: $QEMU_LOG"
    exit 1
fi

print_success "QEMU started (PID: $QEMU_PID)"
print_info "QEMU log: $QEMU_LOG"

# Wait for GDB server to be ready
print_info "Waiting for GDB server..."
MAX_RETRIES=10
RETRY_COUNT=0

while ! lsof -i ":$GDB_PORT" &> /dev/null; do
    sleep 0.2
    RETRY_COUNT=$((RETRY_COUNT + 1))

    if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
        print_error "GDB server did not start in time"
        exit 1
    fi

    if ! kill -0 "$QEMU_PID" 2>/dev/null; then
        print_error "QEMU process died unexpectedly"
        print_info "Check log file: $QEMU_LOG"
        exit 1
    fi
done

print_success "GDB server ready on port $GDB_PORT"

# Change to project root for .gdbinit
cd "$PROJECT_ROOT"

# Launch GDB
print_info "Launching GDB..."
echo ""

# GDB will automatically load .gdbinit from project root
exec "$GDB_CMD" "$KERNEL_BIN"

# Note: The cleanup function will be called automatically on exit
