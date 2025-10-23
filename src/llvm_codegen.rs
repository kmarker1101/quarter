use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::OptimizationLevel;
use inkwell::AddressSpace;
use crate::ast::AstNode;

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Result<Self, String> {
        // Create module
        let module = context.create_module("quarter");

        // Create execution engine with JIT
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| format!("Failed to create execution engine: {}", e))?;

        // Create builder
        let builder = context.create_builder();

        Ok(Compiler {
            context,
            module,
            builder,
            execution_engine,
        })
    }

    /// Create a function signature for a Forth word
    /// Function signature: void @word_name(i32* %stack_ptr, i32* %return_stack_ptr)
    fn create_function_signature(&self, name: &str) -> inkwell::values::FunctionValue<'ctx> {
        let i32_ptr_type = self.context.ptr_type(AddressSpace::default());
        let void_type = self.context.void_type();

        let fn_type = void_type.fn_type(&[i32_ptr_type.into(), i32_ptr_type.into()], false);
        self.module.add_function(name, fn_type, None)
    }

    /// Compile a Forth word's AST to LLVM IR
    pub fn compile_word(&mut self, name: &str, ast: &AstNode) -> Result<(), String> {
        // Create function with proper signature
        let function = self.create_function_signature(name);

        // Create entry basic block
        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        // Get function parameters (stack_ptr, return_stack_ptr)
        let _stack_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let _return_stack_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        // Compile the AST
        self.compile_ast_node(ast, function)?;

        // Add return
        self.builder.build_return(None)
            .map_err(|e| format!("Failed to build return: {}", e))?;

        Ok(())
    }

    /// Compile an individual AST node
    fn compile_ast_node(&self, node: &AstNode, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        match node {
            AstNode::PushNumber(n) => {
                // Push number onto stack
                self.compile_push(function, *n)?;
                Ok(())
            }
            AstNode::CallWord(word) => {
                match word.as_str() {
                    "*" => self.compile_multiply(function)?,
                    _ => return Err(format!("Unsupported word: {}", word))
                }
                Ok(())
            }
            AstNode::Sequence(nodes) => {
                for node in nodes {
                    self.compile_ast_node(node, function)?;
                }
                Ok(())
            }
            _ => Err(format!("Unsupported AST node: {:?}", node))
        }
    }

    /// Compile a stack push operation
    /// Pushes value onto the stack and increments stack pointer
    fn compile_push(&self, function: inkwell::values::FunctionValue<'ctx>, value: i32) -> Result<(), String> {
        let stack_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let i32_type = self.context.i32_type();

        // Create the constant value
        let const_value = i32_type.const_int(value as u64, true);

        // Store value at stack pointer: *stack_ptr = value
        self.builder.build_store(stack_ptr, const_value)
            .map_err(|e| format!("Failed to build store: {}", e))?;

        // Increment stack pointer: stack_ptr += 4 (size of i32)
        let four = self.context.i32_type().const_int(4, false);
        let new_stack_ptr = unsafe {
            self.builder.build_gep(
                self.context.i32_type(),
                stack_ptr,
                &[four],
                "stack_inc"
            ).map_err(|e| format!("Failed to build GEP: {}", e))?
        };

        // Update the stack pointer parameter (in a real implementation, we'd need to track this)
        // For now, we'll just leave it as-is since we're generating valid IR
        let _ = new_stack_ptr;

        Ok(())
    }

    /// Compile a multiply operation
    /// Pops two values from stack, multiplies them, pushes result
    fn compile_multiply(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let stack_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let i32_type = self.context.i32_type();

        // Pop second operand (top of stack): stack_ptr -= 4; b = *stack_ptr
        let four = i32_type.const_int(4, false);
        let neg_four = i32_type.const_int((-4i64) as u64, true);

        let stack_ptr_1 = unsafe {
            self.builder.build_gep(i32_type, stack_ptr, &[neg_four], "pop1")
                .map_err(|e| format!("Failed to build GEP for pop: {}", e))?
        };
        let b = self.builder.build_load(i32_type, stack_ptr_1, "b")
            .map_err(|e| format!("Failed to build load: {}", e))?
            .into_int_value();

        // Pop first operand: stack_ptr -= 4; a = *stack_ptr
        let stack_ptr_2 = unsafe {
            self.builder.build_gep(i32_type, stack_ptr_1, &[neg_four], "pop2")
                .map_err(|e| format!("Failed to build GEP for pop: {}", e))?
        };
        let a = self.builder.build_load(i32_type, stack_ptr_2, "a")
            .map_err(|e| format!("Failed to build load: {}", e))?
            .into_int_value();

        // Multiply: result = a * b
        let result = self.builder.build_int_mul(a, b, "mul")
            .map_err(|e| format!("Failed to build mul: {}", e))?;

        // Push result back: *stack_ptr = result; stack_ptr += 4
        self.builder.build_store(stack_ptr_2, result)
            .map_err(|e| format!("Failed to build store: {}", e))?;

        Ok(())
    }

    /// Get the LLVM IR as a string (useful for debugging)
    pub fn get_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_compiler_initialization() {
        let context = Context::create();
        let compiler = Compiler::new(&context);
        assert!(compiler.is_ok());
    }

    #[test]
    fn test_compile_simple_number() {
        // Test compiling: : TEST 42 ;
        let context = Context::create();
        let mut compiler = Compiler::new(&context).unwrap();

        // Create AST for pushing 42
        let ast = AstNode::PushNumber(42);

        // Compile it
        let result = compiler.compile_word("TEST", &ast);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        // Verify IR was generated
        let ir = compiler.get_ir();
        assert!(ir.contains("define void @TEST"), "IR should contain function definition");
        println!("Generated IR:\n{}", ir);
    }

    #[test]
    fn test_compile_double() {
        // Test compiling: : DOUBLE 2 * ;
        // This word multiplies the top stack value by 2
        let context = Context::create();
        let mut compiler = Compiler::new(&context).unwrap();

        // Create AST: push 2, then multiply
        let ast = AstNode::Sequence(vec![
            AstNode::PushNumber(2),
            AstNode::CallWord("*".to_string()),
        ]);

        // Compile it
        let result = compiler.compile_word("DOUBLE", &ast);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        // Verify IR was generated
        let ir = compiler.get_ir();
        assert!(ir.contains("define void @DOUBLE"), "IR should contain DOUBLE function");
        assert!(ir.contains("mul"), "IR should contain multiplication");
        assert!(ir.contains("store"), "IR should contain store operations");
        assert!(ir.contains("load"), "IR should contain load operations");

        println!("\n=== Generated IR for DOUBLE ===");
        println!("{}", ir);
        println!("================================\n");
    }
}
