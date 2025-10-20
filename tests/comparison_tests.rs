use quarter::{LoopStack, Dictionary, ReturnStack, Stack, Memory};

#[test]
fn test_less_than_true() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(3, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(-1)); // true
}

#[test]
fn test_less_than_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word("<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(0)); // false
}

#[test]
fn test_greater_than_true() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word(">", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(-1)); // true
}

#[test]
fn test_greater_than_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(3, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word(">", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(0)); // false
}

#[test]
fn test_equals_true() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(-1)); // true
}

#[test]
fn test_equals_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(3, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(0)); // false
}

#[test]
fn test_not_equals_true() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(3, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("<>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(-1)); // true
}

#[test]
fn test_not_equals_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("<>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(0)); // false
}

#[test]
fn test_less_or_equal_true_less() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(3, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("<=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(-1)); // true
}

#[test]
fn test_less_or_equal_true_equal() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word("<=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(-1)); // true
}

#[test]
fn test_less_or_equal_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word("<=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(0)); // false
}

#[test]
fn test_greater_or_equal_true_greater() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word(">=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(-1)); // true
}

#[test]
fn test_greater_or_equal_true_equal() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word(">=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(-1)); // true
}

#[test]
fn test_greater_or_equal_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(3, &mut memory);
    stack.push(5, &mut memory);
    dict.execute_word(">=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(&mut memory), Some(0)); // false
}
