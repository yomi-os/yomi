use anyhow::{Context, Result};
use std::process::Command;

use crate::iso::create_iso;
use crate::util::{
    command_exists, ensure_file_exists, kernel_binary, print_info, print_step, print_warning,
    project_root,
};

/// Launch kernel in debug mode with GDB
pub fn debug_kernel(release: bool) -> Result<()> {
    print_step("Launching Debug Session");

    // Ensure ISO exists
    let root = project_root()?;
    let iso_path = root.join("yomios.iso");

    print_info("Rebuilding ISO to match the requested profile...");
    create_iso(release)?;
    ensure_file_exists(&iso_path, "cargo xtask iso")?;

    // Ensure kernel binary exists (needed for symbols)
    let kernel_bin = kernel_binary(release)?;
    ensure_file_exists(
        &kernel_bin,
        "cargo build --package yomi-kernel",
    )?;

    print_info(&format!("Kernel binary: {}", kernel_bin.display()));
    print_info(&format!("ISO image: {}", iso_path.display()));

    // Check for GDB
    let gdb_cmd = if command_exists("rust-gdb") {
        print_info("Using rust-gdb for better Rust type visualization");
        "rust-gdb"
    } else if command_exists("gdb") {
        print_warning("rust-gdb not found, using standard gdb");
        print_info("Install rust-gdb for better Rust debugging experience");
        "gdb"
    } else {
        anyhow::bail!("GDB not found. Install with: sudo apt install gdb");
    };

    // Create a temporary GDB script
    let gdb_script = format!(
        r#"
# Connect to QEMU GDB server
target remote :1234

# Load symbols
symbol-file {}

# Set breakpoints
break kernel_main

# Continue execution
continue
"#,
        kernel_bin.display()
    );

    let gdb_script_path = root.join("build/.gdbinit-temp");
    std::fs::create_dir_all(root.join("build"))?;
    std::fs::write(&gdb_script_path, gdb_script)?;

    print_info("Starting QEMU with GDB server...");
    print_info("In another terminal, you can connect to QEMU using:");
    println!("\n    {} -x {}\n", gdb_cmd, gdb_script_path.display());
    print_info("Or manually connect with:");
    println!("    {}", gdb_cmd);
    println!("    (gdb) target remote :1234");
    println!("    (gdb) symbol-file {}", kernel_bin.display());
    println!();
    print_info("QEMU will wait for GDB connection on port 1234");
    print_info("Press Ctrl+C to stop QEMU");
    println!();

    // Start QEMU with GDB server
    let status = Command::new("qemu-system-x86_64")
        .args(&[
            "-cdrom",
            iso_path.to_str().context("Invalid ISO path")?,
            "-s",                // GDB server on port 1234
            "-S",                // Pause at startup
            "-serial",
            "stdio",
            "-no-reboot",
            "-m",
            "256M",
        ])
        .status()
        .context("Failed to start QEMU")?;

    if !status.success() {
        anyhow::bail!("QEMU exited with error: {:?}", status.code());
    }

    Ok(())
}
