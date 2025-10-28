use std::collections::HashSet;
use quarter::{execute_line, Dictionary, LoopStack, Memory, ReturnStack, Stack, RuntimeContext, CompilerConfig, ExecutionOptions};

#[test]
fn test_s_quote_basic() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    // Define a word that creates a string
    execute_line(
        ": TEST S\" Hello\" ;",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    // Execute it
    execute_line(
        "TEST",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    // Should push address and length
    let len = stack.pop(&mut memory).unwrap();
    let addr = stack.pop(&mut memory).unwrap();

    assert_eq!(len, 5); // "Hello" is 5 characters
    assert_eq!(addr, 131072); // First string at start of user memory
}

#[test]
fn test_s_quote_fetch_string() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    // Create a string
    execute_line(
        ": TEST S\" Hi!\" ;",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    execute_line(
        "TEST",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    let len = stack.pop(&mut memory).unwrap();
    let addr = stack.pop(&mut memory).unwrap();

    // Fetch characters from memory
    assert_eq!(memory.fetch_byte(addr as usize).unwrap(), 'H' as i64);
    assert_eq!(memory.fetch_byte((addr + 1) as usize).unwrap(), 'i' as i64);
    assert_eq!(memory.fetch_byte((addr + 2) as usize).unwrap(), '!' as i64);
    assert_eq!(len, 3);
}

#[test]
fn test_s_quote_multiple_strings() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Create two different strings
        execute_line(
            ": STR1 S\" First\" ;",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();

        execute_line(
            ": STR2 S\" Second\" ;",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();

        // Get first string
        execute_line(
            "STR1",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }

    let len1 = stack.pop(&mut memory).unwrap();
    let addr1 = stack.pop(&mut memory).unwrap();

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Get second string
        execute_line(
            "STR2",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }

    let len2 = stack.pop(&mut memory).unwrap();
    let addr2 = stack.pop(&mut memory).unwrap();

    // Strings should be at different addresses
    assert_eq!(len1, 5); // "First"
    assert_eq!(len2, 6); // "Second"
    assert_eq!(addr2, addr1 + 5); // Second string starts after first
}

#[test]
fn test_s_quote_empty_string() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    execute_line(
        ": EMPTY S\" \" ;",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    execute_line(
        "EMPTY",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    let len = stack.pop(&mut memory).unwrap();
    let _addr = stack.pop(&mut memory).unwrap();

    assert_eq!(len, 0);
}

#[test]
fn test_s_quote_with_spaces() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    execute_line(
        ": PHRASE S\" Hello World\" ;",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    execute_line(
        "PHRASE",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    let len = stack.pop(&mut memory).unwrap();
    let addr = stack.pop(&mut memory).unwrap();

    assert_eq!(len, 11); // "Hello World"

    // Verify the space is there
    assert_eq!(memory.fetch_byte((addr + 5) as usize).unwrap(), ' ' as i64);
}

#[test]
fn test_s_quote_advances_here() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    // Get initial HERE
    let initial_here = memory.here();

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Create a 10-character string
        execute_line(
            ": TEN S\" 1234567890\" ;",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }

    // HERE should still be at initial (string not yet allocated)
    assert_eq!(memory.here(), initial_here);

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Execute the word (allocates the string)
        execute_line(
            "TEN",
            &mut ctx,
            config,
            options,
            &mut HashSet::new(),
        )
        .unwrap();
    }

    // HERE should have advanced by 10 bytes
    assert_eq!(memory.here(), initial_here + 10);

    // Clean up stack
    stack.pop(&mut memory);
    stack.pop(&mut memory);
}

#[test]
fn test_s_quote_with_special_chars() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    execute_line(
        ": SPECIAL S\" !@#$%\" ;",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    execute_line(
        "SPECIAL",
        &mut ctx,
        config,
        options,
        &mut HashSet::new(),
    )
    .unwrap();

    let len = stack.pop(&mut memory).unwrap();
    let addr = stack.pop(&mut memory).unwrap();

    assert_eq!(len, 5);
    assert_eq!(memory.fetch_byte(addr as usize).unwrap(), '!' as i64);
    assert_eq!(memory.fetch_byte((addr + 1) as usize).unwrap(), '@' as i64);
}
