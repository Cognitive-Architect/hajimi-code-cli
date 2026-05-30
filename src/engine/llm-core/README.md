# engine-llm-core

Hajimi IDE 桌面端的 LLM 统一核心引擎模块。

## 功能列表

- **统一客户端抽象** (`LlmClient` trait):
  - 单轮文本流式补全 (`stream_chat`)
  - 多轮上下文流式补全 (`stream_chat_with_context`)
  - **Tool Native 驱动补全 (`stream_chat_with_tools`)** —— 支持工具模型可见、参数验证、模型自主规划执行。
- **三两大 LLM Provider 支持**:
  - **OpenAI Client** (`OpenAiClient`):
    - 支持 GPT-4 / GPT-3.5 系列，兼容 DeepSeek / ChatGLM / Qwen 等所有主流 OpenAI 兼容的第三方大模型。
    - **支持 `tools` 动态暴露与 `tool_choice: auto / none / required` 参数化注入** [已在 Day 05 实装]。
    - 独家支持 DeepSeek 格式的 `reasoning_content` (推理过程/思维链) 提取并在流式段中智能拼接为 `<thinking>` 标签，提供全栈可折叠 Thinking UI 能力。
  - **Anthropic Claude Client** (`AnthropicClient`):
    - 支持 Claude 3.5 Sonnet 等云端模型，支持 standard system prompts。
  - **Ollama Local Client** (`OllamaClient`):
    - 支持本地大模型流式响应。
- **精确 Token 统计管线** (Tiktoken Scheme B):
  - 集成 `tiktoken-rs` (cl100k_base)，支持精确到单个 token 的估算（误差 0%），具备模型级首尾裁剪、省略机制和持久化使用小票。
- **Secrecy 级安全加固**:
  - 核心 API Key 在内存中使用 `secrecy::SecretString` 承载，防止通过内存调试或非安全日志输出泄漏。

## 目录结构

```
F:\hajimi-code-cli\src\engine\llm-core/
├── Cargo.toml      # Crate 依赖（tokio, serde, secrecy, log, reqwest）
├── README.md       # 本文档
└── src/
    ├── mod.rs      # Unified interface definitions & common utils
    ├── error.rs    # Unified LlmClient error variants
    ├── streaming/  # Flowable streams and chunk variants
    ├── anthropic.rs# Anthropic Claude API adapter
    ├── ollama.rs   # Ollama local inference client
    └── openai.rs   # OpenAI GPT-4 with tools injection adapter
```

## 测试覆盖

通过以下命令进行完整验证：
```bash
cargo test -p engine-llm-core
```
当前包含针对 OpenAI SSE 解析、DeepSeek 思维链格式、错误响应降级及 `tools` 序列化的完整测试用例。
