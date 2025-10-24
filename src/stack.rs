// Stack now resides in Memory space starting at address 0x000000
// Stack pointer (SP) tracks current top of stack

pub struct Stack {
    sp: usize,  // Stack pointer (byte address in memory)
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            sp: 0x000000,  // Start at beginning of data stack region
        }
    }

    pub fn push(&mut self, value: i64, memory: &mut crate::Memory) {
        // Store value at current SP
        memory.store(self.sp, value).expect("Stack overflow");
        // Move SP to next cell (8 bytes)
        self.sp += 8;
    }

    pub fn pop(&mut self, memory: &mut crate::Memory) -> Option<i64> {
        if self.sp == 0x000000 {
            return None;  // Stack underflow
        }
        // Move SP back one cell
        self.sp -= 8;
        // Fetch value at new SP
        memory.fetch(self.sp).ok()
    }

    pub fn peek(&self, memory: &crate::Memory) -> Option<i64> {
        if self.sp == 0x000000 {
            return None;  // Stack empty
        }
        // Peek at top of stack (SP - 8)
        memory.fetch(self.sp - 8).ok()
    }

    pub fn is_empty(&self) -> bool {
        self.sp == 0x000000
    }

    pub fn depth(&self) -> usize {
        self.sp / 8
    }

    pub fn print_stack(&self, memory: &crate::Memory) {
        let depth = self.sp / 8;
        if depth == 0 {
            print!("<0> ");
        } else {
            print!("<{}> ", depth);
            for i in 0..depth {
                let addr = i * 8;
                if let Ok(value) = memory.fetch(addr) {
                    print!("{} ", value);
                }
            }
        }
    }

    // New methods for stack pointer access
    pub fn get_sp(&self) -> usize {
        self.sp
    }

    pub fn set_sp(&mut self, sp: usize) {
        self.sp = sp;
    }

    // Helper to get raw pointer (for future use)
    pub fn as_mut_ptr(&mut self) -> usize {
        self.sp
    }

    // Get mutable pointer to stack pointer (for JIT)
    pub fn sp_mut_ptr(&mut self) -> *mut usize {
        &mut self.sp as *mut usize
    }
}
