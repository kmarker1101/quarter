use crate::LoopStack;
use crate::stack::Stack;

// Built-in word definitions
pub fn dot(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack, memory: &mut crate::Memory) {
    if let Some(value) = stack.pop(memory) {
        print!("{} ", value);
    } else {
        println!("Stack underflow!");
    }
}

pub fn dot_s(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    stack.print_stack(memory);
}

// Arithmetic Operations
pub fn add(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
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

pub fn divide(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
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

pub fn modulo(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        if b == 0 {
            print!("Division by zero!");
            stack.push(a, memory);
            stack.push(b, memory);
        } else {
            stack.push(a % b, memory);
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

pub fn negate(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let Some(value) = stack.pop(memory) {
        stack.push(-value, memory);
    } else {
        println!("Stack underflow!");
    }
}

// Stack manipulation
pub fn dup(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let Some(value) = stack.peek(memory) {
        stack.push(value, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn swap(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
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

pub fn equals(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(if a == b { -1 } else { 0 }, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn not_equals(
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

pub fn less_or_equal(
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

pub fn greater_or_equal(
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

pub fn abs(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let Some(value) = stack.pop(memory) {
        stack.push(value.abs(), memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn cr(_stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 _memory: &mut crate::Memory) {
    println!();
}

pub fn drop(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if stack.pop(memory).is_none() {
        println!("Stack underflow!");
    }
}

pub fn rot(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let (Some(a), Some(b), Some(c)) = (stack.pop(memory), stack.pop(memory), stack.pop(memory)) {
        stack.push(b, memory);
        stack.push(a, memory);
        stack.push(c, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn over(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(a, memory);
        stack.push(b, memory);
        stack.push(a, memory);
    } else {
        println!("Stack underflow!");
    }
}

// Loop index words
pub fn loop_i(stack: &mut Stack, loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let Some(index) = loop_stack.get_index() {
        stack.push(index, memory);
    } else {
        println!("Not in a loop!");
    }
}

pub fn loop_j(stack: &mut Stack, loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let Some(index) = loop_stack.get_outer_index() {
        stack.push(index, memory);
    } else {
        println!("Not in a nested loop!");
    }
}

pub fn emit(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
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

pub fn key(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
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

pub fn space(_stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 _memory: &mut crate::Memory) {
    print!(" ");
}

pub fn and(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(a & b, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn or(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(a | b, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn xor(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(a ^ b, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn invert(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let Some(n) = stack.pop(memory) {
        stack.push(!n, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn lshift(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let (Some(u), Some(n)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(n << u, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn rshift(stack: &mut Stack, _loop_stack: &LoopStack, _return_stack: &mut crate::ReturnStack,
 memory: &mut crate::Memory) {
    if let (Some(u), Some(n)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(n >> u, memory);
    } else {
        println!("Stack underflow!");
    }
}

// Return stack words
pub fn to_r(stack: &mut Stack, _loop_stack: &LoopStack, return_stack: &mut crate::ReturnStack, memory: &mut crate::Memory) {
    if let Some(n) = stack.pop(memory) {
        return_stack.push(n, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn r_from(stack: &mut Stack, _loop_stack: &LoopStack, return_stack: &mut crate::ReturnStack, memory: &mut crate::Memory) {
    if let Some(n) = return_stack.pop(memory) {
        stack.push(n, memory);
    } else {
        println!("Return stack underflow!");
    }
}

pub fn r_fetch(stack: &mut Stack, _loop_stack: &LoopStack, return_stack: &mut crate::ReturnStack, memory: &mut crate::Memory) {
    if let Some(n) = return_stack.peek(memory) {
        stack.push(n, memory);
    } else {
        println!("Return stack underflow!");
    }
}

pub fn zero_equals(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,

    memory: &mut crate::Memory,
) {
    if let Some(n) = stack.pop(memory) {
        stack.push(if n == 0 { -1 } else { 0 }, memory);
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
    if let Some(n) = stack.pop(memory) {
        stack.push(if n < 0 { -1 } else { 0 }, memory);
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
    if let Some(n) = stack.pop(memory) {
        stack.push(if n > 0 { -1 } else { 0 }, memory);
    } else {
        println!("Stack underflow!");
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

// Memory arithmetic helpers
pub fn cells(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // CELLS ( n -- n*4 )
    // Convert cell count to byte count
    if let Some(n) = stack.pop(memory) {
        stack.push(n * 4, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn cell_plus(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // CELL+ ( addr -- addr+4 )
    // Add one cell size to address
    if let Some(addr) = stack.pop(memory) {
        stack.push(addr + 4, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn plus_store(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // +! ( n addr -- )
    // Add n to value at addr
    if let (Some(addr), Some(n)) = (stack.pop(memory), stack.pop(memory)) {
        match memory.fetch(addr as usize) {
            Ok(old_value) => {
                match memory.store(addr as usize, old_value + n) {
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
