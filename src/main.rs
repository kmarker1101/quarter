use quarter::{Dictionary, LoopStack, Stack, load_file, load_stdlib, parse_tokens};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

/// Track whether the Forth compiler has been loaded
static FORTH_COMPILER_LOADED: AtomicBool = AtomicBool::new(false);

/// Attempt to compile using the Forth self-hosting compiler
/// Returns true if successful, false otherwise
fn try_forth_compile(
    name: String,
    ast: &quarter::AstNode,
    dict: &mut quarter::Dictionary,
    stack: &mut quarter::Stack,
    loop_stack: &mut quarter::LoopStack,
    return_stack: &mut quarter::ReturnStack,
    memory: &mut quarter::Memory,
    _use_batch: bool,  // Unused - batch compilation only
    no_jit: bool,
    dump_ir: bool,
    verify_ir: bool,
    _included_files: &mut HashSet<String>,  // Unused - function returns immediately
) -> bool {
    // Incremental compilation disabled - use batch_compile_all_words() instead
    let _ = (name, ast, dict, stack, loop_stack, return_stack, memory, no_jit, dump_ir, verify_ir, _included_files);
    return false;

    // Load the Forth compiler if not already loaded
    #[allow(unreachable_code)]
    if !FORTH_COMPILER_LOADED.load(Ordering::Relaxed) {
        // Stdlib is already loaded by main(), no need to reload it here
        // Load compiler
        if let Err(e) = load_file(
            "forth/compiler.fth",
            stack,
            dict,
            loop_stack,
            return_stack,
            memory,
            no_jit,
            dump_ir,
            verify_ir,
            false,
            false,  // Not define-only
            _included_files,
        ) {
            eprintln!("Failed to load Forth compiler: {}", e);
            return false;
        }

        // Now that compiler is loaded, recompile stdlib words to JIT-compiled versions
        // This replaces the interpreted versions with native code
        if let Err(e) = quarter::load_stdlib(
            stack,
            dict,
            loop_stack,
            return_stack,
            memory,
            no_jit,
            dump_ir,
            verify_ir,
            true,
            _included_files,
        ) {
            eprintln!("Warning: Failed to JIT-compile stdlib: {}", e);
            // Continue anyway with interpreted stdlib
        }

        FORTH_COMPILER_LOADED.store(true, Ordering::Relaxed);
    }

    // Register the AST
    use quarter::ast_forth::ast_register_node;
    let ast_handle = ast_register_node(ast.clone());

    // Write word name to memory at address 302000
    let name_addr = 302000;
    for (i, ch) in name.bytes().enumerate() {
        // Store each character as a byte
        if let Err(_) = memory.store_byte(name_addr + i, ch as i64) {
            return false;
        }
    }

    // Push arguments for COMPILE-WORD: ( ast-handle name-addr name-len -- fn-ptr )
    stack.push(ast_handle, memory);
    stack.push(name_addr as i64, memory);
    stack.push(name.len() as i64, memory);

    // Execute COMPILE-WORD
    if let Err(e) = dict.execute_word("COMPILE-WORD", stack, loop_stack, return_stack, memory) {
        eprintln!("Forth compiler error: {}", e);
        return false;
    }

    // Get function pointer from stack (two 32-bit values: high, then low)
    if let (Some(fn_ptr_high), Some(fn_ptr_low)) = (stack.pop(memory), stack.pop(memory)) {
        // Reconstruct 64-bit pointer from two 32-bit values
        let fn_ptr = ((fn_ptr_high as u64) << 32) | ((fn_ptr_low as u64) & 0xFFFFFFFF);

        // Validate pointer is not null
        if fn_ptr == 0 {
            eprintln!("ERROR: Forth compiler returned NULL function pointer!");
            return false;
        }

        // Cast to JITFunction
        let jit_fn: quarter::dictionary::JITFunction =
            unsafe { std::mem::transmute(fn_ptr as *const ()) };

        // Register in dictionary
        dict.add_jit_compiled(name, jit_fn);
        return true;
    }

    eprintln!("ERROR: No function pointer on stack after COMPILE-WORD!");
    false
}

fn main() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = quarter::ReturnStack::new();
    let mut memory = quarter::Memory::new();
    let mut included_files: HashSet<String> = HashSet::new();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut no_jit = false;
    let mut dump_ir = false;
    let mut verify_ir = false;
    let mut compile_stdlib = false;
    let mut jit_mode = false;
    let mut filename: Option<String> = None;

    for arg in args.iter().skip(1) {
        if arg == "--no-jit" {
            no_jit = true;
        } else if arg == "--dump-ir" {
            dump_ir = true;
        } else if arg == "--verify-ir" {
            verify_ir = true;
        } else if arg == "--compile-stdlib" {
            compile_stdlib = true;
        } else if arg == "--jit" {
            jit_mode = true;
            compile_stdlib = true;  // JIT mode implies compile stdlib
        } else if !arg.starts_with("--") {
            filename = Some(arg.clone());
        }
    }


    // Load standard library (always interpreted initially)
    if let Err(e) = load_stdlib(
        &mut stack,
        &mut dict,
        &mut loop_stack,
        &mut return_stack,
        &mut memory,
        no_jit,
        dump_ir,
        verify_ir,
        false,  // Always load interpreted
        &mut included_files,
    ) {
        eprintln!("Error loading stdlib: {}", e);
        std::process::exit(1);
    }

    // Compile stdlib if requested (only for --compile-stdlib mode, not --jit)
    // In --jit mode, we batch compile everything together after loading user code
    if compile_stdlib && !jit_mode {
        if let Err(e) = quarter::batch_compile_all_words(
            &mut dict,
            &mut stack,
            &mut loop_stack,
            &mut return_stack,
            &mut memory,
            no_jit,
            dump_ir,
            verify_ir,
            &mut included_files,
        ) {
            eprintln!("Failed to compile stdlib: {}", e);
            std::process::exit(1);
        }
    }

    println!("Forth Interpreter v0.2");

    // Check for file argument
    // Supported extensions: .qtr, .fth, .forth, .quarter
    if let Some(file) = filename {
        println!("Loading {}", file);

        // Load file - in JIT mode, only load definitions without executing
        match load_file(
            &file,
            &mut stack,
            &mut dict,
            &mut loop_stack,
            &mut return_stack,
            &mut memory,
            no_jit,
            dump_ir,
            verify_ir,
            false,
            jit_mode,  // define_only = true in JIT mode
            &mut included_files,
        ) {
            Ok(_) => {
                // If JIT mode, batch compile user words now
                if jit_mode {
                    if let Err(e) = quarter::batch_compile_all_words(
                        &mut dict,
                        &mut stack,
                        &mut loop_stack,
                        &mut return_stack,
                        &mut memory,
                        no_jit,
                        dump_ir,
                        verify_ir,
                        &mut included_files,
                    ) {
                        eprintln!("Batch compilation failed: {}", e);
                        std::process::exit(1);
                    }

                    // Clear the stack before executing
                    while stack.pop(&mut memory).is_some() {}

                    // Remove the main file from included_files so it can be loaded again in pass 2
                    // (Dependencies loaded via INCLUDED remain in the set and won't be re-loaded)
                    included_files.remove(&file);

                    // Now execute the file with JIT-compiled code
                    if let Err(e) = load_file(
                        &file,
                        &mut stack,
                        &mut dict,
                        &mut loop_stack,
                        &mut return_stack,
                        &mut memory,
                        no_jit,
                        dump_ir,
                        verify_ir,
                        false,
                        false,  // define_only = false, now execute everything
                        &mut included_files,
                    ) {
                        eprintln!("JIT execution failed: {}", e);
                        std::process::exit(1);
                    }
                }
                return;
            }
            Err(e) => {
                eprintln!("Error loading {}: {}", file, e);
                std::process::exit(1);
            }
        }
    }

    println!("Type 'quit' to exit");

    let mut rl = DefaultEditor::new().unwrap();

    // State for multi-line definitions
    let mut compiling = false;
    let mut compile_buffer: Vec<String> = Vec::new();

    loop {
        let prompt = if compiling { "compiled " } else { "quarter> " };
        let readline = rl.readline(prompt);

        match readline {
            Ok(line) => {
                let input = line.trim();

                if input == "quit" {
                    break;
                }

                // Strip comments before processing
                let input = quarter::strip_comments(input);
                let input = input.trim();

                if !input.is_empty() {
                    rl.add_history_entry(input).unwrap();
                }

                let tokens: Vec<&str> = input.split_whitespace().collect();

                if tokens.is_empty() {
                    continue;
                }

                // Handle multi-line compilation mode
                if compiling {
                    // We're in compilation mode, accumulate tokens
                    compile_buffer.push(input.to_string());

                    // Check if this line contains ;
                    if tokens.iter().any(|&t| t.to_uppercase() == ";") {
                        // End of definition - compile everything
                        let full_def = compile_buffer.join(" ");
                        let all_tokens: Vec<&str> = full_def.split_whitespace().collect();

                        if all_tokens.len() < 3 {
                            println!("Invalid word definition");
                            compiling = false;
                            compile_buffer.clear();
                            continue;
                        }

                        let word_name = all_tokens[1].to_uppercase();
                        let word_tokens = &all_tokens[2..all_tokens.len() - 1];

                        match parse_tokens(word_tokens, &dict, Some(&word_name)) {
                            Ok(ast) => {
                                // Validate that all words in the AST exist (allow forward reference for recursion)
                                if let Err(e) = ast.validate_with_name(&dict, Some(&word_name)) {
                                    println!("{}", e);
                                } else {
                                    // Skip JIT compilation for word redefinitions to avoid memory leaks and registry collisions
                                    // When redefining, always use interpreted mode
                                    let is_redefinition = dict.has_word(&word_name);
                                    if !is_redefinition {
                                        // Try Forth compiler
                                        let compiled = try_forth_compile(
                                            word_name.clone(),
                                            &ast,
                                            &mut dict,
                                            &mut stack,
                                            &mut loop_stack,
                                            &mut return_stack,
                                            &mut memory,
                                            false,  // Batch compilation only
                                            no_jit,
                                            dump_ir,
                                            verify_ir,
                                            &mut included_files,
                                        );

                                        if !compiled {
                                            dict.add_compiled(word_name, ast);
                                        }
                                    } else {
                                        dict.add_compiled(word_name, ast);
                                    }
                                    println!("ok");
                                }
                            }
                            Err(e) => {
                                println!("Parse error: {}", e);
                            }
                        }

                        compiling = false;
                        compile_buffer.clear();
                    }
                    continue;
                }

                // Not in compilation mode - process normally
                if tokens.first().map(|s| s.to_uppercase()) == Some("INCLUDE".to_string()) {
                    // INCLUDE <filename>
                    if tokens.len() < 2 {
                        println!("INCLUDE requires a filename");
                        continue;
                    }

                    let filename = tokens[1];
                    match load_file(
                        filename,
                        &mut stack,
                        &mut dict,
                        &mut loop_stack,
                        &mut return_stack,
                        &mut memory,
                        no_jit,
                        dump_ir,
                        verify_ir,
                        false,
                        false,  // Not define-only in REPL
                        &mut included_files,
                    ) {
                        Ok(_) => {
                            println!("ok");
                        }
                        Err(e) => {
                            println!("Error loading {}: {}", filename, e);
                        }
                    }
                } else if tokens.first().map(|s| s.to_uppercase()) == Some(":".to_string()) {
                    // Definition mode
                    if tokens.last().map(|s| s.to_uppercase()) == Some(";".to_string()) {
                        // Single-line definition
                        if tokens.len() < 3 {
                            println!("Invalid word definition");
                            continue;
                        }

                        let word_name = tokens[1].to_uppercase();
                        let word_tokens = &tokens[2..tokens.len() - 1];

                        match parse_tokens(word_tokens, &dict, Some(&word_name)) {
                            Ok(ast) => {
                                // Validate that all words in the AST exist (allow forward reference for recursion)
                                if let Err(e) = ast.validate_with_name(&dict, Some(&word_name)) {
                                    println!("{}", e);
                                } else {
                                    // Skip JIT compilation for word redefinitions to avoid memory leaks and registry collisions
                                    // When redefining, always use interpreted mode
                                    let is_redefinition = dict.has_word(&word_name);
                                    if !is_redefinition {
                                        // Try Forth compiler
                                        let compiled = try_forth_compile(
                                            word_name.clone(),
                                            &ast,
                                            &mut dict,
                                            &mut stack,
                                            &mut loop_stack,
                                            &mut return_stack,
                                            &mut memory,
                                            false,  // Batch compilation only
                                            no_jit,
                                            dump_ir,
                                            verify_ir,
                                            &mut included_files,
                                        );

                                        if !compiled {
                                            dict.add_compiled(word_name, ast);
                                        }
                                    } else {
                                        dict.add_compiled(word_name, ast);
                                    }
                                    println!("ok");
                                }
                            }
                            Err(e) => {
                                println!("Parse error: {}", e);
                            }
                        }
                    } else {
                        // Multi-line definition - enter compilation mode
                        compiling = true;
                        compile_buffer.clear();
                        compile_buffer.push(input.to_string());
                    }
                } else if tokens.first().map(|s| s.to_uppercase()) == Some("VARIABLE".to_string()) {
                    // VARIABLE <name>
                    if tokens.len() < 2 {
                        println!("VARIABLE requires a name");
                        continue;
                    }

                    let var_name = tokens[1].to_uppercase();
                    let addr = memory.here();

                    // Allocate 1 cell (4 bytes) for the variable
                    match memory.allot(4) {
                        Ok(_) => {
                            // Create a word that pushes the variable's address
                            use quarter::AstNode;
                            let var_ast = AstNode::PushNumber(addr);
                            dict.add_compiled(var_name, var_ast);
                            println!("ok");
                        }
                        Err(e) => {
                            println!("{}", e);
                        }
                    }
                } else if tokens.len() >= 3
                    && tokens.get(1).map(|s| s.to_uppercase()) == Some("CONSTANT".to_string())
                {
                    // <value> CONSTANT <name>
                    // Parse and push the value first
                    match parse_tokens(&tokens[0..1], &dict, None) {
                        Ok(ast) => {
                            match ast.execute(
                                &mut stack,
                                &dict,
                                &mut loop_stack,
                                &mut return_stack,
                                &mut memory,
                            ) {
                                Ok(_) => {
                                    // Now pop it back and create the constant
                                    match stack.pop(&mut memory) {
                                        Some(value) => {
                                            let const_name = tokens[2].to_uppercase();
                                            use quarter::AstNode;
                                            let const_ast = AstNode::PushNumber(value);
                                            dict.add_compiled(const_name, const_ast);
                                            println!("ok");
                                        }
                                        None => {
                                            println!("Stack underflow for CONSTANT");
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("{}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Parse error: {}", e);
                        }
                    }
                } else if tokens.first().map(|s| s.to_uppercase()) == Some("CREATE".to_string()) {
                    // CREATE <name>
                    if tokens.len() < 2 {
                        println!("CREATE requires a name");
                        continue;
                    }

                    let create_name = tokens[1].to_uppercase();
                    let addr = memory.here();

                    // Create a word that pushes the data address
                    use quarter::AstNode;
                    let create_ast = AstNode::PushNumber(addr);
                    dict.add_compiled(create_name, create_ast);
                    println!("ok");
                } else if tokens.first().map(|s| s.to_uppercase()) == Some("S\"".to_string())
                    && tokens.last().map(|s| s.to_uppercase()) == Some("INCLUDED".to_string())
                {
                    // S" filename" INCLUDED pattern (for forth-mode)
                    // Parse S" part to get filename on stack, then call INCLUDED
                    let s_quote_end = tokens.iter().position(|&t| t.ends_with('"') && t != "S\"");
                    if let Some(_end_idx) = s_quote_end {
                        let all_tokens_str = tokens.join(" ");
                        match quarter::execute_line(
                            &all_tokens_str,
                            &mut stack,
                            &mut dict,
                            &mut loop_stack,
                            &mut return_stack,
                            &mut memory,
                            no_jit,
                            dump_ir,
                            verify_ir,
                            false,
                            false,  // Not define-only in REPL
                            &mut included_files,
                        ) {
                            Ok(_) => println!("ok"),
                            Err(e) => println!("{}", e),
                        }
                    } else {
                        println!("Malformed S\" ... \" INCLUDED");
                    }
                } else {
                    // Normal execution mode
                    // Check for compile-only words
                    if tokens.iter().any(|&t| {
                        let upper = t.to_uppercase();
                        upper == "IF"
                            || upper == "THEN"
                            || upper == "ELSE"
                            || upper == "BEGIN"
                            || upper == "UNTIL"
                            || upper == "WHILE"
                            || upper == "REPEAT"
                            || upper == "DO"
                            || upper == "LOOP"
                            || upper == "+LOOP"
                            || upper == "LEAVE"
                            || upper == "EXIT"
                            || upper == ".\""
                    }) {
                        println!(
                            "Error: Control flow and string words are compile-only (use inside : ; definitions)"
                        );
                    } else {
                        match parse_tokens(&tokens, &dict, None) {
                            Ok(ast) => match ast.execute(
                                &mut stack,
                                &dict,
                                &mut loop_stack,
                                &mut return_stack,
                                &mut memory,
                            ) {
                                Ok(_) => println!("ok"),
                                Err(e) => println!("{}", e),
                            },
                            Err(e) => {
                                println!("Parse error: {}", e);
                            }
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    println!("Goodbye!");
}
