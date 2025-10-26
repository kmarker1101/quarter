use crate::words;
use crate::{ast::AstNode, stack::Stack};
use std::collections::HashMap;

// Type alias for JIT-compiled Forth functions
// Function signature: void word(u8* memory, usize* sp, usize* rp)
// - memory: pointer to the start of the memory buffer
// - sp: pointer to stack pointer (can be read and modified)
// - rp: pointer to return stack pointer (can be read and modified)
pub type JITFunction = unsafe extern "C" fn(*mut u8, *mut usize, *mut usize);

pub enum Word {
    Primitive(fn(&mut Stack, &crate::LoopStack, &mut crate::ReturnStack, &mut crate::Memory)),
    Compiled(AstNode),
    JITCompiled(JITFunction),
}

pub struct Dictionary {
    words: HashMap<String, Word>,
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
        };

        // Register built-in words as Primitives
        dict.add_primitive(".", words::dot);
        dict.add_primitive("+", words::add);
        dict.add_primitive("-", words::subtract);
        dict.add_primitive("*", words::multiply);
        dict.add_primitive("/", words::divide);
        dict.add_primitive("DUP", words::dup);
        dict.add_primitive("SWAP", words::swap);
        dict.add_primitive("PICK", words::pick);
        dict.add_primitive(".S", words::dot_s);
        dict.add_primitive("U.", words::u_dot);
        dict.add_primitive(".R", words::dot_r);
        dict.add_primitive("U.R", words::u_dot_r);
        dict.add_primitive("<", words::less_than);
        dict.add_primitive(">", words::greater_than);
        dict.add_primitive("=", words::equal);
        dict.add_primitive("<>", words::not_equal);
        dict.add_primitive("<=", words::less_equal);
        dict.add_primitive(">=", words::greater_equal);
        dict.add_primitive("0=", words::zero_equal);
        dict.add_primitive("0<", words::zero_less);
        dict.add_primitive("0>", words::zero_greater);
        dict.add_primitive("NEGATE", words::negate);
        dict.add_primitive("ABS", words::abs);
        dict.add_primitive("MIN", words::min);
        dict.add_primitive("MAX", words::max);
        dict.add_primitive("1+", words::one_plus);
        dict.add_primitive("1-", words::one_minus);
        dict.add_primitive("2*", words::two_star);
        dict.add_primitive("2/", words::two_slash);
        dict.add_primitive("CR", words::cr);
        dict.add_primitive("DROP", words::drop);
        dict.add_primitive("OVER", words::over);
        dict.add_primitive("ROT", words::rot);
        dict.add_primitive("DEPTH", words::depth);
        dict.add_primitive("/MOD", words::slash_modulo);
        dict.add_primitive("MOD", words::mod_word);
        dict.add_primitive("I", words::loop_i);
        dict.add_primitive("J", words::loop_j);
        dict.add_primitive("EMIT", words::emit);
        dict.add_primitive("TYPE", words::type_word);
        dict.add_primitive("KEY", words::key);
        // SPACE now defined in io.fth
        dict.add_primitive("AND", words::and);
        dict.add_primitive("OR", words::or);
        dict.add_primitive("XOR", words::xor);
        dict.add_primitive("INVERT", words::invert);
        dict.add_primitive("LSHIFT", words::lshift);
        dict.add_primitive("RSHIFT", words::rshift);
        dict.add_primitive(">R", words::to_r);
        dict.add_primitive("R>", words::r_from);
        dict.add_primitive("R@", words::r_fetch);
        // TRUE, FALSE now defined in core.fth as constants
        dict.add_primitive("!", words::store);
        dict.add_primitive("@", words::fetch);
        dict.add_primitive("C!", words::c_store);
        dict.add_primitive("C@", words::c_fetch);
        dict.add_primitive("SP@", words::sp_fetch);
        dict.add_primitive("SP!", words::sp_store);
        dict.add_primitive("RP@", words::rp_fetch);
        dict.add_primitive("RP!", words::rp_store);
        // CELLS now defined in core.fth
        dict.add_primitive("HERE", words::here);
        dict.add_primitive("ALLOT", words::allot);
        dict.add_primitive(",", words::comma);

        // LLVM primitives for self-hosting compiler
        dict.add_primitive("LLVM-CREATE-CONTEXT", words::llvm_create_context_word);
        dict.add_primitive("LLVM-CREATE-MODULE", words::llvm_create_module_word);
        dict.add_primitive("LLVM-DECLARE-EXTERNAL", words::llvm_declare_external_word);
        dict.add_primitive("LLVM-CREATE-BUILDER", words::llvm_create_builder_word);
        dict.add_primitive("LLVM-CREATE-FUNCTION", words::llvm_create_function_word);
        dict.add_primitive("LLVM-MODULE-GET-FUNCTION", words::llvm_module_get_function_word);
        dict.add_primitive("LLVM-CREATE-BLOCK", words::llvm_create_block_word);
        dict.add_primitive("LLVM-POSITION-AT-END", words::llvm_position_at_end_word);
        dict.add_primitive("LLVM-BUILD-RET-VOID", words::llvm_build_ret_void_word);
        dict.add_primitive("LLVM-BUILD-RET", words::llvm_build_ret_word);
        dict.add_primitive("LLVM-DUMP-MODULE", words::llvm_dump_module_word);
        dict.add_primitive("LLVM-CREATE-JIT", words::llvm_create_jit_word);
        dict.add_primitive("LLVM-GET-FUNCTION", words::llvm_get_function_word);

        // LLVM IR builder primitives
        dict.add_primitive("LLVM-BUILD-CONST-INT", words::llvm_build_const_int_word);
        dict.add_primitive("LLVM-BUILD-LOAD", words::llvm_build_load_word);
        dict.add_primitive("LLVM-BUILD-STORE", words::llvm_build_store_word);
        dict.add_primitive("LLVM-BUILD-GEP", words::llvm_build_gep_word);
        dict.add_primitive("LLVM-BUILD-ADD", words::llvm_build_add_word);
        dict.add_primitive("LLVM-BUILD-SUB", words::llvm_build_sub_word);
        dict.add_primitive("LLVM-BUILD-MUL", words::llvm_build_mul_word);
        dict.add_primitive("LLVM-BUILD-SDIV", words::llvm_build_sdiv_word);
        dict.add_primitive("LLVM-BUILD-SREM", words::llvm_build_srem_word);
        dict.add_primitive("LLVM-BUILD-AND", words::llvm_build_and_word);
        dict.add_primitive("LLVM-BUILD-OR", words::llvm_build_or_word);
        dict.add_primitive("LLVM-BUILD-XOR", words::llvm_build_xor_word);
        dict.add_primitive("LLVM-BUILD-SHL", words::llvm_build_shl_word);
        dict.add_primitive("LLVM-BUILD-ASHR", words::llvm_build_ashr_word);
        dict.add_primitive("LLVM-BUILD-BR", words::llvm_build_br_word);
        dict.add_primitive("LLVM-BUILD-COND-BR", words::llvm_build_cond_br_word);
        dict.add_primitive("LLVM-BUILD-ICMP", words::llvm_build_icmp_word);
        dict.add_primitive("LLVM-BUILD-SEXT", words::llvm_build_sext_word);
        dict.add_primitive("LLVM-BUILD-CALL", words::llvm_build_call_word);
        dict.add_primitive("LLVM-GET-PARAM", words::llvm_get_param_word);
        dict.add_primitive("LLVM-BUILD-PHI", words::llvm_build_phi_word);
        dict.add_primitive("LLVM-PHI-ADD-INCOMING", words::llvm_phi_add_incoming_word);
        dict.add_primitive("LLVM-GET-INSERT-BLOCK", words::llvm_get_insert_block_word);

        // AST inspection primitives
        dict.add_primitive("AST-TYPE", words::ast_get_type_word);
        dict.add_primitive("AST-GET-NUMBER", words::ast_get_number_word);
        dict.add_primitive("AST-GET-WORD", words::ast_get_word_word);
        dict.add_primitive("AST-GET-STRING", words::ast_get_string_word);
        dict.add_primitive("AST-SEQ-LENGTH", words::ast_seq_length_word);
        dict.add_primitive("AST-SEQ-CHILD", words::ast_seq_child_word);
        dict.add_primitive("AST-IF-THEN", words::ast_if_then_word);
        dict.add_primitive("AST-IF-ELSE", words::ast_if_else_word);
        dict.add_primitive("AST-LOOP-BODY", words::ast_loop_body_word);
        dict.add_primitive("AST-LOOP-CONDITION", words::ast_loop_condition_word);
        dict.add_primitive("AST-LOOP-INCREMENT", words::ast_loop_increment_word);

        // Test primitives for compiler development
        dict.add_primitive("TEST-AST-CREATE", words::test_ast_create_word);

        // JIT word registration
        dict.add_primitive("REGISTER-JIT-WORD", words::register_jit_word);

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
        self.words.insert(name, Word::Compiled(ast));
    }

    pub fn add_jit_compiled(&mut self, name: String, func: JITFunction) {
        self.words.insert(name, Word::JITCompiled(func));
    }


    pub fn has_word(&self, word: &str) -> bool {
        self.words.contains_key(word)
    }

    pub fn get_word(&self, word: &str) -> Option<&Word> {
        self.words.get(word)
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
                    // Execute the AST, catching EXIT to convert it to Ok
                    match ast.execute(stack, self, loop_stack, return_stack, memory) {
                        Err(msg) if msg == "EXIT" => Ok(()),
                        result => result,
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
