use quarter::{Dictionary, LoopStack, Stack, load_file, parse_tokens};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

fn main() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = quarter::ReturnStack::new();
    let mut memory = quarter::Memory::new();

    println!("Forth Interpreter v0.1");

    // Check for file argument
    // Supported extensions: .qtr, .fth, .forth, .quarter
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        println!("Loading {}", filename);
        match load_file(filename, &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory) {
            Ok(_) => {
                return;
            }
            Err(e) => {
                eprintln!("Error loading {}: {}", filename, e);
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

                        match parse_tokens(word_tokens) {
                            Ok(ast) => {
                                // Validate that all words in the AST exist
                                if let Err(e) = ast.validate(&dict) {
                                    println!("{}", e);
                                } else {
                                    dict.add_compiled(word_name, ast);
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
                    match load_file(filename, &mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory) {
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

                        match parse_tokens(word_tokens) {
                            Ok(ast) => {
                                // Validate that all words in the AST exist
                                if let Err(e) = ast.validate(&dict) {
                                    println!("{}", e);
                                } else {
                                    dict.add_compiled(word_name, ast);
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
                } else if tokens.len() >= 3 && tokens.get(1).map(|s| s.to_uppercase()) == Some("CONSTANT".to_string()) {
                    // <value> CONSTANT <name>
                    // Parse and push the value first
                    match parse_tokens(&tokens[0..1]) {
                        Ok(ast) => {
                            match ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory) {
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
                            || upper == "S\""
                    }) {
                        println!(
                            "Error: Control flow and string words are compile-only (use inside : ; definitions)"
                        );
                    } else {
                        match parse_tokens(&tokens) {
                            Ok(ast) => match ast.execute(&mut stack, &dict, &mut loop_stack, &mut return_stack, &mut memory) {
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
