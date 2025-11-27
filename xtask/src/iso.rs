use std::{
    fs,
    process::Command,
};

use anyhow::{
    Context,
    Result,
};

use crate::{
    build::build_kernel,
    util::{
        kernel_binary,
        print_info,
        print_step,
        print_success,
        project_root,
    },
};

/// Create a bootable ISO image
pub fn create_iso(release: bool) -> Result<()> {
    print_step("Creating Bootable ISO Image");

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

    #[allow(clippy::disallowed_methods)]
    fs::write(&grub_cfg, grub_config).context("Failed to write GRUB configuration")?;

    // Create ISO using grub-mkrescue
    let iso_path = root.join("yomios.iso");

    run_grub_mkrescue(&iso_path, &iso_dir)?;

    print_success(&format!("ISO created: {}", iso_path.display()));
    Ok(())
}

/// Run grub-mkrescue, using WSL on Windows
fn run_grub_mkrescue(iso_path: &std::path::Path, iso_dir: &std::path::Path) -> Result<()> {
    print_info("Running grub-mkrescue...");

    #[cfg(windows)]
    {
        // On Windows, use WSL to run grub-mkrescue
        let iso_path_wsl = windows_to_wsl_path(iso_path)?;
        let iso_dir_wsl = windows_to_wsl_path(iso_dir)?;

        print_info(&format!(
            "Using WSL path: {} -> {}",
            iso_dir.display(),
            iso_dir_wsl
        ));

        let status = Command::new("wsl")
            .args(["grub-mkrescue", "-o", &iso_path_wsl, &iso_dir_wsl])
            .status()
            .context("Failed to run grub-mkrescue via WSL. Run 'cargo x setup' first.")?;

        if !status.success() {
            anyhow::bail!(
                "grub-mkrescue failed. Ensure WSL has grub-pc-bin and xorriso installed."
            );
        }
    }

    #[cfg(not(windows))]
    {
        // On Linux/macOS, run grub-mkrescue directly
        let status = Command::new("grub-mkrescue")
            .args([
                "-o",
                iso_path.to_str().context("Invalid ISO path")?,
                iso_dir.to_str().context("Invalid ISO dir path")?,
            ])
            .status()
            .context(
                "Failed to run grub-mkrescue. Install with: sudo apt install grub-pc-bin xorriso",
            )?;

        if !status.success() {
            anyhow::bail!("grub-mkrescue failed");
        }
    }

    Ok(())
}

/// Convert Windows path to WSL path (e.g., C:\foo\bar -> /mnt/c/foo/bar)
#[cfg(windows)]
fn windows_to_wsl_path(path: &std::path::Path) -> Result<String> {
    let path_str = path.to_str().context("Invalid path")?;

    // Handle UNC-style paths like \\?\C:\...
    let path_str = path_str.strip_prefix(r"\\?\").unwrap_or(path_str);

    // Convert C:\foo\bar to /mnt/c/foo/bar
    if let Some(drive_rest) = path_str.strip_prefix(|c: char| c.is_ascii_alphabetic()) {
        if let Some(rest) = drive_rest.strip_prefix(':') {
            let drive_letter = path_str.chars().next().unwrap().to_ascii_lowercase();
            let unix_path = rest.replace('\\', "/");
            return Ok(format!("/mnt/{}{}", drive_letter, unix_path));
        }
    }

    anyhow::bail!("Could not convert Windows path to WSL path: {}", path_str)
}
