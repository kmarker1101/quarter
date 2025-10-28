// Quarter Runtime for Standalone Executables
//
// This module provides the minimal runtime support needed for compiled Forth programs.
// It initializes memory and stack pointers that compiled code uses.

use std::sync::Mutex;

// Memory layout constants (must match Quarter's memory layout)
const MEMORY_SIZE: usize = 8 * 1024 * 1024; // 8MB
const STACK_BASE: usize = 0x000000;
const RSTACK_BASE: usize = 0x010000;

// Global runtime state (behind Mutex for safety, though typically single-threaded)
static RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);

struct Runtime {
    memory: Vec<u8>,
    sp: usize,
    rp: usize,
}

impl Runtime {
    fn new() -> Self {
        Runtime {
            memory: vec![0; MEMORY_SIZE],
            sp: STACK_BASE,
            rp: RSTACK_BASE,
        }
    }
}

/// Initialize the runtime environment
/// Allocates memory and initializes stack pointers
/// Must be called before any compiled Forth code executes
#[unsafe(no_mangle)]
pub extern "C" fn quarter_runtime_init() {
    let mut runtime = RUNTIME.lock().unwrap();
    *runtime = Some(Runtime::new());
}

/// Cleanup the runtime environment
/// Frees allocated memory
#[unsafe(no_mangle)]
pub extern "C" fn quarter_runtime_cleanup() {
    let mut runtime = RUNTIME.lock().unwrap();
    *runtime = None;
}

/// Get pointers to runtime state for Forth code
/// Returns: (memory_ptr, sp_ptr, rp_ptr)
#[unsafe(no_mangle)]
pub extern "C" fn quarter_runtime_get_state(
    memory_out: *mut *mut u8,
    sp_out: *mut *mut usize,
    rp_out: *mut *mut usize,
) {
    let mut runtime = RUNTIME.lock().unwrap();
    if let Some(ref mut rt) = *runtime {
        unsafe {
            *memory_out = rt.memory.as_mut_ptr();
            *sp_out = &mut rt.sp as *mut usize;
            *rp_out = &mut rt.rp as *mut usize;
        }
    } else {
        panic!("Runtime not initialized");
    }
}

// Re-export all the quarter_* primitives that compiled Forth code calls
// These are already defined in src/words.rs with unsafe extern "C"
//
// Note: The Forth compiler calls these directly from LLVM-compiled code.
// They all have the signature: fn(memory: *mut u8, sp: *mut usize, rp: *mut usize)

pub use crate::words::{
    // I/O operations
    quarter_dot,
    quarter_dot_r,
    quarter_u_dot,
    quarter_u_dot_r,
    quarter_emit,
    quarter_key,
    quarter_cr,
    quarter_space,
    quarter_type,

    // Stack operations
    quarter_dup,
    quarter_drop,
    quarter_swap,
    quarter_over,
    quarter_rot,
    quarter_pick,
    quarter_depth,

    // Return stack
    quarter_to_r,
    quarter_r_from,
    quarter_r_fetch,

    // Loop operations
    quarter_i,
    quarter_j,

    // Comparison (< only)
    quarter_lt,

    // Memory operations
    quarter_fetch,
    quarter_store,
    quarter_c_fetch,
    quarter_c_store,

    // Memory allocation
    quarter_here,
    quarter_allot,
    quarter_comma,
};
