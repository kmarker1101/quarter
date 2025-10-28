pub mod ast;
pub mod ast_forth;
pub mod dictionary;
pub mod llvm_forth;
pub mod stack;
pub mod words;

pub use ast::AstNode;
pub use dictionary::Dictionary;
pub use stack::Stack;

use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::RefCell;
use std::collections::HashSet;

/// Track whether the Forth compiler has been loaded
static FORTH_COMPILER_LOADED: AtomicBool = AtomicBool::new(false);

// ============================================================================
// Configuration and Options Structs
// ============================================================================

/// Compiler configuration flags
#[derive(Clone, Copy, Debug)]
pub struct CompilerConfig {
    pub no_jit: bool,
    pub dump_ir: bool,
    pub verify_ir: bool,
}

impl CompilerConfig {
    pub fn new(no_jit: bool, dump_ir: bool, verify_ir: bool) -> Self {
        Self { no_jit, dump_ir, verify_ir }
    }
}

/// Execution options for file loading and compilation
#[derive(Clone, Copy, Debug)]
pub struct ExecutionOptions {
    pub use_forth_compiler: bool,
    pub define_only: bool,
}

impl ExecutionOptions {
    pub fn new(use_forth_compiler: bool, define_only: bool) -> Self {
        Self { use_forth_compiler, define_only }
    }
}

/// Runtime context grouping all mutable state
/// Used to reduce parameter counts in functions
pub struct RuntimeContext<'a> {
    pub stack: &'a mut Stack,
    pub dict: &'a mut Dictionary,
    pub loop_stack: &'a mut LoopStack,
    pub return_stack: &'a mut ReturnStack,
    pub memory: &'a mut Memory,
}

impl<'a> RuntimeContext<'a> {
    pub fn new(
        stack: &'a mut Stack,
        dict: &'a mut Dictionary,
        loop_stack: &'a mut LoopStack,
        return_stack: &'a mut ReturnStack,
        memory: &'a mut Memory,
    ) -> Self {
        Self { stack, dict, loop_stack, return_stack, memory }
    }
}

// ============================================================================
// Global Execution Context (for EVALUATE and self-hosting REPL)
// ============================================================================

/// Catch frame for exception handling
#[derive(Clone, Debug)]
pub struct CatchFrame {
    pub stack_depth: usize,
    pub return_stack_depth: usize,
}

/// Global execution context accessible from primitive words
pub struct ExecutionContext {
    pub stack: Stack,
    pub dict: Dictionary,
    pub loop_stack: LoopStack,
    pub return_stack: ReturnStack,
    pub memory: Memory,
    pub included_files: HashSet<String>,
    pub config: CompilerConfig,
    // Raw pointers for re-entrant access (used by EVALUATE)
    pub dict_ptr: *mut Dictionary,
    pub loop_stack_ptr: *mut LoopStack,
    pub return_stack_ptr: *mut ReturnStack,
    pub memory_ptr: *mut Memory,
    // Exception handling
    pub catch_stack: Vec<CatchFrame>,
}

thread_local! {
    /// Thread-local execution context for EVALUATE and REPL
    static EXECUTION_CONTEXT: RefCell<Option<ExecutionContext>> = const { RefCell::new(None) };

    /// Raw pointers for re-entrant access (stored separately to avoid RefCell conflicts)
    static REENTRANT_POINTERS: std::cell::Cell<(*mut Dictionary, *mut LoopStack, *mut ReturnStack, *mut Memory, *mut HashSet<String>)> =
        const { std::cell::Cell::new((std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut())) };

    /// Config flags for re-entrant access
    static REENTRANT_CONFIG: std::cell::Cell<(bool, bool, bool)> = const { std::cell::Cell::new((false, false, false)) };
}

/// Initialize the global execution context
pub fn init_execution_context(
    stack: Stack,
    dict: Dictionary,
    loop_stack: LoopStack,
    return_stack: ReturnStack,
    memory: Memory,
    included_files: HashSet<String>,
    config: CompilerConfig,
) {
    EXECUTION_CONTEXT.with(|ctx| {
        let context = ExecutionContext {
            stack,
            dict,
            loop_stack,
            return_stack,
            memory,
            included_files,
            config,
            dict_ptr: std::ptr::null_mut(),
            loop_stack_ptr: std::ptr::null_mut(),
            return_stack_ptr: std::ptr::null_mut(),
            memory_ptr: std::ptr::null_mut(),
            catch_stack: Vec::new(),
        };

        // Store context FIRST
        *ctx.borrow_mut() = Some(context);

        // NOW create pointers to the stored context
        if let Some(context_ref) = ctx.borrow_mut().as_mut() {
            let dict_ptr = &mut context_ref.dict as *mut Dictionary;
            let loop_stack_ptr = &mut context_ref.loop_stack as *mut LoopStack;
            let return_stack_ptr = &mut context_ref.return_stack as *mut ReturnStack;
            let memory_ptr = &mut context_ref.memory as *mut Memory;

            // Update the pointers in the context itself
            context_ref.dict_ptr = dict_ptr;
            context_ref.loop_stack_ptr = loop_stack_ptr;
            context_ref.return_stack_ptr = return_stack_ptr;
            context_ref.memory_ptr = memory_ptr;

            let included_files_ptr = &mut context_ref.included_files as *mut HashSet<String>;

            // Store pointers in separate thread-local for re-entrant access
            REENTRANT_POINTERS.with(|ptrs| {
                ptrs.set((dict_ptr, loop_stack_ptr, return_stack_ptr, memory_ptr, included_files_ptr));
            });

            // Store config flags
            REENTRANT_CONFIG.with(|cfg| {
                cfg.set((context_ref.config.no_jit, context_ref.config.dump_ir, context_ref.config.verify_ir));
            });
        }
    });
}

/// Access the global execution context
pub fn with_execution_context<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut ExecutionContext) -> R,
{
    EXECUTION_CONTEXT.with(|ctx| {
        ctx.borrow_mut().as_mut().map(f)
    })
}

/// Take the execution context out (for returning from REPL to main)
pub fn take_execution_context() -> Option<ExecutionContext> {
    EXECUTION_CONTEXT.with(|ctx| ctx.borrow_mut().take())
}

/// Get raw pointers for re-entrant access (used by EVALUATE and CATCH)
/// SAFETY: Caller must ensure pointers are valid and not used across re-entrant calls
pub fn get_reentrant_pointers() -> Option<(*mut Dictionary, *mut LoopStack, *mut ReturnStack, *mut Memory, *mut HashSet<String>)> {
    REENTRANT_POINTERS.with(|ptrs| {
        let (dict_ptr, loop_stack_ptr, return_stack_ptr, memory_ptr, included_files_ptr) = ptrs.get();

        // Check if pointers are null (not initialized)
        if dict_ptr.is_null() || loop_stack_ptr.is_null() || return_stack_ptr.is_null() || memory_ptr.is_null() || included_files_ptr.is_null() {
            None
        } else {
            Some((dict_ptr, loop_stack_ptr, return_stack_ptr, memory_ptr, included_files_ptr))
        }
    })
}

/// Get config flags for re-entrant access
pub fn get_reentrant_config() -> (bool, bool, bool) {
    REENTRANT_CONFIG.with(|cfg| cfg.get())
}

// Embedded standard library files
const CORE_FTH: &str = include_str!("../stdlib/core.fth");
#[allow(dead_code)] // TODO: Re-enable once DEPTH in loops is fixed
const TEST_FRAMEWORK_FTH: &str = include_str!("../stdlib/test-framework.fth");
#[allow(dead_code)] // TODO: Re-enable once test framework is fixed
/// Clear all global registries (for testing)
/// Call this between tests to avoid state pollution
pub fn clear_test_state() {
    ast_forth::ast_clear_registry();
}

// Loop stack for DO...LOOP counters
#[derive(Debug, Clone)]
pub struct LoopStack {
    stack: Vec<(i64, i64)>, // (index, limit) pairs
}

impl Default for LoopStack {
    fn default() -> Self {
        Self::new()
    }
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

impl Default for ReturnStack {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn depth(&self) -> usize {
        (self.rp - 0x010000) / 8
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

// Fixed memory location for dictionary pointer (8 bytes before user memory)
const DP_ADDR: usize = 0x01FFF8;

// Fixed memory location for BASE (numeric radix for I/O)
const BASE_ADDR: usize = 0x7FFFF8;

#[derive(Debug)]
pub struct Memory {
    bytes: Vec<u8>,
    dp: usize, // Dictionary pointer - tracks next allocation address
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub fn new() -> Self {
        let mut memory = Memory {
            bytes: vec![0; 8 * 1024 * 1024], // 8MB like gforth
            dp: 0x020000,                    // Start dictionary at beginning of user memory
        };
        // Sync dp to memory
        memory.sync_dp_to_memory();
        // Initialize BASE to default value (10 = decimal)
        memory.init_base();
        memory
    }

    // Sync dictionary pointer to fixed memory location
    fn sync_dp_to_memory(&mut self) {
        let bytes = (self.dp as i64).to_le_bytes();
        self.bytes[DP_ADDR] = bytes[0];
        self.bytes[DP_ADDR + 1] = bytes[1];
        self.bytes[DP_ADDR + 2] = bytes[2];
        self.bytes[DP_ADDR + 3] = bytes[3];
        self.bytes[DP_ADDR + 4] = bytes[4];
        self.bytes[DP_ADDR + 5] = bytes[5];
        self.bytes[DP_ADDR + 6] = bytes[6];
        self.bytes[DP_ADDR + 7] = bytes[7];
    }

    // Initialize BASE to default value (10 = decimal)
    fn init_base(&mut self) {
        let bytes = (10i64).to_le_bytes();
        self.bytes[BASE_ADDR] = bytes[0];
        self.bytes[BASE_ADDR + 1] = bytes[1];
        self.bytes[BASE_ADDR + 2] = bytes[2];
        self.bytes[BASE_ADDR + 3] = bytes[3];
        self.bytes[BASE_ADDR + 4] = bytes[4];
        self.bytes[BASE_ADDR + 5] = bytes[5];
        self.bytes[BASE_ADDR + 6] = bytes[6];
        self.bytes[BASE_ADDR + 7] = bytes[7];
    }

    // Read dictionary pointer from memory (for JIT)
    pub fn read_dp_from_memory(&self) -> usize {
        let bytes = [
            self.bytes[DP_ADDR],
            self.bytes[DP_ADDR + 1],
            self.bytes[DP_ADDR + 2],
            self.bytes[DP_ADDR + 3],
            self.bytes[DP_ADDR + 4],
            self.bytes[DP_ADDR + 5],
            self.bytes[DP_ADDR + 6],
            self.bytes[DP_ADDR + 7],
        ];
        i64::from_le_bytes(bytes) as usize
    }

    // HERE - return current dictionary pointer
    pub fn here(&self) -> i64 {
        self.dp as i64
    }

    // BASE - return address of numeric base variable
    pub fn base(&self) -> i64 {
        BASE_ADDR as i64
    }

    // ALLOT - allocate n bytes in dictionary space
    pub fn allot(&mut self, n: i64) -> Result<(), String> {
        let new_dp = (self.dp as i64 + n) as usize;
        if new_dp >= self.bytes.len() {
            return Err("Dictionary overflow".to_string());
        }
        self.dp = new_dp;
        // Sync to memory so JIT code can access it
        self.sync_dp_to_memory();
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

pub fn parse_tokens(tokens: &[&str], dict: &crate::Dictionary, current_word: Option<&str>) -> Result<AstNode, String> {
    let mut nodes = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];
        let token_upper = token.to_uppercase();

        match token_upper.as_str() {
            "RECURSE" => {
                // RECURSE - compile-only word for recursion
                if let Some(word_name) = current_word {
                    nodes.push(AstNode::CallWord(word_name.to_string()));
                    i += 1;
                } else {
                    return Err("RECURSE can only be used inside a word definition".to_string());
                }
            }
            ".\"" => {
                // Handle string literals: collect tokens until closing "
                let mut string_parts: Vec<String> = Vec::new();
                i += 1; // Skip past ."

                while i < tokens.len() {
                    let part = tokens[i];
                    if let Some(without_quote) = part.strip_suffix('"') {
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
            ".(" => {
                // Handle .( print-string: collect tokens until closing )
                let mut string_parts: Vec<String> = Vec::new();
                i += 1; // Skip past .(

                while i < tokens.len() {
                    let part = tokens[i];
                    if let Some(without_paren) = part.strip_suffix(')') {
                        // Found closing paren
                        if part == ")" {
                            // Just a closing paren - means there was a trailing space
                            // Add space to the last part if there is one
                            if !string_parts.is_empty() {
                                let last_idx = string_parts.len() - 1;
                                string_parts[last_idx].push(' ');
                            }
                        } else {
                            // Text followed by paren
                            if !without_paren.is_empty() {
                                string_parts.push(without_paren.to_string());
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
                    if let Some(without_quote) = part.strip_suffix('"') {
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
                    let body_ast = parse_tokens(body_tokens, dict, current_word)?;

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

                        let condition_ast = parse_tokens(condition_tokens, dict, current_word)?;
                        let body_ast = parse_tokens(body_tokens, dict, current_word)?;

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
            "DO" | "?DO" => {
                // Find matching LOOP or +LOOP
                let loop_pos = find_do_loop(&tokens[i + 1..])?;
                let loop_keyword = tokens[i + 1 + loop_pos];

                let body_tokens = &tokens[i + 1..i + 1 + loop_pos];
                let body_ast = parse_tokens(body_tokens, dict, current_word)?;

                let increment = if loop_keyword == "+LOOP" {
                    0 // Special marker for +LOOP (stack-based increment)
                } else {
                    1 // LOOP always increments by 1
                };

                let conditional = token_upper == "?DO";

                nodes.push(AstNode::DoLoop {
                    body: if let AstNode::Sequence(v) = body_ast {
                        v
                    } else {
                        vec![body_ast]
                    },
                    increment,
                    conditional,
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
                let then_branch = parse_tokens(then_tokens, dict, current_word)?;

                // Parse ELSE branch if it exists (from after ELSE to THEN)
                let else_branch = if let Some(else_pos) = else_start {
                    let else_tokens = &tokens[i + 1 + else_pos + 1..i + 1 + then_end];
                    Some(parse_tokens(else_tokens, dict, current_word)?)
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
            "UNLOOP" => {
                nodes.push(AstNode::Unloop);
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
                    // Check if word is a simple constant (VARIABLE or CONSTANT)
                    // If so, inline it to avoid JIT lookup errors
                    if let Some(word) = dict.get_word(&token_upper)
                        && let crate::dictionary::Word::Compiled(ast_node) = word
                        && let AstNode::PushNumber(value) = ast_node {
                            // Inline the constant value directly
                            nodes.push(AstNode::PushNumber(*value));
                            i += 1;
                            continue;
                        }

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
    let mut in_string = false; // Track if we're inside .( or ."

    for (i, &token) in tokens.iter().enumerate() {
        // Track if we're inside string literals to avoid matching keywords
        if token == ".(" || token == ".\"" {
            in_string = true;
            continue;
        }
        if in_string {
            if token.ends_with(')') || token.ends_with('"') {
                in_string = false;
            }
            continue;
        }

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
    let mut in_string = false; // Track if we're inside .( or ."

    for (i, &token) in tokens.iter().enumerate() {
        // Track if we're inside string literals to avoid matching keywords
        if token == ".(" || token == ".\"" {
            in_string = true;
            continue;
        }
        if in_string {
            if token.ends_with(')') || token.ends_with('"') {
                in_string = false;
            }
            continue;
        }

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
    let mut in_string = false;

    for (i, &token) in tokens.iter().enumerate() {
        // Track if we're inside string literals to avoid matching keywords
        if token == ".(" || token == ".\"" {
            in_string = true;
            continue;
        }
        if in_string {
            if token.ends_with(')') || token.ends_with('"') {
                in_string = false;
            }
            continue;
        }

        let token_upper = token.to_uppercase();
        match token_upper.as_str() {
            "DO" | "?DO" => depth += 1,
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
/// Preserves .( ... ) which is the print-string word, not a comment
pub fn strip_comments(input: &str) -> String {
    // First, strip backslash comments (everything after \)
    let line = if let Some(pos) = input.find('\\') {
        &input[..pos]
    } else {
        input
    };

    // Then strip parenthesis comments ( ... ) but preserve .( ... )
    let mut result = String::new();
    let mut in_paren_comment = false;
    let mut in_dot_paren = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if ch == '(' {
            // Only check for .( if we're not already in a comment
            // (to handle nested parentheses in stack comments like ( n -- fib(n) ))
            if !in_paren_comment {
                // Check if this is .( (print-string) or just ( (comment)
                // Look back to see if previous non-whitespace char is .
                let is_dot_paren_start = if i > 0 {
                    // Find the previous non-whitespace character
                    let mut j = i - 1;
                    let mut found_dot = false;
                    loop {
                        if chars[j] == '.' {
                            // Found . immediately before (, this is .(
                            found_dot = true;
                            break;
                        } else if chars[j].is_whitespace() {
                            // Skip whitespace
                            if j == 0 {
                                break;
                            }
                            j -= 1;
                        } else {
                            // Found some other character, not .(
                            break;
                        }
                    }
                    found_dot
                } else {
                    false
                };

                if is_dot_paren_start {
                    // Keep .( and its content
                    in_dot_paren = true;
                    result.push(ch);
                } else {
                    // Regular comment, start skipping
                    in_paren_comment = true;
                }
            }
            // If already in comment, just skip this ( character
        } else if ch == ')' {
            if in_paren_comment {
                // End of regular comment
                in_paren_comment = false;
                // Don't include the ) itself
                i += 1;
                continue;
            } else if in_dot_paren {
                // This is the closing ) of .(, keep it
                in_dot_paren = false;
                result.push(ch);
            }
            // Else: standalone ) outside any context - skip it
        } else if !in_paren_comment {
            result.push(ch);
        }

        i += 1;
    }

    result
}

pub fn load_file(
    filename: &str,
    ctx: &mut RuntimeContext,
    config: CompilerConfig,
    options: ExecutionOptions,
    included_files: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    // Check if file has already been included
    if included_files.contains(filename) {
        return Ok(());
    }

    // Mark this file as included
    included_files.insert(filename.to_string());

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
        ctx,
        config,
        options,
        included_files,
    )?;

    Ok(())
}


pub fn execute_line(
    input: &str,
    ctx: &mut RuntimeContext,
    config: CompilerConfig,
    options: ExecutionOptions,
    included_files: &mut std::collections::HashSet<String>,
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
            // INCLUDE always executes (not define-only) because it's used for dependencies
            let include_options = ExecutionOptions::new(options.use_forth_compiler, false);
            load_file(
                filename,
                ctx,
                config,
                include_options,
                included_files,
            )?;
            i += 2;
        } else if token_upper == ":" {
            // Find matching semicolon for definition
            let semicolon_pos = tokens[(i + 1)..]
                .iter()
                .position(|t| t.to_uppercase() == ";")
                .map(|pos| pos + i + 1);

            if let Some(end) = semicolon_pos {
                if end - i < 2 {
                    return Err("Invalid word definition".to_string());
                }

                // Store word names in uppercase for case-insensitive lookup
                let word_name = tokens[i + 1].to_uppercase();

                // Check if word is frozen (JIT-compiled) - if so, skip re-definition
                if ctx.dict.is_frozen(&word_name) {
                    i = end + 1;
                    continue;
                }

                let word_tokens = &tokens[i + 2..end];

                let ast = parse_tokens(word_tokens, ctx.dict, Some(&word_name))?;
                // Validate that all words in the AST exist (allow forward reference for recursion)
                ast.validate_with_name(ctx.dict, Some(&word_name))?;

                // Try JIT compilation if enabled
                let is_redefinition = ctx.dict.has_word(&word_name);
                if options.use_forth_compiler && !is_redefinition {
                    // Try to JIT compile - if successful, try_forth_compile will add it to dict
                    // If it fails, we fall back to interpreted mode below
                    if crate::try_forth_compile_word(word_name.clone(), &ast, ctx, config, included_files) {
                        // JIT compilation succeeded, word is already in dictionary
                        i = end + 1;
                        continue;
                    }
                }

                // Add word to dictionary in interpreted mode (fallback or default)
                ctx.dict.add_compiled(word_name, ast);
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
            let addr = ctx.memory.here();

            // Allocate 1 cell (8 bytes) for the variable
            ctx.memory.allot(8)?;

            // Create a word that pushes the variable's address
            let var_ast = AstNode::PushNumber(addr);
            ctx.dict.add_compiled(var_name, var_ast);
            i += 2;
        } else if token_upper == "CONSTANT" {
            // <value> CONSTANT <name>
            if i + 1 >= tokens.len() {
                return Err("CONSTANT requires a name".to_string());
            }

            // Pop value from stack
            let value = ctx.stack.pop(ctx.memory).ok_or("Stack underflow for CONSTANT")?;
            let const_name = tokens[i + 1].to_uppercase();

            // Create a word that pushes the constant value
            let const_ast = AstNode::PushNumber(value);
            ctx.dict.add_compiled(const_name, const_ast);
            i += 2;
        } else if token_upper == "CREATE" {
            // CREATE <name>
            if i + 1 >= tokens.len() {
                return Err("CREATE requires a name".to_string());
            }

            let create_name = tokens[i + 1].to_uppercase();
            let addr = ctx.memory.here();

            // Create a word that pushes the data address
            // User will typically follow with ALLOT to allocate space
            let create_ast = AstNode::PushNumber(addr);
            ctx.dict.add_compiled(create_name, create_ast);
            i += 2;
        } else if token_upper == "INCLUDED" {
            // INCLUDED ( addr len -- )
            // Takes filename from stack and loads the file
            let len = ctx.stack
                .pop(ctx.memory)
                .ok_or("Stack underflow for INCLUDED (length)")?;
            let addr = ctx.stack
                .pop(ctx.memory)
                .ok_or("Stack underflow for INCLUDED (address)")?;

            // Read the filename from memory
            let mut filename_bytes = Vec::new();
            for offset in 0..len {
                let byte = ctx.memory.fetch_byte((addr + offset) as usize)?;
                filename_bytes.push(byte as u8);
            }

            let filename =
                String::from_utf8(filename_bytes).map_err(|_| "Invalid UTF-8 in filename")?;

            // Load the file - INCLUDED always executes (not define-only)
            // because it's used for loading dependencies that need to run
            let include_options = ExecutionOptions::new(options.use_forth_compiler, false);
            load_file(
                &filename,
                ctx,
                config,
                include_options,
                included_files,
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
                // In define_only mode, still execute if:
                // 1. The last collected token is INCLUDE or INCLUDED
                // 2. The NEXT token (not yet collected) is INCLUDED
                // This handles both "INCLUDE file" and "S\" file\" INCLUDED" patterns
                let next_token_is_included = i < tokens.len() &&
                    tokens[i].to_uppercase() == "INCLUDED";

                let should_execute = !options.define_only ||
                    next_token_is_included ||
                    exec_tokens.last().map(|t| {
                        let upper = t.to_uppercase();
                        upper == "INCLUDED" || upper == "INCLUDE"
                    }).unwrap_or(false);

                if should_execute {
                    // Check for compile-only words (case-insensitive)
                    // Skip tokens inside S" strings to avoid false positives
                    let mut idx = 0;
                    let mut found_compile_only = false;
                    while idx < exec_tokens.len() {
                        let upper = exec_tokens[idx].to_uppercase();

                        // Skip S" string contents
                        if upper == "S\"" {
                            idx += 1;
                            // Skip until we find the closing "
                            while idx < exec_tokens.len() {
                                if exec_tokens[idx].ends_with('"') {
                                    idx += 1;
                                    break;
                                }
                                idx += 1;
                            }
                            continue;
                        }

                        // Skip ." string contents
                        if upper == ".\"" {
                            idx += 1;
                            // Skip until we find the closing "
                            while idx < exec_tokens.len() {
                                if exec_tokens[idx].ends_with('"') {
                                    idx += 1;
                                    break;
                                }
                                idx += 1;
                            }
                            continue;
                        }

                        // Check if this is a compile-only word
                        if upper == "IF"
                            || upper == "THEN"
                            || upper == "ELSE"
                            || upper == "BEGIN"
                            || upper == "UNTIL"
                            || upper == "WHILE"
                            || upper == "REPEAT"
                            || upper == "DO"
                            || upper == "?DO"
                            || upper == "LOOP"
                            || upper == "+LOOP"
                            || upper == "LEAVE"
                            || upper == "EXIT"
                            || upper == "UNLOOP"
                        {
                            found_compile_only = true;
                            break;
                        }

                        idx += 1;
                    }

                    if found_compile_only {
                        return Err("Control flow words (IF/THEN/ELSE/BEGIN/UNTIL/WHILE/REPEAT/DO/?DO/LOOP/LEAVE/EXIT/UNLOOP) are compile-only".to_string());
                    }

                    let ast = parse_tokens(&exec_tokens, ctx.dict, None)?;
                    ast.execute(ctx.stack, ctx.dict, ctx.loop_stack, ctx.return_stack, ctx.memory)?;
                }
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
    ctx: &mut RuntimeContext,
    config: CompilerConfig,
    options: ExecutionOptions,
    included_files: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    // Load core definitions
    let core_processed = process_stdlib_content(CORE_FTH);
    let stdlib_options = ExecutionOptions::new(options.use_forth_compiler, false);
    execute_line(
        &core_processed,
        ctx,
        config,
        stdlib_options,
        included_files,
    )?;

    // TODO: Load test framework - currently has issues with DEPTH in loops
    // let test_framework_processed = process_stdlib_content(TEST_FRAMEWORK_FTH);
    // execute_line(&test_framework_processed, stack, dict, loop_stack, return_stack, memory, no_jit, dump_ir, verify_ir)?;

    // TODO: Load test suite - temporarily disabled to debug segfault
    // let tests_processed = process_stdlib_content(TESTS_FTH);
    // execute_line(&tests_processed, stack, dict, loop_stack, return_stack, memory, no_jit, dump_ir, verify_ir)?;

    Ok(())
}

/// Batch compile all Word::Compiled entries in the dictionary to JIT
/// This creates one global LLVM module with all functions, then JITs them all at once
pub fn batch_compile_all_words(
    ctx: &mut RuntimeContext,
    config: CompilerConfig,
    included_files: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    // IMPORTANT: Get list of words to compile BEFORE loading the compiler
    // Otherwise we'll try to compile the compiler's internal words too!
    let all_words = ctx.dict.get_all_words();
    let mut words_to_compile: Vec<(String, AstNode)> = Vec::new();

    for (name, word) in all_words {
        if let crate::dictionary::Word::Compiled(ast) = word {
            words_to_compile.push((name, ast.clone()));
        }
    }

    // Load the Forth compiler if not already loaded (after capturing words to compile)
    if !FORTH_COMPILER_LOADED.load(Ordering::Relaxed) {
        // Load compiler
        let compiler_options = ExecutionOptions::new(false, false);
        if let Err(e) = load_file("stdlib/compiler.fth", ctx, config, compiler_options, included_files) {
            return Err(format!("Failed to load Forth compiler: {}", e));
        }
        FORTH_COMPILER_LOADED.store(true, Ordering::Relaxed);
    }

    if words_to_compile.is_empty() {
        return Ok(());
    }

    // Step 1: Initialize batch compiler
    ctx.dict.execute_word("INIT-BATCH-COMPILER", ctx.stack, ctx.loop_stack, ctx.return_stack, ctx.memory)?;

    // Step 2: Declare all functions (pass 1 - forward references)
    for (name, _ast) in &words_to_compile {
        // Store word name in memory at HERE
        let here = ctx.memory.here() as usize;
        let name_bytes = name.as_bytes();
        for (i, &byte) in name_bytes.iter().enumerate() {
            ctx.memory.store_byte(here + i, byte as i64)?;
        }

        // Push name address and length
        ctx.stack.push(here as i64, ctx.memory);
        ctx.stack.push(name_bytes.len() as i64, ctx.memory);

        // Call DECLARE-FUNCTION (creates function signature)
        ctx.dict.execute_word("DECLARE-FUNCTION", ctx.stack, ctx.loop_stack, ctx.return_stack, ctx.memory)
            .map_err(|e| format!("Declaration failed for {}: {}", name, e))?;
    }

    // Step 3: Compile each word body (pass 2 - now all functions exist)
    for (name, ast) in &words_to_compile {
        // Register AST node to get a handle
        let ast_handle = crate::ast_forth::ast_register_node(ast.clone());

        // Store word name in memory at HERE
        let here = ctx.memory.here() as usize;
        let name_bytes = name.as_bytes();
        for (i, &byte) in name_bytes.iter().enumerate() {
            ctx.memory.store_byte(here + i, byte as i64)?;
        }

        // Push AST handle
        ctx.stack.push(ast_handle as i64, ctx.memory);

        // Push name address and length
        ctx.stack.push(here as i64, ctx.memory);
        ctx.stack.push(name_bytes.len() as i64, ctx.memory);

        // Call COMPILE-WORD (returns 0 in batch mode)
        ctx.dict.execute_word("COMPILE-WORD", ctx.stack, ctx.loop_stack, ctx.return_stack, ctx.memory)
            .map_err(|e| format!("Compilation failed for {}: {}", name, e))?;
        // Pop and discard the result (0 in batch mode)
        ctx.stack.pop(ctx.memory);
    }

    // Step 4: Finalize batch compilation (create JIT)
    ctx.dict.execute_word("FINALIZE-BATCH", ctx.stack, ctx.loop_stack, ctx.return_stack, ctx.memory)?;

    // Stack now has JIT handle
    let jit_handle = ctx.stack.peek(ctx.memory).ok_or("Failed to get JIT handle")?;

    // Step 5: Get function pointers and update dictionary
    for (name, _ast) in &words_to_compile {
        // Store word name in memory
        let here = ctx.memory.here() as usize;
        let name_bytes = name.as_bytes();
        for (i, &byte) in name_bytes.iter().enumerate() {
            ctx.memory.store_byte(here + i, byte as i64)?;
        }

        // Push JIT handle (duplicate from stack)
        ctx.stack.push(jit_handle, ctx.memory);

        // Push function name: "_fn_WORDNAME"
        let fn_name = format!("_fn_{}", name);
        let fn_name_bytes = fn_name.as_bytes();
        let fn_name_addr = here + name_bytes.len() + 100; // Offset to avoid collision
        for (i, &byte) in fn_name_bytes.iter().enumerate() {
            ctx.memory.store_byte(fn_name_addr + i, byte as i64)?;
        }

        ctx.stack.push(fn_name_addr as i64, ctx.memory);
        ctx.stack.push(fn_name_bytes.len() as i64, ctx.memory);

        // Call LLVM-GET-FUNCTION (returns low and high words)
        ctx.dict.execute_word("LLVM-GET-FUNCTION", ctx.stack, ctx.loop_stack, ctx.return_stack, ctx.memory)?;

        // Get function pointer (two 32-bit values: low, then high)
        let high = ctx.stack.pop(ctx.memory).ok_or(format!("Failed to get function pointer high word for {}", name))?;
        let low = ctx.stack.pop(ctx.memory).ok_or(format!("Failed to get function pointer low word for {}", name))?;

        // Combine into 64-bit pointer
        let fn_ptr = ((high as u64) << 32) | (low as u64 & 0xFFFFFFFF);

        // Convert to JITFunction type and update dictionary
        let jit_fn: crate::dictionary::JITFunction = unsafe { std::mem::transmute(fn_ptr as usize) };
        ctx.dict.add_jit_compiled(name.clone(), jit_fn);

        // Freeze this word to prevent re-definition
        ctx.dict.freeze_word(name);
    }

    // Pop JIT handle from stack
    ctx.stack.pop(ctx.memory);

    Ok(())
}

/// Attempt to compile a word using the Forth self-hosting compiler
/// Loads the compiler if not already loaded
/// Returns true if successful, false otherwise
pub fn try_forth_compile_word(
    name: String,
    ast: &AstNode,
    ctx: &mut RuntimeContext,
    config: CompilerConfig,
    included_files: &mut std::collections::HashSet<String>,
) -> bool {
    // Load the Forth compiler if not already loaded
    if !FORTH_COMPILER_LOADED.load(Ordering::Relaxed) {
        let load_options = ExecutionOptions::new(false, false);
        // Load stdlib first
        if let Err(e) = load_file("stdlib/core.fth", ctx, config, load_options, included_files) {
            eprintln!("Failed to load stdlib for Forth compiler: {}", e);
            return false;
        }
        // Load compiler
        if let Err(e) = load_file("stdlib/compiler.fth", ctx, config, load_options, included_files) {
            eprintln!("Failed to load Forth compiler: {}", e);
            return false;
        }
        FORTH_COMPILER_LOADED.store(true, Ordering::Relaxed);
    }

    // Register the AST
    use crate::ast_forth::ast_register_node;
    let ast_handle = ast_register_node(ast.clone());

    // Write word name to memory at address 302000
    let name_addr = 302000;
    for (i, ch) in name.bytes().enumerate() {
        if ctx.memory.store_byte(name_addr + i, ch as i64).is_err() {
            return false;
        }
    }

    // Save stack pointer in case compilation fails
    let saved_sp = ctx.stack.get_sp();

    // Push arguments for COMPILE-WORD: ( ast-handle name-addr name-len -- fn-ptr )
    ctx.stack.push(ast_handle, ctx.memory);
    ctx.stack.push(name_addr as i64, ctx.memory);
    ctx.stack.push(name.len() as i64, ctx.memory);

    // Execute COMPILE-WORD
    if let Err(e) = ctx.dict.execute_word("COMPILE-WORD", ctx.stack, ctx.loop_stack, ctx.return_stack, ctx.memory) {
        eprintln!("Forth compiler error: {}", e);
        // Restore stack pointer on failure
        ctx.stack.set_sp(saved_sp);
        return false;
    }

    // Get function pointer from stack (two 32-bit values: high, then low)
    if let (Some(fn_ptr_high), Some(fn_ptr_low)) = (ctx.stack.pop(ctx.memory), ctx.stack.pop(ctx.memory)) {
        // Reconstruct 64-bit pointer from two 32-bit values
        let fn_ptr = ((fn_ptr_high as u64) << 32) | ((fn_ptr_low as u64) & 0xFFFFFFFF);

        // Validate pointer is not null
        if fn_ptr == 0 {
            eprintln!("ERROR: Forth compiler returned NULL function pointer!");
            // Restore stack pointer on failure
            ctx.stack.set_sp(saved_sp);
            return false;
        }

        // Cast to JITFunction
        let jit_fn: crate::dictionary::JITFunction = unsafe {
            std::mem::transmute(fn_ptr as *const ())
        };

        // Register in dictionary
        ctx.dict.add_jit_compiled(name.clone(), jit_fn);
        return true;
    }

    eprintln!("ERROR: No function pointer on stack after COMPILE-WORD!");
    // Restore stack pointer on failure
    ctx.stack.set_sp(saved_sp);
    false
}
