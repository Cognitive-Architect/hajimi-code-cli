//! SSE (Server-Sent Events) serialization for StreamChunk
//! DEBT-W02-004: SSE format serialization

use crate::streaming::types::StreamChunk;

/// Convert StreamChunk to SSE format string
pub fn to_sse(chunk: &StreamChunk) -> String {
    match chunk {
        StreamChunk::Output(data) => {
            format!("data: {}\n\n", escape_sse(data))
        }
        StreamChunk::Error(msg) => {
            format!("event: error\ndata: {}\n\n", escape_sse(msg))
        }
        StreamChunk::Done => {
            "event: done\n\n".to_string()
        }
        StreamChunk::Heartbeat => {
            ":heartbeat\n\n".to_string()
        }
    }
}

/// Escape special characters for SSE format
/// Handles newlines by prefixing each line with "data: "
fn escape_sse(data: &str) -> String {
    data.replace('\n', "\ndata: ")
}

/// Extension trait for StreamChunk to provide SSE conversion
impl StreamChunk {
    /// Convert to SSE format
    pub fn to_sse(&self) -> String {
        to_sse(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_to_sse() {
        let chunk = StreamChunk::Output("Hello".to_string());
        assert_eq!(chunk.to_sse(), "data: Hello\n\n");
    }

    #[test]
    fn test_error_to_sse() {
        let chunk = StreamChunk::Error("failed".to_string());
        assert_eq!(chunk.to_sse(), "event: error\ndata: failed\n\n");
    }

    #[test]
    fn test_done_to_sse() {
        let chunk = StreamChunk::Done;
        assert_eq!(chunk.to_sse(), "event: done\n\n");
    }

    #[test]
    fn test_heartbeat_to_sse() {
        let chunk = StreamChunk::Heartbeat;
        assert_eq!(chunk.to_sse(), ":heartbeat\n\n");
    }

    #[test]
    fn test_multiline_escape() {
        let chunk = StreamChunk::Output("line1\nline2".to_string());
        assert_eq!(chunk.to_sse(), "data: line1\ndata: line2\n\n");
    }
}
