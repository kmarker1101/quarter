use crate::words;
use crate::{ast::AstNode, stack::Stack};
use std::collections::HashMap;

pub enum Word {
    Primitive(fn(&mut Stack, &crate::LoopStack, &mut crate::ReturnStack, &mut crate::Memory)),
    Compiled(AstNode),
}

pub struct Dictionary {
    words: HashMap<String, Word>,
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
            }
        } else {
            Err(format!("Unknown word: {}", word))
        }
    }
}
