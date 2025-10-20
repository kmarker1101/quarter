use quarter::{Dictionary, LoopStack, Stack, parse_tokens};

#[test]
fn test_dot_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    stack.push(42);
    assert!(dict.execute_word(".", &mut stack, &mut loop_stack).is_ok());
    assert!(stack.is_empty());
}

#[test]
fn test_add_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    stack.push(5);
    stack.push(3);
    dict.execute_word("+", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(8));
}

#[test]
fn test_subtract_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("-", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(-1));
}

#[test]
fn test_multiply_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("*", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(42));
}

#[test]
fn test_divide_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(12);
    stack.push(5);
    dict.execute_word("/", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(2));
}

#[test]
fn test_mod_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(13);
    stack.push(5);
    dict.execute_word("MOD", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(3));
}

#[test]
fn test_slash_mod_basic() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(10);
    stack.push(3);
    dict.execute_word("/MOD", &mut stack, &mut loop_stack).unwrap();

    // Should leave remainder then quotient: 1 3
    assert_eq!(stack.pop(), Some(3)); // quotient on top
    assert_eq!(stack.pop(), Some(1)); // remainder below
}

#[test]
fn test_dup_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("DUP", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(7));
    assert_eq!(stack.pop(), Some(7));
    assert_eq!(stack.pop(), Some(6));
}

#[test]
fn test_swap_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("SWAP", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(6));
    assert_eq!(stack.pop(), Some(7));
}

#[test]
fn test_dot_s_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(10);
    stack.push(20);
    stack.push(30);

    // .S should not modify the stack
    assert!(dict.execute_word(".S", &mut stack, &mut loop_stack).is_ok());
    assert_eq!(stack.pop(), Some(30));
    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(10));
}

#[test]
fn test_negate_positive() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(42);
    dict.execute_word("NEGATE", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(-42));
}

#[test]
fn test_negate_negative() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(-42);
    dict.execute_word("NEGATE", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(42));
}

#[test]
fn test_unknown_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    assert!(dict.execute_word("UNKNOWN", &mut stack, &mut loop_stack).is_err());
}

#[test]
fn test_abs_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(-12);
    dict.execute_word("ABS", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(12));
}

#[test]
fn test_drop_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(10);
    stack.push(20);
    stack.push(30);

    dict.execute_word("DROP", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(10));
    assert_eq!(stack.pop(), None);
}

#[test]
fn test_rot_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(1);
    stack.push(2);
    stack.push(3);

    dict.execute_word("ROT", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(1)); // 1 rotated to top
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(2));
}

#[test]
fn test_over_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();

    stack.push(5);
    stack.push(10);

    dict.execute_word("OVER", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(5)); // copied to top
    assert_eq!(stack.pop(), Some(10));
    assert_eq!(stack.pop(), Some(5));
}

// Loop tests
#[test]
fn test_begin_until_loop() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    let mut loop_stack = LoopStack::new();

    // Simple BEGIN...UNTIL that counts down from 3 to 0
    // 3 BEGIN 1 - DUP 0 = UNTIL DROP
    let tokens = vec!["3", "BEGIN", "1", "-", "DUP", "0", "=", "UNTIL", "DROP"];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack).unwrap();

    // After loop completes and DROP, stack should be empty
    assert!(stack.is_empty());
}

#[test]
fn test_do_loop_basic() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    let mut loop_stack = LoopStack::new();

    // : TEST 5 0 DO I LOOP ;  (pushes 0 1 2 3 4 onto stack)
    let tokens = vec!["5", "0", "DO", "I", "LOOP"];
    let ast = parse_tokens(&tokens).unwrap();
    
    ast.execute(&mut stack, &dict, &mut loop_stack).unwrap();
    
    // Should have pushed indices 0 through 4
    assert_eq!(stack.pop(), Some(4));
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(1));
    assert_eq!(stack.pop(), Some(0));
}

#[test]
fn test_begin_while_repeat() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    let mut loop_stack = LoopStack::new();

    // : TEST 5 BEGIN DUP WHILE DUP 1 - REPEAT DROP ;
    let tokens = vec!["5", "BEGIN", "DUP", "WHILE", "DUP", "1", "-", "REPEAT", "DROP"];
    let ast = parse_tokens(&tokens).unwrap();
    
    ast.execute(&mut stack, &dict, &mut loop_stack).unwrap();
    
    // Stack should have: 5 4 3 2 1
    assert_eq!(stack.pop(), Some(1));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(4));
    assert_eq!(stack.pop(), Some(5));
}

#[test]
fn test_loop_i_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    // Manually create a loop context
    loop_stack.push_loop(5, 10);

    dict.execute_word("I", &mut stack, &mut loop_stack).unwrap();

    assert_eq!(stack.pop(), Some(5));

    loop_stack.pop_loop();
}

// String tests
#[test]
fn test_print_string() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    // Parse a string literal
    let tokens = vec![".\"", "Hello", "World\""];
    let ast = parse_tokens(&tokens).unwrap();

    // Note: This will print to stdout during test, but shouldn't error
    ast.execute(&mut stack, &dict, &mut loop_stack).unwrap();

    // Stack should be unchanged
    assert!(stack.is_empty());
}

#[test]
fn test_print_string_in_loop() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    // : TEST 3 0 DO ." Hi " LOOP ;
    let tokens = vec!["3", "0", "DO", ".\"", "Hi", "\"", "LOOP"];
    let ast = parse_tokens(&tokens).unwrap();

    // Should print "Hi " three times
    ast.execute(&mut stack, &dict, &mut loop_stack).unwrap();

    assert!(stack.is_empty());
}

// +LOOP tests
#[test]
fn test_plus_loop_increment_by_two() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    // : TEST 10 0 DO I 2 +LOOP ;  (pushes 0 2 4 6 8 onto stack)
    let tokens = vec!["10", "0", "DO", "I", "2", "+LOOP"];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack).unwrap();

    // Should have pushed even indices 0, 2, 4, 6, 8
    assert_eq!(stack.pop(), Some(8));
    assert_eq!(stack.pop(), Some(6));
    assert_eq!(stack.pop(), Some(4));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(0));
}

#[test]
fn test_plus_loop_variable_increment() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    // : TEST 15 0 DO I 3 +LOOP ;  (pushes 0 3 6 9 12 onto stack)
    let tokens = vec!["15", "0", "DO", "I", "3", "+LOOP"];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack).unwrap();

    // Should have pushed indices 0, 3, 6, 9, 12
    assert_eq!(stack.pop(), Some(12));
    assert_eq!(stack.pop(), Some(9));
    assert_eq!(stack.pop(), Some(6));
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(0));
}

// J word tests
#[test]
fn test_loop_j_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    // Manually create nested loop context
    loop_stack.push_loop(5, 10);  // Outer loop
    loop_stack.push_loop(2, 8);   // Inner loop

    dict.execute_word("J", &mut stack, &mut loop_stack).unwrap();

    // Should get outer loop index (5)
    assert_eq!(stack.pop(), Some(5));

    loop_stack.pop_loop();
    loop_stack.pop_loop();
}

#[test]
fn test_nested_loops_with_j() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    // Nested loop: 3 0 DO 2 0 DO J I + LOOP LOOP
    // Outer loop: 0, 1, 2
    // Inner loop for each outer: 0, 1
    // Should push: J+I for each combination = 0, 1, 1, 2, 2, 3
    let tokens = vec!["3", "0", "DO", "2", "0", "DO", "J", "I", "+", "LOOP", "LOOP"];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack).unwrap();

    // Expected results: (0+0), (0+1), (1+0), (1+1), (2+0), (2+1)
    assert_eq!(stack.pop(), Some(3)); // 2+1
    assert_eq!(stack.pop(), Some(2)); // 2+0
    assert_eq!(stack.pop(), Some(2)); // 1+1
    assert_eq!(stack.pop(), Some(1)); // 1+0
    assert_eq!(stack.pop(), Some(1)); // 0+1
    assert_eq!(stack.pop(), Some(0)); // 0+0
}

// LEAVE tests
#[test]
fn test_leave_early_exit() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    // : TEST 10 0 DO I DUP 5 = IF LEAVE THEN LOOP ;
    // I pushes index, DUP duplicates, 5 pushes 5, = compares (consuming DUP and 5)
    // So each iteration leaves one I value, and exits when i=5
    let tokens = vec!["10", "0", "DO", "I", "DUP", "5", "=", "IF", "LEAVE", "THEN", "LOOP"];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack).unwrap();

    // Should have 0 1 2 3 4 5 on stack (one copy each, DUP was consumed by =)
    assert_eq!(stack.pop(), Some(5));
    assert_eq!(stack.pop(), Some(4));
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(1));
    assert_eq!(stack.pop(), Some(0));
}

#[test]
fn test_leave_in_plus_loop() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    // : TEST 20 0 DO I DUP 7 > IF LEAVE THEN 2 +LOOP ;
    // I pushes index, DUP duplicates, 7 pushes 7, > compares (consuming DUP and 7)
    // Exits when i=8 (first time 8 > 7 is true)
    let tokens = vec!["20", "0", "DO", "I", "DUP", "7", ">", "IF", "LEAVE", "THEN", "2", "+LOOP"];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack).unwrap();

    // Should have 0 2 4 6 8 on stack (one copy each)
    assert_eq!(stack.pop(), Some(8));
    assert_eq!(stack.pop(), Some(6));
    assert_eq!(stack.pop(), Some(4));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(0));
}
