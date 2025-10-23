use quarter::{execute_line, load_stdlib, strip_comments, Dictionary, LoopStack, Memory, ReturnStack, Stack};

#[test]
fn test_strip_comments_backslash() {
    assert_eq!(strip_comments("5 3 +"), "5 3 +");
    assert_eq!(strip_comments("5 3 + \\ add numbers"), "5 3 + ");
    assert_eq!(strip_comments("\\ full line comment"), "");
    assert_eq!(strip_comments("DUP \\ duplicate"), "DUP ");
}

#[test]
fn test_strip_comments_parenthesis() {
    assert_eq!(strip_comments("5 ( comment ) 3 +"), "5  3 +");
    assert_eq!(strip_comments("( full line comment )"), "");
    assert_eq!(strip_comments("10 ( a ) 20 ( b ) +"), "10  20  +");
}

#[test]
fn test_strip_comments_mixed() {
    assert_eq!(strip_comments("5 ( inline ) 3 + \\ end of line"), "5  3 + ");
    assert_eq!(strip_comments("( comment ) DUP \\ another"), " DUP ");
}

#[test]
fn test_backslash_comment_in_execution() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    execute_line(
        "5 3 + \\ this is a comment",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
    )
    .unwrap();

    assert_eq!(stack.pop(&mut memory), Some(8));
}

#[test]
fn test_parenthesis_comment_in_execution() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    execute_line(
        "10 ( first number ) 20 ( second number ) +",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
    )
    .unwrap();

    assert_eq!(stack.pop(&mut memory), Some(30));
}

#[test]
fn test_stack_effect_notation() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Define a word with stack effect notation
    execute_line(
        ": SQUARE ( n -- n² ) DUP * ;",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
    )
    .unwrap();

    // Use the word
    execute_line(
        "5 SQUARE",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
    )
    .unwrap();

    assert_eq!(stack.pop(&mut memory), Some(25));
}

#[test]
fn test_mixed_comments_in_definition() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Load stdlib to get NEGATE
    load_stdlib(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory, false).unwrap();

    // Define with both comment types
    execute_line(
        ": ABS ( n -- |n| ) DUP 0 < IF NEGATE THEN ; \\ absolute value",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
    )
    .unwrap();

    // Test with negative number
    stack.push(-42, &mut memory);
    execute_line(
        "ABS",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(42));

    // Test with positive number
    stack.push(42, &mut memory);
    execute_line(
        "ABS",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(42));
}

#[test]
fn test_multiple_parenthesis_comments() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    execute_line(
        "5 ( a ) DUP ( b ) * ( c ) 10 ( d ) +",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
    )
    .unwrap();

    // 5 DUP * 10 + = 5 * 5 + 10 = 35
    assert_eq!(stack.pop(&mut memory), Some(35));
}

#[test]
fn test_comment_only_line() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Should not error on comment-only input
    execute_line(
        "\\ just a comment",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
    )
    .unwrap();

    execute_line(
        "( just a comment )",
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        false,
    )
    .unwrap();

    assert!(stack.is_empty());
}
