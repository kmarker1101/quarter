use crate::stack::Stack;

#[derive(Debug, Clone)]
pub enum AstNode {
    PushNumber(i32),
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
        increment: i32,  // 1 for LOOP, variable for +LOOP
    },
    PrintString(String),
    Leave,
    Exit,
}

impl AstNode {
    /// Validate that all words referenced in this AST exist in the dictionary
    pub fn validate(&self, dict: &crate::dictionary::Dictionary) -> Result<(), String> {
        match self {
            AstNode::PushNumber(_) => Ok(()),
            AstNode::PrintString(_) => Ok(()),
            AstNode::Leave => Ok(()),
            AstNode::Exit => Ok(()),
            AstNode::CallWord(name) => {
                if dict.has_word(name) {
                    Ok(())
                } else {
                    Err(format!("Undefined word: {}", name))
                }
            }
            AstNode::Sequence(nodes) => {
                for node in nodes {
                    node.validate(dict)?;
                }
                Ok(())
            }
            AstNode::IfThenElse {
                then_branch,
                else_branch,
            } => {
                for node in then_branch {
                    node.validate(dict)?;
                }
                if let Some(else_nodes) = else_branch {
                    for node in else_nodes {
                        node.validate(dict)?;
                    }
                }
                Ok(())
            }
            AstNode::BeginUntil { body } => {
                for node in body {
                    node.validate(dict)?;
                }
                Ok(())
            }
            AstNode::BeginWhileRepeat { condition, body } => {
                for node in condition {
                    node.validate(dict)?;
                }
                for node in body {
                    node.validate(dict)?;
                }
                Ok(())
            }
            AstNode::DoLoop { body, .. } => {
                for node in body {
                    node.validate(dict)?;
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
    ) -> Result<(), String> {
        match self {
            AstNode::PushNumber(n) => {
                stack.push(*n);
                Ok(())
            }
            AstNode::CallWord(name) => dict.execute_word(name, stack, loop_stack, return_stack),
            AstNode::Sequence(nodes) => {
                for node in nodes {
                    match node.execute(stack, dict, loop_stack, return_stack) {
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
                if let Some(condition) = stack.pop() {
                    if condition != 0 {
                        // Non-zero is true in Forth
                        for node in then_branch {
                            match node.execute(stack, dict, loop_stack, return_stack) {
                                Err(msg) if msg == "EXIT" => return Err(msg),
                                Err(e) => return Err(e),
                                Ok(()) => {}
                            }
                        }
                    } else if let Some(else_nodes) = else_branch {
                        for node in else_nodes {
                            match node.execute(stack, dict, loop_stack, return_stack) {
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
                        match node.execute(stack, dict, loop_stack, return_stack) {
                            Err(msg) if msg == "EXIT" => return Err(msg),
                            Err(e) => return Err(e),
                            Ok(()) => {}
                        }
                    }
                    // Check condition (top of stack)
                    if let Some(condition) = stack.pop() {
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
                        match node.execute(stack, dict, loop_stack, return_stack) {
                            Err(msg) if msg == "EXIT" => return Err(msg),
                            Err(e) => return Err(e),
                            Ok(()) => {}
                        }
                    }
                    // Check if we should continue
                    if let Some(cond) = stack.pop() {
                        if cond == 0 {
                            break;  // Exit if false (0)
                        }
                    } else {
                        return Err("Stack underflow in WHILE".to_string());
                    }
                    // Execute body
                    for node in body {
                        match node.execute(stack, dict, loop_stack, return_stack) {
                            Err(msg) if msg == "EXIT" => return Err(msg),
                            Err(e) => return Err(e),
                            Ok(()) => {}
                        }
                    }
                }
                Ok(())
            }
            AstNode::DoLoop { body, increment } => {
                // Pop limit and start from stack ( limit start -- )
                if let (Some(start), Some(limit)) = (stack.pop(), stack.pop()) {
                    loop_stack.push_loop(start, limit);

                    let result = loop {
                        // Execute body
                        let mut should_leave = false;
                        for node in body {
                            match node.execute(stack, dict, loop_stack, return_stack) {
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
                            if let Some(n) = stack.pop() {
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
            AstNode::Leave => {
                // Signal to exit the loop
                Err("LEAVE".to_string())
            }
            AstNode::Exit => {
                // Signal to exit the current word
                Err("EXIT".to_string())
            }
        }
    }
}
