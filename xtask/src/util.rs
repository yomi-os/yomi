use std::{
    path::Path,
    process::{
        Command,
        ExitStatus,
    },
};

use anyhow::{
    Context,
    Result,
};
use colored::Colorize;

/// Run a command and print its status
pub fn run_cmd(cmd: &str, args: &[&str]) -> Result<ExitStatus> {
    println!("{} {} {}", "Running:".blue().bold(), cmd, args.join(" "));

    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("Failed to execute: {}", cmd))?;

    if !status.success() {
        anyhow::bail!("{} failed with exit code: {:?}", cmd, status.code());
    }

    Ok(status)
}

/// Check if a command exists in PATH
pub fn command_exists(cmd: &str) -> bool {
    #[cfg(windows)]
    let check_cmd = "where";
    #[cfg(not(windows))]
    let check_cmd = "which";

    Command::new(check_cmd)
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Ensure a command exists, or return an error with installation hint
#[allow(dead_code)]
pub fn ensure_command(cmd: &str, install_hint: &str) -> Result<()> {
    if !command_exists(cmd) {
        anyhow::bail!(
            "{} not found. Install with: {}",
            cmd.red(),
            install_hint.yellow()
        );
    }
    Ok(())
}

/// Get the project root directory
pub fn project_root() -> Result<std::path::PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?;

    // xtask is in project_root/xtask, so go up one level
    let xtask_dir = std::path::PathBuf::from(manifest_dir);
    let root = xtask_dir
        .parent()
        .context("Failed to get project root")?
        .to_path_buf();

    Ok(root)
}

/// Get kernel target directory
pub fn kernel_target_dir(release: bool) -> Result<std::path::PathBuf> {
    let root = project_root()?;
    let profile = if release { "release" } else { "debug" };
    Ok(root.join("target/x86_64-unknown-none").join(profile))
}

/// Get kernel binary path
pub fn kernel_binary(release: bool) -> Result<std::path::PathBuf> {
    Ok(kernel_target_dir(release)?.join("yomi-kernel"))
}

/// Check if a file exists, with a helpful error message
pub fn ensure_file_exists(path: &Path, build_hint: &str) -> Result<()> {
    if !path.exists() {
        anyhow::bail!(
            "File not found: {}\nBuild with: {}",
            path.display().to_string().red(),
            build_hint.yellow()
        );
    }
    Ok(())
}

/// Print a success message
pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg.green());
}

/// Print an info message
pub fn print_info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

/// Print a warning message
pub fn print_warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg.yellow());
}

/// Print an error message
pub fn print_error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg.red());
}

/// Print a step header
pub fn print_step(step: &str) {
    println!("\n{} {}", "▶".cyan().bold(), step.cyan().bold());
}
