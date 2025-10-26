use quarter::{Dictionary, LoopStack, Memory, ReturnStack, Stack};

#[test]
fn test_less_than() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 3 < 5 → -1 (true)
    stack.push(3, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 5 < 3 → 0 (false)
    stack.push(5, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word("<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // 3 < 3 → 0 (false, equal values)
    stack.push(3, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word("<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // -5 < -3 → -1 (true, negative numbers)
    stack.push(-5, &mut memory);
    stack.push(-3, &mut memory);
    dict.execute_word("<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // -3 < 5 → -1 (true, negative vs positive)
    stack.push(-3, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));
}

#[test]
fn test_greater_than() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 5 > 3 → -1 (true)
    stack.push(5, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word(">", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 3 > 5 → 0 (false)
    stack.push(3, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word(">", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // 3 > 3 → 0 (false, equal values)
    stack.push(3, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word(">", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // -3 > -5 → -1 (true, negative numbers)
    stack.push(-3, &mut memory);
    stack.push(-5, &mut memory);
    dict.execute_word(">", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 5 > -3 → -1 (true, positive vs negative)
    stack.push(5, &mut memory);
    stack.push(-3, &mut memory);
    dict.execute_word(">", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));
}

#[test]
fn test_equal() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 5 = 5 → -1 (true)
    stack.push(5, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 5 = 3 → 0 (false)
    stack.push(5, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word("=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // 0 = 0 → -1 (true, zero)
    stack.push(0, &mut memory);
    stack.push(0, &mut memory);
    dict.execute_word("=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // -3 = -3 → -1 (true, negative)
    stack.push(-3, &mut memory);
    stack.push(-3, &mut memory);
    dict.execute_word("=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // -3 = 3 → 0 (false, different sign)
    stack.push(-3, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word("=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));
}

#[test]
fn test_not_equal() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 5 <> 3 → -1 (true)
    stack.push(5, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word("<>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 5 <> 5 → 0 (false)
    stack.push(5, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("<>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // 0 <> 0 → 0 (false, zero)
    stack.push(0, &mut memory);
    stack.push(0, &mut memory);
    dict.execute_word("<>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // -3 <> 3 → -1 (true, different sign)
    stack.push(-3, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word("<>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));
}

#[test]
fn test_less_equal() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 3 <= 5 → -1 (true, less than)
    stack.push(3, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("<=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 5 <= 5 → -1 (true, equal)
    stack.push(5, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("<=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 5 <= 3 → 0 (false, greater than)
    stack.push(5, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word("<=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // -5 <= -3 → -1 (true, negative numbers)
    stack.push(-5, &mut memory);
    stack.push(-3, &mut memory);
    dict.execute_word("<=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 0 <= 0 → -1 (true, zero)
    stack.push(0, &mut memory);
    stack.push(0, &mut memory);
    dict.execute_word("<=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));
}

#[test]
fn test_greater_equal() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 5 >= 3 → -1 (true, greater than)
    stack.push(5, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word(">=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 5 >= 5 → -1 (true, equal)
    stack.push(5, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word(">=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 3 >= 5 → 0 (false, less than)
    stack.push(3, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word(">=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // -3 >= -5 → -1 (true, negative numbers)
    stack.push(-3, &mut memory);
    stack.push(-5, &mut memory);
    dict.execute_word(">=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 0 >= 0 → -1 (true, zero)
    stack.push(0, &mut memory);
    stack.push(0, &mut memory);
    dict.execute_word(">=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));
}

#[test]
fn test_zero_equal() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 0 0= → -1 (true)
    stack.push(0, &mut memory);
    dict.execute_word("0=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 5 0= → 0 (false)
    stack.push(5, &mut memory);
    dict.execute_word("0=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // -3 0= → 0 (false)
    stack.push(-3, &mut memory);
    dict.execute_word("0=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // 1 0= → 0 (false, positive)
    stack.push(1, &mut memory);
    dict.execute_word("0=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // -1 0= → 0 (false, negative)
    stack.push(-1, &mut memory);
    dict.execute_word("0=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));
}

#[test]
fn test_zero_less() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // -5 0< → -1 (true)
    stack.push(-5, &mut memory);
    dict.execute_word("0<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 0 0< → 0 (false)
    stack.push(0, &mut memory);
    dict.execute_word("0<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // 5 0< → 0 (false)
    stack.push(5, &mut memory);
    dict.execute_word("0<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // -1 0< → -1 (true, edge case)
    stack.push(-1, &mut memory);
    dict.execute_word("0<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 1 0< → 0 (false, edge case)
    stack.push(1, &mut memory);
    dict.execute_word("0<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));
}

#[test]
fn test_zero_greater() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 5 0> → -1 (true)
    stack.push(5, &mut memory);
    dict.execute_word("0>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 0 0> → 0 (false)
    stack.push(0, &mut memory);
    dict.execute_word("0>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // -5 0> → 0 (false)
    stack.push(-5, &mut memory);
    dict.execute_word("0>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // 1 0> → -1 (true, edge case)
    stack.push(1, &mut memory);
    dict.execute_word("0>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // -1 0> → 0 (false, edge case)
    stack.push(-1, &mut memory);
    dict.execute_word("0>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));
}
