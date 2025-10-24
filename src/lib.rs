pub mod ast;
pub mod ast_forth;
pub mod dictionary;
pub mod llvm_codegen;
pub mod llvm_forth;
pub mod stack;
pub mod words;

pub use ast::AstNode;
pub use dictionary::Dictionary;
pub use stack::Stack;

use std::fs;

// Embedded standard library files
const CORE_FTH: &str = include_str!("../stdlib/core.fth");
#[allow(dead_code)] // TODO: Re-enable once DEPTH in loops is fixed
const TEST_FRAMEWORK_FTH: &str = include_str!("../stdlib/test-framework.fth");
#[allow(dead_code)] // TODO: Re-enable once test framework is fixed

/// Clear all global registries (for testing)
/// Call this between tests to avoid state pollution
pub fn clear_test_state() {
    ast_forth::ast_clear_registry();
    llvm_codegen::clear_jit_registry();
}

// Loop stack for DO...LOOP counters
#[derive(Debug, Clone)]
pub struct LoopStack {
    stack: Vec<(i64, i64)>, // (index, limit) pairs
}

impl LoopStack {
    pub fn new() -> Self {
        LoopStack { stack: Vec::new() }
    }

    pub fn push_loop(&mut self, start: i64, limit: i64) {
        self.stack.push((start, limit));
    }

    pub fn pop_loop(&mut self) -> Option<(i64, i64)> {
        self.stack.pop()
    }

    pub fn get_index(&self) -> Option<i64> {
        self.stack.last().map(|(index, _)| *index)
    }

    pub fn get_outer_index(&self) -> Option<i64> {
        // Get the second-to-last loop index (for J word)
        if self.stack.len() >= 2 {
            self.stack[self.stack.len() - 2].0.into()
        } else {
            None
        }
    }

    pub fn increment(&mut self, amount: i64) -> bool {
        if let Some((index, limit)) = self.stack.last_mut() {
            *index += amount;
            *index < *limit // Continue if index < limit
        } else {
            false
        }
    }
}

// Return stack for >R, R>, R@
// Return stack now resides in Memory space starting at address 0x010000
// Return stack pointer (RP) tracks current top of return stack
#[derive(Debug, Clone)]
pub struct ReturnStack {
    rp: usize, // Return stack pointer (byte address in memory)
}

impl ReturnStack {
    pub fn new() -> Self {
        ReturnStack {
            rp: 0x010000, // Start at beginning of return stack region
        }
    }

    pub fn push(&mut self, value: i64, memory: &mut Memory) {
        // Store value at current RP
        memory.store(self.rp, value).expect("Return stack overflow");
        // Move RP to next cell (8 bytes)
        self.rp += 8;
    }

    pub fn pop(&mut self, memory: &mut Memory) -> Option<i64> {
        if self.rp == 0x010000 {
            return None; // Return stack underflow
        }
        // Move RP back one cell
        self.rp -= 8;
        // Fetch value at new RP
        memory.fetch(self.rp).ok()
    }

    pub fn peek(&self, memory: &Memory) -> Option<i64> {
        if self.rp == 0x010000 {
            return None; // Return stack empty
        }
        // Peek at top of return stack (RP - 8)
        memory.fetch(self.rp - 8).ok()
    }

    pub fn is_empty(&self) -> bool {
        self.rp == 0x010000
    }

    // New methods for return stack pointer access
    pub fn get_rp(&self) -> usize {
        self.rp
    }

    pub fn set_rp(&mut self, rp: usize) {
        self.rp = rp;
    }

    pub fn as_mut_ptr(&mut self) -> usize {
        self.rp
    }

    // Get mutable pointer to return stack pointer (for JIT)
    pub fn rp_mut_ptr(&mut self) -> *mut usize {
        &mut self.rp as *mut usize
    }
}

// Memory for @, !, C@, C!
// Memory layout:
// 0x000000-0x00FFFF: Data Stack (64KB)
// 0x010000-0x01FFFF: Return Stack (64KB)
// 0x020000-0x7FFFFF: User Memory and Dictionary (~7.5MB)
#[derive(Debug)]
pub struct Memory {
    bytes: Vec<u8>,
    dp: usize, // Dictionary pointer - tracks next allocation address
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            bytes: vec![0; 8 * 1024 * 1024], // 8MB like gforth
            dp: 0x020000,                    // Start dictionary at beginning of user memory
        }
    }

    // HERE - return current dictionary pointer
    pub fn here(&self) -> i64 {
        self.dp as i64
    }

    // ALLOT - allocate n bytes in dictionary space
    pub fn allot(&mut self, n: i64) -> Result<(), String> {
        let new_dp = (self.dp as i64 + n) as usize;
        if new_dp >= self.bytes.len() {
            return Err("Dictionary overflow".to_string());
        }
        self.dp = new_dp;
        Ok(())
    }

    // @ - fetch cell (8 bytes as i64, little-endian)
    pub fn fetch(&self, addr: usize) -> Result<i64, String> {
        if addr + 8 > self.bytes.len() {
            return Err(format!("Memory fetch out of bounds: address {}", addr));
        }
        let bytes = [
            self.bytes[addr],
            self.bytes[addr + 1],
            self.bytes[addr + 2],
            self.bytes[addr + 3],
            self.bytes[addr + 4],
            self.bytes[addr + 5],
            self.bytes[addr + 6],
            self.bytes[addr + 7],
        ];
        Ok(i64::from_le_bytes(bytes))
    }

    // ! - store cell (i64 as 8 bytes, little-endian)
    pub fn store(&mut self, addr: usize, value: i64) -> Result<(), String> {
        if addr + 8 > self.bytes.len() {
            return Err(format!("Memory store out of bounds: address {}", addr));
        }
        let bytes = value.to_le_bytes();
        self.bytes[addr] = bytes[0];
        self.bytes[addr + 1] = bytes[1];
        self.bytes[addr + 2] = bytes[2];
        self.bytes[addr + 3] = bytes[3];
        self.bytes[addr + 4] = bytes[4];
        self.bytes[addr + 5] = bytes[5];
        self.bytes[addr + 6] = bytes[6];
        self.bytes[addr + 7] = bytes[7];
        Ok(())
    }

    // C@ - fetch byte (return as i64)
    pub fn fetch_byte(&self, addr: usize) -> Result<i64, String> {
        if addr >= self.bytes.len() {
            return Err(format!("Memory byte fetch out of bounds: address {}", addr));
        }
        Ok(self.bytes[addr] as i64)
    }

    // C! - store byte (store low byte of i64)
    pub fn store_byte(&mut self, addr: usize, value: i64) -> Result<(), String> {
        if addr >= self.bytes.len() {
            return Err(format!("Memory byte store out of bounds: address {}", addr));
        }
        self.bytes[addr] = (value & 0xFF) as u8;
        Ok(())
    }

    // Get a mutable pointer to memory at a given address
    // Used for JIT-compiled functions
    pub fn get_ptr_at(&mut self, addr: usize) -> *mut i64 {
        unsafe { self.bytes.as_mut_ptr().add(addr) as *mut i64 }
    }

    // Get mutable pointer to start of memory buffer (for JIT)
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.bytes.as_mut_ptr()
    }
}

pub fn parse_tokens(tokens: &[&str]) -> Result<AstNode, String> {
    let mut nodes = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];
        let token_upper = token.to_uppercase();

        match token_upper.as_str() {
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
            "S\"" => {
                // Handle S" string literals: collect tokens until closing "
                let mut string_parts: Vec<String> = Vec::new();
                i += 1; // Skip past S"

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
                nodes.push(AstNode::StackString(string_content));
            }
            "BEGIN" => {
                // Find matching UNTIL or WHILE/REPEAT
                let end_pos = find_begin_end(&tokens[i + 1..])?;

                // Check if it's BEGIN...UNTIL or BEGIN...WHILE...REPEAT
                let end_keyword = tokens[i + 1 + end_pos.0].to_uppercase();

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
            "EXIT" => {
                nodes.push(AstNode::Exit);
                i += 1;
            }
            "INLINE" => {
                // INLINE <instruction> - marks next token as an inline LLVM instruction
                if i + 1 >= tokens.len() {
                    return Err("INLINE requires an instruction name".to_string());
                }
                let instruction = tokens[i + 1].to_uppercase();
                nodes.push(AstNode::InlineInstruction(instruction));
                i += 2;  // Skip both INLINE and the instruction name
            }
            _ => {
                // Try to parse as number, otherwise it's a word
                if let Ok(num) = token.parse::<i64>() {
                    nodes.push(AstNode::PushNumber(num));
                } else {
                    // Store word names in uppercase for case-insensitive lookup
                    nodes.push(AstNode::CallWord(token_upper.clone()));
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
        let token_upper = token.to_uppercase();
        match token_upper.as_str() {
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
        let token_upper = token.to_uppercase();
        match token_upper.as_str() {
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
        let token_upper = token.to_uppercase();
        match token_upper.as_str() {
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

/// Strip comments from a line of Forth code
/// Handles both backslash comments (\) and parenthesis comments ( )
pub fn strip_comments(input: &str) -> String {
    // First, strip backslash comments (everything after \)
    let line = if let Some(pos) = input.find('\\') {
        &input[..pos]
    } else {
        input
    };

    // Then strip parenthesis comments ( ... )
    let mut result = String::new();
    let mut in_paren_comment = false;

    for ch in line.chars() {
        if ch == '(' {
            in_paren_comment = true;
        } else if ch == ')' {
            in_paren_comment = false;
            // Don't include the ) itself
            continue;
        }

        if !in_paren_comment {
            result.push(ch);
        }
    }

    result
}

pub fn load_file(
    filename: &str,
    stack: &mut Stack,
    dict: &mut Dictionary,
    loop_stack: &mut LoopStack,
    return_stack: &mut ReturnStack,
    memory: &mut Memory,
    no_jit: bool,
    dump_ir: bool,
    verify_ir: bool,
) -> Result<(), String> {
    let contents = fs::read_to_string(filename).map_err(|e| format!("Cannot read file: {}", e))?;

    // Process file as token stream to support multi-line definitions
    let mut processed = String::new();

    for line in contents.lines() {
        let line = line.trim();

        // Strip comments from this line
        let line = strip_comments(line);

        processed.push_str(&line);
        processed.push(' ');
    }

    // Now execute the entire file as one token stream
    execute_line(
        &processed,
        stack,
        dict,
        loop_stack,
        return_stack,
        memory,
        no_jit,
        dump_ir,
        verify_ir,
    )?;

    Ok(())
}

/// Attempt to JIT compile an AST to native code and store in dictionary
/// Returns true if successful, false otherwise
fn try_jit_compile(
    name: String,
    ast: &AstNode,
    dict: &mut dictionary::Dictionary,
    no_jit: bool,
    dump_ir: bool,
    verify_ir: bool,
) -> bool {
    if no_jit {
        return false;
    }

    use inkwell::context::Context;
    use llvm_codegen::{Compiler, register_jit_function};

    // Create LLVM context in a Box so it has a stable memory location
    let boxed_context = Box::new(Context::create());

    // SAFETY: Create a 'static reference to the boxed context.
    // This is safe because the Box will be stored in Dictionary.jit_contexts
    // and won't be dropped until Dictionary is dropped.
    let context_ref: &'static Context = unsafe { &*(boxed_context.as_ref() as *const Context) };

    let mut compiler = match Compiler::new(context_ref) {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Try to compile the AST
    match compiler.compile_word(&name, ast, dict) {
        Ok(jit_fn) => {
            // Dump IR if requested
            if dump_ir {
                println!("\n=== IR for {} ===", name);
                println!("{}", compiler.get_ir());
                println!("==================\n");
            }

            // Verify IR if requested
            if verify_ir {
                if let Err(e) = compiler.verify() {
                    eprintln!("IR verification failed for {}: {}", name, e);
                    return false;
                }
            }

            // Register the JIT function in the global registry
            // This allows other words being compiled later to call this word
            if let Err(e) = register_jit_function(name.clone(), jit_fn) {
                eprintln!("Failed to register JIT function {}: {}", name, e);
                return false;
            }

            // Add the JIT function to the dictionary
            // IMPORTANT: We must keep both context and compiler alive so the JIT code memory stays valid
            dict.add_jit_compiled_with_compiler(name, jit_fn, boxed_context, Box::new(compiler));
            true
        }
        Err(_) => false,
    }
}

pub fn execute_line(
    input: &str,
    stack: &mut Stack,
    dict: &mut Dictionary,
    loop_stack: &mut LoopStack,
    return_stack: &mut ReturnStack,
    memory: &mut Memory,
    no_jit: bool,
    dump_ir: bool,
    verify_ir: bool,
) -> Result<(), String> {
    // Strip comments from input
    let input = strip_comments(input);
    let tokens: Vec<&str> = input.split_whitespace().collect();

    if tokens.is_empty() {
        return Ok(());
    }

    // Process tokens sequentially, handling multiple definitions
    let mut i = 0;
    while i < tokens.len() {
        let token_upper = tokens[i].to_uppercase();
        if token_upper == "INCLUDE" {
            // INCLUDE <filename>
            if i + 1 >= tokens.len() {
                return Err("INCLUDE requires a filename".to_string());
            }

            let filename = tokens[i + 1];
            load_file(
                filename,
                stack,
                dict,
                loop_stack,
                return_stack,
                memory,
                no_jit,
                dump_ir,
                verify_ir,
            )?;
            i += 2;
        } else if token_upper == ":" {
            // Find matching semicolon for definition
            let mut semicolon_pos = None;
            for j in (i + 1)..tokens.len() {
                if tokens[j].to_uppercase() == ";" {
                    semicolon_pos = Some(j);
                    break;
                }
            }

            if let Some(end) = semicolon_pos {
                if end - i < 2 {
                    return Err("Invalid word definition".to_string());
                }

                // Store word names in uppercase for case-insensitive lookup
                let word_name = tokens[i + 1].to_uppercase();
                let word_tokens = &tokens[i + 2..end];

                let ast = parse_tokens(word_tokens)?;
                // Validate that all words in the AST exist (allow forward reference for recursion)
                ast.validate_with_name(dict, Some(&word_name))?;

                // Skip JIT compilation for word redefinitions to avoid memory leaks and registry collisions
                // When redefining, always use interpreted mode
                let is_redefinition = dict.has_word(&word_name);
                if !is_redefinition
                    && !try_jit_compile(word_name.clone(), &ast, dict, no_jit, dump_ir, verify_ir)
                {
                    dict.add_compiled(word_name, ast);
                } else if is_redefinition {
                    dict.add_compiled(word_name, ast);
                }
                i = end + 1;
            } else {
                return Err("Missing ; in word definition".to_string());
            }
        } else if token_upper == "VARIABLE" {
            // VARIABLE <name>
            if i + 1 >= tokens.len() {
                return Err("VARIABLE requires a name".to_string());
            }

            let var_name = tokens[i + 1].to_uppercase();
            let addr = memory.here();

            // Allocate 1 cell (8 bytes) for the variable
            memory.allot(8)?;

            // Create a word that pushes the variable's address
            let var_ast = AstNode::PushNumber(addr);
            dict.add_compiled(var_name, var_ast);
            i += 2;
        } else if token_upper == "CONSTANT" {
            // <value> CONSTANT <name>
            if i + 1 >= tokens.len() {
                return Err("CONSTANT requires a name".to_string());
            }

            // Pop value from stack
            let value = stack.pop(memory).ok_or("Stack underflow for CONSTANT")?;
            let const_name = tokens[i + 1].to_uppercase();

            // Create a word that pushes the constant value
            let const_ast = AstNode::PushNumber(value);
            dict.add_compiled(const_name, const_ast);
            i += 2;
        } else if token_upper == "CREATE" {
            // CREATE <name>
            if i + 1 >= tokens.len() {
                return Err("CREATE requires a name".to_string());
            }

            let create_name = tokens[i + 1].to_uppercase();
            let addr = memory.here();

            // Create a word that pushes the data address
            // User will typically follow with ALLOT to allocate space
            let create_ast = AstNode::PushNumber(addr);
            dict.add_compiled(create_name, create_ast);
            i += 2;
        } else if token_upper == "INCLUDED" {
            // INCLUDED ( addr len -- )
            // Takes filename from stack and loads the file
            let len = stack
                .pop(memory)
                .ok_or("Stack underflow for INCLUDED (length)")?;
            let addr = stack
                .pop(memory)
                .ok_or("Stack underflow for INCLUDED (address)")?;

            // Read the filename from memory
            let mut filename_bytes = Vec::new();
            for offset in 0..len {
                let byte = memory.fetch_byte((addr + offset) as usize)?;
                filename_bytes.push(byte as u8);
            }

            let filename =
                String::from_utf8(filename_bytes).map_err(|_| "Invalid UTF-8 in filename")?;

            // Load the file
            load_file(
                &filename,
                stack,
                dict,
                loop_stack,
                return_stack,
                memory,
                no_jit,
                dump_ir,
                verify_ir,
            )?;
            i += 1;
        } else {
            // Collect tokens until we hit : or INCLUDE or INCLUDED or VARIABLE or CONSTANT or CREATE or end
            let mut exec_tokens = Vec::new();
            while i < tokens.len() {
                let check_upper = tokens[i].to_uppercase();
                if check_upper == ":"
                    || check_upper == "INCLUDE"
                    || check_upper == "INCLUDED"
                    || check_upper == "VARIABLE"
                    || check_upper == "CONSTANT"
                    || check_upper == "CREATE"
                {
                    break;
                }
                exec_tokens.push(tokens[i]);
                i += 1;
            }

            if !exec_tokens.is_empty() {
                // Check for compile-only words (case-insensitive)
                if exec_tokens.iter().any(|&t| {
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
                        || upper == "INLINE"
                        || upper == ".\""
                }) {
                    return Err("Control flow and string words (IF/THEN/ELSE/BEGIN/UNTIL/WHILE/REPEAT/DO/LOOP/LEAVE/EXIT/INLINE/.\") are compile-only".to_string());
                }

                let ast = parse_tokens(&exec_tokens)?;
                ast.execute(stack, dict, loop_stack, return_stack, memory)?;
            }
        }
    }

    Ok(())
}

/// Helper function to process embedded stdlib file content
/// Strips comments and joins lines, similar to load_file
fn process_stdlib_content(content: &str) -> String {
    let mut processed = String::new();

    for line in content.lines() {
        let line = line.trim();
        // Strip comments from this line
        let line = strip_comments(line);
        processed.push_str(&line);
        processed.push(' ');
    }

    processed
}

/// Load standard library files embedded in the binary
/// This is called automatically on startup to make stdlib words available
pub fn load_stdlib(
    stack: &mut Stack,
    dict: &mut Dictionary,
    loop_stack: &mut LoopStack,
    return_stack: &mut ReturnStack,
    memory: &mut Memory,
    no_jit: bool,
    dump_ir: bool,
    verify_ir: bool,
) -> Result<(), String> {
    // Load core definitions
    let core_processed = process_stdlib_content(CORE_FTH);
    execute_line(
        &core_processed,
        stack,
        dict,
        loop_stack,
        return_stack,
        memory,
        no_jit,
        dump_ir,
        verify_ir,
    )?;

    // TODO: Load test framework - currently has issues with DEPTH in loops
    // let test_framework_processed = process_stdlib_content(TEST_FRAMEWORK_FTH);
    // execute_line(&test_framework_processed, stack, dict, loop_stack, return_stack, memory, no_jit, dump_ir, verify_ir)?;

    // TODO: Load test suite - temporarily disabled to debug segfault
    // let tests_processed = process_stdlib_content(TESTS_FTH);
    // execute_line(&tests_processed, stack, dict, loop_stack, return_stack, memory, no_jit, dump_ir, verify_ir)?;

    Ok(())
}
