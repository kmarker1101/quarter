use crate::LoopStack;
use crate::stack::Stack;

// Built-in word definitions
pub fn dot(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let Some(value) = stack.pop() {
        print!("{} ", value);
    } else {
        println!("Stack underflow!");
    }
}

pub fn dot_s(stack: &mut Stack, _loop_stack: &LoopStack) {
    stack.print_stack();
}

// Arithmetic Operations
pub fn add(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(a + b);
    } else {
        println!("Stack underflow!");
    }
}

pub fn subtract(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(a - b);
    } else {
        println!("Stack underflow!");
    }
}

pub fn multiply(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(a * b);
    } else {
        println!("Stack underflow!");
    }
}

pub fn divide(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        if b == 0 {
            print!("Division by zero!");
            stack.push(a);
            stack.push(b);
        } else {
            stack.push(a / b);
        }
    } else {
        print!("Stack underflow!");
    }
}

pub fn modulo(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        if b == 0 {
            print!("Division by zero!");
            stack.push(a);
            stack.push(b);
        } else {
            stack.push(a % b);
        }
    } else {
        print!("Stack underflow!");
    }
}

pub fn slash_modulo(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        if b == 0 {
            print!("Division by zero!");
            stack.push(a);
            stack.push(b);
        } else {
            stack.push(a % b);
            stack.push(a / b);
        }
    } else {
        print!("Stack underflow!");
    }
}

pub fn negate(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let Some(value) = stack.pop() {
        stack.push(-value);
    } else {
        println!("Stack underflow!");
    }
}

// Stack manipulation
pub fn dup(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let Some(&value) = stack.peek() {
        stack.push(value);
    } else {
        println!("Stack underflow!");
    }
}

pub fn swap(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(a), Some(b)) = (stack.pop(), stack.pop()) {
        stack.push(a);
        stack.push(b);
    } else {
        println!("Stack underflow!");
    }
}

// Comparison operators (Forth uses 0 for false, -1 for true)
pub fn less_than(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a < b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}

pub fn greater_than(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a > b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}

pub fn equals(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a == b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}

pub fn not_equals(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a != b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}

pub fn less_or_equal(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a <= b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}

pub fn greater_or_equal(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a >= b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}

pub fn abs(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let Some(value) = stack.pop() {
        stack.push(value.abs());
    } else {
        println!("Stack underflow!");
    }
}

pub fn cr(_stack: &mut Stack, _loop_stack: &LoopStack) {
    println!();
}

pub fn drop(stack: &mut Stack, _loop_stack: &LoopStack) {
    if stack.pop().is_none() {
        println!("Stack underflow!");
    }
}

pub fn rot(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(a), Some(b), Some(c)) = (stack.pop(), stack.pop(), stack.pop()) {
        stack.push(b);
        stack.push(a);
        stack.push(c);
    } else {
        println!("Stack underflow!");
    }
}

pub fn over(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(a);
        stack.push(b);
        stack.push(a);
    } else {
        println!("Stack underflow!");
    }
}

// Loop index words
pub fn loop_i(stack: &mut Stack, loop_stack: &LoopStack) {
    if let Some(index) = loop_stack.get_index() {
        stack.push(index);
    } else {
        println!("Not in a loop!");
    }
}

pub fn loop_j(stack: &mut Stack, loop_stack: &LoopStack) {
    if let Some(index) = loop_stack.get_outer_index() {
        stack.push(index);
    } else {
        println!("Not in a nested loop!");
    }
}

pub fn emit(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let Some(value) = stack.pop() {
        if let Some(ch) = char::from_u32(value as u32) {
            print!("{}", ch);
        } else {
            println!("Invalid character code: {}", value);
        }
    } else {
        println!("Stack underflow!");
    }
}

pub fn key(stack: &mut Stack, _loop_stack: &LoopStack) {
    use std::io::Read;

    let mut buffer = [0; 1];
    match std::io::stdin().read_exact(&mut buffer) {
        Ok(_) => stack.push(buffer[0] as i32),
        Err(_) => {
            // EOF or error - push 0
            stack.push(0);
        }
    }
}

pub fn space(_stack: &mut Stack, _loop_stack: &LoopStack) {
    print!(" ");
}

pub fn and(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(a & b);
    } else {
        println!("Stack underflow!");
    }
}

pub fn or(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(a | b);
    } else {
        println!("Stack underflow!");
    }
}

pub fn xor(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(a ^ b);
    } else {
        println!("Stack underflow!");
    }
}

pub fn invert(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let Some(n) = stack.pop() {
        stack.push(!n);
    } else {
        println!("Stack underflow!");
    }
}

pub fn lshift(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(u), Some(n)) = (stack.pop(), stack.pop()) {
        stack.push(n << u);
    } else {
        println!("Stack underflow!");
    }
}

pub fn rshift(stack: &mut Stack, _loop_stack: &LoopStack) {
    if let (Some(u), Some(n)) = (stack.pop(), stack.pop()) {
        stack.push(n >> u);
    } else {
        println!("Stack underflow!");
    }
}
