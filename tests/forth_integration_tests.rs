use std::process::Command;
use std::env;
use std::path::PathBuf;

#[test]
fn test_basic_forth_tests() {
    // Build first to ensure binary is up to date
    let build_status = Command::new("cargo")
        .args(&["build", "--quiet"])
        .status()
        .expect("Failed to build");

    assert!(build_status.success(), "Build failed");

    // Get the project root directory (where Cargo.toml is)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    let project_root = PathBuf::from(manifest_dir);

    // Run the binary directly
    let binary_path = project_root.join("target/debug/quarter");
    let test_path = project_root.join("tests/basic_tests.fth");

    let output = Command::new(&binary_path)
        .arg(&test_path)
        .current_dir(&project_root)
        .output()
        .expect("Failed to run basic_tests.fth");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that the command succeeded
    assert!(output.status.success(),
            "Command failed with exit code: {:?}\nstdout: {}\nstderr: {}",
            output.status.code(), stdout, stderr);

    // Check that the output contains "Failed: 0"
    assert!(stdout.contains("Failed: 0"),
            "Basic tests had failures:\nstdout: {}\nstderr: {}", stdout, stderr);
}
