use quarter::{Dictionary, LoopStack, Memory, ReturnStack, Stack};

#[test]
fn test_store_fetch_basic() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store 42 at address 100
    stack.push(42);
    stack.push(100);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Fetch from address 100
    stack.push(100);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(42));
}

#[test]
fn test_store_fetch_negative() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store -12345 at address 0
    stack.push(-12345);
    stack.push(0);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Fetch from address 0
    stack.push(0);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(-12345));
}

#[test]
fn test_store_fetch_multiple_locations() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store different values at different locations
    stack.push(111);
    stack.push(0);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    stack.push(222);
    stack.push(4);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    stack.push(333);
    stack.push(8);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Fetch them back
    stack.push(8);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(333));

    stack.push(4);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(222));

    stack.push(0);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(111));
}

#[test]
fn test_c_store_fetch_basic() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store byte 65 ('A') at address 10
    stack.push(65);
    stack.push(10);
    dict.execute_word("C!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Fetch byte from address 10
    stack.push(10);
    dict.execute_word("C@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(65));
}

#[test]
fn test_c_store_fetch_multiple_bytes() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store "Hello" as bytes
    let hello_bytes = vec![72, 101, 108, 108, 111]; // "Hello"
    for (i, byte) in hello_bytes.iter().enumerate() {
        stack.push(*byte);
        stack.push(i as i32);
        dict.execute_word("C!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .unwrap();
    }

    // Read them back
    for i in 0..5 {
        stack.push(i);
        dict.execute_word("C@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .unwrap();
    }

    // Stack now has: 72 101 108 108 111 (top)
    assert_eq!(stack.pop(), Some(111)); // o
    assert_eq!(stack.pop(), Some(108)); // l
    assert_eq!(stack.pop(), Some(108)); // l
    assert_eq!(stack.pop(), Some(101)); // e
    assert_eq!(stack.pop(), Some(72));  // H
}

#[test]
fn test_c_store_masks_to_byte() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store 0x12345678, should only store 0x78
    stack.push(0x12345678);
    stack.push(20);
    dict.execute_word("C!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Fetch byte
    stack.push(20);
    dict.execute_word("C@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(0x78));
}

#[test]
fn test_little_endian_encoding() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store 0x12345678 at address 100
    stack.push(0x12345678);
    stack.push(100);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // In little-endian, bytes should be: 78 56 34 12
    stack.push(100);
    dict.execute_word("C@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(0x78));

    stack.push(101);
    dict.execute_word("C@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(0x56));

    stack.push(102);
    dict.execute_word("C@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(0x34));

    stack.push(103);
    dict.execute_word("C@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(0x12));
}

#[test]
fn test_overwrite_value() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store 100 at address 50
    stack.push(100);
    stack.push(50);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Overwrite with 200
    stack.push(200);
    stack.push(50);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Fetch should return 200
    stack.push(50);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(200));
}

#[test]
fn test_memory_isolation() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store value at 1000
    stack.push(999);
    stack.push(1000);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Store value at 2000 (far away)
    stack.push(888);
    stack.push(2000);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Fetch from 1000 - should not be affected by store at 2000
    stack.push(1000);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(999));

    // Fetch from 2000
    stack.push(2000);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(888));
}

#[test]
fn test_zero_address() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Store at address 0 (should work fine)
    stack.push(42);
    stack.push(0);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Fetch from address 0
    stack.push(0);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(42));
}

#[test]
fn test_max_valid_address() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 8MB = 8*1024*1024 bytes
    // Max valid address for i32 store is 8*1024*1024 - 4
    let max_addr = 8 * 1024 * 1024 - 4;

    stack.push(12345);
    stack.push(max_addr);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    stack.push(max_addr);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(12345));
}

#[test]
fn test_max_valid_byte_address() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Max valid address for byte is 8*1024*1024 - 1
    let max_addr = 8 * 1024 * 1024 - 1;

    stack.push(99);
    stack.push(max_addr);
    dict.execute_word("C!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    stack.push(max_addr);
    dict.execute_word("C@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(99));
}

#[test]
fn test_byte_array_operations() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Write sequence of bytes starting at 1000
    for i in 0..10 {
        stack.push(i * 10);
        stack.push(1000 + i);
        dict.execute_word("C!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .unwrap();
    }

    // Read them back in reverse
    for i in (0..10).rev() {
        stack.push(1000 + i);
        dict.execute_word("C@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .unwrap();
        assert_eq!(stack.pop(), Some(i * 10));
    }
}
