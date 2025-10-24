use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::OptimizationLevel;
use inkwell::AddressSpace;
use crate::ast::AstNode;
use crate::dictionary::JITFunction;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    // Stack of loop indices for nested DO/LOOP constructs
    loop_index_stack: RefCell<Vec<inkwell::values::IntValue<'ctx>>>,
    // Name of the function currently being compiled (for recursion support)
    current_function_name: RefCell<Option<String>>,
}

// Global registry of JIT-compiled function pointers
// Maps word names (uppercase) to their JIT function pointers
// This allows different compiled words to call each other
lazy_static::lazy_static! {
    static ref JIT_FUNCTION_REGISTRY: Mutex<HashMap<String, JITFunction>> = {
        Mutex::new(HashMap::new())
    };
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Result<Self, String> {
        // Create module
        let module = context.create_module("quarter");

        // Create execution engine with JIT and aggressive optimization
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::Aggressive)
            .map_err(|e| format!("Failed to create execution engine: {}", e))?;

        // Create builder
        let builder = context.create_builder();

        let mut compiler = Compiler {
            context,
            module,
            builder,
            execution_engine,
            loop_index_stack: RefCell::new(Vec::new()),
            current_function_name: RefCell::new(None),
        };

        // Declare external Rust primitive functions
        compiler.declare_external_primitives();

        // Map external symbols to Rust functions
        use crate::words::*;
        compiler.execution_engine.add_global_mapping(&compiler.module.get_function("quarter_dup").unwrap(), quarter_dup as usize);
        compiler.execution_engine.add_global_mapping(&compiler.module.get_function("quarter_drop").unwrap(), quarter_drop as usize);
        compiler.execution_engine.add_global_mapping(&compiler.module.get_function("quarter_swap").unwrap(), quarter_swap as usize);
        compiler.execution_engine.add_global_mapping(&compiler.module.get_function("quarter_add").unwrap(), quarter_add as usize);
        compiler.execution_engine.add_global_mapping(&compiler.module.get_function("quarter_sub").unwrap(), quarter_sub as usize);
        compiler.execution_engine.add_global_mapping(&compiler.module.get_function("quarter_mul").unwrap(), quarter_mul as usize);
        compiler.execution_engine.add_global_mapping(&compiler.module.get_function("quarter_div").unwrap(), quarter_div as usize);
        compiler.execution_engine.add_global_mapping(&compiler.module.get_function("quarter_less_than").unwrap(), quarter_less_than as usize);

        Ok(compiler)
    }

    /// Declare external Rust primitive functions as LLVM functions
    /// These are callable from JIT-compiled code
    fn declare_external_primitives(&mut self) {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let void_type = self.context.void_type();

        // All primitives have the same signature: void primitive(u8* memory, usize* sp, usize* rp)
        let fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);

        // Declare common primitives
        let primitives = vec![
            "quarter_dup", "quarter_drop", "quarter_swap",
            "quarter_add", "quarter_sub", "quarter_mul", "quarter_div",
            "quarter_less_than",
        ];

        for prim in primitives {
            self.module.add_function(prim, fn_type, None);
        }
    }

    /// Register existing JIT-compiled words as external symbols
    /// This allows the current word being compiled to call other JIT-compiled words
    fn register_jit_symbols(&mut self) -> Result<(), String> {
        let registry = JIT_FUNCTION_REGISTRY.lock()
            .map_err(|e| format!("Failed to lock JIT registry: {}", e))?;

        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);

        for (name, jit_fn) in registry.iter() {
            // Declare the function in this module
            self.module.add_function(name, fn_type, None);

            // Register the function pointer with the execution engine
            // This allows LLVM to resolve calls to this function
            let fn_ptr = *jit_fn as usize;
            self.execution_engine.add_global_mapping(
                &self.module.get_function(name).unwrap(),
                fn_ptr
            );
        }

        Ok(())
    }

    /// Create a function signature for a Forth word
    /// Function signature: void @word_name(u8* memory, usize* sp, usize* rp)
    fn create_function_signature(&self, name: &str) -> inkwell::values::FunctionValue<'ctx> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let void_type = self.context.void_type();

        let fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);
        self.module.add_function(name, fn_type, None)
    }

    /// Compile a Forth word's AST to LLVM IR and return executable function
    pub fn compile_word(&mut self, name: &str, ast: &AstNode) -> Result<JITFunction, String> {
        // Register existing JIT-compiled words so this word can call them
        self.register_jit_symbols()?;

        // Set the current function name for recursion support
        *self.current_function_name.borrow_mut() = Some(name.to_uppercase());

        // Create function with proper signature
        let function = self.create_function_signature(name);

        // Create entry basic block
        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        // Get function parameters (memory, sp, rp)
        let _memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let _sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();
        let _rp_ptr = function.get_nth_param(2).unwrap().into_pointer_value();

        // Compile the AST
        self.compile_ast_node(ast, function)?;

        // Add return
        self.builder.build_return(None)
            .map_err(|e| format!("Failed to build return: {}", e))?;

        // Verify the function
        if !function.verify(true) {
            return Err(format!("Function verification failed"));
        }

        // Run optimization passes on the function
        self.optimize_function(function)?;

        // Get the function pointer from the execution engine
        unsafe {
            let jit_fn = self.execution_engine.get_function::<JITFunction>(name)
                .map_err(|e| format!("Failed to get JIT function: {}", e))?;
            Ok(jit_fn.as_raw())
        }
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
                // Check if this is a recursive call to the function being compiled
                let current_fn_name = self.current_function_name.borrow();
                if let Some(ref fn_name) = *current_fn_name {
                    if word.to_uppercase() == *fn_name {
                        // This is a recursive call - generate function call to self
                        self.compile_recursive_call(function)?;
                        return Ok(());
                    }
                }
                drop(current_fn_name);  // Release the borrow

                // Check if this word has been JIT-compiled already
                // If so, call it directly instead of using primitives or interpreter
                let word_upper = word.to_uppercase();
                let is_jit_compiled = {
                    let registry = JIT_FUNCTION_REGISTRY.lock()
                        .map_err(|e| format!("Failed to lock JIT registry: {}", e))?;
                    registry.contains_key(&word_upper)
                };

                if is_jit_compiled {
                    // This word has been JIT-compiled - call it directly
                    self.compile_jit_word_call(function, &word_upper)?;
                    return Ok(());
                }

                match word.as_str() {
                    "*" => self.compile_multiply(function)?,
                    "DUP" => self.compile_external_call(function, "quarter_dup")?,
                    "DROP" => self.compile_external_call(function, "quarter_drop")?,
                    "SWAP" => self.compile_external_call(function, "quarter_swap")?,
                    "+" => self.compile_external_call(function, "quarter_add")?,
                    "-" => self.compile_external_call(function, "quarter_sub")?,
                    "/" => self.compile_external_call(function, "quarter_div")?,
                    "<" => self.compile_external_call(function, "quarter_less_than")?,
                    "I" => {
                        // Push the current loop index onto the stack
                        let loop_stack = self.loop_index_stack.borrow();
                        if let Some(loop_index) = loop_stack.last() {
                            self.compile_push_value(function, *loop_index)?;
                        } else {
                            return Err("I word used outside of DO/LOOP".to_string());
                        }
                    }
                    _ => return Err(format!("Unsupported word in JIT compilation: {}", word))
                }
                Ok(())
            }
            AstNode::Sequence(nodes) => {
                for node in nodes {
                    self.compile_ast_node(node, function)?;
                }
                Ok(())
            }
            AstNode::IfThenElse { then_branch, else_branch } => {
                self.compile_if_then_else(function, then_branch, else_branch.as_deref())?;
                Ok(())
            }
            AstNode::BeginUntil { body } => {
                self.compile_begin_until(function, body)?;
                Ok(())
            }
            AstNode::DoLoop { body, increment } => {
                self.compile_do_loop(function, body, *increment)?;
                Ok(())
            }
            _ => Err(format!("Unsupported AST node: {:?}", node))
        }
    }

    /// Compile a stack push operation
    /// Pushes value onto the stack and increments stack pointer
    fn compile_push(&self, function: inkwell::values::FunctionValue<'ctx>, value: i32) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i32_type = self.context.i32_type();
        let i64_type = self.context.i64_type(); // usize on 64-bit systems

        // Load current sp value: sp = *sp_ptr
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        // Calculate address in memory: addr = memory + sp
        let addr = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp], "addr")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };

        // Create the constant value
        let const_value = i32_type.const_int(value as u64, true);

        // Store value at address: *(i32*)addr = value
        let addr_i32 = self.builder.build_pointer_cast(addr, self.context.ptr_type(AddressSpace::default()), "addr_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        self.builder.build_store(addr_i32, const_value)
            .map_err(|e| format!("Failed to store value: {}", e))?;

        // Increment sp: sp += 4
        let four = i64_type.const_int(4, false);
        let new_sp = self.builder.build_int_add(sp, four, "new_sp")
            .map_err(|e| format!("Failed to add: {}", e))?;

        // Store new sp: *sp_ptr = new_sp
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a stack push operation for a dynamic value
    /// Pushes an IntValue onto the stack and increments stack pointer
    fn compile_push_value(&self, function: inkwell::values::FunctionValue<'ctx>, value: inkwell::values::IntValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type(); // usize on 64-bit systems

        // Load current sp value: sp = *sp_ptr
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        // Calculate address in memory: addr = memory + sp
        let addr = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp], "addr")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };

        // Store value at address: *(i32*)addr = value
        let addr_i32 = self.builder.build_pointer_cast(addr, self.context.ptr_type(AddressSpace::default()), "addr_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        self.builder.build_store(addr_i32, value)
            .map_err(|e| format!("Failed to store value: {}", e))?;

        // Increment sp: sp += 4
        let four = i64_type.const_int(4, false);
        let new_sp = self.builder.build_int_add(sp, four, "new_sp")
            .map_err(|e| format!("Failed to add: {}", e))?;

        // Store new sp: *sp_ptr = new_sp
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a stack pop operation
    /// Pops a value from the stack and decrements stack pointer
    /// Returns the popped value
    fn compile_pop(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i32_type = self.context.i32_type();
        let i64_type = self.context.i64_type(); // usize on 64-bit systems

        // Load current sp value: sp = *sp_ptr
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        // Decrement sp: sp -= 4
        let four = i64_type.const_int(4, false);
        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to sub: {}", e))?;

        // Store new sp: *sp_ptr = new_sp
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        // Calculate address in memory: addr = memory + new_sp
        let addr = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[new_sp], "addr")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };

        // Load value from address: value = *(i32*)addr
        let addr_i32 = self.builder.build_pointer_cast(addr, self.context.ptr_type(AddressSpace::default()), "addr_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let value = self.builder.build_load(i32_type, addr_i32, "value")
            .map_err(|e| format!("Failed to load value: {}", e))?
            .into_int_value();

        Ok(value)
    }

    /// Compile IF/THEN/ELSE control flow
    /// Pops a value from stack and branches based on whether it's non-zero
    fn compile_if_then_else(
        &self,
        function: inkwell::values::FunctionValue<'ctx>,
        then_branch: &[AstNode],
        else_branch: Option<&[AstNode]>,
    ) -> Result<(), String> {
        // Pop the condition value from the stack
        let cond_value = self.compile_pop(function)?;

        // Compare to zero (Forth: 0 is false, non-zero is true)
        let i32_type = self.context.i32_type();
        let zero = i32_type.const_int(0, false);
        let cond = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            cond_value,
            zero,
            "cond"
        ).map_err(|e| format!("Failed to build comparison: {}", e))?;

        // Create basic blocks
        let then_block = self.context.append_basic_block(function, "then");
        let merge_block = self.context.append_basic_block(function, "merge");

        if let Some(_) = else_branch {
            // We have an ELSE branch
            let else_block = self.context.append_basic_block(function, "else");

            // Branch based on condition
            self.builder.build_conditional_branch(cond, then_block, else_block)
                .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

            // Compile THEN branch
            self.builder.position_at_end(then_block);
            for node in then_branch {
                self.compile_ast_node(node, function)?;
            }
            self.builder.build_unconditional_branch(merge_block)
                .map_err(|e| format!("Failed to build branch: {}", e))?;

            // Compile ELSE branch
            self.builder.position_at_end(else_block);
            if let Some(else_nodes) = else_branch {
                for node in else_nodes {
                    self.compile_ast_node(node, function)?;
                }
            }
            self.builder.build_unconditional_branch(merge_block)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
        } else {
            // No ELSE branch - jump directly to merge if false
            self.builder.build_conditional_branch(cond, then_block, merge_block)
                .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

            // Compile THEN branch
            self.builder.position_at_end(then_block);
            for node in then_branch {
                self.compile_ast_node(node, function)?;
            }
            self.builder.build_unconditional_branch(merge_block)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
        }

        // Continue at merge block
        self.builder.position_at_end(merge_block);

        Ok(())
    }

    /// Compile BEGIN/UNTIL loop
    /// Executes body repeatedly until the condition on stack is true (non-zero)
    fn compile_begin_until(
        &self,
        function: inkwell::values::FunctionValue<'ctx>,
        body: &[AstNode],
    ) -> Result<(), String> {
        // Create basic blocks
        let loop_block = self.context.append_basic_block(function, "loop");
        let exit_block = self.context.append_basic_block(function, "exit");

        // Jump to loop
        self.builder.build_unconditional_branch(loop_block)
            .map_err(|e| format!("Failed to build branch: {}", e))?;

        // Compile loop body
        self.builder.position_at_end(loop_block);
        for node in body {
            self.compile_ast_node(node, function)?;
        }

        // Pop the condition value from the stack
        let cond_value = self.compile_pop(function)?;

        // Compare to zero (Forth: 0 is false/continue, non-zero is true/exit)
        let i32_type = self.context.i32_type();
        let zero = i32_type.const_int(0, false);
        let cond = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            cond_value,
            zero,
            "cond"
        ).map_err(|e| format!("Failed to build comparison: {}", e))?;

        // If condition is true (non-zero), exit. If false (zero), loop back.
        self.builder.build_conditional_branch(cond, exit_block, loop_block)
            .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

        // Continue at exit block
        self.builder.position_at_end(exit_block);

        Ok(())
    }

    /// Compile DO/LOOP construct
    /// Pops start and limit from stack, loops from start to limit-1
    fn compile_do_loop(
        &self,
        function: inkwell::values::FunctionValue<'ctx>,
        body: &[AstNode],
        increment: i32,
    ) -> Result<(), String> {
        let i32_type = self.context.i32_type();

        // Pop limit and start from stack
        let start = self.compile_pop(function)?;
        let limit = self.compile_pop(function)?;

        // Save the block where we computed start/limit (predecessor of loop)
        let preloop_block = self.builder.get_insert_block().unwrap();

        // Create basic blocks
        let loop_block = self.context.append_basic_block(function, "do_loop");
        let exit_block = self.context.append_basic_block(function, "do_exit");

        // Jump to loop
        self.builder.build_unconditional_branch(loop_block)
            .map_err(|e| format!("Failed to build branch: {}", e))?;

        // Compile loop block
        self.builder.position_at_end(loop_block);

        // Create phi node for loop index
        let phi = self.builder.build_phi(i32_type, "i")
            .map_err(|e| format!("Failed to build phi: {}", e))?;

        // Add incoming value from preloop block
        phi.add_incoming(&[(&start, preloop_block)]);

        let loop_index = phi.as_basic_value().into_int_value();

        // Push loop index onto the stack for I word access
        self.loop_index_stack.borrow_mut().push(loop_index);

        // Compile loop body
        for node in body {
            self.compile_ast_node(node, function)?;
        }

        // Pop loop index from the stack
        self.loop_index_stack.borrow_mut().pop();

        // Increment loop index
        let increment_val = i32_type.const_int(increment as u64, true);
        let next_index = self.builder.build_int_add(loop_index, increment_val, "i_next")
            .map_err(|e| format!("Failed to build add: {}", e))?;

        // Check if we should continue looping (next_index < limit)
        let continue_loop = self.builder.build_int_compare(
            inkwell::IntPredicate::SLT,
            next_index,
            limit,
            "continue"
        ).map_err(|e| format!("Failed to build comparison: {}", e))?;

        // Get the current block after compiling the body
        let loop_end_block = self.builder.get_insert_block().unwrap();

        // Add incoming value for phi from loop block
        phi.add_incoming(&[(&next_index, loop_end_block)]);

        // Branch based on condition
        self.builder.build_conditional_branch(continue_loop, loop_block, exit_block)
            .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

        // Continue at exit block
        self.builder.position_at_end(exit_block);

        Ok(())
    }

    /// Compile a call to an external Rust primitive
    fn compile_external_call(&self, function: inkwell::values::FunctionValue<'ctx>, primitive_name: &str) -> Result<(), String> {
        // Get the external function
        let external_fn = self.module.get_function(primitive_name)
            .ok_or_else(|| format!("External function {} not found", primitive_name))?;

        // Get the function parameters (memory, sp, rp)
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();
        let rp_ptr = function.get_nth_param(2).unwrap().into_pointer_value();

        // Call the external function
        self.builder.build_call(
            external_fn,
            &[memory_ptr.into(), sp_ptr.into(), rp_ptr.into()],
            "call"
        ).map_err(|e| format!("Failed to build call: {}", e))?;

        Ok(())
    }

    /// Compile a recursive call to the function being compiled
    fn compile_recursive_call(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        // Get the function parameters (memory, sp, rp)
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();
        let rp_ptr = function.get_nth_param(2).unwrap().into_pointer_value();

        // Call the function recursively with the same parameters
        self.builder.build_call(
            function,
            &[memory_ptr.into(), sp_ptr.into(), rp_ptr.into()],
            "recursive_call"
        ).map_err(|e| format!("Failed to build recursive call: {}", e))?;

        Ok(())
    }

    /// Compile a call to another JIT-compiled word
    /// This allows JIT-compiled words to call each other directly
    fn compile_jit_word_call(&self, function: inkwell::values::FunctionValue<'ctx>, word_name: &str) -> Result<(), String> {
        // Get the JIT-compiled function from the module
        let jit_fn = self.module.get_function(word_name)
            .ok_or_else(|| format!("JIT-compiled function {} not found in module", word_name))?;

        // Get the function parameters (memory, sp, rp)
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();
        let rp_ptr = function.get_nth_param(2).unwrap().into_pointer_value();

        // Call the JIT-compiled function
        self.builder.build_call(
            jit_fn,
            &[memory_ptr.into(), sp_ptr.into(), rp_ptr.into()],
            "jit_word_call"
        ).map_err(|e| format!("Failed to build JIT word call: {}", e))?;

        Ok(())
    }

    /// Compile a multiply operation
    /// Just calls the external quarter_mul function
    fn compile_multiply(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        self.compile_external_call(function, "quarter_mul")
    }

    /// Run LLVM optimization passes on a function
    /// Note: With LLVM 18+, most optimization is done by the execution engine
    /// at OptimizationLevel::Aggressive. The new pass manager handles optimizations
    /// differently than legacy passes.
    fn optimize_function(&self, _function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        // Optimization is handled by the execution engine's OptimizationLevel::Aggressive
        // No explicit pass manager needed with LLVM 18+
        Ok(())
    }

    /// Get the LLVM IR as a string (useful for debugging)
    pub fn get_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Verify the LLVM module for errors
    pub fn verify(&self) -> Result<(), String> {
        self.module
            .verify()
            .map_err(|e| format!("Module verification failed: {}", e))
    }
}

// Public API for JIT compilation with symbol registry

/// Register a JIT-compiled function in the global registry
/// This allows other words to call it
pub fn register_jit_function(name: String, jit_fn: JITFunction) -> Result<(), String> {
    let mut registry = JIT_FUNCTION_REGISTRY.lock()
        .map_err(|e| format!("Failed to lock JIT registry: {}", e))?;

    registry.insert(name.to_uppercase(), jit_fn);
    Ok(())
}

/// Check if a word has been JIT-compiled
pub fn is_jit_compiled(name: &str) -> bool {
    if let Ok(registry) = JIT_FUNCTION_REGISTRY.lock() {
        registry.contains_key(&name.to_uppercase())
    } else {
        false
    }
}

/// Clear all JIT function registrations
/// IMPORTANT: Only call this when you're sure no JIT functions are still in use
/// (e.g., between tests or when shutting down)
pub fn clear_jit_registry() {
    if let Ok(mut registry) = JIT_FUNCTION_REGISTRY.lock() {
        registry.clear();
    }
}

/// Get a JIT-compiled function by name from the registry
pub fn get_jit_function(name: &str) -> Option<JITFunction> {
    if let Ok(registry) = JIT_FUNCTION_REGISTRY.lock() {
        registry.get(&name.to_uppercase()).copied()
    } else {
        None
    }
}

