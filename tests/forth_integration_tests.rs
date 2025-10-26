use std::process::Command;
use std::env;
use std::path::PathBuf;

#[test]
fn test_forth_tests_interpreted() {
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

    // Run the binary directly in interpreted mode
    let binary_path = project_root.join("target/debug/quarter");
    let test_path = project_root.join("tests/run-all-tests.fth");

    let output = Command::new(&binary_path)
        .arg(&test_path)
        .current_dir(&project_root)
        .output()
        .expect("Failed to run run-all-tests.fth");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that the command succeeded
    assert!(output.status.success(),
            "Command failed with exit code: {:?}\nstdout: {}\nstderr: {}",
            output.status.code(), stdout, stderr);

    // Check that the output contains "Failed: 0"
    assert!(stdout.contains("Failed: 0"),
            "Interpreted mode tests had failures:\nstdout: {}\nstderr: {}", stdout, stderr);
}

#[test]
fn test_forth_tests_jit() {
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

    // Run the binary with --jit flag
    let binary_path = project_root.join("target/debug/quarter");
    let test_path = project_root.join("tests/run-all-tests.fth");

    let output = Command::new(&binary_path)
        .arg(&test_path)
        .arg("--jit")
        .current_dir(&project_root)
        .output()
        .expect("Failed to run run-all-tests.fth with --jit");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that the command succeeded
    assert!(output.status.success(),
            "JIT mode command failed with exit code: {:?}\nstdout: {}\nstderr: {}",
            output.status.code(), stdout, stderr);

    // Check that the output contains "Failed: 0"
    assert!(stdout.contains("Failed: 0"),
            "JIT mode tests had failures:\nstdout: {}\nstderr: {}", stdout, stderr);
}

#[test]
fn test_tco_interpreted() {
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

    // Run the TCO tests in interpreted mode
    let binary_path = project_root.join("target/debug/quarter");
    let test_path = project_root.join("tests/tco_tests.fth");

    let output = Command::new(&binary_path)
        .arg(&test_path)
        .current_dir(&project_root)
        .output()
        .expect("Failed to run tco_tests.fth");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that the command succeeded
    assert!(output.status.success(),
            "TCO tests failed with exit code: {:?}\nstdout: {}\nstderr: {}",
            output.status.code(), stdout, stderr);

    // Check that the output contains success message
    assert!(stdout.contains("All TCO tests completed successfully!"),
            "TCO tests had failures:\nstdout: {}\nstderr: {}", stdout, stderr);
}
