use crate::LoopStack;
use crate::stack::Stack;
use std::cell::RefCell;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

// Thread-local rustyline editor for REPL primitives
thread_local! {
    static READLINE_EDITOR: RefCell<Option<DefaultEditor>> = const { RefCell::new(None) };
}

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

// */: Multiply-divide with intermediate product
pub fn star_slash(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // */ ( n1 n2 n3 -- n1*n2/n3 )
    // Multiply n1 by n2, then divide by n3
    // Uses i128 intermediate to prevent overflow
    if let (Some(n3), Some(n2), Some(n1)) = (stack.pop(memory), stack.pop(memory), stack.pop(memory)) {
        if n3 == 0 {
            println!("Division by zero!");
            stack.push(n1, memory);
            stack.push(n2, memory);
            stack.push(n3, memory);
        } else {
            let product = (n1 as i128) * (n2 as i128);
            let result = (product / n3 as i128) as i64;
            stack.push(result, memory);
        }
    } else {
        println!("Stack underflow!");
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

pub fn question_dup(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // ?DUP ( x -- 0 | x x )
    // Duplicate top of stack if non-zero
    if let Some(value) = stack.peek(memory) {
        if value != 0 {
            stack.push(value, memory);
        }
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

pub fn over(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // OVER ( x1 x2 -- x1 x2 x1 )
    // Copy the second item to top
    if let (Some(a), Some(b)) = (stack.pop(memory), stack.pop(memory)) {
        stack.push(b, memory);
        stack.push(a, memory);
        stack.push(b, memory);
    } else {
        println!("Stack underflow!");
    }
}

pub fn rot(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // ROT ( x1 x2 x3 -- x2 x3 x1 )
    // Rotate top three items
    if let (Some(x3), Some(x2), Some(x1)) = (stack.pop(memory), stack.pop(memory), stack.pop(memory)) {
        stack.push(x2, memory);
        stack.push(x3, memory);
        stack.push(x1, memory);
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

pub fn u_less_than(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // U< ( u1 u2 -- flag )
    // Unsigned less than comparison
    if let (Some(b), Some(a)) = (stack.pop(memory), stack.pop(memory)) {
        let ua = a as u64;
        let ub = b as u64;
        stack.push(if ua < ub { -1 } else { 0 }, memory);
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

/// SPACE: Print a space character
/// Stack: ( -- )
pub fn space(
    _stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    _memory: &mut crate::Memory,
) {
    print!(" ");
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

pub fn base(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // BASE ( -- addr )
    // Push address of numeric base variable (default 10)
    stack.push(memory.base(), memory);
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

/// Macro for binary operations (a b -- result)
/// Takes two values from stack, applies operation, pushes result
macro_rules! binary_op {
    ($name:ident, $op:expr) => {
        /// # Safety
        /// The caller must ensure:
        /// - `memory` points to a valid memory buffer of at least 8MB
        /// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
        /// - The data stack contains at least 2 values (16 bytes)
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            memory: *mut u8,
            sp: *mut usize,
            _rp: *mut usize
        ) {
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

                // Apply operation and store result
                *addr_a = $op(a, b);

                // Decrement sp by 8
                *sp = sp_val - 8;
            }
        }
    };
}

/// Macro for unary operations (a -- result)
/// Takes one value from stack, applies operation, pushes result
macro_rules! unary_op {
    ($name:ident, $op:expr) => {
        /// # Safety
        /// The caller must ensure:
        /// - `memory` points to a valid memory buffer of at least 8MB
        /// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
        /// - The data stack contains at least 1 value (8 bytes)
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            memory: *mut u8,
            sp: *mut usize,
            _rp: *mut usize
        ) {
            unsafe {
                let sp_val = *sp;

                // Bounds check: need at least 1 value (8 bytes)
                if !check_sp_read(sp_val, 8) {
                    return;  // Stack underflow
                }

                // Get address of top value
                let addr = memory.add(sp_val - 8) as *mut i64;
                let a = *addr;

                // Apply operation and store result
                *addr = $op(a);
            }
        }
    };
}

/// Macro for binary operations that return two results (a b -- result1 result2)
/// Takes two values from stack, applies operation returning tuple, pushes both results
macro_rules! two_result_binary_op {
    ($name:ident, $op:expr) => {
        /// # Safety
        /// The caller must ensure:
        /// - `memory` points to a valid memory buffer of at least 8MB
        /// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
        /// - The data stack contains at least 2 values (16 bytes)
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            memory: *mut u8,
            sp: *mut usize,
            _rp: *mut usize
        ) {
            unsafe {
                let sp_val = *sp;

                // Bounds check: need at least 2 values (16 bytes)
                if !check_sp_read(sp_val, 16) {
                    return;  // Stack underflow
                }

                // Pop b from sp-8, a from sp-16
                let addr_a = memory.add(sp_val - 16) as *mut i64;
                let addr_b = memory.add(sp_val - 8) as *mut i64;
                let a = addr_a.read_unaligned();
                let b = addr_b.read_unaligned();

                // Apply operation to get two results
                let (result1, result2) = $op(a, b);

                // Store result1 at sp-16, result2 at sp-8
                addr_a.write_unaligned(result1);
                addr_b.write_unaligned(result2);

                // Stack pointer stays the same (2 inputs -> 2 outputs)
            }
        }
    };
}

/// Macro for pointer fetch operations (push pointer value onto stack)
/// Used for SP@, RP@, etc.
macro_rules! pointer_fetch {
    ($name:ident, $pointer:ident) => {
        /// # Safety
        /// The caller must ensure:
        /// - `memory` points to a valid memory buffer of at least 8MB
        /// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
        /// - `$pointer` points to a valid pointer value
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            memory: *mut u8,
            sp: *mut usize,
            $pointer: *mut usize
        ) {
            unsafe {
                let sp_val = *sp;
                let ptr_val = *$pointer;
                // Push pointer value onto data stack
                let dest = memory.add(sp_val) as *mut i64;
                dest.write_unaligned(ptr_val as i64);
                *sp = sp_val + 8;
            }
        }
    };
}

/// Macro for pointer store operations (pop value and set pointer)
/// Used for SP!, RP!, etc.
macro_rules! pointer_store {
    ($name:ident, $pointer:ident, $update_sp:expr) => {
        /// # Safety
        /// The caller must ensure:
        /// - `memory` points to a valid memory buffer of at least 8MB
        /// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
        /// - `$pointer` points to a valid pointer that can be written to
        /// - The data stack contains at least 1 value (8 bytes)
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            memory: *mut u8,
            sp: *mut usize,
            $pointer: *mut usize
        ) {
            unsafe {
                let sp_val = *sp;
                if !check_sp_read(sp_val, 8) {
                    return;
                }
                // Pop value and set pointer
                let addr_ptr = memory.add(sp_val - 8) as *const i64;
                let new_ptr_val = addr_ptr.read_unaligned() as usize;
                if $update_sp {
                    *sp = sp_val - 8;
                }
                *$pointer = new_ptr_val;
            }
        }
    };
}

/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_dup(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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

/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_drop(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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

/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 2 values (16 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_swap(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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

// Arithmetic operations using macros
binary_op!(quarter_add, |a, b| a + b);
binary_op!(quarter_sub, |a, b| a - b);
binary_op!(quarter_mul, |a, b| a * b);
binary_op!(quarter_div, |a, b| if b != 0 { a / b } else { 0 });

// Comparison operations using macros
binary_op!(quarter_less_than, |a, b| if a < b { -1 } else { 0 });
binary_op!(quarter_gt, |a, b| if a > b { -1 } else { 0 });
binary_op!(quarter_equal, |a, b| if a == b { -1 } else { 0 });
binary_op!(quarter_not_equal, |a, b| if a != b { -1 } else { 0 });
binary_op!(quarter_less_equal, |a, b| if a <= b { -1 } else { 0 });

/// JIT-callable less than (alias for quarter_less_than): ( a b -- flag )
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - `rp` points to a valid return stack pointer
/// - The data stack contains at least 2 values (16 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_lt(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
    unsafe {
        quarter_less_than(memory, sp, rp);
    }
}

binary_op!(quarter_greater_equal, |a, b| if a >= b { -1 } else { 0 });

// Unary arithmetic operations
unary_op!(quarter_negate, |a: i64| -a);
unary_op!(quarter_abs, |a: i64| a.abs());
unary_op!(quarter_1plus, |a: i64| a + 1);
unary_op!(quarter_1minus, |a: i64| a - 1);

// Binary min/max operations
binary_op!(quarter_min, |a, b| if a < b { a } else { b });
binary_op!(quarter_max, |a, b| if a > b { a } else { b });

// Multiply/divide by 2 operations
unary_op!(quarter_2star, |a: i64| a * 2);
unary_op!(quarter_2slash, |a: i64| a / 2);

// ============================================================================
// Memory Operations
// ============================================================================

/// JIT-callable store: ( n addr -- )
/// Stores n at address addr (8-byte cell)
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 2 values (16 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_store(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_fetch(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 2 values (16 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_c_store(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_c_fetch(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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

// Bitwise operations using macros
binary_op!(quarter_and, |a, b| a & b);
binary_op!(quarter_or, |a, b| a | b);
binary_op!(quarter_xor, |a, b| a ^ b);
binary_op!(quarter_lshift, |a, b| a << (b as u32));
binary_op!(quarter_rshift, |a, b| a >> (b as u32));
unary_op!(quarter_invert, |a: i64| !a);

// ============================================================================
// Return Stack Operations
// ============================================================================

/// JIT-callable >R: ( n -- ) (R: -- n)
/// Moves value from data stack to return stack
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - `rp` points to a valid return stack pointer within return stack bounds
/// - The data stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_to_r(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - `rp` points to a valid return stack pointer within return stack bounds
/// - The return stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_r_from(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - `rp` points to a valid return stack pointer within return stack bounds
/// - The return stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_r_fetch(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 2 values (16 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_over(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 3 values (24 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_rot(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least (n+1) values
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_pick(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_depth(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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

// /MOD operation using two-result macro
two_result_binary_op!(quarter_slash_mod, |a, b| {
    if b != 0 {
        (a % b, a / b)  // (remainder, quotient)
    } else {
        (0, 0)  // Division by zero protection
    }
});

// ============================================================================
// Loop Access
// ============================================================================

/// JIT-callable i: ( -- n ) (L: index limit -- index limit)
/// Returns current loop index
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_i(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_j(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_emit(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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

/// JIT-callable space: ( -- )
/// Outputs a space character
/// # Safety
/// The caller must ensure:
/// - All pointers are valid (though this function doesn't use them)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_space(_memory: *mut u8, _sp: *mut usize, _rp: *mut usize) {
    print!(" ");
}

/// JIT-callable key: ( -- c )
/// Reads character (placeholder - needs proper I/O handling)
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_key(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(0);  // Placeholder
        *sp = sp_val + 8;
    }
}

/// JIT-callable cr: ( -- )
/// Outputs newline
/// # Safety
/// The caller must ensure:
/// - All pointers are valid (though this function doesn't use them)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_cr(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    println!();
    let _ = (memory, sp, _rp);  // Suppress warnings
}

/// JIT-callable dot: ( n -- )
/// Prints number
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_dot(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_u_dot(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 2 values (16 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_dot_r(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 2 values (16 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_u_dot_r(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 2 values (16 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_type(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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

// Stack pointer operations using macros
// SP@: ( -- addr ) Push current stack pointer
// SP!: ( addr -- ) Set stack pointer
// RP@: ( -- addr ) Push current return stack pointer
// RP!: ( addr -- ) Set return stack pointer
pointer_fetch!(quarter_sp_fetch, sp);
pointer_store!(quarter_sp_store, sp, false);  // SP! sets sp directly, no need to update
pointer_fetch!(quarter_rp_fetch, rp);
pointer_store!(quarter_rp_store, rp, true);   // RP! needs to pop from data stack

/// JIT-callable here: ( -- addr )
/// Push current dictionary pointer
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_here(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_allot(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
/// # Safety
/// The caller must ensure:
/// - `memory` points to a valid memory buffer of at least 8MB
/// - `sp` points to a valid stack pointer within data stack bounds (0-65535)
/// - The data stack contains at least 1 value (8 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_comma(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
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
                    Err(_) => {
                        // Function not found - this is expected when looking up words
                        // that may or may not be JIT-compiled. Push 0 so Forth can detect it.
                        stack.push(0, memory);
                    }
                }
            }
            Err(e) => {
                eprintln!("LLVM-MODULE-GET-FUNCTION string error: {}", e);
                stack.push(0, memory); // Push 0 on failure
            }
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
                    Err(e) => {
                        eprintln!("LLVM-GET-FUNCTION error: {}", e);
                        // Push 0 to indicate failure
                        stack.push(0, memory);
                        stack.push(0, memory);
                    }
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
/// Predicates: 0=EQ 1=NE 2=SLT 3=SLE 4=SGT 5=SGE 6=ULT 7=ULE 8=UGT 9=UGE
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

/// LLVM-BUILD-SELECT: Conditional selection (ternary operator)
/// Stack: ( builder-handle cond-handle true-value false-value -- result-handle )
pub fn llvm_build_select_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(false_handle), Some(true_handle), Some(cond_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::llvm_forth::llvm_build_select(builder_handle, cond_handle, true_handle, false_handle) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-SELECT error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-SELECT: Stack underflow");
    }
}

/// LLVM-BUILD-TRUNC: Truncate i64 to smaller int (for byte operations)
/// Stack: ( builder-handle ctx-handle value-handle bit-width -- result-handle )
pub fn llvm_build_trunc_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(bit_width), Some(value_handle), Some(ctx_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        match crate::llvm_forth::llvm_build_trunc(builder_handle, ctx_handle, value_handle, bit_width) {
            Ok(handle) => stack.push(handle, memory),
            Err(e) => eprintln!("LLVM-BUILD-TRUNC error: {}", e),
        }
    } else {
        eprintln!("LLVM-BUILD-TRUNC: Stack underflow");
    }
}

/// LLVM-BUILD-CALL: Call function
/// Stack: ( builder-handle fn-handle arg1 arg2 arg3 nargs is-tail-call -- )
pub fn llvm_build_call_word(
    stack: &mut crate::Stack,
    _loop_stack: &crate::LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(is_tail_call), Some(nargs), Some(arg3), Some(arg2), Some(arg1), Some(fn_handle), Some(builder_handle)) = (
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
        stack.pop(memory),
    ) {
        if let Err(e) = crate::llvm_forth::llvm_build_call(builder_handle, fn_handle, arg1, arg2, arg3, nargs, is_tail_call) {
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
        // eprintln!("[AST-TYPE-WORD] Popped handle={} (as char='{}', as u8={})",
        //     handle,
        //     if handle >= 0 && handle <= 127 { (handle as u8) as char } else { '?' },
        //     handle as u8);
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

// ============================================================================
// REPL Primitives (for Forth-based REPL)
// ============================================================================

/// READLINE: ( prompt-addr prompt-len -- line-addr line-len flag )
/// Read a line of input with the given prompt using rustyline
/// Returns the line address, length, and a flag (true=-1 if success, false=0 if EOF/interrupt)
pub fn readline_word(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(prompt_len), Some(prompt_addr)) = (stack.pop(memory), stack.pop(memory)) {
        // Extract prompt string from memory
        match extract_string(memory, prompt_addr as usize, prompt_len as usize) {
            Ok(prompt) => {
                // Initialize editor if needed
                READLINE_EDITOR.with(|editor_cell| {
                    let mut editor_opt = editor_cell.borrow_mut();
                    if editor_opt.is_none() {
                        *editor_opt = Some(DefaultEditor::new().unwrap());
                    }

                    if let Some(editor) = editor_opt.as_mut() {
                        match editor.readline(&prompt) {
                            Ok(line) => {
                                // Store the line in a temporary buffer at a fixed high address
                                // Use address 0x300000 (3MB mark) to avoid conflicts with HERE
                                // This gives us plenty of space before we hit this area
                                let temp_buffer = 0x300000_usize;

                                for (i, ch) in line.bytes().enumerate() {
                                    if memory.store_byte(temp_buffer + i, ch as i64).is_err() {
                                        stack.push(0, memory); // addr (dummy)
                                        stack.push(0, memory); // len
                                        stack.push(0, memory); // flag (false)
                                        return;
                                    }
                                }

                                // Push line addr, len, and success flag
                                stack.push(temp_buffer as i64, memory);
                                stack.push(line.len() as i64, memory);
                                stack.push(-1, memory); // true flag
                            }
                            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                                // EOF or interrupt - return false flag
                                stack.push(0, memory); // addr (dummy)
                                stack.push(0, memory); // len
                                stack.push(0, memory); // flag (false)
                            }
                            Err(_) => {
                                // Other error - return false flag
                                stack.push(0, memory); // addr (dummy)
                                stack.push(0, memory); // len
                                stack.push(0, memory); // flag (false)
                            }
                        }
                    } else {
                        // Editor not available
                        stack.push(0, memory); // addr (dummy)
                        stack.push(0, memory); // len
                        stack.push(0, memory); // flag (false)
                    }
                });
            }
            Err(e) => {
                eprintln!("READLINE prompt string error: {}", e);
                stack.push(0, memory); // addr (dummy)
                stack.push(0, memory); // len
                stack.push(0, memory); // flag (false)
            }
        }
    } else {
        eprintln!("READLINE: Stack underflow");
    }
}

/// HISTORY-ADD: ( addr len -- )
/// Add a line to the readline history
pub fn history_add_word(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(len), Some(addr)) = (stack.pop(memory), stack.pop(memory)) {
        match extract_string(memory, addr as usize, len as usize) {
            Ok(line) => {
                READLINE_EDITOR.with(|editor_cell| {
                    if let Some(editor) = editor_cell.borrow_mut().as_mut() {
                        let _ = editor.add_history_entry(line);
                    }
                });
            }
            Err(e) => eprintln!("HISTORY-ADD string error: {}", e),
        }
    } else {
        eprintln!("HISTORY-ADD: Stack underflow");
    }
}

/// HISTORY-LOAD: ( addr len -- flag )
/// Load history from a file, returns true=-1 if success, false=0 if error
pub fn history_load_word(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(len), Some(addr)) = (stack.pop(memory), stack.pop(memory)) {
        match extract_string(memory, addr as usize, len as usize) {
            Ok(filename) => {
                let success = READLINE_EDITOR.with(|editor_cell| {
                    let mut editor_opt = editor_cell.borrow_mut();
                    if editor_opt.is_none() {
                        *editor_opt = Some(DefaultEditor::new().unwrap());
                    }

                    if let Some(editor) = editor_opt.as_mut() {
                        editor.load_history(&filename).is_ok()
                    } else {
                        false
                    }
                });

                stack.push(if success { -1 } else { 0 }, memory);
            }
            Err(e) => {
                eprintln!("HISTORY-LOAD string error: {}", e);
                stack.push(0, memory); // false flag
            }
        }
    } else {
        eprintln!("HISTORY-LOAD: Stack underflow");
        stack.push(0, memory); // false flag
    }
}

/// HISTORY-SAVE: ( addr len -- flag )
/// Save history to a file, returns true=-1 if success, false=0 if error
pub fn history_save_word(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(len), Some(addr)) = (stack.pop(memory), stack.pop(memory)) {
        match extract_string(memory, addr as usize, len as usize) {
            Ok(filename) => {
                let success = READLINE_EDITOR.with(|editor_cell| {
                    if let Some(editor) = editor_cell.borrow_mut().as_mut() {
                        editor.save_history(&filename).is_ok()
                    } else {
                        false
                    }
                });

                stack.push(if success { -1 } else { 0 }, memory);
            }
            Err(e) => {
                eprintln!("HISTORY-SAVE string error: {}", e);
                stack.push(0, memory); // false flag
            }
        }
    } else {
        eprintln!("HISTORY-SAVE: Stack underflow");
        stack.push(0, memory); // false flag
    }
}

/// EVALUATE: ( addr len -- )
/// Execute a string as Forth code
/// Uses the global execution context to parse and execute the code
/// Supports word definitions (: and ;) and all other Forth constructs
pub fn evaluate_word(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // Pop values first before any re-entrant calls
    if let (Some(len), Some(addr)) = (stack.pop(memory), stack.pop(memory)) {
        // Extract the string from memory
        match extract_string(memory, addr as usize, len as usize) {
            Ok(code) => {
                // Get raw pointers for re-entrant access (avoids RefCell borrow panic)
                match crate::get_reentrant_pointers() {
                    Some((dict_ptr, loop_stack_ptr, return_stack_ptr, _memory_ptr, included_files_ptr)) => {
                        // Get config flags
                        let (no_jit, dump_ir, verify_ir) = crate::get_reentrant_config();
                        let config = crate::CompilerConfig::new(no_jit, dump_ir, verify_ir);
                        let options = crate::ExecutionOptions::new(false, false);

                        // SAFETY: Using raw pointers from execution context
                        // These are valid for the lifetime of the execution context
                        unsafe {
                            let mut ctx = crate::RuntimeContext::new(
                                stack,
                                &mut *dict_ptr,
                                &mut *loop_stack_ptr,
                                &mut *return_stack_ptr,
                                memory,
                            );
                            match crate::execute_line(
                                &code,
                                &mut ctx,
                                config,
                                options,
                                &mut *included_files_ptr,
                            ) {
                                Ok(()) => {
                                    // Success
                                }
                                Err(e) => {
                                    eprintln!("EVALUATE error: {}", e);
                                }
                            }
                        }
                    }
                    None => {
                        eprintln!("EVALUATE: No execution context available");
                    }
                }
            }
            Err(e) => {
                eprintln!("EVALUATE string error: {}", e);
            }
        }
    } else {
        eprintln!("EVALUATE: Stack underflow");
    }
}

/// CMOVE: ( src dest count -- )
/// Copy count bytes from src to dest
pub fn cmove_word(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(count), Some(dest), Some(src)) =
        (stack.pop(memory), stack.pop(memory), stack.pop(memory)) {
        let src_addr = src as usize;
        let dest_addr = dest as usize;
        let count_usize = count as usize;

        // Copy bytes from src to dest
        for i in 0..count_usize {
            match memory.fetch_byte(src_addr + i) {
                Ok(byte) => {
                    if let Err(e) = memory.store_byte(dest_addr + i, byte) {
                        eprintln!("CMOVE store error: {}", e);
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("CMOVE fetch error: {}", e);
                    return;
                }
            }
        }
    } else {
        eprintln!("CMOVE: Stack underflow");
    }
}

/// BYE: ( -- )
/// Exit the Forth interpreter
pub fn bye_word(
    _stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    _memory: &mut crate::Memory,
) {
    println!("\nGoodbye!");
    std::process::exit(0);
}

/// THROW: ( k*x n -- k*x | i*x n )
/// If n is non-zero, unwind to nearest CATCH with error code n
/// If n is zero, do nothing
pub fn throw_word(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let Some(n) = stack.pop(memory) {
        if n != 0 {
            // Signal throw by returning a special error with the throw code
            // This will be caught by CATCH
            eprintln!("THROW: Throwing error code {}", n);
            // We can't actually throw from here, so we'll use a special marker
            // The actual unwinding will happen in CATCH
            stack.push(n, memory); // Put code back for CATCH to see
        }
        // If n==0, do nothing (don't throw)
    } else {
        eprintln!("THROW: Stack underflow");
    }
}

/// CATCH: ( i*x addr len -- j*x 0 | i*x n )
/// Execute code at addr/len (like EVALUATE) and catch any errors
/// Returns 0 if successful, or error code n if THROW was called
pub fn catch_word(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    if let (Some(len), Some(addr)) = (stack.pop(memory), stack.pop(memory)) {
        // Save stack depth for unwinding
        let saved_depth = stack.depth();

        // Extract the string from memory
        match extract_string(memory, addr as usize, len as usize) {
            Ok(code) => {
                // Try to execute the code using reentrant pointers
                let result = match crate::get_reentrant_pointers() {
                    Some((dict_ptr, loop_stack_ptr, return_stack_ptr, _memory_ptr, _included_files_ptr)) => {
                        let tokens: Vec<&str> = code.split_whitespace().collect();
                        unsafe {
                            let dict_ref = &*dict_ptr;
                            match crate::parse_tokens(&tokens, dict_ref, None) {
                                Ok(ast) => {
                                    ast.execute(
                                        stack,
                                        dict_ref,
                                        &mut *loop_stack_ptr,
                                        &mut *return_stack_ptr,
                                        memory,
                                    )
                                }
                                Err(e) => Err(e),
                            }
                        }
                    }
                    None => Err("No execution context".to_string()),
                };

                // Check result
                match result {
                    Ok(()) => {
                        // Success - push 0
                        stack.push(0, memory);
                    }
                    Err(e) => {
                        // Error - restore stack and push error code
                        eprintln!("CATCH: Caught error: {}", e);
                        // Restore stack depth
                        while stack.depth() > saved_depth {
                            stack.pop(memory);
                        }
                        // Push error code (-1 for generic error)
                        stack.push(-1, memory);
                    }
                }
            }
            Err(e) => {
                eprintln!("CATCH string error: {}", e);
                stack.push(-3, memory); // String error code
            }
        }
    } else {
        eprintln!("CATCH: Stack underflow");
    }
}
