-- Week 34知识图谱三表Schema（Nodes核心，Edges/Relations预留）
CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY, label TEXT NOT NULL,
    entity_type TEXT CHECK(entity_type IN ('ADR', 'Function', 'Module', 'Concept')),
    properties BLOB, embedding BLOB,
    created_at INTEGER, updated_at INTEGER
);
CREATE TABLE IF NOT EXISTS edges (
    from_id TEXT REFERENCES nodes(id), to_id TEXT REFERENCES nodes(id),
    rel_type TEXT CHECK(rel_type IN ('calls', 'imports', 'depends', 'fixes', 'references')),
    weight REAL DEFAULT 1.0, source TEXT, created_at INTEGER,
    PRIMARY KEY (from_id, to_id, rel_type)
);
CREATE TABLE IF NOT EXISTS relations (
    id TEXT PRIMARY KEY, subject TEXT, predicate TEXT, object TEXT,
    confidence REAL, extracted_from TEXT
);
PRAGMA journal_mode=WAL;
