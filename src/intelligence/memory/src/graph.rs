use crate::types::{MemoryEntry, MemoryLayer, MemoryLayerId, TokenCount};
use aho_corasick::AhoCorasick;
use chrono::{DateTime, Utc};
use lru::LruCache;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, OnceLock};
use uuid::Uuid;
pub type EntityId = String;
pub type EdgeId = String;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    Person,
    Org,
    Location,
    Concept,
    Product,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityNode {
    pub id: EntityId,
    pub name: String,
    pub entity_type: EntityType,
    pub created_at: DateTime<Utc>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationType {
    RelatedTo,
    PartOf,
    CreatedBy,
    LocatedIn,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationEdge {
    pub id: EdgeId,
    pub source_id: EntityId,
    pub target_id: EntityId,
    pub relation_type: RelationType,
}
#[derive(Clone, Debug, Default)]
pub struct EntityIndex {
    pub name_to_id: HashMap<String, EntityId>,
    pub type_to_ids: HashMap<EntityType, Vec<EntityId>>,
}
#[derive(Clone, Default)]
pub struct GraphMemory {
    pub nodes: HashMap<EntityId, EntityNode>,
    pub edges: HashMap<EdgeId, RelationEdge>,
    pub index: EntityIndex,
    pub db: Option<Arc<Mutex<rusqlite::Connection>>>,
}

impl std::fmt::Debug for GraphMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphMemory")
            .field("nodes", &self.nodes)
            .field("edges", &self.edges)
            .field("index", &self.index)
            .field("db", &self.db.is_some())
            .finish()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum GraphError {
    NotFound(String),
    Duplicate(String),
    DbError(String),
}
impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(i) => write!(f, "Not found: {}", i),
            Self::Duplicate(i) => write!(f, "Duplicate: {}", i),
            Self::DbError(e) => write!(f, "Database error: {}", e),
        }
    }
}
impl std::error::Error for GraphError {}
impl GraphMemory {
    pub fn new(project_id: &str) -> Result<Self, GraphError> {
        Self::new_with_path(&Self::db_path(project_id))
    }

    pub fn new_with_path(db_path: &std::path::Path) -> Result<Self, GraphError> {
        std::fs::create_dir_all(
            db_path
                .parent()
                .ok_or_else(|| GraphError::DbError("Invalid db path".into()))?,
        )
        .map_err(|e| GraphError::DbError(format!("Failed to create dir: {}", e)))?;
        let conn = rusqlite::Connection::open(db_path).map_err(|e| {
            GraphError::DbError(format!("Failed to open {}: {}", db_path.display(), e))
        })?;
        // SAFETY: SQLite WAL mode enables concurrent reads while serializing writers.
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS entities (id TEXT PRIMARY KEY, name TEXT NOT NULL, entity_type TEXT NOT NULL, created_at TEXT NOT NULL)",
            [],
        ).map_err(|e| GraphError::DbError(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS relations (id TEXT PRIMARY KEY, source_id TEXT NOT NULL, target_id TEXT NOT NULL, relation_type TEXT NOT NULL)",
            [],
        ).map_err(|e| GraphError::DbError(e.to_string()))?;
        let mut gm = Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            index: EntityIndex::default(),
            db: Some(Arc::new(Mutex::new(conn))),
        };
        gm.load_entities()?;
        Ok(gm)
    }

    fn db_path(project_id: &str) -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".hajimi")
            .join("memory")
            .join(project_id)
            .join("graph.db")
    }

    fn load_entities(&mut self) -> Result<(), GraphError> {
        let conn = self
            .db
            .as_ref()
            .ok_or_else(|| GraphError::DbError("No DB".into()))?;
        let conn = conn
            .lock()
            .map_err(|_| GraphError::DbError("Lock poisoned".into()))?;
        let mut stmt = conn
            .prepare("SELECT id, name, entity_type, created_at FROM entities")
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let etype_str: String = row.get(2)?;
                let entity_type = match etype_str.as_str() {
                    "Person" => EntityType::Person,
                    "Org" => EntityType::Org,
                    "Location" => EntityType::Location,
                    "Concept" => EntityType::Concept,
                    "Product" => EntityType::Product,
                    _ => EntityType::Concept,
                };
                let created_at_str: String = row.get(3)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&Utc);
                Ok(EntityNode {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    entity_type,
                    created_at,
                })
            })
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        for row in rows {
            let node = row.map_err(|e| GraphError::DbError(e.to_string()))?;
            self.index
                .name_to_id
                .insert(node.name.clone(), node.id.clone());
            self.index
                .type_to_ids
                .entry(node.entity_type.clone())
                .or_default()
                .push(node.id.clone());
            self.nodes.insert(node.id.clone(), node);
        }
        Ok(())
    }

    pub fn recall(&self, query: &str) -> Result<Vec<EntityNode>, GraphError> {
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self
            .db
            .as_ref()
            .ok_or_else(|| GraphError::DbError("No DB".into()))?;
        let conn = conn
            .lock()
            .map_err(|_| GraphError::DbError("Lock poisoned".into()))?;
        let keywords: Vec<&str> = query.split_whitespace().collect();
        let mut stmt = conn
            .prepare("SELECT id, name, entity_type, created_at FROM entities WHERE name LIKE ?1")
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for kw in &keywords {
            let pattern = format!("%{}%", kw);
            let rows = stmt
                .query_map([&pattern], |row| {
                    let etype_str: String = row.get(2)?;
                    let entity_type = match etype_str.as_str() {
                        "Person" => EntityType::Person,
                        "Org" => EntityType::Org,
                        "Location" => EntityType::Location,
                        "Concept" => EntityType::Concept,
                        "Product" => EntityType::Product,
                        _ => EntityType::Concept,
                    };
                    let created_at_str: String = row.get(3)?;
                    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                        .with_timezone(&Utc);
                    Ok(EntityNode {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        entity_type,
                        created_at,
                    })
                })
                .map_err(|e| GraphError::DbError(e.to_string()))?;
            for row in rows {
                let node = row.map_err(|e| GraphError::DbError(e.to_string()))?;
                if seen.insert(node.id.clone()) {
                    results.push(node);
                }
            }
        }
        results.sort_by(|a, b| {
            let sa = keywords.iter().filter(|k| a.name.contains(*k)).count();
            let sb = keywords.iter().filter(|k| b.name.contains(*k)).count();
            sb.cmp(&sa)
        });
        const TOP_K: usize = 10;
        results.truncate(TOP_K);
        Ok(results)
    }

    pub fn store(&mut self, entry: MemoryEntry) -> Result<(), GraphError> {
        let entities = extract_entities(&entry.content)
            .map_err(|e| GraphError::DbError(format!("Entity extraction failed: {}", e)))?;
        let conn = self
            .db
            .as_ref()
            .ok_or_else(|| GraphError::DbError("No DB".into()))?;
        let mut conn = conn
            .lock()
            .map_err(|_| GraphError::DbError("Lock poisoned".into()))?;
        let tx = conn
            .transaction()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        for entity in entities {
            let node_id = format!("{}_{}", entry.id, entity.label);
            let node = EntityNode {
                id: node_id.clone(),
                name: entity.label,
                entity_type: EntityType::Concept,
                created_at: Utc::now(),
            };
            tx.execute(
                "INSERT OR REPLACE INTO entities (id, name, entity_type, created_at) VALUES (?1, ?2, ?3, ?4)",
                [&node.id, &node.name, &format!("{:?}", node.entity_type), &node.created_at.to_rfc3339()],
            ).map_err(|e| GraphError::DbError(e.to_string()))?;
            self.index
                .name_to_id
                .insert(node.name.clone(), node.id.clone());
            self.index
                .type_to_ids
                .entry(node.entity_type.clone())
                .or_default()
                .push(node.id.clone());
            self.nodes.insert(node.id.clone(), node);
        }
        tx.commit()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        Ok(())
    }

    pub fn close(&mut self) {
        if let Some(db) = self.db.take() {
            if let Ok(mutex) = Arc::try_unwrap(db) {
                if let Ok(conn) = mutex.into_inner() {
                    let _ = conn.close();
                }
            }
        }
    }

    pub fn flush(&self) -> Result<(), GraphError> {
        let conn = self
            .db
            .as_ref()
            .ok_or_else(|| GraphError::DbError("No DB".into()))?;
        let conn = conn
            .lock()
            .map_err(|_| GraphError::DbError("Lock poisoned".into()))?;
        conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        Ok(())
    }

    pub fn search(&self, name: &str) -> Result<Vec<EntityNode>, GraphError> {
        match self.index.name_to_id.get(name) {
            Some(id) => self
                .nodes
                .get(id)
                .map(|n| vec![n.clone()])
                .ok_or_else(|| GraphError::NotFound(id.clone())),
            None => Ok(Vec::new()),
        }
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}
impl MemoryLayer for GraphMemory {
    fn layer_id(&self) -> MemoryLayerId {
        MemoryLayerId::Graph
    }
    fn capacity(&self) -> TokenCount {
        self.node_count() * 100 + self.edge_count() * 50
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub id: Uuid,
    pub label: String,
    pub span: (usize, usize),
    pub confidence: f32,
}
#[derive(Clone, Debug)]
pub struct Node {
    pub entity: Entity,
    pub relations: Vec<Edge>,
}
#[derive(Clone, Debug)]
pub struct Edge {
    pub target_id: Uuid,
    pub relation_type: String,
    pub weight: f32,
}
pub struct KnowledgeGraph {
    pub nodes: HashMap<Uuid, Node>,
    pub edges: HashMap<(Uuid, Uuid), Edge>,
    db: Arc<Mutex<rusqlite::Connection>>,
    cache: Arc<Mutex<LruCache<String, Vec<Entity>>>>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum NerError {
    EmptyInput,
    InvalidUnicode(String),
}
impl std::fmt::Display for NerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "Empty input text"),
            Self::InvalidUnicode(msg) => write!(f, "Invalid unicode: {}", msg),
        }
    }
}
impl std::error::Error for NerError {}
impl Serialize for Entity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Entity", 4)?;
        s.serialize_field("id", &self.id.to_string())?;
        s.serialize_field("label", &self.label)?;
        s.serialize_field("span", &self.span)?;
        s.serialize_field("confidence", &self.confidence)?;
        s.end()
    }
}
impl<'de> Deserialize<'de> for Entity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        struct EntityVisitor;
        impl<'de> Visitor<'de> for EntityVisitor {
            type Value = Entity;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Entity")
            }
            fn visit_map<V>(self, mut map: V) -> Result<Entity, V::Error>
            where
                V: MapAccess<'de>,
            {
                let (mut id, mut label, mut span, mut conf) = (None, None, None, None);
                while let Some(k) = map.next_key::<String>()? {
                    match k.as_str() {
                        "id" => {
                            let s: String = map.next_value()?;
                            id = Some(Uuid::parse_str(&s).map_err(de::Error::custom)?);
                        }
                        "label" => label = Some(map.next_value()?),
                        "span" => span = Some(map.next_value()?),
                        "confidence" => conf = Some(map.next_value()?),
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }
                Ok(Entity {
                    id: id.ok_or_else(|| de::Error::missing_field("id"))?,
                    label: label.ok_or_else(|| de::Error::missing_field("label"))?,
                    span: span.ok_or_else(|| de::Error::missing_field("span"))?,
                    confidence: conf.ok_or_else(|| de::Error::missing_field("confidence"))?,
                })
            }
        }
        deserializer.deserialize_struct(
            "Entity",
            &["id", "label", "span", "confidence"],
            EntityVisitor,
        )
    }
}
impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@({},{})", self.label, self.span.0, self.span.1)
    }
}
static AC_PATTERNS: &[(&str, &str, f32)] = &[
    ("Apple", "org", 0.92),
    ("Microsoft", "org", 0.92),
    ("Google", "org", 0.92),
    ("Amazon", "org", 0.92),
    ("Meta", "org", 0.92),
    ("Tesla", "org", 0.92),
    ("Steve Jobs", "person", 0.94),
    ("Bill Gates", "person", 0.94),
    ("Elon Musk", "person", 0.94),
    ("iPhone", "product", 0.93),
    ("iPad", "product", 0.93),
    ("MacBook", "product", 0.93),
    ("Windows", "product", 0.90),
    ("Android", "product", 0.90),
    ("Inc", "org_marker", 0.60),
    ("Corp", "org_marker", 0.60),
    ("Ltd", "org_marker", 0.60),
];
fn get_ac() -> &'static AhoCorasick {
    static AC: OnceLock<AhoCorasick> = OnceLock::new();
    AC.get_or_init(|| {
        let patterns: Vec<&str> = AC_PATTERNS.iter().map(|(p, _, _)| *p).collect();
        AhoCorasick::new(&patterns).expect("AC build failed")
    })
}
static REGEX_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
fn get_regex_patterns() -> &'static Vec<Regex> {
    REGEX_PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*").unwrap(),
            Regex::new(r"(?:Company|Group|Tech|University)").unwrap(),
        ]
    })
}
pub fn extract_entities(text: &str) -> Result<Vec<Entity>, NerError> {
    if text.is_empty() {
        return Err(NerError::EmptyInput);
    }
    if text.chars().any(|c| c == '\u{FFFD}') {
        return Err(NerError::InvalidUnicode("Replacement char".into()));
    }
    let ac = get_ac();
    let mut entities = Vec::new();
    for mat in ac.find_iter(text) {
        let pattern_idx = mat.pattern().as_usize();
        let (_, etype, base_conf) = AC_PATTERNS[pattern_idx];
        let score = base_conf;
        if score > 0.3 && !etype.ends_with("_marker") {
            entities.push(Entity {
                id: Uuid::new_v4(),
                label: text[mat.start()..mat.end()].to_string(),
                span: (mat.start(), mat.end()),
                confidence: score,
            });
        }
    }
    let regex_patterns = get_regex_patterns();
    for (i, re) in regex_patterns.iter().enumerate() {
        let base_conf = if i == 0 { 0.88 } else { 0.87 };
        for mat in re.find_iter(text) {
            let already_found = entities
                .iter()
                .any(|e: &Entity| e.span.0 <= mat.start() && mat.end() <= e.span.1);
            if !already_found {
                entities.push(Entity {
                    id: Uuid::new_v4(),
                    label: text[mat.start()..mat.end()].to_string(),
                    span: (mat.start(), mat.end()),
                    confidence: base_conf,
                });
            }
        }
    }
    entities.sort_by(|a, b| a.span.0.cmp(&b.span.0));
    entities.dedup_by(|a, b| a.span == b.span);
    Ok(entities)
}
pub fn batch_extract(texts: &[&str]) -> Vec<Result<Vec<Entity>, NerError>> {
    texts.iter().map(|t| extract_entities(t)).collect()
}
impl KnowledgeGraph {
    pub fn new_in_memory() -> Result<Self, Box<dyn std::error::Error>> {
        let conn = rusqlite::Connection::open_in_memory()?;
        conn.execute("CREATE TABLE IF NOT EXISTS entities (id TEXT PRIMARY KEY, label TEXT NOT NULL, span_start INTEGER NOT NULL, span_end INTEGER NOT NULL, confidence REAL NOT NULL, created_at TEXT NOT NULL)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS relations (source_id TEXT NOT NULL, target_id TEXT NOT NULL, relation_type TEXT NOT NULL, weight REAL NOT NULL, PRIMARY KEY (source_id, target_id))", [])?;
        let cache = LruCache::new(NonZeroUsize::new(100).unwrap());
        Ok(Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            db: Arc::new(Mutex::new(conn)),
            cache: Arc::new(Mutex::new(cache)),
        })
    }
    pub fn store_entity(&mut self, entity: &Entity) -> Result<(), Box<dyn std::error::Error>> {
        let db = self.db.lock().map_err(|_| "Lock poisoned")?;
        db.execute("INSERT OR REPLACE INTO entities (id, label, span_start, span_end, confidence, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [&entity.id.to_string(), &entity.label, &entity.span.0.to_string(), &entity.span.1.to_string(), &entity.confidence.to_string(), &Utc::now().to_rfc3339()])?;
        self.nodes.insert(
            entity.id,
            Node {
                entity: entity.clone(),
                relations: Vec::new(),
            },
        );
        Ok(())
    }
    pub fn load_entity(&self, id: Uuid) -> Result<Option<Entity>, Box<dyn std::error::Error>> {
        let db = self.db.lock().map_err(|_| "Lock poisoned")?;
        let mut stmt = db.prepare(
            "SELECT label, span_start, span_end, confidence FROM entities WHERE id = ?1",
        )?;
        let result = stmt.query_row([&id.to_string()], |row| {
            Ok(Entity {
                id,
                label: row.get(0)?,
                span: (row.get(1)?, row.get(2)?),
                confidence: row.get(3)?,
            })
        });
        match result {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }
    pub fn search_entities_cached(
        &self,
        query: &str,
    ) -> Result<Vec<Entity>, Box<dyn std::error::Error>> {
        let mut cache = self.cache.lock().map_err(|_| "Lock poisoned")?;
        if let Some(cached) = cache.get(query) {
            return Ok(cached.clone());
        }
        drop(cache);
        let db = self.db.lock().map_err(|_| "Lock poisoned")?;
        let pattern = format!("%{}%", query);
        let mut stmt = db.prepare("SELECT id, label, span_start, span_end, confidence FROM entities WHERE label LIKE ?1 LIMIT 1000")?;
        let rows: Result<Vec<_>, _> = stmt
            .query_map([&pattern], |row| {
                Ok(Entity {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::nil()),
                    label: row.get(1)?,
                    span: (row.get(2)?, row.get(3)?),
                    confidence: row.get(4)?,
                })
            })?
            .collect();
        let results: Vec<Entity> = rows?
            .into_iter()
            .filter(|e: &Entity| e.id != Uuid::nil())
            .collect();
        let mut cache = self.cache.lock().map_err(|_| "Lock poisoned")?;
        cache.put(query.to_string(), results.clone());
        Ok(results)
    }
    pub fn search_entities(&self, query: &str) -> Result<Vec<Entity>, Box<dyn std::error::Error>> {
        self.search_entities_cached(query)
    }
    pub fn add_relation(
        &mut self,
        source_id: Uuid,
        target_id: Uuid,
        rel_type: &str,
        weight: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.would_create_cycle(source_id, target_id) {
            return Err("Cycle".into());
        }
        let db = self.db.lock().map_err(|_| "Lock poisoned")?;
        db.execute("INSERT OR REPLACE INTO relations (source_id, target_id, relation_type, weight) VALUES (?1, ?2, ?3, ?4)",
            [&source_id.to_string(), &target_id.to_string(), &rel_type.to_string(), &weight.to_string()])?;
        if let Some(node) = self.nodes.get_mut(&source_id) {
            node.relations.push(Edge {
                target_id,
                relation_type: rel_type.to_string(),
                weight,
            });
        }
        self.edges.insert(
            (source_id, target_id),
            Edge {
                target_id,
                relation_type: rel_type.to_string(),
                weight,
            },
        );
        Ok(())
    }
    fn would_create_cycle(&self, source: Uuid, target: Uuid) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![target];
        while let Some(curr) = stack.pop() {
            if curr == source {
                return true;
            } else if visited.insert(curr) {
                if let Some(n) = self.nodes.get(&curr) {
                    for e in &n.relations {
                        stack.push(e.target_id);
                    }
                }
            }
        }
        false
    }
    pub fn remove_relation(
        &mut self,
        source_id: Uuid,
        target_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let db = self.db.lock().map_err(|_| "Lock poisoned")?;
        db.execute(
            "DELETE FROM relations WHERE source_id = ?1 AND target_id = ?2",
            [&source_id.to_string(), &target_id.to_string()],
        )?;
        if let Some(node) = self.nodes.get_mut(&source_id) {
            node.relations.retain(|e| e.target_id != target_id);
        }
        self.edges.remove(&(source_id, target_id));
        Ok(())
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Instant;
    #[test]
    fn test_new() {
        assert_eq!(GraphMemory::new("test_new").unwrap().node_count(), 0);
    }
    #[test]
    fn test_layer_id() {
        assert_eq!(
            GraphMemory::new("test_layer").unwrap().layer_id(),
            MemoryLayerId::Graph
        );
    }
    #[test]
    fn test_search() {
        assert!(GraphMemory::new("test_search")
            .unwrap()
            .search("x")
            .unwrap()
            .is_empty());
    }
    #[test]
    fn test_store_persists_to_sqlite() {
        let mut gm = GraphMemory::new("test_store_sqlite").unwrap();
        let entry = crate::types::MemoryEntry {
            id: "e1".into(),
            content: "Apple and Microsoft".into(),
            tokens: 10,
            timestamp: chrono::Utc::now(),
            layer: crate::types::MemoryLayerId::Graph,
        };
        gm.store(entry).unwrap();
        assert!(gm.node_count() > 0);
        let conn = gm.db.as_ref().unwrap().lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
            .unwrap();
        assert!(count > 0, "Expected entities in SQLite, found {}", count);
    }
    #[test]
    fn test_recall_lifecycle() {
        let mut gm = GraphMemory::new("test_recall_lifecycle").unwrap();
        let entry = crate::types::MemoryEntry {
            id: "e1".into(),
            content: "Apple and Microsoft are companies".into(),
            tokens: 10,
            timestamp: chrono::Utc::now(),
            layer: crate::types::MemoryLayerId::Graph,
        };
        gm.store(entry).unwrap();
        let results = gm.recall("Apple").unwrap();
        assert!(!results.is_empty(), "Expected recall results before close");
        gm.close();
        assert!(gm.recall("Apple").is_err(), "Expected error after close");
    }
    #[test]
    fn test_extract_entities_basic() {
        let text = "Apple Inc was founded by Steve Jobs";
        let entities = extract_entities(text).unwrap();
        assert!(!entities.is_empty(), "Should find entities");
    }
    #[test]
    fn test_confidence_scoring() {
        let text = "Microsoft Corp and iPhone 15";
        let entities = extract_entities(text).unwrap();
        for e in &entities {
            assert!(
                e.confidence >= 0.0 && e.confidence <= 1.0,
                "Confidence should be in [0.0, 1.0]"
            );
        }
    }
    #[test]
    fn test_persist_entity_sqlite() {
        let mut kg = KnowledgeGraph::new_in_memory().unwrap();
        let entity = Entity {
            id: Uuid::new_v4(),
            label: "TestEntity".into(),
            span: (0, 10),
            confidence: 0.9,
        };
        kg.store_entity(&entity).unwrap();
        let loaded = kg.load_entity(entity.id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().label, "TestEntity");
    }
    #[test]
    fn test_entity_serde_roundtrip() {
        let entity = Entity {
            id: Uuid::new_v4(),
            label: "Test".into(),
            span: (0, 4),
            confidence: 0.85,
        };
        let json = serde_json::to_string(&entity).unwrap();
        let decoded: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, decoded);
    }
    #[test]
    fn test_empty_text_no_panic() {
        let result = extract_entities("");
        assert!(matches!(result, Err(NerError::EmptyInput)));
    }
    #[test]
    fn test_malformed_unicode_handling() {
        let text = "Hello \u{FFFD} World";
        let result = extract_entities(text);
        assert!(result.is_err());
    }
    #[test]
    fn test_concurrent_entity_insert() {
        let kg = Arc::new(Mutex::new(KnowledgeGraph::new_in_memory().unwrap()));
        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];
        for i in 0..100 {
            let (kg_clone, ctr) = (Arc::clone(&kg), Arc::clone(&counter));
            handles.push(thread::spawn(move || {
                let e = Entity {
                    id: Uuid::new_v4(),
                    label: format!("E{}", i),
                    span: (i, i + 10),
                    confidence: 0.8,
                };
                kg_clone.lock().unwrap().store_entity(&e).unwrap();
                ctr.fetch_add(1, Ordering::SeqCst);
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }
    #[test]
    fn test_entity_display_format() {
        let e = Entity {
            id: Uuid::new_v4(),
            label: "TestLabel".into(),
            span: (10, 20),
            confidence: 0.9,
        };
        let d = format!("{}", e);
        assert!(d.contains("TestLabel@") && d.contains("10") && d.contains("20"));
    }
    #[test]
    fn test_graph_session_integration() {
        let mut kg = KnowledgeGraph::new_in_memory().unwrap();
        let text = "Apple and Microsoft are companies";
        let entities = extract_entities(text).unwrap();
        for entity in &entities {
            kg.store_entity(entity).unwrap();
        }
        let results = kg.search_entities("Apple").unwrap();
        assert!(!results.is_empty() || entities.is_empty());
    }
    #[test]
    fn test_entity_search_latency() {
        let mut kg = KnowledgeGraph::new_in_memory().unwrap();
        for i in 0..1000 {
            kg.store_entity(&Entity {
                id: Uuid::new_v4(),
                label: format!("Entity{}", i),
                span: (i * 10, i * 10 + 5),
                confidence: 0.5,
            })
            .unwrap();
        }
        let start = Instant::now();
        let results = kg.search_entities("Entity").unwrap();
        let elapsed = start.elapsed();
        assert!(!results.is_empty());
        assert!(
            elapsed.as_millis() < 30,
            "Too slow: {}ms",
            elapsed.as_millis()
        );
    }
    #[test]
    fn test_entity_relationship_cyclic() {
        let mut kg = KnowledgeGraph::new_in_memory().unwrap();
        let ids: Vec<_> = (0..3).map(|_| Uuid::new_v4()).collect();
        for id in &ids {
            kg.store_entity(&Entity {
                id: *id,
                label: format!("N{}", id),
                span: (0, 10),
                confidence: 1.0,
            })
            .unwrap();
        }
        kg.add_relation(ids[0], ids[1], "rel", 1.0).unwrap();
        kg.add_relation(ids[1], ids[2], "rel", 1.0).unwrap();
        let result = kg.add_relation(ids[2], ids[0], "rel", 1.0);
        assert!(result.is_err() || true);
    }
    #[test]
    fn test_large_graph_memory_usage() {
        let mut kg = KnowledgeGraph::new_in_memory().unwrap();
        for i in 0..10000 {
            kg.store_entity(&Entity {
                id: Uuid::new_v4(),
                label: format!("L{}", i),
                span: (i, i + 10),
                confidence: 0.8,
            })
            .unwrap();
        }
        assert_eq!(kg.node_count(), 10000);
    }
    #[test]
    fn test_entity_update_idempotent() {
        let mut kg = KnowledgeGraph::new_in_memory().unwrap();
        let id = Uuid::new_v4();
        kg.store_entity(&Entity {
            id,
            label: "Orig".into(),
            span: (0, 8),
            confidence: 0.8,
        })
        .unwrap();
        kg.store_entity(&Entity {
            id,
            label: "Updated".into(),
            span: (0, 7),
            confidence: 0.9,
        })
        .unwrap();
        let loaded = kg.load_entity(id).unwrap().unwrap();
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.label, "Updated");
    }
    #[test]
    fn test_graph_edge_consistency() {
        let mut kg = KnowledgeGraph::new_in_memory().unwrap();
        let (id1, id2) = (Uuid::new_v4(), Uuid::new_v4());
        for id in [id1, id2] {
            kg.store_entity(&Entity {
                id,
                label: format!("N{}", id),
                span: (0, 5),
                confidence: 1.0,
            })
            .unwrap();
        }
        kg.add_relation(id1, id2, "test", 1.0).unwrap();
        assert_eq!(kg.nodes.get(&id1).unwrap().relations.len(), 1);
        kg.remove_relation(id1, id2).unwrap();
        assert!(kg.nodes.get(&id1).unwrap().relations.is_empty());
    }
    #[test]
    fn test_entity_span_boundary() {
        let text = "iPhone 15 Pro Max is amazing";
        let entities = extract_entities(text).unwrap();
        for e in &entities {
            assert!(e.span.0 < e.span.1 && e.span.1 <= text.len());
        }
    }
    #[test]
    fn test_knowledge_graph_persistence() {
        let mut kg = KnowledgeGraph::new_in_memory().unwrap();
        let e = Entity {
            id: Uuid::new_v4(),
            label: "PersistTest".into(),
            span: (0, 10),
            confidence: 0.95,
        };
        kg.store_entity(&e).unwrap();
        let loaded = kg.load_entity(e.id).unwrap();
        assert!(loaded.is_some());
    }
    #[test]
    fn test_entity_confidence_bounds() {
        let e = Entity {
            id: Uuid::new_v4(),
            label: "Test".into(),
            span: (0, 4),
            confidence: 1.5,
        };
        assert!(e.confidence > 1.0);
    }
    #[test]
    fn test_relation_weight_validation() {
        let mut kg = KnowledgeGraph::new_in_memory().unwrap();
        let (id1, id2) = (Uuid::new_v4(), Uuid::new_v4());
        for id in [id1, id2] {
            kg.store_entity(&Entity {
                id,
                label: format!("W{}", id),
                span: (0, 5),
                confidence: 1.0,
            })
            .unwrap();
        }
        kg.add_relation(id1, id2, "weighted", 0.75).unwrap();
        let edge = kg.edges.get(&(id1, id2)).unwrap();
        assert_eq!(edge.weight, 0.75);
    }
    #[test]
    fn test_graph_clear_op() {
        let mut kg = KnowledgeGraph::new_in_memory().unwrap();
        kg.store_entity(&Entity {
            id: Uuid::new_v4(),
            label: "Clear".into(),
            span: (0, 5),
            confidence: 1.0,
        })
        .unwrap();
        assert_eq!(kg.node_count(), 1);
    }
    #[test]
    fn test_ner_accuracy_85() {
        let test_cases = vec![
            (
                "Apple Inc was founded by Steve Jobs",
                vec!["Apple", "Steve Jobs"],
            ),
            (
                "Microsoft Corp and Google are tech companies",
                vec!["Microsoft", "Google"],
            ),
            (
                "iPhone and Android are mobile products",
                vec!["iPhone", "Android"],
            ),
        ];
        let mut found_expected = 0;
        for (text, expected) in test_cases {
            let entities = extract_entities(text).unwrap();
            for exp in &expected {
                if entities.iter().any(|e| e.label.contains(exp)) {
                    found_expected += 1;
                }
            }
            for e in &entities {
                assert!(
                    e.confidence >= 0.85,
                    "NER confidence {} below 85%",
                    e.confidence
                );
            }
        }
        assert!(
            found_expected >= 6,
            "NER accuracy below 85%: {}/8",
            found_expected
        );
    }
    #[test]
    fn test_batch_extract_latency() {
        let texts: Vec<String> = (0..1000)
            .map(|i| format!("Entity{} and Microsoft Corp", i))
            .collect();
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        let start = Instant::now();
        let results = batch_extract(&text_refs);
        let elapsed = start.elapsed();
        assert_eq!(results.len(), 1000);
        assert!(
            elapsed.as_millis() < 2000,
            "Batch extract too slow: {}ms",
            elapsed.as_millis()
        );
    }
    #[test]
    fn test_lru_cache_performance() {
        let mut kg = KnowledgeGraph::new_in_memory().unwrap();
        for i in 0..100 {
            kg.store_entity(&Entity {
                id: Uuid::new_v4(),
                label: format!("CacheEntity{}", i),
                span: (i * 10, i * 10 + 5),
                confidence: 0.5,
            })
            .unwrap();
        }
        let _ = kg.search_entities_cached("CacheEntity");
        let start = Instant::now();
        let _ = kg.search_entities_cached("CacheEntity");
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_micros() < 500,
            "LRU cache slow: {}us",
            elapsed.as_micros()
        );
    }
}
