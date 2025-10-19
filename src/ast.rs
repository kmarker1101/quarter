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
}

impl AstNode {
    pub fn execute(
        &self,
        stack: &mut Stack,
        dict: &crate::dictionary::Dictionary,
    ) -> Result<(), String> {
        match self {
            AstNode::PushNumber(n) => {
                stack.push(*n);
                Ok(())
            }
            AstNode::CallWord(name) => dict.execute_word(name, stack),
            AstNode::Sequence(nodes) => {
                for node in nodes {
                    node.execute(stack, dict)?;
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
                            node.execute(stack, dict)?;
                        }
                    } else if let Some(else_nodes) = else_branch {
                        for node in else_nodes {
                            node.execute(stack, dict)?;
                        }
                    }
                    Ok(())
                } else {
                    Err("Stack underflow in IF".to_string())
                }
            }
        }
    }
}
