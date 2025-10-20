pub mod ast;
pub mod dictionary;
pub mod stack;
pub mod words;

pub use ast::AstNode;
pub use dictionary::Dictionary;
pub use stack::Stack;

use std::fs;

// Loop stack for DO...LOOP counters
#[derive(Debug, Clone)]
pub struct LoopStack {
    stack: Vec<(i32, i32)>, // (index, limit) pairs
}

impl LoopStack {
    pub fn new() -> Self {
        LoopStack { stack: Vec::new() }
    }

    pub fn push_loop(&mut self, start: i32, limit: i32) {
        self.stack.push((start, limit));
    }

    pub fn pop_loop(&mut self) -> Option<(i32, i32)> {
        self.stack.pop()
    }

    pub fn get_index(&self) -> Option<i32> {
        self.stack.last().map(|(index, _)| *index)
    }

    pub fn get_outer_index(&self) -> Option<i32> {
        // Get the second-to-last loop index (for J word)
        if self.stack.len() >= 2 {
            self.stack[self.stack.len() - 2].0.into()
        } else {
            None
        }
    }

    pub fn increment(&mut self, amount: i32) -> bool {
        if let Some((index, limit)) = self.stack.last_mut() {
            *index += amount;
            *index < *limit // Continue if index < limit
        } else {
            false
        }
    }
}

// Return stack for >R, R>, R@
#[derive(Debug, Clone)]
pub struct ReturnStack {
    stack: Vec<i32>,
}

impl ReturnStack {
    pub fn new() -> Self {
        ReturnStack { stack: Vec::new() }
    }

    pub fn push(&mut self, value: i32) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<i32> {
        self.stack.pop()
    }

    pub fn peek(&self) -> Option<i32> {
        self.stack.last().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

pub fn parse_tokens(tokens: &[&str]) -> Result<AstNode, String> {
    let mut nodes = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];

        match token {
            ".\"" => {
                // Handle string literals: collect tokens until closing "
                let mut string_parts: Vec<String> = Vec::new();
                i += 1; // Skip past ."

                while i < tokens.len() {
                    let part = tokens[i];
                    if part.ends_with('"') {
                        // Found closing quote
                        if part == "\"" {
                            // Just a closing quote - means there was a trailing space
                            // Add space to the last part if there is one
                            if !string_parts.is_empty() {
                                let last_idx = string_parts.len() - 1;
                                string_parts[last_idx].push(' ');
                            }
                        } else {
                            // Text followed by quote
                            let without_quote = &part[..part.len() - 1];
                            if !without_quote.is_empty() {
                                string_parts.push(without_quote.to_string());
                            }
                        }
                        i += 1;
                        break;
                    } else {
                        string_parts.push(part.to_string());
                        i += 1;
                    }
                }

                let string_content = string_parts.join(" ");
                nodes.push(AstNode::PrintString(string_content));
            }
            "BEGIN" => {
                // Find matching UNTIL or WHILE/REPEAT
                let end_pos = find_begin_end(&tokens[i + 1..])?;

                // Check if it's BEGIN...UNTIL or BEGIN...WHILE...REPEAT
                let end_keyword = tokens[i + 1 + end_pos.0];

                if end_keyword == "UNTIL" {
                    // BEGIN...UNTIL loop
                    let body_tokens = &tokens[i + 1..i + 1 + end_pos.0];
                    let body_ast = parse_tokens(body_tokens)?;

                    nodes.push(AstNode::BeginUntil {
                        body: if let AstNode::Sequence(v) = body_ast {
                            v
                        } else {
                            vec![body_ast]
                        },
                    });

                    i += end_pos.0 + 2; // Skip past UNTIL
                } else if end_keyword == "REPEAT" {
                    // BEGIN...WHILE...REPEAT loop
                    if let Some(while_pos) = end_pos.1 {
                        let condition_tokens = &tokens[i + 1..i + 1 + while_pos];
                        let body_tokens = &tokens[i + 1 + while_pos + 1..i + 1 + end_pos.0];

                        let condition_ast = parse_tokens(condition_tokens)?;
                        let body_ast = parse_tokens(body_tokens)?;

                        nodes.push(AstNode::BeginWhileRepeat {
                            condition: if let AstNode::Sequence(v) = condition_ast {
                                v
                            } else {
                                vec![condition_ast]
                            },
                            body: if let AstNode::Sequence(v) = body_ast {
                                v
                            } else {
                                vec![body_ast]
                            },
                        });

                        i += end_pos.0 + 2; // Skip past REPEAT
                    } else {
                        return Err("BEGIN...REPEAT requires WHILE".to_string());
                    }
                } else {
                    return Err(format!("Unexpected {} after BEGIN", end_keyword));
                }
            }
            "DO" => {
                // Find matching LOOP or +LOOP
                let loop_pos = find_do_loop(&tokens[i + 1..])?;
                let loop_keyword = tokens[i + 1 + loop_pos];

                let body_tokens = &tokens[i + 1..i + 1 + loop_pos];
                let body_ast = parse_tokens(body_tokens)?;

                let increment = if loop_keyword == "+LOOP" {
                    0 // Special marker for +LOOP (stack-based increment)
                } else {
                    1 // LOOP always increments by 1
                };

                nodes.push(AstNode::DoLoop {
                    body: if let AstNode::Sequence(v) = body_ast {
                        v
                    } else {
                        vec![body_ast]
                    },
                    increment,
                });

                i += loop_pos + 2; // Skip past LOOP/+LOOP
            }
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
            "LEAVE" => {
                nodes.push(AstNode::Leave);
                i += 1;
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

// Find matching UNTIL or WHILE/REPEAT for BEGIN
// Returns (end_pos, while_pos)
fn find_begin_end(tokens: &[&str]) -> Result<(usize, Option<usize>), String> {
    let mut depth = 0;
    let mut while_pos = None;

    for (i, &token) in tokens.iter().enumerate() {
        match token {
            "BEGIN" => depth += 1,
            "WHILE" => {
                if depth == 0 && while_pos.is_none() {
                    while_pos = Some(i);
                }
            }
            "UNTIL" => {
                if depth == 0 {
                    return Ok((i, None));
                }
                depth -= 1;
            }
            "REPEAT" => {
                if depth == 0 {
                    return Ok((i, while_pos));
                }
                depth -= 1;
            }
            _ => {}
        }
    }

    Err("Missing UNTIL or REPEAT".to_string())
}

// Find matching LOOP or +LOOP for DO
fn find_do_loop(tokens: &[&str]) -> Result<usize, String> {
    let mut depth = 0;

    for (i, &token) in tokens.iter().enumerate() {
        match token {
            "DO" => depth += 1,
            "LOOP" | "+LOOP" => {
                if depth == 0 {
                    return Ok(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }

    Err("Missing LOOP or +LOOP".to_string())
}

pub fn load_file(
    filename: &str,
    stack: &mut Stack,
    dict: &mut Dictionary,
    loop_stack: &mut LoopStack,
    return_stack: &mut ReturnStack,
) -> Result<(), String> {
    let contents = fs::read_to_string(filename).map_err(|e| format!("Cannot read file: {}", e))?;

    // Process file as token stream to support multi-line definitions
    let mut processed = String::new();

    for line in contents.lines() {
        let line = line.trim();

        // Skip backslash comments (entire line)
        if line.starts_with('\\') {
            continue;
        }

        // Handle inline backslash comments (remove everything after \)
        let line = if let Some(pos) = line.find('\\') {
            &line[..pos]
        } else {
            line
        };

        processed.push_str(line);
        processed.push(' ');
    }

    // Remove parenthesis comments ( ... )
    let mut result = String::new();
    let mut in_paren_comment = false;

    for ch in processed.chars() {
        if ch == '(' {
            in_paren_comment = true;
        } else if ch == ')' {
            in_paren_comment = false;
        } else if !in_paren_comment {
            result.push(ch);
        }
    }

    // Now execute the entire file as one token stream
    execute_line(&result, stack, dict, loop_stack, return_stack)?;

    Ok(())
}

pub fn execute_line(
    input: &str,
    stack: &mut Stack,
    dict: &mut Dictionary,
    loop_stack: &mut LoopStack,
    return_stack: &mut ReturnStack,
) -> Result<(), String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    if tokens.is_empty() {
        return Ok(());
    }

    // Process tokens sequentially, handling multiple definitions
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i] == "INCLUDE" {
            // INCLUDE <filename>
            if i + 1 >= tokens.len() {
                return Err("INCLUDE requires a filename".to_string());
            }

            let filename = tokens[i + 1];
            load_file(filename, stack, dict, loop_stack, return_stack)?;
            i += 2;
        } else if tokens[i] == ":" {
            // Find matching semicolon for definition
            let mut semicolon_pos = None;
            for j in (i + 1)..tokens.len() {
                if tokens[j] == ";" {
                    semicolon_pos = Some(j);
                    break;
                }
            }

            if let Some(end) = semicolon_pos {
                if end - i < 2 {
                    return Err("Invalid word definition".to_string());
                }

                let word_name = tokens[i + 1].to_string();
                let word_tokens = &tokens[i + 2..end];

                let ast = parse_tokens(word_tokens)?;
                dict.add_compiled(word_name, ast);
                i = end + 1;
            } else {
                return Err("Missing ; in word definition".to_string());
            }
        } else {
            // Collect tokens until we hit : or INCLUDE or end
            let mut exec_tokens = Vec::new();
            while i < tokens.len() && tokens[i] != ":" && tokens[i] != "INCLUDE" {
                exec_tokens.push(tokens[i]);
                i += 1;
            }

            if !exec_tokens.is_empty() {
                // Check for compile-only words
                if exec_tokens.iter().any(|&t| {
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
                    return Err("Control flow and string words (IF/THEN/ELSE/BEGIN/UNTIL/WHILE/REPEAT/DO/LOOP/LEAVE/.\") are compile-only".to_string());
                }

                let ast = parse_tokens(&exec_tokens)?;
                ast.execute(stack, dict, loop_stack, return_stack)?;
            }
        }
    }

    Ok(())
}
