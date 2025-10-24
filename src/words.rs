use crate::LoopStack;
use crate::stack::Stack;

// Built-in word definitions
pub fn dot(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(value) = stack.pop(memory) {
        print!("{} ", value);
    } else {
        println!("Stack underflow!");
    }
}

pub fn dot_s(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    stack.print_stack(memory);
}

pub fn u_dot(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // U. ( u -- )
    // Print unsigned number
    if let Some(value) = stack.pop(memory) {
        // Treat as unsigned by converting to u32
        let unsigned_value = value as u32;
        print!("{} ", unsigned_value);
    } else {
        println!("Stack underflow!");
    }
}

pub fn dot_r(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // .R ( n width -- )
    // Print number right-justified in field of width
    if let (Some(width), Some(value)) = (stack.pop(memory), stack.pop(memory)) {
        let num_str = value.to_string();
        let width = width as usize;
        if num_str.len() < width {
            // Pad with spaces on the left
            print!("{:>width$} ", num_str, width = width);
        } else {
            print!("{} ", num_str);
        }
    } else {
        println!("Stack underflow!");
    }
}

pub fn u_dot_r(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // U.R ( u width -- )
    // Print unsigned number right-justified in field of width
    if let (Some(width), Some(value)) = (stack.pop(memory), stack.pop(memory)) {
        let unsigned_value = value as u32;
        let num_str = unsigned_value.to_string();
        let width = width as usize;
        if num_str.len() < width {
            print!("{:>width$} ", num_str, width = width);
        } else {
            print!("{} ", num_str);
        }
    } else {
        println!("Stack underflow!");
    }
}

// Arithmetic Operations
pub fn add(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(a + b, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn subtract(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,

    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(a - b, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn multiply(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,

    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(a * b, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn divide(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        if b == 0 {
            print!("Division by zero!");
            stack.push(a, memory);
            stack.push(b, memory);
        } else {
            stack.push(a / b, memory);
        }
    } else {
        print!("Stack underflow!");
    }
}

pub fn slash_modulo(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,

    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        if b == 0 {
            print!("Division by zero!");
            stack.push(a, memory);
            stack.push(b, memory);
        } else {
            stack.push(a % b, memory);
            stack.push(a / b, memory);
        }
    } else {
        print!("Stack underflow!");
    }
}

// Stack manipulation
pub fn dup(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(value) = stack.peek(memory) {
        stack.push(value, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn swap(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(a), Some(b)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(a, memory);
        stack.push(b, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn pick(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // PICK ( ... n -- ... xn )
    // Copy the nth stack item to top (0 = top item = DUP, 1 = second = OVER, etc.)
    if let Some(n) = stack.pop(memory) {
        let depth = stack.depth();
        let index = n as usize;

        // Check bounds: we need at least (n+1) items on the stack
        if index < depth {
            // Calculate address: sp points to next free, so top is at sp-4
            // nth item is at sp - (n+1)*4
            let sp = stack.get_sp();
            let addr = sp - (index + 1) * 4;

            if let Ok(value) = memory.fetch(addr) {
                stack.push(value, memory);
            } else {
                println!("Memory fetch error in PICK!");
            }
        } else {
            println!("Stack underflow in PICK!");
        }
    } else {
        println!("Stack underflow in PICK!");
    }
}

// Comparison operators (Forth uses 0 for false, -1 for true)
pub fn less_than(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,

    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(if a < b { -1 } else { 0 }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn greater_than(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,

    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(if a > b { -1 } else { 0 }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn abs(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(value) = stack.pop(memory) {
        stack.push(value.abs(), memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn cr(
    _stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    _memory: &mut crate::Memory,
) {
    println!();
}

pub fn drop(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if stack.pop(memory).is_none() {
        println!("Stack underflow!");
    }
}

/// DEPTH: Return the number of items on the stack
/// Stack: ( -- n )
pub fn depth(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    let d = stack.depth() as i32;
    stack.push(d, memory);
}

// Loop index words
pub fn loop_i(
    stack: &mut Stack,
    loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(index) = loop_stack.get_index() {
        stack.push(index, memory);
    } else {
        println!("Not in a loop!");
    }
}

pub fn loop_j(
    stack: &mut Stack,
    loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(index) = loop_stack.get_outer_index() {
        stack.push(index, memory);
    } else {
        println!("Not in a nested loop!");
    }
}

pub fn emit(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(value) = stack.pop(memory) {
        if let Some(ch) = char::from_u32(value as u32) {
            print!("{}", ch);
        } else {
            println!("Invalid character code: {}", value);
        }
    } else {
        println!("Stack underflow!");
    }
}

/// TYPE: Print string from memory
/// Stack: ( addr len -- )
pub fn type_word(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(len), Some(addr)) = (stack.pop(memory), stack.pop(memory)) {
        if len < 0 {
            eprintln!("TYPE: negative length");
            return;
        }
        let addr = addr as usize;
        let len = len as usize;

        // Print each character from memory
        for i in 0..len {
            match memory.fetch_byte(addr + i) {
                Ok(byte) => {
                    if let Some(ch) = char::from_u32(byte as u32) {
                        print!("{}", ch);
                    } else {
                        print!("?");
                    }
                }
                Err(e) => {
                    eprintln!("\nTYPE: {}", e);
                    return;
                }
            }
        }
    } else {
        eprintln!("TYPE: stack underflow");
    }
}

pub fn key(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    use std::io::Read;

    let mut buffer = [0; 1];
    match std::io::stdin().read_exact(&mut buffer) {
        Ok(_) => stack.push(buffer[0] as i32, memory),
        Err(_) => {
            // EOF or error - push 0
            stack.push(0, memory);
        }
    }
}

pub fn and(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(a & b, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn or(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(a | b, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn xor(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(a ^ b, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn invert(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(n) = stack.pop(memory) {
        stack.push(!n, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn lshift(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(u), Some(n)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(n << u, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn rshift(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(u), Some(n)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(n >> u, memory);
    } else {
        println!("Stack underflow!");
    }
}

// Return stack words
pub fn to_r(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(n) = stack.pop(memory) {
        return_stack.push(n, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn r_from(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(n) = return_stack.pop(memory) {
        stack.push(n, memory);
    } else {
        println!("Return stack underflow!");
    }
}

pub fn r_fetch(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(n) = return_stack.peek(memory) {
        stack.push(n, memory);
    } else {
        println!("Return stack underflow!");
    }
}

// Boolean constants
pub fn forth_true(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,

    memory: &mut crate::Memory,
) {
    stack.push(-1, memory);
}

pub fn forth_false(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,

    memory: &mut crate::Memory,
) {
    stack.push(0, memory);
}

// Memory access words
pub fn store(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // ! ( n addr -- )
    if let (Some(addr), Some(value)) = (stack.pop(memory), stack.pop(memory)) {
        match memory.store(addr as usize, value) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }
    } else {
        println!("Stack underflow!");
    }
}

pub fn fetch(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // @ ( addr -- n )
    if let Some(addr) = stack.pop(memory) {
        match memory.fetch(addr as usize) {
            Ok(value) => stack.push(value, memory),
            Err(e) => println!("{}", e),
        }
    } else {
        println!("Stack underflow!");
    }
}

pub fn c_store(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // C! ( c addr -- )
    if let (Some(addr), Some(value)) = (stack.pop(memory), stack.pop(memory)) {
        match memory.store_byte(addr as usize, value) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }
    } else {
        println!("Stack underflow!");
    }
}

pub fn c_fetch(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // C@ ( addr -- c )
    if let Some(addr) = stack.pop(memory) {
        match memory.fetch_byte(addr as usize) {
            Ok(value) => stack.push(value, memory),
            Err(e) => println!("{}", e),
        }
    } else {
        println!("Stack underflow!");
    }
}

// Stack pointer access words
pub fn sp_fetch(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // SP@ ( -- addr )
    // Push current stack pointer onto stack
    let sp = stack.get_sp();
    stack.push(sp as i32, memory);
}

pub fn sp_store(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // SP! ( addr -- )
    // Set stack pointer from top of stack
    if let Some(addr) = stack.pop(memory) {
        stack.set_sp(addr as usize);
    } else {
        println!("Stack underflow!");
    }
}

pub fn rp_fetch(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // RP@ ( -- addr )
    // Push current return stack pointer onto data stack
    let rp = return_stack.get_rp();
    stack.push(rp as i32, memory);
}

pub fn rp_store(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // RP! ( addr -- )
    // Set return stack pointer from top of data stack
    if let Some(addr) = stack.pop(memory) {
        return_stack.set_rp(addr as usize);
    } else {
        println!("Stack underflow!");
    }
}

// Dictionary and Memory Allocation Words

pub fn here(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // HERE ( -- addr )
    // Push current dictionary pointer
    stack.push(memory.here(), memory);
}

pub fn allot(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // ALLOT ( n -- )
    // Allocate n bytes in dictionary space
    if let Some(n) = stack.pop(memory) {
        match memory.allot(n) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }
    } else {
        println!("Stack underflow!");
    }
}

pub fn comma(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // , (comma) ( n -- )
    // Store n at HERE and advance dictionary pointer by 4 bytes
    if let Some(n) = stack.pop(memory) {
        let addr = memory.here() as usize;
        match memory.store(addr, n) {
            Ok(_) => {
                // Advance dictionary pointer by 4 bytes (one cell)
                match memory.allot(4) {
                    Ok(_) => {}
                    Err(e) => println!("{}", e),
                }
            }
            Err(e) => println!("{}", e),
        }
    } else {
        println!("Stack underflow!");
    }
}

// =============================================================================
// JIT-callable wrappers for primitives
// These functions have C calling convention and can be called from LLVM IR
// Signature: void primitive(u8* memory, usize* sp, usize* rp)
// =============================================================================

// Data stack region: 0x000000 to 0x01FFFF (128KB)
const DATA_STACK_END: usize = 0x020000;

/// Check if stack pointer is valid for reading N bytes
#[inline]
unsafe fn check_sp_read(sp_val: usize, bytes_needed: usize) -> bool {
    sp_val >= bytes_needed && sp_val < DATA_STACK_END
}

/// Check if stack can grow by N bytes without overflow
#[inline]
unsafe fn check_sp_write(sp_val: usize, bytes_to_add: usize) -> bool {
    sp_val < DATA_STACK_END && sp_val + bytes_to_add <= DATA_STACK_END
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_dup(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 1 value (4 bytes) and room for 1 more
        if !check_sp_read(sp_val, 4) || !check_sp_write(sp_val, 4) {
            return;  // Stack underflow or overflow
        }

        // Read value from top of stack (sp - 4)
        let addr = memory.add(sp_val - 4) as *const i32;
        let val = *addr;
        // Write value to next position (sp)
        let dest = memory.add(sp_val) as *mut i32;
        *dest = val;
        // Increment sp by 4
        *sp = sp_val + 4;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_drop(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 1 value (4 bytes) to drop
        if !check_sp_read(sp_val, 4) {
            return;  // Stack underflow
        }

        // Decrement sp by 4
        *sp = sp_val - 4;
        let _ = memory; // Suppress warning
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_swap(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 2 values (8 bytes) to swap
        if !check_sp_read(sp_val, 8) {
            return;  // Stack underflow
        }

        // Read a from sp-8, b from sp-4
        let addr_a = memory.add(sp_val - 8) as *mut i32;
        let addr_b = memory.add(sp_val - 4) as *mut i32;
        let a = *addr_a;
        let b = *addr_b;
        // Swap them
        *addr_a = b;
        *addr_b = a;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_add(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 2 values (8 bytes)
        if !check_sp_read(sp_val, 8) {
            return;  // Stack underflow
        }

        // Pop b from sp-4, a from sp-8
        let addr_a = memory.add(sp_val - 8) as *mut i32;
        let addr_b = memory.add(sp_val - 4) as *const i32;
        let a = *addr_a;
        let b = *addr_b;
        // Store result at sp-8
        *addr_a = a + b;
        // Decrement sp by 4
        *sp = sp_val - 4;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_sub(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 2 values (8 bytes)
        if !check_sp_read(sp_val, 8) {
            return;  // Stack underflow
        }

        // Pop b from sp-4, a from sp-8
        let addr_a = memory.add(sp_val - 8) as *mut i32;
        let addr_b = memory.add(sp_val - 4) as *const i32;
        let a = *addr_a;
        let b = *addr_b;
        // Store result at sp-8
        *addr_a = a - b;
        // Decrement sp by 4
        *sp = sp_val - 4;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_mul(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 2 values (8 bytes)
        if !check_sp_read(sp_val, 8) {
            return;  // Stack underflow
        }

        // Pop b from sp-4, a from sp-8
        let addr_a = memory.add(sp_val - 8) as *mut i32;
        let addr_b = memory.add(sp_val - 4) as *const i32;
        let a = *addr_a;
        let b = *addr_b;
        // Store result at sp-8
        *addr_a = a * b;
        // Decrement sp by 4
        *sp = sp_val - 4;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_div(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 2 values (8 bytes)
        if !check_sp_read(sp_val, 8) {
            return;  // Stack underflow
        }

        // Pop b from sp-4, a from sp-8
        let addr_a = memory.add(sp_val - 8) as *mut i32;
        let addr_b = memory.add(sp_val - 4) as *const i32;
        let a = *addr_a;
        let b = *addr_b;
        // Store result at sp-8 (with division by zero check)
        if b != 0 {
            *addr_a = a / b;
        }
        // Decrement sp by 4
        *sp = sp_val - 4;
    }
}

/// JIT-callable less than comparison: ( a b -- flag )
/// Pops two values, pushes -1 if a < b, 0 otherwise
#[unsafe(no_mangle)]
pub extern "C" fn quarter_less_than(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 2 values (8 bytes)
        if !check_sp_read(sp_val, 8) {
            return;  // Stack underflow
        }

        // Pop b from sp-4, a from sp-8
        let addr_a = memory.add(sp_val - 8) as *mut i32;
        let addr_b = memory.add(sp_val - 4) as *const i32;
        let a = *addr_a;
        let b = *addr_b;
        // Store result at sp-8: -1 if a < b, 0 otherwise
        *addr_a = if a < b { -1 } else { 0 };
        // Decrement sp by 4
        *sp = sp_val - 4;
    }
}
// ============================================================================
// LLVM Primitives for Self-Hosting Compiler
// ============================================================================

/// Helper to extract string from memory given address and length
fn extract_string(memory: &crate::Memory, addr: usize, len: usize) -> Result<String, String> {
    let mut bytes = Vec::with_capacity(len);
    for i in 0..len {
        bytes.push(memory.fetch_byte(addr + i)? as u8);
    }
    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8: {}", e))
}

/// LLVM-CREATE-CONTEXT: Create LLVM context
/// Stack: ( -- ctx-handle )
pub fn llvm_create_context_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    match crate::llvm_forth::llvm_create_context() {
        Ok(handle) => stack.push(handle, memory),
        Err(e) => eprintln!("LLVM-CREATE-CONTEXT error: {}", e),
    }
}

/// LLVM-CREATE-MODULE: Create LLVM module
/// Stack: ( ctx-handle name-addr name-len -- module-handle )
pub fn llvm_create_module_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(name_len), Some(name_addr), Some(ctx_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match extract_string(memory, name_addr as usize, name_len as usize) {
            Ok(name) => {
                match crate::llvm_forth::llvm_create_module(ctx_handle, &name) {
                    Ok(handle) => stack.push(handle, memory),
                    Err(e) => eprintln!("LLVM-CREATE-MODULE error: {}", e),
                }
            }
            Err(e) => eprintln!("LLVM-CREATE-MODULE string error: {}", e),
        }
    } else {
        eprintln!("LLVM-CREATE-MODULE: Stack underflow");
    }
}

/// LLVM-DECLARE-EXTERNAL: Declare an external function in the module
/// Stack: ( module-handle ctx-handle name-addr name-len -- )
pub fn llvm_declare_external_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(name_len), Some(name_addr), Some(ctx_handle), Some(module_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match extract_string(memory, name_addr as usize, name_len as usize) {
            Ok(name) => {
                match crate::llvm_forth::llvm_declare_external(module_handle, ctx_handle, &name) {
                    Ok(_) => {},
                    Err(e) => eprintln!("LLVM-DECLARE-EXTERNAL error: {}", e),
                }
            }
            Err(e) => eprintln!("LLVM-DECLARE-EXTERNAL string error: {}", e),
        }
    } else {
        eprintln!("LLVM-DECLARE-EXTERNAL: Stack underflow");
    }
}

/// LLVM-CREATE-BUILDER: Create LLVM builder
/// Stack: ( ctx-handle -- builder-handle )
pub fn llvm_create_builder_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(ctx_handle) = stack.pop(memory) {
        match crate::llvm_forth::llvm_create_builder(ctx_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-CREATE-BUILDER error: {}", e),
        }
    } else {
        eprintln!("LLVM-CREATE-BUILDER: Stack underflow");
    }
}

/// LLVM-CREATE-FUNCTION: Create LLVM function
/// Stack: ( module-handle ctx-handle name-addr name-len -- fn-handle )
pub fn llvm_create_function_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(name_len), Some(name_addr), Some(ctx_handle), Some(module_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match extract_string(memory, name_addr as usize, name_len as usize) {
            Ok(name) => {
                match crate::llvm_forth::llvm_create_function(module_handle, ctx_handle, &name) {
                    Ok(handle) => stack.push(handle, memory),
                    Err(e) => eprintln!("LLVM-CREATE-FUNCTION error: {}", e),
                }
            }
            Err(e) => eprintln!("LLVM-CREATE-FUNCTION string error: {}", e),
        }
    } else {
        eprintln!("LLVM-CREATE-FUNCTION: Stack underflow");
    }
}

/// LLVM-MODULE-GET-FUNCTION: Get existing function from module by name
/// Stack: ( module-handle name-addr name-len -- fn-handle )
pub fn llvm_module_get_function_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(name_len), Some(name_addr), Some(module_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match extract_string(memory, name_addr as usize, name_len as usize) {
            Ok(name) => {
                match crate::llvm_forth::llvm_get_function(module_handle, &name) {
                    Ok(handle) => stack.push(handle, memory),
                    Err(e) => eprintln!("LLVM-MODULE-GET-FUNCTION error: {}", e),
                }
            }
            Err(e) => eprintln!("LLVM-MODULE-GET-FUNCTION string error: {}", e),
        }
    } else {
        eprintln!("LLVM-MODULE-GET-FUNCTION: Stack underflow");
    }
}

/// LLVM-CREATE-BLOCK: Create basic block
/// Stack: ( ctx-handle fn-handle name-addr name-len -- block-handle )
pub fn llvm_create_block_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(name_len), Some(name_addr), Some(fn_handle), Some(ctx_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match extract_string(memory, name_addr as usize, name_len as usize) {
            Ok(name) => {
                match crate::llvm_forth::llvm_create_block(ctx_handle, fn_handle, &name) {
                    Ok(handle) => stack.push(handle, memory),
                    Err(e) => eprintln!("LLVM-CREATE-BLOCK error: {}", e),
                }
            }
            Err(e) => eprintln!("LLVM-CREATE-BLOCK string error: {}", e),
        }
    } else {
        eprintln!("LLVM-CREATE-BLOCK: Stack underflow");
    }
}

/// LLVM-POSITION-AT-END: Position builder at end of block
/// Stack: ( builder-handle block-handle -- )
pub fn llvm_position_at_end_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(block_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
    ) {
        if let Err(e) = crate::llvm_forth::llvm_position_at_end(builder_handle, block_handle) {
            eprintln!("LLVM-POSITION-AT-END error: {}", e);
        }
    } else {
        eprintln!("LLVM-POSITION-AT-END: Stack underflow");
    }
}

/// LLVM-BUILD-RET-VOID: Build return void instruction
/// Stack: ( builder-handle -- )
pub fn llvm_build_ret_void_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(builder_handle) = stack.pop(memory) {
        if let Err(e) = crate::llvm_forth::llvm_build_ret_void(builder_handle) {
            eprintln!("LLVM-BUILD-RET-VOID error: {}", e);
        }
    } else {
        eprintln!("LLVM-BUILD-RET-VOID: Stack underflow");
    }
}

/// LLVM-BUILD-RET: Build return instruction with value
/// Stack: ( builder-handle value-handle -- )
pub fn llvm_build_ret_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(value_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
    ) {
        if let Err(e) = crate::llvm_forth::llvm_build_ret(builder_handle, value_handle) {
            eprintln!("LLVM-BUILD-RET error: {}", e);
        }
    } else {
        eprintln!("LLVM-BUILD-RET: Stack underflow");
    }
}

/// LLVM-DUMP-MODULE: Dump module IR to stdout
/// Stack: ( module-handle -- )
pub fn llvm_dump_module_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(module_handle) = stack.pop(memory) {
        if let Err(e) = crate::llvm_forth::llvm_dump_module(module_handle) {
            eprintln!("LLVM-DUMP-MODULE error: {}", e);
        }
    } else {
        eprintln!("LLVM-DUMP-MODULE: Stack underflow");
    }
}

/// LLVM-CREATE-JIT: Create JIT execution engine
/// Stack: ( module-handle -- engine-handle )
pub fn llvm_create_jit_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(module_handle) = stack.pop(memory) {
        match crate::llvm_forth::llvm_create_jit_engine(module_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-CREATE-JIT error: {}", e),
        }
    } else {
        eprintln!("LLVM-CREATE-JIT: Stack underflow");
    }
}

/// LLVM-GET-FUNCTION: Get JIT function pointer
/// Stack: ( engine-handle name-addr name-len -- fn-ptr-low fn-ptr-high )
/// Returns 64-bit pointer as two 32-bit values (low word, then high word)
pub fn llvm_get_function_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(name_len), Some(name_addr), Some(engine_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match extract_string(memory, name_addr as usize, name_len as usize) {
            Ok(name) => {
                match crate::llvm_forth::llvm_get_jit_function(engine_handle, &name) {
                    Ok(fn_ptr) => {
                        // Split 64-bit pointer into two 32-bit values
                        let low = (fn_ptr & 0xFFFFFFFF) as i32;
                        let high = ((fn_ptr >> 32) & 0xFFFFFFFF) as i32;
                        stack.push(low, memory);
                        stack.push(high, memory);
                    }
                    Err(e) => eprintln!("LLVM-GET-FUNCTION error: {}", e),
                }
            }
            Err(e) => eprintln!("LLVM-GET-FUNCTION string error: {}", e),
        }
    } else {
        eprintln!("LLVM-GET-FUNCTION: Stack underflow");
    }
}

// ============================================================================
// LLVM IR Builder Primitives
// ============================================================================

/// LLVM-BUILD-CONST-INT: Create integer constant
/// Stack: ( ctx-handle value bit-width -- value-handle )
pub fn llvm_build_const_int_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(bit_width), Some(value), Some(ctx_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::llvm_forth::llvm_build_const_int(ctx_handle, value, bit_width) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-CONST-INT error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-CONST-INT: Stack underflow");
    }
}

/// LLVM-BUILD-LOAD: Load value from memory
/// Stack: ( builder-handle ctx-handle ptr-handle bit-width -- value-handle )
pub fn llvm_build_load_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(bit_width), Some(ptr_handle), Some(ctx_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::llvm_forth::llvm_build_load(builder_handle, ctx_handle, ptr_handle, bit_width) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-LOAD error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-LOAD: Stack underflow");
    }
}

/// LLVM-BUILD-STORE: Store value to memory
/// Stack: ( builder-handle value-handle ptr-handle -- )
pub fn llvm_build_store_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(ptr_handle), Some(value_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        if let Err(e) = crate::llvm_forth::llvm_build_store(builder_handle, value_handle, ptr_handle) {
            eprintln!("LLVM-BUILD-STORE error: {}", e);
        }
    } else {
        eprintln!("LLVM-BUILD-STORE: Stack underflow");
    }
}

/// LLVM-BUILD-GEP: Get element pointer (pointer arithmetic)
/// Stack: ( builder-handle ctx-handle ptr-handle offset-handle -- ptr-handle )
pub fn llvm_build_gep_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(offset_handle), Some(ptr_handle), Some(ctx_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::llvm_forth::llvm_build_gep(builder_handle, ctx_handle, ptr_handle, offset_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-GEP error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-GEP: Stack underflow");
    }
}

/// LLVM-BUILD-ADD: Add two values
/// Stack: ( builder-handle lhs-handle rhs-handle -- value-handle )
pub fn llvm_build_add_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(rhs_handle), Some(lhs_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::llvm_forth::llvm_build_add(builder_handle, lhs_handle, rhs_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-ADD error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-ADD: Stack underflow");
    }
}

/// LLVM-BUILD-SUB: Subtract two values
/// Stack: ( builder-handle lhs-handle rhs-handle -- value-handle )
pub fn llvm_build_sub_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(rhs_handle), Some(lhs_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::llvm_forth::llvm_build_sub(builder_handle, lhs_handle, rhs_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-SUB error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-SUB: Stack underflow");
    }
}

/// LLVM-BUILD-MUL: Multiply two values
/// Stack: ( builder-handle lhs-handle rhs-handle -- value-handle )
pub fn llvm_build_mul_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(rhs_handle), Some(lhs_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::llvm_forth::llvm_build_mul(builder_handle, lhs_handle, rhs_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-MUL error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-MUL: Stack underflow");
    }
}

/// LLVM-BUILD-BR: Unconditional branch
/// Stack: ( builder-handle block-handle -- )
pub fn llvm_build_br_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(block_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
    ) {
        if let Err(e) = crate::llvm_forth::llvm_build_br(builder_handle, block_handle) {
            eprintln!("LLVM-BUILD-BR error: {}", e);
        }
    } else {
        eprintln!("LLVM-BUILD-BR: Stack underflow");
    }
}

/// LLVM-BUILD-COND-BR: Conditional branch
/// Stack: ( builder-handle cond-handle then-block else-block -- )
pub fn llvm_build_cond_br_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(else_block), Some(then_block), Some(cond_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        if let Err(e) = crate::llvm_forth::llvm_build_cond_br(builder_handle, cond_handle, then_block, else_block) {
            eprintln!("LLVM-BUILD-COND-BR error: {}", e);
        }
    } else {
        eprintln!("LLVM-BUILD-COND-BR: Stack underflow");
    }
}

/// LLVM-BUILD-ICMP: Integer comparison
/// Stack: ( builder-handle predicate lhs-handle rhs-handle -- value-handle )
/// Predicates: 0=EQ 1=NE 2=SLT 3=SLE 4=SGT 5=SGE
pub fn llvm_build_icmp_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(rhs_handle), Some(lhs_handle), Some(predicate), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::llvm_forth::llvm_build_icmp(builder_handle, predicate, lhs_handle, rhs_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-ICMP error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-ICMP: Stack underflow");
    }
}

/// LLVM-BUILD-CALL: Call function
/// Stack: ( builder-handle fn-handle arg1 arg2 arg3 nargs -- )
pub fn llvm_build_call_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(nargs), Some(arg3), Some(arg2), Some(arg1), Some(fn_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        if let Err(e) = crate::llvm_forth::llvm_build_call(builder_handle, fn_handle, arg1, arg2, arg3, nargs) {
            eprintln!("LLVM-BUILD-CALL error: {}", e);
        }
    } else {
        eprintln!("LLVM-BUILD-CALL: Stack underflow");
    }
}

/// LLVM-GET-PARAM: Get function parameter
/// Stack: ( fn-handle index -- value-handle )
pub fn llvm_get_param_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(index), Some(fn_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::llvm_forth::llvm_get_param(fn_handle, index) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-GET-PARAM error: {}", e),
        }
    } else {
        eprintln!("LLVM-GET-PARAM: Stack underflow");
    }
}

/// LLVM-BUILD-PHI: Build PHI node for SSA merges
/// Stack: ( builder-handle ctx-handle name-addr name-len -- phi-handle )
pub fn llvm_build_phi_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(name_len), Some(name_addr), Some(ctx_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match extract_string(memory, name_addr as usize, name_len as usize) {
            Ok(name) => {
                match crate::llvm_forth::llvm_build_phi(builder_handle, ctx_handle, &name) {
                    Ok(handle) => stack.push(handle, memory),
                    Err(e) => eprintln!("LLVM-BUILD-PHI error: {}", e),
                }
            }
            Err(e) => eprintln!("LLVM-BUILD-PHI string error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-PHI: Stack underflow");
    }
}

/// LLVM-PHI-ADD-INCOMING: Add incoming value/block pair to PHI node
/// Stack: ( phi-handle value-handle block-handle -- )
pub fn llvm_phi_add_incoming_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(block_handle), Some(value_handle), Some(phi_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        if let Err(e) = crate::llvm_forth::llvm_phi_add_incoming(phi_handle, value_handle, block_handle) {
            eprintln!("LLVM-PHI-ADD-INCOMING error: {}", e);
        }
    } else {
        eprintln!("LLVM-PHI-ADD-INCOMING: Stack underflow");
    }
}

/// LLVM-GET-INSERT-BLOCK: Get current insert block
/// Stack: ( builder-handle -- block-handle )
pub fn llvm_get_insert_block_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(builder_handle) = stack.pop(memory) {
        match crate::llvm_forth::llvm_get_insert_block(builder_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-GET-INSERT-BLOCK error: {}", e),
        }
    } else {
        eprintln!("LLVM-GET-INSERT-BLOCK: Stack underflow");
    }
}

// ============================================================================
// AST Inspection Primitives
// ============================================================================

/// AST-TYPE: Get AST node type
/// Stack: ( ast-handle -- type )
/// Types: 1=PushNumber, 2=CallWord, 3=Sequence, 4=IfThenElse, 5=BeginUntil,
///        6=BeginWhileRepeat, 7=DoLoop, 8=PrintString, 9=StackString, 10=Leave, 11=Exit
pub fn ast_get_type_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(handle) = stack.pop(memory) {
        match crate::ast_forth::ast_get_type(handle) {
            Ok(node_type) => stack.push(node_type, memory),
            Err(e) => eprintln!("AST-TYPE error: {}", e),
        }
    } else {
        eprintln!("AST-TYPE: Stack underflow");
    }
}

/// AST-GET-NUMBER: Get number from PushNumber node
/// Stack: ( ast-handle -- number )
pub fn ast_get_number_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(handle) = stack.pop(memory) {
        match crate::ast_forth::ast_get_number(handle) {
            Ok(number) => stack.push(number, memory),
            Err(e) => eprintln!("AST-GET-NUMBER error: {}", e),
        }
    } else {
        eprintln!("AST-GET-NUMBER: Stack underflow");
    }
}

/// AST-GET-WORD: Get word name from CallWord node
/// Stack: ( ast-handle addr -- length )
pub fn ast_get_word_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(addr), Some(handle)) = (
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::ast_forth::ast_get_word_name(handle, memory, addr as usize) {
            Ok(length) => stack.push(length, memory),
            Err(e) => eprintln!("AST-GET-WORD error: {}", e),
        }
    } else {
        eprintln!("AST-GET-WORD: Stack underflow");
    }
}

/// AST-GET-STRING: Get string from PrintString or StackString
/// Stack: ( ast-handle addr -- length )
pub fn ast_get_string_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(addr), Some(handle)) = (
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::ast_forth::ast_get_string(handle, memory, addr as usize) {
            Ok(length) => stack.push(length, memory),
            Err(e) => eprintln!("AST-GET-STRING error: {}", e),
        }
    } else {
        eprintln!("AST-GET-STRING: Stack underflow");
    }
}

/// AST-SEQ-LENGTH: Get number of children in Sequence
/// Stack: ( ast-handle -- length )
pub fn ast_seq_length_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(handle) = stack.pop(memory) {
        match crate::ast_forth::ast_get_sequence_length(handle) {
            Ok(length) => stack.push(length, memory),
            Err(e) => eprintln!("AST-SEQ-LENGTH error: {}", e),
        }
    } else {
        eprintln!("AST-SEQ-LENGTH: Stack underflow");
    }
}

/// AST-SEQ-CHILD: Get nth child from Sequence
/// Stack: ( ast-handle index -- child-handle )
pub fn ast_seq_child_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(index), Some(handle)) = (
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::ast_forth::ast_get_sequence_child(handle, index) {
            Ok(child) => stack.push(child, memory),
            Err(e) => eprintln!("AST-SEQ-CHILD error: {}", e),
        }
    } else {
        eprintln!("AST-SEQ-CHILD: Stack underflow");
    }
}

/// AST-IF-THEN: Get then branch from IfThenElse
/// Stack: ( ast-handle -- then-handle )
pub fn ast_if_then_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(handle) = stack.pop(memory) {
        match crate::ast_forth::ast_get_if_then(handle) {
            Ok(then_handle) => stack.push(then_handle, memory),
            Err(e) => eprintln!("AST-IF-THEN error: {}", e),
        }
    } else {
        eprintln!("AST-IF-THEN: Stack underflow");
    }
}

/// AST-IF-ELSE: Get else branch from IfThenElse
/// Stack: ( ast-handle -- else-handle-or-0 )
pub fn ast_if_else_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(handle) = stack.pop(memory) {
        match crate::ast_forth::ast_get_if_else(handle) {
            Ok(else_handle) => stack.push(else_handle, memory),
            Err(e) => eprintln!("AST-IF-ELSE error: {}", e),
        }
    } else {
        eprintln!("AST-IF-ELSE: Stack underflow");
    }
}

/// AST-LOOP-BODY: Get loop body
/// Stack: ( ast-handle -- body-handle )
pub fn ast_loop_body_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(handle) = stack.pop(memory) {
        match crate::ast_forth::ast_get_loop_body(handle) {
            Ok(body_handle) => stack.push(body_handle, memory),
            Err(e) => eprintln!("AST-LOOP-BODY error: {}", e),
        }
    } else {
        eprintln!("AST-LOOP-BODY: Stack underflow");
    }
}

/// AST-LOOP-CONDITION: Get loop condition (BeginWhileRepeat only)
/// Stack: ( ast-handle -- condition-handle )
pub fn ast_loop_condition_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(handle) = stack.pop(memory) {
        match crate::ast_forth::ast_get_loop_condition(handle) {
            Ok(cond_handle) => stack.push(cond_handle, memory),
            Err(e) => eprintln!("AST-LOOP-CONDITION error: {}", e),
        }
    } else {
        eprintln!("AST-LOOP-CONDITION: Stack underflow");
    }
}

/// AST-LOOP-INCREMENT: Get loop increment (DoLoop only)
/// Stack: ( ast-handle -- increment )
pub fn ast_loop_increment_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(handle) = stack.pop(memory) {
        match crate::ast_forth::ast_get_loop_increment(handle) {
            Ok(increment) => stack.push(increment, memory),
            Err(e) => eprintln!("AST-LOOP-INCREMENT error: {}", e),
        }
    } else {
        eprintln!("AST-LOOP-INCREMENT: Stack underflow");
    }
}

/// TEST-AST-CREATE: Create a test AST for compiler testing
/// Creates AST for: 42 (just pushes number 42)
/// Stack: ( -- ast-handle )
pub fn test_ast_create_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    use crate::ast::AstNode;
    use crate::ast_forth::ast_register_node;

    // Create simple AST: PushNumber(42)
    let ast = AstNode::PushNumber(42);
    let handle = ast_register_node(ast);
    stack.push(handle, memory);
}

/// REGISTER-JIT-WORD: Register a JIT-compiled function in the dictionary
/// Stack: ( fn-ptr name-addr name-len dict-ptr -- )
pub fn register_jit_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(dict_ptr), Some(name_len), Some(name_addr), Some(fn_ptr)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match extract_string(memory, name_addr as usize, name_len as usize) {
            Ok(name) => {
                // Cast the function pointer to JITFunction type
                let jit_fn: crate::dictionary::JITFunction = unsafe {
                    std::mem::transmute(fn_ptr as *const ())
                };

                // Get mutable reference to dictionary
                // SAFETY: dict_ptr is assumed to be a valid pointer to Dictionary
                // This is safe because it's passed from main.rs which owns the dictionary
                let dict: &mut crate::Dictionary = unsafe {
                    &mut *(dict_ptr as *mut crate::Dictionary)
                };

                // Register the function
                dict.add_jit_compiled(name.to_uppercase(), jit_fn);
            }
            Err(e) => eprintln!("REGISTER-JIT-WORD string error: {}", e),
        }
    } else {
        eprintln!("REGISTER-JIT-WORD: Stack underflow");
    }
}
