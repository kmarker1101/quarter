use quarter::{Dictionary, LoopStack, Memory, ReturnStack, Stack, parse_tokens};

#[test]
fn test_dot_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(42);
    assert!(dict.execute_word(".", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).is_ok());
    assert!(stack.is_empty());
}

#[test]
fn test_add_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5);
    stack.push(3);
    dict.execute_word("+", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(), Some(8));
}

#[test]
fn test_subtract_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("-", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(), Some(-1));
}

#[test]
fn test_multiply_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("*", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(), Some(42));
}

#[test]
fn test_divide_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(12);
    stack.push(5);
    dict.execute_word("/", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(), Some(2));
}

#[test]
fn test_mod_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(13);
    stack.push(5);
    dict.execute_word("MOD", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(3));
}

#[test]
fn test_slash_mod_basic() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(10);
    stack.push(3);
    dict.execute_word("/MOD", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Should leave remainder then quotient: 1 3
    assert_eq!(stack.pop(), Some(3)); // quotient on top
    assert_eq!(stack.pop(), Some(1)); // remainder below
}

#[test]
fn test_dup_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("DUP", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(7));
    assert_eq!(stack.pop(), Some(7));
    assert_eq!(stack.pop(), Some(6));
}

#[test]
fn test_swap_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(6);
    stack.push(7);
    dict.execute_word("SWAP", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(6));
    assert_eq!(stack.pop(), Some(7));
}

#[test]
fn test_dot_s_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(10);
    stack.push(20);
    stack.push(30);

    // .S should not modify the stack
    assert!(dict.execute_word(".S", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).is_ok());
    assert_eq!(stack.pop(), Some(30));
    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(10));
}

#[test]
fn test_negate_positive() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(42);
    dict.execute_word("NEGATE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(-42));
}

#[test]
fn test_negate_negative() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(-42);
    dict.execute_word("NEGATE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(42));
}

#[test]
fn test_unknown_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    assert!(
        dict.execute_word("UNKNOWN", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .is_err()
    );
}

#[test]
fn test_abs_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(-12);
    dict.execute_word("ABS", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(12));
}

#[test]
fn test_drop_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(10);
    stack.push(20);
    stack.push(30);

    dict.execute_word("DROP", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(10));
    assert_eq!(stack.pop(), None);
}

#[test]
fn test_rot_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(1);
    stack.push(2);
    stack.push(3);

    dict.execute_word("ROT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(1)); // 1 rotated to top
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(2));
}

#[test]
fn test_over_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5);
    stack.push(10);

    dict.execute_word("OVER", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

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
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Simple BEGIN...UNTIL that counts down from 3 to 0
    // 3 BEGIN 1 - DUP 0 = UNTIL DROP
    let tokens = vec!["3", "BEGIN", "1", "-", "DUP", "0", "=", "UNTIL", "DROP"];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    // After loop completes and DROP, stack should be empty
    assert!(stack.is_empty());
}

#[test]
fn test_do_loop_basic() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : TEST 5 0 DO I LOOP ;  (pushes 0 1 2 3 4 onto stack)
    let tokens = vec!["5", "0", "DO", "I", "LOOP"];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

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
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : TEST 5 BEGIN DUP WHILE DUP 1 - REPEAT DROP ;
    let tokens = vec![
        "5", "BEGIN", "DUP", "WHILE", "DUP", "1", "-", "REPEAT", "DROP",
    ];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

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
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Manually create a loop context
    loop_stack.push_loop(5, 10);

    dict.execute_word("I", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert_eq!(stack.pop(), Some(5));

    loop_stack.pop_loop();
}

// String tests
#[test]
fn test_print_string() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Parse a string literal
    let tokens = vec![".\"", "Hello", "World\""];
    let ast = parse_tokens(&tokens).unwrap();

    // Note: This will print to stdout during test, but shouldn't error
    ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    // Stack should be unchanged
    assert!(stack.is_empty());
}

#[test]
fn test_print_string_in_loop() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : TEST 3 0 DO ." Hi " LOOP ;
    let tokens = vec!["3", "0", "DO", ".\"", "Hi", "\"", "LOOP"];
    let ast = parse_tokens(&tokens).unwrap();

    // Should print "Hi " three times
    ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    assert!(stack.is_empty());
}

// +LOOP tests
#[test]
fn test_plus_loop_increment_by_two() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : TEST 10 0 DO I 2 +LOOP ;  (pushes 0 2 4 6 8 onto stack)
    let tokens = vec!["10", "0", "DO", "I", "2", "+LOOP"];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

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
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : TEST 15 0 DO I 3 +LOOP ;  (pushes 0 3 6 9 12 onto stack)
    let tokens = vec!["15", "0", "DO", "I", "3", "+LOOP"];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

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
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Manually create nested loop context
    loop_stack.push_loop(5, 10); // Outer loop
    loop_stack.push_loop(2, 8); // Inner loop

    dict.execute_word("J", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

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
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Nested loop: 3 0 DO 2 0 DO J I + LOOP LOOP
    // Outer loop: 0, 1, 2
    // Inner loop for each outer: 0, 1
    // Should push: J+I for each combination = 0, 1, 1, 2, 2, 3
    let tokens = vec![
        "3", "0", "DO", "2", "0", "DO", "J", "I", "+", "LOOP", "LOOP",
    ];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

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
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : TEST 10 0 DO I DUP 5 = IF LEAVE THEN LOOP ;
    // I pushes index, DUP duplicates, 5 pushes 5, = compares (consuming DUP and 5)
    // So each iteration leaves one I value, and exits when i=5
    let tokens = vec![
        "10", "0", "DO", "I", "DUP", "5", "=", "IF", "LEAVE", "THEN", "LOOP",
    ];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

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
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : TEST 20 0 DO I DUP 7 > IF LEAVE THEN 2 +LOOP ;
    // I pushes index, DUP duplicates, 7 pushes 7, > compares (consuming DUP and 7)
    // Exits when i=8 (first time 8 > 7 is true)
    let tokens = vec![
        "20", "0", "DO", "I", "DUP", "7", ">", "IF", "LEAVE", "THEN", "2", "+LOOP",
    ];
    let ast = parse_tokens(&tokens).unwrap();

    ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory).unwrap();

    // Should have 0 2 4 6 8 on stack (one copy each)
    assert_eq!(stack.pop(), Some(8));
    assert_eq!(stack.pop(), Some(6));
    assert_eq!(stack.pop(), Some(4));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(0));
}

#[test]
fn test_emit_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Test emitting 'A' (65)
    stack.push(65);
    assert!(
        dict.execute_word("EMIT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .is_ok()
    );

    // Stack should be empty after EMIT
    assert!(stack.is_empty());
}

#[test]
fn test_emit_unicode() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Test emitting Unicode smiley (128515 = 😃)
    stack.push(128515);
    assert!(
        dict.execute_word("EMIT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .is_ok()
    );

    assert!(stack.is_empty());
}

#[test]
fn test_emit_multiple_chars() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Emit "Hi!" (72, 105, 33)
    stack.push(72); // H
    stack.push(105); // i
    stack.push(33); // !

    assert!(
        dict.execute_word("EMIT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .is_ok()
    );
    assert!(
        dict.execute_word("EMIT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .is_ok()
    );
    assert!(
        dict.execute_word("EMIT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .is_ok()
    );

    assert!(stack.is_empty());
}

#[test]
fn test_space_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // SPACE should just print a space, not affect stack
    assert!(
        dict.execute_word("SPACE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .is_ok()
    );
    assert!(stack.is_empty());
}

// Bitwise operation tests
#[test]
fn test_and_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 12 (1100) AND 10 (1010) = 8 (1000)
    stack.push(12);
    stack.push(10);
    dict.execute_word("AND", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(8));
}

#[test]
fn test_or_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 12 (1100) OR 10 (1010) = 14 (1110)
    stack.push(12);
    stack.push(10);
    dict.execute_word("OR", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(14));
}

#[test]
fn test_xor_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 12 (1100) XOR 10 (1010) = 6 (0110)
    stack.push(12);
    stack.push(10);
    dict.execute_word("XOR", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(6));
}

#[test]
fn test_invert_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // INVERT flips all bits
    stack.push(0);
    dict.execute_word("INVERT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(-1)); // All bits set = -1

    // INVERT -1 should give 0
    stack.push(-1);
    dict.execute_word("INVERT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(0));
}

#[test]
fn test_lshift_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 1 << 3 = 8
    stack.push(1);
    stack.push(3);
    dict.execute_word("LSHIFT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(8));
}

#[test]
fn test_rshift_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 8 >> 2 = 2
    stack.push(8);
    stack.push(2);
    dict.execute_word("RSHIFT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(2));
}

#[test]
fn test_and_boolean_logic() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Forth uses -1 for true, 0 for false
    // -1 AND -1 = -1 (true AND true = true)
    stack.push(-1);
    stack.push(-1);
    dict.execute_word("AND", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(-1));

    // -1 AND 0 = 0 (true AND false = false)
    stack.push(-1);
    stack.push(0);
    dict.execute_word("AND", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(0));
}

#[test]
fn test_or_boolean_logic() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // -1 OR 0 = -1 (true OR false = true)
    stack.push(-1);
    stack.push(0);
    dict.execute_word("OR", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(-1));

    // 0 OR 0 = 0 (false OR false = false)
    stack.push(0);
    stack.push(0);
    dict.execute_word("OR", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(0));
}

#[test]
fn test_and_with_negative() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // -1 (all bits set) AND 15 (1111) = 15
    stack.push(-1);
    stack.push(15);
    dict.execute_word("AND", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(), Some(15));
}

#[test]
fn test_rshift_with_negative() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Shifting negative numbers (arithmetic vs logical shift)
    // -8 >> 1 in Rust does arithmetic shift (sign extension)
    stack.push(-8);
    stack.push(1);
    dict.execute_word("RSHIFT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // This will be implementation-dependent
    // Rust's >> on signed integers is arithmetic shift
    assert_eq!(stack.pop(), Some(-4));
}

// Return stack tests
#[test]
fn test_to_r_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Push 42 to data stack, then move to return stack
    stack.push(42);
    dict.execute_word(">R", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Data stack should be empty
    assert!(stack.is_empty());
    // Return stack should have 42
    assert_eq!(return_stack.peek(), Some(42));
}

#[test]
fn test_r_from_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Push directly to return stack
    return_stack.push(99);

    // Move from return stack to data stack
    dict.execute_word("R>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Data stack should have 99
    assert_eq!(stack.pop(), Some(99));
    // Return stack should be empty
    assert!(return_stack.is_empty());
}

#[test]
fn test_r_fetch_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Push directly to return stack
    return_stack.push(77);

    // Copy from return stack to data stack (non-destructive)
    dict.execute_word("R@", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Data stack should have 77
    assert_eq!(stack.pop(), Some(77));
    // Return stack should still have 77
    assert_eq!(return_stack.peek(), Some(77));
}

#[test]
fn test_return_stack_sequence() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Test: 10 20 30 >R >R + R> R> ( -- 30 30 20 )
    stack.push(10);
    stack.push(20);
    stack.push(30);

    // Move 30 to return stack
    dict.execute_word(">R", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    // Move 20 to return stack
    dict.execute_word(">R", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Add 10 (nothing on stack, should underflow - but for this test we'll skip)
    // Actually, let's push more values
    stack.push(5);
    dict.execute_word("+", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap(); // 10 + 5 = 15

    // Pop from return stack (20)
    dict.execute_word("R>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    // Pop from return stack (30)
    dict.execute_word("R>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    // Stack should have: 15 20 30 (top)
    assert_eq!(stack.pop(), Some(30));
    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(15));
}

// Zero comparison tests
#[test]
fn test_zero_equals_true() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(0);
    dict.execute_word("0=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(-1)); // true
}

#[test]
fn test_zero_equals_false() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5);
    dict.execute_word("0=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}

#[test]
fn test_zero_equals_negative() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(-5);
    dict.execute_word("0=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}

#[test]
fn test_zero_less_true() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(-5);
    dict.execute_word("0<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(-1)); // true
}

#[test]
fn test_zero_less_false_zero() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(0);
    dict.execute_word("0<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}

#[test]
fn test_zero_less_false_positive() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5);
    dict.execute_word("0<", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}

#[test]
fn test_zero_greater_true() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(5);
    dict.execute_word("0>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(-1)); // true
}

#[test]
fn test_zero_greater_false_zero() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(0);
    dict.execute_word("0>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}

#[test]
fn test_zero_greater_false_negative() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(-5);
    dict.execute_word("0>", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();

    assert_eq!(stack.pop(), Some(0)); // false
}

// TRUE and FALSE tests
#[test]
fn test_true_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    dict.execute_word("TRUE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(), Some(-1));
}

#[test]
fn test_false_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    dict.execute_word("FALSE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(), Some(0));
}

#[test]
fn test_true_false_comparison() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    dict.execute_word("TRUE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    dict.execute_word("FALSE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    dict.execute_word("=", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(), Some(0)); // TRUE != FALSE
}

// EXIT tests
#[test]
fn test_exit_simple() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : test-exit 42 exit 99 ;
    let tokens = vec!["42", "EXIT", "99"];
    let ast = parse_tokens(&tokens).unwrap();
    dict.add_compiled("TEST-EXIT".to_string(), ast);

    dict.execute_word("TEST-EXIT", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(), Some(42));
    assert!(stack.is_empty()); // 99 should not be pushed
}

#[test]
fn test_exit_in_if() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : test-exit-if 1 if 42 exit then 99 ;
    let tokens = vec!["1", "IF", "42", "EXIT", "THEN", "99"];
    let ast = parse_tokens(&tokens).unwrap();
    dict.add_compiled("TEST-EXIT-IF".to_string(), ast);

    dict.execute_word("TEST-EXIT-IF", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(), Some(42));
    assert!(stack.is_empty()); // 99 should not be pushed
}

#[test]
fn test_exit_in_loop() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : test-exit-loop 10 0 do i 5 = if i exit then loop 99 ;
    let tokens = vec!["10", "0", "DO", "I", "5", "=", "IF", "I", "EXIT", "THEN", "LOOP", "99"];
    let ast = parse_tokens(&tokens).unwrap();
    dict.add_compiled("TEST-EXIT-LOOP".to_string(), ast);

    dict.execute_word("TEST-EXIT-LOOP", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(), Some(5)); // Should exit at i=5
    assert!(stack.is_empty()); // 99 should not be pushed
}

#[test]
fn test_exit_with_true_false() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : test 1 0 = if false exit then true ;
    let tokens = vec!["1", "0", "=", "IF", "FALSE", "EXIT", "THEN", "TRUE"];
    let ast = parse_tokens(&tokens).unwrap();
    dict.add_compiled("TEST".to_string(), ast);

    dict.execute_word("TEST", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(), Some(-1)); // Should get TRUE (no EXIT taken)
}

#[test]
fn test_leap_year() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : leap-year? dup 400 mod 0= if drop true exit then 
    //              dup 100 mod 0= if drop false exit then 
    //              4 mod 0= ;
    let tokens = vec![
        "DUP", "400", "MOD", "0=", "IF", "DROP", "TRUE", "EXIT", "THEN",
        "DUP", "100", "MOD", "0=", "IF", "DROP", "FALSE", "EXIT", "THEN",
        "4", "MOD", "0="
    ];
    let ast = parse_tokens(&tokens).unwrap();
    dict.add_compiled("LEAP-YEAR?".to_string(), ast);

    // Test 2000 (divisible by 400)
    stack.push(2000);
    dict.execute_word("LEAP-YEAR?", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(), Some(-1));

    // Test 1900 (divisible by 100 but not 400)
    stack.push(1900);
    dict.execute_word("LEAP-YEAR?", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(), Some(0));

    // Test 2004 (divisible by 4)
    stack.push(2004);
    dict.execute_word("LEAP-YEAR?", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(), Some(-1));

    // Test 2001 (not divisible by 4)
    stack.push(2001);
    dict.execute_word("LEAP-YEAR?", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(), Some(0));
}
