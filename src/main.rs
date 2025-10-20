use quarter::{Dictionary, LoopStack, Stack, load_file, parse_tokens};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

fn main() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();

    println!("Forth Interpreter v0.1");

    // Check for file argument
    // Supported extensions: .qtr, .fth, .forth, .quarter
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        println!("Loading {}", filename);
        match load_file(filename, &mut stack, &mut dict, &mut loop_stack) {
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

    loop {
        let readline = rl.readline("quarter> ");

        match readline {
            Ok(line) => {
                let input = line.trim();

                if input == "quit" {
                    break;
                }

                if !input.is_empty() {
                    rl.add_history_entry(input).unwrap();
                }

                let tokens: Vec<&str> = input.split_whitespace().collect();

                if tokens.is_empty() {
                    continue;
                }

                if tokens.first() == Some(&"INCLUDE") {
                    // INCLUDE <filename>
                    if tokens.len() < 2 {
                        println!("INCLUDE requires a filename");
                        continue;
                    }

                    let filename = tokens[1];
                    match load_file(filename, &mut stack, &mut dict, &mut loop_stack) {
                        Ok(_) => {
                            println!("ok");
                        }
                        Err(e) => {
                            println!("Error loading {}: {}", filename, e);
                        }
                    }
                } else if tokens.first() == Some(&":") {
                    // Definition mode
                    if let Some(&";") = tokens.last() {
                        if tokens.len() < 3 {
                            println!("Invalid word definition");
                            continue;
                        }

                        let word_name = tokens[1].to_string();
                        let word_tokens = &tokens[2..tokens.len() - 1];

                        match parse_tokens(word_tokens) {
                            Ok(ast) => {
                                dict.add_compiled(word_name, ast);
                                println!("ok");
                            }
                            Err(e) => {
                                println!("Parse error: {}", e);
                            }
                        }
                    } else {
                        println!("Missing ; in word definition");
                    }
                } else {
                    // Normal execution mode
                    // Check for compile-only words
                    if tokens.iter().any(|&t| {
                        t == "IF"
                            || t == "THEN"
                            || t == "ELSE"
                            || t == "BEGIN"
                            || t == "UNTIL"
                            || t == "WHILE"
                            || t == "REPEAT"
                            || t == "DO"
                            || t == "LOOP"
                            || t == "+LOOP"
                            || t == "LEAVE"
                            || t == ".\""
                    }) {
                        println!(
                            "Error: Control flow and string words are compile-only (use inside : ; definitions)"
                        );
                    } else {
                        match parse_tokens(&tokens) {
                            Ok(ast) => match ast.execute(&mut stack, &dict, &mut loop_stack) {
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
