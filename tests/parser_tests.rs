use quarter::{LoopStack, parse_tokens, AstNode, Dictionary, Stack};

#[test]
fn test_parse_simple_number() {
    let tokens = vec!["42"];
    let ast = parse_tokens(&tokens).unwrap();

    match ast {
        AstNode::PushNumber(n) => assert_eq!(n, 42),
        _ => panic!("Expected PushNumber"),
    }
}

#[test]
fn test_parse_simple_word() {
    let tokens = vec!["DUP"];
    let ast = parse_tokens(&tokens).unwrap();

    match ast {
        AstNode::CallWord(s) => assert_eq!(s, "DUP"),
        _ => panic!("Expected CallWord"),
    }
}

#[test]
fn test_parse_sequence() {
    let tokens = vec!["5", "DUP", "*"];
    let ast = parse_tokens(&tokens).unwrap();

    match ast {
        AstNode::Sequence(nodes) => {
            assert_eq!(nodes.len(), 3);
            match &nodes[0] {
                AstNode::PushNumber(n) => assert_eq!(*n, 5),
                _ => panic!("Expected PushNumber"),
            }
            match &nodes[1] {
                AstNode::CallWord(s) => assert_eq!(s, "DUP"),
                _ => panic!("Expected CallWord"),
            }
            match &nodes[2] {
                AstNode::CallWord(s) => assert_eq!(s, "*"),
                _ => panic!("Expected CallWord"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

#[test]
fn test_parse_if_then() {
    let tokens = vec!["1", "IF", "42", "THEN"];
    let ast = parse_tokens(&tokens).unwrap();

    match ast {
        AstNode::Sequence(nodes) => {
            assert_eq!(nodes.len(), 2);
            match &nodes[0] {
                AstNode::PushNumber(n) => assert_eq!(*n, 1),
                _ => panic!("Expected PushNumber"),
            }
            match &nodes[1] {
                AstNode::IfThenElse { then_branch, else_branch } => {
                    assert_eq!(then_branch.len(), 1);
                    assert!(else_branch.is_none());
                }
                _ => panic!("Expected IfThenElse"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

#[test]
fn test_parse_if_else_then() {
    let tokens = vec!["1", "IF", "42", "ELSE", "99", "THEN"];
    let ast = parse_tokens(&tokens).unwrap();

    match ast {
        AstNode::Sequence(nodes) => {
            assert_eq!(nodes.len(), 2);
            match &nodes[1] {
                AstNode::IfThenElse { then_branch, else_branch } => {
                    assert_eq!(then_branch.len(), 1);
                    assert!(else_branch.is_some());
                    assert_eq!(else_branch.as_ref().unwrap().len(), 1);
                }
                _ => panic!("Expected IfThenElse"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

#[test]
fn test_parse_nested_if() {
    let tokens = vec!["1", "IF", "1", "IF", "42", "THEN", "THEN"];
    let result = parse_tokens(&tokens);
    assert!(result.is_ok());
}

#[test]
fn test_parse_missing_then() {
    let tokens = vec!["1", "IF", "42"];
    let result = parse_tokens(&tokens);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Missing THEN");
}

#[test]
fn test_parse_unexpected_then() {
    let tokens = vec!["42", "THEN"];
    let result = parse_tokens(&tokens);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unexpected THEN or ELSE");
}

#[test]
fn test_parse_unexpected_else() {
    let tokens = vec!["42", "ELSE"];
    let result = parse_tokens(&tokens);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unexpected THEN or ELSE");
}

#[test]
fn test_parse_complex_if_expression() {
    let tokens = vec!["10", "5", ">", "IF", "2", "3", "+", "ELSE", "4", "5", "*", "THEN"];
    let result = parse_tokens(&tokens);
    assert!(result.is_ok());

    // Verify execution works correctly
    let mut stack = Stack::new();
    let mut loop_stack = LoopStack::new();
    let dict = Dictionary::new();
    let ast = result.unwrap();
    ast.execute(&mut stack, &dict, &mut loop_stack).unwrap();
    assert_eq!(stack.pop(), Some(5)); // 10 > 5 is true, so 2 + 3
}

#[test]
fn test_parse_negative_numbers() {
    let tokens = vec!["-42"];
    let ast = parse_tokens(&tokens).unwrap();

    match ast {
        AstNode::PushNumber(n) => assert_eq!(n, -42),
        _ => panic!("Expected PushNumber"),
    }
}

#[test]
fn test_parse_empty_tokens() {
    let tokens: Vec<&str> = vec![];
    let result = parse_tokens(&tokens);
    assert!(result.is_ok());

    // Empty tokens should result in an empty sequence
    match result.unwrap() {
        AstNode::Sequence(nodes) => assert_eq!(nodes.len(), 0),
        _ => {}
    }
}
