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
    // Store JIT compilers to keep execution engines alive
    jit_compilers: Vec<Box<crate::llvm_codegen::Compiler<'static>>>,
}

impl Dictionary {
    pub fn new() -> Self {
        let mut dict = Dictionary {
            words: HashMap::new(),
            jit_compilers: Vec::new(),
        };

        // Register built-in words as Primitives
        dict.add_primitive(".", words::dot);
        dict.add_primitive("+", words::add);
        dict.add_primitive("-", words::subtract);
        dict.add_primitive("*", words::multiply);
        dict.add_primitive("/", words::divide);
        dict.add_primitive("DUP", words::dup);
        dict.add_primitive("SWAP", words::swap);
        dict.add_primitive(".S", words::dot_s);
        dict.add_primitive("U.", words::u_dot);
        dict.add_primitive(".R", words::dot_r);
        dict.add_primitive("U.R", words::u_dot_r);
        dict.add_primitive("<", words::less_than);
        dict.add_primitive(">", words::greater_than);
        // =, <>, <=, >= now defined in comparison.fth
        // NEGATE, ABS now defined in core.fth
        dict.add_primitive("CR", words::cr);
        dict.add_primitive("DROP", words::drop);
        // OVER now defined in core.fth
        dict.add_primitive("/MOD", words::slash_modulo);
        dict.add_primitive("I", words::loop_i);
        dict.add_primitive("J", words::loop_j);
        dict.add_primitive("EMIT", words::emit);
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
        // 0=, 0<, 0> now defined in comparison.fth
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

    pub fn add_jit_compiled_with_compiler(&mut self, name: String, func: JITFunction, compiler: Box<crate::llvm_codegen::Compiler<'static>>) {
        self.words.insert(name, Word::JITCompiled(func));
        self.jit_compilers.push(compiler);
    }

    pub fn has_word(&self, word: &str) -> bool {
        self.words.contains_key(word)
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
