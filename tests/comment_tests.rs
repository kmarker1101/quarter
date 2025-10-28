use std::collections::HashSet;
use quarter::{execute_line, load_stdlib, strip_comments, Dictionary, LoopStack, Memory, ReturnStack, Stack, RuntimeContext, CompilerConfig, ExecutionOptions};

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

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    execute_line(
        "5 3 + \\ this is a comment",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
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

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    execute_line(
        "10 ( first number ) 20 ( second number ) +",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
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

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    // Define a word with stack effect notation
    execute_line(
        ": SQUARE ( n -- n² ) DUP * ;",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    // Use the word
    execute_line(
        "5 SQUARE",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
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

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Load stdlib to get NEGATE
        load_stdlib(&mut ctx, config, options, &mut HashSet::new()).unwrap();

        // Define with both comment types
        execute_line(
            ": ABS ( n -- |n| ) DUP 0 < IF NEGATE THEN ; \\ absolute value",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }

    // Test with negative number
    stack.push(-42, &mut memory);
    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);
        execute_line(
            "ABS",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(42));

    // Test with positive number
    stack.push(42, &mut memory);
    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);
        execute_line(
            "ABS",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(42));
}

#[test]
fn test_multiple_parenthesis_comments() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    execute_line(
        "5 ( a ) DUP ( b ) * ( c ) 10 ( d ) +",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
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

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    // Should not error on comment-only input
    execute_line(
        "\\ just a comment",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    execute_line(
        "( just a comment )",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    assert!(stack.is_empty());
}
