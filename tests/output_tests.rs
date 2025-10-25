use quarter::{execute_line, Dictionary, LoopStack, Memory, ReturnStack, Stack};

#[test]
fn test_u_dot() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Test with positive number
    execute_line(
        "42 U.",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
        false,
        false,
        false,
    )
    .unwrap();
    assert!(stack.is_empty());

    // Test with -1 (should print as unsigned)
    execute_line(
        "-1 U.",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
        false,
        false,
        false,
    )
    .unwrap();
    assert!(stack.is_empty());
}

#[test]
fn test_dot_r() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Test right-justified printing
    execute_line(
        "42 10 .R",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
        false,
        false,
        false,
    )
    .unwrap();
    assert!(stack.is_empty());

    // Test with negative number
    execute_line(
        "-123 8 .R",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
        false,
        false,
        false,
    )
    .unwrap();
    assert!(stack.is_empty());

    // Test with number wider than field
    execute_line(
        "12345 3 .R",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
        false,
        false,
        false,
    )
    .unwrap();
    assert!(stack.is_empty());
}

#[test]
fn test_u_dot_r() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Test unsigned right-justified printing
    execute_line(
        "42 10 U.R",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
        false,
        false,
        false,
    )
    .unwrap();
    assert!(stack.is_empty());

    // Test with -1 as unsigned
    execute_line(
        "-1 12 U.R",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
        false,
        false,
        false,
    )
    .unwrap();
    assert!(stack.is_empty());
}
