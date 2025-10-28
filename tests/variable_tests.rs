use std::collections::HashSet;
use quarter::{execute_line, Dictionary, LoopStack, Memory, ReturnStack, Stack, RuntimeContext, CompilerConfig, ExecutionOptions};

#[test]
fn test_here_initial() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // HERE should initially return 0x020000 (131072)
    dict.execute_word("HERE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(131072));
}

#[test]
fn test_allot() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Get initial HERE
    dict.execute_word("HERE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    let initial_here = stack.pop(&mut memory).unwrap();

    // ALLOT 16 bytes
    stack.push(16, &mut memory);
    dict.execute_word("ALLOT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // HERE should be 16 bytes higher
    dict.execute_word("HERE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(initial_here + 16));
}

#[test]
fn test_variable_basic() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Create a variable
        execute_line("VARIABLE X", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();

        // X should push its address (which should be at initial HERE = 131072)
        execute_line("X", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
    }
    let addr = stack.pop(&mut memory).unwrap();
    assert_eq!(addr, 131072);

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Store a value
        execute_line("42 X !", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();

        // Fetch the value
        execute_line("X @", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(42));
}

#[test]
fn test_variable_multiple() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Create two variables
        execute_line("VARIABLE FOO", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
        execute_line("VARIABLE BAR", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();

        // Store different values
        execute_line("10 FOO !", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
        execute_line("20 BAR !", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();

        // Fetch and verify
        execute_line("FOO @", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(10));

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);
        execute_line("BAR @", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(20));
}

#[test]
fn test_constant_basic() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Create a constant
        execute_line("100 CONSTANT HUNDRED", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();

        // HUNDRED should push 100
        execute_line("HUNDRED", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(100));

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Can use multiple times
        execute_line("HUNDRED HUNDRED +", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(200));
}

#[test]
fn test_constant_in_definition() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    // Create constants
    execute_line("10 CONSTANT TEN", &mut ctx, config, options, &mut HashSet::new())
        .unwrap();
    execute_line("5 CONSTANT FIVE", &mut ctx, config, options, &mut HashSet::new())
        .unwrap();

    // Use in a word definition
    execute_line(": SUM TEN FIVE + ;", &mut ctx, config, options, &mut HashSet::new())
        .unwrap();

    // Execute the word
    execute_line("SUM", &mut ctx, config, options, &mut HashSet::new())
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(15));
}

#[test]
fn test_variable_constant_together() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    // Create a variable and a constant
    execute_line("VARIABLE COUNTER", &mut ctx, config, options, &mut HashSet::new())
        .unwrap();
    execute_line("42 CONSTANT ANSWER", &mut ctx, config, options, &mut HashSet::new())
        .unwrap();

    // Use them together
    execute_line("ANSWER COUNTER !", &mut ctx, config, options, &mut HashSet::new())
        .unwrap();
    execute_line("COUNTER @", &mut ctx, config, options, &mut HashSet::new())
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(42));
}

#[test]
fn test_here_after_variables() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    let initial = {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Get initial HERE
        execute_line("HERE", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
        stack.pop(&mut memory).unwrap()
    };

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Create a variable (allocates 8 bytes)
        execute_line("VARIABLE X", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();

        // HERE should be 8 bytes higher
        execute_line("HERE", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(initial + 8));
}

#[test]
fn test_comma_basic() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Get initial HERE
    dict.execute_word("HERE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    let addr = stack.pop(&mut memory).unwrap();

    // Store some values with comma
    stack.push(42, &mut memory);
    dict.execute_word(",", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    stack.push(43, &mut memory);
    dict.execute_word(",", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    stack.push(44, &mut memory);
    dict.execute_word(",", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // HERE should have advanced by 24 (3 cells * 8 bytes)
    dict.execute_word("HERE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(addr + 24));

    // Fetch the values back
    assert_eq!(memory.fetch(addr as usize).unwrap(), 42);
    assert_eq!(memory.fetch((addr + 8) as usize).unwrap(), 43);
    assert_eq!(memory.fetch((addr + 16) as usize).unwrap(), 44);
}

#[test]
fn test_create_basic() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);
    let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

    // CREATE BUFFER should create a word that pushes an address
    execute_line("CREATE BUFFER", &mut ctx, config, options, &mut HashSet::new())
        .unwrap();

    // BUFFER should push its address
    execute_line("BUFFER", &mut ctx, config, options, &mut HashSet::new())
        .unwrap();
    let addr = stack.pop(&mut memory).unwrap();
    assert_eq!(addr, 131072); // Initial HERE
}

#[test]
fn test_create_with_allot() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    let buffer_addr = {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // CREATE BUFFER 100 ALLOT
        execute_line("CREATE BUFFER 100 ALLOT", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();

        // BUFFER should push its address
        execute_line("BUFFER", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
        stack.pop(&mut memory).unwrap()
    };

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Store and fetch from the buffer
        execute_line("42 BUFFER !", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
        execute_line("BUFFER @", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(42));

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // HERE should be 100 bytes past BUFFER
        execute_line("HERE", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(buffer_addr + 100));
}

#[test]
fn test_create_multiple_buffers() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let config = CompilerConfig::new(false, false, false);
    let options = ExecutionOptions::new(false, false);

    let buf1_addr = {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Create two buffers
        execute_line("CREATE BUF1 20 ALLOT", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
        execute_line("CREATE BUF2 30 ALLOT", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();

        // Get their addresses
        execute_line("BUF1", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
        stack.pop(&mut memory).unwrap()
    };

    let buf2_addr = {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);
        execute_line("BUF2", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
        stack.pop(&mut memory).unwrap()
    };

    // BUF2 should be 20 bytes after BUF1
    assert_eq!(buf2_addr, buf1_addr + 20);

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);

        // Store different values
        execute_line("100 BUF1 !", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
        execute_line("200 BUF2 !", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();

        // Fetch and verify
        execute_line("BUF1 @", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(100));

    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);
        execute_line("BUF2 @", &mut ctx, config, options, &mut HashSet::new())
            .unwrap();
    }
    assert_eq!(stack.pop(&mut memory), Some(200));
}

#[test]
fn test_comma_and_fetch() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Get starting address
    dict.execute_word("HERE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    let start_addr = stack.pop(&mut memory).unwrap();

    // Compile a small array
    stack.push(10, &mut memory);
    dict.execute_word(",", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    stack.push(20, &mut memory);
    dict.execute_word(",", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    stack.push(30, &mut memory);
    dict.execute_word(",", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Fetch them back using @
    stack.push(start_addr, &mut memory);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(10));

    stack.push(start_addr + 8, &mut memory);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(20));

    stack.push(start_addr + 16, &mut memory);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(30));
}
