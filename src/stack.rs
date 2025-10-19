pub struct Stack {
    data: Vec<i32>,
}

impl Stack {
    pub fn new() -> Self {
        Stack { data: Vec::new() }
    }

    pub fn push(&mut self, value: i32) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Option<i32> {
        self.data.pop()
    }

    pub fn peek(&self) -> Option<&i32> {
        self.data.last()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn print_stack(&self) {
        if self.data.is_empty() {
            print!("<0> ");
        } else {
            print!("<{}> ", self.data.len());
            for item in &self.data {
                print!("{} ", item);
            }
        }
    }
}
