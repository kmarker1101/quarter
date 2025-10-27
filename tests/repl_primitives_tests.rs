use quarter::{Dictionary, LoopStack, Memory, ReturnStack, Stack};

// Test CMOVE primitive
#[test]
fn test_cmove_basic() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store "HELLO" at address 0x200000
    let src_addr = 0x200000;
    let hello = b"HELLO";
    for (i, &ch) in hello.iter().enumerate() {
        memory.store_byte(src_addr + i, ch as i64).unwrap();
    }

    // CMOVE from src to dest
    let dest_addr = 0x201000;
    stack.push(src_addr as i64, &mut memory); // source
    stack.push(dest_addr as i64, &mut memory); // dest
    stack.push(5, &mut memory); // count

    dict.execute_word("CMOVE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Verify destination has "HELLO"
    assert_eq!(memory.fetch_byte(dest_addr).unwrap(), b'H' as i64);
    assert_eq!(memory.fetch_byte(dest_addr + 1).unwrap(), b'E' as i64);
    assert_eq!(memory.fetch_byte(dest_addr + 2).unwrap(), b'L' as i64);
    assert_eq!(memory.fetch_byte(dest_addr + 3).unwrap(), b'L' as i64);
    assert_eq!(memory.fetch_byte(dest_addr + 4).unwrap(), b'O' as i64);
    assert!(stack.is_empty());
}

#[test]
fn test_cmove_zero_count() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // CMOVE with count=0 should do nothing
    stack.push(0x200000, &mut memory); // source
    stack.push(0x201000, &mut memory); // dest
    stack.push(0, &mut memory); // count

    dict.execute_word("CMOVE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Stack should be empty
    assert!(stack.is_empty());
}

#[test]
fn test_cmove_large_block() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store 256 bytes at source
    let src_addr = 0x200000;
    for i in 0..256 {
        memory.store_byte(src_addr + i, i as i64).unwrap();
    }

    // CMOVE 256 bytes
    let dest_addr = 0x201000;
    stack.push(src_addr as i64, &mut memory);
    stack.push(dest_addr as i64, &mut memory);
    stack.push(256, &mut memory);

    dict.execute_word("CMOVE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Verify all bytes copied correctly
    for i in 0..256 {
        assert_eq!(memory.fetch_byte(dest_addr + i).unwrap(), i as i64);
    }
    assert!(stack.is_empty());
}

#[test]
fn test_throw_with_nonzero() {
    // THROW currently just prints a message and doesn't actually throw
    // This test documents the current behavior
    // See issue #80 for proper THROW implementation
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Push non-zero error code
    stack.push(42, &mut memory);

    // THROW pops the value, prints message, then pushes it back (for CATCH to see)
    dict.execute_word("THROW", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Stack should have the error code back on it (current behavior)
    assert_eq!(stack.pop(&mut memory), Some(42));
    assert!(stack.is_empty());
}

#[test]
fn test_throw_with_zero() {
    // THROW with 0 should do nothing
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Push zero
    stack.push(0, &mut memory);

    dict.execute_word("THROW", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Stack should be empty
    assert!(stack.is_empty());
}
