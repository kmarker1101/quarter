use quarter::{Dictionary, Stack};

#[test]
fn test_dot_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    stack.push(42);
    assert!(dict.execute_word(".", &mut stack).is_ok());
    assert!(stack.is_empty());
}

#[test]
fn test_add_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    stack.push(5);
    stack.push(3);
    dict.execute_word("+", &mut stack).unwrap();

    assert_eq!(stack.pop(), Some(8));
}

#[test]
fn test_subtract_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("-", &mut stack).unwrap();

    assert_eq!(stack.pop(), Some(-1));
}

#[test]
fn test_multiply_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("*", &mut stack).unwrap();

    assert_eq!(stack.pop(), Some(42));
}

#[test]
fn test_divide_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    stack.push(12);
    stack.push(5);
    dict.execute_word("/", &mut stack).unwrap();

    assert_eq!(stack.pop(), Some(2));
}

#[test]
fn test_mod_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    stack.push(13);
    stack.push(5);
    dict.execute_word("MOD", &mut stack).unwrap();

    assert_eq!(stack.pop(), Some(3));
}

#[test]
fn test_dup_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("DUP", &mut stack).unwrap();

    assert_eq!(stack.pop(), Some(7));
    assert_eq!(stack.pop(), Some(7));
    assert_eq!(stack.pop(), Some(6));
}

#[test]
fn test_swap_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("SWAP", &mut stack).unwrap();

    assert_eq!(stack.pop(), Some(6));
    assert_eq!(stack.pop(), Some(7));
}

#[test]
fn test_dot_s_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    stack.push(10);
    stack.push(20);
    stack.push(30);

    // .S should not modify the stack
    assert!(dict.execute_word(".S", &mut stack).is_ok());
    assert_eq!(stack.pop(), Some(30));
    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(10));
}

#[test]
fn test_negate_positive() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    stack.push(42);
    dict.execute_word("NEGATE", &mut stack).unwrap();

    assert_eq!(stack.pop(), Some(-42));
}

#[test]
fn test_negate_negative() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    stack.push(-42);
    dict.execute_word("NEGATE", &mut stack).unwrap();

    assert_eq!(stack.pop(), Some(42));
}

#[test]
fn test_unknown_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    assert!(dict.execute_word("UNKNOWN", &mut stack).is_err());
}
