/// AST Handle-Based API for Forth
///
/// This module exposes AST node inspection to Forth code via integer handles.
/// Forth code can query AST structure and compile it using LLVM primitives.

use crate::ast::AstNode;
use std::cell::RefCell;
use std::collections::HashMap;

/// AST node handle type
pub type AstHandle = i32;

// Thread-local AST registry
thread_local! {
    static AST_REGISTRY: RefCell<AstRegistry> = RefCell::new(AstRegistry::new());
}

/// Registry for AST nodes
struct AstRegistry {
    next_id: i32,
    nodes: HashMap<AstHandle, AstNode>,
}

impl AstRegistry {
    fn new() -> Self {
        AstRegistry {
            next_id: 1,
            nodes: HashMap::new(),
        }
    }

    /// Register an AST node and return its handle
    fn register_node(&mut self, node: AstNode) -> AstHandle {
        let handle = self.next_id;
        self.next_id += 1;
        self.nodes.insert(handle, node);
        handle
    }

    /// Get node type as integer
    /// 1=PushNumber, 2=CallWord, 3=Sequence, 4=IfThenElse, 5=BeginUntil,
    /// 6=BeginWhileRepeat, 7=DoLoop, 8=PrintString, 9=StackString, 10=Leave, 11=Exit, 12=InlineInstruction
    fn get_node_type(&self, handle: AstHandle) -> Result<i32, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        Ok(match node {
            AstNode::PushNumber(_) => 1,
            AstNode::CallWord(_) => 2,
            AstNode::Sequence(_) => 3,
            AstNode::IfThenElse { .. } => 4,
            AstNode::BeginUntil { .. } => 5,
            AstNode::BeginWhileRepeat { .. } => 6,
            AstNode::DoLoop { .. } => 7,
            AstNode::PrintString(_) => 8,
            AstNode::StackString(_) => 9,
            AstNode::Leave => 10,
            AstNode::Exit => 11,
            AstNode::InlineInstruction(_) => 12,
        })
    }

    /// Get number value from PushNumber node
    fn get_number(&self, handle: AstHandle) -> Result<i32, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        match node {
            AstNode::PushNumber(n) => Ok(*n),
            _ => Err(format!("AST node is not a PushNumber")),
        }
    }

    /// Get word name from CallWord node (stores in memory at given address)
    /// Returns length of string
    fn get_word_name(&self, handle: AstHandle, memory: &mut crate::Memory, addr: usize) -> Result<i32, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        match node {
            AstNode::CallWord(name) => {
                // Store string bytes in memory
                for (i, byte) in name.as_bytes().iter().enumerate() {
                    memory.store_byte(addr + i, *byte as i32)
                        .map_err(|e| format!("Failed to store string: {}", e))?;
                }
                Ok(name.len() as i32)
            }
            _ => Err(format!("AST node is not a CallWord")),
        }
    }

    /// Get instruction name from InlineInstruction node (stores in memory at given address)
    /// Returns length of string
    fn get_inline_instruction(&self, handle: AstHandle, memory: &mut crate::Memory, addr: usize) -> Result<i32, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        match node {
            AstNode::InlineInstruction(instruction) => {
                // Store string bytes in memory
                for (i, byte) in instruction.as_bytes().iter().enumerate() {
                    memory.store_byte(addr + i, *byte as i32)
                        .map_err(|e| format!("Failed to store string: {}", e))?;
                }
                Ok(instruction.len() as i32)
            }
            _ => Err(format!("AST node is not an InlineInstruction")),
        }
    }

    /// Get string value from PrintString or StackString node
    fn get_string(&self, handle: AstHandle, memory: &mut crate::Memory, addr: usize) -> Result<i32, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        let string = match node {
            AstNode::PrintString(s) => s,
            AstNode::StackString(s) => s,
            _ => return Err(format!("AST node is not a PrintString or StackString")),
        };

        // Store string bytes in memory
        for (i, byte) in string.as_bytes().iter().enumerate() {
            memory.store_byte(addr + i, *byte as i32)
                .map_err(|e| format!("Failed to store string: {}", e))?;
        }
        Ok(string.len() as i32)
    }

    /// Get number of children in a Sequence node
    fn get_sequence_length(&self, handle: AstHandle) -> Result<i32, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        match node {
            AstNode::Sequence(nodes) => Ok(nodes.len() as i32),
            _ => Err(format!("AST node is not a Sequence")),
        }
    }

    /// Get nth child from Sequence node (0-indexed)
    fn get_sequence_child(&mut self, handle: AstHandle, index: i32) -> Result<AstHandle, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        match node {
            AstNode::Sequence(nodes) => {
                if index < 0 || index >= nodes.len() as i32 {
                    return Err(format!("Sequence index out of bounds: {}", index));
                }
                let child = nodes[index as usize].clone();
                Ok(self.register_node(child))
            }
            _ => Err(format!("AST node is not a Sequence")),
        }
    }

    /// Get then branch from IfThenElse (returns Sequence handle)
    fn get_if_then_branch(&mut self, handle: AstHandle) -> Result<AstHandle, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        match node {
            AstNode::IfThenElse { then_branch, .. } => {
                let seq = AstNode::Sequence(then_branch.clone());
                Ok(self.register_node(seq))
            }
            _ => Err(format!("AST node is not an IfThenElse")),
        }
    }

    /// Get else branch from IfThenElse (returns Sequence handle or 0 if no else)
    fn get_if_else_branch(&mut self, handle: AstHandle) -> Result<AstHandle, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        match node {
            AstNode::IfThenElse { else_branch, .. } => {
                match else_branch {
                    Some(nodes) => {
                        let seq = AstNode::Sequence(nodes.clone());
                        Ok(self.register_node(seq))
                    }
                    None => Ok(0), // 0 indicates no else branch
                }
            }
            _ => Err(format!("AST node is not an IfThenElse")),
        }
    }

    /// Get loop body (for BeginUntil, BeginWhileRepeat, DoLoop)
    fn get_loop_body(&mut self, handle: AstHandle) -> Result<AstHandle, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        let body = match node {
            AstNode::BeginUntil { body } => body,
            AstNode::BeginWhileRepeat { body, .. } => body,
            AstNode::DoLoop { body, .. } => body,
            _ => return Err(format!("AST node is not a loop")),
        };

        let seq = AstNode::Sequence(body.clone());
        Ok(self.register_node(seq))
    }

    /// Get loop condition (for BeginWhileRepeat)
    fn get_loop_condition(&mut self, handle: AstHandle) -> Result<AstHandle, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        match node {
            AstNode::BeginWhileRepeat { condition, .. } => {
                let seq = AstNode::Sequence(condition.clone());
                Ok(self.register_node(seq))
            }
            _ => Err(format!("AST node is not a BeginWhileRepeat")),
        }
    }

    /// Get loop increment (for DoLoop)
    fn get_loop_increment(&self, handle: AstHandle) -> Result<i32, String> {
        let node = self.nodes.get(&handle)
            .ok_or_else(|| format!("Invalid AST handle: {}", handle))?;

        match node {
            AstNode::DoLoop { increment, .. } => Ok(*increment),
            _ => Err(format!("AST node is not a DoLoop")),
        }
    }
}

// =============================================================================
// PUBLIC API FUNCTIONS
// =============================================================================

/// Register an AST node and return its handle
pub fn ast_register_node(node: AstNode) -> AstHandle {
    AST_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.register_node(node)
    })
}

/// Clear the AST registry (for testing)
pub fn ast_clear_registry() {
    AST_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.nodes.clear();
        registry.next_id = 1;
    })
}

/// Get AST node type
/// Stack: ( ast-handle -- type )
pub fn ast_get_type(handle: i32) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let registry = cell.borrow();
        registry.get_node_type(handle)
    })
}

/// Get number from PushNumber node
/// Stack: ( ast-handle -- number )
pub fn ast_get_number(handle: i32) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let registry = cell.borrow();
        registry.get_number(handle)
    })
}

/// Get word name from CallWord node
/// Stack: ( ast-handle addr -- length )
pub fn ast_get_word_name(handle: i32, memory: &mut crate::Memory, addr: usize) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let registry = cell.borrow();
        registry.get_word_name(handle, memory, addr)
    })
}

/// Get instruction name from InlineInstruction
/// Stack: ( ast-handle addr -- length )
pub fn ast_get_inline_instruction(handle: i32, memory: &mut crate::Memory, addr: usize) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let registry = cell.borrow();
        registry.get_inline_instruction(handle, memory, addr)
    })
}

/// Get string from PrintString or StackString
/// Stack: ( ast-handle addr -- length )
pub fn ast_get_string(handle: i32, memory: &mut crate::Memory, addr: usize) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let registry = cell.borrow();
        registry.get_string(handle, memory, addr)
    })
}

/// Get sequence length
/// Stack: ( ast-handle -- length )
pub fn ast_get_sequence_length(handle: i32) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let registry = cell.borrow();
        registry.get_sequence_length(handle)
    })
}

/// Get sequence child
/// Stack: ( ast-handle index -- child-handle )
pub fn ast_get_sequence_child(handle: i32, index: i32) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.get_sequence_child(handle, index)
    })
}

/// Get IF then branch
/// Stack: ( ast-handle -- then-handle )
pub fn ast_get_if_then(handle: i32) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.get_if_then_branch(handle)
    })
}

/// Get IF else branch
/// Stack: ( ast-handle -- else-handle-or-0 )
pub fn ast_get_if_else(handle: i32) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.get_if_else_branch(handle)
    })
}

/// Get loop body
/// Stack: ( ast-handle -- body-handle )
pub fn ast_get_loop_body(handle: i32) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.get_loop_body(handle)
    })
}

/// Get loop condition (BeginWhileRepeat only)
/// Stack: ( ast-handle -- condition-handle )
pub fn ast_get_loop_condition(handle: i32) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.get_loop_condition(handle)
    })
}

/// Get loop increment (DoLoop only)
/// Stack: ( ast-handle -- increment )
pub fn ast_get_loop_increment(handle: i32) -> Result<i32, String> {
    AST_REGISTRY.with(|cell| {
        let registry = cell.borrow();
        registry.get_loop_increment(handle)
    })
}
