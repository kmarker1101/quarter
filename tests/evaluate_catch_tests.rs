/// Integration tests for EVALUATE and CATCH
/// These test the words end-to-end by loading Forth files

#[test]
fn test_evaluate_via_file() {
    // Create a test file that uses EVALUATE
    let test_code = r#"
        S" 5 10 +" EVALUATE
    "#;
    std::fs::write("/tmp/test_evaluate.fth", test_code).unwrap();

    // Run it - should not error
    let output = std::process::Command::new("target/debug/quarter")
        .arg("/tmp/test_evaluate.fth")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success(), "EVALUATE test failed");
}

#[test]
fn test_evaluate_multiline_via_file() {
    // Test EVALUATE with multi-word expression
    let test_code = r#"
        \ Use EVALUATE for multi-word expression
        S" 3 DUP * 2 +" EVALUATE
        \ Should leave 11 on stack (3*3 + 2)
    "#;
    std::fs::write("/tmp/test_evaluate_multi.fth", test_code).unwrap();

    // Run it
    let output = std::process::Command::new("target/debug/quarter")
        .arg("/tmp/test_evaluate_multi.fth")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success(), "EVALUATE multiline test failed");
}

#[test]
fn test_catch_success_via_file() {
    // Test CATCH with successful code
    let test_code = r#"
        S" 5 10 +" CATCH DROP
    "#;
    std::fs::write("/tmp/test_catch_ok.fth", test_code).unwrap();

    let output = std::process::Command::new("target/debug/quarter")
        .arg("/tmp/test_catch_ok.fth")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success(), "CATCH success test failed");
}

#[test]
fn test_catch_error_via_file() {
    // Test CATCH with error
    let test_code = r#"
        S" BADWORD" CATCH DROP
    "#;
    std::fs::write("/tmp/test_catch_err.fth", test_code).unwrap();

    let output = std::process::Command::new("target/debug/quarter")
        .arg("/tmp/test_catch_err.fth")
        .output()
        .expect("Failed to execute");

    // Should succeed (CATCH catches the error)
    assert!(output.status.success(), "CATCH error test failed");
}

#[test]
fn test_cmove_via_file() {
    // Test CMOVE through a Forth file
    let test_code = r#"
        : TEST-CMOVE
            \ Store "HI" at 0x200000
            72 2097152 C!
            73 2097153 C!

            \ CMOVE to 0x201000
            2097152 2101248 2 CMOVE

            \ Verify it copied
            2101248 C@ 72 = IF
                2101249 C@ 73 = IF
                    \ Success - both bytes match
                THEN
            THEN
        ;
        TEST-CMOVE
    "#;
    std::fs::write("/tmp/test_cmove.fth", test_code).unwrap();

    let output = std::process::Command::new("target/debug/quarter")
        .arg("/tmp/test_cmove.fth")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success(), "CMOVE test failed");
}
