mod build;
mod debug;
mod iso;
mod qemu;
mod setup;
mod test;
mod util;

use anyhow::Result;
use build::build_kernel;
use clap::{
    Parser,
    Subcommand,
};
use colored::Colorize;
use debug::debug_kernel;
use iso::create_iso;
use qemu::{
    QemuMode,
    run_qemu,
};
use setup::setup_environment;
use test::run_tests;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "YomiOS Build System", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the kernel
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },

    /// Build bootable ISO image
    Iso {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },

    /// Run kernel in QEMU
    Run {
        /// QEMU mode: run (normal), test, or debug
        #[arg(long, default_value = "run")]
        mode: String,

        /// Build in release mode
        #[arg(long)]
        release: bool,
    },

    /// Run integration tests
    Test {
        /// Filter tests by name
        #[arg(long)]
        filter: Option<String>,
    },

    /// Launch kernel in debug mode with GDB
    Debug {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },

    /// Clean build artifacts
    Clean,

    /// Setup development environment (install dependencies)
    Setup,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Build { release } => {
            build_kernel(release)?;
        }

        Command::Iso { release } => {
            create_iso(release)?;
        }

        Command::Run { mode, release } => {
            let qemu_mode = QemuMode::from_str(&mode)?;
            run_qemu(qemu_mode, release)?;
        }

        Command::Test { filter } => {
            run_tests(filter.as_deref())?;
        }

        Command::Debug { release } => {
            debug_kernel(release)?;
        }

        Command::Clean => {
            clean()?;
        }

        Command::Setup => {
            setup_environment()?;
        }
    }

    Ok(())
}

fn clean() -> Result<()> {
    use crate::util::{
        print_info,
        print_step,
        print_success,
        project_root,
        run_cmd,
    };

    print_step("Cleaning Build Artifacts");

    // Clean cargo build artifacts
    run_cmd("cargo", &["clean"])?;

    // Clean ISO build directory
    let root = project_root()?;
    let build_dir = root.join("build");

    if build_dir.exists() {
        print_info("Removing build directory...");
        std::fs::remove_dir_all(&build_dir)?;
    }

    // Clean ISO image
    let iso_path = root.join("yomios.iso");
    if iso_path.exists() {
        print_info("Removing ISO image...");
        std::fs::remove_file(&iso_path)?;
    }

    print_success("Clean complete");
    Ok(())
}
