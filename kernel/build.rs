use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // List of assembly files to compile
    let asm_files = vec![
        ("multiboot2.asm", "multiboot2.o"),
        ("entry.asm", "entry.o"),
        ("boot64.asm", "boot64.o"),
    ];

    let mut object_files = Vec::new();

    for (asm_name, obj_name) in &asm_files {
        let asm_file = src_dir.join("src/boot").join(asm_name);
        let obj_file = out_dir.join(obj_name);

        println!("cargo:rerun-if-changed={}", asm_file.display());

        let status = Command::new("nasm")
            .args([
                "-f", "elf64",
                "-o", obj_file.to_str().unwrap(),
                asm_file.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to execute nasm. Make sure nasm is installed.");

        if !status.success() {
            panic!("Failed to compile assembly file: {}", asm_file.display());
        }

        object_files.push(obj_file);
    }

    // Link the object files directly
    for obj_file in &object_files {
        println!("cargo:rustc-link-arg={}", obj_file.display());
    }
}
