use crate::{ChatMessage, LlmClient, LlmProvider};

fn test_client() -> crate::OpenAiClient {
    crate::OpenAiClient::new(LlmProvider::OpenAi {
        api_key: secrecy::SecretString::new("test".into()),
        model: "gpt-4".to_string(),
        base_url: "https://api.openai.com".to_string(),
    })
}

fn heuristic(messages: &[ChatMessage]) -> usize {
    crate::heuristic_token_count(messages)
}

#[test]
fn test_token_counting_empty() {
    let client = test_client();
    let count = client.count_tokens(vec![], "gpt-4").unwrap();
    assert_eq!(count, heuristic(&[]));
}

#[test]
fn test_token_counting_simple_english() {
    let client = test_client();
    let msgs = vec![ChatMessage {
        role: "user".into(),
        content: "Hello world".into(),
        timestamp: None,
    }];
    let count = client.count_tokens(msgs.clone(), "gpt-4").unwrap();
    assert!(count > 0);
    assert!(count > 0);
}

#[test]
fn test_token_counting_simple_chinese() {
    let client = test_client();
    let msgs = vec![ChatMessage {
        role: "user".into(),
        content: "你好世界".into(),
        timestamp: None,
    }];
    let count = client.count_tokens(msgs.clone(), "gpt-4").unwrap();
    assert!(count > 0);
}

#[test]
fn test_token_counting_mixed() {
    let client = test_client();
    let msgs = vec![ChatMessage {
        role: "user".into(),
        content: "Hello 你好 world 世界".into(),
        timestamp: None,
    }];
    let count = client.count_tokens(msgs.clone(), "gpt-4").unwrap();
    assert!(count > 0);
}

#[test]
fn test_token_counting_multi_turn() {
    let client = test_client();
    let msgs = vec![
        ChatMessage {
            role: "system".into(),
            content: "You are helpful.".into(),
            timestamp: None,
        },
        ChatMessage {
            role: "user".into(),
            content: "What is Rust?".into(),
            timestamp: None,
        },
        ChatMessage {
            role: "assistant".into(),
            content: "Rust is a systems language.".into(),
            timestamp: None,
        },
        ChatMessage {
            role: "user".into(),
            content: "Tell me more.".into(),
            timestamp: None,
        },
    ];
    let count = client.count_tokens(msgs.clone(), "gpt-4").unwrap();
    let h = heuristic(&msgs);
    println!("multi_turn exact:{} heuristic:{}", count, h);
    assert!(count > 30, "multi-turn should have >30 tokens");
    let err = ((h as f64 - count as f64) / count as f64).abs() * 100.0;
    assert!(err < 20.0, "multi-turn error {:.1}% exceeds 20%", err);
}

#[cfg(feature = "exact-tokens")]
#[test]
fn test_token_counting_vs_heuristic() {
    let client = test_client();
    let cases: Vec<Vec<ChatMessage>> = vec![
        vec![ChatMessage {
            role: "user".into(),
            content: "Hello world this is a test message with some english words.".into(),
            timestamp: None,
        }],
        vec![ChatMessage {
            role: "user".into(),
            content: "这是一个中文测试消息，包含一些中文字符和标点符号。".into(),
            timestamp: None,
        }],
        vec![ChatMessage {
            role: "user".into(),
            content: "Mixed 混合 content 内容 with 一些 english 英文.".into(),
            timestamp: None,
        }],
        vec![
            ChatMessage {
                role: "system".into(),
                content: "You are helpful.".into(),
                timestamp: None,
            },
            ChatMessage {
                role: "user".into(),
                content: "Explain quantum computing in simple terms.".into(),
                timestamp: None,
            },
        ],
        vec![
            ChatMessage {
                role: "system".into(),
                content: "You are helpful.".into(),
                timestamp: None,
            },
            ChatMessage {
                role: "user".into(),
                content: "Hello".into(),
                timestamp: None,
            },
            ChatMessage {
                role: "assistant".into(),
                content: "Hi there! How can I help?".into(),
                timestamp: None,
            },
            ChatMessage {
                role: "user".into(),
                content: "What is the weather?".into(),
                timestamp: None,
            },
        ],
    ];
    let mut max_err = 0.0_f64;
    for msgs in &cases {
        let exact = client.count_tokens(msgs.clone(), "gpt-4").unwrap();
        let h = heuristic(msgs);
        let err = if exact > 0 {
            ((h as f64 - exact as f64) / exact as f64).abs() * 100.0
        } else {
            0.0
        };
        if err > max_err {
            max_err = err;
        }
        println!("exact:{} heuristic:{} err:{:.1}%", exact, h, err);
    }
    println!("max_error:{:.1}%", max_err);
    assert!(max_err < 20.0, "max error {:.1}% exceeds 20%", max_err);
}

#[test]
fn test_normalize_model() {
    assert_eq!(
        crate::normalize_model_for_tiktoken("claude-3-sonnet"),
        "gpt-4"
    );
    assert_eq!(crate::normalize_model_for_tiktoken("gpt-4"), "gpt-4");
    assert_eq!(
        crate::normalize_model_for_tiktoken("gpt-3.5-turbo"),
        "gpt-3.5-turbo"
    );
    assert_eq!(crate::normalize_model_for_tiktoken("llama3"), "gpt-4");
}

#[test]
fn test_token_counting_long_message() {
    let client = test_client();
    let msgs = vec![ChatMessage {
        role: "user".into(),
        content: "word ".repeat(500),
        timestamp: None,
    }];
    let count = client.count_tokens(msgs, "gpt-4").unwrap();
    assert!(count > 100);
}
