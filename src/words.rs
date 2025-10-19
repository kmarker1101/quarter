use crate::stack::Stack;

// Built-in word definitions
pub fn dot(stack: &mut Stack) {
    if let Some(value) = stack.pop() {
        print!("{} ", value);
    } else {
        println!("Stack underflow!");
    }
}

pub fn dot_s(stack: &mut Stack) {
    stack.print_stack();
}

// Arithmetic Operations
pub fn add(stack: &mut Stack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(a + b);
    } else {
        println!("Stack underflow!");
    }
}

pub fn subtract(stack: &mut Stack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(a - b);
    } else {
        println!("Stack underflow!");
    }
}

pub fn multiply(stack: &mut Stack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(a * b);
    } else {
        println!("Stack underflow!");
    }
}

pub fn divide(stack: &mut Stack) {
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

pub fn modulo(stack: &mut Stack) {
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

pub fn negate(stack: &mut Stack) {
    if let Some(value) = stack.pop() {
        stack.push(-value);
    } else {
        println!("Stack underflow!");
    }
}

// Stack manipulation
pub fn dup(stack: &mut Stack) {
    if let Some(&value) = stack.peek() {
        stack.push(value);
    } else {
        println!("Stack underflow!");
    }
}

pub fn swap(stack: &mut Stack) {
    if let (Some(a), Some(b)) = (stack.pop(), stack.pop()) {
        stack.push(a);
        stack.push(b);
    } else {
        println!("Stack underflow!");
    }
}

// Comparison operators (Forth uses 0 for false, -1 for true)
pub fn less_than(stack: &mut Stack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a < b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}

pub fn greater_than(stack: &mut Stack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a > b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}

pub fn equals(stack: &mut Stack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a == b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}

pub fn not_equals(stack: &mut Stack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a != b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}

pub fn less_or_equal(stack: &mut Stack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a <= b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}

pub fn greater_or_equal(stack: &mut Stack) {
    if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
        stack.push(if a >= b { -1 } else { 0 });
    } else {
        println!("Stack underflow!");
    }
}
