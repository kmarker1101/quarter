use quarter::{Dictionary, LoopStack, Memory, ReturnStack, Stack, parse_tokens};

#[test]
fn test_dot_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(42, &mut memory);
    assert!(
        dict.execute_word(
            ".",
            &mut stack,
            &mut loop_stack,
            &mut return_stack,
            &mut memory
        )
        .is_ok()
    );
    assert!(stack.is_empty());
}

#[test]
fn test_dot_s_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    stack.push(10, &mut memory);
    stack.push(20, &mut memory);
    stack.push(30, &mut memory);

    // .S should not modify the stack
    assert!(
        dict.execute_word(
            ".S",
            &mut stack,
            &mut loop_stack,
            &mut return_stack,
            &mut memory
        )
        .is_ok()
    );
    assert_eq!(stack.pop(&mut memory), Some(30));
    assert_eq!(stack.pop(&mut memory), Some(20));
    assert_eq!(stack.pop(&mut memory), Some(10));
}

#[test]
fn test_unknown_word() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    assert!(
        dict.execute_word(
            "UNKNOWN",
            &mut stack,
            &mut loop_stack,
            &mut return_stack,
            &mut memory
        )
        .is_err()
    );
}

// Loop tests
#[test]
fn test_do_loop_basic() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();

    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // : TEST 5 0 DO I LOOP ;  (pushes 0 1 2 3 4 onto stack)
    let tokens = vec!["5", "0", "DO", "I", "LOOP"];
    let ast = parse_tokens(&tokens, &dict).unwrap();

    ast.execute(
        &mut stack,
        &dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Should have pushed indices 0 through 4
    assert_eq!(stack.pop(&mut memory), Some(4));
    assert_eq!(stack.pop(&mut memory), Some(3));
    assert_eq!(stack.pop(&mut memory), Some(2));
    assert_eq!(stack.pop(&mut memory), Some(1));
    assert_eq!(stack.pop(&mut memory), Some(0));
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
    let ast = parse_tokens(&tokens, &dict).unwrap();

    ast.execute(
        &mut stack,
        &dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Stack should have: 5 4 3 2 1
    assert_eq!(stack.pop(&mut memory), Some(1));
    assert_eq!(stack.pop(&mut memory), Some(2));
    assert_eq!(stack.pop(&mut memory), Some(3));
    assert_eq!(stack.pop(&mut memory), Some(4));
    assert_eq!(stack.pop(&mut memory), Some(5));
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

    dict.execute_word(
        "I",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    assert_eq!(stack.pop(&mut memory), Some(5));

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
    let ast = parse_tokens(&tokens, &dict).unwrap();

    // Note: This will print to stdout during test, but shouldn't error
    ast.execute(
        &mut stack,
        &dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

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
    let ast = parse_tokens(&tokens, &dict).unwrap();

    // Should print "Hi " three times
    ast.execute(
        &mut stack,
        &dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

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
    let ast = parse_tokens(&tokens, &dict).unwrap();

    ast.execute(
        &mut stack,
        &dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Should have pushed even indices 0, 2, 4, 6, 8
    assert_eq!(stack.pop(&mut memory), Some(8));
    assert_eq!(stack.pop(&mut memory), Some(6));
    assert_eq!(stack.pop(&mut memory), Some(4));
    assert_eq!(stack.pop(&mut memory), Some(2));
    assert_eq!(stack.pop(&mut memory), Some(0));
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
    let ast = parse_tokens(&tokens, &dict).unwrap();

    ast.execute(
        &mut stack,
        &dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Should have pushed indices 0, 3, 6, 9, 12
    assert_eq!(stack.pop(&mut memory), Some(12));
    assert_eq!(stack.pop(&mut memory), Some(9));
    assert_eq!(stack.pop(&mut memory), Some(6));
    assert_eq!(stack.pop(&mut memory), Some(3));
    assert_eq!(stack.pop(&mut memory), Some(0));
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

    dict.execute_word(
        "J",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Should get outer loop index (5)
    assert_eq!(stack.pop(&mut memory), Some(5));

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
    let ast = parse_tokens(&tokens, &dict).unwrap();

    ast.execute(
        &mut stack,
        &dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Expected results: (0+0), (0+1), (1+0), (1+1), (2+0), (2+1)
    assert_eq!(stack.pop(&mut memory), Some(3)); // 2+1
    assert_eq!(stack.pop(&mut memory), Some(2)); // 2+0
    assert_eq!(stack.pop(&mut memory), Some(2)); // 1+1
    assert_eq!(stack.pop(&mut memory), Some(1)); // 1+0
    assert_eq!(stack.pop(&mut memory), Some(1)); // 0+1
    assert_eq!(stack.pop(&mut memory), Some(0)); // 0+0
}

// LEAVE tests
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
    let ast = parse_tokens(&tokens, &dict).unwrap();

    ast.execute(
        &mut stack,
        &dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Should have 0 2 4 6 8 on stack (one copy each)
    assert_eq!(stack.pop(&mut memory), Some(8));
    assert_eq!(stack.pop(&mut memory), Some(6));
    assert_eq!(stack.pop(&mut memory), Some(4));
    assert_eq!(stack.pop(&mut memory), Some(2));
    assert_eq!(stack.pop(&mut memory), Some(0));
}

#[test]
fn test_emit_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Test emitting 'A' (65)
    stack.push(65, &mut memory);
    assert!(
        dict.execute_word(
            "EMIT",
            &mut stack,
            &mut loop_stack,
            &mut return_stack,
            &mut memory
        )
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
    stack.push(128515, &mut memory);
    assert!(
        dict.execute_word(
            "EMIT",
            &mut stack,
            &mut loop_stack,
            &mut return_stack,
            &mut memory
        )
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
    stack.push(72, &mut memory); // H
    stack.push(105, &mut memory); // i
    stack.push(33, &mut memory); // !

    assert!(
        dict.execute_word(
            "EMIT",
            &mut stack,
            &mut loop_stack,
            &mut return_stack,
            &mut memory
        )
        .is_ok()
    );
    assert!(
        dict.execute_word(
            "EMIT",
            &mut stack,
            &mut loop_stack,
            &mut return_stack,
            &mut memory
        )
        .is_ok()
    );
    assert!(
        dict.execute_word(
            "EMIT",
            &mut stack,
            &mut loop_stack,
            &mut return_stack,
            &mut memory
        )
        .is_ok()
    );

    assert!(stack.is_empty());
}

// Bitwise operation tests
#[test]
fn test_and_boolean_logic() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Forth uses -1 for true, 0 for false
    // -1 AND -1 = -1 (true AND true = true)
    stack.push(-1, &mut memory);
    stack.push(-1, &mut memory);
    dict.execute_word(
        "AND",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // -1 AND 0 = 0 (true AND false = false)
    stack.push(-1, &mut memory);
    stack.push(0, &mut memory);
    dict.execute_word(
        "AND",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));
}

#[test]
fn test_or_boolean_logic() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // -1 OR 0 = -1 (true OR false = true)
    stack.push(-1, &mut memory);
    stack.push(0, &mut memory);
    dict.execute_word(
        "OR",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-1));

    // 0 OR 0 = 0 (false OR false = false)
    stack.push(0, &mut memory);
    stack.push(0, &mut memory);
    dict.execute_word(
        "OR",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));
}

#[test]
fn test_and_with_negative() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // -1 (all bits set) AND 15 (1111) = 15
    stack.push(-1, &mut memory);
    stack.push(15, &mut memory);
    dict.execute_word(
        "AND",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(15));
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
    stack.push(-8, &mut memory);
    stack.push(1, &mut memory);
    dict.execute_word(
        "RSHIFT",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // This will be implementation-dependent
    // Rust's >> on signed integers is arithmetic shift
    assert_eq!(stack.pop(&mut memory), Some(-4));
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
    stack.push(42, &mut memory);
    dict.execute_word(
        ">R",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Data stack should be empty
    assert!(stack.is_empty());
    // Return stack should have 42
    assert_eq!(return_stack.peek(&memory), Some(42));
}

#[test]
fn test_r_from_word() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Push directly to return stack
    return_stack.push(99, &mut memory);

    // Move from return stack to data stack
    dict.execute_word(
        "R>",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Data stack should have 99
    assert_eq!(stack.pop(&mut memory), Some(99));
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
    return_stack.push(77, &mut memory);

    // Copy from return stack to data stack (non-destructive)
    dict.execute_word(
        "R@",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Data stack should have 77
    assert_eq!(stack.pop(&mut memory), Some(77));
    // Return stack should still have 77
    assert_eq!(return_stack.peek(&memory), Some(77));
}

#[test]
fn test_return_stack_sequence() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Test: 10 20 30 >R >R + R> R> ( -- 30 30 20 )
    stack.push(10, &mut memory);
    stack.push(20, &mut memory);
    stack.push(30, &mut memory);

    // Move 30 to return stack
    dict.execute_word(
        ">R",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    // Move 20 to return stack
    dict.execute_word(
        ">R",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Add 10 (nothing on stack, should underflow - but for this test we'll skip)
    // Actually, let's push more values
    stack.push(5, &mut memory);
    dict.execute_word(
        "+",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap(); // 10 + 5 = 15

    // Pop from return stack (20)
    dict.execute_word(
        "R>",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    // Pop from return stack (30)
    dict.execute_word(
        "R>",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();

    // Stack should have: 15 20 30 (top)
    assert_eq!(stack.pop(&mut memory), Some(30));
    assert_eq!(stack.pop(&mut memory), Some(20));
    assert_eq!(stack.pop(&mut memory), Some(15));
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
    let ast = parse_tokens(&tokens, &dict).unwrap();
    dict.add_compiled("TEST-EXIT".to_string(), ast);

    dict.execute_word(
        "TEST-EXIT",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(42));
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
    let ast = parse_tokens(&tokens, &dict).unwrap();
    dict.add_compiled("TEST-EXIT-IF".to_string(), ast);

    dict.execute_word(
        "TEST-EXIT-IF",
        &mut stack,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
    )
    .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(42));
    assert!(stack.is_empty()); // 99 should not be pushed
}

#[test]
fn test_pick() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // 1 2 3 0 PICK → 1 2 3 3 (copy top element)
    stack.push(1, &mut memory);
    stack.push(2, &mut memory);
    stack.push(3, &mut memory);
    stack.push(0, &mut memory);
    dict.execute_word("PICK", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(3));
    assert_eq!(stack.pop(&mut memory), Some(3));
    assert_eq!(stack.pop(&mut memory), Some(2));
    assert_eq!(stack.pop(&mut memory), Some(1));

    // 1 2 3 1 PICK → 1 2 3 2 (copy second element)
    stack.push(1, &mut memory);
    stack.push(2, &mut memory);
    stack.push(3, &mut memory);
    stack.push(1, &mut memory);
    dict.execute_word("PICK", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(2));
    assert_eq!(stack.pop(&mut memory), Some(3));
    assert_eq!(stack.pop(&mut memory), Some(2));
    assert_eq!(stack.pop(&mut memory), Some(1));

    // 1 2 3 2 PICK → 1 2 3 1 (copy third element)
    stack.push(1, &mut memory);
    stack.push(2, &mut memory);
    stack.push(3, &mut memory);
    stack.push(2, &mut memory);
    dict.execute_word("PICK", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(1));
    assert_eq!(stack.pop(&mut memory), Some(3));
    assert_eq!(stack.pop(&mut memory), Some(2));
    assert_eq!(stack.pop(&mut memory), Some(1));
}

#[test]
fn test_depth() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Empty stack: DEPTH → 0
    dict.execute_word("DEPTH", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(0));

    // 1 DEPTH → 1 1
    stack.push(1, &mut memory);
    dict.execute_word("DEPTH", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(1)); // depth
    assert_eq!(stack.pop(&mut memory), Some(1)); // original value

    // 1 2 3 DEPTH → 1 2 3 3
    stack.push(1, &mut memory);
    stack.push(2, &mut memory);
    stack.push(3, &mut memory);
    dict.execute_word("DEPTH", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(3)); // depth
    assert_eq!(stack.pop(&mut memory), Some(3)); // values
    assert_eq!(stack.pop(&mut memory), Some(2));
    assert_eq!(stack.pop(&mut memory), Some(1));
}

#[test]
fn test_cr() {
    let mut stack = Stack::new();
    let dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // CR should not modify the stack and not error
    assert!(
        dict.execute_word("CR", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .is_ok()
    );
    assert!(stack.is_empty());

    // CR should work even with values on the stack
    stack.push(42, &mut memory);
    assert!(
        dict.execute_word("CR", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
            .is_ok()
    );
    assert_eq!(stack.pop(&mut memory), Some(42));
}
