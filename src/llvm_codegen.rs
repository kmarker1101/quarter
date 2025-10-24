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
    // Stack of loop exit blocks for LEAVE support
    loop_exit_stack: RefCell<Vec<inkwell::basic_block::BasicBlock<'ctx>>>,
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
            loop_exit_stack: RefCell::new(Vec::new()),
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
    pub fn compile_word(&mut self, name: &str, ast: &AstNode, dict: &crate::dictionary::Dictionary) -> Result<JITFunction, String> {
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

        // Compile the AST - mark as tail position since it's the last operation before return
        self.compile_ast_node(ast, function, dict, true)?;

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
    ///
    /// The `is_tail_position` parameter indicates whether this node is in tail position,
    /// meaning it's the last operation before a return. This enables tail call optimization
    /// for recursive calls.
    fn compile_ast_node(&self, node: &AstNode, function: inkwell::values::FunctionValue<'ctx>, dict: &crate::dictionary::Dictionary, is_tail_position: bool) -> Result<(), String> {
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
                        // Mark as tail call if in tail position for optimization
                        self.compile_recursive_call(function, is_tail_position)?;
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

                // Check if this word is defined with an INLINE instruction
                // If so, compile it inline instead of calling a primitive
                if let Some(word_def) = dict.get_word(&word_upper) {
                    if let crate::dictionary::Word::Compiled(ast) = word_def {
                        // Check if the AST is a single InlineInstruction
                        if let AstNode::InlineInstruction(instruction) = ast {
                            self.compile_inline_instruction(function, &instruction)?;
                            return Ok(());
                        }
                    }
                }

                match word.as_str() {
                    "*" => self.compile_mul(function)?,
                    "DUP" => self.compile_dup(function)?,
                    "DROP" => self.compile_drop(function)?,
                    "SWAP" => self.compile_swap(function)?,
                    "+" => self.compile_add(function)?,
                    "-" => self.compile_sub(function)?,
                    "/" => self.compile_div(function)?,
                    "<" => self.compile_less_than(function)?,
                    ">" => self.compile_greater_than(function)?,
                    "=" => self.compile_equals(function)?,
                    "AND" => self.compile_and(function)?,
                    "OR" => self.compile_or(function)?,
                    "XOR" => self.compile_xor(function)?,
                    "INVERT" => self.compile_invert(function)?,
                    "LSHIFT" => self.compile_lshift(function)?,
                    "RSHIFT" => self.compile_rshift(function)?,
                    "OVER" => self.compile_over(function)?,
                    "ROT" => self.compile_rot(function)?,
                    "<=" => self.compile_less_equal(function)?,
                    ">=" => self.compile_greater_equal(function)?,
                    "<>" => self.compile_not_equals(function)?,
                    "MOD" => self.compile_mod(function)?,
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
                // Only the last node in a sequence is in tail position
                let len = nodes.len();
                for (i, node) in nodes.iter().enumerate() {
                    let is_last = i == len - 1;
                    self.compile_ast_node(node, function, dict, is_tail_position && is_last)?;
                }
                Ok(())
            }
            AstNode::IfThenElse { then_branch, else_branch } => {
                // Both branches can be in tail position
                self.compile_if_then_else(function, dict, then_branch, else_branch.as_deref(), is_tail_position)?;
                Ok(())
            }
            AstNode::BeginUntil { body } => {
                // Loop body is not in tail position (loop continues)
                self.compile_begin_until(function, dict, body)?;
                Ok(())
            }
            AstNode::DoLoop { body, increment } => {
                // Loop body is not in tail position (loop continues)
                self.compile_do_loop(function, dict, body, *increment)?;
                Ok(())
            }
            AstNode::Exit => {
                // EXIT - early return from word
                self.builder.build_return(None)
                    .map_err(|e| format!("Failed to build return: {}", e))?;
                Ok(())
            }
            AstNode::Leave => {
                // LEAVE - exit current loop early
                let exit_stack = self.loop_exit_stack.borrow();
                if let Some(exit_block) = exit_stack.last() {
                    self.builder.build_unconditional_branch(*exit_block)
                        .map_err(|e| format!("Failed to build branch: {}", e))?;
                    Ok(())
                } else {
                    Err("LEAVE used outside of loop".to_string())
                }
            }
            AstNode::InlineInstruction(instruction) => {
                // INLINE instruction - map to LLVM implementation
                self.compile_inline_instruction(function, instruction)?;
                Ok(())
            }
            _ => Err(format!("Unsupported AST node: {:?}", node))
        }
    }

    /// Compile an inline instruction by mapping instruction name to implementation
    /// This allows Forth-defined primitives to use LLVM implementations
    fn compile_inline_instruction(&self, function: inkwell::values::FunctionValue<'ctx>, instruction: &str) -> Result<(), String> {
        match instruction {
            "LLVM-ADD" => self.compile_add(function),
            "LLVM-SUB" => self.compile_sub(function),
            "LLVM-MUL" => self.compile_mul(function),
            "LLVM-DIV" => self.compile_div(function),
            "LLVM-DUP" => self.compile_dup(function),
            "LLVM-DROP" => self.compile_drop(function),
            "LLVM-SWAP" => self.compile_swap(function),
            "LLVM-OVER" => self.compile_over(function),
            "LLVM-ROT" => self.compile_rot(function),
            "LLVM-LT" => self.compile_less_than(function),
            "LLVM-GT" => self.compile_greater_than(function),
            "LLVM-EQ" => self.compile_equals(function),
            "LLVM-AND" => self.compile_and(function),
            "LLVM-OR" => self.compile_or(function),
            "LLVM-XOR" => self.compile_xor(function),
            "LLVM-INVERT" => self.compile_invert(function),
            "LLVM-LSHIFT" => self.compile_lshift(function),
            "LLVM-RSHIFT" => self.compile_rshift(function),
            _ => Err(format!("Unknown inline instruction: {}", instruction))
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
        dict: &crate::dictionary::Dictionary,
        then_branch: &[AstNode],
        else_branch: Option<&[AstNode]>,
        is_tail_position: bool,
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

            // Compile THEN branch - only last node is in tail position
            self.builder.position_at_end(then_block);
            let then_len = then_branch.len();
            for (i, node) in then_branch.iter().enumerate() {
                let is_last = i == then_len - 1;
                self.compile_ast_node(node, function, dict, is_tail_position && is_last)?;
            }
            // Only add branch to merge if block doesn't already have a terminator (like EXIT)
            let current_block = self.builder.get_insert_block().unwrap();
            if current_block.get_terminator().is_none() {
                self.builder.build_unconditional_branch(merge_block)
                    .map_err(|e| format!("Failed to build branch: {}", e))?;
            }

            // Compile ELSE branch - only last node is in tail position
            self.builder.position_at_end(else_block);
            if let Some(else_nodes) = else_branch {
                let else_len = else_nodes.len();
                for (i, node) in else_nodes.iter().enumerate() {
                    let is_last = i == else_len - 1;
                    self.compile_ast_node(node, function, dict, is_tail_position && is_last)?;
                }
            }
            // Only add branch to merge if block doesn't already have a terminator (like EXIT)
            let current_block = self.builder.get_insert_block().unwrap();
            if current_block.get_terminator().is_none() {
                self.builder.build_unconditional_branch(merge_block)
                    .map_err(|e| format!("Failed to build branch: {}", e))?;
            }
        } else {
            // No ELSE branch - jump directly to merge if false
            self.builder.build_conditional_branch(cond, then_block, merge_block)
                .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

            // Compile THEN branch - only last node is in tail position
            self.builder.position_at_end(then_block);
            let then_len = then_branch.len();
            for (i, node) in then_branch.iter().enumerate() {
                let is_last = i == then_len - 1;
                self.compile_ast_node(node, function, dict, is_tail_position && is_last)?;
            }
            // Only add branch to merge if block doesn't already have a terminator (like EXIT)
            let current_block = self.builder.get_insert_block().unwrap();
            if current_block.get_terminator().is_none() {
                self.builder.build_unconditional_branch(merge_block)
                    .map_err(|e| format!("Failed to build branch: {}", e))?;
            }
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
        dict: &crate::dictionary::Dictionary,
        body: &[AstNode],
    ) -> Result<(), String> {
        // Create basic blocks
        let loop_block = self.context.append_basic_block(function, "loop");
        let exit_block = self.context.append_basic_block(function, "exit");

        // Jump to loop
        self.builder.build_unconditional_branch(loop_block)
            .map_err(|e| format!("Failed to build branch: {}", e))?;

        // Push exit block for LEAVE support
        self.loop_exit_stack.borrow_mut().push(exit_block);

        // Compile loop body - not in tail position (loop continues)
        self.builder.position_at_end(loop_block);
        for node in body {
            self.compile_ast_node(node, function, dict, false)?;
        }

        // Pop exit block
        self.loop_exit_stack.borrow_mut().pop();

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
        dict: &crate::dictionary::Dictionary,
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

        // Push exit block for LEAVE support
        self.loop_exit_stack.borrow_mut().push(exit_block);

        // Compile loop body - not in tail position (loop continues)
        for node in body {
            self.compile_ast_node(node, function, dict, false)?;
        }

        // Pop exit block
        self.loop_exit_stack.borrow_mut().pop();

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
    #[allow(dead_code)]
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
    ///
    /// If `is_tail_call` is true, the call will be marked for tail call optimization,
    /// allowing LLVM to convert the recursion into a loop.
    fn compile_recursive_call(&self, function: inkwell::values::FunctionValue<'ctx>, is_tail_call: bool) -> Result<(), String> {
        // Get the function parameters (memory, sp, rp)
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();
        let rp_ptr = function.get_nth_param(2).unwrap().into_pointer_value();

        // Call the function recursively with the same parameters
        let call_site = self.builder.build_call(
            function,
            &[memory_ptr.into(), sp_ptr.into(), rp_ptr.into()],
            "recursive_call"
        ).map_err(|e| format!("Failed to build recursive call: {}", e))?;

        // Mark as tail call if in tail position - LLVM will optimize to a loop
        if is_tail_call {
            call_site.set_tail_call(true);
        }

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

    /// Compile an add operation with inlined LLVM instructions
    /// This replaces the external function call with direct LLVM add instruction
    fn compile_add(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        // Load current sp
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        // Pop two values: b at sp-4, a at sp-8
        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        // Calculate address for b (top of stack): addr_b = memory + sp - 4
        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        // Calculate address for a (second on stack): addr_a = memory + sp - 8
        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        // Add: result = a + b
        let result = self.builder.build_int_add(a, b, "result")
            .map_err(|e| format!("Failed to add: {}", e))?;

        // Store result at sp-8 (where a was)
        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        // Update sp: sp -= 4 (net effect: popped 2, pushed 1)
        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a subtract operation with inlined LLVM instructions
    fn compile_sub(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        // Load current sp
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        // Pop two values: b at sp-4, a at sp-8
        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        // Calculate address for b (top of stack): addr_b = memory + sp - 4
        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        // Calculate address for a (second on stack): addr_a = memory + sp - 8
        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        // Subtract: result = a - b
        let result = self.builder.build_int_sub(a, b, "result")
            .map_err(|e| format!("Failed to subtract: {}", e))?;

        // Store result at sp-8 (where a was)
        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        // Update sp: sp -= 4 (net effect: popped 2, pushed 1)
        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a multiply operation with inlined LLVM instructions
    fn compile_mul(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        // Load current sp
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        // Pop two values: b at sp-4, a at sp-8
        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        // Calculate address for b (top of stack): addr_b = memory + sp - 4
        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        // Calculate address for a (second on stack): addr_a = memory + sp - 8
        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        // Multiply: result = a * b
        let result = self.builder.build_int_mul(a, b, "result")
            .map_err(|e| format!("Failed to multiply: {}", e))?;

        // Store result at sp-8 (where a was)
        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        // Update sp: sp -= 4 (net effect: popped 2, pushed 1)
        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a divide operation with inlined LLVM instructions
    fn compile_div(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        // Load current sp
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        // Pop two values: b at sp-4, a at sp-8
        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        // Calculate address for b (top of stack): addr_b = memory + sp - 4
        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        // Calculate address for a (second on stack): addr_a = memory + sp - 8
        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        // Divide: result = a / b (signed division)
        let result = self.builder.build_int_signed_div(a, b, "result")
            .map_err(|e| format!("Failed to divide: {}", e))?;

        // Store result at sp-8 (where a was)
        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        // Update sp: sp -= 4 (net effect: popped 2, pushed 1)
        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a DUP operation with inlined LLVM instructions
    /// Stack effect: ( n -- n n )
    fn compile_dup(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        // Load current sp
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);

        // Load top value at sp-4
        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_top = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_top")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_top_i32 = self.builder.build_pointer_cast(addr_top, self.context.ptr_type(AddressSpace::default()), "addr_top_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let top_value = self.builder.build_load(i32_type, addr_top_i32, "top_value")
            .map_err(|e| format!("Failed to load top value: {}", e))?
            .into_int_value();

        // Store duplicate at sp
        let addr_new = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp], "addr_new")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_new_i32 = self.builder.build_pointer_cast(addr_new, self.context.ptr_type(AddressSpace::default()), "addr_new_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        self.builder.build_store(addr_new_i32, top_value)
            .map_err(|e| format!("Failed to store duplicate: {}", e))?;

        // Increment sp by 4
        let new_sp = self.builder.build_int_add(sp, four, "new_sp")
            .map_err(|e| format!("Failed to add: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a DROP operation with inlined LLVM instructions
    /// Stack effect: ( n -- )
    fn compile_drop(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();

        // Load current sp
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        // Decrement sp by 4 (remove top value)
        let four = i64_type.const_int(4, false);
        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a SWAP operation with inlined LLVM instructions
    /// Stack effect: ( a b -- b a )
    fn compile_swap(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        // Load current sp
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        // Load b (top of stack) at sp-4
        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        // Load a (second on stack) at sp-8
        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        // Store a where b was (sp-4)
        self.builder.build_store(addr_b_i32, a)
            .map_err(|e| format!("Failed to store a: {}", e))?;

        // Store b where a was (sp-8)
        self.builder.build_store(addr_a_i32, b)
            .map_err(|e| format!("Failed to store b: {}", e))?;

        Ok(())
    }

    /// Compile a less-than comparison with inlined LLVM instructions
    /// Stack effect: ( a b -- flag ) where flag is -1 (true) or 0 (false)
    fn compile_less_than(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        // Load current sp
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        // Pop two values: b at sp-4, a at sp-8
        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        // Compare: a < b (signed)
        let cmp_result = self.builder.build_int_compare(inkwell::IntPredicate::SLT, a, b, "cmp")
            .map_err(|e| format!("Failed to compare: {}", e))?;

        // Sign-extend i1 to i32 (false=0, true=-1)
        let result = self.builder.build_int_s_extend(cmp_result, i32_type, "result")
            .map_err(|e| format!("Failed to extend: {}", e))?;

        // Store result at sp-8 (where a was)
        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        // Update sp: sp -= 4 (net effect: popped 2, pushed 1)
        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a greater-than comparison with inlined LLVM instructions
    /// Stack effect: ( a b -- flag ) where flag is -1 (true) or 0 (false)
    fn compile_greater_than(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        // Load current sp
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        // Pop two values: b at sp-4, a at sp-8
        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        // Compare: a > b (signed)
        let cmp_result = self.builder.build_int_compare(inkwell::IntPredicate::SGT, a, b, "cmp")
            .map_err(|e| format!("Failed to compare: {}", e))?;

        // Sign-extend i1 to i32 (false=0, true=-1)
        let result = self.builder.build_int_s_extend(cmp_result, i32_type, "result")
            .map_err(|e| format!("Failed to extend: {}", e))?;

        // Store result at sp-8 (where a was)
        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        // Update sp: sp -= 4 (net effect: popped 2, pushed 1)
        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile an equals comparison with inlined LLVM instructions
    /// Stack effect: ( a b -- flag ) where flag is -1 (true) or 0 (false)
    fn compile_equals(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        // Load current sp
        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        // Pop two values: b at sp-4, a at sp-8
        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        // Compare: a == b
        let cmp_result = self.builder.build_int_compare(inkwell::IntPredicate::EQ, a, b, "cmp")
            .map_err(|e| format!("Failed to compare: {}", e))?;

        // Sign-extend i1 to i32 (false=0, true=-1)
        let result = self.builder.build_int_s_extend(cmp_result, i32_type, "result")
            .map_err(|e| format!("Failed to extend: {}", e))?;

        // Store result at sp-8 (where a was)
        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        // Update sp: sp -= 4 (net effect: popped 2, pushed 1)
        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile an AND operation with inlined LLVM instructions
    /// Stack effect: ( a b -- a&b )
    fn compile_and(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        let result = self.builder.build_and(a, b, "result")
            .map_err(|e| format!("Failed to AND: {}", e))?;

        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile an OR operation with inlined LLVM instructions
    /// Stack effect: ( a b -- a|b )
    fn compile_or(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        let result = self.builder.build_or(a, b, "result")
            .map_err(|e| format!("Failed to OR: {}", e))?;

        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile an XOR operation with inlined LLVM instructions
    /// Stack effect: ( a b -- a^b )
    fn compile_xor(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        let result = self.builder.build_xor(a, b, "result")
            .map_err(|e| format!("Failed to XOR: {}", e))?;

        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile an INVERT operation with inlined LLVM instructions
    /// Stack effect: ( n -- ~n ) - bitwise NOT
    fn compile_invert(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);

        // Load top value at sp-4
        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_i32 = self.builder.build_pointer_cast(addr, self.context.ptr_type(AddressSpace::default()), "addr_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let value = self.builder.build_load(i32_type, addr_i32, "value")
            .map_err(|e| format!("Failed to load value: {}", e))?
            .into_int_value();

        // Invert: XOR with -1 (all bits set)
        let neg_one = i32_type.const_int((-1i32) as u64, true);
        let result = self.builder.build_xor(value, neg_one, "result")
            .map_err(|e| format!("Failed to XOR: {}", e))?;

        self.builder.build_store(addr_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        Ok(())
    }

    /// Compile an LSHIFT operation with inlined LLVM instructions
    /// Stack effect: ( n shift -- n<<shift )
    fn compile_lshift(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let shift = self.builder.build_load(i32_type, addr_b_i32, "shift")
            .map_err(|e| format!("Failed to load shift: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let value = self.builder.build_load(i32_type, addr_a_i32, "value")
            .map_err(|e| format!("Failed to load value: {}", e))?
            .into_int_value();

        let result = self.builder.build_left_shift(value, shift, "result")
            .map_err(|e| format!("Failed to left shift: {}", e))?;

        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile an RSHIFT operation with inlined LLVM instructions
    /// Stack effect: ( n shift -- n>>shift ) - arithmetic right shift
    fn compile_rshift(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let shift = self.builder.build_load(i32_type, addr_b_i32, "shift")
            .map_err(|e| format!("Failed to load shift: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let value = self.builder.build_load(i32_type, addr_a_i32, "value")
            .map_err(|e| format!("Failed to load value: {}", e))?
            .into_int_value();

        let result = self.builder.build_right_shift(value, shift, true, "result")
            .map_err(|e| format!("Failed to right shift: {}", e))?;

        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile an OVER operation with inlined LLVM instructions
    /// Stack effect: ( a b -- a b a )
    fn compile_over(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        // Load second value (a) at sp-8
        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        // Store copy of a at sp (new top)
        let addr_new = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp], "addr_new")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_new_i32 = self.builder.build_pointer_cast(addr_new, self.context.ptr_type(AddressSpace::default()), "addr_new_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        self.builder.build_store(addr_new_i32, a)
            .map_err(|e| format!("Failed to store copy: {}", e))?;

        // Increment sp by 4
        let new_sp = self.builder.build_int_add(sp, four, "new_sp")
            .map_err(|e| format!("Failed to add: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a ROT operation with inlined LLVM instructions
    /// Stack effect: ( a b c -- b c a )
    fn compile_rot(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);
        let twelve = i64_type.const_int(12, false);

        // Load c (top) at sp-4
        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_c = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_c")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_c_i32 = self.builder.build_pointer_cast(addr_c, self.context.ptr_type(AddressSpace::default()), "addr_c_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let c = self.builder.build_load(i32_type, addr_c_i32, "c")
            .map_err(|e| format!("Failed to load c: {}", e))?
            .into_int_value();

        // Load b at sp-8
        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        // Load a at sp-12
        let sp_minus_12 = self.builder.build_int_sub(sp, twelve, "sp_minus_12")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_12], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        // Store b at sp-12 (where a was)
        self.builder.build_store(addr_a_i32, b)
            .map_err(|e| format!("Failed to store b: {}", e))?;

        // Store c at sp-8 (where b was)
        self.builder.build_store(addr_b_i32, c)
            .map_err(|e| format!("Failed to store c: {}", e))?;

        // Store a at sp-4 (where c was)
        self.builder.build_store(addr_c_i32, a)
            .map_err(|e| format!("Failed to store a: {}", e))?;

        Ok(())
    }

    /// Compile a <= comparison with inlined LLVM instructions
    /// Stack effect: ( a b -- flag ) where flag is -1 (true) or 0 (false)
    fn compile_less_equal(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        let cmp_result = self.builder.build_int_compare(inkwell::IntPredicate::SLE, a, b, "cmp")
            .map_err(|e| format!("Failed to compare: {}", e))?;

        let result = self.builder.build_int_s_extend(cmp_result, i32_type, "result")
            .map_err(|e| format!("Failed to extend: {}", e))?;

        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a >= comparison with inlined LLVM instructions
    /// Stack effect: ( a b -- flag ) where flag is -1 (true) or 0 (false)
    fn compile_greater_equal(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        let cmp_result = self.builder.build_int_compare(inkwell::IntPredicate::SGE, a, b, "cmp")
            .map_err(|e| format!("Failed to compare: {}", e))?;

        let result = self.builder.build_int_s_extend(cmp_result, i32_type, "result")
            .map_err(|e| format!("Failed to extend: {}", e))?;

        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a <> (not equal) comparison with inlined LLVM instructions
    /// Stack effect: ( a b -- flag ) where flag is -1 (true) or 0 (false)
    fn compile_not_equals(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        let cmp_result = self.builder.build_int_compare(inkwell::IntPredicate::NE, a, b, "cmp")
            .map_err(|e| format!("Failed to compare: {}", e))?;

        let result = self.builder.build_int_s_extend(cmp_result, i32_type, "result")
            .map_err(|e| format!("Failed to extend: {}", e))?;

        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Compile a MOD operation with inlined LLVM instructions
    /// Stack effect: ( a b -- a%b )
    fn compile_mod(&self, function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let sp_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let sp = self.builder.build_load(i64_type, sp_ptr, "sp")
            .map_err(|e| format!("Failed to load sp: {}", e))?
            .into_int_value();

        let four = i64_type.const_int(4, false);
        let eight = i64_type.const_int(8, false);

        let sp_minus_4 = self.builder.build_int_sub(sp, four, "sp_minus_4")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_b = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_4], "addr_b")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_b_i32 = self.builder.build_pointer_cast(addr_b, self.context.ptr_type(AddressSpace::default()), "addr_b_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let b = self.builder.build_load(i32_type, addr_b_i32, "b")
            .map_err(|e| format!("Failed to load b: {}", e))?
            .into_int_value();

        let sp_minus_8 = self.builder.build_int_sub(sp, eight, "sp_minus_8")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        let addr_a = unsafe {
            self.builder.build_gep(self.context.i8_type(), memory_ptr, &[sp_minus_8], "addr_a")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };
        let addr_a_i32 = self.builder.build_pointer_cast(addr_a, self.context.ptr_type(AddressSpace::default()), "addr_a_i32")
            .map_err(|e| format!("Failed to cast pointer: {}", e))?;
        let a = self.builder.build_load(i32_type, addr_a_i32, "a")
            .map_err(|e| format!("Failed to load a: {}", e))?
            .into_int_value();

        let result = self.builder.build_int_signed_rem(a, b, "result")
            .map_err(|e| format!("Failed to compute remainder: {}", e))?;

        self.builder.build_store(addr_a_i32, result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        let new_sp = self.builder.build_int_sub(sp, four, "new_sp")
            .map_err(|e| format!("Failed to subtract: {}", e))?;
        self.builder.build_store(sp_ptr, new_sp)
            .map_err(|e| format!("Failed to store sp: {}", e))?;

        Ok(())
    }

    /// Run LLVM optimization passes on a function
    ///
    /// **LLVM 18+ Note**: The legacy pass manager was removed in LLVM 17+. All optimization
    /// is now handled automatically by the execution engine's OptimizationLevel::Aggressive
    /// setting through the new pass manager. This includes:
    ///
    /// - **Instruction combining**: Merges redundant operations
    /// - **GVN**: Global value numbering for redundancy elimination
    /// - **SCCP**: Sparse conditional constant propagation
    /// - **Dead code elimination**: Removes unused instructions
    /// - **Inlining**: Function inlining where beneficial
    /// - **CFG simplification**: Control flow optimization
    ///
    /// The execution engine applies these optimizations automatically when functions
    /// are JIT-compiled, so no explicit pass manager setup is needed.
    ///
    /// **Performance Impact**: The inline primitives we generate (direct LLVM instructions
    /// for +, -, *, DUP, etc.) are optimized by LLVM's optimizer automatically, eliminating
    /// redundant loads/stores and performing constant folding at JIT time.
    fn optimize_function(&self, _function: inkwell::values::FunctionValue<'ctx>) -> Result<(), String> {
        // With LLVM 18+, optimization is handled automatically by the execution engine's
        // OptimizationLevel::Aggressive setting. The legacy PassManager API is not
        // available in LLVM 17+, as the new pass manager handles all optimizations
        // transparently at JIT compilation time.
        //
        // This function is kept for API compatibility and future extensibility, but
        // performs no explicit optimization operations.
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

