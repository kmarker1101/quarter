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
        // Treat as unsigned by converting to u64
        let unsigned_value = value as u64;
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
        let unsigned_value = value as u64;
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
            let addr = sp - (index + 1) * 8;

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

pub fn equal(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(if a == b { -1 } else { 0 }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn not_equal(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(if a != b { -1 } else { 0 }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn less_equal(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(if a <= b { -1 } else { 0 }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn greater_equal(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(if a >= b { -1 } else { 0 }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn zero_equal(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(a) = stack.pop(memory) {
        stack.push(if a == 0 { -1 } else { 0 }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn zero_less(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(a) = stack.pop(memory) {
        stack.push(if a < 0 { -1 } else { 0 }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn zero_greater(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(a) = stack.pop(memory) {
        stack.push(if a > 0 { -1 } else { 0 }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn mod_word(
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
        }
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

pub fn negate(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(value) = stack.pop(memory) {
        stack.push(-value, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn min(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(if a < b { a } else { b }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn max(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(if a > b { a } else { b }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn one_plus(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(value) = stack.pop(memory) {
        stack.push(value + 1, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn one_minus(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(value) = stack.pop(memory) {
        stack.push(value - 1, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn two_star(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(value) = stack.pop(memory) {
        stack.push(value * 2, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn two_slash(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(value) = stack.pop(memory) {
        stack.push(value / 2, memory);
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
    let d = stack.depth() as i64;
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
        Ok(_) => stack.push(buffer[0] as i64, memory),
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
    stack.push(sp as i64, memory);
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
    stack.push(rp as i64, memory);
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
    // Store n at HERE and advance dictionary pointer by 8 bytes
    if let Some(n) = stack.pop(memory) {
        let addr = memory.here() as usize;
        match memory.store(addr, n) {
            Ok(_) => {
                // Advance dictionary pointer by 8 bytes (one cell)
                match memory.allot(8) {
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
        if !check_sp_read(sp_val, 8) || !check_sp_write(sp_val, 8) {
            return;  // Stack underflow or overflow
        }

        // Read value from top of stack (sp - 4)
        let addr = memory.add(sp_val - 8) as *const i64;
        let val = *addr;
        // Write value to next position (sp)
        let dest = memory.add(sp_val) as *mut i64;
        *dest = val;
        // Increment sp by 8
        *sp = sp_val + 8;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_drop(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 1 value (4 bytes) to drop
        if !check_sp_read(sp_val, 8) {
            return;  // Stack underflow
        }

        // Decrement sp by 8
        *sp = sp_val - 8;
        let _ = memory; // Suppress warning
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_swap(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 2 values (16 bytes) to swap
        if !check_sp_read(sp_val, 16) {
            return;  // Stack underflow
        }

        // Read a from sp-16, b from sp-8
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *mut i64;
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

        // Bounds check: need at least 2 values (16 bytes)
        if !check_sp_read(sp_val, 16) {
            return;  // Stack underflow
        }

        // Pop b from sp-8, a from sp-16
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let b = *addr_b;
        // Store result at sp-8
        *addr_a = a + b;
        // Decrement sp by 8
        *sp = sp_val - 8;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_sub(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 2 values (16 bytes)
        if !check_sp_read(sp_val, 16) {
            return;  // Stack underflow
        }

        // Pop b from sp-8, a from sp-16
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let b = *addr_b;
        // Store result at sp-8
        *addr_a = a - b;
        // Decrement sp by 8
        *sp = sp_val - 8;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_mul(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 2 values (16 bytes)
        if !check_sp_read(sp_val, 16) {
            return;  // Stack underflow
        }

        // Pop b from sp-8, a from sp-16
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let b = *addr_b;
        // Store result at sp-8
        *addr_a = a * b;
        // Decrement sp by 8
        *sp = sp_val - 8;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quarter_div(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 2 values (16 bytes)
        if !check_sp_read(sp_val, 16) {
            return;  // Stack underflow
        }

        // Pop b from sp-8, a from sp-16
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let b = *addr_b;
        // Store result at sp-8 (with division by zero check)
        if b != 0 {
            *addr_a = a / b;
        }
        // Decrement sp by 8
        *sp = sp_val - 8;
    }
}

/// JIT-callable less than comparison: ( a b -- flag )
/// Pops two values, pushes -1 if a < b, 0 otherwise
#[unsafe(no_mangle)]
pub extern "C" fn quarter_less_than(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;

        // Bounds check: need at least 2 values (16 bytes)
        if !check_sp_read(sp_val, 16) {
            return;  // Stack underflow
        }

        // Pop b from sp-8, a from sp-16
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let b = *addr_b;
        // Store result at sp-8: -1 if a < b, 0 otherwise
        *addr_a = if a < b { -1 } else { 0 };
        // Decrement sp by 8
        *sp = sp_val - 8;
    }
}

/// JIT-callable greater than comparison: ( a b -- flag )
/// Pops two values, pushes -1 if a > b, 0 otherwise
#[unsafe(no_mangle)]
pub extern "C" fn quarter_gt(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let b = *addr_b;
        *addr_a = if a > b { -1 } else { 0 };
        *sp = sp_val - 8;
    }
}

/// JIT-callable less than (alias for quarter_less_than): ( a b -- flag )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_lt(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
    quarter_less_than(memory, sp, rp);
}

/// JIT-callable equal comparison: ( a b -- flag )
/// Pops two values, pushes -1 if a == b, 0 otherwise
#[unsafe(no_mangle)]
pub extern "C" fn quarter_equal(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = addr_a.read_unaligned();
        let b = addr_b.read_unaligned();
        addr_a.write_unaligned(if a == b { -1 } else { 0 });
        *sp = sp_val - 8;
    }
}

/// quarter_not_equal: ( a b -- flag )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_not_equal(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = addr_a.read_unaligned();
        let b = addr_b.read_unaligned();
        addr_a.write_unaligned(if a != b { -1 } else { 0 });
        *sp = sp_val - 8;
    }
}

/// quarter_less_equal: ( a b -- flag )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_less_equal(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = addr_a.read_unaligned();
        let b = addr_b.read_unaligned();
        addr_a.write_unaligned(if a <= b { -1 } else { 0 });
        *sp = sp_val - 8;
    }
}

/// quarter_greater_equal: ( a b -- flag )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_greater_equal(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = addr_a.read_unaligned();
        let b = addr_b.read_unaligned();
        addr_a.write_unaligned(if a >= b { -1 } else { 0 });
        *sp = sp_val - 8;
    }
}

/// quarter_negate: ( n -- -n )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_negate(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *mut i64;
        let value = addr.read_unaligned();
        addr.write_unaligned(-value);
    }
}

/// quarter_abs: ( n -- |n| )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_abs(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *mut i64;
        let value = addr.read_unaligned();
        addr.write_unaligned(value.abs());
    }
}

/// quarter_min: ( n1 n2 -- min )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_min(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = addr_a.read_unaligned();
        let b = addr_b.read_unaligned();
        addr_a.write_unaligned(if a < b { a } else { b });
        *sp = sp_val - 8;
    }
}

/// quarter_max: ( n1 n2 -- max )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_max(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = addr_a.read_unaligned();
        let b = addr_b.read_unaligned();
        addr_a.write_unaligned(if a > b { a } else { b });
        *sp = sp_val - 8;
    }
}

/// quarter_1plus: ( n -- n+1 )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_1plus(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *mut i64;
        let value = addr.read_unaligned();
        addr.write_unaligned(value + 1);
    }
}

/// quarter_1minus: ( n -- n-1 )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_1minus(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *mut i64;
        let value = addr.read_unaligned();
        addr.write_unaligned(value - 1);
    }
}

/// quarter_2star: ( n -- n*2 )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_2star(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *mut i64;
        let value = addr.read_unaligned();
        addr.write_unaligned(value * 2);
    }
}

/// quarter_2slash: ( n -- n/2 )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_2slash(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *mut i64;
        let value = addr.read_unaligned();
        addr.write_unaligned(value / 2);
    }
}

// ============================================================================
// Memory Operations
// ============================================================================

/// JIT-callable store: ( n addr -- )
/// Stores n at address addr (8-byte cell)
#[unsafe(no_mangle)]
pub extern "C" fn quarter_store(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        // Pop addr from sp-8, value from sp-16 using unaligned read
        let value_ptr = memory.add(sp_val - 16) as *const i64;
        let addr_ptr = memory.add(sp_val - 8) as *const i64;
        let value = value_ptr.read_unaligned();
        let addr = addr_ptr.read_unaligned() as usize;

        // Store value at addr using unaligned write
        if addr + 8 <= 8 * 1024 * 1024 {  // 8MB bounds check
            let dest = memory.add(addr) as *mut i64;
            dest.write_unaligned(value);
        }

        // Decrement sp by 16 (consumed both values)
        *sp = sp_val - 16;
    }
}

/// JIT-callable fetch: ( addr -- n )
/// Fetches 8-byte cell from address addr
#[unsafe(no_mangle)]
pub extern "C" fn quarter_fetch(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        // Pop addr from sp-8 using unaligned read
        let addr_ptr = memory.add(sp_val - 8) as *const i64;
        let addr = addr_ptr.read_unaligned() as usize;

        // Fetch value from addr
        if addr + 8 <= 8 * 1024 * 1024 {
            let src = memory.add(addr) as *const i64;
            let value = src.read_unaligned();
            // Replace addr on stack with value using unaligned write
            let dest = memory.add(sp_val - 8) as *mut i64;
            dest.write_unaligned(value);
        }
    }
}

/// JIT-callable c-store: ( c addr -- )
/// Stores byte c at address addr
#[unsafe(no_mangle)]
pub extern "C" fn quarter_c_store(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        // Pop addr from sp-8, value from sp-16 using unaligned read
        let value_ptr = memory.add(sp_val - 16) as *const i64;
        let addr_ptr = memory.add(sp_val - 8) as *const i64;
        let value = (value_ptr.read_unaligned() & 0xFF) as u8;
        let addr = addr_ptr.read_unaligned() as usize;

        // Store byte at addr
        if addr < 8 * 1024 * 1024 {
            let dest = memory.add(addr);
            *dest = value;
        }

        // Decrement sp by 16
        *sp = sp_val - 16;
    }
}

/// JIT-callable c-fetch: ( addr -- c )
/// Fetches byte from address addr
#[unsafe(no_mangle)]
pub extern "C" fn quarter_c_fetch(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        // Pop addr from sp-8 using unaligned read
        let addr_ptr = memory.add(sp_val - 8) as *const i64;
        let addr = addr_ptr.read_unaligned() as usize;

        // Fetch byte from addr
        if addr < 8 * 1024 * 1024 {
            let byte_val = *memory.add(addr) as i64;
            // Replace addr on stack with byte value using unaligned write
            let dest = memory.add(sp_val - 8) as *mut i64;
            dest.write_unaligned(byte_val);
        }
    }
}

// ============================================================================
// Bitwise Operations
// ============================================================================

/// JIT-callable and: ( a b -- a&b )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_and(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let b = *addr_b;
        *addr_a = a & b;
        *sp = sp_val - 8;
    }
}

/// JIT-callable or: ( a b -- a|b )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_or(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let b = *addr_b;
        *addr_a = a | b;
        *sp = sp_val - 8;
    }
}

/// JIT-callable xor: ( a b -- a^b )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_xor(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let b = *addr_b;
        *addr_a = a ^ b;
        *sp = sp_val - 8;
    }
}

/// JIT-callable invert: ( a -- ~a )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_invert(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *mut i64;
        *addr = !*addr;
    }
}

/// JIT-callable lshift: ( a u -- a<<u )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_lshift(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_u = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let u = *addr_u as u32;
        *addr_a = a << u;
        *sp = sp_val - 8;
    }
}

/// JIT-callable rshift: ( a u -- a>>u )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_rshift(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_u = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let u = *addr_u as u32;
        *addr_a = a >> u;
        *sp = sp_val - 8;
    }
}

// ============================================================================
// Return Stack Operations
// ============================================================================

/// JIT-callable >R: ( n -- ) (R: -- n)
/// Moves value from data stack to return stack
#[unsafe(no_mangle)]
pub extern "C" fn quarter_to_r(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        let rp_val = *rp;

        if !check_sp_read(sp_val, 8) {
            return;
        }

        // Check return stack bounds (return stack is at 0x010000-0x01FFFF)
        if rp_val + 8 > 0x020000 {
            return;  // Return stack overflow
        }

        // Pop from data stack
        let value_addr = memory.add(sp_val - 8) as *const i64;
        let value = *value_addr;
        *sp = sp_val - 8;

        // Push to return stack
        let r_dest = memory.add(rp_val) as *mut i64;
        *r_dest = value;
        *rp = rp_val + 8;
    }
}

/// JIT-callable R>: ( -- n ) (R: n -- )
/// Moves value from return stack to data stack
#[unsafe(no_mangle)]
pub extern "C" fn quarter_r_from(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        let rp_val = *rp;

        // Check return stack underflow
        if rp_val < 0x010000 + 8 {
            return;  // Return stack underflow
        }

        // Check data stack bounds
        if sp_val + 8 > 0x010000 {
            return;  // Data stack overflow
        }

        // Pop from return stack
        let r_addr = memory.add(rp_val - 8) as *const i64;
        let value = *r_addr;
        *rp = rp_val - 8;

        // Push to data stack
        let dest = memory.add(sp_val) as *mut i64;
        *dest = value;
        *sp = sp_val + 8;
    }
}

/// JIT-callable R@: ( -- n ) (R: n -- n)
/// Copies top of return stack to data stack
#[unsafe(no_mangle)]
pub extern "C" fn quarter_r_fetch(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        let rp_val = *rp;

        // Check return stack underflow
        if rp_val < 0x010000 + 8 {
            return;
        }

        // Check data stack bounds
        if sp_val + 8 > 0x010000 {
            return;
        }

        // Peek from return stack
        let r_addr = memory.add(rp_val - 8) as *const i64;
        let value = *r_addr;

        // Push to data stack
        let dest = memory.add(sp_val) as *mut i64;
        *dest = value;
        *sp = sp_val + 8;
    }
}

// ============================================================================
// Additional Stack Operations
// ============================================================================

/// JIT-callable over: ( a b -- a b a )
/// Copies second stack item to top
#[unsafe(no_mangle)]
pub extern "C" fn quarter_over(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        // Read second item (at sp-16) and push it
        let addr = memory.add(sp_val - 16) as *const i64;
        let value = addr.read_unaligned();
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(value);
        *sp = sp_val + 8;
    }
}

/// JIT-callable rot: ( a b c -- b c a )
/// Rotates top three stack items
#[unsafe(no_mangle)]
pub extern "C" fn quarter_rot(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 24) {
            return;
        }
        // Read a (sp-24), b (sp-16), c (sp-8)
        let addr_a = memory.add(sp_val - 24) as *mut i64;
        let addr_b = memory.add(sp_val - 16) as *mut i64;
        let addr_c = memory.add(sp_val - 8) as *mut i64;
        let a = addr_a.read_unaligned();
        let b = addr_b.read_unaligned();
        let c = addr_c.read_unaligned();
        // Write b, c, a
        addr_a.write_unaligned(b);
        addr_b.write_unaligned(c);
        addr_c.write_unaligned(a);
    }
}

/// JIT-callable pick: ( ... n -- ... xn )
/// Copies nth stack item to top (0=top)
#[unsafe(no_mangle)]
pub extern "C" fn quarter_pick(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let n_ptr = memory.add(sp_val - 8) as *const i64;
        let n = n_ptr.read_unaligned();

        let offset = ((n + 1) * 8) as usize;
        if !check_sp_read(sp_val, offset) {
            return;
        }

        let src_addr = memory.add(sp_val - offset) as *const i64;
        let value = src_addr.read_unaligned();
        // Replace n with the picked value
        let dest = memory.add(sp_val - 8) as *mut i64;
        dest.write_unaligned(value);
    }
}

/// JIT-callable depth: ( -- n )
/// Returns number of items on stack
#[unsafe(no_mangle)]
pub extern "C" fn quarter_depth(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        let depth = sp_val / 8;  // Each cell is 8 bytes
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(depth as i64);
        *sp = sp_val + 8;
    }
}

// ============================================================================
// Arithmetic Operations
// ============================================================================

/// JIT-callable /mod: ( n1 n2 -- remainder quotient )
#[unsafe(no_mangle)]
pub extern "C" fn quarter_slash_mod(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_n1 = memory.add(sp_val - 16) as *mut i64;
        let addr_n2 = memory.add(sp_val - 8) as *const i64;
        let n1 = addr_n1.read_unaligned();
        let n2 = addr_n2.read_unaligned();

        if n2 != 0 {
            let remainder = n1 % n2;
            let quotient = n1 / n2;
            addr_n1.write_unaligned(remainder);
            let quot_addr = memory.add(sp_val - 8) as *mut i64;
            quot_addr.write_unaligned(quotient);
        }
    }
}

// ============================================================================
// Loop Access
// ============================================================================

/// JIT-callable i: ( -- n ) (L: index limit -- index limit)
/// Returns current loop index
#[unsafe(no_mangle)]
pub extern "C" fn quarter_i(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    // Note: This needs loop stack access which isn't passed to JIT functions
    // For now, mark as unimplemented - this will need special handling
    unsafe {
        let sp_val = *sp;
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(0);  // Placeholder
        *sp = sp_val + 8;
    }
}

/// JIT-callable j: ( -- n ) (L: ... outer_index outer_limit ... -- ...)
/// Returns outer loop index
#[unsafe(no_mangle)]
pub extern "C" fn quarter_j(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    // Note: Same issue as quarter_i - needs loop stack access
    unsafe {
        let sp_val = *sp;
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(0);  // Placeholder
        *sp = sp_val + 8;
    }
}

// ============================================================================
// I/O Operations
// ============================================================================

/// JIT-callable emit: ( c -- )
/// Outputs character
#[unsafe(no_mangle)]
pub extern "C" fn quarter_emit(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *const i64;
        let code = addr.read_unaligned() as u32;
        if let Some(ch) = char::from_u32(code) {
            print!("{}", ch);
        }
        *sp = sp_val - 8;
    }
}

/// JIT-callable key: ( -- c )
/// Reads character (placeholder - needs proper I/O handling)
#[unsafe(no_mangle)]
pub extern "C" fn quarter_key(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(0);  // Placeholder
        *sp = sp_val + 8;
    }
}

/// JIT-callable cr: ( -- )
/// Outputs newline
#[unsafe(no_mangle)]
pub extern "C" fn quarter_cr(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    print!("\n");
    let _ = (memory, sp, _rp);  // Suppress warnings
}

/// JIT-callable dot: ( n -- )
/// Prints number
#[unsafe(no_mangle)]
pub extern "C" fn quarter_dot(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *const i64;
        let value = addr.read_unaligned();
        print!("{} ", value);
        *sp = sp_val - 8;
    }
}

/// JIT-callable u_dot: ( u -- )
/// Prints unsigned number
#[unsafe(no_mangle)]
pub extern "C" fn quarter_u_dot(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *const i64;
        let value = addr.read_unaligned();
        let unsigned_value = value as u64;
        print!("{} ", unsigned_value);
        *sp = sp_val - 8;
    }
}

/// JIT-callable dot_r: ( n width -- )
/// Prints number right-justified in field of width
#[unsafe(no_mangle)]
pub extern "C" fn quarter_dot_r(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let value_addr = memory.add(sp_val - 16) as *const i64;
        let width_addr = memory.add(sp_val - 8) as *const i64;
        let value = value_addr.read_unaligned();
        let width = width_addr.read_unaligned() as usize;

        let num_str = value.to_string();
        if num_str.len() < width {
            print!("{:>width$} ", num_str, width = width);
        } else {
            print!("{} ", num_str);
        }
        *sp = sp_val - 16;
    }
}

/// JIT-callable u_dot_r: ( u width -- )
/// Prints unsigned number right-justified in field of width
#[unsafe(no_mangle)]
pub extern "C" fn quarter_u_dot_r(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let value_addr = memory.add(sp_val - 16) as *const i64;
        let width_addr = memory.add(sp_val - 8) as *const i64;
        let value = value_addr.read_unaligned();
        let width = width_addr.read_unaligned() as usize;

        let unsigned_value = value as u64;
        let num_str = unsigned_value.to_string();
        if num_str.len() < width {
            print!("{:>width$} ", num_str, width = width);
        } else {
            print!("{} ", num_str);
        }
        *sp = sp_val - 16;
    }
}

/// JIT-callable type: ( addr len -- )
/// Prints string from memory
#[unsafe(no_mangle)]
pub extern "C" fn quarter_type(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_ptr = memory.add(sp_val - 16) as *const i64;
        let len_ptr = memory.add(sp_val - 8) as *const i64;
        let addr = addr_ptr.read_unaligned() as usize;
        let len = len_ptr.read_unaligned();

        if len < 0 {
            eprintln!("TYPE: negative length");
            *sp = sp_val - 16;
            return;
        }

        let len = len as usize;
        // Print each character from memory
        for i in 0..len {
            if addr + i < 8 * 1024 * 1024 {
                let byte_ptr = memory.add(addr + i);
                let byte = *byte_ptr;
                if let Some(ch) = char::from_u32(byte as u32) {
                    print!("{}", ch);
                }
            }
        }
        *sp = sp_val - 16;
    }
}

// ============================================================================
// Stack Pointer and Memory Allocation Primitives
// ============================================================================

/// JIT-callable sp_fetch: ( -- addr )
/// Push current stack pointer onto stack
#[unsafe(no_mangle)]
pub extern "C" fn quarter_sp_fetch(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        // Push sp value onto stack
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(sp_val as i64);
        *sp = sp_val + 8;
    }
}

/// JIT-callable sp_store: ( addr -- )
/// Set stack pointer from top of stack
#[unsafe(no_mangle)]
pub extern "C" fn quarter_sp_store(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        // Pop addr and set sp to it
        let addr_ptr = memory.add(sp_val - 8) as *const i64;
        let new_sp = addr_ptr.read_unaligned() as usize;
        *sp = new_sp;
    }
}

/// JIT-callable rp_fetch: ( -- addr )
/// Push current return stack pointer onto data stack
#[unsafe(no_mangle)]
pub extern "C" fn quarter_rp_fetch(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        let rp_val = *rp;
        // Push rp value onto data stack
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(rp_val as i64);
        *sp = sp_val + 8;
    }
}

/// JIT-callable rp_store: ( addr -- )
/// Set return stack pointer from top of data stack
#[unsafe(no_mangle)]
pub extern "C" fn quarter_rp_store(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        // Pop addr and set rp to it
        let addr_ptr = memory.add(sp_val - 8) as *const i64;
        let new_rp = addr_ptr.read_unaligned() as usize;
        *sp = sp_val - 8;
        *rp = new_rp;
    }
}

/// JIT-callable here: ( -- addr )
/// Push current dictionary pointer
#[unsafe(no_mangle)]
pub extern "C" fn quarter_here(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        // Read dp from fixed memory location (0x01FFF8)
        const DP_ADDR: usize = 0x01FFF8;
        let dp_ptr = memory.add(DP_ADDR) as *const i64;
        let dp_val = dp_ptr.read_unaligned();

        // Push dp onto stack
        let sp_val = *sp;
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(dp_val);
        *sp = sp_val + 8;
    }
}

/// JIT-callable allot: ( n -- )
/// Allocate n bytes in dictionary space
#[unsafe(no_mangle)]
pub extern "C" fn quarter_allot(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }

        // Pop n from stack
        let n_ptr = memory.add(sp_val - 8) as *const i64;
        let n = n_ptr.read_unaligned();
        *sp = sp_val - 8;

        // Read current dp from memory
        const DP_ADDR: usize = 0x01FFF8;
        let dp_ptr = memory.add(DP_ADDR) as *mut i64;
        let dp_val = dp_ptr.read_unaligned();

        // Calculate new dp
        let new_dp = dp_val + n;

        // Check for overflow (8MB limit)
        if new_dp >= 8 * 1024 * 1024 {
            eprintln!("Dictionary overflow");
            return;
        }

        // Write new dp back to memory
        dp_ptr.write_unaligned(new_dp);
    }
}

/// JIT-callable comma: ( n -- )
/// Store n at HERE and advance dictionary pointer by 8 bytes
#[unsafe(no_mangle)]
pub extern "C" fn quarter_comma(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }

        // Pop n from stack
        let n_ptr = memory.add(sp_val - 8) as *const i64;
        let n = n_ptr.read_unaligned();
        *sp = sp_val - 8;

        // Read current dp from memory
        const DP_ADDR: usize = 0x01FFF8;
        let dp_ptr = memory.add(DP_ADDR) as *mut i64;
        let dp_val = dp_ptr.read_unaligned();

        // Store n at dp
        if dp_val >= 0 && (dp_val as usize + 8) <= 8 * 1024 * 1024 {
            let dest = memory.add(dp_val as usize) as *mut i64;
            dest.write_unaligned(n);

            // Advance dp by 8 bytes
            let new_dp = dp_val + 8;
            dp_ptr.write_unaligned(new_dp);
        }
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
                        let low = (fn_ptr & 0xFFFFFFFF) as i64;
                        let high = ((fn_ptr >> 32) & 0xFFFFFFFF) as i64;
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

/// LLVM-BUILD-SDIV: Signed integer division
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_sdiv_word(
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
        match crate::llvm_forth::llvm_build_sdiv(builder_handle, lhs_handle, rhs_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-SDIV error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-SDIV: Stack underflow");
    }
}

/// LLVM-BUILD-SREM: Signed integer remainder (modulo)
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_srem_word(
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
        match crate::llvm_forth::llvm_build_srem(builder_handle, lhs_handle, rhs_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-SREM error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-SREM: Stack underflow");
    }
}

/// LLVM-BUILD-AND: Bitwise AND
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_and_word(
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
        match crate::llvm_forth::llvm_build_and(builder_handle, lhs_handle, rhs_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-AND error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-AND: Stack underflow");
    }
}

/// LLVM-BUILD-OR: Bitwise OR
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_or_word(
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
        match crate::llvm_forth::llvm_build_or(builder_handle, lhs_handle, rhs_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-OR error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-OR: Stack underflow");
    }
}

/// LLVM-BUILD-XOR: Bitwise XOR
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_xor_word(
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
        match crate::llvm_forth::llvm_build_xor(builder_handle, lhs_handle, rhs_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-XOR error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-XOR: Stack underflow");
    }
}

/// LLVM-BUILD-SHL: Shift left
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_shl_word(
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
        match crate::llvm_forth::llvm_build_shl(builder_handle, lhs_handle, rhs_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-SHL error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-SHL: Stack underflow");
    }
}

/// LLVM-BUILD-ASHR: Arithmetic shift right
/// Stack: ( builder-handle lhs-handle rhs-handle -- result-handle )
pub fn llvm_build_ashr_word(
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
        match crate::llvm_forth::llvm_build_ashr(builder_handle, lhs_handle, rhs_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-ASHR error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-ASHR: Stack underflow");
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

/// LLVM-BUILD-SEXT: Sign-extend i1 to i64 (for Forth booleans)
/// Stack: ( builder-handle ctx-handle value-handle -- result-handle )
pub fn llvm_build_sext_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(value_handle), Some(ctx_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::llvm_forth::llvm_build_sext(builder_handle, ctx_handle, value_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-SEXT error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-SEXT: Stack underflow");
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
