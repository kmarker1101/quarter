use quarter::{execute_line, load_file, Dictionary, LoopStack, ReturnStack, Stack, Memory};
use std::fs;
use std::io::Write;

// Test fixture that automatically clears global state on drop
struct TestGuard;

impl Drop for TestGuard {
    fn drop(&mut self) {
        quarter::clear_test_state();
    }
}

#[test]
fn test_execute_line_simple_expression() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    execute_line("5 3 +", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, false, false, false).unwrap();
    assert_eq!(stack.pop(&mut memory), Some(8));
}

#[test]
fn test_execute_line_word_definition() {
    let _guard = TestGuard;
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Define SQUARE (JIT disabled - tests run faster in interpreter mode)
    execute_line(": SQUARE DUP * ;", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();

    // Use SQUARE
    execute_line("5 SQUARE", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();
    assert_eq!(stack.pop(&mut memory), Some(25));
}

#[test]
fn test_execute_line_if_then_error() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // IF/THEN outside definition should error
    let result = execute_line("1 IF 42 THEN", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, false, false, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("compile-only"));
}

#[test]
fn test_execute_line_empty() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Empty line should not error
    let result = execute_line("", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, false, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_execute_line_whitespace_only() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Whitespace-only line should not error
    let result = execute_line("   ", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, false, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_execute_line_invalid_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Unknown word should error
    let result = execute_line("NONEXISTENT", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, false, false, false);
    assert!(result.is_err());
}

#[test]
fn test_execute_line_incomplete_definition() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Missing semicolon
    let result = execute_line(": SQUARE DUP *", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, false, false, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Missing ;"));
}

#[test]
fn test_load_file_simple() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Create a temporary test file
    let test_file = "/tmp/quarter_test_simple.qtr";
    let mut file = fs::File::create(test_file).unwrap();
    writeln!(file, "5 3 +").unwrap();
    writeln!(file, "10 *").unwrap();

    // Load and execute (JIT disabled - avoids stack overflow and runs faster)
    load_file(test_file, &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(80)); // (5 + 3) * 10

    // Cleanup
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_load_file_with_comments() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Create a temporary test file
    let test_file = "/tmp/quarter_test_comments.qtr";
    let mut file = fs::File::create(test_file).unwrap();
    writeln!(file, "\\ This is a comment").unwrap();
    writeln!(file, "5 3 +").unwrap();
    writeln!(file, "\\ Another comment").unwrap();
    writeln!(file, "2 *").unwrap();

    // Load and execute (JIT disabled - avoids stack overflow and runs faster)
    load_file(test_file, &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(16)); // (5 + 3) * 2

    // Cleanup
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_load_file_with_definitions() {
    let _guard = TestGuard;
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Create a temporary test file
    let test_file = "/tmp/quarter_test_defs.qtr";
    let mut file = fs::File::create(test_file).unwrap();
    writeln!(file, ": SQUARE DUP * ;").unwrap();
    writeln!(file, ": CUBE DUP SQUARE * ;").unwrap();
    writeln!(file, "3 CUBE").unwrap();

    // Load and execute (JIT disabled - avoids stack overflow and runs faster)
    load_file(test_file, &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(27)); // 3^3

    // Cleanup
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_load_file_with_empty_lines() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Create a temporary test file
    let test_file = "/tmp/quarter_test_empty.qtr";
    let mut file = fs::File::create(test_file).unwrap();
    writeln!(file).unwrap();
    writeln!(file, "5 3 +").unwrap();
    writeln!(file).unwrap();
    writeln!(file).unwrap();
    writeln!(file, "2 *").unwrap();
    writeln!(file).unwrap();

    // Load and execute (JIT disabled - avoids stack overflow and runs faster)
    load_file(test_file, &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(16)); // (5 + 3) * 2

    // Cleanup
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_load_file_nonexistent() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Try to load a file that doesn't exist
    let result = load_file("/tmp/nonexistent_file.qtr", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, false, false, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot read file"));
}

#[test]
fn test_load_file_with_paren_comments() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Create a temporary test file
    let test_file = "/tmp/quarter_test_paren.qtr";
    let mut file = fs::File::create(test_file).unwrap();
    writeln!(file, "( This is a paren comment )").unwrap();
    writeln!(file, "5 3 +").unwrap();

    // Load and execute (JIT disabled - avoids stack overflow and runs faster)
    load_file(test_file, &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(8));

    // Cleanup
    fs::remove_file(test_file).unwrap();
}

// INCLUDE tests
#[test]
fn test_include_simple() {
    let _guard = TestGuard;
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Create a library file
    let lib_file = "/tmp/quarter_test_include_lib.qtr";
    let mut file = fs::File::create(lib_file).unwrap();
    writeln!(file, ": DOUBLE DUP + ;").unwrap();
    writeln!(file, "10 20 +").unwrap();

    // Use INCLUDE via execute_line (JIT disabled - avoids stack overflow)
    execute_line(&format!("INCLUDE {}", lib_file), &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();

    // Should have 30 on stack from the file
    assert_eq!(stack.pop(&mut memory), Some(30));

    // DOUBLE should be defined
    execute_line("5 DOUBLE", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();
    assert_eq!(stack.pop(&mut memory), Some(10));

    // Cleanup
    fs::remove_file(lib_file).unwrap();
}

#[test]
fn test_include_nested() {
    let _guard = TestGuard;
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Create first library file
    let lib1_file = "/tmp/quarter_test_include_lib1.qtr";
    let mut file1 = fs::File::create(lib1_file).unwrap();
    writeln!(file1, ": WORD1 42 ;").unwrap();

    // Create second library file that includes the first
    let lib2_file = "/tmp/quarter_test_include_lib2.qtr";
    let mut file2 = fs::File::create(lib2_file).unwrap();
    writeln!(file2, "INCLUDE {}", lib1_file).unwrap();
    writeln!(file2, ": WORD2 WORD1 2 * ;").unwrap();

    // Include the second file (which includes the first) - JIT disabled
    execute_line(&format!("INCLUDE {}", lib2_file), &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();

    // WORD1 from lib1 should be defined
    execute_line("WORD1", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();
    assert_eq!(stack.pop(&mut memory), Some(42));

    // WORD2 from lib2 should be defined and use WORD1
    execute_line("WORD2", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, true, false, false).unwrap();
    assert_eq!(stack.pop(&mut memory), Some(84));

    // Cleanup
    fs::remove_file(lib1_file).unwrap();
    fs::remove_file(lib2_file).unwrap();
}

#[test]
fn test_include_nonexistent() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Try to include a nonexistent file
    let result = execute_line("INCLUDE /tmp/nonexistent_lib.qtr", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, false, false, false);

    // Should error gracefully
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot read file"));
}

#[test]
fn test_include_missing_filename() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // INCLUDE without filename should error
    let result = execute_line("INCLUDE", &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, false, false, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("requires a filename"));
}
