use anyhow::Result;
use crate::util::{print_step, print_success, run_cmd};

/// Build the kernel
pub fn build_kernel(release: bool) -> Result<()> {
    print_step("Building Yomi Kernel");

    let mut args = vec![
        "build",
        "--package",
        "yomi-kernel",
        "--target",
        "x86_64-unknown-none",
        "-Z",
        "build-std=core,compiler_builtins,alloc",
        "-Z",
        "build-std-features=compiler-builtins-mem",
    ];

    if release {
        args.push("--release");
    }

    run_cmd("cargo", &args)?;

    print_success("Kernel build complete");
    Ok(())
}
