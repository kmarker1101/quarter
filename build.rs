// Build script to compile runtime.rs into the library

fn main() {
    // Compile runtime.rs to object file
    let status = std::process::Command::new("rustc")
        .args([
            "--crate-type=staticlib",
            "--edition", "2021",
            "-C", "opt-level=3",
            "src/runtime.rs",
            "-o", "target/libquarter_runtime.a",
        ])
        .status()
        .expect("Failed to compile runtime.rs");
    
    if !status.success() {
        panic!("Failed to build runtime.rs");
    }
    
    // Tell cargo to link the runtime
    println!("cargo:rustc-link-search=native=target");
    println!("cargo:rustc-link-lib=static=quarter_runtime");
    
    // Re-run if runtime.rs changes
    println!("cargo:rerun-if-changed=src/runtime.rs");
}
