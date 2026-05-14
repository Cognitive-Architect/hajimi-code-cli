//! Context Window Manager — Token budget management and priority-based context assembly.
//!
//! Blocks are included by priority (P0 highest, P4 lowest). P0 overflow is a hard error.
//! P1 blocks attempt compaction before omission. P2–P4 are omitted when budget is exceeded.
//! This module has no LLM, network, or async dependencies — pure synchronous logic.

use std::fmt;

/// Priority level for a context block (P0 = must-include, P4 = lowest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContextPriority {
    /// Critical — overflow is a hard error.
    P0,
    /// High — attempt compaction before omission.
    P1,
    /// Medium — omitted when budget exceeded.
    P2,
    /// Low — omitted when budget exceeded.
    P3,
    /// Lowest — omitted first when budget exceeded.
    P4,
}

impl fmt::Display for ContextPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::P0 => write!(f, "P0"),
            Self::P1 => write!(f, "P1"),
            Self::P2 => write!(f, "P2"),
            Self::P3 => write!(f, "P3"),
            Self::P4 => write!(f, "P4"),
        }
    }
}

/// Content format of a context block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    /// System prompt (highest structural importance).
    SystemPrompt,
    /// Structured JSON payload.
    Json,
    /// Plain text content.
    Text,
    /// Markdown-formatted content.
    Markdown,
}

/// A single block of context to be included in the LLM prompt.
#[derive(Debug, Clone)]
pub struct ContextBlock {
    /// Human-readable name identifying this block (e.g. "system_prompt").
    pub name: String,
    /// Priority level — determines inclusion order and overflow behaviour.
    pub priority: ContextPriority,
    /// Format of the content payload.
    pub content_type: ContentType,
    /// The actual content string.
    pub content: String,
    /// Estimated token count for this block.
    pub token_estimate: usize,
    /// Whether this block may be truncated to fit the budget.
    pub truncatable: bool,
}

/// Tracks token budget consumption during assembly.
#[derive(Debug, Clone)]
pub struct TokenAccount {
    /// Maximum tokens allowed.
    pub limit: usize,
    /// Tokens consumed so far.
    pub used: usize,
}

impl TokenAccount {
    /// Create a new account with the given token limit.
    pub fn new(limit: usize) -> Self {
        Self { limit, used: 0 }
    }
    /// Remaining tokens available.
    pub fn remaining(&self) -> usize {
        self.limit.saturating_sub(self.used)
    }
    /// Try to consume `amount` tokens. Returns `true` on success.
    pub fn try_consume(&mut self, amount: usize) -> bool {
        if amount <= self.remaining() {
            self.used += amount;
            true
        } else {
            false
        }
    }
}

/// Record of a block that was omitted during assembly.
#[derive(Debug, Clone)]
pub struct OmittedBlock {
    /// Name of the omitted block.
    pub name: String,
    /// Priority of the omitted block.
    pub priority: ContextPriority,
    /// Token estimate of the omitted block.
    pub token_estimate: usize,
    /// Reason it was omitted.
    pub reason: String,
}

/// Result of a successful context assembly.
#[derive(Debug, Clone)]
pub struct AssembledContext {
    /// Blocks included in the final context.
    pub blocks: Vec<ContextBlock>,
    /// Blocks that were omitted (with reasons).
    pub omitted: Vec<OmittedBlock>,
    /// Total tokens consumed by included blocks.
    pub total_tokens: usize,
}

/// Errors that can occur during context assembly.
#[derive(Debug, Clone)]
pub enum ContextError {
    /// A P0 (critical) block cannot fit within the remaining budget.
    Overflow(ContextPriority),
    /// The token limit is zero or nonsensical.
    InvalidLimit(String),
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow(p) => write!(f, "context overflow: {} block exceeds budget", p),
            Self::InvalidLimit(msg) => write!(f, "invalid token limit: {}", msg),
        }
    }
}

impl std::error::Error for ContextError {}

/// Manages context window assembly within a fixed token budget.
///
/// Accepts [`ContextBlock`]s and assembles them into an [`AssembledContext`]
/// respecting priority ordering and budget constraints.
pub struct ContextWindowManager {
    token_limit: usize,
}

impl ContextWindowManager {
    /// Create a new manager with the given token limit.
    pub fn new(token_limit: usize) -> Self {
        Self { token_limit }
    }

    /// Assemble context blocks into a budget-constrained prompt.
    ///
    /// # Algorithm
    /// 1. Sort blocks by priority (P0 first, P4 last).
    /// 2. P0 must fit — overflow returns `Err(ContextError::Overflow(P0))`.
    /// 3. P1 tries `compact_block`; P2–P4 omitted when budget exceeded.
    /// 4. Returns included blocks, omitted list, and total tokens.
    pub fn assemble(&self, blocks: Vec<ContextBlock>) -> Result<AssembledContext, ContextError> {
        if self.token_limit == 0 && !blocks.is_empty() {
            return Err(ContextError::InvalidLimit(
                "token limit is 0 but blocks are provided".to_string(),
            ));
        }
        let mut sorted = blocks;
        sorted.sort_by_key(|b| b.priority);
        let mut account = TokenAccount::new(self.token_limit);
        let mut included: Vec<ContextBlock> = Vec::new();
        let mut omitted: Vec<OmittedBlock> = Vec::new();

        for block in sorted {
            match block.priority {
                ContextPriority::P0 => {
                    if !account.try_consume(block.token_estimate) {
                        return Err(ContextError::Overflow(ContextPriority::P0));
                    }
                    included.push(block);
                }
                ContextPriority::P1 => {
                    if account.try_consume(block.token_estimate) {
                        included.push(block);
                    } else if let Some(c) = Self::compact_block(&block) {
                        if account.try_consume(c.token_estimate) {
                            included.push(c);
                        } else {
                            omitted.push(Self::omit(&block, "P1 compacted but exceeds budget"));
                        }
                    } else {
                        omitted.push(Self::omit(&block, "P1 compact unavailable"));
                    }
                }
                ContextPriority::P2 | ContextPriority::P3 | ContextPriority::P4 => {
                    if account.try_consume(block.token_estimate) {
                        included.push(block);
                    } else {
                        let reason = format!("{} omitted, budget exceeded", block.priority);
                        omitted.push(OmittedBlock {
                            name: block.name,
                            priority: block.priority,
                            token_estimate: block.token_estimate,
                            reason,
                        });
                    }
                }
            }
        }
        Ok(AssembledContext {
            total_tokens: account.used,
            blocks: included,
            omitted,
        })
    }

    /// Attempt to compact a block to reduce its token footprint.
    ///
    /// Compaction strategy varies by `ContentType`:
    /// - **SystemPrompt** → keep first 60% of content (core instructions).
    /// - **Json** → truncate to first 50% (keep schema, drop verbose descriptions).
    /// - **Text** / **Markdown** → keep first 40% as summary.
    ///
    /// Returns `Some(compacted_block)` if compaction produced a smaller block,
    /// `None` if the block is not truncatable or compaction would not help.
    fn compact_block(block: &ContextBlock) -> Option<ContextBlock> {
        if !block.truncatable || block.content.is_empty() {
            return None;
        }
        // Compaction ratio by ContentType: SystemPrompt keeps more, Text/Markdown less
        let ratio = match block.content_type {
            ContentType::SystemPrompt => 0.6,
            ContentType::Json => 0.5,
            ContentType::Text | ContentType::Markdown => 0.4,
        };
        let target_len = (block.content.len() as f64 * ratio) as usize;
        // Find a char boundary at or before target_len
        let end = find_char_boundary(&block.content, target_len);
        if end >= block.content.len() {
            return None; // No compaction possible
        }
        let compacted_content = format!("{}…[truncated]", &block.content[..end]);
        let compacted_tokens = estimate_tokens(&compacted_content);
        Some(ContextBlock {
            name: block.name.clone(),
            priority: block.priority,
            content_type: block.content_type,
            content: compacted_content,
            token_estimate: compacted_tokens,
            truncatable: false, // Already compacted, don't compact again
        })
    }

    /// Helper: create an OmittedBlock record from a block reference.
    fn omit(block: &ContextBlock, reason: &str) -> OmittedBlock {
        OmittedBlock {
            name: block.name.clone(),
            priority: block.priority,
            token_estimate: block.token_estimate,
            reason: reason.to_string(),
        }
    }
}

/// Estimate token count for a content string using heuristics.
///
/// Uses language-aware estimation:
/// - **CJK characters** (Chinese/Japanese/Korean): ~0.9 tokens per character.
/// - **ASCII words**: ~1.3 tokens per word.
///
/// This is a fallback when `LlmClient::count_tokens` is unavailable.
/// Note: Real LLMs usually use a tokenizer, but we fallback to heuristics here.
pub fn estimate_tokens(content: &str) -> usize {
    if content.is_empty() {
        return 0;
    }
    let mut cjk_chars = 0usize;
    let mut ascii_words = 0usize;
    let mut in_word = false;
    for ch in content.chars() {
        if is_cjk(ch) {
            cjk_chars += 1;
            in_word = false;
        } else if ch.is_ascii_alphanumeric() || ch == '_' {
            if !in_word {
                ascii_words += 1;
                in_word = true;
            }
        } else {
            in_word = false;
        }
    }
    // Chinese/CJK: 0.9 tokens/char, English: 1.3 tokens/word
    let cjk_tokens = (cjk_chars as f64 * 0.9).ceil() as usize;
    let ascii_tokens = (ascii_words as f64 * 1.3).ceil() as usize;
    // Minimum 1 token for non-empty content
    (cjk_tokens + ascii_tokens).max(1)
}

/// Check if a character is in the CJK Unified Ideographs range.
fn is_cjk(ch: char) -> bool {
    matches!(ch, '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '\u{F900}'..='\u{FAFF}')
}

/// Find the nearest char boundary at or before `pos` in `s`.
fn find_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    let mut boundary = pos;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    boundary
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_block(name: &str, priority: ContextPriority, tokens: usize) -> ContextBlock {
        ContextBlock {
            name: name.to_string(),
            priority,
            content_type: ContentType::Text,
            content: "test".to_string(),
            token_estimate: tokens,
            truncatable: false,
        }
    }

    #[test]
    fn test_empty_blocks_returns_empty_context() {
        let mgr = ContextWindowManager::new(1000);
        let result = mgr.assemble(vec![]).expect("should succeed");
        assert!(result.blocks.is_empty());
        assert!(result.omitted.is_empty());
        assert_eq!(result.total_tokens, 0);
    }

    #[test]
    fn test_p0_overflow_returns_error() {
        let mgr = ContextWindowManager::new(10);
        let block = make_block("big", ContextPriority::P0, 100);
        let err = mgr.assemble(vec![block]).unwrap_err();
        assert!(matches!(err, ContextError::Overflow(ContextPriority::P0)));
    }

    #[test]
    fn test_p2_omitted_when_budget_full() {
        let mgr = ContextWindowManager::new(50);
        let blocks = vec![
            make_block("sys", ContextPriority::P0, 40),
            make_block("extra", ContextPriority::P2, 20),
        ];
        let result = mgr.assemble(blocks).expect("should succeed");
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(result.omitted.len(), 1);
        assert_eq!(result.omitted[0].name, "extra");
    }

    #[test]
    fn test_all_priorities_fit() {
        let mgr = ContextWindowManager::new(500);
        let blocks = vec![
            make_block("p0", ContextPriority::P0, 100),
            make_block("p1", ContextPriority::P1, 100),
            make_block("p2", ContextPriority::P2, 100),
            make_block("p3", ContextPriority::P3, 100),
            make_block("p4", ContextPriority::P4, 100),
        ];
        let result = mgr.assemble(blocks).expect("should succeed");
        assert_eq!(result.blocks.len(), 5);
        assert_eq!(result.total_tokens, 500);
        assert!(result.omitted.is_empty());
    }

    #[test]
    fn test_token_account_remaining() {
        let mut acc = TokenAccount::new(100);
        assert_eq!(acc.remaining(), 100);
        assert!(acc.try_consume(60));
        assert_eq!(acc.remaining(), 40);
        assert!(!acc.try_consume(50));
        assert_eq!(acc.remaining(), 40);
    }

    #[test]
    fn test_compact_block_truncatable_text() {
        let block = ContextBlock {
            name: "notes".to_string(),
            priority: ContextPriority::P1,
            content_type: ContentType::Text,
            content: "a".repeat(100),
            token_estimate: 100,
            truncatable: true,
        };
        let compacted = ContextWindowManager::compact_block(&block);
        assert!(compacted.is_some());
        let c = compacted.unwrap();
        assert!(c.content.len() < block.content.len());
        assert!(c.content.ends_with("…[truncated]"));
    }

    #[test]
    fn test_compact_block_not_truncatable_returns_none() {
        let block = ContextBlock {
            name: "sys".to_string(),
            priority: ContextPriority::P0,
            content_type: ContentType::SystemPrompt,
            content: "important".to_string(),
            token_estimate: 5,
            truncatable: false,
        };
        assert!(ContextWindowManager::compact_block(&block).is_none());
    }

    #[test]
    fn test_estimate_tokens_english() {
        // "hello world" = 2 words → ceil(2 * 1.3) = 3
        let tokens = estimate_tokens("hello world");
        assert_eq!(tokens, 3);
    }

    #[test]
    fn test_estimate_tokens_chinese() {
        // 3 CJK chars → ceil(3 * 0.9) = 3
        let tokens = estimate_tokens("你好世");
        assert_eq!(tokens, 3);
    }

    #[test]
    fn test_estimate_tokens_mixed() {
        // "hello 你好" = 1 word (1.3→2) + 2 CJK (0.9*2=1.8→2) = 4
        let tokens = estimate_tokens("hello 你好");
        assert_eq!(tokens, 4);
    }

    #[test]
    fn test_estimate_tokens_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }
}
