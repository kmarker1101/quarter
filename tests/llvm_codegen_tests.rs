use quarter::llvm_codegen::Compiler;
use quarter::AstNode;
use inkwell::context::Context;

#[test]
fn test_compiler_initialization() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    assert!(compiler.is_ok());
}

#[test]
fn test_compile_simple_number() {
    // Test compiling: : TEST 42 ;
    let context = Context::create();
    let mut compiler = Compiler::new(&context).unwrap();

    // Create AST for pushing 42
    let ast = AstNode::PushNumber(42);

    // Compile it
    let result = compiler.compile_word("TEST", &ast);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

    // Verify IR was generated
    let ir = compiler.get_ir();
    assert!(ir.contains("define void @TEST"), "IR should contain function definition");
    println!("Generated IR:\n{}", ir);
}

#[test]
fn test_compile_double() {
    // Test compiling: : DOUBLE 2 * ;
    // This word multiplies the top stack value by 2
    let context = Context::create();
    let mut compiler = Compiler::new(&context).unwrap();

    // Create AST: push 2, then multiply
    let ast = AstNode::Sequence(vec![
        AstNode::PushNumber(2),
        AstNode::CallWord("*".to_string()),
    ]);

    // Compile it
    let result = compiler.compile_word("DOUBLE", &ast);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

    // Verify IR was generated
    let ir = compiler.get_ir();
    assert!(ir.contains("define void @DOUBLE"), "IR should contain DOUBLE function");
    assert!(ir.contains("mul"), "IR should contain multiplication");
    assert!(ir.contains("store"), "IR should contain store operations");
    assert!(ir.contains("load"), "IR should contain load operations");

    println!("\n=== Generated IR for DOUBLE ===");
    println!("{}", ir);
    println!("================================\n");
}

#[test]
fn test_compile_square() {
    // Test compiling: : SQUARE DUP * ;
    let context = Context::create();
    let mut compiler = Compiler::new(&context).unwrap();

    // Create AST: DUP then multiply
    let ast = AstNode::Sequence(vec![
        AstNode::CallWord("DUP".to_string()),
        AstNode::CallWord("*".to_string()),
    ]);

    // Compile it
    let result = compiler.compile_word("SQUARE", &ast);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

    // Verify IR was generated
    let ir = compiler.get_ir();
    assert!(ir.contains("define void @SQUARE"), "IR should contain SQUARE function");
    assert!(ir.contains("quarter_dup"), "IR should call quarter_dup");
    assert!(ir.contains("mul"), "IR should contain multiplication");

    println!("\n=== Generated IR for SQUARE ===");
    println!("{}", ir);
    println!("================================\n");
}

#[test]
fn test_compile_if_then() {
    // Test compiling: : TEST-IF 5 3 < IF 42 THEN ;
    // If 5 < 3 (false), don't push 42. If true, push 42.
    let context = Context::create();
    let mut compiler = Compiler::new(&context).unwrap();

    // Create AST: 5 3 < IF 42 THEN
    let ast = AstNode::Sequence(vec![
        AstNode::PushNumber(5),
        AstNode::PushNumber(3),
        AstNode::CallWord("<".to_string()),
        AstNode::IfThenElse {
            then_branch: vec![AstNode::PushNumber(42)],
            else_branch: None,
        },
    ]);

    // Compile it
    let result = compiler.compile_word("TEST_IF", &ast);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

    // Verify IR was generated
    let ir = compiler.get_ir();

    println!("\n=== Generated IR for TEST_IF ===");
    println!("{}", ir);
    println!("=================================\n");

    assert!(ir.contains("define void @TEST_IF"), "IR should contain TEST_IF function");
    // With aggressive optimization, LLVM may optimize away branches
    // Check for either control flow blocks OR optimized away (no branches for known false condition)
    let has_branches = ir.contains("then:") || ir.contains("merge:") || ir.contains("br i1");
    let has_conditional = ir.contains("icmp") || ir.contains("select");
    assert!(has_branches || has_conditional, "IR should contain some form of control flow");
}

#[test]
fn test_compile_if_then_else() {
    // Test compiling: : TEST-IF-ELSE 5 3 < IF 42 ELSE 99 THEN ;
    let context = Context::create();
    let mut compiler = Compiler::new(&context).unwrap();

    // Create AST: 5 3 < IF 42 ELSE 99 THEN
    let ast = AstNode::Sequence(vec![
        AstNode::PushNumber(5),
        AstNode::PushNumber(3),
        AstNode::CallWord("<".to_string()),
        AstNode::IfThenElse {
            then_branch: vec![AstNode::PushNumber(42)],
            else_branch: Some(vec![AstNode::PushNumber(99)]),
        },
    ]);

    // Compile it
    let result = compiler.compile_word("TEST_IF_ELSE", &ast);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

    // Verify IR was generated
    let ir = compiler.get_ir();

    println!("\n=== Generated IR for TEST_IF_ELSE ===");
    println!("{}", ir);
    println!("=======================================\n");

    assert!(ir.contains("define void @TEST_IF_ELSE"), "IR should contain TEST_IF_ELSE function");
    // With aggressive optimization, LLVM may optimize branches into select instructions
    // Check for either control flow blocks OR select instruction
    let has_branches = ir.contains("then:") && ir.contains("else:") && ir.contains("merge:");
    let has_select = ir.contains("select i1");
    assert!(has_branches || has_select, "IR should contain either branch blocks or select instruction");
}

#[test]
fn test_compile_begin_until() {
    // Test compiling: : COUNTDOWN 5 BEGIN 1 - DUP 0 < UNTIL DROP ;
    // Counts down from 5 to 0
    let context = Context::create();
    let mut compiler = Compiler::new(&context).unwrap();

    // Create AST: 5 BEGIN 1 - DUP 0 < UNTIL DROP
    let ast = AstNode::Sequence(vec![
        AstNode::PushNumber(5),
        AstNode::BeginUntil {
            body: vec![
                AstNode::PushNumber(1),
                AstNode::CallWord("-".to_string()),
                AstNode::CallWord("DUP".to_string()),
                AstNode::PushNumber(0),
                AstNode::CallWord("<".to_string()),
            ],
        },
        AstNode::CallWord("DROP".to_string()),
    ]);

    // Compile it
    let result = compiler.compile_word("COUNTDOWN", &ast);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

    // Verify IR was generated
    let ir = compiler.get_ir();
    assert!(ir.contains("define void @COUNTDOWN"), "IR should contain COUNTDOWN function");
    assert!(ir.contains("loop:"), "IR should contain loop block");
    assert!(ir.contains("exit:"), "IR should contain exit block");
    assert!(ir.contains("br i1"), "IR should contain conditional branch");

    println!("\n=== Generated IR for COUNTDOWN ===");
    println!("{}", ir);
    println!("===================================\n");
}

#[test]
fn test_compile_do_loop() {
    // Test compiling: : SUM-TO-N 0 SWAP 0 DO + LOOP ;
    // Note: This doesn't use I yet, so it just loops N times
    // For now, just test that it compiles without the I word
    let context = Context::create();
    let mut compiler = Compiler::new(&context).unwrap();

    // Create AST: 0 SWAP 0 DO 1 + LOOP (adds 1 N times)
    let ast = AstNode::Sequence(vec![
        AstNode::PushNumber(0),
        AstNode::CallWord("SWAP".to_string()),
        AstNode::PushNumber(0),
        AstNode::DoLoop {
            body: vec![
                AstNode::PushNumber(1),
                AstNode::CallWord("+".to_string()),
            ],
            increment: 1,
        },
    ]);

    // Compile it
    let result = compiler.compile_word("SUM_SIMPLE", &ast);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

    // Verify IR was generated
    let ir = compiler.get_ir();
    assert!(ir.contains("define void @SUM_SIMPLE"), "IR should contain SUM_SIMPLE function");
    assert!(ir.contains("do_loop:"), "IR should contain do_loop block");
    assert!(ir.contains("do_exit:"), "IR should contain do_exit block");
    assert!(ir.contains("phi"), "IR should contain phi node");

    println!("\n=== Generated IR for SUM_SIMPLE ===");
    println!("{}", ir);
    println!("====================================\n");
}

#[test]
fn test_compile_do_loop_with_i() {
    // Test compiling: : SUM-TO-N 0 SWAP 0 DO I + LOOP ;
    // This uses the I word to access the loop index
    let context = Context::create();
    let mut compiler = Compiler::new(&context).unwrap();

    // Create AST: 0 SWAP 0 DO I + LOOP
    let ast = AstNode::Sequence(vec![
        AstNode::PushNumber(0),
        AstNode::CallWord("SWAP".to_string()),
        AstNode::PushNumber(0),
        AstNode::DoLoop {
            body: vec![
                AstNode::CallWord("I".to_string()),
                AstNode::CallWord("+".to_string()),
            ],
            increment: 1,
        },
    ]);

    // Compile it
    let result = compiler.compile_word("SUM_TO_N", &ast);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

    // Verify IR was generated
    let ir = compiler.get_ir();
    assert!(ir.contains("define void @SUM_TO_N"), "IR should contain SUM_TO_N function");
    assert!(ir.contains("do_loop:"), "IR should contain do_loop block");
    assert!(ir.contains("phi"), "IR should contain phi node");

    println!("\n=== Generated IR for SUM_TO_N ===");
    println!("{}", ir);
    println!("==================================\n");
}
