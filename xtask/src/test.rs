use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use crate::util::{
    kernel_target_dir, print_error, print_info, print_step, print_success, print_warning,
    project_root,
};

/// Run integration tests
pub fn run_tests(filter: Option<&str>) -> Result<()> {
    print_step("Running Integration Tests");

    let root = project_root()?;
    let tests_dir = root.join("kernel/tests");

    if !tests_dir.exists() {
        print_warning("No tests directory found");
        return Ok(());
    }

    // Build the kernel first
    print_info("Building kernel for tests...");
    let build_status = Command::new("cargo")
        .args(&[
            "build",
            "--package",
            "yomi-kernel",
            "--target",
            "x86_64-unknown-none",
            "-Z",
            "build-std=core,compiler_builtins,alloc",
            "-Z",
            "build-std-features=compiler-builtins-mem",
        ])
        .status()
        .context("Failed to build kernel")?;

    if !build_status.success() {
        anyhow::bail!("Kernel build failed");
    }

    // Discover test files
    let mut test_files = Vec::new();
    for entry in fs::read_dir(&tests_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let test_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .context("Invalid test filename")?;

            // Apply filter if specified
            if let Some(f) = filter {
                if !test_name.contains(f) {
                    continue;
                }
            }

            test_files.push((test_name.to_string(), path));
        }
    }

    if test_files.is_empty() {
        print_warning("No test files found");
        return Ok(());
    }

    print_info(&format!("Found {} test(s)", test_files.len()));

    let mut passed = 0;
    let mut failed = 0;
    let mut failed_tests = Vec::new();

    for (test_name, _test_path) in test_files {
        print_info(&format!("Running test: {}", test_name));

        // Build the test binary
        let build_result = Command::new("cargo")
            .args(&[
                "rustc",
                "--manifest-path",
                "kernel/Cargo.toml",
                "--test",
                &test_name,
                "--target",
                "x86_64-unknown-none",
                "-Z",
                "build-std=core,compiler_builtins,alloc",
                "-Z",
                "build-std-features=compiler-builtins-mem",
                "--",
                "-C",
                "link-arg=--nmagic",
                "-C",
                "link-arg=--no-dynamic-linker",
                "-C",
                "link-arg=-Tkernel/linker.ld",
                "-C",
                "relocation-model=static",
            ])
            .current_dir(&root)
            .output()
            .context("Failed to build test")?;

        if !build_result.status.success() {
            print_error(&format!("Failed to build test: {}", test_name));
            failed += 1;
            failed_tests.push(test_name.clone());
            continue;
        }

        // Find the hashed test binary under target/.../deps
        let deps_dir = kernel_target_dir(false)?.join("deps");
        let test_bin = fs::read_dir(&deps_dir)
            .with_context(|| format!("Failed to list {}", deps_dir.display()))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name.starts_with(&format!("{}-", test_name))
                            && !name.ends_with(".d")
                    })
            });

        let test_bin = match test_bin {
            Some(bin) => bin,
            None => {
                print_error(&format!("Test binary not found for: {}", test_name));
                failed += 1;
                failed_tests.push(test_name.clone());
                continue;
            }
        };

        // Run the test in QEMU
        let test_result = Command::new("qemu-system-x86_64")
            .args(&[
                "-kernel",
                test_bin.to_str().context("Invalid test binary path")?,
                "-serial",
                "stdio",
                "-device",
                "isa-debug-exit,iobase=0xf4,iosize=0x04",
                "-no-reboot",
                "-no-shutdown",
                "-display",
                "none",
                "-m",
                "256M",
            ])
            .output()
            .context("Failed to run test in QEMU")?;

        let exit_code = test_result.status.code().unwrap_or(1);

        // QEMU isa-debug-exit returns: (exit_value << 1) | 1
        // Success (0x10) becomes 33
        if exit_code == 33 {
            print_success(&format!("✓ Test passed: {}", test_name));
            passed += 1;
        } else {
            print_error(&format!("✗ Test failed: {} (exit code: {})", test_name, exit_code));
            failed += 1;
            failed_tests.push(test_name);
        }
    }

    // Print summary
    println!("\n{}", "=".repeat(50));
    println!("Test Results:");
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("{}", "=".repeat(50));

    if failed > 0 {
        println!("\nFailed tests:");
        for test in &failed_tests {
            println!("  - {}", test);
        }
        anyhow::bail!("{} test(s) failed", failed);
    }

    print_success("All tests passed!");
    Ok(())
}
