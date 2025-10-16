use std::env;
use std::process::{exit, Command};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("build") => build(),
        Some("run") => run(),
        Some("test") => test(),
        Some("clean") => clean(),
        _ => {
            print_help();
            exit(1);
        }
    }
}

fn build() {
    println!("🔨 Building Yomi Kernel...");

    run_cmd("cargo", &["build", "--package", "yomi-kernel", "--release"]);

    println!("✓ Build complete!");
}

fn run() {
    build();

    println!("🚀 Starting Yomi in QEMU...");

    run_cmd("cargo", &["run", "--package", "yomi-kernel", "--release"]);
}

fn test() {
    println!("🧪 Running tests...");

    run_cmd("cargo", &["test", "--workspace"]);
}

fn clean() {
    println!("🧹 Cleaning build artifacts...");

    run_cmd("cargo", &["clean"]);

    println!("✓ Clean complete!");
}

fn print_help() {
    println!("Yomi Build System");
    println!("");
    println!("USAGE:");
    println!("    cargo xtask <COMMAND>");
    println!("");
    println!("COMMANDS:");
    println!("    build    Build the kernel");
    println!("    run      Build and run in QEMU");
    println!("    test     Run all tests");
    println!("    clean    Clean build artifacts");
}

fn run_cmd(cmd: &str, args: &[&str]) {
    let status = Command::new(cmd).args(args).status().unwrap_or_else(|e| {
        eprintln!("Failed to execute {}: {}", cmd, e);
        exit(1);
    });

    if !status.success() {
        eprintln!("{} failed with exit code: {:?}", cmd, status.code());
        exit(1);
    }
}
