use crate::ast::AstNode;
use crate::stack::Stack;
use crate::words;
use std::collections::HashMap;

pub enum Word {
    Primitive(fn(&mut Stack, &crate::LoopStack)),
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

        dict
    }

    pub fn add_primitive(&mut self, name: &str, func: fn(&mut Stack, &crate::LoopStack)) {
        self.words.insert(name.to_string(), Word::Primitive(func));
    }

    pub fn add_compiled(&mut self, name: String, ast: AstNode) {
        self.words.insert(name, Word::Compiled(ast));
    }

    pub fn execute_word(
        &self,
        word: &str,
        stack: &mut Stack,
        loop_stack: &mut crate::LoopStack,
    ) -> Result<(), String> {
        if let Some(w) = self.words.get(word) {
            match w {
                Word::Primitive(func) => {
                    func(stack, loop_stack);
                    Ok(())
                }
                Word::Compiled(ast) => ast.execute(stack, self, loop_stack),
            }
        } else {
            Err(format!("Unknown word: {}", word))
        }
    }
}
