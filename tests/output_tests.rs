use std::collections::HashSet;
use quarter::{execute_line, Dictionary, LoopStack, Memory, ReturnStack, Stack, RuntimeContext, CompilerConfig, ExecutionOptions};

#[test]
fn test_u_dot() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Test with positive number
        execute_line(
            "42 U.",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }
    assert!(stack.is_empty());

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Test with -1 (should print as unsigned)
        execute_line(
            "-1 U.",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }
    assert!(stack.is_empty());
}

#[test]
fn test_dot_r() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Test right-justified printing
        execute_line(
            "42 10 .R",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }
    assert!(stack.is_empty());

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Test with negative number
        execute_line(
            "-123 8 .R",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }
    assert!(stack.is_empty());

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Test with number wider than field
        execute_line(
            "12345 3 .R",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }
    assert!(stack.is_empty());
}

#[test]
fn test_u_dot_r() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Test unsigned right-justified printing
        execute_line(
            "42 10 U.R",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }
    assert!(stack.is_empty());

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Test with -1 as unsigned
        execute_line(
            "-1 12 U.R",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }
    assert!(stack.is_empty());
}
