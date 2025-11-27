use std::process::Command;

use anyhow::{
    Context,
    Result,
};

use crate::{
    iso::create_iso,
    util::{
        ensure_file_exists,
        print_info,
        print_step,
        project_root,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QemuMode {
    Run,
    Test,
    Debug,
}

impl QemuMode {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "run" => Ok(Self::Run),
            "test" => Ok(Self::Test),
            "debug" => Ok(Self::Debug),
            _ => anyhow::bail!("Invalid QEMU mode: {}. Valid modes: run, test, debug", s),
        }
    }
}

/// Get the QEMU executable path (platform-specific)
fn get_qemu_path() -> Result<String> {
    #[cfg(windows)]
    {
        // Check common Windows installation paths
        let paths = [
            r"C:\Program Files\qemu\qemu-system-x86_64.exe",
            r"C:\Program Files (x86)\qemu\qemu-system-x86_64.exe",
        ];

        for path in &paths {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // Try PATH as fallback
        if which_qemu().is_some() {
            return Ok("qemu-system-x86_64".to_string());
        }

        anyhow::bail!(
            "QEMU not found. Install with: winget install SoftwareFreedomConservancy.QEMU\nOr \
             run: cargo x setup"
        );
    }

    #[cfg(not(windows))]
    {
        if which_qemu().is_some() {
            return Ok("qemu-system-x86_64".to_string());
        }

        anyhow::bail!(
            "QEMU not found. Install with: sudo apt install qemu-system-x86\nOr run: cargo x setup"
        );
    }
}

/// Check if qemu-system-x86_64 is in PATH
fn which_qemu() -> Option<()> {
    #[cfg(windows)]
    let check_cmd = "where";
    #[cfg(not(windows))]
    let check_cmd = "which";

    Command::new(check_cmd)
        .arg("qemu-system-x86_64")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|_| ())
}

/// Run the kernel in QEMU
pub fn run_qemu(mode: QemuMode, release: bool) -> Result<()> {
    print_step(&format!("Starting QEMU in {:?} mode", mode));

    // Get QEMU path
    let qemu_path = get_qemu_path()?;

    // Ensure ISO exists
    let root = project_root()?;
    let iso_path = root.join("yomios.iso");

    print_info("Rebuilding ISO to match the requested profile...");
    create_iso(release)?;
    ensure_file_exists(&iso_path, "cargo xtask iso")?;
    print_info(&format!("Booting from ISO: {}", iso_path.display()));

    // Build QEMU command
    let mut cmd = Command::new(&qemu_path);

    // Common options
    cmd.arg("-cdrom")
        .arg(&iso_path)
        .arg("-serial")
        .arg("stdio")
        .arg("-no-reboot")
        .arg("-m")
        .arg("256M");

    // Mode-specific options
    match mode {
        QemuMode::Test => {
            // Test mode: add isa-debug-exit device for programmatic exit
            cmd.arg("-device")
                .arg("isa-debug-exit,iobase=0xf4,iosize=0x04")
                .arg("-no-shutdown")
                .arg("-display")
                .arg("none");
            print_info("Test mode: Using isa-debug-exit device for exit codes");
        }
        QemuMode::Debug => {
            // Debug mode: enable GDB server
            cmd.arg("-s").arg("-S");
            print_info("Debug mode: GDB server listening on port 1234");
            print_info("Connect with: gdb target/x86_64-unknown-none/debug/yomi-kernel");
            print_info("Then run: target remote :1234");
        }
        QemuMode::Run => {
            // Normal run mode - no additional flags
            print_info("Normal mode: Running kernel");
        }
    }

    // Execute QEMU
    let status = cmd.status().context("Failed to start QEMU")?;

    // Handle exit code for test mode
    if mode == QemuMode::Test {
        let exit_code = status.code().unwrap_or(1);
        // QEMU isa-debug-exit returns: (exit_value << 1) | 1
        // So exit code 0x10 becomes (0x10 << 1) | 1 = 33
        if exit_code == 33 {
            print_info("Test passed (exit code 33 = success)");
            return Ok(());
        } else {
            anyhow::bail!("Test failed with exit code: {}", exit_code);
        }
    }

    if !status.success() {
        anyhow::bail!("QEMU exited with error: {:?}", status.code());
    }

    Ok(())
}
