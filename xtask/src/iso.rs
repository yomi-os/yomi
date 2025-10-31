use anyhow::{Context, Result};
use std::fs;

use crate::build::build_kernel;
use crate::util::{
    ensure_command, kernel_binary, print_info, print_step, print_success, project_root, run_cmd,
};

/// Create a bootable ISO image
pub fn create_iso(release: bool) -> Result<()> {
    print_step("Creating Bootable ISO Image");

    // Ensure GRUB is installed
    ensure_command(
        "grub-mkrescue",
        "sudo apt install grub-pc-bin xorriso",
    )?;

    // First build the kernel
    build_kernel(release)?;

    // Verify kernel binary exists
    let kernel_bin = kernel_binary(release)?;
    if !kernel_bin.exists() {
        anyhow::bail!("Kernel binary not found at: {}", kernel_bin.display());
    }

    print_info(&format!("Using kernel: {}", kernel_bin.display()));

    // Setup ISO directory structure
    let root = project_root()?;
    let iso_dir = root.join("build/iso");
    let boot_dir = iso_dir.join("boot");
    let grub_dir = boot_dir.join("grub");

    print_info("Creating ISO directory structure...");
    fs::create_dir_all(&grub_dir).context("Failed to create GRUB directory")?;

    // Copy kernel
    print_info("Copying kernel...");
    let kernel_dest = boot_dir.join("kernel.bin");
    fs::copy(&kernel_bin, &kernel_dest)
        .with_context(|| format!("Failed to copy kernel to {}", kernel_dest.display()))?;

    // Create grub.cfg
    print_info("Creating GRUB configuration...");
    let grub_cfg = grub_dir.join("grub.cfg");
    let grub_config = r#"set timeout=5
set default=0

# Enable serial console
serial --unit=0 --speed=115200
terminal_input console serial
terminal_output console serial

menuentry "YomiOS" {
    multiboot2 /boot/kernel.bin
    boot
}
"#;

    fs::write(&grub_cfg, grub_config)
        .context("Failed to write GRUB configuration")?;

    // Create ISO using grub-mkrescue
    print_info("Running grub-mkrescue...");
    let iso_path = root.join("yomios.iso");

    run_cmd(
        "grub-mkrescue",
        &[
            "-o",
            iso_path.to_str().context("Invalid ISO path")?,
            iso_dir.to_str().context("Invalid ISO dir path")?,
        ],
    )?;

    print_success(&format!("ISO created: {}", iso_path.display()));
    Ok(())
}
