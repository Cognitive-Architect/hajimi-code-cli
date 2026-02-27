-- ============================================================
-- Hajimi V3 Storage - 分片内 Schema
-- 
-- 每个分片数据库 (shard_XX.db) 使用此 Schema
-- ============================================================

-- 分片元信息表
CREATE TABLE IF NOT EXISTS shard_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 初始化分片元信息
INSERT OR IGNORE INTO shard_meta (key, value) VALUES 
    ('version', '3'),
    ('shard_id', '0'),
    ('chunk_count', '0'),
    ('created_at', CAST(strftime('%s', 'now') AS INTEGER)),
    ('updated_at', CAST(strftime('%s', 'now') AS INTEGER));

-- ============================================================
-- 核心表：分片存储
-- ============================================================

CREATE TABLE IF NOT EXISTS chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- SimHash 高64位（路由键）
    simhash_hi BIGINT NOT NULL,
    -- SimHash 低64位（完整标识）
    simhash_lo BIGINT NOT NULL,
    -- 数据哈希 (MD5-128, 32字节)
    md5 BLOB NOT NULL,
    -- 数据大小
    size INTEGER NOT NULL,
    -- 存储路径（NULL表示内联存储）
    storage_path TEXT,
    -- 存储类型 (0=内联, 1=外部文件)
    storage_type INTEGER DEFAULT 0,
    -- 内联数据（小文件<1KB）
    inline_data BLOB,
    -- 创建时间
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    -- 最后访问时间（LRU）
    last_access INTEGER DEFAULT (strftime('%s', 'now')),
    -- 访问计数
    access_count INTEGER DEFAULT 0,
    -- 标签（JSON数组）
    tags TEXT
);

-- 核心索引：SimHash 查询
CREATE INDEX IF NOT EXISTS idx_chunks_simhash_hi ON chunks(simhash_hi);

-- 唯一索引：防止重复存储（SimHash唯一）
CREATE UNIQUE INDEX IF NOT EXISTS idx_chunks_simhash_full ON chunks(simhash_hi, simhash_lo);

-- 索引：按大小查询（用于清理）
CREATE INDEX IF NOT EXISTS idx_chunks_size ON chunks(size);

-- 索引：LRU 淘汰
CREATE INDEX IF NOT EXISTS idx_chunks_lru ON chunks(last_access);

-- ============================================================
-- 辅助表：向量关联
-- ============================================================

CREATE TABLE IF NOT EXISTS chunk_vectors (
    chunk_id INTEGER NOT NULL,
    vector_id INTEGER NOT NULL,
    -- 相似度分数 (0-10000, 表示0-100%)
    similarity INTEGER,
    PRIMARY KEY (chunk_id, vector_id),
    FOREIGN KEY (chunk_id) REFERENCES chunks(id) ON DELETE CASCADE
);

-- 索引：向量反查
CREATE INDEX IF NOT EXISTS idx_vectors_vector_id ON chunk_vectors(vector_id);

-- ============================================================
-- 辅助表：同步状态
-- ============================================================

CREATE TABLE IF NOT EXISTS sync_peers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 设备唯一标识
    device_id TEXT UNIQUE NOT NULL,
    -- 设备名称
    device_name TEXT,
    -- 最后同步时间
    last_sync INTEGER,
    -- 同步次数
    sync_count INTEGER DEFAULT 0,
    -- 同步状态 (0=未同步, 1=同步中, 2=同步完成)
    status INTEGER DEFAULT 0
);

-- ============================================================
-- 触发器：自动更新时间
-- ============================================================

CREATE TRIGGER IF NOT EXISTS update_timestamp 
AFTER INSERT ON chunks
BEGIN
    UPDATE shard_meta SET value = CAST(strftime('%s', 'now') AS INTEGER) 
    WHERE key = 'updated_at';
    UPDATE shard_meta SET value = (SELECT COUNT(*) FROM chunks) 
    WHERE key = 'chunk_count';
END;

CREATE TRIGGER IF NOT EXISTS delete_timestamp 
AFTER DELETE ON chunks
BEGIN
    UPDATE shard_meta SET value = CAST(strftime('%s', 'now') AS INTEGER) 
    WHERE key = 'updated_at';
    UPDATE shard_meta SET value = (SELECT COUNT(*) FROM chunks) 
    WHERE key = 'chunk_count';
END;

-- ============================================================
-- 初始化完成标记
-- ============================================================

INSERT OR REPLACE INTO shard_meta (key, value) VALUES 
    ('schema_initialized', CAST(strftime('%s', 'now') AS INTEGER));
