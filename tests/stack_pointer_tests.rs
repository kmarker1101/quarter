use quarter::{Dictionary, LoopStack, Memory, ReturnStack, Stack};

#[test]
fn test_sp_fetch() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Empty stack - SP should be 0
    dict.execute_word(
        "SP@",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // Push 3 values - SP should be 24 (3 cells * 8 bytes)
    stack.push(10, &mut memory);
    stack.push(20, &mut memory);
    stack.push(30, &mut memory);
    dict.execute_word(
        "SP@",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(24)); // SP was 24 before SP@ pushed the value
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

    // Reset SP to 8 (1 cell)
    stack.push(8, &mut memory);
    dict.execute_word(
        "SP!",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Verify SP was set
    dict.execute_word(
        "SP@",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(8)); // SP is now 8
}

#[test]
fn test_rp_fetch() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Empty return stack - RP should be 0x010000 (65536)
    dict.execute_word(
        "RP@",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(65536));

    // Push 2 values to return stack - RP should be 0x010010 (65552)
    stack.push(100, &mut memory);
    dict.execute_word(
        ">R",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    stack.push(200, &mut memory);
    dict.execute_word(
        ">R",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    dict.execute_word(
        "RP@",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(65552)); // 65536 + 16
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
    dict.execute_word(
        ">R",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    stack.push(200, &mut memory);
    dict.execute_word(
        ">R",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Reset RP to 0x010000
    stack.push(65536, &mut memory);
    dict.execute_word(
        "RP!",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Verify RP was reset
    dict.execute_word(
        "RP@",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(65536));
}
