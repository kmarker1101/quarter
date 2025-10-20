use quarter::{LoopStack, Stack, Memory};

#[test]
fn test_stack_push_pop() {
    let mut stack = Stack::new();
    let mut memory = Memory::new();
    let mut _loop_stack = LoopStack::new();
    stack.push(10, &mut memory);
    stack.push(20, &mut memory);

    assert_eq!(stack.pop(&mut memory), Some(20));
    assert_eq!(stack.pop(&mut memory), Some(10));
    assert_eq!(stack.pop(&mut memory), None);
}

#[test]
fn test_stack_peek() {
    let mut stack = Stack::new();
    let mut memory = Memory::new();
    let mut _loop_stack = LoopStack::new();
    stack.push(42, &mut memory);

    assert_eq!(stack.peek(&memory), Some(42));
    assert_eq!(stack.pop(&mut memory), Some(42));
}

#[test]
fn test_stack_is_empty() {
    let mut stack = Stack::new();
    let mut memory = Memory::new();
    let mut _loop_stack = LoopStack::new();
    assert!(stack.is_empty());

    stack.push(1, &mut memory);
    assert!(!stack.is_empty());

    stack.pop(&mut memory);
    assert!(stack.is_empty());
}
