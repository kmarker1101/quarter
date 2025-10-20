use quarter::{LoopStack, Dictionary, Stack};

#[test]
fn test_less_than_true() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(3);
    stack.push(5);
    dict.execute_word("<", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(-1)); // true
}

#[test]
fn test_less_than_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(5);
    stack.push(3);
    dict.execute_word("<", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}

#[test]
fn test_greater_than_true() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(5);
    stack.push(3);
    dict.execute_word(">", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(-1)); // true
}

#[test]
fn test_greater_than_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(3);
    stack.push(5);
    dict.execute_word(">", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}

#[test]
fn test_equals_true() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(5);
    stack.push(5);
    dict.execute_word("=", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(-1)); // true
}

#[test]
fn test_equals_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(3);
    stack.push(5);
    dict.execute_word("=", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}

#[test]
fn test_not_equals_true() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(3);
    stack.push(5);
    dict.execute_word("<>", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(-1)); // true
}

#[test]
fn test_not_equals_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(5);
    stack.push(5);
    dict.execute_word("<>", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}

#[test]
fn test_less_or_equal_true_less() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(3);
    stack.push(5);
    dict.execute_word("<=", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(-1)); // true
}

#[test]
fn test_less_or_equal_true_equal() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(5);
    stack.push(5);
    dict.execute_word("<=", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(-1)); // true
}

#[test]
fn test_less_or_equal_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(5);
    stack.push(3);
    dict.execute_word("<=", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}

#[test]
fn test_greater_or_equal_true_greater() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(5);
    stack.push(3);
    dict.execute_word(">=", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(-1)); // true
}

#[test]
fn test_greater_or_equal_true_equal() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(5);
    stack.push(5);
    dict.execute_word(">=", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(-1)); // true
}

#[test]
fn test_greater_or_equal_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(3);
    stack.push(5);
    dict.execute_word(">=", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}
