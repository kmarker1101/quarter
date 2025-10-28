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

/// Generate a main() wrapper C file that initializes runtime and calls Forth code
fn generate_main_wrapper(main_word: &str, output_path: &str) -> Result<(), String> {
    let main_c_content = format!(r#"/**
 * Generated main() wrapper for Quarter Forth executable
 * This file is auto-generated during compilation
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

// Runtime library functions
extern void quarter_runtime_init(void);
extern void quarter_runtime_cleanup(void);
extern void quarter_runtime_get_state(uint8_t** memory, size_t** sp, size_t** rp);

// Forward declaration of compiled Forth word
extern void _fn_{}(uint8_t* memory, size_t* sp, size_t* rp);

int main(void) {{
    // Initialize runtime
    quarter_runtime_init();

    // Get runtime state
    uint8_t* memory;
    size_t* sp;
    size_t* rp;
    quarter_runtime_get_state(&memory, &sp, &rp);

    // Call main Forth word
    _fn_{}(memory, sp, rp);

    // Cleanup
    quarter_runtime_cleanup();

    return 0;
}}
"#, main_word, main_word);

    std::fs::write(output_path, main_c_content)
        .map_err(|e| format!("Failed to write main wrapper: {}", e))
}

/// Compile a Forth source file to a standalone executable
///
/// Implementation roadmap:
/// 1. Load standard library and source file (parse to AST)
/// 2. Use batch_compile_all_words() to compile to LLVM IR
/// 3. Instead of FINALIZE-BATCH (creates JIT), create FINALIZE-AOT that:
///    - Initializes LLVM target (InitializeAllTargets, InitializeAllTargetMCs, etc.)
///    - Creates TargetMachine for native target
///    - Runs optimization passes (based on opt_level)
///    - Generates object file using write_to_file()
/// 4. Create a minimal runtime library (runtime.c):
///    - Stack management functions
///    - I/O primitives (., EMIT, KEY, etc.)
///    - Memory allocation
///    - Error handling
/// 5. Compile runtime.c to runtime.o
/// 6. Create a main() wrapper that:
///    - Initializes stacks and memory
///    - Calls the top-level Forth words
///    - Handles exit codes
/// 7. Link object files:
///    - main.o (generated wrapper)
///    - forth_code.o (compiled Forth words)
///    - runtime.o (runtime library)
///    - Using system linker (cc or ld)
/// 8. Set executable permissions
///
/// Dependencies:
/// - inkwell TargetMachine API
/// - LLVM target initialization
/// - System linker (cc/clang/gcc)
/// - Runtime library implementation
fn compile_to_executable(
    source_file: &str,
    output_file: &str,
    opt_level: u8,
    debug_symbols: bool,
    verbose: bool,
) {
    if verbose {
        println!("Compiling {} to {}...", source_file, output_file);
        println!("  Optimization level: {}", opt_level);
        println!("  Debug symbols: {}", if debug_symbols { "yes" } else { "no" });
    }

    // Step 1: Build runtime library (Rust staticlib)
    if verbose {
        println!("Step 1: Building runtime library...");
    }
    let runtime_result = std::process::Command::new("cargo")
        .args(&["build", "--lib", "--release"])
        .output()
        .map_err(|e| format!("Failed to run cargo: {}", e));

    match runtime_result {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Failed to build runtime library:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error building runtime: {}", e);
            std::process::exit(1);
        }
    }

    // Step 2: Compile Forth source to object file
    if verbose {
        println!("Step 2: Compiling Forth source to object file...");
    }

    let forth_obj_path = format!("{}_forth.o", output_file);

    // Initialize Quarter execution context
    quarter::init_execution_context(
        quarter::Stack::new(),
        quarter::Dictionary::new(),
        quarter::LoopStack::new(),
        quarter::ReturnStack::new(),
        quarter::Memory::new(),
        std::collections::HashSet::new(),
        quarter::CompilerConfig::new(false, false, false),
    );

    // Load source file and compile to object
    let compile_result = quarter::with_execution_context(|exec_ctx| {
        // Load stdlib
        let load_options = quarter::ExecutionOptions::new(false, false);
        let mut ctx = quarter::RuntimeContext::new(
            &mut exec_ctx.stack,
            &mut exec_ctx.dict,
            &mut exec_ctx.loop_stack,
            &mut exec_ctx.return_stack,
            &mut exec_ctx.memory
        );

        if let Err(e) = quarter::load_stdlib(&mut ctx, exec_ctx.config, load_options, &mut exec_ctx.included_files) {
            return Err(format!("Failed to load stdlib: {}", e));
        }

        // Load source file
        if let Err(e) = quarter::load_file(source_file, &mut ctx, exec_ctx.config, load_options, &mut exec_ctx.included_files) {
            return Err(format!("Failed to load source file: {}", e));
        }

        // Compile to object file
        quarter::compile_to_object_file(&mut ctx, &forth_obj_path, opt_level, exec_ctx.config, &mut exec_ctx.included_files)
    });

    match compile_result {
        Some(Ok(())) => {
            if verbose {
                println!("  Successfully compiled to {}", forth_obj_path);
            }
        }
        Some(Err(e)) => {
            eprintln!("Failed to compile Forth source:");
            eprintln!("{}", e);
            std::process::exit(1);
        }
        None => {
            eprintln!("No execution context available");
            std::process::exit(1);
        }
    }

    // Step 3: Generate main wrapper
    if verbose {
        println!("Step 3: Generating main() wrapper...");
    }

    // TODO: Determine main word (could be "MAIN" or first defined word)
    let main_word = "MAIN";
    let main_c_path = format!("{}_main.c", output_file);

    if let Err(e) = generate_main_wrapper(main_word, &main_c_path) {
        eprintln!("Failed to generate main wrapper: {}", e);
        std::process::exit(1);
    }

    // Step 4: Compile main wrapper
    if verbose {
        println!("Step 4: Compiling main wrapper...");
    }

    let main_o_path = format!("{}_main.o", output_file);
    let cc_result = std::process::Command::new("cc")
        .args(&["-c", "-O2", &main_c_path, "-o", &main_o_path])
        .output();

    match cc_result {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Failed to compile main wrapper:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error compiling main wrapper: {}", e);
            std::process::exit(1);
        }
    }

    // Step 5: Link everything together
    if verbose {
        println!("Step 5: Linking...");
    }

    // Use absolute paths to Homebrew libraries for better compatibility
    let link_result = std::process::Command::new("cc")
        .args(&[
            &main_o_path,
            &forth_obj_path,
            "target/release/libquarter.a",
            "-o",
            output_file,
            "-lc++",    // LLVM requires C++ standard library
            "/opt/homebrew/lib/libzstd.dylib",        // zstd (compression)
            "/opt/homebrew/opt/libffi/lib/libffi.dylib", // libffi (foreign function interface)
            "/opt/homebrew/opt/zlib/lib/libz.dylib",  // zlib (compression)
            "-Wl,-U,_del_curterm",  // Allow undefined ncurses symbols
            "-Wl,-U,_set_curterm",
            "-Wl,-U,_setupterm",
            "-Wl,-U,_tigetnum",
        ])
        .output();

    match link_result {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Failed to link executable:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error running linker: {}", e);
            std::process::exit(1);
        }
    }

    // Success!
    if verbose {
        println!();
        println!("===========================================");
        println!("✓ Successfully created executable: {}", output_file);
        println!("===========================================");
        println!();
        println!("Files generated:");
        println!("  - {} (main wrapper - C source)", main_c_path);
        println!("  - {} (main wrapper - object)", main_o_path);
        println!("  - {} (Forth code - object)", forth_obj_path);
        println!("  - {} (final executable)", output_file);
        println!();
        println!("Linked with:");
        println!("  - target/release/libquarter.a (Rust runtime)");
        println!();
        println!("Run with: ./{}", output_file);
    } else {
        println!("Successfully created: {}", output_file);
    }
}

fn print_help() {
    println!("Quarter - Forth Interpreter and Compiler v0.2");
    println!();
    println!("USAGE:");
    println!("  quarter [OPTIONS] [FILE]");
    println!();
    println!("OPTIONS:");
    println!("  --compile, -c          Compile source file to native executable");
    println!("  -o <output>            Output filename (default: a.out)");
    println!("  --optimize, -O<level>  Optimization level: 0, 1, 2, 3 (default: 2)");
    println!("  --debug, -g            Include debug symbols");
    println!("  --verbose, -v          Show compilation progress");
    println!("  --jit                  Enable JIT compilation mode");
    println!("  --no-jit               Disable JIT compilation");
    println!("  --dump-ir              Dump LLVM IR to stdout");
    println!("  --verify-ir            Verify LLVM IR");
    println!("  --compile-stdlib       Compile standard library");
    println!("  --help, -h             Show this help message");
    println!("  --version              Show version");
    println!();
    println!("EXAMPLES:");
    println!("  quarter                           # Start interactive REPL");
    println!("  quarter myapp.fth                 # Run source file (interpreted)");
    println!("  quarter --jit myapp.fth           # Run with JIT compilation");
    println!("  quarter --compile myapp.fth       # Compile to a.out");
    println!("  quarter -c myapp.fth -o myapp     # Compile to 'myapp'");
    println!("  quarter -c -O3 myapp.fth          # Compile with max optimization");
    println!();
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
    let mut compile_mode = false;
    let mut output_file: Option<String> = None;
    let mut opt_level: u8 = 2;  // Default optimization level
    let mut debug_symbols = false;
    let mut verbose = false;
    let mut filename: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

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
        } else if arg == "--compile" || arg == "-c" {
            compile_mode = true;
        } else if arg == "-o" {
            // Get output filename from next argument
            i += 1;
            if i < args.len() {
                output_file = Some(args[i].clone());
            } else {
                eprintln!("Error: -o requires an output filename");
                std::process::exit(1);
            }
        } else if arg == "--optimize" {
            // Get optimization level from next argument
            i += 1;
            if i < args.len() {
                opt_level = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("Error: --optimize requires a number 0-3");
                    std::process::exit(1);
                });
                if opt_level > 3 {
                    eprintln!("Error: optimization level must be 0-3");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: --optimize requires a level (0-3)");
                std::process::exit(1);
            }
        } else if arg.starts_with("-O") {
            // Handle -O0, -O1, -O2, -O3
            opt_level = arg[2..].parse().unwrap_or_else(|_| {
                eprintln!("Error: -O requires a number 0-3");
                std::process::exit(1);
            });
            if opt_level > 3 {
                eprintln!("Error: optimization level must be 0-3");
                std::process::exit(1);
            }
        } else if arg == "--debug" || arg == "-g" {
            debug_symbols = true;
        } else if arg == "--verbose" || arg == "-v" {
            verbose = true;
        } else if arg == "--help" || arg == "-h" {
            print_help();
            std::process::exit(0);
        } else if arg == "--version" {
            println!("Quarter Forth Interpreter v0.2");
            std::process::exit(0);
        } else if !arg.starts_with("-") {
            filename = Some(arg.clone());
        } else {
            eprintln!("Unknown option: {}", arg);
            std::process::exit(1);
        }

        i += 1;
    }

    // Validate compile mode
    if compile_mode {
        if filename.is_none() {
            eprintln!("Error: --compile requires a source file");
            std::process::exit(1);
        }
        // Set default output file if not specified
        if output_file.is_none() {
            output_file = Some("a.out".to_string());
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
        // Compile mode: generate standalone executable
        if compile_mode {
            if verbose {
                println!("Compiling {} to {}...", file, output_file.as_ref().unwrap());
            }

            // Use the compilation function (to be implemented)
            compile_to_executable(
                &file,
                output_file.as_ref().unwrap(),
                opt_level,
                debug_symbols,
                verbose,
            );

            return;
        }

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


