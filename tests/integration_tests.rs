use quarter::{execute_line, load_file, Dictionary, Stack};
use std::fs;
use std::io::Write;

#[test]
fn test_execute_line_simple_expression() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    execute_line("5 3 +", &mut stack, &mut dict).unwrap();
    assert_eq!(stack.pop(), Some(8));
}

#[test]
fn test_execute_line_word_definition() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Define SQUARE
    execute_line(": SQUARE DUP * ;", &mut stack, &mut dict).unwrap();

    // Use SQUARE
    execute_line("5 SQUARE", &mut stack, &mut dict).unwrap();
    assert_eq!(stack.pop(), Some(25));
}

#[test]
fn test_execute_line_if_then_error() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // IF/THEN outside definition should error
    let result = execute_line("1 IF 42 THEN", &mut stack, &mut dict);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("compile-only"));
}

#[test]
fn test_execute_line_empty() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Empty line should not error
    let result = execute_line("", &mut stack, &mut dict);
    assert!(result.is_ok());
}

#[test]
fn test_execute_line_whitespace_only() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Whitespace-only line should not error
    let result = execute_line("   ", &mut stack, &mut dict);
    assert!(result.is_ok());
}

#[test]
fn test_execute_line_invalid_word() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Unknown word should error
    let result = execute_line("NONEXISTENT", &mut stack, &mut dict);
    assert!(result.is_err());
}

#[test]
fn test_execute_line_incomplete_definition() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Missing semicolon
    let result = execute_line(": SQUARE DUP *", &mut stack, &mut dict);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Missing ;"));
}

#[test]
fn test_execute_line_definition_with_if() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Define word with IF/THEN
    execute_line(": ABS DUP 0 < IF NEGATE THEN ;", &mut stack, &mut dict).unwrap();

    // Test with negative
    execute_line("-5 ABS", &mut stack, &mut dict).unwrap();
    assert_eq!(stack.pop(), Some(5));

    // Test with positive
    execute_line("5 ABS", &mut stack, &mut dict).unwrap();
    assert_eq!(stack.pop(), Some(5));
}

#[test]
fn test_load_file_simple() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Create a temporary test file
    let test_file = "/tmp/quarter_test_simple.qtr";
    let mut file = fs::File::create(test_file).unwrap();
    writeln!(file, "5 3 +").unwrap();
    writeln!(file, "10 *").unwrap();

    // Load and execute
    load_file(test_file, &mut stack, &mut dict).unwrap();

    assert_eq!(stack.pop(), Some(80)); // (5 + 3) * 10

    // Cleanup
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_load_file_with_comments() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Create a temporary test file
    let test_file = "/tmp/quarter_test_comments.qtr";
    let mut file = fs::File::create(test_file).unwrap();
    writeln!(file, "\\ This is a comment").unwrap();
    writeln!(file, "5 3 +").unwrap();
    writeln!(file, "\\ Another comment").unwrap();
    writeln!(file, "2 *").unwrap();

    // Load and execute
    load_file(test_file, &mut stack, &mut dict).unwrap();

    assert_eq!(stack.pop(), Some(16)); // (5 + 3) * 2

    // Cleanup
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_load_file_with_definitions() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Create a temporary test file
    let test_file = "/tmp/quarter_test_defs.qtr";
    let mut file = fs::File::create(test_file).unwrap();
    writeln!(file, ": SQUARE DUP * ;").unwrap();
    writeln!(file, ": CUBE DUP SQUARE * ;").unwrap();
    writeln!(file, "3 CUBE").unwrap();

    // Load and execute
    load_file(test_file, &mut stack, &mut dict).unwrap();

    assert_eq!(stack.pop(), Some(27)); // 3^3

    // Cleanup
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_load_file_with_empty_lines() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Create a temporary test file
    let test_file = "/tmp/quarter_test_empty.qtr";
    let mut file = fs::File::create(test_file).unwrap();
    writeln!(file).unwrap();
    writeln!(file, "5 3 +").unwrap();
    writeln!(file).unwrap();
    writeln!(file).unwrap();
    writeln!(file, "2 *").unwrap();
    writeln!(file).unwrap();

    // Load and execute
    load_file(test_file, &mut stack, &mut dict).unwrap();

    assert_eq!(stack.pop(), Some(16)); // (5 + 3) * 2

    // Cleanup
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_load_file_nonexistent() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Try to load a file that doesn't exist
    let result = load_file("/tmp/nonexistent_file.qtr", &mut stack, &mut dict);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot read file"));
}

#[test]
fn test_load_file_with_paren_comments() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();

    // Create a temporary test file
    let test_file = "/tmp/quarter_test_paren.qtr";
    let mut file = fs::File::create(test_file).unwrap();
    writeln!(file, "( This is a paren comment )").unwrap();
    writeln!(file, "5 3 +").unwrap();

    // Load and execute
    load_file(test_file, &mut stack, &mut dict).unwrap();

    assert_eq!(stack.pop(), Some(8));

    // Cleanup
    fs::remove_file(test_file).unwrap();
}
