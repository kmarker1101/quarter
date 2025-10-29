use crate::words;
use crate::{ast::AstNode, stack::Stack};
use std::collections::{HashMap, HashSet};

// Type alias for JIT-compiled Forth functions
// Function signature: void word(u8* memory, usize* sp, usize* rp)
// - memory: pointer to the start of the memory buffer
// - sp: pointer to stack pointer (can be read and modified)
// - rp: pointer to return stack pointer (can be read and modified)
pub type JITFunction = unsafe extern "C" fn(*mut u8, *mut usize, *mut usize);

// Macro to register multiple primitive words at once
// Usage: register_primitives!(dict, "NAME" => words::function, ...)
macro_rules! register_primitives {
    ($dict:expr, $($name:expr => $func:expr),* $(,)?) => {
        $(
            $dict.add_primitive($name, $func);
        )*
    };
}

pub enum Word {
    Primitive(fn(&mut Stack, &crate::LoopStack, &mut crate::ReturnStack, &mut crate::Memory)),
    Compiled(AstNode),
    JITCompiled(JITFunction),
}

pub struct Dictionary {
    words: HashMap<String, Word>,
    frozen_words: HashSet<String>,
    immediate_words: HashSet<String>,
    last_defined_word: Option<String>,
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl Dictionary {
    pub fn new() -> Self {
        let mut dict = Dictionary {
            words: HashMap::new(),
            frozen_words: HashSet::new(),
            immediate_words: HashSet::new(),
            last_defined_word: None,
        };

        // Register all built-in primitive words using macro
        register_primitives!(dict,
            // I/O operations
            "." => words::dot,
            ".S" => words::dot_s,
            "U." => words::u_dot,
            ".R" => words::dot_r,
            "U.R" => words::u_dot_r,
            "CR" => words::cr,
            "EMIT" => words::emit,
            "SPACE" => words::space,
            "TYPE" => words::type_word,
            "COMPARE" => words::compare,
            "-TRAILING" => words::minus_trailing,
            "SEARCH" => words::search,
            "KEY" => words::key,

            // Arithmetic operations
            "+" => words::add,
            "-" => words::subtract,
            "*" => words::multiply,
            "/" => words::divide,
            "*/" => words::star_slash,
            "/MOD" => words::slash_modulo,
            "MOD" => words::mod_word,
            "NEGATE" => words::negate,
            "ABS" => words::abs,
            "MIN" => words::min,
            "MAX" => words::max,
            "1+" => words::one_plus,
            "1-" => words::one_minus,
            "2*" => words::two_star,
            "2/" => words::two_slash,

            // Comparison operations
            "<" => words::less_than,
            ">" => words::greater_than,
            "=" => words::equal,
            "<>" => words::not_equal,
            "<=" => words::less_equal,
            ">=" => words::greater_equal,
            "U<" => words::u_less_than,
            "0=" => words::zero_equal,
            "0<" => words::zero_less,
            "0>" => words::zero_greater,

            // Stack operations
            "DUP" => words::dup,
            "?DUP" => words::question_dup,
            "DROP" => words::drop,
            "SWAP" => words::swap,
            "OVER" => words::over,
            "ROT" => words::rot,
            "PICK" => words::pick,
            "DEPTH" => words::depth,

            // Return stack operations
            ">R" => words::to_r,
            "R>" => words::r_from,
            "R@" => words::r_fetch,

            // Bitwise operations
            "AND" => words::and,
            "OR" => words::or,
            "XOR" => words::xor,
            "INVERT" => words::invert,
            "LSHIFT" => words::lshift,
            "RSHIFT" => words::rshift,

            // Memory operations
            "!" => words::store,
            "@" => words::fetch,
            "C!" => words::c_store,
            "C@" => words::c_fetch,

            // Stack pointer operations
            "SP@" => words::sp_fetch,
            "SP!" => words::sp_store,
            "RP@" => words::rp_fetch,
            "RP!" => words::rp_store,

            // Memory allocation
            "HERE" => words::here,
            "ALLOT" => words::allot,
            "," => words::comma,
            "BASE" => words::base,
            ">NUMBER" => words::to_number,

            // Loop operations
            "I" => words::loop_i,
            "J" => words::loop_j,

            // LLVM context and module operations
            "LLVM-CREATE-CONTEXT" => words::llvm_create_context_word,
            "LLVM-CREATE-MODULE" => words::llvm_create_module_word,
            "LLVM-DECLARE-EXTERNAL" => words::llvm_declare_external_word,
            "LLVM-CREATE-BUILDER" => words::llvm_create_builder_word,
            "LLVM-CREATE-FUNCTION" => words::llvm_create_function_word,
            "LLVM-MODULE-GET-FUNCTION" => words::llvm_module_get_function_word,
            "LLVM-CREATE-BLOCK" => words::llvm_create_block_word,
            "LLVM-POSITION-AT-END" => words::llvm_position_at_end_word,
            "LLVM-BUILD-RET-VOID" => words::llvm_build_ret_void_word,
            "LLVM-BUILD-RET" => words::llvm_build_ret_word,
            "LLVM-DUMP-MODULE" => words::llvm_dump_module_word,
            "LLVM-CREATE-JIT" => words::llvm_create_jit_word,
            "LLVM-GET-FUNCTION" => words::llvm_get_function_word,

            // LLVM IR builder operations
            "LLVM-BUILD-CONST-INT" => words::llvm_build_const_int_word,
            "LLVM-BUILD-LOAD" => words::llvm_build_load_word,
            "LLVM-BUILD-STORE" => words::llvm_build_store_word,
            "LLVM-BUILD-GEP" => words::llvm_build_gep_word,
            "LLVM-BUILD-ADD" => words::llvm_build_add_word,
            "LLVM-BUILD-SUB" => words::llvm_build_sub_word,
            "LLVM-BUILD-MUL" => words::llvm_build_mul_word,
            "LLVM-BUILD-SDIV" => words::llvm_build_sdiv_word,
            "LLVM-BUILD-SREM" => words::llvm_build_srem_word,
            "LLVM-BUILD-AND" => words::llvm_build_and_word,
            "LLVM-BUILD-OR" => words::llvm_build_or_word,
            "LLVM-BUILD-XOR" => words::llvm_build_xor_word,
            "LLVM-BUILD-SHL" => words::llvm_build_shl_word,
            "LLVM-BUILD-ASHR" => words::llvm_build_ashr_word,
            "LLVM-BUILD-BR" => words::llvm_build_br_word,
            "LLVM-BUILD-COND-BR" => words::llvm_build_cond_br_word,
            "LLVM-BUILD-ICMP" => words::llvm_build_icmp_word,
            "LLVM-BUILD-SEXT" => words::llvm_build_sext_word,
            "LLVM-BUILD-SELECT" => words::llvm_build_select_word,
            "LLVM-BUILD-TRUNC" => words::llvm_build_trunc_word,
            "LLVM-BUILD-CALL" => words::llvm_build_call_word,
            "LLVM-GET-PARAM" => words::llvm_get_param_word,
            "LLVM-BUILD-PHI" => words::llvm_build_phi_word,
            "LLVM-PHI-ADD-INCOMING" => words::llvm_phi_add_incoming_word,
            "LLVM-GET-INSERT-BLOCK" => words::llvm_get_insert_block_word,
            "LLVM-INITIALIZE-NATIVE-TARGET" => words::llvm_initialize_native_target_word,
            "LLVM-WRITE-OBJECT-FILE" => words::llvm_write_object_file_word,
            "LLVM-BUILD-PTRTOINT" => words::llvm_build_ptrtoint_word,
            "LLVM-CREATE-GLOBAL-STRING" => words::llvm_create_global_string_word,

            // AST inspection operations
            "AST-TYPE" => words::ast_get_type_word,
            "AST-GET-NUMBER" => words::ast_get_number_word,
            "AST-GET-WORD" => words::ast_get_word_word,
            "AST-GET-STRING" => words::ast_get_string_word,
            "AST-SEQ-LENGTH" => words::ast_seq_length_word,
            "AST-SEQ-CHILD" => words::ast_seq_child_word,
            "AST-IF-THEN" => words::ast_if_then_word,
            "AST-IF-ELSE" => words::ast_if_else_word,
            "AST-LOOP-BODY" => words::ast_loop_body_word,
            "AST-LOOP-CONDITION" => words::ast_loop_condition_word,
            "AST-LOOP-INCREMENT" => words::ast_loop_increment_word,

            // Test and JIT operations
            "TEST-AST-CREATE" => words::test_ast_create_word,
            "REGISTER-JIT-WORD" => words::register_jit_word,

            // REPL operations (for Forth-based REPL)
            "READLINE" => words::readline_word,
            "HISTORY-ADD" => words::history_add_word,
            "HISTORY-LOAD" => words::history_load_word,
            "HISTORY-SAVE" => words::history_save_word,
            "EVALUATE" => words::evaluate_word,
            "CMOVE" => words::cmove_word,
            "BYE" => words::bye_word,
            "ABORT" => words::abort_word,
            "THROW" => words::throw_word,
            "CATCH" => words::catch_word,
        );

        // Add EXECUTE as a compiled word that takes xt from stack and executes it
        dict.add_compiled("EXECUTE".to_string(), AstNode::Execute);

        // Add FIND as a compiled word that searches dictionary
        dict.add_compiled("FIND".to_string(), AstNode::Find);

        dict
    }

    pub fn add_primitive(
        &mut self,
        name: &str,
        func: fn(&mut Stack, &crate::LoopStack, &mut crate::ReturnStack, &mut crate::Memory),
    ) {
        self.words.insert(name.to_string(), Word::Primitive(func));
    }

    pub fn add_compiled(&mut self, name: String, ast: AstNode) {
        self.last_defined_word = Some(name.clone());
        self.words.insert(name, Word::Compiled(ast));
    }

    pub fn add_jit_compiled(&mut self, name: String, func: JITFunction) {
        self.last_defined_word = Some(name.clone());
        self.words.insert(name, Word::JITCompiled(func));
    }


    pub fn has_word(&self, word: &str) -> bool {
        self.words.contains_key(word)
    }

    pub fn get_word(&self, word: &str) -> Option<&Word> {
        self.words.get(word)
    }

    /// Get all words with their names (for batch compilation)
    pub fn get_all_words(&self) -> Vec<(String, &Word)> {
        self.words.iter().map(|(k, v)| (k.clone(), v)).collect()
    }

    /// Freeze a word to prevent re-definition (used after JIT compilation)
    pub fn freeze_word(&mut self, name: &str) {
        self.frozen_words.insert(name.to_uppercase());
    }

    /// Check if a word is frozen (cannot be re-defined)
    pub fn is_frozen(&self, name: &str) -> bool {
        self.frozen_words.contains(&name.to_uppercase())
    }

    /// Mark the most recently defined word as immediate
    pub fn mark_immediate(&mut self) {
        if let Some(ref word_name) = self.last_defined_word {
            self.immediate_words.insert(word_name.to_uppercase());
        }
    }

    /// Check if a word is immediate (executes during compilation)
    pub fn is_immediate(&self, name: &str) -> bool {
        self.immediate_words.contains(&name.to_uppercase())
    }

    /// Get the last defined word name
    pub fn get_last_defined_word(&self) -> Option<&String> {
        self.last_defined_word.as_ref()
    }

    /// Check if an AST node is a tail-recursive call to the given word
    fn is_tail_recursive_call(node: &AstNode, word_name: &str) -> bool {
        match node {
            AstNode::CallWord(name) => name.to_uppercase() == word_name.to_uppercase(),
            AstNode::Sequence(nodes) => {
                // Only the last node can be a tail call
                if let Some(last) = nodes.last() {
                    Self::is_tail_recursive_call(last, word_name)
                } else {
                    false
                }
            }
            AstNode::IfThenElse { then_branch, else_branch } => {
                // Check if BOTH branches end with tail calls
                let then_tail = if let Some(last) = then_branch.last() {
                    Self::is_tail_recursive_call(last, word_name)
                } else {
                    false
                };
                let else_tail = if let Some(else_nodes) = else_branch {
                    if let Some(last) = else_nodes.last() {
                        Self::is_tail_recursive_call(last, word_name)
                    } else {
                        false
                    }
                } else {
                    false
                };
                then_tail || else_tail
            }
            _ => false,
        }
    }

    pub fn execute_word(
        &self,
        word: &str,
        stack: &mut Stack,
        loop_stack: &mut crate::LoopStack,
        return_stack: &mut crate::ReturnStack,
        memory: &mut crate::Memory,
    ) -> Result<(), String> {
        if let Some(w) = self.words.get(word) {
            match w {
                Word::Primitive(func) => {
                    func(stack, loop_stack, return_stack, memory);
                    Ok(())
                }
                Word::Compiled(ast) => {
                    // Check if this is a tail-recursive function
                    if Self::is_tail_recursive_call(ast, word) {
                        // Tail call optimization: execute in a loop instead of recursion
                        loop {
                            match ast.execute_with_tco_check(stack, self, loop_stack, return_stack, memory, word) {
                                Ok(true) => {
                                    // Tail call detected, continue loop
                                    continue;
                                }
                                Ok(false) => {
                                    // Normal completion
                                    return Ok(());
                                }
                                Err(msg) if msg == "EXIT" => {
                                    return Ok(());
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                    } else {
                        // No tail recursion, execute normally
                        match ast.execute(stack, self, loop_stack, return_stack, memory) {
                            Err(msg) if msg == "EXIT" => Ok(()),
                            result => result,
                        }
                    }
                }
                Word::JITCompiled(jit_fn) => {
                    // Execute JIT-compiled native code
                    // Pass memory buffer and mutable references to sp/rp
                    let memory_ptr = memory.as_mut_ptr();
                    let sp_ptr = stack.sp_mut_ptr();
                    let rp_ptr = return_stack.rp_mut_ptr();

                    unsafe {
                        jit_fn(memory_ptr, sp_ptr, rp_ptr);
                    }
                    Ok(())
                }
            }
        } else {
            Err(format!("Unknown word: {}", word))
        }
    }
}
