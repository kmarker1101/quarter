pub mod ast;
pub mod dictionary;
pub mod stack;
pub mod words;

pub use ast::AstNode;
pub use dictionary::Dictionary;
pub use stack::Stack;

use std::fs;

pub fn parse_tokens(tokens: &[&str]) -> Result<AstNode, String> {
    let mut nodes = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];

        match token {
            "IF" => {
                // Find matching THEN or ELSE/THEN
                let (then_end, else_start) = find_then_else(&tokens[i + 1..])?;

                // Parse THEN branch (from after IF to ELSE, or to THEN if no ELSE)
                let then_tokens = if let Some(else_pos) = else_start {
                    &tokens[i + 1..i + 1 + else_pos]
                } else {
                    &tokens[i + 1..i + 1 + then_end]
                };
                let then_branch = parse_tokens(then_tokens)?;

                // Parse ELSE branch if it exists (from after ELSE to THEN)
                let else_branch = if let Some(else_pos) = else_start {
                    let else_tokens = &tokens[i + 1 + else_pos + 1..i + 1 + then_end];
                    Some(parse_tokens(else_tokens)?)
                } else {
                    None
                };

                nodes.push(AstNode::IfThenElse {
                    then_branch: if let AstNode::Sequence(v) = then_branch {
                        v
                    } else {
                        vec![then_branch]
                    },
                    else_branch: else_branch.map(|e| {
                        if let AstNode::Sequence(v) = e {
                            v
                        } else {
                            vec![e]
                        }
                    }),
                });

                i += then_end + 2; // Skip past THEN
            }
            "THEN" | "ELSE" => {
                return Err("Unexpected THEN or ELSE".to_string());
            }
            _ => {
                // Try to parse as number, otherwise it's a word
                if let Ok(num) = token.parse::<i32>() {
                    nodes.push(AstNode::PushNumber(num));
                } else {
                    nodes.push(AstNode::CallWord(token.to_string()));
                }
                i += 1;
            }
        }
    }

    if nodes.len() == 1 {
        Ok(nodes.into_iter().next().unwrap())
    } else {
        Ok(AstNode::Sequence(nodes))
    }
}

fn find_then_else(tokens: &[&str]) -> Result<(usize, Option<usize>), String> {
    let mut depth = 0;
    let mut else_pos = None;

    for (i, &token) in tokens.iter().enumerate() {
        match token {
            "IF" => depth += 1,
            "ELSE" => {
                if depth == 0 && else_pos.is_none() {
                    else_pos = Some(i);
                }
            }
            "THEN" => {
                if depth == 0 {
                    return Ok((i, else_pos));
                }
                depth -= 1;
            }
            _ => {}
        }
    }

    Err("Missing THEN".to_string())
}

pub fn load_file(filename: &str, stack: &mut Stack, dict: &mut Dictionary) -> Result<(), String> {
    let contents = fs::read_to_string(filename)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    for line in contents.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('\\') || line.starts_with('(') {
            continue;
        }

        execute_line(line, stack, dict)?;
    }

    Ok(())
}

pub fn execute_line(input: &str, stack: &mut Stack, dict: &mut Dictionary) -> Result<(), String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    if tokens.is_empty() {
        return Ok(());
    }

    if tokens.first() == Some(&":") {
        // Definition mode
        if let Some(&";") = tokens.last() {
            if tokens.len() < 3 {
                return Err("Invalid word definition".to_string());
            }

            let word_name = tokens[1].to_string();
            let word_tokens = &tokens[2..tokens.len() - 1];

            let ast = parse_tokens(word_tokens)?;
            dict.add_compiled(word_name, ast);
        } else {
            return Err("Missing ; in word definition".to_string());
        }
    } else {
        // Check for compile-only words
        if tokens
            .iter()
            .any(|&t| t == "IF" || t == "THEN" || t == "ELSE")
        {
            return Err("IF/THEN/ELSE are compile-only words".to_string());
        }

        let ast = parse_tokens(&tokens)?;
        ast.execute(stack, dict)?;
    }

    Ok(())
}
