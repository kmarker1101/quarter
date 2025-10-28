use crate::stack::Stack;

#[derive(Debug, Clone)]
pub enum AstNode {
    PushNumber(i64),
    CallWord(String),
    Sequence(Vec<AstNode>),
    IfThenElse {
        then_branch: Vec<AstNode>,
        else_branch: Option<Vec<AstNode>>,
    },
    BeginUntil {
        body: Vec<AstNode>,
    },
    BeginWhileRepeat {
        condition: Vec<AstNode>,
        body: Vec<AstNode>,
    },
    DoLoop {
        body: Vec<AstNode>,
        increment: i64,  // 1 for LOOP, variable for +LOOP
        conditional: bool,  // true for ?DO (skip if start >= limit), false for DO
    },
    PrintString(String),
    StackString(String),  // S" - push address and length
    Leave,
    Exit,
    Unloop,  // Discard loop parameters (used before EXIT when exiting from within a loop)
    Execute,  // EXECUTE - takes xt from stack and executes the word
    InlineInstruction(String),  // INLINE directive - maps to LLVM instruction (e.g., "LLVM-ADD")
    TickLiteral(String),  // ['] - compile-only, stores word name, pushes xt at runtime
}

impl AstNode {
    /// Validate that all words referenced in this AST exist in the dictionary
    pub fn validate(&self, dict: &crate::dictionary::Dictionary) -> Result<(), String> {
        self.validate_with_name(dict, None)
    }

    pub fn validate_with_name(&self, dict: &crate::dictionary::Dictionary, defining_word: Option<&str>) -> Result<(), String> {
        match self {
            AstNode::PushNumber(_) => Ok(()),
            AstNode::PrintString(_) => Ok(()),
            AstNode::StackString(_) => Ok(()),
            AstNode::Leave => Ok(()),
            AstNode::Exit => Ok(()),
            AstNode::Unloop => Ok(()),
            AstNode::Execute => Ok(()),  // Execute resolves word at runtime, no compile-time validation
            AstNode::InlineInstruction(_) => Ok(()),  // Inline instructions are validated at JIT time
            AstNode::TickLiteral(name) => {
                // ['] validates word exists at compile time
                if dict.has_word(name) {
                    Ok(())
                } else {
                    Err(format!("Undefined word in [']:  {}", name))
                }
            }
            AstNode::CallWord(name) => {
                // Allow forward reference if this is the word being defined (for recursion)
                if let Some(def_name) = defining_word
                    && name.to_uppercase() == def_name.to_uppercase() {
                        return Ok(());
                    }
                if dict.has_word(name) {
                    Ok(())
                } else {
                    Err(format!("Undefined word: {}", name))
                }
            }
            AstNode::Sequence(nodes) => {
                for node in nodes {
                    node.validate_with_name(dict, defining_word)?;
                }
                Ok(())
            }
            AstNode::IfThenElse {
                then_branch,
                else_branch,
            } => {
                for node in then_branch {
                    node.validate_with_name(dict, defining_word)?;
                }
                if let Some(else_nodes) = else_branch {
                    for node in else_nodes {
                        node.validate_with_name(dict, defining_word)?;
                    }
                }
                Ok(())
            }
            AstNode::BeginUntil { body } => {
                for node in body {
                    node.validate_with_name(dict, defining_word)?;
                }
                Ok(())
            }
            AstNode::BeginWhileRepeat { condition, body } => {
                for node in condition {
                    node.validate_with_name(dict, defining_word)?;
                }
                for node in body {
                    node.validate_with_name(dict, defining_word)?;
                }
                Ok(())
            }
            AstNode::DoLoop { body, .. } => {
                for node in body {
                    node.validate_with_name(dict, defining_word)?;
                }
                Ok(())
            }
        }
    }

    pub fn execute(
        &self,
        stack: &mut Stack,
        dict: &crate::dictionary::Dictionary,
        loop_stack: &mut crate::LoopStack,
        return_stack: &mut crate::ReturnStack,
        memory: &mut crate::Memory,
    ) -> Result<(), String> {
        match self {
            AstNode::PushNumber(n) => {
                stack.push(*n, memory);
                Ok(())
            }
            AstNode::CallWord(name) => dict.execute_word(name, stack, loop_stack, return_stack, memory),
            AstNode::Sequence(nodes) => {
                for node in nodes {
                    match node.execute(stack, dict, loop_stack, return_stack, memory) {
                        Err(msg) if msg == "EXIT" => {
                            // EXIT was called, stop processing and return success
                            return Ok(());
                        }
                        Err(e) => return Err(e),
                        Ok(()) => {}
                    }
                }
                Ok(())
            }
            AstNode::IfThenElse {
                then_branch,
                else_branch,
            } => {
                // Pop the condition from the stack
                if let Some(condition) = stack.pop(memory) {
                    if condition != 0 {
                        // Non-zero is true in Forth
                        for node in then_branch {
                            match node.execute(stack, dict, loop_stack, return_stack, memory) {
                                Err(msg) if msg == "EXIT" => return Err(msg),
                                Err(e) => return Err(e),
                                Ok(()) => {}
                            }
                        }
                    } else if let Some(else_nodes) = else_branch {
                        for node in else_nodes {
                            match node.execute(stack, dict, loop_stack, return_stack, memory) {
                                Err(msg) if msg == "EXIT" => return Err(msg),
                                Err(e) => return Err(e),
                                Ok(()) => {}
                            }
                        }
                    }
                    Ok(())
                } else {
                    Err("Stack underflow in IF".to_string())
                }
            }
            AstNode::BeginUntil { body } => {
                loop {
                    // Execute body
                    for node in body {
                        match node.execute(stack, dict, loop_stack, return_stack, memory) {
                            Err(msg) if msg == "EXIT" => return Err(msg),
                            Err(e) => return Err(e),
                            Ok(()) => {}
                        }
                    }
                    // Check condition (top of stack)
                    if let Some(condition) = stack.pop(memory) {
                        if condition != 0 {
                            break;  // Exit if true (-1)
                        }
                    } else {
                        return Err("Stack underflow in UNTIL".to_string());
                    }
                }
                Ok(())
            }
            AstNode::BeginWhileRepeat { condition, body } => {
                loop {
                    // Evaluate condition
                    for node in condition {
                        match node.execute(stack, dict, loop_stack, return_stack, memory) {
                            Err(msg) if msg == "EXIT" => return Err(msg),
                            Err(e) => return Err(e),
                            Ok(()) => {}
                        }
                    }
                    // Check if we should continue
                    if let Some(cond) = stack.pop(memory) {
                        if cond == 0 {
                            break;  // Exit if false (0)
                        }
                    } else {
                        return Err("Stack underflow in WHILE".to_string());
                    }
                    // Execute body
                    for node in body {
                        match node.execute(stack, dict, loop_stack, return_stack, memory) {
                            Err(msg) if msg == "EXIT" => return Err(msg),
                            Err(e) => return Err(e),
                            Ok(()) => {}
                        }
                    }
                }
                Ok(())
            }
            AstNode::DoLoop { body, increment, conditional: _ } => {
                // Pop limit and start from stack ( limit start -- )
                if let (Some(start), Some(limit)) = (stack.pop(memory), stack.pop(memory)) {
                    // Both DO and ?DO skip if start >= limit
                    // (?DO explicitly documents this behavior, DO matches for safety)
                    if start >= limit {
                        // Don't execute - loop would run 0 times
                        return Ok(());
                    }

                    loop_stack.push_loop(start, limit);

                    let result = loop {
                        // Execute body
                        let mut should_leave = false;
                        for node in body {
                            match node.execute(stack, dict, loop_stack, return_stack, memory) {
                                Err(msg) if msg == "LEAVE" => {
                                    // LEAVE was called, exit loop early
                                    should_leave = true;
                                    break;
                                }
                                Err(msg) if msg == "EXIT" => {
                                    // EXIT was called, exit loop AND word
                                    loop_stack.pop_loop();
                                    return Err(msg);
                                }
                                Err(e) => {
                                    loop_stack.pop_loop();
                                    return Err(e);
                                }
                                Ok(()) => {}
                            }
                        }

                        if should_leave {
                            break Ok(());
                        }

                        // Get increment value
                        let inc = if *increment == 0 {
                            // +LOOP: pop increment from stack
                            if let Some(n) = stack.pop(memory) {
                                n
                            } else {
                                loop_stack.pop_loop();
                                return Err("Stack underflow in +LOOP".to_string());
                            }
                        } else {
                            // LOOP: use fixed increment
                            *increment
                        };

                        // Increment and check if done
                        if !loop_stack.increment(inc) {
                            break Ok(());
                        }
                    };

                    loop_stack.pop_loop();
                    result
                } else {
                    Err("Stack underflow in DO".to_string())
                }
            }
            AstNode::PrintString(s) => {
                print!("{}", s);
                Ok(())
            }
            AstNode::StackString(s) => {
                // S" - Store string in memory and push address and length
                let addr = memory.here();
                let bytes = s.as_bytes();
                let len = bytes.len() as i64;

                // Store each byte in memory
                for (i, &byte) in bytes.iter().enumerate() {
                    memory.store_byte((addr as usize) + i, byte as i64)?;
                }

                // Advance HERE by string length
                memory.allot(len)?;

                // Push address and length onto stack
                stack.push(addr, memory);
                stack.push(len, memory);
                Ok(())
            }
            AstNode::Leave => {
                // Signal to exit the loop
                Err("LEAVE".to_string())
            }
            AstNode::Exit => {
                // Signal to exit the current word
                Err("EXIT".to_string())
            }
            AstNode::Unloop => {
                // Pop loop control parameters from loop stack
                // Used when exiting from within a loop (before EXIT)
                loop_stack.pop_loop();
                Ok(())
            }
            AstNode::Execute => {
                // EXECUTE ( xt -- )
                // Execute word from execution token
                // xt is the address of a counted string (length byte + characters)
                let xt = stack.pop(memory).ok_or("Stack underflow for EXECUTE")?;
                let addr = xt as usize;

                // Read the length byte
                let len = memory.fetch_byte(addr)? as usize;

                // Read the word name from memory
                let mut word_name = String::with_capacity(len);
                for i in 0..len {
                    let byte = memory.fetch_byte(addr + 1 + i)? as u8;
                    word_name.push(byte as char);
                }

                // Execute the word
                dict.execute_word(&word_name, stack, loop_stack, return_stack, memory)?;
                Ok(())
            }
            AstNode::InlineInstruction(instruction) => {
                // Inline instructions can only be executed in JIT-compiled code
                Err(format!("Inline instruction {} can only be used in JIT-compiled words", instruction))
            }
            AstNode::TickLiteral(word_name) => {
                // ['] ( -- xt )
                // Store word name as counted string at HERE, push xt (address)
                let xt_addr = memory.here();
                let name_bytes = word_name.as_bytes();

                // Store length byte
                memory.store_byte(xt_addr as usize, name_bytes.len() as i64)?;

                // Store character bytes
                for (offset, &byte) in name_bytes.iter().enumerate() {
                    memory.store_byte(xt_addr as usize + 1 + offset, byte as i64)?;
                }

                // Advance HERE
                memory.allot((1 + name_bytes.len()) as i64)?;

                // Push xt (address of counted string) onto stack
                stack.push(xt_addr, memory);
                Ok(())
            }
        }
    }

    /// Execute with tail call optimization check
    /// Returns Ok(true) if a tail call was detected (should loop back)
    /// Returns Ok(false) if execution completed normally
    pub fn execute_with_tco_check(
        &self,
        stack: &mut Stack,
        dict: &crate::dictionary::Dictionary,
        loop_stack: &mut crate::LoopStack,
        return_stack: &mut crate::ReturnStack,
        memory: &mut crate::Memory,
        word_name: &str,
    ) -> Result<bool, String> {
        // Execute all nodes except the last one normally
        match self {
            AstNode::Sequence(nodes) => {
                for (i, node) in nodes.iter().enumerate() {
                    let is_last = i == nodes.len() - 1;

                    if is_last {
                        // Last node - check if it's a tail call
                        if let AstNode::CallWord(name) = node
                            && name.to_uppercase() == word_name.to_uppercase() {
                                // Tail call detected! Signal to loop back WITHOUT executing
                                return Ok(true);
                            }
                        // Last node might be IfThenElse with tail calls
                        if let AstNode::IfThenElse { then_branch, else_branch } = node {
                            // Check if either branch has tail calls
                            let has_tail_call =
                                then_branch.last().map(|n| matches!(n, AstNode::CallWord(name) if name.to_uppercase() == word_name.to_uppercase())).unwrap_or(false)
                                || else_branch.as_ref().and_then(|b| b.last()).map(|n| matches!(n, AstNode::CallWord(name) if name.to_uppercase() == word_name.to_uppercase())).unwrap_or(false);

                            if has_tail_call {
                                // Execute the IfThenElse with TCO check
                                return node.execute_with_tco_check(stack, dict, loop_stack, return_stack, memory, word_name);
                            }
                        }
                        // Last node but not a tail call, fall through to execute
                    }

                    // Execute node normally (skipped if we returned above)
                    match node.execute(stack, dict, loop_stack, return_stack, memory) {
                        Err(msg) if msg == "EXIT" => {
                            return Err(msg);
                        }
                        Err(e) => return Err(e),
                        Ok(()) => {}
                    }
                }
                Ok(false)  // Normal completion
            }
            AstNode::IfThenElse { then_branch, else_branch } => {
                // Pop the condition from the stack
                if let Some(condition) = stack.pop(memory) {
                    if condition != 0 {
                        // Execute then branch
                        for (i, node) in then_branch.iter().enumerate() {
                            let is_last = i == then_branch.len() - 1;
                            if is_last
                                && let AstNode::CallWord(name) = node
                                && name.to_uppercase() == word_name.to_uppercase() {
                                    return Ok(true);  // Tail call
                                }

                            match node.execute(stack, dict, loop_stack, return_stack, memory) {
                                Err(msg) if msg == "EXIT" => return Err(msg),
                                Err(e) => return Err(e),
                                Ok(()) => {}
                            }
                        }
                    } else if let Some(else_nodes) = else_branch {
                        // Execute else branch
                        for (i, node) in else_nodes.iter().enumerate() {
                            let is_last = i == else_nodes.len() - 1;
                            if is_last
                                && let AstNode::CallWord(name) = node
                                && name.to_uppercase() == word_name.to_uppercase() {
                                    return Ok(true);  // Tail call
                                }

                            match node.execute(stack, dict, loop_stack, return_stack, memory) {
                                Err(msg) if msg == "EXIT" => return Err(msg),
                                Err(e) => return Err(e),
                                Ok(()) => {}
                            }
                        }
                    }
                    Ok(false)
                } else {
                    Err("Stack underflow in IF".to_string())
                }
            }
            AstNode::CallWord(name) => {
                // Direct call - check if it's a tail call
                if name.to_uppercase() == word_name.to_uppercase() {
                    Ok(true)  // Tail call
                } else {
                    // Regular call, execute normally
                    dict.execute_word(name, stack, loop_stack, return_stack, memory)?;
                    Ok(false)
                }
            }
            _ => {
                // For other node types, just execute normally
                self.execute(stack, dict, loop_stack, return_stack, memory)?;
                Ok(false)
            }
        }
    }
}
