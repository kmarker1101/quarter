use quarter::{Dictionary, LoopStack, Stack, load_file, load_stdlib, CompilerConfig, ExecutionOptions, RuntimeContext};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

/// Track whether the Forth compiler has been loaded
#[allow(dead_code)]
static FORTH_COMPILER_LOADED: AtomicBool = AtomicBool::new(false);

/// Attempt to compile using the Forth self-hosting compiler
/// Returns true if successful, false otherwise
#[allow(dead_code)]
fn try_forth_compile(
    name: String,
    ast: &quarter::AstNode,
    ctx: &mut RuntimeContext,
    _use_batch: bool,  // Unused - batch compilation only
    config: CompilerConfig,
    _included_files: &mut HashSet<String>,  // Unused - function returns immediately
) -> bool {
    // Incremental compilation disabled - use batch_compile_all_words() instead
    let _ = (name, ast, ctx, config, _included_files);
    return false;

    // Load the Forth compiler if not already loaded
    #[allow(unreachable_code)]
    if !FORTH_COMPILER_LOADED.load(Ordering::Relaxed) {
        // Stdlib is already loaded by main(), no need to reload it here
        // Load compiler
        let compiler_options = ExecutionOptions::new(false, false);
        if let Err(e) = load_file(
            "stdlib/compiler.fth",
            ctx,
            config,
            compiler_options,
            _included_files,
        ) {
            eprintln!("Failed to load Forth compiler: {}", e);
            return false;
        }

        // Now that compiler is loaded, recompile stdlib words to JIT-compiled versions
        // This replaces the interpreted versions with native code
        let stdlib_options = ExecutionOptions::new(true, false);
        if let Err(e) = quarter::load_stdlib(
            ctx,
            config,
            stdlib_options,
            _included_files,
        ) {
            eprintln!("Warning: Failed to JIT-compile stdlib: {}", e);
            // Continue anyway with interpreted stdlib
        }

        FORTH_COMPILER_LOADED.store(true, Ordering::Relaxed);
    }

    // Register the AST
    use quarter::ast_forth::ast_register_node;
    let ast_handle = ast_register_node(ast.clone());

    // Write word name to memory at address 302000
    let name_addr = 302000;
    for (i, ch) in name.bytes().enumerate() {
        // Store each character as a byte
        if ctx.memory.store_byte(name_addr + i, ch as i64).is_err() {
            return false;
        }
    }

    // Push arguments for COMPILE-WORD: ( ast-handle name-addr name-len -- fn-ptr )
    ctx.stack.push(ast_handle, ctx.memory);
    ctx.stack.push(name_addr as i64, ctx.memory);
    ctx.stack.push(name.len() as i64, ctx.memory);

    // Execute COMPILE-WORD
    if let Err(e) = ctx.dict.execute_word("COMPILE-WORD", ctx.stack, ctx.loop_stack, ctx.return_stack, ctx.memory) {
        eprintln!("Forth compiler error: {}", e);
        return false;
    }

    // Get function pointer from stack (two 32-bit values: high, then low)
    if let (Some(fn_ptr_high), Some(fn_ptr_low)) = (ctx.stack.pop(ctx.memory), ctx.stack.pop(ctx.memory)) {
        // Reconstruct 64-bit pointer from two 32-bit values
        let fn_ptr = ((fn_ptr_high as u64) << 32) | ((fn_ptr_low as u64) & 0xFFFFFFFF);

        // Validate pointer is not null
        if fn_ptr == 0 {
            eprintln!("ERROR: Forth compiler returned NULL function pointer!");
            return false;
        }

        // Cast to JITFunction
        let jit_fn: quarter::dictionary::JITFunction =
            unsafe { std::mem::transmute(fn_ptr as *const ()) };

        // Register in dictionary
        ctx.dict.add_jit_compiled(name, jit_fn);
        return true;
    }

    eprintln!("ERROR: No function pointer on stack after COMPILE-WORD!");
    false
}

fn main() {
    let mut stack = Stack::new();
    let mut dict = Dictionary::new();
    let mut loop_stack = LoopStack::new();
    let mut return_stack = quarter::ReturnStack::new();
    let mut memory = quarter::Memory::new();
    let mut included_files: HashSet<String> = HashSet::new();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut no_jit = false;
    let mut dump_ir = false;
    let mut verify_ir = false;
    let mut compile_stdlib = false;
    let mut jit_mode = false;
    let mut filename: Option<String> = None;

    for arg in args.iter().skip(1) {
        if arg == "--no-jit" {
            no_jit = true;
        } else if arg == "--dump-ir" {
            dump_ir = true;
        } else if arg == "--verify-ir" {
            verify_ir = true;
        } else if arg == "--compile-stdlib" {
            compile_stdlib = true;
        } else if arg == "--jit" {
            jit_mode = true;
            compile_stdlib = true;  // JIT mode implies compile stdlib
        } else if !arg.starts_with("--") {
            filename = Some(arg.clone());
        }
    }

    // Create compiler configuration
    let config = CompilerConfig::new(no_jit, dump_ir, verify_ir);

    // Load standard library (always interpreted initially)
    let load_options = ExecutionOptions::new(false, false);
    {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);
        if let Err(e) = load_stdlib(
            &mut ctx,
            config,
            load_options,
            &mut included_files,
        ) {
            eprintln!("Error loading stdlib: {}", e);
            std::process::exit(1);
        }
    }

    // Compile stdlib if requested (only for --compile-stdlib mode, not --jit)
    // In --jit mode, we batch compile everything together after loading user code
    if compile_stdlib && !jit_mode {
        let mut ctx = RuntimeContext::new(&mut stack, &mut dict, &mut loop_stack, &mut return_stack, &mut memory);
        if let Err(e) = quarter::batch_compile_all_words(
            &mut ctx,
            config,
            &mut included_files,
        ) {
            eprintln!("Failed to compile stdlib: {}", e);
            std::process::exit(1);
        }
    }

    println!("Forth Interpreter v0.2");

    // Initialize global execution context for EVALUATE, CATCH/THROW, and Forth REPL
    quarter::init_execution_context(
        stack,
        dict,
        loop_stack,
        return_stack,
        memory,
        included_files,
        config,
    );

    // Check for file argument
    // Supported extensions: .qtr, .fth, .forth, .quarter
    if let Some(file) = filename {
        println!("Loading {}", file);

        // Load file - in JIT mode, only load definitions without executing
        let result = quarter::with_execution_context(|exec_ctx| {
            let file_options = ExecutionOptions::new(false, jit_mode);
            let mut ctx = RuntimeContext::new(&mut exec_ctx.stack, &mut exec_ctx.dict, &mut exec_ctx.loop_stack, &mut exec_ctx.return_stack, &mut exec_ctx.memory);
            load_file(
                &file,
                &mut ctx,
                exec_ctx.config,
                file_options,
                &mut exec_ctx.included_files,
            )
        });

        match result {
            Some(Ok(_)) => {
                // If JIT mode, batch compile user words now
                if jit_mode {
                    let compile_result = quarter::with_execution_context(|exec_ctx| {
                        let mut ctx = RuntimeContext::new(&mut exec_ctx.stack, &mut exec_ctx.dict, &mut exec_ctx.loop_stack, &mut exec_ctx.return_stack, &mut exec_ctx.memory);
                        quarter::batch_compile_all_words(
                            &mut ctx,
                            exec_ctx.config,
                            &mut exec_ctx.included_files,
                        )
                    });

                    if let Some(Err(e)) = compile_result {
                        eprintln!("Batch compilation failed: {}", e);
                        std::process::exit(1);
                    }

                    // Clear the stack and remove file from included_files
                    quarter::with_execution_context(|ctx| {
                        while ctx.stack.pop(&mut ctx.memory).is_some() {}
                        ctx.included_files.remove(&file);
                    });

                    // Now execute the file with JIT-compiled code
                    let exec_result = quarter::with_execution_context(|exec_ctx| {
                        let exec_options = ExecutionOptions::new(false, false);
                        let mut ctx = RuntimeContext::new(&mut exec_ctx.stack, &mut exec_ctx.dict, &mut exec_ctx.loop_stack, &mut exec_ctx.return_stack, &mut exec_ctx.memory);
                        load_file(
                            &file,
                            &mut ctx,
                            exec_ctx.config,
                            exec_options,
                            &mut exec_ctx.included_files,
                        )
                    });

                    if let Some(Err(e)) = exec_result {
                        eprintln!("JIT execution failed: {}", e);
                        std::process::exit(1);
                    }
                }
                return;
            }
            Some(Err(e)) => {
                eprintln!("Error loading {}: {}", file, e);
                std::process::exit(1);
            }
            None => {
                eprintln!("No execution context available");
                std::process::exit(1);
            }
        }
    }

    // Load the Forth REPL
    let result = quarter::with_execution_context(|exec_ctx| {
        let repl_options = ExecutionOptions::new(false, false);
        let mut ctx = RuntimeContext::new(&mut exec_ctx.stack, &mut exec_ctx.dict, &mut exec_ctx.loop_stack, &mut exec_ctx.return_stack, &mut exec_ctx.memory);
        quarter::load_file(
            "stdlib/repl.fth",
            &mut ctx,
            exec_ctx.config,
            repl_options,
            &mut exec_ctx.included_files,
        )
    });

    match result {
        Some(Ok(())) => {
            // REPL loaded successfully
        }
        Some(Err(e)) => {
            eprintln!("Error loading Forth REPL: {}", e);
            std::process::exit(1);
        }
        None => {
            eprintln!("No execution context available");
            std::process::exit(1);
        }
    }

    // Start the Forth REPL by executing QUARTER-REPL
    println!("Type CTRL-C or CTRL-D to exit");

    let result = quarter::with_execution_context(|ctx| {
        ctx.dict.execute_word(
            "QUARTER-REPL",
            &mut ctx.stack,
            &mut ctx.loop_stack,
            &mut ctx.return_stack,
            &mut ctx.memory,
        )
    });

    match result {
        Some(Ok(())) => {
            // REPL exited normally
        }
        Some(Err(e)) => {
            eprintln!("REPL error: {}", e);
            std::process::exit(1);
        }
        None => {
            eprintln!("No execution context available");
            std::process::exit(1);
        }
    }
}


