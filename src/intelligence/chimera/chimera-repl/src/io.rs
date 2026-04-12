//! I/O abstraction layer - InputSource trait for testable input.
use async_trait::async_trait;

/// Abstract input source for REPL input operations.
#[async_trait]
pub trait InputSource: Send {
    /// Read a line of input. Returns None on EOF.
    async fn read_line(&mut self) -> Option<String>;
}

/// Standard input implementation using tokio::io::Stdin.
pub struct StdinInput {
    reader: tokio::io::BufReader<tokio::io::Stdin>,
}

impl Default for StdinInput {
    fn default() -> Self {
        Self { reader: tokio::io::BufReader::new(tokio::io::stdin()) }
    }
}

#[async_trait]
impl InputSource for StdinInput {
    async fn read_line(&mut self) -> Option<String> {
        use tokio::io::AsyncBufReadExt;
        let mut line = String::new();
        match self.reader.read_line(&mut line).await {
            Ok(0) => None,
            Ok(_) => Some(line.trim_end().to_string()),
            Err(_) => None,
        }
    }
}

/// Mock input for testing with predefined inputs.
pub struct MockInput {
    inputs: Vec<String>,
    index: usize,
}

impl MockInput {
    /// Create mock input with predefined lines.
    pub fn with_input(inputs: Vec<String>) -> Self {
        Self { inputs, index: 0 }
    }
}

#[async_trait]
impl InputSource for MockInput {
    async fn read_line(&mut self) -> Option<String> {
        if self.index < self.inputs.len() {
            let line = self.inputs[self.index].clone();
            self.index += 1;
            Some(line)
        } else {
            None
        }
    }
}
