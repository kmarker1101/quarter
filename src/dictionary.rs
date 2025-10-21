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
        dict.add_primitive("MOD", words::modulo);
        dict.add_primitive(".S", words::dot_s);
        dict.add_primitive("<", words::less_than);
        dict.add_primitive(">", words::greater_than);
        dict.add_primitive("=", words::equals);
        dict.add_primitive("<>", words::not_equals);
        dict.add_primitive("<=", words::less_or_equal);
        dict.add_primitive(">=", words::greater_or_equal);
        dict.add_primitive("NEGATE", words::negate);
        dict.add_primitive("ABS", words::abs);
        dict.add_primitive("CR", words::cr);
        dict.add_primitive("DROP", words::drop);
        dict.add_primitive("ROT", words::rot);
        dict.add_primitive("OVER", words::over);
        dict.add_primitive("/MOD", words::slash_modulo);
        dict.add_primitive("I", words::loop_i);
        dict.add_primitive("J", words::loop_j);
        dict.add_primitive("EMIT", words::emit);
        dict.add_primitive("KEY", words::key);
        dict.add_primitive("SPACE", words::space);
        dict.add_primitive("AND", words::and);
        dict.add_primitive("OR", words::or);
        dict.add_primitive("XOR", words::xor);
        dict.add_primitive("INVERT", words::invert);
        dict.add_primitive("LSHIFT", words::lshift);
        dict.add_primitive("RSHIFT", words::rshift);
        dict.add_primitive(">R", words::to_r);
        dict.add_primitive("R>", words::r_from);
        dict.add_primitive("R@", words::r_fetch);
        dict.add_primitive("0=", words::zero_equals);
        dict.add_primitive("0<", words::zero_less);
        dict.add_primitive("0>", words::zero_greater);
        dict.add_primitive("TRUE", words::forth_true);
        dict.add_primitive("FALSE", words::forth_false);
        dict.add_primitive("!", words::store);
        dict.add_primitive("@", words::fetch);
        dict.add_primitive("C!", words::c_store);
        dict.add_primitive("C@", words::c_fetch);
        dict.add_primitive("SP@", words::sp_fetch);
        dict.add_primitive("SP!", words::sp_store);
        dict.add_primitive("RP@", words::rp_fetch);
        dict.add_primitive("RP!", words::rp_store);
        dict.add_primitive("CELLS", words::cells);
        dict.add_primitive("CELL+", words::cell_plus);
        dict.add_primitive("+!", words::plus_store);
        dict.add_primitive("HERE", words::here);
        dict.add_primitive("ALLOT", words::allot);
        dict.add_primitive(",", words::comma);

        dict
    }

    pub fn add_primitive(&mut self, name: &str, func: fn(&mut Stack, &crate::LoopStack, &mut crate::ReturnStack, &mut crate::Memory)) {
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
