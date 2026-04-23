//! StreamOutput: Buffered terminal output component for async rendering.
//! Writes bytes into an internal buffer and flushes to String on demand.

use std::io::{self, Write};

/// Buffered output sink for terminal stream content.
pub struct StreamOutput {
    buffer: Vec<u8>,
}

impl StreamOutput {
    /// Create an empty output buffer.
    pub fn new() -> Self { Self { buffer: Vec::new() } }

    /// Write data into the buffer.
    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.buffer.extend_from_slice(data);
        Ok(())
    }

    /// Flush buffer to a String and clear it.
    pub fn flush(&mut self) -> io::Result<String> {
        if self.buffer.is_empty() { return Ok(String::new()); }
        let s = String::from_utf8_lossy(&self.buffer).to_string();
        self.buffer.clear();
        Ok(s)
    }

    /// Check if buffer is empty.
    pub fn is_empty(&self) -> bool { self.buffer.is_empty() }

    /// Current buffered length.
    pub fn len(&self) -> usize { self.buffer.len() }
}

impl Default for StreamOutput {
    fn default() -> Self { Self::new() }
}

impl Write for StreamOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
