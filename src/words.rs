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

#[inline(always)]
#[unsafe(no_mangle)]
pub extern "C" fn quarter_dup(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
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

#[inline(always)]
#[unsafe(no_mangle)]
pub extern "C" fn quarter_drop(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        // Decrement sp by 4
        *sp = sp_val - 4;
        let _ = memory; // Suppress warning
    }
}

#[inline(always)]
#[unsafe(no_mangle)]
pub extern "C" fn quarter_swap(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
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

#[inline(always)]
#[unsafe(no_mangle)]
pub extern "C" fn quarter_add(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
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

#[inline(always)]
#[unsafe(no_mangle)]
pub extern "C" fn quarter_sub(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
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

#[inline(always)]
#[unsafe(no_mangle)]
pub extern "C" fn quarter_mul(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        // Pop b from sp-4, a from sp-8
        let addr_a = memory.add(sp_val.wrapping_sub(8)) as *mut i32;
        let addr_b = memory.add(sp_val.wrapping_sub(4)) as *const i32;
        let a = *addr_a;
        let b = *addr_b;
        // Store result at sp-8
        *addr_a = a * b;
        // Decrement sp by 4
        *sp = sp_val - 4;
    }
}

#[inline(always)]
#[unsafe(no_mangle)]
pub extern "C" fn quarter_div(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
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
#[inline(always)]
#[unsafe(no_mangle)]
pub extern "C" fn quarter_less_than(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
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
