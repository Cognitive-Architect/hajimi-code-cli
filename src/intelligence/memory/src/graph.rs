#[derive(Debug, Clone)]
pub struct GraphMemory;

impl GraphMemory {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GraphMemory {
    fn default() -> Self {
        Self::new()
    }
}
