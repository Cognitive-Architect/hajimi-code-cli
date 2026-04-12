//! SSE format serialization tests
//! Tests for B-04: SSE format serialization

use hajimi_core::streaming::StreamChunk;

/// Test 1: SSE format for Output chunk
#[test]
fn test_sse_output_format() {
    let chunk = StreamChunk::Output("Hello World".to_string());
    let sse = chunk.to_sse();
    
    // Must start with "data: " and end with double newline
    assert!(sse.starts_with("data: "));
    assert!(sse.ends_with("\n\n"));
    assert_eq!(sse, "data: Hello World\n\n");
}

/// Test 2: Error event format
#[test]
fn test_sse_error_event() {
    let chunk = StreamChunk::Error("Something went wrong".to_string());
    let sse = chunk.to_sse();
    
    // Must contain "event: error" and "data: "
    assert!(sse.contains("event: error"));
    assert!(sse.contains("data: "));
    assert!(sse.ends_with("\n\n"));
    assert_eq!(sse, "event: error\ndata: Something went wrong\n\n");
}

/// Test 3: Done event format
#[test]
fn test_sse_done_event() {
    let chunk = StreamChunk::Done;
    let sse = chunk.to_sse();
    
    assert_eq!(sse, "event: done\n\n");
}

/// Test 4: Heartbeat comment format
#[test]
fn test_sse_heartbeat_format() {
    let chunk = StreamChunk::Heartbeat;
    let sse = chunk.to_sse();
    
    // Heartbeat is SSE comment format (starts with :)
    assert!(sse.starts_with(":"));
    assert!(sse.contains("heartbeat"));
    assert!(sse.ends_with("\n\n"));
    assert_eq!(sse, ":heartbeat\n\n");
}

/// Test 5: Multiline data escape
#[test]
fn test_sse_multiline_escape() {
    let multiline = "First line\nSecond line\nThird line";
    let chunk = StreamChunk::Output(multiline.to_string());
    let sse = chunk.to_sse();
    
    // Each line should have "data: " prefix
    let lines: Vec<&str> = sse.trim_end().split('\n').collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("data: First"));
    assert!(lines[1].starts_with("data: Second"));
    assert!(lines[2].starts_with("data: Third"));
}

/// Test 6: Empty string handling
#[test]
fn test_sse_empty_output() {
    let chunk = StreamChunk::Output("".to_string());
    let sse = chunk.to_sse();
    
    assert_eq!(sse, "data: \n\n");
}

/// Test 7: Special characters in error message
#[test]
fn test_sse_error_with_newlines() {
    let error_msg = "Error at line 1\nError at line 2";
    let chunk = StreamChunk::Error(error_msg.to_string());
    let sse = chunk.to_sse();
    
    assert!(sse.contains("event: error"));
    assert!(sse.contains("data: Error at line 1"));
    assert!(sse.contains("data: Error at line 2"));
}

/// Test 8: Standalone to_sse function
#[test]
fn test_to_sse_function() {
    use hajimi_core::streaming::sse::to_sse;
    
    let chunk = StreamChunk::Output("test".to_string());
    let sse = to_sse(&chunk);
    
    assert_eq!(sse, "data: test\n\n");
}

/// Test 9: Verify SSE fields are present
#[test]
fn test_sse_required_fields() {
    let output = StreamChunk::Output("test".to_string()).to_sse();
    assert!(output.contains("data:"));
    
    let error = StreamChunk::Error("test".to_string()).to_sse();
    assert!(error.contains("event:"));
    assert!(error.contains("data:"));
    
    let done = StreamChunk::Done.to_sse();
    assert!(done.contains("event:"));
    
    let heartbeat = StreamChunk::Heartbeat.to_sse();
    assert!(heartbeat.contains(":"));
}

/// Test 10: End-to-end SSE stream simulation
#[test]
fn test_sse_end_to_end_stream() {
    let chunks = vec![
        StreamChunk::Output("Message 1".to_string()),
        StreamChunk::Heartbeat,
        StreamChunk::Output("Message 2".to_string()),
        StreamChunk::Done,
    ];
    
    let stream_output: String = chunks.iter().map(|c| c.to_sse()).collect();
    
    // Should be consumable by EventSource
    assert!(stream_output.contains("data: Message 1"));
    assert!(stream_output.contains(":heartbeat"));
    assert!(stream_output.contains("data: Message 2"));
    assert!(stream_output.contains("event: done"));
    
    // Each message separated by empty line
    let messages: Vec<&str> = stream_output.split("\n\n").filter(|s| !s.is_empty()).collect();
    assert_eq!(messages.len(), 4);
}
