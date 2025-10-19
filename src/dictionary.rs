use crate::ast::AstNode;
use crate::stack::Stack;
use crate::words;
use std::collections::HashMap;

pub enum Word {
    Primitive(fn(&mut Stack)),
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

        dict
    }

    pub fn add_primitive(&mut self, name: &str, func: fn(&mut Stack)) {
        self.words.insert(name.to_string(), Word::Primitive(func));
    }

    pub fn add_compiled(&mut self, name: String, ast: AstNode) {
        self.words.insert(name, Word::Compiled(ast));
    }

    pub fn execute_word(&self, word: &str, stack: &mut Stack) -> Result<(), String> {
        if let Some(w) = self.words.get(word) {
            match w {
                Word::Primitive(func) => {
                    func(stack);
                    Ok(())
                }
                Word::Compiled(ast) => ast.execute(stack, self),
            }
        } else {
            Err(format!("Unknown word: {}", word))
        }
    }
}
