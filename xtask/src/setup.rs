use anyhow::{Context, Result};
use std::process::Command;

use crate::util::{command_exists, print_error, print_info, print_step, print_success, print_warning};

/// Setup development environment
pub fn setup_environment() -> Result<()> {
    print_step("Setting up Development Environment");

    let os = std::env::consts::OS;
    print_info(&format!("Detected OS: {}", os));

    match os {
        "windows" => setup_windows()?,
        "linux" => setup_linux()?,
        "macos" => setup_macos()?,
        _ => anyhow::bail!("Unsupported operating system: {}", os),
    }

    // Common setup for all platforms
    setup_rust_components()?;

    print_success("Development environment setup complete!");
    Ok(())
}

fn setup_windows() -> Result<()> {
    print_step("Windows Setup");

    // Check for NASM
    if !command_exists("nasm") {
        print_info("NASM not found. Installing via winget...");
        let status = Command::new("winget")
            .args(["install", "--id", "NASM.NASM", "-e", "--accept-source-agreements"])
            .status()
            .context("Failed to run winget. Is winget installed?")?;

        if status.success() {
            print_success("NASM installed successfully");
            print_warning("Please restart your terminal to update PATH");
        } else {
            print_error("Failed to install NASM via winget");
            print_info("Manual installation: https://www.nasm.us/pub/nasm/releasebuilds/3.01/win64/");
        }
    } else {
        print_success("NASM is already installed");
    }

    // Check for QEMU
    if !command_exists("qemu-system-x86_64") {
        print_info("QEMU not found. Installing via winget...");
        let status = Command::new("winget")
            .args(["install", "--id", "SoftwareFreedomConservancy.QEMU", "-e", "--accept-source-agreements"])
            .status()
            .context("Failed to run winget")?;

        if status.success() {
            print_success("QEMU installed successfully");
            print_warning("Please restart your terminal to update PATH");
        } else {
            print_error("Failed to install QEMU via winget");
            print_info("Manual installation: https://www.qemu.org/download/#windows");
        }
    } else {
        print_success("QEMU is already installed");
    }

    // Check for LLVM (needed for ld.lld linker)
    if !command_exists("ld.lld") {
        print_info("LLVM (ld.lld) not found. Installing via winget...");
        let status = Command::new("winget")
            .args(["install", "--id", "LLVM.LLVM", "-e", "--accept-source-agreements"])
            .status()
            .context("Failed to run winget")?;

        if status.success() {
            print_success("LLVM installed successfully");
            print_warning("Please restart your terminal to update PATH");
        } else {
            print_error("Failed to install LLVM via winget");
            print_info("Manual installation: https://releases.llvm.org/download.html");
        }
    } else {
        print_success("LLVM (ld.lld) is already installed");
    }

    // Check for WSL (needed for grub-mkrescue)
    print_info("Checking WSL availability for ISO creation...");
    let wsl_check = Command::new("wsl")
        .args(["--version"])
        .output();

    match wsl_check {
        Ok(output) if output.status.success() => {
            print_success("WSL is available");
            setup_wsl_dependencies()?;
        }
        _ => {
            print_warning("WSL is not available or not installed");
            print_info("WSL is required for creating bootable ISO images");
            print_info("Install WSL: wsl --install");
            print_info("Then run: cargo x setup");
        }
    }

    Ok(())
}

fn setup_wsl_dependencies() -> Result<()> {
    print_info("Checking WSL dependencies for ISO creation...");

    // Check if grub-mkrescue is available in WSL
    let grub_check = Command::new("wsl")
        .args(["which", "grub-mkrescue"])
        .output()
        .context("Failed to check grub-mkrescue in WSL")?;

    if !grub_check.status.success() {
        print_info("Installing grub and xorriso in WSL...");
        let install_status = Command::new("wsl")
            .args(["sudo", "apt", "update"])
            .status()
            .context("Failed to run apt update in WSL")?;

        if !install_status.success() {
            print_warning("Failed to update apt. You may need to run manually:");
            print_info("  wsl sudo apt update && wsl sudo apt install -y grub-pc-bin xorriso");
            return Ok(());
        }

        let install_status = Command::new("wsl")
            .args(["sudo", "apt", "install", "-y", "grub-pc-bin", "xorriso"])
            .status()
            .context("Failed to install grub packages in WSL")?;

        if install_status.success() {
            print_success("WSL dependencies installed successfully");
        } else {
            print_error("Failed to install WSL dependencies");
            print_info("Run manually: wsl sudo apt install -y grub-pc-bin xorriso");
        }
    } else {
        print_success("WSL dependencies are already installed");
    }

    Ok(())
}

fn setup_linux() -> Result<()> {
    print_step("Linux Setup");

    // Check for package manager
    let (pkg_manager, install_cmd) = if command_exists("apt") {
        ("apt", vec!["sudo", "apt", "install", "-y"])
    } else if command_exists("dnf") {
        ("dnf", vec!["sudo", "dnf", "install", "-y"])
    } else if command_exists("pacman") {
        ("pacman", vec!["sudo", "pacman", "-S", "--noconfirm"])
    } else {
        print_warning("Could not detect package manager");
        print_info("Please install manually: nasm, qemu-system-x86, grub-pc-bin, xorriso");
        return Ok(());
    };

    print_info(&format!("Detected package manager: {}", pkg_manager));

    // Install NASM
    if !command_exists("nasm") {
        print_info("Installing NASM...");
        let mut cmd = Command::new(&install_cmd[0]);
        cmd.args(&install_cmd[1..]);
        cmd.arg("nasm");
        let status = cmd.status().context("Failed to install nasm")?;
        if status.success() {
            print_success("NASM installed");
        }
    } else {
        print_success("NASM is already installed");
    }

    // Install QEMU
    if !command_exists("qemu-system-x86_64") {
        print_info("Installing QEMU...");
        let qemu_pkg = match pkg_manager {
            "apt" => "qemu-system-x86",
            "dnf" => "qemu-system-x86",
            "pacman" => "qemu-system-x86",
            _ => "qemu",
        };
        let mut cmd = Command::new(&install_cmd[0]);
        cmd.args(&install_cmd[1..]);
        cmd.arg(qemu_pkg);
        let status = cmd.status().context("Failed to install QEMU")?;
        if status.success() {
            print_success("QEMU installed");
        }
    } else {
        print_success("QEMU is already installed");
    }

    // Install grub tools
    if !command_exists("grub-mkrescue") {
        print_info("Installing GRUB tools...");
        let grub_pkgs: &[&str] = match pkg_manager {
            "apt" => &["grub-pc-bin", "xorriso"],
            "dnf" => &["grub2-tools", "xorriso"],
            "pacman" => &["grub", "xorriso"],
            _ => &["grub", "xorriso"],
        };
        for pkg in grub_pkgs {
            let mut cmd = Command::new(&install_cmd[0]);
            cmd.args(&install_cmd[1..]);
            cmd.arg(pkg);
            let _ = cmd.status();
        }
        print_success("GRUB tools installed");
    } else {
        print_success("GRUB tools are already installed");
    }

    Ok(())
}

fn setup_macos() -> Result<()> {
    print_step("macOS Setup");

    if !command_exists("brew") {
        print_error("Homebrew is required but not installed");
        print_info("Install Homebrew: /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"");
        return Ok(());
    }

    // Install NASM
    if !command_exists("nasm") {
        print_info("Installing NASM...");
        Command::new("brew")
            .args(["install", "nasm"])
            .status()
            .context("Failed to install nasm")?;
        print_success("NASM installed");
    } else {
        print_success("NASM is already installed");
    }

    // Install QEMU
    if !command_exists("qemu-system-x86_64") {
        print_info("Installing QEMU...");
        Command::new("brew")
            .args(["install", "qemu"])
            .status()
            .context("Failed to install QEMU")?;
        print_success("QEMU installed");
    } else {
        print_success("QEMU is already installed");
    }

    // Install xorriso (grub-mkrescue depends on it)
    print_info("Installing xorriso...");
    let _ = Command::new("brew")
        .args(["install", "xorriso"])
        .status();

    print_warning("grub-mkrescue may not be available on macOS");
    print_info("Consider using a Docker container or VM for ISO creation");

    Ok(())
}

fn setup_rust_components() -> Result<()> {
    print_step("Rust Components");

    // Ensure nightly toolchain
    print_info("Checking Rust nightly toolchain...");
    let status = Command::new("rustup")
        .args(["toolchain", "install", "nightly"])
        .status()
        .context("Failed to install nightly toolchain")?;

    if status.success() {
        print_success("Nightly toolchain available");
    }

    // Add x86_64-unknown-none target
    print_info("Adding x86_64-unknown-none target...");
    let status = Command::new("rustup")
        .args(["target", "add", "x86_64-unknown-none", "--toolchain", "nightly"])
        .status()
        .context("Failed to add target")?;

    if status.success() {
        print_success("Target x86_64-unknown-none added");
    }

    // Add rust-src component (needed for build-std)
    print_info("Adding rust-src component...");
    let status = Command::new("rustup")
        .args(["component", "add", "rust-src", "--toolchain", "nightly"])
        .status()
        .context("Failed to add rust-src")?;

    if status.success() {
        print_success("rust-src component added");
    }

    // Add llvm-tools for rust-lld
    print_info("Adding llvm-tools component...");
    let status = Command::new("rustup")
        .args(["component", "add", "llvm-tools", "--toolchain", "nightly"])
        .status()
        .context("Failed to add llvm-tools")?;

    if status.success() {
        print_success("llvm-tools component added");
    }

    Ok(())
}
