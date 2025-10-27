// LLVM primitives exposed to Forth for self-hosting compiler
//
// This module provides a handle-based API for LLVM that Forth code can use.
// Forth manipulates integer handles (IDs) and Rust maintains the actual LLVM objects.

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::execution_engine::ExecutionEngine;

// Macro to create symbol array without repetitive 'as usize' casts
// Usage: symbol_array!(func1, func2, func3, ...)
macro_rules! symbol_array {
    ($($func:path),* $(,)?) => {
        [
            $($func as usize),*
        ]
    };
}

// Force quarter_ symbols to be included in binary
// This function references all quarter_ functions so the linker includes them
#[inline(never)]
fn register_quarter_symbols() -> usize {
    let symbols = symbol_array!(
        // Stack operations
        crate::words::quarter_dup,
        crate::words::quarter_drop,
        crate::words::quarter_swap,
        crate::words::quarter_over,
        crate::words::quarter_rot,
        crate::words::quarter_pick,
        crate::words::quarter_depth,

        // Arithmetic operations
        crate::words::quarter_add,
        crate::words::quarter_sub,
        crate::words::quarter_mul,
        crate::words::quarter_div,
        crate::words::quarter_slash_mod,
        crate::words::quarter_negate,
        crate::words::quarter_abs,

        // Comparison operations
        crate::words::quarter_less_than,
        crate::words::quarter_lt,
        crate::words::quarter_gt,
        crate::words::quarter_equal,
        crate::words::quarter_not_equal,
        crate::words::quarter_less_equal,
        crate::words::quarter_greater_equal,
        crate::words::quarter_min,
        crate::words::quarter_max,
        crate::words::quarter_1plus,
        crate::words::quarter_1minus,
        crate::words::quarter_2star,
        crate::words::quarter_2slash,

        // Memory operations
        crate::words::quarter_store,
        crate::words::quarter_fetch,
        crate::words::quarter_c_store,
        crate::words::quarter_c_fetch,

        // Bitwise operations
        crate::words::quarter_and,
        crate::words::quarter_or,
        crate::words::quarter_xor,
        crate::words::quarter_invert,
        crate::words::quarter_lshift,
        crate::words::quarter_rshift,

        // Return stack operations
        crate::words::quarter_to_r,
        crate::words::quarter_r_from,
        crate::words::quarter_r_fetch,

        // Loop operations
        crate::words::quarter_i,
        crate::words::quarter_j,

        // I/O operations
        crate::words::quarter_emit,
        crate::words::quarter_space,
        crate::words::quarter_key,
        crate::words::quarter_cr,
        crate::words::quarter_dot,
        crate::words::quarter_u_dot,
        crate::words::quarter_dot_r,
        crate::words::quarter_u_dot_r,
        crate::words::quarter_type,

        // Stack pointer operations
        crate::words::quarter_sp_fetch,
        crate::words::quarter_sp_store,
        crate::words::quarter_rp_fetch,
        crate::words::quarter_rp_store,

        // Memory allocation operations
        crate::words::quarter_here,
        crate::words::quarter_allot,
        crate::words::quarter_comma,
    );
    symbols[0] // Return something to prevent optimization
}
use inkwell::values::{FunctionValue, BasicValueEnum, PhiValue, BasicValue};
use inkwell::basic_block::BasicBlock;
use inkwell::types::BasicType;
use inkwell::OptimizationLevel;
use inkwell::AddressSpace;
use std::collections::HashMap;
use std::cell::RefCell;

thread_local! {
    /// Thread-local registry of LLVM objects accessible from Forth
    /// Using thread_local because LLVM types are not Send/Sync
    static LLVM_REGISTRY: RefCell<LLVMRegistry> = RefCell::new(LLVMRegistry::new());
}

/// Handle types for different LLVM objects
pub type ContextHandle = i64;
pub type ModuleHandle = i64;
pub type BuilderHandle = i64;
pub type FunctionHandle = i64;
pub type BlockHandle = i64;
pub type ValueHandle = i64;
pub type EngineHandle = i64;

/// Registry storing all LLVM objects with handles
pub struct LLVMRegistry {
    next_id: i64,

    // LLVM objects stored with their handles
    contexts: HashMap<ContextHandle, &'static Context>,
    modules: HashMap<ModuleHandle, Box<Module<'static>>>,
    builders: HashMap<BuilderHandle, Box<Builder<'static>>>,
    functions: HashMap<FunctionHandle, FunctionValue<'static>>,
    blocks: HashMap<BlockHandle, BasicBlock<'static>>,
    values: HashMap<ValueHandle, BasicValueEnum<'static>>,
    engines: HashMap<EngineHandle, Box<ExecutionEngine<'static>>>,
    phis: HashMap<ValueHandle, PhiValue<'static>>,
}

impl LLVMRegistry {
    fn new() -> Self {
        LLVMRegistry {
            next_id: 1,
            contexts: HashMap::new(),
            modules: HashMap::new(),
            builders: HashMap::new(),
            functions: HashMap::new(),
            blocks: HashMap::new(),
            values: HashMap::new(),
            engines: HashMap::new(),
            phis: HashMap::new(),
        }
    }

    fn next_handle(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Create a new LLVM context and return its handle
    pub fn create_context(&mut self) -> ContextHandle {
        let context = Box::leak(Box::new(Context::create()));
        let handle = self.next_handle();
        self.contexts.insert(handle, context);
        handle
    }

    /// Create a new module in the given context
    pub fn create_module(&mut self, ctx_handle: ContextHandle, name: &str) -> Result<ModuleHandle, String> {
        let context = self.contexts.get(&ctx_handle)
            .ok_or_else(|| format!("Invalid context handle: {}", ctx_handle))?;

        let module = context.create_module(name);
        let handle = self.next_handle();
        self.modules.insert(handle, Box::new(module));
        Ok(handle)
    }

    /// Create a new builder in the given context
    pub fn create_builder(&mut self, ctx_handle: ContextHandle) -> Result<BuilderHandle, String> {
        let context = self.contexts.get(&ctx_handle)
            .ok_or_else(|| format!("Invalid context handle: {}", ctx_handle))?;

        let builder = context.create_builder();
        let handle = self.next_handle();
        self.builders.insert(handle, Box::new(builder));
        Ok(handle)
    }

    /// Declare an external function (for calling primitives)
    /// All primitives have signature: void fn(u8* memory, usize* sp, usize* rp)
    pub fn declare_external_function(&mut self,
                                     module_handle: ModuleHandle,
                                     ctx_handle: ContextHandle,
                                     name: &str) -> Result<(), String> {
        let context = self.contexts.get(&ctx_handle)
            .ok_or_else(|| format!("Invalid context handle: {}", ctx_handle))?;
        let module = self.modules.get_mut(&module_handle)
            .ok_or_else(|| format!("Invalid module handle: {}", module_handle))?;

        // Create function type: void(i8*, i64*, i64*)
        // Note: LLVM 15+ doesn't differentiate between pointer types
        let void_type = context.void_type();
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

        let param_types = &[
            ptr_type.into(),   // memory pointer
            ptr_type.into(),   // sp pointer
            ptr_type.into(),   // rp pointer
        ];

        let fn_type = void_type.fn_type(param_types, false);

        // Add function declaration to module
        module.add_function(name, fn_type, None);

        Ok(())
    }

    /// Create a new function in the given module
    pub fn create_function(&mut self,
                          module_handle: ModuleHandle,
                          ctx_handle: ContextHandle,
                          name: &str) -> Result<FunctionHandle, String> {
        let context = self.contexts.get(&ctx_handle)
            .ok_or_else(|| format!("Invalid context handle: {}", ctx_handle))?;

        let module = self.modules.get(&module_handle)
            .ok_or_else(|| format!("Invalid module handle: {}", module_handle))?;

        // Create function signature: void fn(u8* memory, usize* sp, usize* rp)
        let ptr_type = context.ptr_type(AddressSpace::default());
        let void_type = context.void_type();
        let fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);

        let function = module.add_function(name, fn_type, None);
        let handle = self.next_handle();
        self.functions.insert(handle, function);
        Ok(handle)
    }

    /// Get an existing function from the module by name
    pub fn get_function(&mut self,
                       module_handle: ModuleHandle,
                       name: &str) -> Result<FunctionHandle, String> {
        let module = self.modules.get(&module_handle)
            .ok_or_else(|| format!("Invalid module handle: {}", module_handle))?;

        let function = module.get_function(name)
            .ok_or_else(|| format!("Function '{}' not found in module", name))?;

        let handle = self.next_handle();
        self.functions.insert(handle, function);
        Ok(handle)
    }

    /// Create a basic block in the given function
    pub fn create_block(&mut self,
                        ctx_handle: ContextHandle,
                        fn_handle: FunctionHandle,
                        name: &str) -> Result<BlockHandle, String> {
        let context = self.contexts.get(&ctx_handle)
            .ok_or_else(|| format!("Invalid context handle: {}", ctx_handle))?;

        let function = self.functions.get(&fn_handle)
            .ok_or_else(|| format!("Invalid function handle: {}", fn_handle))?;

        let block = context.append_basic_block(*function, name);
        let handle = self.next_handle();
        self.blocks.insert(handle, block);
        Ok(handle)
    }

    /// Position builder at the end of a basic block
    pub fn position_at_end(&mut self,
                          builder_handle: BuilderHandle,
                          block_handle: BlockHandle) -> Result<(), String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let block = self.blocks.get(&block_handle)
            .ok_or_else(|| format!("Invalid block handle: {}", block_handle))?;

        builder.position_at_end(*block);
        Ok(())
    }

    /// Get a function parameter as a value
    pub fn get_param(&mut self,
                     fn_handle: FunctionHandle,
                     index: u32) -> Result<ValueHandle, String> {
        let function = self.functions.get(&fn_handle)
            .ok_or_else(|| format!("Invalid function handle: {}", fn_handle))?;

        let param = function.get_nth_param(index)
            .ok_or_else(|| format!("Function has no parameter at index {}", index))?;

        let handle = self.next_handle();
        self.values.insert(handle, param);
        Ok(handle)
    }

    /// Build a return void instruction
    pub fn build_ret_void(&mut self, builder_handle: BuilderHandle) -> Result<(), String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        builder.build_return(None)
            .map_err(|e| format!("Failed to build return: {}", e))?;
        Ok(())
    }

    pub fn build_ret(&mut self, builder_handle: BuilderHandle, value_handle: ValueHandle) -> Result<(), String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;
        let value = self.values.get(&value_handle)
            .ok_or_else(|| format!("Invalid value handle: {}", value_handle))?;

        builder.build_return(Some(value))
            .map_err(|e| format!("Failed to build return: {}", e))?;
        Ok(())
    }

    /// Dump module IR to string
    pub fn dump_module_ir(&self, module_handle: ModuleHandle) -> Result<String, String> {
        let module = self.modules.get(&module_handle)
            .ok_or_else(|| format!("Invalid module handle: {}", module_handle))?;

        Ok(module.print_to_string().to_string())
    }

    /// Create a JIT execution engine for the module
    pub fn create_jit_engine(&mut self,
                            module_handle: ModuleHandle) -> Result<EngineHandle, String> {
        let module = self.modules.remove(&module_handle)
            .ok_or_else(|| format!("Invalid module handle: {}", module_handle))?;

        let engine = module.create_jit_execution_engine(OptimizationLevel::Aggressive)
            .map_err(|e| format!("Failed to create JIT engine: {}", e))?;

        // Call function to ensure quarter_ symbols are included in binary
        // LLVM MCJIT on macOS should then automatically resolve them from the current process
        let _ = register_quarter_symbols();

        let handle = self.next_handle();
        self.engines.insert(handle, Box::new(engine));

        Ok(handle)
    }

    /// Get a JIT-compiled function pointer
    pub fn get_jit_function(&self,
                           engine_handle: EngineHandle,
                           name: &str) -> Result<usize, String> {
        let engine = self.engines.get(&engine_handle)
            .ok_or_else(|| format!("Invalid engine handle: {}", engine_handle))?;

        unsafe {
            let jit_fn = engine.get_function::<unsafe extern "C" fn(*mut u8, *mut usize, *mut usize)>(name)
                .map_err(|e| format!("Failed to get JIT function: {}", e))?;
            Ok(jit_fn.as_raw() as usize)
        }
    }

    /// Build a constant integer
    pub fn build_const_int(&mut self,
                          ctx_handle: ContextHandle,
                          value: i64,
                          bit_width: i64) -> Result<ValueHandle, String> {
        let context = self.contexts.get(&ctx_handle)
            .ok_or_else(|| format!("Invalid context handle: {}", ctx_handle))?;

        // Select type based on bit width
        let const_val = match bit_width {
            32 => {
                let i32_type = context.i32_type();
                i32_type.const_int(value as u64, true).into()
            },
            64 => {
                let i64_type = context.i64_type();
                i64_type.const_int(value as u64, true).into()
            },
            _ => return Err(format!("Unsupported bit width for const int: {}", bit_width)),
        };

        let handle = self.next_handle();
        self.values.insert(handle, const_val);
        Ok(handle)
    }

    /// Build load instruction
    pub fn build_load(&mut self,
                     builder_handle: BuilderHandle,
                     ctx_handle: ContextHandle,
                     ptr_handle: ValueHandle,
                     bit_width: i64) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let context = self.contexts.get(&ctx_handle)
            .ok_or_else(|| format!("Invalid context handle: {}", ctx_handle))?;

        let ptr_val = self.values.get(&ptr_handle)
            .ok_or_else(|| format!("Invalid value handle: {}", ptr_handle))?;

        let ptr = ptr_val.into_pointer_value();

        // Select type based on bit width
        let load_type = match bit_width {
            8 => context.i8_type().as_basic_type_enum(),
            32 => context.i32_type().as_basic_type_enum(),
            64 => context.i64_type().as_basic_type_enum(),
            _ => return Err(format!("Unsupported bit width for load: {}", bit_width)),
        };

        let loaded = builder.build_load(load_type, ptr, "loaded")
            .map_err(|e| format!("Failed to build load: {}", e))?;

        // Set alignment based on bit width (i64 = 4 bytes, i64 = 8 bytes)
        let alignment = (bit_width / 8) as u32;
        if let Some(instruction) = loaded.as_instruction_value() {
            instruction.set_alignment(alignment)
                .map_err(|e| format!("Failed to set load alignment: {}", e))?;
        }

        let handle = self.next_handle();
        self.values.insert(handle, loaded);
        Ok(handle)
    }

    /// Build store instruction
    pub fn build_store(&mut self,
                      builder_handle: BuilderHandle,
                      value_handle: ValueHandle,
                      ptr_handle: ValueHandle) -> Result<(), String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let value = self.values.get(&value_handle)
            .ok_or_else(|| format!("Invalid value handle: {}", value_handle))?;

        let ptr_val = self.values.get(&ptr_handle)
            .ok_or_else(|| format!("Invalid pointer handle: {}", ptr_handle))?;

        let ptr = ptr_val.into_pointer_value();

        let store_inst = builder.build_store(ptr, *value)
            .map_err(|e| format!("Failed to build store: {}", e))?;

        // Set alignment based on value type
        if value.is_int_value() {
            let int_val = value.into_int_value();
            let bit_width = int_val.get_type().get_bit_width();
            let alignment = bit_width / 8;
            store_inst.set_alignment(alignment)
                .map_err(|e| format!("Failed to set store alignment: {}", e))?;
        }

        Ok(())
    }

    /// Build GEP (GetElementPtr) instruction for pointer arithmetic
    pub fn build_gep(&mut self,
                    builder_handle: BuilderHandle,
                    ctx_handle: ContextHandle,
                    ptr_handle: ValueHandle,
                    offset_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let context = self.contexts.get(&ctx_handle)
            .ok_or_else(|| format!("Invalid context handle: {}", ctx_handle))?;

        let ptr_val = self.values.get(&ptr_handle)
            .ok_or_else(|| format!("Invalid pointer handle: {}", ptr_handle))?;

        let offset_val = self.values.get(&offset_handle)
            .ok_or_else(|| format!("Invalid offset handle: {}", offset_handle))?;

        let ptr = ptr_val.into_pointer_value();
        let offset = offset_val.into_int_value();

        let result = unsafe {
            builder.build_gep(context.i8_type(), ptr, &[offset], "gep")
                .map_err(|e| format!("Failed to build GEP: {}", e))?
        };

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build integer add instruction
    pub fn build_add(&mut self,
                    builder_handle: BuilderHandle,
                    lhs_handle: ValueHandle,
                    rhs_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let lhs = self.values.get(&lhs_handle)
            .ok_or_else(|| format!("Invalid LHS handle: {}", lhs_handle))?
            .into_int_value();

        let rhs = self.values.get(&rhs_handle)
            .ok_or_else(|| format!("Invalid RHS handle: {}", rhs_handle))?
            .into_int_value();

        let result = builder.build_int_add(lhs, rhs, "add")
            .map_err(|e| format!("Failed to build add: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build integer sub instruction
    pub fn build_sub(&mut self,
                    builder_handle: BuilderHandle,
                    lhs_handle: ValueHandle,
                    rhs_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let lhs = self.values.get(&lhs_handle)
            .ok_or_else(|| format!("Invalid LHS handle: {}", lhs_handle))?
            .into_int_value();

        let rhs = self.values.get(&rhs_handle)
            .ok_or_else(|| format!("Invalid RHS handle: {}", rhs_handle))?
            .into_int_value();

        let result = builder.build_int_sub(lhs, rhs, "sub")
            .map_err(|e| format!("Failed to build sub: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build integer mul instruction
    pub fn build_mul(&mut self,
                    builder_handle: BuilderHandle,
                    lhs_handle: ValueHandle,
                    rhs_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let lhs = self.values.get(&lhs_handle)
            .ok_or_else(|| format!("Invalid LHS handle: {}", lhs_handle))?
            .into_int_value();

        let rhs = self.values.get(&rhs_handle)
            .ok_or_else(|| format!("Invalid RHS handle: {}", rhs_handle))?
            .into_int_value();

        let result = builder.build_int_mul(lhs, rhs, "mul")
            .map_err(|e| format!("Failed to build mul: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build integer signed div instruction
    pub fn build_sdiv(&mut self,
                     builder_handle: BuilderHandle,
                     lhs_handle: ValueHandle,
                     rhs_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let lhs = self.values.get(&lhs_handle)
            .ok_or_else(|| format!("Invalid LHS handle: {}", lhs_handle))?
            .into_int_value();

        let rhs = self.values.get(&rhs_handle)
            .ok_or_else(|| format!("Invalid RHS handle: {}", rhs_handle))?
            .into_int_value();

        let result = builder.build_int_signed_div(lhs, rhs, "div")
            .map_err(|e| format!("Failed to build sdiv: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build integer signed remainder instruction
    pub fn build_srem(&mut self,
                     builder_handle: BuilderHandle,
                     lhs_handle: ValueHandle,
                     rhs_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let lhs = self.values.get(&lhs_handle)
            .ok_or_else(|| format!("Invalid LHS handle: {}", lhs_handle))?
            .into_int_value();

        let rhs = self.values.get(&rhs_handle)
            .ok_or_else(|| format!("Invalid RHS handle: {}", rhs_handle))?
            .into_int_value();

        let result = builder.build_int_signed_rem(lhs, rhs, "rem")
            .map_err(|e| format!("Failed to build srem: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build bitwise AND instruction
    pub fn build_and(&mut self,
                    builder_handle: BuilderHandle,
                    lhs_handle: ValueHandle,
                    rhs_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let lhs = self.values.get(&lhs_handle)
            .ok_or_else(|| format!("Invalid LHS handle: {}", lhs_handle))?
            .into_int_value();

        let rhs = self.values.get(&rhs_handle)
            .ok_or_else(|| format!("Invalid RHS handle: {}", rhs_handle))?
            .into_int_value();

        let result = builder.build_and(lhs, rhs, "and")
            .map_err(|e| format!("Failed to build and: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build bitwise OR instruction
    pub fn build_or(&mut self,
                   builder_handle: BuilderHandle,
                   lhs_handle: ValueHandle,
                   rhs_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let lhs = self.values.get(&lhs_handle)
            .ok_or_else(|| format!("Invalid LHS handle: {}", lhs_handle))?
            .into_int_value();

        let rhs = self.values.get(&rhs_handle)
            .ok_or_else(|| format!("Invalid RHS handle: {}", rhs_handle))?
            .into_int_value();

        let result = builder.build_or(lhs, rhs, "or")
            .map_err(|e| format!("Failed to build or: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build bitwise XOR instruction
    pub fn build_xor(&mut self,
                    builder_handle: BuilderHandle,
                    lhs_handle: ValueHandle,
                    rhs_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let lhs = self.values.get(&lhs_handle)
            .ok_or_else(|| format!("Invalid LHS handle: {}", lhs_handle))?
            .into_int_value();

        let rhs = self.values.get(&rhs_handle)
            .ok_or_else(|| format!("Invalid RHS handle: {}", rhs_handle))?
            .into_int_value();

        let result = builder.build_xor(lhs, rhs, "xor")
            .map_err(|e| format!("Failed to build xor: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build left shift instruction
    pub fn build_shl(&mut self,
                    builder_handle: BuilderHandle,
                    lhs_handle: ValueHandle,
                    rhs_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let lhs = self.values.get(&lhs_handle)
            .ok_or_else(|| format!("Invalid LHS handle: {}", lhs_handle))?
            .into_int_value();

        let rhs = self.values.get(&rhs_handle)
            .ok_or_else(|| format!("Invalid RHS handle: {}", rhs_handle))?
            .into_int_value();

        let result = builder.build_left_shift(lhs, rhs, "shl")
            .map_err(|e| format!("Failed to build shl: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build arithmetic right shift instruction (sign-preserving)
    pub fn build_ashr(&mut self,
                     builder_handle: BuilderHandle,
                     lhs_handle: ValueHandle,
                     rhs_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let lhs = self.values.get(&lhs_handle)
            .ok_or_else(|| format!("Invalid LHS handle: {}", lhs_handle))?
            .into_int_value();

        let rhs = self.values.get(&rhs_handle)
            .ok_or_else(|| format!("Invalid RHS handle: {}", rhs_handle))?
            .into_int_value();

        let result = builder.build_right_shift(lhs, rhs, true, "ashr")
            .map_err(|e| format!("Failed to build ashr: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build unconditional branch
    pub fn build_br(&mut self,
                   builder_handle: BuilderHandle,
                   block_handle: BlockHandle) -> Result<(), String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let block = self.blocks.get(&block_handle)
            .ok_or_else(|| format!("Invalid block handle: {}", block_handle))?;

        builder.build_unconditional_branch(*block)
            .map_err(|e| format!("Failed to build branch: {}", e))?;
        Ok(())
    }

    /// Build conditional branch
    pub fn build_cond_br(&mut self,
                        builder_handle: BuilderHandle,
                        cond_handle: ValueHandle,
                        then_block: BlockHandle,
                        else_block: BlockHandle) -> Result<(), String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let cond = self.values.get(&cond_handle)
            .ok_or_else(|| format!("Invalid condition handle: {}", cond_handle))?
            .into_int_value();

        let then_bb = self.blocks.get(&then_block)
            .ok_or_else(|| format!("Invalid then block handle: {}", then_block))?;

        let else_bb = self.blocks.get(&else_block)
            .ok_or_else(|| format!("Invalid else block handle: {}", else_block))?;

        builder.build_conditional_branch(cond, *then_bb, *else_bb)
            .map_err(|e| format!("Failed to build conditional branch: {}", e))?;
        Ok(())
    }

    /// Build integer comparison
    pub fn build_icmp(&mut self,
                     builder_handle: BuilderHandle,
                     predicate: i64,  // 0=eq, 1=ne, 2=slt, 3=sle, 4=sgt, 5=sge
                     lhs_handle: ValueHandle,
                     rhs_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let lhs = self.values.get(&lhs_handle)
            .ok_or_else(|| format!("Invalid LHS handle: {}", lhs_handle))?
            .into_int_value();

        let rhs = self.values.get(&rhs_handle)
            .ok_or_else(|| format!("Invalid RHS handle: {}", rhs_handle))?
            .into_int_value();

        use inkwell::IntPredicate;
        let pred = match predicate {
            0 => IntPredicate::EQ,
            1 => IntPredicate::NE,
            2 => IntPredicate::SLT,
            3 => IntPredicate::SLE,
            4 => IntPredicate::SGT,
            5 => IntPredicate::SGE,
            _ => return Err(format!("Invalid predicate: {}", predicate)),
        };

        let result = builder.build_int_compare(pred, lhs, rhs, "cmp")
            .map_err(|e| format!("Failed to build comparison: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build sign-extend instruction (i1 -> i64 for Forth booleans)
    pub fn build_sext(&mut self,
                     builder_handle: BuilderHandle,
                     ctx_handle: ContextHandle,
                     value_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let context = self.contexts.get(&ctx_handle)
            .ok_or_else(|| format!("Invalid context handle: {}", ctx_handle))?;

        let value = self.values.get(&value_handle)
            .ok_or_else(|| format!("Invalid value handle: {}", value_handle))?
            .into_int_value();

        let i64_type = context.i64_type();
        let result = builder.build_int_s_extend(value, i64_type, "sext")
            .map_err(|e| format!("Failed to build sext: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build select instruction (conditional selection)
    pub fn build_select(&mut self,
                       builder_handle: BuilderHandle,
                       cond_handle: ValueHandle,
                       true_handle: ValueHandle,
                       false_handle: ValueHandle) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let cond_value = self.values.get(&cond_handle)
            .ok_or_else(|| format!("Invalid condition handle: {}", cond_handle))?
            .into_int_value();

        let true_value = self.values.get(&true_handle)
            .ok_or_else(|| format!("Invalid true value handle: {}", true_handle))?
            .into_int_value();

        let false_value = self.values.get(&false_handle)
            .ok_or_else(|| format!("Invalid false value handle: {}", false_handle))?
            .into_int_value();

        let result = builder.build_select(cond_value, true_value, false_value, "select")
            .map_err(|e| format!("Failed to build select: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build truncate instruction (i64 -> i8 for byte operations)
    pub fn build_trunc(&mut self,
                      builder_handle: BuilderHandle,
                      ctx_handle: ContextHandle,
                      value_handle: ValueHandle,
                      bit_width: i64) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let context = self.contexts.get(&ctx_handle)
            .ok_or_else(|| format!("Invalid context handle: {}", ctx_handle))?;

        let value = self.values.get(&value_handle)
            .ok_or_else(|| format!("Invalid value handle: {}", value_handle))?
            .into_int_value();

        let target_type = match bit_width {
            8 => context.i8_type(),
            32 => context.i32_type(),
            _ => return Err(format!("Unsupported bit width for trunc: {}", bit_width)),
        };

        let result = builder.build_int_truncate(value, target_type, "trunc")
            .map_err(|e| format!("Failed to build trunc: {}", e))?;

        let handle = self.next_handle();
        self.values.insert(handle, result.into());
        Ok(handle)
    }

    /// Build function call
    pub fn build_call(&mut self,
                     builder_handle: BuilderHandle,
                     fn_handle: FunctionHandle,
                     args: &[ValueHandle],
                     is_tail_call: bool) -> Result<(), String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let function = self.functions.get(&fn_handle)
            .ok_or_else(|| format!("Invalid function handle: {}", fn_handle))?;

        // Convert argument handles to BasicMetadataValueEnum
        let mut arg_values = Vec::new();
        for &arg_handle in args {
            let val = self.values.get(&arg_handle)
                .ok_or_else(|| format!("Invalid argument handle: {}", arg_handle))?;
            arg_values.push((*val).into());
        }

        let call_site = builder.build_call(*function, &arg_values, "call")
            .map_err(|e| format!("Failed to build call: {}", e))?;

        // Mark as tail call if requested - LLVM will optimize to a loop
        if is_tail_call {
            call_site.set_tail_call(true);
        }

        Ok(())
    }

    /// Build PHI node (for SSA merges in loops)
    pub fn build_phi(&mut self,
                    builder_handle: BuilderHandle,
                    ctx_handle: ContextHandle,
                    name: &str) -> Result<ValueHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let context = self.contexts.get(&ctx_handle)
            .ok_or_else(|| format!("Invalid context handle: {}", ctx_handle))?;

        let i32_type = context.i32_type();
        let phi = builder.build_phi(i32_type, name)
            .map_err(|e| format!("Failed to build phi: {}", e))?;

        let handle = self.next_handle();
        // Store PHI in both phis (for add_incoming) and values (for general use)
        self.phis.insert(handle, phi);
        self.values.insert(handle, phi.as_basic_value());
        Ok(handle)
    }

    /// Add incoming value/block pair to PHI node
    pub fn phi_add_incoming(&mut self,
                           phi_handle: ValueHandle,
                           value_handle: ValueHandle,
                           block_handle: BlockHandle) -> Result<(), String> {
        let phi = self.phis.get(&phi_handle)
            .ok_or_else(|| format!("Invalid PHI handle: {}", phi_handle))?;

        let value = self.values.get(&value_handle)
            .ok_or_else(|| format!("Invalid value handle: {}", value_handle))?;

        let block = self.blocks.get(&block_handle)
            .ok_or_else(|| format!("Invalid block handle: {}", block_handle))?;

        phi.add_incoming(&[(value, *block)]);
        Ok(())
    }

    /// Get current insert block
    pub fn get_insert_block(&mut self, builder_handle: BuilderHandle) -> Result<BlockHandle, String> {
        let builder = self.builders.get(&builder_handle)
            .ok_or_else(|| format!("Invalid builder handle: {}", builder_handle))?;

        let block = builder.get_insert_block()
            .ok_or_else(|| "No insert block set".to_string())?;

        let handle = self.next_handle();
        self.blocks.insert(handle, block);
        Ok(handle)
    }
}

// Public API functions that Forth will call

/// Create a new LLVM context
/// Stack: ( -- ctx-handle )
pub fn llvm_create_context() -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        Ok(registry.create_context())
    })
}

/// Create a new module
/// Stack: ( ctx-handle name-addr name-len -- module-handle )
pub fn llvm_create_module(ctx_handle: i64, name: &str) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.create_module(ctx_handle, name)
    })
}

/// Declare an external function in the module
/// Stack: ( module-handle ctx-handle name-addr name-len -- )
pub fn llvm_declare_external(module_handle: i64, ctx_handle: i64, name: &str) -> Result<(), String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.declare_external_function(module_handle, ctx_handle, name)
    })
}

/// Create a new builder
/// Stack: ( ctx-handle -- builder-handle )
pub fn llvm_create_builder(ctx_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.create_builder(ctx_handle)
    })
}

/// Create a new function
/// Stack: ( module-handle ctx-handle name-addr name-len -- fn-handle )
pub fn llvm_create_function(module_handle: i64, ctx_handle: i64, name: &str) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.create_function(module_handle, ctx_handle, name)
    })
}

/// Get existing function from module by name
/// Stack: ( module-handle name-addr name-len -- fn-handle )
pub fn llvm_get_function(module_handle: i64, name: &str) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.get_function(module_handle, name)
    })
}

/// Create a basic block
/// Stack: ( ctx-handle fn-handle name-addr name-len -- block-handle )
pub fn llvm_create_block(ctx_handle: i64, fn_handle: i64, name: &str) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.create_block(ctx_handle, fn_handle, name)
    })
}

/// Position builder at end of block
/// Stack: ( builder-handle block-handle -- )
pub fn llvm_position_at_end(builder_handle: i64, block_handle: i64) -> Result<(), String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.position_at_end(builder_handle, block_handle)
    })
}

/// Build return void instruction
/// Stack: ( builder-handle -- )
pub fn llvm_build_ret_void(builder_handle: i64) -> Result<(), String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_ret_void(builder_handle)
    })
}

/// Build return instruction with value
/// Stack: ( builder-handle value-handle -- )
pub fn llvm_build_ret(builder_handle: i64, value_handle: i64) -> Result<(), String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_ret(builder_handle, value_handle)
    })
}

/// Dump module IR to stdout
/// Stack: ( module-handle -- )
pub fn llvm_dump_module(module_handle: i64) -> Result<(), String> {
    LLVM_REGISTRY.with(|cell| {
        let registry = cell.borrow();
        let ir = registry.dump_module_ir(module_handle)?;
        println!("\n=== LLVM IR from Forth Compiler ===");
        println!("{}", ir);
        println!("====================================\n");
        Ok(())
    })
}

/// Create JIT execution engine
/// Stack: ( module-handle -- engine-handle )
pub fn llvm_create_jit_engine(module_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.create_jit_engine(module_handle)
    })
}

/// Get JIT function pointer
/// Stack: ( engine-handle name-addr name-len -- fn-ptr )
pub fn llvm_get_jit_function(engine_handle: i64, name: &str) -> Result<usize, String> {
    LLVM_REGISTRY.with(|cell| {
        let registry = cell.borrow();
        registry.get_jit_function(engine_handle, name)
    })
}

// Additional IR builder primitives

/// Build constant integer
/// Stack: ( ctx-handle value bit-width -- value-handle )
pub fn llvm_build_const_int(ctx_handle: i64, value: i64, bit_width: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_const_int(ctx_handle, value, bit_width)
    })
}

/// Build load instruction
/// Stack: ( builder-handle ctx-handle ptr-handle bit-width -- value-handle )
pub fn llvm_build_load(builder_handle: i64, ctx_handle: i64, ptr_handle: i64, bit_width: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_load(builder_handle, ctx_handle, ptr_handle, bit_width)
    })
}

/// Build store instruction
/// Stack: ( builder-handle value-handle ptr-handle -- )
pub fn llvm_build_store(builder_handle: i64, value_handle: i64, ptr_handle: i64) -> Result<(), String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_store(builder_handle, value_handle, ptr_handle)
    })
}

/// Build GEP instruction
/// Stack: ( builder-handle ctx-handle ptr-handle offset-handle -- result-handle )
pub fn llvm_build_gep(builder_handle: i64, ctx_handle: i64, ptr_handle: i64, offset_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_gep(builder_handle, ctx_handle, ptr_handle, offset_handle)
    })
}

/// Build add instruction
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_add(builder_handle: i64, lhs_handle: i64, rhs_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_add(builder_handle, lhs_handle, rhs_handle)
    })
}

/// Build sub instruction
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_sub(builder_handle: i64, lhs_handle: i64, rhs_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_sub(builder_handle, lhs_handle, rhs_handle)
    })
}

/// Build mul instruction
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_mul(builder_handle: i64, lhs_handle: i64, rhs_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_mul(builder_handle, lhs_handle, rhs_handle)
    })
}

/// Build sdiv instruction
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_sdiv(builder_handle: i64, lhs_handle: i64, rhs_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_sdiv(builder_handle, lhs_handle, rhs_handle)
    })
}

/// Build srem instruction
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_srem(builder_handle: i64, lhs_handle: i64, rhs_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_srem(builder_handle, lhs_handle, rhs_handle)
    })
}

/// Build and instruction
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_and(builder_handle: i64, lhs_handle: i64, rhs_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_and(builder_handle, lhs_handle, rhs_handle)
    })
}

/// Build or instruction
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_or(builder_handle: i64, lhs_handle: i64, rhs_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_or(builder_handle, lhs_handle, rhs_handle)
    })
}

/// Build xor instruction
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_xor(builder_handle: i64, lhs_handle: i64, rhs_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_xor(builder_handle, lhs_handle, rhs_handle)
    })
}

/// Build shl (shift left) instruction
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_shl(builder_handle: i64, lhs_handle: i64, rhs_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_shl(builder_handle, lhs_handle, rhs_handle)
    })
}

/// Build ashr (arithmetic shift right) instruction
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_ashr(builder_handle: i64, lhs_handle: i64, rhs_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_ashr(builder_handle, lhs_handle, rhs_handle)
    })
}

/// Build unconditional branch
/// Stack: ( builder-handle block-handle -- )
pub fn llvm_build_br(builder_handle: i64, block_handle: i64) -> Result<(), String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_br(builder_handle, block_handle)
    })
}

/// Build conditional branch
/// Stack: ( builder-handle cond-handle then-block else-block -- )
pub fn llvm_build_cond_br(builder_handle: i64, cond_handle: i64, then_block: i64, else_block: i64) -> Result<(), String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_cond_br(builder_handle, cond_handle, then_block, else_block)
    })
}

/// Build integer comparison
/// Stack: ( builder-handle predicate lhs-handle rhs-handle -- result-handle )
/// Predicates: 0=eq, 1=ne, 2=slt, 3=sle, 4=sgt, 5=sge
pub fn llvm_build_icmp(builder_handle: i64, predicate: i64, lhs_handle: i64, rhs_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_icmp(builder_handle, predicate, lhs_handle, rhs_handle)
    })
}

/// Build sign-extend instruction (i1 -> i64 for Forth booleans)
/// Stack: ( builder-handle ctx-handle value-handle -- result-handle )
pub fn llvm_build_sext(builder_handle: i64, ctx_handle: i64, value_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_sext(builder_handle, ctx_handle, value_handle)
    })
}

/// Build select instruction (picks one of two values based on condition)
/// Stack: ( builder-handle cond-handle true-value false-value -- result-handle )
pub fn llvm_build_select(builder_handle: i64, cond_handle: i64, true_handle: i64, false_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_select(builder_handle, cond_handle, true_handle, false_handle)
    })
}

/// Build truncate instruction (i64 -> i8 for byte operations)
/// Stack: ( builder-handle ctx-handle value-handle bit-width -- result-handle )
pub fn llvm_build_trunc(builder_handle: i64, ctx_handle: i64, value_handle: i64, bit_width: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_trunc(builder_handle, ctx_handle, value_handle, bit_width)
    })
}

/// Build function call with up to 3 arguments (for now)
/// Stack: ( builder-handle fn-handle arg1 arg2 arg3 nargs -- )
pub fn llvm_build_call(builder_handle: i64, fn_handle: i64, arg1: i64, arg2: i64, arg3: i64, nargs: i64, is_tail_call: i64) -> Result<(), String> {
    let args: Vec<i64> = match nargs {
        0 => vec![],
        1 => vec![arg1],
        2 => vec![arg1, arg2],  // Correct order: memory, sp
        3 => vec![arg1, arg2, arg3],  // Correct order: memory, sp, rp
        _ => return Err(format!("Unsupported number of arguments: {}", nargs)),
    };

    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_call(builder_handle, fn_handle, &args, is_tail_call != 0)
    })
}

/// Get function parameter as value
/// Stack: ( fn-handle index -- value-handle )
pub fn llvm_get_param(fn_handle: i64, index: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.get_param(fn_handle, index as u32)
    })
}

/// Build PHI node (for SSA merges in loops)
/// Stack: ( builder-handle ctx-handle name-addr name-len -- phi-handle )
pub fn llvm_build_phi(builder_handle: i64, ctx_handle: i64, name: &str) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.build_phi(builder_handle, ctx_handle, name)
    })
}

/// Add incoming value/block pair to PHI node
/// Stack: ( phi-handle value-handle block-handle -- )
pub fn llvm_phi_add_incoming(phi_handle: i64, value_handle: i64, block_handle: i64) -> Result<(), String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.phi_add_incoming(phi_handle, value_handle, block_handle)
    })
}

/// Get current insert block
/// Stack: ( builder-handle -- block-handle )
pub fn llvm_get_insert_block(builder_handle: i64) -> Result<i64, String> {
    LLVM_REGISTRY.with(|cell| {
        let mut registry = cell.borrow_mut();
        registry.get_insert_block(builder_handle)
    })
}
