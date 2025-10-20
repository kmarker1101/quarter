use quarter::{Dictionary, LoopStack, Memory, ReturnStack, Stack};

#[test]
fn test_sp_fetch() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Empty stack - SP should be 0
    dict.execute_word("SP@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // Push 3 values - SP should be 12 (3 cells * 4 bytes)
    stack.push(10, &mut memory);
    stack.push(20, &mut memory);
    stack.push(30, &mut memory);
    dict.execute_word("SP@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(12)); // SP was 12 before SP@ pushed the value
}

#[test]
fn test_sp_store() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Push some values
    stack.push(10, &mut memory);
    stack.push(20, &mut memory);
    stack.push(30, &mut memory);

    // Reset SP to 4 (1 cell)
    stack.push(4, &mut memory);
    dict.execute_word("SP!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Verify SP was set
    dict.execute_word("SP@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(4)); // SP is now 4
}

#[test]
fn test_rp_fetch() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Empty return stack - RP should be 0x010000 (65536)
    dict.execute_word("RP@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(65536));

    // Push 2 values to return stack - RP should be 0x010008 (65544)
    stack.push(100, &mut memory);
    dict.execute_word(">R", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    stack.push(200, &mut memory);
    dict.execute_word(">R", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    dict.execute_word("RP@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(65544)); // 65536 + 8
}

#[test]
fn test_rp_store() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Push some values to return stack
    stack.push(100, &mut memory);
    dict.execute_word(">R", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    stack.push(200, &mut memory);
    dict.execute_word(">R", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Reset RP to 0x010000
    stack.push(65536, &mut memory);
    dict.execute_word("RP!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Verify RP was reset
    dict.execute_word("RP@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(65536));
}

#[test]
fn test_cells() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 0 CELLS = 0
    stack.push(0, &mut memory);
    dict.execute_word("CELLS", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // 1 CELLS = 4
    stack.push(1, &mut memory);
    dict.execute_word("CELLS", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(4));

    // 3 CELLS = 12
    stack.push(3, &mut memory);
    dict.execute_word("CELLS", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(12));

    // 10 CELLS = 40
    stack.push(10, &mut memory);
    dict.execute_word("CELLS", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(40));
}

#[test]
fn test_cell_plus() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 0 CELL+ = 4
    stack.push(0, &mut memory);
    dict.execute_word("CELL+", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(4));

    // 100 CELL+ = 104
    stack.push(100, &mut memory);
    dict.execute_word("CELL+", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(104));

    // 131072 CELL+ = 131076 (0x020000 CELL+ = 0x020004)
    stack.push(131072, &mut memory);
    dict.execute_word("CELL+", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(131076));
}

#[test]
fn test_plus_store() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let addr = 131072; // 0x020000

    // Store initial value
    stack.push(42, &mut memory);
    stack.push(addr, &mut memory);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Add 10 to it
    stack.push(10, &mut memory);
    stack.push(addr, &mut memory);
    dict.execute_word("+!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Fetch and verify
    stack.push(addr, &mut memory);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(52));

    // Add negative value
    stack.push(-5, &mut memory);
    stack.push(addr, &mut memory);
    dict.execute_word("+!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Fetch and verify
    stack.push(addr, &mut memory);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(47));
}

#[test]
fn test_plus_store_zero() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    let addr = 131072;

    // Store initial value
    stack.push(100, &mut memory);
    stack.push(addr, &mut memory);
    dict.execute_word("!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Add 0 (no change)
    stack.push(0, &mut memory);
    stack.push(addr, &mut memory);
    dict.execute_word("+!", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Verify unchanged
    stack.push(addr, &mut memory);
    dict.execute_word("@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(100));
}
