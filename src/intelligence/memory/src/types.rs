use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Embedding向量维度常量 - 384维用于轻量级语义检索
pub const EMBEDDING_DIMENSION: usize = 384;

/// 5层记忆架构的层标识
/// Session: 4K tokens - 短期会话
/// Auto: 128K context - 自动上下文窗口
/// Dream: 长期记忆 - 压缩存储
/// Graph: 知识图谱 - 结构化关系网络
/// Cloud: 云端备份 - 持久化存储
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryLayerId {
    Session,
    Auto,
    Dream,
    Graph,
    Cloud,
}

/// Token计数类型别名 - 用于跨层统一计量
pub type TokenCount = usize;

/// 通用记忆条目 - 跨层统一数据类型
/// 作为5层架构的核心数据载体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub tokens: TokenCount,
    pub timestamp: DateTime<Utc>,
    pub layer: MemoryLayerId,
}

impl MemoryEntry {
    /// 创建新的记忆条目
    pub fn new(id: String, content: String, tokens: TokenCount, layer: MemoryLayerId) -> Self {
        Self {
            id,
            content,
            tokens,
            timestamp: Utc::now(),
            layer,
        }
    }

    /// 获取条目的token数量
    pub fn token_count(&self) -> TokenCount {
        self.tokens
    }

    /// 获取条目所属层
    pub fn layer_id(&self) -> MemoryLayerId {
        self.layer
    }
}

/// 跨层转换trait - 实现层间数据流协议
/// 允许各层自定义类型转换为统一MemoryEntry
pub trait IntoMemoryEntry {
    fn into_entry(self) -> MemoryEntry;
}

/// 跨层数据流结果类型
/// 表示层间迁移操作的结果状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LayerFlowResult {
    Success(MemoryEntry),
    Skipped { reason: String },
}

/// Dream层记忆条目 - 支持向量嵌入的长期记忆
/// 用于语义检索和长期知识保留
#[derive(Clone, Debug)]
pub struct DreamEntry {
    pub id: String,
    pub content: String,
    pub embedding: [f32; EMBEDDING_DIMENSION],
    pub timestamp: DateTime<Utc>,
    pub layer: MemoryLayerId,
}

impl Serialize for DreamEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DreamEntry", 5)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("embedding", &self.embedding.to_vec())?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.serialize_field("layer", &self.layer)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for DreamEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DreamEntryHelper {
            id: String,
            content: String,
            embedding: Vec<f32>,
            timestamp: DateTime<Utc>,
            layer: MemoryLayerId,
        }
        let helper = DreamEntryHelper::deserialize(deserializer)?;
        let mut embedding = [0.0f32; EMBEDDING_DIMENSION];
        if helper.embedding.len() == EMBEDDING_DIMENSION {
            embedding.copy_from_slice(&helper.embedding);
        }
        Ok(DreamEntry {
            id: helper.id,
            content: helper.content,
            embedding,
            timestamp: helper.timestamp,
            layer: helper.layer,
        })
    }
}

/// 跨层统一接口 - 定义层间交互契约
/// 所有记忆层实现此trait以支持统一调度
pub trait MemoryLayer: Send + Sync {
    fn layer_id(&self) -> MemoryLayerId;
    fn capacity(&self) -> TokenCount;
}

/// 线程安全的记忆存储接口
/// 所有存储后端需实现此trait以保证Send + Sync
pub trait MemoryStorage: Send + Sync {
    fn store(&self, entry: MemoryEntry) -> Result<(), String>;
    fn retrieve(&self, id: &str) -> Result<Option<MemoryEntry>, String>;
}
