use quarter::{LoopStack, parse_tokens, Dictionary, ReturnStack, Stack, Memory};

#[test]
fn test_if_then_true() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Define: : TEST 5 3 > IF 99 THEN ;
    let tokens = vec!["5", "3", ">", "IF", "99", "THEN"];
    let ast = parse_tokens(&tokens, &dict).unwrap();
    dict.add_compiled("TEST".to_string(), ast);

    dict.execute_word("TEST", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(&mut memory), Some(99));
}

#[test]
fn test_if_then_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Define: : TEST 3 5 > IF 99 THEN ;
    let tokens = vec!["3", "5", ">", "IF", "99", "THEN"];
    let ast = parse_tokens(&tokens, &dict).unwrap();
    dict.add_compiled("TEST".to_string(), ast);

    dict.execute_word("TEST", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    // Stack should be empty since IF was false
    assert_eq!(stack.pop(&mut memory), None);
}

#[test]
fn test_if_else_then_true() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Define: : TEST 5 3 > IF 99 ELSE 88 THEN ;
    let tokens = vec!["5", "3", ">", "IF", "99", "ELSE", "88", "THEN"];
    let ast = parse_tokens(&tokens, &dict).unwrap();
    dict.add_compiled("TEST".to_string(), ast);

    dict.execute_word("TEST", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(&mut memory), Some(99));
}

#[test]
fn test_if_else_then_false() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Define: : TEST 3 5 > IF 99 ELSE 88 THEN ;
    let tokens = vec!["3", "5", ">", "IF", "99", "ELSE", "88", "THEN"];
    let ast = parse_tokens(&tokens, &dict).unwrap();
    dict.add_compiled("TEST".to_string(), ast);

    dict.execute_word("TEST", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(&mut memory), Some(88));
}

#[test]
fn test_if_with_multiple_operations() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Define: : TEST 10 5 > IF 2 3 + ELSE 4 5 * THEN ;
    let tokens = vec![
        "10", "5", ">", "IF", "2", "3", "+", "ELSE", "4", "5", "*", "THEN",
    ];
    let ast = parse_tokens(&tokens, &dict).unwrap();
    dict.add_compiled("TEST".to_string(), ast);

    dict.execute_word("TEST", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(&mut memory), Some(5)); // 2 + 3
}

#[test]
fn test_nested_if() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Define: : TEST 1 IF 1 IF 42 THEN THEN ;
    let tokens = vec!["1", "IF", "1", "IF", "42", "THEN", "THEN"];
    let ast = parse_tokens(&tokens, &dict).unwrap();
    dict.add_compiled("TEST".to_string(), ast);

    dict.execute_word("TEST", &mut stack, &mut loop_stack, &mut return_stack, &mut memory).unwrap();
    assert_eq!(stack.pop(&mut memory), Some(42));
}

#[test]
fn test_if_with_stack_operations() {
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let mut dict = Dictionary::new();
    let mut return_stack = ReturnStack::new();
    let mut memory = Memory::new();

    // Define: : DOUBLE_IF_POSITIVE DUP 0 > IF DUP + THEN ;
    let tokens = vec!["DUP", "0", ">", "IF", "DUP", "+", "THEN"];
    let ast = parse_tokens(&tokens, &dict).unwrap();
    dict.add_compiled("DOUBLE_IF_POSITIVE".to_string(), ast);

    // Test with positive number
    stack.push(21, &mut memory);
    dict.execute_word("DOUBLE_IF_POSITIVE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(42));

    // Test with negative number (should not double)
    stack.push(-5, &mut memory);
    dict.execute_word("DOUBLE_IF_POSITIVE", &mut stack, &mut loop_stack, &mut return_stack, &mut memory)
        .unwrap();
    assert_eq!(stack.pop(&mut memory), Some(-5));
}

#[test]
fn test_if_compile_only_error() {
    // Try to parse IF outside of definition should work in parse_tokens
    // but executing it directly should be prevented by the REPL
    let dict = Dictionary::new();
    let tokens = vec!["1", "IF", "2", "THEN"];
    let ast = parse_tokens(&tokens, &dict);
    assert!(ast.is_ok()); // Parsing should succeed
}
