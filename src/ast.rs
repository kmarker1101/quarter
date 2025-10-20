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
}

impl AstNode {
    pub fn execute(
        &self,
        stack: &mut Stack,
        dict: &crate::dictionary::Dictionary,
        loop_stack: &mut crate::LoopStack,
    ) -> Result<(), String> {
        match self {
            AstNode::PushNumber(n) => {
                stack.push(*n);
                Ok(())
            }
            AstNode::CallWord(name) => dict.execute_word(name, stack, loop_stack),
            AstNode::Sequence(nodes) => {
                for node in nodes {
                    node.execute(stack, dict, loop_stack)?;
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
                            node.execute(stack, dict, loop_stack)?;
                        }
                    } else if let Some(else_nodes) = else_branch {
                        for node in else_nodes {
                            node.execute(stack, dict, loop_stack)?;
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
                        node.execute(stack, dict, loop_stack)?;
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
                        node.execute(stack, dict, loop_stack)?;
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
                        node.execute(stack, dict, loop_stack)?;
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
                            match node.execute(stack, dict, loop_stack) {
                                Err(msg) if msg == "LEAVE" => {
                                    // LEAVE was called, exit loop early
                                    should_leave = true;
                                    break;
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
        }
    }
}
