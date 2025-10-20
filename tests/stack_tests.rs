use quarter::{LoopStack, Stack};

#[test]
fn test_stack_push_pop() {
    let mut stack = Stack::new();
    let mut _loop_stack = LoopStack::new();
    stack.push(10);
    stack.push(20);

    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(10));
    assert_eq!(stack.pop(), None);
}

#[test]
fn test_stack_peek() {
    let mut stack = Stack::new();
    let mut _loop_stack = LoopStack::new();
    stack.push(42);

    assert_eq!(stack.peek(), Some(&42));
    assert_eq!(stack.pop(), Some(42));
}

#[test]
fn test_stack_is_empty() {
    let mut stack = Stack::new();
    let mut _loop_stack = LoopStack::new();
    assert!(stack.is_empty());

    stack.push(1);
    assert!(!stack.is_empty());

    stack.pop();
    assert!(stack.is_empty());
}
