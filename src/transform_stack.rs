use glam::Mat4;

// Transformation stack for storing transforms
pub struct TransformStack {
    stack: Vec<Mat4>,
}

impl TransformStack {
    pub fn new() -> Self {
        Self {
            stack: vec![Mat4::IDENTITY],
        }
    }

    // Add transform to the stack
    pub fn push(&mut self, transform: Mat4) {
        let current = *self.stack.last().unwrap();
        self.stack.push(current * transform);
    }

    // Remove transform from stack
    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    // Get top transform from the stack 
    pub fn current(&self) -> Mat4 {
        *self.stack.last().unwrap()
    }
}