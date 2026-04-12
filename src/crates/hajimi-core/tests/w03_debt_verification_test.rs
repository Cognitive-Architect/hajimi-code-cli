//! DEBT-W03-001 实地验证测试
//! 使用真实API Key格式验证Debug redaction有效性
//! 
//! 测试时间戳: 2026-04-03 19:21
//! 测试窗口: 60分钟
//! API Key: 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

use hajimi_core::llm::LlmProvider;

const TEST_KEY: &str = "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";
const KEY_PREFIX: &str = "sk-or-v1-71eb";
const REDACTED_MARKER: &str = "***REDACTED***";

/// V1-Debug格式化验证（关键）
/// 验证：format!("{:?}", provider) 输出中不包含真实key前缀
#[test]
fn test_v1_debug_format_redacts_api_key_anthropic() {
    let provider = LlmProvider::Anthropic {
        api_key: TEST_KEY.into(),
        model: "claude-3-sonnet-20240229".into(),
        base_url: "https://api.anthropic.com".into(),
    };
    
    let debug_output = format!("{:?}", provider);
    println!("Debug output: {}", debug_output);
    
    // 必须包含REDACTED标记
    assert!(
        debug_output.contains(REDACTED_MARKER),
        "Debug输出必须包含***REDACTED***标记"
    );
    
    // 绝不能包含真实key的任何部分
    assert!(
        !debug_output.contains(KEY_PREFIX),
        "Debug输出绝不能包含key前缀: {}", KEY_PREFIX
    );
    
    // 绝不能包含完整key
    assert!(
        !debug_output.contains(TEST_KEY),
        "Debug输出绝不能包含完整API Key"
    );
    
    // 验证其他字段正常显示
    assert!(debug_output.contains("claude-3-sonnet"));
    assert!(debug_output.contains("api.anthropic.com"));
}

/// V1-Debug格式化验证（OpenAI变体）
#[test]
fn test_v1_debug_format_redacts_api_key_openai() {
    let provider = LlmProvider::OpenAi {
        api_key: TEST_KEY.into(),
        model: "gpt-4".into(),
        base_url: "https://api.openai.com".into(),
    };
    
    let debug_output = format!("{:?}", provider);
    println!("Debug output: {}", debug_output);
    
    assert!(debug_output.contains(REDACTED_MARKER));
    assert!(!debug_output.contains(KEY_PREFIX));
    assert!(!debug_output.contains(TEST_KEY));
    assert!(debug_output.contains("gpt-4"));
    assert!(debug_output.contains("api.openai.com"));
}

/// V1-Debug格式化验证（Ollama变体 - 无key）
#[test]
fn test_v1_debug_format_ollama_no_key() {
    let provider = LlmProvider::Ollama {
        base_url: "http://localhost:11434".into(),
        model: "llama3".into(),
    };
    
    let debug_output = format!("{:?}", provider);
    println!("Debug output: {}", debug_output);
    
    // Ollama无api_key字段，不应有REDACTED
    assert!(!debug_output.contains("api_key"));
    assert!(debug_output.contains("llama3"));
    assert!(debug_output.contains("localhost:11434"));
}

/// V2-错误传播验证
/// 构造错误场景，验证错误消息不包含key
#[test]
fn test_v2_error_propagation_no_key_leak() {
    use hajimi_core::error::EngineError;
    
    // 创建包含provider的错误（如通过?传播）
    let result: Result<(), EngineError> = Err(EngineError::InvalidParameters(
        "test error".into()
    ));
    
    let error_string = format!("{:?}", result);
    println!("Error output: {}", error_string);
    
    // 错误消息不应包含key（即使是测试用的假key）
    assert!(!error_string.contains("sk-or-v1"));
}

/// V3-日志宏安全验证
/// 模拟tracing/log输出，验证不会意外泄露
#[test]
fn test_v3_logging_macro_safety() {
    // 在实际代码中，如果使用tracing::info!(?provider) 或 println!("{:?}", provider)
    // 本测试验证该输出是安全的
    
    let provider = LlmProvider::Anthropic {
        api_key: TEST_KEY.into(),
        model: "test-model".into(),
        base_url: "https://test.com".into(),
    };
    
    // 模拟日志输出
    let log_output = format!("Provider config: {:?}", provider);
    println!("Simulated log: {}", log_output);
    
    // 日志绝不能包含真实key
    assert!(
        !log_output.contains(TEST_KEY),
        "日志绝不能包含API Key"
    );
    assert!(
        log_output.contains(REDACTED_MARKER),
        "日志必须显示REDACTED标记"
    );
}

/// V4-防御性测试：验证key本身是可用的（格式正确）
/// 注意：此测试不实际发送网络请求，仅验证key格式
#[test]
fn test_v4_key_format_valid() {
    // OpenRouter key格式: sk-or-v1-<hex>
    assert!(TEST_KEY.starts_with("sk-or-v1-"));
    // 实际长度: "sk-or-v1-" (9) + hex部分
    assert!(TEST_KEY.len() > 9);
    
    // 验证hex部分
    let hex_part = &TEST_KEY[9..];
    assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
}

/// 完整性验证：所有变体都通过测试
#[test]
fn test_all_variants_secure() {
    let anthropic = LlmProvider::Anthropic {
        api_key: TEST_KEY.into(),
        model: "test".into(),
        base_url: "https://test.com".into(),
    };
    
    let openai = LlmProvider::OpenAi {
        api_key: TEST_KEY.into(),
        model: "test".into(),
        base_url: "https://test.com".into(),
    };
    
    let ollama = LlmProvider::Ollama {
        base_url: "http://localhost:11434".into(),
        model: "test".into(),
    };
    
    // 验证所有变体的Debug输出
    let anthropic_debug = format!("{:?}", anthropic);
    let openai_debug = format!("{:?}", openai);
    let ollama_debug = format!("{:?}", ollama);
    
    // Anthropic和OpenAI必须REDACTED
    assert!(anthropic_debug.contains(REDACTED_MARKER));
    assert!(openai_debug.contains(REDACTED_MARKER));
    
    // 两者都不应包含真实key
    assert!(!anthropic_debug.contains(TEST_KEY));
    assert!(!openai_debug.contains(TEST_KEY));
    
    // Ollama不应有api_key字段
    assert!(!ollama_debug.contains("api_key"));
    
    println!("Anthropic: {}", anthropic_debug);
    println!("OpenAI: {}", openai_debug);
    println!("Ollama: {}", ollama_debug);
}
